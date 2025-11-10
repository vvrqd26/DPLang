// 内置函数实现

use super::Executor;
use super::data_stream::CURRENT_DATA_STREAM;
use crate::runtime::{Value, RuntimeError};
use crate::indicators;

impl Executor {
    /// 执行内置函数
    pub(crate) fn execute_builtin(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        match name {
            "MA" => {
                // 简化的均线函数:MA(array/number, period)
                if args.len() != 2 {
                    return Err(RuntimeError::type_error("MA 需要 2 个参数"));
                }
                
                // 这里简化处理,假设第一个参数是数字,返回数字本身
                // 实际应该处理时间序列数据
                Ok(args[0].clone())
            }
            
            "sum" => {
                // sum([1,2,3]) => 6
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
            
            "max" => {
                if args.is_empty() {
                    return Err(RuntimeError::type_error("max 需要至少一个参数"));
                }
                
                let values: Vec<f64> = if let Value::Array(arr) = &args[0] {
                    arr.iter().map(|v| v.to_number()).collect::<Result<Vec<_>, _>>()?
                } else {
                    args.iter().map(|v| v.to_number()).collect::<Result<Vec<_>, _>>()?
                };
                
                let max_val = values.into_iter().fold(f64::NEG_INFINITY, f64::max);
                Ok(Value::Number(max_val))
            }
            
            "min" => {
                if args.is_empty() {
                    return Err(RuntimeError::type_error("min 需要至少一个参数"));
                }
                
                let values: Vec<f64> = if let Value::Array(arr) = &args[0] {
                    arr.iter().map(|v| v.to_number()).collect::<Result<Vec<_>, _>>()?
                } else {
                    args.iter().map(|v| v.to_number()).collect::<Result<Vec<_>, _>>()?
                };
                
                let min_val = values.into_iter().fold(f64::INFINITY, f64::min);
                Ok(Value::Number(min_val))
            }
            
            // 高阶函数
            "map" => self.builtin_map(args),
            "filter" => self.builtin_filter(args),
            "reduce" => self.builtin_reduce(args),
            
            // 时间序列函数
            "ref" => self.builtin_ref(args),
            "past" => self.builtin_past(args),
            "offset" => self.builtin_offset(args),
            "window" => self.builtin_window(args),
            
            // 技术指标函数
            "SMA" => self.builtin_sma(args),
            "EMA" => self.builtin_ema(args),
            "MACD" => self.builtin_macd(args),
            "RSI" => self.builtin_rsi(args),
            "BOLL" => self.builtin_boll(args),
            "ATR" => self.builtin_atr(args),
            "KDJ" => self.builtin_kdj(args),
            
            // 工具函数
            "print" => self.builtin_print(args),
            
            _ => Err(RuntimeError::undefined_function(name)),
        }
    }
    
    /// map 函数
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
    
    /// filter 函数
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
    
    /// reduce 函数
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
    
    /// ref 函数 - 引用历史值
    fn builtin_ref(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        // ref("varname", offset) - 引用历史值
        // offset=1 表示上一行，offset=2 表示上上行
        if args.len() != 2 {
            return Err(RuntimeError::type_error("ref 需要 2 个参数"));
        }
        
        let var_name = match &args[0] {
            Value::String(s) => s,
            _ => return Err(RuntimeError::type_error("ref 的第一个参数必须是变量名（字符串）")),
        };
        
        let offset = args[1].to_number()? as usize;
        
        // 从线程局部变量获取当前 DataStreamExecutor
        CURRENT_DATA_STREAM.with(|cell| {
            let executor_ptr = cell.borrow();
            if let Some(ptr) = *executor_ptr {
                // 安全：我们确保 DataStreamExecutor 在整个执行期间有效
                let executor = unsafe { &*ptr };
                
                // 先尝试从输出矩阵获取（计算出来的值）
                if let Some(value) = executor.get_output_history(var_name, offset) {
                    return Ok(value);
                }
                
                // 如果输出矩阵没有，再从输入矩阵获取
                if let Some(value) = executor.get_input_history(var_name, offset) {
                    return Ok(value);
                }
                
                // 历史不足，返回 null
                Ok(Value::Null)
            } else {
                // 不在 DataStreamExecutor 上下文中，报错
                Err(RuntimeError::type_error("ref 函数只能在数据流执行器中使用"))
            }
        })
    }
    
    /// past 函数 - 获取过去 n 个周期的值数组
    fn builtin_past(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        // past("varname", n) - 返回过去 n 个周期的值数组 [t-n+1, t-n+2, ..., t]
        if args.len() != 2 {
            return Err(RuntimeError::type_error("past 需要 2 个参数"));
        }
        
        let var_name = match &args[0] {
            Value::String(s) => s,
            _ => return Err(RuntimeError::type_error("past 的第一个参数必须是变量名（字符串）")),
        };
        
        let n = args[1].to_number()? as usize;
        
        if n == 0 {
            return Ok(Value::Array(vec![]));
        }
        
        CURRENT_DATA_STREAM.with(|cell| {
            let executor_ptr = cell.borrow();
            if let Some(ptr) = *executor_ptr {
                let executor = unsafe { &*ptr };
                let mut result = Vec::new();
                
                // 收集过去 n 个周期的值（从旧到新）
                for i in (1..=n).rev() {
                    // 先尝试从输出矩阵获取
                    if let Some(value) = executor.get_output_history(var_name, i) {
                        result.push(value);
                    } else if let Some(value) = executor.get_input_history(var_name, i) {
                        result.push(value);
                    } else {
                        // 历史不足，填充 null
                        result.push(Value::Null);
                    }
                }
                
                Ok(Value::Array(result))
            } else {
                Err(RuntimeError::type_error("past 函数只能在数据流执行器中使用"))
            }
        })
    }
    
    /// offset 函数 - ref 的简化版本
    fn builtin_offset(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        // offset("varname", n) - 等同于 ref("varname", n)
        self.builtin_ref(args)
    }
    
    /// window 函数 - 滑动窗口，返回最近 size 个值的数组（包括当前值）
    fn builtin_window(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        // window("varname", size) - 返回 [t-size+1, t-size+2, ..., t-1, t]
        if args.len() != 2 {
            return Err(RuntimeError::type_error("window 需要 2 个参数"));
        }
        
        let var_name = match &args[0] {
            Value::String(s) => s,
            _ => return Err(RuntimeError::type_error("window 的第一个参数必须是变量名（字符串）")),
        };
        
        let size = args[1].to_number()? as usize;
        
        if size == 0 {
            return Ok(Value::Array(vec![]));
        }
        
        CURRENT_DATA_STREAM.with(|cell| {
            let executor_ptr = cell.borrow();
            if let Some(ptr) = *executor_ptr {
                let executor = unsafe { &*ptr };
                let mut result = Vec::new();
                
                // 收集窗口内的值（从 size-1 到 0，即从旧到新）
                for i in (1..size).rev() {
                    // 先尝试从输出矩阵获取
                    if let Some(value) = executor.get_output_history(var_name, i) {
                        result.push(value);
                    } else if let Some(value) = executor.get_input_history(var_name, i) {
                        result.push(value);
                    } else {
                        // 历史不足，填充 null
                        result.push(Value::Null);
                    }
                }
                
                // 添加当前值（offset=0 是当前行，还未计算，所以从 INPUT 获取）
                if let Some(value) = executor.get_input_history(var_name, 0) {
                    result.push(value);
                } else {
                    result.push(Value::Null);
                }
                
                Ok(Value::Array(result))
            } else {
                Err(RuntimeError::type_error("window 函数只能在数据流执行器中使用"))
            }
        })
    }
    
    /// SMA 函数 - 简单移动平均线
    fn builtin_sma(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        // SMA(prices, period)
        if args.len() != 2 {
            return Err(RuntimeError::type_error("SMA 需要 2 个参数"));
        }
        
        let prices = match &args[0] {
            Value::Array(arr) => arr,
            _ => return Err(RuntimeError::type_error("SMA 的第一个参数必须是数组")),
        };
        
        let period = args[1].to_number()? as usize;
        indicators::sma(prices, period)
    }
    
    /// EMA 函数 - 指数移动平均线
    fn builtin_ema(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        // EMA(prices, period)
        if args.len() != 2 {
            return Err(RuntimeError::type_error("EMA 需要 2 个参数"));
        }
        
        let prices = match &args[0] {
            Value::Array(arr) => arr,
            _ => return Err(RuntimeError::type_error("EMA 的第一个参数必须是数组")),
        };
        
        let period = args[1].to_number()? as usize;
        indicators::ema(prices, period)
    }
    
    /// MACD 函数
    fn builtin_macd(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        // MACD(prices, fast, slow, signal)
        if args.len() != 4 {
            return Err(RuntimeError::type_error("MACD 需要 4 个参数"));
        }
        
        let prices = match &args[0] {
            Value::Array(arr) => arr,
            _ => return Err(RuntimeError::type_error("MACD 的第一个参数必须是数组")),
        };
        
        let fast = args[1].to_number()? as usize;
        let slow = args[2].to_number()? as usize;
        let signal = args[3].to_number()? as usize;
        
        indicators::macd(prices, fast, slow, signal)
    }
    
    /// RSI 函数 - 相对强弱指标
    fn builtin_rsi(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        // RSI(prices, period)
        if args.len() != 2 {
            return Err(RuntimeError::type_error("RSI 需要 2 个参数"));
        }
        
        let prices = match &args[0] {
            Value::Array(arr) => arr,
            _ => return Err(RuntimeError::type_error("RSI 的第一个参数必须是数组")),
        };
        
        let period = args[1].to_number()? as usize;
        indicators::rsi(prices, period)
    }
    
    /// BOLL 函数 - 布林带
    fn builtin_boll(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        // BOLL(prices, period, std_dev)
        if args.len() != 3 {
            return Err(RuntimeError::type_error("BOLL 需要 3 个参数"));
        }
        
        let prices = match &args[0] {
            Value::Array(arr) => arr,
            _ => return Err(RuntimeError::type_error("BOLL 的第一个参数必须是数组")),
        };
        
        let period = args[1].to_number()? as usize;
        let std_dev = args[2].to_number()?;
        
        indicators::bollinger_bands(prices, period, std_dev)
    }
    
    /// ATR 函数 - 真实波动幅度
    fn builtin_atr(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        // ATR(high, low, close, period)
        if args.len() != 4 {
            return Err(RuntimeError::type_error("ATR 需要 4 个参数"));
        }
        
        let high = match &args[0] {
            Value::Array(arr) => arr,
            _ => return Err(RuntimeError::type_error("ATR 的第一个参数必须是数组")),
        };
        
        let low = match &args[1] {
            Value::Array(arr) => arr,
            _ => return Err(RuntimeError::type_error("ATR 的第二个参数必须是数组")),
        };
        
        let close = match &args[2] {
            Value::Array(arr) => arr,
            _ => return Err(RuntimeError::type_error("ATR 的第三个参数必须是数组")),
        };
        
        let period = args[3].to_number()? as usize;
        indicators::atr(high, low, close, period)
    }
    
    /// KDJ 函数
    fn builtin_kdj(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        // KDJ(high, low, close, n, m1, m2)
        if args.len() != 6 {
            return Err(RuntimeError::type_error("KDJ 需要 6 个参数"));
        }
        
        let high = match &args[0] {
            Value::Array(arr) => arr,
            _ => return Err(RuntimeError::type_error("KDJ 的第一个参数必须是数组")),
        };
        
        let low = match &args[1] {
            Value::Array(arr) => arr,
            _ => return Err(RuntimeError::type_error("KDJ 的第二个参数必须是数组")),
        };
        
        let close = match &args[2] {
            Value::Array(arr) => arr,
            _ => return Err(RuntimeError::type_error("KDJ 的第三个参数必须是数组")),
        };
        
        let n = args[3].to_number()? as usize;
        let m1 = args[4].to_number()? as usize;
        let m2 = args[5].to_number()? as usize;
        
        indicators::kdj(high, low, close, n, m1, m2)
    }
    
    /// print 函数 - 打印输出（用于调试）
    fn builtin_print(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        // print(value1, value2, ...) - 打印所有参数，用空格分隔
        let mut output = Vec::new();
        
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                output.push(" ".to_string());
            }
            output.push(self.value_to_string(arg));
        }
        
        println!("{}", output.join(""));
        
        // 返回 null
        Ok(Value::Null)
    }
    
    /// 将 Value 转换为可读字符串
    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::Decimal(d) => d.to_string(),
            Value::String(s) => s.clone(),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| self.value_to_string(v)).collect();
                format!("[{}]", items.join(", "))
            }
            Value::ArraySlice { column_data, start, len } => {
                let items: Vec<String> = (0..*len)
                    .filter_map(|i| column_data.get(*start + i))
                    .map(|v| self.value_to_string(v))
                    .collect();
                format!("[{}]", items.join(", "))
            }
            Value::Lambda { params, .. } => {
                format!("<lambda({})>", params.join(", "))
            }
            Value::Function(func_def) => {
                format!("<function {}>", func_def.name)
            }
        }
    }
}
