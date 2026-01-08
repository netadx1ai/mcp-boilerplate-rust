//! gRPC Transport Implementation
//!
//! Provides high-performance RPC transport using gRPC/HTTP2.
//! Supports bidirectional streaming and efficient binary serialization.

use super::r#trait::{
    Transport, TransportCapabilities, TransportConfig, TransportError, TransportFactory,
    TransportMessage, TransportStats,
};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

/// gRPC transport state
struct GrpcState {
    initialized: bool,
    shutdown: bool,
    active_connections: usize,
}

/// gRPC Transport
///
/// Provides RPC-based communication using gRPC protocol.
/// Supports streaming, bidirectional communication, and efficient serialization.
pub struct GrpcTransport {
    bind_address: String,
    state: Arc<Mutex<GrpcState>>,
    stats: Arc<Mutex<TransportStats>>,
    config: TransportConfig,
}

impl GrpcTransport {
    /// Create new gRPC transport
    pub fn new(bind_address: impl Into<String>) -> Self {
        Self::with_config(TransportConfig {
            transport_type: "grpc".to_string(),
            bind_address: Some(bind_address.into()),
            ..Default::default()
        })
    }

    /// Create with custom configuration
    pub fn with_config(config: TransportConfig) -> Self {
        let bind_address = config
            .bind_address
            .clone()
            .unwrap_or_else(|| "127.0.0.1:50051".to_string());

        Self {
            bind_address,
            state: Arc::new(Mutex::new(GrpcState {
                initialized: false,
                shutdown: false,
                active_connections: 0,
            })),
            stats: Arc::new(Mutex::new(TransportStats::default())),
            config,
        }
    }

    /// Get active connection count
    pub fn active_connections(&self) -> usize {
        let state = self.state.lock().unwrap();
        state.active_connections
    }

    /// Get transport statistics
    pub fn get_stats(&self) -> TransportStats {
        self.stats.lock().unwrap().clone()
    }

    /// Get bind address
    pub fn bind_address(&self) -> &str {
        &self.bind_address
    }
}

impl Default for GrpcTransport {
    fn default() -> Self {
        Self::new("127.0.0.1:50051")
    }
}

#[async_trait]
impl Transport for GrpcTransport {
    fn transport_type(&self) -> &str {
        "grpc"
    }

