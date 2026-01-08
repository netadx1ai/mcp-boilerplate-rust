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

#[cfg(feature = "websocket")]
pub mod websocket;

#[cfg(feature = "http-stream")]
pub mod http_stream;

#[cfg(feature = "grpc")]
pub mod grpc;

// Re-export core types for convenience
pub use r#trait::{
    Transport, TransportCapabilities, TransportConfig, TransportError,
    TransportFactory, TransportMessage, TransportMetadata, TransportStats,
};
pub use registry::TransportRegistry;
pub use stdio::StdioTransport;

#[cfg(feature = "sse")]
pub use sse::SseTransport;

#[cfg(feature = "websocket")]
pub use websocket::WebSocketTransport;

#[cfg(feature = "http-stream")]
pub use http_stream::HttpStreamTransport;

#[cfg(feature = "grpc")]
pub use grpc::GrpcTransport;

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
    
    // Register WebSocket transport (when feature enabled)
    #[cfg(feature = "websocket")]
    {
        let ws_factory = std::sync::Arc::new(websocket::WebSocketTransportFactory);
        if let Err(e) = registry.register("websocket", ws_factory) {
            eprintln!("Failed to register WebSocket transport: {}", e);
        }
    }
    
    // Register HTTP streaming transport (when feature enabled)
    #[cfg(feature = "http-stream")]
    {
        let http_stream_factory = std::sync::Arc::new(http_stream::HttpStreamTransportFactory);
        if let Err(e) = registry.register("http-stream", http_stream_factory) {
            eprintln!("Failed to register HTTP streaming transport: {}", e);
        }
    }
    
    // Future transports will be registered here:
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
        
        #[cfg(feature = "websocket")]
        assert!(available.contains(&"websocket".to_string()));
        
        #[cfg(feature = "http-stream")]
        assert!(available.contains(&"http-stream".to_string()));
    }
}