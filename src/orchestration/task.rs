// 任务定义与状态管理

use crate::orchestration::config::TaskConfig;
use crate::orchestration::compute_pool::ComputePool;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Created,   // 已创建，配置待验证
    Ready,     // 配置验证通过，等待启动
    Running,   // 任务运行中
    Paused,    // 任务暂停
    Stopped,   // 任务已停止
    Error,     // 任务异常
}

impl TaskStatus {
    pub fn as_str(&self) -> &str {
        match self {
            TaskStatus::Created => "created",
            TaskStatus::Ready => "ready",
            TaskStatus::Running => "running",
            TaskStatus::Paused => "paused",
            TaskStatus::Stopped => "stopped",
            TaskStatus::Error => "error",
        }
    }
}

/// 任务统计信息
#[derive(Debug, Clone)]
pub struct TaskStats {
    pub processed_count: u64,
    pub error_count: u64,
    pub created_at: SystemTime,
    pub started_at: Option<SystemTime>,
    pub stopped_at: Option<SystemTime>,
}

impl Default for TaskStats {
    fn default() -> Self {
        TaskStats {
            processed_count: 0,
            error_count: 0,
            created_at: SystemTime::now(),
            started_at: None,
            stopped_at: None,
        }
    }
}

/// 任务实例
pub struct Task {
    pub id: String,
    pub name: String,
    pub config: TaskConfig,
    pub status: TaskStatus,
    pub stats: TaskStats,
    pub compute_pool: Option<Arc<Mutex<ComputePool>>>,
    // 暂时移除管道，因为Send trait问题
    // pub input_pipeline: Option<Box<dyn InputPipeline>>,
    // pub output_pipeline: Option<Box<dyn OutputPipeline>>,
}

impl Task {
    /// 创建新任务
    pub fn new(config: TaskConfig) -> Self {
        Task {
            id: config.id.clone(),
            name: config.name.clone(),
            config,
            status: TaskStatus::Created,
            stats: TaskStats::default(),
            compute_pool: None,
        }
    }
    
    /// 验证配置并转换到Ready状态
    pub fn validate(&mut self) -> Result<(), String> {
        if self.status != TaskStatus::Created {
            return Err(format!("任务状态不正确，当前为: {:?}", self.status));
        }
        
        // 这里可以添加更多验证逻辑
        self.status = TaskStatus::Ready;
        Ok(())
    }
    
    /// 启动任务
    pub fn start(&mut self) -> Result<(), String> {
        match self.status {
            TaskStatus::Ready | TaskStatus::Stopped => {
                self.status = TaskStatus::Running;
                self.stats.started_at = Some(SystemTime::now());
                Ok(())
            },
            _ => Err(format!("无法启动任务，当前状态: {:?}", self.status)),
        }
    }
    
    /// 暂停任务
    pub fn pause(&mut self) -> Result<(), String> {
        if self.status == TaskStatus::Running {
            self.status = TaskStatus::Paused;
            Ok(())
        } else {
            Err(format!("无法暂停任务，当前状态: {:?}", self.status))
        }
    }
    
    /// 继续任务
    pub fn resume(&mut self) -> Result<(), String> {
        if self.status == TaskStatus::Paused {
            self.status = TaskStatus::Running;
            Ok(())
        } else {
            Err(format!("无法继续任务，当前状态: {:?}", self.status))
        }
    }
    
    /// 停止任务
    pub fn stop(&mut self) -> Result<(), String> {
        match self.status {
            TaskStatus::Running | TaskStatus::Paused | TaskStatus::Error => {
                self.status = TaskStatus::Stopped;
                self.stats.stopped_at = Some(SystemTime::now());
                Ok(())
            },
            _ => Err(format!("无法停止任务，当前状态: {:?}", self.status)),
        }
    }
    
    /// 标记为错误状态
    pub fn mark_error(&mut self) {
        self.status = TaskStatus::Error;
    }
    
    /// 重置错误状态
    pub fn reset_error(&mut self) -> Result<(), String> {
        if self.status == TaskStatus::Error {
            self.status = TaskStatus::Ready;
            Ok(())
        } else {
            Err(format!("任务不处于错误状态: {:?}", self.status))
        }
    }
    
    /// 增加处理计数
    pub fn increment_processed(&mut self) {
        self.stats.processed_count += 1;
    }
    
    /// 增加错误计数
    pub fn increment_error(&mut self) {
        self.stats.error_count += 1;
    }
    
    /// 获取任务摘要信息
    pub fn get_summary(&self) -> TaskSummary {
        TaskSummary {
            id: self.id.clone(),
            name: self.name.clone(),
            status: self.status.as_str().to_string(),
            compute_pool_size: self.compute_pool.as_ref()
                .and_then(|p| p.lock().ok())
                .map(|p| p.size())
                .unwrap_or(0),
            processed_count: self.stats.processed_count,
        }
    }
}

/// 任务摘要信息（用于列表展示）
#[derive(Debug, Clone)]
pub struct TaskSummary {
    pub id: String,
    pub name: String,
    pub status: String,
    pub compute_pool_size: usize,
    pub processed_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestration::config::*;
    
    fn create_test_config() -> TaskConfig {
        TaskConfig {
            id: "test-001".to_string(),
            name: "测试任务".to_string(),
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
        }
    }
    
    #[test]
    fn test_task_state_machine() {
        let config = create_test_config();
        let mut task = Task::new(config);
        
        assert_eq!(task.status, TaskStatus::Created);
        
        // Created -> Ready
        assert!(task.validate().is_ok());
        assert_eq!(task.status, TaskStatus::Ready);
        
        // Ready -> Running
        assert!(task.start().is_ok());
        assert_eq!(task.status, TaskStatus::Running);
        
        // Running -> Paused
        assert!(task.pause().is_ok());
        assert_eq!(task.status, TaskStatus::Paused);
        
        // Paused -> Running
        assert!(task.resume().is_ok());
        assert_eq!(task.status, TaskStatus::Running);
        
        // Running -> Stopped
        assert!(task.stop().is_ok());
        assert_eq!(task.status, TaskStatus::Stopped);
    }
    
    #[test]
    fn test_task_stats() {
        let config = create_test_config();
        let mut task = Task::new(config);
        
        task.increment_processed();
        task.increment_processed();
        task.increment_error();
        
        assert_eq!(task.stats.processed_count, 2);
        assert_eq!(task.stats.error_count, 1);
    }
}
