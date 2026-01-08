//! Transport layer for MCP protocol
//! 
//! This module provides a unified transport abstraction supporting multiple
//! communication methods:
//! 
//! - **stdio**: Standard input/output (CLI, process spawning)
//! - **SSE**: Server-Sent Events (browser push notifications)
//! - **WebSocket**: Bidirectional real-time communication
//! - **HTTP Streaming**: Large data transfers with chunked encoding
//! - **RPC**: Remote procedure calls (gRPC, custom protocols)
//! 
//! # Architecture
//! 
//! The transport layer uses a trait-based design for extensibility:
//! 
//! ```text
//! Transport Trait (core interface)
//!     ├── StdioTransport (CLI/Desktop apps)
//!     ├── SseTransport (Browser push)
//!     ├── WebSocketTransport (Real-time bidirectional)
//!     ├── HttpStreamTransport (Streaming responses)
//!     └── RpcTransport (gRPC/Custom protocols)
//! ```
//! 
//! # Usage
//! 
//! ```rust,ignore
//! use mcp_boilerplate_rust::transport::{
//!     Transport, TransportRegistry, TransportConfig
//! };
//! 
//! // Create transport from configuration
//! let config = TransportConfig {
//!     transport_type: "stdio".to_string(),
//!     ..Default::default()
//! };
//! 
//! let registry = TransportRegistry::global();
//! let mut transport = registry.create(config)?;
//! 
//! // Initialize and use
//! transport.initialize().await?;
//! transport.send(message).await?;
//! let response = transport.receive().await?;
//! transport.shutdown().await?;
//! ```

pub mod r#trait;
pub mod registry;
pub mod stdio;

#[cfg(feature = "sse")]
pub mod sse;

// Re-export core types for convenience
pub use r#trait::{
    Transport, TransportCapabilities, TransportConfig, TransportError,
    TransportFactory, TransportMessage, TransportMetadata, TransportStats,
};
pub use registry::TransportRegistry;
pub use stdio::StdioTransport;

#[cfg(feature = "sse")]
pub use sse::SseTransport;

/// Initialize the global transport registry with default transports
/// 
/// This function registers all available transport implementations.
/// Call this once at application startup.
pub fn init_registry() {
    let registry = TransportRegistry::global();
    
    // Register stdio transport
    let stdio_factory = std::sync::Arc::new(stdio::StdioTransportFactory);
    if let Err(e) = registry.register("stdio", stdio_factory) {
        eprintln!("Failed to register stdio transport: {}", e);
    }
    
    // Register SSE transport (when feature enabled)
    #[cfg(feature = "sse")]
    {
        let sse_factory = std::sync::Arc::new(sse::SseTransportFactory);
        if let Err(e) = registry.register("sse", sse_factory) {
            eprintln!("Failed to register SSE transport: {}", e);
        }
    }
    
    // Future transports will be registered here:
    // - WebSocket transport (when feature = "websocket" is enabled)
    // - HTTP streaming (when feature = "http-stream" is enabled)
    // - RPC transport (when feature = "rpc" is enabled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_registry() {
        init_registry();
        let registry = TransportRegistry::global();
        assert!(registry.is_registered("stdio"));
    }

    #[test]
    fn test_registry_list_available() {
        init_registry();
        let registry = TransportRegistry::global();
        let available = registry.list_available();
        assert!(!available.is_empty());
        assert!(available.contains(&"stdio".to_string()));
        
        #[cfg(feature = "sse")]
        assert!(available.contains(&"sse".to_string()));
    }
}