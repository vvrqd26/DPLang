// 表达式求值逻辑

use super::Executor;
use crate::parser::{Expr, BinaryOp, UnaryOp, FunctionDef};
use crate::runtime::{Value, RuntimeError};
use crate::lexer::{FStringPart, Lexer};
use std::collections::HashMap;

impl Executor {
    /// 执行表达式
    pub(crate) fn execute_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            
            Expr::FString(parts) => {
                // 处理 f-string 字符串插值
                let mut result = String::new();
                for part in parts {
                    match part {
                        FStringPart::Text(text) => result.push_str(text),
                        FStringPart::Expr(expr_str) => {
                            // 解析表达式
                            let mut lexer = Lexer::new(expr_str);
                            let tokens = lexer.tokenize()
                                .map_err(|e| RuntimeError::type_error(&format!("f-string 表达式解析错误: {}", e)))?;
                            let mut parser = crate::parser::Parser::new(tokens);
                            let script = parser.parse()
                                .map_err(|e| RuntimeError::type_error(&format!("f-string 表达式解析错误: {}", e)))?;
                            
                            // 执行表达式（假设是单个表达式语句）
                            if let crate::parser::Script::DataScript { body, .. } = script {
                                if let Some(crate::parser::Stmt::Expression(expr)) = body.first() {
                                    let value = self.execute_expr(expr)?;
                                    // 使用自定义格式化，字符串不加引号
                                    let formatted = match value {
                                        Value::String(s) => s,
                                        Value::Number(n) => n.to_string(),
                                        Value::Bool(b) => b.to_string(),
                                        Value::Null => "null".to_string(),
                                        _ => value.to_string(),
                                    };
                                    result.push_str(&formatted);
                                } else {
                                    return Err(RuntimeError::type_error("f-string 中的表达式无效"));
                                }
                            } else {
                                return Err(RuntimeError::type_error("f-string 中的表达式无效"));
                            }
                        }
                    }
                }
                Ok(Value::String(result))
            }
            
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Null => Ok(Value::Null),
            
            Expr::Identifier(name) => {
                // 检查是否是内置变量
                if name.starts_with('_') {
                    // 尝试从 DataStreamExecutor 获取内置变量
                    if let Some(value) = self.get_builtin_variable(name) {
                        return Ok(value);
                    }
                }
                
                self.context.get(name)
                    .cloned()
                    .ok_or_else(|| RuntimeError::undefined_variable(name))
            }
            
            Expr::Array(elements) => {
                let mut arr = Vec::new();
                for elem in elements {
                    arr.push(self.execute_expr(elem)?);
                }
                Ok(Value::Array(arr))
            }
            
            Expr::Binary { left, op, right } => {
                let left_val = self.execute_expr(left)?;
                let right_val = self.execute_expr(right)?;
                
                match op {
                    BinaryOp::Add => left_val.add(&right_val),
                    BinaryOp::Sub => left_val.sub(&right_val),
                    BinaryOp::Mul => left_val.mul(&right_val),
                    BinaryOp::Div => left_val.div(&right_val),
                    BinaryOp::Mod => left_val.modulo(&right_val),
                    BinaryOp::Pow => left_val.pow(&right_val),
                    BinaryOp::Gt => left_val.gt(&right_val),
                    BinaryOp::Lt => left_val.lt(&right_val),
                    BinaryOp::GtEq => left_val.gte(&right_val),
                    BinaryOp::LtEq => left_val.lte(&right_val),
                    BinaryOp::Eq => left_val.eq(&right_val),
                    BinaryOp::NotEq => left_val.neq(&right_val),
                    BinaryOp::And => left_val.and(&right_val),
                    BinaryOp::Or => left_val.or(&right_val),
                }
            }
            
            Expr::Unary { op, operand } => {
                let val = self.execute_expr(operand)?;
                match op {
                    UnaryOp::Neg => val.neg(),
                    UnaryOp::Not => val.not(),
                }
            }
            
            Expr::Ternary { condition, then_expr, else_expr } => {
                let cond = self.execute_expr(condition)?;
                if cond.to_bool() {
                    self.execute_expr(then_expr)
                } else {
                    self.execute_expr(else_expr)
                }
            }
            
            Expr::When { branches, else_expr } => {
                // when 表达式：依次求值每个分支的条件
                for branch in branches {
                    let cond = self.execute_expr(&branch.condition)?;
                    if cond.to_bool() {
                        return self.execute_expr(&branch.result);
                    }
                }
                
                // 如果所有条件都不满足，返回 else 分支或 null
                if let Some(else_result) = else_expr {
                    self.execute_expr(else_result)
                } else {
                    Ok(Value::Null)
                }
            }
            
            Expr::Call { callee, args } => {
                self.execute_call(callee, args)
            }
            
