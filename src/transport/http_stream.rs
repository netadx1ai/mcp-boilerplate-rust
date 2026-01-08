//! HTTP Streaming Transport Implementation
//! 
//! Provides chunked transfer encoding for large data transfers.
//! Supports server-sent streaming for progressive data delivery.

use super::r#trait::{Transport, TransportCapabilities, TransportConfig, TransportError, TransportFactory, TransportMessage, TransportStats};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

/// HTTP streaming transport configuration
const CHUNK_SIZE: usize = 8192; // 8KB chunks

/// HTTP streaming transport state
struct HttpStreamState {
    initialized: bool,
    shutdown: bool,
    active_streams: usize,
}

/// HTTP Streaming Transport
pub struct HttpStreamTransport {
    bind_address: String,
    state: Arc<Mutex<HttpStreamState>>,
    stats: Arc<Mutex<TransportStats>>,
    config: TransportConfig,
}

impl HttpStreamTransport {
    /// Create new HTTP streaming transport
    pub fn new(bind_address: String) -> Self {
        Self::with_config(TransportConfig {
            transport_type: "http-stream".to_string(),
            bind_address: Some(bind_address.clone()),
            ..Default::default()
        })
    }

    /// Create with custom configuration
    pub fn with_config(config: TransportConfig) -> Self {
        let bind_address = config.bind_address.clone().unwrap_or_else(|| "127.0.0.1:8026".to_string());
        
        Self {
            bind_address,
            state: Arc::new(Mutex::new(HttpStreamState {
                initialized: false,
                shutdown: false,
                active_streams: 0,
            })),
            stats: Arc::new(Mutex::new(TransportStats::default())),
            config,
        }
    }

    /// Get active stream count
    pub fn active_streams(&self) -> usize {
        let state = self.state.lock().unwrap();
        state.active_streams
    }

    /// Get transport statistics
    pub fn get_stats(&self) -> TransportStats {
        self.stats.lock().unwrap().clone()
    }

    /// Create chunked stream from data
    pub fn create_chunks(&self, data: Vec<u8>) -> Vec<Vec<u8>> {
        data.chunks(CHUNK_SIZE)
            .map(|chunk| chunk.to_vec())
            .collect()
    }
}

impl Default for HttpStreamTransport {
    fn default() -> Self {
        Self::new("127.0.0.1:8026".to_string())
    }
}

#[async_trait]
impl Transport for HttpStreamTransport {
    fn transport_type(&self) -> &str {
        "http-stream"
    }

