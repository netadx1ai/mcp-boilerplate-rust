//! Core MCP server implementation.

use crate::error::{ServerError, ServerResult};
use crate::registry::ToolRegistry;
use crate::ServerConfig;
use mcp_core::{
    McpError, McpRequest, McpResponse, McpServer, ResponseResult, ServerCapabilities, ServerInfo,
    Tool, ToolInputSchema, ToolContent,
};
use mcp_transport::Transport;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, error, info, instrument, warn};


/// Core MCP server implementation
/// 
/// This is the main server orchestrator that handles MCP protocol requests
/// and delegates them to appropriate tools. It manages the request lifecycle
/// and provides the main entry point for MCP communication.
pub struct McpServerImpl {
    /// Server configuration
    config: ServerConfig,
    /// Tool registry for managing available tools
    registry: Arc<RwLock<ToolRegistry>>,
    /// Semaphore for controlling concurrent requests
    request_limiter: Arc<Semaphore>,
    /// Server state
    state: Arc<RwLock<ServerState>>,
}

/// Server runtime state
#[derive(Debug, Clone)]
struct ServerState {
    /// Whether the server is running
    is_running: bool,
    /// Server start time
    start_time: Option<std::time::Instant>,
    /// Request statistics
    stats: ServerStats,
}

/// Server statistics
#[derive(Debug, Clone, Default)]
struct ServerStats {
    /// Total requests processed
    total_requests: u64,
    /// Successful requests
    successful_requests: u64,
    /// Failed requests
    failed_requests: u64,
    /// Current active requests
    active_requests: u64,
}

impl McpServerImpl {
    /// Create a new MCP server instance
    /// 
    /// # Arguments
    /// 
    /// * `config` - Server configuration
    /// * `registry` - Tool registry with registered tools
    /// 
    /// # Returns
    /// 
    /// Result containing the server instance or an error
    pub fn new(config: ServerConfig, registry: ToolRegistry) -> ServerResult<Self> {
        let request_limiter = Arc::new(Semaphore::new(config.max_concurrent_requests));
        
        let state = ServerState {
            is_running: false,
            start_time: None,
            stats: ServerStats::default(),
        };

        Ok(Self {
            config,
            registry: Arc::new(RwLock::new(registry)),
            request_limiter,
            state: Arc::new(RwLock::new(state)),
        })
    }

    /// Start the server
    /// 
    /// This initializes the server and marks it as running.
    /// 
    /// # Returns
    /// 
    /// Result indicating success or failure
    pub async fn start(&self) -> ServerResult<()> {
        let mut state = self.state.write().await;
        
        if state.is_running {
            return Err(ServerError::AlreadyRunning);
        }
        
        // Initialize server
        self.initialize().await?;
        
        state.is_running = true;
        state.start_time = Some(std::time::Instant::now());
        
        info!(
            "Started MCP server '{}' v{} with {} tools",
            self.config.name,
            self.config.version,
            self.registry.read().await.tool_count()
        );
        
        Ok(())
    }

    /// Stop the server
    /// 
    /// This gracefully shuts down the server and cleans up resources.
    /// 
    /// # Returns
    /// 
    /// Result indicating success or failure
    pub async fn stop(&self) -> ServerResult<()> {
        let mut state = self.state.write().await;
        
        if !state.is_running {
            return Err(ServerError::NotRunning);
        }
        
        // Shutdown server
        self.shutdown().await?;
        
        state.is_running = false;
        state.start_time = None;
        
        info!("Stopped MCP server '{}'", self.config.name);
        
        Ok(())
    }

    /// Check if the server is running
    /// 
    /// # Returns
    /// 
    /// True if the server is running, false otherwise
    pub async fn is_running(&self) -> bool {
        self.state.read().await.is_running
    }

    /// Get server statistics
    /// 
    /// # Returns
    /// 
    /// Current server statistics
    pub async fn stats(&self) -> ServerStats {
        self.state.read().await.stats.clone()
    }

