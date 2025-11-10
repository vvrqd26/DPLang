// DPLang 语法分析器

pub mod ast;

use crate::lexer::{Token, TokenType};
pub use ast::*;
use std::fmt;

/// 解析错误
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "语法错误 [{}:{}]: {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for ParseError {}

/// 语法分析器
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }
    
    /// 解析整个脚本
    pub fn parse(&mut self) -> Result<Script, ParseError> {
        self.skip_newlines();
        
        // 检查是否是包脚本
        if self.match_token(&[TokenType::Package]) {
            self.parse_package_script()
        } else {
            self.parse_data_script()
        }
    }
    
    // ========== 脚本解析 ==========
    
    fn parse_package_script(&mut self) -> Result<Script, ParseError> {
        // package 名称
        let name = self.expect_identifier("期望包名")?;
        self.consume_newlines()?;
        
        let mut variables = Vec::new();
        let mut functions = Vec::new();
        
        while !self.is_at_end() {
            self.skip_newlines();
            
            if self.is_at_end() {
                break;
            }
            
            // 检查是否是函数定义 (有参数列表)
            if self.is_function_definition() {
                functions.push(self.parse_function_def()?);
            } else {
                // 包级变量
                variables.push(self.parse_variable_def()?);
            }
            
            self.skip_newlines();
        }
        
        Ok(Script::Package {
            name,
            variables,
            functions,
        })
    }
    
    fn parse_data_script(&mut self) -> Result<Script, ParseError> {
        let mut imports = Vec::new();
        let mut input = Vec::new();
        let mut output = Vec::new();
        let mut error_block = None;
        let mut precision = None;
        let mut body = Vec::new();
        
        // 解析声明部分
        while !self.is_at_end() {
            self.skip_newlines();
            
            if let TokenType::Import(content) = &self.peek().token_type.clone() {
                self.advance();
                // 解析包列表："math, utils" -> ["math", "utils"]
                imports = content
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            } else if let TokenType::Input(content) = &self.peek().token_type.clone() {
                self.advance();
                input = self.parse_param_list_from_string(content)?;
            } else if let TokenType::Output(content) = &self.peek().token_type.clone() {
                self.advance();
                output = self.parse_param_list_from_string(content)?;
            } else if self.match_token(&[TokenType::Error]) {
                self.consume_newlines()?;
                error_block = Some(self.parse_block_until_error_end()?);
            } else if let TokenType::Precision(content) = &self.peek().token_type.clone() {
                self.advance();
                // 觢析精度位数，例如 "-- PRECISION 6 --" -> content="PRECISION 6"
                // 提取数字部分
                let scale = content
                    .split_whitespace()
                    .last()
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(6);
                precision = Some(PrecisionSetting { scale });
            } else {
                break;
            }
        }
        
        // 解析主体
        while !self.is_at_end() {
            self.skip_newlines();
            if self.is_at_end() {
                break;
            }
            body.push(self.parse_statement()?);
        }
        
        Ok(Script::DataScript {
            imports,
            input,
            output,
            error_block,
            precision,
            body,
        })
    }
    
    /// 从字符串解析参数列表 "code:string, close:number"
    fn parse_param_list_from_string(&self, content: &str) -> Result<Vec<Parameter>, ParseError> {
        let mut params = Vec::new();
        
        // 分割参数
        for param_str in content.split(',') {
            let param_str = param_str.trim();
            if param_str.is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = param_str.split(':').collect();
            if parts.len() != 2 {
                return Err(ParseError {
                    message: format!("参数格式错误: {}", param_str),
                    line: 0,
                    column: 0,
                });
            }
            
            let name = parts[0].trim().to_string();
            let type_str = parts[1].trim();
            
            let type_annotation = match type_str {
                "number" => Some(TypeAnnotation::Number),
                "decimal" => Some(TypeAnnotation::Decimal),
                "string" => Some(TypeAnnotation::String),
                "bool" => Some(TypeAnnotation::Bool),
                "array" => Some(TypeAnnotation::Array),
                "null" => Some(TypeAnnotation::Null),
                _ => {
                    return Err(ParseError {
                        message: format!("未知的类型: {}", type_str),
                        line: 0,
                        column: 0,
                    });
                }
            };
            
            params.push(Parameter { name, type_annotation });
        }
        
        Ok(params)
    }
    
    fn parse_type_annotation(&mut self) -> Result<TypeAnnotation, ParseError> {
        let token = self.advance();
        
        if let TokenType::Identifier(name) = &token.token_type {
            match name.as_str() {
                "number" => Ok(TypeAnnotation::Number),
                "decimal" => Ok(TypeAnnotation::Decimal),
                "string" => Ok(TypeAnnotation::String),
                "bool" => Ok(TypeAnnotation::Bool),
                "array" => Ok(TypeAnnotation::Array),
                "null" => Ok(TypeAnnotation::Null),
                _ => Err(self.error(format!("未知的类型: {}", name))),
            }
        } else {
            Err(self.error("期望类型名称"))
        }
    }
    
    fn parse_block_until_error_end(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        
        while !self.is_at_end() && !self.check(&TokenType::ErrorEnd) {
            self.skip_newlines();
            if self.check(&TokenType::ErrorEnd) {
                break;
            }
            stmts.push(self.parse_statement()?);
        }
        
        self.consume(TokenType::ErrorEnd, "期望 -- ERROR_END --")?;
        Ok(stmts)
    }
    
    fn is_function_definition(&self) -> bool {
        // 查找 identifier ( 模式
        let mut i = self.current;
        
        // 跳过 mut
        if i < self.tokens.len() && matches!(self.tokens[i].token_type, TokenType::Mut) {
            i += 1;
        }
        
        // identifier
        if i < self.tokens.len() && matches!(self.tokens[i].token_type, TokenType::Identifier(_)) {
            i += 1;
            // (
            if i < self.tokens.len() && matches!(self.tokens[i].token_type, TokenType::LeftParen) {
                return true;
            }
        }
        
        false
    }
    
    fn parse_variable_def(&mut self) -> Result<VariableDef, ParseError> {
        let is_mut = self.match_token(&[TokenType::Mut]);
        let name = self.expect_identifier("期望变量名")?;
        let is_private = name.starts_with('_');
        
        self.consume(TokenType::Assign, "期望 =")?;
        let value = self.parse_expression()?;
        self.skip_newlines();
        
        Ok(VariableDef {
            name,
            value,
            is_mut,
            is_private,
        })
    }
    
    fn parse_function_def(&mut self) -> Result<FunctionDef, ParseError> {
        let name = self.expect_identifier("期望函数名")?;
        let is_private = name.starts_with('_');
        
        self.consume(TokenType::LeftParen, "期望 (")?;
        
        let mut params = Vec::new();
        while !self.check(&TokenType::RightParen) {
            let param_name = self.expect_identifier("期望参数名")?;
            let type_annotation = if self.match_token(&[TokenType::Colon]) {
                Some(self.parse_type_annotation()?)
            } else {
                None
            };
            
            params.push(Parameter { name: param_name, type_annotation });
            
            if !self.match_token(&[TokenType::Comma]) {
                break;
            }
        }
        
        self.consume(TokenType::RightParen, "期望 )")?;
        
        let return_type = if self.match_token(&[TokenType::Arrow]) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };
        
        self.consume(TokenType::Colon, "期望 :")?;
        self.consume_newlines()?;
        
        // 解析函数体 (缩进块)
        self.consume(TokenType::Indent, "期望缩进的函数体")?;
        
        let mut body = Vec::new();
        while !self.check(&TokenType::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&TokenType::Dedent) {
                break;
            }
            body.push(self.parse_statement()?);
        }
        
        if !self.is_at_end() {
            self.consume(TokenType::Dedent, "期望函数体结束")?;
        }
        
        Ok(FunctionDef {
            name,
            params,
            return_type,
            body,
            is_private,
        })
    }
    
    // ========== 语句解析 ==========
    
    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        self.skip_newlines();
        
        // return 语句
        if self.match_token(&[TokenType::Return]) {
            let expr = self.parse_expression()?;
            self.skip_newlines();
            return Ok(Stmt::Return(expr));
        }
        
        // if 语句
        if self.match_token(&[TokenType::If]) {
            return self.parse_if_statement();
        }
        
        // 解构赋值 [a, b, c] = ...
        if self.check(&TokenType::LeftBracket) {
            let checkpoint = self.current;
            if let Ok(destructure) = self.try_parse_destructure() {
                return Ok(destructure);
            }
            self.current = checkpoint;
        }
        
        // 赋值或表达式
        if let TokenType::Identifier(_) = self.peek().token_type {
            let checkpoint = self.current;
            let name = self.expect_identifier("")?;
            
            if self.match_token(&[TokenType::Assign]) {
                // 赋值
                let value = self.parse_expression()?;
                self.skip_newlines();
                return Ok(Stmt::Assignment {
                    name,
                    value,
                    is_mut: false,
                });
            } else {
                // 不是赋值,回退并解析为表达式
                self.current = checkpoint;
            }
        }
        
        // 表达式语句
        let expr = self.parse_expression()?;
        self.skip_newlines();
        Ok(Stmt::Expression(expr))
    }
    
    fn parse_if_statement(&mut self) -> Result<Stmt, ParseError> {
        let condition = self.parse_expression()?;
        self.consume(TokenType::Colon, "期望 :")?;
        self.consume_newlines()?;
        
        // then 块
        self.consume(TokenType::Indent, "期望缩进的 if 块")?;
        let mut then_block = Vec::new();
        while !self.check(&TokenType::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&TokenType::Dedent) {
                break;
            }
            then_block.push(self.parse_statement()?);
        }
        
        if !self.is_at_end() {
            self.consume(TokenType::Dedent, "期望 if 块结束")?;
        }
        
        self.skip_newlines();  // 跳过if块后的换行
        
        // else 块 (可选)
        let else_block = if self.match_token(&[TokenType::Else]) {
            self.consume(TokenType::Colon, "期望 :")?;
            self.consume_newlines()?;
            self.consume(TokenType::Indent, "期望缩进的 else 块")?;
            
            let mut stmts = Vec::new();
            while !self.check(&TokenType::Dedent) && !self.is_at_end() {
                self.skip_newlines();
                if self.check(&TokenType::Dedent) {
                    break;
                }
                stmts.push(self.parse_statement()?);
            }
            
            if !self.is_at_end() {
                self.consume(TokenType::Dedent, "期望 else 块结束")?;
            }
            
            self.skip_newlines();  // 跳过else块后的换行
            
            Some(stmts)
        } else {
            None
        };
        
        Ok(Stmt::If {
            condition,
            then_block,
            else_block,
        })
    }
    
    fn try_parse_destructure(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftBracket, "")?;
        
        let mut pattern = Vec::new();
        while !self.check(&TokenType::RightBracket) {
            if self.match_token(&[TokenType::Underscore]) {
                pattern.push(DestructurePattern::Ignore);
            } else if self.match_token(&[TokenType::Spread]) {
                let name = self.expect_identifier("期望变量名")?;
                pattern.push(DestructurePattern::Spread(name));
            } else {
                let name = self.expect_identifier("期望变量名")?;
                pattern.push(DestructurePattern::Identifier(name));
            }
            
            if !self.match_token(&[TokenType::Comma]) {
                break;
            }
        }
        
        self.consume(TokenType::RightBracket, "期望 ]")?;
        self.consume(TokenType::Assign, "期望 =")?;
        
        let value = self.parse_expression()?;
        self.skip_newlines();
        
        Ok(Stmt::Destructure { pattern, value })
    }
    
    // ========== 表达式解析 (递归下降) ==========
    
    pub fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_pipeline()
    }
    
    // 管道运算符 (最低优先级)
    fn parse_pipeline(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_ternary()?;
        
        let mut stages = Vec::new();
        while self.match_token(&[TokenType::Pipeline]) {
            stages.push(self.parse_ternary()?);
        }
        
        if !stages.is_empty() {
            expr = Expr::Pipeline {
                value: Box::new(expr),
                stages,
            };
        }
        
        Ok(expr)
    }
    
    // 三元运算符（右结合）
    fn parse_ternary(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_or()?;
        
        if self.match_token(&[TokenType::Question]) {
            // then_expr 部分使用 parse_or 避免歧义
            // 如果需要嵌套三元，应该加括号
            let then_expr = self.parse_or()?;
            self.consume(TokenType::Colon, "期望三元表达式中的 :")?;
            // else_expr 递归调用 parse_ternary 以支持串联三元表达式
            // 例如: a ? b : c ? d : e 解析为 a ? b : (c ? d : e)
            let else_expr = self.parse_ternary()?;
            
            expr = Expr::Ternary {
                condition: Box::new(expr),
                then_expr: Box::new(then_expr),
                else_expr: Box::new(else_expr),
            };
        }
        
        Ok(expr)
    }
    
    // 逻辑或
    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and()?;
        
        while self.match_token(&[TokenType::Or]) {
            let right = self.parse_and()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinaryOp::Or,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    // 逻辑与
    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_comparison()?;
        
        while self.match_token(&[TokenType::And]) {
            let right = self.parse_comparison()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinaryOp::And,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    // 比较运算
    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_addition()?;
        
        while let Some(op) = self.match_comparison_op() {
            let right = self.parse_addition()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn match_comparison_op(&mut self) -> Option<BinaryOp> {
        let op = match self.peek().token_type {
            TokenType::Greater => BinaryOp::Gt,
            TokenType::Less => BinaryOp::Lt,
            TokenType::GreaterEq => BinaryOp::GtEq,
            TokenType::LessEq => BinaryOp::LtEq,
            TokenType::Equal => BinaryOp::Eq,
            TokenType::NotEqual => BinaryOp::NotEq,
            _ => return None,
        };
        self.advance();
        Some(op)
    }
    
    // 加减运算
    fn parse_addition(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplication()?;
        
        while let Some(op) = self.match_add_op() {
            let right = self.parse_multiplication()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn match_add_op(&mut self) -> Option<BinaryOp> {
        let op = match self.peek().token_type {
            TokenType::Plus => BinaryOp::Add,
            TokenType::Minus => BinaryOp::Sub,
            _ => return None,
        };
        self.advance();
        Some(op)
    }
    
    // 乘除模运算
    fn parse_multiplication(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_power()?;
        
        while let Some(op) = self.match_mul_op() {
            let right = self.parse_power()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn match_mul_op(&mut self) -> Option<BinaryOp> {
        let op = match self.peek().token_type {
            TokenType::Star => BinaryOp::Mul,
            TokenType::Slash => BinaryOp::Div,
            TokenType::Percent => BinaryOp::Mod,
            _ => return None,
        };
        self.advance();
        Some(op)
    }
    
    // 幂运算
    fn parse_power(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;
        
        if self.match_token(&[TokenType::Caret]) {
            let right = self.parse_power()?; // 右结合
            left = Expr::Binary {
                left: Box::new(left),
                op: BinaryOp::Pow,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    // 一元运算
    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(&[TokenType::Minus]) {
            let operand = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Neg,
                operand: Box::new(operand),
            });
        }
        
        if self.match_token(&[TokenType::Not]) {
            let operand = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(operand),
            });
        }
        
        self.parse_call()
    }
    
    // 函数调用和成员访问
    fn parse_call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;
        
        loop {
            if self.match_token(&[TokenType::LeftParen]) {
                // 函数调用
                let mut args = Vec::new();
                while !self.check(&TokenType::RightParen) {
                    args.push(self.parse_expression()?);
                    if !self.match_token(&[TokenType::Comma]) {
                        break;
                    }
                }
                self.consume(TokenType::RightParen, "期望 )")?;
                
                if let Expr::Identifier(name) = expr {
                    expr = Expr::Call { callee: name, args };
                } else if let Expr::MemberAccess { object, member } = expr {
                    expr = Expr::Call {
                        callee: format!("{}.{}", object, member),
                        args,
                    };
                } else {
                    return Err(self.error("只能调用函数或方法"));
                }
            } else if self.match_token(&[TokenType::LeftBracket]) {
                // 索引或切片访问
                // 判断是索引 base[index] 还是切片 base[start:end]
                
                // 检查是否为空切片 [:]
                if self.check(&TokenType::Colon) {
                    // [:] 或 [:end]
                    self.advance(); // 消耗 ':'
                    let end = if self.check(&TokenType::RightBracket) {
                        None
                    } else {
                        Some(Box::new(self.parse_expression()?))
                    };
                    self.consume(TokenType::RightBracket, "期望 ]")?;
                    expr = Expr::Slice {
                        base: Box::new(expr),
                        start: None,
                        end,
                    };
                } else {
                    // 解析第一个表达式
                    let first_expr = self.parse_expression()?;
                    
                    if self.match_token(&[TokenType::Colon]) {
                        // 切片: base[start:] 或 base[start:end]
                        let end = if self.check(&TokenType::RightBracket) {
                            None
                        } else {
                            Some(Box::new(self.parse_expression()?))
                        };
                        self.consume(TokenType::RightBracket, "期望 ]")?;
                        expr = Expr::Slice {
                            base: Box::new(expr),
                            start: Some(Box::new(first_expr)),
                            end,
                        };
                    } else {
                        // 索引: base[index]
                        self.consume(TokenType::RightBracket, "期望 ]")?;
                        expr = Expr::Index {
                            base: Box::new(expr),
                            index: Box::new(first_expr),
                        };
                    }
                }
            } else if self.match_token(&[TokenType::Dot]) {
                // 成员访问
                let member = self.expect_identifier("期望成员名")?;
                if let Expr::Identifier(object) = expr {
                    expr = Expr::MemberAccess { object, member };
                } else {
                    return Err(self.error("只能访问标识符的成员"));
                }
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    // 基础表达式
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.peek().clone();
        
        match &token.token_type {
            TokenType::Number(n) => {
                self.advance();
                Ok(Expr::Number(*n))
            }
            TokenType::String(s) => {
                self.advance();
                Ok(Expr::String(s.clone()))
            }
            TokenType::True => {
                self.advance();
                Ok(Expr::Bool(true))
            }
            TokenType::False => {
                self.advance();
                Ok(Expr::Bool(false))
            }
            TokenType::Null => {
                self.advance();
                Ok(Expr::Null)
            }
            TokenType::Identifier(_) => {
                let name = self.expect_identifier("")?;
                
                // Lambda 表达式检测: identifier ->
                if self.check(&TokenType::Arrow) {
                    self.current -= 1; // 回退
                    return self.parse_lambda();
                }
                
                Ok(Expr::Identifier(name))
            }
            TokenType::LeftParen => {
                self.advance();
                
                // Lambda 表达式: (params) ->
                if self.is_lambda_params() {
                    self.current -= 1; // 回退
                    return self.parse_lambda();
                }
                
                let expr = self.parse_expression()?;
                self.consume(TokenType::RightParen, "期望 )")?;
                Ok(expr)
            }
            TokenType::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();
                
                while !self.check(&TokenType::RightBracket) {
                    elements.push(self.parse_expression()?);
                    if !self.match_token(&[TokenType::Comma]) {
                        break;
                    }
                }
                
                self.consume(TokenType::RightBracket, "期望 ]")?;
                Ok(Expr::Array(elements))
            }
            TokenType::Spread => {
                self.advance();
                let expr = self.parse_primary()?;
                Ok(Expr::Spread(Box::new(expr)))
            }
            _ => Err(self.error(format!("意外的 token: {:?}", token.token_type))),
        }
    }
    
    // Lambda 表达式
    fn parse_lambda(&mut self) -> Result<Expr, ParseError> {
        let mut params = Vec::new();
        
        if self.match_token(&[TokenType::LeftParen]) {
            // (a, b, c) -> expr
            while !self.check(&TokenType::RightParen) {
                let name = self.expect_identifier("期望参数名")?;
                params.push(name);
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
            self.consume(TokenType::RightParen, "期望 )")?;
        } else {
            // x -> expr
            let name = self.expect_identifier("期望参数名")?;
            params.push(name);
        }
        
        self.consume(TokenType::Arrow, "期望 ->")?;
        let body = self.parse_ternary()?; // Lambda 体是单个表达式
        
        Ok(Expr::Lambda {
            params,
            body: Box::new(body),
        })
    }
    
    fn is_lambda_params(&self) -> bool {
        let mut i = self.current;
        
        // (
        if i >= self.tokens.len() || !matches!(self.tokens[i].token_type, TokenType::LeftParen) {
            return false;
        }
        i += 1;
        
        // identifier (, identifier)* )
        loop {
            if i >= self.tokens.len() {
                return false;
            }
            
            if matches!(self.tokens[i].token_type, TokenType::RightParen) {
                i += 1;
                break;
            }
            
            if !matches!(self.tokens[i].token_type, TokenType::Identifier(_)) {
                return false;
            }
            i += 1;
            
            if i < self.tokens.len() && matches!(self.tokens[i].token_type, TokenType::Comma) {
                i += 1;
            }
        }
        
        // ->
        i < self.tokens.len() && matches!(self.tokens[i].token_type, TokenType::Arrow)
    }
    
    // ========== 工具方法 ==========
    
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type, TokenType::Eof)
    }
    
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.tokens[self.current - 1].clone()
    }
    
    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(token_type)
    }
    
    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }
    
    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, ParseError> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            Err(self.error(message))
        }
    }
    
    fn expect_identifier(&mut self, message: &str) -> Result<String, ParseError> {
        let token = self.advance();
        if let TokenType::Identifier(name) = token.token_type {
            Ok(name)
        } else {
            Err(self.error(message))
        }
    }
    
    fn skip_newlines(&mut self) {
        while self.match_token(&[TokenType::Newline]) {
            // skip
        }
    }
    
    fn consume_newlines(&mut self) -> Result<(), ParseError> {
        if !self.match_token(&[TokenType::Newline]) {
            return Err(self.error("期望换行"));
        }
        self.skip_newlines();
        Ok(())
    }
    
    fn error(&self, message: impl Into<String>) -> ParseError {
        let token = self.peek();
        ParseError {
            message: message.into(),
            line: token.line,
            column: token.column,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    
    #[test]
    fn test_parse_expression() {
        let source = "ma5 + ma10 * 2";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        
        let expr = parser.parse_expression().unwrap();
        assert!(matches!(expr, Expr::Binary { .. }));
    }
    
    #[test]
    fn test_parse_data_script() {
        let source = r#"
-- INPUT code:string, close:number --
-- OUTPUT code:string, ma5:number --

ma5 = MA(close, 5)
return [code, ma5]
"#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        
        let script = parser.parse().unwrap();
        if let Script::DataScript { input, output, body, .. } = script {
            assert_eq!(input.len(), 2);
            assert_eq!(output.len(), 2);
            assert_eq!(body.len(), 2);
        } else {
            panic!("Expected DataScript");
        }
    }
    
    #[test]
    fn test_parse_lambda() {
        let source = "map([1,2,3], x -> x * 2)";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        
        let expr = parser.parse_expression().unwrap();
        if let Expr::Call { callee, args } = expr {
            assert_eq!(callee, "map");
            assert_eq!(args.len(), 2);
            assert!(matches!(args[1], Expr::Lambda { .. }));
        } else {
            panic!("Expected Call");
        }
    }
}
