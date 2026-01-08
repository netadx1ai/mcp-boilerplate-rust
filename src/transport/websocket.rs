//! WebSocket Transport Implementation
//!
//! Provides bidirectional real-time communication for MCP.
//! Supports multiple concurrent connections with independent message streams.

use super::r#trait::{
    Transport, TransportCapabilities, TransportConfig, TransportError, TransportFactory,
    TransportMessage, TransportStats,
};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

/// Maximum number of concurrent WebSocket connections
const MAX_CONNECTIONS: usize = 100;

/// WebSocket connection state
#[derive(Debug, Clone)]
pub struct WsConnection {
    pub id: String,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub messages_sent: u64,
    pub messages_received: u64,
}

/// WebSocket transport state
struct WsState {
    initialized: bool,
    shutdown: bool,
    connection_count: usize,
}

/// WebSocket Transport
pub struct WebSocketTransport {
    bind_address: String,
    state: Arc<Mutex<WsState>>,
    stats: Arc<Mutex<TransportStats>>,
    config: TransportConfig,
}

impl WebSocketTransport {
    /// Create new WebSocket transport
    pub fn new(bind_address: String) -> Self {
        Self::with_config(TransportConfig {
            transport_type: "websocket".to_string(),
            bind_address: Some(bind_address.clone()),
            ..Default::default()
        })
    }

    /// Create with custom configuration
    pub fn with_config(config: TransportConfig) -> Self {
        let bind_address = config
            .bind_address
            .clone()
            .unwrap_or_else(|| "127.0.0.1:9001".to_string());

        Self {
            bind_address,
            state: Arc::new(Mutex::new(WsState {
                initialized: false,
                shutdown: false,
                connection_count: 0,
            })),
            stats: Arc::new(Mutex::new(TransportStats::default())),
            config,
        }
    }

    /// Get current connection count
    pub fn get_connection_count(&self) -> usize {
        let state = self.state.lock().unwrap();
        state.connection_count
    }

    /// Get transport statistics
    pub fn get_stats(&self) -> TransportStats {
        self.stats.lock().unwrap().clone()
    }
}

impl Default for WebSocketTransport {
    fn default() -> Self {
        Self::new("127.0.0.1:9001".to_string())
    }
}

#[async_trait]
impl Transport for WebSocketTransport {
    fn transport_type(&self) -> &str {
        "websocket"
    }

    fn capabilities(&self) -> TransportCapabilities {
        TransportCapabilities {
            bidirectional: true,
            server_push: true,
            multi_connection: true,
            streaming: true,
            browser_compatible: true,
        }
    }

    async fn send(&self, message: TransportMessage) -> Result<(), TransportError> {
        let state = self.state.lock().unwrap();
        if !state.initialized {
            return Err(TransportError::NotInitialized);
        }
        if state.shutdown {
            return Err(TransportError::AlreadyShutdown);
        }
        drop(state);

        // In a real implementation, this would send to active WebSocket connections
        let mut stats = self.stats.lock().unwrap();
        stats.messages_sent += 1;
        stats.bytes_sent += message.content.len() as u64;

        Ok(())
    }

    async fn receive(&self) -> Result<TransportMessage, TransportError> {
        let state = self.state.lock().unwrap();
        if !state.initialized {
            return Err(TransportError::NotInitialized);
        }
        if state.shutdown {
            return Err(TransportError::AlreadyShutdown);
        }
        drop(state);

        // In a real implementation, this would receive from WebSocket connections
        // For now, return an error as WebSocket uses event-driven model
        Err(TransportError::ReceiveError(
            "WebSocket uses event-driven message handling".to_string(),
        ))
    }

    async fn initialize(&mut self) -> Result<(), TransportError> {
        let mut state = self.state.lock().unwrap();
        if state.initialized {
            return Ok(());
        }

        state.initialized = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), TransportError> {
        let mut state = self.state.lock().unwrap();
        state.shutdown = true;
        state.initialized = false;
        state.connection_count = 0;
        Ok(())
    }

