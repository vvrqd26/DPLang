// 回测配置模块
use serde::{Deserialize, Serialize};

/// 回测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    /// 初始资金
    pub initial_capital: f64,
    
    /// 手续费率（例如：0.0003 表示万3）
    pub commission_rate: f64,
    
    /// 滑点率（例如：0.001 表示千1）
    pub slippage_rate: f64,
    
    /// 最小手续费
    pub min_commission: f64,
    
    /// 印花税率（仅卖出收取，A股为千1）
    pub stamp_duty_rate: f64,
    
    /// 单次最大仓位比例（1.0表示全仓）
    pub position_limit: f64,
    
    /// 是否允许做空
    pub enable_short: bool,
    
    /// 基准标的（用于对比）
    pub benchmark: Option<String>,
    
    /// 无风险利率（年化，用于计算夏普比率等）
    pub risk_free_rate: f64,
    
    /// 输出配置
    pub output_config: OutputConfig,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: 100000.0,       // 10万初始资金
            commission_rate: 0.0003,         // 万3手续费
            slippage_rate: 0.001,            // 千1滑点
            min_commission: 5.0,             // 最小手续费5元
            stamp_duty_rate: 0.001,          // 千1印花税
            position_limit: 1.0,             // 允许全仓
            enable_short: false,             // 不允许做空
            benchmark: None,                 // 无基准
            risk_free_rate: 0.03,            // 3%无风险利率
            output_config: OutputConfig::default(),
        }
    }
}

impl BacktestConfig {
    /// 创建新的回测配置
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 设置初始资金
    pub fn with_initial_capital(mut self, capital: f64) -> Self {
        self.initial_capital = capital;
        self
    }
    
    /// 设置手续费率
    pub fn with_commission_rate(mut self, rate: f64) -> Self {
        self.commission_rate = rate;
        self
    }
    
    /// 设置滑点率
    pub fn with_slippage_rate(mut self, rate: f64) -> Self {
        self.slippage_rate = rate;
        self
    }
    
    /// 设置输出目录
    pub fn with_output_dir(mut self, dir: String) -> Self {
        self.output_config.output_dir = dir;
        self
    }
}

/// 输出配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// 输出目录
    pub output_dir: String,
    
    /// 是否保存交易明细
    pub save_trades: bool,
    
    /// 是否保存持仓记录
    pub save_positions: bool,
    
    /// 是否保存每日统计
    pub save_daily_stats: bool,
    
    /// 是否生成HTML报告
    pub generate_html_report: bool,
    
    /// 详细输出模式
    pub verbose: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            output_dir: "./backtest_results".to_string(),
            save_trades: true,
            save_positions: true,
            save_daily_stats: true,
            generate_html_report: false,
            verbose: false,
        }
    }
}
