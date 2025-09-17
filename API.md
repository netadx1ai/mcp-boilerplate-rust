# MCP Boilerplate Rust - API Documentation

A comprehensive guide to the Model Context Protocol (MCP) implementation in Rust, covering all core APIs, transport layers, AI integration, and production-ready usage patterns.

## Table of Contents

- [Core Protocol API](#core-protocol-api)
- [Transport Layer APIs](#transport-layer-apis)
- [Server Framework API](#server-framework-api)
- [Tool Development API](#tool-development-api)
- [HTTP Endpoints](#http-endpoints)
- [AI Integration API](#ai-integration-api)
- [Error Handling](#error-handling)
- [Integration Guide](#integration-guide)
- [Production Deployment](#production-deployment)

## Core Protocol API

### McpRequest

The core request type for all MCP operations, fully compliant with MCP specification.

```rust
pub enum McpRequest {
    CallTool { name: String, arguments: HashMap<String, Value> },
    ListTools,
    Ping,
    Initialize { version: String, capabilities: ClientCapabilities, client_info: ClientInfo },
    Initialized,
}
```

**Usage Examples:**
```rust
use mcp_core::McpRequest;
use std::collections::HashMap;

// Call a tool
let mut args = HashMap::new();
args.insert("path".to_string(), serde_json::json!("README.md"));
let request = McpRequest::CallTool {
    name: "read_file".to_string(),
    arguments: args,
};

// List available tools
let request = McpRequest::ListTools;

// Initialize connection
let request = McpRequest::Initialize {
    version: "2024-11-05".to_string(),
    capabilities: ClientCapabilities::default(),
    client_info: ClientInfo {
        name: "my-client".to_string(),
        version: "1.0.0".to_string(),
    },
};
```

### McpResponse

The core response type for all MCP operations with rich content support.

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
    Initialize { 
        version: String, 
        capabilities: ServerCapabilities, 
        server_info: ServerInfo 
    },
    Success { message: String },
}
```

**Content Types:**
```rust
pub enum ToolContent {
    Text { text: String },
    Image { 
        data: String,        // Base64 encoded image data
        mime_type: String,   // e.g., "image/png"
        alt_text: Option<String>,
    },
    Resource { 
        uri: String, 
        mime_type: Option<String>,
        metadata: Option<Value>,
    },
}
```

### Tool Schema Definition

```rust
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: ToolInputSchema,
}

pub struct ToolInputSchema {
    pub type_: String,
    pub properties: HashMap<String, Value>,
    pub required: Vec<String>,
    pub additional_properties: Option<bool>,
}
```

## Transport Layer APIs

### Transport Trait

Common interface for all transport implementations with full async support.

```rust
#[async_trait]
pub trait Transport: Send + Sync {
    async fn send_request(&self, request: McpRequest) -> Result<(), TransportError>;
    async fn receive_request(&self) -> Result<Option<McpRequest>, TransportError>;
    async fn send_response(&self, response: McpResponse) -> Result<(), TransportError>;
    async fn receive_response(&self) -> Result<Option<McpResponse>, TransportError>;
    async fn close(&self) -> Result<(), TransportError>;
    
    // Metadata methods
    fn transport_type(&self) -> &str;
    fn is_connected(&self) -> bool;
    fn metadata(&self) -> TransportMetadata;
}
```

### STDIO Transport

High-performance pipe-based communication for command-line integration.

```rust
use mcp_transport::StdioTransport;

// Create with defaults
let transport = StdioTransport::with_defaults()?;

// Custom configuration
let transport = StdioTransport::builder()
    .buffer_size(16384)
    .timeout(Duration::from_secs(60))
    .max_message_size(1024 * 1024)
    .build()?;

// Send and receive
transport.send_request(McpRequest::ListTools).await?;
let response = transport.receive_response().await?;
```

### HTTP Transport

Production-ready HTTP transport with comprehensive endpoint support.

```rust
use mcp_transport::HttpTransport;
use std::net::SocketAddr;

// Create HTTP transport
let addr = "127.0.0.1:3000".parse::<SocketAddr>()?;
let transport = HttpTransport::with_defaults(addr)?;

// Advanced configuration
let transport = HttpTransport::builder(addr)
    .timeout(Duration::from_secs(120))
    .max_connections(1000)
    .cors_enabled(true)
    .enable_compression(true)
    .max_request_size(10 * 1024 * 1024) // 10MB
    .build()?;

// Start server
transport.start_server().await?;
```

## Server Framework API

### McpServerBuilder

Fluent builder for production-ready MCP server configuration.

```rust
use mcp_server::{McpServerBuilder, McpServerImpl};
use std::sync::Arc;
use std::time::Duration;

let server = McpServerBuilder::new()
    .with_name("production-server")
    .with_version("1.0.0")
    .add_tool(Arc::new(ReadFileTool::default()))
    .add_tool(Arc::new(GenerateImageTool::default()))
    .enable_tracing(true)
    .max_concurrent_requests(100)
    .request_timeout(Duration::from_secs(30))
    .build()?;
```

**Builder Configuration:**

| Method | Description | Default | Production Rec. |
|--------|-------------|---------|-----------------|
| `with_name(name)` | Set server name | "mcp-server" | Use descriptive name |
| `with_version(version)` | Set server version | "0.1.0" | Use semantic versioning |
| `add_tool(tool)` | Register a tool | - | Add all required tools |
| `enable_tracing(bool)` | Enable request tracing | false | true for production |
| `max_concurrent_requests(n)` | Set concurrency limit | 10 | 100+ for production |
| `request_timeout(duration)` | Set request timeout | 30s | 60s+ for AI tools |

### Server Operations

```rust
// Handle requests
let response = server.handle_request(request).await?;

// Server lifecycle
server.start().await?;
let is_running = server.is_running().await;
server.stop().await?;

// Server statistics
let stats = server.stats().await;
println!("Tools: {}, Requests: {}", stats.tool_count, stats.request_count);

// Tool management
let tools = server.list_tools().await;
let tool_count = server.tool_count().await;
```

## Tool Development API

### McpTool Trait

Enhanced trait for implementing production-ready MCP tools.

```rust
#[async_trait]
pub trait McpTool: Send + Sync {
    async fn call(&self, request: McpRequest) -> Result<McpResponse, McpError>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
}
```

### File System Tool Example

```rust
use async_trait::async_trait;
use mcp_core::{McpTool, McpRequest, McpResponse, ResponseResult, ToolContent, McpError};
use std::path::Path;

pub struct ReadFileTool {
    base_dir: String,
}

#[async_trait]
impl McpTool for ReadFileTool {
    async fn call(&self, request: McpRequest) -> Result<McpResponse, McpError> {
        match request {
            McpRequest::CallTool { name, arguments } => {
                if name != self.name() {
                    return Err(McpError::method_not_found(&name));
                }
                
                let path = arguments.get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing parameter 'path'"))?;
                
                // Security: validate path is within base directory
                let resolved_path = self.resolve_path(path)?;
                
                // Read file with proper error handling
                match tokio::fs::read_to_string(&resolved_path).await {
                    Ok(content) => {
                        let result = ResponseResult::ToolResult {
                            content: vec![ToolContent::Text { text: content }],
                            is_error: false,
                        };
                        Ok(McpResponse::success(result))
                    }
                    Err(e) => Err(McpError::from_io_error(e, path)),
                }
            }
            _ => Err(McpError::invalid_request("Expected CallTool request")),
        }
    }
    
    fn name(&self) -> &str { "read_file" }
    fn description(&self) -> &str { "Read the contents of a text file" }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read"
                }
            },
            "required": ["path"]
        })
    }
}
```

### AI-Powered Tool Example

```rust
pub struct GenerateImageTool {
    provider: String,
    client: reqwest::Client,
    use_ai: bool,
}

#[async_trait]
impl McpTool for GenerateImageTool {
    async fn call(&self, request: McpRequest) -> Result<McpResponse, McpError> {
        match request {
            McpRequest::CallTool { name, arguments } => {
                let prompt = arguments.get("prompt")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing parameter 'prompt'"))?;
                
                let style = arguments.get("style")
                    .and_then(|v| v.as_str())
                    .unwrap_or("photorealistic");
                
                let size = arguments.get("size")
                    .and_then(|v| v.as_str())
                    .unwrap_or("1024x1024");
                
                // Generate image with AI or mock response
                let image_result = if self.use_ai {
                    self.generate_with_ai_provider(prompt, style, size).await?
                } else {
                    self.generate_placeholder_image(prompt, style, size).await?
                };
                
                let result = ResponseResult::ToolResult {
                    content: vec![ToolContent::Text { text: image_result }],
                    is_error: false,
                };
                
                Ok(McpResponse::success(result))
            }
            _ => Err(McpError::invalid_request("Expected CallTool request")),
        }
    }
    
    fn name(&self) -> &str { "generate_image" }
    fn description(&self) -> &str { "Generate an image from a text prompt using AI" }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "Text description of the image to generate",
                    "maxLength": 1000
                },
                "style": {
                    "type": "string",
                    "description": "Image style",
                    "enum": ["photorealistic", "artistic", "sketch", "cartoon"],
                    "default": "photorealistic"
                },
                "size": {
                    "type": "string",
                    "description": "Image dimensions",
                    "enum": ["256x256", "512x512", "1024x1024", "1024x1792"],
                    "default": "1024x1024"
                }
            },
            "required": ["prompt"]
        })
    }
}
```

## HTTP Endpoints

### Standard Endpoints

All HTTP transport implementations provide these standardized endpoints:

#### Health Check
```http
GET /health
```
**Response:** `200 OK` with body: `OK`

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
  ]
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
**Success Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "# My Project\n\nThis is the file content..."
    }
  ],
  "isError": false
}
```

