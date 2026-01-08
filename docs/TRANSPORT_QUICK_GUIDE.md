# Transport Quick Start Guide

**Last Updated:** 2026-01-09 (HCMC Timezone)  
**Version:** v0.5.0
**Status:** Production Ready

---

##  Quick Start

Choose your transport based on your use case:

| Transport | Use Case | Command |
|-----------|----------|---------|
| **stdio** | Claude Desktop, CLI tools | `cargo run -- --mode stdio` |
| **SSE** | Browser notifications, live updates | `cargo run --features sse -- --mode sse` |
| **WebSocket** | Real-time chat, bidirectional | `cargo run --features websocket -- --mode websocket` |
| **HTTP** | REST API, simple integration | `cargo run --features http -- --mode http` |
| **gRPC** | Microservices, high performance | `cargo run --features grpc -- --mode grpc` |

---

##  Transport Modes

### 1. Stdio (Default)

**Best for:** Claude Desktop, CLI tools, process spawning

```bash
# Build
cargo build --release

# Run
./target/release/mcp-boilerplate-rust --mode stdio

# Or with cargo
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | \
  cargo run --release -- --mode stdio
```

**Configuration:**
- No network required
- Uses stdin/stdout
- No logging to stdout (would interfere with JSON-RPC)

**Claude Desktop Config:**
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

---

### 2. SSE (Server-Sent Events)

**Best for:** Browser push notifications, live updates, one-way server→client

```bash
# Build with SSE feature
cargo build --release --features sse

# Run (default: 127.0.0.1:8025)
./target/release/mcp-boilerplate-rust --mode sse

# Custom bind address
./target/release/mcp-boilerplate-rust --mode sse --bind 0.0.0.0:8080
```

**Endpoints:**
- `GET /sse` - SSE event stream (EventSource)
- `POST /rpc` - JSON-RPC endpoint (recommended)
- `POST /tools/call` - Legacy tool call endpoint
- `GET /tools` - List available tools
- `GET /health` - Health check

**Browser Client Example:**
```javascript
// Connect to SSE stream
const eventSource = new EventSource('http://localhost:8025/sse');

eventSource.onmessage = (event) => {
    const data = JSON.parse(event.data);
    console.log('Received:', data);
};

eventSource.onerror = (error) => {
    console.error('SSE Error:', error);
};

// Call a tool via JSON-RPC
fetch('http://localhost:8025/rpc', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
        jsonrpc: '2.0',
        id: 1,
        method: 'tools/call',
        params: {
            name: 'echo',
            arguments: { message: 'Hello SSE!' }
        }
    })
})
.then(res => res.json())
.then(data => console.log('Response:', data));
```

**Features:**
- Real-time server→client push
- Automatic reconnection
- CORS enabled for browsers
- Multi-client support
- Progress notifications

**Test with curl:**
```bash
# Listen to SSE stream
curl -N http://localhost:8025/sse

# Call a tool
curl -X POST http://localhost:8025/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "ping",
      "arguments": {}
    }
  }'

# List tools
curl http://localhost:8025/tools
```

**HTML Example:**
See `examples/sse_client.html` for a complete working example.

---

### 3. WebSocket

**Best for:** Real-time bidirectional communication, chat applications, live dashboards

```bash
# Build with WebSocket feature
cargo build --release --features websocket

# Run (default: 127.0.0.1:9001)
./target/release/mcp-boilerplate-rust --mode websocket

# Custom bind address
./target/release/mcp-boilerplate-rust --mode websocket --bind 0.0.0.0:9001
```

**Endpoints:**
- `WS /ws` - WebSocket endpoint
- `GET /health` - Health check

