use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use dplang::executor::{ContextPool, PoolConfig, ColumnarStorage, OutputManager, OutputManagerConfig, OutputMode};
use dplang::runtime::Value;
use std::collections::HashMap;

/// 测试上下文对象池性能
fn benchmark_context_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("ContextPool");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("with_pool", size), size, |b, &size| {
            b.iter(|| {
                let mut pool = ContextPool::with_default();
                for _ in 0..size {
                    let ctx = pool.acquire();
                    black_box(&ctx);
                    pool.release(ctx);
                }
            });
        });
        
        group.bench_with_input(BenchmarkId::new("without_pool", size), size, |b, &size| {
            b.iter(|| {
                for _ in 0..size {
                    let ctx = dplang::executor::ExecutionContext::new();
                    black_box(&ctx);
                }
            });
        });
    }
    
    group.finish();
}

/// 测试列式存储性能
fn benchmark_columnar_storage(c: &mut Criterion) {
    let mut group = c.benchmark_group("ColumnarStorage");
    
    for row_count in [100, 1000, 10000].iter() {
        // 创建测试数据
        let mut rows = Vec::new();
        for i in 0..*row_count {
            let mut row = HashMap::new();
            row.insert("close".to_string(), Value::Number(100.0 + (i as f64 % 100.0)));
            row.insert("volume".to_string(), Value::Number(1000.0 + (i as f64 % 1000.0)));
            row.insert("open".to_string(), Value::Number(99.0 + (i as f64 % 100.0)));
            row.insert("high".to_string(), Value::Number(101.0 + (i as f64 % 100.0)));
            row.insert("low".to_string(), Value::Number(98.0 + (i as f64 % 100.0)));
            rows.push(row);
        }
        
        // 测试行式存储访问
        group.bench_with_input(
            BenchmarkId::new("row_storage_access", row_count),
            &rows,
            |b, rows| {
                b.iter(|| {
                    let mut sum = 0.0;
                    for row in rows {
                        if let Some(Value::Number(val)) = row.get("close") {
                            sum += val;
                        }
                    }
                    black_box(sum);
                });
            },
        );
        
        // 测试列式存储访问
        let columnar = ColumnarStorage::from_rows(&rows);
        group.bench_with_input(
            BenchmarkId::new("columnar_storage_access", row_count),
            &columnar,
            |b, storage| {
                b.iter(|| {
                    if let Some(column) = storage.get_column("close") {
                        let mut sum = 0.0;
                        for val in column.iter() {
                            if let Value::Number(v) = val {
                                sum += v;
                            }
                        }
                        black_box(sum);
                    }
                });
            },
        );
    }
    
    group.finish();
}

/// 测试输出管理器性能
fn benchmark_output_manager(c: &mut Criterion) {
    let mut group = c.benchmark_group("OutputManager");
    
    for row_count in [100, 1000, 10000].iter() {
        // 测试内存模式
        group.bench_with_input(
            BenchmarkId::new("in_memory_mode", row_count),
            row_count,
            |b, &count| {
                b.iter(|| {
                    let mut manager = OutputManager::with_default();
                    for i in 0..count {
                        let mut row = HashMap::new();
                        row.insert("close".to_string(), Value::Number(100.0 + (i as f64)));
                        row.insert("volume".to_string(), Value::Number(1000.0 + (i as f64)));
                        manager.write_row(row).unwrap();
                    }
                    black_box(manager.finalize().unwrap());
                });
            },
        );
        
        // 测试流式写入文件模式（使用临时文件）
        group.bench_with_input(
            BenchmarkId::new("stream_to_file_mode", row_count),
            row_count,
            |b, &count| {
                b.iter(|| {
                    let temp_path = format!("bench_output_{}.csv", count);
                    let config = OutputManagerConfig {
                        mode: OutputMode::StreamToFile {
                            path: temp_path.clone().into(),
                            buffer_size: 8,
                        },
                        buffer_size: 1000,
                        flush_interval: 1000,
                    };
                    
                    let mut manager = OutputManager::new(config).unwrap();
                    for i in 0..count {
                        let mut row = HashMap::new();
                        row.insert("close".to_string(), Value::Number(100.0 + (i as f64)));
                        row.insert("volume".to_string(), Value::Number(1000.0 + (i as f64)));
                        manager.write_row(row).unwrap();
                    }
                    manager.finalize().unwrap();
                    
                    // 清理临时文件
                    std::fs::remove_file(&temp_path).ok();
                });
            },
        );
    }
    
    group.finish();
}