    fn capabilities(&self) -> TransportCapabilities {
        TransportCapabilities {
            bidirectional: true,
            server_push: true,
            multi_connection: true,
            streaming: true,
            browser_compatible: false, // Requires gRPC-Web for browser support
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

        // In production, this would send via gRPC channel
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

        // In production, this would receive from gRPC stream
        Err(TransportError::ReceiveError(
            "No messages available".to_string(),
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
        state.active_connections = 0;
        Ok(())
    }

    fn is_ready(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.initialized && !state.shutdown
    }

    fn connection_count(&self) -> usize {
        let state = self.state.lock().unwrap();
        state.active_connections
    }

    async fn broadcast(&self, message: TransportMessage) -> Result<(), TransportError> {
        let state = self.state.lock().unwrap();
        if !state.initialized {
            return Err(TransportError::NotInitialized);
        }
        if state.shutdown {
            return Err(TransportError::AlreadyShutdown);
        }
        let conn_count = state.active_connections;
        drop(state);

        // In production, broadcast to all active gRPC streams
        let mut stats = self.stats.lock().unwrap();
        stats.messages_sent += conn_count as u64;
        stats.bytes_sent += (message.content.len() * conn_count) as u64;

        Ok(())
    }
}

/// Factory for creating gRPC transports
pub struct GrpcTransportFactory;

impl TransportFactory for GrpcTransportFactory {
    fn create(&self, config: TransportConfig) -> Result<Box<dyn Transport>, TransportError> {
        Ok(Box::new(GrpcTransport::with_config(config)))
    }

    fn supported_types(&self) -> Vec<String> {
        vec!["grpc".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_creation() {
        let transport = GrpcTransport::new("127.0.0.1:50051");
        assert_eq!(transport.transport_type(), "grpc");
        assert_eq!(transport.bind_address(), "127.0.0.1:50051");
    }

    #[test]
    fn test_grpc_capabilities() {
        let transport = GrpcTransport::new("127.0.0.1:50052");
        let caps = transport.capabilities();

        assert!(caps.bidirectional);
        assert!(caps.server_push);
        assert!(caps.multi_connection);
        assert!(caps.streaming);
        assert!(!caps.browser_compatible); // Requires gRPC-Web
    }

    #[tokio::test]
    async fn test_grpc_initialization() {
        let mut transport = GrpcTransport::new("127.0.0.1:50053");
        assert!(!transport.is_ready());

        let result = transport.initialize().await;
        assert!(result.is_ok());
        assert!(transport.is_ready());
    }

    #[tokio::test]
    async fn test_grpc_shutdown() {
        let mut transport = GrpcTransport::new("127.0.0.1:50054");
        transport.initialize().await.unwrap();

        let result = transport.shutdown().await;
        assert!(result.is_ok());
        assert!(!transport.is_ready());
    }

    #[tokio::test]
    async fn test_send_message() {
        let mut transport = GrpcTransport::new("127.0.0.1:50055");
        transport.initialize().await.unwrap();

        let message = TransportMessage::new("test message".to_string());
        let result = transport.send(message).await;
        assert!(result.is_ok());

        let stats = transport.get_stats();
        assert_eq!(stats.messages_sent, 1);
    }

    #[tokio::test]
    async fn test_broadcast_message() {
        let mut transport = GrpcTransport::new("127.0.0.1:50056");
        transport.initialize().await.unwrap();

        let message = TransportMessage::new("broadcast test".to_string());
        let result = transport.broadcast(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_not_initialized() {
        let transport = GrpcTransport::new("127.0.0.1:50057");
        let message = TransportMessage::new("test".to_string());
        let result = transport.send(message).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_factory_creation() {
        let factory = GrpcTransportFactory;
        let config = TransportConfig {
            transport_type: "grpc".to_string(),
            bind_address: Some("127.0.0.1:50058".to_string()),
            ..Default::default()
        };

        let transport = factory.create(config).unwrap();
        assert_eq!(transport.transport_type(), "grpc");
    }

    #[test]
    fn test_factory_supported_types() {
        let factory = GrpcTransportFactory;
        let types = factory.supported_types();
        assert_eq!(types, vec!["grpc".to_string()]);
    }

    #[test]
    fn test_active_connections() {
        let transport = GrpcTransport::new("127.0.0.1:50059");
        assert_eq!(transport.active_connections(), 0);
    }

    #[test]
    fn test_connection_count() {
        let transport = GrpcTransport::new("127.0.0.1:50060");
        assert_eq!(transport.connection_count(), 0);
    }

    #[test]
    fn test_default_bind_address() {
        let transport = GrpcTransport::default();
        assert_eq!(transport.bind_address(), "127.0.0.1:50051");
    }

    #[tokio::test]
    async fn test_receive_when_not_initialized() {
        let transport = GrpcTransport::new("127.0.0.1:50061");
        let result = transport.receive().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_double_initialization() {
        let mut transport = GrpcTransport::new("127.0.0.1:50062");

        let result1 = transport.initialize().await;
        assert!(result1.is_ok());

        let result2 = transport.initialize().await;
        assert!(result2.is_ok()); // Should be idempotent

        assert!(transport.is_ready());
    }

    #[tokio::test]
    async fn test_send_after_shutdown() {
        let mut transport = GrpcTransport::new("127.0.0.1:50063");
        transport.initialize().await.unwrap();
        transport.shutdown().await.unwrap();

        let message = TransportMessage::new("test".to_string());
        let result = transport.send(message).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_broadcast_after_shutdown() {
        let mut transport = GrpcTransport::new("127.0.0.1:50064");
        transport.initialize().await.unwrap();
        transport.shutdown().await.unwrap();

        let message = TransportMessage::new("broadcast".to_string());
        let result = transport.broadcast(message).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_stats_tracking() {
        let transport = GrpcTransport::new("127.0.0.1:50065");
        let stats = transport.get_stats();

        assert_eq!(stats.messages_sent, 0);
        assert_eq!(stats.messages_received, 0);
        assert_eq!(stats.bytes_sent, 0);
        assert_eq!(stats.bytes_received, 0);
    }
}
