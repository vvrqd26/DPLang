// 语句执行逻辑

use super::Executor;
use crate::parser::Stmt;
use crate::runtime::{Value, RuntimeError};

impl Executor {
    /// 执行语句
    pub(crate) fn execute_stmt(&mut self, stmt: &Stmt) -> Result<Option<Value>, RuntimeError> {
        match stmt {
            Stmt::Assignment { name, value, .. } => {
                let val = self.execute_expr(value)?;
                self.context.set(name.clone(), val);
                Ok(None)
            }
            Stmt::Return(expr) => {
                let val = self.execute_expr(expr)?;
                Ok(Some(val))
            }
            Stmt::If { condition, then_block, else_block } => {
                let cond = self.execute_expr(condition)?;
                if cond.to_bool() {
                    for stmt in then_block {
                        if let Some(result) = self.execute_stmt(stmt)? {
                            return Ok(Some(result));
                        }
                    }
                } else if let Some(else_stmts) = else_block {
                    for stmt in else_stmts {
                        if let Some(result) = self.execute_stmt(stmt)? {
                            return Ok(Some(result));
                        }
                    }
                }
                Ok(None)
            }
            Stmt::Expression(expr) => {
                self.execute_expr(expr)?;
                Ok(None)
            }
            Stmt::Destructure { pattern, value } => {
                let val = self.execute_expr(value)?;
                if let Value::Array(arr) = val {
                    for (i, p) in pattern.iter().enumerate() {
                        match p {
                            crate::parser::DestructurePattern::Identifier(name) => {
                                if i < arr.len() {
                                    self.context.set(name.clone(), arr[i].clone());
                                }
                            }
                            crate::parser::DestructurePattern::Ignore => {
                                // 忽略
                            }
                            crate::parser::DestructurePattern::Spread(name) => {
                                // 收集剩余元素
                                let rest: Vec<Value> = arr.iter().skip(i).cloned().collect();
                                self.context.set(name.clone(), Value::Array(rest));
                                break;
                            }
                        }
                    }
                }
                Ok(None)
            }
        }
    }
}
