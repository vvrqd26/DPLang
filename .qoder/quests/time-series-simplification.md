# 时间序列语法简化设计

## 设计目标

将当前基于函数调用的时间序列访问语法（如 `ref("close", 1)`、`past("close", 5)`）简化为更直观的类Python下标语法（如 `close[-1]`、`close[-5:]`），提升代码可读性和编写效率。

## 当前问题分析

### 现有语法的不足

当前时间序列访问需要使用函数调用并传入字符串形式的变量名：

``dplang
-- INPUT close:number --
-- OUTPUT ma5:number --

# 获取前一个收盘价
prev_close = ref("close", 1)

# 获取过去5个值
prices = past("close", 5)

# 获取滑动窗口
window_data = window("close", 5)

ma5 = sum(window_data) / 5
return [ma5]
```

**存在的问题**：
1. 变量名需要以字符串形式传递，容易出错（拼写错误无法在编译时发现）
2. 语法冗长，不够直观
3. 函数名语义不够清晰（ref、past、window 的区别需要记忆）
4. 与主流编程语言的数组访问习惯不一致

## 设计方案

### 语法设计

采用类Python的下标语法，支持单个索引和切片访问：

#### 1. 单值引用

使用负数索引表示历史数据（符合Python习惯）：

``dplang
-- INPUT close:number --
-- OUTPUT change:number --

# close[-1] 表示前1个周期的值
prev_close = close[-1]

# close[-5] 表示前5个周期的值
close_5_ago = close[-5]

# 计算涨跌幅
change = prev_close == null ? null : (close - prev_close) / prev_close * 100

return [change]
```

**语义说明**：
- `变量名[-n]` 表示当前值之前第 n 个周期的值
- 当前值使用 `变量名` 或 `变量名[0]` 访问
- 历史数据不足时返回 `null`

#### 2. 切片访问（数组）

支持Python风格的切片语法获取连续的时间序列数据：

``dplang
-- INPUT close:number --
-- OUTPUT ma5:number, volatility:number --

# close[-5:] 表示过去5个值到当前（包含当前，共6个元素）
window_6 = close[-5:]       # [t-5, t-4, t-3, t-2, t-1, t]

# close[-5:-1] 表示过去5个到过去1个（不包含当前，5个元素）
past_5 = close[-5:-1]       # [t-5, t-4, t-3, t-2, t-1]

# close[-10:0] 等同于 close[-10:] （显式包含当前）
window_11 = close[-10:0]    # [t-10, ..., t-1, t]

# 基于窗口计算指标
ma5 = sum(close[-5:]) / 6
volatility = max(close[-5:]) - min(close[-5:])

return [ma5, volatility]
```

**切片语义规则**：

| 语法 | 含义 | 结果数组范围 | 元素数量 |
|------|------|-------------|---------|
| `close[-5:]` | 过去5个到当前 | `[t-5, t-4, ..., t-1, t]` | 6个 |
| `close[-5:-1]` | 过去5个到过去1个 | `[t-5, t-4, ..., t-2, t-1]` | 5个 |
| `close[-10:0]` | 过去10个到当前 | `[t-10, ..., t-1, t]` | 11个 |
| `close[-3:]` | 过去3个到当前 | `[t-3, t-2, t-1, t]` | 4个 |
| `close[:]` | 所有历史到当前 | `[t-n, ..., t-1, t]` | n+1个 |

**边界处理**：
- 当历史数据不足时，用 `null` 填充缺失位置
- 切片始终返回数组类型，即使只有一个元素
- 空切片（如历史不足且范围无效）返回空数组 `[]`

#### 3. 完整示例

``dplang
-- INPUT open:number, high:number, low:number, close:number, volume:number --
-- OUTPUT ma5:number, ma10:number, signal:string, atr:number --

# 单值引用
prev_close = close[-1]
prev_volume = volume[-1]

# 切片窗口计算
ma5 = sum(close[-4:]) / 5      # 最近5个值的平均（包括当前）
ma10 = sum(close[-9:]) / 10    # 最近10个值的平均

# 判断金叉死叉
prev_ma5 = sum(close[-5:-1]) / 5
prev_ma10 = sum(close[-10:-1]) / 10

signal = (ma5 > ma10 and prev_ma5 <= prev_ma10) ? "金叉" : 
         (ma5 < ma10 and prev_ma5 >= prev_ma10) ? "死叉" : "持有"

# 计算ATR（真实波幅）
high_5 = high[-4:]
low_5 = low[-4:]
close_5 = close[-5:-1]  # 前一日收盘价

# 计算真实波幅数组
tr_array = map(
    Range(0, 4), 
    i -> max(high_5[i] - low_5[i], 
             abs(high_5[i] - close_5[i]), 
             abs(low_5[i] - close_5[i]))
)
atr = sum(tr_array) / 5

return [ma5, ma10, signal, atr]
```

