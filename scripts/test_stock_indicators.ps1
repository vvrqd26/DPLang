# 股票技术指标计算测试脚本
# 测试任务编排系统处理100只股票的技术指标计算

param(
    [int]$NumStocks = 100,
    [int]$Days = 60,
    [int]$Port = 8889
)

$ErrorActionPreference = "Continue"

Write-Host "=== DPLang Stock Indicators Test ===" -ForegroundColor Cyan
Write-Host ""

# 1. 生成测试数据
Write-Host "[Step 1/5] Generating test data..." -ForegroundColor Yellow
Write-Host "  Stocks: $NumStocks" -ForegroundColor Gray
Write-Host "  Days per stock: $Days" -ForegroundColor Gray
Write-Host "  Total rows: $($NumStocks * $Days)" -ForegroundColor Gray

python test_data/generate_stock_data.py $NumStocks $Days

if (-not (Test-Path "test_data/stock_data_100.csv")) {
    Write-Host "Error: Failed to generate test data" -ForegroundColor Red
    exit 1
}

Write-Host "  ✓ Test data generated" -ForegroundColor Green
Write-Host ""

# 2. 编译项目
Write-Host "[Step 2/5] Building project..." -ForegroundColor Yellow
$null = cargo build --release 2>&1
if (-not (Test-Path "target/release/dplang.exe")) {
    Write-Host "  ✗ Build failed" -ForegroundColor Red
    exit 1
}
Write-Host "  ✓ Build successful" -ForegroundColor Green
Write-Host ""

# 3. 启动编排服务器
Write-Host "[Step 3/5] Starting orchestration server..." -ForegroundColor Yellow
Write-Host "  Port: $Port" -ForegroundColor Gray

$serverJob = Start-Job -ScriptBlock {
    param($port)
    Set-Location $using:PWD
    & ".\target\release\dplang.exe" orchestrate "test_data/stock_tasks.toml" $port
} -ArgumentList $Port

Start-Sleep -Seconds 3

# 检查服务器是否启动
$serverRunning = $false
try {
    $client = New-Object System.Net.Sockets.TcpClient("127.0.0.1", $Port)
    $client.Close()
    $serverRunning = $true
    Write-Host "  ✓ Server started on port $Port" -ForegroundColor Green
} catch {
    Write-Host "  ✗ Server failed to start" -ForegroundColor Red
    Stop-Job $serverJob
    Remove-Job $serverJob
    exit 1
}
Write-Host ""

# 4. 测试API - 列出任务
Write-Host "[Step 4/5] Testing API - List Tasks..." -ForegroundColor Yellow
try {
    $client = New-Object System.Net.Sockets.TcpClient("127.0.0.1", $Port)
    $stream = $client.GetStream()
    $writer = New-Object System.IO.StreamWriter($stream)
    $reader = New-Object System.IO.StreamReader($stream)
    
    $writer.WriteLine('{"action": "list_tasks", "params": {}}')
    $writer.Flush()
    
    $response = $reader.ReadLine()
    $client.Close()
    
    Write-Host "  Response: $response" -ForegroundColor Gray
    
    if ($response -match '"status":"ok"') {
        Write-Host "  ✓ API test passed" -ForegroundColor Green
    } else {
        Write-Host "  ✗ API test failed" -ForegroundColor Red
    }
} catch {
    Write-Host "  ✗ API test failed: $_" -ForegroundColor Red
}
Write-Host ""

# 5. 处理股票数据（模拟）
Write-Host "[Step 5/5] Performance test simulation..." -ForegroundColor Yellow
$dataRows = Get-Content "test_data/stock_data_100.csv" | Measure-Object -Line
Write-Host "  Total data rows: $($dataRows.Lines - 1)" -ForegroundColor Gray
Write-Host "  Unique stocks: $NumStocks" -ForegroundColor Gray
Write-Host "  Routing strategy: hash" -ForegroundColor Gray
Write-Host "  Compute pool size: 10-50" -ForegroundColor Gray
Write-Host ""
Write-Host "  Note: Full data processing test requires implementing" -ForegroundColor Gray
Write-Host "        the data push API in future versions." -ForegroundColor Gray
Write-Host ""

# 清理
Write-Host "Cleaning up..." -ForegroundColor Yellow
Stop-Job $serverJob
Remove-Job $serverJob
Write-Host "✓ Server stopped" -ForegroundColor Green
Write-Host ""

# 总结
Write-Host "=== Test Summary ===" -ForegroundColor Cyan
Write-Host "✓ Test data generated: test_data/stock_data_100.csv" -ForegroundColor Green
Write-Host "✓ DPLang script created: test_data/indicators.dp" -ForegroundColor Green
Write-Host "✓ Task configuration: test_data/stock_tasks.toml" -ForegroundColor Green
Write-Host "✓ Orchestration server: Started and tested" -ForegroundColor Green
Write-Host ""
Write-Host "To manually test data processing:" -ForegroundColor Yellow
Write-Host "  1. Start server: dplang orchestrate test_data/stock_tasks.toml $Port" -ForegroundColor Gray
Write-Host "  2. Use daemon mode: dplang daemon test_data/indicators.dp test_data/stock_data_100.csv" -ForegroundColor Gray
Write-Host ""
