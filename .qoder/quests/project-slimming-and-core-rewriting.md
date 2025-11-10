# DPLang 项目瘦身与核心重构设计

## 战略目标

DPLang 回归本质：**一个简单、高效、AI友好的流式处理语言解释器**

当前项目已偏离初心，混入了过多上层应用（回测、编排、监控等），导致复杂度失控。本次重构目标是：

- 专注于语言解释器核心
- 去除所有上层应用功能
- 精简内置函数库至最小必需集
- 开放扩展机制，便于嵌入式集成
- 清晰的职责边界：解释器只负责执行脚本，不负责具体业务

## 一、架构瘦身原则

### 1.1 核心保留范围

**保留模块（解释器核心）**
- Lexer：词法分析器
- Parser：语法分析器
- Semantic Analyzer：语义分析器
- Runtime：运行时类型系统
- Executor：执行器核心
- Package Loader：包加载机制

**保留功能**
- 基本语法解析与执行
- 数据流处理引擎（流式计算核心）
- 包机制与模块化支持
- Lambda 表达式与高阶函数
- 基础内置函数（见1.3）

### 1.2 彻底移除范围

**移除上层应用模块**
- backtest/：整个回测系统（8个文件）
- orchestration/：整个任务编排系统（9个文件）
- indicators.rs：专用技术指标库（12.8KB）
- streaming/：CSV流式写入（仅保留基本CSV解析）

**移除CLI场景化命令**
- `backtest`：策略回测
- `screen`：策略选股
- `monitor`：实时监控
- `server`：任务编排服务器
- `demo`：内置演示

**移除技术指标内置函数**
- SMA、EMA、MACD、RSI、BOLL、ATR、KDJ
- BUY、SELL、CLOSE、POSITION（仓位操作）
- ref、past、offset、window（时间序列访问）

**移除业务相关依赖**
- toml：仅用于编排配置
- chrono：时间日期功能（过于业务化）

### 1.3 最小内置函数集

**保留原则**：仅保留让解释器正常运行和数据流处理的必需函数

**核心语言函数（保留）**
- `print`：输出调试
- `map`、`filter`、`reduce`：高阶函数（语言核心特性）
- `is_null`：null值检测

**基础数据操作（保留）**
- `sum`：求和
- `max`、`min`：最值
- `length`：数组长度（需新增）
- `concat`：数组拼接（需新增）

**移除的函数**
- Null处理函数：coalesce、nvl、if_null、nullif（可通过语法实现）
- 时间日期函数：now、today、parse_time等（业务功能）
- 技术指标函数：所有金融指标
- 仓位操作函数：所有交易相关

## 二、保留模块设计

### 2.1 目录结构（简化后）

```
src/
├── lexer.rs              # 词法分析器
├── parser/
│   ├── mod.rs           # 语法分析器
│   └── ast.rs           # AST定义
├── semantic.rs          # 语义分析器
├── runtime.rs           # 运行时类型系统
├── executor/
│   ├── mod.rs           # 执行器主模块
│   ├── builtin.rs       # 最小内置函数集
│   ├── expression.rs    # 表达式求值
│   ├── statement.rs     # 语句执行
│   ├── context.rs       # 执行上下文
│   ├── context_pool.rs  # 上下文池（流式计算）
│   ├── data_stream.rs   # 数据流执行器
│   └── columnar_storage.rs # 列式存储（流式计算优化）
├── package_loader.rs    # 包加载机制
├── api.rs               # 公共API接口
├── lib.rs               # 库入口
└── main.rs              # CLI工具（仅保留基础功能）
```

**移除文件**
- src/backtest/（整个目录）
- src/orchestration/（整个目录）
- src/indicators.rs
- src/streaming/csv_writer.rs（仅保留基本CSV功能）
- examples/configs/（编排配置）
- scripts/run_golden_cross_backtest.ps1
- scripts/test_orchestrate.ps1
- scripts/test_stock_indicators.ps1
- 文档：BACKTEST_TESTING.md、DAEMON_MODE_GUIDE.md、ORCHESTRATION_QUICKSTART.md、SCENARIOS_GUIDE.md

### 2.2 依赖项（精简后）

**保留依赖**
```toml
[dependencies]
rust_decimal = "1.33"    # 高精度计算
csv = "1.3"              # CSV基本解析
thiserror = "1.0"        # 错误处理
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"       # JSON支持
```

