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
            
            // 数组构造函数
            "Range" => self.builtin_range(args),
            "Array" => self.builtin_array(args),
            
            // 数组工具函数
            "mean" => self.builtin_mean(args),
            "first" => self.builtin_first(args),
            "last" => self.builtin_last(args),
            "sort" => self.builtin_sort(args),
            "unique" => self.builtin_unique(args),
            "reverse" => self.builtin_reverse(args),
            
            // 安全函数
            "safe_div" => self.builtin_safe_div(args),
            "safe_get" => self.builtin_safe_get(args),
            "safe_number" => self.builtin_safe_number(args),
            
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
    
    // ==================== 数组构造函数 ====================
    
    /// Range 函数 - 生成数字序列
    /// Range(1, 10) => [1,2,3,4,5,6,7,8,9,10]
    /// Range(0, 10, 2) => [0,2,4,6,8,10]
    fn builtin_range(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() < 2 || args.len() > 3 {
            return Err(RuntimeError::type_error("Range 需要 2-3 个参数"));
        }
        
        let start = args[0].to_number()?;
        let end = args[1].to_number()?;
        let step = if args.len() == 3 {
            args[2].to_number()?
        } else {
            1.0
        };
        
        if step == 0.0 {
            return Err(RuntimeError::type_error("Range 的步长不能为 0"));
        }
        
        let mut result = Vec::new();
        let mut current = start;
        
        if step > 0.0 {
            while current <= end {
                result.push(Value::Number(current));
                current += step;
            }
        } else {
            while current >= end {
                result.push(Value::Number(current));
                current += step;
            }
        }
        
        Ok(Value::Array(result))
    }
    
    /// Array 函数 - 创建固定长度数组
    /// Array(10, 0) => [0,0,0,0,0,0,0,0,0,0]
    /// Array(10, i -> i * 2) => [0,2,4,6,8,10,12,14,16,18]
    fn builtin_array(&mut self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::type_error("Array 需要 2 个参数"));
        }
        
        let size = args[0].to_number()? as usize;
        
        match &args[1] {
            Value::Lambda { params, body, captures } => {
                // Array(10, i -> i * 2)
                if params.len() != 1 {
                    return Err(RuntimeError::type_error("Array 的 Lambda 必须有 1 个参数"));
                }
                
                let mut result = Vec::new();
                for i in 0..size {
                    let value = self.execute_lambda(
                        params.clone(),
                        body.clone(),
                        captures.clone(),
                        &[Value::Number(i as f64)],
                    )?;
                    result.push(value);
                }
                Ok(Value::Array(result))
            }
            default_value => {
                // Array(10, 0)
                Ok(Value::Array(vec![default_value.clone(); size]))
            }
        }
    }
    
    // ==================== 数组工具函数 ====================
    
    /// mean 函数 - 计算平均值
    fn builtin_mean(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.is_empty() {
            return Err(RuntimeError::type_error("mean 需要至少一个参数"));
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
        
        let sum: f64 = values.iter().sum();
        Ok(Value::Number(sum / values.len() as f64))
    }
    
    /// first 函数 - 获取数组第一个元素
    fn builtin_first(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::type_error("first 需要 1 个参数"));
        }
        
        match &args[0] {
            Value::Array(arr) => {
                if arr.is_empty() {
                    Ok(Value::Null)
                } else {
                    Ok(arr[0].clone())
                }
            }
            _ => Err(RuntimeError::type_error("first 的参数必须是数组")),
        }
    }
    
    /// last 函数 - 获取数组最后一个元素
    fn builtin_last(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::type_error("last 需要 1 个参数"));
        }
        
        match &args[0] {
            Value::Array(arr) => {
                if arr.is_empty() {
                    Ok(Value::Null)
                } else {
                    Ok(arr[arr.len() - 1].clone())
                }
            }
            _ => Err(RuntimeError::type_error("last 的参数必须是数组")),
        }
    }
    
    /// sort 函数 - 对数组排序
    fn builtin_sort(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::type_error("sort 需要 1 个参数"));
        }
        
        match &args[0] {
            Value::Array(arr) => {
                let mut sorted = arr.clone();
                sorted.sort_by(|a, b| {
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => {
                            x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                        }
                        (Value::String(x), Value::String(y)) => x.cmp(y),
                        (Value::Null, Value::Null) => std::cmp::Ordering::Equal,
                        (Value::Null, _) => std::cmp::Ordering::Less,
                        (_, Value::Null) => std::cmp::Ordering::Greater,
                        _ => std::cmp::Ordering::Equal,
                    }
                });
                Ok(Value::Array(sorted))
            }
            _ => Err(RuntimeError::type_error("sort 的参数必须是数组")),
        }
    }
    
    /// unique 函数 - 数组去重
    fn builtin_unique(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::type_error("unique 需要 1 个参数"));
        }
        
        match &args[0] {
            Value::Array(arr) => {
                let mut result = Vec::new();
                for item in arr {
                    if !result.iter().any(|x| self.values_equal(x, item)) {
                        result.push(item.clone());
                    }
                }
                Ok(Value::Array(result))
            }
            _ => Err(RuntimeError::type_error("unique 的参数必须是数组")),
        }
    }
    
    /// reverse 函数 - 反转数组
    fn builtin_reverse(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::type_error("reverse 需要 1 个参数"));
        }
        
        match &args[0] {
            Value::Array(arr) => {
                let mut reversed = arr.clone();
                reversed.reverse();
                Ok(Value::Array(reversed))
            }
            _ => Err(RuntimeError::type_error("reverse 的参数必须是数组")),
        }
    }
    
    // ==================== 安全函数 ====================
    
    /// safe_div 函数 - 安全除法（避免除零错误）
    /// safe_div(a, b, default=0.0)
    fn builtin_safe_div(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() < 2 || args.len() > 3 {
            return Err(RuntimeError::type_error("safe_div 需要 2-3 个参数"));
        }
        
        let a = args[0].to_number()?;
        let b = args[1].to_number()?;
        let default = if args.len() == 3 {
            args[2].to_number()?
        } else {
            0.0
        };
        
        if b == 0.0 {
            Ok(Value::Number(default))
        } else {
            Ok(Value::Number(a / b))
        }
    }
    
    /// safe_get 函数 - 安全数组访问（避免越界）
    /// safe_get(array, index, default=null)
    fn builtin_safe_get(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() < 2 || args.len() > 3 {
            return Err(RuntimeError::type_error("safe_get 需要 2-3 个参数"));
        }
        
        let arr = match &args[0] {
            Value::Array(a) => a,
            _ => return Err(RuntimeError::type_error("safe_get 的第一个参数必须是数组")),
        };
        
        let index = args[1].to_number()? as i64;
        let default = if args.len() == 3 {
            args[2].clone()
        } else {
            Value::Null
        };
        
        // 处理负数索引
        let actual_index = if index < 0 {
            let positive_index = arr.len() as i64 + index;
            if positive_index < 0 {
                return Ok(default);
            }
            positive_index as usize
        } else {
            index as usize
        };
        
        if actual_index >= arr.len() {
            Ok(default)
        } else {
            Ok(arr[actual_index].clone())
        }
    }
    
    /// safe_number 函数 - 安全类型转换
    /// safe_number(value, default=0)
    fn builtin_safe_number(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() < 1 || args.len() > 2 {
            return Err(RuntimeError::type_error("safe_number 需要 1-2 个参数"));
        }
        
        let default = if args.len() == 2 {
            args[1].to_number()?
        } else {
            0.0
        };
        
        match args[0].to_number() {
            Ok(n) => Ok(Value::Number(n)),
            Err(_) => Ok(Value::Number(default)),
        }
    }
    
    /// 辅助函数 - 判断两个值是否相等
    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => (x - y).abs() < f64::EPSILON,
            (Value::String(x), Value::String(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::Null, Value::Null) => true,
            (Value::Array(x), Value::Array(y)) => {
                x.len() == y.len() && x.iter().zip(y.iter()).all(|(a, b)| self.values_equal(a, b))
            }
            _ => false,
        }
    }
}
