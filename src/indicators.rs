// 技术指标库 - 金融数据分析常用指标

use crate::runtime::{Value, RuntimeError};

/// 简单移动平均线 (Simple Moving Average)
/// 返回当前行的 MA 值
pub fn sma(values: &[Value], period: usize) -> Result<Value, RuntimeError> {
    if values.len() < period {
        return Ok(Value::Null);
    }
    
    let mut sum = 0.0;
    let mut count = 0;
    
    // 计算窗口内的有效值和
    for i in (values.len() - period)..values.len() {
        if !values[i].is_null() {
            sum += values[i].to_number()?;
            count += 1;
        }
    }
    
    // 如果没有有效数据，返回 null
    if count == 0 {
        return Ok(Value::Null);
    }
    
    // 使用实际有效数据个数计算平均值
    Ok(Value::Number(sum / count as f64))
}

/// 指数移动平均线 (Exponential Moving Average)
/// 返回当前行的 EMA 值
pub fn ema(values: &[Value], period: usize) -> Result<Value, RuntimeError> {
    if values.is_empty() {
        return Ok(Value::Null);
    }
    
    // 过滤掉 null 值
    let valid_values: Vec<f64> = values.iter()
        .filter(|v| !v.is_null())
        .map(|v| v.to_number())
        .collect::<Result<Vec<_>, _>>()?;
    
    if valid_values.len() < period {
        // 有效数据不足，返回简单平均
        if valid_values.is_empty() {
            return Ok(Value::Null);
        }
        let sum: f64 = valid_values.iter().sum();
        return Ok(Value::Number(sum / valid_values.len() as f64));
    }
    
    // EMA 平滑系数
    let k = 2.0 / (period as f64 + 1.0);
    
    // 初始 EMA 使用 SMA
    let mut ema_value = 0.0;
    for i in 0..period {
        ema_value += valid_values[i];
    }
    ema_value /= period as f64;
    
    // 递推计算 EMA
    for i in period..valid_values.len() {
        let price = valid_values[i];
        ema_value = price * k + ema_value * (1.0 - k);
    }
    
    Ok(Value::Number(ema_value))
}

/// MACD 指标 (Moving Average Convergence Divergence)
/// 返回 [MACD, Signal, Histogram]
pub fn macd(
    prices: &[Value],
    fast_period: usize,
    slow_period: usize,
    _signal_period: usize,
) -> Result<Value, RuntimeError> {
    if prices.len() < slow_period {
        return Ok(Value::Array(vec![Value::Null, Value::Null, Value::Null]));
    }
    
    // 计算快线和慢线 EMA
    let fast_ema = ema(prices, fast_period)?.to_number()?;
    let slow_ema = ema(prices, slow_period)?.to_number()?;
    
    // MACD = 快线 - 慢线
    let macd_value = fast_ema - slow_ema;
    
    // Signal 线（MACD 的 EMA）
    // 这里简化处理，实际应该对 MACD 序列计算 EMA
    let signal = macd_value; // 简化版本
    
    // Histogram = MACD - Signal
    let histogram = macd_value - signal;
    
    Ok(Value::Array(vec![
        Value::Number(macd_value),
        Value::Number(signal),
        Value::Number(histogram),
    ]))
}

/// RSI 指标 (Relative Strength Index)
/// 返回当前行的 RSI 值
pub fn rsi(prices: &[Value], period: usize) -> Result<Value, RuntimeError> {
    if prices.len() < period + 1 {
        return Ok(Value::Null);
    }
    
    let mut gains = 0.0;
    let mut losses = 0.0;
    let mut count = 0;
    
    // 计算价格变化，跳过 null 值
    for i in (prices.len() - period)..prices.len() {
        if !prices[i].is_null() && !prices[i - 1].is_null() {
            let prev = prices[i - 1].to_number()?;
            let curr = prices[i].to_number()?;
            let change = curr - prev;
            
            if change > 0.0 {
                gains += change;
            } else {
                losses += -change;
            }
            count += 1;
        }
    }
    
    // 有效数据不足
    if count == 0 {
        return Ok(Value::Null);
    }
    
    // 平均涨跌
    let avg_gain = gains / count as f64;
    let avg_loss = losses / count as f64;
    
    if avg_loss == 0.0 {
        return Ok(Value::Number(100.0));
    }
    
    // RSI = 100 - (100 / (1 + RS))
    let rs = avg_gain / avg_loss;
    let rsi_value = 100.0 - (100.0 / (1.0 + rs));
    
    Ok(Value::Number(rsi_value))
}

