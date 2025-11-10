// DPLang 运行时 - Value 类型和基本运算

use std::fmt;
use rust_decimal::Decimal;
use std::str::FromStr;

/// 运行时值
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Decimal(Decimal),
    String(String),
    Bool(bool),
    Null,
    Array(Vec<Value>),
    /// 数组切片（零拷贝引用）
    ArraySlice {
        /// 底层列数据的共享引用
        column_data: std::rc::Rc<Vec<Value>>,
        /// 切片起始索引
        start: usize,
        /// 切片长度
        len: usize,
    },
    /// Lambda 函数
    Lambda {
        params: Vec<String>,
        body: Box<crate::parser::Expr>,
        /// 捕获的变量环境
        captures: std::collections::HashMap<String, Box<Value>>,
    },
    /// 用户定义函数（包函数）
    Function(Box<crate::parser::FunctionDef>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Decimal(d) => write!(f, "{}", d),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::ArraySlice { column_data, start, len } => {
                write!(f, "[")?;
                for i in 0..*len {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    if let Some(v) = column_data.get(*start + i) {
                        write!(f, "{}", v)?;
                    }
                }
                write!(f, "]")
            }
            Value::Lambda { params, .. } => {
                write!(f, "<lambda({})>", params.join(", "))
            }
            Value::Function(func_def) => {
                write!(f, "<function {}>", func_def.name)
            }
        }
    }
}

