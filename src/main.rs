// DPLang 命令行解释器

use dplang::{
    lexer::Lexer,
    parser::Parser,
    executor::{DataStreamExecutor, StreamingExecutor},
    runtime::Value,
    api::{parse_csv, format_output_csv},
    streaming::{CSVStreamWriter, CSVWriterConfig, CSVMode},
    orchestration::server::OrchestrationServer,
};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write, BufRead, BufReader};
use std::path::PathBuf;

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
            if args.len() >= 4 {
                // 使用 CSV 文件输入
                run_script_with_csv(&args[2], &args[3]);
            } else {
                // 交互式输入
                run_script(&args[2]);
            }
        }
        "daemon" => {
            if args.len() < 3 {
                eprintln!("错误: 请指定要运行的脚本文件");
                eprintln!("用法: dplang daemon <script.dp> [data.csv]");
                return;
            }
            let csv_args = if args.len() >= 4 {
                &args[3..]
            } else {
                &[]
            };
            run_daemon(&args[2], csv_args);
        }
        "demo" => {
            run_demo();
        }
        "orchestrate" => {
            let config_file = if args.len() >= 3 {
                &args[2]
            } else {
                "tasks.toml"
            };
            let port = if args.len() >= 4 {
                args[3].parse::<u16>().unwrap_or(8888)
            } else {
                8888
            };
            run_orchestrate(config_file, port);
        }
        "help" | "-h" | "--help" => {
            print_usage();
        }
        _ => {
            eprintln!("未知命令: {}", args[1]);
            print_usage();
        }
    }
}

fn print_usage() {
    println!("DPLang - 金融数据分析语言\n");
    println!("用法:");
    println!("  dplang run <script.dp>             运行指定的脚本文件 (交互式 JSON 输入)");
    println!("  dplang run <script.dp> <data.csv>  使用 CSV 文件作为输入");
    println!("  dplang daemon <script.dp> [data.csv]  实时流式计算模式");
    println!("  dplang orchestrate [config] [port] 启动任务编排服务器");
    println!("  dplang demo                        运行内置演示");
    println!("  dplang help                        显示帮助信息\n");
    println!("示例:");
    println!("  dplang run examples/hello.dp");
    println!("  dplang run examples/moving_average.dp data.csv");
    println!("  dplang daemon examples/realtime.dp data.csv");
    println!("  dplang orchestrate tasks.toml 8888");
}

fn run_script(file_path: &str) {
    println!("运行脚本: {}\n", file_path);
    
    // 读取文件
    let source = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("错误: 无法读取文件 '{}': {}", file_path, e);
            return;
        }
    };
    
    println!("脚本内容:");
    println!("{}", source);
    println!();
    
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
    
    // 提示输入数据
    println!("请输入数据（JSON格式，每行一条，空行结束）:");
    println!("示例: {{\"close\": 100.5, \"volume\": 1000}}");
    println!();
    
    let mut input_matrix = Vec::new();
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
        
        // 简单的 JSON 解析（仅支持基本类型）
        if let Ok(row) = parse_simple_json(line) {
            input_matrix.push(row);
        } else {
            eprintln!("警告: 无法解析输入: {}", line);
        }
    }
    
    if input_matrix.is_empty() {
        println!("使用空输入执行...");
        input_matrix.push(HashMap::new());
    }
    
    // 执行脚本
    let mut executor = DataStreamExecutor::new(script, input_matrix);
    match executor.execute_all() {
        Ok(output) => {
            println!("\n✅ 执行成功!\n");
            println!("输出结果:");
            for (i, row) in output.iter().enumerate() {
                println!("  行 {}: {:?}", i + 1, row);
            }
        }
        Err(e) => {
            eprintln!("\n❌ 执行错误: {:?}", e);
        }
    }
}

/// 使用 CSV 文件运行脚本
fn run_script_with_csv(script_path: &str, csv_path: &str) {
    println!("运行脚本: {}", script_path);
    println!("CSV 数据: {}\n", csv_path);
    
    // 读取脚本文件
    let source = match fs::read_to_string(script_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("错误: 无法读取脚本文件 '{}': {}", script_path, e);
            return;
        }
    };
    
    // 读取 CSV 文件
    let csv_content = match fs::read_to_string(csv_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("错误: 无法读取 CSV 文件 '{}': {}", csv_path, e);
            return;
        }
    };
    
    println!("CSV 数据:");
    println!("{}", csv_content);
    println!();
    
    // 解析 CSV
    let input_matrix = match parse_csv(&csv_content) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("CSV 解析错误: {}", e);
            return;
        }
    };
    
    println!("✅ CSV 解析成功，共 {} 行数据\n", input_matrix.len());
    
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
    
    // 执行脚本
    let mut executor = DataStreamExecutor::new(script, input_matrix);
    match executor.execute_all() {
        Ok(output) => {
            println!("✅ 执行成功!\n");
            println!("输出结果 (CSV 格式):");
            println!("{}", format_output_csv(&output));
        }
        Err(e) => {
            eprintln!("执行错误: {:?}", e);
        }
    }
}

