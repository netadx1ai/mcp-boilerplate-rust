//! SSE (Server-Sent Events) transport implementation for MCP protocol
//! 
//! This transport provides server-to-client push notifications using SSE,
//! making it ideal for:
//! - Real-time progress updates
//! - Browser-based clients
//! - Live notifications
//! - Multi-client broadcasting
//! 
//! # Features
//! 
//! - One-way server → client communication
//! - Multiple simultaneous clients
//! - Automatic reconnection support
//! - Browser EventSource API compatible
//! - Efficient event streaming
//! 
//! # Example
//! 
//! ```rust,ignore
//! use mcp_boilerplate_rust::transport::{SseTransport, Transport};
//! 
//! let mut transport = SseTransport::new("127.0.0.1:8025");
//! transport.initialize().await?;
//! 
//! // Broadcast to all connected clients
//! transport.broadcast(progress_update).await?;
//! ```

#[cfg(feature = "sse")]
use super::r#trait::{
    Transport, TransportCapabilities, TransportConfig, TransportError,
    TransportFactory, TransportMessage, TransportStats,
};

#[cfg(feature = "sse")]
use async_trait::async_trait;

#[cfg(feature = "sse")]
use axum::response::sse::{Event, KeepAlive, Sse};

#[cfg(feature = "sse")]
use futures::stream::{Stream, StreamExt};

#[cfg(feature = "sse")]
use std::collections::HashMap;

#[cfg(feature = "sse")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "sse")]
use tokio::sync::broadcast;

#[cfg(feature = "sse")]
type ClientId = String;

/// SSE transport implementation
/// 
/// Manages multiple SSE connections and broadcasts events to all connected clients.
/// Uses Axum's SSE support for efficient event streaming.
#[cfg(feature = "sse")]
pub struct SseTransport {
    /// Bind address
    bind_address: String,
    /// Broadcast channel for sending events to all clients
    tx: Arc<Mutex<Option<broadcast::Sender<TransportMessage>>>>,
    /// Connected clients tracking
    clients: Arc<Mutex<HashMap<ClientId, ClientInfo>>>,
    /// Transport state
    state: Arc<Mutex<SseState>>,
    /// Statistics tracking
    stats: Arc<Mutex<TransportStats>>,
    /// Configuration
    config: TransportConfig,
}

#[cfg(feature = "sse")]
struct SseState {
    initialized: bool,
    shutdown: bool,
}

#[cfg(feature = "sse")]
struct ClientInfo {
    id: ClientId,
    connected_at: chrono::DateTime<chrono::Utc>,
    user_agent: Option<String>,
}

#[cfg(feature = "sse")]
impl SseTransport {
    /// Create a new SSE transport
    pub fn new(bind_address: impl Into<String>) -> Self {
        let bind_address = bind_address.into();
        Self::with_config(TransportConfig {
            transport_type: "sse".to_string(),
            bind_address: Some(bind_address.clone()),
            ..Default::default()
        })
    }

    /// Create with custom configuration
    pub fn with_config(config: TransportConfig) -> Self {
        let bind_address = config
            .bind_address
            .clone()
            .unwrap_or_else(|| "127.0.0.1:8025".to_string());

        Self {
            bind_address,
            tx: Arc::new(Mutex::new(None)),
            clients: Arc::new(Mutex::new(HashMap::new())),
            state: Arc::new(Mutex::new(SseState {
                initialized: false,
                shutdown: false,
            })),
            stats: Arc::new(Mutex::new(TransportStats::default())),
            config,
        }
    }

    /// Get transport statistics
    pub fn stats(&self) -> TransportStats {
        self.stats.lock().unwrap().clone()
    }

    /// Register a new client connection
    pub fn register_client(&self, client_id: ClientId, user_agent: Option<String>) {
        let mut clients = self.clients.lock().unwrap();
        clients.insert(
            client_id.clone(),
            ClientInfo {
                id: client_id,
                connected_at: chrono::Utc::now(),
                user_agent,
            },
        );
    }

    /// Unregister a client connection
    pub fn unregister_client(&self, client_id: &str) {
        let mut clients = self.clients.lock().unwrap();
        clients.remove(client_id);
    }

    /// Create an SSE stream for a client
    pub fn create_stream(&self) -> Result<broadcast::Receiver<TransportMessage>, TransportError> {
        let tx = self.tx.lock().unwrap();
        let sender = tx
            .as_ref()
            .ok_or_else(|| TransportError::NotInitialized)?
            .clone();

        Ok(sender.subscribe())
    }

    /// Send an event to all connected clients
    pub async fn send_event(&self, message: TransportMessage) -> Result<(), TransportError> {
        let tx = self.tx.lock().unwrap();
        let sender = tx
            .as_ref()
            .ok_or_else(|| TransportError::NotInitialized)?;

        sender
            .send(message.clone())
            .map_err(|e| TransportError::SendError(format!("Broadcast failed: {}", e)))?;

        // Update stats
        {
            let mut stats = self.stats.lock().unwrap();
            stats.messages_sent += 1;
            stats.bytes_sent += message.content.len() as u64;
        }

        Ok(())
    }

    /// Get current client count
    pub fn client_count(&self) -> usize {
        self.clients.lock().unwrap().len()
    }

    /// Get list of connected client IDs
    pub fn connected_clients(&self) -> Vec<ClientId> {
        self.clients
            .lock()
            .unwrap()
            .keys()
            .cloned()
            .collect()
    }
}

#[cfg(feature = "sse")]
impl Default for SseTransport {
    fn default() -> Self {
        Self::new("127.0.0.1:8025")
    }
}

#[cfg(feature = "sse")]
#[async_trait]
impl Transport for SseTransport {
    fn transport_type(&self) -> &str {
        "sse"
    }