impl Value {
    /// 转换为 bool (用于条件判断)
    pub fn to_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Number(n) => *n != 0.0,
            Value::Decimal(d) => !d.is_zero(),
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::ArraySlice { len, .. } => *len > 0,
            Value::Lambda { .. } => true,  // Lambda 总是真值
            Value::Function(_) => true,  // Function 总是真值
        }
    }
    
    /// 转换为 f64
    pub fn to_number(&self) -> Result<f64, RuntimeError> {
        match self {
            Value::Number(n) => Ok(*n),
            Value::Decimal(d) => Ok(d.to_string().parse().unwrap_or(0.0)),
            Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
            Value::Null => Ok(0.0),  // Null 转换为 0，不影响运行
            Value::String(s) => s.parse().map_err(|_| RuntimeError::type_error("无法转换为数字")),
            Value::Lambda { .. } => Err(RuntimeError::type_error("Lambda 无法转换为数字")),
            _ => Err(RuntimeError::type_error("无法转换为数字")),
        }
    }
        
    /// 转换为 Decimal
    pub fn to_decimal(&self) -> Result<Decimal, RuntimeError> {
        match self {
            Value::Decimal(d) => Ok(*d),
            Value::Number(n) => {
                // 将 f64 转为字符串再解析为 Decimal
                Decimal::from_str(&n.to_string())
                    .map_err(|_| RuntimeError::type_error("无法转换为 Decimal"))
            }
            Value::Bool(b) => Ok(if *b { Decimal::ONE } else { Decimal::ZERO }),
            Value::String(s) => {
                Decimal::from_str(s).map_err(|_| RuntimeError::type_error("无法转换为 Decimal"))
            }
            _ => Err(RuntimeError::type_error("无法转换为 Decimal")),
        }
    }
        
    /// 应用精度设置，转换为 Decimal 并设置小数位数
    pub fn apply_precision(&self, scale: u32) -> Result<Value, RuntimeError> {
        let decimal = self.to_decimal()?;
        // 设置小数位数
        let rounded = decimal.round_dp(scale);
        Ok(Value::Decimal(rounded))
    }
    
    /// 加法
    pub fn add(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::Decimal(a), Value::Decimal(b)) => Ok(Value::Decimal(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            (Value::Array(a), Value::Array(b)) => {
                // 向量加法 (逐元素)
                if a.len() != b.len() {
                    return Err(RuntimeError::type_error("数组长度不匹配"));
                }
                let mut result = Vec::new();
                for (av, bv) in a.iter().zip(b.iter()) {
                    result.push(av.add(bv)?);
                }
                Ok(Value::Array(result))
            }
            (Value::Array(a), scalar) | (scalar, Value::Array(a)) if !matches!(scalar, Value::Array(_)) => {
                // 数组与标量相加 (广播)
                let mut result = Vec::new();
                for av in a.iter() {
                    result.push(av.add(scalar)?);
                }
                Ok(Value::Array(result))
            }
            _ => Err(RuntimeError::type_error(&format!("无法执行加法: {} + {}", self, other))),
        }
    }
    
    /// 减法
    pub fn sub(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
            (Value::Decimal(a), Value::Decimal(b)) => Ok(Value::Decimal(a - b)),
            (Value::Array(a), Value::Array(b)) => {
                if a.len() != b.len() {
                    return Err(RuntimeError::type_error("数组长度不匹配"));
                }
                let mut result = Vec::new();
                for (av, bv) in a.iter().zip(b.iter()) {
                    result.push(av.sub(bv)?);
                }
                Ok(Value::Array(result))
            }
            (Value::Array(a), scalar) => {
                let mut result = Vec::new();
                for av in a.iter() {
                    result.push(av.sub(scalar)?);
                }
                Ok(Value::Array(result))
            }
            (scalar, Value::Array(a)) if !matches!(scalar, Value::Array(_)) => {
                let mut result = Vec::new();
                for av in a.iter() {
                    result.push(scalar.sub(av)?);
                }
                Ok(Value::Array(result))
            }
            _ => Err(RuntimeError::type_error(&format!("无法执行减法: {} - {}", self, other))),
        }
    }
    
    /// 乘法
    pub fn mul(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
            (Value::Decimal(a), Value::Decimal(b)) => Ok(Value::Decimal(a * b)),
            (Value::Array(a), Value::Array(b)) => {
                if a.len() != b.len() {
                    return Err(RuntimeError::type_error("数组长度不匹配"));
                }
                let mut result = Vec::new();
                for (av, bv) in a.iter().zip(b.iter()) {
                    result.push(av.mul(bv)?);
                }
                Ok(Value::Array(result))
            }
            (Value::Array(a), scalar) | (scalar, Value::Array(a)) if !matches!(scalar, Value::Array(_)) => {
                let mut result = Vec::new();
                for av in a.iter() {
                    result.push(av.mul(scalar)?);
                }
                Ok(Value::Array(result))
            }
            _ => Err(RuntimeError::type_error(&format!("无法执行乘法: {} * {}", self, other))),
        }
    }
    
    /// 除法
    pub fn div(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => {
                if *b == 0.0 {
                    return Err(RuntimeError::zero_division());
                }
                Ok(Value::Number(a / b))
            }
            (Value::Decimal(a), Value::Decimal(b)) => {
                if b.is_zero() {
                    return Err(RuntimeError::zero_division());
                }
                Ok(Value::Decimal(a / b))
            }
            (Value::Array(a), Value::Array(b)) => {
                if a.len() != b.len() {
                    return Err(RuntimeError::type_error("数组长度不匹配"));
                }
                let mut result = Vec::new();
                for (av, bv) in a.iter().zip(b.iter()) {
                    result.push(av.div(bv)?);
                }
                Ok(Value::Array(result))
            }
            (Value::Array(a), scalar) => {
                let mut result = Vec::new();
                for av in a.iter() {
                    result.push(av.div(scalar)?);
                }
                Ok(Value::Array(result))
            }
            (scalar, Value::Array(a)) if !matches!(scalar, Value::Array(_)) => {
                let mut result = Vec::new();
                for av in a.iter() {
                    result.push(scalar.div(av)?);
                }
                Ok(Value::Array(result))
            }
            _ => Err(RuntimeError::type_error(&format!("无法执行除法: {} / {}", self, other))),
        }
    }
    
    /// 取模
    pub fn modulo(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => {
                if *b == 0.0 {
                    return Err(RuntimeError::zero_division());
                }
                Ok(Value::Number(a % b))
            }
            _ => Err(RuntimeError::type_error("取模运算仅支持数字")),
        }
    }
    
    /// 幂运算
    pub fn pow(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a.powf(*b))),
            _ => Err(RuntimeError::type_error("幂运算仅支持数字")),
        }
    }
    
    /// 比较运算
    pub fn gt(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a > b)),
            (Value::Decimal(a), Value::Decimal(b)) => Ok(Value::Bool(a > b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a > b)),
            (Value::Array(a), Value::Array(b)) => {
                if a.len() != b.len() {
                    return Err(RuntimeError::type_error("数组长度不匹配"));
                }
                let mut result = Vec::new();
                for (av, bv) in a.iter().zip(b.iter()) {
                    result.push(av.gt(bv)?);
                }
                Ok(Value::Array(result))
            }
            (Value::Array(a), scalar) | (scalar, Value::Array(a)) if !matches!(scalar, Value::Array(_)) => {
                let mut result = Vec::new();
                for av in a.iter() {
                    result.push(if matches!(self, Value::Array(_)) { av.gt(scalar)? } else { scalar.gt(av)? });
                }
                Ok(Value::Array(result))
            }
            _ => Err(RuntimeError::type_error("无法比较")),
        }
    }
    
    pub fn lt(&self, other: &Value) -> Result<Value, RuntimeError> {
        other.gt(self)
    }
    
    pub fn gte(&self, other: &Value) -> Result<Value, RuntimeError> {
        Ok(Value::Bool(self.gt(other)?.to_bool() || self.eq(other)?.to_bool()))
    }
    
    pub fn lte(&self, other: &Value) -> Result<Value, RuntimeError> {
        Ok(Value::Bool(self.lt(other)?.to_bool() || self.eq(other)?.to_bool()))
    }
    
    pub fn eq(&self, other: &Value) -> Result<Value, RuntimeError> {
        Ok(Value::Bool(self == other))
    }
    
    pub fn neq(&self, other: &Value) -> Result<Value, RuntimeError> {
        Ok(Value::Bool(self != other))
    }
    
    /// 逻辑与
    pub fn and(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Array(a), Value::Array(b)) => {
                if a.len() != b.len() {
                    return Err(RuntimeError::type_error("数组长度不匹配"));
                }
                let mut result = Vec::new();
                for (av, bv) in a.iter().zip(b.iter()) {
                    result.push(Value::Bool(av.to_bool() && bv.to_bool()));
                }
                Ok(Value::Array(result))
            }
            _ => Ok(Value::Bool(self.to_bool() && other.to_bool())),
        }
    }
    
    /// 逻辑或
    pub fn or(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Array(a), Value::Array(b)) => {
                if a.len() != b.len() {
                    return Err(RuntimeError::type_error("数组长度不匹配"));
                }
                let mut result = Vec::new();
                for (av, bv) in a.iter().zip(b.iter()) {
                    result.push(Value::Bool(av.to_bool() || bv.to_bool()));
                }
                Ok(Value::Array(result))
            }
            _ => Ok(Value::Bool(self.to_bool() || other.to_bool())),
        }
    }
    
    /// 逻辑非
    pub fn not(&self) -> Result<Value, RuntimeError> {
        match self {
            Value::Array(a) => {
                let mut result = Vec::new();
                for av in a.iter() {
                    result.push(Value::Bool(!av.to_bool()));
                }
                Ok(Value::Array(result))
            }
            _ => Ok(Value::Bool(!self.to_bool())),
        }
    }
    
    /// 负号
    pub fn neg(&self) -> Result<Value, RuntimeError> {
        match self {
            Value::Number(n) => Ok(Value::Number(-n)),
            Value::Decimal(d) => Ok(Value::Decimal(-d)),
            Value::Array(a) => {
                let mut result = Vec::new();
                for av in a.iter() {
                    result.push(av.neg()?);
                }
                Ok(Value::Array(result))
            }
            _ => Err(RuntimeError::type_error("无法取负")),
        }
    }
}

