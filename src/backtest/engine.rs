// 回测引擎核心模块
use std::collections::HashMap;
use crate::executor::DataStreamExecutor;
use crate::parser::ast::Script;
use crate::runtime::Value;
#[allow(unused_imports)]
use super::{
    BacktestConfig, Portfolio, TradeTracker, Trade,
    PerformanceMetrics, ReturnMetrics, RiskMetrics, PerformanceRatios, TradeStatistics,
    PerformanceCalculator,
};

/// 回测结果
#[derive(Debug)]
pub struct BacktestResult {
    /// 性能指标
    pub metrics: PerformanceMetrics,
    
    /// 交易记录
    pub trades: Vec<Trade>,
    
    /// 持仓记录
    pub positions: Vec<super::portfolio::Position>,
    
    /// 资金曲线
    pub equity_curve: Vec<f64>,
    
    /// 基本信息
    pub basic_info: BasicInfo,
}

/// 基本信息
#[derive(Debug, Clone)]
pub struct BasicInfo {
    pub strategy_name: String,
    pub data_file: String,
    pub start_date: String,
    pub end_date: String,
    pub trading_days: usize,
    pub initial_capital: f64,
}

/// 回测引擎
pub struct BacktestEngine {
    config: BacktestConfig,
    portfolio: Portfolio,
    trade_tracker: TradeTracker,
    row_counter: usize,
}

impl BacktestEngine {
    /// 创建回测引擎
    pub fn new(config: BacktestConfig) -> Self {
        let portfolio = Portfolio::new(config.initial_capital);
        let trade_tracker = TradeTracker::new();
        
        Self {
            config,
            portfolio,
            trade_tracker,
            row_counter: 0,
        }
    }
    
    /// 执行回测
    pub fn run(
        &mut self,
        script: Script,
        input_data: Vec<HashMap<String, Value>>,
        strategy_name: &str,
        data_file: &str,
    ) -> Result<BacktestResult, String> {
        if input_data.is_empty() {
            return Err("输入数据为空".to_string());
        }
        
        // 提取开始和结束日期
        let start_date = self.extract_date(&input_data[0]);
        let end_date = self.extract_date(&input_data[input_data.len() - 1]);
        
        // 执行策略脚本获取所有信号
        let mut executor = DataStreamExecutor::new(script, input_data.clone());
        let output_data = executor.execute_all()
            .map_err(|e| format!("策略执行失败: {:?}", e))?;
        
        // 逐行处理回测
        for (i, row) in output_data.iter().enumerate() {
            if i >= input_data.len() {
                break;
            }
            
            let input_row = &input_data[i];
            self.process_row(row, input_row, i)?;
        }
        
        // 计算性能指标
        let metrics = self.calculate_metrics();
        
        // 构建基本信息
        let basic_info = BasicInfo {
            strategy_name: strategy_name.to_string(),
            data_file: data_file.to_string(),
            start_date,
            end_date,
            trading_days: self.row_counter,
            initial_capital: self.config.initial_capital,
        };
        
        Ok(BacktestResult {
            metrics,
            trades: self.trade_tracker.completed_trades.clone(),
            positions: self.portfolio.positions.clone(),
            equity_curve: self.portfolio.equity_curve.clone(),
            basic_info,
        })
    }
    
    /// 处理单行数据
    fn process_row(
        &mut self,
        output_row: &HashMap<String, Value>,
        input_row: &HashMap<String, Value>,
        index: usize,
    ) -> Result<(), String> {
        self.row_counter += 1;
        
        // 提取价格
        let price = self.extract_price(input_row)?;
        
        // 提取信号
        let signal = self.extract_signal(output_row);
        
        // 提取日期
        let date = self.extract_date(input_row);
        
        // 执行交易逻辑
        self.execute_signal(&signal, price, &date, index)?;
        
        // 更新资产状态
        self.portfolio.update(price);
        
        // 记录持仓
        self.portfolio.record_position(date, signal, price);
        
        Ok(())
    }
    
    /// 执行信号
    fn execute_signal(&mut self, signal: &str, price: f64, date: &str, _index: usize) -> Result<(), String> {
        match signal {
            "买入" | "buy" => self.execute_buy(price, date)?,
            "卖出" | "sell" => self.execute_sell(price, date)?,
            _ => {}, // 持有或其他信号，不操作
        }
        Ok(())
    }
    
    /// 执行买入
    fn execute_buy(&mut self, price: f64, date: &str) -> Result<(), String> {
        // 如果已有持仓，不重复买入
        if self.trade_tracker.has_position() {
            return Ok(());
        }
        
        // 计算买入价格（含滑点）
        let buy_price = price * (1.0 + self.config.slippage_rate);
        
        // 计算可买金额
        let available_cash = self.portfolio.get_cash();
        let max_amount = available_cash * self.config.position_limit;
        
        // 计算买入数量
        let shares = (max_amount / buy_price).floor();
        if shares <= 0.0 {
            return Ok(()); // 资金不足，忽略信号
        }
        
        let amount = shares * buy_price;
        
        // 计算手续费
        let commission = (amount * self.config.commission_rate).max(self.config.min_commission);
        
        // 检查现金是否足够
        if amount + commission > available_cash {
            // 调整买入数量
            let adjusted_shares = ((available_cash - self.config.min_commission) / buy_price / (1.0 + self.config.commission_rate)).floor();
            if adjusted_shares <= 0.0 {
                return Ok(()); // 资金不足
            }
            return self.execute_buy_with_shares(adjusted_shares, buy_price, date);
        }
        
        // 执行买入
        match self.portfolio.buy(buy_price, shares, commission) {
            Ok(_) => {
                self.trade_tracker.record_entry(
                    date.to_string(),
                    buy_price,
                    "买入".to_string(),
                    shares,
                    commission,
                );
                Ok(())
            }
            Err(e) => {
                if self.config.output_config.verbose {
                    eprintln!("⚠️  买入失败: {}", e);
                }
                Ok(())
            }
        }
    }
    