**Error Response:**
```json
{
  "error": {
    "code": -32001,
    "message": "Resource not found",
    "data": {
      "resource": "nonexistent.txt",
      "type": "file"
    }
  }
}
```

### Server-Specific Endpoints

Each server example provides specialized tool endpoints:

#### Filesystem Server (Port 3000)
```bash
# Available tools
curl http://localhost:3000/mcp/tools/list

# Read file
curl -X POST http://localhost:3000/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{"name": "read_file", "arguments": {"path": "README.md"}}'
```

#### Image Generation Server (Port 3001)
```bash
# Generate image (mock mode)
curl -X POST http://localhost:3001/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "generate_image",
    "arguments": {
      "prompt": "A serene mountain landscape",
      "style": "photorealistic",
      "size": "1024x1024"
    }
  }'
```

#### Blog Generation Server (Port 3002)
```bash
# Create blog post
curl -X POST http://localhost:3002/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "create_blog_post",
    "arguments": {
      "topic": "AI in Healthcare",
      "word_count": 1500,
      "style": "professional",
      "include_seo": true
    }
  }'
```

#### Creative Content Server (Port 3003)
```bash
# Generate story
curl -X POST http://localhost:3003/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "generate_story",
    "arguments": {
      "genre": "sci-fi",
      "length": 1000,
      "theme": "space exploration"
    }
  }'

# Create poem
curl -X POST http://localhost:3003/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "create_poem",
    "arguments": {
      "style": "haiku",
      "theme": "nature",
      "mood": "peaceful"
    }
  }'
```