### 语法映射关系

新旧语法对照表：

| 旧语法（函数调用） | 新语法（下标） | 说明 |
|------------------|--------------|------|
| `ref("close", 1)` | `close[-1]` | 前1个值 |
| `ref("close", 5)` | `close[-5]` | 前5个值 |
| `offset("close", 1)` | `close[-1]` | 同ref |
| `past("close", 5)` | `close[-5:-1]` | 过去5个值（不含当前） |
| `window("close", 5)` | `close[-4:]` | 窗口5个值（含当前） |
| - | `close[:]` | 全部历史（含当前） |

## 技术实现要点

### 词法分析层（Lexer）

**新增Token类型**：

```
Token::LeftBracket       # [
Token::RightBracket      # ]
Token::Colon             # : (用于切片)
```

**注意事项**：
- `[` 和 `]` 已用于数组字面量，需区分上下文
- `:` 需与函数定义中的类型标注语法区分
- 负数索引需正确解析（`-` 作为负号而非减法运算符）

### 语法分析层（Parser）

**AST 新增节点类型**：

**IndexAccess（索引访问）**：
- 字段：
  - `base`: 被访问的表达式（变量名或其他表达式）
  - `index`: 索引表达式（可为负数）
- 示例：`close[-1]` → IndexAccess { base: "close", index: -1 }

**SliceAccess（切片访问）**：
- 字段：
  - `base`: 被访问的表达式
  - `start`: 起始索引（可选，默认为历史起点）
  - `end`: 结束索引（可选，默认为当前0）
- 示例：
  - `close[-5:]` → SliceAccess { base: "close", start: Some(-5), end: None }
  - `close[-5:-1]` → SliceAccess { base: "close", start: Some(-5), end: Some(-1) }
  - `close[:]` → SliceAccess { base: "base", start: None, end: None }

**解析优先级**：
- 索引/切片访问优先级应高于算术运算符，低于函数调用
- 优先级顺序：函数调用 > 索引/切片 > 乘除 > 加减

**二义性消解**：

表达式 | 解析为 | 消歧规则
-------|--------|----------
`[1, 2, 3]` | 数组字面量 | 行首或赋值右侧
`arr[1]` | 索引访问 | 紧跟标识符后
`close[-5:]` | 切片访问 | 包含冒号
`f(x)[0]` | 函数调用结果的索引 | 优先解析函数调用

### 语义分析层（Semantic Analyzer）

**检查项**：

1. **基础变量检查**：
   - 被访问的变量必须已定义（在INPUT或之前赋值）
   - 与当前函数调用语义检查保持一致

2. **索引类型检查**：
   - 索引必须为整数类型（编译时常量或运行时表达式）
   - 切片的 start/end 必须为整数或空

3. **上下文检查**：
   - 时间序列访问（负索引/切片）只能在 DataStreamExecutor 上下文使用
   - 在非流式执行环境中使用时报错或警告

**类型推断**：
- 单值索引访问：返回类型与原变量类型一致（可能为 null）
- 切片访问：返回数组类型 `Array<T>`，其中 T 为原变量类型

### 执行器层（Executor）

**执行逻辑**：

