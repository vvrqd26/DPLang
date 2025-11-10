# 项目结构优化设计文档

## 一、优化目标

### 1.1 核心目标
- 清理项目仓库中的临时文件和测试数据
- 建立清晰、规范的目录结构
- 优化 Git 忽略规则，避免不必要的文件提交
- 清理并重新初始化远程仓库，移除历史中的大文件

### 1.2 预期收益
- 减少仓库体积，提升克隆和拉取速度
- 提高项目可维护性和可读性
- 规范化开发流程，避免测试文件误提交
- 改善协作体验

## 二、当前项目结构分析

### 2.1 现有目录结构
```
DPLang/
├── .git/                      # Git 版本控制
├── .gitignore                 # Git 忽略规则 (当前仅忽略 /target 和 /dev_logs)
├── src/                       # 核心源代码 ✓
│   ├── executor/              # 执行器模块
│   ├── orchestration/         # 任务编排系统
│   ├── parser/                # 语法分析器
│   ├── streaming/             # 流式处理
│   └── *.rs                   # 其他核心模块
├── benches/                   # 性能基准测试 ✓
├── examples/                  # 示例脚本 (当前几乎为空)
│   └── test_datas/            # ⚠️ 空目录
├── test_data/                 # ⚠️ 测试数据目录 (包含生成的CSV等)
│   ├── stock_data_100.csv     # ⚠️ 323.4KB 测试数据
│   ├── output_results.csv     # ⚠️ 生成的结果文件
│   └── *.py, *.ps1            # 测试脚本
├── output/                    # ⚠️ 运行输出目录
│   ├── output_000001.csv      # ⚠️ 生成的输出文件
│   └── output_000002.csv
├── packages/                  # DPLang 包目录 ✓
│   └── math.dp
├── __pycache__/               # ⚠️ Python 缓存
├── *.py                       # ⚠️ 根目录下的测试脚本
├── *.ps1                      # ⚠️ PowerShell 测试脚本
├── tasks.toml                 # ⚠️ 任务配置 (应该在示例或测试目录)
├── Cargo.toml                 # ✓ Rust 项目配置
├── Cargo.lock                 # ✓ 依赖锁定文件
└── *.md                       # ✓ 文档文件
```

### 2.2 识别的问题

#### 问题 1: 测试数据占用空间
- `test_data/stock_data_100.csv` (323.4KB) - 代码生成的随机测试数据
- `test_data/output_results.csv` - 测试输出结果
- `output/` 目录下的 CSV 文件 - 运行时生成的输出
- 这些文件应该被 Git 忽略，不应提交到仓库

#### 问题 2: 临时文件未被忽略
- `__pycache__/` - Python 字节码缓存
- `output/` - 运行时输出目录
- 各种生成的 `.csv` 文件

#### 问题 3: 目录结构不清晰
- 根目录下有多个测试脚本 (`*.py`, `*.ps1`)
- `examples/test_datas/` 空目录存在
- `tasks.toml` 应该归类到更合适的位置

#### 问题 4: 缺少标准目录
- 缺少 `docs/` 用于存放详细文档
- 缺少 `scripts/` 用于存放构建和测试脚本
- 缺少 `tests/` 用于存放集成测试

## 三、优化方案设计

### 3.1 目标目录结构

```
DPLang/
├── .git/
├── .gitignore              # 更新后的忽略规则
├── .qoder/                 # Qoder 工具配置
├── src/                    # 源代码 (保持不变)
│   ├── executor/
│   ├── orchestration/
│   ├── parser/
│   ├── streaming/
│   └── *.rs
├── benches/                # 性能基准测试
├── tests/                  # 集成测试 (新建)
├── examples/               # 示例脚本和集成示例 (重新组织)
│   ├── scripts/            # DPLang 示例脚本
│   │   ├── hello.dp
│   │   ├── moving_average.dp
│   │   └── technical_analysis.dp
│   └── integrations/       # 集成示例
│       ├── rust_demo.rs
│       └── python_demo.py
├── packages/               # DPLang 标准包库
│   └── math.dp
├── scripts/                # 构建、测试、部署脚本 (新建)
│   ├── generate_test_data.py
│   ├── performance_test.py
│   ├── extract_result.py
│   └── test_orchestrate.ps1
├── docs/                   # 详细文档 (新建)
│   ├── design/             # 设计文档
│   ├── guides/             # 使用指南
│   └── references/         # 参考文档
├── test_data/              # 测试数据 (Git 忽略)
│   └── .gitkeep            # 保留目录但不提交内容
├── output/                 # 运行输出 (Git 忽略)
│   └── .gitkeep
├── Cargo.toml
├── Cargo.lock
├── README.md
├── QUICKSTART.md
├── DAEMON_MODE_GUIDE.md
├── ORCHESTRATION_QUICKSTART.md
└── ANGETS.md
```

