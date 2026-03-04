//! HTTP Streaming Server for MCP Protocol - Đấu Trường Vui Backend
//!
//! Simplified server with core endpoints only:
//! - /health - Health check
//! - /rpc - JSON-RPC endpoint (MCP protocol)
//! - /tools - List available tools
//! - /tools/call - Call a tool

use crate::mcp::protocol_handler::ProtocolHandler;
use crate::credits::routes::credit_routes;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

/// HTTP streaming server state
#[derive(Clone)]
pub struct AppState {
    pub protocol_handler: Arc<ProtocolHandler>,
}

/// Start HTTP streaming server
pub async fn run_http_stream_server(bind_address: &str) -> anyhow::Result<()> {
    info!("Starting mcp-dautruongvui-be HTTP server");
    info!("Bind address: {}", bind_address);

    let protocol_handler = Arc::new(ProtocolHandler::new());

    let state = AppState { protocol_handler };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route("/rpc", post(rpc_handler))
        .route("/tools", get(list_tools_handler))
        .route("/tools/call", post(call_tool_handler))
        .nest("/credits", credit_routes().with_state(()))
        .layer(cors)
        .with_state(state);

    info!("HTTP server ready on http://{}", bind_address);
    info!("Endpoints:");
    info!("  GET  /                          - Server info");
    info!("  GET  /health                    - Health check");
    info!("  POST /rpc                       - JSON-RPC endpoint");
    info!("  GET  /tools                     - List tools");
    info!("  POST /tools/call                - Call a tool");
    info!("  POST /credits/wallet            - Get/create credit wallet");
    info!("  POST /credits/deduct            - Deduct credits");
    info!("  POST /credits/claim-welcome-bonus - Claim welcome bonus");
    info!("  POST /credits/claim-daily-bonus   - Claim daily bonus");

    let listener = tokio::net::TcpListener::bind(bind_address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Root handler - server information
async fn root_handler() -> Json<Value> {
    Json(json!({
        "service": "mcp-dautruongvui-be",
        "version": env!("CARGO_PKG_VERSION"),
        "transport": "http-stream",
        "endpoints": {
            "health": "/health",
            "rpc": "/rpc",
            "tools": "/tools",
            "tools_call": "/tools/call"
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Health check handler
async fn health_handler() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "mcp-dautruongvui-be",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// RPC handler - JSON-RPC over HTTP
async fn rpc_handler(
    State(state): State<AppState>,
    Json(request): Json<Value>,
) -> Response {
    let request_str = serde_json::to_string(&request).unwrap_or_default();
    let response_str = state
        .protocol_handler
        .handle_request(&request_str)
        .await
        .unwrap_or_else(|e| {
            json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32603,
                    "message": format!("Internal error: {}", e)
                }
            })
            .to_string()
        });

    let response: Value =
        serde_json::from_str(&response_str).unwrap_or_else(|_| json!({}));

    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/json")],
        Json(response),
    )
        .into_response()
}

/// List tools handler
async fn list_tools_handler(State(state): State<AppState>) -> Json<Value> {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });

    let request_str = serde_json::to_string(&request).unwrap();
    let response_str = state
        .protocol_handler
        .handle_request(&request_str)
        .await
        .unwrap_or_default();
    let response: Value =
        serde_json::from_str(&response_str).unwrap_or_else(|_| json!({}));
    Json(response)
}

/// Call tool handler
async fn call_tool_handler(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Response {
    let tool_name = payload["name"].as_str().unwrap_or("unknown");
    let arguments = &payload["arguments"];

    info!("Tool call: {} with args: {}", tool_name, arguments);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": arguments
        }
    });

    let request_str = serde_json::to_string(&request).unwrap();
    let response_str = state
        .protocol_handler
        .handle_request(&request_str)
        .await
        .unwrap_or_default();
    let response: Value =
        serde_json::from_str(&response_str).unwrap_or_else(|_| json!({}));

    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/json")],
        Json(response),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let protocol_handler = Arc::new(ProtocolHandler::new());
        let _state = AppState { protocol_handler };
    }
}