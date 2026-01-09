# Setup Guide

Complete setup guide for MCP Boilerplate Rust.

**Version:** 0.5.2  
**Last Updated:** 2026-01-09 HCMC

---

## Prerequisites

- Rust 1.75+ (`rustup`, `cargo`)
- macOS / Linux / Windows

---

## Installation

```bash
git clone https://github.com/netadx/mcp-boilerplate-rust.git
cd mcp-boilerplate-rust
cargo build --release
```

---

## Quick Test

```bash
# Test stdio mode
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | \
  ./target/release/mcp-boilerplate-rust --mode stdio

# Test with MCP Inspector
npx @modelcontextprotocol/inspector ./target/release/mcp-boilerplate-rust --mode stdio
```

---

## Claude Desktop Setup

**1. Build:**
```bash
cargo build --release
```

**2. Configure:**

Edit `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "mcp-rust": {
      "command": "/absolute/path/to/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust",
      "args": ["--mode", "stdio"]
    }
  }
}
```

**3. Restart Claude Desktop:**
```bash
killall Claude
open -a Claude
```

**4. Test:**
Ask Claude: "What MCP tools are available?"

---

## Build Options

```bash
# Minimal (stdio only) - 2.4 MB
cargo build --release

# HTTP transport - 3.1 MB
cargo build --release --features http

# HTTP with auth
cargo build --release --features "http,auth"

# All transports - 4.2 MB
cargo build --release --features full
```

---

## Feature Flags

| Flag | Description |
|------|-------------|
| (none) | Stdio only |
| `http` | HTTP REST API |
| `sse` | Server-Sent Events |
| `websocket` | WebSocket |
| `http-stream` | HTTP streaming |
| `grpc` | gRPC |
| `auth` | JWT authentication |
| `metrics` | Prometheus metrics |
| `otel` | OpenTelemetry tracing |
| `full` | All features |

---

## Running

```bash
# Stdio (default)
./target/release/mcp-boilerplate-rust --mode stdio

# HTTP
./target/release/mcp-boilerplate-rust --mode http

# SSE
./target/release/mcp-boilerplate-rust --mode sse --bind 127.0.0.1:8025

# WebSocket
./target/release/mcp-boilerplate-rust --mode websocket --bind 127.0.0.1:9001

# HTTP with auth
JWT_SECRET="your-secret-key" ./target/release/mcp-boilerplate-rust --mode http
```

---

## Environment Variables

```bash
# Logging
RUST_LOG=info                    # info|debug|trace|off

# Server
HOST=0.0.0.0                     # Bind host
PORT=8025                        # Bind port

# Auth (requires http,auth features)
JWT_SECRET=your-secret-key       # Required
JWT_EXPIRY_SECONDS=86400         # Optional, default 24h
AUTH_USERNAME=admin              # Optional, default admin
```

---

## Available Tools

| Tool | Description |
|------|-------------|
| echo | Echo back a message |
| ping | Health check |
| info | Server info |
| calculate | Arithmetic operations |
| evaluate | Math expression evaluation |
| process_with_progress | Progress demo |
| batch_process | Batch operations |
| transform_data | Data transformation |
| simulate_upload | Upload simulation |
| health_check | System health |
| long_task | Long-running task demo |

---

## Testing

```bash
# Unit tests
cargo test --features full

# Integration tests
./scripts/integration_test.sh

# Specific test
cargo test middleware::auth
```

---

## Directory Structure

```
mcp-boilerplate-rust/
├── src/
│   ├── main.rs              # Entry point
│   ├── mcp/                 # MCP servers
│   ├── tools/               # Tool implementations
│   ├── middleware/          # Auth middleware
│   └── utils/               # Utilities
├── docs/                    # Documentation
├── examples/                # Browser test clients
├── proto/                   # gRPC definitions
└── scripts/                 # Build/test scripts
```

---

## Troubleshooting

### Binary not found
```bash
cargo build --release
ls -la target/release/mcp-boilerplate-rust
```

### Port in use
```bash
lsof -i :8025
kill -9 <PID>
```

### Claude Desktop not connecting
1. Check absolute path in config
2. Verify JSON syntax
3. Restart Claude completely
4. Check Claude logs

### Auth not working
```bash
# Ensure JWT_SECRET is set
echo $JWT_SECRET

# Test login
curl -X POST http://127.0.0.1:8025/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}'
```

---

## Performance

| Mode | Binary | Memory | Latency |
|------|--------|--------|---------|
| Stdio | 2.4 MB | <5 MB | 2ms |
| HTTP | 3.1 MB | <8 MB | 20ms |
| SSE | 3.3 MB | <8 MB | 15ms |
| WebSocket | 3.3 MB | <8 MB | 8ms |
| gRPC | 3.9 MB | <10 MB | 4ms |

---

## Next Steps

- [TRANSPORTS.md](TRANSPORTS.md) - Transport details
- [features/AUTH.md](features/AUTH.md) - Authentication
- [API.md](API.md) - API reference