//! Builder pattern implementation for MCP server configuration.

use crate::error::{ServerError, ServerResult};
use crate::registry::ToolRegistry;
use crate::server::McpServerImpl;
use crate::ServerConfig;
use mcp_core::McpTool;
use mcp_transport::Transport;
use std::sync::Arc;
use tracing::{debug, info};

/// Builder for constructing MCP server instances with fluent configuration
/// 
/// The builder pattern allows for easy configuration of server instances
/// with sensible defaults and type-safe construction.
/// 
/// # Example
/// 
/// ```rust
/// use mcp_server::McpServerBuilder;
/// 
/// let server = McpServerBuilder::new()
///     .with_name("my-server")
///     .with_version("1.0.0")
///     .max_concurrent_requests(50)
///     .enable_tracing(true)
///     .build();
/// ```
pub struct McpServerBuilder {
    /// Server configuration
    config: ServerConfig,
    /// Tool registry for managing tools
    registry: ToolRegistry,
}

impl McpServerBuilder {
    /// Create a new server builder with default configuration
    /// 
    /// # Returns
    /// 
    /// A new builder instance with default settings
    pub fn new() -> Self {
        Self {
            config: ServerConfig::default(),
            registry: ToolRegistry::new(),
        }
    }

    /// Set the server name
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the server
    /// 
    /// # Returns
    /// 
    /// The builder instance for chaining
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.config.name = name.into();
        self
    }

    /// Set the server version
    /// 
    /// # Arguments
    /// 
    /// * `version` - The version string for the server
    /// 
    /// # Returns
    /// 
    /// The builder instance for chaining
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.config.version = version.into();
        self
    }

    /// Set the maximum number of concurrent requests
    /// 
    /// # Arguments
    /// 
    /// * `max_requests` - Maximum concurrent request limit
    /// 
    /// # Returns
    /// 
    /// The builder instance for chaining
    pub fn max_concurrent_requests(mut self, max_requests: usize) -> Self {
        self.config.max_concurrent_requests = max_requests;
        self
    }

    /// Enable or disable request tracing
    /// 
    /// # Arguments
    /// 
    /// * `enabled` - Whether to enable tracing
    /// 
    /// # Returns
    /// 
    /// The builder instance for chaining
    pub fn enable_tracing(mut self, enabled: bool) -> Self {
        self.config.enable_tracing = enabled;
        self
    }

    /// Add a tool to the server
    /// 
    /// # Arguments
    /// 
    /// * `tool` - The tool implementation to add
    /// 
    /// # Returns
    /// 
    /// The builder instance for chaining
    pub fn add_tool(mut self, tool: Arc<dyn McpTool>) -> Self {
        let name = tool.name().to_string();
        if let Err(e) = self.registry.register_tool(tool) {
            debug!("Failed to register tool '{}': {}", name, e);
        } else {
            debug!("Added tool '{}' to builder", name);
        }
        self
    }

    /// Add multiple tools to the server
    /// 
    /// # Arguments
    /// 
    /// * `tools` - A vector of tools to add
    /// 
    /// # Returns
    /// 
    /// The builder instance for chaining
    pub fn add_tools(mut self, tools: Vec<Arc<dyn McpTool>>) -> Self {
        for tool in tools {
            self = self.add_tool(tool);
        }
        self
    }

    /// Set a custom server configuration
    /// 
    /// # Arguments
    /// 
    /// * `config` - The server configuration to use
    /// 
    /// # Returns
    /// 
    /// The builder instance for chaining
    pub fn with_config(mut self, config: ServerConfig) -> Self {
        self.config = config;
        self
    }

    /// Configure the server from environment variables
    /// 
    /// This method reads common configuration values from environment variables:
    /// - `MCP_SERVER_NAME`: Server name
    /// - `MCP_SERVER_VERSION`: Server version
    /// - `MCP_MAX_CONCURRENT_REQUESTS`: Maximum concurrent requests
    /// - `MCP_ENABLE_TRACING`: Enable tracing (true/false)
    /// 
    /// # Returns
    /// 
    /// The builder instance for chaining
    pub fn from_env(mut self) -> Self {
        if let Ok(name) = std::env::var("MCP_SERVER_NAME") {
            self.config.name = name;
        }
        
        if let Ok(version) = std::env::var("MCP_SERVER_VERSION") {
            self.config.version = version;
        }
        
        if let Ok(max_requests) = std::env::var("MCP_MAX_CONCURRENT_REQUESTS") {
            if let Ok(parsed) = max_requests.parse::<usize>() {
                self.config.max_concurrent_requests = parsed;
            }
        }
        
        if let Ok(tracing) = std::env::var("MCP_ENABLE_TRACING") {
            self.config.enable_tracing = tracing.to_lowercase() == "true";
        }
        
        debug!("Configured server from environment variables");
        self
    }

    /// Validate the current configuration
    /// 
    /// This method checks that the server configuration is valid and
    /// all required components are properly set up.
    /// 
    /// # Returns
    /// 
    /// Result indicating validation success or failure
    pub fn validate(&self) -> ServerResult<()> {
        if self.config.name.is_empty() {
            return Err(ServerError::Configuration(
                "Server name cannot be empty".to_string()
            ));
        }
        
        if self.config.version.is_empty() {
            return Err(ServerError::Configuration(
                "Server version cannot be empty".to_string()
            ));
        }
        
        if self.config.max_concurrent_requests == 0 {
            return Err(ServerError::Configuration(
                "Maximum concurrent requests must be greater than 0".to_string()
            ));
        }
        
        // Validate all registered tools
        self.registry.validate_tools()?;
        
        debug!("Server configuration validated successfully");
        Ok(())
    }

    /// Build the MCP server instance
    /// 
    /// This method validates the configuration and constructs the final
    /// server instance. It consumes the builder.
    /// 
    /// # Returns
    /// 
    /// Result containing the configured server or an error
    pub fn build(self) -> ServerResult<McpServerImpl> {
        // Validate configuration before building
        self.validate()?;
        
        info!(
            "Building MCP server '{}' v{} with {} tools",
            self.config.name,
            self.config.version,
            self.registry.tool_count()
        );
        
        let server = McpServerImpl::new(self.config, self.registry)?;
        
        info!("Successfully built MCP server");
        Ok(server)
    }

    /// Get the current configuration (for inspection)
    /// 
    /// # Returns
    /// 
    /// A reference to the current server configuration
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// Get the current tool registry (for inspection)
    /// 
    /// # Returns
    /// 
    /// A reference to the current tool registry
    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }

    /// Get the number of registered tools
    /// 
    /// # Returns
    /// 
    /// The count of currently registered tools
    pub fn tool_count(&self) -> usize {
        self.registry.tool_count()
    }

    /// Check if a tool is registered
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the tool to check
    /// 
    /// # Returns
    /// 
    /// True if the tool is registered, false otherwise
    pub fn has_tool(&self, name: &str) -> bool {
        self.registry.has_tool(name)
    }

    /// Clone the builder with the same configuration but empty registry
    /// 
    /// This is useful for creating multiple similar servers with different tools.
    /// 
    /// # Returns
    /// 
    /// A new builder with the same configuration but no tools
    pub fn clone_config(&self) -> Self {
        Self {
            config: self.config.clone(),
            registry: ToolRegistry::new(),
        }
    }
}