    fn capabilities(&self) -> TransportCapabilities {
        TransportCapabilities {
            bidirectional: false,
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

        // In a real implementation, this would stream chunks over HTTP
        let mut stats = self.stats.lock().unwrap();
        stats.messages_sent += 1;
        stats.bytes_sent += message.content.len() as u64;

        Ok(())
    }

    async fn receive(&self) -> Result<TransportMessage, TransportError> {
        // HTTP streaming is send-only (server to client)
        Err(TransportError::ReceiveError(
            "HTTP streaming is send-only. Use HTTP requests for bidirectional communication.".to_string()
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
        state.active_streams = 0;
        Ok(())
    }

    fn is_ready(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.initialized && !state.shutdown
    }

    fn connection_count(&self) -> usize {
        let state = self.state.lock().unwrap();
        state.active_streams
    }

    async fn broadcast(&self, message: TransportMessage) -> Result<(), TransportError> {
        let state = self.state.lock().unwrap();
        if !state.initialized {
            return Err(TransportError::NotInitialized);
        }
        if state.shutdown {
            return Err(TransportError::AlreadyShutdown);
        }
        let stream_count = state.active_streams;
        drop(state);

        // In a real implementation, broadcast to all active streams
        let mut stats = self.stats.lock().unwrap();
        stats.messages_sent += stream_count as u64;
        stats.bytes_sent += (message.content.len() * stream_count) as u64;

        Ok(())
    }
}

/// Factory for creating HTTP streaming transports
pub struct HttpStreamTransportFactory;

impl TransportFactory for HttpStreamTransportFactory {
    fn create(&self, config: TransportConfig) -> Result<Box<dyn Transport>, TransportError> {
        Ok(Box::new(HttpStreamTransport::with_config(config)))
    }

    fn supported_types(&self) -> Vec<String> {
        vec!["http-stream".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_stream_creation() {
        let transport = HttpStreamTransport::new("127.0.0.1:8026".to_string());
        assert_eq!(transport.transport_type(), "http-stream");
    }

    #[test]
    fn test_http_stream_capabilities() {
        let transport = HttpStreamTransport::new("127.0.0.1:8027".to_string());
        let caps = transport.capabilities();
        
        assert!(!caps.bidirectional);
        assert!(caps.server_push);
        assert!(caps.multi_connection);
        assert!(caps.streaming);
        assert!(caps.browser_compatible);
    }

    #[tokio::test]
    async fn test_http_stream_initialization() {
        let mut transport = HttpStreamTransport::new("127.0.0.1:8028".to_string());
        assert!(!transport.is_ready());
        
        let result = transport.initialize().await;
        assert!(result.is_ok());
        assert!(transport.is_ready());
    }

    #[tokio::test]
    async fn test_http_stream_shutdown() {
        let mut transport = HttpStreamTransport::new("127.0.0.1:8029".to_string());
        transport.initialize().await.unwrap();
        
        let result = transport.shutdown().await;
        assert!(result.is_ok());
        assert!(!transport.is_ready());
    }

    #[test]
    fn test_create_chunks() {
        let transport = HttpStreamTransport::new("127.0.0.1:8030".to_string());
        let data = vec![1u8; 20000]; // 20KB data
        
        let chunks = transport.create_chunks(data);
        
        // Should be split into multiple chunks
        assert!(chunks.len() > 1);
        // Each chunk (except last) should be CHUNK_SIZE
        assert_eq!(chunks[0].len(), CHUNK_SIZE);
    }

    #[test]
    fn test_factory_creation() {
        let factory = HttpStreamTransportFactory;
        let config = TransportConfig {
            transport_type: "http-stream".to_string(),
            bind_address: Some("127.0.0.1:8031".to_string()),
            ..Default::default()
        };
        
        let transport = factory.create(config).unwrap();
        assert_eq!(transport.transport_type(), "http-stream");
    }

    #[test]
    fn test_factory_supported_types() {
        let factory = HttpStreamTransportFactory;
        let types = factory.supported_types();
        assert_eq!(types, vec!["http-stream".to_string()]);
    }

    #[tokio::test]
    async fn test_send_message() {
        let mut transport = HttpStreamTransport::new("127.0.0.1:8032".to_string());
        transport.initialize().await.unwrap();
        
        let message = TransportMessage::new("test message".to_string());
        let result = transport.send(message).await;
        assert!(result.is_ok());
        
        let stats = transport.get_stats();
        assert_eq!(stats.messages_sent, 1);
    }

    #[tokio::test]
    async fn test_broadcast_message() {
        let mut transport = HttpStreamTransport::new("127.0.0.1:8033".to_string());
        transport.initialize().await.unwrap();
        
        let message = TransportMessage::new("broadcast test".to_string());
        let result = transport.broadcast(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_receive_not_supported() {
        let mut transport = HttpStreamTransport::new("127.0.0.1:8034".to_string());
        transport.initialize().await.unwrap();
        
        let result = transport.receive().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_not_initialized() {
        let transport = HttpStreamTransport::new("127.0.0.1:8035".to_string());
        let message = TransportMessage::new("test".to_string());
        let result = transport.send(message).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_active_streams() {
        let transport = HttpStreamTransport::new("127.0.0.1:8036".to_string());
        assert_eq!(transport.active_streams(), 0);
    }

    #[test]
    fn test_connection_count() {
        let transport = HttpStreamTransport::new("127.0.0.1:8037".to_string());
        assert_eq!(transport.connection_count(), 0);
    }
}