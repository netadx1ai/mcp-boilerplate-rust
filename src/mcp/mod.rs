pub mod protocol_handler;
pub mod stdio_server;

#[cfg(feature = "sse")]
pub mod sse_server;

#[cfg(feature = "websocket")]
pub mod websocket_server;

#[cfg(feature = "http-stream")]
pub mod http_stream_server;

#[cfg(feature = "grpc")]
pub mod grpc_server;

pub use stdio_server::McpServer;

#[cfg(feature = "sse")]
pub use sse_server::run_sse_server;

#[cfg(feature = "websocket")]
pub use websocket_server::create_router as create_websocket_router;

#[cfg(feature = "http-stream")]
pub use http_stream_server::run_http_stream_server;

#[cfg(feature = "grpc")]
pub use grpc_server::run_grpc_server;
