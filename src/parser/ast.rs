// DPLang 语法分析器 - AST 定义

use std::fmt;
use crate::lexer::FStringPart;

/// when 表达式的分支
#[derive(Debug, Clone, PartialEq)]
pub struct WhenBranch {
    pub condition: Expr,
    pub result: Expr,
}

/// 表达式节点
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// 数字字面量
    Number(f64),
    
    /// 字符串字面量
    String(String),
    
    /// f-string 字符串插值
    FString(Vec<FStringPart>),
    
    /// 布尔字面量
    Bool(bool),
    
    /// null
    Null,
    
    /// 标识符
    Identifier(String),
    
    /// 数组字面量
    Array(Vec<Expr>),
    
    /// 二元运算
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    
    /// 一元运算
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    
    /// 三元表达式 cond ? then : else
    Ternary {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
    
    /// when 表达式
    When {
        branches: Vec<WhenBranch>,
        else_expr: Option<Box<Expr>>,
    },
    
    /// 函数调用
    Call {
        callee: String,
        args: Vec<Expr>,
    },
    
    /// 成员访问 object.member
    MemberAccess {
        object: String,
        member: String,
    },
    
    /// 数组索引访问 array[index] 或时间序列访问 var[-1]
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
    },
    
    /// 切片访问 var[-5:] 或 var[-5:-1]
    Slice {
        base: Box<Expr>,
        start: Option<Box<Expr>>,  // 起始索引（None表示从头开始）
        end: Option<Box<Expr>>,    // 结束索引（None表示到当前/末尾）
    },
    
    /// 展开运算符 ...expr
    Spread(Box<Expr>),
    
    /// Lambda 表达式
    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
    },
    
    /// 管道表达式
    Pipeline {
        value: Box<Expr>,
        stages: Vec<Expr>,
    },
}

/// 二元运算符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // 算术运算
    Add,  // +
    Sub,  // -
    Mul,  // *
    Div,  // /
    Mod,  // %
    Pow,  // ^
    
    // 比较运算
    Gt,    // >
    Lt,    // <
    GtEq,  // >=
    LtEq,  // <=
    Eq,    // ==
    NotEq, // !=
    
    // 逻辑运算
    And, // and
    Or,  // or
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Pow => "^",
            BinaryOp::Gt => ">",
            BinaryOp::Lt => "<",
            BinaryOp::GtEq => ">=",
            BinaryOp::LtEq => "<=",
            BinaryOp::Eq => "==",
            BinaryOp::NotEq => "!=",
            BinaryOp::And => "and",
            BinaryOp::Or => "or",
        };
        write!(f, "{}", s)
    }
}

/// 一元运算符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg, // -
    Not, // not
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "not",
        };
        write!(f, "{}", s)
    }
}

/// 语句节点
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// 变量赋值
    Assignment {
        name: String,
        value: Expr,
        is_mut: bool,
    },
    
    /// 解构赋值
    Destructure {
        pattern: Vec<DestructurePattern>,
        value: Expr,
    },
    
    /// 条件语句
    If {
        condition: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
    
    /// 返回语句
    Return(Expr),
    
    /// 表达式语句
    Expression(Expr),
}

/// 解构模式
#[derive(Debug, Clone, PartialEq)]
pub enum DestructurePattern {
    Identifier(String),
    Ignore,  // _
    Spread(String),  // ...rest
}

/// 函数定义
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<TypeAnnotation>,
    pub body: Vec<Stmt>,
    pub is_private: bool,
}

/// 函数参数
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: Option<TypeAnnotation>,
    pub default_value: Option<Expr>,  // 默认参数值
}

/// 类型标注
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeAnnotation {
    Number,
    Decimal,
    String,
    Bool,
    Array,
    Null,
}

/// 精度设置
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrecisionSetting {
    pub scale: u32,  // 小数位数
}

impl fmt::Display for TypeAnnotation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            TypeAnnotation::Number => "number",
            TypeAnnotation::Decimal => "decimal",
            TypeAnnotation::String => "string",
            TypeAnnotation::Bool => "bool",
            TypeAnnotation::Array => "array",
            TypeAnnotation::Null => "null",
        };
        write!(f, "{}", s)
    }
}

/// 变量定义 (包级)
#[derive(Debug, Clone, PartialEq)]
pub struct VariableDef {
    pub name: String,
    pub value: Expr,
    pub is_mut: bool,
    pub is_private: bool,
}

/// 脚本类型
#[derive(Debug, Clone, PartialEq)]
pub enum Script {
    /// 包脚本
    Package {
        name: String,
        variables: Vec<VariableDef>,
        functions: Vec<FunctionDef>,
    },
    
    /// 数据处理脚本
    DataScript {
        imports: Vec<String>,  // 导入的包列表
        input: Vec<Parameter>,
        output: Vec<Parameter>,
        error_block: Option<Vec<Stmt>>,
        precision: Option<PrecisionSetting>,
        body: Vec<Stmt>,
    },
}
