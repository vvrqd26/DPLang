# DPLang 任务编排系统 - 快速开始

## 编译项目

```bash
cargo build --release
```

## 使用方式

### 1. 启动编排服务器

```bash
# 使用默认配置文件 tasks.toml 和默认端口 8888
dplang orchestrate

# 指定配置文件和端口
dplang orchestrate tasks.toml 9000
```

### 2. 使用API管理任务

通过TCP连接到服务器（默认端口8888），发送JSON格式的API请求。

#### 列出所有任务
```json
{"action": "list_tasks", "params": {}}
```

#### 启动任务
```json
{"action": "start_task", "params": {"task_id": "demo-task"}}
```

#### 暂停任务
```json
{"action": "pause_task", "params": {"task_id": "demo-task"}}
```

#### 继续任务
```json
{"action": "resume_task", "params": {"task_id": "demo-task"}}
```

#### 停止任务
```json
{"action": "stop_task", "params": {"task_id": "demo-task"}}
```

#### 删除任务
```json
{"action": "delete_task", "params": {"task_id": "demo-task"}}
```

## 示例：使用PowerShell测试

```powershell
# 连接到服务器并发送请求
$client = New-Object System.Net.Sockets.TcpClient("127.0.0.1", 8888)
$stream = $client.GetStream()
$writer = New-Object System.IO.StreamWriter($stream)
$reader = New-Object System.IO.StreamReader($stream)

# 列出任务
$writer.WriteLine('{"action": "list_tasks", "params": {}}')
$writer.Flush()
$response = $reader.ReadLine()
Write-Host $response

# 清理
$client.Close()
```

## 已实现的功能（MVP第一阶段）

✅ 配置文件加载（TOML格式）
✅ 任务创建、删除、查询
✅ 任务状态管理（启动、暂停、继续、停止）
✅ 计算元池管理（固定大小）
✅ 数据路由器（哈希、轮询、粘性会话策略）
✅ TCP API服务器
✅ 任务状态机
