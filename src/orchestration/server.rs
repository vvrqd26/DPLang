// 任务编排服务器 - TCP API服务器

use crate::orchestration::{TaskManager, TasksConfig, load_config};
use crate::orchestration::api::{ApiRequest, ApiResponse};
use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::path::Path;
use serde_json::json;

/// 编排服务器
pub struct OrchestrationServer {
    task_manager: Arc<Mutex<TaskManager>>,
    port: u16,
}

impl OrchestrationServer {
    /// 创建服务器
    pub fn new(port: u16) -> Self {
        OrchestrationServer {
            task_manager: Arc::new(Mutex::new(TaskManager::new())),
            port,
        }
    }
    
    /// 加载配置文件并创建任务
    pub fn load_config(&self, config_path: &Path) -> Result<(), String> {
        let config = load_config(config_path)?;
        
        let mut manager = self.task_manager.lock().unwrap();
        
        for task_config in config.task {
            if task_config.enabled {
                println!("创建任务: {} ({})", task_config.id, task_config.name);
                manager.create_task(task_config)?;
            }
        }
        
        Ok(())
    }
    
    /// 启动服务器
    pub fn start(&self) -> Result<(), String> {
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr)
            .map_err(|e| format!("无法绑定端口 {}: {}", addr, e))?;
        
        println!("🚀 任务编排服务器启动成功");
        println!("监听地址: {}", addr);
        println!("等待客户端连接...\n");
        
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    // 暂时不使用多线程，因为Rc<Vec>不是Send安全
                    // let manager = Arc::clone(&self.task_manager);
                    // thread::spawn(move || {
                    //     handle_client(stream, manager);
                    // });
                    
                    // 单线程处理（MVP版本）
                    handle_client(stream, Arc::clone(&self.task_manager));
                },
                Err(e) => {
                    eprintln!("连接错误: {}", e);
                }
            }
        }
        
        Ok(())
    }
}

/// 处理客户端连接
fn handle_client(mut stream: TcpStream, task_manager: Arc<Mutex<TaskManager>>) {
    let peer_addr = stream.peer_addr().ok();
    println!("✅ 客户端连接: {:?}", peer_addr);
    
    let reader = BufReader::new(stream.try_clone().unwrap());
    
    for line_result in reader.lines() {
        match line_result {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }
                
                // 解析请求
                let request: ApiRequest = match serde_json::from_str(&line) {
                    Ok(req) => req,
                    Err(e) => {
                        let response = ApiResponse::error(&format!("请求解析错误: {}", e));
                        send_response(&mut stream, &response);
                        continue;
                    }
                };
                
                // 处理请求
                let response = process_request(&request, &task_manager);
                send_response(&mut stream, &response);
            },
            Err(e) => {
                eprintln!("读取错误: {}", e);
                break;
            }
        }
    }
    
    println!("❌ 客户端断开: {:?}", peer_addr);
}

/// 处理API请求
fn process_request(request: &ApiRequest, task_manager: &Arc<Mutex<TaskManager>>) -> ApiResponse {
    let mut manager = task_manager.lock().unwrap();
    
    match request.action.as_str() {
        "list_tasks" => {
            let tasks = manager.list_tasks();
            let tasks_json: Vec<_> = tasks.iter().map(|t| {
                json!({
                    "id": t.id,
                    "name": t.name,
                    "status": t.status,
                    "compute_pool_size": t.compute_pool_size,
                    "processed_count": t.processed_count,
                })
            }).collect();
            
            ApiResponse::ok_with_data(
                json!({"tasks": tasks_json}),
                "任务列表获取成功"
            )
        },
        "start_task" => {
            if let Some(task_id) = request.params.get("task_id").and_then(|v| v.as_str()) {
                match manager.start_task(task_id) {
                    Ok(_) => ApiResponse::ok("任务已启动"),
                    Err(e) => ApiResponse::error(&e),
                }
            } else {
                ApiResponse::error("缺少参数: task_id")
            }
        },
        "stop_task" => {
            if let Some(task_id) = request.params.get("task_id").and_then(|v| v.as_str()) {
                match manager.stop_task(task_id) {
                    Ok(_) => ApiResponse::ok("任务已停止"),
                    Err(e) => ApiResponse::error(&e),
                }
            } else {
                ApiResponse::error("缺少参数: task_id")
            }
        },
        "pause_task" => {
            if let Some(task_id) = request.params.get("task_id").and_then(|v| v.as_str()) {
                match manager.pause_task(task_id) {
                    Ok(_) => ApiResponse::ok("任务已暂停"),
                    Err(e) => ApiResponse::error(&e),
                }
            } else {
                ApiResponse::error("缺少参数: task_id")
            }
        },
        "resume_task" => {
            if let Some(task_id) = request.params.get("task_id").and_then(|v| v.as_str()) {
                match manager.resume_task(task_id) {
                    Ok(_) => ApiResponse::ok("任务已继续"),
                    Err(e) => ApiResponse::error(&e),
                }
            } else {
                ApiResponse::error("缺少参数: task_id")
            }
        },
        "delete_task" => {
            if let Some(task_id) = request.params.get("task_id").and_then(|v| v.as_str()) {
                match manager.delete_task(task_id) {
                    Ok(_) => ApiResponse::ok("任务已删除"),
                    Err(e) => ApiResponse::error(&e),
                }
            } else {
                ApiResponse::error("缺少参数: task_id")
            }
        },
        _ => ApiResponse::error(&format!("未知操作: {}", request.action)),
    }
}

/// 发送响应
fn send_response(stream: &mut TcpStream, response: &ApiResponse) {
    if let Ok(json) = serde_json::to_string(response) {
        if let Err(e) = writeln!(stream, "{}", json) {
            eprintln!("发送响应失败: {}", e);
        }
    }
}