/// 测试列式存储创建性能
fn benchmark_columnar_storage_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ColumnarStorage_Creation");
    
    for row_count in [100, 1000, 10000].iter() {
        let mut rows = Vec::new();
        for i in 0..*row_count {
            let mut row = HashMap::new();
            row.insert("close".to_string(), Value::Number(100.0 + (i as f64)));
            row.insert("volume".to_string(), Value::Number(1000.0 + (i as f64)));
            rows.push(row);
        }
        
        group.bench_with_input(
            BenchmarkId::from_parameter(row_count),
            &rows,
            |b, rows| {
                b.iter(|| {
                    black_box(ColumnarStorage::from_rows(rows));
                });
            },
        );
    }
    
    group.finish();
}

/// 测试列式存储切片访问
fn benchmark_columnar_storage_slice(c: &mut Criterion) {
    let mut group = c.benchmark_group("ColumnarStorage_Slice");
    
    // 创建大数据集
    let row_count = 10000;
    let mut rows = Vec::new();
    for i in 0..row_count {
        let mut row = HashMap::new();
        row.insert("close".to_string(), Value::Number(100.0 + (i as f64)));
        rows.push(row);
    }
    
    let storage = ColumnarStorage::from_rows(&rows);
    
    for slice_size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(slice_size),
            slice_size,
            |b, &size| {
                b.iter(|| {
                    black_box(storage.get_column_slice("close", 0, size));
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_context_pool,
    benchmark_columnar_storage,
    benchmark_output_manager,
    benchmark_columnar_storage_creation,
    benchmark_columnar_storage_slice
);
criterion_main!(benches);
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use dplang::executor::{ContextPool, PoolConfig, ColumnarStorage, OutputManager, OutputManagerConfig, OutputMode};
use dplang::runtime::Value;
use std::collections::HashMap;

/// 测试上下文对象池性能
fn benchmark_context_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("ContextPool");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("with_pool", size), size, |b, &size| {
            b.iter(|| {
                let mut pool = ContextPool::with_default();
                for _ in 0..size {
                    let ctx = pool.acquire();
                    black_box(&ctx);
                    pool.release(ctx);
                }
            });
        });
        
        group.bench_with_input(BenchmarkId::new("without_pool", size), size, |b, &size| {
            b.iter(|| {
                for _ in 0..size {
                    let ctx = dplang::executor::ExecutionContext::new();
                    black_box(&ctx);
                }
            });
        });
    }
    
    group.finish();
}

/// 测试列式存储性能
fn benchmark_columnar_storage(c: &mut Criterion) {
    let mut group = c.benchmark_group("ColumnarStorage");
    
    for row_count in [100, 1000, 10000].iter() {
        // 创建测试数据
        let mut rows = Vec::new();
        for i in 0..*row_count {
            let mut row = HashMap::new();
            row.insert("close".to_string(), Value::Number(100.0 + (i as f64 % 100.0)));
            row.insert("volume".to_string(), Value::Number(1000.0 + (i as f64 % 1000.0)));
            row.insert("open".to_string(), Value::Number(99.0 + (i as f64 % 100.0)));
            row.insert("high".to_string(), Value::Number(101.0 + (i as f64 % 100.0)));
            row.insert("low".to_string(), Value::Number(98.0 + (i as f64 % 100.0)));
            rows.push(row);
        }
        
        // 测试行式存储访问
        group.bench_with_input(
            BenchmarkId::new("row_storage_access", row_count),
            &rows,
            |b, rows| {
                b.iter(|| {
                    let mut sum = 0.0;
                    for row in rows {
                        if let Some(Value::Number(val)) = row.get("close") {
                            sum += val;
                        }
                    }
                    black_box(sum);
                });
            },
        );
        
        // 测试列式存储访问
        let columnar = ColumnarStorage::from_rows(&rows);
        group.bench_with_input(
            BenchmarkId::new("columnar_storage_access", row_count),
            &columnar,
            |b, storage| {
                b.iter(|| {
                    if let Some(column) = storage.get_column("close") {
                        let mut sum = 0.0;
                        for val in column.iter() {
                            if let Value::Number(v) = val {
                                sum += v;
                            }
                        }
                        black_box(sum);
                    }
                });
            },
        );
    }
    
    group.finish();
}

