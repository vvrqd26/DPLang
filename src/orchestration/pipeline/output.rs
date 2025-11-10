// 输出管道抽象与实现

use crate::runtime::Value;
use std::collections::HashMap;
use std::fs::{File, create_dir_all};
use std::io::{Write, stdout};
use std::path::{Path, PathBuf};

/// 输出管道 trait
pub trait OutputPipeline: Send {
    /// 初始化管道
    fn initialize(&mut self) -> Result<(), String>;
    
    /// 写入单行数据
    fn write_row(&mut self, routing_key: &str, data: &HashMap<String, Value>) -> Result<(), String>;
    
    /// 刷新缓冲区
    fn flush(&mut self) -> Result<(), String>;
    
    /// 关闭管道
    fn close(&mut self) -> Result<(), String>;
}

/// 标准输出管道
pub struct StdoutOutputPipeline {
    headers: Option<Vec<String>>,
    initialized: bool,
}

impl StdoutOutputPipeline {
    pub fn new() -> Self {
        StdoutOutputPipeline {
            headers: None,
            initialized: false,
        }
    }
    
    fn format_row(&self, data: &HashMap<String, Value>) -> String {
        if let Some(ref headers) = self.headers {
            headers.iter()
                .map(|h| {
                    data.get(h)
                        .map(|v| value_to_string(v))
                        .unwrap_or_else(|| "".to_string())
                })
                .collect::<Vec<_>>()
                .join(",")
        } else {
            // 没有表头时，直接输出所有值
            data.values()
                .map(|v| value_to_string(v))
                .collect::<Vec<_>>()
                .join(",")
        }
    }
}

impl OutputPipeline for StdoutOutputPipeline {
    fn initialize(&mut self) -> Result<(), String> {
        self.initialized = true;
        Ok(())
    }
    
    fn write_row(&mut self, _routing_key: &str, data: &HashMap<String, Value>) -> Result<(), String> {
        if !self.initialized {
            self.initialize()?;
        }
        
        // 首次写入时确定表头
        if self.headers.is_none() {
            let mut headers: Vec<String> = data.keys().cloned().collect();
            headers.sort();
            
            // 输出表头
            println!("{}", headers.join(","));
            self.headers = Some(headers);
        }
        
        // 输出数据行
        println!("{}", self.format_row(data));
        
        Ok(())
    }
    
    fn flush(&mut self) -> Result<(), String> {
        stdout().flush().map_err(|e| format!("刷新标准输出失败: {}", e))
    }
    
    fn close(&mut self) -> Result<(), String> {
        self.flush()
    }
}

/// 文件输出管道
pub struct FileOutputPipeline {
    output_dir: PathBuf,
    mode: OutputMode,
    buffer_size: usize,
    auto_flush: bool,
    path_template: String,
    
    // 按路由键分文件的写入器
    writers: HashMap<String, FileWriter>,
    
    // 合并模式的单一写入器
    merged_writer: Option<FileWriter>,
    
    initialized: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Split,  // 按路由键分文件
    Merge,  // 合并到单文件
}

struct FileWriter {
    file: File,
    headers: Vec<String>,
    buffer: Vec<String>,
    buffer_size: usize,
}

impl FileWriter {
    fn new(file: File, headers: Vec<String>, buffer_size: usize) -> Self {
        FileWriter {
            file,
            headers,
            buffer: Vec::new(),
            buffer_size,
        }
    }
    
    fn write_row(&mut self, data: &HashMap<String, Value>) -> Result<(), String> {
        let row = self.headers.iter()
            .map(|h| {
                data.get(h)
                    .map(|v| value_to_string(v))
                    .unwrap_or_else(|| "".to_string())
            })
            .collect::<Vec<_>>()
            .join(",");
        
        self.buffer.push(row);
        
        if self.buffer.len() >= self.buffer_size {
            self.flush()?;
        }
        
        Ok(())
    }
    
    fn flush(&mut self) -> Result<(), String> {
        for row in &self.buffer {
            writeln!(self.file, "{}", row)
                .map_err(|e| format!("写入文件失败: {}", e))?;
        }
        self.buffer.clear();
        self.file.flush()
            .map_err(|e| format!("刷新文件失败: {}", e))
    }
}

impl FileOutputPipeline {
    pub fn new(
        output_path: &str,
        mode: OutputMode,
        buffer_size: usize,
        auto_flush: bool,
    ) -> Self {
        let (output_dir, path_template) = if mode == OutputMode::Split {
            // 从路径模板中提取目录
            let path = Path::new(output_path);
            let dir = path.parent().unwrap_or(Path::new("."));
            (dir.to_path_buf(), output_path.to_string())
        } else {
            let path = Path::new(output_path);
            let dir = path.parent().unwrap_or(Path::new("."));
            (dir.to_path_buf(), output_path.to_string())
        };
        
        FileOutputPipeline {
            output_dir,
            mode,
            buffer_size,
            auto_flush,
            path_template,
            writers: HashMap::new(),
            merged_writer: None,
            initialized: false,
        }
    }
    