// 简单的 JSON 解析器（仅支持基本类型）
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
        
        let key = parts[0].trim().trim_matches('\"').to_string();
        let value_str = parts[1].trim();
        
        let value = if value_str.starts_with('\"') && value_str.ends_with('\"') {
            // 字符串
            Value::String(value_str.trim_matches('\"').to_string())
        } else if let Ok(n) = value_str.parse::<f64>() {
            // 数字
            Value::Number(n)
        } else if value_str == "true" {
            Value::Bool(true)
        } else if value_str == "false" {
            Value::Bool(false)
        } else {
            continue;
        };
        
        result.insert(key, value);
    }
    
    Ok(result)
}

fn run_demo() {
    println!("=== DPLang 演示 ===");
    println!();
    
    demo_simple_calculation();
    demo_technical_indicators();
}

fn demo_simple_calculation() {
    println!("--- 示例 1: 简单计算 ---");
    
    let source = r#"
-- INPUT price:number, quantity:number --
-- OUTPUT total:number, tax:number, final:number --

total = price * quantity
tax = total * 0.1
final = total + tax

print("单价:", price, "数量:", quantity)
print("小计:", total, "税:", tax, "总计:", final)

return [total, tax, final]
"#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let script = parser.parse().unwrap();

    let input_matrix = vec![
        vec![
            ("price".to_string(), Value::Number(100.0)),
            ("quantity".to_string(), Value::Number(5.0)),
        ].into_iter().collect(),
    ];

    let mut executor = DataStreamExecutor::new(script, input_matrix);
    let output = executor.execute_all().unwrap();

    println!("输出结果:");
    for (i, row) in output.iter().enumerate() {
        println!("  行 {}: {:?}", i + 1, row);
    }
    println!();
}

fn demo_technical_indicators() {
    println!("--- 示例 2: 技术指标计算 ---");
    
    let source = r#"
-- INPUT --
-- OUTPUT ma5:number --

prices = [100, 102, 101, 103, 105]
ma5 = SMA(prices, 5)

print("MA5:", ma5)

return [ma5]
"#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let script = parser.parse().unwrap();

    let input_matrix = vec![HashMap::new()];

    let mut executor = DataStreamExecutor::new(script, input_matrix);
    let output = executor.execute_all().unwrap();

    println!("输出结果:");
    for (i, row) in output.iter().enumerate() {
        println!("  行 {}: {:?}", i + 1, row);
    }
    println!();
}

/// 实时流式计算模式
fn run_daemon(script_path: &str, args: &[String]) {
    println!("=== DPLang 实时流式计算模式 ===");
    println!("脚本: {}\n", script_path);
    
    // 解析命令行参数
    let csv_file = if !args.is_empty() {
        Some(args[0].clone())
    } else {
        None
    };
    
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
    
    // 创建流式执行器
    let mut executor = StreamingExecutor::new(script, 1000);
    
    // 创建 CSV 写入器
    let writer_config = CSVWriterConfig {
        output_dir: PathBuf::from("./output"),
        mode: CSVMode::Split,
        buffer_size: 100,
        auto_flush: false,
    };
    
    let mut csv_writer = match CSVStreamWriter::new(writer_config) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("错误: 无法创建 CSV 写入器: {}", e);
            return;
        }
    };
    
    println!("🚀 实时引擎已启动");
    println!("输出目录: ./output");
    println!("窗口大小: 1000 行\n");
    
    if let Some(csv_path) = csv_file {
        // 从 CSV 文件流式读取
        println!("读取 CSV 文件: {}\n", csv_path);
        run_daemon_from_csv(&csv_path, &mut executor, &mut csv_writer);
    } else {
        // 从标准输入读取
        println!("等待标准输入 (CSV 格式, Ctrl+C 退出)...");
        println!("格式: stock_code,field1,field2,...\n");
        run_daemon_from_stdin(&mut executor, &mut csv_writer);
    }
    
    // 刷新所有输出
    if let Err(e) = csv_writer.flush_all() {
        eprintln!("警告: 刷新输出失败: {}", e);
    }
    
    println!("\n✅ 实时引擎已停止");
}

/// 任务编排模式
fn run_orchestrate(config_file: &str, port: u16) {
    use std::path::Path;
    
    println!("=== DPLang 任务编排服务器 ===");
    println!("配置文件: {}", config_file);
    println!("监听端口: {}\n", port);
    
    let server = OrchestrationServer::new(port);
    
    // 加载配置文件
    let config_path = Path::new(config_file);
    if config_path.exists() {
        match server.load_config(config_path) {
            Ok(_) => println!("✅ 配置文件加载成功\n"),
            Err(e) => {
                eprintln!("❌ 配置文件加载失败: {}\n", e);
                eprintln!("服务器将以空配置启动，可通过API创建任务");
            }
        }
    } else {
        println!("⚠ 配置文件不存在: {}", config_file);
        println!("服务器将以空配置启动，可通过API创建任务\n");
    }
    
    // 启动服务器
    if let Err(e) = server.start() {
        eprintln!("❌ 服务器启动失败: {}", e);
    }
}

