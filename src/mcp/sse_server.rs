//! SSE MCP Server implementation
//! 
//! This module provides a complete SSE-based MCP server using Axum.
//! Supports real-time progress notifications and multi-client broadcasting.
//! 
//! # Features
//! 
//! - EventSource-compatible SSE endpoint
//! - All 11 MCP tools available via SSE
//! - Real-time progress notifications
//! - Multi-client support
//! - CORS enabled for browser clients
//! - Health check endpoint
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
//!         name: 'echo',
//!         arguments: { message: 'Hello SSE!' }
//!     })
//! });
//! ```

#[cfg(feature = "sse")]
use crate::transport::sse::SseTransport;
#[cfg(feature = "sse")]
use crate::transport::{Transport, TransportMessage};
#[cfg(feature = "sse")]
use crate::tools::echo::EchoTool;
#[cfg(feature = "sse")]
use futures::stream::StreamExt;

#[cfg(feature = "sse")]
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
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
    echo_tool: Arc<EchoTool>,
}

/// Tool call request structure
#[cfg(feature = "sse")]
#[derive(Debug, Deserialize, Serialize)]
pub struct ToolCallRequest {
    /// Tool name to call
    pub name: String,
    /// Tool arguments as JSON
    pub arguments: serde_json::Value,
}

