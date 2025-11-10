// DPLang 词法分析器

use std::fmt;

/// Token 类型定义
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // 关键字
    Return,
    If,
    Elif,
    Else,
    Package,
    Mut,
    Null,
    True,
    False,
    
    // 特殊标记
    Input(String),      // -- INPUT ... -- （内容）
    Output(String),     // -- OUTPUT ... --
    Import(String),     // -- IMPORT ... -- （包列表）
    Error,      // -- ERROR --
    ErrorEnd,   // -- ERROR_END --
    Precision(String),  // -- PRECISION ... --
    
    // 标识符和字面量
    Identifier(String),
    Number(f64),
    String(String),
    
    // 运算符
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Percent,    // %
    Caret,      // ^
    
    // 比较运算符
    Greater,    // >
    Less,       // <
    GreaterEq,  // >=
    LessEq,     // <=
    Equal,      // ==
    NotEqual,   // !=
    
    // 逻辑运算符
    And,        // and
    Or,         // or
    Not,        // not
    
    // 赋值和箭头
    Assign,     // =
    Arrow,      // ->
    Pipeline,   // |>
    
    // 括号和分隔符
    LeftParen,    // (
    RightParen,   // )
    LeftBracket,  // [
    RightBracket, // ]
    LeftBrace,    // {
    RightBrace,   // }
    Comma,        // ,
    Colon,        // :
    Question,     // ?
    Spread,       // ...
    Dot,          // .
    Underscore,   // _
    
    // 特殊
    Newline,
    Indent,
    Dedent,
    Eof,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenType::Identifier(s) => write!(f, "Identifier({})", s),
            TokenType::Number(n) => write!(f, "Number({})", n),
            TokenType::String(s) => write!(f, "String(\"{}\")", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

/// Token 结构
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize, column: usize) -> Self {
        Token { token_type, lexeme, line, column }
    }
}

/// 词法错误
#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "词法错误 [{}:{}]: {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for LexError {}

