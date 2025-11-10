// 性能指标计算模块
use serde::{Deserialize, Serialize};

/// 收益指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnMetrics {
    /// 总收益率
    pub total_return: f64,
    
    /// 年化收益率
    pub annual_return: f64,
    
    /// 基准收益率
    pub benchmark_return: Option<f64>,
    
    /// 超额收益
    pub excess_return: Option<f64>,
}

/// 风险指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    /// 最大回撤
    pub max_drawdown: f64,
    
    /// 最大回撤持续期（天数）
    pub max_drawdown_duration: usize,
    
    /// 年化波动率
    pub annual_volatility: f64,
    
    /// 下行波动率
    pub downside_volatility: f64,
    
    /// VaR(95%)
    pub var_95: f64,
}

/// 综合评价指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRatios {
    /// 夏普比率
    pub sharpe_ratio: f64,
    
    /// 索提诺比率
    pub sortino_ratio: f64,
    
    /// 卡玛比率
    pub calmar_ratio: f64,
    
    /// 信息比率
    pub information_ratio: Option<f64>,
    
    /// 胜率
    pub win_rate: f64,
    
    /// 盈亏比
    pub profit_loss_ratio: f64,
}

/// 交易统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeStatistics {
    /// 总交易次数
    pub total_trades: usize,
    
    /// 盈利交易次数
    pub winning_trades: usize,
    
    /// 亏损交易次数
    pub losing_trades: usize,
    
    /// 平均持仓天数
    pub avg_holding_days: f64,
    
    /// 最长持仓天数
    pub max_holding_days: usize,
    
    /// 平均盈利
    pub avg_profit: f64,
    
    /// 平均亏损
    pub avg_loss: f64,
    
    /// 最大单笔盈利
    pub max_profit: f64,
    
    /// 最大单笔亏损
    pub max_loss: f64,
    
    /// 最长连续盈利次数
    pub max_consecutive_wins: usize,
    
    /// 最长连续亏损次数
    pub max_consecutive_losses: usize,
}

/// 完整性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub return_metrics: ReturnMetrics,
    pub risk_metrics: RiskMetrics,
    pub performance_ratios: PerformanceRatios,
    pub trade_statistics: TradeStatistics,
}

/// 性能计算器
pub struct PerformanceCalculator {
    equity_curve: Vec<f64>,
    initial_capital: f64,
    trading_days: usize,
    risk_free_rate: f64,
}

impl PerformanceCalculator {
    /// 创建性能计算器
    pub fn new(equity_curve: Vec<f64>, initial_capital: f64, risk_free_rate: f64) -> Self {
        let trading_days = equity_curve.len().saturating_sub(1);
        Self {
            equity_curve,
            initial_capital,
            trading_days,
            risk_free_rate,
        }
    }
    
    /// 计算收益指标
    pub fn calculate_return_metrics(&self, benchmark_curve: Option<&[f64]>) -> ReturnMetrics {
        let total_return = self.calculate_total_return();
        let annual_return = self.calculate_annual_return(total_return);
        
        let (benchmark_return, excess_return) = if let Some(benchmark) = benchmark_curve {
            let bench_return = Self::calc_total_return_from_curve(benchmark);
            let excess = total_return - bench_return;
            (Some(bench_return), Some(excess))
        } else {
            (None, None)
        };
        
        ReturnMetrics {
            total_return,
            annual_return,
            benchmark_return,
            excess_return,
        }
    }
    
    /// 计算风险指标
    pub fn calculate_risk_metrics(&self) -> RiskMetrics {
        let (max_drawdown, max_drawdown_duration) = self.calculate_max_drawdown();
        let returns = self.calculate_daily_returns();
        let annual_volatility = self.calculate_annual_volatility(&returns);
        let downside_volatility = self.calculate_downside_volatility(&returns);
        let var_95 = self.calculate_var(&returns, 0.05);
        
        RiskMetrics {
            max_drawdown,
            max_drawdown_duration,
            annual_volatility,
            downside_volatility,
            var_95,
        }
    }
    
