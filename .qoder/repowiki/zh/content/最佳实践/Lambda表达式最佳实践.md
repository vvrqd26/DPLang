# Lambda表达式最佳实践

<cite>
**本文档引用文件**   
- [6.最佳实践.md](file://dev_logs/6.最佳实践.md)
- [2.语法参考.md](file://dev_logs/2.语法参考.md)
- [LANGUAGE_GUIDE.md](file://docs/LANGUAGE_GUIDE.md)
- [ast.rs](file://src/parser/ast.rs)
- [mod.rs](file://src/parser/mod.rs)
- [expression.rs](file://src/executor/expression.rs)
- [builtin.rs](file://src/executor/builtin.rs)
</cite>

## 目录
1. [简介](#简介)
2. [单表达式Lambda的简洁写法](#单表达式lambda的简洁写法)
3. [复杂逻辑使用描述性参数名](#复杂逻辑使用描述性参数名)
4. [外部变量引用规则](#外部变量引用规则)
5. [管道链式调用](#管道链式调用)
6. [避免复杂嵌套三元表达式](#避免复杂嵌套三元表达式)
7. [总结](#总结)

## 简介
Lambda表达式是DPLang中实现函数式编程的核心特性，广泛应用于`map`、`filter`、`reduce`等高阶函数中。本文档基于`dev_logs/6.最佳实践.md`中的指导原则，结合语言规范和实现细节，为开发者提供编写清晰、高效函数式代码的最佳实践指南。

**Section sources**
- [6.最佳实践.md](file://dev_logs/6.最佳实践.md#L177-L268)
- [2.语法参考.md](file://dev_logs/2.语法参考.md#L267-L325)

## 单表达式Lambda的简洁写法
Lambda表达式在DPLang中仅支持**单个表达式**，这使得代码简洁且易于理解。对于简单的转换或计算，应使用简洁的单字母参数名。

```dplang
# ✅ 好：简洁清晰
doubled = map(array, x -> x * 2)
filtered = filter(prices, p -> p > 100)
sum = reduce(values, (a, b) -> a + b)
```

这种写法直接表达了操作的本质，避免了冗余的代码块，符合函数式编程的简洁性原则。

**Section sources**
- [6.最佳实践.md](file://dev_logs/6.最佳实践.md#L180-L187)
- [2.语法参考.md](file://dev_logs/2.语法参考.md#L274-L293)

## 复杂逻辑使用描述性参数名
当Lambda表达式中的逻辑较为复杂时，应使用描述性的参数名来提高代码的可读性。这有助于其他开发者（或未来的自己）快速理解代码的意图。

```dplang
# ✅ 好：复杂逻辑用描述性参数名
高收益股票 = filter(
    stocks,
    stock -> stock.收益率 > 0.1 and stock.风险度 < 0.5
)

# ❌ 差：复杂逻辑用单字母
高收益股票 = filter(
    stocks,
    s -> s.收益率 > 0.1 and s.风险度 < 0.5  # 不够清晰
)
```

如上所示，使用`stock`比`x`或`s`更能清晰地表达参数的含义，尤其是在涉及多个属性的复杂条件判断中。

**Section sources**
- [6.最佳实践.md](file://dev_logs/6.最佳实践.md#L189-L203)
- [6.最佳实践.md](file://dev_logs/6.最佳实践.md#L77-L94)

## 外部变量引用规则
Lambda表达式可以读取其词法作用域内的外部变量，但**禁止修改**这些变量。这是为了保持函数的纯度，避免产生副作用。

```dplang
# ✅ 好：读取外部变量
阈值 = 100
倍数 = 2

result = prices
    |> filter(p -> p > 阈值)
    |> map(p -> p * 倍数)

# ❌ 错：试图修改外部变量
计数 = 0
map(array, x -> 计数 = 计数 + 1)  # 编译错误
```

在底层实现中，Lambda表达式在创建时会捕获其作用域内的变量（`captures`字段），但这些捕获的变量在执行时是只读的。任何试图修改它们的操作都会在编译时被拒绝。

```rust
// Lambda 表达式结构定义
Lambda {
    params: Vec<String>,
    body: Box<Expr>,
    captures: std::collections::HashMap<String, Box<Value>>,
}
```

**Section sources**
- [6.最佳实践.md](file://dev_logs/6.最佳实践.md#L205-L219)
- [2.语法参考.md](file://dev_logs/2.语法参考.md#L294-L309)
- [ast.rs](file://src/parser/ast.rs#L75-L78)
- [expression.rs](file://src/executor/expression.rs#L216-L229)

## 管道链式调用
推荐使用管道操作符 `|>` 来构建清晰的数据处理流程。管道操作符将前一个表达式的结果作为第一个参数传递给下一个函数，从而形成一条从左到右的数据流。

```dplang
# ✅ 好：清晰的数据处理流程
成本 = 100
高收益阈值 = 50

result = prices
    |> filter(p -> p > 成本)          # 筛选
    |> map(p -> p - 成本)             # 转换
    |> filter(profit -> profit > 高收益阈值)  # 再筛选
    |> reduce((sum, p) -> sum + p)    # 聚合

# ❌ 差：嵌套调用难读
result = reduce(
    filter(
        map(
            filter(prices, p -> p > 成本),
            p -> p - 成本
        ),
        profit -> profit > 高收益阈值
    ),
    (sum, p) -> sum + p
)
```

管道操作符不仅提高了代码的可读性，还优化了执行顺序。例如，先进行`filter`可以减少后续`map`和`reduce`操作的数据量，从而提升性能。

**Section sources**
- [6.最佳实践.md](file://dev_logs/6.最佳实践.md#L221-L244)
- [LANGUAGE_GUIDE.md](file://docs/LANGUAGE_GUIDE.md#L273-L286)
- [mod.rs](file://src/parser/mod.rs#L463-L478)
- [builtin.rs](file://src/executor/builtin.rs#L231-L245)

## 避免复杂嵌套三元表达式
应避免在Lambda表达式中使用过于复杂的嵌套三元表达式。这类表达式难以阅读和维护，容易出错。相反，应将复杂的逻辑封装为包函数。

```dplang
# ❌ 差：逻辑太复杂
complex = map(
    array,
    x -> x > 10 ? (x < 20 ? (x % 2 == 0 ? x * 2 : x * 3) : x * 4) : x * 0.5
)

# ✅ 好：复杂逻辑用包函数
package 工具包

计算倍数(x:number) -> number:
    if x > 10:
        if x < 20:
            return x % 2 == 0 ? x * 2 : x * 3
        return x * 4
    return x * 0.5

# 数据脚本中使用
result = map(array, x -> 工具包.计算倍数(x))
```

通过将复杂逻辑封装到包函数中，代码变得更加模块化和可重用。同时，包函数可以拥有完整的函数体，支持`if-else`语句和多行逻辑，这比单表达式的Lambda更加灵活。

**Section sources**
- [6.最佳实践.md](file://dev_logs/6.最佳实践.md#L247-L268)
- [2.语法参考.md](file://dev_logs/2.语法参考.md#L301-L305)

## 总结
遵循这些最佳实践，可以编写出既简洁又高效的DPLang代码：
1.  **简洁性**：对简单操作使用单字母参数的单表达式Lambda。
2.  **可读性**：对复杂逻辑使用描述性参数名。
3.  **纯函数**：只读取外部变量，绝不修改，保证函数的纯度。
4.  **清晰流程**：利用管道操作符构建从左到右、易于理解的数据处理链。
5.  **模块化**：将复杂逻辑封装为包函数，避免在Lambda中编写难以理解的嵌套三元表达式。

通过结合`dev_logs/6.最佳实践.md`中的正反例和语言实现细节，开发者可以充分利用Lambda表达式和函数式编程的优势，编写出高质量的代码。