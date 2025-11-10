# DPLang 实时流式计算模式使用指南

## 概述

DPLang 支持实时流式计算模式（daemon 模式），可以持续处理 tick 级数据流，实现高性能的实时金融数据分析。

## 基本用法

```bash
# 从 CSV 文件流式读取
dplang daemon <script.dp> <data.csv>

# 从标准输入流式读取
dplang daemon <script.dp>
```

## 功能特性

### 1. 内存窗口管理
- 固定窗口大小：默认 1000 行数据
- 自动淘汰：窗口满时自动淘汰最旧数据
- 高效访问：使用 VecDeque 实现快速的历史数据访问

### 2. 流式 CSV 输出
- 按股票分组：自动为每只股票创建独立的输出文件
- 缓冲写入：默认缓冲 100 行后批量写入
- 输出目录：默认输出到 ./output 目录

### 3. 历史数据访问
使用 ref() 函数访问历史数据

## 示例运行

```bash
cargo run --release -- daemon examples/realtime.dp test_data.csv
```

## 输出

生成按股票代码分组的 CSV 文件：
- output/output_000001.csv
- output/output_000002.csv

## 架构设计

### 核心组件

1. StreamingExecutor - 支持增量 tick 推送，维护固定长度的内存窗口
2. CSVStreamWriter - 按股票分组写入，带缓冲的流式写入
3. Daemon 模式 - CSV 文件流式读取，持续运行直到数据流结束

## 性能特性

- 内存优化：固定窗口大小，不会无限增长
- 批量写入：缓冲机制减少 I/O 次数
- 并发友好：纯函数设计，天然并发安全
- 实时响应：逐 tick 处理，无需等待完整数据集

## 注意事项

1. 历史数据不足：前 N 行数据使用 ref() 访问不存在的历史数据时，会返回 null
2. CSV 格式：必须包含 stock_code 列用于分组
3. 输出目录：确保有权限创建 ./output 目录