    /// Get server uptime
    /// 
    /// # Returns
    /// 
    /// Server uptime duration, or None if not running
    pub async fn uptime(&self) -> Option<std::time::Duration> {
        let state = self.state.read().await;
        state.start_time.map(|start| start.elapsed())
    }

    /// Get the number of registered tools
    /// 
    /// # Returns
    /// 
    /// The count of registered tools
    pub async fn tool_count(&self) -> usize {
        self.registry.read().await.tool_count()
    }

    /// Run the server with a transport
    /// 
    /// This method starts the server and begins processing requests from the transport.
    /// It will run until the transport is closed or an error occurs.
    /// 
    /// # Arguments
    /// 
    /// * `transport` - The transport to use for communication
    /// 
    /// # Returns
    /// 
    /// Result indicating success or failure
    pub async fn run_with_transport(&self, transport: Arc<dyn Transport>) -> ServerResult<()> {
        self.start().await?;
        
        info!("Starting server event loop with {} transport", transport.transport_type());
        
        loop {
            match transport.receive_request().await {
                Ok(Some(request)) => {
                    // Process request asynchronously
                    let server = self.clone_arc();
                    let transport_clone = transport.clone();
                    
                    tokio::spawn(async move {
                        let response = server.handle_request(request).await.unwrap_or_else(|e| {
                            error!("Request handling failed: {}", e);
                            McpResponse::error(e.into())
                        });
                        
                        if let Err(e) = transport_clone.send_response(response).await {
                            error!("Failed to send response: {}", e);
                        }
                    });
                }
                Ok(None) => {
                    info!("Transport closed, stopping server");
                    break;
                }
                Err(e) => {
                    error!("Transport error: {}", e);
                    break;
                }
            }
        }
        
        self.stop().await?;
        Ok(())
    }

    /// Create an Arc reference to self for async tasks
    fn clone_arc(&self) -> Arc<Self> {
        // This is a simplified approach - in practice you'd store self in an Arc
        // For now, we'll return a placeholder
        unimplemented!("This would require restructuring to store self in Arc")
    }

    /// Handle a list tools request
    #[instrument(skip(self))]
    async fn handle_list_tools(&self, cursor: Option<String>) -> ServerResult<McpResponse> {
        let registry = self.registry.read().await;
        let tools = registry.list_tools();
        
        let tool_list: Vec<Tool> = tools
            .into_iter()
            .map(|tool| Tool {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                input_schema: ToolInputSchema {
                    schema_type: "object".to_string(),
                    properties: Some(HashMap::new()),
                    required: None,
                },
            })
            .collect();

        let result = ResponseResult::Tools {
            tools: tool_list,
            next_cursor: None, // Simple implementation without pagination
        };

        debug!("Listed {} tools", registry.tool_count());
        Ok(McpResponse::success(result))
    }

    /// Handle a call tool request
    #[instrument(skip(self))]
    async fn handle_call_tool(&self, name: String, arguments: HashMap<String, serde_json::Value>) -> ServerResult<McpResponse> {
        let registry = self.registry.read().await;
        
        let tool = registry
            .get_tool(&name)
            .ok_or_else(|| ServerError::ToolNotFound(name.clone()))?;

        debug!("Calling tool '{}' with {} arguments", name, arguments.len());

        // Create a CallTool request for the tool
        let tool_request = McpRequest::CallTool { name: name.clone(), arguments };

        // Call the tool
        match tool.call(tool_request).await {
            Ok(response) => {
                debug!("Tool '{}' executed successfully", name);
                Ok(response)
            }
            Err(e) => {
                warn!("Tool '{}' execution failed: {}", name, e);
                Err(ServerError::ToolExecution(e.to_string()))
            }
        }
    }