/// 词法分析器
pub struct Lexer {
    source: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
    indent_stack: Vec<usize>,  // 缩进栈
    pending_tokens: Vec<Token>, // 待发送的 token
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            source: source.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
            indent_stack: vec![0],
            pending_tokens: Vec::new(),
        }
    }
    
    /// 获取所有 tokens
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token.token_type, TokenType::Eof);
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        
        Ok(tokens)
    }
    
    /// 获取下一个 token
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        // 先返回待发送的 token
        if !self.pending_tokens.is_empty() {
            return Ok(self.pending_tokens.remove(0));
        }
        
        self.skip_whitespace_except_newline();
        
        if self.is_at_end() {
            // 处理文件结束时的 dedent
            self.handle_eof_dedents();
            if !self.pending_tokens.is_empty() {
                return Ok(self.pending_tokens.remove(0));
            }
            return Ok(Token::new(TokenType::Eof, String::new(), self.line, self.column));
        }
        
        let ch = self.peek();
        let start_line = self.line;
        let start_column = self.column;
        
        // 处理换行
        if ch == '\n' {
            self.advance();
            let indent_tokens = self.handle_indent()?;
            if !indent_tokens.is_empty() {
                self.pending_tokens.extend(indent_tokens);
                return Ok(self.pending_tokens.remove(0));
            }
            return Ok(Token::new(TokenType::Newline, "\n".to_string(), start_line, start_column));
        }
        
        // 注释
        if ch == '#' {
            self.skip_comment();
            return self.next_token();
        }
        
        // 特殊声明 -- XXX --
        if ch == '-' && self.peek_ahead(1) == Some('-') {
            return self.scan_special_declaration();
        }
        
        // 数字
        if ch.is_ascii_digit() {
            return self.scan_number();
        }
        
        // 字符串
        if ch == '"' || ch == '\'' {
            return self.scan_string(ch);
        }
        
        // 标识符和关键字
        if Self::is_identifier_start(ch) {
            return self.scan_identifier();
        }
        
        // 运算符和符号
        self.scan_operator()
    }
    
    // ========== 辅助方法 ==========
    
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }
    
    fn peek_ahead(&self, n: usize) -> Option<char> {
        let pos = self.current + n;
        if pos < self.source.len() {
            Some(self.source[pos])
        } else {
            None
        }
    }
    
    fn advance(&mut self) -> char {
        let ch = self.source[self.current];
        self.current += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        ch
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
    
    fn skip_whitespace_except_newline(&mut self) {
        while !self.is_at_end() {
            let ch = self.peek();
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn skip_comment(&mut self) {
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }
    
    fn handle_indent(&mut self) -> Result<Vec<Token>, LexError> {
        let mut indent_level = 0;
        let start_line = self.line;
        
        // 计算缩进级别
        while !self.is_at_end() {
            let ch = self.peek();
            if ch == ' ' {
                indent_level += 1;
                self.advance();
            } else if ch == '\t' {
                indent_level += 4;  // tab = 4 spaces
                self.advance();
            } else {
                break;
            }
        }
        
        // 空行或注释行不影响缩进
        if self.is_at_end() || self.peek() == '\n' || self.peek() == '#' {
            return Ok(Vec::new());
        }
        
        let mut tokens = Vec::new();
        let current_indent = *self.indent_stack.last().unwrap();
        
        if indent_level > current_indent {
            // 缩进增加
            self.indent_stack.push(indent_level);
            tokens.push(Token::new(TokenType::Indent, " ".repeat(indent_level), start_line, 1));
        } else if indent_level < current_indent {
            // 缩进减少
            while let Some(&stack_indent) = self.indent_stack.last() {
                if stack_indent <= indent_level {
                    break;
                }
                self.indent_stack.pop();
                tokens.push(Token::new(TokenType::Dedent, String::new(), start_line, 1));
            }
            
            // 检查缩进对齐
            if self.indent_stack.last() != Some(&indent_level) {
                return Err(LexError {
                    message: "缩进不对齐".to_string(),
                    line: start_line,
                    column: 1,
                });
            }
        }
        
        Ok(tokens)
    }
    
    fn handle_eof_dedents(&mut self) {
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            self.pending_tokens.push(Token::new(TokenType::Dedent, String::new(), self.line, self.column));
        }
    }
    
    fn scan_number(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_column = self.column;
        let mut num_str = String::new();
        
        // 整数部分
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            num_str.push(self.advance());
        }
        
        // 小数部分
        if !self.is_at_end() && self.peek() == '.' && self.peek_ahead(1).map_or(false, |c| c.is_ascii_digit()) {
            num_str.push(self.advance()); // '.'
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                num_str.push(self.advance());
            }
        }
        
        // 科学计数法
        if !self.is_at_end() && (self.peek() == 'e' || self.peek() == 'E') {
            num_str.push(self.advance());
            if !self.is_at_end() && (self.peek() == '+' || self.peek() == '-') {
                num_str.push(self.advance());
            }
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                num_str.push(self.advance());
            }
        }
        
        match num_str.parse::<f64>() {
            Ok(n) => Ok(Token::new(TokenType::Number(n), num_str, start_line, start_column)),
            Err(_) => Err(LexError {
                message: format!("无效的数字: {}", num_str),
                line: start_line,
                column: start_column,
            }),
        }
    }
    
    fn scan_string(&mut self, quote: char) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_column = self.column;
        let mut value = String::new();
        
        self.advance(); // 跳过开始引号
        
        while !self.is_at_end() && self.peek() != quote {
            if self.peek() == '\\' {
                self.advance();
                if self.is_at_end() {
                    break;
                }
                let escaped = match self.peek() {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    '"' => '"',
                    '\'' => '\'',
                    _ => self.peek(),
                };
                value.push(escaped);
                self.advance();
            } else {
                value.push(self.advance());
            }
        }
        
        if self.is_at_end() {
            return Err(LexError {
                message: "未闭合的字符串".to_string(),
                line: start_line,
                column: start_column,
            });
        }
        
        self.advance(); // 跳过结束引号
        
        Ok(Token::new(TokenType::String(value.clone()), format!("{}{}{}", quote, value, quote), start_line, start_column))
    }
    
    fn scan_identifier(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_column = self.column;
        let mut ident = String::new();
        
        while !self.is_at_end() && Self::is_identifier_continue(self.peek()) {
            ident.push(self.advance());
        }
        
        let token_type = match ident.as_str() {
            "return" => TokenType::Return,
            "if" => TokenType::If,
            "elif" => TokenType::Elif,
            "else" => TokenType::Else,
            "package" => TokenType::Package,
            "mut" => TokenType::Mut,
            "null" => TokenType::Null,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "and" => TokenType::And,
            "or" => TokenType::Or,
            "not" => TokenType::Not,
            "_" => TokenType::Underscore,
            _ => TokenType::Identifier(ident.clone()),
        };
        
        Ok(Token::new(token_type, ident, start_line, start_column))
    }
    
    fn scan_special_declaration(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_column = self.column;
        let mut text = String::new();
        
        // 读取 -- XXX ... -- 部分
        while !self.is_at_end() && self.peek() != '\n' {
            let ch = self.peek();
            text.push(ch);
            self.advance();
            
            // 如果找到第二个 --，停止
            if text.len() >= 4 && text.ends_with("--") {
                break;
            }
        }
        
        let text_upper = text.to_uppercase();
        
        // 提取声明中的内容（在两个 -- 之间）
        let content = if let Some(start_pos) = text.find("--") {
            let end_pos = text.rfind("--").unwrap();
            if end_pos > start_pos + 2 {
                text[start_pos + 2..end_pos].trim().to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        
        let token_type = if text_upper.contains("INPUT") {
            // 移除 "INPUT" 前缀，只保留参数列表
            let param_content = content.trim_start_matches("INPUT")
                .trim_start_matches("input")
                .trim()
                .to_string();
            TokenType::Input(param_content)
        } else if text_upper.contains("OUTPUT") {
            // 移除 "OUTPUT" 前缀
            let param_content = content.trim_start_matches("OUTPUT")
                .trim_start_matches("output")
                .trim()
                .to_string();
            TokenType::Output(param_content)
        } else if text_upper.contains("IMPORT") {
            // 移除 "IMPORT" 前缀
            let import_content = content.trim_start_matches("IMPORT")
                .trim_start_matches("import")
                .trim()
                .to_string();
            TokenType::Import(import_content)
        } else if text_upper.contains("ERROR_END") {
            TokenType::ErrorEnd
        } else if text_upper.contains("ERROR") {
            TokenType::Error
        } else if text_upper.contains("PRECISION") {
            TokenType::Precision(content)
        } else {
            return Err(LexError {
                message: format!("未知的特殊声明: {}", text),
                line: start_line,
                column: start_column,
            });
        };
        
        Ok(Token::new(token_type, text, start_line, start_column))
    }
    
    fn scan_operator(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_column = self.column;
        let ch = self.advance();
        
        let token_type = match ch {
            '+' => TokenType::Plus,
            '-' => {
                if !self.is_at_end() && self.peek() == '>' {
                    self.advance();
                    TokenType::Arrow
                } else {
                    TokenType::Minus
                }
            },
            '*' => TokenType::Star,
            '/' => TokenType::Slash,
            '%' => TokenType::Percent,
            '^' => TokenType::Caret,
            '>' => {
                if !self.is_at_end() && self.peek() == '=' {
                    self.advance();
                    TokenType::GreaterEq
                } else {
                    TokenType::Greater
                }
            },
            '<' => {
                if !self.is_at_end() && self.peek() == '=' {
                    self.advance();
                    TokenType::LessEq
                } else {
                    TokenType::Less
                }
            },
            '=' => {
                if !self.is_at_end() && self.peek() == '=' {
                    self.advance();
                    TokenType::Equal
                } else {
                    TokenType::Assign
                }
            },
            '!' => {
                if !self.is_at_end() && self.peek() == '=' {
                    self.advance();
                    TokenType::NotEqual
                } else {
                    return Err(LexError {
                        message: format!("意外的字符: {}", ch),
                        line: start_line,
                        column: start_column,
                    });
                }
            },
            '|' => {
                if !self.is_at_end() && self.peek() == '>' {
                    self.advance();
                    TokenType::Pipeline
                } else {
                    return Err(LexError {
                        message: format!("意外的字符: {}", ch),
                        line: start_line,
                        column: start_column,
                    });
                }
            },
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '[' => TokenType::LeftBracket,
            ']' => TokenType::RightBracket,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            ',' => TokenType::Comma,
            ':' => TokenType::Colon,
            '?' => TokenType::Question,
            '.' => {
                if !self.is_at_end() && self.peek() == '.' && self.peek_ahead(1) == Some('.') {
                    self.advance();
                    self.advance();
                    TokenType::Spread
                } else {
                    TokenType::Dot
                }
            },
            _ => {
                return Err(LexError {
                    message: format!("意外的字符: {}", ch),
                    line: start_line,
                    column: start_column,
                });
            }
        };
        
        let lexeme = match &token_type {
            TokenType::GreaterEq => ">=".to_string(),
            TokenType::LessEq => "<=".to_string(),
            TokenType::Equal => "==".to_string(),
            TokenType::NotEqual => "!=".to_string(),
            TokenType::Arrow => "->".to_string(),
            TokenType::Pipeline => "|>".to_string(),
            TokenType::Spread => "...".to_string(),
            _ => ch.to_string(),
        };
        
        Ok(Token::new(token_type, lexeme, start_line, start_column))
    }
    
    fn is_identifier_start(ch: char) -> bool {
        ch.is_alphabetic() || ch == '_' || Self::is_chinese_char(ch)
    }
    
    fn is_identifier_continue(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_' || Self::is_chinese_char(ch)
    }
    
    fn is_chinese_char(ch: char) -> bool {
        matches!(ch, '\u{4E00}'..='\u{9FFF}')
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_tokens() {
        let source = "ma5 = MA(close, 5)";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        assert!(matches!(tokens[0].token_type, TokenType::Identifier(_)));
        assert!(matches!(tokens[1].token_type, TokenType::Assign));
    }
    
    #[test]
    fn test_chinese_identifier() {
        let source = "涨幅 = 100";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        if let TokenType::Identifier(name) = &tokens[0].token_type {
            assert_eq!(name, "涨幅");
        } else {
            panic!("Expected identifier");
        }
    }
    
    #[test]
    fn test_number() {
        let source = "123 45.67 1.23e-4";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        assert!(matches!(tokens[0].token_type, TokenType::Number(123.0)));
        assert!(matches!(tokens[1].token_type, TokenType::Number(45.67)));
    }
    
    #[test]
    fn test_operators() {
        let source = "+ - * / % ^ > < >= <= == != -> |>";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        assert!(matches!(tokens[0].token_type, TokenType::Plus));
        assert!(matches!(tokens[12].token_type, TokenType::Arrow));
        assert!(matches!(tokens[13].token_type, TokenType::Pipeline));
    }
}