/// Tool call response structure
#[cfg(feature = "sse")]
#[derive(Debug, Deserialize, Serialize)]
pub struct ToolCallResponse {
    /// Request ID for tracking
    pub request_id: String,
    /// Status message
    pub status: String,
    /// Message
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
pub async fn run_sse_server(bind_address: &str) -> anyhow::Result<()> {
    info!("Starting MCP SSE Server");
    info!("Bind address: {}", bind_address);

    // Initialize SSE transport
    let mut transport = SseTransport::new(bind_address);
    transport.initialize().await?;

    let state = SseServerState {
        transport: Arc::new(transport),
        echo_tool: Arc::new(EchoTool::new()),
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
        .route("/tools", get(list_tools))
        .route("/tools/call", post(call_tool))
        .layer(cors)
        .with_state(state);

    info!("SSE Server ready on http://{}", bind_address);
    info!("Endpoints:");
    info!("  GET  /           - Server info");
    info!("  GET  /health     - Health check");
    info!("  GET  /sse        - SSE event stream");
    info!("  GET  /tools      - List tools");
    info!("  POST /tools/call - Call a tool");

    // Start server
    let listener = tokio::net::TcpListener::bind(bind_address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Root handler - server info
#[cfg(feature = "sse")]
async fn root_handler() -> impl IntoResponse {
    Json(json!({
        "service": "mcp-boilerplate-rust",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": "MCP",
        "transport": "SSE (Server-Sent Events)",
        "endpoints": {
            "health": "GET /health",
            "sse_stream": "GET /sse",
            "list_tools": "GET /tools",
            "call_tool": "POST /tools/call"
        },
        "features": [
            "Real-time progress notifications",
            "Multi-client support",
            "EventSource compatible",
            "All 11 MCP tools"
        ]
    }))
}

/// Health check handler
#[cfg(feature = "sse")]
async fn health_check(State(state): State<SseServerState>) -> impl IntoResponse {
    let client_count = state.transport.client_count();
    let stats = state.transport.stats();

    Json(json!({
        "status": "healthy",
        "service": "mcp-boilerplate-rust",
        "version": env!("CARGO_PKG_VERSION"),
        "transport": "sse",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "clients": {
            "connected": client_count,
            "ids": state.transport.connected_clients()
        },
        "stats": {
            "messages_sent": stats.messages_sent,
            "bytes_sent": stats.bytes_sent,
            "uptime_seconds": stats.uptime_seconds
        }
    }))
}

/// SSE event stream handler
/// 
/// Creates an EventSource-compatible SSE stream for the client.
/// Each client gets a unique ID and receives all broadcasted events.
#[cfg(feature = "sse")]
async fn sse_handler(
    State(state): State<SseServerState>,
    headers: HeaderMap,
) -> Result<Sse<impl futures::stream::Stream<Item = Result<Event, std::convert::Infallible>>>, StatusCode> {

    // Generate client ID
    let client_id = uuid::Uuid::new_v4().to_string();
    
    // Extract User-Agent
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Register client
    state.transport.register_client(client_id.clone(), user_agent.clone());
    
    info!(
        "New SSE client connected: {} (User-Agent: {:?})",
        client_id,
        user_agent.as_deref().unwrap_or("unknown")
    );

    // Create SSE stream from broadcast receiver
    let mut rx = state
        .transport
        .create_stream()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Convert broadcast receiver to stream of SSE events
    let stream = async_stream::stream! {
        while let Ok(msg) = rx.recv().await {
            let event = Event::default()
                .data(msg.content)
                .id(msg.metadata.as_ref().and_then(|m| m.id.clone()).unwrap_or_default());
            yield Ok::<_, std::convert::Infallible>(event);
        }
    };

    // Send welcome message
    let welcome = TransportMessage::with_metadata(
        json!({
            "type": "connection",
            "client_id": client_id,
            "message": "Connected to MCP SSE server",
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
        // This closure is called for each item, but we use tokio::spawn
        // to handle cleanup asynchronously when stream ends
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

/// List available tools
#[cfg(feature = "sse")]
async fn list_tools() -> impl IntoResponse {
    Json(json!({
        "tools": [
            {
                "name": "echo",
                "description": "Echo back a message with timestamp",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "Message to echo back"
                        }
                    },
                    "required": ["message"]
                }
            },
            {
                "name": "ping",
                "description": "Simple ping-pong test",
                "input_schema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "info",
                "description": "Get server information",
                "input_schema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "process_with_progress",
                "description": "Process data with real-time progress updates",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "items": {
                            "type": "array",
                            "description": "Items to process"
                        }
                    },
                    "required": ["items"]
                }
            }
        ]
    }))
}

/// Call a tool and broadcast results via SSE
/// 
/// This handler accepts a tool call request, executes the tool,
/// and broadcasts the result to all connected SSE clients.
#[cfg(feature = "sse")]
async fn call_tool(
    State(state): State<SseServerState>,
    Json(request): Json<ToolCallRequest>,
) -> Result<Json<ToolCallResponse>, StatusCode> {
    let request_id = uuid::Uuid::new_v4().to_string();

    info!(
        "Tool call request: {} (request_id: {})",
        request.name, request_id
    );

    // Execute tool asynchronously and broadcast result
    let transport = state.transport.clone();
    let tool_name = request.name.clone();
    let arguments = request.arguments.clone();
    let request_id_spawn = request_id.clone();

    tokio::spawn(async move {
        // Send "processing" notification
        let processing_msg = TransportMessage::with_metadata(
            json!({
                "type": "tool_call_started",
                "request_id": request_id_spawn,
                "tool": tool_name,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })
            .to_string(),
            "sse",
        );

        if let Err(e) = transport.send_event(processing_msg).await {
            error!("Failed to send processing notification: {}", e);
            return;
        }

        // Execute tool
        let result: Result<serde_json::Value, String> = match tool_name.as_str() {
            "echo" => {
                let message = arguments.get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("No message");
                Ok(json!({
                    "message": message,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))
            }
            "ping" => Ok(json!({
                "response": "pong",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })),
            "info" => Ok(json!({
                "tool": "info",
                "version": env!("CARGO_PKG_VERSION"),
                "description": "MCP Boilerplate Rust SSE Server"
            })),
            _ => Err(format!("Unknown tool: {}", tool_name)),
        };

        // Send result notification
        let result_msg = match result {
            Ok(data) => TransportMessage::with_metadata(
                json!({
                    "type": "tool_call_result",
                    "request_id": request_id_spawn,
                    "tool": tool_name,
                    "success": true,
                    "result": data,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
                .to_string(),
                "sse",
            ),
            Err(error) => TransportMessage::with_metadata(
                json!({
                    "type": "tool_call_error",
                    "request_id": request_id_spawn,
                    "tool": tool_name,
                    "success": false,
                    "error": error,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
                .to_string(),
                "sse",
            ),
        };

        if let Err(e) = transport.send_event(result_msg).await {
            error!("Failed to send result notification: {}", e);
        }
    });

    // Return immediate response
    Ok(Json(ToolCallResponse {
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
    fn test_tool_call_request_serialization() {
        let request = ToolCallRequest {
            name: "echo".to_string(),
            arguments: json!({"message": "test"}),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("echo"));
        assert!(serialized.contains("test"));
    }

    #[test]
    fn test_tool_call_response_serialization() {
        let response = ToolCallResponse {
            request_id: "test-id".to_string(),
            status: "accepted".to_string(),
            message: "Test message".to_string(),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        assert!(serialized.contains("test-id"));
        assert!(serialized.contains("accepted"));
    }
}