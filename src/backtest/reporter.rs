// 报告生成器模块
use std::fs;
use std::path::Path;
use serde_json;
use super::engine::BacktestResult;
use super::Trade;
use super::portfolio::Position;

/// 报告生成器
pub struct Reporter {
    output_dir: String,
}

impl Reporter {
    /// 创建报告生成器
    pub fn new(output_dir: String) -> Self {
        Self { output_dir }
    }
    
    /// 生成所有报告
    pub fn generate_all(&self, result: &BacktestResult) -> Result<(), String> {
        // 创建输出目录
        fs::create_dir_all(&self.output_dir)
            .map_err(|e| format!("创建输出目录失败: {}", e))?;
        
        // 生成各种格式的报告
        self.generate_summary_json(result)?;
        self.generate_summary_text(result)?;
        self.generate_trades_csv(&result.trades)?;
        self.generate_positions_csv(&result.positions)?;
        self.generate_daily_stats_csv(result)?;
        self.generate_equity_curve_csv(&result.equity_curve)?;
        
        Ok(())
    }
    
    /// 生成JSON摘要
    pub fn generate_summary_json(&self, result: &BacktestResult) -> Result<(), String> {
        let summary = serde_json::json!({
            "basic_info": {
                "strategy": result.basic_info.strategy_name,
                "data_file": result.basic_info.data_file,
                "start_date": result.basic_info.start_date,
                "end_date": result.basic_info.end_date,
                "trading_days": result.basic_info.trading_days,
                "initial_capital": result.basic_info.initial_capital,
            },
            "return_metrics": {
                "total_return": result.metrics.return_metrics.total_return,
                "annual_return": result.metrics.return_metrics.annual_return,
                "benchmark_return": result.metrics.return_metrics.benchmark_return,
                "excess_return": result.metrics.return_metrics.excess_return,
            },
            "risk_metrics": {
                "max_drawdown": result.metrics.risk_metrics.max_drawdown,
                "max_drawdown_duration": result.metrics.risk_metrics.max_drawdown_duration,
                "annual_volatility": result.metrics.risk_metrics.annual_volatility,
                "downside_volatility": result.metrics.risk_metrics.downside_volatility,
                "var_95": result.metrics.risk_metrics.var_95,
            },
            "performance_metrics": {
                "sharpe_ratio": result.metrics.performance_ratios.sharpe_ratio,
                "sortino_ratio": result.metrics.performance_ratios.sortino_ratio,
                "calmar_ratio": result.metrics.performance_ratios.calmar_ratio,
                "information_ratio": result.metrics.performance_ratios.information_ratio,
                "win_rate": result.metrics.performance_ratios.win_rate,
                "profit_loss_ratio": result.metrics.performance_ratios.profit_loss_ratio,
            },
            "trade_statistics": {
                "total_trades": result.metrics.trade_statistics.total_trades,
                "winning_trades": result.metrics.trade_statistics.winning_trades,
                "losing_trades": result.metrics.trade_statistics.losing_trades,
                "avg_holding_days": result.metrics.trade_statistics.avg_holding_days,
                "max_holding_days": result.metrics.trade_statistics.max_holding_days,
                "avg_profit": result.metrics.trade_statistics.avg_profit,
                "avg_loss": result.metrics.trade_statistics.avg_loss,
                "max_profit": result.metrics.trade_statistics.max_profit,
                "max_loss": result.metrics.trade_statistics.max_loss,
                "max_consecutive_wins": result.metrics.trade_statistics.max_consecutive_wins,
                "max_consecutive_losses": result.metrics.trade_statistics.max_consecutive_losses,
            }
        });
        
        let json_str = serde_json::to_string_pretty(&summary)
            .map_err(|e| format!("JSON序列化失败: {}", e))?;
        
        let file_path = Path::new(&self.output_dir).join("summary.json");
        fs::write(file_path, json_str)
            .map_err(|e| format!("写入summary.json失败: {}", e))?;
        
        Ok(())
    }
    
