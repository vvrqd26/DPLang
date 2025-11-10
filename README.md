# DPLang - 简单、高效、AI友好的数据处理语言

> v0.4.0 - 回归本质，专注语言核心

## 🎯 项目定位

DPLang 是一个**纯粹的语言解释器**，专注于提供简洁而强大的数据处理能力：

✅ **简单易学** - 极简语法，支持中文标识符，AI 友好  
⚡ **高效执行** - 流式计算架构，零拷贝设计  
🔌 **灵活扩展** - 开放的语言核心，易于嵌入和扩展  
📦 **轻量依赖** - 最小化依赖，适合集成到各类 Rust 项目

**设计哲学**：
- 语言本身只提供核心计算能力（表达式、函数、流程控制）
- 业务逻辑通过扩展机制实现，保持核心简洁
- 专注做好「数据处理语言」这一件事

## ✨ 核心特性

### 1. 完整的语言解析链

**词法分析器 (Lexer)**
- ✅ 中文标识符支持
- ✅ 缩进敏感语法 (Indent/Dedent)
- ✅ 完整的运算符支持 (`+`, `-`, `*`, `/`, `%`, `^`, `>`, `<`, `==`, `!=`, `and`, `or`, `not`)
- ✅ 管道运算符 `|>` 和 Lambda 箭头 `->`

**语法分析器 (Parser)**
- ✅ 递归下降解析器，正确的运算符优先级
- ✅ 表达式解析：算术、逻辑、三元、管道
- ✅ 语句解析：赋值、条件、返回
- ✅ Lambda 表达式和数组字面量
- ✅ 包导入机制 (`-- IMPORT --`)

**语义分析器 (Semantic Analyzer)**
- ✅ 未定义变量检测
- ✅ 变量遮蔽检测
- ✅ 未使用变量警告
- ✅ 作用域管理（if 块、Lambda、函数）

### 2. 强大的运行时系统

**类型系统 (Value)**
- ✅ Number - 浮点数
- ✅ Decimal - 高精度小数
- ✅ String - 字符串
- ✅ Bool - 布尔值
- ✅ Null - 空值
- ✅ Array - 数组（支持向量运算）
- ✅ Lambda - 函数值

**向量运算**
- ✅ 数组逐元素运算：`[1, 2, 3] * 2 => [2, 4, 6]`
- ✅ 广播机制：标量与数组自动扩展
- ✅ 完整的算术和逻辑运算支持

### 3. 流式执行引擎

**核心执行器 (Executor)**
- ✅ 表达式求值
- ✅ 变量管理和作用域
- ✅ 条件语句 (if/else)
- ✅ Lambda 表达式执行
- ✅ 用户自定义函数
- ✅ 包导入和成员访问
- ✅ 错误处理机制 (ERROR 块)
- ✅ 精度控制 (PRECISION)

**流式处理 (DataStreamExecutor)**
- ✅ 逐行流式处理
- ✅ 零拷贝设计
- ✅ CSV 输入输出支持
- ✅ 时间序列索引（历史数据访问）

**内置函数**

*基础聚合*
- ✅ `sum`, `max`, `min`, `mean` - 聚合函数
- ✅ `length`, `concat` - 数组操作

*高阶函数*
- ✅ `map`, `filter`, `reduce` - 高阶函数

*数组构造*
- ✅ `Range(start, end, step)` - 生成数字序列
- ✅ `Array(size, value|lambda)` - 创建固定长度数组

*数组工具*
- ✅ `first`, `last` - 获取首尾元素
- ✅ `reverse` - 反转数组
- ✅ `sort` - 排序
- ✅ `unique` - 去重

*安全函数*
- ✅ `safe_div(a, b, default)` - 安全除法（避免除零）
- ✅ `safe_get(arr, idx, default)` - 安全数组访问
- ✅ `safe_number(val, default)` - 安全类型转换

*工具函数*
- ✅ `print` - 调试输出
- ✅ `is_null` - Null 检测

**内置变量**
- ✅ `_index` - 当前数据行索引
- ✅ `_total` - 总数据行数
- ✅ `_args` - 当前输入行的所有值
- ✅ `_args_names` - 输入字段名数组

### 4. 包管理系统

**PackageLoader**
- ✅ 从 `.dp` 文件自动加载包
- ✅ 多路径搜索：`packages/`, 当前目录, `stdlib/`
- ✅ 包缓存机制，避免重复加载
- ✅ 包变量和函数导出

