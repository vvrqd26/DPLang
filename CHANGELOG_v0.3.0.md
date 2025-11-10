# DPLang v0.3.0 更新日志

## 🎯 重大改进：场景化命令行接口

**发布日期：** 2024-11-10  
**版本号：** v0.3.0

### 📝 更新概述

本次更新对DPLang的命令行接口进行了全面重构，将原本模糊的三种模式（run、daemon、orchestrate）改造为清晰的场景化命令，让用户能够根据实际业务场景快速选择正确的工具。

### ✨ 新增场景化命令

#### 1. `calc` - 单次指标计算
**替代：** `run`  
**使用场景：** 计算单只股票的技术指标

```bash
# 基本用法
dplang calc <script.dp> <data.csv>

# 示例
dplang calc examples/scripts/simple_indicators.dp data/stock_600000.csv
```

**特点：**
- 一次性批量计算
- 适合单只股票的完整历史数据分析
- 输出CSV格式结果

---

#### 2. `backtest` - 策略回测
**新增功能**  
**使用场景：** 回测交易策略，评估历史表现

```bash
# 基本用法
dplang backtest <strategy.dp> <history.csv> [--output <dir>]

# 示例
dplang backtest examples/scripts/ma_crossover_strategy.dp data/history.csv --output results/
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

#### 3. `screen` - 策略选股
**新增功能**  
**使用场景：** 从股票池中筛选符合条件的股票

```bash
# 基本用法
dplang screen <strategy.dp> <stocks.csv> [--output <file>]

# 示例
dplang screen examples/scripts/momentum_screen.dp data/all_stocks.csv --output selected.csv
```

**特点：**
- 多股票批量处理
- 自动过滤 `selected` 字段为 `true` 的股票
- 显示筛选前后的股票数量
- 输出前10条结果预览

**脚本要求：**
- 必须包含 `selected:bool` 输出字段

---

#### 4. `monitor` - 实时监控
**替代：** `daemon`  
**使用场景：** 实时监控市场数据，持续计算指标

```bash
# 基本用法
dplang monitor <script.dp> [data.csv] [--window <size>]

# 示例
dplang monitor examples/scripts/realtime_alerts.dp --window 1000
```

**特点：**
- 流式处理，逐条处理数据
- 固定内存窗口（默认1000行）
- 按股票代码分组输出到 `./output/` 目录
- 适合长时间运行的监控任务

---

#### 5. `server` - 任务编排服务器
**替代：** `orchestrate`  
**使用场景：** 管理多任务并发执行

```bash
# 基本用法
dplang server [config.toml] [--port <port>]

# 示例
dplang server tasks.toml --port 8888
```

**特点：**
- 基于配置文件管理多个任务
- TCP API接口控制任务
- 支持任务的启动、暂停、停止
- 计算元池管理，支持并发执行

---

### 🔧 兼容性说明

旧命令仍然可用，但会显示废弃警告：

| 旧命令 | 新命令 | 状态 |
|--------|--------|------|
| `run` | `calc` | ⚠️ 已废弃 |
| `daemon` | `monitor` | ⚠️ 已废弃 |
| `orchestrate` | `server` | ⚠️ 已废弃 |

---

### 📚 新增文档

#### SCENARIOS_GUIDE.md
全新的场景化使用指南，包含：
- 四大核心场景详细说明
- 命令行参数完整说明
- 场景选择指南
- 最佳实践建议
- 故障排查指南

#### 示例脚本
新增四个场景化示例脚本：

1. **simple_indicators.dp** - 技术指标计算
   - 移动平均线（MA5, MA20）
   - RSI指标

2. **ma_crossover_strategy.dp** - 双均线回测策略
   - 金叉买入，死叉卖出
   - 自动计算仓位和收益

3. **momentum_screen.dp** - 动量选股策略
   - 筛选涨幅大且放量的股票
   - 综合评分机制

4. **realtime_alerts.dp** - 实时异常监控
   - 检测急涨、急跌
   - 成交量异常提醒

---

### 🎨 用户界面改进

#### 新的帮助信息
```
DPLang v0.3.0 - 金融数据分析语言

