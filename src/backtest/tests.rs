// 回测模块集成测试

use dplang::{
    lexer::Lexer,
    parser::Parser,
    backtest::{BacktestConfig, BacktestEngine},
};
use std::collections::HashMap;
use dplang::runtime::Value;

#[test]
fn test_basic_backtest() {
    // 准备测试数据
    let mut test_data = Vec::new();
    for i in 0..50 {
        let mut row = HashMap::new();
        row.insert("date".to_string(), Value::String(format!("2024-01-{:02}", i + 1)));
        row.insert("close".to_string(), Value::Number(100.0 + i as f64 * 0.5));
        row.insert("volume".to_string(), Value::Number(1000000.0));
        test_data.push(row);
    }
    
    // 准备策略脚本
    let script_source = r#"
-- INPUT date:string, close:number, volume:number --
-- OUTPUT date:string, close:number, ma5:number, signal:string --

ma5 = MA(close, 5)
prev_ma5 = ma5[-1]

signal = (prev_ma5 != null and ma5 > prev_ma5) ? "买入" :
         (prev_ma5 != null and ma5 < prev_ma5) ? "卖出" : "持有"

return [date, close, ma5, signal]
"#;
    
    // 解析脚本
    let mut lexer = Lexer::new(script_source);
    let tokens = lexer.tokenize().expect("词法分析失败");
    let mut parser = Parser::new(tokens);
    let script = parser.parse().expect("语法分析失败");
    
    // 创建回测配置
    let config = BacktestConfig::new()
        .with_initial_capital(100000.0)
        .with_commission_rate(0.0003)
        .with_output_dir("./test_output".to_string());
    
    // 运行回测
    let mut engine = BacktestEngine::new(config);
    let result = engine.run(script, test_data, "测试策略", "test_data.csv")
        .expect("回测失败");
    
    // 验证结果
    assert!(result.metrics.return_metrics.total_return != 0.0 || result.trades.is_empty());
    assert!(result.positions.len() > 0);
    assert_eq!(result.basic_info.strategy_name, "测试策略");
    
    println!("回测测试通过!");
    println!("总收益率: {:.2}%", result.metrics.return_metrics.total_return * 100.0);
    println!("交易次数: {}", result.trades.len());
}

#[test]
fn test_performance_metrics() {
    use dplang::backtest::PerformanceCalculator;
    
    // 创建模拟的资金曲线
    let equity_curve = vec![
        100000.0, 101000.0, 102000.0, 101500.0, 103000.0,
        104000.0, 103500.0, 105000.0, 106000.0, 107000.0,
    ];
    
    let calculator = PerformanceCalculator::new(
        equity_curve.clone(),
        100000.0,
        0.03,
    );
    
    // 测试收益指标
    let return_metrics = calculator.calculate_return_metrics(None);
    assert!(return_metrics.total_return > 0.0);
    assert!(return_metrics.annual_return > 0.0);
    
    // 测试风险指标
    let risk_metrics = calculator.calculate_risk_metrics();
    assert!(risk_metrics.max_drawdown >= 0.0);
    assert!(risk_metrics.annual_volatility >= 0.0);
    
    println!("性能指标计算测试通过!");
    println!("总收益: {:.2}%", return_metrics.total_return * 100.0);
    println!("最大回撤: {:.2}%", risk_metrics.max_drawdown * 100.0);
}
