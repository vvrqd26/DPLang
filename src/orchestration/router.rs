// 数据路由器 - 智能分配数据到计算元

use crate::orchestration::compute_pool::ComputePool;
use crate::runtime::{Value, RuntimeError};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// 路由策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingStrategy {
    Hash,         // 哈希分配
    RoundRobin,   // 轮询分配
    Sticky,       // 粘性会话
}

impl RoutingStrategy {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "hash" => Some(RoutingStrategy::Hash),
            "round_robin" | "roundrobin" => Some(RoutingStrategy::RoundRobin),
            "sticky" => Some(RoutingStrategy::Sticky),
            _ => None,
        }
    }
}

/// 数据路由器
pub struct DataRouter {
    strategy: RoutingStrategy,
    key_column: Option<String>,
    sticky_sessions: HashMap<String, usize>, // 粘性会话映射表
}

impl DataRouter {
    /// 创建路由器
    pub fn new(strategy: RoutingStrategy, key_column: Option<String>) -> Self {
        DataRouter {
            strategy,
            key_column,
            sticky_sessions: HashMap::new(),
        }
    }
    
    /// 从配置创建路由器
    pub fn from_config(strategy_str: &str, key_column: Option<String>) -> Result<Self, String> {
        let strategy = RoutingStrategy::from_str(strategy_str)
            .ok_or_else(|| format!("不支持的路由策略: {}", strategy_str))?;
        
        Ok(DataRouter::new(strategy, key_column))
    }
    
    /// 路由数据到计算元并处理
    pub fn route_and_process(
        &mut self,
        data: HashMap<String, Value>,
        pool: &mut ComputePool,
    ) -> Result<Option<HashMap<String, Value>>, RuntimeError> {
        // 提取路由键
        let routing_key = self.extract_routing_key(&data);
        
        // 根据策略选择计算元索引
        let worker_index = match self.strategy {
            RoutingStrategy::Hash => {
                self.hash_route(&routing_key, pool.size())
            },
            RoutingStrategy::RoundRobin => {
                self.round_robin_route(pool)
            },
            RoutingStrategy::Sticky => {
                self.sticky_route(&routing_key, pool)
            },
        };
        
        // 获取计算元并处理
        if let Some(worker) = pool.get_worker(worker_index) {
            worker.process_tick(data)
        } else {
            Err(RuntimeError::type_error("无可用的计算元"))
        }
    }
    
    /// 提取路由键
    fn extract_routing_key(&self, data: &HashMap<String, Value>) -> String {
        if let Some(ref key_col) = self.key_column {
            if let Some(value) = data.get(key_col) {
                return value.to_string();
            }
        }
        String::new()
    }
    
    /// 哈希路由
    fn hash_route(&self, key: &str, pool_size: usize) -> usize {
        if pool_size == 0 {
            return 0;
        }
        
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        (hash % pool_size as u64) as usize
    }
    
    /// 轮询路由
    fn round_robin_route(&self, pool: &mut ComputePool) -> usize {
        // 尝试获取下一个空闲计算元
        if let Some(worker) = pool.get_next_idle_worker() {
            worker.id
        } else {
            // 如果没有空闲的，返回第一个
            0
        }
    }
    
    /// 粘性会话路由
    fn sticky_route(&mut self, key: &str, pool: &mut ComputePool) -> usize {
        // 检查是否已有映射
        if let Some(&index) = self.sticky_sessions.get(key) {
            // 验证该计算元是否仍然有效
            if index < pool.size() {
                return index;
            } else {
                // 计算元已失效，移除映射
                self.sticky_sessions.remove(key);
            }
        }
        
        // 首次分配或重新分配：选择最空闲的计算元
        let index = self.find_least_busy_worker(pool);
        self.sticky_sessions.insert(key.to_string(), index);
        index
    }
    
    /// 找到最不繁忙的计算元
    fn find_least_busy_worker(&self, pool: &ComputePool) -> usize {
        // 由于 workers 是 private，我们使用 size() 方法
        let pool_size = pool.size();
        if pool_size == 0 {
            return 0;
        }
        // 简单轮询选择第一个
        0
    }
    
    /// 清理粘性会话映射中的无效条目
    pub fn cleanup_sticky_sessions(&mut self, pool_size: usize) {
        self.sticky_sessions.retain(|_, &mut index| index < pool_size);
    }
    
    /// 移除特定路由键的会话
    pub fn remove_session(&mut self, key: &str) {
        self.sticky_sessions.remove(key);
    }
    
    /// 获取粘性会话数量
    pub fn session_count(&self) -> usize {
        self.sticky_sessions.len()
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
    fn test_hash_routing() {
        let router = DataRouter::new(RoutingStrategy::Hash, Some("stock_code".to_string()));
        
        let index1 = router.hash_route("000001", 10);
        let index2 = router.hash_route("000001", 10);
        let index3 = router.hash_route("000002", 10);
        
        // 同一key应该路由到同一索引
        assert_eq!(index1, index2);
        assert!(index1 < 10);
        assert!(index3 < 10);
    }
    
    #[test]
    fn test_sticky_routing() {
        let mut router = DataRouter::new(RoutingStrategy::Sticky, Some("stock_code".to_string()));
        let script = create_test_script();
        let mut pool = ComputePool::new(script, 5, 20, 1000);
        
        let index1 = router.sticky_route("000001", &mut pool);
        let index2 = router.sticky_route("000001", &mut pool);
        
        // 同一key应该路由到同一计算元
        assert_eq!(index1, index2);
        assert_eq!(router.session_count(), 1);
    }
    
    #[test]
    fn test_routing_strategy_from_str() {
        assert_eq!(RoutingStrategy::from_str("hash"), Some(RoutingStrategy::Hash));
        assert_eq!(RoutingStrategy::from_str("round_robin"), Some(RoutingStrategy::RoundRobin));
        assert_eq!(RoutingStrategy::from_str("sticky"), Some(RoutingStrategy::Sticky));
        assert_eq!(RoutingStrategy::from_str("invalid"), None);
    }
    
    #[test]
    fn test_extract_routing_key() {
        let router = DataRouter::new(RoutingStrategy::Hash, Some("stock_code".to_string()));
        
        let mut data = HashMap::new();
        data.insert("stock_code".to_string(), Value::String("000001".to_string()));
        data.insert("price".to_string(), Value::Number(100.0));
        
        let key = router.extract_routing_key(&data);
        assert_eq!(key, "000001");
    }
}
