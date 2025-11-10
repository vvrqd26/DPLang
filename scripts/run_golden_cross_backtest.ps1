# Golden Cross Backtest Script
param(
    [int]$NumStocks = 500,
    [int]$Days = 250,
    [string]$OutputDir = "test_data/golden_cross_results"
)

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Golden Cross Strategy Backtest" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "Step 1: Generate test data" -ForegroundColor Yellow
Write-Host "  Number of stocks: $NumStocks" -ForegroundColor Gray
Write-Host "  Trading days: $Days" -ForegroundColor Gray
Write-Host "  Total rows: $($NumStocks * $Days)" -ForegroundColor Gray
Write-Host ""

$dataFile = "test_data/large_stock_data.csv"

if (-not (Test-Path "test_data")) {
    New-Item -ItemType Directory -Force -Path "test_data" | Out-Null
}

python scripts/generate_stock_data.py $NumStocks $Days

if (-not (Test-Path $dataFile)) {
    if (Test-Path "test_data/stock_data_100.csv") {
        Move-Item "test_data/stock_data_100.csv" $dataFile -Force
    } else {
        Write-Host "Error: Data generation failed" -ForegroundColor Red
        exit 1
    }
}

Write-Host "Data generated: $dataFile" -ForegroundColor Green
Write-Host ""

Write-Host "Step 2: Run backtest" -ForegroundColor Yellow

if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
}

$strategyFile = "examples/scripts/golden_cross_simple.dp"

# Use single stock data for meaningful backtest
$dataFile = "test_data/single_stock.csv"

Write-Host "Strategy: $strategyFile" -ForegroundColor Gray
Write-Host "Output: $OutputDir" -ForegroundColor Gray
Write-Host ""

$startTime = Get-Date

cargo run --release -- backtest $strategyFile $dataFile --output $OutputDir

$endTime = Get-Date
$duration = $endTime - $startTime

Write-Host ""
Write-Host "Backtest completed in $($duration.TotalSeconds) seconds" -ForegroundColor Green
Write-Host ""

# Display trade statistics if any
if (Test-Path "$OutputDir/trades.csv") {
    $trades = Import-Csv "$OutputDir/trades.csv"
    $tradeCount = $trades.Count
    
    if ($tradeCount -gt 0) {
        Write-Host "Trade Summary:" -ForegroundColor Cyan
        Write-Host "  Total Trades: $tradeCount" -ForegroundColor Gray
        
        $profitTrades = ($trades | Where-Object { [double]$_.net_profit -gt 0 }).Count
        $winRate = ($profitTrades / $tradeCount * 100)
        
        Write-Host "  Profitable: $profitTrades" -ForegroundColor Green
        Write-Host "  Losing: $($tradeCount - $profitTrades)" -ForegroundColor Red
        Write-Host "  Win Rate: $([math]::Round($winRate, 2))%" -ForegroundColor Cyan
        Write-Host ""
    } else {
        Write-Host "No trades generated - check strategy logic" -ForegroundColor Yellow
        Write-Host ""
    }
}

Write-Host "Results saved to: $OutputDir" -ForegroundColor Cyan
