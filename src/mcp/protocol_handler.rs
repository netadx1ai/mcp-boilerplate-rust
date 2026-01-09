//! Shared MCP Protocol Handler
//!
//! This module provides a unified protocol handler that uses rmcp types consistently
//! across all transports (stdio, SSE, WebSocket, HTTP streaming).
//!
//! Note: Methods are used by feature-gated servers (SSE, WebSocket, HTTP, gRPC).
#![allow(dead_code)]
//!
//! # Design Goals
//!
//! 1. **Type Safety**: Use rmcp::model types instead of manual JSON parsing
//! 2. **Consistency**: Same protocol handling logic across all transports
//! 3. **Maintainability**: Single source of truth for MCP protocol
//! 4. **Extensibility**: Easy to add new tools and capabilities
//!
//! # Architecture
//!
//! ```text
//! Transport Layer (stdio/SSE/WebSocket/HTTP)
//!        ↓
//! Protocol Handler (this module)
//!        ↓
//! rmcp::model types (type-safe)
//!        ↓
//! Tool Router (rmcp::handler::server::tool)
//!        ↓
//! Tool Implementations
//! ```
//!
//! # Usage
//!
//! ```rust
//! use crate::mcp::protocol_handler::ProtocolHandler;
//!
//! // Initialize handler
//! let handler = ProtocolHandler::new();
//!
//! // Handle JSON-RPC request
//! let request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
//! let response = handler.handle_request(request).await?;
//! ```