    /// Handle an initialize request
    #[instrument(skip(self))]
    async fn handle_initialize(&self) -> ServerResult<McpResponse> {
        let capabilities = ServerCapabilities::default();
        let server_info = ServerInfo {
            name: self.config.name.clone(),
            version: self.config.version.clone(),
        };

        let result = ResponseResult::Initialize {
            protocol_version: mcp_core::MCP_VERSION.to_string(),
            capabilities,
            server_info,
        };

        debug!("Server initialized successfully");
        Ok(McpResponse::success(result))
    }

    /// Update request statistics
    async fn update_stats(&self, success: bool) {
        let mut state = self.state.write().await;
        state.stats.total_requests += 1;
        
        if success {
            state.stats.successful_requests += 1;
        } else {
            state.stats.failed_requests += 1;
        }
    }

    /// Acquire a request processing permit
    async fn acquire_permit(&self) -> Result<tokio::sync::SemaphorePermit, ServerError> {
        self.request_limiter
            .acquire()
            .await
            .map_err(|_| ServerError::Concurrency("Failed to acquire request permit".to_string()))
    }
}

#[async_trait]
impl McpServer for McpServerImpl {
    #[instrument(skip(self, request))]
    async fn handle_request(&self, request: McpRequest) -> Result<McpResponse, McpError> {
        // Acquire permit for concurrency control
        let _permit = self.acquire_permit().await.map_err(|e| {
            error!("Failed to acquire request permit: {}", e);
            McpError::server_overloaded("Server is overloaded")
        })?;

        // Update active request count
        {
            let mut state = self.state.write().await;
            state.stats.active_requests += 1;
        }

        let result = match request {
            McpRequest::Initialize { .. } => {
                self.handle_initialize().await
            }
            McpRequest::ListTools { cursor } => {
                self.handle_list_tools(cursor).await
            }
            McpRequest::CallTool { name, arguments } => {
                self.handle_call_tool(name, arguments).await
            }
            McpRequest::Ping => {
                debug!("Received ping request");
                Ok(McpResponse::pong())
            }
            _ => {
                warn!("Unsupported request type: {:?}", request);
                Err(ServerError::RequestHandling(
                    "Unsupported request type".to_string()
                ))
            }
        };

        // Update statistics
        let success = result.is_ok();
        self.update_stats(success).await;

        // Update active request count
        {
            let mut state = self.state.write().await;
            state.stats.active_requests = state.stats.active_requests.saturating_sub(1);
        }

        // Convert ServerError to McpError
        result.map_err(|e| e.into())
    }

    async fn initialize(&self) -> Result<(), McpError> {
        debug!("Initializing MCP server");
        
        // Validate tools
        let registry = self.registry.read().await;
        registry.validate_tools().map_err(|e| {
            error!("Tool validation failed during initialization: {}", e);
            McpError::internal_error(e.to_string())
        })?;

        info!("MCP server initialized with {} tools", registry.tool_count());
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), McpError> {
        debug!("Shutting down MCP server");
        
        // Wait for active requests to complete (with timeout)
        let timeout = std::time::Duration::from_secs(30);
        let start = std::time::Instant::now();
        
        while start.elapsed() < timeout {
            let active_requests = {
                let state = self.state.read().await;
                state.stats.active_requests
            };
            
            if active_requests == 0 {
                break;
            }
            
            debug!("Waiting for {} active requests to complete", active_requests);
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        info!("MCP server shutdown complete");
        Ok(())
    }

    fn server_info(&self) -> (&str, &str) {
        (&self.config.name, &self.config.version)
    }

    fn capabilities(&self) -> HashMap<String, serde_json::Value> {
        let mut caps = HashMap::new();
        caps.insert("tools".to_string(), serde_json::json!({}));
        caps.insert("resources".to_string(), serde_json::json!({}));
        caps
    }

    async fn health_check(&self) -> bool {
        self.is_running().await
    }

    async fn metrics(&self) -> HashMap<String, serde_json::Value> {
        let state = self.state.read().await;
        let registry = self.registry.read().await;
        
        let mut metrics = HashMap::new();
        metrics.insert("total_requests".to_string(), state.stats.total_requests.into());
        metrics.insert("successful_requests".to_string(), state.stats.successful_requests.into());
        metrics.insert("failed_requests".to_string(), state.stats.failed_requests.into());
        metrics.insert("active_requests".to_string(), state.stats.active_requests.into());
        metrics.insert("tool_count".to_string(), registry.tool_count().into());
        metrics.insert("uptime_seconds".to_string(), 
            self.uptime().await.map(|d| d.as_secs()).unwrap_or(0).into());
        
        metrics
    }
}

impl std::fmt::Debug for McpServerImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpServerImpl")
            .field("config", &self.config)
            .field("max_concurrent_requests", &self.config.max_concurrent_requests)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcp_core::{McpTool};
    use async_trait::async_trait;

