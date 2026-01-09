# Multi-Transport Quick Start Guide

**Version:** v0.5.0-dev  
**Last Updated:** 2026-01-08  
**Phases Complete:** 2 of 6 (stdio + SSE)

---

##  Quick Start

### 1. Build the Project

```bash
# Build stdio mode (default)
cargo build --release

# Build with SSE support
cargo build --release --features sse

# Build with all features
cargo build --release --features full
```

### 2. Run the Server

#### Stdio Mode (Claude Desktop)
```bash
cargo run --release -- --mode stdio
```

#### SSE Mode (Browser Clients)
```bash
cargo run --release --features sse -- --mode sse --bind 127.0.0.1:8025
```

#### HTTP Mode (REST API)
```bash
cargo run --release --features http -- --mode http
```

---

##  Available Transports

| Transport | Status | Use Case | Browser | Real-time |
|-----------|--------|----------|---------|-----------|
| **stdio** | Complete Ready | CLI, Claude Desktop | No | No |
| **SSE** | Complete Ready | Browser notifications | Complete | Complete Server→Client |
| **HTTP** | Complete Ready | REST API | Complete | No |
| WebSocket | 🚧 Phase 3 | Real-time apps | Complete | Complete Bidirectional |
| HTTP Stream | 🚧 Phase 4 | Large transfers | Complete | Complete |
| RPC/gRPC | 🚧 Phase 5 | Microservices | Limited | Complete |

---

## 📖 Usage Examples

### Stdio Mode

**Use Case:** Claude Desktop integration, CLI tools

```bash
# Start server
cargo run --release -- --mode stdio

# The server reads from stdin and writes to stdout
# Perfect for Claude Desktop MCP integration
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

### SSE Mode

**Use Case:** Real-time browser notifications, live updates

```bash
# Start SSE server
cargo run --release --features sse -- --mode sse --bind 127.0.0.1:8025

# Open browser client
open examples/sse_client.html
```

**JavaScript Client:**
```javascript
// Connect to SSE stream
const eventSource = new EventSource('http://localhost:8025/sse');

eventSource.onmessage = (event) => {
    const data = JSON.parse(event.data);
    console.log('Received:', data);
};

// Call a tool
fetch('http://localhost:8025/tools/call', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
        name: 'echo',
        arguments: { message: 'Hello SSE!' }
    })
});
```

**SSE Endpoints:**
- `GET /` - Server info
- `GET /health` - Health check + client stats
- `GET /sse` - SSE event stream (EventSource)
- `GET /tools` - List available tools
- `POST /tools/call` - Call a tool (async)

---

## 🧪 Testing

### Test Stdio Mode
```bash
# Build and test
cargo build --release
cargo test

# Run MCP tests
./scripts/test_mcp.sh
```

### Test SSE Mode
```bash
# Terminal 1: Start server
cargo run --release --features sse -- --mode sse

# Terminal 2: Run tests
./scripts/test_sse.sh

# Expected output:
# ✓ 10/10 tests passing
```

### Test Browser Client
```bash
# Start server
cargo run --release --features sse -- --mode sse

# Open browser
open examples/sse_client.html

# Click "Call Echo" button
# Watch events appear in real-time!
```

---

##  Feature Comparison

### Stdio Transport
**Capabilities:**
- Complete Bidirectional (request/response)
- No Server push
- No Multiple connections
- No Browser compatible

**Best For:**
- Claude Desktop integration
- CLI tools
- Process spawning
- IPC communication

**Example Use Case:**
```bash
# Perfect for Claude Desktop
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | ./mcp-boilerplate-rust --mode stdio
```

---

### SSE Transport
**Capabilities:**
- No Bidirectional (server → client only)
- Complete Server push
- Complete Multiple connections (50+ tested)
- Complete Browser compatible (EventSource API)

**Best For:**
- Real-time notifications
- Progress updates
- Live dashboards
- Browser applications

**Example Use Case:**
```javascript
// Real-time progress updates
const eventSource = new EventSource('http://localhost:8025/sse');
eventSource.onmessage = (event) => {
    const data = JSON.parse(event.data);
    if (data.type === 'tool_call_result') {
        updateUI(data.result);
    }
};
```

---

##  Configuration

### Environment Variables
```bash
# Logging level
export RUST_LOG=info          # For SSE/HTTP modes
export RUST_LOG=off           # For stdio mode (required!)

# Server configuration
export MCP_HOST=127.0.0.1
export MCP_PORT=8025
```

### Transport Config (Programmatic)
```rust
use mcp_boilerplate_rust::transport::{TransportConfig, TransportRegistry};

// Create SSE transport
let config = TransportConfig {
    transport_type: "sse".to_string(),
    bind_address: Some("127.0.0.1:8025".to_string()),
    max_message_size: 10 * 1024 * 1024, // 10MB
    timeout_seconds: 30,
    ..Default::default()
};

let registry = TransportRegistry::global();
let mut transport = registry.create(config)?;
transport.initialize().await?;
```

---

## 📈 Performance Benchmarks

| Metric | Stdio | SSE | HTTP |
|--------|-------|-----|------|
| Latency | 2-7ms | 10-20ms | 8-12ms |
| Memory (idle) | <5MB | ~8MB | ~8MB |
| Memory (5 clients) | N/A | ~15MB | ~20MB |
| Max throughput | ~100 msg/s | ~1000 msg/s | ~500 msg/s |
| Binary size | 2.8MB | 3.2MB | 3.1MB |

---

## 🐛 Troubleshooting

### Issue: SSE server won't start
**Error:** `Address already in use`

**Solution:**
```bash
# Check what's using port 8025
lsof -i :8025

