//! HTTP transport implementation for MCP communication.

use crate::error::{TransportError, TransportResult};
use crate::transport::{Transport, TransportConfig};
use async_trait::async_trait;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use mcp_core::{McpRequest, McpResponse};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, oneshot};
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info, trace};

/// HTTP transport implementation
/// 
/// This transport uses HTTP/REST for MCP communication, making it suitable
/// for web-based integration and when STDIO is not available.
pub struct HttpTransport {
    /// Transport configuration
    config: TransportConfig,
    /// Server address
    addr: SocketAddr,
    /// Connection state
    connected: Arc<std::sync::atomic::AtomicBool>,
    /// Request sender channel
    request_sender: Arc<Mutex<Option<mpsc::UnboundedSender<McpRequest>>>>,
    /// Request receiver channel
    request_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<McpRequest>>>>,
    /// Response sender for handling HTTP responses
    response_sender: Arc<Mutex<Option<mpsc::UnboundedSender<McpResponse>>>>,
    /// Response receiver for handling HTTP responses
    response_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<McpResponse>>>>,
    /// Server shutdown sender
    shutdown_sender: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

/// Shared state for HTTP handlers
#[derive(Clone)]
struct HttpState {
    request_sender: mpsc::UnboundedSender<McpRequest>,
    response_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<McpResponse>>>>,
}

impl HttpTransport {
    /// Create a new HTTP transport
    /// 
    /// # Arguments
    /// 
    /// * `addr` - Socket address to bind the HTTP server to
    /// * `config` - Transport configuration
    /// 
    /// # Returns
    /// 
    /// A new HTTP transport instance
    pub fn new(addr: SocketAddr, config: TransportConfig) -> TransportResult<Self> {
        let (request_sender, request_receiver) = mpsc::unbounded_channel();
        let (response_sender, response_receiver) = mpsc::unbounded_channel();
        let (shutdown_sender, _) = oneshot::channel();

        Ok(Self {
            config,
            addr,
            connected: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            request_sender: Arc::new(Mutex::new(Some(request_sender))),
            request_receiver: Arc::new(Mutex::new(Some(request_receiver))),
            response_sender: Arc::new(Mutex::new(Some(response_sender))),
            response_receiver: Arc::new(Mutex::new(Some(response_receiver))),
            shutdown_sender: Arc::new(Mutex::new(Some(shutdown_sender))),
        })
    }

    /// Create a new HTTP transport with default configuration
    /// 
    /// # Arguments
    /// 
    /// * `addr` - Socket address to bind the HTTP server to
    /// 
    /// # Returns
    /// 
    /// A new HTTP transport instance with default settings
    pub fn with_defaults(addr: SocketAddr) -> TransportResult<Self> {
        Self::new(addr, TransportConfig::default())
    }

