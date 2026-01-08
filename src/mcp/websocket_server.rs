//! WebSocket MCP Server Implementation
//! 
//! Provides bidirectional MCP protocol over WebSocket transport.
//! Supports real-time tool execution with concurrent connections.
//! 
//! # Architecture
//! 
//! Uses the shared ProtocolHandler for consistent rmcp-based protocol handling:
//! 
//! ```text
//! WebSocket Client
//!     ↓
//! WebSocket Transport Layer
//!     ↓
//! ProtocolHandler (rmcp types)
//!     ↓
//! Tool Implementations
//! ```
//! 
//! # Features
//! 
//! - Bidirectional real-time communication
//! - Multi-client support
//! - Type-safe protocol handling with rmcp
//! - Automatic connection management
//! - Statistics tracking
//! 
//! # Example Client
//! 
//! ```javascript
//! const ws = new WebSocket('ws://localhost:9001/ws');
//! 
//! ws.onopen = () => {
//!     ws.send(JSON.stringify({
//!         jsonrpc: '2.0',
//!         id: 1,
//!         method: 'initialize',
//!         params: {}
//!     }));
//! };
//! 
//! ws.onmessage = (event) => {
//!     const response = JSON.parse(event.data);
//!     console.log('Received:', response);
//! };
//! ```

use crate::mcp::protocol_handler::ProtocolHandler;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use futures::StreamExt;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

/// WebSocket server state
#[derive(Clone)]
pub struct WsServerState {
    pub active_connections: Arc<RwLock<u32>>,
    pub total_requests: Arc<RwLock<u64>>,
    pub protocol_handler: Arc<ProtocolHandler>,
}

impl WsServerState {
    /// Create new WebSocket server state
    pub fn new() -> Self {
        Self {
            active_connections: Arc::new(RwLock::new(0)),
            total_requests: Arc::new(RwLock::new(0)),
            protocol_handler: Arc::new(ProtocolHandler::new()),
        }
    }
}

impl Default for WsServerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Create WebSocket MCP server router
pub fn create_router() -> Router {
    let state = WsServerState::new();

    Router::new()
        .route("/", get(root_handler))
        .route("/ws", get(websocket_handler))
        .route("/health", get(health_handler))
        .route("/stats", get(stats_handler))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
}

/// Root endpoint handler
async fn root_handler() -> Json<Value> {
    Json(json!({
        "name": "MCP WebSocket Server",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": "mcp-2024-11-05",
        "transport": "websocket",
        "endpoints": {
            "websocket": "/ws",
            "health": "/health",
            "stats": "/stats"
        },
        "features": [
            "bidirectional",
            "real-time",
            "multi-connection",
            "server-push",
            "rmcp-types"
        ],
        "note": "Now using ProtocolHandler with rmcp types for type safety"
    }))
}

