#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
生成100只股票的测试数据
用于测试DPLang任务编排系统的技术指标计算
"""

import csv
import random
import sys
from datetime import datetime, timedelta

def generate_stock_data(num_stocks=100, days=60):
    """
    生成股票测试数据
    
    参数:
        num_stocks: 股票数量（默认100）
        days: 每只股票的交易日数量（默认60天）
    
    返回:
        包含所有股票数据的列表
    """
    data = []
    base_date = datetime.now() - timedelta(days=days)
    
    for stock_id in range(1, num_stocks + 1):
        stock_code = f"SH{stock_id:06d}"
        
        # 随机初始价格（50-200之间）
        base_price = random.uniform(50, 200)
        
        for day in range(days):
            current_date = base_date + timedelta(days=day)
            date_str = current_date.strftime("%Y-%m-%d")
            
            # 模拟价格波动（±3%）
            change_rate = random.uniform(-0.03, 0.03)
            base_price = base_price * (1 + change_rate)
            
            # 确保价格不为负
            if base_price < 10:
                base_price = 10
            
            # 生成OHLC数据
            open_price = base_price * random.uniform(0.98, 1.02)
            high_price = max(open_price, base_price) * random.uniform(1.00, 1.05)
            low_price = min(open_price, base_price) * random.uniform(0.95, 1.00)
            close_price = base_price
            
            # 成交量（1000000 - 10000000之间）
            volume = random.randint(1000000, 10000000)
            
            data.append({
                'date': date_str,
                'stock_code': stock_code,
                'open': round(open_price, 2),
                'high': round(high_price, 2),
                'low': round(low_price, 2),
                'close': round(close_price, 2),
                'volume': volume
            })
    
    return data

def save_to_csv(data, filename):
    """保存数据到CSV文件"""
    if not data:
        print("Error: No data to save")
        return
    
    fieldnames = ['date', 'stock_code', 'open', 'high', 'low', 'close', 'volume']
    
    with open(filename, 'w', newline='', encoding='utf-8') as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(data)
    
    print(f"Generated {len(data)} rows of stock data")
    print(f"Saved to: {filename}")

if __name__ == "__main__":
    num_stocks = 100
    days = 60
    
    if len(sys.argv) > 1:
        num_stocks = int(sys.argv[1])
    if len(sys.argv) > 2:
        days = int(sys.argv[2])
    
    print(f"Generating stock data...")
    print(f"Number of stocks: {num_stocks}")
    print(f"Days per stock: {days}")
    print(f"Total rows: {num_stocks * days}")
    
    data = generate_stock_data(num_stocks, days)
    save_to_csv(data, 'test_data/stock_data_100.csv')
