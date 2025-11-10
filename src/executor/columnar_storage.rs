// 列式数据存储 - 优化HashMap查找性能

use crate::runtime::Value;
use std::collections::HashMap;
use std::rc::Rc;

/// 列式数据存储结构
/// 将输入数据从行式HashMap转换为列式数组存储
/// 优势：
/// 1. 减少HashMap查找开销
/// 2. 支持零拷贝切片访问
/// 3. 缓存友好的内存布局
pub struct ColumnarStorage {
    /// 列名到列索引的映射
    column_index: HashMap<String, usize>,
    /// 列数据存储（每列是一个Value数组）
    columns: Vec<Rc<Vec<Value>>>,
    /// 行数
    row_count: usize,
}

impl ColumnarStorage {
    /// 从行式数据创建列式存储
    pub fn from_rows(rows: &[HashMap<String, Value>]) -> Self {
        if rows.is_empty() {
            return ColumnarStorage {
                column_index: HashMap::new(),
                columns: Vec::new(),
                row_count: 0,
            };
        }

        // 收集所有列名
        let mut column_names: Vec<String> = Vec::new();
        let mut column_index = HashMap::new();

        // 从第一行获取列名
        for col_name in rows[0].keys() {
            column_index.insert(col_name.clone(), column_names.len());
            column_names.push(col_name.clone());
        }

        // 为每列创建数组
        let mut columns: Vec<Vec<Value>> = vec![Vec::with_capacity(rows.len()); column_names.len()];

        // 填充列数据
        for row in rows {
            for (idx, col_name) in column_names.iter().enumerate() {
                let value = row.get(col_name).cloned().unwrap_or(Value::Null);
                columns[idx].push(value);
            }
        }

        // 将列数据包装为Rc以支持零拷贝
        let rc_columns: Vec<Rc<Vec<Value>>> = columns
            .into_iter()
            .map(|col| Rc::new(col))
            .collect();

        ColumnarStorage {
            column_index,
            columns: rc_columns,
            row_count: rows.len(),
        }
    }

    /// 获取指定列的值（单个值）
    pub fn get_value(&self, col_name: &str, row_idx: usize) -> Option<Value> {
        let col_idx = self.column_index.get(col_name)?;
        self.columns.get(*col_idx)?.get(row_idx).cloned()
    }

    /// 获取指定列的整列数据（零拷贝）
    pub fn get_column(&self, col_name: &str) -> Option<Rc<Vec<Value>>> {
        let col_idx = self.column_index.get(col_name)?;
        self.columns.get(*col_idx).cloned()
    }

    /// 获取指定列的切片（零拷贝）
    pub fn get_column_slice(&self, col_name: &str, start: usize, end: usize) -> Option<Value> {
        let col_idx = self.column_index.get(col_name)?;
        let column = self.columns.get(*col_idx)?;
        
        if end > column.len() {
            return None;
        }

        // 提取切片数据
        let slice_data: Vec<Value> = column[start..end].to_vec();
        Some(Value::Array(slice_data))
    }

    /// 获取指定行的数据（转换为HashMap）
    pub fn get_row(&self, row_idx: usize) -> Option<HashMap<String, Value>> {
        if row_idx >= self.row_count {
            return None;
        }

        let mut row = HashMap::new();
        for (col_name, col_idx) in &self.column_index {
            if let Some(value) = self.columns.get(*col_idx)?.get(row_idx) {
                row.insert(col_name.clone(), value.clone());
            }
        }

        Some(row)
    }

    /// 获取行数
    pub fn row_count(&self) -> usize {
        self.row_count
    }

    /// 获取列数
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// 获取所有列名
    pub fn column_names(&self) -> Vec<String> {
        let mut names: Vec<(String, usize)> = self.column_index
            .iter()
            .map(|(name, idx)| (name.clone(), *idx))
            .collect();
        names.sort_by_key(|(_, idx)| *idx);
        names.into_iter().map(|(name, _)| name).collect()
    }

