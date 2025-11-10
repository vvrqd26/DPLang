// 数据流执行器 - 用于处理时间序列数据的行级执行

use super::{Executor, ContextPool};
use crate::parser::{Script, PrecisionSetting};
use crate::runtime::{Value, RuntimeError};
use crate::package_loader::PackageLoader;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

// 线程局部变量：当前数据流执行器的引用（用于 ref 函数访问历史数据）
thread_local! {
    pub(crate) static CURRENT_DATA_STREAM: RefCell<Option<*const DataStreamExecutor>> = RefCell::new(None);
}

/// 数据流执行器
pub struct DataStreamExecutor {
    /// 脚本定义
    script: Script,
    
    /// 输入矩阵（只读引用，使用 Rc 共享，零拷贝）
    input_matrix: Rc<Vec<HashMap<String, Value>>>,
    
    /// 输出矩阵（累积结果）
    output_matrix: Vec<HashMap<String, Value>>,
    
    /// 当前处理的行索引
    current_index: usize,
    
    /// 精度设置
    precision: Option<PrecisionSetting>,
    
    /// 导入的包（包名 -> 包的变量和函数）
    packages: HashMap<String, HashMap<String, Value>>,
    
    /// 上下文对象池（复用ExecutionContext）
    context_pool: ContextPool,
}

impl DataStreamExecutor {
    /// 创建数据流执行器
    pub fn new(script: Script, input_matrix: Vec<HashMap<String, Value>>) -> Self {
        // 处理空输入：转换为 [[]]（一个空行）
        let normalized_input = if input_matrix.is_empty() {
            vec![HashMap::new()]  // 一个空行
        } else {
            input_matrix
        };
        
        // 提取精度设置
        let precision = if let Script::DataScript { precision, .. } = &script {
            precision.clone()
        } else {
            None
        };
        
        DataStreamExecutor {
            script,
            input_matrix: Rc::new(normalized_input),
            output_matrix: Vec::new(),
            current_index: 0,
            precision,
            packages: HashMap::new(),
            context_pool: ContextPool::with_default(),
        }
    }
    
    /// 创建数据流执行器（带包加载）
    pub fn new_with_packages(
        script: Script,
        input_matrix: Vec<HashMap<String, Value>>,
        package_scripts: HashMap<String, Script>,  // 包名 -> 包脚本
    ) -> Result<Self, RuntimeError> {
        let mut executor = Self::new(script, input_matrix);
        
        // 加载导入的包（包脚本只执行一次）
        if let Script::DataScript { imports, .. } = &executor.script {
            for package_name in imports {
                if let Some(package_script) = package_scripts.get(package_name) {
                    // 执行包脚本，获取包的变量和函数
                    let package_data = executor.execute_package_once(package_script)?;
                    executor.packages.insert(package_name.clone(), package_data);
                } else {
                    return Err(RuntimeError::type_error(&format!("找不到包: {}", package_name)));
                }
            }
        }
        
        Ok(executor)
    }
    
    /// 创建数据流执行器（使用包加载器从文件系统加载）
    pub fn new_with_loader(
        script: Script,
        input_matrix: Vec<HashMap<String, Value>>,
        loader: &mut PackageLoader,
    ) -> Result<Self, RuntimeError> {
        let mut executor = Self::new(script, input_matrix);
        
        // 从文件系统加载导入的包
        if let Script::DataScript { imports, .. } = &executor.script {
            if !imports.is_empty() {
                let package_scripts = loader.load_packages(imports)?;
                
                for (package_name, package_script) in package_scripts {
                    let package_data = executor.execute_package_once(&package_script)?;
                    executor.packages.insert(package_name, package_data);
                }
            }
        }
        
        Ok(executor)
    }
    
    /// 执行包脚本一次，返回包的所有变量和函数
    fn execute_package_once(&self, package_script: &Script) -> Result<HashMap<String, Value>, RuntimeError> {
        if let Script::Package { variables, functions, .. } = package_script {
            let mut package_executor = Executor::new();
            
            // 执行包脚本
            package_executor.execute_package_script(package_script)?;
            
            // 收集包的所有变量
            let mut package_data = HashMap::new();
            
            // 添加包变量
            for var_def in variables {
                if let Some(value) = package_executor.package_vars.get(&var_def.name) {
                    package_data.insert(var_def.name.clone(), value.clone());
                }
            }
            
            // 添加包函数（封装为 Function 类型）
            for func_def in functions {
                let func_value = Value::Function(Box::new(func_def.clone()));
                package_data.insert(func_def.name.clone(), func_value);
            }
            
            Ok(package_data)
        } else {
            Err(RuntimeError::type_error("不是包脚本"))
        }
    }
    
    /// 执行整个数据流
    pub fn execute_all(&mut self) -> Result<Vec<HashMap<String, Value>>, RuntimeError> {
        let row_count = self.input_matrix.len();
        
        for row_idx in 0..row_count {
            self.current_index = row_idx;
            self.execute_row()?;
        }
        
        Ok(self.output_matrix.clone())
    }
    
    /// 获取输入矩阵的历史值（通过 index 引用）
    pub fn get_input_history(&self, name: &str, offset: usize) -> Option<Value> {
        if offset > self.current_index {
            return None;  // 历史不足
        }
        
        let target_index = self.current_index - offset;
        self.input_matrix
            .get(target_index)
            .and_then(|row| row.get(name))
            .cloned()
    }
    
