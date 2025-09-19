//! Filesystem server example for MCP
//!
//! This example demonstrates a complete MCP server that provides file system operations
//! using both STDIO and HTTP transports. It showcases the core MCP architecture with
//! a real-world tool implementation.

use anyhow::Result;
use async_trait::async_trait;
use clap::{Parser, ValueEnum};
use mcp_core::{
    McpError, McpRequest, McpResponse, McpServer, McpTool, ResponseResult, ToolContent,
};
use mcp_server::{McpServerBuilder, McpServerImpl};
use mcp_transport::{HttpTransport, StdioTransport, Transport};
use serde_json::Value;

use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tracing::{debug, error, info, warn};

/// Command line arguments
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Transport type to use
    #[arg(short, long, default_value = "stdio")]
    transport: TransportType,

    /// Port for HTTP transport
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Host for HTTP transport
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Base directory for file operations (security constraint)
    #[arg(short, long, default_value = ".")]
    base_dir: String,
}

/// Available transport types
#[derive(Clone, Debug, ValueEnum)]
enum TransportType {
    /// STDIO transport for pipe communication
    Stdio,
    /// HTTP transport for RESTful API
    Http,
}

/// File reading tool implementation
pub struct ReadFileTool {
    /// Base directory for file operations (security constraint)
    base_dir: String,
}

impl ReadFileTool {
    /// Create a new ReadFileTool with the specified base directory
    pub fn new(base_dir: String) -> Self {
        Self { base_dir }
    }

    /// Validate and resolve file path within base directory
    fn resolve_path(&self, requested_path: &str) -> Result<std::path::PathBuf, McpError> {
        let base = Path::new(&self.base_dir)
            .canonicalize()
            .map_err(|e| McpError::internal_error(format!("Invalid base directory: {e}")))?;

        let requested = Path::new(requested_path);

        // Handle relative paths
        let full_path = if requested.is_absolute() {
            requested.to_path_buf()
        } else {
            base.join(requested)
        };

        // First check if the path would be outside base directory (even if it doesn't exist)
        // This prevents path traversal attacks
        if let Ok(canonical) = full_path.canonicalize() {
            if !canonical.starts_with(&base) {
                return Err(McpError::permission_denied(format!(
                    "Path '{}' is outside base directory '{}'",
                    requested_path, self.base_dir
                )));
            }
            Ok(canonical)
        } else {
            // File doesn't exist, but we still need to check if the resolved path would be valid
            // Use the parent directory to check if we're trying to escape
            let mut check_path = full_path.clone();
            while !check_path.exists() && check_path.parent().is_some() {
                check_path = check_path.parent().unwrap().to_path_buf();
            }

            if let Ok(canonical_parent) = check_path.canonicalize() {
                let intended_canonical = canonical_parent
                    .join(full_path.strip_prefix(&check_path).unwrap_or(&full_path));
                if !intended_canonical.starts_with(&base) {
                    return Err(McpError::permission_denied(format!(
                        "Path '{}' is outside base directory '{}'",
                        requested_path, self.base_dir
                    )));
                }
            }

            // If we get here, the path is within bounds but the file doesn't exist
            Err(McpError::resource_not_found(requested_path))
        }
    }
}