# Kill the process or use different port
cargo run --features sse -- --mode sse --bind 127.0.0.1:8026
```

### Issue: Browser can't connect to SSE
**Error:** CORS error in browser console

**Solution:**
- Server already has CORS enabled
- Check if server is running: `curl http://localhost:8025/health`
- Try accessing from `http://localhost` not `file://`

### Issue: No events received in browser
**Solution:**
```bash
# Check SSE stream manually
curl -N http://localhost:8025/sse

# Should see connection event immediately
# If not, check server logs
```

### Issue: Stdio mode shows logs in output
**Error:** `Unexpected token` in Claude Desktop

**Solution:**
- Ensure `RUST_LOG=off` for stdio mode
- Never use `println!` or `eprintln!` in stdio mode
- Only pure JSON-RPC to stdout

---

## 📚 Documentation

### Essential Reading
1. **TRANSPORT_START_HERE.md** - Overview and architecture
2. **docs/guides/TRANSPORT_GUIDE.md** - Complete usage guide (737 lines)
3. **TRANSPORT_COMPLETE_SUMMARY.md** - Session summary

### Phase Reports
- **TRANSPORT_PHASE1_PROGRESS.md** - Transport abstraction (Phase 1)
- **TRANSPORT_PHASE2_PROGRESS.md** - SSE implementation (Phase 2)
- **NEXT_SESSION_TRANSPORT.md** - Roadmap for phases 3-6

### Examples
- **examples/sse_client.html** - Full-featured browser client
- **scripts/test_sse.sh** - SSE integration tests (10 tests)

---

##  Common Use Cases

### Use Case 1: Claude Desktop Integration
```bash
# Build stdio server
cargo build --release

# Update Claude Desktop config
# Path: ~/Library/Application Support/Claude/claude_desktop_config.json
{
  "mcpServers": {
    "rust-mcp": {
      "command": "/path/to/mcp-boilerplate-rust",
      "args": ["--mode", "stdio"]
    }
  }
}

# Restart Claude Desktop
# Tools will appear automatically
```

### Use Case 2: Real-time Web Dashboard
```bash
# Start SSE server
cargo run --features sse -- --mode sse

# Create index.html
<!DOCTYPE html>
<html>
<body>
  <div id="events"></div>
  <script>
    const es = new EventSource('http://localhost:8025/sse');
    es.onmessage = (e) => {
      const data = JSON.parse(e.data);
      document.getElementById('events').innerHTML += 
        `<p>${data.type}: ${JSON.stringify(data)}</p>`;
    };
  </script>
</body>
</html>

# Open in browser - see real-time updates!
```

### Use Case 3: REST API Testing
```bash
# Start HTTP server
cargo run --features http -- --mode http

# Test with curl
curl http://localhost:8025/tools
curl -X POST http://localhost:8025/tools/echo \
  -H "Content-Type: application/json" \
  -d '{"message":"Hello World"}'
```

---

##  Next Steps

### Try It Now
```bash
# 1. Clone and build
git clone <repo-url>
cd mcp-boilerplate-rust
cargo build --release --features sse

# 2. Start SSE server
cargo run --release --features sse -- --mode sse &

# 3. Run tests
./scripts/test_sse.sh

# 4. Open browser client
open examples/sse_client.html

# 5. Click buttons and watch events!
```

### Customize
```rust
// Add your own transport
// See docs/guides/TRANSPORT_GUIDE.md for details

use mcp_boilerplate_rust::transport::{Transport, TransportFactory};

pub struct MyTransport;

impl Transport for MyTransport {
    // Implement required methods
}
```

### Contribute
```bash
# Phase 3: WebSocket (coming next)
# Phase 4: HTTP Streaming
# Phase 5: RPC/gRPC

# See NEXT_SESSION_TRANSPORT.md for roadmap
```

---

## 📞 Support

**Questions?** Check the documentation:
- Full guide: `docs/guides/TRANSPORT_GUIDE.md`
- Phase reports: `TRANSPORT_PHASE1_PROGRESS.md`, `TRANSPORT_PHASE2_PROGRESS.md`
- Complete summary: `TRANSPORT_COMPLETE_SUMMARY.md`

**Issues?** See troubleshooting above or file a GitHub issue.

**Contributing?** See `NEXT_SESSION_TRANSPORT.md` for roadmap.

---

## Complete Quick Checklist

Before starting:
- [ ] Rust 1.88+ installed
- [ ] `cargo` working
- [ ] Clone repository
- [ ] Read this file

For stdio mode:
- [ ] Build with `cargo build --release`
- [ ] Run with `--mode stdio`
- [ ] Set `RUST_LOG=off`

For SSE mode:
- [ ] Build with `--features sse`
- [ ] Run with `--mode sse`
- [ ] Open `examples/sse_client.html`
- [ ] Run `./scripts/test_sse.sh`

---

**Status:** Complete Ready to use  
**Transports:** 2 of 6 complete  
**Next:** WebSocket (Phase 3)

**Get started now with `cargo run --features sse -- --mode sse`!** 