## AI Integration API

### Provider Architecture

The AI integration follows an extensible provider pattern:

```rust
pub trait AIProvider: Send + Sync {
    async fn generate_image(&self, prompt: &str, params: &ImageParams) -> Result<ImageResult, AIError>;
    async fn generate_text(&self, prompt: &str, params: &TextParams) -> Result<String, AIError>;
    fn provider_name(&self) -> &str;
    fn supported_capabilities(&self) -> Vec<AICapability>;
}
```

### Google/Gemini Integration

**Setup:**
```bash
export GEMINI_API_KEY="your-gemini-api-key"
```

**Implementation:**
```rust
async fn generate_with_gemini(&self, prompt: &str) -> Result<String, McpError> {
    let api_key = std::env::var("GEMINI_API_KEY")
        .map_err(|_| McpError::internal_error("GEMINI_API_KEY not set"))?;
    
    let request_body = serde_json::json!({
        "instances": [{
            "prompt": prompt
        }],
        "parameters": {
            "sampleCount": 1
        }
    });
    
    let response = self.client
        .post("https://us-central1-aiplatform.googleapis.com/v1/projects/YOUR_PROJECT/locations/us-central1/publishers/google/models/imagegeneration:predict")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| McpError::internal_error(format!("Gemini API request failed: {}", e)))?;
    
    // Process response and return formatted result
    // Implementation includes proper error handling and response parsing
}
```

