//! STDIO transport implementation for MCP communication.

use crate::error::{TransportError, TransportResult};
use crate::transport::{utils, MessageContent, Transport, TransportConfig, TransportMessage};
use async_trait::async_trait;
use mcp_core::{McpRequest, McpResponse};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tracing::{debug, error, trace, warn};

/// STDIO transport implementation
///
/// This transport uses standard input and output streams for communication,
/// which is ideal for pipe-based MCP communication patterns.
pub struct StdioTransport {
    /// Transport configuration
    config: TransportConfig,
    /// Stdin reader wrapped in Arc<Mutex<>> for thread safety
    stdin: Arc<Mutex<BufReader<tokio::io::Stdin>>>,
    /// Stdout writer wrapped in Arc<Mutex<>> for thread safety
    stdout: Arc<Mutex<tokio::io::Stdout>>,
    /// Connection state
    connected: Arc<std::sync::atomic::AtomicBool>,
}

impl StdioTransport {
    /// Create a new STDIO transport
    ///
    /// # Arguments
    ///
    /// * `config` - Transport configuration
    ///
    /// # Returns
    ///
    /// A new STDIO transport instance
    pub fn new(config: TransportConfig) -> TransportResult<Self> {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        Ok(Self {
            config,
            stdin: Arc::new(Mutex::new(BufReader::new(stdin))),
            stdout: Arc::new(Mutex::new(stdout)),
            connected: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        })
    }

    /// Create a new STDIO transport with default configuration
    ///
    /// # Returns
    ///
    /// A new STDIO transport instance with default settings
    pub fn with_defaults() -> TransportResult<Self> {
        Self::new(TransportConfig::default())
    }

    /// Write a JSON line to stdout
    ///
    /// # Arguments
    ///
    /// * `json` - The JSON string to write
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    async fn write_json_line(&self, json: &str) -> TransportResult<()> {
        let mut stdout = self.stdout.lock().await;

        // Validate message size
        utils::validate_message_size(json.len(), &self.config)?;

        stdout.write_all(json.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;

        trace!("Sent JSON: {}", json);
        Ok(())
    }

    /// Read a JSON line from stdin
    ///
    /// # Returns
    ///
    /// Result containing the JSON string or None if EOF
    async fn read_json_line(&self) -> TransportResult<Option<String>> {
        let mut stdin = self.stdin.lock().await;
        let mut line = String::new();

        match stdin.read_line(&mut line).await? {
            0 => {
                debug!("EOF reached on stdin");
                self.connected
                    .store(false, std::sync::atomic::Ordering::Relaxed);
                Ok(None)
            }
            n => {
                // Validate message size
                utils::validate_message_size(n, &self.config)?;

                // Remove trailing newline
                if line.ends_with('\n') {
                    line.pop();
                    if line.ends_with('\r') {
                        line.pop();
                    }
                }

                trace!("Received JSON: {}", line);
                Ok(Some(line))
            }
        }
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn send_response(&self, response: McpResponse) -> TransportResult<()> {
        if !self.is_connected() {
            return Err(TransportError::NotConnected);
        }

        let message = TransportMessage {
            id: Some(uuid::Uuid::new_v4().to_string()),
            content: MessageContent::Response(response),
            metadata: std::collections::HashMap::new(),
        };

        let json = serde_json::to_string(&message)?;
        self.write_json_line(&json).await?;

        debug!("Sent MCP response via STDIO");
        Ok(())
    }

    async fn receive_request(&self) -> TransportResult<Option<McpRequest>> {
        if !self.is_connected() {
            return Err(TransportError::NotConnected);
        }

        loop {
            match self.read_json_line().await? {
                Some(line) => {
                    if line.trim().is_empty() {
                        continue; // Skip empty lines
                    }

                    match serde_json::from_str::<TransportMessage>(&line) {
                        Ok(message) => {
                            match message.content {
                                MessageContent::Request(request) => {
                                    debug!("Received MCP request via STDIO");
                                    return Ok(Some(request));
                                }
                                MessageContent::Control(control) => {
                                    debug!("Received control message: {:?}", control);
                                    // Handle control messages if needed
                                    continue;
                                }
                                MessageContent::Response(_) => {
                                    warn!(
                                        "Received unexpected response message on STDIO transport"
                                    );
                                    continue;
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse JSON message: {}", e);
                            return Err(TransportError::InvalidMessage(format!(
                                "Invalid JSON: {e}"
                            )));
                        }
                    }
                }
                None => {
                    debug!("STDIO stream closed");
                    return Ok(None);
                }
            }
        }
    }

    fn is_connected(&self) -> bool {
        self.connected.load(std::sync::atomic::Ordering::Relaxed)
    }

    async fn close(&self) -> TransportResult<()> {
        self.connected
            .store(false, std::sync::atomic::Ordering::Relaxed);

        // Flush stdout before closing
        let mut stdout = self.stdout.lock().await;
        stdout.flush().await?;

        debug!("STDIO transport closed");
        Ok(())
    }

    fn config(&self) -> &TransportConfig {
        &self.config
    }

    fn metadata(&self) -> std::collections::HashMap<String, String> {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("transport".to_string(), "stdio".to_string());
        metadata.insert("version".to_string(), "1.0".to_string());
        metadata.insert("bidirectional".to_string(), "true".to_string());
        metadata
    }

    fn is_bidirectional(&self) -> bool {
        true
    }

    fn transport_type(&self) -> &'static str {
        "stdio"
    }
}

/// Builder for STDIO transport with custom configuration
pub struct StdioTransportBuilder {
    config: TransportConfig,
}

impl StdioTransportBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: TransportConfig::default(),
        }
    }