/// 从 CSV 文件流式读取并执行
fn run_daemon_from_csv(
    csv_path: &str,
    executor: &mut StreamingExecutor,
    csv_writer: &mut CSVStreamWriter,
) {
    let file = match fs::File::open(csv_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("错误: 无法打开 CSV 文件: {}", e);
            return;
        }
    };
    
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    // 读取表头
    let headers = if let Some(Ok(header_line)) = lines.next() {
        header_line.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>()
    } else {
        eprintln!("错误: CSV 文件为空");
        return;
    };
    
    println!("表头: {:?}\n", headers);
    
    let mut processed = 0;
    let mut errors = 0;
    
    // 逐行处理
    for (line_no, line_result) in lines.enumerate() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                eprintln!("警告: 读取第 {} 行失败: {}", line_no + 2, e);
                errors += 1;
                continue;
            }
        };
        
        // 解析 CSV 行
        let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if values.len() != headers.len() {
            eprintln!("警告: 第 {} 行列数不匹配", line_no + 2);
            errors += 1;
            continue;
        }
        
        // 构造 tick 数据
        let mut tick_data = HashMap::new();
        let mut stock_code = String::new();
        
        for (i, header) in headers.iter().enumerate() {
            if header == "stock_code" {
                stock_code = values[i].to_string();
            }
            
            let value_str = values[i];
            let value = if let Ok(n) = value_str.parse::<f64>() {
                Value::Number(n)
            } else if value_str == "true" {
                Value::Bool(true)
            } else if value_str == "false" {
                Value::Bool(false)
            } else if value_str.is_empty() || value_str == "null" {
                Value::Null
            } else {
                Value::String(value_str.to_string())
            };
            
            tick_data.insert(header.clone(), value);
        }
        
        // 执行 tick
        match executor.push_tick(tick_data) {
            Ok(Some(output)) => {
                // 写入输出
                if let Err(e) = csv_writer.write_row(&stock_code, &output) {
                    eprintln!("警告: 写入输出失败: {}", e);
                    errors += 1;
                } else {
                    processed += 1;
                    if processed % 100 == 0 {
                        println!("已处理: {} 行", processed);
                    }
                }
            }
            Ok(None) => {
                processed += 1;
            }
            Err(e) => {
                eprintln!("警告: 执行第 {} 行失败: {:?}", line_no + 2, e);
                errors += 1;
            }
        }
    }
    
    println!("\n总计处理: {} 行", processed);
    if errors > 0 {
        println!("错误: {} 行", errors);
    }
}

/// 从标准输入流式读取并执行
fn run_daemon_from_stdin(
    executor: &mut StreamingExecutor,
    csv_writer: &mut CSVStreamWriter,
) {
    let stdin = io::stdin();
    let reader = stdin.lock();
    let mut lines = reader.lines();
    
    // 读取表头
    let headers = if let Some(Ok(header_line)) = lines.next() {
        header_line.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>()
    } else {
        eprintln!("错误: 未接收到表头");
        return;
    };
    
    println!("表头: {:?}\n", headers);
    
    let mut processed = 0;
    
    // 逐行处理
    for line_result in lines {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                eprintln!("警告: 读取输入失败: {}", e);
                continue;
            }
        };
        
        if line.trim().is_empty() {
            continue;
        }
        
        // 解析 CSV 行
        let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if values.len() != headers.len() {
            eprintln!("警告: 列数不匹配");
            continue;
        }
        
        // 构造 tick 数据
        let mut tick_data = HashMap::new();
        let mut stock_code = String::new();
        
        for (i, header) in headers.iter().enumerate() {
            if header == "stock_code" {
                stock_code = values[i].to_string();
            }
            
            let value_str = values[i];
            let value = if let Ok(n) = value_str.parse::<f64>() {
                Value::Number(n)
            } else if value_str == "true" {
                Value::Bool(true)
            } else if value_str == "false" {
                Value::Bool(false)
            } else if value_str.is_empty() || value_str == "null" {
                Value::Null
            } else {
                Value::String(value_str.to_string())
            };
            
            tick_data.insert(header.clone(), value);
        }
        
        // 执行 tick
        match executor.push_tick(tick_data) {
            Ok(Some(output)) => {
                // 写入输出
                if let Err(e) = csv_writer.write_row(&stock_code, &output) {
                    eprintln!("警告: 写入输出失败: {}", e);
                } else {
                    processed += 1;
                    if processed % 100 == 0 {
                        println!("已处理: {} 行", processed);
                    }
                }
            }
            Ok(None) => {
                processed += 1;
            }
            Err(e) => {
                eprintln!("警告: 执行失败: {:?}", e);
            }
        }
    }
    
    println!("\n总计处理: {} 行", processed);
}
