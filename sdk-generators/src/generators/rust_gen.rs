use crate::{SdkGenerator, ToolSchema};
use std::fs;

impl SdkGenerator {
    pub fn generate_rust_sdk(&self) -> String {
        let mut code = String::new();

        // File header with documentation
        code.push_str(&self.generate_rust_header());
        
        // Imports
        code.push_str(&self.generate_rust_imports());
        
        // Error types (custom, not Box<dyn Error>)
        code.push_str(&self.generate_rust_error_types());
        
        // Type definitions
        code.push_str(&self.generate_rust_types());
        
        // Transport trait
        code.push_str(&self.generate_rust_transport_trait());
        
        // HTTP Transport implementation
        code.push_str(&self.generate_rust_http_transport());
        
        // Main Client struct
        code.push_str(&self.generate_rust_client_struct());
        
        // Client implementation
        code.push_str(&self.generate_rust_client_impl());
        
        // Tool methods (idiomatic)
        code.push_str(&self.generate_rust_tool_methods());
        
        // Tests
        code.push_str(&self.generate_rust_tests());

        code
    }

    fn generate_rust_header(&self) -> String {
        format!(
            r#"//! MCP Client SDK for Rust
//!
//! Auto-generated high-performance client library for Model Context Protocol.
//! Generated from MCP Boilerplate Rust v{}
//!
//! ## Features
//!
//! - Type-safe API with zero-cost abstractions
//! - Async/await on Tokio runtime
//! - Custom error types for pattern matching
//! - Idiomatic Rust patterns
//! - Optimized for performance
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use mcp_client::{{McpClient, HttpTransport}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), McpError> {{
//!     let transport = HttpTransport::new("http://127.0.0.1:8080");
//!     let mut client = McpClient::new(transport);
//!     
//!     client.connect().await?;
//!     let result = client.echo("Hello, MCP!").await?;
//!     println!("{{}}", result);
//!     
//!     Ok(())
//! }}
//! ```

#![warn(missing_docs)]
#![deny(unsafe_code)]

"#,
            self.project_version
        )
    }

    fn generate_rust_imports(&self) -> String {
        r#"use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

"#
        .to_string()
    }

    fn generate_rust_error_types(&self) -> String {
        r#"/// Result type alias for MCP operations
pub type Result<T> = std::result::Result<T, McpError>;

/// MCP Client errors with full type information
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    /// Connection failed
    #[error("Connection error: {0}")]
    Connection(String),
    
    /// Transport error
    #[error("Transport error: {0}")]
    Transport(String),
    
    /// Tool execution failed
    #[error("Tool execution error: {0}")]
    ToolExecution(String),
    
    /// Invalid response format
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    /// Request timeout
    #[error("Timeout: {0}")]
    Timeout(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    /// HTTP error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    /// Not connected
    #[error("Not connected: {0}")]
    NotConnected(String),
}

"#
        .to_string()
    }

    fn generate_rust_types(&self) -> String {
        r#"/// JSON-RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Value,
}

/// JSON-RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC error
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

/// Tool call result content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Content {
    /// Text content
    #[serde(rename = "text")]
    Text { text: String },
}

/// Tool call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    /// Result content
    pub content: Vec<Content>,
    /// Whether this is an error
    #[serde(rename = "isError")]
    pub is_error: Option<bool>,
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
}

/// Initialize result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    /// Protocol version
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Server capabilities
    pub capabilities: Value,
    /// Server information
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

"#
        .to_string()
    }

    fn generate_rust_transport_trait(&self) -> String {
        r#"/// Transport trait for different connection types
#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    /// Connect to the server
    async fn connect(&mut self) -> Result<()>;
    
    /// Send a request and receive response
    async fn send(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse>;
    
    /// Close the connection
    async fn close(&mut self) -> Result<()>;
    
    /// Check if connected
    fn is_connected(&self) -> bool;
}

"#
        .to_string()
    }

    fn generate_rust_http_transport(&self) -> String {
        r#"/// HTTP transport implementation
pub struct HttpTransport {
    url: String,
    client: Option<reqwest::Client>,
    timeout: std::time::Duration,
    connected: bool,
}

impl HttpTransport {
    /// Create new HTTP transport
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            client: None,
            timeout: std::time::Duration::from_secs(30),
            connected: false,
        }
    }
    
    /// Set timeout
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    fn rpc_url(&self) -> String {
        format!("{}/rpc", self.url.trim_end_matches('/'))
    }
}