    /// Set the maximum message size
    pub fn max_message_size(mut self, size: usize) -> Self {
        self.config.max_message_size = size;
        self
    }

    /// Set the buffer size
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.config.buffer_size = size;
        self
    }

    /// Set the timeout duration
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Build the STDIO transport
    pub fn build(self) -> TransportResult<StdioTransport> {
        StdioTransport::new(self.config)
    }
}

impl Default for StdioTransportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stdio_transport_creation() {
        let transport = StdioTransport::with_defaults().unwrap();
        assert_eq!(transport.transport_type(), "stdio");
        assert!(transport.is_connected());
        assert!(transport.is_bidirectional());
    }

    #[test]
    fn test_stdio_transport_builder() {
        let transport = StdioTransportBuilder::new()
            .max_message_size(2048)
            .buffer_size(4096)
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap();

        assert_eq!(transport.config.max_message_size, 2048);
        assert_eq!(transport.config.buffer_size, 4096);
        assert_eq!(transport.config.timeout, std::time::Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_stdio_transport_metadata() {
        let transport = StdioTransport::with_defaults().unwrap();
        let metadata = transport.metadata();

        assert_eq!(metadata.get("transport"), Some(&"stdio".to_string()));
        assert_eq!(metadata.get("version"), Some(&"1.0".to_string()));
        assert_eq!(metadata.get("bidirectional"), Some(&"true".to_string()));
    }

    #[tokio::test]
    async fn test_stdio_transport_close() {
        let transport = StdioTransport::with_defaults().unwrap();
        assert!(transport.is_connected());

        transport.close().await.unwrap();
        assert!(!transport.is_connected());
    }

    #[test]
    fn test_message_size_validation() {
        let config = TransportConfig {
            max_message_size: 100,
            ..Default::default()
        };

        // This would be tested in integration tests with actual STDIO
        // Here we just verify the config is set correctly
        assert_eq!(config.max_message_size, 100);
    }

    // Note: Full integration tests for STDIO transport would require
    // setting up actual stdin/stdout pipes, which is complex in unit tests.
    // These would typically be covered in integration tests.
}
