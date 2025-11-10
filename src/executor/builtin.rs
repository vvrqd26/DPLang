// 内置函数实现 - 核心最小集

use super::Executor;
use crate::runtime::{Value, RuntimeError};

impl Executor {
    /// 执行内置函数
    pub(crate) fn execute_builtin(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        match name {
            // 基础数据操作
            "sum" => self.builtin_sum(args),
            "max" => self.builtin_max(args),
            "min" => self.builtin_min(args),
            "length" => self.builtin_length(args),
            "concat" => self.builtin_concat(args),
            
            // 高阶函数
            "map" => self.builtin_map(args),
            "filter" => self.builtin_filter(args),
            "reduce" => self.builtin_reduce(args),
            
            // 工具函数
            "print" => self.builtin_print(args),
            
            // Null 处理函数
            "is_null" => self.builtin_is_null(args),
            
            _ => Err(RuntimeError::undefined_function(name)),
        }
    }
    
    /// sum 函数 - 求和
    fn builtin_sum(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        // sum([1,2,3]) => 6 或 sum(1,2,3) => 6
        if args.is_empty() {
            return Ok(Value::Number(0.0));
        }
        
        if let Value::Array(arr) = &args[0] {
            let mut total = 0.0;
            for v in arr {
                // 跳过 null 值
                if !matches!(v, Value::Null) {
                    total += v.to_number()?;
                }
            }
            Ok(Value::Number(total))
        } else {
            // 可变参数求和
            let mut total = 0.0;
            for v in args {
                if !matches!(v, Value::Null) {
                    total += v.to_number()?;
                }
            }
            Ok(Value::Number(total))
        }
    }
    
    /// max 函数 - 最大值
    fn builtin_max(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.is_empty() {
            return Err(RuntimeError::type_error("max 需要至少一个参数"));
        }
        
        let values: Vec<f64> = if let Value::Array(arr) = &args[0] {
            arr.iter()
                .filter(|v| !matches!(v, Value::Null))
                .map(|v| v.to_number())
                .collect::<Result<Vec<_>, _>>()?
        } else {
            args.iter()
                .filter(|v| !matches!(v, Value::Null))
                .map(|v| v.to_number())
                .collect::<Result<Vec<_>, _>>()?
        };
        
        if values.is_empty() {
            return Ok(Value::Null);
        }
        
        let max_val = values.into_iter().fold(f64::NEG_INFINITY, f64::max);
        Ok(Value::Number(max_val))
    }
    
    /// min 函数 - 最小值
    fn builtin_min(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.is_empty() {
            return Err(RuntimeError::type_error("min 需要至少一个参数"));
        }
        
        let values: Vec<f64> = if let Value::Array(arr) = &args[0] {
            arr.iter()
                .filter(|v| !matches!(v, Value::Null))
                .map(|v| v.to_number())
                .collect::<Result<Vec<_>, _>>()?
        } else {
            args.iter()
                .filter(|v| !matches!(v, Value::Null))
                .map(|v| v.to_number())
                .collect::<Result<Vec<_>, _>>()?
        };
        
        if values.is_empty() {
            return Ok(Value::Null);
        }
        
        let min_val = values.into_iter().fold(f64::INFINITY, f64::min);
        Ok(Value::Number(min_val))
    }
    