### Mock Provider for Development

```rust
async fn generate_placeholder_image(&self, prompt: &str, style: &str, size: &str) -> Result<String, McpError> {
    // Simulate AI processing delay
    tokio::time::sleep(Duration::from_millis(self.processing_delay)).await;
    
    // Generate realistic mock response
    let image_data = format!(
        r#"{{
            "type": "image_generation_result",
            "content": [{{
                "type": "text",
                "text": "Generated image for prompt: {}",
                "image": {{
                    "description": "{}",
                    "alt_text": "Generated image showing {}",
                    "dimensions": "{}",
                    "url": "https://placeholder.example.com/generated/{}.png",
                    "created_at": "{}",
                    "metadata": {{
                        "model": "placeholder-diffusion-v2.1",
                        "style": "{}",
                        "guidance_scale": 7.5,
                        "processing_time_ms": {}
                    }}
                }}
            }}]],
            "isError": false
        }}"#,
        prompt,
        prompt,
        prompt,
        size,
        uuid::Uuid::new_v4(),
        chrono::Utc::now().to_rfc3339(),
        style,
        self.processing_delay
    );
    
    Ok(image_data)
}
```

## Error Handling

### Enhanced Error Types

```rust
pub struct McpError {
    pub code: McpErrorCode,
    pub message: String,
    pub data: Option<Value>,
}

pub enum McpErrorCode {
    // Standard JSON-RPC errors
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    
    // MCP-specific errors
    ResourceNotFound = -32001,
    PermissionDenied = -32002,
    Timeout = -32003,
    
    // AI-specific errors
    AIProviderError = -32004,
    APIKeyMissing = -32005,
    RateLimitExceeded = -32006,
}
```

**Error Creation Helpers:**
```rust
// File system errors
let error = McpError::resource_not_found("file.txt");
let error = McpError::permission_denied("Access denied to /secure/path");

// AI provider errors
let error = McpError::ai_provider_error("Gemini API quota exceeded");
let error = McpError::api_key_missing("GEMINI_API_KEY environment variable not set");

// Custom errors with data
let error = McpError::new(
    McpErrorCode::RateLimitExceeded,
    "Rate limit exceeded".to_string(),
    Some(serde_json::json!({
        "retry_after": 60,
        "current_usage": 100,
        "limit": 100
    }))
);
```

### Error Response Examples

**File Not Found:**
```json
{
  "error": {
    "code": -32001,
    "message": "Resource not found",
    "data": {
      "resource": "nonexistent.txt",
      "type": "file",
      "base_directory": "/safe/path"
    }
  }
}
```

**AI Provider Error:**
```json
{
  "error": {
    "code": -32004,
    "message": "AI provider error",
    "data": {
      "provider": "gemini",
      "error_type": "quota_exceeded",
      "retry_after": 3600,
      "documentation": "https://cloud.google.com/vertex-ai/docs/quotas"
    }
  }
}
```

**Invalid Parameters:**
```json
{
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "parameter": "style",
      "expected": "one of [photorealistic, artistic, sketch, cartoon]",
      "received": "invalid_style"
    }
  }
}
```

## Integration Guide

### Basic Server Integration

```rust
use mcp_core::McpTool;
use mcp_server::McpServerBuilder;
use mcp_transport::{HttpTransport, StdioTransport, Transport};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    // Create server with tools
    let server = McpServerBuilder::new()
        .with_name("my-production-server")
        .with_version("1.0.0")
        .add_tool(Arc::new(ReadFileTool::new("/safe/base/dir".to_string())))
        .add_tool(Arc::new(GenerateImageTool::new(true, "gemini".to_string())))
        .enable_tracing(true)
        .max_concurrent_requests(100)
        .request_timeout(Duration::from_secs(60))
        .build()?;
    
    // Configure transport
    let addr = "0.0.0.0:3000".parse()?;
    let transport = HttpTransport::builder(addr)
        .cors_enabled(true)
        .max_request_size(10 * 1024 * 1024) // 10MB for large prompts
        .build()?;
    
    // Start server
    transport.start_server().await?;
    
    // Request handling loop
    let transport_arc: Arc<dyn Transport> = Arc::new(transport);
    loop {
        match transport_arc.receive_request().await {
            Ok(Some(request)) => {
                let response = server.handle_request(request).await
                    .unwrap_or_else(|e| McpResponse::error(e));
                transport_arc.send_response(response).await?;
            }
            Ok(None) => break,
            Err(e) => {
                tracing::error!("Transport error: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}
```

