//! Core traits for the MCP protocol implementation.

use crate::{McpError, McpRequest, McpResponse};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Trait for implementing MCP tools
///
/// This trait defines the interface that all MCP tools must implement.
/// Tools are discrete units of business logic that can be called by clients
/// through the MCP protocol.
///
/// # Example
///
/// ```rust
/// use mcp_core::{McpTool, McpRequest, McpResponse, McpError};
/// use async_trait::async_trait;
///
/// struct EchoTool;
///
/// #[async_trait]
/// impl McpTool for EchoTool {
///     async fn call(&self, request: McpRequest) -> Result<McpResponse, McpError> {
///         match request {
///             McpRequest::CallTool { name, arguments } => {
///                 let echo_text = arguments.get("text")
///                     .and_then(|v| v.as_str())
///                     .unwrap_or("No text provided");
///                 
///                 Ok(McpResponse::simple_success(format!("Echo: {}", echo_text)))
///             },
///             _ => Err(McpError::invalid_request("Expected CallTool request")),
///         }
///     }
///     
///     fn name(&self) -> &str {
///         "echo"
///     }
///     
///     fn description(&self) -> &str {
///         "Echoes the input text back to the caller"
///     }
/// }
/// ```
#[async_trait]
pub trait McpTool: Send + Sync {
    /// Execute the tool with the given request
    ///
    /// This method contains the core logic of the tool. It should handle
    /// the request parameters, perform the necessary operations, and return
    /// an appropriate response.
    ///
    /// # Arguments
    ///
    /// * `request` - The MCP request containing tool parameters
    ///
    /// # Returns
    ///
    /// A `Result` containing either a successful `McpResponse` or an `McpError`
    async fn call(&self, request: McpRequest) -> Result<McpResponse, McpError>;

    /// Get the name of this tool
    ///
    /// The name must be unique within a server instance and should follow
    /// naming conventions (lowercase, underscores for separation).
    ///
    /// # Returns
    ///
    /// The tool's name as a string slice
    fn name(&self) -> &str;

    /// Get a human-readable description of this tool
    ///
    /// This description is used in tool listings and documentation to help
    /// users understand what the tool does.
    ///
    /// # Returns
    ///
    /// The tool's description as a string slice
    fn description(&self) -> &str;

    /// Get the input schema for this tool
    ///
    /// This method returns a JSON schema describing the expected input format
    /// for the tool. The default implementation returns a generic object schema.
    ///
    /// # Returns
    ///
    /// A `Value` containing the JSON schema for the tool's input
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "additionalProperties": true
        })
    }

    /// Get additional metadata for this tool
    ///
    /// This method allows tools to provide additional metadata that might
    /// be useful for clients or server management. The default implementation
    /// returns an empty HashMap.
    ///
    /// # Returns
    ///
    /// A HashMap containing metadata key-value pairs
    fn metadata(&self) -> HashMap<String, Value> {
        HashMap::new()
    }

    /// Check if this tool supports a specific capability
    ///
    /// This method allows tools to declare support for optional capabilities
    /// or extensions. The default implementation returns false for all capabilities.
    ///
    /// # Arguments
    ///
    /// * `capability` - The name of the capability to check
    ///
    /// # Returns
    ///
    /// `true` if the tool supports the capability, `false` otherwise
    fn supports_capability(&self, _capability: &str) -> bool {
        false
    }
}

