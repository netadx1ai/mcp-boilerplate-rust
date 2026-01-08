//! Stdio transport implementation for MCP protocol
//! 
//! This transport uses standard input/output for communication, making it ideal for:
//! - CLI tools and desktop applications
//! - Process spawning and IPC
//! - Claude Desktop integration
//! 
//! # Important Notes
//! 
//! - **NO logging to stdout/stderr** - This would break JSON-RPC parsing
//! - **ANSI codes disabled** - Clean JSON output only
//! - **Synchronous message flow** - One request/response at a time

use super::r#trait::{
    Transport, TransportCapabilities, TransportConfig, TransportError, 
    TransportFactory, TransportMessage, TransportStats,
};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// Stdio transport implementation
/// 
/// Reads JSON-RPC messages from stdin and writes responses to stdout.
/// Each message is a complete JSON object on a single line.
pub struct StdioTransport {
    /// Transport state
    state: Arc<Mutex<StdioState>>,
    /// Statistics tracking
    stats: Arc<Mutex<TransportStats>>,
    /// Configuration
    config: TransportConfig,
}

struct StdioState {
    initialized: bool,
    shutdown: bool,
}

impl StdioTransport {
    /// Create a new stdio transport
    pub fn new() -> Self {
        Self::with_config(TransportConfig {
            transport_type: "stdio".to_string(),
            ..Default::default()
        })
    }

    /// Create with custom configuration
    pub fn with_config(config: TransportConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(StdioState {
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
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for StdioTransport {
    fn transport_type(&self) -> &str {
        "stdio"
    }

    fn capabilities(&self) -> TransportCapabilities {
        TransportCapabilities {
            bidirectional: true,
            server_push: false, // stdio is request/response only
            multi_connection: false,
            streaming: false,
            browser_compatible: false,
        }
    }

    async fn send(&self, message: TransportMessage) -> Result<(), TransportError> {
        // Check if shutdown
        {
            let state = self.state.lock().unwrap();
            if state.shutdown {
                return Err(TransportError::AlreadyShutdown);
            }
            if !state.initialized {
                return Err(TransportError::NotInitialized);
            }
        }

        // Check message size
        let content_bytes = message.content.as_bytes();
        if content_bytes.len() > self.config.max_message_size {
            return Err(TransportError::MessageTooLarge {
                actual: content_bytes.len(),
                max: self.config.max_message_size,
            });
        }

        // Write to stdout with newline
        let mut stdout = tokio::io::stdout();
        stdout
            .write_all(content_bytes)
            .await
            .map_err(|e| TransportError::SendError(e.to_string()))?;
        
        stdout
            .write_all(b"\n")
            .await
            .map_err(|e| TransportError::SendError(e.to_string()))?;
        
        stdout
            .flush()
            .await
            .map_err(|e| TransportError::SendError(e.to_string()))?;

        // Update stats
        {
            let mut stats = self.stats.lock().unwrap();
            stats.messages_sent += 1;
            stats.bytes_sent += content_bytes.len() as u64;
        }

        Ok(())
    }

    async fn receive(&self) -> Result<TransportMessage, TransportError> {
        // Check if shutdown
        {
            let state = self.state.lock().unwrap();
            if state.shutdown {
                return Err(TransportError::AlreadyShutdown);
            }
            if !state.initialized {
                return Err(TransportError::NotInitialized);
            }
        }

        // Read line from stdin
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        reader
            .read_line(&mut line)
            .await
            .map_err(|e| TransportError::ReceiveError(e.to_string()))?;

        // Check if EOF (empty line indicates stdin closed)
        if line.is_empty() {
            return Err(TransportError::ReceiveError("EOF reached".to_string()));
        }

        // Trim newline
        let content = line.trim_end().to_string();

        // Check message size
        if content.len() > self.config.max_message_size {
            return Err(TransportError::MessageTooLarge {
                actual: content.len(),
                max: self.config.max_message_size,
            });
        }

        // Update stats
        {
            let mut stats = self.stats.lock().unwrap();
            stats.messages_received += 1;
            stats.bytes_received += content.len() as u64;
        }

        Ok(TransportMessage::new(content))
    }

    async fn initialize(&mut self) -> Result<(), TransportError> {
        let mut state = self.state.lock().unwrap();
        if state.initialized {
            return Ok(()); // Already initialized
        }

        // For stdio, initialization is simple - just mark as ready
        state.initialized = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), TransportError> {
        let mut state = self.state.lock().unwrap();
        if state.shutdown {
            return Ok(()); // Already shutdown
        }

        state.shutdown = true;
        state.initialized = false;
        Ok(())
    }

    fn is_ready(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.initialized && !state.shutdown
    }
}

/// Factory for creating stdio transport instances
pub struct StdioTransportFactory;

impl TransportFactory for StdioTransportFactory {
    fn create(&self, config: TransportConfig) -> Result<Box<dyn Transport>, TransportError> {
        // Validate config
        if config.transport_type != "stdio" {
            return Err(TransportError::ConfigError(format!(
                "Invalid transport type '{}' for stdio factory",
                config.transport_type
            )));
        }

        Ok(Box::new(StdioTransport::with_config(config)))
    }

    fn supported_types(&self) -> Vec<String> {
        vec!["stdio".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdio_transport_creation() {
        let transport = StdioTransport::new();
        assert_eq!(transport.transport_type(), "stdio");
        assert!(!transport.is_ready()); // Not initialized yet
    }

    #[test]
    fn test_stdio_capabilities() {
        let transport = StdioTransport::new();
        let caps = transport.capabilities();
        assert!(caps.bidirectional);
        assert!(!caps.server_push);
        assert!(!caps.multi_connection);
        assert!(!caps.browser_compatible);
    }

    #[tokio::test]
    async fn test_stdio_initialization() {
        let mut transport = StdioTransport::new();
        assert!(!transport.is_ready());

        transport.initialize().await.unwrap();
        assert!(transport.is_ready());

        // Double initialization should be ok
        transport.initialize().await.unwrap();
        assert!(transport.is_ready());
    }

    #[tokio::test]
    async fn test_stdio_shutdown() {
        let mut transport = StdioTransport::new();
        transport.initialize().await.unwrap();
        assert!(transport.is_ready());

        transport.shutdown().await.unwrap();
        assert!(!transport.is_ready());

        // Double shutdown should be ok
        transport.shutdown().await.unwrap();
        assert!(!transport.is_ready());
    }

    #[tokio::test]
    async fn test_send_without_initialization() {
        let transport = StdioTransport::new();
        let message = TransportMessage::new("test".to_string());
        let result = transport.send(message).await;
        assert!(matches!(result, Err(TransportError::NotInitialized)));
    }

    #[test]
    fn test_factory_creation() {
        let factory = StdioTransportFactory;
        let config = TransportConfig {
            transport_type: "stdio".to_string(),
            ..Default::default()
        };

        let transport = factory.create(config).unwrap();
        assert_eq!(transport.transport_type(), "stdio");
    }

    #[test]
    fn test_factory_invalid_type() {
        let factory = StdioTransportFactory;
        let config = TransportConfig {
            transport_type: "invalid".to_string(),
            ..Default::default()
        };

        let result = factory.create(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_stats_tracking() {
        let transport = StdioTransport::new();
        let stats = transport.stats();
        assert_eq!(stats.messages_sent, 0);
        assert_eq!(stats.messages_received, 0);
    }
}