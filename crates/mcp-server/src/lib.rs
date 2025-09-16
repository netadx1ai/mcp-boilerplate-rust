//! # MCP Server
//!
//! Server framework for the Model Context Protocol (MCP) implementation in Rust.
//!
//! This crate provides the orchestration layer that connects MCP tools with transport
//! mechanisms. It handles request routing, tool management, and response coordination.
//!
//! The server acts as a simple orchestrator - its sole purpose is to manage tools
//! and delegate incoming calls to the appropriate tool implementations.
//!
//! # Architecture
//!
//! - `McpServerBuilder`: Fluent builder for server configuration
//! - `McpServerImpl`: Core server implementation handling request routing
//! - Tool registry: Dynamic tool registration and lookup
//! - Transport integration: Works with any transport implementing the Transport trait
//!
//! # Example
//!
//! ```rust
//! use mcp_server::{McpServerBuilder, McpServerImpl};
//! use mcp_core::McpTool;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let server = McpServerBuilder::new()
//!         .with_name("example-server")
//!         .with_version("1.0.0")
//!         .add_tool(Box::new(my_tool))
//!         .build();
//!         
//!     // Use server with transport
//!     Ok(())
//! }
//! ```

pub mod builder;
pub mod error;
pub mod registry;
pub mod server;

pub use builder::McpServerBuilder;
pub use error::{ServerError, ServerResult};
pub use registry::ToolRegistry;
pub use server::McpServerImpl;

/// Re-export core types for convenience
pub use mcp_core::{McpError, McpRequest, McpResponse, McpServer, McpTool};
pub use mcp_transport::{Transport, TransportConfig};

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
    /// Enable request tracing
    pub enable_tracing: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: "mcp-server".to_string(),
            version: "0.1.0".to_string(),
            max_concurrent_requests: 100,
            enable_tracing: true,
        }
    }
}
