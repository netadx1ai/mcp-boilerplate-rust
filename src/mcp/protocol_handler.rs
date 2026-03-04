//! MCP Protocol Handler for Đấu Trường Vui Backend
//!
//! Stripped-down protocol handler with only the `db` tool.
//! Auth and textgen tools will be added in later tasks.

use anyhow::Result;
use rmcp::model::{
    InitializeResult, Implementation, ProtocolVersion, ServerCapabilities,
    Tool,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{error, info, instrument};

type JsonObject = serde_json::Map<String, Value>;

#[cfg(feature = "postgres")]
use crate::tools::db;

#[cfg(feature = "auth")]
use crate::tools::auth;

use crate::metrics;

/// Helper function to convert Value to Arc<JsonObject>
fn value_to_schema(value: Value) -> Arc<JsonObject> {
    match value {
        Value::Object(map) => Arc::new(map),
        _ => Arc::new(serde_json::Map::new()),
    }
}

/// Protocol handler for HTTP streaming transport
#[derive(Clone)]
pub struct ProtocolHandler {
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
            name: "mcp-dautruongvui-be".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl ProtocolHandler {
    /// Create a new protocol handler
    pub fn new() -> Self {
        Self {
            server_info: ServerInfo::default(),
        }
    }

    /// Handle a JSON-RPC request string and return a JSON-RPC response string
    #[instrument(skip(self, request_str))]
    pub async fn handle_request(&self, request_str: &str) -> Result<String> {
        let request: Value = match serde_json::from_str(request_str) {
            Ok(v) => v,
            Err(e) => {
                let response = self.error_response(None, -32700, format!("Parse error: {e}"));
                return Ok(response.to_string());
            }
        };

        let id = request.get("id").cloned();
        let method = request
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("");

        let response = match method {
            "initialize" => self.handle_initialize(id).await,
            "initialized" => self.handle_initialized().await,
            "tools/list" => self.handle_list_tools(id).await,
            "tools/call" => self.handle_call_tool(id, request).await,
            "ping" => self.handle_ping(id).await,
            _ => self.error_response(id, -32601, format!("Method not found: {method}")),
        };

        Ok(response.to_string())
    }

    /// Handle initialize request
    async fn handle_initialize(&self, id: Option<Value>) -> Value {
        info!("Initialize request received");

        let result = InitializeResult {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: self.server_info.name.clone(),
                version: self.server_info.version.clone(),
                title: None,
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Đấu Trường Vui MCP Backend. Tools: auth (PostgreSQL auth), db (PostgreSQL via PostgREST).".to_string(),
            ),
        };

        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": serde_json::to_value(result).unwrap_or(json!({}))
        })
    }

    /// Handle initialized notification
    async fn handle_initialized(&self) -> Value {
        info!("Client initialized");
        json!({})
    }

    /// Handle tools/list request
    async fn handle_list_tools(&self, id: Option<Value>) -> Value {
        info!("List tools request");

        let mut tools: Vec<Tool> = Vec::new();

        #[cfg(feature = "postgres")]
        tools.push(Tool {
            name: "db".to_string().into(),
            title: None,
            description: Some(
                "PostgreSQL database tool via PostgREST. Actions: query, insert, update, delete, upsert, rpc, list_tables, describe. Supports filters (eq, neq, gt, gte, lt, lte, like, ilike, is, in, not, contains, containedBy, overlaps).".into()
            ),
            input_schema: value_to_schema(json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["query", "insert", "update", "delete", "upsert", "rpc", "list_tables", "describe"],
                        "description": "Database action to perform"
                    },
                    "table": {
                        "type": "string",
                        "description": "Target table name (required for CRUD actions)"
                    },
                    "select": {
                        "description": "Columns to select (string or array)"
                    },
                    "filters": {
                        "type": "object",
                        "description": "Filter conditions: { \"col\": { \"op\": value } }"
                    },
                    "data": {
                        "description": "Data payload for insert/update/upsert"
                    },
                    "order": {
                        "description": "Order spec: [{ \"column\": \"name\", \"direction\": \"asc\" }]"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Max rows to return"
                    },
                    "offset": {
                        "type": "number",
                        "description": "Rows to skip"
                    },
                    "options": {
                        "type": "object",
                        "description": "Options: count (exact), single (bool), return (minimal/representation)"
                    },
                    "function_name": {
                        "type": "string",
                        "description": "Function name for rpc action"
                    },
                    "params": {
                        "type": "object",
                        "description": "Parameters for rpc action"
                    },
                    "conflict": {
                        "type": "string",
                        "description": "Conflict columns for upsert (comma-separated)"
                    },
                    "token": {
                        "type": "string",
                        "description": "JWT token for PostgREST authorization"
                    }
                },
                "required": ["action"]
            })),
            output_schema: None,
            annotations: None,
            icons: None,
            meta: None,
        });

        #[cfg(feature = "auth")]
        tools.push(Tool {
            name: "auth".to_string().into(),
            title: None,
            description: Some(
                "Authentication tool for Đấu Trường Vui. Actions: login, register, google_auth, get_user_info, check_role. Uses PostgreSQL dtv_users table via PostgREST, bcrypt passwords, JWT tokens.".into()
            ),
            input_schema: value_to_schema(json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["login", "register", "google_auth", "get_user_info", "check_role"],
                        "description": "Auth action to perform"
                    },
                    "email": {
                        "type": "string",
                        "description": "User email (login, register)"
                    },
                    "password": {
                        "type": "string",
                        "description": "User password (login, register)"
                    },
                    "name": {
                        "type": "string",
                        "description": "Display name (register)"
                    },
                    "google_token": {
                        "type": "string",
                        "description": "Google OAuth ID token (google_auth)"
                    },
                    "token": {
                        "type": "string",
                        "description": "JWT token (get_user_info, check_role)"
                    }
                },
                "required": ["action"]
            })),
            output_schema: None,
            annotations: None,
            icons: None,
            meta: None,
        });

        // textgen tool will be added in Phase E (Task 11)

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

        let result = match tool_name {
            #[cfg(feature = "postgres")]
            "db" => self.execute_db(arguments).await,
            #[cfg(feature = "auth")]
            "auth" => self.execute_auth(arguments).await,
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

    /// Handle ping request
    async fn handle_ping(&self, id: Option<Value>) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {}
        })
    }

    /// Build a JSON-RPC error response
    fn error_response(&self, id: Option<Value>, code: i32, message: String) -> Value {
        error!("Error {}: {}", code, message);
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": code,
                "message": message
            }
        })
    }

    // ==================== Tool Executors ====================

    #[cfg(feature = "postgres")]
    async fn execute_db(&self, args: Value) -> Result<Vec<Value>, String> {
        let req: db::DbRequest = serde_json::from_value(args)
            .map_err(|e| format!("Invalid db request: {e}"))?;
        let client = db::get_client();
        let config = db::get_config();
        let response = db::execute_db(client, config, &req).await;
        let text = serde_json::to_string_pretty(&response)
            .unwrap_or_else(|_| format!("{response:?}"));
        Ok(vec![json!({
            "type": "text",
            "text": text
        })])
    }

    #[cfg(feature = "auth")]
    async fn execute_auth(&self, args: Value) -> Result<Vec<Value>, String> {
        let response = auth::execute(args).await;
        let text = serde_json::to_string_pretty(&response)
            .unwrap_or_else(|_| format!("{response:?}"));
        Ok(vec![json!({
            "type": "text",
            "text": text
        })])
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

    #[test]
    fn test_protocol_handler_creation() {
        let handler = ProtocolHandler::new();
        assert_eq!(handler.server_info.name, "mcp-dautruongvui-be");
    }

    #[tokio::test]
    async fn test_handle_initialize() {
        let handler = ProtocolHandler::new();
        let request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let response = handler.handle_request(request).await.unwrap();
        let parsed: Value = serde_json::from_str(&response).unwrap();
        assert!(parsed.get("result").is_some());
    }

    #[tokio::test]
    async fn test_handle_tools_list() {
        let handler = ProtocolHandler::new();
        let request = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#;
        let response = handler.handle_request(request).await.unwrap();
        let parsed: Value = serde_json::from_str(&response).unwrap();
        let tools = parsed["result"]["tools"].as_array().unwrap();
        // With postgres feature, should have db tool
        assert!(!tools.is_empty() || cfg!(not(feature = "postgres")));
    }

    #[tokio::test]
    async fn test_handle_ping() {
        let handler = ProtocolHandler::new();
        let request = r#"{"jsonrpc":"2.0","id":1,"method":"ping","params":{}}"#;
        let response = handler.handle_request(request).await.unwrap();
        let parsed: Value = serde_json::from_str(&response).unwrap();
        assert!(parsed.get("result").is_some());
    }

    #[tokio::test]
    async fn test_invalid_json() {
        let handler = ProtocolHandler::new();
        let response = handler.handle_request("not json").await.unwrap();
        let parsed: Value = serde_json::from_str(&response).unwrap();
        assert!(parsed.get("error").is_some());
    }

    #[tokio::test]
    async fn test_unknown_method() {
        let handler = ProtocolHandler::new();
        let request = r#"{"jsonrpc":"2.0","id":1,"method":"nonexistent","params":{}}"#;
        let response = handler.handle_request(request).await.unwrap();
        let parsed: Value = serde_json::from_str(&response).unwrap();
        assert!(parsed.get("error").is_some());
    }
}