**移除依赖**
```toml
toml = "0.8"    # 仅用于编排配置
chrono = "0.4"  # 时间日期（业务功能）
```

### 2.3 核心API设计

**保持现有API接口**

DPLang作为库，对外提供三种调用方式：

```rust
// 方式1：HashMap输入
pub fn execute(script: &str, input: Vec<HashMap<String, Value>>) 
    -> Result<Vec<HashMap<String, Value>>, RuntimeError>

// 方式2：JSON格式
pub fn execute_json(script: &str, json_input: &str) 
    -> Result<String, RuntimeError>

// 方式3：CSV格式
pub fn execute_csv(script: &str, csv_input: &str) 
    -> Result<String, RuntimeError>
```

**新增：内置函数扩展机制**

允许外部注册自定义内置函数：

```rust
// 扩展API设计
pub trait BuiltinFunction {
    fn name(&self) -> &str;
    fn execute(&self, args: &[Value]) -> Result<Value, RuntimeError>;
}

pub struct DPLangInterpreter {
    custom_functions: HashMap<String, Box<dyn BuiltinFunction>>,
}

impl DPLangInterpreter {
    // 注册自定义内置函数
    pub fn register_function(&mut self, func: Box<dyn BuiltinFunction>)
    
    // 批量注册
    pub fn register_functions(&mut self, funcs: Vec<Box<dyn BuiltinFunction>>)
}
```

**使用示例**

```rust
// 外部定义技术指标库
struct SMAFunction;
impl BuiltinFunction for SMAFunction {
    fn name(&self) -> &str { "SMA" }
    fn execute(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        // 实现SMA逻辑
    }
}

// 在应用中使用
let mut interpreter = DPLangInterpreter::new();
interpreter.register_function(Box::new(SMAFunction));
let result = interpreter.execute(script, input)?;
```

## 三、CLI工具简化

### 3.1 保留命令

**仅保留基础功能**

```bash
dplang run <script.dp> [data.csv]    # 执行脚本
dplang help                          # 帮助信息
dplang version                       # 版本信息
```

### 3.2 移除命令

```bash
dplang calc       # 移除（与run重复）
dplang backtest   # 移除（上层应用）
dplang screen     # 移除（上层应用）
dplang monitor    # 移除（上层应用）
dplang server     # 移除（上层应用）
dplang demo       # 移除（可通过example代替）
```

### 3.3 简化的CLI实现

main.rs 精简至基础脚本执行功能：

- 读取脚本文件
- 解析CSV或JSON输入
- 执行脚本
- 输出结果
- 基本错误处理

**预计代码量**：从1107行降至200行以内

## 四、内置函数重构

### 4.1 builtin.rs 精简设计

**精简前**：1093行，包含34个内置函数
**精简后**：预计200-300行，包含8-10个核心函数

**保留函数列表**

| 函数名 | 功能 | 保留原因 |
|--------|------|----------|
| print | 输出调试 | 开发调试必需 |
| map | 数组映射 | 语言核心特性 |
| filter | 数组过滤 | 语言核心特性 |
| reduce | 数组归约 | 语言核心特性 |
| sum | 求和 | 基础数据操作 |
| max | 最大值 | 基础数据操作 |
| min | 最小值 | 基础数据操作 |
| is_null | null检测 | null语义支持 |
| length | 数组长度 | 基础数组操作 |
| concat | 数组拼接 | 基础数组操作 |

**移除函数分类**

- Null处理（4个）：coalesce、nvl、if_null、nullif
- 时间日期（15个）：now、today、parse_time、format_time等
- 技术指标（7个）：SMA、EMA、MACD、RSI、BOLL、ATR、KDJ
- 时间序列（4个）：ref、past、offset、window
- 仓位操作（4个）：BUY、SELL、CLOSE、POSITION

### 4.2 扩展机制设计

**核心理念**：解释器不内置业务函数，通过扩展机制由外部提供

**扩展点设计**

Executor内部增加函数注册表：

```rust
pub struct Executor {
    // 现有字段
    context: Context,
    context_pool: ContextPool,
    
    // 新增：外部注册的函数
    custom_builtins: HashMap<String, Arc<dyn BuiltinFunction>>,
}

impl Executor {
    // 查找函数时先查自定义，再查内置
    fn execute_builtin(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        // 1. 查找自定义函数
        if let Some(func) = self.custom_builtins.get(name) {
            return func.execute(args);
        }
        
        // 2. 查找内置函数
        match name {
            "print" => self.builtin_print(args),
            "map" => self.builtin_map(args),
            // ... 其他核心函数
            _ => Err(RuntimeError::undefined_function(name)),
        }
    }
}
```