📊 场景化命令:
  dplang calc <script.dp> [data.csv]           单次指标计算
  dplang backtest <strategy.dp> <history.csv>  策略回测（批量历史数据）
  dplang screen <strategy.dp> <stocks.csv>     策略选股（多股票筛选）
  dplang monitor <script.dp> [data.csv]        实时监控（流式计算）
  dplang server [config.toml] [--port 8888]    任务编排服务器

🔧 通用命令:
  dplang demo                                  运行内置演示
  dplang help                                  显示帮助信息
  dplang version                               显示版本信息
```

#### 场景化的输出提示
每个命令都有清晰的emoji图标和场景说明：
- 🧮 单次指标计算模式
- 📈 策略回测模式
- 🔍 策略选股模式
- 📡 实时监控模式
- 🔧 任务编排服务器模式

---

### 🔨 技术实现

#### 新增函数
在 `main.rs` 中新增以下函数：

1. **参数解析函数**
   - `parse_output_dir()` - 解析输出目录参数
   - `parse_output_file()` - 解析输出文件参数
   - `parse_window_size()` - 解析窗口大小参数
   - `parse_port()` - 解析端口参数

2. **场景化命令实现**
   - `run_calc_interactive()` - 交互式指标计算
   - `run_calc_mode()` - CSV文件指标计算
   - `run_backtest_mode()` - 策略回测
   - `run_screen_mode()` - 策略选股
   - `run_monitor_mode()` - 实时监控
   - `run_server_mode()` - 任务编排服务器

3. **辅助函数**
   - `print_backtest_summary()` - 回测统计报告
   - `print_version()` - 版本信息显示

#### 代码结构改进
- 命令行参数解析更加清晰
- 每个场景有独立的函数实现
- 统一的错误处理和输出格式

---

### 📊 测试覆盖

所有92个单元测试全部通过 ✅

```bash
cargo test
```

**测试结果：**
- 92 passed
- 0 failed
- 执行时间：0.03s

---

### 🎯 设计理念

#### 用户中心设计
- **场景优先**：从用户实际业务场景出发
- **语义明确**：命令名称直接表达使用场景
- **易于记忆**：简短有意义的命令名

#### 渐进式增强
- **向后兼容**：旧命令仍然可用
- **平滑迁移**：提供废弃警告引导用户
- **文档完善**：详细的使用指南和示例

---

### 📈 使用建议

#### 场景选择指南

| 需求 | 推荐命令 | 说明 |
|------|----------|------|
| 分析单只股票历史数据 | `calc` | 一次性计算所有指标 |
| 评估策略历史表现 | `backtest` | 包含收益统计 |
| 从股票池选股 | `screen` | 批量筛选符合条件的股票 |
| 监控实时行情 | `monitor` | 流式处理，持续运行 |
| 管理多个分析任务 | `server` | 编排服务器模式 |

#### 最佳实践

1. **数据准备**
   - 确保CSV文件包含必要的字段
   - 对于监控模式，必须包含 `stock_code` 字段
   - 数据按时间顺序排列

2. **脚本编写**
   - 回测脚本应包含 `signal`, `position`, `profit` 等字段
   - 选股脚本必须有 `selected:bool` 输出字段
   - 监控脚本应输出 `stock_code` 用于分组

3. **性能优化**
   - 大数据集使用 `backtest` 而不是 `calc`
   - 实时场景使用 `monitor` 以控制内存
   - 多任务场景使用 `server` 实现并发

---

### 🚀 下一步计划

1. **增强回测功能**
   - 支持更多统计指标（最大回撤、夏普比率等）
   - 可视化回测结果
   - 导出交易明细

2. **选股功能增强**
   - 支持多条件组合筛选
   - 排序和分页
   - 导出多种格式

3. **监控功能增强**
   - 实时告警推送
   - 支持WebSocket输出
   - 性能监控仪表盘

4. **服务器功能增强**
   - Web管理界面
   - 任务调度和定时执行
   - 分布式计算支持

---

### 📝 文档更新

本次更新涉及的文档：

1. **README.md** - 更新快速开始部分
2. **SCENARIOS_GUIDE.md** - 新增场景化使用指南
3. **examples/scripts/** - 新增四个示例脚本
4. **CHANGELOG_v0.3.0.md** - 本更新日志

---

### 🙏 致谢

感谢所有使用DPLang的用户，你们的反馈帮助我们不断改进产品。

---

**DPLang v0.3.0** - 让金融数据分析更简单、更直观！ 🚀
