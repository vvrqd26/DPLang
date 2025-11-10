// DPLang 执行器模块

mod context;
mod context_pool;
mod columnar_storage;
mod data_stream;
mod streaming;
mod output_manager;
mod expression;
mod statement;
mod builtin;

pub use context::ExecutionContext;
pub use context_pool::{ContextPool, PoolConfig};
pub use columnar_storage::ColumnarStorage;
pub use data_stream::DataStreamExecutor;
pub use streaming::StreamingExecutor;
pub use output_manager::{OutputManager, OutputManagerConfig, OutputMode, OutputRow};

use data_stream::CURRENT_DATA_STREAM;
use crate::parser::{Stmt, Script, FunctionDef, PrecisionSetting};
use crate::runtime::{Value, RuntimeError};
use std::collections::HashMap;

/// 执行器
pub struct Executor {
    pub(crate) context: ExecutionContext,
    /// 包级函数
    pub(crate) functions: HashMap<String, FunctionDef>,
    /// 包级变量
    pub(crate) package_vars: HashMap<String, Value>,
    /// 精度设置
    pub(crate) precision: Option<PrecisionSetting>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            context: ExecutionContext::new(),
            functions: HashMap::new(),
            package_vars: HashMap::new(),
            precision: None,
        }
    }
    
    /// 执行数据脚本
    pub fn execute_data_script(&mut self, script: &Script) -> Result<Option<Value>, RuntimeError> {
        if let Script::DataScript { body, error_block, precision, .. } = script {
            // 设置精度
            self.precision = precision.clone();
            
            // 尝试执行主体
            let result = self.execute_body(body);
            
            // 如果有错误且定义了 ERROR 块，执行 ERROR 块
            if let Err(ref error) = result {
                if let Some(error_stmts) = error_block {
                    // 设置错误信息变量
                    self.context.set("__error__".to_string(), Value::String(error.message.clone()));
                    
                    // 执行 ERROR 块
                    for stmt in error_stmts {
                        if let Some(result) = self.execute_stmt(stmt)? {
                            // 应用精度
                            let final_result = self.apply_precision_to_value(result)?;
                            return Ok(Some(final_result));
                        }
                    }
                    
                    // ERROR 块执行完毕，返回 null
                    return Ok(None);
                }
            }
            
            // 应用精度到返回值
            match result {
                Ok(Some(val)) => Ok(Some(self.apply_precision_to_value(val)?)),
                Ok(None) => Ok(None),
                Err(e) => Err(e),
            }
        } else {
            Err(RuntimeError::type_error("期望数据脚本"))
        }
    }
    
    /// 执行语句体
    pub(crate) fn execute_body(&mut self, body: &[Stmt]) -> Result<Option<Value>, RuntimeError> {
        for stmt in body {
            if let Some(result) = self.execute_stmt(stmt)? {
                return Ok(Some(result));
            }
        }
        Ok(None)
    }
    
    /// 执行包脚本
    pub fn execute_package_script(&mut self, script: &Script) -> Result<(), RuntimeError> {
        if let Script::Package { variables, functions, .. } = script {
            // 加载包级变量
            for var_def in variables {
                let value = self.execute_expr(&var_def.value)?;
                self.package_vars.insert(var_def.name.clone(), value.clone());
                // 也放入当前上下文以便其他变量可以引用
                self.context.set(var_def.name.clone(), value);
            }
            
            // 加载包级函数
            for func_def in functions {
                self.functions.insert(func_def.name.clone(), func_def.clone());
            }
            
            Ok(())
        } else {
            Err(RuntimeError::type_error("期望包脚本"))
        }
    }
    
    /// 设置输入变量 (用于数据脚本)
    pub fn set_input(&mut self, name: String, value: Value) {
        self.context.set(name, value);
    }
    
    /// 应用精度到值
    fn apply_precision_to_value(&self, value: Value) -> Result<Value, RuntimeError> {
        if let Some(ref precision) = self.precision {
            match value {
                Value::Array(arr) => {
                    // 对数组中的每个元素应用精度
                    let mut result = Vec::new();
                    for v in arr {
                        // 对 Decimal 类型应用精度
                        if matches!(v, Value::Decimal(_)) {
                            result.push(v.apply_precision(precision.scale)?);
                        } else {
                            result.push(v);
                        }
                    }
                    Ok(Value::Array(result))
                }
                Value::Decimal(_) => {
                    // 对 Decimal 应用精度
                    value.apply_precision(precision.scale)
                }
                _ => Ok(value), // 其他类型不变
            }
        } else {
            Ok(value)
        }
    }
    
    /// 从 DataStreamExecutor 获取时间序列单值
    pub(crate) fn get_time_series_value(&self, var_name: &str, offset: usize) -> Option<Value> {
        CURRENT_DATA_STREAM.with(|cell| {
            if let Some(executor_ptr) = *cell.borrow() {
                unsafe {
                    let executor = &*executor_ptr;
                    // 先尝试从输入获取
                    if let Some(val) = executor.get_input_history(var_name, offset) {
                        return Some(val);
                    }
                    // 再尝试从输出获取
                    executor.get_output_history(var_name, offset)
                }
            } else {
                None
            }
        })
    }
    
    /// 从 DataStreamExecutor 获取时间序列切片
    pub(crate) fn get_time_series_slice(
        &self,
        var_name: &str,
        start_idx: isize,  // 负数，如 -5
        end_idx: isize,    // 负数或0，如 -1 或 0
    ) -> Result<Value, RuntimeError> {
        CURRENT_DATA_STREAM.with(|cell| {
            if let Some(executor_ptr) = *cell.borrow() {
                unsafe {
                    let executor = &*executor_ptr;
                    
                    // 转换为偏移量
                    let start_offset = if start_idx < 0 { (-start_idx) as usize } else { 0 };
                    let end_offset = if end_idx < 0 { (-end_idx) as usize } else { 0 };
                    
                    // 先尝试从输入获取
                    if let Ok(val) = executor.get_input_slice(var_name, start_offset, end_offset) {
                        return Ok(val);
                    }
                    
                    // 再尝试从输出获取
                    executor.get_output_slice(var_name, start_offset, end_offset)
                }
            } else {
                Err(RuntimeError::type_error("时间序列访问只能在数据流脚本中使用"))
            }
        })
    }
    
    /// 获取内置变量 (_index, _total, _args, _args_names)
    pub(crate) fn get_builtin_variable(&self, name: &str) -> Option<Value> {
        CURRENT_DATA_STREAM.with(|cell| {
            if let Some(executor_ptr) = *cell.borrow() {
                unsafe {
                    let executor = &*executor_ptr;
                    match name {
                        "_index" => Some(Value::Number(executor.get_current_index() as f64)),
                        "_total" => Some(Value::Number(executor.get_total_rows() as f64)),
                        "_args" => {
                            // 返回当前输入行的所有值
                            if let Some(row) = executor.get_current_row() {
                                let values: Vec<Value> = row.values().cloned().collect();
                                Some(Value::Array(values))
                            } else {
                                None
                            }
                        }
                        "_args_names" => {
                            // 返回输入字段名数组
                            if let Some(row) = executor.get_current_row() {
                                let names: Vec<Value> = row.keys()
                                    .map(|k| Value::String(k.clone()))
                                    .collect();
                                Some(Value::Array(names))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                }
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests;
