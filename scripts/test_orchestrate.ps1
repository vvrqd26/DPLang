# Test orchestration server

Write-Host "=== Testing DPLang Orchestration Server ===" -ForegroundColor Cyan

# Start server in background
Write-Host "`nStarting server..." -ForegroundColor Yellow
$server = Start-Process -FilePath ".\target\release\dplang.exe" -ArgumentList "orchestrate","tasks.toml","8888" -PassThru -NoNewWindow

Start-Sleep -Seconds 2

# Test API calls
Write-Host "`nConnecting to server..." -ForegroundColor Yellow
try {
    $client = New-Object System.Net.Sockets.TcpClient("127.0.0.1", 8888)
    $stream = $client.GetStream()
    $writer = New-Object System.IO.StreamWriter($stream)
    $reader = New-Object System.IO.StreamReader($stream)
    
    # List tasks
    Write-Host "`nSending: list_tasks" -ForegroundColor Green
    $writer.WriteLine('{"action": "list_tasks", "params": {}}')
    $writer.Flush()
    $response = $reader.ReadLine()
    Write-Host "Response: $response" -ForegroundColor White
    
    Start-Sleep -Milliseconds 500
    
    # Close connection
    $client.Close()
    Write-Host "`nTest completed successfully!" -ForegroundColor Green
}
catch {
    Write-Host "Error: $_" -ForegroundColor Red
}
finally {
    # Stop server
    Write-Host "`nStopping server..." -ForegroundColor Yellow
    if ($server -and !$server.HasExited) {
        Stop-Process -Id $server.Id -Force
    }
}
