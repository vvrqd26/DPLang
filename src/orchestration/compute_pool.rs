// 计算元池 - StreamingExecutor 池化管理

use crate::executor::StreamingExecutor;
use crate::parser::Script;
use crate::runtime::{Value, RuntimeError};
use std::collections::HashMap;

/// 计算元状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerStatus {
    Idle,    // 空闲
    Busy,    // 繁忙
    Error,   // 错误
}

/// 计算元包装
pub struct Worker {
    pub id: usize,
    pub executor: StreamingExecutor,
    pub status: WorkerStatus,
    pub processed_count: u64,
}

impl Worker {
    pub fn new(id: usize, executor: StreamingExecutor) -> Self {
        Worker {
            id,
            executor,
            status: WorkerStatus::Idle,
            processed_count: 0,
        }
    }
    
    /// 处理单个tick数据
    pub fn process_tick(&mut self, data: HashMap<String, Value>) -> Result<Option<HashMap<String, Value>>, RuntimeError> {
        self.status = WorkerStatus::Busy;
        let result = self.executor.push_tick(data);
        
        match &result {
            Ok(_) => {
                self.processed_count += 1;
                self.status = WorkerStatus::Idle;
            },
            Err(_) => {
                self.status = WorkerStatus::Error;
            }
        }
        
        result
    }
    
    /// 重置计算元状态
    pub fn reset(&mut self) {
        self.status = WorkerStatus::Idle;
    }
}

/// 计算元池
pub struct ComputePool {
    workers: Vec<Worker>,
    window_size: usize,
    max_size: usize,
    next_round_robin: usize,
}

impl ComputePool {
    /// 创建计算元池
    pub fn new(
        script: Script,
        initial_size: usize,
        max_size: usize,
        window_size: usize,
    ) -> Self {
        let mut workers = Vec::with_capacity(initial_size);
        
        for id in 0..initial_size {
            // 每个worker持有script的克隆
            let executor = StreamingExecutor::new(script.clone(), window_size);
            workers.push(Worker::new(id, executor));
        }
        
        ComputePool {
            workers,
            window_size,
            max_size,
            next_round_robin: 0,
        }
    }
    
    /// 获取池大小
    pub fn size(&self) -> usize {
        self.workers.len()
    }
    
    /// 获取空闲计算元数量
    pub fn idle_count(&self) -> usize {
        self.workers.iter()
            .filter(|w| w.status == WorkerStatus::Idle)
            .count()
    }
    
    /// 获取繁忙计算元数量
    pub fn busy_count(&self) -> usize {
        self.workers.iter()
            .filter(|w| w.status == WorkerStatus::Busy)
            .count()
    }
    
    /// 获取错误计算元数量
    pub fn error_count(&self) -> usize {
        self.workers.iter()
            .filter(|w| w.status == WorkerStatus::Error)
            .count()
    }
    
    /// 按索引获取计算元
    pub fn get_worker(&mut self, index: usize) -> Option<&mut Worker> {
        self.workers.get_mut(index)
    }
    
    /// 轮询获取下一个空闲计算元
    pub fn get_next_idle_worker(&mut self) -> Option<&mut Worker> {
        let start = self.next_round_robin;
        let len = self.workers.len();
        
        for i in 0..len {
            let index = (start + i) % len;
            if self.workers[index].status == WorkerStatus::Idle {
                self.next_round_robin = (index + 1) % len;
                return Some(&mut self.workers[index]);
            }
        }
        
        None
    }
    
    /// 计算负载率
    pub fn load_rate(&self) -> f64 {
        if self.workers.is_empty() {
            return 0.0;
        }
        
        let busy = self.busy_count() as f64;
        let total = self.workers.len() as f64;
        busy / total
    }
    
    /// 扩容（添加新的计算元）
    pub fn scale_up(&mut self, script: &Script, count: usize) -> Result<(), String> {
        let current_size = self.workers.len();
        let new_size = current_size + count;
        
        if new_size > self.max_size {
            return Err(format!(
                "扩容失败：目标大小 {} 超过最大限制 {}",
                new_size, self.max_size
            ));
        }
        
        for id in current_size..new_size {
            let executor = StreamingExecutor::new(script.clone(), self.window_size);
            self.workers.push(Worker::new(id, executor));
        }
        
        Ok(())
    }
    
    /// 缩容（移除空闲计算元）
    pub fn scale_down(&mut self, target_size: usize) -> Result<usize, String> {
        if target_size >= self.workers.len() {
            return Ok(0);
        }
        
        let current_size = self.workers.len();
        let mut removed = 0;
        
        // 只移除空闲的计算元
        self.workers.retain(|w| {
            if w.status == WorkerStatus::Idle && current_size - removed > target_size {
                removed += 1;
                false
            } else {
                true
            }
        });
        
        // 重新分配ID
        for (i, worker) in self.workers.iter_mut().enumerate() {
            worker.id = i;
        }
        
        Ok(removed)
    }
    
    /// 移除错误的计算元
    pub fn remove_error_workers(&mut self) -> usize {
        let before = self.workers.len();
        self.workers.retain(|w| w.status != WorkerStatus::Error);
        
        // 重新分配ID
        for (i, worker) in self.workers.iter_mut().enumerate() {
            worker.id = i;
        }
        
        before - self.workers.len()
    }
    
    /// 获取总处理数
    pub fn total_processed(&self) -> u64 {
        self.workers.iter().map(|w| w.processed_count).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Script;
    
    fn create_test_script() -> Script {
        Script::DataScript {
            imports: vec![],
            input: vec![],
            output: vec![],
            error_block: None,
            precision: None,
            body: vec![],
        }
    }
    
    #[test]
    fn test_compute_pool_creation() {
        let script = create_test_script();
        let pool = ComputePool::new(script, 5, 20, 1000);
        
        assert_eq!(pool.size(), 5);
        assert_eq!(pool.idle_count(), 5);
        assert_eq!(pool.busy_count(), 0);
    }
    
    #[test]
    fn test_compute_pool_scale_up() {
        let script = create_test_script();
        let mut pool = ComputePool::new(script.clone(), 5, 20, 1000);
        
        assert!(pool.scale_up(&script, 5).is_ok());
        assert_eq!(pool.size(), 10);
        
        assert!(pool.scale_up(&script, 15).is_err()); // 超过最大限制
    }
    
    #[test]
    fn test_compute_pool_scale_down() {
        let script = create_test_script();
        let mut pool = ComputePool::new(script, 10, 20, 1000);
        
        let removed = pool.scale_down(5).unwrap();
        assert!(removed <= 5);
        assert!(pool.size() >= 5);
    }
    
    #[test]
    fn test_load_rate() {
        let script = create_test_script();
        let pool = ComputePool::new(script, 5, 20, 1000);
        
        assert_eq!(pool.load_rate(), 0.0);
    }
}