**IndexAccess 执行**：
1. 计算 base 表达式（通常为变量名）
2. 计算 index 表达式得到偏移量 n
3. 根据偏移量类型：
   - `n == 0`：返回当前值
   - `n < 0`：从 DataStreamExecutor 获取历史值 `get_history(var_name, |n|)`
   - `n > 0`：暂不支持未来数据访问，返回错误或 null
4. 历史不足时返回 `null`

**SliceAccess 执行**：
1. 计算 base 表达式
2. 计算 start 和 end 索引（处理默认值）
3. 根据索引范围从 DataStreamExecutor 批量获取历史数据
4. 构造数组并返回
5. 数据不足部分用 `null` 填充

**与 DataStreamExecutor 集成**：
- 复用现有的 `get_input_history()` 和 `get_output_history()` 方法
- 新增 `get_input_slice(name, start_offset, end_offset)` 批量获取方法
- 新增 `get_output_slice(name, start_offset, end_offset)` 批量获取方法
- **关键优化**：返回 `ArraySlice` 而非拷贝数据

**当前数据结构分析**：

```
// 当前 DataStreamExecutor 数据结构
pub struct DataStreamExecutor {
    // 输入矩阵使用 Rc 共享（已优化）
    input_matrix: Rc<Vec<HashMap<String, Value>>>,
    
    // 输出矩阵使用 Vec（存在拷贝问题）
    output_matrix: Vec<HashMap<String, Value>>,
    
    // ...
}
```

**性能问题识别**：

1. **输入矩阵（已优化）**：
   - 使用 `Rc<Vec<...>>` 共享，创建执行器时不拷贝
   - ✅ 符合零拷贝设计

2. **输出矩阵（存在问题）**：
   - 每次 `get_output_history()` 都调用 `.cloned()`
   - 📍 **性能瓶颈**：重复访问时重复拷贝
   - 建议优化方向：
     - 返回 `&Value` 引用而非克隆
     - 或使用 `Rc<Value>` 包装输出值

3. **切片访问（新增场景）**：
   - 如果返回 `Vec<Value>`，会发生数据拷贝
   - ❌ **必须避免**：大窗口切片（如100个值）的完整拷贝
   - ✅ **解决方案**：返回 `ArraySlice` 类型

**优化建议**：

```
// 优化后的数据结构
pub struct DataStreamExecutor {
    // 输入矩阵（保持不变）
    input_matrix: Rc<Vec<HashMap<String, Value>>>,
    
    // 输出矩阵改用 Rc 包装值（可选优化）
    output_matrix: Vec<HashMap<String, Rc<Value>>>,
    
    // 为切片访问准备的列式缓存（可选，按需实现）
    column_cache: HashMap<String, Rc<Vec<Value>>>,
}
```

**实施优先级**：
- **P0（必须）**：切片返回 `ArraySlice`，避免大数据拷贝
- **P1（推荐）**：输出矩阵值使用 `Rc<Value>`，减少克隆开销
- **P2（优化）**：列式缓存，适用于频繁切片访问场景

### 运行时层（Runtime）

**值类型处理**：
- 单值访问返回：`Value::Number`、`Value::String` 或 `Value::Null`
- 切片访问返回：`Value::ArraySlice` （新增类型，零拷贝引用）
- 保持与现有 Value 类型系统一致

**新增 Value 类型变体**：

```
// Value 枚举新增：
ArraySlice {
    /// 源数据的共享引用
    source: Rc<Vec<Value>>,
    /// 切片起始索引（在源数据中的绝对位置）
    start: usize,
    /// 切片长度
    len: usize,
}
```

**设计理由**：
1. **零拷贝**：切片不复制数据，仅持有 `Rc` 引用
2. **内存共享**：多个切片可以共享同一份底层数据
3. **懒计算兼容**：与现有 `Vec<Value>` 互操作时自动转换

**错误处理**：
- 索引越界：返回 `null`（而非报错）
- 类型不匹配：`RuntimeError::TypeError`
- 非流式上下文使用：`RuntimeError::ContextError`

