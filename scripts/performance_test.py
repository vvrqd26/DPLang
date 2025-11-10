#!/usr/bin/env python3
"""
DPLang 性能测试脚本
"""
import subprocess
import time
import sys

def run_performance_test():
    print("=== DPLang 性能测试 ===\n")
    
    # 步骤 1: 生成测试数据
    print("步骤 1: 生成测试数据...")
    import generate_test_data
    generate_test_data.generate_stock_data(5000, "test_stock_data.csv")
    print()
    
    # 步骤 2: 运行 DPLang 计算技术指标
    print("步骤 2: 运行 DPLang 计算技术指标...")
    print("正在处理 5000 行数据，计算 MA5, MA15, MA20, BOLL, MACD, RSI...\n")
    
    start_time = time.time()
    
    try:
        result = subprocess.run(
            ["cargo", "run", "--release", "--", "run", 
             "examples/technical_indicators_full.dp", 
             "test_stock_data.csv"],
            capture_output=True,
            text=True,
            encoding='utf-8',
            timeout=60
        )
        
        end_time = time.time()
        elapsed = end_time - start_time
        
        # 提取输出结果
        output = result.stdout
        
        # 查找 CSV 输出部分
        if "输出结果 (CSV 格式):" in output:
            csv_start = output.find("输出结果 (CSV 格式):") + len("输出结果 (CSV 格式):")
            csv_output = output[csv_start:].strip()
            
            # 保存到文件
            with open("test_result.csv", "w", encoding='utf-8') as f:
                f.write(csv_output)
            
            # 统计结果
            lines = csv_output.split('\n')
            header = lines[0] if lines else ""
            data_rows = len(lines) - 1 if len(lines) > 1 else 0
            
            print("✅ 执行成功!\n")
            print(f"执行时间: {elapsed:.2f} 秒")
            print(f"处理速度: {5000/elapsed:.0f} 行/秒")
            print(f"输出行数: {data_rows}")
            print(f"输出列数: {len(header.split(','))}")
            print(f"\n结果已保存到: test_result.csv")
            
            # 显示前5行和后5行
            print("\n前5行数据:")
            for i, line in enumerate(lines[1:6], 1):
                if line.strip():
                    cols = line.split(',')
                    print(f"  {i}. {cols[0] if len(cols) > 0 else ''} {cols[1] if len(cols) > 1 else ''} 收盘:{cols[2] if len(cols) > 2 else ''}")
            
            print("\n后5行数据:")
            for i, line in enumerate(lines[-5:], len(lines)-5):
                if line.strip():
                    cols = line.split(',')
                    print(f"  {i}. {cols[0] if len(cols) > 0 else ''} {cols[1] if len(cols) > 1 else ''} 收盘:{cols[2] if len(cols) > 2 else ''}")
            
        else:
            print("❌ 未找到输出结果")
            print("\n完整输出:")
            print(output)
            if result.stderr:
                print("\n错误信息:")
                print(result.stderr)
        
    except subprocess.TimeoutExpired:
        print("❌ 执行超时 (>60秒)")
    except Exception as e:
        print(f"❌ 执行错误: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    run_performance_test()