**JavaScript Client Example:**
```javascript
const ws = new WebSocket('ws://localhost:9001/ws');

ws.onopen = () => {
    console.log('Connected!');
    
    // Send JSON-RPC request
    ws.send(JSON.stringify({
        jsonrpc: '2.0',
        id: 1,
        method: 'initialize',
        params: {}
    }));
};

ws.onmessage = (event) => {
    const response = JSON.parse(event.data);
    console.log('Received:', response);
};

ws.onerror = (error) => {
    console.error('WebSocket Error:', error);
};

// Call a tool
function callTool(name, args) {
    ws.send(JSON.stringify({
        jsonrpc: '2.0',
        id: Date.now(),
        method: 'tools/call',
        params: {
            name: name,
            arguments: args
        }
    }));
}

// Example: Echo
callTool('echo', { message: 'Hello WebSocket!' });

// Example: Calculate
callTool('calculate', { operation: 'add', a: 5, b: 3 });
```

**Features:**
- Full duplex communication
- Low latency
- Connection persistence
- Multi-client support
- Real-time updates

**Test with wscat:**
```bash
# Install wscat
npm install -g wscat

# Connect
wscat -c ws://localhost:9001/ws

# Send JSON-RPC (paste this when connected)
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ping","arguments":{}}}
```

---

### 4. HTTP (REST API)

**Best for:** Simple REST integrations, testing, debugging

```bash
# Build with HTTP feature
cargo build --release --features http

# Run (default: 127.0.0.1:3000)
./target/release/mcp-boilerplate-rust --mode http
```

**Endpoints:**
- `GET /health` - Health check
- `GET /tools` - List available tools
- `POST /tools/echo` - Echo tool
- `POST /tools/ping` - Ping tool
- `POST /tools/info` - Info tool

**Test with curl:**
```bash
# Health check
curl http://localhost:3000/health

# List tools
curl http://localhost:3000/tools

# Call echo tool
curl -X POST http://localhost:3000/tools/echo \
  -H "Content-Type: application/json" \
  -d '{"message": "Hello HTTP!"}'

# Call ping tool
curl -X POST http://localhost:3000/tools/ping
```

**Features:**
- Simple REST interface
- CORS enabled
- JSON responses
- Standard HTTP methods

---

### 5. gRPC & gRPC-Web

**Best for:** Microservices, high-performance APIs, internal services, browser clients

```bash
# Build with gRPC feature
cargo build --release --features grpc

# Run (default: 127.0.0.1:50051)
./target/release/mcp-boilerplate-rust --mode grpc
```

**Features:**
- Protocol Buffers serialization
- HTTP/2 multiplexing
- gRPC-Web support (Browser compatible)
- Bidirectional streaming
- Sub-5ms latency

**Test with grpcurl:**
```bash
grpcurl -plaintext 127.0.0.1:50051 list
grpcurl -plaintext 127.0.0.1:50051 mcp.Mcp/HealthCheck
```

---

##  Build Configurations

### Minimal (Stdio only)
```bash
cargo build --release
# Binary size: ~2.4 MB
```

### Single Transport
```bash
# SSE only
cargo build --release --features sse

# WebSocket only
cargo build --release --features websocket

# HTTP only
cargo build --release --features http

# gRPC only
cargo build --release --features grpc
```

### Multiple Transports
```bash
# SSE + WebSocket
cargo build --release --features "sse,websocket"

# All transports
cargo build --release --features "sse,websocket,http,http-stream"
```

### Everything
```bash
# All features (includes database, auth, etc.)
cargo build --release --features full
# Binary size: ~3.5 MB
```

---

## 🧪 Testing

### Run All Tests
```bash
cargo test
```

### Test Specific Transport
```bash
# SSE tests
cargo test --features sse -- transport::sse

# WebSocket tests
cargo test --features websocket -- transport::websocket

# All transport tests
cargo test --features "sse,websocket,http-stream,grpc"
```

### Integration Testing
```bash
# Start server in one terminal
cargo run --release --features sse -- --mode sse

# In another terminal, test with curl
curl -N http://localhost:8025/sse
```

---

##  Transport Comparison

