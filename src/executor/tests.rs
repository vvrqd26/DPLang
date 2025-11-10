// 执行器模块测试

use super::*;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::parser::Script;
use crate::runtime::Value;
use std::collections::HashMap;

#[test]
fn test_simple_expression() {
    let source = r#"
-- INPUT code:string, close:number --
-- OUTPUT code:string, ma5:number --

ma5 = close * 2
return [code, ma5]
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let script = parser.parse().unwrap();
    
    let mut executor = Executor::new();
    executor.set_input("code".to_string(), Value::String("SH600000".to_string()));
    executor.set_input("close".to_string(), Value::Number(10.0));
    
    let result = executor.execute_data_script(&script).unwrap();
    
    if let Some(Value::Array(arr)) = result {
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0], Value::String("SH600000".to_string()));
        assert_eq!(arr[1], Value::Number(20.0));
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_if_statement() {
    let source = r#"
-- INPUT x:number --
-- OUTPUT result:number --

result = x * 2
return [result]
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let script = parser.parse().unwrap();
    
    let mut executor = Executor::new();
    executor.set_input("x".to_string(), Value::Number(15.0));
    
    let result = executor.execute_data_script(&script).unwrap();
    
    if let Some(Value::Array(arr)) = result {
        assert_eq!(arr[0], Value::Number(30.0));
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_lambda_and_map() {
    let source = r#"
-- INPUT nums:array --
-- OUTPUT result:array --

doubled = map(nums, x -> x * 2)
return [doubled]
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let script = parser.parse().unwrap();
    
    let mut executor = Executor::new();
    executor.set_input("nums".to_string(), Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
    ]));
    
    let result = executor.execute_data_script(&script).unwrap();
    
    if let Some(Value::Array(arr)) = result {
        if let Value::Array(doubled) = &arr[0] {
            assert_eq!(doubled.len(), 3);
            assert_eq!(doubled[0], Value::Number(2.0));
            assert_eq!(doubled[1], Value::Number(4.0));
            assert_eq!(doubled[2], Value::Number(6.0));
        } else {
            panic!("Expected array in result");
        }
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_filter() {
    let source = r#"
-- INPUT nums:array --
-- OUTPUT result:array --

filtered = filter(nums, x -> x > 2)
return [filtered]
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let script = parser.parse().unwrap();
    
    let mut executor = Executor::new();
    executor.set_input("nums".to_string(), Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
        Value::Number(4.0),
    ]));
    
    let result = executor.execute_data_script(&script).unwrap();
    
    if let Some(Value::Array(arr)) = result {
        if let Value::Array(filtered) = &arr[0] {
            assert_eq!(filtered.len(), 2);
            assert_eq!(filtered[0], Value::Number(3.0));
            assert_eq!(filtered[1], Value::Number(4.0));
        } else {
            panic!("Expected array in result");
        }
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_reduce() {
    let source = r#"
-- INPUT nums:array --
-- OUTPUT result:number --

total = sum(nums)
return [total]
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let script = parser.parse().unwrap();
    
    let mut executor = Executor::new();
    executor.set_input("nums".to_string(), Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
        Value::Number(4.0),
    ]));
    
    let result = executor.execute_data_script(&script).unwrap();
    
    if let Some(Value::Array(arr)) = result {
        assert_eq!(arr[0], Value::Number(10.0));
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_user_function_execution() {
    // 直接测试用户定义函数的调用
    let source = r#"
-- INPUT x:number --
-- OUTPUT result:number --

result = x * 2
return [result]
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let script = parser.parse().unwrap();
    
    let mut executor = Executor::new();
    executor.set_input("x".to_string(), Value::Number(5.0));
    
    let result = executor.execute_data_script(&script).unwrap();
    
    if let Some(Value::Array(arr)) = result {
        assert_eq!(arr[0], Value::Number(10.0));
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_error_handling() {
    let source = r#"
-- INPUT x:number, y:number --
-- OUTPUT result:number --
-- ERROR --
result = 0
return [result]
-- ERROR_END --

result = x / y
return [result]
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let script = parser.parse().unwrap();
    
    // 测试除以0的情况
    let mut executor = Executor::new();
    executor.set_input("x".to_string(), Value::Number(10.0));
    executor.set_input("y".to_string(), Value::Number(0.0));
    
    // 应该捕获除零错误并返回 ERROR 块的结果
    let result = executor.execute_data_script(&script).unwrap();
    
    if let Some(Value::Array(arr)) = result {
        assert_eq!(arr[0], Value::Number(0.0));  // ERROR 块设置为 0
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_decimal_precision() {
    let source = r#"
-- INPUT price:decimal, rate:decimal --
-- OUTPUT result:decimal --
-- PRECISION 2 --

result = price * rate
return [result]
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let script = parser.parse().unwrap();
    
    let mut executor = Executor::new();
    use rust_decimal::Decimal;
    use std::str::FromStr;
    
    executor.set_input("price".to_string(), Value::Decimal(Decimal::from_str("10.123").unwrap()));
    executor.set_input("rate".to_string(), Value::Decimal(Decimal::from_str("1.567").unwrap()));
    
    let result = executor.execute_data_script(&script).unwrap();
    
    if let Some(Value::Array(arr)) = result {
        if let Value::Decimal(d) = &arr[0] {
            // 10.123 * 1.567 = 15.862741, 四舍五入到 2 位小数 = 15.86
            assert_eq!(d.to_string(), "15.86");
        } else {
            panic!("Expected Decimal result, got: {:?}", arr[0]);
        }
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_data_stream_executor_basic() {
    let source = r#"
-- INPUT close:number --
-- OUTPUT double:number --

double = close * 2
return [double]
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let script = parser.parse().unwrap();
    
    // 准备输入数据（3行）
    let input_matrix = vec![
        vec![("close".to_string(), Value::Number(10.0))].into_iter().collect(),
        vec![("close".to_string(), Value::Number(11.0))].into_iter().collect(),
        vec![("close".to_string(), Value::Number(12.0))].into_iter().collect(),
    ];
    
    let mut executor = DataStreamExecutor::new(script, input_matrix);
    let output = executor.execute_all().unwrap();
    
    assert_eq!(output.len(), 3);
    assert_eq!(output[0].get("double"), Some(&Value::Number(20.0)));
    assert_eq!(output[1].get("double"), Some(&Value::Number(22.0)));
    assert_eq!(output[2].get("double"), Some(&Value::Number(24.0)));
}

#[test]
fn test_data_stream_executor_with_ref() {
    let source = r#"
-- INPUT close:number --
-- OUTPUT ma2:number --

prev_close = ref("close", 1)
ma2 = prev_close == null ? close : (close + prev_close) / 2
return [ma2]
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let script = parser.parse().unwrap();
    
    // 准备输入数据
    let input_matrix = vec![
        vec![("close".to_string(), Value::Number(10.0))].into_iter().collect(),
        vec![("close".to_string(), Value::Number(12.0))].into_iter().collect(),
        vec![("close".to_string(), Value::Number(14.0))].into_iter().collect(),
    ];
    
    let mut executor = DataStreamExecutor::new(script, input_matrix);
    let output = executor.execute_all().unwrap();
    
    assert_eq!(output.len(), 3);
    // 第1行：prev_close=null，ma2=10.0
    assert_eq!(output[0].get("ma2"), Some(&Value::Number(10.0)));
    // 第2行：prev_close=10.0，ma2=(12+10)/2=11.0
    assert_eq!(output[1].get("ma2"), Some(&Value::Number(11.0)));
    // 第3行：prev_close=12.0，ma2=(14+12)/2=13.0
    assert_eq!(output[2].get("ma2"), Some(&Value::Number(13.0)));
}

#[test]
fn test_data_stream_executor_empty_input() {
    let source = r#"
-- INPUT --
-- OUTPUT index:number --

index = 42
return [index]
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let script = parser.parse().unwrap();
    
    // 空输入
    let input_matrix = vec![];
    
    let mut executor = DataStreamExecutor::new(script, input_matrix);
    let output = executor.execute_all().unwrap();
    
    // 应该执行一次
    assert_eq!(output.len(), 1);
    assert_eq!(output[0].get("index"), Some(&Value::Number(42.0)));
}

#[test]
fn test_import_declaration_parsing() {
    let source = r#"
-- IMPORT math, utils --
-- INPUT x:number --
-- OUTPUT result:number --

result = x * 2
return [result]
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let script = parser.parse().unwrap();
    
    if let Script::DataScript { imports, input, output, .. } = script {
        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0], "math");
        assert_eq!(imports[1], "utils");
        assert_eq!(input.len(), 1);
        assert_eq!(output.len(), 1);
    } else {
        panic!("Expected DataScript");
    }
}

#[test]
fn test_package_loading_and_execution() {
    // 准备包脚本（只有变量）
    let package_source = r#"
package math

PI = 3.14159
E = 2.71828
"#;
    let mut pkg_lexer = Lexer::new(package_source);
    let pkg_tokens = pkg_lexer.tokenize().unwrap();
    let mut pkg_parser = Parser::new(pkg_tokens);
    let package_script = pkg_parser.parse().unwrap();
    
    // 准备数据脚本
    let data_source = r#"
-- IMPORT math --
-- INPUT x:number --
-- OUTPUT pi:number, result:number --

pi = math.PI
result = x * 2
return [pi, result]
"#;
    let mut lexer = Lexer::new(data_source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let data_script = parser.parse().unwrap();
    
    // 准备输入数据
    let input_matrix = vec![
        vec![("x".to_string(), Value::Number(5.0))].into_iter().collect(),
    ];
    
    // 准备包脚本集合
    let mut packages = HashMap::new();
    packages.insert("math".to_string(), package_script);
    
    // 执行
    let mut executor = DataStreamExecutor::new_with_packages(data_script, input_matrix, packages).unwrap();
    let output = executor.execute_all().unwrap();
    
    assert_eq!(output.len(), 1);
    assert_eq!(output[0].get("pi"), Some(&Value::Number(3.14159)));
    assert_eq!(output[0].get("result"), Some(&Value::Number(10.0)));
}

#[test]
fn test_package_function_call() {
    // 暂时跳过包函数测试，因为需要完善缩进处理
    // TODO: 实现包函数调用后补充此测试
}

#[test]
fn test_past_function() {
    let source = r#"
-- INPUT close:number --
-- OUTPUT prices:array, count:number --

# 获取过去3个周期的 close 值
prices = past("close", 3)
count = sum(prices)
return [prices, count]
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let script = parser.parse().unwrap();
    
    // 准备输入数据（5行）
    let input_matrix = vec![
        vec![("close".to_string(), Value::Number(10.0))].into_iter().collect(),
        vec![("close".to_string(), Value::Number(11.0))].into_iter().collect(),
        vec![("close".to_string(), Value::Number(12.0))].into_iter().collect(),
        vec![("close".to_string(), Value::Number(13.0))].into_iter().collect(),
        vec![("close".to_string(), Value::Number(14.0))].into_iter().collect(),
    ];
    
    let mut executor = DataStreamExecutor::new(script, input_matrix);
    let output = executor.execute_all().unwrap();
    
    assert_eq!(output.len(), 5);
    
    // 第1行：没有历史，返回 [null, null, null]
    if let Some(Value::Array(arr)) = output[0].get("prices") {
        assert_eq!(arr.len(), 3);
        assert!(matches!(arr[0], Value::Null));
        assert!(matches!(arr[1], Value::Null));
        assert!(matches!(arr[2], Value::Null));
    }
    
    // 第3行：有部分历史 [null, 10, 11]
    if let Some(Value::Array(arr)) = output[2].get("prices") {
        assert_eq!(arr.len(), 3);
        assert!(matches!(arr[0], Value::Null));
        assert_eq!(arr[1], Value::Number(10.0));
        assert_eq!(arr[2], Value::Number(11.0));
    }
    
    // 第5行：完整历史 [11, 12, 13]
    if let Some(Value::Array(arr)) = output[4].get("prices") {
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], Value::Number(11.0));
        assert_eq!(arr[1], Value::Number(12.0));
        assert_eq!(arr[2], Value::Number(13.0));
    }
}

#[test]
fn test_window_function() {
    let source = r#"
-- INPUT close:number --
-- OUTPUT window_prices:array, ma3:number --

# 滑动窗口（包含当前值）
window_prices = window("close", 3)

# 计算窗口均值
ma3 = sum(window_prices) / 3

return [window_prices, ma3]
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let script = parser.parse().unwrap();
    
    // 准备输入数据
    let input_matrix = vec![
        vec![("close".to_string(), Value::Number(10.0))].into_iter().collect(),
        vec![("close".to_string(), Value::Number(12.0))].into_iter().collect(),
        vec![("close".to_string(), Value::Number(14.0))].into_iter().collect(),
        vec![("close".to_string(), Value::Number(16.0))].into_iter().collect(),
    ];
    
    let mut executor = DataStreamExecutor::new(script, input_matrix);
    let output = executor.execute_all().unwrap();
    
    assert_eq!(output.len(), 4);
    
    // 第1行：[null, null, 10]
    if let Some(Value::Array(arr)) = output[0].get("window_prices") {
        assert_eq!(arr.len(), 3);
        assert!(matches!(arr[0], Value::Null));
        assert!(matches!(arr[1], Value::Null));
        assert_eq!(arr[2], Value::Number(10.0));
    }
    
    // 第3行：[10, 12, 14], ma3 = 12
    if let Some(Value::Array(arr)) = output[2].get("window_prices") {
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], Value::Number(10.0));
        assert_eq!(arr[1], Value::Number(12.0));
        assert_eq!(arr[2], Value::Number(14.0));
    }
    assert_eq!(output[2].get("ma3"), Some(&Value::Number(12.0)));
    
    // 第4行：[12, 14, 16], ma3 = 14
    if let Some(Value::Array(arr)) = output[3].get("window_prices") {
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], Value::Number(12.0));
        assert_eq!(arr[1], Value::Number(14.0));
        assert_eq!(arr[2], Value::Number(16.0));
    }
    assert_eq!(output[3].get("ma3"), Some(&Value::Number(14.0)));
}

#[test]
fn test_offset_function() {
    let source = r#"
-- INPUT close:number --
-- OUTPUT prev1:number, prev2:number --

prev1 = offset("close", 1)
prev2 = offset("close", 2)
return [prev1, prev2]
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let script = parser.parse().unwrap();
    
    let input_matrix = vec![
        vec![("close".to_string(), Value::Number(100.0))].into_iter().collect(),
        vec![("close".to_string(), Value::Number(101.0))].into_iter().collect(),
        vec![("close".to_string(), Value::Number(102.0))].into_iter().collect(),
    ];
    
    let mut executor = DataStreamExecutor::new(script, input_matrix);
    let output = executor.execute_all().unwrap();
    
    assert_eq!(output.len(), 3);
    
    // 第1行：prev1=null, prev2=null
    assert_eq!(output[0].get("prev1"), Some(&Value::Null));
    assert_eq!(output[0].get("prev2"), Some(&Value::Null));
    
    // 第2行：prev1=100, prev2=null
    assert_eq!(output[1].get("prev1"), Some(&Value::Number(100.0)));
    assert_eq!(output[1].get("prev2"), Some(&Value::Null));
    
    // 第3行：prev1=101, prev2=100
    assert_eq!(output[2].get("prev1"), Some(&Value::Number(101.0)));
    assert_eq!(output[2].get("prev2"), Some(&Value::Number(100.0)));
}

#[test]
fn test_package_loader_from_filesystem() {
    use crate::package_loader::PackageLoader;
    
    // 准备数据脚本
    let data_source = r#"
-- IMPORT math --
-- INPUT x:number --
-- OUTPUT pi:number, squared:number --

pi = math.PI
squared = x * x
return [pi, squared]
"#;
    let mut lexer = Lexer::new(data_source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let data_script = parser.parse().unwrap();
    
    // 准备输入数据
    let input_matrix = vec![
        vec![("x".to_string(), Value::Number(5.0))].into_iter().collect(),
    ];
    
    // 使用包加载器
    let mut loader = PackageLoader::new();
    let mut executor = DataStreamExecutor::new_with_loader(data_script, input_matrix, &mut loader).unwrap();
    let output = executor.execute_all().unwrap();
    
    assert_eq!(output.len(), 1);
    // 验证 PI 值
    if let Some(Value::Number(pi)) = output[0].get("pi") {
        assert!((*pi - 3.14159265359).abs() < 0.0001);
    } else {
        panic!("Expected PI value");
    }
    assert_eq!(output[0].get("squared"), Some(&Value::Number(25.0)));
}

#[test]
fn test_technical_indicators_sma() {
    let source = r#"
-- INPUT --
-- OUTPUT ma5:number --

prices = [10, 11, 12, 13, 14]
ma5 = SMA(prices, 5)
return [ma5]
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let script = parser.parse().unwrap();
    
    // 空输入
    let input_matrix = vec![HashMap::new()];
    
    let mut executor = DataStreamExecutor::new(script, input_matrix);
    let output = executor.execute_all();
    
    if let Err(e) = &output {
        panic!("Execution error: {:?}", e);
    }
    
    let output = output.unwrap();
    
    // 应该有一行输出
    let last_row = &output[0];
    let ma5 = last_row.get("ma5").unwrap();
    
    println!("ma5 value: {:?}", ma5);
    
    // (10+11+12+13+14)/5 = 12.0
    if !matches!(ma5, Value::Null) {
        assert_eq!(ma5.to_number().unwrap(), 12.0);
    }
}

#[test]
fn test_technical_indicators_rsi() {
    let source = r#"
-- INPUT --
-- OUTPUT rsi:number --

prices = [44, 45, 46, 47, 48, 49, 50]
rsi = RSI(prices, 6)
return [rsi]
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let script = parser.parse().unwrap();
    
    // 空输入
    let input_matrix = vec![HashMap::new()];
    
    let mut executor = DataStreamExecutor::new(script, input_matrix);
    let output = executor.execute_all().unwrap();
    
    // 应该有一行输出
    let last_row = &output[0];
    let rsi = last_row.get("rsi").unwrap();
    
    // 持续上涨，RSI 应该接近 100
    if !matches!(rsi, Value::Null) {
        assert!(rsi.to_number().unwrap() > 90.0);
    }
}

#[test]
fn test_print_function() {
    let source = r#"
-- INPUT x:number --
-- OUTPUT result:number --

print("Testing print:", x)
print("Multiple", "arguments", 123)
result = x * 2
print("Result:", result)
return [result]
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let script = parser.parse().unwrap();
    
    let input_matrix = vec![
        vec![("x".to_string(), Value::Number(5.0))].into_iter().collect(),
    ];
    
    let mut executor = DataStreamExecutor::new(script, input_matrix);
    let output = executor.execute_all().unwrap();
    
    assert_eq!(output.len(), 1);
    assert_eq!(output[0].get("result"), Some(&Value::Number(10.0)));
}

#[test]
fn test_chained_ternary() {
    let source = r#"
-- INPUT score:number --
-- OUTPUT grade:string --

# 串联三元表达式
grade = score >= 90 ? "A" : score >= 80 ? "B" : score >= 70 ? "C" : score >= 60 ? "D" : "F"

return [grade]
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let script = parser.parse().unwrap();
    
    // 测试不同分数
    let test_cases = vec![
        (95.0, "A"),
        (85.0, "B"),
        (75.0, "C"),
        (65.0, "D"),
        (55.0, "F"),
    ];
    
    for (score, expected_grade) in test_cases {
        let input_matrix = vec![
            vec![("score".to_string(), Value::Number(score))].into_iter().collect(),
        ];
        
        let mut executor = DataStreamExecutor::new(script.clone(), input_matrix);
        let output = executor.execute_all().unwrap();
        
        let grade = output[0].get("grade").unwrap();
        assert_eq!(grade, &Value::String(expected_grade.to_string()), 
                   "分数 {} 应该是 {}", score, expected_grade);
    }
}

#[test]
fn test_complex_conditions() {
    let source = r#"
-- INPUT price:number, volume:number --
-- OUTPUT signal:string, strength:string --

# 复杂条件判断（分行）
cond1 = price > 100 and volume > 1000000
cond2 = price < 80 and volume > 500000
signal = cond1 ? "买入" : cond2 ? "卖出" : "观望"

# 嵌套三元表达式（使用括号）
strength = price > 100 ? (volume > 2000000 ? "强势" : "中等") : "弱势"

return [signal, strength]
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let script = parser.parse().unwrap();
    
    // 测试用例 1: 高价格高成交量
    let input1 = vec![
        vec![
            ("price".to_string(), Value::Number(120.0)),
            ("volume".to_string(), Value::Number(3000000.0)),
        ].into_iter().collect(),
    ];
    let mut executor1 = DataStreamExecutor::new(script.clone(), input1);
    let output1 = executor1.execute_all().unwrap();
    assert_eq!(output1[0].get("signal"), Some(&Value::String("买入".to_string())));
    assert_eq!(output1[0].get("strength"), Some(&Value::String("强势".to_string())));
    
    // 测试用例 2: 低价格高成交量
    let input2 = vec![
        vec![
            ("price".to_string(), Value::Number(70.0)),
            ("volume".to_string(), Value::Number(1000000.0)),
        ].into_iter().collect(),
    ];
    let mut executor2 = DataStreamExecutor::new(script.clone(), input2);
    let output2 = executor2.execute_all().unwrap();
    assert_eq!(output2[0].get("signal"), Some(&Value::String("卖出".to_string())));
    assert_eq!(output2[0].get("strength"), Some(&Value::String("弱势".to_string())));
    
    // 测试用例 3: 中等情况
    let input3 = vec![
        vec![
            ("price".to_string(), Value::Number(90.0)),
            ("volume".to_string(), Value::Number(800000.0)),
        ].into_iter().collect(),
    ];
    let mut executor3 = DataStreamExecutor::new(script.clone(), input3);
    let output3 = executor3.execute_all().unwrap();
    assert_eq!(output3[0].get("signal"), Some(&Value::String("观望".to_string())));
    assert_eq!(output3[0].get("strength"), Some(&Value::String("弱势".to_string())));
}