## 边界情况处理

### 数据不足场景

| 场景 | 示例 | 行为 |
|------|------|------|
| 第1行数据访问历史 | `close[-1]` | 返回 `null` |
| 第3行访问前5个 | `close[-5:]` | 返回 `[null, null, t-2, t-1, t]` |
| 切片超出范围 | `close[-100:-50]` | 返回填充 null 的数组 |
| 空切片 | `close[-1:-5]` | 返回空数组 `[]` |

### 特殊索引值

| 索引 | 含义 | 示例 |
|------|------|------|
| `[0]` | 当前值 | `close[0]` 等同于 `close` |
| `[-1]` | 前1个值 | 最常用的历史引用 |
| `[:]` | 全部历史+当前 | 从第1行到当前行 |
| `[-n:]` | 最近n+1个值 | 包含当前的滑动窗口 |

### 类型约束

- **只支持一维索引**：不支持多维数组访问（如 `arr[1][2]`）
- **索引必须为整数**：不支持浮点数索引（如 `close[1.5]` 报错）
- **不支持步长**：不实现 `close[-10::2]` 这样的步长切片

## 迁移策略

### 直接替换方案

**不保留兼容性**，直接移除旧的时间序列函数：

**移除的函数**：
- `ref(name, offset)` - 移除
- `past(name, n)` - 移除
- `window(name, size)` - 移除
- `offset(name, n)` - 移除

**迁移映射**：

| 旧函数调用 | 新下标语法 | 说明 |
|-----------|-----------|------|
| `ref("close", 1)` | `close[-1]` | 前1个值 |
| `ref("close", 5)` | `close[-5]` | 前5个值 |
| `past("close", 5)` | `close[-5:-1]` | 过去5个值（不含当前） |
| `window("close", 5)` | `close[-4:]` | 窗口5个值（含当前） |
| `offset("close", 1)` | `close[-1]` | 同ref |

**实施步骤**：
1. 从 Executor builtin 函数表中移除这四个函数
2. 从 Semantic Analyzer 内置函数列表中移除
3. 更新所有示例代码和测试用例使用新语法
4. 更新文档

**影响评估**：
- 破坏性变更，所有使用旧函数的代码需要手动修改
- 代码库较小，可快速迁移
- 简化维护，避免两套API共存

## 错误处理设计

### 编译时错误

| 错误类型 | 触发条件 | 错误信息示例 |
|---------|---------|-------------|
| UndefinedVariable | 访问未定义变量 | `变量 'prices' 未定义` |
| InvalidIndexType | 索引非整数 | `索引必须为整数类型，得到 string` |
| ContextError | 非流式上下文使用 | `时间序列访问只能在数据流脚本中使用` |

### 运行时行为

| 场景 | 行为 | 理由 |
|------|------|------|
| 历史数据不足 | 返回 `null` | 符合金融数据分析习惯 |
| 负索引越界 | 返回 `null` | 宽容处理，避免中断计算 |
| 正索引访问未来 | 抛出错误 | 明确禁止未来数据访问 |

## 性能分析与优化

### 当前架构性能评估

**数据流向分析**：
```
输入CSV → input_matrix (Rc<Vec<...>>) → 执行器 → output_matrix (Vec<...>)
                ↓                                        ↓
           零拷贝共享                               每次克隆
```

**性能问题定位**：

1. **输入数据读取（✅ 优化良好）**：
   - `input_matrix` 使用 `Rc` 包装，创建执行器时零拷贝
   - `get_input_history()` 返回 `cloned()`，但单值克隆开销可接受
   - **切片场景问题**：`close[-100:]` 需要拷贝100个值 → **必须优化**

2. **输出数据读取（⚠️ 存在拷贝）**：
   - `get_output_history()` 每次都克隆值
   - 单值访问：开销可接受
   - 切片访问：重复克隆 → **需要优化**

3. **切片构造（❌ 严重性能隐患）**：
   - 当前 `past()/window()` 实现：循环调用 `get_*_history()` 并 `push` 到 `Vec`
   - 每个值都被克隆一次
   - 大窗口（如200个值）会产生显著开销