    /// 计算综合指标
    pub fn calculate_performance_ratios(
        &self,
        return_metrics: &ReturnMetrics,
        risk_metrics: &RiskMetrics,
        trade_stats: &TradeStatistics,
        benchmark_curve: Option<&[f64]>,
    ) -> PerformanceRatios {
        let sharpe_ratio = self.calculate_sharpe_ratio(
            return_metrics.annual_return,
            risk_metrics.annual_volatility,
        );
        
        let sortino_ratio = self.calculate_sortino_ratio(
            return_metrics.annual_return,
            risk_metrics.downside_volatility,
        );
        
        let calmar_ratio = if risk_metrics.max_drawdown > 0.0 {
            return_metrics.annual_return / risk_metrics.max_drawdown
        } else {
            0.0
        };
        
        let information_ratio = if let Some(benchmark) = benchmark_curve {
            Some(self.calculate_information_ratio(benchmark))
        } else {
            None
        };
        
        let win_rate = if trade_stats.total_trades > 0 {
            trade_stats.winning_trades as f64 / trade_stats.total_trades as f64
        } else {
            0.0
        };
        
        let profit_loss_ratio = if trade_stats.avg_loss < 0.0 {
            trade_stats.avg_profit / trade_stats.avg_loss.abs()
        } else {
            0.0
        };
        
        PerformanceRatios {
            sharpe_ratio,
            sortino_ratio,
            calmar_ratio,
            information_ratio,
            win_rate,
            profit_loss_ratio,
        }
    }
    
    // ========== 私有计算方法 ==========
    
    /// 计算总收益率
    fn calculate_total_return(&self) -> f64 {
        if self.initial_capital > 0.0 && !self.equity_curve.is_empty() {
            let final_value = self.equity_curve.last().unwrap();
            (final_value - self.initial_capital) / self.initial_capital
        } else {
            0.0
        }
    }
    
    /// 从资金曲线计算总收益率
    fn calc_total_return_from_curve(curve: &[f64]) -> f64 {
        if curve.len() >= 2 {
            let initial = curve[0];
            let final_val = curve[curve.len() - 1];
            if initial > 0.0 {
                (final_val - initial) / initial
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
    
    /// 计算年化收益率
    fn calculate_annual_return(&self, total_return: f64) -> f64 {
        if self.trading_days > 0 {
            let years = self.trading_days as f64 / 252.0;
            if years > 0.0 {
                ((1.0 + total_return).powf(1.0 / years)) - 1.0
            } else {
                total_return
            }
        } else {
            0.0
        }
    }
    
    /// 计算每日收益率
    fn calculate_daily_returns(&self) -> Vec<f64> {
        let mut returns = Vec::new();
        for i in 1..self.equity_curve.len() {
            let prev = self.equity_curve[i - 1];
            let curr = self.equity_curve[i];
            if prev > 0.0 {
                returns.push((curr - prev) / prev);
            } else {
                returns.push(0.0);
            }
        }
        returns
    }
    
    /// 计算最大回撤和持续期
    fn calculate_max_drawdown(&self) -> (f64, usize) {
        let mut max_drawdown = 0.0;
        let mut max_duration = 0;
        let mut peak = self.initial_capital;
        let mut peak_index = 0;
        
        for (i, &value) in self.equity_curve.iter().enumerate() {
            if value > peak {
                peak = value;
                peak_index = i;
            }
            
            let drawdown = if peak > 0.0 {
                (peak - value) / peak
            } else {
                0.0
            };
            
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
                max_duration = i - peak_index;
            }
        }
        
        (max_drawdown, max_duration)
    }
    
    /// 计算年化波动率
    fn calculate_annual_volatility(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }
        
        let std_dev = Self::calculate_std(returns);
        std_dev * (252.0_f64).sqrt()
    }
    
    /// 计算下行波动率
    fn calculate_downside_volatility(&self, returns: &[f64]) -> f64 {
        let negative_returns: Vec<f64> = returns.iter()
            .filter(|&&r| r < 0.0)
            .copied()
            .collect();
        
        if negative_returns.is_empty() {
            return 0.0;
        }
        
        let std_dev = Self::calculate_std(&negative_returns);
        std_dev * (252.0_f64).sqrt()
    }
    
    /// 计算VaR
    fn calculate_var(&self, returns: &[f64], confidence: f64) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }
        
