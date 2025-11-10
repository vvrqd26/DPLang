// 回测模块 - 提供专业级策略回测功能

pub mod config;
pub mod portfolio;
pub mod trade_tracker;
pub mod performance;
pub mod engine;
pub mod reporter;

#[cfg(test)]
mod tests;

// 导出主要类型
pub use config::{BacktestConfig, OutputConfig};
pub use portfolio::Portfolio;
pub use trade_tracker::{Trade, TradeTracker};
pub use performance::{
    PerformanceMetrics, ReturnMetrics, RiskMetrics, PerformanceRatios, 
    TradeStatistics, PerformanceCalculator,
};
pub use engine::BacktestEngine;
pub use reporter::Reporter;
