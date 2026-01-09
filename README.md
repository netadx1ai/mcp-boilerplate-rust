# MCP Boilerplate Rust

**Version 0.5.1** | Production-Ready Multi-Transport MCP Server

A production-ready Rust implementation of the Model Context Protocol (MCP) featuring 6 transport modes, comprehensive observability, and enterprise-grade tooling.

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-48%20passing-brightgreen.svg)]()

## Features

- **6 Transport Modes** - Stdio, SSE, WebSocket, HTTP, HTTP Streaming, gRPC (w/ gRPC-Web)
- **11 Production Tools** - Complete suite with progress, batching, and long-running tasks
- **4 Auto-Generated SDKs** - TypeScript, Python, Go, and Rust (Race Car Edition 🏎️)
- **Load Balancing** - Enterprise-grade with 5 strategies, health checks, auto-failover
- **Observability** - OpenTelemetry Tracing + Prometheus Metrics
- **Type-Safe** - Full Rust type safety with schemars validation
- **High Performance** - Optimized binaries (2.4MB - 4.2MB)
- **Production Ready** - Zero errors, extensive error handling, Docker support

## Quick Start

### Prerequisites

```bash
# Rust 1.75+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install & Run

```bash
# Clone
git clone https://github.com/netadx/mcp-boilerplate-rust
cd mcp-boilerplate-rust

# Build (Minimal / Stdio)
cargo build --release

# Run
./target/release/mcp-boilerplate-rust --mode stdio
```

### Test with MCP Inspector

```bash
npx @modelcontextprotocol/inspector ./target/release/mcp-boilerplate-rust --mode stdio
```

## Transport Modes

### 1. Stdio (Default)
**Best for:** Desktop apps, Claude Desktop, CLI tools
```bash
cargo run --release -- --mode stdio
```

### 2. SSE (Server-Sent Events)
**Best for:** Browser push notifications, live updates
```bash
cargo run --release --features sse -- --mode sse --bind 127.0.0.1:8025
```

### 3. WebSocket
**Best for:** Real-time bidirectional communication
```bash
cargo run --release --features websocket -- --mode websocket --bind 127.0.0.1:9001
```

### 4. HTTP Streaming
**Best for:** Large file transfers
```bash
cargo run --release --features http-stream -- --mode http-stream --bind 127.0.0.1:8026
```

### 5. gRPC & gRPC-Web
**Best for:** Microservices, high-performance APIs, browser clients
```bash
cargo run --release --features grpc -- --mode grpc --bind 127.0.0.1:50051
```
*Supports HTTP/2 multiplexing, Protocol Buffers, and gRPC-Web.*

### 6. HTTP (REST API)
**Best for:** Standard REST APIs
```bash
cargo run --release --features http -- --mode http
```

## Tools

| Tool | Description |
|------|-------------|
| `ping` | Health check |
| `echo` | Message validation |
| `info` | Server metadata |
| `calculate` | Math operations |
| `evaluate` | Expression evaluation |
| `process_with_progress` | Data processing with progress bars |
| `batch_process` | Batch operations with logging |
| `transform_data` | Array transformations |
| `simulate_upload` | File upload simulation |
| `health_check` | System health status |
| `long_task` | Long operation simulation |

## Client SDKs

Auto-generate type-safe client libraries in 4 languages:

```bash
cd sdk-generators
cargo run --release

# Generates:
# - TypeScript: output/typescript/mcp-client.ts
# - Python: output/python/mcp_client.py
# - Go: output/go/mcpclient/client.go
# - Rust: output/rust/mcp_client.rs (Race Car Edition 🏎️)
```

### Rust SDK (Race Car Edition 🏎️)

High-performance generated Rust client with:
- Custom error types (not `Box<dyn Error>`)
- Borrowing optimizations (`&str` vs `String`)
- Zero-cost abstractions
- Pattern matching on enums
- Auto-generated, stays in sync

```rust
use mcp_client::{McpClient, HttpTransport, Result};

let transport = HttpTransport::new("http://127.0.0.1:8080");
let mut client = McpClient::new(transport);
client.connect().await?;

let result = client.echo("Hello, MCP!").await?;
```

📖 [SDK Documentation](docs/features/SDK_GENERATORS.md) | [Rust SDK Guide](docs/features/RUST_SDK.md)

## Load Balancing

Enterprise-grade load balancing with:
- **5 Strategies**: Round-robin, least connections, random, weighted, IP hash
- **Health Checks**: Automatic backend monitoring
- **Auto Failover**: Seamless failover to healthy backends
- **Real-time Stats**: Request counts, success rates, response times

```rust
use mcp_boilerplate_rust::loadbalancer::{LoadBalancer, LoadBalancerConfig, Backend, Strategy};

let config = LoadBalancerConfig::new(Strategy::RoundRobin)
    .add_backend(Backend::new("b1".to_string(), "127.0.0.1:8081".to_string()))
    .add_backend(Backend::new("b2".to_string(), "127.0.0.1:8082".to_string()))
    .with_failover(true);

let lb = LoadBalancer::new(config);
lb.start_health_checks().await;
```

📖 [Load Balancing Guide](docs/features/LOAD_BALANCING.md)

## Observability

### OpenTelemetry Tracing

Enable distributed tracing to track requests across services.

```bash
# Build with otel feature
cargo build --release --features otel

# Run with OTLP exporter
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
export OTEL_SERVICE_NAME="mcp-server"
./target/release/mcp-boilerplate-rust --mode stdio
```

### Prometheus Metrics

Metrics endpoint available at `/metrics` (HTTP/SSE modes).

```bash
cargo build --release --features metrics
```

## Build Options

| Feature | Command | Size |
|---------|---------|------|
| Minimal (Stdio) | `cargo build --release` | ~2.4 MB |
| Web (SSE/WS) | `cargo build --release --features "sse,websocket"` | ~3.3 MB |
| gRPC | `cargo build --release --features grpc` | ~3.9 MB |
| Full | `cargo build --release --features full` | ~4.2 MB |

## Testing

```bash
# Run all tests
cargo test --features full

# Integration tests
./scripts/integration_test.sh
```

## Docker

```bash
docker build -t mcp-server .
docker run -p 8025:8025 mcp-server
```

## Documentation

- [START_HERE.md](START_HERE.md) - 5-minute quick start
- [PROJECT_STATUS.md](PROJECT_STATUS.md) - Complete project status
- [docs/README.md](docs/README.md) - Full documentation index
- [docs/features/](docs/features/) - Feature guides (SDKs, Load Balancing)
- [docs/guides/](docs/guides/) - How-to guides
- [CLAUDE.md](CLAUDE.md) - AI assistant guide

## Project Statistics

- **Transport Modes:** 6
- **Production Tools:** 11
- **Client SDKs:** 4 (auto-generated)
- **Code:** ~16,500 lines
- **Documentation:** ~12,000 lines
- **Tests:** 89+ passing (100%)
- **Binary Size:** 2.4MB - 4.2MB

## License

MIT License - see [LICENSE](LICENSE) file.

## Support

- **GitHub:** https://github.com/netadx/mcp-boilerplate-rust
- **Email:** hello@netadx.ai
- **Website:** https://netadx.ai

---

**Version:** 0.5.0  
**Status:** Production Ready  
**Maintained by:** NetADX Team