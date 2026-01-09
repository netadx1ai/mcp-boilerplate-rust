use anyhow::Result;
use clap::{Parser, ValueEnum};

mod mcp;
mod metrics;
mod prompts;
mod resources;
mod tools;
mod transport;
mod types;
mod utils;
mod loadbalancer;

#[cfg(feature = "auth")]
mod middleware;

#[cfg(feature = "auth")]
use middleware::{auth_middleware, auth_router, oauth_router, wellknown_router, OAuthState};

use mcp::McpServer;
#[cfg(feature = "http")]
use mcp::protocol_handler::ProtocolHandler;
use utils::Logger;

#[cfg(feature = "sse")]
use mcp::run_sse_server;

#[cfg(feature = "websocket")]
use mcp::create_websocket_router;

#[cfg(feature = "http-stream")]
use mcp::run_http_stream_server;

#[cfg(feature = "grpc")]
use mcp::run_grpc_server;

#[cfg(any(
    feature = "http",
    feature = "sse",
    feature = "websocket",
    feature = "http-stream",
    feature = "grpc"
))]
use tracing::info;

#[cfg(feature = "http")]
use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
};
#[cfg(feature = "http")]
use serde_json::json;
#[cfg(feature = "http")]
use std::sync::Arc;

#[derive(Debug, Clone, ValueEnum)]
enum ServerMode {
    Stdio,
    #[cfg(feature = "http")]
    Http,
    #[cfg(feature = "sse")]
    Sse,
    #[cfg(feature = "websocket")]
    Websocket,
    #[cfg(feature = "http-stream")]
    HttpStream,
    #[cfg(feature = "grpc")]
    Grpc,
}

#[derive(Parser, Debug)]
#[command(name = "mcp-boilerplate-rust")]
#[command(version, about = "MCP v5 Rust boilerplate using official rust-sdk")]
struct Args {
    #[arg(short, long, value_enum, default_value = "stdio")]
    mode: ServerMode,

    #[arg(short, long, help = "Enable verbose logging")]
    verbose: bool,

    #[arg(
        short,
        long,
        help = "Bind address for SSE server",
        default_value = "127.0.0.1:8025"
    )]
    bind: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let args = Args::parse();

    // Initialize transport registry
    transport::init_registry();

    // For stdio mode, disable logging to avoid interfering with JSON-RPC
    // Claude Desktop can't parse logs mixed with JSON responses
    let result = match args.mode {
        ServerMode::Stdio => {
            if args.verbose {
                std::env::set_var("RUST_LOG", "error");
            } else {
                std::env::set_var("RUST_LOG", "off");
            }
            Logger::init();
            run_stdio_server().await
        }
        #[cfg(feature = "http")]
        ServerMode::Http => {
            if args.verbose {
                std::env::set_var("RUST_LOG", "debug,mcp_boilerplate_rust=trace");
            }
            Logger::init();
            info!("MCP Boilerplate Rust v{}", env!("CARGO_PKG_VERSION"));
            info!("Using official rmcp SDK v0.12");
            info!("Starting MCP server in HTTP mode");
            run_http_server().await
        }
        #[cfg(feature = "sse")]
        ServerMode::Sse => {
            if args.verbose {
                std::env::set_var("RUST_LOG", "debug,mcp_boilerplate_rust=trace");
            } else {
                std::env::set_var("RUST_LOG", "info");
            }
            Logger::init();
            info!("MCP Boilerplate Rust v{}", env!("CARGO_PKG_VERSION"));
            info!("Using official rmcp SDK v0.12");
            info!("Starting MCP server in SSE mode");
            run_sse_server(&args.bind).await
        }
        #[cfg(feature = "websocket")]
        ServerMode::Websocket => {
            if args.verbose {
                std::env::set_var("RUST_LOG", "debug,mcp_boilerplate_rust=trace");
            } else {
                std::env::set_var("RUST_LOG", "info");
            }
            Logger::init();
            info!("MCP Boilerplate Rust v{}", env!("CARGO_PKG_VERSION"));
            info!("Using official rmcp SDK v0.12");
            info!("Starting MCP server in WebSocket mode");
            run_websocket_server(&args.bind).await
        }
        #[cfg(feature = "http-stream")]
        ServerMode::HttpStream => {
            if args.verbose {
                std::env::set_var("RUST_LOG", "debug,mcp_boilerplate_rust=trace");
            } else {
                std::env::set_var("RUST_LOG", "info");
            }
            Logger::init();
            info!("MCP Boilerplate Rust v{}", env!("CARGO_PKG_VERSION"));
            info!("Using official rmcp SDK v0.12");
            info!("Starting MCP server in HTTP Streaming mode");
            run_http_stream_server(&args.bind).await
        }
        #[cfg(feature = "grpc")]
        ServerMode::Grpc => {
            if args.verbose {
                std::env::set_var("RUST_LOG", "debug,mcp_boilerplate_rust=trace");
            } else {
                std::env::set_var("RUST_LOG", "info");
            }
            Logger::init();
            info!("MCP Boilerplate Rust v{}", env!("CARGO_PKG_VERSION"));
            info!("Using official rmcp SDK v0.12");
            info!("Starting MCP server in gRPC mode");
            run_grpc_server(&args.bind).await
        }
    };

    Logger::shutdown();
    result
}

