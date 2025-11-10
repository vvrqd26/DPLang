// 语义分析器 - 在执行前进行静态检查

use crate::parser::{Script, Stmt, Expr, FunctionDef};
use std::collections::{HashMap, HashSet};

/// 语义分析错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticErrorType {
    UndefinedVariable,
    VariableShadowing,
    UnusedVariable,
    TypeMismatch,
    InvalidOperation,
}

/// 语义分析错误
#[derive(Debug, Clone)]
pub struct SemanticError {
    pub error_type: SemanticErrorType,
    pub message: String,
    pub variable: Option<String>,
}

impl SemanticError {
    pub fn undefined_variable(name: &str) -> Self {
        SemanticError {
            error_type: SemanticErrorType::UndefinedVariable,
            message: format!("未定义的变量: {}", name),
            variable: Some(name.to_string()),
        }
    }
    
    pub fn variable_shadowing(name: &str) -> Self {
        SemanticError {
            error_type: SemanticErrorType::VariableShadowing,
            message: format!("变量遮蔽: {} 已在外层作用域定义", name),
            variable: Some(name.to_string()),
        }
    }
    
    pub fn unused_variable(name: &str) -> Self {
        SemanticError {
            error_type: SemanticErrorType::UnusedVariable,
            message: format!("未使用的变量: {}", name),
            variable: Some(name.to_string()),
        }
    }
    
    pub fn type_mismatch(expected: &str, actual: &str) -> Self {
        SemanticError {
            error_type: SemanticErrorType::TypeMismatch,
            message: format!("类型不匹配: 期望 {}, 实际 {}", expected, actual),
            variable: None,
        }
    }
}

/// 语义分析结果
#[derive(Debug)]
pub struct SemanticAnalysisResult {
    /// 错误列表
    pub errors: Vec<SemanticError>,
    /// 警告列表
    pub warnings: Vec<SemanticError>,
}

impl SemanticAnalysisResult {
    pub fn new() -> Self {
        SemanticAnalysisResult {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    pub fn add_error(&mut self, error: SemanticError) {
        self.errors.push(error);
    }
    
    pub fn add_warning(&mut self, warning: SemanticError) {
        self.warnings.push(warning);
    }
}

/// 作用域信息
#[derive(Debug, Clone)]
struct Scope {
    /// 定义的变量（变量名 -> 是否被使用）
    variables: HashMap<String, bool>,
    /// 父作用域
    parent: Option<Box<Scope>>,
}

impl Scope {
    fn new() -> Self {
        Scope {
            variables: HashMap::new(),
            parent: None,
        }
    }
    
    fn new_child(parent: Scope) -> Self {
        Scope {
            variables: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }
    
    /// 在当前作用域定义变量
    fn define(&mut self, name: String) {
        self.variables.insert(name, false);
    }
    
    /// 标记变量为已使用
    fn mark_used(&mut self, name: &str) -> bool {
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), true);
            true
        } else if let Some(ref mut parent) = self.parent {
            parent.mark_used(name)
        } else {
            false
        }
    }
    
    /// 检查变量是否在当前作用域定义
    fn is_defined_in_current(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }
    
    /// 检查变量是否在任何作用域定义
    fn is_defined(&self, name: &str) -> bool {
        if self.variables.contains_key(name) {
            true
        } else if let Some(ref parent) = self.parent {
            parent.is_defined(name)
        } else {
            false
        }
    }
    
    /// 获取未使用的变量
    fn get_unused_variables(&self) -> Vec<String> {
        self.variables
            .iter()
            .filter(|(_, used)| !**used)
            .map(|(name, _)| name.clone())
            .collect()
    }
}

/// 语义分析器
pub struct SemanticAnalyzer {
    /// 当前作用域
    scope: Scope,
    /// 分析结果
    result: SemanticAnalysisResult,
    /// 内置函数集合
    builtin_functions: HashSet<String>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut builtin_functions = HashSet::new();
        // 注册内置函数
        builtin_functions.insert("MA".to_string());
        builtin_functions.insert("sum".to_string());
        builtin_functions.insert("max".to_string());
        builtin_functions.insert("min".to_string());
        builtin_functions.insert("map".to_string());
        builtin_functions.insert("filter".to_string());
        builtin_functions.insert("reduce".to_string());
        builtin_functions.insert("ref".to_string());
        builtin_functions.insert("past".to_string());
        builtin_functions.insert("offset".to_string());
        builtin_functions.insert("window".to_string());
        
