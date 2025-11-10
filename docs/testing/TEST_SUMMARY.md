# Stock Technical Indicators Test - Summary

## Test Configuration

- **Test Data Location**: `test_data/`
- **Number of Stocks**: 100 (SH000001 - SH000100)
- **Days per Stock**: 60 trading days
- **Total Data Rows**: 6,000 rows
- **Data Columns**: date, stock_code, open, high, low, close, volume

## Test Components

### 1. Data Generation Script
**File**: `test_data/generate_stock_data.py`
- Generates realistic stock price data with OHLCV values
- Simulates price volatility (±3% per day)
- Output: `test_data/stock_data_100.csv`

### 2. DPLang Analysis Script  
**File**: `test_data/indicators.dp`
- Calculates multiple technical indicators:
  - MA5, MA10, MA20 (Simple Moving Averages)
  - More indicators can be added: EMA12, EMA26, RSI14, BOLL bands
- Per-stock calculation with proper time-series handling

### 3. Task Orchestration Config
**File**: `test_data/stock_tasks.toml`
- Routing: Hash-based on `stock_code` field
- Compute Pool: 10-50 instances
- Input: CSV format from stdin
- Output: CSV format to stdout

### 4. Automated Test Script
**File**: `test_data/test_stock_indicators.ps1`
- End-to-end testing workflow:
  1. Generate test data
  2. Build DPLang project
  3. Start orchestration server
  4. Test API communication
  5. Cleanup and summary

## Test Results

✅ **Data Generation**: Successfully created 6,000 rows of stock data  
✅ **Script Parsing**: DPLang script compiled without errors  
✅ **Data Processing**: All 6,000 rows processed successfully  
✅ **Output Generation**: Technical indicators calculated for all stocks  
✅ **Performance**: Completed processing in real-time streaming mode  

## Output Files

Results are stored in `./output/` directory:
- Multiple CSV files with calculated indicators
- Each file contains: stock_code, close, ma5, ma10, ma20

## Running the Test

### Quick Test
```powershell
# Run automated test
powershell -ExecutionPolicy Bypass -File test_data/test_stock_indicators.ps1
```

### Manual Test with Daemon Mode
```powershell
# Generate data
python test_data/generate_stock_data.py 100 60

# Process with DPLang
dplang daemon test_data/indicators.dp test_data/stock_data_100.csv
```

### Test with Orchestration Mode
```powershell
# Start server
dplang orchestrate test_data/stock_tasks.toml 8889

# Use API to manage tasks (list, start, stop, etc.)
```

## Key Features Demonstrated

1. **Multi-stock Processing**: Handles 100 different stocks simultaneously
2. **Time-series Calculation**: Moving averages calculated per stock with proper history
3. **Routing Strategy**: Hash-based routing distributes stocks across compute units
4. **Streaming Mode**: Real-time processing of large datasets
5. **Task Orchestration**: Server-based task management with API control

## Notes

- All test files are organized in `test_data/` directory (not in project root)
- Test data is generated programmatically for consistency
- Output demonstrates DPLang's capability to handle real-world financial data analysis scenarios
- The test setup can be easily scaled to more stocks or longer time periods

## Next Steps

To extend the test:
1. Add more technical indicators (RSI, MACD, Bollinger Bands)
2. Increase number of stocks to 1000+
3. Add performance benchmarking
4. Test with real market data
5. Implement data push API for orchestration mode
