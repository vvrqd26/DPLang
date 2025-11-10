// 交易跟踪模块
use serde::{Deserialize, Serialize};

/// 交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// 交易编号
    pub trade_id: usize,
    
    /// 开仓日期
    pub entry_date: String,
    
    /// 开仓价格
    pub entry_price: f64,
    
    /// 开仓信号
    pub entry_signal: String,
    
    /// 平仓日期
    pub exit_date: String,
    
    /// 平仓价格
    pub exit_price: f64,
    
    /// 平仓信号
    pub exit_signal: String,
    
    /// 交易数量
    pub shares: f64,
    
    /// 持仓天数
    pub holding_days: usize,
    
    /// 毛利润
    pub gross_profit: f64,
    
    /// 手续费总计
    pub commission: f64,
    
    /// 印花税
    pub stamp_duty: f64,
    
    /// 净利润
    pub net_profit: f64,
    
    /// 收益率
    pub return_rate: f64,
}

/// 交易跟踪器
#[derive(Debug)]
pub struct TradeTracker {
    /// 已完成的交易列表
    pub completed_trades: Vec<Trade>,
    
    /// 当前持仓信息
    current_position: Option<PositionInfo>,
    
    /// 交易计数器
    trade_counter: usize,
}

/// 当前持仓信息
#[derive(Debug, Clone)]
struct PositionInfo {
    entry_date: String,
    entry_price: f64,
    entry_signal: String,
    shares: f64,
    entry_commission: f64,
}

impl TradeTracker {
    /// 创建新的交易跟踪器
    pub fn new() -> Self {
        Self {
            completed_trades: Vec::new(),
            current_position: None,
            trade_counter: 0,
        }
    }
    
    /// 记录开仓
    pub fn record_entry(&mut self, date: String, price: f64, signal: String, shares: f64, commission: f64) {
        self.current_position = Some(PositionInfo {
            entry_date: date,
            entry_price: price,
            entry_signal: signal,
            shares,
            entry_commission: commission,
        });
    }
    
    /// 记录平仓
    pub fn record_exit(&mut self, date: String, price: f64, signal: String, exit_commission: f64, stamp_duty: f64) {
        if let Some(pos) = self.current_position.take() {
            self.trade_counter += 1;
            
            let gross_profit = (price - pos.entry_price) * pos.shares;
            let total_commission = pos.entry_commission + exit_commission;
            let net_profit = gross_profit - total_commission - stamp_duty;
            let return_rate = if pos.entry_price > 0.0 {
                (price - pos.entry_price) / pos.entry_price
            } else {
                0.0
            };
            
            // 计算持仓天数（简化处理，实际应该用日期计算）
            let holding_days = self.calculate_holding_days(&pos.entry_date, &date);
            
            let trade = Trade {
                trade_id: self.trade_counter,
                entry_date: pos.entry_date,
                entry_price: pos.entry_price,
                entry_signal: pos.entry_signal,
                exit_date: date,
                exit_price: price,
                exit_signal: signal,
                shares: pos.shares,
                holding_days,
                gross_profit,
                commission: total_commission,
                stamp_duty,
                net_profit,
                return_rate,
            };
            
            self.completed_trades.push(trade);
        }
    }
    
    /// 是否有持仓
    pub fn has_position(&self) -> bool {
        self.current_position.is_some()
    }
    
    /// 获取交易总数
    pub fn total_trades(&self) -> usize {
        self.completed_trades.len()
    }
    
    /// 计算持仓天数（简化版本，通过行号差估算）
    fn calculate_holding_days(&self, _entry_date: &str, _exit_date: &str) -> usize {
        // 简化处理：通过交易记录数量估算
        // 实际应该解析日期字符串计算天数差
        1
    }
    
    /// 获取盈利交易数量
    pub fn winning_trades(&self) -> usize {
        self.completed_trades.iter().filter(|t| t.net_profit > 0.0).count()
    }
    
    /// 获取亏损交易数量
    pub fn losing_trades(&self) -> usize {
        self.completed_trades.iter().filter(|t| t.net_profit < 0.0).count()
    }
    
    /// 计算平均盈利
    pub fn average_profit(&self) -> f64 {
        let profits: Vec<f64> = self.completed_trades
            .iter()
            .filter(|t| t.net_profit > 0.0)
            .map(|t| t.net_profit)
            .collect();
        
        if profits.is_empty() {
            0.0
        } else {
            profits.iter().sum::<f64>() / profits.len() as f64
        }
    }
    
    /// 计算平均亏损
    pub fn average_loss(&self) -> f64 {
        let losses: Vec<f64> = self.completed_trades
            .iter()
            .filter(|t| t.net_profit < 0.0)
            .map(|t| t.net_profit)
            .collect();
        
        if losses.is_empty() {
            0.0
        } else {
            losses.iter().sum::<f64>() / losses.len() as f64
        }
    }
    
    /// 计算最大单笔盈利
    pub fn max_profit(&self) -> f64 {
        self.completed_trades
            .iter()
            .map(|t| t.net_profit)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }
    
    /// 计算最大单笔亏损
    pub fn max_loss(&self) -> f64 {
        self.completed_trades
            .iter()
            .map(|t| t.net_profit)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }
    
    /// 计算最长连续盈利次数
    pub fn max_consecutive_wins(&self) -> usize {
        let mut max_wins = 0;
        let mut current_wins = 0;
        
        for trade in &self.completed_trades {
            if trade.net_profit > 0.0 {
                current_wins += 1;
                max_wins = max_wins.max(current_wins);
            } else {
                current_wins = 0;
            }
        }
        
        max_wins
    }
    
    /// 计算最长连续亏损次数
    pub fn max_consecutive_losses(&self) -> usize {
        let mut max_losses = 0;
        let mut current_losses = 0;
        
        for trade in &self.completed_trades {
            if trade.net_profit < 0.0 {
                current_losses += 1;
                max_losses = max_losses.max(current_losses);
            } else {
                current_losses = 0;
            }
        }
        
        max_losses
    }
}

impl Default for TradeTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_trade_tracker() {
        let mut tracker = TradeTracker::new();
        
        // 记录开仓
        tracker.record_entry("2023-01-01".to_string(), 10.0, "买入".to_string(), 100.0, 3.0);
        assert!(tracker.has_position());
        
        // 记录平仓
        tracker.record_exit("2023-01-05".to_string(), 11.0, "卖出".to_string(), 3.3, 1.1);
        assert!(!tracker.has_position());
        assert_eq!(tracker.total_trades(), 1);
        
        let trade = &tracker.completed_trades[0];
        assert_eq!(trade.entry_price, 10.0);
        assert_eq!(trade.exit_price, 11.0);
        assert!(trade.net_profit > 0.0);
    }
    
    #[test]
    fn test_trade_statistics() {
        let mut tracker = TradeTracker::new();
        
        // 盈利交易
        tracker.record_entry("2023-01-01".to_string(), 10.0, "买入".to_string(), 100.0, 3.0);
        tracker.record_exit("2023-01-05".to_string(), 11.0, "卖出".to_string(), 3.3, 1.1);
        
        // 亏损交易
        tracker.record_entry("2023-01-06".to_string(), 11.0, "买入".to_string(), 100.0, 3.3);
        tracker.record_exit("2023-01-10".to_string(), 10.5, "卖出".to_string(), 3.15, 1.05);
        
        assert_eq!(tracker.winning_trades(), 1);
        assert_eq!(tracker.losing_trades(), 1);
        assert!(tracker.average_profit() > 0.0);
        assert!(tracker.average_loss() < 0.0);
    }
}