**扩展函数Trait**

```rust
pub trait BuiltinFunction: Send + Sync {
    fn name(&self) -> &str;
    fn execute(&self, args: &[Value]) -> Result<Value, RuntimeError>;
    
    // 可选：参数验证
    fn validate_args(&self, args: &[Value]) -> Result<(), RuntimeError> {
        Ok(())
    }
}
```

## 五、数据流引擎保留

### 5.1 核心价值

DPLang的核心竞争力是**高性能流式计算引擎**，必须保留：

- DataStreamExecutor：行级流式处理
- ContextPool：上下文复用
- ColumnarStorage：列式存储优化
- 增量计算架构（虽然移除具体指标，但保留机制）

### 5.2 流式计算API

保留现有流式执行接口：

```rust
pub struct DataStreamExecutor {
    script: Script,
    input_matrix: Vec<HashMap<String, Value>>,
    context_pool: ContextPool,
}

impl DataStreamExecutor {
    pub fn new(script: Script, input_matrix: Vec<HashMap<String, Value>>) -> Self
    pub fn execute_all(&mut self) -> Result<Vec<HashMap<String, Value>>, RuntimeError>
}
```

### 5.3 扩展机制与流式计算结合

自定义函数可以是有状态的（支持增量计算）：

```rust
pub trait StatefulBuiltinFunction: BuiltinFunction {
    // 重置状态（处理新数据流时调用）
    fn reset(&mut self);
    
    // 状态化执行（增量更新）
    fn execute_stateful(&mut self, args: &[Value]) -> Result<Value, RuntimeError>;
}
```

## 六、包机制保留

### 6.1 保留原因

包机制是模块化和复用的基础，是语言核心特性，必须保留：

- 函数定义与复用
- 作用域隔离
- 公开/私有访问控制
- 包导入机制（IMPORT声明）

### 6.2 包加载器保留

package_loader.rs 保持不变：

- 文件系统包加载
- 包缓存机制
- 搜索路径支持

### 6.3 包与扩展的关系

- 包脚本（.dp文件）：用户自定义逻辑复用
- 扩展函数（Rust代码）：高性能原生功能

两者互补：
- 简单逻辑用包脚本
- 性能敏感或复杂算法用扩展函数

## 七、迁移策略

### 7.1 上层功能迁移方案

**回测系统**

从内置功能改为独立项目或示例：

- 项目定位：基于DPLang的回测应用
- 实现方式：使用DPLang API + 自定义扩展函数
- 代码位置：examples/backtest/（作为应用示例）

**任务编排系统**

改为独立服务项目：

- 项目定位：DPLang任务调度框架
- 实现方式：包装DPLang解释器，添加HTTP API和任务管理
- 代码位置：独立仓库或examples/orchestration/

**技术指标库**

改为扩展库：

- 项目定位：DPLang金融指标扩展包
- 实现方式：实现BuiltinFunction trait
- 代码位置：独立crate（dplang-indicators）

### 7.2 文档迁移

**保留文档**
- README.md（重写，聚焦语言核心）
- QUICKSTART.md（简化为基础教程）
- CHANGELOG.md（添加v0.4.0重构说明）

**移除或归档**
- BACKTEST_TESTING.md
- DAEMON_MODE_GUIDE.md
- ORCHESTRATION_QUICKSTART.md
- SCENARIOS_GUIDE.md

**新增文档**
- EXTENSION_GUIDE.md（扩展开发指南）
- EMBEDDING_GUIDE.md（嵌入式集成指南）
- API_REFERENCE.md（API参考文档）

### 7.3 示例迁移

**保留示例**
- examples/basic/：基础语法示例
- examples/advanced/：高级特性示例
- examples/packages/：包机制示例

**迁移示例**
- examples/backtest/：回测应用示例（展示如何基于DPLang构建上层应用）
- examples/indicators/：技术指标扩展示例（展示如何开发扩展函数）
- examples/embedding/：嵌入式集成示例（展示如何在Rust/Python/Java中使用）

## 八、实施计划

### 阶段一：代码移除与重构（2-3天）

**任务清单**