/// Health check handler
async fn health_handler() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "transport": "websocket",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Statistics handler
async fn stats_handler(State(state): State<WsServerState>) -> Json<Value> {
    let active = *state.active_connections.read().await;
    let total = *state.total_requests.read().await;

    Json(json!({
        "active_connections": active,
        "total_requests": total,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// WebSocket upgrade handler
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<WsServerState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

/// Handle WebSocket connection
async fn handle_websocket(mut socket: WebSocket, state: WsServerState) {
    let connection_id = uuid::Uuid::new_v4().to_string();
    
    info!("New WebSocket connection: {}", connection_id);

    // Increment active connections
    {
        let mut active = state.active_connections.write().await;
        *active += 1;
    }

    // Send welcome message
    let welcome = json!({
        "type": "connection",
        "connection_id": connection_id,
        "message": "Connected to MCP WebSocket server",
        "protocol": "mcp-2024-11-05",
        "handler": "ProtocolHandler",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    if let Ok(welcome_str) = serde_json::to_string(&welcome) {
        let _ = socket.send(Message::Text(welcome_str)).await;
    }

    // Handle incoming messages
    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(text) = msg {
            // Increment request counter
            {
                let mut total = state.total_requests.write().await;
                *total += 1;
            }

            // Validate JSON
            if let Err(_) = serde_json::from_str::<Value>(&text) {
                let error_response = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": {
                        "code": -32700,
                        "message": "Parse error: Invalid JSON"
                    }
                });

                if let Ok(response_text) = serde_json::to_string(&error_response) {
                    if socket.send(Message::Text(response_text)).await.is_err() {
                        break;
                    }
                }
                continue;
            }

            // Process request via ProtocolHandler
            let protocol_handler = state.protocol_handler.clone();
            
            match protocol_handler.handle_request(&text).await {
                Ok(response_str) => {
                    if socket.send(Message::Text(response_str)).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    error!("Error handling request: {}", e);
                    
                    let error_response = json!({
                        "jsonrpc": "2.0",
                        "id": null,
                        "error": {
                            "code": -32603,
                            "message": format!("Internal error: {}", e)
                        }
                    });

                    if let Ok(response_text) = serde_json::to_string(&error_response) {
                        if socket.send(Message::Text(response_text)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        } else if let Message::Close(_) = msg {
            info!("WebSocket connection closed: {}", connection_id);
            break;
        } else if let Message::Ping(data) = msg {
            // Respond to ping with pong
            if socket.send(Message::Pong(data)).await.is_err() {
                break;
            }
        }
    }

    // Decrement active connections
    {
        let mut active = state.active_connections.write().await;
        if *active > 0 {
            *active -= 1;
        }
    }

    info!("WebSocket connection terminated: {}", connection_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_state_creation() {
        let state = WsServerState::new();
        assert_eq!(*state.active_connections.read().await, 0);
        assert_eq!(*state.total_requests.read().await, 0);
        
        // Test that protocol handler is functional by testing initialize
        let init_request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let response = state.protocol_handler.handle_request(init_request).await.unwrap();
        assert!(response.contains("protocolVersion") || response.contains("protocol_version"));
    }

    #[tokio::test]
    async fn test_protocol_handler_integration() {
        let state = WsServerState::new();
        
        // Test initialize
        let init_request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let response = state.protocol_handler.handle_request(init_request).await.unwrap();
        assert!(response.contains("protocolVersion") || response.contains("protocol_version"));
        assert!(response.contains("capabilities"));

        // Test tools/list
        let list_request = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
        let response = state.protocol_handler.handle_request(list_request).await.unwrap();
        assert!(response.contains("tools"));
        assert!(response.contains("echo"));

        // Test echo tool
        let echo_request = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"echo","arguments":{"message":"test"}}}"#;
        let response = state.protocol_handler.handle_request(echo_request).await.unwrap();
        assert!(response.contains("result"));
    }

    #[tokio::test]
    async fn test_invalid_json_handling() {
        let state = WsServerState::new();
        let result = state.protocol_handler.handle_request("invalid json").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unknown_method() {
        let state = WsServerState::new();
        let request = r#"{"jsonrpc":"2.0","id":1,"method":"unknown_method","params":{}}"#;
        let response = state.protocol_handler.handle_request(request).await.unwrap();
        assert!(response.contains("error"));
        assert!(response.contains("Method not found"));
    }

    #[tokio::test]
    async fn test_ping_tool() {
        let state = WsServerState::new();
        let request = r#"{"jsonrpc":"2.0","id":1,"method":"ping","params":{}}"#;
        let response = state.protocol_handler.handle_request(request).await.unwrap();
        assert!(response.contains("pong"));
    }

    #[tokio::test]
    async fn test_calculate_tool() {
        let state = WsServerState::new();
        let request = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"calculate","arguments":{"operation":"add","a":5,"b":3}}}"#;
        let response = state.protocol_handler.handle_request(request).await.unwrap();
        assert!(response.contains("result"));
        assert!(response.contains("8"));
    }

    #[tokio::test]
    async fn test_evaluate_tool() {
        let state = WsServerState::new();
        let request = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"evaluate","arguments":{"expression":"2+3*4"}}}"#;
        let response = state.protocol_handler.handle_request(request).await.unwrap();
        assert!(response.contains("result"));
        assert!(response.contains("14"));
    }

    #[test]
    fn test_router_creation() {
        let _router = create_router();
    }

    #[tokio::test]
    async fn test_connection_counter() {
        let state = WsServerState::new();
        
        // Simulate connection
        {
            let mut active = state.active_connections.write().await;
            *active += 1;
        }
        
        assert_eq!(*state.active_connections.read().await, 1);
        
        // Simulate disconnect
        {
            let mut active = state.active_connections.write().await;
            *active -= 1;
        }
        
        assert_eq!(*state.active_connections.read().await, 0);
    }

    #[tokio::test]
    async fn test_request_counter() {
        let state = WsServerState::new();
        
        // Simulate requests
        for _ in 0..5 {
            let mut total = state.total_requests.write().await;
            *total += 1;
        }
        
        assert_eq!(*state.total_requests.read().await, 5);
    }
}