# format_output_csv 函数

<cite>
**Referenced Files in This Document**  
- [api.rs](file://src/api.rs#L243-L282)
- [api.rs](file://src/api.rs#L284-L313)
- [runtime.rs](file://src/runtime.rs#L6-L33)
</cite>

## Table of Contents
1. [函数概述](#函数概述)
2. [表头生成机制](#表头生成机制)
3. [数据行输出流程](#数据行输出流程)
4. [值格式化规则](#值格式化规则)
5. [输入输出示例](#输入输出示例)
6. [兼容性与限制](#兼容性与限制)

## 函数概述

`format_output_csv` 函数是 DPLang 项目中用于将执行结果 `Vec<HashMap<String, Value>>` 反向格式化为标准 CSV 字符串的核心函数。该函数位于 `src/api.rs` 文件中，主要负责将程序内部的数据结构转换为可读的 CSV 格式输出。

当输入为空时，函数直接返回空字符串。对于非空输入，函数首先收集所有行中的列名，然后按字典序排序生成表头，最后逐行输出数据。该函数通过调用 `format_value_csv` 辅助函数来处理单个值的格式化，确保特殊字符得到正确转义。

**Section sources**
- [api.rs](file://src/api.rs#L243-L282)

## 表头生成机制

`format_output_csv` 函数通过遍历所有数据行来收集完整的列名集合。算法首先创建一个空的 `headers` 向量，然后对每一行数据中的键（即列名）进行迭代，若该列名尚未存在于 `headers` 中，则将其添加进去。这种去重机制确保了即使不同行包含不同的列，最终也能生成包含所有列的完整表头。

在收集完所有列名后，函数调用 `sort()` 方法对 `headers` 向量进行字典序排序。这一排序步骤保证了输出的 CSV 文件具有稳定的列顺序，无论输入数据的原始顺序如何。排序后的列名通过 `join(",")` 方法连接成以逗号分隔的字符串，并以换行符结尾，形成最终的 CSV 表头。

**Section sources**
- [api.rs](file://src/api.rs#L250-L260)

## 数据行输出流程

在生成表头后，`format_output_csv` 函数开始处理数据行。对于每一行数据，函数根据已排序的 `headers` 列表，按顺序查找对应列的值。如果某列在当前行中不存在，则使用空字符串作为默认值。

具体实现中，函数使用 `map` 迭代器对 `headers` 中的每个列名进行处理，通过 `row.get(h)` 获取对应值，并调用 `format_value_csv` 函数将其转换为适当的 CSV 格式字符串。所有列的格式化值被收集到一个 `values` 向量中，然后通过 `join(",")` 方法连接成一行 CSV 数据，最后添加换行符并追加到结果字符串中。

**Section sources**
- [api.rs](file://src/api.rs#L267-L280)

## 值格式化规则

`format_value_csv` 辅助函数定义了各种 `Value` 类型到 CSV 字符串的转换规则：

- **空值**：`Value::Null` 转换为空字符串
- **布尔值**：`Value::Bool` 转换为 "true" 或 "false"
- **数字**：`Value::Number` 和 `Value::Decimal` 直接转换为字符串表示
- **字符串**：若包含逗号或引号，则用双引号包裹，并将内部的引号转义为两个连续的引号
- **数组和数组切片**：元素序列化后用分号分隔，并包裹在方括号中，整体用双引号包裹
- **函数和 Lambda**：分别转换为 "<function>" 和 "<lambda>" 字符串

这些规则确保了特殊字符在 CSV 格式中的正确表示，避免了数据解析时的歧义。

**Section sources**
- [api.rs](file://src/api.rs#L284-L313)
- [runtime.rs](file://src/runtime.rs#L6-L33)

## 输入输出示例

考虑以下输入数据：
```rust
let mut row1 = HashMap::new();
row1.insert("name".to_string(), Value::String("Alice".to_string()));
row1.insert("score".to_string(), Value::Number(95.5));
row1.insert("comment".to_string(), Value::String("Good, \"excellent\"".to_string()));

let output = vec![row1];
```

`format_output_csv` 函数将生成以下 CSV 输出：
```
comment,name,score
"Good, ""excellent""",Alice,95.5
```

在此示例中，表头按字典序排序为 "comment,name,score"。数据行中，包含逗号和引号的字符串 "Good, \"excellent\"" 被正确转义为 `"Good, ""excellent"""`，确保了 CSV 格式的兼容性。

**Section sources**
- [api.rs](file://src/api.rs#L329-L340)

## 兼容性与限制

`format_output_csv` 函数生成的 CSV 格式与标准 CSV 规范兼容，能够被大多数 CSV 解析器正确读取。其转义规则遵循常见的 CSV 实现，特别是对包含特殊字符的字符串的处理方式。

然而，该函数在处理复杂数据类型时存在一定的表示限制。数组和数组切片被序列化为用分号分隔的字符串，这种表示方式虽然保留了元素信息，但失去了原始的数组结构。此外，函数和 Lambda 类型被简化为占位符字符串，无法在 CSV 中恢复其原始功能。这些限制是扁平化数据格式固有的，用户在使用时应予以注意。

**Section sources**
- [api.rs](file://src/api.rs#L284-L313)
- [runtime.rs](file://src/runtime.rs#L6-L33)