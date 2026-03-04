pub mod protocol_handler;
pub mod stdio_server;

#[cfg(feature = "http-stream")]
pub mod http_stream_server;

pub use stdio_server::McpServer;

#[cfg(feature = "http-stream")]
pub use http_stream_server::run_http_stream_server;