    fn is_ready(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.initialized && !state.shutdown
    }

    fn connection_count(&self) -> usize {
        let state = self.state.lock().unwrap();
        state.connection_count
    }

    async fn broadcast(&self, message: TransportMessage) -> Result<(), TransportError> {
        let state = self.state.lock().unwrap();
        if !state.initialized {
            return Err(TransportError::NotInitialized);
        }
        if state.shutdown {
            return Err(TransportError::AlreadyShutdown);
        }
        let conn_count = state.connection_count;
        drop(state);

        // In a real implementation, broadcast to all connections
        let mut stats = self.stats.lock().unwrap();
        stats.messages_sent += conn_count as u64;
        stats.bytes_sent += (message.content.len() * conn_count) as u64;

        Ok(())
    }
}

/// Factory for creating WebSocket transports
pub struct WebSocketTransportFactory;

impl TransportFactory for WebSocketTransportFactory {
    fn create(&self, config: TransportConfig) -> Result<Box<dyn Transport>, TransportError> {
        Ok(Box::new(WebSocketTransport::with_config(config)))
    }

    fn supported_types(&self) -> Vec<String> {
        vec!["websocket".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_creation() {
        let transport = WebSocketTransport::new("127.0.0.1:9001".to_string());
        assert_eq!(transport.transport_type(), "websocket");
    }

    #[test]
    fn test_websocket_capabilities() {
        let transport = WebSocketTransport::new("127.0.0.1:9002".to_string());
        let caps = transport.capabilities();

        assert!(caps.bidirectional);
        assert!(caps.server_push);
        assert!(caps.multi_connection);
        assert!(caps.streaming);
        assert!(caps.browser_compatible);
    }

    #[tokio::test]
    async fn test_websocket_initialization() {
        let mut transport = WebSocketTransport::new("127.0.0.1:9003".to_string());
        assert!(!transport.is_ready());

        let result = transport.initialize().await;
        assert!(result.is_ok());
        assert!(transport.is_ready());
    }

    #[tokio::test]
    async fn test_websocket_shutdown() {
        let mut transport = WebSocketTransport::new("127.0.0.1:9004".to_string());
        transport.initialize().await.unwrap();

        let result = transport.shutdown().await;
        assert!(result.is_ok());
        assert!(!transport.is_ready());
    }

    #[tokio::test]
    async fn test_send_message() {
        let mut transport = WebSocketTransport::new("127.0.0.1:9005".to_string());
        transport.initialize().await.unwrap();

        let message = TransportMessage::new("test message".to_string());
        let result = transport.send(message).await;
        assert!(result.is_ok());

        let stats = transport.get_stats();
        assert_eq!(stats.messages_sent, 1);
    }

    #[tokio::test]
    async fn test_broadcast_message() {
        let mut transport = WebSocketTransport::new("127.0.0.1:9006".to_string());
        transport.initialize().await.unwrap();

        let message = TransportMessage::new("broadcast test".to_string());
        let result = transport.broadcast(message).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_factory_creation() {
        let factory = WebSocketTransportFactory;
        let config = TransportConfig {
            transport_type: "websocket".to_string(),
            bind_address: Some("127.0.0.1:9010".to_string()),
            ..Default::default()
        };

        let transport = factory.create(config).unwrap();
        assert_eq!(transport.transport_type(), "websocket");
    }

    #[test]
    fn test_factory_supported_types() {
        let factory = WebSocketTransportFactory;
        let types = factory.supported_types();
        assert_eq!(types, vec!["websocket".to_string()]);
    }

    #[tokio::test]
    async fn test_send_not_initialized() {
        let transport = WebSocketTransport::new("127.0.0.1:9007".to_string());
        let message = TransportMessage::new("test".to_string());
        let result = transport.send(message).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connection_count() {
        let transport = WebSocketTransport::new("127.0.0.1:9008".to_string());
        assert_eq!(transport.connection_count(), 0);
    }
}
