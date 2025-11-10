#!/bin/bash

# 回测模块功能测试脚本

echo "======================================"
echo "   DPLang 回测模块功能测试"
echo "======================================"
echo ""

# 测试1: 基本回测功能
echo "测试1: 运行双均线交叉策略回测"
echo "--------------------------------------"

./target/release/dplang backtest \
    examples/scripts/ma_crossover_strategy.dp \
    test_data/backtest_sample.csv \
    --output test_backtest_results

echo ""
echo "测试完成！"
echo ""
echo "检查输出文件："
ls -lh test_backtest_results/

echo ""
echo "查看回测摘要："
cat test_backtest_results/summary.txt

echo ""
echo "======================================"
echo "   测试结束"
echo "======================================"
