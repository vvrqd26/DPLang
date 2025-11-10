# DPLang 场景化使用指南

DPLang v0.3.0 提供了清晰的场景化命令行接口，针对不同的金融数据分析场景优化。

## 📊 四大核心场景

### 1. 单次指标计算 (`calc`)

**使用场景：** 计算单只股票的技术指标

**命令格式：**
```bash
dplang calc <script.dp> <data.csv>
```

**示例：**
```bash
# 计算某只股票的技术指标
dplang calc examples/scripts/simple_indicators.dp data/stock_600000.csv
```

**特点：**
- 一次性批量计算
- 适合单只股票的完整历史数据分析
- 输出CSV格式结果

---

### 2. 策略回测 (`backtest`)

**使用场景：** 回测交易策略，评估历史表现

**命令格式：**
```bash
dplang backtest <strategy.dp> <history.csv> [--output <dir>]
```

**示例：**
```bash
# 回测双均线交叉策略
dplang backtest examples/scripts/ma_crossover_strategy.dp data/history.csv --output backtest_results/

# 查看回测结果
cat backtest_results/backtest_result.csv
```

**特点：**
- 自动计算交易信号、仓位、收益
- 生成回测统计报告（总收益、胜率等）
- 结果保存到指定目录
- 显示执行耗时

**输出统计：**
- 总交易数
- 总收益
- 胜率（盈利交易占比）

---

### 3. 策略选股 (`screen`)

**使用场景：** 从股票池中筛选符合条件的股票

**命令格式：**
```bash
dplang screen <strategy.dp> <stocks.csv> [--output <file>]
```

**示例：**
```bash
# 使用动量策略筛选股票
dplang screen examples/scripts/momentum_screen.dp data/all_stocks.csv --output selected_stocks.csv

# 查看筛选结果
cat selected_stocks.csv
```

**特点：**
- 多股票批量处理
- 自动过滤 `selected` 字段为 `true` 的股票
- 显示筛选前后的股票数量
- 输出前10条结果预览

**脚本要求：**
- 必须包含 `selected:bool` 输出字段
- `selected=true` 的股票会被筛选出来

---

### 4. 实时监控 (`monitor`)

**使用场景：** 实时监控市场数据，持续计算指标

**命令格式：**
```bash
dplang monitor <script.dp> [data.csv] [--window <size>]
```

**示例：**
```bash
# 从CSV文件流式读取（模拟实时数据流）
dplang monitor examples/scripts/realtime_alerts.dp data/realtime_ticks.csv --window 1000

# 从标准输入读取
cat data/stream.csv | dplang monitor examples/scripts/realtime_alerts.dp
```

**特点：**
- 流式处理，逐条处理数据
- 固定内存窗口（默认1000行）
- 按股票代码分组输出到 `./output/` 目录
- 适合长时间运行的监控任务

**输出：**
- 自动按 `stock_code` 分组
- 生成 `output/output_<stock_code>.csv` 文件
- 缓冲写入，定期刷新

---

### 5. 任务编排服务器 (`server`)

**使用场景：** 管理多任务并发执行

**命令格式：**
```bash
dplang server [config.toml] [--port <port>]
```

**示例：**
```bash
# 启动服务器（默认端口8888）
dplang server examples/configs/tasks.toml --port 8888

# 使用默认配置
dplang server
```

**特点：**
- 基于配置文件管理多个任务
- TCP API接口控制任务
- 支持任务的启动、暂停、停止
- 计算元池管理，支持并发执行

**详细文档：** 参见 [ORCHESTRATION_QUICKSTART.md](../ORCHESTRATION_QUICKSTART.md)

---

## 🔧 命令行参数说明

### 通用参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `--output <path>` | 输出文件/目录路径 | 根据场景自动生成 |
| `--window <size>` | 监控模式的窗口大小 | 1000 |
| `--port <port>` | 服务器端口 | 8888 |

---

## 📝 示例脚本说明

### simple_indicators.dp
单次技术指标计算，包含MA、RSI、MACD等常用指标。

### ma_crossover_strategy.dp
双均线交叉策略，用于回测。包含交易信号、仓位、收益计算。

### momentum_screen.dp
动量选股策略，筛选出涨幅大且放量的股票。

### realtime_alerts.dp
实时监控异常波动，检测急涨、急跌、放量等异常情况。

---

## 🎯 场景选择指南

| 需求 | 推荐命令 | 说明 |
|------|----------|------|
| 分析单只股票历史数据 | `calc` | 一次性计算所有指标 |
| 评估策略历史表现 | `backtest` | 包含收益统计 |
| 从股票池选股 | `screen` | 批量筛选符合条件的股票 |
| 监控实时行情 | `monitor` | 流式处理，持续运行 |
| 管理多个分析任务 | `server` | 编排服务器模式 |

---

## 🆚 与旧命令的对比

| 旧命令 | 新命令 | 说明 |
|--------|--------|------|
| `run` | `calc` | 更明确的语义 |
| `daemon` | `monitor` | 更贴近业务场景 |
| `orchestrate` | `server` | 更简洁的命名 |

**注意：** 旧命令仍然可用，但会显示废弃警告。

---

## 💡 最佳实践

### 1. 数据准备
- 确保CSV文件包含必要的字段（如 `date`, `close`, `volume` 等）
- 对于监控模式，必须包含 `stock_code` 字段
- 数据按时间顺序排列

### 2. 脚本编写
- 回测脚本应包含 `signal`, `position`, `profit` 等字段
- 选股脚本必须有 `selected:bool` 输出字段
- 监控脚本应输出 `stock_code` 用于分组

### 3. 性能优化
- 大数据集使用 `backtest` 而不是 `calc`
- 实时场景使用 `monitor` 以控制内存
- 多任务场景使用 `server` 实现并发

---

## 🐛 故障排查

### 问题：脚本解析失败
- 检查语法是否正确
- 确认 INPUT/OUTPUT 声明完整

### 问题：CSV解析错误
- 确认文件格式为标准CSV
- 检查列名与脚本声明是否匹配

### 问题：历史数据不足
- 前N行数据使用 `ref()` 或 `[-n]` 访问历史时会返回 `null`
- 使用 `!= null` 检查避免错误

---

## 📚 更多资源

- [快速开始](../QUICKSTART.md)
- [任务编排指南](../ORCHESTRATION_QUICKSTART.md)
- [实时流式计算指南](../DAEMON_MODE_GUIDE.md)

---

**版本：** DPLang v0.3.0  
**更新日期：** 2025-11-10
