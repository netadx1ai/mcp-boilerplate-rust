# MCP Boilerplate Rust - API Documentation

A comprehensive guide to the Model Context Protocol (MCP) implementation in Rust, covering all core APIs, transport layers, and usage patterns.

## Table of Contents

- [Core Protocol API](#core-protocol-api)
- [Transport Layer APIs](#transport-layer-apis)
- [Server Framework API](#server-framework-api)
- [Tool Development API](#tool-development-api)
- [HTTP Endpoints](#http-endpoints)
- [Error Handling](#error-handling)
- [Integration Guide](#integration-guide)

## Core Protocol API

### McpRequest

The core request type for all MCP operations.

```rust
pub enum McpRequest {
    CallTool { name: String, arguments: HashMap<String, Value> },
    ListTools,
    Ping,
    Initialize { version: String },
}
```

**Usage:**
```rust
use mcp_core::McpRequest;

// Call a tool
let request = McpRequest::CallTool {
    name: "read_file".to_string(),
    arguments: [("path".to_string(), "README.md".into())].into(),
};

// List available tools
let request = McpRequest::ListTools;
```

### McpResponse

The core response type for all MCP operations.

```rust
pub enum McpResponse {
    Success { result: ResponseResult },
    Error { error: McpError },
}
```

**Response Types:**
```rust
pub enum ResponseResult {
    ToolResult {
        content: Vec<ToolContent>,
        is_error: bool,
    },
    ToolList {
        tools: Vec<Tool>,
        next_cursor: Option<String>,
    },
    Pong,
    Initialize { version: String, capabilities: Value },
}
```

### McpError

Standardized error handling for MCP operations.

```rust
pub struct McpError {
    pub code: McpErrorCode,
    pub message: String,
    pub data: Option<Value>,
}

pub enum McpErrorCode {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    ResourceNotFound = -32001,
    PermissionDenied = -32002,
    Timeout = -32003,
}
```

**Creating Errors:**
```rust
use mcp_core::{McpError, McpErrorCode};

// Resource not found
let error = McpError::resource_not_found("file.txt");

// Permission denied
let error = McpError::permission_denied("Access denied to /etc/passwd");

// Custom error
let error = McpError::new(
    McpErrorCode::InternalError,
    "Database connection failed".to_string(),
    Some(json!({"retry_after": 30}))
);
```

## Transport Layer APIs

### Transport Trait

Common interface for all transport implementations.

```rust
#[async_trait]
pub trait Transport: Send + Sync {
    async fn send_request(&self, request: McpRequest) -> Result<(), TransportError>;
    async fn receive_request(&self) -> Result<Option<McpRequest>, TransportError>;
    async fn send_response(&self, response: McpResponse) -> Result<(), TransportError>;
    async fn receive_response(&self) -> Result<Option<McpResponse>, TransportError>;
    async fn close(&self) -> Result<(), TransportError>;
}
```

### STDIO Transport

Pipe-based communication for command-line integration.

```rust
use mcp_transport::StdioTransport;

// Create STDIO transport
let transport = StdioTransport::with_defaults()?;

// Send request
transport.send_request(McpRequest::ListTools).await?;

// Receive response
let response = transport.receive_response().await?;
```

**Configuration Options:**
```rust
let transport = StdioTransport::builder()
    .buffer_size(8192)
    .timeout(Duration::from_secs(30))
    .build()?;
```

### HTTP Transport

RESTful HTTP transport for web integration.

```rust
use mcp_transport::HttpTransport;
use std::net::SocketAddr;

// Create HTTP transport
let addr = "127.0.0.1:3000".parse::<SocketAddr>()?;
let transport = HttpTransport::with_defaults(addr)?;

// Start HTTP server
transport.start_server().await?;
```

**Configuration Options:**
```rust
let transport = HttpTransport::builder(addr)
    .timeout(Duration::from_secs(60))
    .max_connections(1000)
    .cors_enabled(true)
    .build()?;
```

## Server Framework API

### McpServerBuilder

Fluent builder for MCP server configuration.

```rust
use mcp_server::{McpServerBuilder, McpServerImpl};
use std::sync::Arc;

let server = McpServerBuilder::new()
    .with_name("my-server")
    .with_version("1.0.0")
    .add_tool(Arc::new(MyTool::default()))
    .enable_tracing(true)
    .max_concurrent_requests(100)
    .request_timeout(Duration::from_secs(30))
    .build()?;
```

**Builder Methods:**

| Method | Description | Default |
|--------|-------------|---------|
| `with_name(name)` | Set server name | "mcp-server" |
| `with_version(version)` | Set server version | "1.0.0" |
| `add_tool(tool)` | Register a tool | - |
| `enable_tracing(bool)` | Enable request tracing | false |
| `max_concurrent_requests(n)` | Set concurrency limit | 10 |
| `request_timeout(duration)` | Set request timeout | 30s |

### McpServerImpl

Core server implementation handling request routing.

```rust
// Handle individual request
let response = server.handle_request(request).await?;

// Get server statistics
let stats = server.stats().await;

// Get tool count
let count = server.tool_count().await;

// List registered tools
let tools = server.list_tools().await;
```

## Tool Development API

### McpTool Trait

Core trait for implementing MCP tools.

```rust
#[async_trait]
pub trait McpTool: Send + Sync {
    async fn call(&self, request: McpRequest) -> Result<McpResponse, McpError>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
}
```

### Tool Implementation Example

```rust
use async_trait::async_trait;
use mcp_core::{McpTool, McpRequest, McpResponse, McpError};
use serde_json::Value;

#[derive(Default)]
pub struct CalculatorTool;

#[async_trait]
impl McpTool for CalculatorTool {
    async fn call(&self, request: McpRequest) -> Result<McpResponse, McpError> {
        match request {
            McpRequest::CallTool { name, arguments } => {
                if name != self.name() {
                    return Err(McpError::method_not_found(&name));
                }
                
                let a = arguments.get("a")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| McpError::invalid_params("Missing parameter 'a'"))?;
                
                let b = arguments.get("b")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| McpError::invalid_params("Missing parameter 'b'"))?;
                
                let operation = arguments.get("operation")
                    .and_then(|v| v.as_str())
                    .unwrap_or("add");
                
                let result = match operation {
                    "add" => a + b,
                    "subtract" => a - b,
                    "multiply" => a * b,
                    "divide" => {
                        if b == 0.0 {
                            return Err(McpError::invalid_params("Division by zero"));
                        }
                        a / b
                    }
                    _ => return Err(McpError::invalid_params("Invalid operation")),
                };
                
                Ok(McpResponse::success(ResponseResult::ToolResult {
                    content: vec![ToolContent::Text {
                        text: result.to_string(),
                    }],
                    is_error: false,
                }))
            }
            _ => Err(McpError::invalid_request("Expected CallTool request")),
        }
    }
    
    fn name(&self) -> &str {
        "calculator"
    }
    
    fn description(&self) -> &str {
        "Perform basic arithmetic operations"
    }
    
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "a": {
                    "type": "number",
                    "description": "First operand"
                },
                "b": {
                    "type": "number", 
                    "description": "Second operand"
                },
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"],
                    "description": "Operation to perform",
                    "default": "add"
                }
            },
            "required": ["a", "b"]
        })
    }
}
```

### Tool Content Types

```rust
pub enum ToolContent {
    Text { text: String },
    Image { url: String, alt_text: Option<String> },
    Resource { uri: String, mime_type: Option<String> },
}
```

## HTTP Endpoints

### Standard Endpoints

All HTTP transport implementations provide these standard endpoints:

#### Health Check
```http
GET /health
```
**Response:**
```
OK
```

#### List Tools
```http
GET /mcp/tools/list
```
**Response:**
```json
{
  "tools": [
    {
      "name": "read_file",
      "description": "Read the contents of a text file",
      "inputSchema": {
        "type": "object",
        "properties": {
          "path": {
            "type": "string",
            "description": "Path to the file to read"
          }
        },
        "required": ["path"]
      }
    }
  ],
  "next_cursor": null
}
```

#### Call Tool
```http
POST /mcp/tools/call
Content-Type: application/json

{
  "name": "read_file",
  "arguments": {
    "path": "README.md"
  }
}
```
**Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "# My Project\n\nThis is a sample file..."
    }
  ],
  "isError": false
}
```

#### Generic MCP Request
```http
POST /mcp/request
Content-Type: application/json

{
  "method": "tools/call",
  "params": {
    "name": "read_file",
    "arguments": {
      "path": "README.md"
    }
  }
}
```

### HTTP Status Codes

| Status Code | Description | When Used |
|-------------|-------------|-----------|
| 200 OK | Success | Successful request |
| 400 Bad Request | Invalid request | Malformed JSON, missing parameters |
| 404 Not Found | Resource not found | Unknown endpoint, missing file |
| 405 Method Not Allowed | Invalid HTTP method | Wrong HTTP verb for endpoint |
| 429 Too Many Requests | Rate limited | Too many requests |
| 500 Internal Server Error | Server error | Internal server failure |
| 503 Service Unavailable | Server overloaded | Server at capacity |

## Error Handling

### Error Response Format

All errors follow a consistent JSON-RPC 2.0 format:

```json
{
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "parameter": "path",
      "expected": "string",
      "received": "null"
    }
  }
}
```

### Common Error Codes

| Code | Name | Description |
|------|------|-------------|
| -32700 | Parse Error | Invalid JSON |
| -32600 | Invalid Request | Invalid request object |
| -32601 | Method Not Found | Unknown method/tool |
| -32602 | Invalid Params | Invalid parameters |
| -32603 | Internal Error | Server error |
| -32001 | Resource Not Found | Resource doesn't exist |
| -32002 | Permission Denied | Access denied |
| -32003 | Timeout | Operation timed out |

### Error Handling Best Practices

```rust
// In tool implementations
fn validate_params(args: &HashMap<String, Value>) -> Result<(), McpError> {
    if !args.contains_key("required_param") {
        return Err(McpError::invalid_params("Missing required parameter 'required_param'"));
    }
    Ok(())
}

// In server code
async fn handle_request_safely(request: McpRequest) -> McpResponse {
    match handle_request(request).await {
        Ok(response) => response,
        Err(e) => {
            error!("Request failed: {}", e);
            McpResponse::error(e)
        }
    }
}
```

## Integration Guide

### Basic Server Setup

```rust
use mcp_core::McpTool;
use mcp_server::McpServerBuilder;
use mcp_transport::{HttpTransport, StdioTransport};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create server with tools
    let server = McpServerBuilder::new()
        .with_name("my-server")
        .add_tool(Arc::new(MyTool::default()) as Arc<dyn McpTool>)
        .build()?;
    
    // Choose transport
    if use_http {
        let transport = HttpTransport::with_defaults("127.0.0.1:3000".parse()?)?;
        transport.start_server().await?;
        
        // Handle requests
        let transport_arc: Arc<dyn Transport> = Arc::new(transport);
        loop {
            if let Some(request) = transport_arc.receive_request().await? {
                let response = server.handle_request(request).await
                    .unwrap_or_else(|e| McpResponse::error(e));
                transport_arc.send_response(response).await?;
            }
        }
    } else {
        let transport = StdioTransport::with_defaults()?;
        let transport_arc: Arc<dyn Transport> = Arc::new(transport);
        
        // Handle requests
        loop {
            if let Some(request) = transport_arc.receive_request().await? {
                let response = server.handle_request(request).await
                    .unwrap_or_else(|e| McpResponse::error(e));
                transport_arc.send_response(response).await?;
            }
        }
    }
}
```

### Client Usage (HTTP)

```bash
# List available tools
curl -X GET http://localhost:3000/mcp/tools/list

# Call a tool
curl -X POST http://localhost:3000/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my_tool",
    "arguments": {
      "param1": "value1",
      "param2": 42
    }
  }'
