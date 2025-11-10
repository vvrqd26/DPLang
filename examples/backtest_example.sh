#!/bin/bash

# DPLang 回测功能使用示例

echo "=========================================="
echo "  DPLang 专业回测系统使用示例"
echo "=========================================="
echo ""

# 示例1: 基础回测
echo "示例1: 双均线交叉策略回测"
echo "------------------------------------------"
echo "命令:"
echo "  dplang backtest examples/scripts/ma_crossover_strategy.dp test_data/backtest_sample.csv"
echo ""

dplang backtest \
    examples/scripts/ma_crossover_strategy.dp \
    test_data/backtest_sample.csv

echo ""
echo ""

# 示例2: 自定义输出目录
echo "示例2: 指定输出目录"
echo "------------------------------------------"
echo "命令:"
echo "  dplang backtest examples/scripts/ma_crossover_strategy.dp test_data/backtest_sample.csv --output my_results/"
echo ""

dplang backtest \
    examples/scripts/ma_crossover_strategy.dp \
    test_data/backtest_sample.csv \
    --output my_results/

echo ""
echo ""

# 查看结果
echo "查看回测结果："
echo "------------------------------------------"
echo ""
echo "1. 文本摘要:"
cat backtest_results/summary.txt
echo ""
echo ""

echo "2. 交易明细（前5笔）:"
head -6 backtest_results/trades.csv
echo ""
echo ""

echo "3. 每日统计（前5天）:"
head -6 backtest_results/daily_stats.csv
echo ""
echo ""

echo "=========================================="
echo "  使用说明"
echo "=========================================="
echo ""
echo "生成的报告文件："
echo "  - summary.txt        文本格式摘要（易读）"
echo "  - summary.json       JSON格式摘要（程序化）"
echo "  - trades.csv         交易明细"
echo "  - positions.csv      持仓记录"
echo "  - daily_stats.csv    每日统计"
echo "  - equity_curve.csv   资金曲线"
echo ""
echo "关键指标说明："
echo "  - 总收益率：策略整体表现"
echo "  - 年化收益率：标准化后的年化收益"
echo "  - 最大回撤：最大亏损幅度"
echo "  - 夏普比率：风险调整后收益（>1为优秀）"
echo "  - 胜率：盈利交易占比"
echo "  - 盈亏比：平均盈利/平均亏损"
echo ""
