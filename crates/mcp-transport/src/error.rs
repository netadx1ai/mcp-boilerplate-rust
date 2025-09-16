//! Error types for MCP transport operations.

use mcp_core::McpError;
use thiserror::Error;

/// Transport-specific error types
#[derive(Debug, Error)]
pub enum TransportError {
    /// IO error occurred during transport operations
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// HTTP-specific error
    #[error("HTTP error: {0}")]
    Http(String),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Protocol error
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Timeout error
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Transport is closed
    #[error("Transport is closed")]
    Closed,

    /// Invalid message format
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    /// Message too large
    #[error("Message too large: {0} bytes (max: {1})")]
    MessageTooLarge(usize, usize),

    /// Transport not connected
    #[error("Transport not connected")]
    NotConnected,

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Generic transport error
    #[error("Transport error: {0}")]
    Other(String),
}

impl From<TransportError> for McpError {
    fn from(error: TransportError) -> Self {
        match error {
            TransportError::Json(_) => McpError::parse_error(error.to_string()),
            TransportError::InvalidMessage(_) => McpError::invalid_request(error.to_string()),
            TransportError::Protocol(_) => McpError::invalid_request(error.to_string()),
            TransportError::AuthenticationFailed(_) => {
                McpError::permission_denied(error.to_string())
            }
            _ => McpError::internal_error(error.to_string()),
        }
    }
}

/// Result type for transport operations
pub type TransportResult<T> = Result<T, TransportError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_error_display() {
        let error = TransportError::Connection("Failed to connect".to_string());
        assert_eq!(error.to_string(), "Connection error: Failed to connect");
    }

    #[test]
    fn test_transport_error_to_mcp_error() {
        let transport_error = TransportError::Json(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "test",
        )));
        let mcp_error: McpError = transport_error.into();

        assert_eq!(mcp_error.code, mcp_core::McpErrorCode::ParseError);
    }

    #[test]
    fn test_message_too_large_error() {
        let error = TransportError::MessageTooLarge(2048, 1024);
        assert_eq!(
            error.to_string(),
            "Message too large: 2048 bytes (max: 1024)"
        );
    }
}
