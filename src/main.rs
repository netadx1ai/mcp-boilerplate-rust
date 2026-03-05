use anyhow::Result;
use clap::{Parser, ValueEnum};

mod mcp;
mod tools;
mod transport;
mod types;
mod utils;
mod metrics;

#[cfg(feature = "auth")]
mod auth;

#[cfg(feature = "http-stream")]
mod credits;

#[cfg(feature = "http-stream")]
mod upload;

use mcp::McpServer;
use utils::Logger;

#[cfg(feature = "http-stream")]
use mcp::run_http_stream_server;

#[cfg(feature = "http-stream")]
use tracing::info;

#[derive(Debug, Clone, ValueEnum)]
enum ServerMode {
    Stdio,
    #[cfg(feature = "http-stream")]
    HttpStream,
}

#[derive(Parser, Debug)]
#[command(name = "mcp-dautruongvui-be")]
#[command(version, about = "MCP backend for Đấu Trường Vui")]
struct Args {
    #[arg(short, long, value_enum, default_value = "stdio")]
    mode: ServerMode,

    #[arg(short, long, help = "Enable verbose logging")]
    verbose: bool,

    #[arg(
        short,
        long,
        help = "Bind address for HTTP server",
        default_value = "127.0.0.1:8030"
    )]
    bind: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let args = Args::parse();

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
        #[cfg(feature = "http-stream")]
        ServerMode::HttpStream => {
            if args.verbose {
                std::env::set_var("RUST_LOG", "debug,mcp_dautruongvui_be=trace");
            } else if std::env::var("RUST_LOG").is_err() {
                std::env::set_var("RUST_LOG", "info");
            }
            Logger::init();
            info!("mcp-dautruongvui-be v{}", env!("CARGO_PKG_VERSION"));
            info!("Starting MCP server in HTTP Streaming mode");
            run_http_stream_server(&args.bind).await
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