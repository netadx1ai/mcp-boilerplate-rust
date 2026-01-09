# MCP Boilerplate Rust

**Version 0.5.0** | Production-Ready Multi-Transport MCP Server

A production-ready Rust implementation of the Model Context Protocol (MCP) featuring 6 transport modes, comprehensive observability, and enterprise-grade tooling.

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-89%20passing-brightgreen.svg)]()

## Features

- **6 Transport Modes** - Stdio, SSE, WebSocket, HTTP, HTTP Streaming, gRPC (w/ gRPC-Web)
- **11 Production Tools** - Complete suite with progress, batching, and long-running tasks
- **Client SDK Generators** - Auto-generate TypeScript, Python, and Go client libraries
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

## Client SDK Generators

Auto-generate type-safe client libraries in multiple languages:

```bash
cd sdk-generators
cargo run --release
```

**Generated SDKs:**
- **TypeScript** (209 lines) - Full type safety, zero dependencies, Browser + Node.js
- **Python** (111 lines) - Type hints, dataclasses, requests only
- **Go** (172 lines) - Idiomatic interfaces, stdlib only

**Example Usage:**

```typescript
// TypeScript
import { McpClient } from './mcp-client';
const client = new McpClient({ transport: 'http', port: 8080 });
const result = await client.echo({ message: 'Hello' });
```

```python
# Python
from mcp_client import McpClient, McpClientConfig
config = McpClientConfig(transport='http', port=8080)
client = McpClient(config)
result = client.echo(message='Hello from Python')
```

```go
// Go
import "your-project/mcpclient"
config := mcpclient.Config{Transport: "http", Port: 8080}
client := mcpclient.NewClient(config)
result, _ := client.Echo(map[string]interface{}{"message": "Hello"})
```

**Features:**
- All 11 tools supported with full type definitions
- All 6 transports supported (SSE, WebSocket, HTTP, HTTP Stream, gRPC)
- Complete documentation and working examples
- Generation time: <500ms

See [docs/SDK_GENERATORS.md](docs/SDK_GENERATORS.md) for complete documentation.

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

### Getting Started
- [START_HERE.md](START_HERE.md) - 5-minute quick start
- [docs/guides/QUICK_START.md](docs/guides/QUICK_START.md) - Detailed quick start
- [docs/guides/INSTALLATION.md](docs/guides/INSTALLATION.md) - Installation guide

### Features
- [docs/SDK_GENERATORS.md](docs/SDK_GENERATORS.md) - Client SDK generators
- [docs/TRANSPORT_QUICK_GUIDE.md](docs/TRANSPORT_QUICK_GUIDE.md) - Transport guide
- [docs/guides/METRICS_GUIDE.md](docs/guides/METRICS_GUIDE.md) - Metrics and monitoring

### Integration
- [docs/guides/integration/CLAUDE_DESKTOP_SETUP.md](docs/guides/integration/CLAUDE_DESKTOP_SETUP.md) - Claude Desktop
- [docs/guides/integration/INTEGRATION_GUIDE.md](docs/guides/integration/INTEGRATION_GUIDE.md) - Integration guide

### Reference
- [CLAUDE.md](CLAUDE.md) - Developer guide for AI assistants
- [docs/reference/API.md](docs/reference/API.md) - API reference
- [docs/reference/QUICK_REFERENCE.md](docs/reference/QUICK_REFERENCE.md) - Command reference
- [docs/README.md](docs/README.md) - Complete documentation index

## License

MIT License - see [LICENSE](LICENSE) file.

---
**Maintained by:** NetAdx AI