// 内置函数实现

use super::Executor;
use super::data_stream::CURRENT_DATA_STREAM;
use crate::runtime::{Value, RuntimeError};
use crate::indicators;
use chrono::{NaiveDateTime, NaiveDate, Duration, Utc, Local, TimeZone, Datelike, Timelike};

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
            
            // Null 处理函数
            "is_null" => self.builtin_is_null(args),
            "coalesce" => self.builtin_coalesce(args),
            "nvl" => self.builtin_nvl(args),
            "if_null" => self.builtin_if_null(args),
            "nullif" => self.builtin_nullif(args),
            
            // 时间日期函数
            "now" => self.builtin_now(args),
            "today" => self.builtin_today(args),
            "parse_time" => self.builtin_parse_time(args),
            "format_time" => self.builtin_format_time(args),
            "time_diff" => self.builtin_time_diff(args),
            "time_add" => self.builtin_time_add(args),
            "time_sub" => self.builtin_time_sub(args),
            "year" => self.builtin_year(args),
            "month" => self.builtin_month(args),
            "day" => self.builtin_day(args),
            "hour" => self.builtin_hour(args),
            "minute" => self.builtin_minute(args),
            "second" => self.builtin_second(args),
            "weekday" => self.builtin_weekday(args),
            "timestamp" => self.builtin_timestamp(args),
            "from_timestamp" => self.builtin_from_timestamp(args),
            
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
    
    // ==================== Null 处理函数 ====================
    
    /// is_null(value) - 检查值是否为 null
    fn builtin_is_null(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::type_error("is_null 需要 1 个参数"));
        }
        Ok(Value::Bool(args[0].is_null()))
    }
    
    /// coalesce(value1, value2, ...) - 返回第一个非 null 值
    fn builtin_coalesce(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.is_empty() {
            return Err(RuntimeError::type_error("coalesce 需要至少一个参数"));
        }
        
        for arg in args {
            if !arg.is_null() {
                return Ok(arg.clone());
            }
        }
        
        // 所有参数都是 null，返回 null
        Ok(Value::Null)
    }
    
    /// nvl(value, default) - 如果 value 为 null，返回 default
    fn builtin_nvl(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::type_error("nvl 需要 2 个参数"));
        }
        
        if args[0].is_null() {
            Ok(args[1].clone())
        } else {
            Ok(args[0].clone())
        }
    }
    
    /// if_null(value, default) - nvl 的别名
    fn builtin_if_null(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        self.builtin_nvl(args)
    }
    
    /// nullif(value1, value2) - 如果两个值相等则返回 null，否则返回 value1
    fn builtin_nullif(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::type_error("nullif 需要 2 个参数"));
        }
        
        if args[0] == args[1] {
            Ok(Value::Null)
        } else {
            Ok(args[0].clone())
        }
    }
    
    // ==================== 时间日期函数 ====================
    
    /// now() - 返回当前时间戳字符串（ISO 8601格式）
    fn builtin_now(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if !args.is_empty() {
            return Err(RuntimeError::type_error("now 不需要参数"));
        }
        
        let now = Local::now();
        Ok(Value::String(now.format("%Y-%m-%d %H:%M:%S").to_string()))
    }
    
    /// today() - 返回当前日期字符串（YYYY-MM-DD格式）
    fn builtin_today(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if !args.is_empty() {
            return Err(RuntimeError::type_error("today 不需要参数"));
        }
        
        let today = Local::now().date_naive();
        Ok(Value::String(today.format("%Y-%m-%d").to_string()))
    }
    
    /// parse_time(time_string, format?) - 解析时间字符串
    /// 如果不指定格式，尝试常见格式
    fn builtin_parse_time(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.is_empty() || args.len() > 2 {
            return Err(RuntimeError::type_error("parse_time 需要 1-2 个参数"));
        }
        
        let time_str = match &args[0] {
            Value::String(s) => s,
            _ => return Err(RuntimeError::type_error("parse_time 的第一个参数必须是字符串")),
        };
        
        // 如果提供了格式
        if args.len() == 2 {
            let format = match &args[1] {
                Value::String(s) => s,
                _ => return Err(RuntimeError::type_error("parse_time 的第二个参数必须是字符串")),
            };
            
            // 尝试解析为日期时间
            if let Ok(dt) = NaiveDateTime::parse_from_str(time_str, format) {
                return Ok(Value::String(dt.format("%Y-%m-%d %H:%M:%S").to_string()));
            }
            
            // 尝试仅解析日期
            if let Ok(date) = NaiveDate::parse_from_str(time_str, format) {
                return Ok(Value::String(date.format("%Y-%m-%d").to_string()));
            }
            
            return Err(RuntimeError::type_error(&format!("无法解析时间字符串: {}", time_str)));
        }
        
        // 尝试常见格式
        let formats = vec![
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d %H:%M",
            "%Y-%m-%d",
            "%Y/%m/%d %H:%M:%S",
            "%Y/%m/%d",
            "%Y%m%d",
        ];
        
        for fmt in formats {
            if let Ok(dt) = NaiveDateTime::parse_from_str(time_str, fmt) {
                return Ok(Value::String(dt.format("%Y-%m-%d %H:%M:%S").to_string()));
            }
            if let Ok(date) = NaiveDate::parse_from_str(time_str, fmt) {
                return Ok(Value::String(date.format("%Y-%m-%d").to_string()));
            }
        }
        
        Err(RuntimeError::type_error(&format!("无法解析时间字符串: {}", time_str)))
    }
    
    /// format_time(time_string, format) - 格式化时间字符串
    fn builtin_format_time(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::type_error("format_time 需要 2 个参数"));
        }
        
        let time_str = match &args[0] {
            Value::String(s) => s,
            _ => return Err(RuntimeError::type_error("format_time 的第一个参数必须是字符串")),
        };
        
        let format = match &args[1] {
            Value::String(s) => s,
            _ => return Err(RuntimeError::type_error("format_time 的第二个参数必须是字符串")),
        };
        
        // 先尝试解析时间字符串
        let parsed = self.builtin_parse_time(&[Value::String(time_str.clone())])?;
        
        if let Value::String(parsed_str) = parsed {
            // 解析标准格式
            if let Ok(dt) = NaiveDateTime::parse_from_str(&parsed_str, "%Y-%m-%d %H:%M:%S") {
                return Ok(Value::String(dt.format(format).to_string()));
            }
            if let Ok(date) = NaiveDate::parse_from_str(&parsed_str, "%Y-%m-%d") {
                return Ok(Value::String(date.format(format).to_string()));
            }
        }
        
        Err(RuntimeError::type_error("格式化时间失败"))
    }
    
    /// time_diff(time1, time2, unit?) - 计算两个时间的差值
    /// unit: "seconds", "minutes", "hours", "days" (默认 "days")
    fn builtin_time_diff(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() < 2 || args.len() > 3 {
            return Err(RuntimeError::type_error("time_diff 需要 2-3 个参数"));
        }
        
        let time1_str = match &args[0] {
            Value::String(s) => s,
            _ => return Err(RuntimeError::type_error("time_diff 的第一个参数必须是字符串")),
        };
        
        let time2_str = match &args[1] {
            Value::String(s) => s,
            _ => return Err(RuntimeError::type_error("time_diff 的第二个参数必须是字符串")),
        };
        
        let unit = if args.len() == 3 {
            match &args[2] {
                Value::String(s) => s.as_str(),
                _ => return Err(RuntimeError::type_error("time_diff 的第三个参数必须是字符串")),
            }
        } else {
            "days"
        };
        
        // 解析两个时间
        let dt1 = self.parse_datetime_flexible(time1_str)?;
        let dt2 = self.parse_datetime_flexible(time2_str)?;
        
        // 计算差值
        let duration = dt1.signed_duration_since(dt2);
        
        let result = match unit {
            "seconds" => duration.num_seconds() as f64,
            "minutes" => duration.num_minutes() as f64,
            "hours" => duration.num_hours() as f64,
            "days" => duration.num_days() as f64,
            _ => return Err(RuntimeError::type_error(&format!("不支持的时间单位: {}", unit))),
        };
        
        Ok(Value::Number(result))
    }
    
    /// time_add(time_string, amount, unit) - 时间加法
    fn builtin_time_add(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 3 {
            return Err(RuntimeError::type_error("time_add 需要 3 个参数"));
        }
        
        let time_str = match &args[0] {
            Value::String(s) => s,
            _ => return Err(RuntimeError::type_error("time_add 的第一个参数必须是字符串")),
        };
        
        let amount = args[1].to_number()? as i64;
        
        let unit = match &args[2] {
            Value::String(s) => s.as_str(),
            _ => return Err(RuntimeError::type_error("time_add 的第三个参数必须是字符串")),
        };
        
        let dt = self.parse_datetime_flexible(time_str)?;
        
        let new_dt = match unit {
            "seconds" => dt + Duration::seconds(amount),
            "minutes" => dt + Duration::minutes(amount),
            "hours" => dt + Duration::hours(amount),
            "days" => dt + Duration::days(amount),
            "weeks" => dt + Duration::weeks(amount),
            _ => return Err(RuntimeError::type_error(&format!("不支持的时间单位: {}", unit))),
        };
        
        Ok(Value::String(new_dt.format("%Y-%m-%d %H:%M:%S").to_string()))
    }
    
    /// time_sub(time_string, amount, unit) - 时间减法
    fn builtin_time_sub(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 3 {
            return Err(RuntimeError::type_error("time_sub 需要 3 个参数"));
        }
        
        // 取反 amount 并调用 time_add
        let amount = args[1].to_number()?;
        let negated_amount = Value::Number(-amount);
        
        self.builtin_time_add(&[args[0].clone(), negated_amount, args[2].clone()])
    }
    
    /// year(time_string) - 提取年份
    fn builtin_year(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::type_error("year 需要 1 个参数"));
        }
        
        let time_str = match &args[0] {
            Value::String(s) => s,
            _ => return Err(RuntimeError::type_error("year 的参数必须是字符串")),
        };
        
        let dt = self.parse_datetime_flexible(time_str)?;
        Ok(Value::Number(dt.year() as f64))
    }
    
    /// month(time_string) - 提取月份
    fn builtin_month(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::type_error("month 需要 1 个参数"));
        }
        
        let time_str = match &args[0] {
            Value::String(s) => s,
            _ => return Err(RuntimeError::type_error("month 的参数必须是字符串")),
        };
        
        let dt = self.parse_datetime_flexible(time_str)?;
        Ok(Value::Number(dt.month() as f64))
    }
    
    /// day(time_string) - 提取日期
    fn builtin_day(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::type_error("day 需要 1 个参数"));
        }
        
        let time_str = match &args[0] {
            Value::String(s) => s,
            _ => return Err(RuntimeError::type_error("day 的参数必须是字符串")),
        };
        
        let dt = self.parse_datetime_flexible(time_str)?;
        Ok(Value::Number(dt.day() as f64))
    }
    
    /// hour(time_string) - 提取小时
    fn builtin_hour(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::type_error("hour 需要 1 个参数"));
        }
        
        let time_str = match &args[0] {
            Value::String(s) => s,
            _ => return Err(RuntimeError::type_error("hour 的参数必须是字符串")),
        };
        
        let dt = self.parse_datetime_flexible(time_str)?;
        Ok(Value::Number(dt.hour() as f64))
    }
    
    /// minute(time_string) - 提取分钟
    fn builtin_minute(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::type_error("minute 需要 1 个参数"));
        }
        
        let time_str = match &args[0] {
            Value::String(s) => s,
            _ => return Err(RuntimeError::type_error("minute 的参数必须是字符串")),
        };
        
        let dt = self.parse_datetime_flexible(time_str)?;
        Ok(Value::Number(dt.minute() as f64))
    }
    
    /// second(time_string) - 提取秒数
    fn builtin_second(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::type_error("second 需要 1 个参数"));
        }
        
        let time_str = match &args[0] {
            Value::String(s) => s,
            _ => return Err(RuntimeError::type_error("second 的参数必须是字符串")),
        };
        
        let dt = self.parse_datetime_flexible(time_str)?;
        Ok(Value::Number(dt.second() as f64))
    }
    
    /// weekday(time_string) - 获取星期几（0=周一，6=周日）
    fn builtin_weekday(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::type_error("weekday 需要 1 个参数"));
        }
        
        let time_str = match &args[0] {
            Value::String(s) => s,
            _ => return Err(RuntimeError::type_error("weekday 的参数必须是字符串")),
        };
        
        let dt = self.parse_datetime_flexible(time_str)?;
        // 0 = Monday, 6 = Sunday
        Ok(Value::Number(dt.weekday().num_days_from_monday() as f64))
    }
    
    /// timestamp(time_string) - 转换为Unix时间戳（秒）
    fn builtin_timestamp(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::type_error("timestamp 需要 1 个参数"));
        }
        
        let time_str = match &args[0] {
            Value::String(s) => s,
            _ => return Err(RuntimeError::type_error("timestamp 的参数必须是字符串")),
        };
        
        let dt = self.parse_datetime_flexible(time_str)?;
        Ok(Value::Number(dt.and_utc().timestamp() as f64))
    }
    
    /// from_timestamp(timestamp) - 从Unix时间戳转换为时间字符串
    fn builtin_from_timestamp(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::type_error("from_timestamp 需要 1 个参数"));
        }
        
        let timestamp = args[0].to_number()? as i64;
        
        match Utc.timestamp_opt(timestamp, 0) {
            chrono::LocalResult::Single(dt) => {
                let local_dt = dt.with_timezone(&Local);
                Ok(Value::String(local_dt.format("%Y-%m-%d %H:%M:%S").to_string()))
            },
            _ => Err(RuntimeError::type_error("无效的时间戳")),
        }
    }
    
    // ==================== 辅助函数 ====================
    
    /// 灵活解析时间字符串（支持多种格式）
    fn parse_datetime_flexible(&self, time_str: &str) -> Result<NaiveDateTime, RuntimeError> {
        // 尝试常见的日期时间格式
        let datetime_formats = vec![
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d %H:%M",
            "%Y/%m/%d %H:%M:%S",
            "%Y/%m/%d %H:%M",
        ];
        
        for fmt in datetime_formats {
            if let Ok(dt) = NaiveDateTime::parse_from_str(time_str, fmt) {
                return Ok(dt);
            }
        }
        
        // 尝试仅日期格式，补充时间为 00:00:00
        let date_formats = vec![
            "%Y-%m-%d",
            "%Y/%m/%d",
            "%Y%m%d",
        ];
        
        for fmt in date_formats {
            if let Ok(date) = NaiveDate::parse_from_str(time_str, fmt) {
                return Ok(date.and_hms_opt(0, 0, 0).unwrap());
            }
        }
        
        Err(RuntimeError::type_error(&format!("无法解析时间字符串: {}", time_str)))
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