1. 移除模块文件
   - 删除 src/backtest/（8个文件）
   - 删除 src/orchestration/（9个文件）
   - 删除 src/indicators.rs
   - 删除 src/streaming/csv_writer.rs

2. 精简 builtin.rs
   - 保留10个核心函数
   - 移除34个业务函数
   - 代码量从1093行降至约250行

3. 简化 main.rs
   - 移除场景化命令（5个）
   - 保留基础执行功能
   - 代码量从1107行降至约200行

4. 精简 lib.rs
   - 移除已删除模块的导出
   - 更新公共API

5. 更新 Cargo.toml
   - 移除 toml 依赖
   - 移除 chrono 依赖
   - 移除 benchmark 配置（移至示例项目）

**预期成果**
- 代码量减少约50%（从约15,000行降至7,500行）
- 模块数量减少60%（从23个降至9个）
- 依赖减少40%（从5个降至3个）

### 阶段二：扩展机制实现（1-2天）

**任务清单**

1. 定义扩展Trait
   - BuiltinFunction trait
   - StatefulBuiltinFunction trait
   - 完善API文档

2. 修改Executor
   - 添加 custom_builtins 字段
   - 修改函数查找逻辑
   - 支持函数注册

3. 扩展DPLangInterpreter
   - 添加 register_function 方法
   - 添加 register_functions 方法
   - 线程安全处理（Arc + RwLock）

4. 单元测试
   - 测试函数注册
   - 测试自定义函数调用
   - 测试与内置函数优先级

**预期成果**
- 扩展机制完全可用
- API文档完善
- 测试覆盖率100%

### 阶段三：示例与文档（2-3天）

**任务清单**

1. 迁移上层功能为示例
   - examples/backtest/：回测应用示例
   - examples/indicators/：指标扩展示例
   - examples/orchestration/：编排框架示例

2. 创建扩展开发示例
   - 简单扩展函数（字符串处理）
   - 有状态扩展函数（EMA指标）
   - 批量注册示例

3. 编写指南文档
   - EXTENSION_GUIDE.md
   - EMBEDDING_GUIDE.md
   - API_REFERENCE.md

4. 重写README.md
   - 聚焦语言核心定位
   - 突出扩展性和嵌入式友好
   - 简化快速开始

5. 归档旧文档
   - 创建 docs/archive/
   - 移动场景化文档

**预期成果**
- 示例代码完整可运行
- 文档清晰易懂
- 定位明确

### 阶段四：测试与发布（1天）

**任务清单**

1. 完整测试
   - 单元测试（cargo test）
   - 集成测试
   - 示例验证

2. 性能基准测试
   - 确保瘦身后性能不退化
   - 对比v0.3.0版本

3. 版本发布
   - 更新版本号至 v0.4.0
   - 编写 CHANGELOG_v0.4.0.md
   - 打标签发布

**预期成果**
- 所有测试通过
- 性能持平或提升
- v0.4.0 正式发布

## 九、预期效果

### 9.1 代码量对比

| 模块 | v0.3.0 | v0.4.0 | 减少 |
|------|--------|--------|------|
| src/main.rs | 1,107行 | ~200行 | -82% |
| src/executor/builtin.rs | 1,093行 | ~250行 | -77% |
| src/backtest/ | 8文件 | 0文件 | -100% |
| src/orchestration/ | 9文件 | 0文件 | -100% |
| src/indicators.rs | 12.8KB | 0KB | -100% |
| **总计** | ~15,000行 | ~7,500行 | **-50%** |

### 9.2 职责边界清晰

**v0.4.0 核心职责**
- ✅ 解析并执行DPLang脚本
- ✅ 提供高性能流式计算引擎
- ✅ 支持包机制和模块化
- ✅ 提供扩展机制
- ✅ 简洁的API接口

**明确排除职责**
- ❌ 不负责回测逻辑
- ❌ 不负责任务编排
- ❌ 不负责业务指标实现
- ❌ 不负责特定场景应用

### 9.3 扩展性提升

**v0.3.0**：封闭式设计，新功能需修改核心代码
**v0.4.0**：开放式设计，新功能通过扩展机制添加

**扩展示例**

```rust
// 技术指标库（独立crate）
use dplang::{BuiltinFunction, Value, RuntimeError};

pub struct IndicatorsExtension;

impl IndicatorsExtension {
    pub fn register_all() -> Vec<Box<dyn BuiltinFunction>> {
        vec![
            Box::new(SMAFunction),
            Box::new(EMAFunction),
            Box::new(MACDFunction),
            // ... 更多指标
        ]
    }
}

// 应用代码
let mut interpreter = DPLangInterpreter::new();
interpreter.register_functions(IndicatorsExtension::register_all());
```

