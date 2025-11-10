// DPLang 命令行解释器 - 精简版

use dplang::{
    lexer::Lexer,
    parser::Parser,
    executor::DataStreamExecutor,
    runtime::Value,
    api::{parse_csv, format_output_csv},
};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }
    
    match args[1].as_str() {
        "run" => {
            if args.len() < 3 {
                eprintln!("错误: 请指定要运行的脚本文件");
                eprintln!("用法: dplang run <script.dp> [data.csv]");
                return;
            }
            
            let script_path = &args[2];
            let csv_path = if args.len() >= 4 {
                Some(&args[3])
            } else {
                None
            };
            
            run_script(script_path, csv_path);
        }
        "help" | "-h" | "--help" => {
            print_usage();
        }
        "version" | "-v" | "--version" => {
            print_version();
        }
        _ => {
            eprintln!("未知命令: {}", args[1]);
            eprintln!("运行 'dplang help' 查看帮助信息");
        }
    }
}

fn print_usage() {
    println!("DPLang v0.4.0 - 流式数据处理语言解释器\n");
    println!("用法:");
    println!("  dplang run <script.dp> [data.csv]    执行脚本");
    println!("  dplang help                          显示帮助信息");
    println!("  dplang version                       显示版本信息\n");
    
    println!("示例:");
    println!("  # 执行脚本（交互式输入数据）");
    println!("  dplang run script.dp");
    println!();
    println!("  # 使用CSV文件作为输入");
    println!("  dplang run script.dp data.csv");
    println!();
    
    println!("更多信息: https://github.com/yourusername/dplang");
}

fn print_version() {
    println!("DPLang v0.4.0");
    println!("简单、高效、AI友好的流式数据处理语言解释器");
}

/// 执行脚本
fn run_script(script_path: &str, csv_path: Option<&String>) {
    // 读取脚本文件
    let source = match fs::read_to_string(script_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("错误: 无法读取脚本文件 '{}': {}", script_path, e);
            return;
        }
    };
    
    // 解析脚本
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("词法分析错误: {:?}", e);
            return;
        }
    };
    
    let mut parser = Parser::new(tokens);
    let script = match parser.parse() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("语法分析错误: {:?}", e);
            return;
        }
    };
    
    println!("✅ 脚本解析成功\n");
    
    // 根据是否提供CSV文件选择不同的输入方式
    let input_matrix = if let Some(csv_file) = csv_path {
        // 使用CSV文件输入
        let csv_content = match fs::read_to_string(csv_file) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("错误: 无法读取CSV文件 '{}': {}", csv_file, e);
                return;
            }
        };
        
        match parse_csv(&csv_content) {
            Ok(data) => {
                println!("✅ CSV解析成功，共 {} 行数据\n", data.len());
                data
            }
            Err(e) => {
                eprintln!("CSV解析错误: {}", e);
                return;
            }
        }
    } else {
        // 交互式输入
        println!("请输入数据（JSON格式，每行一条，空行结束）:");
        println!("示例: {{\"close\": 100.5, \"volume\": 1000}}");
        println!();
        
        let mut input = Vec::new();
        let stdin = io::stdin();
        
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            
            let mut line = String::new();
            stdin.read_line(&mut line).unwrap();
            let line = line.trim();
            
            if line.is_empty() {
                break;
            }
            
            if let Ok(row) = parse_simple_json(line) {
                input.push(row);
            } else {
                eprintln!("警告: 无法解析输入: {}", line);
            }
        }
        
        if input.is_empty() {
            println!("使用空输入执行...");
            vec![HashMap::new()]
        } else {
            input
        }
    };
    
    // 执行脚本
    let mut executor = DataStreamExecutor::new(script, input_matrix);
    match executor.execute_all() {
        Ok(output) => {
            println!("\n✅ 执行成功!\n");
            
            if csv_path.is_some() {
                // CSV输入时，输出CSV格式
                println!("输出结果 (CSV格式):");
                println!("{}", format_output_csv(&output));
            } else {
                // 交互式输入时，输出JSON格式
                println!("输出结果:");
                for (i, row) in output.iter().enumerate() {
                    println!("  行 {}: {:?}", i + 1, row);
                }
            }
        }
        Err(e) => {
            eprintln!("\n❌ 执行错误: {:?}", e);
        }
    }
}

/// 简单的JSON解析器（仅支持基本类型）
fn parse_simple_json(line: &str) -> Result<HashMap<String, Value>, ()> {
    let line = line.trim();
    if !line.starts_with('{') || !line.ends_with('}') {
        return Err(());
    }
    
    let mut result = HashMap::new();
    let content = &line[1..line.len()-1];
    
    for pair in content.split(',') {
        let parts: Vec<&str> = pair.split(':').collect();
        if parts.len() != 2 {
            continue;
        }
        
        let key = parts[0].trim().trim_matches('"').to_string();
        let value_str = parts[1].trim();
        
        let value = if value_str.starts_with('"') && value_str.ends_with('"') {
            // 字符串
            Value::String(value_str.trim_matches('"').to_string())
        } else if let Ok(n) = value_str.parse::<f64>() {
            // 数字
            Value::Number(n)
        } else if value_str == "true" {
            Value::Bool(true)
        } else if value_str == "false" {
            Value::Bool(false)
        } else if value_str == "null" {
            Value::Null
        } else {
            continue;
        };
        
        result.insert(key, value);
    }
    
    Ok(result)
}
