//! Error types for MCP server operations.

use mcp_core::McpError;
use mcp_transport::TransportError;
use thiserror::Error;

/// Server-specific error types
#[derive(Debug, Error)]
pub enum ServerError {
    /// MCP protocol error
    #[error("MCP error: {0}")]
    Mcp(#[from] McpError),

    /// Transport error
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),

    /// Tool registration error
    #[error("Tool registration error: {0}")]
    ToolRegistration(String),

    /// Tool not found error
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    /// Tool execution error
    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    /// Server initialization error
    #[error("Server initialization error: {0}")]
    Initialization(String),

    /// Server shutdown error
    #[error("Server shutdown error: {0}")]
    Shutdown(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Request handling error
    #[error("Request handling error: {0}")]
    RequestHandling(String),

    /// Resource management error
    #[error("Resource error: {0}")]
    Resource(String),

    /// Concurrency error
    #[error("Concurrency error: {0}")]
    Concurrency(String),

    /// Server is not running
    #[error("Server is not running")]
    NotRunning,

    /// Server is already running
    #[error("Server is already running")]
    AlreadyRunning,

    /// Invalid server state
    #[error("Invalid server state: {0}")]
    InvalidState(String),

    /// Generic server error
    #[error("Server error: {0}")]
    Other(String),
}

impl From<ServerError> for McpError {
    fn from(error: ServerError) -> Self {
        match error {
            ServerError::Mcp(mcp_error) => mcp_error,
            ServerError::ToolNotFound(name) => McpError::method_not_found(name),
            ServerError::ToolExecution(msg) => McpError::tool_error(msg),
            ServerError::Configuration(msg) => McpError::invalid_params(msg),
            ServerError::RequestHandling(msg) => McpError::invalid_request(msg),
            _ => McpError::internal_error(error.to_string()),
        }
    }
}

/// Result type for server operations
pub type ServerResult<T> = Result<T, ServerError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_error_display() {
        let error = ServerError::ToolNotFound("test_tool".to_string());
        assert_eq!(error.to_string(), "Tool not found: test_tool");
    }

    #[test]
    fn test_server_error_to_mcp_error() {
        let server_error = ServerError::ToolNotFound("missing_tool".to_string());
        let mcp_error: McpError = server_error.into();

        assert_eq!(mcp_error.code, mcp_core::McpErrorCode::MethodNotFound);
        assert_eq!(mcp_error.message, "Method 'missing_tool' not found");
    }

    #[test]
    fn test_error_chain() {
        let mcp_error = McpError::tool_error("Tool failed");
        let server_error = ServerError::Mcp(mcp_error);

        match server_error {
            ServerError::Mcp(inner) => {
                assert_eq!(inner.code, mcp_core::McpErrorCode::ToolError);
            }
            _ => panic!("Expected MCP error"),
        }
    }
}
