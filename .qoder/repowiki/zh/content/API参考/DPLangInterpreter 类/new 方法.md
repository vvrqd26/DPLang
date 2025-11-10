# new 方法

<cite>
**本文档引用的文件**   
- [api.rs](file://src/api.rs)
- [lib.rs](file://src/lib.rs)
</cite>

## 目录
1. [简介](#简介)
2. [方法定义与参数说明](#方法定义与参数说明)
3. [生命周期管理与内部存储设计](#生命周期管理与内部存储设计)
4. [调用示例](#调用示例)
5. [作为API入口的安全性保证](#作为api入口的安全性保证)
6. [实际集成使用模式](#实际集成使用模式)
7. [结论](#结论)

## 简介
`new` 方法是 DPLang 解释器的核心构造函数，用于从脚本源码字符串创建解释器实例。该方法定义在 `DPLangInterpreter` 结构体中，是用户与 DPLang 脚本交互的首要入口点。通过此方法，开发者可以将任意 DPLang 脚本代码加载到内存中，并准备执行。该方法的设计体现了 Rust 的所有权和生命周期管理原则，确保了内存安全和高效的数据处理。

**Section sources**
- [api.rs](file://src/api.rs#L16-L20)

## 方法定义与参数说明
`new` 方法接收一个不可变的字符串切片 `&str` 作为参数，表示 DPLang 脚本的源代码。该方法的签名如下：

```rust
pub fn new(source: &str) -> Self
```

参数 `source` 是一个对字符串的不可变引用，允许方法在不获取所有权的情况下读取脚本内容。这是 Rust 中处理字符串输入的常见模式，既保证了调用者的灵活性（可以传入 `String`、`&str` 或字面量），又避免了不必要的数据拷贝。该方法返回一个 `DPLangInterpreter` 实例，该实例内部持有脚本源码的副本。

**Section sources**
- [api.rs](file://src/api.rs#L16-L20)

## 生命周期管理与内部存储设计
`new` 方法通过调用 `source.to_string()` 将传入的 `&str` 转换为一个拥有所有权的 `String` 类型，并将其存储在 `DPLangInterpreter` 结构体的 `source` 字段中。这种设计确保了脚本源码的生命周期与解释器实例的生命周期完全绑定。一旦解释器实例被销毁，其内部存储的源码也会被自动释放。

```rust
pub struct DPLangInterpreter {
    source: String,
}
```

将源码存储在结构体内部有多个设计考量：
1.  **数据持久性**：解释器实例可以在其整个生命周期内访问源码，这对于多次执行、语法分析和错误报告至关重要。
2.  **所有权清晰**：`String` 类型拥有其数据的所有权，避免了悬垂引用和生命周期管理的复杂性。
3.  **性能优化**：源码只需在初始化时拷贝一次，后续执行无需重新传入，减少了函数调用的开销。

**Section sources**
- [api.rs](file://src/api.rs#L10-L12)
- [api.rs](file://src/api.rs#L18)

## 调用示例
以下是在 Rust 应用中初始化 DPLang 解释器的典型调用示例：

```rust
let source = r#"
-- INPUT x:number --
-- OUTPUT result:number --

result = x * 2
return [result]
"#;

let interpreter = DPLangInterpreter::new(source);
```

在此示例中，一个包含 DPLang 脚本的原始字符串字面量被传递给 `new` 方法。该方法创建了一个 `DPLangInterpreter` 实例，该实例现在可以用于执行脚本。

**Section sources**
- [api.rs](file://src/api.rs#L345-L362)

## 作为API入口的安全性保证
`new` 方法本身不执行任何输入验证或错误处理，因为它仅负责创建解释器实例并存储源码。安全性保证主要体现在以下几个方面：
1.  **内存安全**：Rust 的类型系统和所有权规则确保了 `source` 字段的内存安全。`to_string()` 方法创建了一个新的、独立的 `String` 副本，与原始输入完全解耦。
2.  **无副作用**：`new` 方法是一个纯构造函数，它不会修改任何外部状态或执行脚本代码，因此是完全安全的。
3.  **错误处理延迟**：实际的输入验证（如语法分析、词法分析）和错误处理被推迟到 `execute` 方法中进行。这使得 `new` 方法非常轻量，可以快速创建实例，而复杂的错误检查则在执行时进行。

**Section sources**
- [api.rs](file://src/api.rs#L16-L20)
- [api.rs](file://src/api.rs#L30-L45)

## 实际集成使用模式
在实际集成场景中，`new` 方法通常作为工作流的起点。常见的使用模式包括：
1.  **从文件加载**：结合 `from_file` 方法，先从磁盘读取脚本，再用 `new` 方法创建解释器。
2.  **动态脚本生成**：在运行时动态生成 DPLang 脚本字符串，然后直接传递给 `new` 方法。
3.  **配置驱动**：将 DPLang 脚本作为配置的一部分存储在数据库或配置文件中，启动时读取并用 `new` 方法初始化解释器。

这种模式使得 DPLang 可以灵活地集成到各种应用中，无论是作为嵌入式脚本引擎还是作为数据处理管道的一部分。

**Section sources**
- [api.rs](file://src/api.rs#L22-L27)

## 结论
`new` 方法是 DPLang 解释器 API 的基石，它以一种安全、高效且符合 Rust 哲学的方式，实现了从脚本源码到解释器实例的转换。通过不可变引用接收源码、在内部存储为 `String`，并推迟错误处理，该方法为后续的脚本执行提供了坚实的基础。其简洁的设计和清晰的职责划分，使其成为集成 DPLang 功能的理想入口点。