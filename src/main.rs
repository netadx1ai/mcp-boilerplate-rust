use anyhow::Result;
use clap::{Parser, ValueEnum};

mod mcp;
mod prompts;
mod resources;
mod tools;
mod types;
mod utils;

#[cfg(feature = "http")]
mod middleware;

use mcp::McpServer;
use utils::Logger;

#[cfg(feature = "http")]
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
#[cfg(feature = "http")]
use tools::{echo::EchoTool, shared::*};

#[derive(Debug, Clone, ValueEnum)]
enum ServerMode {
    Stdio,
    #[cfg(feature = "http")]
    Http,
}

#[derive(Parser, Debug)]
#[command(name = "mcp-boilerplate-rust")]
#[command(version, about = "MCP v5 Rust boilerplate using official rust-sdk")]
struct Args {
    #[arg(short, long, value_enum, default_value = "stdio")]
    mode: ServerMode,

    #[arg(short, long, help = "Enable verbose logging")]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let args = Args::parse();

    // For stdio mode, disable logging to avoid interfering with JSON-RPC
    // Claude Desktop can't parse logs mixed with JSON responses
    match args.mode {
        ServerMode::Stdio => {
            if args.verbose {
                std::env::set_var("RUST_LOG", "error");
            } else {
                std::env::set_var("RUST_LOG", "off");
            }
            Logger::init();
            run_stdio_server().await?;
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
            run_http_server().await?;
        }
    }

    Ok(())
}

async fn run_stdio_server() -> Result<()> {
    let server = McpServer::new();
    server.run().await?;
    Ok(())
}

#[cfg(feature = "http")]
async fn run_http_server() -> Result<()> {
    use axum::{
        routing::{get, post},
        Router,
    };
    use tower_http::cors::{Any, CorsLayer};

    use types::AppState;
    use utils::Config;

    let config = Config::from_env();
    config.validate()?;

    let state = Arc::new(AppState::new());
    let echo_tool = Arc::new(EchoTool::new());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/tools", get(list_tools))
        .route(
            "/tools/echo",
            post({
                let tool = Arc::clone(&echo_tool);
                move |payload| handle_echo_tool(tool, payload)
            }),
        )
        .route(
            "/tools/ping",
            post({
                let tool = Arc::clone(&echo_tool);
                move |payload| handle_ping_tool(tool, payload)
            }),
        )
        .route(
            "/tools/info",
            post({
                let tool = Arc::clone(&echo_tool);
                move |payload| handle_info_tool(tool, payload)
            }),
        )
        .layer(cors)
        .with_state(state);

    let addr = format!("{}:{}", config.host, config.port);
    info!("MCP HTTP Server starting on {}", addr);
    info!("Protocol: MCP v5 (HTTP wrapper)");
    info!("Endpoints:");
    info!("  GET  /health");
    info!("  GET  /tools");
    info!("  POST /tools/echo");
    info!("  POST /tools/ping");
    info!("  POST /tools/info");

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
async fn list_tools() -> impl IntoResponse {
    Json(json!({
        "tools": [
            {
                "name": "echo",
                "description": "Echo back a message",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "Message to echo back"
                        }
                    },
                    "required": ["message"]
                },
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
                "parameters": {
                    "type": "object",
                    "properties": {}
                },
                "input_schema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "info",
                "description": "Get tool information",
                "parameters": {
                    "type": "object",
                    "properties": {}
                },
                "input_schema": {
                    "type": "object",
                    "properties": {}
                }
            }
        ]
    }))
}

#[cfg(feature = "http")]
async fn handle_echo_tool(
    tool: Arc<EchoTool>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    match serde_json::from_value::<EchoRequest>(payload) {
        Ok(req) => {
            let params = rmcp::handler::server::wrapper::Parameters(req);
            match tool.echo(params).await {
                Ok(result) => (
                    StatusCode::OK,
                    Json(json!({
                        "content": [{
                            "type": "text",
                            "text": serde_json::to_string_pretty(&result.0).unwrap()
                        }],
                        "is_error": false,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    })),
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "content": [{
                            "type": "text",
                            "text": e.message
                        }],
                        "is_error": true,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    })),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Invalid request: {}", e)
                }],
                "is_error": true,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })),
        )
            .into_response(),
    }
}

#[cfg(feature = "http")]
async fn handle_ping_tool(
    tool: Arc<EchoTool>,
    Json(_payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    match tool.ping().await {
        Ok(result) => (
            StatusCode::OK,
            Json(json!({
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&result.0).unwrap()
                }],
                "is_error": false,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "content": [{
                    "type": "text",
                    "text": e.message
                }],
                "is_error": true,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })),
        )
            .into_response(),
    }
}

#[cfg(feature = "http")]
async fn handle_info_tool(
    tool: Arc<EchoTool>,
    Json(_payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    match tool.info().await {
        Ok(result) => (
            StatusCode::OK,
            Json(json!({
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&result.0).unwrap()
                }],
                "is_error": false,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "content": [{
                    "type": "text",
                    "text": e.message
                }],
                "is_error": true,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })),
        )
            .into_response(),
    }
}
