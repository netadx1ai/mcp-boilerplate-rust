#![allow(dead_code)]
//! Stdio transport - minimal stub for DTV backend
//!
//! The actual stdio MCP transport is handled by rmcp::transport::stdio()
//! in stdio_server.rs. This module exists only for module structure.

/// Placeholder stdio transport (not used directly -- rmcp handles stdio)
pub struct StdioTransport;

impl StdioTransport {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}