        SemanticAnalyzer {
            scope: Scope::new(),
            result: SemanticAnalysisResult::new(),
            builtin_functions,
        }
    }
    
    /// 分析脚本
    pub fn analyze(&mut self, script: &Script) -> SemanticAnalysisResult {
        match script {
            Script::DataScript { input, output, body, error_block, .. } => {
                // 定义 INPUT 变量
                for param in input {
                    self.scope.define(param.name.clone());
                }
                
                // 定义 OUTPUT 变量（需要在 body 中赋值）
                for param in output {
                    self.scope.define(param.name.clone());
                }
                
                // 分析主体
                self.analyze_statements(body);
                
                // 分析 ERROR 块
                if let Some(error_stmts) = error_block {
                    // ERROR 块有特殊变量 __error__
                    self.scope.define("__error__".to_string());
                    self.analyze_statements(error_stmts);
                }
                
                // 检查未使用的变量（排除 INPUT 和 OUTPUT）
                let unused = self.scope.get_unused_variables();
                for var in unused {
                    // INPUT 和 OUTPUT 变量可以不使用
                    let is_io = input.iter().any(|p| p.name == var) 
                             || output.iter().any(|p| p.name == var);
                    if !is_io && var != "__error__" {
                        self.result.add_warning(SemanticError::unused_variable(&var));
                    }
                }
            }
            
            Script::Package { variables, functions, .. } => {
                // 定义包变量
                for var_def in variables {
                    self.scope.define(var_def.name.clone());
                    self.analyze_expr(&var_def.value);
                }
                
                // 定义包函数
                for func_def in functions {
                    self.analyze_function(func_def);
                }
            }
        }
        
        std::mem::replace(&mut self.result, SemanticAnalysisResult::new())
    }
    
    /// 分析语句列表
    fn analyze_statements(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.analyze_stmt(stmt);
        }
    }
    
    /// 分析单条语句
    fn analyze_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Assignment { name, value, .. } => {
                // 检查是否遮蔽
                if self.scope.is_defined_in_current(name) {
                    // 重复赋值，不是遮蔽
                } else if self.scope.is_defined(name) {
                    self.result.add_warning(SemanticError::variable_shadowing(name));
                }
                
                // 定义变量
                self.scope.define(name.clone());
                
                // 分析右值表达式
                self.analyze_expr(value);
            }
            
            Stmt::Return(expr) => {
                self.analyze_expr(expr);
            }
            
            Stmt::If { condition, then_block, else_block } => {
                self.analyze_expr(condition);
                
                // then 块 - 使用两阶段方法避免借用冲突
                let old_scope = std::mem::replace(&mut self.scope, Scope::new());
                self.scope = Scope::new_child(old_scope);
                self.analyze_statements(then_block);
                if let Some(parent) = self.scope.parent.take() {
                    self.scope = *parent;
                }
                
                // else 块
                if let Some(else_stmts) = else_block {
                    let old_scope = std::mem::replace(&mut self.scope, Scope::new());
                    self.scope = Scope::new_child(old_scope);
                    self.analyze_statements(else_stmts);
                    if let Some(parent) = self.scope.parent.take() {
                        self.scope = *parent;
                    }
                }
            }
            
            Stmt::Expression(expr) => {
                self.analyze_expr(expr);
            }
            
            Stmt::Destructure { pattern, value } => {
                self.analyze_expr(value);
                
                // 定义模式中的变量
                for p in pattern {
                    match p {
                        crate::parser::DestructurePattern::Identifier(name) => {
                            self.scope.define(name.clone());
                        }
                        crate::parser::DestructurePattern::Spread(name) => {
                            self.scope.define(name.clone());
                        }
                        crate::parser::DestructurePattern::Ignore => {}
                    }
                }
            }
        }
    }
    
    /// 分析表达式
    fn analyze_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Identifier(name) => {
                // 检查变量是否定义
                if !self.scope.is_defined(name) && !self.builtin_functions.contains(name) {
                    self.result.add_error(SemanticError::undefined_variable(name));
                } else {
                    self.scope.mark_used(name);
                }
            }
            
            Expr::Array(elements) => {
                for elem in elements {
                    self.analyze_expr(elem);
                }
            }
            
            Expr::Binary { left, right, .. } => {
                self.analyze_expr(left);
                self.analyze_expr(right);
            }
            
            Expr::Unary { operand, .. } => {
                self.analyze_expr(operand);
            }
            
            Expr::Ternary { condition, then_expr, else_expr } => {
                self.analyze_expr(condition);
                self.analyze_expr(then_expr);
                self.analyze_expr(else_expr);
            }
            
            Expr::FString(parts) => {
                // 分析 f-string 中的表达式
                for part in parts {
                    if let crate::lexer::FStringPart::Expr(expr_str) = part {
                        // 解析并分析嵌入的表达式
                        if let Ok(mut lexer) = std::panic::catch_unwind(|| crate::lexer::Lexer::new(expr_str)) {
                            if let Ok(tokens) = lexer.tokenize() {
                                let mut parser = crate::parser::Parser::new(tokens);
                                if let Ok(script) = parser.parse() {
                                    if let crate::parser::Script::DataScript { body, .. } = script {
                                        for stmt in &body {
                                            self.analyze_stmt(stmt);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            Expr::When { branches, else_expr } => {
                // 分析 when 表达式的所有分支
                for branch in branches {
                    self.analyze_expr(&branch.condition);
                    self.analyze_expr(&branch.result);
                }
                if let Some(else_result) = else_expr {
                    self.analyze_expr(else_result);
                }
            }
            
            Expr::Call { callee, args } => {
                // 检查函数是否定义
                if !self.builtin_functions.contains(callee) && !self.scope.is_defined(callee) {
                    self.result.add_error(SemanticError::undefined_variable(callee));
                } else {
                    self.scope.mark_used(callee);
                }
                
                for arg in args {
                    self.analyze_expr(arg);
                }
            }
            
            Expr::Index { base, index } => {
                self.analyze_expr(base);
                self.analyze_expr(index);
            }
            
            Expr::Slice { base, start, end } => {
                self.analyze_expr(base);
                if let Some(s) = start {
                    self.analyze_expr(s);
                }
                if let Some(e) = end {
                    self.analyze_expr(e);
                }
            }
            
            Expr::Spread(inner) => {
                self.analyze_expr(inner);
            }
            
            Expr::Lambda { params, body } => {
                // Lambda 创建新作用域 - 使用两阶段方法
                let old_scope = std::mem::replace(&mut self.scope, Scope::new());
                self.scope = Scope::new_child(old_scope);
                
                // 定义参数
                for param in params {
                    self.scope.define(param.clone());
                }
                
                // 分析 body
                self.analyze_expr(body);
                
                if let Some(parent) = self.scope.parent.take() {
                    self.scope = *parent;
                }
            }
            
            Expr::Pipeline { value, stages } => {
                self.analyze_expr(value);
                for stage in stages {
                    self.analyze_expr(stage);
                }
            }
            
            Expr::MemberAccess { object, member } => {
                // 包成员访问，object 是包名
                // 这里简化处理，不检查包是否存在
                let _ = (object, member);
            }
            
            // 字面量不需要检查
            Expr::Number(_) | Expr::String(_) | Expr::Bool(_) | Expr::Null => {}
        }
    }
    
    /// 分析函数定义
    fn analyze_function(&mut self, func_def: &FunctionDef) {
        // 创建新作用域 - 使用两阶段方法
        let old_scope = std::mem::replace(&mut self.scope, Scope::new());
        self.scope = Scope::new_child(old_scope);
        
        // 定义参数
        for param in &func_def.params {
            self.scope.define(param.name.clone());
        }
        
        // 分析函数体
        self.analyze_statements(&func_def.body);
        
        // 检查未使用的变量
        let unused = self.scope.get_unused_variables();
        for var in unused {
            // 参数可以不使用
            let is_param = func_def.params.iter().any(|p| p.name == var);
            if !is_param {
                self.result.add_warning(SemanticError::unused_variable(&var));
            }
        }
        
        if let Some(parent) = self.scope.parent.take() {
            self.scope = *parent;
        }
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    
    #[test]
    fn test_undefined_variable() {
        let source = r#"
-- INPUT x:number --
-- OUTPUT result:number --

result = y * 2
return [result]
"#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let script = parser.parse().unwrap();
        
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&script);
        
        assert!(result.has_errors());
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].error_type, SemanticErrorType::UndefinedVariable);
    }
    
    #[test]
    fn test_variable_shadowing() {
        let source = r#"
-- INPUT x:number --
-- OUTPUT result:number --

y = x * 2
y = y + 1
result = y
return [result]
"#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let script = parser.parse().unwrap();
        
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&script);
        
        // 重复赋值不算遮蔽
        assert!(!result.has_errors());
    }
    
    #[test]
    fn test_unused_variable() {
        let source = r#"
-- INPUT x:number --
-- OUTPUT result:number --

y = x * 2
z = 10
result = y
return [result]
"#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let script = parser.parse().unwrap();
        
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&script);
        
        assert!(!result.has_errors());
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].error_type, SemanticErrorType::UnusedVariable);
        assert_eq!(result.warnings[0].variable, Some("z".to_string()));
    }
    
    #[test]
    fn test_valid_script() {
        let source = r#"
-- INPUT close:number --
-- OUTPUT ma2:number --

prev_close = ref("close", 1)
ma2 = prev_close == null ? close : (close + prev_close) / 2
return [ma2]
"#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let script = parser.parse().unwrap();
        
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&script);
        
        assert!(!result.has_errors());
        assert!(result.warnings.is_empty());
    }
    
    #[test]
    fn test_lambda_scope() {
        let source = r#"
-- INPUT nums:array --
-- OUTPUT result:array --

doubled = map(nums, x -> x * 2)
result = doubled
return [result]
"#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let script = parser.parse().unwrap();
        
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&script);
        
        assert!(!result.has_errors());
    }
}