#[async_trait]
impl McpTool for ReadFileTool {
    async fn call(&self, request: McpRequest) -> Result<McpResponse, McpError> {
        match request {
            McpRequest::CallTool { name, arguments } => {
                if name != self.name() {
                    return Err(McpError::method_not_found(&name));
                }

                // Extract path parameter
                let path = arguments
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter 'path'"))?;

                debug!("Reading file: {}", path);

                // Resolve and validate path
                let resolved_path = self.resolve_path(path)?;

                // Read file content
                match fs::read_to_string(&resolved_path).await {
                    Ok(content) => {
                        let result = ResponseResult::ToolResult {
                            content: vec![ToolContent::Text { text: content }],
                            is_error: false,
                        };

                        info!("Successfully read file: {}", path);
                        Ok(McpResponse::success(result))
                    }
                    Err(e) => {
                        warn!("Failed to read file '{}': {}", path, e);
                        match e.kind() {
                            std::io::ErrorKind::NotFound => Err(McpError::resource_not_found(path)),
                            std::io::ErrorKind::PermissionDenied => {
                                Err(McpError::permission_denied(format!(
                                    "Permission denied reading file '{path}'"
                                )))
                            }
                            _ => Err(McpError::internal_error(format!(
                                "Failed to read file '{path}': {e}"
                            ))),
                        }
                    }
                }
            }
            _ => Err(McpError::invalid_request("Expected CallTool request")),
        }
    }

    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a text file and return it as a string"
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read (relative to base directory)"
                }
            },
            "required": ["path"]
        })
    }
}

/// Create and configure the MCP server
async fn create_server(base_dir: String) -> Result<McpServerImpl> {
    let read_file_tool = Arc::new(ReadFileTool::new(base_dir)) as Arc<dyn McpTool>;

    let server = McpServerBuilder::new()
        .with_name("filesystem-server")
        .with_version("1.0.0")
        .add_tool(read_file_tool)
        .enable_tracing(true)
        .max_concurrent_requests(10)
        .build()?;

    info!(
        "Created filesystem server with {} tools",
        server.tool_count().await
    );
    Ok(server)
}

/// Run server with STDIO transport
async fn run_with_stdio(server: McpServerImpl) -> Result<()> {
    info!("Starting filesystem server with STDIO transport");

    let transport = StdioTransport::with_defaults()?;
    let transport: Arc<dyn Transport> = Arc::new(transport);

    info!("STDIO transport ready - listening on stdin/stdout");
    info!("Send MCP requests as JSON lines to interact with the server");

    // Simple request loop instead of complex transport integration
    loop {
        match transport.receive_request().await {
            Ok(Some(request)) => {
                let response = server.handle_request(request).await.unwrap_or_else(|e| {
                    error!("Request handling failed: {}", e);
                    McpResponse::error(e)
                });

                if let Err(e) = transport.send_response(response).await {
                    error!("Failed to send response: {}", e);
                    break;
                }
            }
            Ok(None) => {
                info!("Transport closed, stopping server");
                break;
            }
            Err(e) => {
                error!("Transport error: {}", e);
                break;
            }
        }
    }
    Ok(())
}

/// Run server with HTTP transport  
async fn run_with_http(server: McpServerImpl, host: String, port: u16) -> Result<()> {
    let addr = SocketAddr::new(host.parse::<IpAddr>()?, port);
    info!("Starting filesystem server with HTTP transport on {}", addr);

    let transport = HttpTransport::with_defaults(addr)?;

    // Start the HTTP server
    transport.start_server().await?;

    info!("HTTP server running on http://{}", addr);
    info!("Available endpoints:");
    info!("  GET  /health                    - Health check");
    info!("  POST /mcp/tools/call           - Call a tool");
    info!("  GET  /mcp/tools/list           - List available tools");
    info!("");
    info!("Example curl command:");
    info!("  curl -X POST http://{}/mcp/tools/call \\", addr);
    info!("    -H 'Content-Type: application/json' \\");
    info!("    -d '{{\"name\": \"read_file\", \"arguments\": {{\"path\": \"README.md\"}}}}'");

    // Simple request loop for HTTP
    let transport_arc: Arc<dyn Transport> = Arc::new(transport);
    loop {
        match transport_arc.receive_request().await {
            Ok(Some(request)) => {
                let response = server.handle_request(request).await.unwrap_or_else(|e| {
                    error!("Request handling failed: {}", e);
                    McpResponse::error(e)
                });

                if let Err(e) = transport_arc.send_response(response).await {
                    error!("Failed to send response: {}", e);
                    break;
                }
            }
            Ok(None) => {
                info!("Transport closed, stopping server");
                break;
            }
            Err(e) => {
                error!("Transport error: {}", e);
                break;
            }
        }
    }
    Ok(())
}