async fn run_stdio_server() -> Result<()> {
    let server = McpServer::new();
    server.run().await?;
    Ok(())
}

#[cfg(feature = "websocket")]
async fn run_websocket_server(bind_address: &str) -> Result<()> {
    info!("Starting WebSocket server on {}", bind_address);

    let app = create_websocket_router();

    let listener = tokio::net::TcpListener::bind(bind_address).await?;
    info!("WebSocket server listening on {}", bind_address);
    info!("Connect to ws://{}/ws", bind_address);

    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(feature = "http")]
async fn run_http_server() -> Result<()> {
    use axum::{
        routing::{get, post},
        Router,
    };
    use tower_http::cors::{Any, CorsLayer};

    use utils::Config;

    let config = Config::from_env();
    config.validate()?;

    let protocol_handler = Arc::new(ProtocolHandler::new());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build base router with protocol_handler state
    let mut app: Router<()> = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/tools", get(list_tools_handler))
        .route("/tools/call", post(call_tool_handler))
        .route("/rpc", post(rpc_handler))
        .with_state(protocol_handler.clone());

    // Add auth routes if auth feature enabled
    #[cfg(feature = "auth")]
    {
        // Create OAuth state with defaults
        let oauth_state = OAuthState::with_defaults();
        
        // Simple JWT auth routes (stateless)
        app = app.nest("/auth", auth_router());
        
        // Add protected endpoint example
        app = app.route(
            "/protected/tools",
            get(list_tools_handler)
                .layer(axum::middleware::from_fn(auth_middleware))
                .with_state(protocol_handler),
        );
        
        // OAuth 2.1 routes (MCP 2025-11-25) - separate state
        app = app.nest("/oauth", oauth_router(oauth_state.clone()));
        
        // Well-known metadata endpoints (RFC 8414, RFC 9728, OIDC)
        app = app.nest("/.well-known", wellknown_router(oauth_state));
    }

    let app = app.layer(cors);

    let addr = format!("{}:{}", config.host, config.port);
    info!("MCP HTTP Server starting on {}", addr);
    info!("Protocol: MCP 2025-03-26 (HTTP wrapper)");
    info!("Tools: 11 available");
    info!("Endpoints:");
    info!("  GET  /health");
    info!("  GET  /tools");
    info!("  POST /tools/call");
    info!("  POST /rpc (JSON-RPC)");
    #[cfg(feature = "auth")]
    {
        info!("Auth Endpoints (auth feature enabled):");
        info!("  POST /auth/login");
        info!("  GET  /auth/verify");
        info!("  GET  /auth/me (protected)");
        info!("  GET  /protected/tools (protected)");
        info!("OAuth 2.1 Endpoints (MCP 2025-11-25):");
        info!("  GET  /oauth/authorize");
        info!("  POST /oauth/token");
        info!("  POST /oauth/register");
        info!("  POST /oauth/introspect");
        info!("  POST /oauth/revoke");
        info!("Well-Known Metadata (RFC 8414, RFC 9728, OIDC):");
        info!("  GET  /.well-known/oauth-authorization-server");
        info!("  GET  /.well-known/openid-configuration");
        info!("  GET  /.well-known/oauth-protected-resource");
    }

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(feature = "http")]
async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "service": "mcp-boilerplate-rust",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": "MCP v5",
        "mode": "http",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[cfg(feature = "http")]
async fn list_tools_handler(
    axum::extract::State(handler): axum::extract::State<Arc<ProtocolHandler>>,
) -> impl IntoResponse {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });
    
    match handler.handle_request(&request.to_string()).await {
        Ok(response) => {
            let parsed: serde_json::Value = serde_json::from_str(&response).unwrap_or(json!({"error": "parse error"}));
            if let Some(result) = parsed.get("result") {
                (StatusCode::OK, Json(result.clone())).into_response()
            } else {
                (StatusCode::OK, Json(parsed)).into_response()
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ).into_response(),
    }
}

#[cfg(feature = "http")]
async fn call_tool_handler(
    axum::extract::State(handler): axum::extract::State<Arc<ProtocolHandler>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Extract tool name and arguments from payload
    let tool_name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let arguments = payload.get("arguments").cloned().unwrap_or(json!({}));
    
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": arguments
        }
    });
    
    match handler.handle_request(&request.to_string()).await {
        Ok(response) => {
            let parsed: serde_json::Value = serde_json::from_str(&response).unwrap_or(json!({"error": "parse error"}));
            if let Some(result) = parsed.get("result") {
                (StatusCode::OK, Json(result.clone())).into_response()
            } else if let Some(error) = parsed.get("error") {
                (StatusCode::BAD_REQUEST, Json(error.clone())).into_response()
            } else {
                (StatusCode::OK, Json(parsed)).into_response()
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ).into_response(),
    }
}

#[cfg(feature = "http")]
async fn rpc_handler(
    axum::extract::State(handler): axum::extract::State<Arc<ProtocolHandler>>,
    body: String,
) -> impl IntoResponse {
    match handler.handle_request(&body).await {
        Ok(response) => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            response,
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32603,
                    "message": e.to_string()
                }
            })),
        ).into_response(),
    }
}