    /// 执行指定数量的买入
    fn execute_buy_with_shares(&mut self, shares: f64, price: f64, date: &str) -> Result<(), String> {
        let amount = shares * price;
        let commission = (amount * self.config.commission_rate).max(self.config.min_commission);
        
        match self.portfolio.buy(price, shares, commission) {
            Ok(_) => {
                self.trade_tracker.record_entry(
                    date.to_string(),
                    price,
                    "买入".to_string(),
                    shares,
                    commission,
                );
                Ok(())
            }
            Err(e) => {
                if self.config.output_config.verbose {
                    eprintln!("⚠️  买入失败: {}", e);
                }
                Ok(())
            }
        }
    }
    
    /// 执行卖出
    fn execute_sell(&mut self, price: f64, date: &str) -> Result<(), String> {
        // 如果没有持仓，不能卖出
        if !self.trade_tracker.has_position() {
            return Ok(());
        }
        
        let shares = self.portfolio.get_shares();
        if shares <= 0.0 {
            return Ok(());
        }
        
        // 计算卖出价格（含滑点）
        let sell_price = price * (1.0 - self.config.slippage_rate);
        
        let amount = shares * sell_price;
        
        // 计算手续费和印花税
        let commission = (amount * self.config.commission_rate).max(self.config.min_commission);
        let stamp_duty = amount * self.config.stamp_duty_rate;
        
        // 执行卖出
        match self.portfolio.sell(sell_price, shares, commission, stamp_duty) {
            Ok(_) => {
                self.trade_tracker.record_exit(
                    date.to_string(),
                    sell_price,
                    "卖出".to_string(),
                    commission,
                    stamp_duty,
                );
                Ok(())
            }
            Err(e) => {
                if self.config.output_config.verbose {
                    eprintln!("⚠️  卖出失败: {}", e);
                }
                Ok(())
            }
        }
    }
    
    /// 计算性能指标
    fn calculate_metrics(&self) -> PerformanceMetrics {
        let calculator = PerformanceCalculator::new(
            self.portfolio.equity_curve.clone(),
            self.config.initial_capital,
            self.config.risk_free_rate,
        );
        
        let return_metrics = calculator.calculate_return_metrics(None);
        let risk_metrics = calculator.calculate_risk_metrics();
        
        let trade_statistics = TradeStatistics {
            total_trades: self.trade_tracker.total_trades(),
            winning_trades: self.trade_tracker.winning_trades(),
            losing_trades: self.trade_tracker.losing_trades(),
            avg_holding_days: self.calculate_avg_holding_days(),
            max_holding_days: self.calculate_max_holding_days(),
            avg_profit: self.trade_tracker.average_profit(),
            avg_loss: self.trade_tracker.average_loss(),
            max_profit: self.trade_tracker.max_profit(),
            max_loss: self.trade_tracker.max_loss(),
            max_consecutive_wins: self.trade_tracker.max_consecutive_wins(),
            max_consecutive_losses: self.trade_tracker.max_consecutive_losses(),
        };
        
        let performance_ratios = calculator.calculate_performance_ratios(
            &return_metrics,
            &risk_metrics,
            &trade_statistics,
            None,
        );
        
        PerformanceMetrics {
            return_metrics,
            risk_metrics,
            performance_ratios,
            trade_statistics,
        }
    }
    
    /// 计算平均持仓天数
    fn calculate_avg_holding_days(&self) -> f64 {
        if self.trade_tracker.completed_trades.is_empty() {
            return 0.0;
        }
        
        let total: usize = self.trade_tracker.completed_trades
            .iter()
            .map(|t| t.holding_days)
            .sum();
        
        total as f64 / self.trade_tracker.completed_trades.len() as f64
    }
    
    /// 计算最长持仓天数
    fn calculate_max_holding_days(&self) -> usize {
        self.trade_tracker.completed_trades
            .iter()
            .map(|t| t.holding_days)
            .max()
            .unwrap_or(0)
    }
    
    /// 提取价格
    fn extract_price(&self, row: &HashMap<String, Value>) -> Result<f64, String> {
        // 优先使用 close 字段
        if let Some(Value::Number(price)) = row.get("close") {
            return Ok(*price);
        }
        
        // 尝试 price 字段
        if let Some(Value::Number(price)) = row.get("price") {
            return Ok(*price);
        }
        
        Err("数据中缺少价格字段 (close 或 price)".to_string())
    }
    
    /// 提取信号
    fn extract_signal(&self, row: &HashMap<String, Value>) -> String {
        if let Some(value) = row.get("signal") {
            match value {
                Value::String(s) => s.clone(),
                Value::Number(n) if *n > 0.0 => "买入".to_string(),
                Value::Number(n) if *n < 0.0 => "卖出".to_string(),
                _ => "持有".to_string(),
            }
        } else {
            "持有".to_string()
        }
    }
    
    /// 提取日期
    fn extract_date(&self, row: &HashMap<String, Value>) -> String {
        if let Some(Value::String(date)) = row.get("date") {
            date.clone()
        } else if let Some(Value::String(date)) = row.get("time") {
            date.clone()
        } else {
            format!("Day-{}", self.row_counter)
        }
    }
}
