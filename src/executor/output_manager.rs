// 流式输出管理器 - 支持大规模数据输出

use crate::runtime::{Value, RuntimeError};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

/// 输出行类型
pub type OutputRow = HashMap<String, Value>;

/// 输出模式
#[derive(Clone, Debug)]
pub enum OutputMode {
    /// 内存模式：全量保存到内存中
    InMemory,
    /// 流式写入文件
    StreamToFile { 
        path: PathBuf,
        buffer_size: usize,
    },
    /// 回调模式：每行调用回调函数
    Callback,
}

/// 输出管理器配置
#[derive(Clone)]
pub struct OutputManagerConfig {
    /// 输出模式
    pub mode: OutputMode,
    /// 缓冲区大小（行数）
    pub buffer_size: usize,
    /// 刷新间隔（行数）
    pub flush_interval: usize,
}

impl Default for OutputManagerConfig {
    fn default() -> Self {
        OutputManagerConfig {
            mode: OutputMode::InMemory,
            buffer_size: 1000,
            flush_interval: 1000,
        }
    }
}

/// 流式输出管理器
pub struct OutputManager {
    /// 配置
    config: OutputManagerConfig,
    /// 内存缓冲区
    buffer: Vec<OutputRow>,
    /// 文件写入器
    file_writer: Option<BufWriter<File>>,
    /// 已写入行数
    written_count: usize,
    /// 列名顺序（用于CSV输出）
    column_order: Vec<String>,
}

impl OutputManager {
    /// 创建输出管理器
    pub fn new(config: OutputManagerConfig) -> Result<Self, RuntimeError> {
        let file_writer = match &config.mode {
            OutputMode::StreamToFile { path, buffer_size } => {
                let file = File::create(path)
                    .map_err(|e| RuntimeError::type_error(&format!("无法创建输出文件: {}", e)))?;
                Some(BufWriter::with_capacity(*buffer_size * 1024, file))
            }
            _ => None,
        };

        Ok(OutputManager {
            config,
            buffer: Vec::new(),
            file_writer,
            written_count: 0,
            column_order: Vec::new(),
        })
    }

    /// 使用默认配置创建（内存模式）
    pub fn with_default() -> Self {
        OutputManager {
            config: OutputManagerConfig::default(),
            buffer: Vec::new(),
            file_writer: None,
            written_count: 0,
            column_order: Vec::new(),
        }
    }

    /// 写入单行数据
    pub fn write_row(&mut self, row: OutputRow) -> Result<(), RuntimeError> {
        // 如果是第一行，确定列顺序
        if self.column_order.is_empty() && !row.is_empty() {
            self.column_order = row.keys().cloned().collect();
            self.column_order.sort(); // 保证顺序一致
        }

        match &self.config.mode {
            OutputMode::InMemory => {
                self.buffer.push(row);
            }
            OutputMode::StreamToFile { .. } => {
                // 先缓冲，达到阈值后批量写入
                self.buffer.push(row);
                
                if self.buffer.len() >= self.config.flush_interval {
                    self.flush_buffer()?;
                }
            }
            OutputMode::Callback => {
                // 回调模式暂不实现具体逻辑
                self.buffer.push(row);
            }
        }

        self.written_count += 1;
        Ok(())
    }

    /// 刷新缓冲区到文件
    fn flush_buffer(&mut self) -> Result<(), RuntimeError> {
        if let Some(ref mut writer) = self.file_writer {
            // 写入CSV格式
            for row in &self.buffer {
                let line = Self::format_row_as_csv_static(row, &self.column_order);
                writer.write_all(line.as_bytes())
                    .map_err(|e| RuntimeError::type_error(&format!("写入文件失败: {}", e)))?;
                writer.write_all(b"\n")
                    .map_err(|e| RuntimeError::type_error(&format!("写入文件失败: {}", e)))?;
            }
            
            writer.flush()
                .map_err(|e| RuntimeError::type_error(&format!("刷新文件失败: {}", e)))?;
        }

        self.buffer.clear();
        Ok(())
    }

    /// 将行格式化为CSV（静态方法）
    fn format_row_as_csv_static(row: &OutputRow, column_order: &[String]) -> String {
        let mut values = Vec::new();
        
        for col_name in column_order {
            let value_str = if let Some(value) = row.get(col_name) {
                Self::value_to_csv_string_static(value)
            } else {
                String::new()
            };
            values.push(value_str);
        }

        values.join(",")
    }