    /// Start the HTTP server
    /// 
    /// # Returns
    /// 
    /// Result indicating success or failure of server start
    pub async fn start_server(&self) -> TransportResult<()> {
        let request_sender = self.request_sender.lock().await
            .take()
            .ok_or_else(|| TransportError::Other("Request sender already taken".to_string()))?;

        let response_receiver = self.response_receiver.clone();

        let state = HttpState {
            request_sender,
            response_receiver,
        };

        let app = Router::new()
            .route("/health", get(health_check))
            .route("/mcp/request", post(handle_mcp_request))
            .route("/mcp/tools/list", get(handle_list_tools))
            .route("/mcp/tools/call", post(handle_call_tool))
            .route("/mcp/resources/list", get(handle_list_resources))
            .route("/mcp/resources/read", post(handle_read_resource))
            .layer(TraceLayer::new_for_http())
            .with_state(state);

        let listener = tokio::net::TcpListener::bind(self.addr).await
            .map_err(|e| TransportError::Connection(format!("Failed to bind to {}: {}", self.addr, e)))?;

        info!("HTTP transport server starting on {}", self.addr);
        
        self.connected.store(true, std::sync::atomic::Ordering::Relaxed);

        // Start the server in a separate task
        let connected = self.connected.clone();
        let shutdown_receiver = {
            let mut guard = self.shutdown_sender.lock().await;
            let (sender, receiver) = oneshot::channel();
            *guard = Some(sender);
            receiver
        };

        tokio::spawn(async move {
            let server = axum::serve(listener, app);
            
            tokio::select! {
                result = server => {
                    if let Err(e) = result {
                        error!("HTTP server error: {}", e);
                    }
                }
                _ = shutdown_receiver => {
                    info!("HTTP server shutdown requested");
                }
            }
            
            connected.store(false, std::sync::atomic::Ordering::Relaxed);
        });

        Ok(())
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn send_response(&self, response: McpResponse) -> TransportResult<()> {
        if !self.is_connected() {
            return Err(TransportError::NotConnected);
        }

        let sender = self.response_sender.lock().await;
        if let Some(ref sender) = *sender {
            sender.send(response)
                .map_err(|_| TransportError::Other("Failed to send response".to_string()))?;
            debug!("Sent MCP response via HTTP");
        } else {
            return Err(TransportError::Other("Response sender not available".to_string()));
        }

        Ok(())
    }

    async fn receive_request(&self) -> TransportResult<Option<McpRequest>> {
        if !self.is_connected() {
            return Err(TransportError::NotConnected);
        }

        let mut receiver = self.request_receiver.lock().await;
        if let Some(ref mut receiver) = *receiver {
            match receiver.recv().await {
                Some(request) => {
                    debug!("Received MCP request via HTTP");
                    Ok(Some(request))
                }
                None => {
                    debug!("HTTP request channel closed");
                    Ok(None)
                }
            }
        } else {
            Err(TransportError::Other("Request receiver not available".to_string()))
        }
    }

    fn is_connected(&self) -> bool {
        self.connected.load(std::sync::atomic::Ordering::Relaxed)
    }

    async fn close(&self) -> TransportResult<()> {
        self.connected.store(false, std::sync::atomic::Ordering::Relaxed);
        
        // Send shutdown signal
        let mut shutdown_sender = self.shutdown_sender.lock().await;
        if let Some(sender) = shutdown_sender.take() {
            let _ = sender.send(());
        }
        
        debug!("HTTP transport closed");
        Ok(())
    }

    fn config(&self) -> &TransportConfig {
        &self.config
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("transport".to_string(), "http".to_string());
        metadata.insert("version".to_string(), "1.0".to_string());
        metadata.insert("address".to_string(), self.addr.to_string());
        metadata.insert("bidirectional".to_string(), "true".to_string());
        metadata
    }

    fn is_bidirectional(&self) -> bool {
        true
    }

    fn transport_type(&self) -> &'static str {
        "http"
    }
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

/// Handle generic MCP request
async fn handle_mcp_request(
    State(state): State<HttpState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    trace!("Received raw MCP request: {}", payload);
    
    // Try to deserialize as MCP request
    let request: McpRequest = serde_json::from_value(payload)
        .map_err(|e| {
            error!("Failed to deserialize MCP request: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    // Send request through channel
    state.request_sender.send(request)
        .map_err(|e| {
            error!("Failed to send request: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // For now, return a simple acknowledgment
    // In a full implementation, you'd wait for and return the actual response
    Ok(Json(serde_json::json!({
        "status": "received",
        "message": "Request processed"
    })))
}

/// Handle list tools request
async fn handle_list_tools(
    State(state): State<HttpState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let request = McpRequest::ListTools { cursor: None };
    
    state.request_sender.send(request)
        .map_err(|e| {
            error!("Failed to send list tools request: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Return placeholder response
    Ok(Json(serde_json::json!({
        "tools": [],
        "next_cursor": null
    })))
}

/// Handle call tool request
async fn handle_call_tool(
    State(state): State<HttpState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let name = payload.get("name")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?
        .to_string();

    let arguments = payload.get("arguments")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        })
        .unwrap_or_default();

    let request = McpRequest::CallTool { name, arguments };
    
    state.request_sender.send(request)
        .map_err(|e| {
            error!("Failed to send call tool request: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Return placeholder response
    Ok(Json(serde_json::json!({
        "content": [
            {
                "type": "text",
                "text": "Tool executed successfully"
            }
        ],
        "isError": false
    })))
}

/// Handle list resources request
async fn handle_list_resources(
    State(state): State<HttpState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let request = McpRequest::ListResources { cursor: None };
    
    state.request_sender.send(request)
        .map_err(|e| {
            error!("Failed to send list resources request: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Return placeholder response
    Ok(Json(serde_json::json!({
        "resources": [],
        "next_cursor": null
    })))
}

/// Handle read resource request
async fn handle_read_resource(
    State(state): State<HttpState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let uri = payload.get("uri")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?
        .to_string();

    let request = McpRequest::ReadResource { uri };
    
    state.request_sender.send(request)
        .map_err(|e| {
            error!("Failed to send read resource request: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Return placeholder response
    Ok(Json(serde_json::json!({
        "contents": [
            {
                "uri": payload.get("uri"),
                "mimeType": "text/plain",
                "text": "Resource content"
            }
        ]
    })))
}

/// Builder for HTTP transport with custom configuration
pub struct HttpTransportBuilder {
    addr: SocketAddr,
    config: TransportConfig,
}

impl HttpTransportBuilder {
    /// Create a new builder with the specified address
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
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

    /// Enable or disable compression
    pub fn compression(mut self, enabled: bool) -> Self {
        self.config.compression = enabled;
        self
    }

    /// Build the HTTP transport
    pub fn build(self) -> TransportResult<HttpTransport> {
        HttpTransport::new(self.addr, self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_http_transport_creation() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
        let transport = HttpTransport::with_defaults(addr).unwrap();
        
        assert_eq!(transport.transport_type(), "http");
        assert!(!transport.is_connected()); // Not connected until server starts
        assert!(transport.is_bidirectional());
    }

    #[test]
    fn test_http_transport_builder() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000);
        let transport = HttpTransportBuilder::new(addr)
            .max_message_size(2048)
            .buffer_size(4096)
            .timeout(std::time::Duration::from_secs(60))
            .compression(true)
            .build()
            .unwrap();

        assert_eq!(transport.config.max_message_size, 2048);
        assert_eq!(transport.config.buffer_size, 4096);
        assert_eq!(transport.config.timeout, std::time::Duration::from_secs(60));
        assert!(transport.config.compression);
        assert_eq!(transport.addr, addr);
    }

    #[tokio::test]
    async fn test_http_transport_metadata() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000);
        let transport = HttpTransport::with_defaults(addr).unwrap();
        let metadata = transport.metadata();
        
        assert_eq!(metadata.get("transport"), Some(&"http".to_string()));
        assert_eq!(metadata.get("version"), Some(&"1.0".to_string()));
        assert_eq!(metadata.get("address"), Some(&addr.to_string()));
        assert_eq!(metadata.get("bidirectional"), Some(&"true".to_string()));
    }

    #[tokio::test]
    async fn test_http_transport_close() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
        let transport = HttpTransport::with_defaults(addr).unwrap();
        
        // Start server to set connected state
        transport.start_server().await.unwrap();
        assert!(transport.is_connected());
        
        transport.close().await.unwrap();
        
        // Give it a moment for the async close to take effect
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        assert!(!transport.is_connected());
    }

    // Note: Full integration tests for HTTP transport would require
    // actual HTTP clients and servers, which are complex in unit tests.
    // These would typically be covered in integration tests.
}