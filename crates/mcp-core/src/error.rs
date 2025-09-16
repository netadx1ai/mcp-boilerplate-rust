//! Error types for MCP Core operations.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Standard MCP error codes based on JSON-RPC 2.0 specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum McpErrorCode {
    /// Invalid JSON was received by the server
    ParseError = -32700,
    /// The JSON sent is not a valid Request object
    InvalidRequest = -32600,
    /// The method does not exist / is not available
    MethodNotFound = -32601,
    /// Invalid method parameter(s)
    InvalidParams = -32602,
    /// Internal JSON-RPC error
    InternalError = -32603,
    /// Tool execution failed
    ToolError = -32000,
    /// Resource not found
    ResourceNotFound = -32001,
    /// Permission denied
    PermissionDenied = -32002,
    /// Rate limit exceeded
    RateLimitExceeded = -32003,
    /// Server overloaded
    ServerOverloaded = -32004,
}

impl fmt::Display for McpErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            McpErrorCode::ParseError => write!(f, "Parse error"),
            McpErrorCode::InvalidRequest => write!(f, "Invalid Request"),
            McpErrorCode::MethodNotFound => write!(f, "Method not found"),
            McpErrorCode::InvalidParams => write!(f, "Invalid params"),
            McpErrorCode::InternalError => write!(f, "Internal error"),
            McpErrorCode::ToolError => write!(f, "Tool error"),
            McpErrorCode::ResourceNotFound => write!(f, "Resource not found"),
            McpErrorCode::PermissionDenied => write!(f, "Permission denied"),
            McpErrorCode::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            McpErrorCode::ServerOverloaded => write!(f, "Server overloaded"),
        }
    }
}

/// MCP protocol error
#[derive(Debug, Clone, Serialize, Deserialize, Error, PartialEq)]
pub struct McpError {
    /// Error code
    pub code: McpErrorCode,
    /// Human-readable error message
    pub message: String,
    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl fmt::Display for McpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MCP Error {}: {}", self.code as i32, self.message)
    }
}

impl McpError {
    /// Create a new MCP error
    pub fn new(code: McpErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Create a new MCP error with additional data
    pub fn with_data(
        code: McpErrorCode,
        message: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }

    /// Create a parse error
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::new(McpErrorCode::ParseError, message)
    }

    /// Create an invalid request error
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(McpErrorCode::InvalidRequest, message)
    }

    /// Create a method not found error
    pub fn method_not_found(method: impl Into<String>) -> Self {
        Self::new(
            McpErrorCode::MethodNotFound,
            format!("Method '{}' not found", method.into()),
        )
    }

    /// Create an invalid params error
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(McpErrorCode::InvalidParams, message)
    }

    /// Create an internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(McpErrorCode::InternalError, message)
    }

    /// Create a tool error
    pub fn tool_error(message: impl Into<String>) -> Self {
        Self::new(McpErrorCode::ToolError, message)
    }

    /// Create a resource not found error
    pub fn resource_not_found(resource: impl Into<String>) -> Self {
        Self::new(
            McpErrorCode::ResourceNotFound,
            format!("Resource '{}' not found", resource.into()),
        )
    }

    /// Create a permission denied error
    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::new(McpErrorCode::PermissionDenied, message)
    }

    /// Create a rate limit exceeded error
    pub fn rate_limit_exceeded(message: impl Into<String>) -> Self {
        Self::new(McpErrorCode::RateLimitExceeded, message)
    }

    /// Create a server overloaded error
    pub fn server_overloaded(message: impl Into<String>) -> Self {
        Self::new(McpErrorCode::ServerOverloaded, message)
    }
}

impl From<serde_json::Error> for McpError {
    fn from(err: serde_json::Error) -> Self {
        McpError::parse_error(err.to_string())
    }
}

impl From<std::io::Error> for McpError {
    fn from(err: std::io::Error) -> Self {
        McpError::internal_error(format!("IO error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_display() {
        assert_eq!(McpErrorCode::ParseError.to_string(), "Parse error");
        assert_eq!(McpErrorCode::MethodNotFound.to_string(), "Method not found");
    }

    #[test]
    fn test_error_creation() {
        let error = McpError::method_not_found("test_method");
        assert_eq!(error.code, McpErrorCode::MethodNotFound);
        assert_eq!(error.message, "Method 'test_method' not found");
        assert!(error.data.is_none());
    }

    #[test]
    fn test_error_with_data() {
        let data = serde_json::json!({"extra": "info"});
        let error = McpError::with_data(McpErrorCode::ToolError, "Test error", data.clone());
        assert_eq!(error.code, McpErrorCode::ToolError);
        assert_eq!(error.message, "Test error");
        assert_eq!(error.data, Some(data));
    }

    #[test]
    fn test_error_serialization() {
        let error = McpError::invalid_params("Missing required parameter");
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: McpError = serde_json::from_str(&json).unwrap();
        
        assert_eq!(error.code, deserialized.code);
        assert_eq!(error.message, deserialized.message);
        assert_eq!(error.data, deserialized.data);
    }
}