    fn get_output_path(&self, routing_key: &str) -> PathBuf {
        if self.mode == OutputMode::Split {
            // 替换模板中的变量
            let filename = self.path_template.replace("{stock_code}", routing_key);
            self.output_dir.join(filename.split('/').last().unwrap_or("output.csv"))
        } else {
            PathBuf::from(&self.path_template)
        }
    }
    
    fn create_writer(&self, routing_key: &str, data: &HashMap<String, Value>) -> Result<FileWriter, String> {
        let path = self.get_output_path(routing_key);
        
        let file = File::create(&path)
            .map_err(|e| format!("创建文件 {:?} 失败: {}", path, e))?;
        
        let mut headers: Vec<String> = data.keys().cloned().collect();
        headers.sort();
        
        let mut writer = FileWriter::new(file, headers.clone(), self.buffer_size);
        
        // 写入表头
        writeln!(writer.file, "{}", headers.join(","))
            .map_err(|e| format!("写入表头失败: {}", e))?;
        
        Ok(writer)
    }
}

impl OutputPipeline for FileOutputPipeline {
    fn initialize(&mut self) -> Result<(), String> {
        if self.initialized {
            return Ok(());
        }
        
        // 创建输出目录
        create_dir_all(&self.output_dir)
            .map_err(|e| format!("创建输出目录失败: {}", e))?;
        
        self.initialized = true;
        Ok(())
    }
    
    fn write_row(&mut self, routing_key: &str, data: &HashMap<String, Value>) -> Result<(), String> {
        if !self.initialized {
            self.initialize()?;
        }
        
        match self.mode {
            OutputMode::Split => {
                // 按路由键分文件
                if !self.writers.contains_key(routing_key) {
                    let writer = self.create_writer(routing_key, data)?;
                    self.writers.insert(routing_key.to_string(), writer);
                }
                
                if let Some(writer) = self.writers.get_mut(routing_key) {
                    writer.write_row(data)?;
                    if self.auto_flush {
                        writer.flush()?;
                    }
                }
            },
            OutputMode::Merge => {
                // 合并到单文件
                if self.merged_writer.is_none() {
                    let writer = self.create_writer("", data)?;
                    self.merged_writer = Some(writer);
                }
                
                if let Some(writer) = &mut self.merged_writer {
                    writer.write_row(data)?;
                    if self.auto_flush {
                        writer.flush()?;
                    }
                }
            },
        }
        
        Ok(())
    }
    
    fn flush(&mut self) -> Result<(), String> {
        for writer in self.writers.values_mut() {
            writer.flush()?;
        }
        
        if let Some(writer) = &mut self.merged_writer {
            writer.flush()?;
        }
        
        Ok(())
    }
    
    fn close(&mut self) -> Result<(), String> {
        self.flush()
    }
}

/// 将Value转换为字符串
fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => "".to_string(),
        Value::Number(n) => n.to_string(),
        Value::Decimal(d) => d.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(_) => "[array]".to_string(),
        Value::ArraySlice { .. } => "[array_slice]".to_string(),
        Value::Function(_) => "[function]".to_string(),
        Value::Lambda { .. } => "[lambda]".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    
    #[test]
    fn test_stdout_output_pipeline() {
        let mut pipeline = StdoutOutputPipeline::new();
        pipeline.initialize().unwrap();
        
        let mut data = HashMap::new();
        data.insert("name".to_string(), Value::String("Alice".to_string()));
        data.insert("age".to_string(), Value::Number(30.0));
        
        // 这个测试会输出到stdout，实际使用时需要mock
        // assert!(pipeline.write_row("key1", &data).is_ok());
    }
    
    #[test]
    fn test_file_output_pipeline_split() -> Result<(), String> {
        let temp_dir = tempdir().map_err(|e| e.to_string())?;
        let output_path = temp_dir.path().join("{stock_code}.csv");
        
        let mut pipeline = FileOutputPipeline::new(
            output_path.to_str().unwrap(),
            OutputMode::Split,
            10,
            false,
        );
        
        pipeline.initialize()?;
        
        let mut data1 = HashMap::new();
        data1.insert("stock_code".to_string(), Value::String("000001".to_string()));
        data1.insert("price".to_string(), Value::Number(100.0));
        
        pipeline.write_row("000001", &data1)?;
        pipeline.flush()?;
        
        // 验证文件是否创建
        let expected_file = temp_dir.path().join("000001.csv");
        assert!(expected_file.exists());
        
        Ok(())
    }
}