    /// 生成文本摘要
    pub fn generate_summary_text(&self, result: &BacktestResult) -> Result<(), String> {
        let mut text = String::new();
        
        text.push_str("═══════════════════════════════════════════════════════════\n");
        text.push_str("                    回测报告摘要\n");
        text.push_str("═══════════════════════════════════════════════════════════\n\n");
        
        // 基本信息
        text.push_str("【基本信息】\n");
        text.push_str(&format!("  策略名称: {}\n", result.basic_info.strategy_name));
        text.push_str(&format!("  数据文件: {}\n", result.basic_info.data_file));
        text.push_str(&format!("  回测区间: {} 至 {}\n", result.basic_info.start_date, result.basic_info.end_date));
        text.push_str(&format!("  交易天数: {}天\n", result.basic_info.trading_days));
        text.push_str(&format!("  初始资金: {:.2}\n\n", result.basic_info.initial_capital));
        
        // 收益指标
        text.push_str("【收益指标】\n");
        text.push_str(&format!("  总收益率:          {:.2}%\n", result.metrics.return_metrics.total_return * 100.0));
        text.push_str(&format!("  年化收益率:        {:.2}%\n", result.metrics.return_metrics.annual_return * 100.0));
        if let Some(bench) = result.metrics.return_metrics.benchmark_return {
            text.push_str(&format!("  基准收益率:        {:.2}%\n", bench * 100.0));
        }
        if let Some(excess) = result.metrics.return_metrics.excess_return {
            text.push_str(&format!("  超额收益:          {:.2}%\n", excess * 100.0));
        }
        text.push_str("\n");
        
        // 风险指标
        text.push_str("【风险指标】\n");
        text.push_str(&format!("  最大回撤:          {:.2}%\n", result.metrics.risk_metrics.max_drawdown * 100.0));
        text.push_str(&format!("  最大回撤持续期:    {}天\n", result.metrics.risk_metrics.max_drawdown_duration));
        text.push_str(&format!("  年化波动率:        {:.2}%\n", result.metrics.risk_metrics.annual_volatility * 100.0));
        text.push_str(&format!("  下行波动率:        {:.2}%\n", result.metrics.risk_metrics.downside_volatility * 100.0));
        text.push_str(&format!("  VaR(95%):          {:.2}%\n\n", result.metrics.risk_metrics.var_95 * 100.0));
        
        // 综合评价
        text.push_str("【综合评价】\n");
        text.push_str(&format!("  夏普比率:          {:.2}\n", result.metrics.performance_ratios.sharpe_ratio));
        text.push_str(&format!("  索提诺比率:        {:.2}\n", result.metrics.performance_ratios.sortino_ratio));
        text.push_str(&format!("  卡玛比率:          {:.2}\n", result.metrics.performance_ratios.calmar_ratio));
        if let Some(ir) = result.metrics.performance_ratios.information_ratio {
            text.push_str(&format!("  信息比率:          {:.2}\n", ir));
        }
        text.push_str(&format!("  胜率:              {:.2}%\n", result.metrics.performance_ratios.win_rate * 100.0));
        text.push_str(&format!("  盈亏比:            {:.2}\n\n", result.metrics.performance_ratios.profit_loss_ratio));
        
        // 交易统计
        text.push_str("【交易统计】\n");
        text.push_str(&format!("  总交易次数:        {}\n", result.metrics.trade_statistics.total_trades));
        text.push_str(&format!("  盈利次数:          {}\n", result.metrics.trade_statistics.winning_trades));
        text.push_str(&format!("  亏损次数:          {}\n", result.metrics.trade_statistics.losing_trades));
        text.push_str(&format!("  平均持仓天数:      {:.1}天\n", result.metrics.trade_statistics.avg_holding_days));
        text.push_str(&format!("  最长持仓:          {}天\n", result.metrics.trade_statistics.max_holding_days));
        text.push_str(&format!("  平均盈利:          {:.2}\n", result.metrics.trade_statistics.avg_profit));
        text.push_str(&format!("  平均亏损:          {:.2}\n", result.metrics.trade_statistics.avg_loss));
        text.push_str(&format!("  最大单笔盈利:      {:.2}\n", result.metrics.trade_statistics.max_profit));
        text.push_str(&format!("  最大单笔亏损:      {:.2}\n", result.metrics.trade_statistics.max_loss));
        text.push_str(&format!("  最长连续盈利:      {}次\n", result.metrics.trade_statistics.max_consecutive_wins));
        text.push_str(&format!("  最长连续亏损:      {}次\n\n", result.metrics.trade_statistics.max_consecutive_losses));
        
        text.push_str("═══════════════════════════════════════════════════════════\n");
        
        let file_path = Path::new(&self.output_dir).join("summary.txt");
        fs::write(file_path, text)
            .map_err(|e| format!("写入summary.txt失败: {}", e))?;
        
        Ok(())
    }
    