impl Default for McpServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for McpServerBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpServerBuilder")
            .field("config", &self.config)
            .field("tool_count", &self.registry.tool_count())
            .field("tool_names", &self.registry.tool_names())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcp_core::{McpRequest, McpResponse, McpError};
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

    #[test]
    fn test_builder_creation() {
        let builder = McpServerBuilder::new();
        assert_eq!(builder.config().name, "mcp-server");
        assert_eq!(builder.config().version, "0.1.0");
        assert_eq!(builder.tool_count(), 0);
    }

    #[test]
    fn test_builder_configuration() {
        let builder = McpServerBuilder::new()
            .with_name("test-server")
            .with_version("2.0.0")
            .max_concurrent_requests(50)
            .enable_tracing(false);

        assert_eq!(builder.config().name, "test-server");
        assert_eq!(builder.config().version, "2.0.0");
        assert_eq!(builder.config().max_concurrent_requests, 50);
        assert!(!builder.config().enable_tracing);
    }

    #[test]
    fn test_tool_registration() {
        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
        });

        let builder = McpServerBuilder::new()
            .add_tool(tool);

        assert_eq!(builder.tool_count(), 1);
        assert!(builder.has_tool("test_tool"));
    }

    #[test]
    fn test_multiple_tool_registration() {
        let tools = vec![
            Arc::new(MockTool {
                name: "tool1".to_string(),
                description: "First tool".to_string(),
            }) as Arc<dyn McpTool>,
            Arc::new(MockTool {
                name: "tool2".to_string(),
                description: "Second tool".to_string(),
            }) as Arc<dyn McpTool>,
        ];

        let builder = McpServerBuilder::new()
            .add_tools(tools);

        assert_eq!(builder.tool_count(), 2);
        assert!(builder.has_tool("tool1"));
        assert!(builder.has_tool("tool2"));
    }

    #[test]
    fn test_builder_validation() {
        let builder = McpServerBuilder::new()
            .with_name("test-server")
            .with_version("1.0.0");

        assert!(builder.validate().is_ok());

        // Test empty name validation
        let invalid_builder = McpServerBuilder::new()
            .with_name("")
            .with_version("1.0.0");

        assert!(invalid_builder.validate().is_err());

        // Test zero max requests validation
        let invalid_builder2 = McpServerBuilder::new()
            .with_name("test")
            .with_version("1.0.0")
            .max_concurrent_requests(0);

        assert!(invalid_builder2.validate().is_err());
    }

    #[test]
    fn test_builder_clone_config() {
        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
        });

        let builder = McpServerBuilder::new()
            .with_name("test-server")
            .with_version("1.0.0")
            .add_tool(tool);

        let cloned = builder.clone_config();

        // Same configuration
        assert_eq!(cloned.config().name, builder.config().name);
        assert_eq!(cloned.config().version, builder.config().version);

        // But no tools
        assert_eq!(builder.tool_count(), 1);
        assert_eq!(cloned.tool_count(), 0);
    }

    #[test]
    fn test_from_env() {
        // This test would need to set environment variables
        // In a real test environment, you would set these variables first
        let builder = McpServerBuilder::new().from_env();
        // The builder should handle missing env vars gracefully
        assert!(builder.validate().is_ok());
    }
}