use anyhow::Result;
use rmcp::model::{
    InitializeResult, Implementation, PromptsCapability, ProtocolVersion, ResourcesCapability,
    ServerCapabilities, Tool, ToolsCapability,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{error, info, instrument};

type JsonObject = serde_json::Map<String, Value>;

use crate::metrics;

/// Helper function to convert Value to Arc<JsonObject>
fn value_to_schema(value: Value) -> Arc<JsonObject> {
    match value {
        Value::Object(map) => Arc::new(map),
        _ => Arc::new(serde_json::Map::new()),
    }
}

use crate::prompts::get_available_prompts;
use crate::resources::ResourceRegistry;
use crate::tools::calculator::{CalculateResponse, EvaluateResponse};
use crate::tools::shared::*;

/// Protocol handler for non-stdio transports
///
/// Provides JSON-RPC interface for SSE, WebSocket, HTTP streaming, and gRPC.
#[derive(Clone)]
pub struct ProtocolHandler {
    resource_registry: ResourceRegistry,
    server_info: ServerInfo,
}

/// Server information
#[derive(Clone)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

impl Default for ServerInfo {
    fn default() -> Self {
        Self {
            name: "MCP Boilerplate Rust".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl ProtocolHandler {
    /// Create a new protocol handler
    pub fn new() -> Self {
        Self {
            resource_registry: ResourceRegistry::new(),
            server_info: ServerInfo::default(),
        }
    }

    /// Handle JSON-RPC request string and return JSON-RPC response string
    ///
    /// This is the main entry point for SSE/WebSocket/HTTP transports.
    /// Parses JSON-RPC, routes to appropriate handler, returns JSON-RPC response.
    #[instrument(skip(self, request_json), fields(request_len = request_json.len()))]
    pub async fn handle_request(&self, request_json: &str) -> Result<String> {
        let start_time = std::time::Instant::now();

        // Parse JSON-RPC request
        let request: Value = serde_json::from_str(request_json)
            .map_err(|e| anyhow::anyhow!("Invalid JSON: {e}"))?;

        let id = request.get("id").cloned();
        let method = request
            .get("method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing method"))?
            .to_string();

        info!("Handling JSON-RPC method: {}", method);

        // Route to appropriate handler
        let response = match method.as_str() {
            "initialize" => self.handle_initialize(id, request).await,
            "initialized" => self.handle_initialized(id).await,
            "tools/list" => self.handle_list_tools(id).await,
            "tools/call" => self.handle_call_tool(id, request).await,
            "prompts/list" => self.handle_list_prompts(id).await,
            "prompts/get" => self.handle_get_prompt(id, request).await,
            "resources/list" => self.handle_list_resources(id).await,
            "resources/read" => self.handle_read_resource(id, request).await,
            "resources/templates/list" => self.handle_list_resource_templates(id).await,
            "ping" => self.handle_ping(id).await,
            _ => self.error_response(id, -32601, format!("Method not found: {method}")),
        };

        // Record metrics
        let duration = start_time.elapsed().as_secs_f64();
        let status = if response.get("error").is_some() {
            "error"
        } else {
            "success"
        };

        // Use "json_rpc" as transport label since this handler is transport-agnostic
        // The transport-specific handlers should record their own metrics if needed
        metrics::record_request("json_rpc", &method, status, duration);

        if status == "error" {
            metrics::record_error("json_rpc", "method_execution_failed");
        }

        Ok(serde_json::to_string(&response)?)
    }

    /// Handle initialize request
    #[instrument(skip(self, _request))]
    async fn handle_initialize(&self, id: Option<Value>, _request: Value) -> Value {
        info!("Initialize request");

        let result = InitializeResult {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability { list_changed: None }),
                prompts: Some(PromptsCapability { list_changed: None }),
                resources: Some(ResourcesCapability {
                    subscribe: None,
                    list_changed: None,
                }),
                logging: None,
                completions: None,
                experimental: None,
                tasks: None,
            },
            server_info: Implementation {
                name: self.server_info.name.clone(),
                title: None,
                version: self.server_info.version.clone(),
                icons: None,
                website_url: None,
            },
            instructions: Some("MCP Boilerplate Rust Server - Multi-transport support".to_string()),
        };

        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        })
    }

    /// Handle initialized notification
    async fn handle_initialized(&self, _id: Option<Value>) -> Value {
        info!("Initialized notification received");
        json!({
            "jsonrpc": "2.0",
            "result": null
        })
    }

    /// Handle tools/list request
    async fn handle_list_tools(&self, id: Option<Value>) -> Value {
        info!("List tools request");

        let tools = vec![
            Tool {
                name: "echo".to_string().into(),
                title: None,
                description: Some("Echo back a message".into()),
                input_schema: value_to_schema(json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "Message to echo back"
                        }
                    },
                    "required": ["message"]
                })),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "ping".to_string().into(),
                title: None,
                description: Some("Simple ping-pong test to verify connection".into()),
                input_schema: value_to_schema(json!({
                    "type": "object",
                    "properties": {}
                })),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "info".to_string().into(),
                title: None,
                description: Some("Get information about the server capabilities".into()),
                input_schema: value_to_schema(json!({
                    "type": "object",
                    "properties": {}
                })),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "calculate".to_string().into(),
                title: None,
                description: Some("Perform basic arithmetic operations (add, subtract, multiply, divide, modulo, power)".into()),
                input_schema: value_to_schema(json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": ["add", "subtract", "multiply", "divide", "modulo", "power"],
                            "description": "Arithmetic operation to perform"
                        },
                        "a": {
                            "type": "number",
                            "description": "First operand"
                        },
                        "b": {
                            "type": "number",
                            "description": "Second operand"
                        }
                    },
                    "required": ["operation", "a", "b"]
                })),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "evaluate".to_string().into(),
                title: None,
                description: Some("Evaluate mathematical expressions with support for parentheses and operator precedence".into()),
                input_schema: value_to_schema(json!({
                    "type": "object",
                    "properties": {
                        "expression": {
                            "type": "string",
                            "description": "Mathematical expression to evaluate (e.g., '2 + 3 * 4', '(10 - 5) * 2')"
                        }
                    },
                    "required": ["expression"]
                })),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "long_task".to_string().into(),
                title: None,
                description: Some("Simulate a long-running task with configurable duration".into()),
                input_schema: value_to_schema(json!({
                    "type": "object",
                    "properties": {
                        "duration_seconds": {
                            "type": "number",
                            "description": "Duration in seconds (max 60)"
                        },
                        "task_name": {
                            "type": "string",
                            "description": "Name of the task"
                        }
                    },
                    "required": ["duration_seconds"]
                })),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "process_with_progress".to_string().into(),
                title: None,
                description: Some("Process items with progress updates (demonstrates progress reporting)".into()),
                input_schema: value_to_schema(json!({
                    "type": "object",
                    "properties": {
                        "items": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Items to process"
                        }
                    },
                    "required": ["items"]
                })),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "batch_process".to_string().into(),
                title: None,
                description: Some("Process multiple operations in batch".into()),
                input_schema: value_to_schema(json!({
                    "type": "object",
                    "properties": {
                        "operations": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Operations to process"
                        }
                    },
                    "required": ["operations"]
                })),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "transform_data".to_string().into(),
                title: None,
                description: Some("Transform data using specified transformation".into()),
                input_schema: value_to_schema(json!({
                    "type": "object",
                    "properties": {
                        "data": {
                            "type": "string",
                            "description": "Data to transform"
                        },
                        "transformation": {
                            "type": "string",
                            "enum": ["uppercase", "lowercase", "reverse"],
                            "description": "Transformation to apply"
                        }
                    },
                    "required": ["data", "transformation"]
                })),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "simulate_upload".to_string().into(),
                title: None,
                description: Some("Simulate file upload with size validation".into()),
                input_schema: value_to_schema(json!({
                    "type": "object",
                    "properties": {
                        "filename": {
                            "type": "string",
                            "description": "File name"
                        },
                        "size_bytes": {
                            "type": "number",
                            "description": "File size in bytes"
                        }
                    },
                    "required": ["filename", "size_bytes"]
                })),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "health_check".to_string().into(),
                title: None,
                description: Some("Check server health status".into()),
                input_schema: value_to_schema(json!({
                    "type": "object",
                    "properties": {}
                })),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
        ];

        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": tools
            }
        })
    }

    /// Handle tools/call request
    #[instrument(skip(self, request))]
    async fn handle_call_tool(&self, id: Option<Value>, request: Value) -> Value {
        let start_time = std::time::Instant::now();

        let params = match request.get("params") {
            Some(p) => p,
            None => return self.error_response(id, -32602, "Missing params".to_string()),
        };

        let tool_name = match params.get("name").and_then(|v| v.as_str()) {
            Some(name) => name,
            None => return self.error_response(id, -32602, "Missing tool name".to_string()),
        };

        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        info!("Calling tool: {} with args: {:?}", tool_name, arguments);

        // Execute tool using internal methods
        let result = match tool_name {
            "echo" => self.execute_echo(arguments).await,
            "ping" => self.execute_ping().await,
            "info" => self.execute_info().await,
            "calculate" => self.execute_calculate(arguments).await,
            "evaluate" => self.execute_evaluate(arguments).await,
            "long_task" => self.execute_long_task(arguments).await,
            "process_with_progress" => self.execute_process_with_progress(arguments).await,
            "batch_process" => self.execute_batch_process(arguments).await,
            "transform_data" => self.execute_transform_data(arguments).await,
            "simulate_upload" => self.execute_simulate_upload(arguments).await,
            "health_check" => self.execute_health_check().await,
            _ => Err(format!("Unknown tool: {tool_name}")),
        };

        // Record metrics
        let duration = start_time.elapsed().as_secs_f64();
        let status = if result.is_ok() { "success" } else { "error" };
        metrics::record_tool_invocation(tool_name, status, duration);

        match result {
            Ok(content) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": content,
                    "isError": false
                }
            }),
            Err(error) => self.error_response(id, -32603, error),
        }
    }

    /// Handle prompts/list request
    async fn handle_list_prompts(&self, id: Option<Value>) -> Value {
        info!("List prompts request");

        let templates = get_available_prompts();
        let prompts: Vec<Value> = templates
            .iter()
            .map(|t| {
                json!({
                    "name": t.name,
                    "description": t.description,
                    "arguments": t.arguments.iter().map(|a| json!({
                        "name": a.name,
                        "description": a.description,
                        "required": a.required
                    })).collect::<Vec<_>>()
                })
            })
            .collect();

        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "prompts": prompts
            }
        })
    }

    /// Handle prompts/get request
    async fn handle_get_prompt(&self, id: Option<Value>, request: Value) -> Value {
        let params = match request.get("params") {
            Some(p) => p,
            None => return self.error_response(id, -32602, "Missing params".to_string()),
        };

        let prompt_name = match params.get("name").and_then(|v| v.as_str()) {
            Some(name) => name,
            None => return self.error_response(id, -32602, "Missing prompt name".to_string()),
        };

        let arguments = params
            .get("arguments")
            .and_then(|v| v.as_object())
            .map(|m| {
                m.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect::<std::collections::HashMap<_, _>>()
            })
            .unwrap_or_default();

        // Generate prompt based on name
        let result = match prompt_name {
            "code_review" => {
                let language = arguments.get("language").map(|s| s.as_str()).unwrap_or("code");
                let focus = arguments.get("focus").map(|s| s.as_str()).unwrap_or("general");
                json!({
                    "description": format!("Code review prompt for {} with focus on {}", language, focus),
                    "messages": [{
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": format!(
                                "Please review the following {} code with a focus on {}. \
                                Provide detailed feedback on:\n\
                                1. Code quality and best practices\n\
                                2. Potential bugs or issues\n\
                                3. Performance considerations\n\
                                4. Security vulnerabilities\n\
                                5. Suggestions for improvement",
                                language, focus
                            )
                        }
                    }]
                })
            }
            "explain_code" => {
                let complexity = arguments.get("complexity").map(|s| s.as_str()).unwrap_or("intermediate");
                let level_desc = match complexity {
                    "beginner" => "in simple terms suitable for beginners",
                    "advanced" => "with technical depth for experienced developers",
                    _ => "at an intermediate level",
                };
                json!({
                    "description": format!("Code explanation prompt at {} level", complexity),
                    "messages": [{
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": format!(
                                "Please explain the following code {}. Include:\n\
                                1. What the code does\n\
                                2. How it works step by step\n\
                                3. Key concepts and patterns used\n\
                                4. Any important considerations",
                                level_desc
                            )
                        }
                    }]
                })
            }
            "debug_help" => {
                let error_type = arguments.get("error_type").map(|s| s.as_str()).unwrap_or("general");
                json!({
                    "description": format!("Debug assistance prompt for {} errors", error_type),
                    "messages": [{
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": format!(
                                "Help me debug this {} error. Please:\n\
                                1. Analyze the error message and code\n\
                                2. Identify the root cause\n\
                                3. Suggest specific fixes\n\
                                4. Explain why the error occurred\n\
                                5. Recommend preventive measures",
                                error_type
                            )
                        }
                    }]
                })
            }
            _ => {
                return self.error_response(id, -32602, format!("Unknown prompt: {prompt_name}"));
            }
        };

        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        })
    }

    /// Handle resources/list request
    async fn handle_list_resources(&self, id: Option<Value>) -> Value {
        info!("List resources request");

        let resources = self.resource_registry.list_resources();

        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "resources": resources
            }
        })
    }

    /// Handle resources/read request
    async fn handle_read_resource(&self, id: Option<Value>, request: Value) -> Value {
        let params = match request.get("params") {
            Some(p) => p,
            None => return self.error_response(id, -32602, "Missing params".to_string()),
        };

        let uri = match params.get("uri").and_then(|v| v.as_str()) {
            Some(u) => u,
            None => return self.error_response(id, -32602, "Missing resource URI".to_string()),
        };

        match self.resource_registry.read_resource(uri) {
            Ok(resource) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": resource
            }),
            Err(e) => self.error_response(id, -32602, e.to_string()),
        }
    }

    /// Handle resources/templates/list request
    async fn handle_list_resource_templates(&self, id: Option<Value>) -> Value {
        info!("List resource templates request");

        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "resourceTemplates": []
            }
        })
    }

    /// Handle ping request
    async fn handle_ping(&self, id: Option<Value>) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "status": "pong",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        })
    }

    /// Create JSON-RPC error response
    fn error_response(&self, id: Option<Value>, code: i32, message: String) -> Value {
        error!("Error response: {} - {}", code, message);
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": code,
                "message": message
            }
        })
    }

    // Tool execution methods (using shared tool implementations)

    async fn execute_echo(&self, args: Value) -> Result<Vec<Value>, String> {
        let message = args
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or("Missing message parameter")?;

        let response =
            create_echo_response(message.to_string()).map_err(|e| format!("Echo error: {e}"))?;

        Ok(vec![json!({
            "type": "text",
            "text": serde_json::to_string(&response).unwrap()
        })])
    }

    async fn execute_ping(&self) -> Result<Vec<Value>, String> {
        let response = create_ping_response();
        Ok(vec![json!({
            "type": "text",
            "text": serde_json::to_string(&response).unwrap()
        })])
    }

    async fn execute_info(&self) -> Result<Vec<Value>, String> {
        let response = create_info_response();
        Ok(vec![json!({
            "type": "text",
            "text": serde_json::to_string(&response).unwrap()
        })])
    }

    async fn execute_calculate(&self, args: Value) -> Result<Vec<Value>, String> {
        let operation = args
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or("Missing operation parameter")?;

        let a = args
            .get("a")
            .and_then(|v| v.as_f64())
            .ok_or("Missing or invalid parameter 'a'")?;

        let b = args
            .get("b")
            .and_then(|v| v.as_f64())
            .ok_or("Missing or invalid parameter 'b'")?;

        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                a / b
            }
            "modulo" => a % b,
            "power" => a.powf(b),
            _ => return Err(format!("Unknown operation: {operation}")),
        };

        let response = CalculateResponse {
            operation: operation.to_string(),
            a,
            b,
            result,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        Ok(vec![json!({
            "type": "text",
            "text": serde_json::to_string(&response).unwrap()
        })])
    }

    async fn execute_evaluate(&self, args: Value) -> Result<Vec<Value>, String> {
        let expression = args
            .get("expression")
            .and_then(|v| v.as_str())
            .ok_or("Missing expression parameter")?;

        // Simple expression evaluator (reuse from stdio_server.rs logic)
        let result = self
            .evaluate_expression(expression)
            .map_err(|e| format!("Evaluation error: {e}"))?;

        let response = EvaluateResponse {
            expression: expression.to_string(),
            result,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        Ok(vec![json!({
            "type": "text",
            "text": serde_json::to_string(&response).unwrap()
        })])
    }

    async fn execute_long_task(&self, args: Value) -> Result<Vec<Value>, String> {
        let duration = args
            .get("duration_seconds")
            .and_then(|v| v.as_u64())
            .ok_or("Missing duration_seconds parameter")?;

        let task_name = args
            .get("task_name")
            .and_then(|v| v.as_str())
            .unwrap_or("unnamed_task");

        if duration > 60 {
            return Err("Duration exceeds maximum of 60 seconds".to_string());
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(duration)).await;

        let response = create_long_task_response(task_name.to_string(), duration as u32);

        Ok(vec![json!({
            "type": "text",
            "text": serde_json::to_string(&response).unwrap()
        })])
    }

    async fn execute_process_with_progress(&self, args: Value) -> Result<Vec<Value>, String> {
        let items = args
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or("Missing items parameter")?;

        let items_str: Vec<String> = items
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        let response = create_process_response(items_str);

        Ok(vec![json!({
            "type": "text",
            "text": serde_json::to_string(&response).unwrap()
        })])
    }

    async fn execute_batch_process(&self, args: Value) -> Result<Vec<Value>, String> {
        let operations = args
            .get("operations")
            .and_then(|v| v.as_array())
            .ok_or("Missing operations parameter")?;

        let ops_str: Vec<String> = operations
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        let response = create_batch_response(ops_str);

        Ok(vec![json!({
            "type": "text",
            "text": serde_json::to_string(&response).unwrap()
        })])
    }

    async fn execute_transform_data(&self, args: Value) -> Result<Vec<Value>, String> {
        let data = args
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or("Missing data parameter")?;

        let transformation = args
            .get("transformation")
            .and_then(|v| v.as_str())
            .ok_or("Missing transformation parameter")?;

        let response = create_transform_response(data.to_string(), transformation.to_string())
            .map_err(|e| format!("Transform error: {e}"))?;

        Ok(vec![json!({
            "type": "text",
            "text": serde_json::to_string(&response).unwrap()
        })])
    }

    async fn execute_simulate_upload(&self, args: Value) -> Result<Vec<Value>, String> {
        let filename = args
            .get("filename")
            .and_then(|v| v.as_str())
            .ok_or("Missing filename parameter")?;

        let size_bytes = args
            .get("size_bytes")
            .and_then(|v| v.as_u64())
            .ok_or("Missing size_bytes parameter")?;

        let response = create_upload_response(filename.to_string(), size_bytes)
            .map_err(|e| format!("Upload error: {e}"))?;

        Ok(vec![json!({
            "type": "text",
            "text": serde_json::to_string(&response).unwrap()
        })])
    }

    async fn execute_health_check(&self) -> Result<Vec<Value>, String> {
        let response = create_health_response();

        Ok(vec![json!({
            "type": "text",
            "text": serde_json::to_string(&response).unwrap()
        })])
    }

    // Expression evaluator (simplified version from stdio_server.rs)
    fn evaluate_expression(&self, expr: &str) -> Result<f64, String> {
        let expr = expr.replace(" ", "");
        self.parse_expression(&expr, &mut 0)
    }

    fn parse_expression(&self, expr: &str, pos: &mut usize) -> Result<f64, String> {
        let mut result = self.parse_term(expr, pos)?;

        while *pos < expr.len() {
            let ch = expr.chars().nth(*pos).unwrap();
            if ch == '+' || ch == '-' {
                *pos += 1;
                let right = self.parse_term(expr, pos)?;
                if ch == '+' {
                    result += right;
                } else {
                    result -= right;
                }
            } else {
                break;
            }
        }

        Ok(result)
    }

    fn parse_term(&self, expr: &str, pos: &mut usize) -> Result<f64, String> {
        let mut result = self.parse_factor(expr, pos)?;

        while *pos < expr.len() {
            let ch = expr.chars().nth(*pos).unwrap();
            if ch == '*' || ch == '/' || ch == '%' {
                *pos += 1;
                let right = self.parse_factor(expr, pos)?;
                match ch {
                    '*' => result *= right,
                    '/' => {
                        if right == 0.0 {
                            return Err("Division by zero".to_string());
                        }
                        result /= right;
                    }
                    '%' => result %= right,
                    _ => {}
                }
            } else {
                break;
            }
        }

        Ok(result)
    }

    fn parse_factor(&self, expr: &str, pos: &mut usize) -> Result<f64, String> {
        if *pos >= expr.len() {
            return Err("Unexpected end of expression".to_string());
        }

        let ch = expr.chars().nth(*pos).unwrap();

        if ch == '(' {
            *pos += 1;
            let result = self.parse_expression(expr, pos)?;
            if *pos >= expr.len() || expr.chars().nth(*pos).unwrap() != ')' {
                return Err("Mismatched parentheses".to_string());
            }
            *pos += 1;
            Ok(result)
        } else if ch == '-' {
            *pos += 1;
            Ok(-self.parse_factor(expr, pos)?)
        } else {
            self.parse_number(expr, pos)
        }
    }

    fn parse_number(&self, expr: &str, pos: &mut usize) -> Result<f64, String> {
        let start = *pos;
        let mut has_dot = false;

        while *pos < expr.len() {
            let ch = expr.chars().nth(*pos).unwrap();
            if ch.is_ascii_digit() {
                *pos += 1;
            } else if ch == '.' && !has_dot {
                has_dot = true;
                *pos += 1;
            } else {
                break;
            }
        }

        if start == *pos {
            return Err("Expected number".to_string());
        }

        expr[start..*pos]
            .parse::<f64>()
            .map_err(|_| "Invalid number".to_string())
    }
}

