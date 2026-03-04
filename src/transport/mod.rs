//! Transport layer for MCP protocol
//!
//! Minimal transport support: stdio + HTTP streaming only.

pub mod stdio;

#[cfg(feature = "http-stream")]
pub mod http_stream;