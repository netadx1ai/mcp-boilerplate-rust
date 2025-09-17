//! Core transport trait and types for MCP communication.

use crate::error::{TransportError, TransportResult};
use async_trait::async_trait;
use mcp_core::{McpRequest, McpResponse};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for transport connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Connection timeout
    pub timeout: Duration,
    /// Keep-alive interval
    pub keep_alive: Option<Duration>,
    /// Enable compression
    pub compression: bool,
    /// Buffer size for message handling
    pub buffer_size: usize,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            max_message_size: 1024 * 1024, // 1MB
            timeout: Duration::from_secs(30),
            keep_alive: Some(Duration::from_secs(60)),
            compression: false,
            buffer_size: 8192, // 8KB
        }
    }
}

/// Transport layer abstraction for MCP communication
///
/// This trait defines the interface for different transport mechanisms
/// (STDIO, HTTP, WebSocket, etc.) used by MCP servers and clients.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send an MCP response through the transport
    ///
    /// # Arguments
    ///
    /// * `response` - The MCP response to send
    ///
    /// # Returns
    ///
    /// A result indicating success or transport-specific error
    async fn send_response(&self, response: McpResponse) -> TransportResult<()>;

    /// Receive an MCP request from the transport
    ///
    /// This method should block until a request is received or an error occurs.
    /// Returns `None` if the transport is gracefully closed.
    ///
    /// # Returns
    ///
    /// A result containing an optional MCP request or transport error
    async fn receive_request(&self) -> TransportResult<Option<McpRequest>>;

    /// Check if the transport is currently connected and ready for communication
    ///
    /// # Returns
    ///
    /// `true` if connected, `false` otherwise
    fn is_connected(&self) -> bool;

    /// Gracefully close the transport connection
    ///
    /// This should clean up any resources and ensure pending operations complete.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure of the close operation
    async fn close(&self) -> TransportResult<()>;

    /// Get the transport configuration
    ///
    /// # Returns
    ///
    /// The current transport configuration
    fn config(&self) -> &TransportConfig;

    /// Get transport-specific metadata
    ///
    /// This can include connection info, peer details, protocol version, etc.
    ///
    /// # Returns
    ///
    /// A map of metadata key-value pairs
    fn metadata(&self) -> std::collections::HashMap<String, String> {
        std::collections::HashMap::new()
    }

    /// Check if the transport supports bidirectional communication
    ///
    /// # Returns
    ///
    /// `true` if bidirectional, `false` for unidirectional transports
    fn is_bidirectional(&self) -> bool {
        true
    }

    /// Get the transport type identifier
    ///
    /// # Returns
    ///
    /// A string identifying the transport type (e.g., "stdio", "http", "websocket")
    fn transport_type(&self) -> &'static str;
}

/// Message wrapper for transport-level message handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportMessage {
    /// Message ID for correlation
    pub id: Option<String>,
    /// Message content
    pub content: MessageContent,
    /// Message metadata
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

/// Content types that can be transported
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MessageContent {
    /// MCP request
    Request(McpRequest),
    /// MCP response  
    Response(McpResponse),
    /// Control message
    Control(ControlMessage),
}

/// Control messages for transport management
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum ControlMessage {
    /// Ping message for keep-alive
    Ping { timestamp: u64 },
    /// Pong response to ping
    Pong { timestamp: u64 },
    /// Close connection gracefully
    Close { reason: Option<String> },
    /// Acknowledge message receipt
    Ack { message_id: String },
    /// Negotiate transport parameters
    Negotiate { parameters: TransportConfig },
}

/// Transport factory for creating transport instances
pub struct TransportFactory;

impl TransportFactory {
    /// Create a new transport from configuration
    ///
    /// # Arguments
    ///
    /// * `transport_type` - Type of transport to create
    /// * `config` - Transport configuration
    ///
    /// # Returns
    ///
    /// A result containing the transport instance or creation error
    pub async fn create_transport(
        transport_type: &str,
        config: TransportConfig,
    ) -> TransportResult<Box<dyn Transport>> {
        match transport_type {
            #[cfg(feature = "stdio")]
            "stdio" => {
                let transport = crate::stdio::StdioTransport::new(config)?;
                Ok(Box::new(transport))
            }
            #[cfg(feature = "http")]
            "http" => {
                use std::net::{IpAddr, Ipv4Addr, SocketAddr};
                let default_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000);
                let transport = crate::http::HttpTransport::new(default_addr, config)?;
                Ok(Box::new(transport))
            }
            _ => Err(TransportError::Configuration(format!(
                "Unknown transport type: {transport_type}"
            ))),
        }
    }

    /// List available transport types
    ///
    /// # Returns
    ///
    /// A vector of available transport type names
    #[allow(clippy::vec_init_then_push)]
    pub fn available_transports() -> Vec<&'static str> {
        let mut vec = Vec::new();
        
        #[cfg(feature = "stdio")]
        vec.push("stdio");
        
        #[cfg(feature = "http")]
        vec.push("http");
        
        vec
    }
}