### Client Integration (Python Example)

```python
import requests
import json

class MCPClient:
    def __init__(self, base_url: str):
        self.base_url = base_url.rstrip('/')
    
    def list_tools(self):
        response = requests.get(f"{self.base_url}/mcp/tools/list")
        return response.json()
    
    def call_tool(self, name: str, arguments: dict):
        response = requests.post(
            f"{self.base_url}/mcp/tools/call",
            json={"name": name, "arguments": arguments}
        )
        return response.json()

# Usage
client = MCPClient("http://localhost:3001")
tools = client.list_tools()

# Generate image
result = client.call_tool("generate_image", {
    "prompt": "A beautiful sunset",
    "style": "photorealistic",
    "size": "1024x1024"
})
```

### JavaScript/Node.js Integration

```javascript
class MCPClient {
    constructor(baseUrl) {
        this.baseUrl = baseUrl.replace(/\/$/, '');
    }
    
    async listTools() {
        const response = await fetch(`${this.baseUrl}/mcp/tools/list`);
        return await response.json();
    }
    
    async callTool(name, arguments) {
        const response = await fetch(`${this.baseUrl}/mcp/tools/call`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ name, arguments })
        });
        return await response.json();
    }
}

// Usage
const client = new MCPClient('http://localhost:3002');
const blogPost = await client.callTool('create_blog_post', {
    topic: 'Sustainable Technology',
    word_count: 1200,
    style: 'engaging'
});
```

## Production Deployment

### Environment Configuration

```bash
# Required for AI features
export GEMINI_API_KEY="your-gemini-api-key"

# Optional: Custom configurations
export MCP_SERVER_PORT="3000"
export MCP_SERVER_HOST="0.0.0.0"
export MCP_LOG_LEVEL="info"
export MCP_MAX_CONCURRENT_REQUESTS="200"
export MCP_REQUEST_TIMEOUT_SECONDS="60"
```

### Docker Deployment

**Dockerfile:**
```dockerfile
FROM rust:1.70-bullseye as builder

WORKDIR /app
COPY . .
RUN cargo build --release --bin image-generation-server

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/image-generation-server /usr/local/bin/
EXPOSE 3000

ENV MCP_SERVER_PORT=3000
ENV MCP_SERVER_HOST=0.0.0.0

CMD ["image-generation-server", "--transport", "http", "--use-ai", "--provider", "gemini"]
```

**Docker Compose:**
```yaml
version: '3.8'

services:
  mcp-filesystem:
    build: .
    command: ["filesystem-server", "--transport", "http", "--port", "3000"]
    ports:
      - "3000:3000"
    volumes:
      - "./data:/app/data:ro"
    
  mcp-ai-image:
    build: .
    command: ["image-generation-server", "--transport", "http", "--port", "3001", "--use-ai"]
    ports:
      - "3001:3001"
    environment:
      - GEMINI_API_KEY=${GEMINI_API_KEY}
    
  mcp-ai-blog:
    build: .
    command: ["blog-generation-server", "--transport", "http", "--port", "3002"]
    ports:
      - "3002:3002"
    
  mcp-ai-creative:
    build: .
    command: ["creative-content-server", "--transport", "http", "--port", "3003"]
    ports:
      - "3003:3003"
```

### Kubernetes Deployment

**Complete Kubernetes Configuration:**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mcp-server-suite
  labels:
    app: mcp-server
spec:
  replicas: 3
  selector:
    matchLabels:
      app: mcp-server
  template:
    metadata:
      labels:
        app: mcp-server
    spec:
      containers:
      - name: filesystem-server
        image: your-registry/mcp-boilerplate-rust:latest
        command: ["filesystem-server"]
        args: ["--transport", "http", "--port", "3000", "--base-dir", "/app/data"]
        ports:
        - containerPort: 3000
        volumeMounts:
        - name: data-volume
          mountPath: /app/data
          readOnly: true
        resources:
          requests:
            memory: "32Mi"
            cpu: "50m"
          limits:
            memory: "128Mi"
            cpu: "200m"
      
      - name: image-generation-server
        image: your-registry/mcp-boilerplate-rust:latest
        command: ["image-generation-server"]
        args: ["--transport", "http", "--port", "3001", "--use-ai", "--provider", "gemini"]
        ports:
        - containerPort: 3001
        env:
        - name: GEMINI_API_KEY
          valueFrom:
            secretKeyRef:
              name: ai-api-keys
              key: gemini-key
        resources:
          requests:
            memory: "64Mi"
            cpu: "100m"
          limits:
            memory: "256Mi"
            cpu: "500m"
      
      volumes:
      - name: data-volume
        configMap:
          name: filesystem-data