/// 运行时错误
#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub error_type: ErrorType,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorType {
    ZeroDivision,
    TypeError,
    IndexOutOfBounds,
    NullReference,
    UndefinedVariable,
    UndefinedFunction,
    ArgumentMismatch,
}

impl RuntimeError {
    pub fn zero_division() -> Self {
        RuntimeError {
            error_type: ErrorType::ZeroDivision,
            message: "除零错误".to_string(),
        }
    }
    
    pub fn type_error(message: &str) -> Self {
        RuntimeError {
            error_type: ErrorType::TypeError,
            message: message.to_string(),
        }
    }
    
    pub fn undefined_variable(name: &str) -> Self {
        RuntimeError {
            error_type: ErrorType::UndefinedVariable,
            message: format!("未定义的变量: {}", name),
        }
    }
    
    pub fn undefined_function(name: &str) -> Self {
        RuntimeError {
            error_type: ErrorType::UndefinedFunction,
            message: format!("未定义的函数: {}", name),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "运行时错误: {}", self.message)
    }
}

impl std::error::Error for RuntimeError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_number_add() {
        let a = Value::Number(1.0);
        let b = Value::Number(2.0);
        assert_eq!(a.add(&b).unwrap(), Value::Number(3.0));
    }
    
    #[test]
    fn test_array_add() {
        let a = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
        let b = Value::Array(vec![Value::Number(3.0), Value::Number(4.0)]);
        let result = a.add(&b).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr[0], Value::Number(4.0));
            assert_eq!(arr[1], Value::Number(6.0));
        } else {
            panic!("Expected array");
        }
    }
    
    #[test]
    fn test_array_scalar() {
        let a = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
        let s = Value::Number(10.0);
        let result = a.add(&s).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr[0], Value::Number(11.0));
            assert_eq!(arr[1], Value::Number(12.0));
        } else {
            panic!("Expected array");
        }
    }
    
    #[test]
    fn test_comparison() {
        let a = Value::Number(5.0);
        let b = Value::Number(3.0);
        assert_eq!(a.gt(&b).unwrap(), Value::Bool(true));
        assert_eq!(a.lt(&b).unwrap(), Value::Bool(false));
    }
}
