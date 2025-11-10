# 回测模块测试指南

## 测试准备

### 1. 编译项目

```bash
cargo build --release
```

### 2. 运行单元测试

```bash
# 测试所有模块
cargo test

# 仅测试回测模块
cargo test --lib backtest

# 查看测试详情
cargo test -- --nocapture
```

## 功能测试

### 测试1: 基础回测功能

使用双均线交叉策略进行回测：

```bash
./target/release/dplang backtest \
    examples/scripts/ma_crossover_strategy.dp \
    test_data/backtest_sample.csv \
    --output test_results/
```

**预期结果：**
- 创建 `test_results/` 目录
- 生成所有报告文件（summary.txt, summary.json, trades.csv等）
- 控制台显示快速摘要
- 无错误信息

**验证步骤：**
```bash
# 检查输出文件
ls -lh test_results/

# 查看摘要
cat test_results/summary.txt

# 查看交易明细
head test_results/trades.csv

# 验证JSON格式
cat test_results/summary.json | python -m json.tool
```

### 测试2: RSI策略回测

使用RSI超买超卖策略：

```bash
./target/release/dplang backtest \
    examples/scripts/rsi_strategy.dp \
    test_data/stock_trend.csv \
    --output rsi_results/
```

**预期结果：**
- 成功执行回测
- 生成完整报告
- 指标计算正确

### 测试3: 长周期数据测试

测试100天以上的历史数据：

```bash
./target/release/dplang backtest \
    examples/scripts/ma_crossover_strategy.dp \
    test_data/stock_trend.csv \
    --output long_term_results/
```

**验证要点：**
- 性能指标合理（年化收益率、夏普比率等）
- 最大回撤计算正确
- 交易次数合理

## 性能测试

### 测试大数据集

创建包含1000+条记录的测试数据：

```bash
# 可使用scripts/generate_stock_data.py生成
python scripts/generate_stock_data.py --rows 1000 --output test_data/large_dataset.csv

# 运行回测
time ./target/release/dplang backtest \
    examples/scripts/ma_crossover_strategy.dp \
    test_data/large_dataset.csv
```

**性能目标：**
- 1000条数据 < 0.5秒
- 5000条数据 < 2秒
- 10000条数据 < 5秒

## 指标验证

### 手动验证关键指标

以下是手动验证指标计算准确性的方法：

#### 1. 总收益率验证

```
总收益率 = (期末总资产 - 初始资金) / 初始资金

从 positions.csv 中查看：
- 初始资金：第一行 total_value
- 期末资产：最后一行 total_value
```

#### 2. 最大回撤验证

```
从 positions.csv 或 daily_stats.csv 中：
- 找到最大的 drawdown 值
- 验证该值与 summary 中的 max_drawdown 一致
```

#### 3. 胜率验证

```
从 trades.csv 中统计：
- 盈利交易数：net_profit > 0
- 总交易数：总行数
- 胜率 = 盈利交易数 / 总交易数
```

#### 4. 交易成本验证

检查第一笔交易：
```bash
# 查看交易明细
sed -n '2p' test_results/trades.csv
```

验证：
- 手续费 = 买入金额 × 0.0003（至少5元）
- 印花税 = 卖出金额 × 0.001
- 净利润 = 毛利润 - 手续费总计 - 印花税

## 边界情况测试

### 1. 空交易策略

策略从不发出买入信号：

```bash
# 创建一个永远持有的策略
echo '-- INPUT close:number --
-- OUTPUT signal:string --
signal = "持有"
return [signal]' > test_hold_only.dp

./target/release/dplang backtest test_hold_only.dp test_data/backtest_sample.csv
```

**预期：**
- 交易次数为0
- 总收益为0
- 不应出错

### 2. 高频交易策略

每天都交易：

**预期：**
- 手续费显著影响收益
- 交易次数等于数据行数

### 3. 数据异常

测试缺失字段、空数据等情况：

```bash
# 缺少close字段
echo 'date,volume
2024-01-01,1000' > test_invalid.csv

./target/release/dplang backtest \
    examples/scripts/ma_crossover_strategy.dp \
    test_invalid.csv
```

**预期：**
- 友好的错误提示
- 指出缺少的字段

## 报告质量检查

### 检查清单

- [ ] summary.txt 格式美观，数字对齐
- [ ] summary.json 格式正确，可被解析
- [ ] trades.csv 包含所有必需字段
- [ ] positions.csv 记录完整
- [ ] daily_stats.csv 每日都有记录
- [ ] equity_curve.csv 数据连续
- [ ] 所有百分比数值合理（如胜率在0-100%之间）
- [ ] 所有金额计算准确
- [ ] 日期格式一致

## 集成测试

运行完整的测试套件：

```bash
# 运行测试脚本
bash scripts/test_backtest.sh

# 或使用示例脚本
bash examples/backtest_example.sh
```

## 已知问题和限制

### 当前限制

1. **单标的回测**：仅支持单个股票回测
2. **固定仓位**：仅支持全仓或按比例买入
3. **简化滑点**：使用固定比例，未考虑市场深度
4. **持仓天数计算**：当前为简化版本

### 计划改进

1. 支持多标的组合回测
2. 高级仓位管理策略
3. 更真实的交易成本模拟
4. 优化持仓天数计算算法

## 故障排查

### 常见问题

**问题1：CSV解析失败**
- 检查CSV格式是否正确
- 确保包含必需字段（close或price，date可选）
- 验证数据类型正确

**问题2：性能指标异常**
- 检查数据质量
- 验证策略逻辑
- 查看交易明细确认交易是否合理

**问题3：报告文件未生成**
- 检查输出目录权限
- 查看控制台错误信息
- 验证磁盘空间充足

## 回归测试

在每次修改后运行：

```bash
# 快速回归测试
cargo test --lib backtest

# 完整功能测试
bash scripts/test_backtest.sh

# 性能基准测试
cargo bench
```

## 测试报告模板

记录测试结果：

```
测试日期: YYYY-MM-DD
测试版本: vX.X.X
测试人员: [姓名]

| 测试项 | 状态 | 备注 |
|--------|------|------|
| 单元测试 | ✅/❌ | |
| 基础回测 | ✅/❌ | |
| RSI策略 | ✅/❌ | |
| 性能测试 | ✅/❌ | |
| 边界测试 | ✅/❌ | |
| 报告质量 | ✅/❌ | |

发现问题：
1. ...
2. ...

建议改进：
1. ...
2. ...
```
