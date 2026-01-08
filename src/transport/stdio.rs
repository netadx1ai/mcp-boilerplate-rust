use anyhow::Result;
use rmcp::transport::stdio;
use tokio::io::{AsyncRead, AsyncWrite};

pub struct StdioTransport;

impl StdioTransport {
    pub fn new() -> (impl AsyncRead + Unpin, impl AsyncWrite + Unpin) {
        stdio()
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self
    }
}

pub async fn run_stdio_server() -> Result<()> {
    Ok(())
}