#[async_trait::async_trait]
impl Transport for HttpTransport {
    async fn connect(&mut self) -> Result<()> {
        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()?;
        
        self.client = Some(client);
        self.connected = true;
        Ok(())
    }
    
    async fn send(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let client = self.client.as_ref()
            .ok_or_else(|| McpError::NotConnected("Call connect() first".into()))?;
        
        let response = client
            .post(self.rpc_url())
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(McpError::Transport(
                format!("HTTP {}", response.status())
            ));
        }
        
        let json_response: JsonRpcResponse = response.json().await?;
        
        if let Some(error) = &json_response.error {
            return Err(McpError::ToolExecution(
                format!("Error {}: {}", error.code, error.message)
            ));
        }
        
        Ok(json_response)
    }
    
    async fn close(&mut self) -> Result<()> {
        self.client = None;
        self.connected = false;
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        self.connected && self.client.is_some()
    }
}

"#
        .to_string()
    }

    fn generate_rust_client_struct(&self) -> String {
        r#"/// High-performance MCP client with zero-cost abstractions
pub struct McpClient<T: Transport> {
    transport: Arc<RwLock<T>>,
    request_id: AtomicU64,
    initialized: Arc<RwLock<bool>>,
    server_info: Arc<RwLock<Option<InitializeResult>>>,
}

"#
        .to_string()
    }

    fn generate_rust_client_impl(&self) -> String {
        r#"impl<T: Transport> McpClient<T> {
    /// Create new MCP client
    pub fn new(transport: T) -> Self {
        Self {
            transport: Arc::new(RwLock::new(transport)),
            request_id: AtomicU64::new(1),
            initialized: Arc::new(RwLock::new(false)),
            server_info: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Connect and initialize session
    pub async fn connect(&mut self) -> Result<InitializeResult> {
        let mut transport = self.transport.write().await;
        transport.connect().await?;
        drop(transport);
        
        let params = json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {
                "name": "mcp-client-rust",
                "version": env!("CARGO_PKG_VERSION")
            }
        });
        
        let request = self.create_request("initialize".into(), params);
        let response = self.send_request(request).await?;
        
        let result: InitializeResult = serde_json::from_value(
            response.result.ok_or_else(|| 
                McpError::InvalidResponse("Missing result".into())
            )?
        )?;
        
        let mut initialized = self.initialized.write().await;
        *initialized = true;
        
        let mut server_info = self.server_info.write().await;
        *server_info = Some(result.clone());
        
        Ok(result)
    }
    
    /// Call a tool with arguments
    async fn call_tool(
        &self,
        name: &str,
        arguments: Option<HashMap<String, Value>>,
    ) -> Result<CallToolResult> {
        self.ensure_initialized().await?;
        
        let params = json!({
            "name": name,
            "arguments": arguments
        });
        
        let request = self.create_request("tools/call".into(), params);
        let response = self.send_request(request).await?;
        
        let result: CallToolResult = serde_json::from_value(
            response.result.ok_or_else(||
                McpError::InvalidResponse("Missing result".into())
            )?
        )?;
        
        if result.is_error == Some(true) {
            return Err(McpError::ToolExecution("Tool returned error".into()));
        }
        
        Ok(result)
    }
    
    /// Close connection
    pub async fn close(&mut self) -> Result<()> {
        let mut transport = self.transport.write().await;
        transport.close().await?;
        
        let mut initialized = self.initialized.write().await;
        *initialized = false;
        
        Ok(())
    }
    
    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        let transport = self.transport.read().await;
        let initialized = self.initialized.read().await;
        transport.is_connected() && *initialized
    }
    
    fn create_request(&self, method: String, params: Value) -> JsonRpcRequest {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id,
            method,
            params,
        }
    }
    
    async fn send_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let mut transport = self.transport.write().await;
        transport.send(request).await
    }
    
    async fn ensure_initialized(&self) -> Result<()> {
        let initialized = self.initialized.read().await;
        if !*initialized {
            return Err(McpError::NotConnected(
                "Call connect() first".into()
            ));
        }
        Ok(())
    }
}