    /// 将Value转换为CSV字符串（静态方法）
    fn value_to_csv_string_static(value: &Value) -> String {
        match value {
            Value::Number(n) => n.to_string(),
            Value::String(s) => format!("\"{}\"", s.replace("\"", "\"\"")),
            Value::Bool(b) => b.to_string(),
            Value::Null => String::new(),
            Value::Decimal(d) => d.to_string(),
            Value::Array(_) => "\"[array]\"".to_string(),
            Value::ArraySlice { .. } => "\"[array_slice]\"".to_string(),
            Value::Function(_) => "\"[function]\"".to_string(),
            Value::Lambda { .. } => "\"[lambda]\"".to_string(),
        }
    }

    /// 完成输出并返回结果
    pub fn finalize(mut self) -> Result<Option<Vec<OutputRow>>, RuntimeError> {
        // 刷新剩余缓冲区
        if !self.buffer.is_empty() && self.file_writer.is_some() {
            self.flush_buffer()?;
        }

        // 根据模式返回结果
        match self.config.mode {
            OutputMode::InMemory => Ok(Some(self.buffer)),
            OutputMode::StreamToFile { .. } => {
                // 流式模式不返回数据
                Ok(None)
            }
            OutputMode::Callback => Ok(Some(self.buffer)),
        }
    }

    /// 获取已写入行数
    pub fn written_count(&self) -> usize {
        self.written_count
    }

    /// 获取当前缓冲区大小
    pub fn buffer_size(&self) -> usize {
        self.buffer.len()
    }

    /// 设置列顺序（用于控制CSV列顺序）
    pub fn set_column_order(&mut self, columns: Vec<String>) {
        self.column_order = columns;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Read;

    #[test]
    fn test_in_memory_mode() {
        let mut manager = OutputManager::with_default();
        
        let mut row1 = HashMap::new();
        row1.insert("close".to_string(), Value::Number(100.0));
        row1.insert("volume".to_string(), Value::Number(1000.0));
        
        let mut row2 = HashMap::new();
        row2.insert("close".to_string(), Value::Number(101.0));
        row2.insert("volume".to_string(), Value::Number(1100.0));
        
        manager.write_row(row1).unwrap();
        manager.write_row(row2).unwrap();
        
        assert_eq!(manager.written_count(), 2);
        assert_eq!(manager.buffer_size(), 2);
        
        let result = manager.finalize().unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_stream_to_file_mode() {
        let temp_path = PathBuf::from("test_output.csv");
        
        let config = OutputManagerConfig {
            mode: OutputMode::StreamToFile {
                path: temp_path.clone(),
                buffer_size: 8,
            },
            buffer_size: 1000,
            flush_interval: 2,
        };
        
        let mut manager = OutputManager::new(config).unwrap();
        
        let mut row1 = HashMap::new();
        row1.insert("close".to_string(), Value::Number(100.0));
        row1.insert("volume".to_string(), Value::Number(1000.0));
        
        let mut row2 = HashMap::new();
        row2.insert("close".to_string(), Value::Number(101.0));
        row2.insert("volume".to_string(), Value::Number(1100.0));
        
        manager.write_row(row1).unwrap();
        manager.write_row(row2).unwrap();
        
        assert_eq!(manager.written_count(), 2);
        
        manager.finalize().unwrap();
        
        // 验证文件已创建
        assert!(temp_path.exists());
        
        // 读取文件内容验证
        let mut file = File::open(&temp_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        
        assert!(contents.contains("100"));
        assert!(contents.contains("1000"));
        
        // 清理测试文件
        fs::remove_file(temp_path).ok();
    }

    #[test]
    fn test_value_to_csv_string() {
        assert_eq!(OutputManager::value_to_csv_string_static(&Value::Number(123.45)), "123.45");
        assert_eq!(OutputManager::value_to_csv_string_static(&Value::String("test".to_string())), "\"test\"");
        assert_eq!(OutputManager::value_to_csv_string_static(&Value::Bool(true)), "true");
        assert_eq!(OutputManager::value_to_csv_string_static(&Value::Null), "");
    }

    #[test]
    fn test_buffer_flush_threshold() {
        let temp_path = PathBuf::from("test_flush.csv");
        
        let config = OutputManagerConfig {
            mode: OutputMode::StreamToFile {
                path: temp_path.clone(),
                buffer_size: 8,
            },
            buffer_size: 1000,
            flush_interval: 3, // 每3行刷新一次
        };
        
        let mut manager = OutputManager::new(config).unwrap();
        
        // 写入5行数据
        for i in 0..5 {
            let mut row = HashMap::new();
            row.insert("value".to_string(), Value::Number(i as f64));
            manager.write_row(row).unwrap();
        }
        
        // 前3行应该已经刷新，缓冲区剩余2行
        assert_eq!(manager.buffer_size(), 2);
        
        manager.finalize().unwrap();
        
        // 清理测试文件
        fs::remove_file(temp_path).ok();
    }
}