/// 测试输出管理器性能
fn benchmark_output_manager(c: &mut Criterion) {
    let mut group = c.benchmark_group("OutputManager");
    
    for row_count in [100, 1000, 10000].iter() {
        // 测试内存模式
        group.bench_with_input(
            BenchmarkId::new("in_memory_mode", row_count),
            row_count,
            |b, &count| {
                b.iter(|| {
                    let mut manager = OutputManager::with_default();
                    for i in 0..count {
                        let mut row = HashMap::new();
                        row.insert("close".to_string(), Value::Number(100.0 + (i as f64)));
                        row.insert("volume".to_string(), Value::Number(1000.0 + (i as f64)));
                        manager.write_row(row).unwrap();
                    }
                    black_box(manager.finalize().unwrap());
                });
            },
        );
        
        // 测试流式写入文件模式（使用临时文件）
        group.bench_with_input(
            BenchmarkId::new("stream_to_file_mode", row_count),
            row_count,
            |b, &count| {
                b.iter(|| {
                    let temp_path = format!("bench_output_{}.csv", count);
                    let config = OutputManagerConfig {
                        mode: OutputMode::StreamToFile {
                            path: temp_path.clone().into(),
                            buffer_size: 8,
                        },
                        buffer_size: 1000,
                        flush_interval: 1000,
                    };
                    
                    let mut manager = OutputManager::new(config).unwrap();
                    for i in 0..count {
                        let mut row = HashMap::new();
                        row.insert("close".to_string(), Value::Number(100.0 + (i as f64)));
                        row.insert("volume".to_string(), Value::Number(1000.0 + (i as f64)));
                        manager.write_row(row).unwrap();
                    }
                    manager.finalize().unwrap();
                    
                    // 清理临时文件
                    std::fs::remove_file(&temp_path).ok();
                });
            },
        );
    }
    
    group.finish();
}

/// 测试列式存储创建性能
fn benchmark_columnar_storage_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ColumnarStorage_Creation");
    
    for row_count in [100, 1000, 10000].iter() {
        let mut rows = Vec::new();
        for i in 0..*row_count {
            let mut row = HashMap::new();
            row.insert("close".to_string(), Value::Number(100.0 + (i as f64)));
            row.insert("volume".to_string(), Value::Number(1000.0 + (i as f64)));
            rows.push(row);
        }
        
        group.bench_with_input(
            BenchmarkId::from_parameter(row_count),
            &rows,
            |b, rows| {
                b.iter(|| {
                    black_box(ColumnarStorage::from_rows(rows));
                });
            },
        );
    }
    
    group.finish();
}

/// 测试列式存储切片访问
fn benchmark_columnar_storage_slice(c: &mut Criterion) {
    let mut group = c.benchmark_group("ColumnarStorage_Slice");
    
    // 创建大数据集
    let row_count = 10000;
    let mut rows = Vec::new();
    for i in 0..row_count {
        let mut row = HashMap::new();
        row.insert("close".to_string(), Value::Number(100.0 + (i as f64)));
        rows.push(row);
    }
    
    let storage = ColumnarStorage::from_rows(&rows);
    
    for slice_size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(slice_size),
            slice_size,
            |b, &size| {
                b.iter(|| {
                    black_box(storage.get_column_slice("close", 0, size));
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_context_pool,
    benchmark_columnar_storage,
    benchmark_output_manager,
    benchmark_columnar_storage_creation,
    benchmark_columnar_storage_slice
);
criterion_main!(benches);