**内存布局问题**：

```rust
// 当前结构（行式存储）
input_matrix: Rc<Vec<HashMap<String, Value>>>
// 行0: {"close": 100.0, "open": 99.0, ...}
// 行1: {"close": 101.0, "open": 100.5, ...}
// ...
```

- **优点**：按行插入高效
- **缺点**：按列切片需要跨行访问，缓存不友好
- **影响**：`close[-100:]` 需要访问100个不同的 HashMap

### 零拷贝设计方案

#### 方案1：ArraySlice 类型（推荐）

**新增 Value 类型**：
```rust
pub enum Value {
    // ... 现有类型
    
    /// 数组切片（零拷贝引用）
    ArraySlice {
        /// 底层列数据的共享引用
        column_data: Rc<Vec<Value>>,
        /// 切片起始索引
        start: usize,
        /// 切片长度
        len: usize,
    },
}
```

**优点**：
- 零拷贝：仅持有 `Rc` 引用，不复制数据
- 延迟物化：只有真正需要 `Vec` 时才转换
- 透明兼容：`ArraySlice` 可以自动转换为 `Array`

**使用示例**：
```rust
// 获取切片（零拷贝）
let slice = executor.get_input_slice("close", 5, 0)?;
// 返回: Value::ArraySlice { column_data: Rc, start: current_index-5, len: 6 }

// 传递给函数时自动展开（如果需要）
let ma = sum(slice) / 6;  // sum() 可以直接处理 ArraySlice
```

#### 方案2：列式缓存（按需实现）

**DataStreamExecutor 新增字段**：
```rust
pub struct DataStreamExecutor {
    // ... 现有字段
    
    /// 列式数据缓存（按需构建）
    /// key: 列名, value: 该列所有值的数组
    column_cache: RefCell<HashMap<String, Rc<Vec<Value>>>>,
}
```

**缓存策略**：
- 第一次访问某列切片时，构建整列的 `Rc<Vec<Value>>`
- 后续访问直接返回 `ArraySlice` 引用
- 适用于同一变量多次切片的场景

**权衡**：
- ✅ 大幅提升重复切片性能
- ❌ 增加内存占用（每列一份缓存）
- 🤔 适用场景：指标计算密集型（如同时计算MA5/MA10/MA20）

### 优化实施策略

#### 阶段1：基础零拷贝（MVP必须）

**任务**：
1. 新增 `Value::ArraySlice` 类型
2. 实现 `get_input_slice(name, start_offset, end_offset)` 返回切片
3. 实现 `get_output_slice(...)` 同理
4. ArraySlice 与 Array 互操作（自动转换）

**验收标准**：
- `close[-100:]` 不产生100个值的拷贝
- Benchmark 测试证明零拷贝生效

#### 阶段2：列式缓存（可选优化）

**触发条件**：
- 性能测试发现重复切片仍有开销
- 或用户场景需要（如计算20个不同周期的MA）

**任务**：
1. 添加 `column_cache` 字段
2. 实现按需缓存逻辑
3. 在行迭代完成后清理缓存

#### 阶段3：输出矩阵优化（低优先级）

**优化点**：
- 将 `output_matrix: Vec<HashMap<String, Value>>` 
  改为 `Vec<HashMap<String, Rc<Value>>>`
- 减少 `get_output_history()` 的克隆开销

**权衡**：
- 收益有限（单值访问为主）
- 代码改动较大
- 建议延后实施

### 性能目标

| 操作 | 当前实现 | 优化后 | 预期提升 |
|------|---------|--------|----------|
| 单值访问 `close[-1]` | clone 1个值 | clone 1个값 | 0% (基准) |
| 小窗口 `close[-5:]` | clone 6个值 | Rc引用 | 5-10x |
| 大窗口 `close[-100:]` | clone 101个值 | Rc引用 | 50-100x |
| 重复切片（缓存） | 每次clone | 缓存命中 | 100x+ |