            Expr::Index { base, index } => {
                // 支持时间序列索引 var[-1]
                let idx_val = self.execute_expr(index)?;
                
                // 获取索引值
                let idx = match idx_val {
                    Value::Number(n) => n as isize,
                    _ => return Err(RuntimeError::type_error("索引必须为数字")),
                };
                
                // 判断 base 是否为变量名（时间序列访问）
                if let Expr::Identifier(var_name) = base.as_ref() {
                    // 负数索引：时间序列访问
                    if idx < 0 {
                        let offset = (-idx) as usize;
                        
                        // 尝试从 DataStreamExecutor 获取历史数据
                        if let Some(history_val) = self.get_time_series_value(var_name, offset) {
                            return Ok(history_val);
                        }
                        
                        // 如果没有在 DataStreamExecutor 中，则尝试从普通变量获取
                        let base_val = self.execute_expr(base)?;
                        if let Value::Array(arr) = base_val {
                            let len = arr.len() as isize;
                            let actual_idx = (len + idx) as usize;
                            if actual_idx < arr.len() {
                                return Ok(arr[actual_idx].clone());
                            } else {
                                return Ok(Value::Null);
                            }
                        }
                        
                        return Ok(Value::Null);
                    } else if idx == 0 {
                        // 索引 0 返回当前值
                        return self.context.get(var_name)
                            .cloned()
                            .ok_or_else(|| RuntimeError::undefined_variable(var_name));
                    }
                }
                
                // 普通数组索引
                let arr_val = self.execute_expr(base)?;
                if let Value::Array(arr) = arr_val {
                    if idx < 0 {
                        let len = arr.len() as isize;
                        let actual_idx = (len + idx) as usize;
                        if actual_idx < arr.len() {
                            Ok(arr[actual_idx].clone())
                        } else {
                            Ok(Value::Null)
                        }
                    } else {
                        let i = idx as usize;
                        if i < arr.len() {
                            Ok(arr[i].clone())
                        } else {
                            Ok(Value::Null)
                        }
                    }
                } else {
                    Err(RuntimeError::type_error("索引操作需要数组类型"))
                }
            }
            
            Expr::Slice { base, start, end } => {
                // 实现切片访问 var[-5:]
                
                // 解析 start 和 end 索引
                let start_idx = if let Some(s) = start {
                    let s_val = self.execute_expr(s)?;
                    match s_val {
                        Value::Number(n) => Some(n as isize),
                        _ => return Err(RuntimeError::type_error("切片索引必须为数字")),
                    }
                } else {
                    None
                };
                
                let end_idx = if let Some(e) = end {
                    let e_val = self.execute_expr(e)?;
                    match e_val {
                        Value::Number(n) => Some(n as isize),
                        _ => return Err(RuntimeError::type_error("切片索引必须为数字")),
                    }
                } else {
                    Some(0)  // 默认到当前值
                };
                
                // 判断 base 是否为变量名（时间序列访问）
                if let Expr::Identifier(var_name) = base.as_ref() {
                    // 如果 start或end是负数，则是时间序列切片
                    if start_idx.unwrap_or(0) < 0 || end_idx.unwrap_or(0) < 0 {
                        return self.get_time_series_slice(
                            var_name,
                            start_idx.unwrap_or(-1000000),  // 默认从很早开始
                            end_idx.unwrap_or(0),
                        );
                    }
                }
                
                // 普通数组切片
                let arr_val = self.execute_expr(base)?;
                if let Value::Array(arr) = arr_val {
                    let len = arr.len() as isize;
                    
                    let actual_start = match start_idx {
                        Some(s) if s < 0 => ((len + s).max(0)) as usize,
                        Some(s) => (s as usize).min(arr.len()),
                        None => 0,
                    };
                    
                    let actual_end = match end_idx {
                        Some(e) if e < 0 => ((len + e).max(0)) as usize,
                        Some(e) => (e as usize).min(arr.len()),
                        None => arr.len(),
                    };
                    
                    if actual_start <= actual_end {
                        Ok(Value::Array(arr[actual_start..actual_end].to_vec()))
                    } else {
                        Ok(Value::Array(vec![]))
                    }
                } else {
                    Err(RuntimeError::type_error("切片操作需要数组类型"))
                }
            }
            
            Expr::Spread(inner) => {
                // 展开在特定上下文中处理,这里直接返回数组
                self.execute_expr(inner)
            }
            
            Expr::Lambda { params, body } => {
                // 创建 Lambda 值，捕获当前环境中的变量
                let mut captures = HashMap::new();
                // 简化版：捕获所有当前变量
                for (name, value) in &self.context.variables {
                    captures.insert(name.clone(), Box::new(value.clone()));
                }
                
                Ok(Value::Lambda {
                    params: params.clone(),
                    body: body.clone(),
                    captures,
                })
            }
            
