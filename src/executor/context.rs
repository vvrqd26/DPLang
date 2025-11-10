// 执行上下文

use crate::runtime::Value;
use std::collections::HashMap;

/// 执行上下文
pub struct ExecutionContext {
    /// 变量存储
    pub(crate) variables: HashMap<String, Value>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        ExecutionContext {
            variables: HashMap::new(),
        }
    }
    
    pub fn set(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }
    
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }
}
