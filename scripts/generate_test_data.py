#!/usr/bin/env python3
"""
生成测试用的股票数据
"""
import csv
import random
from datetime import datetime, timedelta

def generate_stock_data(num_rows=5000, output_file="test_stock_data.csv"):
    """生成模拟的股票数据"""
    
    print(f"正在生成 {num_rows} 行股票数据...")
    
    # 初始价格
    base_price = 100.0
    
    data = []
    start_date = datetime(2020, 1, 1)
    
    for i in range(num_rows):
        date = start_date + timedelta(days=i)
        
        # 模拟价格波动
        change = random.uniform(-0.05, 0.05)  # -5% 到 +5%
        base_price = base_price * (1 + change)
        
        # 生成 OHLC 数据
        open_price = base_price * random.uniform(0.98, 1.02)
        close_price = base_price * random.uniform(0.98, 1.02)
        high_price = max(open_price, close_price) * random.uniform(1.0, 1.03)
        low_price = min(open_price, close_price) * random.uniform(0.97, 1.0)
        
        # 成交量
        volume = random.randint(1000000, 10000000)
        
        data.append({
            'code': 'SH600000',
            'date': date.strftime('%Y-%m-%d'),
            'open': round(open_price, 2),
            'high': round(high_price, 2),
            'low': round(low_price, 2),
            'close': round(close_price, 2),
            'volume': volume
        })
    
    # 写入 CSV
    with open(output_file, 'w', newline='', encoding='utf-8') as f:
        writer = csv.DictWriter(f, fieldnames=['code', 'date', 'open', 'high', 'low', 'close', 'volume'])
        writer.writeheader()
        writer.writerows(data)
    
    print(f"✅ 数据已保存到 {output_file}")
    print(f"   总行数: {num_rows}")
    print(f"   价格范围: {min(d['close'] for d in data):.2f} - {max(d['close'] for d in data):.2f}")

if __name__ == "__main__":
    generate_stock_data(5000, "test_stock_data.csv")
