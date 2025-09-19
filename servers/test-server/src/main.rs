//! Simple test server using official RMCP SDK
//!
//! This is a minimal MCP server implementation to validate the official SDK integration.
//! It provides basic tools to test the MCP protocol functionality.

use anyhow::Result;
use clap::Parser;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    service::RequestContext,
    tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

/// Command line arguments
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

/// Echo tool arguments
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EchoArgs {
    /// Text to echo back
    pub text: String,
}

/// Simple test server implementation
#[derive(Clone)]
pub struct TestServer {
    /// Counter for demonstration
    counter: Arc<Mutex<i32>>,
    /// Tool router for handling tool calls
    tool_router: ToolRouter<TestServer>,
}

#[tool_router]
impl TestServer {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
            tool_router: Self::tool_router(),
        }
    }

    /// Echo tool - returns the input text with a prefix
    #[tool(description = "Echo back the provided text with a prefix")]
    async fn echo(
        &self,
        Parameters(args): Parameters<EchoArgs>,
    ) -> Result<CallToolResult, McpError> {
        let response_text = format!("Echo from RMCP SDK: {}", args.text);

        Ok(CallToolResult::success(vec![Content::text(response_text)]))
    }

    /// Get current time tool
    #[tool(description = "Get the current UTC time")]
    async fn get_time(&self) -> Result<CallToolResult, McpError> {
        let now = chrono::Utc::now();
        let time_text = format!("Current time: {}", now.format("%Y-%m-%d %H:%M:%S UTC"));

        Ok(CallToolResult::success(vec![Content::text(time_text)]))
    }

    /// Increment counter tool
    #[tool(description = "Increment the internal counter by 1")]
    async fn increment(&self) -> Result<CallToolResult, McpError> {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        let count_text = format!("Counter incremented to: {}", *counter);

        Ok(CallToolResult::success(vec![Content::text(count_text)]))
    }

    /// Get counter value tool
    #[tool(description = "Get the current counter value")]
    async fn get_counter(&self) -> Result<CallToolResult, McpError> {
        let counter = self.counter.lock().await;
        let count_text = format!("Current counter value: {}", *counter);

        Ok(CallToolResult::success(vec![Content::text(count_text)]))
    }

    /// Reset counter tool
    #[tool(description = "Reset the counter to zero")]
    async fn reset_counter(&self) -> Result<CallToolResult, McpError> {
        let mut counter = self.counter.lock().await;
        *counter = 0;

        Ok(CallToolResult::success(vec![Content::text(
            "Counter reset to 0",
        )]))
    }
}

#[tool_handler]
impl ServerHandler for TestServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "This is a test server for validating official RMCP SDK integration. \
                Available tools: echo (echo text), get_time (current UTC time), \
                increment (increment counter), get_counter (get counter value), \
                reset_counter (reset counter to 0)."
                    .to_string(),
            ),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        info!("✅ Test server initialized successfully");
        Ok(self.get_info())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Setup logging
    let log_level = if args.debug { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(format!("test_server={}", log_level).parse()?)
                .add_directive(format!("rmcp={}", log_level).parse()?),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    info!("🚀 Starting Test Server using official RMCP SDK");
    info!("SDK Version: 0.6.3+");
    info!("Available tools: echo, get_time, increment, get_counter, reset_counter");

    // Create server instance
    let server = TestServer::new();

    // Start the server with STDIO transport
    let service = server.serve(stdio()).await.inspect_err(|e| {
        error!("Failed to start server: {:?}", e);
    })?;

    info!("✅ Test server started and ready for MCP connections");
    info!("Connect via MCP client using STDIO transport");

    // Wait for the service to complete
    service.waiting().await?;

    info!("Server shutdown complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let server = TestServer::new();
        let info = server.get_info();

        assert!(info.capabilities.tools.is_some());
        assert!(info.instructions.is_some());
    }

    #[tokio::test]
    async fn test_echo_tool() {
        let server = TestServer::new();
        let args = EchoArgs {
            text: "Hello, RMCP!".to_string(),
        };

        let result = server.echo(Parameters(args)).await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("Echo from RMCP SDK: Hello, RMCP!"));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content in result");
        }
    }

    #[tokio::test]
    async fn test_counter_operations() {
        let server = TestServer::new();

        // Test initial counter value
        let result = server.get_counter().await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        // Test increment
        let result = server.increment().await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        // Test counter value after increment
        let result = server.get_counter().await.unwrap();
        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("Current counter value: 1"));
            }
        }

        // Test reset
        let result = server.reset_counter().await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        // Verify reset
        let result = server.get_counter().await.unwrap();
        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("Current counter value: 0"));
            }
        }
    }

    #[tokio::test]
    async fn test_get_time_tool() {
        let server = TestServer::new();

        let result = server.get_time().await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("Current time:"));
                assert!(text.text.contains("UTC"));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content in result");
        }
    }

    #[tokio::test]
    async fn test_tool_router_functionality() {
        let server = TestServer::new();
        let router = &server.tool_router;

        // Verify tools are registered
        let tools = router.list_all();
        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();

        assert!(tool_names.contains(&"echo"));
        assert!(tool_names.contains(&"get_time"));
        assert!(tool_names.contains(&"increment"));
        assert!(tool_names.contains(&"get_counter"));
        assert!(tool_names.contains(&"reset_counter"));
    }
}
