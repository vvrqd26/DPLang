// DPLang 公共 API - 供其他程序调用

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::executor::DataStreamExecutor;
use crate::runtime::Value;
use std::collections::HashMap;

/// DPLang 解释器
pub struct DPLangInterpreter {
    source: String,
}

impl DPLangInterpreter {
    /// 创建新的解释器实例
    pub fn new(source: &str) -> Self {
        DPLangInterpreter {
            source: source.to_string(),
        }
    }
    
    /// 从文件创建解释器实例
    pub fn from_file(file_path: &str) -> Result<Self, String> {
        let source = std::fs::read_to_string(file_path)
            .map_err(|e| format!("无法读取文件: {}", e))?;
        Ok(DPLangInterpreter { source })
    }
    
    /// 执行脚本，返回结果
    pub fn execute(&self, input_data: Vec<HashMap<String, Value>>) -> Result<Vec<HashMap<String, Value>>, String> {
        // 词法分析
        let mut lexer = Lexer::new(&self.source);
        let tokens = lexer.tokenize()
            .map_err(|e| format!("词法分析错误: {:?}", e))?;
        
        // 语法分析
        let mut parser = Parser::new(tokens);
        let script = parser.parse()
            .map_err(|e| format!("语法分析错误: {:?}", e))?;
        
        // 执行
        let mut executor = DataStreamExecutor::new(script, input_data);
        executor.execute_all()
            .map_err(|e| format!("执行错误: {:?}", e))
    }
    
    /// 执行脚本（JSON 输入格式）
    pub fn execute_json(&self, json_input: &str) -> Result<String, String> {
        let input_data = parse_json_array(json_input)?;
        let output = self.execute(input_data)?;
        Ok(format_output_json(&output))
    }
    
    /// 执行脚本（CSV 输入格式）
    pub fn execute_csv(&self, csv_input: &str) -> Result<String, String> {
        let input_data = parse_csv(csv_input)?;
        let output = self.execute(input_data)?;
        Ok(format_output_csv(&output))
    }
}

/// 解析 JSON 数组输入
fn parse_json_array(json_str: &str) -> Result<Vec<HashMap<String, Value>>, String> {
    // 简化版 JSON 解析，仅支持对象数组
    let json_str = json_str.trim();
    if !json_str.starts_with('[') || !json_str.ends_with(']') {
        return Err("JSON 必须是数组格式".to_string());
    }
    
    let mut result = Vec::new();
    let content = &json_str[1..json_str.len()-1];
    
    // 分割对象（简化处理）
    let mut obj_start = 0;
    let mut brace_count = 0;
    let chars: Vec<char> = content.chars().collect();
    
    for (i, &ch) in chars.iter().enumerate() {
        match ch {
            '{' => {
                if brace_count == 0 {
                    obj_start = i;
                }
                brace_count += 1;
            }
            '}' => {
                brace_count -= 1;
                if brace_count == 0 {
                    let obj_str: String = chars[obj_start..=i].iter().collect();
                    if let Ok(obj) = parse_json_object(&obj_str) {
                        result.push(obj);
                    }
                }
            }
            _ => {}
        }
    }
    
    if result.is_empty() {
        result.push(HashMap::new());
    }
    
    Ok(result)
}

/// 解析单个 JSON 对象
fn parse_json_object(json_str: &str) -> Result<HashMap<String, Value>, String> {
    let json_str = json_str.trim();
    if !json_str.starts_with('{') || !json_str.ends_with('}') {
        return Err("格式错误".to_string());
    }
    
    let mut result = HashMap::new();
    let content = &json_str[1..json_str.len()-1];
    
    for pair in content.split(',') {
        let parts: Vec<&str> = pair.split(':').collect();
        if parts.len() != 2 {
            continue;
        }
        
        let key = parts[0].trim().trim_matches('"').to_string();
        let value_str = parts[1].trim();
        
        let value = if value_str.starts_with('"') && value_str.ends_with('"') {
            Value::String(value_str.trim_matches('"').to_string())
        } else if let Ok(n) = value_str.parse::<f64>() {
            Value::Number(n)
        } else if value_str == "true" {
            Value::Bool(true)
        } else if value_str == "false" {
            Value::Bool(false)
        } else if value_str == "null" {
            Value::Null
        } else {
            continue;
        };
        
        result.insert(key, value);
    }
    
    Ok(result)
}

