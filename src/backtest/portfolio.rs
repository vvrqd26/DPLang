// 投资组合管理模块
use serde::{Deserialize, Serialize};

/// 持仓记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// 日期
    pub date: String,
    
    /// 信号
    pub signal: String,
    
    /// 价格
    pub price: f64,
    
    /// 持仓数量
    pub shares: f64,
    
    /// 持仓市值
    pub position_value: f64,
    
    /// 现金余额
    pub cash: f64,
    
    /// 总资产
    pub total_value: f64,
    
    /// 当日收益率
    pub daily_return: f64,
    
    /// 累计收益率
    pub cumulative_return: f64,
    
    /// 当前回撤
    pub drawdown: f64,
}

/// 投资组合状态
#[derive(Debug, Clone)]
pub struct Portfolio {
    /// 现金余额
    pub cash: f64,
    
    /// 持仓数量
    pub shares: f64,
    
    /// 总资产
    pub total_value: f64,
    
    /// 历史最高净值
    pub peak_value: f64,
    
    /// 当前回撤
    pub current_drawdown: f64,
    
    /// 初始资金
    pub initial_capital: f64,
    
    /// 持仓历史
    pub positions: Vec<Position>,
    
    /// 资金曲线（每日总资产）
    pub equity_curve: Vec<f64>,
}

impl Portfolio {
    /// 创建新的投资组合
    pub fn new(initial_capital: f64) -> Self {
        Self {
            cash: initial_capital,
            shares: 0.0,
            total_value: initial_capital,
            peak_value: initial_capital,
            current_drawdown: 0.0,
            initial_capital,
            positions: Vec::new(),
            equity_curve: vec![initial_capital],
        }
    }
    
    /// 执行买入
    /// 返回：(实际买入数量, 总成本)
    pub fn buy(&mut self, price: f64, quantity: f64, commission: f64) -> Result<(f64, f64), String> {
        let total_cost = price * quantity + commission;
        
        if total_cost > self.cash {
            return Err(format!("现金不足: 需要 {:.2}, 可用 {:.2}", total_cost, self.cash));
        }
        
        self.cash -= total_cost;
        self.shares += quantity;
        
        Ok((quantity, total_cost))
    }
    
    /// 执行卖出
    /// 返回：(实际卖出数量, 收到金额)
    pub fn sell(&mut self, price: f64, quantity: f64, commission: f64, stamp_duty: f64) -> Result<(f64, f64), String> {
        if quantity > self.shares {
            return Err(format!("持仓不足: 需要 {:.2}, 可用 {:.2}", quantity, self.shares));
        }
        
        let gross_amount = price * quantity;
        let net_amount = gross_amount - commission - stamp_duty;
        
        self.cash += net_amount;
        self.shares -= quantity;
        
        Ok((quantity, net_amount))
    }
    
    /// 更新资产状态
    pub fn update(&mut self, current_price: f64) {
        let position_value = self.shares * current_price;
        self.total_value = self.cash + position_value;
        
        // 更新峰值和回撤
        if self.total_value > self.peak_value {
            self.peak_value = self.total_value;
            self.current_drawdown = 0.0;
        } else {
            self.current_drawdown = (self.peak_value - self.total_value) / self.peak_value;
        }
        
        // 记录资金曲线
        self.equity_curve.push(self.total_value);
    }
    
    /// 记录持仓信息
    pub fn record_position(&mut self, date: String, signal: String, price: f64) {
        let position_value = self.shares * price;
        let total_value = self.cash + position_value;
        
        let daily_return = if self.positions.is_empty() {
            0.0
        } else {
            let prev_value = self.positions.last().unwrap().total_value;
            if prev_value > 0.0 {
                (total_value - prev_value) / prev_value
            } else {
                0.0
            }
        };
        
        let cumulative_return = if self.initial_capital > 0.0 {
            (total_value - self.initial_capital) / self.initial_capital
        } else {
            0.0
        };
        
        self.positions.push(Position {
            date,
            signal,
            price,
            shares: self.shares,
            position_value,
            cash: self.cash,
            total_value,
            daily_return,
            cumulative_return,
            drawdown: self.current_drawdown,
        });
    }
    
    /// 获取总收益率
    pub fn total_return(&self) -> f64 {
        if self.initial_capital > 0.0 {
            (self.total_value - self.initial_capital) / self.initial_capital
        } else {
            0.0
        }
    }
    
    /// 获取当前持仓数量
    pub fn get_shares(&self) -> f64 {
        self.shares
    }
    
    /// 获取当前现金
    pub fn get_cash(&self) -> f64 {
        self.cash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_portfolio_creation() {
        let portfolio = Portfolio::new(100000.0);
        assert_eq!(portfolio.cash, 100000.0);
        assert_eq!(portfolio.shares, 0.0);
        assert_eq!(portfolio.initial_capital, 100000.0);
    }
    
    #[test]
    fn test_buy_operation() {
        let mut portfolio = Portfolio::new(100000.0);
        let result = portfolio.buy(10.0, 100.0, 3.0);
        
        assert!(result.is_ok());
        let (qty, cost) = result.unwrap();
        assert_eq!(qty, 100.0);
        assert_eq!(cost, 1003.0);
        assert_eq!(portfolio.shares, 100.0);
        assert_eq!(portfolio.cash, 98997.0);
    }
    
    #[test]
    fn test_buy_insufficient_cash() {
        let mut portfolio = Portfolio::new(1000.0);
        let result = portfolio.buy(100.0, 100.0, 3.0);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_sell_operation() {
        let mut portfolio = Portfolio::new(100000.0);
        portfolio.buy(10.0, 100.0, 3.0).unwrap();
        
        let result = portfolio.sell(11.0, 100.0, 3.3, 1.1);
        assert!(result.is_ok());
        
        let (qty, amount) = result.unwrap();
        assert_eq!(qty, 100.0);
        assert_eq!(portfolio.shares, 0.0);
        assert!((portfolio.cash - 99989.6).abs() < 0.01);
    }
    
    #[test]
    fn test_update_and_drawdown() {
        let mut portfolio = Portfolio::new(100000.0);
        portfolio.buy(10.0, 1000.0, 30.0).unwrap();
        
        // 价格上涨
        portfolio.update(12.0);
        assert_eq!(portfolio.total_value, 12000.0 + portfolio.cash);
        assert!(portfolio.current_drawdown < 0.01);
        
        // 价格下跌
        portfolio.update(8.0);
        assert!(portfolio.current_drawdown > 0.0);
    }
}