### 9.4 嵌入式友好

**v0.3.0**：依赖多，体积大，难以嵌入
**v0.4.0**：依赖少，体积小，易于嵌入

**嵌入场景**
- Rust项目：直接依赖crate
- Python项目：通过PyO3封装
- Java项目：通过JNI封装
- WASM：编译为WebAssembly
- 移动端：嵌入iOS/Android应用

### 9.5 性能预期

**理论分析**
- 移除业务代码不影响核心性能
- 减少依赖可能略微提升编译速度
- 扩展机制使用trait对象，性能损失<5%

**基准测试目标**
- 流式计算速度：≥20,000行/秒（v0.3.0为23,000）
- 编译时间：减少20-30%
- 二进制体积：减少30-40%

## 十、风险与应对

### 10.1 兼容性风险

**风险**：v0.4.0 移除大量功能，现有用户代码可能失效

**应对**
- 明确标记为 Breaking Change
- 提供迁移指南
- 在 examples/ 中提供迁移后的替代方案
- 保留v0.3.0分支供过渡使用

### 10.2 功能缺失风险

**风险**：用户可能需要被移除的功能（如技术指标）

**应对**
- 提供独立的扩展库（dplang-indicators）
- 编写详细的扩展开发指南
- 示例代码展示如何重新实现
- 社区可以贡献更多扩展库

### 10.3 学习成本风险

**风险**：扩展机制增加使用复杂度

**应对**
- 提供预编译的扩展库（零配置使用）
- 简化扩展API设计
- 丰富的示例代码
- 详细的文档和教程

### 10.4 性能风险

**风险**：扩展机制可能引入性能损失

**应对**
- 使用静态分发（泛型）而非动态分发
- 提供内联优化选项
- 性能基准测试验证
- 允许关键函数内置到核心（按需）

## 十一、成功标准

### 11.1 代码质量

- ✅ 代码量减少至少40%
- ✅ 模块耦合度降低（模块间依赖<3个）
- ✅ 单一职责原则：每个模块职责清晰
- ✅ 测试覆盖率≥80%

### 11.2 性能指标

- ✅ 流式计算性能≥20,000行/秒
- ✅ 编译时间减少≥20%
- ✅ 二进制体积减少≥30%
- ✅ 内存占用不增加

### 11.3 易用性

- ✅ API数量≤5个（核心接口）
- ✅ 扩展函数注册≤3行代码
- ✅ 快速开始文档≤500字
- ✅ 示例代码可直接运行

### 11.4 扩展性

- ✅ 扩展机制文档完善
- ✅ 提供≥3个扩展示例
- ✅ 支持有状态扩展函数
- ✅ 线程安全

## 十二、后续演进

### 12.1 短期计划（v0.4.x）

- 完善扩展机制（更多trait方法）
- 提供官方扩展库（字符串、数学、文件IO）
- 优化错误信息（更友好的提示）
- 改进包管理（包版本、依赖解析）

### 12.2 中期计划（v0.5.x）

- 增加类型系统（可选静态类型）
- 支持协程（异步流式处理）
- WASM支持（浏览器中运行）
- FFI机制（调用C库）

### 12.3 长期愿景（v1.0）

- 成为嵌入式脚本语言的标准选择
- 生态丰富（社区扩展库≥50个）
- 多语言绑定（Python、Java、Go、Node.js）
- 企业级应用案例（≥10个）

## 十三、总结

本次重构是DPLang发展的关键转折点，将项目从"金融数据处理工具"回归到"通用流式脚本语言解释器"的本质定位。

**核心价值**
- 简单：专注语言核心，移除业务逻辑
- 高效：保持流式计算优势，性能不妥协
- 开放：扩展机制让生态自然生长
- 友好：易于理解、易于嵌入、易于扩展

**战略意义**
- 清晰定位：不做"大而全"，做"小而美"
- 可持续性：核心稳定，扩展灵活
- 社区友好：降低贡献门槛
- 商业价值：适合嵌入商业产品

DPLang v0.4.0 将是一个**纯粹的语言解释器**，专注于做好一件事：高效执行流式数据处理脚本。上层应用、业务逻辑、特定领域功能，交给扩展机制和生态社区。
