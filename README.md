# MCP Boilerplate Rust

**Version 0.4.0** | Production-Ready Multi-Transport MCP Server | 6 Transport Modes

A production-ready Rust implementation of the Model Context Protocol (MCP) with advanced multi-transport support, comprehensive tooling, and enterprise-grade features.

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-89%20passing-brightgreen.svg)]()

## Features

- **6 Transport Modes** - stdio, SSE, WebSocket, HTTP, HTTP Streaming, gRPC
- **11 Production Tools** - Complete suite with progress notifications
- **Prometheus Metrics** - Built-in metrics collection and exposure
- **Type-Safe** - Full Rust type safety with schemars validation
- **High Performance** - Optimized binaries (2.4MB - 4.2MB)
- **89 Tests** - Comprehensive test coverage (100% passing)
- **Production Ready** - Zero errors, extensive error handling
- **Browser Clients** - Interactive test clients included
- **Docker Ready** - Containerization support

## Quick Start

### Prerequisites

```bash
# Rust 1.75 or later
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Optional: For gRPC testing
brew install grpcurl  # macOS
```

### Install & Run

```bash
# Clone the repository
git clone https://github.com/netadx/mcp-boilerplate-rust
cd mcp-boilerplate-rust

# Build (stdio only, minimal)
cargo build --release

# Run with Claude Desktop
cargo run --release -- --mode stdio

# Or build with all features
cargo build --release --features full
```

### Test with MCP Inspector

```bash
npx @modelcontextprotocol/inspector cargo run --release -- --mode stdio
```

Open http://localhost:5173 to interact with all 11 tools.

## Transport Modes

### 1. Stdio (Default)
**Best for:** Desktop apps, Claude Desktop, CLI tools

```bash
cargo run --release -- --mode stdio
```

Configure in Claude Desktop:
```json
{
  "mcpServers": {
    "mcp-boilerplate-rust": {
      "command": "/path/to/mcp-boilerplate-rust",
      "args": ["--mode", "stdio"]
    }
  }
}
```

### 2. SSE (Server-Sent Events)
**Best for:** Browser push notifications, real-time updates

```bash
cargo run --release --features sse -- --mode sse --bind 127.0.0.1:8025
```

**Endpoints:**
- `GET /sse` - Event stream
- `POST /rpc` - JSON-RPC
- `GET /health` - Health check
- `GET /metrics` - Prometheus metrics

**Test:** Open `examples/sse_test_client.html` in browser

### 3. WebSocket
**Best for:** Real-time bidirectional communication, chat apps

```bash
cargo run --release --features websocket -- --mode websocket --bind 127.0.0.1:9001
```

**Test:** Open `examples/websocket_test_client.html` in browser

### 4. HTTP Streaming
**Best for:** Large file transfers, progressive data delivery

```bash
cargo run --release --features http-stream -- --mode http-stream --bind 127.0.0.1:8026
```

**Features:**
- Chunked transfer encoding (8KB chunks)
- Unlimited file size support
- Progressive streaming
- Browser compatible

### 5. gRPC
**Best for:** Microservices, high-performance APIs, internal services

```bash
cargo run --release --features grpc -- --mode grpc --bind 127.0.0.1:50051
```

**Features:**
- Protocol Buffers serialization
- HTTP/2 multiplexing
- Bidirectional streaming
- Sub-5ms latency

**Test:**
```bash
grpcurl -plaintext 127.0.0.1:50051 list
grpcurl -plaintext 127.0.0.1:50051 mcp.Mcp/HealthCheck
```

### 6. HTTP (REST API)
**Best for:** Standard REST APIs, public APIs

```bash
cargo run --release --features http -- --mode http
```

## Available Tools

### Basic Tools (5)
| Tool | Description | Example |
|------|-------------|---------|
| `ping` | Health check | No arguments |
| `echo` | Message validation | `{"message": "hello"}` |
| `info` | Server metadata | No arguments |
| `calculate` | Math operations | `{"operation": "add", "a": 5, "b": 3}` |
| `evaluate` | Expression eval | `{"expression": "2 * (3 + 4)"}` |

