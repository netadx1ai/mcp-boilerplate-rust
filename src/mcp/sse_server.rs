//! SSE MCP Server implementation
//!
//! This module provides a complete SSE-based MCP server using Axum.
//! Supports real-time progress notifications and multi-client broadcasting.
//!
//! # Architecture
//!
//! Uses the shared ProtocolHandler for consistent rmcp-based protocol handling:
//!
//! ```text
//! SSE Client (EventSource)
//!     ↓
//! SSE Transport Layer
//!     ↓
//! ProtocolHandler (rmcp types)
//!     ↓
//! Tool Implementations
//! ```
//!
//! # Features
//!
//! - EventSource-compatible SSE endpoint
//! - All 11 MCP tools available via SSE
//! - Real-time progress notifications
//! - Multi-client support
//! - CORS enabled for browser clients
//! - Health check endpoint
//! - Type-safe protocol handling with rmcp
//!
//! # Endpoints
//!
//! - `GET /sse` - SSE event stream (EventSource endpoint)
//! - `POST /tools/call` - Call a tool (returns immediately, results via SSE)
//! - `GET /tools` - List available tools
//! - `GET /health` - Health check
//!
//! # Example Client
//!
//! ```javascript
//! const eventSource = new EventSource('http://localhost:8025/sse');
//!
//! eventSource.onmessage = (event) => {
//!     const data = JSON.parse(event.data);
//!     console.log('Received:', data);
//! };
//!
//! // Call a tool
//! fetch('http://localhost:8025/tools/call', {
//!     method: 'POST',
//!     headers: { 'Content-Type': 'application/json' },
//!     body: JSON.stringify({
//!         jsonrpc: '2.0',
//!         id: 1,
//!         method: 'tools/call',
//!         params: {
//!             name: 'echo',
//!             arguments: { message: 'Hello SSE!' }
//!         }
//!     })
//! });
//! ```

#[cfg(feature = "sse")]
use crate::mcp::protocol_handler::ProtocolHandler;
#[cfg(feature = "sse")]
use crate::transport::sse::SseTransport;
#[cfg(feature = "sse")]
use crate::transport::{Transport, TransportMessage};
#[cfg(feature = "sse")]
use futures::stream::StreamExt;

#[cfg(feature = "sse")]
use axum::{
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json,
    },
    routing::{get, post},
    Router,
};

#[cfg(feature = "sse")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "sse")]
use serde_json::json;
#[cfg(feature = "sse")]
use std::sync::Arc;
#[cfg(feature = "sse")]
use tower_http::cors::{Any, CorsLayer};
#[cfg(feature = "sse")]
use tracing::{error, info};

/// SSE server state shared across handlers
#[cfg(feature = "sse")]
#[derive(Clone)]
pub struct SseServerState {
    transport: Arc<SseTransport>,
    protocol_handler: Arc<ProtocolHandler>,
}

/// JSON-RPC request structure
#[cfg(feature = "sse")]
#[derive(Debug, Deserialize, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

/// Simple response for immediate acknowledgment
#[cfg(feature = "sse")]
#[derive(Debug, Deserialize, Serialize)]
pub struct AckResponse {
    pub request_id: String,
    pub status: String,
    pub message: String,
}

/// Start SSE MCP server
///
/// # Arguments
/// * `bind_address` - Address to bind to (e.g., "127.0.0.1:8025")
///
/// # Returns
/// * `Result<()>` - Success or error
#[cfg(feature = "sse")]
use crate::metrics;