### 3.2 .gitignore 更新规则

需要添加以下忽略规则:

```
# Rust 构建产物
/target

# 开发日志
/dev_logs

# 测试数据和输出
/test_data/*.csv
/test_data/*.json
/output/
*.csv
!examples/**/*.csv

# Python 临时文件
__pycache__/
*.pyc
*.pyo
*.pyd
.Python

# 编辑器和 IDE
.vscode/
.idea/
*.swp
*.swo
*~
.DS_Store

# 操作系统
Thumbs.db
desktop.ini

# 临时文件
*.tmp
*.log
*.bak

# 本地配置
.env
.env.local
```

### 3.3 文件迁移计划

#### 迁移操作清单

| 当前位置 | 目标位置 | 操作类型 | 说明 |
|---------|---------|---------|------|
| `generate_test_data.py` | `scripts/generate_test_data.py` | 移动 | 测试数据生成脚本 |
| `performance_test.py` | `scripts/performance_test.py` | 移动 | 性能测试脚本 |
| `extract_result.py` | `scripts/extract_result.py` | 移动 | 结果提取脚本 |
| `test_orchestrate.ps1` | `scripts/test_orchestrate.ps1` | 移动 | 编排测试脚本 |
| `tasks.toml` | `examples/configs/tasks.toml` | 移动 | 任务配置示例 |
| `test_data/stock_tasks.toml` | `examples/configs/stock_tasks.toml` | 移动 | 股票任务配置示例 |
| `test_data/*.py` | `scripts/` | 移动 | 测试相关脚本 |
| `test_data/*.ps1` | `scripts/` | 移动 | 测试相关脚本 |
| `test_data/TEST_SUMMARY.md` | `docs/testing/` | 移动 | 测试总结文档 |
| `test_data/indicators.dp` | `examples/scripts/` | 移动 | 示例脚本 |
| `examples/test_datas/` | - | 删除 | 空目录 |
| `__pycache__/` | - | 删除 | Python 缓存 |
| `output/*.csv` | - | 删除 | 临时输出文件 |
| `test_data/*.csv` | - | 删除 | 生成的测试数据 |

### 3.4 新建目录说明

| 目录路径 | 用途 | 说明 |
|---------|------|------|
| `scripts/` | 开发工具脚本 | 存放构建、测试、数据生成等脚本 |
| `docs/` | 项目文档 | 存放设计文档、指南、参考文档 |
| `docs/design/` | 设计文档 | 架构设计、模块设计等 |
| `docs/guides/` | 使用指南 | 快速开始、最佳实践等 |
| `docs/references/` | 参考文档 | API 参考、语法参考等 |
| `docs/testing/` | 测试文档 | 测试策略、测试结果等 |
| `tests/` | 集成测试 | 端到端测试、集成测试用例 |
| `examples/scripts/` | DPLang 示例脚本 | 演示语言特性的脚本 |
| `examples/configs/` | 配置示例 | 任务配置、编排配置等示例 |
| `examples/integrations/` | 集成示例 | Rust、Python 等集成示例代码 |

### 3.5 文档迁移计划

将根目录和 dev_logs 中的文档重新组织:

| 当前位置 | 目标位置 | 说明 |
|---------|---------|------|
| `README.md` | `README.md` | 保持不变，项目主文档 |
| `QUICKSTART.md` | `docs/guides/QUICKSTART.md` | 快速开始指南 |
| `DAEMON_MODE_GUIDE.md` | `docs/guides/DAEMON_MODE_GUIDE.md` | 守护进程模式指南 |
| `ORCHESTRATION_QUICKSTART.md` | `docs/guides/ORCHESTRATION_QUICKSTART.md` | 编排系统快速开始 |
| `ANGETS.md` | `docs/AGENTS.md` 或删除 | 根据内容决定 |
| `dev_logs/1.核心设计.md` | `docs/design/core-design.md` | 核心设计文档 |
| `dev_logs/2.语法参考.md` | `docs/references/syntax.md` | 语法参考 |
| `dev_logs/4.内置函数参考.md` | `docs/references/builtin-functions.md` | 内置函数参考 |
| `dev_logs/5.完整示例.md` | `docs/guides/examples.md` | 完整示例 |
| `dev_logs/7.解释器实现设计.md` | `docs/design/interpreter-implementation.md` | 解释器实现设计 |