    /// 生成交易明细CSV
    pub fn generate_trades_csv(&self, trades: &[Trade]) -> Result<(), String> {
        let mut csv = String::new();
        csv.push_str("trade_id,entry_date,entry_price,entry_signal,exit_date,exit_price,exit_signal,shares,holding_days,gross_profit,commission,stamp_duty,net_profit,return_rate\n");
        
        for trade in trades {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                trade.trade_id,
                trade.entry_date,
                trade.entry_price,
                trade.entry_signal,
                trade.exit_date,
                trade.exit_price,
                trade.exit_signal,
                trade.shares,
                trade.holding_days,
                trade.gross_profit,
                trade.commission,
                trade.stamp_duty,
                trade.net_profit,
                trade.return_rate
            ));
        }
        
        let file_path = Path::new(&self.output_dir).join("trades.csv");
        fs::write(file_path, csv)
            .map_err(|e| format!("写入trades.csv失败: {}", e))?;
        
        Ok(())
    }
    
    /// 生成持仓记录CSV
    pub fn generate_positions_csv(&self, positions: &[Position]) -> Result<(), String> {
        let mut csv = String::new();
        csv.push_str("date,signal,price,shares,position_value,cash,total_value,daily_return,cumulative_return,drawdown\n");
        
        for pos in positions {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{}\n",
                pos.date,
                pos.signal,
                pos.price,
                pos.shares,
                pos.position_value,
                pos.cash,
                pos.total_value,
                pos.daily_return,
                pos.cumulative_return,
                pos.drawdown
            ));
        }
        
        let file_path = Path::new(&self.output_dir).join("positions.csv");
        fs::write(file_path, csv)
            .map_err(|e| format!("写入positions.csv失败: {}", e))?;
        
        Ok(())
    }
    
    /// 生成每日统计CSV
    pub fn generate_daily_stats_csv(&self, result: &BacktestResult) -> Result<(), String> {
        let mut csv = String::new();
        csv.push_str("date,total_value,daily_return,cumulative_return,drawdown\n");
        
        for pos in &result.positions {
            csv.push_str(&format!(
                "{},{},{},{},{}\n",
                pos.date,
                pos.total_value,
                pos.daily_return,
                pos.cumulative_return,
                pos.drawdown
            ));
        }
        
        let file_path = Path::new(&self.output_dir).join("daily_stats.csv");
        fs::write(file_path, csv)
            .map_err(|e| format!("写入daily_stats.csv失败: {}", e))?;
        
        Ok(())
    }
    
    /// 生成资金曲线CSV
    pub fn generate_equity_curve_csv(&self, equity_curve: &[f64]) -> Result<(), String> {
        let mut csv = String::new();
        csv.push_str("day,total_value\n");
        
        for (i, value) in equity_curve.iter().enumerate() {
            csv.push_str(&format!("{},{}\n", i, value));
        }
        
        let file_path = Path::new(&self.output_dir).join("equity_curve.csv");
        fs::write(file_path, csv)
            .map_err(|e| format!("写入equity_curve.csv失败: {}", e))?;
        
        Ok(())
    }
    
    /// 生成控制台快速摘要
    pub fn print_quick_summary(result: &BacktestResult) {
        println!("\n📊 快速摘要");
        println!("  总收益:    {:.2}%", result.metrics.return_metrics.total_return * 100.0);
        println!("  最大回撤:  {:.2}%", result.metrics.risk_metrics.max_drawdown * 100.0);
        println!("  夏普比率:  {:.2}", result.metrics.performance_ratios.sharpe_ratio);
        println!("  胜率:      {:.2}%", result.metrics.performance_ratios.win_rate * 100.0);
    }
}
