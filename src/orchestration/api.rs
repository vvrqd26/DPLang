// API消息定义

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// API请求
#[derive(Debug, Deserialize)]
pub struct ApiRequest {
    pub action: String,
    #[serde(default)]
    pub params: JsonValue,
}

/// API响应
#[derive(Debug, Serialize)]
pub struct ApiResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<JsonValue>,
    pub message: String,
}

impl ApiResponse {
    pub fn ok(message: &str) -> Self {
        ApiResponse {
            status: "ok".to_string(),
            data: None,
            message: message.to_string(),
        }
    }
    
    pub fn ok_with_data(data: JsonValue, message: &str) -> Self {
        ApiResponse {
            status: "ok".to_string(),
            data: Some(data),
            message: message.to_string(),
        }
    }
    
    pub fn error(message: &str) -> Self {
        ApiResponse {
            status: "error".to_string(),
            data: None,
            message: message.to_string(),
        }
    }
}
