// 流式执行器 - 支持增量 tick 推送

use super::{Executor, ExecutionContext};
use crate::parser::{Script, PrecisionSetting};
use crate::runtime::{Value, RuntimeError};
use crate::package_loader::PackageLoader;
use std::collections::{HashMap, VecDeque};

use super::data_stream::CURRENT_DATA_STREAM;

/// 实时流式执行器（支持增量 tick 推送）
pub struct StreamingExecutor {
    /// 脚本定义
    script: Script,
    
    /// 输入历史窗口（固定长度，用于快速计算）
    input_window: VecDeque<HashMap<String, Value>>,
    
    /// 输出历史窗口
    output_window: VecDeque<HashMap<String, Value>>,
    
    /// 窗口最大长度
    window_size: usize,
    
    /// 当前处理的 tick 索引（从 0 开始）
    current_index: usize,
    
    /// 精度设置
    precision: Option<PrecisionSetting>,
    
    /// 导入的包
    packages: HashMap<String, HashMap<String, Value>>,
}

impl StreamingExecutor {
    /// 创建流式执行器
    pub fn new(script: Script, window_size: usize) -> Self {
        let precision = if let Script::DataScript { precision, .. } = &script {
            precision.clone()
        } else {
            None
        };
        
        StreamingExecutor {
            script,
            input_window: VecDeque::with_capacity(window_size),
            output_window: VecDeque::with_capacity(window_size),
            window_size,
            current_index: 0,
            precision,
            packages: HashMap::new(),
        }
    }
    
    /// 创建流式执行器（带包加载）
    pub fn new_with_packages(
        script: Script,
        window_size: usize,
        package_scripts: HashMap<String, Script>,
    ) -> Result<Self, RuntimeError> {
        let mut executor = Self::new(script, window_size);
        
        if let Script::DataScript { imports, .. } = &executor.script {
            for package_name in imports {
                if let Some(package_script) = package_scripts.get(package_name) {
                    let package_data = executor.execute_package_once(package_script)?;
                    executor.packages.insert(package_name.clone(), package_data);
                } else {
                    return Err(RuntimeError::type_error(&format!("找不到包: {}", package_name)));
                }
            }
        }
        
        Ok(executor)
    }
    
    /// 创建流式执行器（使用包加载器）
    pub fn new_with_loader(
        script: Script,
        window_size: usize,
        loader: &mut PackageLoader,
    ) -> Result<Self, RuntimeError> {
        let mut executor = Self::new(script, window_size);
        
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
    
    /// 推送单个 tick 数据，执行计算，返回输出结果
    pub fn push_tick(&mut self, tick_data: HashMap<String, Value>) -> Result<Option<HashMap<String, Value>>, RuntimeError> {
        // 设置线程局部变量（用于 ref 函数访问）
        CURRENT_DATA_STREAM.with(|cell| {
            *cell.borrow_mut() = Some(self as *const _ as *const super::DataStreamExecutor);
        });
        
        // 执行单个 tick
        let result = self.execute_tick(&tick_data)?;
        
        // 更新窗口：添加到历史
        self.input_window.push_back(tick_data);
        if let Some(ref output) = result {
            self.output_window.push_back(output.clone());
        }
        
        // 窗口满时，淘汰最旧数据
        if self.input_window.len() > self.window_size {
            self.input_window.pop_front();
        }
        if self.output_window.len() > self.window_size {
            self.output_window.pop_front();
        }
        
        self.current_index += 1;
        
        // 清理线程局部变量
        CURRENT_DATA_STREAM.with(|cell| {
            *cell.borrow_mut() = None;
        });
        
        Ok(result)
    }
    
    /// 执行单个 tick 的计算
    fn execute_tick(&self, tick_data: &HashMap<String, Value>) -> Result<Option<HashMap<String, Value>>, RuntimeError> {
        if let Script::DataScript { input, body, output, .. } = &self.script {
            let mut context = ExecutionContext::new();
            
            // 从 tick 数据填充 INPUT 变量
            for param in input {
                let value = tick_data
                    .get(&param.name)
                    .cloned()
                    .unwrap_or(Value::Null);
                context.set(param.name.clone(), value);
            }
            
            // 创建执行器
            let mut executor = Executor {
                context,
                functions: HashMap::new(),
                package_vars: HashMap::new(),
                precision: self.precision.clone(),
            };
            
            // 注入包数据
            for (pkg_name, pkg_data) in &self.packages {
                for (member_name, value) in pkg_data {
                    let full_name = format!("{}.{}", pkg_name, member_name);
                    executor.package_vars.insert(full_name, value.clone());
                }
            }
            
            let result = executor.execute_body(body)?;
            
            // 收集输出
            if let Some(Value::Array(output_values)) = result {
                let mut output_row = HashMap::new();
                
                for (i, param) in output.iter().enumerate() {
                    if let Some(value) = output_values.get(i) {
                        output_row.insert(param.name.clone(), value.clone());
                    }
                }
                
                return Ok(Some(output_row));
            }
        }
        
        Ok(None)
    }
    
    /// 获取输入历史（用于 ref 函数）
    pub fn get_input_history(&self, name: &str, offset: usize) -> Option<Value> {
        let window_len = self.input_window.len();
        if offset == 0 || offset > window_len {
            return None;
        }
        
        let target_index = window_len - offset;
        self.input_window
            .get(target_index)
            .and_then(|row| row.get(name))
            .cloned()
    }
    
    /// 获取输出历史
    pub fn get_output_history(&self, name: &str, offset: usize) -> Option<Value> {
        if offset == 0 {
            return None;
        }
        
        let window_len = self.output_window.len();
        if offset > window_len {
            return None;
        }
        
        let target_index = window_len - offset;
        self.output_window
            .get(target_index)
            .and_then(|row| row.get(name))
            .cloned()
    }
    
    /// 执行包脚本一次
    fn execute_package_once(&self, package_script: &Script) -> Result<HashMap<String, Value>, RuntimeError> {
        if let Script::Package { variables, functions, .. } = package_script {
            let mut package_executor = Executor::new();
            package_executor.execute_package_script(package_script)?;
            
            let mut package_data = HashMap::new();
            
            for var_def in variables {
                if let Some(value) = package_executor.package_vars.get(&var_def.name) {
                    package_data.insert(var_def.name.clone(), value.clone());
                }
            }
            
            for func_def in functions {
                let func_value = Value::Function(Box::new(func_def.clone()));
                package_data.insert(func_def.name.clone(), func_value);
            }
            
            Ok(package_data)
        } else {
            Err(RuntimeError::type_error("不是包脚本"))
        }
    }
    
    /// 获取当前索引
    pub fn current_index(&self) -> usize {
        self.current_index
    }
    
    /// 获取窗口大小
    pub fn window_size(&self) -> usize {
        self.window_size
    }
}
