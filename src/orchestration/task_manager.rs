// 任务管理器 - 管理所有任务的生命周期

use crate::orchestration::{Task, TaskConfig, TaskSummary, ComputePool};
#[allow(unused_imports)]
use crate::orchestration::pipeline::{InputPipeline, OutputPipeline, FileInputPipeline, StdinInputPipeline, FileOutputPipeline, StdoutOutputPipeline, OutputMode};
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fs;

/// 任务管理器
pub struct TaskManager {
    tasks: HashMap<String, Arc<Mutex<Task>>>,
}

impl TaskManager {
    /// 创建新的任务管理器
    pub fn new() -> Self {
        TaskManager {
            tasks: HashMap::new(),
        }
    }
    
    /// 创建任务
    pub fn create_task(&mut self, config: TaskConfig) -> Result<String, String> {
        let task_id = config.id.clone();
        
        if self.tasks.contains_key(&task_id) {
            return Err(format!("任务ID已存在: {}", task_id));
        }
        
        let mut task = Task::new(config.clone());
        task.validate()?;
        
        // 解析脚本
        let script = self.load_script(&config.scripts[0])?;
        
        // 创建计算元池
        let pool = ComputePool::new(
            script,
            config.compute_pool.initial_size,
            config.compute_pool.max_size,
            config.compute_pool.window_size,
        );
        task.compute_pool = Some(Arc::new(Mutex::new(pool)));
        
        // 暂时注释管道创建，因为Send trait问题
        // 创建输入管道
        // task.input_pipeline = Some(self.create_input_pipeline(&config.input)?);
        // 创建输出管道
        // task.output_pipeline = Some(self.create_output_pipeline(&config.output)?);
        
        self.tasks.insert(task_id.clone(), Arc::new(Mutex::new(task)));
        
        Ok(task_id)
    }
    
    /// 删除任务
    pub fn delete_task(&mut self, task_id: &str) -> Result<(), String> {
        if let Some(task_arc) = self.tasks.remove(task_id) {
            let mut task = task_arc.lock().unwrap();
            task.stop().ok();
            Ok(())
        } else {
            Err(format!("任务不存在: {}", task_id))
        }
    }
    
    /// 启动任务
    pub fn start_task(&mut self, task_id: &str) -> Result<(), String> {
        if let Some(task_arc) = self.tasks.get(task_id) {
            let mut task = task_arc.lock().unwrap();
            task.start()
        } else {
            Err(format!("任务不存在: {}", task_id))
        }
    }
    
    /// 暂停任务
    pub fn pause_task(&mut self, task_id: &str) -> Result<(), String> {
        if let Some(task_arc) = self.tasks.get(task_id) {
            let mut task = task_arc.lock().unwrap();
            task.pause()
        } else {
            Err(format!("任务不存在: {}", task_id))
        }
    }
    
    /// 继续任务
    pub fn resume_task(&mut self, task_id: &str) -> Result<(), String> {
        if let Some(task_arc) = self.tasks.get(task_id) {
            let mut task = task_arc.lock().unwrap();
            task.resume()
        } else {
            Err(format!("任务不存在: {}", task_id))
        }
    }
    
    /// 停止任务
    pub fn stop_task(&mut self, task_id: &str) -> Result<(), String> {
        if let Some(task_arc) = self.tasks.get(task_id) {
            let mut task = task_arc.lock().unwrap();
            task.stop()
        } else {
            Err(format!("任务不存在: {}", task_id))
        }
    }
    
    /// 获取任务列表
    pub fn list_tasks(&self) -> Vec<TaskSummary> {
        self.tasks.values()
            .filter_map(|task_arc| {
                task_arc.lock().ok().map(|task| task.get_summary())
            })
            .collect()
    }
    
    /// 获取任务详情
    pub fn get_task(&self, task_id: &str) -> Option<Arc<Mutex<Task>>> {
        self.tasks.get(task_id).cloned()
    }
    
    /// 加载脚本文件
    fn load_script(&self, script_path: &str) -> Result<crate::parser::Script, String> {
        let source = fs::read_to_string(script_path)
            .map_err(|e| format!("无法读取脚本文件 {}: {}", script_path, e))?;
        
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize()
            .map_err(|e| format!("词法分析错误: {:?}", e))?;
        
        let mut parser = Parser::new(tokens);
        parser.parse()
            .map_err(|e| format!("语法分析错误: {:?}", e))
    }
    
    /*
    // 暂时注释，等解决Send trait问题
    /// 创建输入管道
    fn create_input_pipeline(&self, config: &crate::orchestration::config::IOConfig) -> Result<Box<dyn InputPipeline>, String> {
        match config.io_type.as_str() {
            "stdin" => Ok(Box::new(StdinInputPipeline::new())),
            "file" => {
                let path = config.path.as_ref()
                    .ok_or("文件输入管道需要指定path")?;
                Ok(Box::new(FileInputPipeline::new(path.clone())))
            },
            _ => Err(format!("不支持的输入类型: {}", config.io_type)),
        }
    }
    
    /// 创建输出管道
    fn create_output_pipeline(&self, config: &crate::orchestration::config::IOConfig) -> Result<Box<dyn OutputPipeline>, String> {
        match config.io_type.as_str() {
            "stdout" => Ok(Box::new(StdoutOutputPipeline::new())),
            "file" => {
                let path = config.path.as_ref()
                    .ok_or("文件输出管道需要指定path")?;
                let mode = match config.mode.as_deref() {
                    Some("split") => OutputMode::Split,
                    Some("merge") => OutputMode::Merge,
                    _ => OutputMode::Merge,
                };
                Ok(Box::new(FileOutputPipeline::new(
                    path,
                    mode,
                    config.buffer_size,
                    config.auto_flush,
                )))
            },
            _ => Err(format!("不支持的输出类型: {}", config.io_type)),
        }
    }
    */
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}