    struct MockTool {
        name: String,
        description: String,
    }

    #[async_trait]
    impl McpTool for MockTool {
        async fn call(&self, _request: McpRequest) -> Result<McpResponse, McpError> {
            Ok(McpResponse::simple_success("mock response"))
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }
    }

    #[tokio::test]
    async fn test_server_creation() {
        let config = ServerConfig::default();
        let registry = ToolRegistry::new();
        
        let server = McpServerImpl::new(config, registry).unwrap();
        assert!(!server.is_running().await);
    }

    #[tokio::test]
    async fn test_server_start_stop() {
        let config = ServerConfig::default();
        let registry = ToolRegistry::new();
        let server = McpServerImpl::new(config, registry).unwrap();

        // Start server
        assert!(server.start().await.is_ok());
        assert!(server.is_running().await);

        // Stop server
        assert!(server.stop().await.is_ok());
        assert!(!server.is_running().await);
    }

    #[tokio::test]
    async fn test_ping_request() {
        let config = ServerConfig::default();
        let registry = ToolRegistry::new();
        let server = McpServerImpl::new(config, registry).unwrap();

        let response = server.handle_request(McpRequest::Ping).await.unwrap();
        match response {
            McpResponse::Success { result: ResponseResult::Pong } => (),
            _ => panic!("Expected pong response"),
        }
    }

    #[tokio::test]
    async fn test_list_tools_empty() {
        let config = ServerConfig::default();
        let registry = ToolRegistry::new();
        let server = McpServerImpl::new(config, registry).unwrap();

        let response = server.handle_request(McpRequest::ListTools { cursor: None }).await.unwrap();
        match response {
            McpResponse::Success { result: ResponseResult::Tools { tools, .. } } => {
                assert!(tools.is_empty());
            }
            _ => panic!("Expected tools response"),
        }
    }

    #[tokio::test]
    async fn test_list_tools_with_tools() {
        let config = ServerConfig::default();
        let mut registry = ToolRegistry::new();
        
        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
        });
        registry.register_tool(tool).unwrap();
        
        let server = McpServerImpl::new(config, registry).unwrap();

        let response = server.handle_request(McpRequest::ListTools { cursor: None }).await.unwrap();
        match response {
            McpResponse::Success { result: ResponseResult::Tools { tools, .. } } => {
                assert_eq!(tools.len(), 1);
                assert_eq!(tools[0].name, "test_tool");
            }
            _ => panic!("Expected tools response"),
        }
    }

    #[tokio::test]
    async fn test_call_nonexistent_tool() {
        let config = ServerConfig::default();
        let registry = ToolRegistry::new();
        let server = McpServerImpl::new(config, registry).unwrap();

        let request = McpRequest::CallTool {
            name: "nonexistent".to_string(),
            arguments: HashMap::new(),
        };

        let response = server.handle_request(request).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_server_metrics() {
        let config = ServerConfig::default();
        let registry = ToolRegistry::new();
        let server = McpServerImpl::new(config, registry).unwrap();

        let metrics = server.metrics().await;
        assert!(metrics.contains_key("total_requests"));
        assert!(metrics.contains_key("tool_count"));
    }
}