---
apiVersion: v1
kind: Service
metadata:
  name: mcp-server-service
spec:
  selector:
    app: mcp-server
  ports:
  - name: filesystem
    port: 3000
    targetPort: 3000
  - name: image-gen
    port: 3001
    targetPort: 3001
  type: LoadBalancer

---
apiVersion: v1
kind: Secret
metadata:
  name: ai-api-keys
type: Opaque
stringData:
  gemini-key: "your-gemini-api-key"
```

### Monitoring & Observability

**Health Checks:**
```bash
# Kubernetes readiness probe
curl -f http://localhost:3000/health || exit 1

# Comprehensive health check
curl -X GET http://localhost:3000/mcp/tools/list | jq '.tools | length'
```

**Metrics Collection:**
```rust
// Server metrics endpoint (custom implementation)
#[derive(Serialize)]
pub struct ServerMetrics {
    pub uptime_seconds: u64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub error_rate: f64,
    pub average_response_time_ms: f64,
    pub active_tools: usize,
    pub concurrent_requests: usize,
}

// Usage in server
let metrics = server.get_metrics().await;
```

## Performance Guidelines

### Server Performance

| Metric | Target | Production |
|--------|--------|------------|
| Startup Time | < 2s | < 5s |
| Response Time (Local Tools) | < 100ms | < 500ms |
| Response Time (AI Tools) | < 5s | < 30s |
| Memory Usage | < 100MB | < 500MB |
| Concurrent Requests | 100+ | 1000+ |

### Optimization Patterns

**Connection Pooling:**
```rust
let client = reqwest::Client::builder()
    .pool_max_idle_per_host(10)
    .pool_idle_timeout(Duration::from_secs(30))
    .timeout(Duration::from_secs(60))
    .build()?;
```

**Request Batching:**
```rust
// For AI providers that support batch requests
async fn batch_generate_images(&self, prompts: Vec<String>) -> Result<Vec<String>, McpError> {
    // Implementation for batch processing
}
```

**Caching:**
```rust
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct CachedTool {
    cache: Arc<RwLock<HashMap<String, String>>>,
    inner_tool: Arc<dyn McpTool>,
}

impl CachedTool {
    async fn get_cached_or_generate(&self, key: &str) -> Result<String, McpError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(key) {
                return Ok(cached.clone());
            }
        }
        
        // Generate and cache
        let result = self.inner_tool.generate(key).await?;
        {
            let mut cache = self.cache.write().await;
            cache.insert(key.to_string(), result.clone());
        }
        
        Ok(result)
    }
}
```

## Security Considerations

### Input Validation
```rust
fn validate_file_path(path: &str, base_dir: &Path) -> Result<PathBuf, McpError> {
    let requested = Path::new(path);
    
    // Prevent path traversal
    if path.contains("..") || path.starts_with('/') {
        return Err(McpError::permission_denied("Invalid path: contains traversal"));
    }
    
    let full_path = base_dir.join(requested);
    
    // Ensure within base directory
    match full_path.canonicalize() {
        Ok(canonical) if canonical.starts_with(base_dir) => Ok(canonical),
        _ => Err(McpError::permission_denied("Path outside base directory")),
    }
}
```

### Content Filtering
```rust
fn validate_ai_prompt(prompt: &str) -> Result<(), McpError> {
    // Length validation
    if prompt.len() > 1000 {
        return Err(McpError::invalid_params("Prompt too long (max 1000 characters)"));
    }
    
    // Content filtering (implement based on requirements)
    let forbidden_terms = ["harmful", "illegal", "inappropriate"];
    for term in forbidden_terms {
        if prompt.to_lowercase().contains(term) {
            return Err(McpError::invalid_params("Prompt contains inappropriate content"));
        }
    }
    
    Ok(())
}
```

### Rate Limiting
```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window_duration: Duration,
}