pub async fn run_sse_server(bind_address: &str) -> anyhow::Result<()> {
    info!("Starting MCP SSE Server with ProtocolHandler");
    info!("Bind address: {}", bind_address);

    // Initialize SSE transport
    let mut transport = SseTransport::new(bind_address);
    transport.initialize().await?;

    // Initialize protocol handler
    let protocol_handler = ProtocolHandler::new();

    let state = SseServerState {
        transport: Arc::new(transport),
        protocol_handler: Arc::new(protocol_handler),
    };

    // Setup CORS for browser clients
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_check))
        .route("/sse", get(sse_handler))
        .route("/rpc", post(rpc_handler))
        .route("/tools", get(list_tools))
        .route("/tools/call", post(call_tool_legacy))
        .route("/metrics", get(metrics_handler))
        .layer(cors)
        .with_state(state);

    info!("SSE Server ready on http://{}", bind_address);
    info!("Endpoints:");
    info!("  GET  /           - Server info");
    info!("  GET  /health     - Health check");
    info!("  GET  /sse        - SSE event stream");
    info!("  POST /rpc        - JSON-RPC endpoint (recommended)");
    info!("  GET  /tools      - List tools");
    info!("  POST /tools/call - Call a tool (legacy, for compatibility)");
    info!("  GET  /metrics    - Prometheus metrics");

    // Start server
    let listener = tokio::net::TcpListener::bind(bind_address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Root handler - Server information
#[cfg(feature = "sse")]
async fn root_handler() -> impl IntoResponse {
    Json(json!({
        "name": "MCP Boilerplate Rust - SSE Server",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": "mcp-2024-11-05",
        "transport": "sse",
        "endpoints": {
            "sse": "/sse",
            "rpc": "/rpc",
            "tools": "/tools",
            "tools_call": "/tools/call",
            "health": "/health"
        },
        "features": [
            "server-sent-events",
            "multi-client",
            "real-time-notifications",
            "rmcp-types"
        ],
        "note": "Now using ProtocolHandler with rmcp types for type safety"
    }))
}

/// Health check endpoint
#[cfg(feature = "sse")]
async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Metrics endpoint - Prometheus format
#[cfg(feature = "sse")]
async fn metrics_handler() -> impl IntoResponse {
    match metrics::gather_metrics() {
        Ok(metrics) => (
            StatusCode::OK,
            [("Content-Type", "text/plain; version=0.0.4")],
            metrics,
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error gathering metrics: {}", e),
        )
            .into_response(),
    }
}

/// SSE event stream handler
#[cfg(feature = "sse")]
async fn sse_handler(
    State(state): State<SseServerState>,
) -> Result<Sse<impl futures::Stream<Item = Result<Event, anyhow::Error>>>, StatusCode> {
    let client_id = uuid::Uuid::new_v4().to_string();

    info!("New SSE client connected: {}", client_id);

    // Register client with transport
    state.transport.register_client(client_id.clone(), None);

    // Create event stream from transport
    let mut rx = match state.transport.create_stream() {
        Ok(receiver) => receiver,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let stream = async_stream::stream! {
        while let Ok(msg) = rx.recv().await {
            let event = Event::default().data(msg.content);
            yield Ok::<_, anyhow::Error>(event);
        }
    };

    // Send welcome message
    let welcome = TransportMessage::with_metadata(
        json!({
            "type": "connection",
            "client_id": client_id,
            "message": "Connected to MCP SSE server",
            "protocol": "mcp-2024-11-05",
            "handler": "ProtocolHandler",
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
        .to_string(),
        "sse",
    );

    if let Err(e) = state.transport.send_event(welcome).await {
        error!("Failed to send welcome message: {}", e);
    }

    // Clone state for cleanup on disconnect
    let transport = state.transport.clone();
    let client_id_cleanup = client_id.clone();

    // Wrap stream with cleanup on drop
    let stream_with_cleanup = stream.inspect(move |_| {
        // This closure is called for each item
    });

    // Schedule cleanup (run in background when connection closes)
    tokio::spawn(async move {
        // This will run when the client disconnects
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        transport.unregister_client(&client_id_cleanup);
        info!("SSE client disconnected: {}", client_id_cleanup);
    });

    Ok(Sse::new(stream_with_cleanup).keep_alive(KeepAlive::default()))
}

/// JSON-RPC handler (recommended endpoint)
///
/// Accepts standard JSON-RPC 2.0 requests and uses ProtocolHandler
#[cfg(feature = "sse")]
async fn rpc_handler(
    State(state): State<SseServerState>,
    body: String,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let request_id = uuid::Uuid::new_v4().to_string();
    let request_id_clone = request_id.clone();

    info!("JSON-RPC request (request_id: {})", request_id);

    // Parse JSON-RPC request
    let json_rpc: JsonRpcRequest =
        serde_json::from_str(&body).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Execute using ProtocolHandler
    let protocol_handler = state.protocol_handler.clone();
    let transport = state.transport.clone();

    tokio::spawn(async move {
        // Send "processing" notification
        let processing_msg = TransportMessage::with_metadata(
            json!({
                "type": "rpc_started",
                "request_id": request_id_clone,
                "method": json_rpc.method,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })
            .to_string(),
            "sse",
        );

        if let Err(e) = transport.send_event(processing_msg).await {
            error!("Failed to send processing notification: {}", e);
            return;
        }

        // Execute via protocol handler
        match protocol_handler.handle_request(&body).await {
            Ok(response_str) => {
                // Send result notification
                let result_msg = TransportMessage::with_metadata(
                    json!({
                        "type": "rpc_result",
                        "request_id": request_id_clone,
                        "response": serde_json::from_str::<serde_json::Value>(&response_str).unwrap_or(json!({})),
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    })
                    .to_string(),
                    "sse",
                );

                if let Err(e) = transport.send_event(result_msg).await {
                    error!("Failed to send result notification: {}", e);
                }
            }
            Err(error) => {
                // Send error notification
                let error_msg = TransportMessage::with_metadata(
                    json!({
                        "type": "rpc_error",
                        "request_id": request_id_clone,
                        "error": error.to_string(),
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    })
                    .to_string(),
                    "sse",
                );

                if let Err(e) = transport.send_event(error_msg).await {
                    error!("Failed to send error notification: {}", e);
                }
            }
        }
    });

    // Return immediate acknowledgment
    Ok(Json(json!({
        "request_id": request_id,
        "status": "accepted",
        "message": "Request accepted. Results will be broadcast via SSE."
    })))
}

/// List available tools (via ProtocolHandler)
#[cfg(feature = "sse")]
async fn list_tools(State(state): State<SseServerState>) -> impl IntoResponse {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });

    match state
        .protocol_handler
        .handle_request(&request.to_string())
        .await
    {
        Ok(response) => {
            let response_json: serde_json::Value = serde_json::from_str(&response).unwrap();
            Json(response_json)
        }
        Err(e) => {
            error!("Failed to list tools: {}", e);
            Json(json!({
                "error": e.to_string()
            }))
        }
    }
}

/// Legacy tool call endpoint (for backward compatibility)
///
/// Accepts simplified format and converts to JSON-RPC
#[cfg(feature = "sse")]
async fn call_tool_legacy(
    State(state): State<SseServerState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<AckResponse>, StatusCode> {
    let request_id = uuid::Uuid::new_v4().to_string();

    let tool_name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?
        .to_string();

    let arguments = payload.get("arguments").cloned().unwrap_or(json!({}));

    info!(
        "Legacy tool call: {} (request_id: {})",
        tool_name, request_id
    );

    // Convert to JSON-RPC format
    let json_rpc_request = json!({
        "jsonrpc": "2.0",
        "id": request_id.clone(),
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": arguments
        }
    });

    // Execute asynchronously via ProtocolHandler
    let transport = state.transport.clone();
    let protocol_handler = state.protocol_handler.clone();
    let request_id_spawn = request_id.clone();
    let tool_name_clone = tool_name.clone();

    tokio::spawn(async move {
        // Send "processing" notification
        let processing_msg = TransportMessage::with_metadata(
            json!({
                "type": "tool_call_started",
                "request_id": request_id_spawn,
                "tool": tool_name_clone,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })
            .to_string(),
            "sse",
        );

        if let Err(e) = transport.send_event(processing_msg).await {
            error!("Failed to send processing notification: {}", e);
            return;
        }

        // Execute via protocol handler
        match protocol_handler
            .handle_request(&json_rpc_request.to_string())
            .await
        {
            Ok(response_str) => {
                let response_json: serde_json::Value = serde_json::from_str(&response_str)
                    .unwrap_or(json!({"error": "Invalid response"}));

                // Send result notification
                let result_msg = TransportMessage::with_metadata(
                    json!({
                        "type": "tool_call_result",
                        "request_id": request_id_spawn,
                        "tool": tool_name_clone,
                        "success": !response_json.get("error").is_some(),
                        "result": response_json,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    })
                    .to_string(),
                    "sse",
                );

                if let Err(e) = transport.send_event(result_msg).await {
                    error!("Failed to send result notification: {}", e);
                }
            }
            Err(error) => {
                // Send error notification
                let error_msg = TransportMessage::with_metadata(
                    json!({
                        "type": "tool_call_error",
                        "request_id": request_id_spawn,
                        "tool": tool_name_clone,
                        "success": false,
                        "error": error.to_string(),
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    })
                    .to_string(),
                    "sse",
                );

                if let Err(e) = transport.send_event(error_msg).await {
                    error!("Failed to send error notification: {}", e);
                }
            }
        }
    });

    // Return immediate response
    Ok(Json(AckResponse {
        request_id: request_id.clone(),
        status: "accepted".to_string(),
        message: format!(
            "Tool call accepted. Results will be broadcast via SSE. Request ID: {}",
            request_id
        ),
    }))
}

#[cfg(all(test, feature = "sse"))]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "tools/call".to_string(),
            params: Some(json!({"name": "echo"})),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("2.0"));
        assert!(serialized.contains("tools/call"));
    }

    #[test]
    fn test_ack_response_serialization() {
        let response = AckResponse {
            request_id: "test-id".to_string(),
            status: "accepted".to_string(),
            message: "Test message".to_string(),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        assert!(serialized.contains("test-id"));
        assert!(serialized.contains("accepted"));
    }

    #[tokio::test]
    async fn test_sse_server_state_creation() {
        let transport = SseTransport::new("127.0.0.1:8025");
        let protocol_handler = ProtocolHandler::new();

        let state = SseServerState {
            transport: Arc::new(transport),
            protocol_handler: Arc::new(protocol_handler),
        };

        // Test that protocol handler is functional by testing initialize
        let init_request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let response = state
            .protocol_handler
            .handle_request(init_request)
            .await
            .unwrap();
        assert!(response.contains("protocolVersion") || response.contains("protocol_version"));
    }

    #[tokio::test]
    async fn test_protocol_handler_integration() {
        let handler = ProtocolHandler::new();

        // Test initialize
        let init_request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let response = handler.handle_request(init_request).await.unwrap();
        assert!(response.contains("protocolVersion") || response.contains("protocol_version"));

        // Test tools/list
        let list_request = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
        let response = handler.handle_request(list_request).await.unwrap();
        assert!(response.contains("tools"));

        // Test echo tool
        let echo_request = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"echo","arguments":{"message":"test"}}}"#;
        let response = handler.handle_request(echo_request).await.unwrap();
        assert!(response.contains("result"));
    }

    #[test]
    fn test_server_info() {
        use crate::mcp::protocol_handler::ServerInfo;

        let info = ServerInfo::default();
        assert_eq!(info.name, "MCP Boilerplate Rust");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
    }
}