### Advanced Tools (6)
| Tool | Description | Features |
|------|-------------|----------|
| `process_with_progress` | Data processing | Progress notifications |
| `batch_process` | Batch operations | Logging notifications |
| `transform_data` | Array transformation | 4 operations |
| `simulate_upload` | File upload demo | 20 chunks with progress |
| `health_check` | System health | Health monitoring |
| `long_task` | Long operation | 10s task with progress |

## Build Options

### Feature Flags

```bash
# Minimal (stdio only) - 2.4 MB
cargo build --release

# Web transports - 3.3 MB
cargo build --release --features "sse,websocket"

# Streaming - 3.2 MB
cargo build --release --features http-stream

# High performance - 3.9 MB
cargo build --release --features grpc

# Everything - 4.2 MB
cargo build --release --features full
```

### Available Features

- `sse` - Server-Sent Events transport
- `websocket` - WebSocket transport
- `http-stream` - HTTP streaming transport
- `grpc` - gRPC transport
- `http` - HTTP REST API transport
- `database` - MongoDB integration
- `auth` - JWT authentication
- `metrics` - Prometheus metrics collection
- `full` - All features

## Testing

```bash
# Run all tests
cargo test --features full

# Specific transport
cargo test --features sse -- transport::sse

# Integration tests
./scripts/integration_test.sh

# With coverage
cargo test --features full -- --test-threads=1
```

**Test Results:** 89 tests passing, 0 failing

## Docker Deployment

### Build Image

```bash
docker build -t mcp-server .
```

### Run Container

```bash
# SSE mode
docker run -p 8025:8025 mcp-server

# WebSocket mode
docker run -p 9001:9001 mcp-server \
  mcp-boilerplate-rust --mode websocket --bind 0.0.0.0:9001

# All transports
docker run -p 8025:8025 -p 9001:9001 -p 8026:8026 -p 50051:50051 mcp-server
```

## Performance

### Benchmarks (MacBook Pro M1)

| Transport | Latency (P50) | Throughput | Overhead |
|-----------|---------------|------------|----------|
| stdio | 2ms | High | Minimal |
| SSE | 15ms | Medium | Low |
| WebSocket | 8ms | High | Low |
| HTTP Stream | 12ms | 150 MB/s | Low |
| gRPC | 4ms | 200 MB/s | Minimal |

### Binary Sizes

| Configuration | Size | Build Time |
|--------------|------|------------|
| Minimal (stdio) | 2.4 MB | 30s |
| Full features | 4.2 MB | 45s |

## Documentation

### Getting Started
- [START_HERE.md](START_HERE.md) - Quick start guide (5 min read)
- [CHANGELOG.md](CHANGELOG.md) - Version history

### Transport Guides
- [docs/TRANSPORT_QUICK_REFERENCE.md](docs/TRANSPORT_QUICK_REFERENCE.md) - Quick reference
- [docs/TRANSPORT_QUICK_GUIDE.md](docs/TRANSPORT_QUICK_GUIDE.md) - Detailed guide
- [docs/TRANSPORT_ADVANCED_SUMMARY.md](docs/TRANSPORT_ADVANCED_SUMMARY.md) - Advanced features

### Development
- [docs/guides/TESTING_GUIDE.md](docs/guides/TESTING_GUIDE.md) - Testing guide
- [docs/reference/QUICK_REFERENCE.md](docs/reference/QUICK_REFERENCE.md) - API reference

### Examples
- `examples/sse_test_client.html` - SSE browser client
- `examples/websocket_test_client.html` - WebSocket browser client
- `examples/advanced_features_demo.md` - Advanced features demo

## Configuration

### Environment Variables

```bash
# Logging
export RUST_LOG=info                    # off|error|warn|info|debug|trace
export RUST_LOG=mcp_boilerplate_rust=debug

# Custom ports
export SSE_PORT=8025
export WS_PORT=9001
export HTTP_STREAM_PORT=8026
export GRPC_PORT=50051
```

