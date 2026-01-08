//! HTTP Streaming Server for MCP Protocol
//!
//! Provides HTTP-based streaming transport with chunked transfer encoding.
//! Supports large data transfers and progressive response streaming.

use crate::mcp::protocol_handler::ProtocolHandler;
use crate::metrics;
use crate::transport::http_stream::HttpStreamTransport;
use crate::transport::{Transport, TransportMessage};
use axum::{
    body::Body,
    extract::{Json, Path, State},
    http::{header, StatusCode},
    response::Response,
    routing::{get, post},
    Router,
};
use futures::stream::{self, StreamExt};
use serde_json::{json, Value};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

#[cfg(feature = "http-stream")]
use axum::response::IntoResponse;

const CHUNK_SIZE: usize = 8192; // 8KB chunks

/// HTTP streaming server state
#[derive(Clone)]
pub struct HttpStreamServerState {
    pub active_streams: Arc<RwLock<u32>>,
    pub total_requests: Arc<RwLock<u64>>,
    pub protocol_handler: Arc<ProtocolHandler>,
    pub transport: Arc<RwLock<HttpStreamTransport>>,
    pub broadcast_tx: broadcast::Sender<Vec<u8>>,
}

/// Start HTTP streaming server
pub async fn run_http_stream_server(bind_address: &str) -> anyhow::Result<()> {
    info!("Starting MCP HTTP Streaming Server with ProtocolHandler");
    info!("Bind address: {}", bind_address);

    let protocol_handler = Arc::new(ProtocolHandler::new());
    let transport = Arc::new(RwLock::new(HttpStreamTransport::new(
        bind_address.to_string(),
    )));

    // Initialize transport
    {
        let mut t = transport.write().await;
        t.initialize().await?;
    }

    let (broadcast_tx, _) = broadcast::channel::<Vec<u8>>(100);

    let state = HttpStreamServerState {
        active_streams: Arc::new(RwLock::new(0)),
        total_requests: Arc::new(RwLock::new(0)),
        protocol_handler,
        transport,
        broadcast_tx,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route("/stream", get(stream_handler))
        .route("/stream/:id", get(stream_by_id_handler))
        .route("/rpc", post(rpc_handler))
        .route("/tools", get(list_tools_handler))
        .route("/tools/call", post(call_tool_handler))
        .route("/stats", get(stats_handler))
        .route("/metrics", get(metrics_handler))
        .layer(cors)
        .with_state(state);

    info!("HTTP Streaming Server ready on http://{}", bind_address);
    info!("Endpoints:");
    info!("  GET  /           - Server info");
    info!("  GET  /health     - Health check");
    info!("  GET  /stream     - Start streaming connection");
    info!("  GET  /stream/:id - Stream specific resource");
    info!("  POST /rpc        - JSON-RPC endpoint");
    info!("  GET  /tools      - List tools");
    info!("  POST /tools/call - Call a tool");
    info!("  GET  /stats      - Server statistics");
    info!("  GET  /metrics    - Prometheus metrics");

    let listener = tokio::net::TcpListener::bind(bind_address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Root handler - server information
async fn root_handler(State(state): State<HttpStreamServerState>) -> Json<Value> {
    Json(json!({
        "service": "MCP HTTP Streaming Server",
        "version": env!("CARGO_PKG_VERSION"),
        "transport": "http-stream",
        "endpoints": {
            "health": "/health",
            "stream": "/stream",
            "rpc": "/rpc",
            "tools": "/tools",
            "stats": "/stats",
            "metrics": "/metrics"
        },
        "features": {
            "chunked_transfer": true,
            "progressive_streaming": true,
            "large_data_support": true,
            "broadcast": true
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Health check handler
async fn health_handler(State(state): State<HttpStreamServerState>) -> Json<Value> {
    let active_streams = *state.active_streams.read().await;
    let total_requests = *state.total_requests.read().await;
    let transport = state.transport.read().await;
    let is_ready = transport.is_ready();

    Json(json!({
        "status": if is_ready { "healthy" } else { "unhealthy" },
        "active_streams": active_streams,
        "total_requests": total_requests,
        "transport_ready": is_ready,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Metrics endpoint - Prometheus format
#[cfg(feature = "http-stream")]
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

/// Stream handler - creates chunked streaming response
async fn stream_handler(State(state): State<HttpStreamServerState>) -> Response {
    let stream_id = uuid::Uuid::new_v4().to_string();
    info!("New streaming connection: {}", stream_id);

    {
        let mut active = state.active_streams.write().await;
        *active += 1;
    }

    // Record metrics
    metrics::increment_active_connections();
    metrics::record_connection("http-stream");

    // Create a sample data stream (in production, this would stream real MCP data)
    let sample_data = generate_sample_data();
    let chunks: Vec<Vec<u8>> = chunk_data(sample_data, CHUNK_SIZE);

    let stream = stream::iter(chunks).map(|chunk| Ok::<_, Infallible>(chunk));

    // Cleanup when stream ends
    let cleanup_state = state.clone();
    let cleanup_stream_id = stream_id.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        let mut active = cleanup_state.active_streams.write().await;
        if *active > 0 {
            *active -= 1;
        }
        info!("Stream connection closed: {}", cleanup_stream_id);
    });

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::TRANSFER_ENCODING, "chunked")
        .header("X-Stream-Id", stream_id)
        .header("X-Chunk-Size", CHUNK_SIZE.to_string())
        .body(Body::from_stream(stream))
        .unwrap()
}

/// Stream by ID handler - stream specific resource
async fn stream_by_id_handler(
    Path(id): Path<String>,
    State(state): State<HttpStreamServerState>,
) -> Response {
    info!("Streaming resource: {}", id);

    {
        let mut requests = state.total_requests.write().await;
        *requests += 1;
    }

    // In production, fetch data based on ID
    let data = json!({
        "id": id,
        "type": "stream",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "data": "Streamed content for resource"
    });

    let json_bytes = serde_json::to_vec(&data).unwrap();
    let chunks = chunk_data(json_bytes, CHUNK_SIZE);

    let stream = stream::iter(chunks).map(|chunk| Ok::<_, Infallible>(chunk));

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::TRANSFER_ENCODING, "chunked")
        .body(Body::from_stream(stream))
        .unwrap()
}

/// RPC handler - JSON-RPC over HTTP with streaming response
async fn rpc_handler(
    State(state): State<HttpStreamServerState>,
    Json(request): Json<Value>,
) -> Response {
    {
        let mut requests = state.total_requests.write().await;
        *requests += 1;
    }

    info!("RPC request received: {}", request);

    // Process JSON-RPC request through protocol handler
    let request_str = serde_json::to_string(&request).unwrap_or_default();
    let response_str = state
        .protocol_handler
        .handle_request(&request_str)
        .await
        .unwrap_or_else(|e| {
            serde_json::json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32603,
                    "message": format!("Internal error: {}", e)
                }
            })
            .to_string()
        });
    let response: serde_json::Value =
        serde_json::from_str(&response_str).unwrap_or_else(|_| serde_json::json!({}));

    // Convert response to streaming chunks
    let response_bytes = serde_json::to_vec(&response).unwrap();

    // For small responses, return directly
    if response_bytes.len() < CHUNK_SIZE {
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(response_bytes))
            .unwrap();
    }

    // For large responses, stream in chunks
    let chunks = chunk_data(response_bytes, CHUNK_SIZE);
    let stream = stream::iter(chunks).map(|chunk| Ok::<_, Infallible>(chunk));

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::TRANSFER_ENCODING, "chunked")
        .body(Body::from_stream(stream))
        .unwrap()
}

/// List tools handler
async fn list_tools_handler(State(state): State<HttpStreamServerState>) -> Json<Value> {
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
    let response: serde_json::Value =
        serde_json::from_str(&response_str).unwrap_or_else(|_| serde_json::json!({}));
    Json(response)
}

/// Call tool handler
async fn call_tool_handler(
    State(state): State<HttpStreamServerState>,
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
    let response: serde_json::Value =
        serde_json::from_str(&response_str).unwrap_or_else(|_| serde_json::json!({}));
    let response_bytes = serde_json::to_vec(&response).unwrap();

    // Stream large tool responses
    if response_bytes.len() > CHUNK_SIZE {
        let chunks = chunk_data(response_bytes, CHUNK_SIZE);
        let stream = stream::iter(chunks).map(|chunk| Ok::<_, Infallible>(chunk));

        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::TRANSFER_ENCODING, "chunked")
            .body(Body::from_stream(stream))
            .unwrap();
    }

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(response_bytes))
        .unwrap()
}

/// Stats handler
async fn stats_handler(State(state): State<HttpStreamServerState>) -> Json<Value> {
    let active_streams = *state.active_streams.read().await;
    let total_requests = *state.total_requests.read().await;
    let transport = state.transport.read().await;
    let transport_stats = transport.get_stats();

    Json(json!({
        "active_streams": active_streams,
        "total_requests": total_requests,
        "transport": {
            "messages_sent": transport_stats.messages_sent,
            "messages_received": transport_stats.messages_received,
            "bytes_sent": transport_stats.bytes_sent,
            "bytes_received": transport_stats.bytes_received,
            "errors": transport_stats.error_count
        },
        "chunk_size": CHUNK_SIZE,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Helper: Generate sample streaming data
fn generate_sample_data() -> Vec<u8> {
    let data = json!({
        "type": "stream",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "chunks": (0..100).map(|i| {
            format!("Chunk {} - Sample data for HTTP streaming demonstration", i)
        }).collect::<Vec<_>>(),
        "metadata": {
            "total_size": "Large dataset",
            "encoding": "chunked",
            "compression": "none"
        }
    });

    serde_json::to_vec_pretty(&data).unwrap()
}

/// Helper: Split data into chunks
fn chunk_data(data: Vec<u8>, chunk_size: usize) -> Vec<Vec<u8>> {
    data.chunks(chunk_size)
        .map(|chunk| chunk.to_vec())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_data() {
        let data = vec![1u8; 20000];
        let chunks = chunk_data(data, CHUNK_SIZE);

        assert!(chunks.len() > 1);
        assert_eq!(chunks[0].len(), CHUNK_SIZE);
    }

    #[test]
    fn test_generate_sample_data() {
        let data = generate_sample_data();
        assert!(data.len() > 0);

        let parsed: Value = serde_json::from_slice(&data).unwrap();
        assert_eq!(parsed["type"], "stream");
    }

    #[tokio::test]
    async fn test_server_state_creation() {
        let protocol_handler = Arc::new(ProtocolHandler::new());
        let transport = Arc::new(RwLock::new(HttpStreamTransport::new(
            "127.0.0.1:8026".to_string(),
        )));
        let (broadcast_tx, _) = broadcast::channel::<Vec<u8>>(100);

        let state = HttpStreamServerState {
            active_streams: Arc::new(RwLock::new(0)),
            total_requests: Arc::new(RwLock::new(0)),
            protocol_handler,
            transport,
            broadcast_tx,
        };

        assert_eq!(*state.active_streams.read().await, 0);
        assert_eq!(*state.total_requests.read().await, 0);
    }
}