/// 解析 CSV 输入
pub fn parse_csv(csv_str: &str) -> Result<Vec<HashMap<String, Value>>, String> {
    let lines: Vec<&str> = csv_str.trim().lines().collect();
    if lines.is_empty() {
        return Ok(vec![HashMap::new()]);
    }
    
    // 第一行是表头
    let headers: Vec<&str> = lines[0].split(',').map(|s| s.trim()).collect();
    
    let mut result = Vec::new();
    
    // 解析数据行
    for line in &lines[1..] {
        let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        let mut row = HashMap::new();
        
        for (i, &header) in headers.iter().enumerate() {
            if i >= values.len() {
                break;
            }
            
            let value_str = values[i];
            let value = if let Ok(n) = value_str.parse::<f64>() {
                Value::Number(n)
            } else if value_str == "true" {
                Value::Bool(true)
            } else if value_str == "false" {
                Value::Bool(false)
            } else if value_str.is_empty() || value_str == "null" {
                Value::Null
            } else {
                Value::String(value_str.to_string())
            };
            
            row.insert(header.to_string(), value);
        }
        
        result.push(row);
    }
    
    if result.is_empty() {
        result.push(HashMap::new());
    }
    
    Ok(result)
}

/// 格式化输出为 JSON
fn format_output_json(output: &[HashMap<String, Value>]) -> String {
    let mut result = String::from("[\n");
    
    for (i, row) in output.iter().enumerate() {
        if i > 0 {
            result.push_str(",\n");
        }
        result.push_str("  {");
        
        let mut first = true;
        for (key, value) in row {
            if !first {
                result.push_str(", ");
            }
            first = false;
            
            result.push_str(&format!("\"{}\": {}", key, format_value_json(value)));
        }
        
        result.push_str("}");
    }
    
    result.push_str("\n]");
    result
}

/// 格式化单个值为 JSON
fn format_value_json(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Decimal(d) => d.to_string(),
        Value::String(s) => format!("\"{}\"", s),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_value_json).collect();
            format!("[{}]", items.join(", "))
        }
        Value::ArraySlice { column_data, start, len } => {
            let items: Vec<String> = (0..*len)
                .filter_map(|i| column_data.get(*start + i))
                .map(format_value_json)
                .collect();
            format!("[{}]", items.join(", "))
        }
        Value::Lambda { .. } => "\"<lambda>\"".to_string(),
        Value::Function(_) => "\"<function>\"".to_string(),
    }
}

/// 格式化输出为 CSV
pub fn format_output_csv(output: &[HashMap<String, Value>]) -> String {
    if output.is_empty() {
        return String::new();
    }
    
    // 收集所有列名
    let mut headers = Vec::new();
    for row in output {
        for key in row.keys() {
            if !headers.contains(key) {
                headers.push(key.clone());
            }
        }
    }
    headers.sort();
    
    let mut result = String::new();
    
    // 写入表头
    result.push_str(&headers.join(","));
    result.push('\n');
    
    // 写入数据行
    for row in output {
        let values: Vec<String> = headers
            .iter()
            .map(|h| {
                row.get(h)
                    .map(format_value_csv)
                    .unwrap_or_else(|| String::new())
            })
            .collect();
        
        result.push_str(&values.join(","));
        result.push('\n');
    }
    
    result
}

/// 格式化单个值为 CSV
fn format_value_csv(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Decimal(d) => d.to_string(),
        Value::String(s) => {
            // 如果包含逗号或引号，需要转义
            if s.contains(',') || s.contains('"') {
                format!("\"{}\"", s.replace('"', "\"\""))
            } else {
                s.clone()
            }
        }
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_value_csv).collect();
            format!("\"[{}]\"", items.join("; "))
        }
        Value::ArraySlice { column_data, start, len } => {
            let items: Vec<String> = (0..*len)
                .filter_map(|i| column_data.get(*start + i))
                .map(format_value_csv)
                .collect();
            format!("\"[{}]\"", items.join("; "))
        }
        Value::Lambda { .. } => "<lambda>".to_string(),
        Value::Function(_) => "<function>".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv() {
        let csv = "name,age,price\nAlice,30,100.5\nBob,25,200.0";
        let result = parse_csv(csv).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].get("name"), Some(&Value::String("Alice".to_string())));
        assert_eq!(result[0].get("age"), Some(&Value::Number(30.0)));
    }

    #[test]
    fn test_format_output_csv() {
        let mut row1 = HashMap::new();
        row1.insert("name".to_string(), Value::String("Alice".to_string()));
        row1.insert("score".to_string(), Value::Number(95.5));
        
        let output = vec![row1];
        let csv = format_output_csv(&output);
        
        assert!(csv.contains("name,score"));
        assert!(csv.contains("Alice,95.5"));
    }

    #[test]
    fn test_interpreter_api() {
        let source = r#"
-- INPUT x:number --
-- OUTPUT result:number --

result = x * 2
return [result]
"#;
        
        let interpreter = DPLangInterpreter::new(source);
        
        let mut input = HashMap::new();
        input.insert("x".to_string(), Value::Number(5.0));
        
        let output = interpreter.execute(vec![input]).unwrap();
        
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].get("result"), Some(&Value::Number(10.0)));
    }
}