    /// 获取当前行索引
    pub fn get_current_index(&self) -> usize {
        self.current_index
    }
    
    /// 获取总行数
    pub fn get_total_rows(&self) -> usize {
        self.input_matrix.len()
    }
    
    /// 获取当前输入行
    pub fn get_current_row(&self) -> Option<&HashMap<String, Value>> {
        self.input_matrix.get(self.current_index)
    }
    
    /// 获取输入矩阵的切片（零拷贝）
    /// start_offset: 开始偏移量（相对于当前行）
    /// end_offset: 结束偏移量（相对于当前行，0表示当前行）
    /// 返回: [current_index - start_offset, ..., current_index - end_offset]
    pub fn get_input_slice(&self, name: &str, start_offset: usize, end_offset: usize) -> Result<Value, RuntimeError> {
        // 构建列数据
        let mut column_data = Vec::new();
        
        // 计算实际索引范围
        let start_idx = if start_offset > self.current_index {
            // 历史不足，填充 null
            for _ in 0..(start_offset - self.current_index) {
                column_data.push(Value::Null);
            }
            0
        } else {
            self.current_index - start_offset
        };
        
        let end_idx = if end_offset > self.current_index {
            0
        } else {
            self.current_index - end_offset
        };
        
        // 提取数据
        for i in start_idx..=end_idx {
            let value = self.input_matrix
                .get(i)
                .and_then(|row| row.get(name))
                .cloned()
                .unwrap_or(Value::Null);
            column_data.push(value);
        }
        
        // 返回普通数组（后期可优化为 ArraySlice）
        Ok(Value::Array(column_data))
    }
    
    /// 获取输出矩阵的历史值（通过 index 引用）
    pub fn get_output_history(&self, name: &str, offset: usize) -> Option<Value> {
        if offset == 0 || offset > self.current_index {
            return None;  // offset=0 是当前行，还未计算
        }
        
        let target_index = self.current_index - offset;
        self.output_matrix
            .get(target_index)
            .and_then(|row| row.get(name))
            .cloned()
    }
    
    /// 获取输出矩阵的切片
    pub fn get_output_slice(&self, name: &str, start_offset: usize, end_offset: usize) -> Result<Value, RuntimeError> {
        let mut column_data = Vec::new();
        
        let start_idx = if start_offset > self.current_index {
            for _ in 0..(start_offset - self.current_index) {
                column_data.push(Value::Null);
            }
            0
        } else {
            self.current_index - start_offset
        };
        
        let end_idx = if end_offset > self.current_index || end_offset == 0 {
            if end_offset == 0 {
                return Err(RuntimeError::type_error("无法获取当前行的输出值"));
            }
            0
        } else {
            self.current_index - end_offset
        };
        
        for i in start_idx..=end_idx {
            let value = self.output_matrix
                .get(i)
                .and_then(|row| row.get(name))
                .cloned()
                .unwrap_or(Value::Null);
            column_data.push(value);
        }
        
        Ok(Value::Array(column_data))
    }
    
    /// 执行单行
    fn execute_row(&mut self) -> Result<(), RuntimeError> {
        // 设置线程局部变量,让 ref() 函数能访问历史数据
        CURRENT_DATA_STREAM.with(|cell| {
            *cell.borrow_mut() = Some(self as *const DataStreamExecutor);
        });
        
        // 1. 设置当前行的 INPUT 变量（可能为空）
        let current_input = &self.input_matrix[self.current_index];
        let mut context = self.context_pool.acquire();

        // 从当前行填充 INPUT 变量
        // 如果输入为空行，所有 INPUT 变量默认为 null
        if let Script::DataScript { input, body, output, .. } = &self.script {
            for param in input {
                let value = current_input
                    .get(&param.name)
                    .cloned()
                    .unwrap_or(Value::Null);  // 空行时为 null
                
                context.set(param.name.clone(), value);
            }
            
            // 2. 创建临时执行器并执行
            let mut executor = Executor {
                context,
                functions: HashMap::new(),
                package_vars: HashMap::new(),
                precision: self.precision.clone(),
            };
            
            // 将包数据注入到 package_vars（扩展为平面结构）
            for (pkg_name, pkg_data) in &self.packages {
                for (member_name, value) in pkg_data {
                    // 使用 "pkg.member" 作为 key
                    let full_name = format!("{}.{}", pkg_name, member_name);
                    executor.package_vars.insert(full_name, value.clone());
                }
            }
            
            let result = executor.execute_body(body)?;
            
            // 归还上下文到对象池
            self.context_pool.release(executor.context);
            
            // 3. 收集输出
            if let Some(Value::Array(output_values)) = result {
                let mut output_row = HashMap::new();
                
                for (i, param) in output.iter().enumerate() {
                    if let Some(value) = output_values.get(i) {
                        output_row.insert(param.name.clone(), value.clone());
                    }
                }
                
                self.output_matrix.push(output_row);
            }
        }
        
        // 清理线程局部变量
        CURRENT_DATA_STREAM.with(|cell| {
            *cell.borrow_mut() = None;
        });
        
        Ok(())
    }
}