    /// length 函数 - 数组长度
    fn builtin_length(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::type_error("length 需要 1 个参数"));
        }
        
        match &args[0] {
            Value::Array(arr) => Ok(Value::Number(arr.len() as f64)),
            Value::String(s) => Ok(Value::Number(s.len() as f64)),
            _ => Err(RuntimeError::type_error("length 参数必须是数组或字符串")),
        }
    }
    
    /// concat 函数 - 数组拼接
    fn builtin_concat(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.is_empty() {
            return Ok(Value::Array(vec![]));
        }
        
        let mut result = Vec::new();
        
        for arg in args {
            match arg {
                Value::Array(arr) => result.extend(arr.clone()),
                other => result.push(other.clone()),
            }
        }
        
        Ok(Value::Array(result))
    }
    
    /// map 函数 - 数组映射
    fn builtin_map(&mut self, args: &[Value]) -> Result<Value, RuntimeError> {
        // map([1,2,3], x -> x * 2)
        if args.len() != 2 {
            return Err(RuntimeError::type_error("map 需要 2 个参数"));
        }
        
        let arr = match &args[0] {
            Value::Array(a) => a,
            _ => return Err(RuntimeError::type_error("map 的第一个参数必须是数组")),
        };
        
        let lambda = match &args[1] {
            Value::Lambda { params, body, captures } => (params, body, captures),
            _ => return Err(RuntimeError::type_error("map 的第二个参数必须是 Lambda")),
        };
        
        let mut result = Vec::new();
        for item in arr {
            let mapped = self.execute_lambda(
                lambda.0.clone(),
                lambda.1.clone(),
                lambda.2.clone(),
                &[item.clone()],
            )?;
            result.push(mapped);
        }
        
        Ok(Value::Array(result))
    }
    
    /// filter 函数 - 数组过滤
    fn builtin_filter(&mut self, args: &[Value]) -> Result<Value, RuntimeError> {
        // filter([1,2,3,4], x -> x > 2)
        if args.len() != 2 {
            return Err(RuntimeError::type_error("filter 需要 2 个参数"));
        }
        
        let arr = match &args[0] {
            Value::Array(a) => a,
            _ => return Err(RuntimeError::type_error("filter 的第一个参数必须是数组")),
        };
        
        let lambda = match &args[1] {
            Value::Lambda { params, body, captures } => (params, body, captures),
            _ => return Err(RuntimeError::type_error("filter 的第二个参数必须是 Lambda")),
        };
        
        let mut result = Vec::new();
        for item in arr {
            let keep = self.execute_lambda(
                lambda.0.clone(),
                lambda.1.clone(),
                lambda.2.clone(),
                &[item.clone()],
            )?;
            if keep.to_bool() {
                result.push(item.clone());
            }
        }
        
        Ok(Value::Array(result))
    }
    
    /// reduce 函数 - 数组归约
    fn builtin_reduce(&mut self, args: &[Value]) -> Result<Value, RuntimeError> {
        // reduce([1,2,3,4], (acc, x) -> acc + x, 0)
        if args.len() < 2 || args.len() > 3 {
            return Err(RuntimeError::type_error("reduce 需要 2-3 个参数"));
        }
        
        let arr = match &args[0] {
            Value::Array(a) => a,
            _ => return Err(RuntimeError::type_error("reduce 的第一个参数必须是数组")),
        };
        
        let lambda = match &args[1] {
            Value::Lambda { params, body, captures } => (params, body, captures),
            _ => return Err(RuntimeError::type_error("reduce 的第二个参数必须是 Lambda")),
        };
        
        if lambda.0.len() != 2 {
            return Err(RuntimeError::type_error("reduce 的 Lambda 必须有 2 个参数"));
        }
        
        if arr.is_empty() {
            return if args.len() == 3 {
                Ok(args[2].clone())
            } else {
                Err(RuntimeError::type_error("reduce 的空数组需要初始值"))
            };
        }
        
        let mut accumulator = if args.len() == 3 {
            args[2].clone()
        } else {
            arr[0].clone()
        };
        
        let start_idx = if args.len() == 3 { 0 } else { 1 };
        
        for item in &arr[start_idx..] {
            accumulator = self.execute_lambda(
                lambda.0.clone(),
                lambda.1.clone(),
                lambda.2.clone(),
                &[accumulator, item.clone()],
            )?;
        }
        
        Ok(accumulator)
    }
    
    /// print 函数 - 输出调试
    fn builtin_print(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        let output = args
            .iter()
            .map(|v| format!("{:?}", v))
            .collect::<Vec<_>>()
            .join(" ");
        println!("{}", output);
        Ok(Value::Null)
    }
    
    /// is_null 函数 - 检查值是否为null
    fn builtin_is_null(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::type_error("is_null 需要 1 个参数"));
        }
        
        Ok(Value::Bool(matches!(args[0], Value::Null)))
    }
}