/// 布林带 (Bollinger Bands)
/// 返回 [上轨, 中轨, 下轨]
pub fn bollinger_bands(
    prices: &[Value],
    period: usize,
    std_dev: f64,
) -> Result<Value, RuntimeError> {
    if prices.len() < period {
        return Ok(Value::Array(vec![Value::Null, Value::Null, Value::Null]));
    }
    
    // 中轨 = SMA
    let middle_value = sma(prices, period)?;
    if middle_value.is_null() {
        return Ok(Value::Array(vec![Value::Null, Value::Null, Value::Null]));
    }
    let middle = middle_value.to_number()?;
    
    // 计算标准差（跳过 null 值）
    let mut variance = 0.0;
    let mut count = 0;
    for i in (prices.len() - period)..prices.len() {
        if !prices[i].is_null() {
            let price = prices[i].to_number()?;
            let diff = price - middle;
            variance += diff * diff;
            count += 1;
        }
    }
    
    if count == 0 {
        return Ok(Value::Array(vec![Value::Null, Value::Null, Value::Null]));
    }
    
    variance /= count as f64;
    let std = variance.sqrt();
    
    // 上轨 = 中轨 + N * 标准差
    let upper = middle + std_dev * std;
    // 下轨 = 中轨 - N * 标准差
    let lower = middle - std_dev * std;
    
    Ok(Value::Array(vec![
        Value::Number(upper),
        Value::Number(middle),
        Value::Number(lower),
    ]))
}

/// ATR 指标 (Average True Range)
pub fn atr(
    high: &[Value],
    low: &[Value],
    close: &[Value],
    period: usize,
) -> Result<Value, RuntimeError> {
    if high.len() < period + 1 || low.len() < period + 1 || close.len() < period + 1 {
        return Ok(Value::Null);
    }
    
    let mut tr_sum = 0.0;
    let mut count = 0;
    
    for i in (high.len() - period)..high.len() {
        // 跳过包含 null 的数据
        if !high[i].is_null() && !low[i].is_null() && !close[i - 1].is_null() {
            let h = high[i].to_number()?;
            let l = low[i].to_number()?;
            let c_prev = close[i - 1].to_number()?;
            
            // TR = max(H-L, |H-C_prev|, |L-C_prev|)
            let tr = (h - l)
                .max((h - c_prev).abs())
                .max((l - c_prev).abs());
            
            tr_sum += tr;
            count += 1;
        }
    }
    
    if count == 0 {
        return Ok(Value::Null);
    }
    
    Ok(Value::Number(tr_sum / count as f64))
}