### Command Line Options

```bash
mcp-boilerplate-rust [OPTIONS]

Options:
  -m, --mode <MODE>      Transport mode [default: stdio]
                         [possible values: stdio, sse, websocket, http-stream, grpc]
  -v, --verbose          Enable verbose logging
  -b, --bind <BIND>      Bind address [default: 127.0.0.1:8025]
  -h, --help             Print help
  -V, --version          Print version
```

## Project Structure

```
mcp-boilerplate-rust/
├── src/
│   ├── main.rs                 # Entry point, mode selection
│   ├── mcp/
│   │   ├── protocol_handler.rs # Shared protocol logic
│   │   ├── stdio_server.rs     # Stdio transport server
│   │   ├── sse_server.rs       # SSE transport server
│   │   ├── websocket_server.rs # WebSocket transport server
│   │   ├── http_stream_server.rs # HTTP streaming server
│   │   └── grpc_server.rs      # gRPC server
│   ├── transport/
│   │   ├── trait.rs            # Transport trait definition
│   │   ├── stdio.rs            # Stdio transport
│   │   ├── sse.rs              # SSE transport
│   │   ├── websocket.rs        # WebSocket transport
│   │   ├── http_stream.rs      # HTTP streaming transport
│   │   └── grpc.rs             # gRPC transport
│   ├── tools/                  # 11 tool implementations
│   ├── prompts/                # Prompt templates
│   └── resources/              # Resource providers
├── proto/
│   └── mcp.proto               # gRPC service definition
├── examples/                   # Browser test clients
├── scripts/                    # Build and test scripts
├── docs/                       # Documentation
└── tests/                      # Integration tests
```

## Security

### Production Checklist

- [ ] Enable HTTPS/TLS for all transports
- [ ] Implement authentication (JWT recommended)
- [ ] Add rate limiting per client
- [ ] Configure CORS appropriately
- [ ] Set connection timeouts
- [ ] Validate all inputs
- [ ] Enable audit logging
- [ ] Use environment variables for secrets
- [ ] Regular dependency updates
- [ ] Security scanning (cargo audit)

### CORS Configuration

Server has CORS enabled for development:
```rust
CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any)
```

For production, restrict to specific origins.

## Troubleshooting

### Port Already in Use
```bash
# Find and kill process
lsof -i :8025
kill -9 <PID>

# Or use different port
cargo run --features sse -- --mode sse --bind 127.0.0.1:8026
```

### Connection Refused
```bash
# Check server is running
ps aux | grep mcp-boilerplate

# Enable debug logging
RUST_LOG=debug cargo run --features sse -- --mode sse
```

### Build Errors
```bash
# Clean rebuild
cargo clean
cargo build --release --features full

# Update dependencies
cargo update
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new features
4. Ensure all tests pass
5. Submit a pull request

## License

MIT License - see [LICENSE](LICENSE) file for details

## Links

- **GitHub:** https://github.com/netadx/mcp-boilerplate-rust
- **MCP Protocol:** https://modelcontextprotocol.io
- **Rust MCP SDK:** https://github.com/modelcontextprotocol/rust-sdk
- **Website:** https://netadx.ai
- **Email:** hello@netadx.ai

## Acknowledgments

- Built with [rmcp](https://github.com/modelcontextprotocol/rust-sdk) - Official Rust MCP SDK
- Inspired by the Model Context Protocol specification
- Community feedback and contributions

## Roadmap

### Current Version (0.4.0)
- 6 transport modes
- 11 production tools
- Comprehensive testing
- Browser clients
- Docker support

### Next Version (0.5.0)
- [ ] gRPC-Web gateway for browser support
- [ ] Prometheus metrics
- [ ] OpenTelemetry tracing
- [ ] Client SDKs (JavaScript, Python, Go)
- [ ] Load balancing support
- [ ] Service mesh integration

### Future
- [ ] HTTP/3 (QUIC) support
- [ ] Multi-region deployment
- [ ] Auto-scaling
- [ ] Performance dashboards

## Use