### Benchmark 设计

**测试用例**：
```rust
// 测试1: 单值访问基准
bench_single_value_access()  // close[-1] × 10000次

// 测试2: 小窗口切片
bench_small_slice()  // close[-5:] × 10000次

// 测试3: 大窗口切片
bench_large_slice()  // close[-100:] × 1000次

// 测试4: 重复切片（验证缓存）
bench_repeated_slice()  // 同一行内访问 close[-20:] × 100次
```

**性能验收标准**：
- 大窗口切片性能提升 > 10x
- 内存占用不增加（ArraySlice 仅 24字节 overhead）
- 缓存命中率 > 90%（如果实现缓存）

## 文档更新要求

### 需要更新的文档

1. **语法参考文档**（`dev_logs/2.语法参考.md`）：
   - 替换"数据引用（时间序列）"章节
   - 添加下标语法完整说明
   - 提供迁移示例

2. **快速开始文档**（`QUICKSTART.md`）：
   - 更新时间序列示例
   - 使用新语法重写代码示例

3. **README.md**：
   - 更新特性列表
   - 添加新语法示例
   - 标记旧函数为已弃用

### 示例代码更新

所有 `examples/` 目录下的示例脚本需改用新语法：
- `demo.dp`
- 集成示例中的时间序列访问代码

## 测试策略

### 单元测试覆盖

**Lexer 测试**：
- 正确识别 `[`、`]`、`:` Token
- 区分数组字面量和索引访问
- 负数索引的正确解析

**Parser 测试**：
- 单值索引解析：`close[-1]`
- 切片解析：`close[-5:]`、`close[-5:-1]`、`close[:]`
- 复杂表达式：`close[-1] + open[-1]`
- 优先级：`close[-1] * 2`、`SMA(close[-10:], 5)`

**Executor 测试**：
- 单值访问正确性（与旧 ref 函数对比）
- 切片访问正确性（与旧 past/window 对比）
- 边界情况：第1行、历史不足、空切片
- null 填充行为

### 集成测试

**端到端场景**：
1. 简单MA指标计算（对比新旧语法结果一致性）
2. 复杂技术指标（MACD、KDJ使用新语法）
3. 大数据集性能测试（1000行以上）

**回归测试**：
- 确保旧函数在过渡期仍正常工作
- 新旧语法混用场景

## 实施优先级

### 第一阶段（MVP）

1. Lexer 支持 `[`、`]`、`:` Token识别
2. Parser 实现 IndexAccess 和 SliceAccess AST节点
3. Executor 实现基本的索引和切片访问
4. 单元测试覆盖核心场景

**验收标准**：
- `close[-1]` 能正确获取前一个值
- `close[-5:]` 能正确获取窗口数组
- 基本错误处理到位

### 第二阶段（完善）

1. 优化性能（引用语义、批量获取）
2. 完善边界情况处理
3. 添加缓存机制
4. 更新文档和示例

**验收标准**：
- 性能测试达到预期
- 所有边界测试通过
- 文档完整准确

### 第三阶段（文档和示例）

1. 更新所有文档使用新语法
2. 更新所有官方示例
3. 发布版本说明

**验收标准**：
- 所有示例使用新语法
- 文档完整准确
- 用户指南清晰

## 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| 语法歧义（数组 vs 索引） | 中 | 明确解析规则，充分测试 |
| 性能回退（ArraySlice实现） | 高 | 性能基准测试，零拷贝设计 |
| 破坏性变更影响用户 | 低 | 项目早期阶段，用户少 |
| 实现复杂度（引用语义） | 中 | 分阶段实施，优先MVP |

## 成功标准

1. **功能完整性**：所有旧函数功能均可用新语法实现
2. **性能达标**：大窗口切片性能提升 > 10x，零拷贝设计生效
3. **易用性提升**：语法更简洁直观，符合Python习惯
4. **零拷贝保证**：切片操作不产生数据拷贝
5. **测试覆盖率**：核心路径单元测试覆盖率 > 90%