## 📝 语言特性展示

### 中文编程支持

```dplang
价格 = 100
数量 = 5
总价 = 价格 * 数量
折扣 = 总价 > 500 ? 总价 * 0.1 : 0
实际支付 = 总价 - 折扣
```

### 向量化运算

```dplang
# 数组逐元素运算
prices = [100, 200, 300]
adjusted = prices * 1.1        # [110, 220, 330]

# 逻辑运算返回布尔数组
high = prices > 150            # [false, true, true]

# 数组间运算
a = [1, 2, 3]
b = [4, 5, 6]
c = a + b                      # [5, 7, 9]
```

### Lambda 表达式

```dplang
# 单参数 Lambda
doubled = map([1, 2, 3], x -> x * 2)

# 多参数 Lambda
result = reduce([1,2,3], 0, (acc, x) -> acc + x)

# 管道运算符
result = [1, 2, 3] |> map(x -> x * 2) |> filter(x -> x > 3)
```

### 条件表达式

```dplang
# 三元表达式
result = x > 10 ? "big" : "small"

# 嵌套三元
grade = score >= 90 ? "A" : score >= 60 ? "B" : "C"

# if-elif-else 语句
if temperature > 30:
    level = "hot"
elif temperature > 20:
    level = "warm"
else:
    level = "cold"

# 链式比较
valid_pe = 0 < pe < 20        # 等价于: (0 < pe) and (pe < 20)
moderate = 0.02 < vol < 0.05  # 链式比较更简洁
```

### 流式数据处理

```dplang
-- INPUT close:number --
-- OUTPUT ma5:number, change:number --

# 访问历史数据（时间序列）
prev_close = close[-1]         # 上一行的 close

# 计算涨跌幅
change = prev_close == null ? 0 : 
         (close - prev_close) / prev_close * 100

# 手动计算5日均线
history = close[-4:0]          # 最近5个值
ma5 = sum(history) / length(history)

return [ma5, change]
```

### 使用新增功能

```dplang
-- INPUT prices:array --
-- OUTPUT stats:array, cleaned:array --

# 数组构造
indices = Range(0, 9)              # [0,1,2,3,4,5,6,7,8,9]
zeros = Array(10, 0)               # [0,0,0,0,0,0,0,0,0,0]
squares = Array(10, i -> i * i)    # [0,1,4,9,16,25,36,49,64,81]

# 数组工具
first_price = first(prices)
last_price = last(prices)
avg_price = mean(prices)
sorted_prices = sort(prices)
unique_prices = unique(prices)

# 安全函数
change_pct = safe_div(last_price - first_price, first_price, 0.0)
safe_value = safe_get(prices, -1, 0.0)

# 内置变量
current_index = _index          # 当前行索引
total_rows = _total             # 总行数
input_names = _args_names       # 输入字段名

stats = [avg_price, change_pct, current_index]
cleaned = unique_prices |> sort() |> reverse()

return [stats, cleaned]
```

### 包系统

**math.dp** (包文件)
```dplang
package math

PI = 3.14159

# 默认参数
circle_area(r, precision = 2):
    area = PI * r * r
    return round(area, precision)

square(x):
    return x * x
```

**main.dp** (主脚本)
```dplang
-- IMPORT math --
-- INPUT radius:number --
-- OUTPUT area:number, diameter:number --

area = math.circle_area(radius)      # 使用默认 precision=2
precise_area = math.circle_area(radius, 4)  # 指定 precision=4
diameter = 2 * math.PI * radius

return [area, diameter]
```

## 🚀 快速开始

### 安装

```bash
# 从源码构建
cargo build --release
```

### 基本使用

**1. 编写一个简单的脚本** (`hello.dp`)

```dplang
-- INPUT name:string, age:number --
-- OUTPUT greeting:string, is_adult:bool --

greeting = "Hello, " + name + "!"
is_adult = age >= 18

return [greeting, is_adult]
```

**2. 准备数据文件** (`data.csv`)

```csv
name,age
Alice,25
Bob,16
Charlie,30
```

**3. 运行脚本**

```bash
# 使用 CSV 文件作为输入
dplang run hello.dp data.csv

# 或交互式输入 (JSON 格式)
dplang run hello.dp
> {"name": "Alice", "age": 25}
> {"name": "Bob", "age": 16}
>
```

