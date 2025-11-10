use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use dplang::runtime::Value;
use dplang::indicators::*;

fn benchmark_sma(c: &mut Criterion) {
    let mut group = c.benchmark_group("SMA");
    
    for size in [10, 50, 100, 500, 1000].iter() {
        let prices: Vec<Value> = (0..*size)
            .map(|i| Value::Number(100.0 + (i as f64 % 10.0)))
            .collect();
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                sma(black_box(&prices), black_box(20))
            });
        });
    }
    group.finish();
}

fn benchmark_ema(c: &mut Criterion) {
    let mut group = c.benchmark_group("EMA");
    
    for size in [10, 50, 100, 500, 1000].iter() {
        let prices: Vec<Value> = (0..*size)
            .map(|i| Value::Number(100.0 + (i as f64 % 10.0)))
            .collect();
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                ema(black_box(&prices), black_box(20))
            });
        });
    }
    group.finish();
}

fn benchmark_macd(c: &mut Criterion) {
    let mut group = c.benchmark_group("MACD");
    
    for size in [50, 100, 500, 1000].iter() {
        let prices: Vec<Value> = (0..*size)
            .map(|i| Value::Number(100.0 + (i as f64 % 10.0)))
            .collect();
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                macd(black_box(&prices), black_box(12), black_box(26), black_box(9))
            });
        });
    }
    group.finish();
}

fn benchmark_rsi(c: &mut Criterion) {
    let mut group = c.benchmark_group("RSI");
    
    for size in [20, 50, 100, 500, 1000].iter() {
        let prices: Vec<Value> = (0..*size)
            .map(|i| Value::Number(100.0 + (i as f64 % 10.0)))
            .collect();
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                rsi(black_box(&prices), black_box(14))
            });
        });
    }
    group.finish();
}

fn benchmark_macd_calculator_incremental(c: &mut Criterion) {
    let mut group = c.benchmark_group("MACD_Calculator_Incremental");
    
    for size in [50, 100, 500, 1000, 5000].iter() {
        let prices: Vec<f64> = (0..*size)
            .map(|i| 100.0 + (i as f64 % 10.0))
            .collect();
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut calc = MACDCalculator::new(12, 26, 9);
                for &price in &prices {
                    black_box(calc.update(price));
                }
            });
        });
    }
    group.finish();
}

fn benchmark_kdj_calculator_incremental(c: &mut Criterion) {
    let mut group = c.benchmark_group("KDJ_Calculator_Incremental");
    
    for size in [20, 50, 100, 500, 1000].iter() {
        let data: Vec<(f64, f64, f64)> = (0..*size)
            .map(|i| {
                let base = 100.0 + (i as f64 % 10.0);
                (base + 5.0, base - 5.0, base)
            })
            .collect();
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut calc = KDJCalculator::new(9, 3, 3);
                for &(h, l, c) in &data {
                    black_box(calc.update(h, l, c));
                }
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    benchmark_sma,
    benchmark_ema,
    benchmark_macd,
    benchmark_rsi,
    benchmark_macd_calculator_incremental,
    benchmark_kdj_calculator_incremental
);
criterion_main!(benches);
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use dplang::runtime::Value;
use dplang::indicators::*;

fn benchmark_sma(c: &mut Criterion) {
    let mut group = c.benchmark_group("SMA");
    
    for size in [10, 50, 100, 500, 1000].iter() {
        let prices: Vec<Value> = (0..*size)
            .map(|i| Value::Number(100.0 + (i as f64 % 10.0)))
            .collect();
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                sma(black_box(&prices), black_box(20))
            });
        });
    }
    group.finish();
}

fn benchmark_ema(c: &mut Criterion) {
    let mut group = c.benchmark_group("EMA");
    
    for size in [10, 50, 100, 500, 1000].iter() {
        let prices: Vec<Value> = (0..*size)
            .map(|i| Value::Number(100.0 + (i as f64 % 10.0)))
            .collect();
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                ema(black_box(&prices), black_box(20))
            });
        });
    }
    group.finish();
}

fn benchmark_macd(c: &mut Criterion) {
    let mut group = c.benchmark_group("MACD");
    
    for size in [50, 100, 500, 1000].iter() {
        let prices: Vec<Value> = (0..*size)
            .map(|i| Value::Number(100.0 + (i as f64 % 10.0)))
            .collect();
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                macd(black_box(&prices), black_box(12), black_box(26), black_box(9))
            });
        });
    }
    group.finish();
}

fn benchmark_rsi(c: &mut Criterion) {
    let mut group = c.benchmark_group("RSI");
    
    for size in [20, 50, 100, 500, 1000].iter() {
        let prices: Vec<Value> = (0..*size)
            .map(|i| Value::Number(100.0 + (i as f64 % 10.0)))
            .collect();
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                rsi(black_box(&prices), black_box(14))
            });
        });
    }
    group.finish();
}

fn benchmark_macd_calculator_incremental(c: &mut Criterion) {
    let mut group = c.benchmark_group("MACD_Calculator_Incremental");
    
    for size in [50, 100, 500, 1000, 5000].iter() {
        let prices: Vec<f64> = (0..*size)
            .map(|i| 100.0 + (i as f64 % 10.0))
            .collect();
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut calc = MACDCalculator::new(12, 26, 9);
                for &price in &prices {
                    black_box(calc.update(price));
                }
            });
        });
    }
    group.finish();
}

fn benchmark_kdj_calculator_incremental(c: &mut Criterion) {
    let mut group = c.benchmark_group("KDJ_Calculator_Incremental");
    
    for size in [20, 50, 100, 500, 1000].iter() {
        let data: Vec<(f64, f64, f64)> = (0..*size)
            .map(|i| {
                let base = 100.0 + (i as f64 % 10.0);
                (base + 5.0, base - 5.0, base)
            })
            .collect();
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut calc = KDJCalculator::new(9, 3, 3);
                for &(h, l, c) in &data {
                    black_box(calc.update(h, l, c));
                }
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    benchmark_sma,
    benchmark_ema,
    benchmark_macd,
    benchmark_rsi,
    benchmark_macd_calculator_incremental,
    benchmark_kdj_calculator_incremental
);
criterion_main!(benches);