"#
        .to_string()
    }

    fn generate_rust_tool_methods(&self) -> String {
        let mut code = String::from("// Tool methods with zero-copy optimizations\n");
        code.push_str("impl<T: Transport> McpClient<T> {\n");
        
        for tool in &self.tools {
            code.push_str(&self.generate_rust_tool_method(tool));
        }
        
        code.push_str("}\n\n");
        code
    }

    fn generate_rust_tool_method(&self, tool: &ToolSchema) -> String {
        match tool.name.as_str() {
            "echo" => r#"
    /// Echo a message (optimized with borrowing)
    pub async fn echo(&self, message: &str) -> Result<String> {
        let mut args = HashMap::new();
        args.insert("message".to_string(), json!(message));
        
        let result = self.call_tool("echo", Some(args)).await?;
        
        if let Some(Content::Text { text }) = result.content.first() {
            Ok(text.clone())
        } else {
            Err(McpError::InvalidResponse("Invalid echo response".into()))
        }
    }
"#.to_string(),
            "ping" => r#"
    /// Ping the server
    pub async fn ping(&self) -> Result<String> {
        let result = self.call_tool("ping", None).await?;
        
        if let Some(Content::Text { text }) = result.content.first() {
            Ok(text.clone())
        } else {
            Err(McpError::InvalidResponse("Invalid ping response".into()))
        }
    }
"#.to_string(),
            "info" => r#"
    /// Get server info
    pub async fn info(&self) -> Result<String> {
        let result = self.call_tool("info", None).await?;
        
        if let Some(Content::Text { text }) = result.content.first() {
            Ok(text.clone())
        } else {
            Err(McpError::InvalidResponse("Invalid info response".into()))
        }
    }
"#.to_string(),
            "calculate" => r#"
    /// Perform calculation (zero-copy operation)
    pub async fn calculate(&self, operation: &str, a: f64, b: f64) -> Result<f64> {
        let mut args = HashMap::new();
        args.insert("operation".to_string(), json!(operation));
        args.insert("a".to_string(), json!(a));
        args.insert("b".to_string(), json!(b));
        
        let result = self.call_tool("calculate", Some(args)).await?;
        
        if let Some(Content::Text { text }) = result.content.first() {
            let parsed: Value = serde_json::from_str(text)?;
            parsed.get("result")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| McpError::InvalidResponse("Missing result".into()))
        } else {
            Err(McpError::InvalidResponse("Invalid calculate response".into()))
        }
    }
"#.to_string(),
            "evaluate" => r#"
    /// Evaluate math expression
    pub async fn evaluate(&self, expression: &str) -> Result<f64> {
        let mut args = HashMap::new();
        args.insert("expression".to_string(), json!(expression));
        
        let result = self.call_tool("evaluate", Some(args)).await?;
        
        if let Some(Content::Text { text }) = result.content.first() {
            let parsed: Value = serde_json::from_str(text)?;
            parsed.get("result")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| McpError::InvalidResponse("Missing result".into()))
        } else {
            Err(McpError::InvalidResponse("Invalid evaluate response".into()))
        }
    }
"#.to_string(),
            "transform_data" => r#"
    /// Transform data array
    pub async fn transform_data(&self, data: Vec<Value>, operation: &str) -> Result<Vec<Value>> {
        let mut args = HashMap::new();
        args.insert("data".to_string(), json!(data));
        args.insert("operation".to_string(), json!(operation));
        
        let result = self.call_tool("transform_data", Some(args)).await?;
        
        if let Some(Content::Text { text }) = result.content.first() {
            let parsed: Value = serde_json::from_str(text)?;
            parsed.get("result")
                .and_then(|v| v.as_array())
                .map(|a| a.clone())
                .ok_or_else(|| McpError::InvalidResponse("Missing result".into()))
        } else {
            Err(McpError::InvalidResponse("Invalid transform response".into()))
        }
    }