**输出结果**:

```csv
greeting,is_adult
Hello, Alice!,true
Hello, Bob!,false
Hello, Charlie!,true
```

### 作为库使用

在你的 Rust 项目中集成 DPLang：

```toml
# Cargo.toml
[dependencies]
dplang = { path = "path/to/dplang" }
```

```rust
use dplang::DPLangInterpreter;
use dplang::runtime::Value;
use std::collections::HashMap;

fn main() {
    let script = r#"
-- INPUT x:number, y:number --
-- OUTPUT sum:number, product:number --

sum = x + y
product = x * y

return [sum, product]
    "#;
    
    let interpreter = DPLangInterpreter::new(script);
    
    let mut input = HashMap::new();
    input.insert("x".to_string(), Value::Number(10.0));
    input.insert("y".to_string(), Value::Number(5.0));
    
    match interpreter.execute(vec![input]) {
        Ok(output) => println!("Result: {:?}", output),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## 📊 测试

项目包含全面的单元测试：

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test lexer
cargo test parser
cargo test executor
```

**测试覆盖**：
- ✅ Lexer: 词法分析
- ✅ Parser: 语法分析
- ✅ Semantic: 语义分析
- ✅ Runtime: 类型系统和运算
- ✅ Executor: 执行器逻辑
- ✅ API: 公共接口
- ✅ PackageLoader: 包加载

## 🏗️ 架构设计

### 执行流程

```
源代码 (*.dp)
    ↓
词法分析 (Lexer)
    ↓
Tokens 流
    ↓
语法分析 (Parser)
    ↓
AST (抽象语法树)
    ↓
语义分析 (Semantic) - 可选
    ↓
执行器 (Executor)
    ↓
输出结果 (Value)
```

### 模块结构

```
dplang/
├── lexer        # 词法分析：源码 -> Tokens
├── parser       # 语法分析：Tokens -> AST
├── semantic     # 语义分析：AST 验证
├── runtime      # 运行时类型系统
├── executor/    # 执行引擎
│   ├── mod.rs            # 主执行器
│   ├── builtin.rs        # 内置函数
│   ├── context.rs        # 执行上下文
│   ├── data_stream.rs    # 流式执行器
│   └── ...               # 其他模块
├── package_loader  # 包加载系统
└── api          # 公共 API 接口
```

## 📂 项目结构

```
DPLang/
├── src/                  # 核心源代码
│   ├── executor/        # 执行器模块
│   ├── parser/          # 语法分析器
│   ├── lexer.rs         # 词法分析器
│   ├── runtime.rs       # 运行时类型系统
│   ├── semantic.rs      # 语义分析器
│   ├── package_loader.rs # 包加载器
│   ├── api.rs           # API 接口
│   ├── lib.rs           # 库入口
│   └── main.rs          # CLI 程序入口
├── scripts/              # 测试和工具脚本
├── Cargo.toml            # 项目配置
├── README.md             # 项目文档
└── QUICKSTART.md         # 快速开始指南
```




## ⚠️ 当前限制

作为一个专注于语言核心的解释器，以下功能需要通过扩展机制实现：

- 业务相关的内置函数（如技术指标、时间函数等）
- 高级类型检查和类型推导
- JIT 编译和并行优化
- 标准库（需要用户通过包机制实现）

## 🔜 后续发展方向

1. **扩展机制完善** - 提供更便捷的函数注册和扩展接口
2. **性能优化** - 并行执行、向量化运算、内存优化
3. **工具链** - 语法高亮、LSP支持、调试工具
4. **标准库** - 官方维护的常用包库
5. **生态建设** - 包管理、文档生成、社区建设

## 📖 文档

- [快速开始指南](QUICKSTART.md) - 5分钟上手 DPLang

## 💡 使用场景

DPLang 适合以下场景：

- **数据处理管道** - CSV/JSON 数据转换和清洗
- **嵌入式脚本引擎** - 为应用提供可配置的计算逻辑
- **规则引擎** - 业务规则的动态配置和执行
- **数据分析工具** - 快速原型开发和数据探索
- **教学工具** - 编译原理和解释器实现学习

## 📜 开源协议

MIT License

---

**DPLang** - 简单、高效、AI友好的数据处理语言 🚀
