// 包加载器 - 从文件系统加载 DPLang 包

use crate::lexer::Lexer;
use crate::parser::{Parser, Script};
use crate::runtime::RuntimeError;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// 包加载器
pub struct PackageLoader {
    /// 包搜索路径
    search_paths: Vec<PathBuf>,
    /// 已加载的包缓存（包名 -> Script）
    cache: HashMap<String, Script>,
}

impl PackageLoader {
    /// 创建新的包加载器
    pub fn new() -> Self {
        let mut loader = PackageLoader {
            search_paths: Vec::new(),
            cache: HashMap::new(),
        };
        
        // 添加默认搜索路径
        loader.add_default_paths();
        
        loader
    }
    
    /// 添加默认搜索路径
    fn add_default_paths(&mut self) {
        // 1. 当前目录的 packages 子目录
        if let Ok(current_dir) = std::env::current_dir() {
            self.search_paths.push(current_dir.join("packages"));
        }
        
        // 2. 当前目录
        if let Ok(current_dir) = std::env::current_dir() {
            self.search_paths.push(current_dir);
        }
        
        // 3. stdlib 目录（标准库）
        if let Ok(current_dir) = std::env::current_dir() {
            self.search_paths.push(current_dir.join("stdlib"));
        }
    }
    
    /// 添加搜索路径
    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        self.search_paths.push(path.as_ref().to_path_buf());
    }
    
    /// 加载包（从文件系统或缓存）
    pub fn load_package(&mut self, name: &str) -> Result<Script, RuntimeError> {
        // 先检查缓存
        if let Some(script) = self.cache.get(name) {
            return Ok(script.clone());
        }
        
        // 查找包文件
        let package_file = self.find_package_file(name)?;
        
        // 从文件加载
        let script = self.load_from_file(&package_file)?;
        
        // 验证是否是包脚本
        if !matches!(script, Script::Package { .. }) {
            return Err(RuntimeError::type_error(&format!(
                "文件 {} 不是有效的包脚本",
                package_file.display()
            )));
        }
        
        // 缓存
        self.cache.insert(name.to_string(), script.clone());
        
        Ok(script)
    }
    
    /// 查找包文件
    fn find_package_file(&self, name: &str) -> Result<PathBuf, RuntimeError> {
        let filename = format!("{}.dp", name);
        
        // 在所有搜索路径中查找
        for search_path in &self.search_paths {
            let candidate = search_path.join(&filename);
            if candidate.exists() && candidate.is_file() {
                return Ok(candidate);
            }
        }
        
        Err(RuntimeError::type_error(&format!(
            "找不到包: {} (在搜索路径中查找 {})",
            name, filename
        )))
    }
    
    /// 从文件加载包
    fn load_from_file(&self, path: &Path) -> Result<Script, RuntimeError> {
        // 读取文件内容
        let source = fs::read_to_string(path).map_err(|e| {
            RuntimeError::type_error(&format!(
                "无法读取包文件 {}: {}",
                path.display(),
                e
            ))
        })?;
        
        // 词法分析
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize().map_err(|e| {
            RuntimeError::type_error(&format!(
                "包文件 {} 词法分析错误: {}",
                path.display(),
                e.message
            ))
        })?;
        
        // 语法分析
        let mut parser = Parser::new(tokens);
        let script = parser.parse().map_err(|e| {
            RuntimeError::type_error(&format!(
                "包文件 {} 语法分析错误: {}",
                path.display(),
                e.message
            ))
        })?;
        
        Ok(script)
    }
    
    /// 批量加载包
    pub fn load_packages(&mut self, names: &[String]) -> Result<HashMap<String, Script>, RuntimeError> {
        let mut packages = HashMap::new();
        
        for name in names {
            let script = self.load_package(name)?;
            packages.insert(name.clone(), script);
        }
        
        Ok(packages)
    }
    
    /// 清空缓存
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
    
    /// 获取搜索路径列表（用于调试）
    pub fn search_paths(&self) -> &[PathBuf] {
        &self.search_paths
    }
}

impl Default for PackageLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;
    
    #[test]
    fn test_package_loader_creation() {
        let loader = PackageLoader::new();
        assert!(!loader.search_paths.is_empty());
    }
    
    #[test]
    fn test_add_search_path() {
        let mut loader = PackageLoader::new();
        let initial_count = loader.search_paths.len();
        
        loader.add_search_path("/custom/path");
        assert_eq!(loader.search_paths.len(), initial_count + 1);
    }
    
    #[test]
    fn test_load_package_from_file() {
        // 创建临时目录
        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path().join("test_math.dp");
        
        // 创建测试包文件
        let package_source = r#"
package test_math

PI = 3.14159
E = 2.71828
"#;
        let mut file = fs::File::create(&package_path).unwrap();
        file.write_all(package_source.as_bytes()).unwrap();
        
        // 创建加载器并添加临时目录到搜索路径
        let mut loader = PackageLoader::new();
        loader.add_search_path(temp_dir.path());
        
        // 加载包
        let script = loader.load_package("test_math").unwrap();
        
        // 验证是包脚本
        assert!(matches!(script, Script::Package { .. }));
    }
    
    #[test]
    fn test_package_cache() {
        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path().join("cached.dp");
        
        let package_source = r#"
package cached

VALUE = 42
"#;
        let mut file = fs::File::create(&package_path).unwrap();
        file.write_all(package_source.as_bytes()).unwrap();
        
        let mut loader = PackageLoader::new();
        loader.add_search_path(temp_dir.path());
        
        // 第一次加载
        let script1 = loader.load_package("cached").unwrap();
        
        // 第二次加载（应该从缓存）
        let script2 = loader.load_package("cached").unwrap();
        
        // 两次应该是同一个脚本
        assert!(matches!(script1, Script::Package { .. }));
        assert!(matches!(script2, Script::Package { .. }));
    }
    
    #[test]
    fn test_load_nonexistent_package() {
        let mut loader = PackageLoader::new();
        
        // 尝试加载不存在的包
        let result = loader.load_package("nonexistent_package_12345");
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_batch_load_packages() {
        let temp_dir = TempDir::new().unwrap();
        
        // 创建多个包文件
        for name in &["pkg1", "pkg2"] {
            let path = temp_dir.path().join(format!("{}.dp", name));
            let source = format!("package {}\n\nVALUE = 1\n", name);
            let mut file = fs::File::create(&path).unwrap();
            file.write_all(source.as_bytes()).unwrap();
        }
        
        let mut loader = PackageLoader::new();
        loader.add_search_path(temp_dir.path());
        
        // 批量加载
        let packages = loader.load_packages(&["pkg1".to_string(), "pkg2".to_string()]).unwrap();
        
        assert_eq!(packages.len(), 2);
        assert!(packages.contains_key("pkg1"));
        assert!(packages.contains_key("pkg2"));
    }
}
