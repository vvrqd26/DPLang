#!/usr/bin/env python3
"""
运行性能测试并提取CSV结果
"""
import subprocess
import time
import re

print("=== DPLang 性能测试 ===\n")
print("处理 5000 行股票数据，计算 MA5, MA15, MA20, RSI...\n")

start = time.time()

result = subprocess.run(
    ["cargo", "run", "--release", "--", "run",
     "examples/technical_indicators_full.dp",
     "test_stock_data.csv"],
    capture_output=True,
    text=True,
    encoding='utf-8'
)

elapsed = time.time() - start

output = result.stdout + result.stderr

# 查找CSV输出
match = re.search(r'close,code,date.*', output, re.DOTALL)

if match:
    csv_output = match.group(0)
    
    # 保存到文件
    with open("test_result_final.csv", "w", encoding='utf-8') as f:
        f.write(csv_output)
    
    lines = csv_output.count('\n')
    
    print(f"执行时间: {elapsed:.2f} 秒")
    print(f"处理速度: {5000/elapsed:.0f} 行/秒")
    print(f"输出行数: {lines}")
    print(f"\n结果已保存到: test_result_final.csv")
    
    # 显示前5行
    csv_lines = csv_output.split('\n')
    print("\n前5行数据:")
    for i, line in enumerate(csv_lines[:6]):
        print(f"  {line}")
else:
    print("未找到CSV输出")
    print("\n完整输出:")
    print(output)
