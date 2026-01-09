# Transport Documentation

All 6 transport modes for MCP Boilerplate Rust.

**Version:** 0.5.2  
**Last Updated:** 2026-01-09 HCMC

---

## Overview

| Transport | Port | Latency | Best For |
|-----------|------|---------|----------|
| Stdio | N/A | 2ms | Desktop apps, Claude Desktop |
| SSE | 8025 | 15ms | Browser push notifications |
| WebSocket | 9001 | 8ms | Real-time bidirectional |
| HTTP | 8080 | 20ms | REST APIs |
| HTTP Stream | 8026 | 12ms | Large file transfers |
| gRPC | 50051 | 4ms | Microservices |

---

## Quick Start

```bash
# Stdio (default, no feature needed)
cargo run --release -- --mode stdio

# SSE
cargo run --release --features sse -- --mode sse --bind 127.0.0.1:8025

# WebSocket
cargo run --release --features websocket -- --mode websocket --bind 127.0.0.1:9001

# HTTP
cargo run --release --features http -- --mode http

# HTTP Streaming
cargo run --release --features http-stream -- --mode http-stream --bind 127.0.0.1:8026

# gRPC
cargo run --release --features grpc -- --mode grpc --bind 127.0.0.1:50051
```

---

## 1. Stdio

Default transport for desktop apps and Claude Desktop.

**Build:**
```bash
cargo build --release  # 2.4MB binary
```

**Test:**
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | \
  ./target/release/mcp-boilerplate-rust --mode stdio
```

**Claude Desktop Config:**
```json
{
  "mcpServers": {
    "mcp-rust": {
      "command": "/path/to/mcp-boilerplate-rust",
      "args": ["--mode", "stdio"]
    }
  }
}
```

---

## 2. SSE (Server-Sent Events)

Browser push notifications and live updates.

**Build:**
```bash
cargo build --release --features sse
```

**Endpoints:**
```
GET  /           Server info
GET  /health     Health check
GET  /sse        Event stream
POST /rpc        JSON-RPC
GET  /tools      List tools
POST /tools/call Call tool
```

**Test:**
```bash
# Event stream
curl -N http://127.0.0.1:8025/sse

# RPC call
curl -X POST http://127.0.0.1:8025/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
```

**Browser:**
```javascript
const es = new EventSource('http://127.0.0.1:8025/sse');
es.onmessage = (e) => console.log(JSON.parse(e.data));
```

---

## 3. WebSocket

Real-time bidirectional communication.

**Build:**
```bash
cargo build --release --features websocket
```

**Endpoints:**
```
GET  /        Server info
GET  /health  Health check
GET  /ws      WebSocket upgrade
```

**Test:**
```bash
# Using websocat
websocat ws://127.0.0.1:9001/ws
# Then type: {"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}
```

**Browser:**
```javascript
const ws = new WebSocket('ws://127.0.0.1:9001/ws');
ws.onmessage = (e) => console.log(JSON.parse(e.data));
ws.send(JSON.stringify({jsonrpc:"2.0",id:1,method:"tools/list",params:{}}));
```

---

## 4. HTTP

Standard REST API.

**Build:**
```bash
cargo build --release --features http
```

**Endpoints:**
```
GET  /           Server info
GET  /health     Health check
GET  /tools      List tools
POST /tools/call Call tool
POST /rpc        JSON-RPC
```

**With Auth:**
```bash
cargo build --release --features "http,auth"
JWT_SECRET="your-secret" ./target/release/mcp-boilerplate-rust --mode http

# Additional endpoints when auth enabled:
POST /auth/login        Get token
GET  /auth/verify       Verify token
GET  /auth/me           Current user (protected)
GET  /protected/tools   Protected tools list
```

**Test:**
```bash
curl http://127.0.0.1:8080/tools
curl -X POST http://127.0.0.1:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{"name":"echo","arguments":{"message":"hello"}}'
```

---

## 5. HTTP Streaming

Large file transfers and progressive data delivery.

**Build:**
```bash
cargo build --release --features http-stream
```

**Endpoints:**
```
GET  /           Server info
GET  /health     Health check
GET  /stream     Start stream
GET  /stream/:id Stream by ID
POST /rpc        JSON-RPC
```

**Test:**
```bash
curl -N http://127.0.0.1:8026/stream
```

**Performance:** 150 MB/s throughput

---

## 6. gRPC

High-performance microservices communication.

**Build:**
```bash
cargo build --release --features grpc
```

**Methods:**
```
mcp.Mcp/JsonRpc             JSON-RPC handler
mcp.Mcp/ListTools           List tools
mcp.Mcp/CallTool            Call tool
mcp.Mcp/StreamResponses     Server streaming
mcp.Mcp/BidirectionalStream Bidirectional
mcp.Mcp/HealthCheck         Health check
```

**Test:**
```bash
# List services
grpcurl -plaintext 127.0.0.1:50051 list

# Call method
grpcurl -plaintext \
  -d '{"payload": "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\"}"}' \
  127.0.0.1:50051 mcp.Mcp/JsonRpc
```

**Performance:** 200 MB/s throughput, 4ms latency

---

## Build Options

```bash
# Minimal (stdio only)
cargo build --release                           # 2.4 MB

# Single transport
cargo build --release --features sse            # 3.3 MB
cargo build --release --features websocket      # 3.3 MB
cargo build --release --features http           # 3.1 MB
cargo build --release --features http-stream    # 3.2 MB
cargo build --release --features grpc           # 3.9 MB

# Multiple transports
cargo build --release --features "sse,websocket,http"

# All features
cargo build --release --features full           # 4.2 MB
```

---

## Feature Flags

| Feature | Includes |
|---------|----------|
| `sse` | axum, tokio-stream, futures, async-stream |
| `websocket` | axum, tokio-tungstenite, futures |
| `http` | axum, tower, tower-http |
| `http-stream` | axum, tower, tower-http, futures |
| `grpc` | tonic, prost, futures |
| `auth` | jsonwebtoken (requires http) |
| `full` | All above + metrics, otel, database |

---

## Environment Variables

```bash
# Logging
RUST_LOG=info                    # info|debug|trace|off

# Server
HOST=0.0.0.0                     # Bind host
PORT=8025                        # Bind port

# Auth (http,auth features)
JWT_SECRET=your-secret-key       # Required for auth
JWT_EXPIRY_SECONDS=86400         # Token expiry (default 24h)
```

---

## Troubleshooting

### Port in use
```bash
lsof -i :8025
kill -9 <PID>
```

### Connection refused
```bash
RUST_LOG=debug cargo run --features sse -- --mode sse
```

### CORS issues
CORS is enabled by default for all origins.

### Build errors
```bash
cargo clean
cargo build --release --features full
```

---

## Test Clients

Browser test clients in `examples/`:
- `sse_test_client.html`
- `websocket_test_client.html`

```bash
open examples/sse_test_client.html
open examples/websocket_test_client.html
```

---

## Decision Guide

| Need | Use |
|------|-----|
| Desktop/CLI integration | Stdio |
| Browser push notifications | SSE |
| Real-time chat | WebSocket |
| REST API | HTTP |
| Large file streaming | HTTP Stream |
| Microservices | gRPC |