## 四、远程仓库清理方案

### 4.1 清理策略

由于远程仓库包含大量测试数据历史记录，采用以下策略:

#### 方案一: 完全重建仓库 (推荐)
**适用场景**: 项目处于早期阶段，历史提交不重要

**优点**:
- 彻底清理历史，仓库体积最小
- 操作简单，风险低
- 适合当前项目状态

**缺点**:
- 丢失所有历史提交记录

**实施步骤**:
1. 备份当前代码到本地
2. 删除 `.git` 目录
3. 重新初始化 Git 仓库
4. 添加优化后的 `.gitignore`
5. 提交清理后的代码
6. 强制推送到远程仓库

#### 方案二: 使用 Git Filter 清理历史
**适用场景**: 需要保留提交历史

**优点**:
- 保留提交历史和作者信息
- 仅移除特定文件的历史记录

**缺点**:
- 操作复杂，需要谨慎处理
- 所有协作者需要重新克隆仓库
- 耗时较长

**实施步骤**:
1. 使用 `git filter-repo` 或 `BFG Repo-Cleaner` 工具
2. 删除历史中的大文件和测试数据
3. 强制推送到远程仓库
4. 通知协作者重新克隆

### 4.2 推荐方案: 完全重建

考虑到项目是个人学习项目，且主要关注当前功能实现，推荐采用方案一。

### 4.3 操作前检查清单

在执行远程仓库清理前，需要确认:

- [ ] 本地代码已备份到安全位置
- [ ] 重要的提交信息已记录 (如果需要保留)
- [ ] 已更新 `.gitignore` 文件
- [ ] 已完成文件清理和目录重组
- [ ] 已测试项目编译和运行正常
- [ ] 已通知协作者 (如有)

## 五、实施步骤

### 5.1 第一阶段: 本地清理和重组

#### 步骤 1: 创建备份
- 将整个项目目录复制到安全位置
- 确认备份完整性

#### 步骤 2: 创建新目录
- 创建 `scripts/` 目录
- 创建 `docs/` 及其子目录
- 创建 `tests/` 目录
- 创建 `examples/scripts/` 和 `examples/configs/` 等子目录

#### 步骤 3: 迁移文件
- 按照迁移计划移动文件到新位置
- 删除临时文件和生成文件
- 清理空目录

#### 步骤 4: 更新 .gitignore
- 添加新的忽略规则
- 确保测试数据和输出目录被忽略

#### 步骤 5: 更新文档引用
- 更新 README.md 中的文件路径引用
- 更新代码中的文件路径 (如脚本中的导入路径)
- 检查所有文档的内部链接

#### 步骤 6: 添加 .gitkeep 文件
- 在 `test_data/` 和 `output/` 目录添加 `.gitkeep`
- 保留目录结构但不提交内容

### 5.2 第二阶段: 验证和测试

#### 步骤 7: 编译验证
- 运行 `cargo build` 确保编译通过
- 运行 `cargo test` 确保所有测试通过
- 运行 `cargo bench` 验证基准测试

#### 步骤 8: 功能验证
- 测试示例脚本是否正常运行
- 验证文档链接是否正确
- 检查工具脚本是否正常工作

#### 步骤 9: Git 状态检查
- 运行 `git status` 查看未跟踪文件
- 确认不应提交的文件已被忽略
- 确认应提交的文件都已包含

### 5.3 第三阶段: 远程仓库重建

#### 步骤 10: 删除旧的 Git 历史
- 删除 `.git` 目录
- 重新初始化: `git init`

#### 步骤 11: 首次提交
- 添加所有文件: `git add .`
- 创建初始提交: `git commit -m "Initial commit: Clean project structure"`

#### 步骤 12: 推送到远程
- 添加远程仓库地址
- 强制推送: `git push -f origin main` (或 `master`)

#### 步骤 13: 验证远程仓库
- 在新位置克隆仓库
- 验证文件结构正确
- 验证忽略规则生效

### 5.4 第四阶段: 文档更新

#### 步骤 14: 更新 README.md
- 更新项目结构说明
- 更新文档链接
- 添加目录结构说明