"#.to_string(),
            "health_check" => r#"
    /// Check server health
    pub async fn health_check(&self) -> Result<Value> {
        let result = self.call_tool("health_check", None).await?;
        
        if let Some(Content::Text { text }) = result.content.first() {
            Ok(serde_json::from_str(text)?)
        } else {
            Err(McpError::InvalidResponse("Invalid health response".into()))
        }
    }
"#.to_string(),
            _ => format!(
                r#"
    /// Call {} tool
    pub async fn {}(&self, args: HashMap<String, Value>) -> Result<CallToolResult> {{
        self.call_tool("{}", Some(args)).await
    }}
"#,
                tool.name, tool.name.replace("-", "_"), tool.name
            ),
        }
    }

    fn generate_rust_tests(&self) -> String {
        r#"#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_types() {
        let err = McpError::Connection("test".into());
        assert!(matches!(err, McpError::Connection(_)));
    }
    
    #[test]
    fn test_http_transport_creation() {
        let transport = HttpTransport::new("http://localhost:8080");
        assert!(!transport.is_connected());
    }
}
"#
        .to_string()
    }

    pub fn save_rust_sdk(&self, output_dir: &str) -> std::io::Result<()> {
        let output_path = format!("{}/rust", output_dir);
        fs::create_dir_all(&output_path)?;
        
        let sdk_code = self.generate_rust_sdk();
        fs::write(format!("{}/mcp_client.rs", output_path), sdk_code)?;
        
        // Generate Cargo.toml
        let cargo_toml = self.generate_rust_cargo_toml();
        fs::write(format!("{}/Cargo.toml", output_path), cargo_toml)?;
        
        // Generate README
        let readme = self.generate_rust_readme();
        fs::write(format!("{}/README.md", output_path), readme)?;
        
        Ok(())
    }
    
    fn generate_rust_cargo_toml(&self) -> String {
        r#"[package]
name = "mcp-client"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"
thiserror = "1.0"
"#
        .to_string()
    }
    
    fn generate_rust_readme(&self) -> String {
        format!(
            r#"# MCP Client - Generated Rust SDK (Race Car Edition 🏎️)

High-performance, idiomatic Rust SDK for Model Context Protocol.

## Features

- ✅ Zero-cost abstractions
- ✅ Custom error types (no `Box<dyn Error>`)
- ✅ Borrowing optimizations (`&str` vs `String`)
- ✅ Async/await on Tokio
- ✅ Type-safe pattern matching
- ✅ Production-ready performance

## Installation

```toml
[dependencies]
mcp-client = {{ path = "./rust" }}
tokio = {{ version = "1.35", features = ["full"] }}
```

## Usage

```rust
use mcp_client::{{McpClient, HttpTransport, Result}};

#[tokio::main]
async fn main() -> Result<()> {{
    let transport = HttpTransport::new("http://127.0.0.1:8080");
    let mut client = McpClient::new(transport);
    
    // Connect
    let server_info = client.connect().await?;
    println!("Connected to: {{}}", server_info.server_info.name);
    
    // Echo (with borrowing)
    let result = client.echo("Hello, MCP!").await?;
    println!("Echo: {{}}", result);
    
    // Calculate (zero-copy)
    let result = client.calculate("add", 10.0, 5.0).await?;
    println!("10 + 5 = {{}}", result);
    
    // Evaluate
    let result = client.evaluate("2 * (3 + 4)").await?;
    println!("Result: {{}}", result);
    
    client.close().await?;
    Ok(())
}}
```

## Performance

This SDK is optimized for:
- Zero allocations where possible
- Borrowing instead of cloning
- Custom error types for pattern matching
- Efficient async patterns

## Generated

Auto-generated from MCP Boilerplate Rust v{}

**Quality:** Race Car 🏎️ (not sedan!)
"#,
            self.project_version
        )
    }
}