/// Utility functions for transport implementations
pub mod utils {
    use super::*;

    /// Serialize a message to JSON bytes
    ///
    /// # Arguments
    ///
    /// * `message` - The message to serialize
    ///
    /// # Returns
    ///
    /// Serialized message bytes or error
    pub fn serialize_message(message: &TransportMessage) -> TransportResult<Vec<u8>> {
        serde_json::to_vec(message).map_err(TransportError::from)
    }

    /// Deserialize JSON bytes to a message
    ///
    /// # Arguments
    ///
    /// * `bytes` - The bytes to deserialize
    ///
    /// # Returns
    ///
    /// Deserialized message or error
    pub fn deserialize_message(bytes: &[u8]) -> TransportResult<TransportMessage> {
        serde_json::from_slice(bytes).map_err(TransportError::from)
    }

    /// Validate message size against configuration
    ///
    /// # Arguments
    ///
    /// * `message_size` - Size of the message in bytes
    /// * `config` - Transport configuration
    ///
    /// # Returns
    ///
    /// Ok if valid, error if message is too large
    pub fn validate_message_size(
        message_size: usize,
        config: &TransportConfig,
    ) -> TransportResult<()> {
        if message_size > config.max_message_size {
            Err(TransportError::MessageTooLarge(
                message_size,
                config.max_message_size,
            ))
        } else {
            Ok(())
        }
    }

    /// Create a ping control message
    ///
    /// # Returns
    ///
    /// A transport message containing a ping
    pub fn create_ping() -> TransportMessage {
        TransportMessage {
            id: Some(uuid::Uuid::new_v4().to_string()),
            content: MessageContent::Control(ControlMessage::Ping {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            }),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create a pong response message
    ///
    /// # Arguments
    ///
    /// * `ping_timestamp` - Timestamp from the original ping
    ///
    /// # Returns
    ///
    /// A transport message containing a pong response
    pub fn create_pong(ping_timestamp: u64) -> TransportMessage {
        TransportMessage {
            id: Some(uuid::Uuid::new_v4().to_string()),
            content: MessageContent::Control(ControlMessage::Pong {
                timestamp: ping_timestamp,
            }),
            metadata: std::collections::HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcp_core::McpRequest;

    #[test]
    fn test_transport_config_default() {
        let config = TransportConfig::default();
        assert_eq!(config.max_message_size, 1024 * 1024);
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.buffer_size, 8192);
        assert!(!config.compression);
    }

    #[test]
    fn test_message_serialization() {
        let message = TransportMessage {
            id: Some("test-123".to_string()),
            content: MessageContent::Request(McpRequest::Ping),
            metadata: std::collections::HashMap::new(),
        };

        let serialized = utils::serialize_message(&message).unwrap();
        let deserialized = utils::deserialize_message(&serialized).unwrap();

        assert_eq!(message.id, deserialized.id);
        match (&message.content, &deserialized.content) {
            (MessageContent::Request(req1), MessageContent::Request(req2)) => {
                assert_eq!(req1, req2);
            }
            _ => panic!("Message content mismatch"),
        }
    }

    #[test]
    fn test_message_size_validation() {
        let config = TransportConfig {
            max_message_size: 1000,
            ..Default::default()
        };

        assert!(utils::validate_message_size(500, &config).is_ok());
        assert!(utils::validate_message_size(1500, &config).is_err());
    }

    #[test]
    fn test_control_message_creation() {
        let ping = utils::create_ping();
        match ping.content {
            MessageContent::Control(ControlMessage::Ping { timestamp }) => {
                assert!(timestamp > 0);
            }
            _ => panic!("Expected ping message"),
        }

        let pong = utils::create_pong(12345);
        match pong.content {
            MessageContent::Control(ControlMessage::Pong { timestamp }) => {
                assert_eq!(timestamp, 12345);
            }
            _ => panic!("Expected pong message"),
        }
    }

    #[test]
    fn test_transport_factory_available_transports() {
        let transports = TransportFactory::available_transports();
        assert!(!transports.is_empty());

        #[cfg(feature = "stdio")]
        assert!(transports.contains(&"stdio"));

        #[cfg(feature = "http")]
        assert!(transports.contains(&"http"));
    }
}