        let mut sorted = returns.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let index = (returns.len() as f64 * confidence) as usize;
        if index < sorted.len() {
            sorted[index]
        } else {
            sorted[sorted.len() - 1]
        }
    }
    
    /// 计算夏普比率
    fn calculate_sharpe_ratio(&self, annual_return: f64, annual_volatility: f64) -> f64 {
        if annual_volatility > 0.0 {
            (annual_return - self.risk_free_rate) / annual_volatility
        } else {
            0.0
        }
    }
    
    /// 计算索提诺比率
    fn calculate_sortino_ratio(&self, annual_return: f64, downside_volatility: f64) -> f64 {
        if downside_volatility > 0.0 {
            (annual_return - self.risk_free_rate) / downside_volatility
        } else {
            0.0
        }
    }
    
    /// 计算信息比率
    fn calculate_information_ratio(&self, benchmark_curve: &[f64]) -> f64 {
        let strategy_returns = self.calculate_daily_returns();
        let benchmark_returns = Self::calc_daily_returns_from_curve(benchmark_curve);
        
        if strategy_returns.len() != benchmark_returns.len() {
            return 0.0;
        }
        
        let excess_returns: Vec<f64> = strategy_returns.iter()
            .zip(benchmark_returns.iter())
            .map(|(s, b)| s - b)
            .collect();
        
        let mean_excess = Self::calculate_mean(&excess_returns);
        let tracking_error = Self::calculate_std(&excess_returns);
        
        if tracking_error > 0.0 {
            mean_excess * (252.0_f64).sqrt() / (tracking_error * (252.0_f64).sqrt())
        } else {
            0.0
        }
    }
    
    /// 从资金曲线计算日收益率
    fn calc_daily_returns_from_curve(curve: &[f64]) -> Vec<f64> {
        let mut returns = Vec::new();
        for i in 1..curve.len() {
            let prev = curve[i - 1];
            let curr = curve[i];
            if prev > 0.0 {
                returns.push((curr - prev) / prev);
            } else {
                returns.push(0.0);
            }
        }
        returns
    }
    
    /// 计算均值
    fn calculate_mean(values: &[f64]) -> f64 {
        if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<f64>() / values.len() as f64
        }
    }
    
    /// 计算标准差
    fn calculate_std(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }
        
        let mean = Self::calculate_mean(values);
        let variance = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (values.len() - 1) as f64;
        
        variance.sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_return_calculation() {
        let equity_curve = vec![100000.0, 105000.0, 110000.0, 108000.0, 112000.0];
        let calc = PerformanceCalculator::new(equity_curve, 100000.0, 0.03);
        
        let metrics = calc.calculate_return_metrics(None);
        assert!((metrics.total_return - 0.12).abs() < 0.01);
        assert!(metrics.annual_return > 0.0);
    }
    
    #[test]
    fn test_max_drawdown() {
        let equity_curve = vec![100000.0, 110000.0, 105000.0, 95000.0, 100000.0];
        let calc = PerformanceCalculator::new(equity_curve, 100000.0, 0.03);
        
        let (max_dd, _) = calc.calculate_max_drawdown();
        assert!(max_dd > 0.13); // (110000 - 95000) / 110000
    }
    
    #[test]
    fn test_volatility() {
        let equity_curve = vec![100000.0, 101000.0, 99000.0, 102000.0, 98000.0];
        let calc = PerformanceCalculator::new(equity_curve, 100000.0, 0.03);
        
        let returns = calc.calculate_daily_returns();
        let vol = calc.calculate_annual_volatility(&returns);
        assert!(vol > 0.0);
    }
}
