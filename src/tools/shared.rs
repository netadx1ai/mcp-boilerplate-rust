use crate::types::McpError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Security: Max message size 10KB to prevent memory issues
const MAX_MESSAGE_LENGTH: usize = 10 * 1024;

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct EchoRequest {
    pub message: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct EchoResponse {
    pub message: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct PingResponse {
    pub response: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct InfoResponse {
    pub tool: String,
    pub version: String,
    pub description: String,
    pub timestamp: String,
}

pub fn create_echo_response(message: String) -> Result<EchoResponse, McpError> {
    if message.len() > MAX_MESSAGE_LENGTH {
        return Err(McpError::InvalidParams(format!(
            "Message too long: {} bytes (max: {} bytes)",
            message.len(),
            MAX_MESSAGE_LENGTH
        )));
    }
    if message.is_empty() {
        return Err(McpError::InvalidParams(
            "Message cannot be empty".to_string(),
        ));
    }
    Ok(EchoResponse {
        message,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

pub fn create_ping_response() -> PingResponse {
    PingResponse {
        response: "pong".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}

pub fn create_info_response() -> InfoResponse {
    InfoResponse {
        tool: "mcp-boilerplate-rust".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "MCP Boilerplate Rust Server".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}

// Additional response types for advanced tools
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SimpleLongTaskResponse {
    pub status: String,
    pub task_name: String,
    pub duration_seconds: u32,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SimpleProcessResponse {
    pub items_processed: usize,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SimpleBatchResponse {
    pub operations_completed: usize,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SimpleTransformResponse {
    pub result: String,
    pub transformation: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SimpleUploadResponse {
    pub status: String,
    pub filename: String,
    pub size_bytes: u64,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SimpleHealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: String,
}

pub fn create_long_task_response(
    task_name: String,
    duration_seconds: u32,
) -> SimpleLongTaskResponse {
    SimpleLongTaskResponse {
        status: "completed".to_string(),
        task_name,
        duration_seconds,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}

pub fn create_process_response(items: Vec<String>) -> SimpleProcessResponse {
    SimpleProcessResponse {
        items_processed: items.len(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}

pub fn create_batch_response(operations: Vec<String>) -> SimpleBatchResponse {
    SimpleBatchResponse {
        operations_completed: operations.len(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}

pub fn create_transform_response(
    data: String,
    transformation: String,
) -> Result<SimpleTransformResponse, McpError> {
    let result = match transformation.as_str() {
        "uppercase" => data.to_uppercase(),
        "lowercase" => data.to_lowercase(),
        "reverse" => data.chars().rev().collect(),
        _ => {
            return Err(McpError::InvalidParams(format!(
                "Unknown transformation: {}",
                transformation
            )))
        }
    };

    Ok(SimpleTransformResponse {
        result,
        transformation,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

pub fn create_upload_response(
    filename: String,
    size_bytes: u64,
) -> Result<SimpleUploadResponse, McpError> {
    const MAX_SIZE: u64 = 100 * 1024 * 1024; // 100MB

    if size_bytes > MAX_SIZE {
        return Err(McpError::InvalidParams(format!(
            "File size {} bytes exceeds maximum of {} bytes",
            size_bytes, MAX_SIZE
        )));
    }

    Ok(SimpleUploadResponse {
        status: "uploaded".to_string(),
        filename,
        size_bytes,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

pub fn create_health_response() -> SimpleHealthResponse {
    SimpleHealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}