impl RateLimiter {
    pub async fn check_rate_limit(&self, client_id: &str) -> Result<(), McpError> {
        let now = Instant::now();
        let mut requests = self.requests.write().await;
        let client_requests = requests.entry(client_id.to_string()).or_default();
        
        // Remove old requests outside window
        client_requests.retain(|&time| now.duration_since(time) < self.window_duration);
        
        // Check limit
        if client_requests.len() >= self.max_requests {
            return Err(McpError::new(
                McpErrorCode::RateLimitExceeded,
                "Rate limit exceeded".to_string(),
                Some(serde_json::json!({
                    "retry_after": self.window_duration.as_secs(),
                    "current_requests": client_requests.len(),
                    "limit": self.max_requests
                }))
            ));
        }
        
        // Record this request
        client_requests.push(now);
        Ok(())
    }
}
```

## Testing API

### E2E Testing Framework

The project includes a comprehensive E2E testing framework with real server lifecycle testing:

```rust
use tokio::time::{timeout, Duration};
use tempfile::TempDir;
use reqwest::Client;

#[tokio::test]
async fn test_full_server_workflow() {
    // Setup test environment
    let temp_dir = TempDir::new().unwrap();
    let port = get_random_port();
    
    // Start server with timeout protection
    let server_handle = timeout(
        Duration::from_secs(5),
        start_test_server(port, temp_dir.path())
    ).await.expect("Server startup should not hang").unwrap();
    
    // Test protocol compliance
    let client = Client::new();
    
    // Health check
    let response = client.get(&format!("http://localhost:{}/health", port))
        .send().await.unwrap();
    assert_eq!(response.status(), 200);
    
    // List tools
    let tools = client.get(&format!("http://localhost:{}/mcp/tools/list", port))
        .send().await.unwrap()
        .json::<Value>().await.unwrap();
    assert!(!tools["tools"].as_array().unwrap().is_empty());
    
    // Call tool
    let result = client.post(&format!("http://localhost:{}/mcp/tools/call", port))
        .json(&serde_json::json!({
            "name": "read_file",
            "arguments": {"path": "test.txt"}
        }))
        .send().await.unwrap();
    
    assert!(result.status().is_success());
    
    // Cleanup
    server_handle.abort();
}
```

### Test Utilities

```rust
// Random port allocation for parallel testing
fn get_random_port() -> u16 {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

// Test server spawning with proper cleanup
async fn spawn_test_server(port: u16) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let server = create_test_server().await.unwrap();
        let transport = HttpTransport::with_defaults(
            format!("127.0.0.1:{}", port).parse().unwrap()
        ).unwrap();
        
        transport.start_server().await.unwrap();
        // Server runs until handle is aborted
    })
}

// Timeout wrapper for all async operations
async fn with_timeout<T>(operation: impl Future<Output = T>) -> T {
    timeout(Duration::from_secs(5), operation)
        .await
        .expect("Operation should complete within 5 seconds")
}
```

## Version History & Compatibility

| Version | Features | AI Integration | Breaking Changes |
|---------|----------|----------------|------------------|
| 0.1.0 | Core protocol, basic servers | Mock responses | Initial release |
| 0.2.0 | AI integration, comprehensive testing | Google/Gemini live | Enhanced tool trait |
| 1.0.0 | Production ready, security hardened | Multi-provider support | Stable API |

### Migration Guide

**From 0.1.x to 0.2.x:**
- Tool trait now requires `input_schema()` method
- AI tools need provider configuration
- Enhanced error types with AI-specific codes

**From 0.2.x to 1.0.x:**
- No breaking changes (backward compatible)
- Enhanced security and performance features
- Additional AI provider support

## Advanced Usage

### Custom AI Provider Integration

```rust
#[async_trait]
pub trait CustomAIProvider: Send + Sync {
    async fn generate_content(&self, prompt: &str, params: &AIParams) -> Result<String, AIError>;
    fn provider_name(&self) -> &str;
    fn max_prompt_length(&self) -> usize;
    fn supported_content_types(&self) -> Vec<ContentType>;
}