/// Initialize logging based on debug flag
fn init_logging(debug: bool) {
    use tracing_subscriber::FmtSubscriber;

    let level = if debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(debug)
        .with_line_number(debug)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    init_logging(args.debug);

    info!("Starting MCP Filesystem Server");
    info!("Base directory: {}", args.base_dir);
    info!("Transport: {:?}", args.transport);

    // Validate base directory
    if !Path::new(&args.base_dir).exists() {
        error!("Base directory '{}' does not exist", args.base_dir);
        std::process::exit(1);
    }

    // Create the server
    let server = create_server(args.base_dir).await?;

    // Run with selected transport
    match args.transport {
        TransportType::Stdio => {
            run_with_stdio(server).await?;
        }
        TransportType::Http => {
            run_with_http(server, args.host, args.port).await?;
        }
    }

    info!("Filesystem server shutdown complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use tempfile::TempDir;
    use tokio::fs;

    async fn create_test_tool() -> (ReadFileTool, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let tool = ReadFileTool::new(temp_dir.path().to_string_lossy().to_string());
        (tool, temp_dir)
    }

    #[tokio::test]
    async fn test_read_existing_file() {
        let (tool, temp_dir) = create_test_tool().await;

        // Create test file
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello, World!").await.unwrap();

        // Test reading
        let mut args = HashMap::new();
        args.insert(
            "path".to_string(),
            serde_json::Value::String("test.txt".to_string()),
        );

        let request = McpRequest::CallTool {
            name: "read_file".to_string(),
            arguments: args,
        };

        let response = tool.call(request).await.unwrap();

        match response {
            McpResponse::Success {
                result: ResponseResult::ToolResult { content, is_error },
            } => {
                assert!(!is_error);
                assert_eq!(content.len(), 1);
                match &content[0] {
                    ToolContent::Text { text } => {
                        assert_eq!(text, "Hello, World!");
                    }
                    _ => panic!("Expected text content"),
                }
            }
            _ => panic!("Expected successful tool result"),
        }
    }

    #[tokio::test]
    async fn test_read_nonexistent_file() {
        let (tool, _temp_dir) = create_test_tool().await;

        let mut args = HashMap::new();
        args.insert(
            "path".to_string(),
            serde_json::Value::String("nonexistent.txt".to_string()),
        );

        let request = McpRequest::CallTool {
            name: "read_file".to_string(),
            arguments: args,
        };

        let result = tool.call(request).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.code, mcp_core::McpErrorCode::ResourceNotFound);
    }

    #[tokio::test]
    async fn test_path_traversal_protection() {
        let (tool, _temp_dir) = create_test_tool().await;

        let mut args = HashMap::new();
        args.insert(
            "path".to_string(),
            serde_json::Value::String("../../../etc/passwd".to_string()),
        );

        let request = McpRequest::CallTool {
            name: "read_file".to_string(),
            arguments: args,
        };

        let result = tool.call(request).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.code, mcp_core::McpErrorCode::PermissionDenied);
    }

    #[tokio::test]
    async fn test_missing_path_parameter() {
        let (tool, _temp_dir) = create_test_tool().await;

        let args = HashMap::new(); // No path parameter

        let request = McpRequest::CallTool {
            name: "read_file".to_string(),
            arguments: args,
        };

        let result = tool.call(request).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.code, mcp_core::McpErrorCode::InvalidParams);
    }

    #[test]
    fn test_tool_metadata() {
        let tool = ReadFileTool::new("/tmp".to_string());

        assert_eq!(tool.name(), "read_file");
        assert!(!tool.description().is_empty());

        let schema = tool.input_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].get("path").is_some());
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&serde_json::Value::String("path".to_string())));
    }
}