| Feature | stdio | SSE | WebSocket | HTTP | gRPC |
|---------|-------|-----|-----------|------|------|
| **Bidirectional** | Complete | No | Complete | No | Complete |
| **Multi-client** | No | Complete | Complete | Complete | Complete |
| **Browser support** | No | Complete | Complete | Complete | Complete |
| **Real-time push** | N/A | Complete | Complete | No | Complete |
| **Low latency** | Complete | 🟡 | Complete | 🟡 | Complete |
| **Auto-reconnect** | N/A | Complete | No | N/A | Client-side |
| **Network required** | No | Complete | Complete | Complete | Complete |
| **Complexity** | Low | Medium | Medium | Low | High |

---

## 🔐 Production Deployment

### Environment Variables
```bash
# Set in .env or export
RUST_LOG=info                    # Logging level
HOST=0.0.0.0                     # Bind address (HTTP mode)
PORT=3000                        # Port (HTTP mode)
```

### Systemd Service (Linux)
```ini
[Unit]
Description=MCP Boilerplate Rust Server (SSE)
After=network.target

[Service]
Type=simple
User=mcp
WorkingDirectory=/opt/mcp-boilerplate-rust
ExecStart=/opt/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust --mode sse --bind 0.0.0.0:8025
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

### Docker
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --features "sse,websocket"

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/mcp-boilerplate-rust /usr/local/bin/
EXPOSE 8025 9001
CMD ["mcp-boilerplate-rust", "--mode", "sse", "--bind", "0.0.0.0:8025"]
```

### Nginx Reverse Proxy
```nginx
# SSE
location /sse {
    proxy_pass http://localhost:8025;
    proxy_http_version 1.1;
    proxy_set_header Connection '';
    proxy_buffering off;
    proxy_cache off;
    chunked_transfer_encoding off;
}

# WebSocket
location /ws {
    proxy_pass http://localhost:9001;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
}
```

---

## 🐛 Troubleshooting

### Port Already in Use
```bash
# Find process using port
lsof -i :8025

# Kill process
kill -9 <PID>
```

### Connection Refused
```bash
# Check server is running
ps aux | grep mcp-boilerplate-rust

# Check logs
RUST_LOG=debug cargo run --features sse -- --mode sse --verbose
```

### CORS Issues (Browser)
- Server has CORS enabled by default
- Verify Origin header is allowed
- Check browser console for errors

### WebSocket Connection Drops
- Check firewall settings
- Verify reverse proxy configuration (if using)
- Enable verbose logging: `--verbose`

---

## 📚 Additional Resources

- **Full Documentation:** See `docs/` directory
- **API Reference:** See `docs/reference/`
- **Examples:** See `examples/` directory
- **Testing Guide:** See `docs/guides/TESTING_GUIDE.md`
- **Architecture:** See `docs/advanced-features/`

---

##  Common Use Cases

### Claude Desktop Integration
```bash
# Use stdio mode
./target/release/mcp-boilerplate-rust --mode stdio
```

### Web Dashboard
```bash
# Use WebSocket for real-time updates
cargo run --release --features websocket -- --mode websocket --bind 0.0.0.0:9001
```

### Mobile App Backend
```bash
# Use HTTP or WebSocket
cargo run --release --features http -- --mode http
```

### Live Notifications
```bash
# Use SSE for server-push notifications
cargo run --release --features sse -- --mode sse --bind 0.0.0.0:8025
```

### High Performance Backend
```bash
# Use gRPC for microservices
cargo run --release --features grpc -- --mode grpc --bind 0.0.0.0:50051
```

---

## Complete Quick Verification

```bash
# 1. Build succeeds
cargo build --release --features "sse,websocket"

# 2. All tests pass
cargo test --features "sse,websocket"

# 3. Server starts
./target/release/mcp-boilerplate-rust --mode sse &
curl http://localhost:8025/health
```

Expected output:
```json
{
  "status": "healthy",
  "service": "mcp-boilerplate-rust",
  "version": "0.3.1",
  "protocol": "MCP v5",
  "mode": "sse",
  "timestamp": "2026-01-09T..."
}
```

---

**Ready to go!** 

Choose your transport, build, and deploy. All modes are production-ready and fully tested.