```

### Client Usage (STDIO)

```bash
# Run server in background
./my-server --transport stdio &

# Send JSON request
echo '{"method": "tools/call", "params": {"name": "my_tool", "arguments": {"param": "value"}}}' | ./my-server --transport stdio
```

### Testing Tools

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_tool_call() {
        let tool = MyTool::default();
        
        let mut args = HashMap::new();
        args.insert("param".to_string(), serde_json::Value::String("test".to_string()));
        
        let request = McpRequest::CallTool {
            name: tool.name().to_string(),
            arguments: args,
        };
        
        let response = tool.call(request).await.unwrap();
        
        match response {
            McpResponse::Success { result } => {
                // Assert success
            }
            McpResponse::Error { error } => {
                panic!("Unexpected error: {}", error.message);
            }
        }
    }
}
```

### Performance Considerations

1. **Concurrency**: Use `max_concurrent_requests` to limit server load
2. **Timeouts**: Set appropriate request timeouts for long-running tools
3. **Memory**: Monitor memory usage for tools that process large data
4. **Rate Limiting**: Implement rate limiting for production deployments
5. **Caching**: Cache expensive computations where appropriate

### Security Considerations

1. **Input Validation**: Always validate tool parameters
2. **Path Traversal**: Prevent access outside allowed directories
3. **Resource Limits**: Limit memory and CPU usage per request
4. **Authentication**: Implement authentication for sensitive tools
5. **Content Filtering**: Filter inappropriate content in AI tools

## Version Compatibility

| Version | Features | Breaking Changes |
|---------|----------|------------------|
| 0.1.0 | Core protocol, STDIO/HTTP transports | Initial release |
| 0.2.0 | Enhanced error handling, tool metadata | Tool trait signature |
| 1.0.0 | Stable API, production ready | None (backward compatible) |

For the latest API changes, see [CHANGELOG.md](CHANGELOG.md).