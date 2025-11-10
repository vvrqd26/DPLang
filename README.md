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

**内置函数（最小集）**
- ✅ `sum`, `max`, `min` - 聚合函数
- ✅ `length`, `concat` - 数组操作
- ✅ `map`, `filter`, `reduce` - 高阶函数
- ✅ `print` - 调试输出
- ✅ `is_null` - Null 检测

### 4. 包管理系统

**PackageLoader**
- ✅ 从 `.dp` 文件自动加载包
- ✅ 多路径搜索：`packages/`, 当前目录, `stdlib/`
- ✅ 包缓存机制，避免重复加载
- ✅ 包变量和函数导出

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

```
源代码 (Source Code)
    ↓
词法分析 (Lexer) → Tokens
    ↓
语法分析 (Parser) → AST
    ↓
执行器 (Executor) → 运行时值 (Value)
```

## 📂 项目结构

```
DPLang/
├── src/                    # 核心源代码
│   ├── executor/            # 执行器模块
│   ├── orchestration/       # 任务编排系统
│   ├── parser/              # 语法分析器
│   ├── streaming/           # 流式处理
│   ├── lexer.rs             # 词法分析器
│   ├── runtime.rs           # 运行时值和运算
│   ├── semantic.rs          # 语义分析器
│   ├── indicators.rs        # 技术指标库
│   ├── package_loader.rs    # 包加载器
│   ├── api.rs               # API 接口
│   ├── lib.rs               # 库入口
│   └── main.rs              # 可执行程序入口
├── benches/                 # 性能基准测试
│   ├── indicator_benchmarks.rs
│   └── optimization_benchmarks.rs
├── tests/                   # 集成测试 (预留)
├── packages/                # DPLang 标准包库
│   └── math.dp              # 数学库包
├── scripts/                 # 测试和工具脚本
│   ├── generate_test_data.py
│   ├── generate_stock_data.py
│   ├── performance_test.py
│   ├── extract_result.py
│   └── test_stock_indicators.ps1
├── docs/                    # 项目文档
│   └── testing/             # 测试文档
│       └── TEST_SUMMARY.md
├── test_data/               # 测试数据目录 (Git 忽略)
│   └── .gitkeep
├── output/                  # 运行输出目录 (Git 忽略)
│   └── .gitkeep
├── Cargo.toml               # Rust 项目配置
├── Cargo.lock               # 依赖锁定文件
├── README.md                # 项目主文档
├── QUICKSTART.md            # 快速开始指南
├── DAEMON_MODE_GUIDE.md     # 守护进程模式指南
└── ORCHESTRATION_QUICKSTART.md  # 编排系统快速开始
```

### 目录说明

- **src/**: 核心源代码，包含词法分析、语法分析、执行器等模块
- **benches/**: 性能基准测试，用于性能跟踪和优化
- **tests/**: 集成测试目录 (预留使用)
- **examples/**: 示例脚本和配置文件，帮助用户快速上手
- **packages/**: DPLang 标准包库，存放可重用的包文件
- **scripts/**: 测试数据生成、性能测试等工具脚本
- **docs/**: 项目文档，包含测试总结等详细文档
- **test_data/**: 测试数据目录 (由 Git 忽略)
- **output/**: 运行输出目录 (由 Git 忽略)




## 🔜 下一步计划

### 最近更新 (v0.3.0) - 2024-11-10 ✅ 已完成
1. **场景化命令行接口** ✅ 已完成
   - ✅ 新增 `calc` 命令 - 单次指标计算
   - ✅ 新增 `backtest` 命令 - 策略回测（自动统计收益、胜率）
   - ✅ 新增 `screen` 命令 - 策略选股（批量筛选）
   - ✅ 新增 `monitor` 命令 - 实时监控（取代 daemon）
   - ✅ 新增 `server` 命令 - 任务编排（取代 orchestrate）
   - ✅ 创建完整的场景化使用指南 [SCENARIOS_GUIDE.md](SCENARIOS_GUIDE.md)
   - ✅ 提供四大场景示例脚本（指标计算、回测、选股、监控）



### 后续计划
2. **性能优化** - 并行执行、向量化运算
3. **类型推导** - 更智能的类型系统
4. **调试工具** - 断点、变量监视、执行跟踪
5. **标准库扩展** - 字符串处理、日期时间、JSON解析

## 📄 参考文档

### 用户文档
- [**场景化使用指南**](SCENARIOS_GUIDE.md) - **四大场景完整使用指南**（推荐）
- [语言参考手册](docs/LANGUAGE_GUIDE.md) - 完整的语言使用指南
- [快速开始指南](QUICKSTART.md) - 快速上手
- [守护进程模式指南](DAEMON_MODE_GUIDE.md) - 长驻服务模式
- [编排系统快速开始](ORCHESTRATION_QUICKSTART.md) - 任务编排系统

### 开发者文档（内部）
- [核心设计](dev_logs/1.核心设计.md) - 设计原则和架构
- [语法参考](dev_logs/2.语法参考.md) - 详细语法规范
- [完整示例](dev_logs/5.完整示例.md) - 高级示例代码
- [内置函数参考](dev_logs/4.内置函数参考.md) - 所有内置函数
- [解释器实现设计](dev_logs/7.解释器实现设计.md) - 实现细节

## 📜 License

本项目采用 MIT License with Commercial Use Notice 许可协议。

**使用条款：**
- ✅ 可自由使用、修改、分发，但需注明原作者
- ✅ 小规模商业应用和学习使用不受限制


详见 [LICENSE](LICENSE) 文件。

---

**DPLang** - 让数据分析更简单 🚀
