//! # MCP Transport
//!
//! Transport layer implementations for the Model Context Protocol (MCP) in Rust.
//!
//! This crate provides multiple transport mechanisms for MCP communication:
//! - STDIO transport for pipe-based communication
//! - HTTP transport for RESTful API communication
//!
//! The transport layer is responsible for moving serialized bytes between clients
//! and servers, without knowledge of the actual tool implementations or business logic.
//!
//! # Features
//!
//! - `stdio`: Enable STDIO transport (default)
//! - `http`: Enable HTTP transport (default)
//!
//! # Example
//!
//! ```rust
//! use mcp_transport::{Transport, StdioTransport};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let transport = StdioTransport::new();
//!     // Use transport to handle MCP messages
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod transport;

#[cfg(feature = "stdio")]
pub mod stdio;

#[cfg(feature = "http")]
pub mod http;

pub use error::{TransportError, TransportResult};
pub use transport::Transport;

#[cfg(feature = "stdio")]
pub use stdio::StdioTransport;

#[cfg(feature = "http")]
pub use http::HttpTransport;

/// Transport configuration options
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Connection timeout in milliseconds
    pub timeout_ms: u64,
    /// Enable debug logging
    pub debug: bool,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            max_message_size: 1024 * 1024, // 1MB
            timeout_ms: 30000,             // 30 seconds
            debug: false,
        }
    }
}