#### 步骤 15: 创建迁移说明文档
- 记录目录结构变化
- 说明文件迁移映射
- 提供迁移后的使用指南

## 六、风险评估与应对

### 6.1 潜在风险

| 风险 | 影响 | 概率 | 应对措施 |
|------|------|------|---------|
| 文件迁移后路径引用错误 | 高 | 中 | 全面测试，检查所有硬编码路径 |
| 误删重要文件 | 高 | 低 | 事先完整备份，逐步验证 |
| Git 忽略规则遗漏 | 中 | 中 | 参考标准模板，多次检查 |
| 远程仓库推送失败 | 中 | 低 | 提前测试推送权限，准备备选方案 |
| 协作者工作中断 | 低 | 低 | 单人项目，无影响 |

### 6.2 回滚方案

如果在任何阶段出现问题:
- **本地清理阶段**: 从备份恢复
- **远程推送阶段**: 使用备份的 `.git` 目录恢复
- **验证失败**: 暂停推送，修复问题后重新执行

## 七、后续维护建议

### 7.1 持续规范

- 建立 `.editorconfig` 统一编码风格
- 添加 CI/CD 配置自动检查文件大小
- 定期审查 Git 历史，清理不必要的文件
- 使用 Git Hooks 防止大文件提交

### 7.2 文档维护

- 保持 README.md 与实际结构同步
- 在添加新目录时更新结构文档
- 为新脚本和工具添加使用说明

### 7.3 开发流程优化

- 测试数据统一使用 `test_data/` 目录
- 运行输出统一使用 `output/` 目录
- 新增脚本放入 `scripts/` 目录
- 新增文档放入 `docs/` 相应子目录

## 八、验收标准

### 8.1 结构验收

- [ ] 所有源代码文件在 `src/` 目录
- [ ] 测试脚本在 `scripts/` 目录
- [ ] 文档在 `docs/` 相关子目录
- [ ] 示例在 `examples/` 相关子目录
- [ ] 根目录仅包含配置文件和主要文档

### 8.2 功能验收

- [ ] `cargo build` 编译成功
- [ ] `cargo test` 所有测试通过
- [ ] 示例脚本运行正常
- [ ] 文档链接全部有效

### 8.3 Git 验收

- [ ] 生成的测试数据未被跟踪
- [ ] 输出文件未被跟踪
- [ ] Python 缓存未被跟踪
- [ ] 仓库体积显著减小 (预期 < 5MB)
- [ ] Git 历史清晰，无冗余提交

### 8.4 文档验收

- [ ] README.md 更新完整
- [ ] 目录结构说明准确
- [ ] 文档链接全部可用
- [ ] 迁移说明文档已创建

## 九、执行时间估算

| 阶段 | 预估时间 | 说明 |
|------|---------|------|
| 本地清理和重组 | 30-45 分钟 | 创建目录、迁移文件、更新配置 |
| 验证和测试 | 15-20 分钟 | 编译、测试、功能验证 |
| 远程仓库重建 | 10-15 分钟 | Git 操作和推送 |
| 文档更新 | 20-30 分钟 | 更新文档、检查链接 |
| **总计** | **75-110 分钟** | 约 1.5-2 小时 |

## 十、关键决策点

在实施过程中需要用户确认的关键决策:

### 决策 1: ANGETS.md 文件处理
**问题**: 该文件名称疑似拼写错误 (AGENTS?)，内容未知

**选项**:
- a) 移动到 `docs/` 目录并重命名为 AGENTS.md
- b) 如果内容不重要则删除
- c) 保持在根目录

### 决策 2: dev_logs 目录处理
**问题**: 当前被 `.gitignore` 忽略，但包含重要设计文档

**选项**:
- a) 将文档迁移到 `docs/` 后删除 dev_logs
- b) 从 `.gitignore` 移除，将 dev_logs 纳入版本控制
- c) 保持忽略，文档仅存在本地

### 决策 3: 远程仓库清理方式
**问题**: 选择重建还是过滤历史

**选项**:
- a) 完全重建 (推荐) - 简单快速，丢失历史
- b) 过滤历史 - 保留历史，但操作复杂

### 决策 4: 文档迁移范围
**问题**: 是否将根目录的指南文档移入 docs/

**选项**:
- a) 仅保留 README.md 在根目录，其他移入 docs/guides/
- b) 常用指南保留根目录，详细文档移入 docs/
- c) 保持现状，不移动根目录文档
