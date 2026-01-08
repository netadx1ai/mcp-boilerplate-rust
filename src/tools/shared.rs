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