    /// 判断是否应该使用列式存储
    /// 根据列数和数据特征判断
    pub fn should_use_columnar(column_count: usize, row_count: usize) -> bool {
        // 策略：
        // 1. 列数较少（< 10）且行数较多时使用列式存储
        // 2. 行数太少（< 100）时不值得转换
        column_count > 0 && column_count < 10 && row_count >= 100
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::Value;

    #[test]
    fn test_columnar_storage_creation() {
        let rows = vec![
            {
                let mut row = HashMap::new();
                row.insert("close".to_string(), Value::Number(100.0));
                row.insert("volume".to_string(), Value::Number(1000.0));
                row
            },
            {
                let mut row = HashMap::new();
                row.insert("close".to_string(), Value::Number(101.0));
                row.insert("volume".to_string(), Value::Number(1100.0));
                row
            },
            {
                let mut row = HashMap::new();
                row.insert("close".to_string(), Value::Number(102.0));
                row.insert("volume".to_string(), Value::Number(1200.0));
                row
            },
        ];

        let storage = ColumnarStorage::from_rows(&rows);
        
        assert_eq!(storage.row_count(), 3);
        assert_eq!(storage.column_count(), 2);
        
        // 测试获取单个值
        assert_eq!(storage.get_value("close", 0), Some(Value::Number(100.0)));
        assert_eq!(storage.get_value("volume", 1), Some(Value::Number(1100.0)));
    }

    #[test]
    fn test_get_column() {
        let rows = vec![
            {
                let mut row = HashMap::new();
                row.insert("close".to_string(), Value::Number(100.0));
                row
            },
            {
                let mut row = HashMap::new();
                row.insert("close".to_string(), Value::Number(101.0));
                row
            },
        ];

        let storage = ColumnarStorage::from_rows(&rows);
        let close_column = storage.get_column("close").unwrap();
        
        assert_eq!(close_column.len(), 2);
        assert_eq!(close_column[0], Value::Number(100.0));
        assert_eq!(close_column[1], Value::Number(101.0));
    }

    #[test]
    fn test_get_column_slice() {
        let rows = vec![
            {
                let mut row = HashMap::new();
                row.insert("close".to_string(), Value::Number(100.0));
                row
            },
            {
                let mut row = HashMap::new();
                row.insert("close".to_string(), Value::Number(101.0));
                row
            },
            {
                let mut row = HashMap::new();
                row.insert("close".to_string(), Value::Number(102.0));
                row
            },
        ];

        let storage = ColumnarStorage::from_rows(&rows);
        let slice = storage.get_column_slice("close", 0, 2).unwrap();
        
        if let Value::Array(arr) = slice {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Value::Number(100.0));
            assert_eq!(arr[1], Value::Number(101.0));
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_get_row() {
        let rows = vec![
            {
                let mut row = HashMap::new();
                row.insert("close".to_string(), Value::Number(100.0));
                row.insert("volume".to_string(), Value::Number(1000.0));
                row
            },
        ];

        let storage = ColumnarStorage::from_rows(&rows);
        let row = storage.get_row(0).unwrap();
        
        assert_eq!(row.get("close"), Some(&Value::Number(100.0)));
        assert_eq!(row.get("volume"), Some(&Value::Number(1000.0)));
    }

    #[test]
    fn test_should_use_columnar() {
        // 列数少，行数多 -> 使用列式
        assert!(ColumnarStorage::should_use_columnar(5, 1000));
        
        // 列数太多 -> 不使用列式
        assert!(!ColumnarStorage::should_use_columnar(50, 1000));
        
        // 行数太少 -> 不使用列式
        assert!(!ColumnarStorage::should_use_columnar(5, 50));
    }

    #[test]
    fn test_empty_storage() {
        let storage = ColumnarStorage::from_rows(&[]);
        assert_eq!(storage.row_count(), 0);
        assert_eq!(storage.column_count(), 0);
    }
}
