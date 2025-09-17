//! # MCP Core
//!
//! Core types and traits for the Model Context Protocol (MCP) implementation in Rust.
//!
//! This crate provides the fundamental building blocks for implementing MCP servers:
//! - Protocol message types (`McpRequest`, `McpResponse`, `McpError`)
//! - Core traits (`McpTool`, `McpServer`)
//! - Common utilities and error handling
//!
//! The design follows the principle that this crate defines the "what" (protocol structure)
//! but not the "how" (transport or implementation details).
//!
//! # Example
//!
//! ```rust
//! use mcp_core::{McpTool, McpRequest, McpResponse, ResponseResult, ToolContent};
//! use async_trait::async_trait;
//!
//! struct ExampleTool;
//!
//! #[async_trait]
//! impl McpTool for ExampleTool {
//!     async fn call(&self, request: McpRequest) -> Result<McpResponse, mcp_core::McpError> {
//!         // Tool implementation here
//!         let result = ResponseResult::ToolResult {
//!             content: vec![ToolContent::Text {
//!                 text: "Hello from tool".to_string()
//!             }],
//!             is_error: false,
//!         };
//!         Ok(McpResponse::success(result))
//!     }
//!     
//!     fn name(&self) -> &str {
//!         "example_tool"
//!     }
//!     
//!     fn description(&self) -> &str {
//!         "An example tool implementation"
//!     }
//! }
//! ```

pub mod error;
pub mod messages;
pub mod traits;

pub use error::{McpError, McpErrorCode};
pub use messages::{
    ClientCapabilities, ClientInfo, McpRequest, McpResponse, ResponseResult, ServerCapabilities,
    ServerInfo, Tool, ToolCall, ToolContent, ToolInputSchema, ToolResult,
};
pub use traits::{McpServer, McpTool};

/// MCP protocol version supported by this implementation
pub const MCP_VERSION: &str = "2024-11-05";

/// Standard result type for MCP operations
pub type McpResult<T> = Result<T, McpError>;