impl Default for ProtocolHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_protocol_handler_creation() {
        let handler = ProtocolHandler::new();
        assert_eq!(handler.server_info.name, "MCP Boilerplate Rust");
    }

    #[tokio::test]
    async fn test_handle_initialize() {
        let handler = ProtocolHandler::new();
        let request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let response = handler.handle_request(request).await.unwrap();
        println!("Response: {response}");
        assert!(response.contains("protocolVersion") || response.contains("protocol_version"));
        assert!(response.contains("capabilities"));
    }

    #[tokio::test]
    async fn test_handle_tools_list() {
        let handler = ProtocolHandler::new();
        let request = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
        let response = handler.handle_request(request).await.unwrap();
        assert!(response.contains("tools"));
        assert!(response.contains("echo"));
    }

    #[tokio::test]
    async fn test_handle_echo_tool() {
        let handler = ProtocolHandler::new();
        let request = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"echo","arguments":{"message":"test"}}}"#;
        let response = handler.handle_request(request).await.unwrap();
        assert!(response.contains("result"));
        assert!(!response.contains("error"));
    }

    #[tokio::test]
    async fn test_handle_ping() {
        let handler = ProtocolHandler::new();
        let request = r#"{"jsonrpc":"2.0","id":4,"method":"ping","params":{}}"#;
        let response = handler.handle_request(request).await.unwrap();
        assert!(response.contains("pong"));
    }

    #[tokio::test]
    async fn test_invalid_json() {
        let handler = ProtocolHandler::new();
        let result = handler.handle_request("invalid json").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unknown_method() {
        let handler = ProtocolHandler::new();
        let request = r#"{"jsonrpc":"2.0","id":5,"method":"unknown_method","params":{}}"#;
        let response = handler.handle_request(request).await.unwrap();
        assert!(response.contains("error"));
        assert!(response.contains("Method not found"));
    }

    #[tokio::test]
    async fn test_calculate_tool() {
        let handler = ProtocolHandler::new();
        let request = r#"{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"calculate","arguments":{"operation":"add","a":5,"b":3}}}"#;
        let response = handler.handle_request(request).await.unwrap();
        assert!(response.contains("result"));
        assert!(response.contains("8"));
    }

    #[tokio::test]
    async fn test_evaluate_tool() {
        let handler = ProtocolHandler::new();
        let request = r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"evaluate","arguments":{"expression":"2+3*4"}}}"#;
        let response = handler.handle_request(request).await.unwrap();
        assert!(response.contains("result"));
        assert!(response.contains("14"));
    }

    #[test]
    fn test_expression_evaluator() {
        let handler = ProtocolHandler::new();
        assert_eq!(handler.evaluate_expression("2+3").unwrap(), 5.0);
        assert_eq!(handler.evaluate_expression("2*3+4").unwrap(), 10.0);
        assert_eq!(handler.evaluate_expression("(2+3)*4").unwrap(), 20.0);
    }
}
