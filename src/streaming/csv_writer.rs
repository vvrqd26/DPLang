// CSV 流式写入器 - 带缓冲的实时输出

use crate::runtime::Value;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

/// CSV 输出模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CSVMode {
    /// 按股票分文件
    Split,
    /// 统一文件
    Unified,
}

/// CSV 流式写入器配置
#[derive(Debug, Clone)]
pub struct CSVWriterConfig {
    /// 输出目录
    pub output_dir: PathBuf,
    /// 输出模式
    pub mode: CSVMode,
    /// 缓冲区大小（行数）
    pub buffer_size: usize,
    /// 是否在每次写入后刷盘
    pub auto_flush: bool,
}

impl Default for CSVWriterConfig {
    fn default() -> Self {
        CSVWriterConfig {
            output_dir: PathBuf::from("./output"),
            mode: CSVMode::Split,
            buffer_size: 100,
            auto_flush: false,
        }
    }
}

/// CSV 流式写入器
pub struct CSVStreamWriter {
    config: CSVWriterConfig,
    /// 文件写入器缓存（股票代码 -> 写入器）
    writers: HashMap<String, BufWriter<File>>,
    /// 缓冲行数（股票代码 -> 行数）
    buffer_counts: HashMap<String, usize>,
    /// 已写入的列名（用于确保列顺序一致）
    headers: Option<Vec<String>>,
}

impl CSVStreamWriter {
    /// 创建 CSV 流式写入器
    pub fn new(config: CSVWriterConfig) -> std::io::Result<Self> {
        // 创建输出目录
        std::fs::create_dir_all(&config.output_dir)?;
        
        Ok(CSVStreamWriter {
            config,
            writers: HashMap::new(),
            buffer_counts: HashMap::new(),
            headers: None,
        })
    }
    
    /// 写入单条数据
    pub fn write_row(
        &mut self,
        stock_code: &str,
        data: &HashMap<String, Value>,
    ) -> std::io::Result<()> {
        // 初始化列名（第一次写入时）
        if self.headers.is_none() {
            let mut headers: Vec<String> = data.keys().cloned().collect();
            headers.sort();
            
            // 确保 stock_code 在第一列
            if !headers.contains(&"stock_code".to_string()) {
                headers.insert(0, "stock_code".to_string());
            } else {
                headers.retain(|h| h != "stock_code");
                headers.insert(0, "stock_code".to_string());
            }
            
            self.headers = Some(headers);
        }
        
        // 准备数据
        let headers_clone = self.headers.as_ref().unwrap().clone();
        let stock_code_owned = stock_code.to_string();
        
        // 构建 CSV 行数据
        let values: Vec<String> = headers_clone
            .iter()
            .map(|header| {
                if header == "stock_code" {
                    stock_code.to_string()
                } else {
                    data.get(header)
                        .map(|v| format_value_csv(v))
                        .unwrap_or_else(|| String::new())
                }
            })
            .collect();
        let csv_line = values.join(",");
        
        // 获取或创建写入器并写入
        let writer = self.get_or_create_writer(&stock_code_owned)?;
        writeln!(writer, "{}", csv_line)?;
        
        // 更新缓冲计数
        let count = self.buffer_counts.entry(stock_code_owned.clone()).or_insert(0);
        *count += 1;
        
        // 检查是否需要刷新
        let should_flush = self.config.auto_flush || *count >= self.config.buffer_size;
        
        if should_flush {
            if let Some(w) = self.writers.get_mut(&stock_code_owned) {
                w.flush()?;
            }
            self.buffer_counts.insert(stock_code_owned, 0);
        }
        
        Ok(())
    }
    
    /// 刷新所有缓冲
    pub fn flush_all(&mut self) -> std::io::Result<()> {
        for writer in self.writers.values_mut() {
            writer.flush()?;
        }
        self.buffer_counts.clear();
        Ok(())
    }
    
    /// 刷新指定股票的缓冲
    pub fn flush(&mut self, stock_code: &str) -> std::io::Result<()> {
        if let Some(writer) = self.writers.get_mut(stock_code) {
            writer.flush()?;
            self.buffer_counts.insert(stock_code.to_string(), 0);
        }
        Ok(())
    }
    
    /// 获取或创建写入器
    fn get_or_create_writer(&mut self, stock_code: &str) -> std::io::Result<&mut BufWriter<File>> {
        if !self.writers.contains_key(stock_code) {
            let file_path = self.get_file_path(stock_code);
            let file_exists = file_path.exists();
            
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)?;
            
            let mut writer = BufWriter::new(file);
            
            // 如果文件是新创建的，写入表头
            if !file_exists {
                if let Some(ref headers) = self.headers {
                    writeln!(writer, "{}", headers.join(","))?;
                }
            }
            
            self.writers.insert(stock_code.to_string(), writer);
        }
        
        Ok(self.writers.get_mut(stock_code).unwrap())
    }
    
    /// 获取输出文件路径
    fn get_file_path(&self, stock_code: &str) -> PathBuf {
        match self.config.mode {
            CSVMode::Split => {
                let filename = format!("output_{}.csv", stock_code);
                self.config.output_dir.join(filename)
            }
            CSVMode::Unified => {
                self.config.output_dir.join("output_all.csv")
            }
        }
    }
}

impl Drop for CSVStreamWriter {
    fn drop(&mut self) {
        let _ = self.flush_all();
    }
}

/// 将 Value 格式化为 CSV 字段
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
    use tempfile::tempdir;
    
    #[test]
    fn test_csv_writer_split_mode() {
        let dir = tempdir().unwrap();
        let config = CSVWriterConfig {
            output_dir: dir.path().to_path_buf(),
            mode: CSVMode::Split,
            buffer_size: 2,
            auto_flush: false,
        };
        
        let mut writer = CSVStreamWriter::new(config).unwrap();
        
        // 写入数据
        let mut data = HashMap::new();
        data.insert("close".to_string(), Value::Number(100.5));
        data.insert("volume".to_string(), Value::Number(1000.0));
        
        writer.write_row("000001", &data).unwrap();
        writer.write_row("000001", &data).unwrap();
        
        // 手动刷新
        writer.flush_all().unwrap();
        
        // 检查文件是否创建
        let file_path = dir.path().join("output_000001.csv");
        assert!(file_path.exists());
    }
}
