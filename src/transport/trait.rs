//! Transport trait for MCP protocol communication
//!
//! This module defines the core Transport trait that all transport implementations
//! must implement. It provides a unified interface for different transport methods
//! (stdio, SSE, WebSocket, HTTP streaming, RPC).
//!
//! Note: Many types are defined for public API extensibility but may not be used internally.
#![allow(dead_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Generic message type for transport layer
/// Wraps JSON-RPC messages for transport-agnostic communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportMessage {
    /// JSON-RPC message content
    pub content: String,
    /// Optional metadata for routing/debugging
    pub metadata: Option<TransportMetadata>,
}

/// Metadata for transport messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportMetadata {
    /// Unique message ID
    pub id: Option<String>,
    /// Timestamp (RFC 3339 format)
    pub timestamp: String,
    /// Transport method used
    pub transport_type: String,
    /// Optional correlation ID for request/response tracking
    pub correlation_id: Option<String>,
}

impl TransportMessage {
    /// Create a new transport message from JSON content
    pub fn new(content: String) -> Self {
        Self {
            content,
            metadata: None,
        }
    }

    /// Create a message with metadata
    pub fn with_metadata(content: String, transport_type: impl Into<String>) -> Self {
        Self {
            content,
            metadata: Some(TransportMetadata {
                id: Some(uuid::Uuid::new_v4().to_string()),
                timestamp: chrono::Utc::now().to_rfc3339(),
                transport_type: transport_type.into(),
                correlation_id: None,
            }),
        }
    }

    /// Set correlation ID for request/response tracking
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        if let Some(ref mut metadata) = self.metadata {
            metadata.correlation_id = Some(correlation_id);
        }
        self
    }
}

/// Transport capability flags
#[derive(Debug, Clone)]
pub struct TransportCapabilities {
    /// Supports bidirectional communication
    pub bidirectional: bool,
    /// Supports server-to-client push
    pub server_push: bool,
    /// Supports multiple simultaneous connections
    pub multi_connection: bool,
    /// Supports streaming responses
    pub streaming: bool,
    /// Browser-compatible
    pub browser_compatible: bool,
}

/// Transport configuration
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Transport type name
    pub transport_type: String,
    /// Bind address (for network transports)
    pub bind_address: Option<String>,
    /// Port number (for network transports)
    pub port: Option<u16>,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Connection timeout in seconds
    pub timeout_seconds: u64,
    /// Enable TLS/SSL
    pub enable_tls: bool,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            transport_type: "stdio".to_string(),
            bind_address: Some("127.0.0.1".to_string()),
            port: Some(8025),
            max_message_size: 10 * 1024 * 1024, // 10MB
            timeout_seconds: 30,
            enable_tls: false,
        }
    }
}

/// Core Transport trait that all transport implementations must implement
///
/// This trait provides a unified interface for different transport methods:
/// - stdio: Standard input/output (CLI, process spawning)
/// - SSE: Server-Sent Events (browser push notifications)
/// - WebSocket: Bidirectional real-time communication
/// - HTTP Streaming: Large data transfers with chunked encoding
/// - RPC: Remote procedure calls (gRPC, custom protocols)
#[async_trait]
pub trait Transport: Send + Sync {
    /// Get transport type name (e.g., "stdio", "sse", "websocket")
    fn transport_type(&self) -> &str;

    /// Get transport capabilities
    fn capabilities(&self) -> TransportCapabilities;

    /// Send a message through this transport
    ///
    /// # Arguments
    /// * `message` - The message to send
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    async fn send(&self, message: TransportMessage) -> Result<(), TransportError>;

    /// Receive a message from this transport
    ///
    /// This method blocks until a message is available or an error occurs.
    ///
    /// # Returns
    /// * `Result<TransportMessage>` - The received message or error
    async fn receive(&self) -> Result<TransportMessage, TransportError>;

    /// Initialize the transport
    ///
    /// Called once before any send/receive operations.
    /// Use this to set up connections, bind ports, etc.
    async fn initialize(&mut self) -> Result<(), TransportError>;

    /// Shutdown the transport gracefully
    ///
    /// Close connections, flush buffers, cleanup resources.
    async fn shutdown(&mut self) -> Result<(), TransportError>;

    /// Check if transport is ready to send/receive
    fn is_ready(&self) -> bool;

    /// Get current connection count (for multi-connection transports)
    fn connection_count(&self) -> usize {
        1 // Default: single connection
    }

    /// Broadcast message to all connections (for multi-connection transports)
    ///
    /// Default implementation just calls send() once
    async fn broadcast(&self, message: TransportMessage) -> Result<(), TransportError> {
        self.send(message).await
    }
}

/// Transport-specific errors
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    /// Failed to send message
    #[error("Failed to send message: {0}")]
    SendError(String),

    /// Failed to receive message
    #[error("Failed to receive message: {0}")]
    ReceiveError(String),

    /// Transport not initialized
    #[error("Transport not initialized")]
    NotInitialized,

    /// Transport already shutdown
    #[error("Transport already shutdown")]
    AlreadyShutdown,

    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Timeout error
    #[error("Operation timed out after {0} seconds")]
    Timeout(u64),

    /// Message size exceeded
    #[error("Message size {actual} exceeds maximum {max} bytes")]
    MessageTooLarge { actual: usize, max: usize },

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Generic error
    #[error("Transport error: {0}")]
    Other(String),
}

/// Transport factory for creating transport instances
pub trait TransportFactory: Send + Sync {
    /// Create a new transport instance from configuration
    fn create(&self, config: TransportConfig) -> Result<Box<dyn Transport>, TransportError>;

    /// Get supported transport types
    fn supported_types(&self) -> Vec<String>;
}

/// Transport metadata for logging and monitoring
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct TransportStats {
    /// Total messages sent
    pub messages_sent: u64,
    /// Total messages received
    pub messages_received: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Number of errors encountered
    pub error_count: u64,
    /// Transport uptime in seconds
    pub uptime_seconds: u64,
}


impl fmt::Display for TransportStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Sent: {} msgs ({} bytes), Received: {} msgs ({} bytes), Errors: {}, Uptime: {}s",
            self.messages_sent,
            self.bytes_sent,
            self.messages_received,
            self.bytes_received,
            self.error_count,
            self.uptime_seconds
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_message_creation() {
        let msg = TransportMessage::new("test content".to_string());
        assert_eq!(msg.content, "test content");
        assert!(msg.metadata.is_none());
    }

    #[test]
    fn test_transport_message_with_metadata() {
        let msg = TransportMessage::with_metadata("test".to_string(), "stdio");
        assert_eq!(msg.content, "test");
        assert!(msg.metadata.is_some());

        let metadata = msg.metadata.unwrap();
        assert_eq!(metadata.transport_type, "stdio");
        assert!(metadata.id.is_some());
    }

    #[test]
    fn test_transport_config_default() {
        let config = TransportConfig::default();
        assert_eq!(config.transport_type, "stdio");
        assert_eq!(config.max_message_size, 10 * 1024 * 1024);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_transport_stats_display() {
        let stats = TransportStats {
            messages_sent: 100,
            messages_received: 95,
            bytes_sent: 10240,
            bytes_received: 9500,
            error_count: 2,
            uptime_seconds: 3600,
        };

        let display = format!("{stats}");
        assert!(display.contains("100 msgs"));
        assert!(display.contains("95 msgs"));
        assert!(display.contains("Errors: 2"));
    }
}