// Implement for your AI provider
pub struct OpenAIProvider {
    api_key: String,
    client: reqwest::Client,
}

#[async_trait]
impl CustomAIProvider for OpenAIProvider {
    async fn generate_content(&self, prompt: &str, params: &AIParams) -> Result<String, AIError> {
        // OpenAI API integration
        todo!()
    }
    
    fn provider_name(&self) -> &str { "openai" }
    fn max_prompt_length(&self) -> usize { 4000 }
    fn supported_content_types(&self) -> Vec<ContentType> {
        vec![ContentType::Text, ContentType::Image]
    }
}
```

### Middleware Integration

```rust
// Request middleware pattern
#[async_trait]
pub trait RequestMiddleware: Send + Sync {
    async fn before_request(&self, request: &mut McpRequest) -> Result<(), McpError>;
    async fn after_response(&self, response: &mut McpResponse) -> Result<(), McpError>;
}

// Example: Authentication middleware
pub struct AuthMiddleware {
    valid_tokens: HashSet<String>,
}

#[async_trait]
impl RequestMiddleware for AuthMiddleware {
    async fn before_request(&self, request: &mut McpRequest) -> Result<(), McpError> {
        // Extract and validate auth token from request
        // Return error if authentication fails
        Ok(())
    }
    
    async fn after_response(&self, response: &mut McpResponse) -> Result<(), McpError> {
        // Add response headers, logging, etc.
        Ok(())
    }
}
```

## Quick Reference

### Command Cheat Sheet

```bash
# Development
./setup.sh all                    # Complete setup
./test.sh                        # Interactive testing
./generate_image.py "prompt"     # Generate image

# Building
cargo build --workspace          # Build all components
cargo test --workspace          # Run all tests (< 10s)
cargo clippy --workspace        # Lint check
cargo fmt --all                 # Format code

# Server Operations
cargo run --bin filesystem-server -- --transport http --port 3000
cargo run --bin image-generation-server -- --transport http --port 3001 --use-ai
cargo run --bin blog-generation-server -- --transport http --port 3002
cargo run --bin creative-content-server -- --transport http --port 3003

# Testing Individual Servers
curl http://localhost:3000/health                    # Health check
curl http://localhost:3000/mcp/tools/list            # List tools
```

### Tool Schema Reference

**Filesystem Tools:**
- `read_file(path: string)` → File content as text

**Image Generation Tools:**
- `generate_image(prompt: string, style?: string, size?: string)` → Image data/metadata

**Blog Generation Tools:**
- `create_blog_post(topic: string, word_count?: number, style?: string, include_seo?: boolean)` → Blog post content

**Creative Content Tools:**
- `generate_story(genre: string, length?: number, theme?: string)` → Story content
- `create_poem(style: string, theme?: string, mood?: string)` → Poem content
- `develop_character(archetype?: string, background?: string)` → Character profile

### Error Code Reference

| Code | Name | Description | Recovery Action |
|------|------|-------------|-----------------|
| -32700 | Parse Error | Invalid JSON | Fix request format |
| -32600 | Invalid Request | Invalid request object | Check request structure |
| -32601 | Method Not Found | Unknown tool name | Use valid tool name |
| -32602 | Invalid Params | Invalid parameters | Check parameter schema |
| -32603 | Internal Error | Server error | Retry or contact support |
| -32001 | Resource Not Found | File/resource missing | Check resource exists |
| -32002 | Permission Denied | Access denied | Check permissions |
| -32003 | Timeout | Operation timed out | Retry with simpler request |
| -32004 | AI Provider Error | AI service failure | Check API key and quota |
| -32005 | API Key Missing | Missing credentials | Set required environment variables |
| -32006 | Rate Limit Exceeded | Too many requests | Wait and retry |

## Support & Resources

- **Repository**: https://github.com/netadx1ai/mcp-boilerplate-rust
- **Issues**: https://github.com/netadx1ai/mcp-boilerplate-rust/issues
- **Documentation**: https://docs.rs/mcp-boilerplate-rust
- **MCP Specification**: https://spec.modelcontextprotocol.io/
- **Examples**: See `examples/` directory for complete implementations

---

**API Version**: 1.0 | **Last Updated**: 2025-01-17 | **MCP Protocol**: 2024-11-05 | **Status**: Production Ready 🚀