/// KDJ 指标
/// 返回 [K, D, J]
pub fn kdj(
    high: &[Value],
    low: &[Value],
    close: &[Value],
    n: usize,
    _m1: usize,
    _m2: usize,
) -> Result<Value, RuntimeError> {
    if high.len() < n || low.len() < n || close.len() < n {
        return Ok(Value::Array(vec![Value::Null, Value::Null, Value::Null]));
    }
    
    // 找出 N 日内的最高价和最低价（跳过 null 值）
    let mut highest = f64::MIN;
    let mut lowest = f64::MAX;
    let mut has_valid_data = false;
    
    for i in (high.len() - n)..high.len() {
        if !high[i].is_null() && !low[i].is_null() {
            let h = high[i].to_number()?;
            let l = low[i].to_number()?;
            highest = highest.max(h);
            lowest = lowest.min(l);
            has_valid_data = true;
        }
    }
    
    if !has_valid_data || close[close.len() - 1].is_null() {
        return Ok(Value::Array(vec![Value::Null, Value::Null, Value::Null]));
    }
    
    let c = close[close.len() - 1].to_number()?;
    
    // RSV = (C - Ln) / (Hn - Ln) * 100
    let rsv = if highest == lowest {
        50.0
    } else {
        (c - lowest) / (highest - lowest) * 100.0
    };
    
    // K = SMA(RSV, m1)
    // D = SMA(K, m2)
    // 这里简化处理
    let k = rsv;
    let d = k;
    let j = 3.0 * k - 2.0 * d;
    
    Ok(Value::Array(vec![
        Value::Number(k),
        Value::Number(d),
        Value::Number(j),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma() {
        let prices = vec![
            Value::Number(10.0),
            Value::Number(11.0),
            Value::Number(12.0),
            Value::Number(13.0),
            Value::Number(14.0),
        ];
        
        let result = sma(&prices, 3).unwrap();
        assert_eq!(result.to_number().unwrap(), 13.0); // (12+13+14)/3
    }

    #[test]
    fn test_ema() {
        let prices = vec![
            Value::Number(10.0),
            Value::Number(11.0),
            Value::Number(12.0),
            Value::Number(13.0),
            Value::Number(14.0),
        ];
        
        let result = ema(&prices, 3).unwrap();
        let ema_value = result.to_number().unwrap();
        // EMA 应该在 SMA(13.0) 和最新价格(14.0) 之间
        assert!(ema_value >= 13.0 && ema_value <= 14.0);
    }

    #[test]
    fn test_rsi() {
        let prices = vec![
            Value::Number(44.0),
            Value::Number(45.0),
            Value::Number(46.0),
            Value::Number(47.0),
            Value::Number(48.0),
            Value::Number(49.0),
        ];
        
        let result = rsi(&prices, 3).unwrap();
        let rsi_value = result.to_number().unwrap();
        // 持续上涨，RSI 应该接近 100
        assert!(rsi_value > 90.0);
    }

    #[test]
    fn test_bollinger_bands() {
        let prices = vec![
            Value::Number(10.0),
            Value::Number(11.0),
            Value::Number(12.0),
            Value::Number(13.0),
            Value::Number(14.0),
        ];
        
        let result = bollinger_bands(&prices, 3, 2.0).unwrap();
        if let Value::Array(bands) = result {
            assert_eq!(bands.len(), 3);
            let upper = bands[0].to_number().unwrap();
            let middle = bands[1].to_number().unwrap();
            let lower = bands[2].to_number().unwrap();
            
            assert!(upper > middle);
            assert!(middle > lower);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_atr() {
        let high = vec![
            Value::Number(15.0),
            Value::Number(16.0),
            Value::Number(17.0),
            Value::Number(18.0),
        ];
        let low = vec![
            Value::Number(10.0),
            Value::Number(11.0),
            Value::Number(12.0),
            Value::Number(13.0),
        ];
        let close = vec![
            Value::Number(12.0),
            Value::Number(13.0),
            Value::Number(14.0),
            Value::Number(15.0),
        ];
        
        let result = atr(&high, &low, &close, 2).unwrap();
        assert!(result.to_number().unwrap() > 0.0);
    }
    
    #[test]
    fn test_sma_with_null() {
        // 测试 SMA 正确处理 null 值
        let prices = vec![
            Value::Number(10.0),
            Value::Null,           // null 应该被跳过
            Value::Number(12.0),
            Value::Number(13.0),
            Value::Number(14.0),
        ];
        
        // 窗口期为 3，最后 3 个值是 [12, 13, 14]（没有null）
        let result = sma(&prices, 3).unwrap();
        assert_eq!(result.to_number().unwrap(), 13.0); // (12+13+14)/3
    }
    
    #[test]
    fn test_sma_all_null() {
        // 测试当窗口内全是 null 时返回 null
        let prices = vec![
            Value::Number(10.0),
            Value::Null,
            Value::Null,
            Value::Null,
        ];
        
        let result = sma(&prices, 3).unwrap();
        assert!(result.is_null());
    }
    
    #[test]
    fn test_rsi_with_null() {
        // 测试 RSI 正确处理 null 值
        let prices = vec![
            Value::Number(44.0),
            Value::Null,           // null 应该被跳过
            Value::Number(45.0),
            Value::Number(46.0),
            Value::Number(47.0),
            Value::Number(48.0),
        ];
        
        let result = rsi(&prices, 3).unwrap();
        // 应该能正常计算，跳过 null
        assert!(result.to_number().is_ok());
    }
    
    #[test]
    fn test_null_conversion_error() {
        // 测试系统层面 null 的语义明确性
        let null_value = Value::Null;
        
        // null 直接转换为数字应该报错
        assert!(null_value.to_number().is_err());
        
        // null 使用默认值转换应该成功
        assert_eq!(null_value.to_number_or_default(0.0).unwrap(), 0.0);
        
        // is_null 检查
        assert!(null_value.is_null());
        assert!(!Value::Number(0.0).is_null());
    }
}