/// Trait for implementing MCP servers
///
/// This trait defines the interface for MCP server implementations.
/// Servers are responsible for managing tools, handling protocol messages,
/// and coordinating communication with clients.
///
/// # Example
///
/// ```rust
/// use mcp_core::{McpServer, McpRequest, McpResponse, McpError, McpTool};
/// use async_trait::async_trait;
/// use std::sync::Arc;
///
/// struct SimpleServer {
///     tools: Vec<Arc<dyn McpTool>>,
/// }
///
/// #[async_trait]
/// impl McpServer for SimpleServer {
///     async fn handle_request(&self, request: McpRequest) -> Result<McpResponse, McpError> {
///         match request {
///             McpRequest::ListTools { .. } => {
///                 // Return list of available tools
///                 todo!("Implement tool listing")
///             },
///             McpRequest::CallTool { name, .. } => {
///                 // Find and call the appropriate tool
///                 todo!("Implement tool calling")
///             },
///             _ => Err(McpError::method_not_found("Unsupported method")),
///         }
///     }
///     
///     async fn initialize(&self) -> Result<(), McpError> {
///         // Server initialization logic
///         Ok(())
///     }
///     
///     async fn shutdown(&self) -> Result<(), McpError> {
///         // Server cleanup logic
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait McpServer: Send + Sync {
    /// Handle an incoming MCP request
    ///
    /// This is the main entry point for processing MCP protocol messages.
    /// The server should route the request to the appropriate handler and
    /// return a valid response.
    ///
    /// # Arguments
    ///
    /// * `request` - The incoming MCP request to handle
    ///
    /// # Returns
    ///
    /// A `Result` containing either a successful `McpResponse` or an `McpError`
    async fn handle_request(&self, request: McpRequest) -> Result<McpResponse, McpError>;

    /// Initialize the server
    ///
    /// This method is called when the server is starting up and should
    /// perform any necessary initialization tasks.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of initialization
    async fn initialize(&self) -> Result<(), McpError> {
        Ok(())
    }

    /// Shutdown the server
    ///
    /// This method is called when the server is shutting down and should
    /// perform any necessary cleanup tasks.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of shutdown
    async fn shutdown(&self) -> Result<(), McpError> {
        Ok(())
    }

    /// Get server information
    ///
    /// This method returns basic information about the server implementation.
    /// The default implementation returns generic server information.
    ///
    /// # Returns
    ///
    /// A tuple containing the server name and version
    fn server_info(&self) -> (&str, &str) {
        ("mcp-server", "0.1.0")
    }

    /// Get server capabilities
    ///
    /// This method returns the capabilities supported by this server instance.
    /// The default implementation returns basic tool support.
    ///
    /// # Returns
    ///
    /// A HashMap containing capability declarations
    fn capabilities(&self) -> HashMap<String, Value> {
        let mut caps = HashMap::new();
        caps.insert("tools".to_string(), serde_json::json!({}));
        caps
    }

    /// Check if the server is healthy
    ///
    /// This method allows external systems to check the health status of the server.
    /// The default implementation always returns true.
    ///
    /// # Returns
    ///
    /// `true` if the server is healthy, `false` otherwise
    async fn health_check(&self) -> bool {
        true
    }

    /// Get server metrics
    ///
    /// This method returns runtime metrics about the server's performance
    /// and usage. The default implementation returns an empty HashMap.
    ///
    /// # Returns
    ///
    /// A HashMap containing metric key-value pairs
    async fn metrics(&self) -> HashMap<String, Value> {
        HashMap::new()
    }
}

/// Trait for MCP transport implementations
///
/// This trait defines the interface for transport layer implementations
/// that handle the actual communication between MCP clients and servers.
#[async_trait]
pub trait McpTransport: Send + Sync {
    /// Send an MCP response
    ///
    /// # Arguments
    ///
    /// * `response` - The response to send
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the send operation
    async fn send_response(&self, response: McpResponse) -> Result<(), McpError>;

    /// Receive an MCP request
    ///
    /// # Returns
    ///
    /// A `Result` containing either a received `McpRequest` or an `McpError`
    async fn receive_request(&self) -> Result<Option<McpRequest>, McpError>;

    /// Check if the transport is connected
    ///
    /// # Returns
    ///
    /// `true` if the transport is connected, `false` otherwise
    fn is_connected(&self) -> bool;

    /// Close the transport connection
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the close operation
    async fn close(&self) -> Result<(), McpError>;
}

/// Trait for tool registry implementations
///
/// This trait defines the interface for managing collections of MCP tools
/// within a server implementation.
pub trait ToolRegistry: Send + Sync {
    /// Register a new tool
    ///
    /// # Arguments
    ///
    /// * `tool` - The tool to register
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of registration
    fn register_tool(&mut self, tool: Arc<dyn McpTool>) -> Result<(), McpError>;

    /// Unregister a tool by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tool to unregister
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of unregistration
    fn unregister_tool(&mut self, name: &str) -> Result<(), McpError>;

    /// Get a tool by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tool to retrieve
    ///
    /// # Returns
    ///
    /// An `Option` containing the tool if found
    fn get_tool(&self, name: &str) -> Option<Arc<dyn McpTool>>;

    /// List all registered tools
    ///
    /// # Returns
    ///
    /// A vector of all registered tools
    fn list_tools(&self) -> Vec<Arc<dyn McpTool>>;

    /// Get the number of registered tools
    ///
    /// # Returns
    ///
    /// The count of registered tools
    fn tool_count(&self) -> usize;

    /// Check if a tool is registered
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tool to check
    ///
    /// # Returns
    ///
    /// `true` if the tool is registered, `false` otherwise
    fn has_tool(&self, name: &str) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

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
    async fn test_mock_tool() {
        let tool = MockTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
        };

        assert_eq!(tool.name(), "test_tool");
        assert_eq!(tool.description(), "A test tool");

        let request = McpRequest::Ping;
        let response = tool.call(request).await.unwrap();

        match response {
            McpResponse::Success { .. } => (),
            _ => panic!("Expected success response"),
        }
    }

    #[test]
    fn test_tool_defaults() {
        let tool = MockTool {
            name: "test".to_string(),
            description: "test".to_string(),
        };

        // Test default implementations
        let schema = tool.input_schema();
        assert_eq!(schema["type"], "object");

        let metadata = tool.metadata();
        assert!(metadata.is_empty());

        assert!(!tool.supports_capability("unknown"));
    }
}
