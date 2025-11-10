// 输入管道抽象与实现

use crate::runtime::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, stdin};
use csv::ReaderBuilder;

/// 输入管道 trait
pub trait InputPipeline: Send {
    /// 初始化管道
    fn initialize(&mut self) -> Result<(), String>;
    
    /// 读取下一条数据（阻塞）
    fn read_next(&mut self) -> Result<Option<HashMap<String, Value>>, String>;
    
    /// 关闭管道
    fn close(&mut self) -> Result<(), String>;
    
    /// 检查是否已结束
    fn is_finished(&self) -> bool;
}

/// 标准输入管道
pub struct StdinInputPipeline {
    headers: Vec<String>,
    initialized: bool,
    finished: bool,
}

impl StdinInputPipeline {
    pub fn new() -> Self {
        StdinInputPipeline {
            headers: Vec::new(),
            initialized: false,
            finished: false,
        }
    }
    
    fn parse_csv_line(&self, line: &str) -> Result<HashMap<String, Value>, String> {
        let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if values.len() != self.headers.len() {
            return Err(format!("列数不匹配: 期望 {}, 实际 {}", self.headers.len(), values.len()));
        }
        
        let mut result = HashMap::new();
        
        for (i, header) in self.headers.iter().enumerate() {
            let value_str = values[i];
            let value = parse_value(value_str);
            result.insert(header.clone(), value);
        }
        
        Ok(result)
    }
}

impl InputPipeline for StdinInputPipeline {
    fn initialize(&mut self) -> Result<(), String> {
        if self.initialized {
            return Ok(());
        }
        
        let stdin_handle = stdin();
        let mut lines = stdin_handle.lock().lines();
        
        // 读取表头
        if let Some(Ok(header_line)) = lines.next() {
            self.headers = header_line.split(',')
                .map(|s| s.trim().to_string())
                .collect();
        } else {
            return Err("无法读取CSV表头".to_string());
        }
        
        // 保存剩余行的迭代器 - 但这里有问题，因为无法跨线程传递
        // 改用简单的逐行读取
        
        self.initialized = true;
        Ok(())
    }
    
    fn read_next(&mut self) -> Result<Option<HashMap<String, Value>>, String> {
        if !self.initialized {
            self.initialize()?;
        }
        
        if self.finished {
            return Ok(None);
        }
        
        let stdin_handle = stdin();
        let mut line = String::new();
        
        match stdin_handle.lock().read_line(&mut line) {
            Ok(0) => {
                self.finished = true;
                Ok(None)
            },
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    return self.read_next(); // 跳过空行
                }
                self.parse_csv_line(line).map(Some)
            },
            Err(e) => Err(format!("读取输入失败: {}", e)),
        }
    }
    
    fn close(&mut self) -> Result<(), String> {
        self.finished = true;
        Ok(())
    }
    
    fn is_finished(&self) -> bool {
        self.finished
    }
}

/// 文件输入管道
pub struct FileInputPipeline {
    file_path: String,
    reader: Option<csv::Reader<File>>,
    headers: Vec<String>,
    initialized: bool,
    finished: bool,
}

impl FileInputPipeline {
    pub fn new(file_path: String) -> Self {
        FileInputPipeline {
            file_path,
            reader: None,
            headers: Vec::new(),
            initialized: false,
            finished: false,
        }
    }
}

impl InputPipeline for FileInputPipeline {
    fn initialize(&mut self) -> Result<(), String> {
        if self.initialized {
            return Ok(());
        }
        
        let file = File::open(&self.file_path)
            .map_err(|e| format!("无法打开文件 {}: {}", self.file_path, e))?;
        
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);
        
        // 读取表头
        self.headers = csv_reader.headers()
            .map_err(|e| format!("读取表头失败: {}", e))?
            .iter()
            .map(|s| s.to_string())
            .collect();
        
        self.reader = Some(csv_reader);
        self.initialized = true;
        Ok(())
    }
    
    fn read_next(&mut self) -> Result<Option<HashMap<String, Value>>, String> {
        if !self.initialized {
            self.initialize()?;
        }
        
        if self.finished {
            return Ok(None);
        }
        
        let reader = self.reader.as_mut().unwrap();
        let mut record = csv::StringRecord::new();
        
        match reader.read_record(&mut record) {
            Ok(true) => {
                let mut result = HashMap::new();
                
                for (i, header) in self.headers.iter().enumerate() {
                    if let Some(value_str) = record.get(i) {
                        let value = parse_value(value_str);
                        result.insert(header.clone(), value);
                    }
                }
                
                Ok(Some(result))
            },
            Ok(false) => {
                self.finished = true;
                Ok(None)
            },
            Err(e) => Err(format!("读取记录失败: {}", e)),
        }
    }
    
    fn close(&mut self) -> Result<(), String> {
        self.reader = None;
        self.finished = true;
        Ok(())
    }
    
    fn is_finished(&self) -> bool {
        self.finished
    }
}

/// 解析值字符串
fn parse_value(s: &str) -> Value {
    if s.is_empty() || s == "null" {
        Value::Null
    } else if let Ok(n) = s.parse::<f64>() {
        Value::Number(n)
    } else if s == "true" {
        Value::Bool(true)
    } else if s == "false" {
        Value::Bool(false)
    } else {
        Value::String(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_file_input_pipeline() -> Result<(), String> {
        // 创建临时CSV文件
        let mut temp_file = NamedTempFile::new().map_err(|e| e.to_string())?;
        writeln!(temp_file, "name,age,active").map_err(|e| e.to_string())?;
        writeln!(temp_file, "Alice,30,true").map_err(|e| e.to_string())?;
        writeln!(temp_file, "Bob,25,false").map_err(|e| e.to_string())?;
        temp_file.flush().map_err(|e| e.to_string())?;
        
        let file_path = temp_file.path().to_str().unwrap().to_string();
        let mut pipeline = FileInputPipeline::new(file_path);
        
        pipeline.initialize()?;
        
        // 读取第一行
        let row1 = pipeline.read_next()?.unwrap();
        assert_eq!(row1.get("name"), Some(&Value::String("Alice".to_string())));
        assert_eq!(row1.get("age"), Some(&Value::Number(30.0)));
        assert_eq!(row1.get("active"), Some(&Value::Bool(true)));
        
        // 读取第二行
        let row2 = pipeline.read_next()?.unwrap();
        assert_eq!(row2.get("name"), Some(&Value::String("Bob".to_string())));
        
        // 读取完毕
        assert!(pipeline.read_next()?.is_none());
        assert!(pipeline.is_finished());
        
        Ok(())
    }
    
    #[test]
    fn test_parse_value() {
        assert_eq!(parse_value("123"), Value::Number(123.0));
        assert_eq!(parse_value("true"), Value::Bool(true));
        assert_eq!(parse_value("false"), Value::Bool(false));
        assert_eq!(parse_value("null"), Value::Null);
        assert_eq!(parse_value("hello"), Value::String("hello".to_string()));
    }
}
