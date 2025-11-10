// 执行上下文对象池 - 避免每行创建新的ExecutionContext

use super::context::ExecutionContext;
use crate::runtime::Value;

/// 上下文对象池
pub struct ContextPool {
    /// 空闲上下文队列
    free_contexts: Vec<ExecutionContext>,
    /// 池容量配置
    config: PoolConfig,
}

/// 对象池配置
#[derive(Clone)]
pub struct PoolConfig {
    /// 初始池大小
    pub initial_size: usize,
    /// 最大池大小
    pub max_size: usize,
}

impl Default for PoolConfig {
    fn default() -> Self {
        PoolConfig {
            initial_size: 16,
            max_size: 1024,
        }
    }
}

impl ContextPool {
    /// 创建新的对象池
    pub fn new(config: PoolConfig) -> Self {
        let mut free_contexts = Vec::with_capacity(config.initial_size);
        
        // 预分配初始上下文
        for _ in 0..config.initial_size {
            free_contexts.push(ExecutionContext::new());
        }
        
        ContextPool {
            free_contexts,
            config,
        }
    }
    
    /// 创建默认配置的对象池
    pub fn with_default() -> Self {
        Self::new(PoolConfig::default())
    }
    
    /// 从池中获取上下文
    pub fn acquire(&mut self) -> ExecutionContext {
        if let Some(mut ctx) = self.free_contexts.pop() {
            // 重置上下文状态
            ctx.reset();
            ctx
        } else {
            // 池为空，创建新实例
            ExecutionContext::new()
        }
    }
    
    /// 归还上下文到池中
    pub fn release(&mut self, ctx: ExecutionContext) {
        // 仅当未达到最大容量时才回收
        if self.free_contexts.len() < self.config.max_size {
            self.free_contexts.push(ctx);
        }
        // 超过最大容量时，直接丢弃（自动释放）
    }
    
    /// 获取当前池中空闲上下文数量
    pub fn available_count(&self) -> usize {
        self.free_contexts.len()
    }
    
    /// 清空池中的空闲上下文
    pub fn clear(&mut self) {
        self.free_contexts.clear();
    }
}

/// 实现reset方法扩展
impl ExecutionContext {
    /// 重置上下文状态，清空所有变量
    pub fn reset(&mut self) {
        self.variables.clear();
    }
    
    /// 批量设置变量（避免多次HashMap插入的开销）
    pub fn set_batch(&mut self, vars: impl Iterator<Item = (String, Value)>) {
        for (name, value) in vars {
            self.variables.insert(name, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pool_acquire_release() {
        let mut pool = ContextPool::with_default();
        
        // 获取上下文
        let mut ctx = pool.acquire();
        ctx.set("test".to_string(), Value::Number(42.0));
        
        // 归还
        pool.release(ctx);
        
        // 再次获取，应该是重置后的
        let ctx2 = pool.acquire();
        assert!(ctx2.get("test").is_none());
    }
    
    #[test]
    fn test_pool_capacity_limit() {
        let config = PoolConfig {
            initial_size: 2,
            max_size: 3,
        };
        let mut pool = ContextPool::new(config);
        
        // 释放超过最大容量的上下文
        for _ in 0..5 {
            pool.release(ExecutionContext::new());
        }
        
        // 池中最多保留max_size个
        assert!(pool.available_count() <= 3);
    }
}
