# DPLang 快速开始

## 🚀 第一步：编译项目

```bash
cd c:\Users\dp\Desktop\work\DPLang
cargo build --release
```

## 📝 第二步：理解基本语法

### 1. 数据脚本结构

```dplang
-- INPUT 输入参数:类型 --
-- OUTPUT 输出参数:类型 --

# 这里是脚本逻辑
result = input1 + input2

return [result]
```

### 2. 支持的数据类型

- `number` - 浮点数
- `decimal` - 高精度小数
- `string` - 字符串
- `bool` - 布尔值
- `array` - 数组

### 3. 常用内置函数

**数学函数**：
- `sum(array)` - 求和
- `max(array)` - 最大值
- `min(array)` - 最小值

**时间序列函数**：
- `window("变量名", size)` - 获取滑动窗口数据
- `ref("变量名", offset)` - 引用历史值
- `past("变量名", n)` - 获取过去n个值

**技术指标**：
- `SMA(prices, period)` - 简单移动平均
- `EMA(prices, period)` - 指数移动平均
- `RSI(prices, period)` - 相对强弱指标
- `MACD(prices, fast, slow, signal)` - MACD指标
- `BOLL(prices, period, std)` - 布林带
- `KDJ(high, low, close, n, m1, m2)` - KDJ指标

**调试函数**：
- `print(...)` - 打印输出

## 📊 第三步：运行示例

### 示例 1: Hello World

文件：`examples/hello.dp`
```dplang
-- INPUT name:string --
-- OUTPUT greeting:string --

greeting = "Hello, " + name + "!"
print("Greeting:", greeting)

return [greeting]
```

### 示例 2: 移动平均线

文件：`examples/moving_average.dp`
```dplang
-- INPUT close:number --
-- OUTPUT ma5:number, ma20:number, signal:string --

# 计算5日和20日均线
ma5 = MA(close, 5)
ma20 = MA(close, 20)

# 生成交易信号
signal = ma5 > ma20 ? "金叉" : "死叉"

return [ma5, ma20, signal]
```

### 示例 3: 综合技术分析

文件：`examples/technical_analysis.dp`
```dplang
-- INPUT close:number, high:number, low:number --
-- OUTPUT rsi:number, signal:string --

# 计算RSI指标
rsi = RSI(close, 14)

# 判断超买超卖
signal = rsi < 30 ? "超卖" : 
         rsi > 70 ? "超买" : "中性"

return [rsi, signal]
```

## 🧪 第四步：运行测试

查看所有功能是否正常：

```bash
cargo test
```

应该看到：**48个测试全部通过 ✅**

## 💡 第五步：编写你的第一个脚本

### 场景：计算股票收益率

创建文件 `my_first.dp`：

```dplang
-- INPUT code:string, open:number, close:number --
-- OUTPUT code:string, 收益率:number, 涨跌:string --

# 计算收益率（百分比）
收益率 = (close - open) / open * 100

# 判断涨跌
涨跌 = 收益率 > 0 ? "上涨" : 
       收益率 < 0 ? "下跌" : "平盘"

# 打印调试信息
print(code, "收益率:", 收益率, "%", 涨跌)

return [code, 收益率, 涨跌]
```

## 📚 第六步：使用包系统

### 创建数学工具包

文件：`packages/mymath.dp`
```dplang
package mymath

# 定义常量
TAU = 6.28318530718

# 定义函数
square(x) -> number:
    return x * x

cube(x) -> number:
    return x * x * x
```

### 使用包

在数据脚本中：
```dplang
-- IMPORT mymath --
-- INPUT x:number --
-- OUTPUT result:number --

result = mymath.square(x) + mymath.TAU
return [result]
```

## 🎯 常用模式

### 1. 条件判断

```dplang
if price > 100:
    signal = "高价"
else:
    signal = "低价"
```

### 2. 三元表达式

```dplang
signal = price > 100 ? "高价" : "低价"
```

### 3. 数组操作

```dplang
prices = [100, 200, 300]
doubled = map(prices, x -> x * 2)
filtered = filter(prices, x -> x > 150)
```

### 4. 向量运算

```dplang
prices = [100, 200, 300]
adjusted = prices * 1.1  # 全部上涨10%
```

### 5. 时间序列计算

```dplang
# 获取前一天的收盘价
prev_close = close[-1]

# 计算涨跌幅
change = prev_close == null ? 0 : (close - prev_close) / prev_close * 100
```

## ⚠️ 注意事项

1. **缩进敏感**：使用 4 个空格或 1 个 Tab 进行缩进
2. **变量作用域**：在 if/else 块中定义的变量仅在该块内有效
3. **历史数据不足**：使用 `close[-1]` 或 `close[-5:]` 时，历史不足会返回 `null`
4. **精度控制**：需要高精度时使用 `-- PRECISION n --` 声明

## 🔧 调试技巧

### 使用 print 函数

```dplang
print("变量值:", x, y, z)
print("数组:", prices)
print("计算结果:", ma5, ma20)
```

### 检查 null 值

```dplang
if value == null:
    print("警告: 值为空")
else:
    print("正常值:", value)
```

## 📖 更多资源

- 查看 `README.md` 了解完整功能列表
- 查看 `dev_logs/` 目录了解设计文档
- 运行 `cargo test` 查看测试用例

## 🎉 开始你的 DPLang 之旅！

从简单的例子开始，逐步探索更复杂的功能。祝你使用愉快！