    fn capabilities(&self) -> TransportCapabilities {
        TransportCapabilities {
            bidirectional: false, // SSE is one-way only (server → client)
            server_push: true,
            multi_connection: true,
            streaming: true,
            browser_compatible: true,
        }
    }

    async fn send(&self, message: TransportMessage) -> Result<(), TransportError> {
        self.send_event(message).await
    }

    async fn receive(&self) -> Result<TransportMessage, TransportError> {
        // SSE is server → client only, receiving not supported
        Err(TransportError::Other(
            "SSE transport does not support receiving messages (server → client only)".to_string(),
        ))
    }

    async fn initialize(&mut self) -> Result<(), TransportError> {
        let mut state = self.state.lock().unwrap();
        if state.initialized {
            return Ok(());
        }

        // Create broadcast channel with buffer size based on config
        let channel_size = 1000; // Buffer for 1000 messages
        let (tx, _rx) = broadcast::channel(channel_size);

        *self.tx.lock().unwrap() = Some(tx);

        state.initialized = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), TransportError> {
        let mut state = self.state.lock().unwrap();
        if state.shutdown {
            return Ok(());
        }

        // Clear broadcast channel
        *self.tx.lock().unwrap() = None;

        // Clear all clients
        self.clients.lock().unwrap().clear();

        state.shutdown = true;
        state.initialized = false;
        Ok(())
    }

    fn is_ready(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.initialized && !state.shutdown
    }

    fn connection_count(&self) -> usize {
        self.client_count()
    }

    async fn broadcast(&self, message: TransportMessage) -> Result<(), TransportError> {
        self.send_event(message).await
    }
}

/// Factory for creating SSE transport instances
#[cfg(feature = "sse")]
pub struct SseTransportFactory;

#[cfg(feature = "sse")]
impl TransportFactory for SseTransportFactory {
    fn create(&self, config: TransportConfig) -> Result<Box<dyn Transport>, TransportError> {
        if config.transport_type != "sse" {
            return Err(TransportError::ConfigError(format!(
                "Invalid transport type '{}' for SSE factory",
                config.transport_type
            )));
        }

        Ok(Box::new(SseTransport::with_config(config)))
    }

    fn supported_types(&self) -> Vec<String> {
        vec!["sse".to_string()]
    }
}

#[cfg(all(test, feature = "sse"))]
mod tests {
    use super::*;

    #[test]
    fn test_sse_transport_creation() {
        let transport = SseTransport::new("127.0.0.1:8025");
        assert_eq!(transport.transport_type(), "sse");
        assert!(!transport.is_ready());
    }

    #[test]
    fn test_sse_capabilities() {
        let transport = SseTransport::new("127.0.0.1:8025");
        let caps = transport.capabilities();
        assert!(!caps.bidirectional); // SSE is one-way
        assert!(caps.server_push);
        assert!(caps.multi_connection);
        assert!(caps.streaming);
        assert!(caps.browser_compatible);
    }

    #[tokio::test]
    async fn test_sse_initialization() {
        let mut transport = SseTransport::new("127.0.0.1:8025");
        assert!(!transport.is_ready());

        transport.initialize().await.unwrap();
        assert!(transport.is_ready());

        // Double initialization should be ok
        transport.initialize().await.unwrap();
        assert!(transport.is_ready());
    }

    #[tokio::test]
    async fn test_sse_shutdown() {
        let mut transport = SseTransport::new("127.0.0.1:8025");
        transport.initialize().await.unwrap();
        assert!(transport.is_ready());

        transport.shutdown().await.unwrap();
        assert!(!transport.is_ready());
    }

    #[tokio::test]
    async fn test_sse_receive_not_supported() {
        let mut transport = SseTransport::new("127.0.0.1:8025");
        transport.initialize().await.unwrap();

        let result = transport.receive().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_client_tracking() {
        let transport = SseTransport::new("127.0.0.1:8025");

        assert_eq!(transport.client_count(), 0);

        transport.register_client("client1".to_string(), Some("Mozilla/5.0".to_string()));
        assert_eq!(transport.client_count(), 1);

        transport.register_client("client2".to_string(), None);
        assert_eq!(transport.client_count(), 2);

        transport.unregister_client("client1");
        assert_eq!(transport.client_count(), 1);

        let clients = transport.connected_clients();
        assert_eq!(clients.len(), 1);
        assert!(clients.contains(&"client2".to_string()));
    }

    #[tokio::test]
    async fn test_broadcast() {
        let mut transport = SseTransport::new("127.0.0.1:8025");
        transport.initialize().await.unwrap();

        let message = TransportMessage::with_metadata("test event".to_string(), "sse");
        let result = transport.broadcast(message).await;
        assert!(result.is_ok());

        let stats = transport.stats();
        assert_eq!(stats.messages_sent, 1);
    }

    #[test]
    fn test_factory_creation() {
        let factory = SseTransportFactory;
        let config = TransportConfig {
            transport_type: "sse".to_string(),
            bind_address: Some("127.0.0.1:8025".to_string()),
            ..Default::default()
        };

        let transport = factory.create(config).unwrap();
        assert_eq!(transport.transport_type(), "sse");
    }

    #[test]
    fn test_factory_invalid_type() {
        let factory = SseTransportFactory;
        let config = TransportConfig {
            transport_type: "invalid".to_string(),
            ..Default::default()
        };

        let result = factory.create(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_stats_tracking() {
        let transport = SseTransport::new("127.0.0.1:8025");
        let stats = transport.stats();
        assert_eq!(stats.messages_sent, 0);
        assert_eq!(stats.bytes_sent, 0);
    }
}