            Expr::Pipeline { value, stages } => {
                let mut result = self.execute_expr(value)?;
                for stage in stages {
                    // 管道: value |> func(arg) => func(value, arg)
                    if let Expr::Call { callee, args } = stage {
                        let mut new_args = vec![result];
                        for arg in args {
                            new_args.push(self.execute_expr(arg)?);
                        }
                        result = self.execute_builtin(callee, &new_args)?;
                    } else {
                        return Err(RuntimeError::type_error("管道右侧必须是函数调用"));
                    }
                }
                Ok(result)
            }
            
            Expr::MemberAccess { object, member } => {
                // 包.成员 访问
                // object 是包名，member 是变量/函数名
                let full_name = format!("{}.{}", object, member);
                if let Some(value) = self.package_vars.get(&full_name) {
                    Ok(value.clone())
                } else {
                    Err(RuntimeError::undefined_variable(&full_name))
                }
            }
        }
    }
    
    /// 执行函数调用
    pub(crate) fn execute_call(&mut self, callee: &str, args: &[Expr]) -> Result<Value, RuntimeError> {
        // 先尝试内置函数
        let arg_values: Result<Vec<Value>, _> = args.iter().map(|a| self.execute_expr(a)).collect();
        let arg_values = arg_values?;
        
        // 检查是否是 Lambda 函数
        if let Some(lambda_val) = self.context.get(callee) {
            if let Value::Lambda { params, body, captures } = lambda_val {
                return self.execute_lambda(params.clone(), body.clone(), captures.clone(), &arg_values);
            }
        }
        
        // 检查是否是包函数 (Function 类型)
        if let Some(func_val) = self.package_vars.get(callee) {
            if let Value::Function(func_def) = func_val {
                let func_def_clone = (**func_def).clone();
                return self.execute_user_function(&func_def_clone, &arg_values);
            }
        }
        
        // 检查是否是包函数 (functions map)
        if let Some(func_def) = self.functions.get(callee).cloned() {
            return self.execute_user_function(&func_def, &arg_values);
        }
        
        self.execute_builtin(callee, &arg_values)
    }
    
    /// 执行 Lambda 函数
    pub(crate) fn execute_lambda(
        &mut self,
        params: Vec<String>,
        body: Box<Expr>,
        captures: HashMap<String, Box<Value>>,
        args: &[Value],
    ) -> Result<Value, RuntimeError> {
        if params.len() != args.len() {
            return Err(RuntimeError::type_error(&format!(
                "Lambda 参数数量不匹配: 期望 {} 个，实际 {} 个",
                params.len(),
                args.len()
            )));
        }
        
        // 保存当前上下文
        let saved_vars = self.context.variables.clone();
        
        // 恢复捕获的环境
        for (name, value) in captures {
            self.context.set(name, (*value).clone());
        }
        
        // 绑定参数
        for (param, arg) in params.iter().zip(args.iter()) {
            self.context.set(param.clone(), arg.clone());
        }
        
        // 执行 Lambda 体
        let result = self.execute_expr(&body);
        
        // 恢复上下文
        self.context.variables = saved_vars;
        
        result
    }
    
    /// 执行用户定义函数
    pub(crate) fn execute_user_function(
        &mut self,
        func_def: &FunctionDef,
        args: &[Value],
    ) -> Result<Value, RuntimeError> {
        // 计算必需参数和总参数数量
        let required_params = func_def.params.iter()
            .take_while(|p| p.default_value.is_none())
            .count();
        let total_params = func_def.params.len();
        
        // 检查参数数量
        if args.len() < required_params || args.len() > total_params {
            return Err(RuntimeError::type_error(&format!(
                "函数 {} 参数数量不匹配: 期望 {}-{} 个，实际 {} 个",
                func_def.name,
                required_params,
                total_params,
                args.len()
            )));
        }
        
        // 保存当前上下文
        let saved_vars = self.context.variables.clone();
        
        // 绑定参数
        for (i, param) in func_def.params.iter().enumerate() {
            let arg_value = if i < args.len() {
                // 使用传入的参数值
                args[i].clone()
            } else {
                // 使用默认值
                if let Some(default_expr) = &param.default_value {
                    self.execute_expr(default_expr)?
                } else {
                    // 这里不应该发生，因为上面已经检查过参数数量
                    return Err(RuntimeError::type_error(&format!(
                        "缺少参数 {}",
                        param.name
                    )));
                }
            };
            
            self.context.set(param.name.clone(), arg_value);
        }
        
        // 执行函数体
        let mut result = Value::Null;
        for stmt in &func_def.body {
            if let Some(ret_val) = self.execute_stmt(stmt)? {
                result = ret_val;
                break;
            }
        }
        
        // 恢复上下文
        self.context.variables = saved_vars;
        
        Ok(result)
    }
}
