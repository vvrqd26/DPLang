// 配置文件加载与解析

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 全局配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GlobalConfig {
    #[serde(default = "default_window_size")]
    pub default_window_size: usize,
    #[serde(default = "default_pool_size")]
    pub default_pool_size: usize,
    #[serde(default = "default_pool_max_size")]
    pub default_pool_max_size: usize,
}

fn default_window_size() -> usize { 1000 }
fn default_pool_size() -> usize { 10 }
fn default_pool_max_size() -> usize { 100 }

impl Default for GlobalConfig {
    fn default() -> Self {
        GlobalConfig {
            default_window_size: 1000,
            default_pool_size: 10,
            default_pool_max_size: 100,
        }
    }
}

/// 任务配置文件
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TasksConfig {
    #[serde(default)]
    pub global: GlobalConfig,
    #[serde(default)]
    pub task: Vec<TaskConfig>,
}

/// 单个任务配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskConfig {
    pub id: String,
    pub name: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    pub scripts: Vec<String>,
    pub schema: SchemaConfig,
    pub routing: RoutingConfig,
    pub compute_pool: ComputePoolConfig,
    pub input: IOConfig,
    pub output: IOConfig,
    #[serde(default)]
    pub error_handling: ErrorHandlingConfig,
}

fn default_enabled() -> bool { true }

/// Schema配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SchemaConfig {
    pub columns: Vec<ColumnConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ColumnConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub col_type: String,
    #[serde(default)]
    pub required: bool,
}

/// 路由配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RoutingConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub key_column: Option<String>,
    #[serde(default = "default_strategy")]
    pub strategy: String,
}

fn default_strategy() -> String { "hash".to_string() }

/// 计算元池配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ComputePoolConfig {
    #[serde(default = "default_initial_size")]
    pub initial_size: usize,
    #[serde(default = "default_max_size")]
    pub max_size: usize,
    #[serde(default = "default_window_size")]
    pub window_size: usize,
    #[serde(default)]
    pub auto_scale: bool,
    #[serde(default = "default_scale_threshold")]
    pub scale_threshold: f64,
}

fn default_initial_size() -> usize { 5 }
fn default_max_size() -> usize { 20 }
fn default_scale_threshold() -> f64 { 0.8 }

/// I/O配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IOConfig {
    #[serde(rename = "type")]
    pub io_type: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    #[serde(default)]
    pub auto_flush: bool,
}

fn default_format() -> String { "csv".to_string() }
fn default_buffer_size() -> usize { 100 }

/// 错误处理配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErrorHandlingConfig {
    #[serde(default = "default_retry_count")]
    pub retry_count: usize,
    #[serde(default = "default_skip_on_error")]
    pub skip_on_error: bool,
    #[serde(default = "default_log_errors")]
    pub log_errors: bool,
}

fn default_retry_count() -> usize { 3 }
fn default_skip_on_error() -> bool { true }
fn default_log_errors() -> bool { true }

impl Default for ErrorHandlingConfig {
    fn default() -> Self {
        ErrorHandlingConfig {
            retry_count: 3,
            skip_on_error: true,
            log_errors: true,
        }
    }
}

/// 加载配置文件
pub fn load_config(path: &Path) -> Result<TasksConfig, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("无法读取配置文件: {}", e))?;
    
    let config: TasksConfig = toml::from_str(&content)
        .map_err(|e| format!("配置文件解析错误: {}", e))?;
    
    // 验证配置
    validate_config(&config)?;
    
    Ok(config)
}

/// 验证配置合法性
fn validate_config(config: &TasksConfig) -> Result<(), String> {
    for task in &config.task {
        // 检查任务ID不为空
        if task.id.is_empty() {
            return Err("任务ID不能为空".to_string());
        }
        
        // 检查脚本列表不为空
        if task.scripts.is_empty() {
            return Err(format!("任务 {} 的脚本列表不能为空", task.id));
        }
        
        // 检查路由配置
        if task.routing.enabled && task.routing.key_column.is_none() {
            return Err(format!("任务 {} 启用路由但未指定路由键字段", task.id));
        }
        
        // 检查计算元池大小
        if task.compute_pool.initial_size == 0 {
            return Err(format!("任务 {} 的初始池大小不能为0", task.id));
        }
        
        if task.compute_pool.max_size < task.compute_pool.initial_size {
            return Err(format!("任务 {} 的最大池大小不能小于初始大小", task.id));
        }
        
        // 检查I/O类型
        match task.input.io_type.as_str() {
            "stdin" | "file" | "tcp" => {},
            _ => return Err(format!("任务 {} 的输入类型不支持: {}", task.id, task.input.io_type)),
        }
        
        match task.output.io_type.as_str() {
            "stdout" | "file" | "tcp" => {},
            _ => return Err(format!("任务 {} 的输出类型不支持: {}", task.id, task.output.io_type)),
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = GlobalConfig::default();
        assert_eq!(config.default_window_size, 1000);
        assert_eq!(config.default_pool_size, 10);
    }
    
    #[test]
    fn test_validate_empty_task_id() {
        let config = TasksConfig {
            global: GlobalConfig::default(),
            task: vec![TaskConfig {
                id: "".to_string(),
                name: "test".to_string(),
                enabled: true,
                scripts: vec!["test.dp".to_string()],
                schema: SchemaConfig { columns: vec![] },
                routing: RoutingConfig {
                    enabled: false,
                    key_column: None,
                    strategy: "hash".to_string(),
                },
                compute_pool: ComputePoolConfig {
                    initial_size: 5,
                    max_size: 20,
                    window_size: 1000,
                    auto_scale: false,
                    scale_threshold: 0.8,
                },
                input: IOConfig {
                    io_type: "stdin".to_string(),
                    path: None,
                    host: None,
                    port: None,
                    format: "csv".to_string(),
                    mode: None,
                    buffer_size: 100,
                    auto_flush: false,
                },
                output: IOConfig {
                    io_type: "stdout".to_string(),
                    path: None,
                    host: None,
                    port: None,
                    format: "csv".to_string(),
                    mode: None,
                    buffer_size: 100,
                    auto_flush: false,
                },
                error_handling: ErrorHandlingConfig::default(),
            }],
        };
        
        assert!(validate_config(&config).is_err());
    }
}
