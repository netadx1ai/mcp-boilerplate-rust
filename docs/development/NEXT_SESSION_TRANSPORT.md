# Next Session: Multi-Transport Implementation

**Created:** 2026-01-08 (HCMC Timezone)  
**Current Version:** 0.4.0-rc  
**Target Version:** 0.5.0  
**Focus:** Transport Layer Implementation

---

## 🎯 Session Objectives

Implement comprehensive transport layer supporting multiple communication methods for MCP protocol:

1. **stdio** (✅ Already implemented)
2. **SSE** (Server-Sent Events)
3. **Streamable HTTP**
4. **RPC** (Remote Procedure Call)
5. **WebSocket**

---

## 📋 Current State

### What We Have (v0.4.0-rc)

#### ✅ Implemented
- **Stdio Transport** - Fully functional with RequestContext
- **HTTP Mode** - Basic REST endpoints (optional feature)
- **11 Advanced Tools** - All with progress notifications
- **RequestContext Integration** - Bidirectional communication
- **Progress Notifications** - Real-time updates
- **Logging Notifications** - Structured logging

#### 📊 Current Architecture
```
src/
├── main.rs                    # Entry point (stdio/http modes)
├── mcp/
│   └── stdio_server.rs        # Stdio transport implementation
├── transport/
│   ├── mod.rs                 # Transport module
│   └── stdio.rs               # Stdio transport layer
└── tools/                     # 11 tools ready for all transports
```

### What We Need (v0.5.0)

#### Transport Architecture
```
src/
├── main.rs                    # Multi-transport entry point
├── mcp/
│   ├── stdio_server.rs        # Stdio (existing)
│   ├── sse_server.rs          # SSE (NEW)
│   ├── http_server.rs         # Streamable HTTP (NEW)
│   ├── rpc_server.rs          # RPC (NEW)
│   └── ws_server.rs           # WebSocket (NEW)
├── transport/
│   ├── mod.rs                 # Transport trait
│   ├── stdio.rs               # Stdio implementation
│   ├── sse.rs                 # SSE implementation (NEW)
│   ├── http_stream.rs         # HTTP streaming (NEW)
│   ├── rpc.rs                 # RPC implementation (NEW)
│   └── websocket.rs           # WebSocket implementation (NEW)
└── tools/                     # Transport-agnostic tools
```

---

## 🔧 Transport Methods Overview

### 1. stdio (Current - ✅ Complete)

**Status:** Fully implemented in v0.4.0-rc

**Use Cases:**
- Claude Desktop integration
- CLI tools
- Process spawning

**Implementation:**
- Read from stdin, write to stdout
- JSON-RPC over stdio
- No logging to stdout/stderr

**File:** `src/mcp/stdio_server.rs`

---

### 2. SSE (Server-Sent Events) - Priority: HIGH

**What is SSE:**
- One-way server → client push
- HTTP-based, keeps connection open
- Built-in reconnection
- Text-based protocol

**Use Cases:**
- Real-time progress updates
- Live notifications
- Event streaming
- Dashboard updates

**Implementation Plan:**

```rust
// src/transport/sse.rs
use axum::response::sse::{Event, Sse};
use tokio_stream::StreamExt;

pub struct SseTransport {
    tx: mpsc::Sender<Event>,
}

impl SseTransport {
    pub async fn send_event(&self, event: Event) -> Result<()> {
        self.tx.send(event).await?;
        Ok(())
    }
    
    pub fn stream(&self) -> Sse<impl Stream<Item = Result<Event>>> {
        // Return SSE stream
    }
}
```

**Endpoints:**
- `GET /events` - SSE stream endpoint
- `POST /tools/{name}` - Execute tool (response via SSE)
- `GET /health` - Health check

**Dependencies:**
```toml
[dependencies]
axum = { version = "0.7", features = ["sse"] }
tokio-stream = "0.1"
tower-http = { version = "0.5", features = ["cors"] }
```

**Testing:**
```bash
# Subscribe to SSE stream
curl -N http://localhost:8025/events

# Execute tool (results pushed via SSE)
curl -X POST http://localhost:8025/tools/process_with_progress \
  -H "Content-Type: application/json" \
  -d '{"items": 100, "delay_ms": 50}'
```

---

### 3. Streamable HTTP - Priority: HIGH

**What is Streamable HTTP:**
- HTTP/1.1 chunked transfer encoding
- HTTP/2 streaming
- Bidirectional request/response streaming
- Lower overhead than WebSocket

**Use Cases:**
- Large data transfers
- Progressive responses
- File uploads/downloads
- Batch processing

**Implementation Plan:**

```rust
// src/transport/http_stream.rs
use axum::body::StreamBody;
use futures::Stream;

pub struct HttpStreamTransport {
    chunk_size: usize,
}

impl HttpStreamTransport {
    pub fn stream_response<T>(&self, data: T) 
        -> StreamBody<impl Stream<Item = Result<Bytes>>>
    where
        T: Iterator<Item = Bytes>,
    {
        // Return chunked stream
    }
}
```

**Endpoints:**
- `POST /stream/tools/{name}` - Streaming tool execution
- `POST /stream/upload` - Streaming upload
- `GET /stream/download/{resource}` - Streaming download

**Features:**
- Chunked transfer encoding
- Progress tracking per chunk
- Resumable uploads/downloads
- Backpressure handling

**Testing:**
```bash
# Stream large dataset transformation
curl -N -X POST http://localhost:8025/stream/tools/transform_data \
  -H "Content-Type: application/json" \
  --data-binary @large_dataset.json
```

---

### 4. RPC (Remote Procedure Call) - Priority: MEDIUM

**What is RPC:**
- Traditional request/response
- Multiple RPC protocols (JSON-RPC, gRPC, tRPC)
- Typed interfaces
- Code generation support

**Use Cases:**
- Microservices communication
- Internal APIs
- Type-safe clients
- High-performance scenarios

**Implementation Plan:**

```rust
// src/transport/rpc.rs
use tonic::{Request, Response, Status};
use prost::Message;

#[derive(Message)]
pub struct ToolRequest {
    #[prost(string, tag = "1")]
    pub name: String,
    
    #[prost(bytes, tag = "2")]
    pub arguments: Vec<u8>,
}

pub struct RpcTransport {
    addr: SocketAddr,
}

impl RpcTransport {
    pub async fn serve(&self) -> Result<()> {
        // Serve gRPC
    }
}
```

**Options:**
1. **JSON-RPC 2.0** (Current protocol, extend it)
2. **gRPC** (High performance, protobuf)
3. **tRPC** (TypeScript-first)

**Recommendation:** Start with JSON-RPC 2.0 (already using it), add gRPC later

**Dependencies:**
```toml
[dependencies]
tonic = "0.11"  # For gRPC
prost = "0.12"
```

**Testing:**
```bash
# gRPC call
grpcurl -plaintext -d '{"name":"echo","arguments":"..."}' \
  localhost:9090 mcp.ToolService/Execute
```

---

### 5. WebSocket - Priority: HIGH

**What is WebSocket:**
- Full-duplex bidirectional communication
- Persistent connection
- Low latency
- Event-driven

**Use Cases:**
- Real-time collaboration
- Interactive applications
- Chat/messaging
- Live updates

**Implementation Plan:**

```rust
// src/transport/websocket.rs
use axum::extract::ws::{WebSocket, Message};
use futures::{SinkExt, StreamExt};

pub struct WebSocketTransport {
    connections: Arc<Mutex<HashMap<String, WebSocket>>>,
}

impl WebSocketTransport {
    pub async fn handle_connection(&self, ws: WebSocket) {
        let (mut tx, mut rx) = ws.split();
        
        while let Some(msg) = rx.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Handle JSON-RPC message
                    let response = self.handle_message(text).await;
                    tx.send(Message::Text(response)).await.ok();
                }
                _ => break,
            }
        }
    }
    
    pub async fn broadcast(&self, event: Event) {
        // Broadcast to all connections
    }
}
```

**Endpoints:**
- `WS /ws` - Main WebSocket endpoint
- `WS /ws/tools` - Tool execution channel
- `WS /ws/events` - Event notifications

**Features:**
- Connection pooling
- Heartbeat/ping-pong
- Automatic reconnection
- Room/channel support

**Dependencies:**
```toml
[dependencies]
axum = { version = "0.7", features = ["ws"] }
tokio-tungstenite = "0.21"
```

**Testing:**
```bash
# WebSocket client
wscat -c ws://localhost:8025/ws

# Send JSON-RPC
> {"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"echo","arguments":{"message":"test"}}}
```

---

## 🏗️ Implementation Roadmap

### Phase 1: Transport Abstraction (Week 1)

**Goal:** Create transport-agnostic architecture

**Tasks:**
1. Define `Transport` trait
2. Refactor stdio to use trait
3. Create transport registry
4. Implement transport selection

**Files to Create:**
- `src/transport/trait.rs` - Transport trait definition
- `src/transport/registry.rs` - Transport registry

**Files to Modify:**
- `src/main.rs` - Multi-transport CLI
- `src/transport/mod.rs` - Export trait and implementations
- `src/mcp/stdio_server.rs` - Implement Transport trait

**Code Example:**
```rust
// src/transport/trait.rs
#[async_trait]
pub trait Transport: Send + Sync {
    async fn send(&self, message: Message) -> Result<()>;
    async fn receive(&self) -> Result<Message>;
    async fn close(&self) -> Result<()>;
}

// src/main.rs
#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "stdio")]
    transport: String, // stdio, sse, http, rpc, ws
}
```

### Phase 2: SSE Implementation (Week 2)

**Goal:** Add Server-Sent Events support

**Tasks:**
1. Implement SSE transport
2. Add SSE endpoints
3. Integrate with progress notifications
4. Add SSE tests

**Files to Create:**
- `src/transport/sse.rs` - SSE transport implementation
- `src/mcp/sse_server.rs` - SSE server setup
- `examples/sse_client.html` - Browser client example
- `scripts/test_sse.sh` - SSE tests

**Testing:**
```bash
cargo build --release --features sse
./target/release/mcp-boilerplate-rust --transport sse
./scripts/test_sse.sh
```

### Phase 3: WebSocket Implementation (Week 2-3)

**Goal:** Add WebSocket support

**Tasks:**
1. Implement WebSocket transport
2. Add WS endpoints
3. Connection management
4. Room/channel support
5. Add WS tests

**Files to Create:**
- `src/transport/websocket.rs` - WebSocket implementation
- `src/mcp/ws_server.rs` - WebSocket server setup
- `examples/ws_client.html` - Browser client example
- `scripts/test_websocket.sh` - WebSocket tests

**Features:**
- Auto-reconnect
- Heartbeat
- Message queuing
- Broadcast support

### Phase 4: Streamable HTTP (Week 3-4)

**Goal:** Add HTTP streaming support

**Tasks:**
1. Implement HTTP streaming
2. Chunked transfer encoding
3. Upload/download streaming
4. Backpressure handling
5. Add streaming tests

**Files to Create:**
- `src/transport/http_stream.rs` - HTTP streaming
- `src/mcp/http_server.rs` - Enhanced HTTP server
- `scripts/test_http_stream.sh` - Streaming tests

**Features:**
- Chunked responses
- Progress tracking
- Resumable transfers
- Memory efficient

### Phase 5: RPC Implementation (Week 4-5)

**Goal:** Add gRPC support (optional)

**Tasks:**
1. Define protobuf schemas
2. Generate code
3. Implement gRPC server
4. Add gRPC tests

**Files to Create:**
- `proto/mcp.proto` - Protocol definitions
- `src/transport/rpc.rs` - gRPC implementation
- `scripts/test_grpc.sh` - gRPC tests

**Note:** This is optional, JSON-RPC over other transports may be sufficient

### Phase 6: Documentation & Examples (Week 5-6)

**Goal:** Complete documentation and examples

**Tasks:**
1. Document each transport
2. Create usage examples
3. Performance comparisons
4. Migration guide
5. Best practices

**Files to Create:**
- `docs/transports/SSE.md` - SSE documentation
- `docs/transports/WEBSOCKET.md` - WebSocket docs
- `docs/transports/HTTP_STREAM.md` - HTTP streaming docs
- `docs/transports/RPC.md` - RPC documentation
- `docs/transports/COMPARISON.md` - Transport comparison
- `examples/clients/` - Client examples for each transport

---

## 📦 Dependencies to Add

### Core Dependencies
```toml
[dependencies]
# Existing
rmcp = "0.12.0"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# New for transports
axum = { version = "0.7", features = ["ws", "sse"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
tokio-stream = "0.1"
tokio-tungstenite = "0.21"
futures = "0.3"
async-stream = "0.3"

# Optional - for gRPC
tonic = { version = "0.11", optional = true }
prost = { version = "0.12", optional = true }
```

### Feature Flags
```toml
[features]
default = ["stdio"]
stdio = []
sse = ["axum/sse"]
websocket = ["axum/ws", "tokio-tungstenite"]
http-stream = ["axum", "tower-http"]
grpc = ["tonic", "prost"]
all-transports = ["sse", "websocket", "http-stream"]
```

---

## 🧪 Testing Strategy

### Unit Tests
- Each transport in isolation
- Message serialization/deserialization
- Error handling
- Connection management

### Integration Tests
- End-to-end tool execution
- Multi-client scenarios
- Connection stability
- Performance benchmarks

### Test Scripts
```bash
scripts/
├── test_stdio.sh         # Existing
├── test_sse.sh           # NEW
├── test_websocket.sh     # NEW
├── test_http_stream.sh   # NEW
├── test_grpc.sh          # NEW
└── test_all_transports.sh # Run all
```

### Performance Tests
- Latency measurements
- Throughput tests
- Connection overhead
- Memory usage
- CPU utilization

**Tool:** Criterion.rs for benchmarks

---

## 📊 Transport Comparison Matrix

| Feature | stdio | SSE | HTTP Stream | WebSocket | gRPC |
|---------|-------|-----|-------------|-----------|------|
| Bidirectional | ✅ | ❌ | ✅ | ✅ | ✅ |
| Push Events | ✅ | ✅ | ✅ | ✅ | ✅ |
| Browser Support | ❌ | ✅ | ✅ | ✅ | ⚠️ |
| Latency | Low | Medium | Low | Very Low | Very Low |
| Overhead | Minimal | Low | Low | Low | Medium |
| Complexity | Simple | Simple | Medium | Medium | High |
| Use Case | CLI/Desktop | Live Updates | Streaming | Real-time | Microservices |

**Legend:**
- ✅ Full support
- ⚠️ Limited/requires workaround
- ❌ Not supported

---

## 🎯 Success Criteria

### For Each Transport

**Must Have:**
- [ ] Transport implementation complete
- [ ] All 11 tools working
- [ ] Progress notifications functional
- [ ] Error handling comprehensive
- [ ] Tests passing (unit + integration)
- [ ] Documentation complete
- [ ] Example client provided

**Should Have:**
- [ ] Performance benchmarks
- [ ] Connection pooling (where applicable)
- [ ] Automatic reconnection
- [ ] Health checks
- [ ] Monitoring/metrics

**Nice to Have:**
- [ ] Load balancing
- [ ] Rate limiting
- [ ] Circuit breakers
- [ ] Distributed tracing

---

## 📝 Example Usage

### SSE Example
```javascript
// Browser client
const eventSource = new EventSource('http://localhost:8025/events');

eventSource.addEventListener('progress', (e) => {
  const data = JSON.parse(e.data);
  console.log(`Progress: ${data.progress * 100}%`);
});

// Execute tool
fetch('http://localhost:8025/tools/process_with_progress', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ items: 100, delay_ms: 50 })
});
```

### WebSocket Example
```javascript
// Browser client
const ws = new WebSocket('ws://localhost:8025/ws');

ws.onopen = () => {
  ws.send(JSON.stringify({
    jsonrpc: '2.0',
    id: 1,
    method: 'tools/call',
    params: {
      name: 'echo',
      arguments: { message: 'Hello WebSocket!' }
    }
  }));
};

ws.onmessage = (event) => {
  const response = JSON.parse(event.data);
  console.log('Response:', response);
};
```

### HTTP Streaming Example
```bash
# Curl with streaming
curl -N -X POST http://localhost:8025/stream/tools/transform_data \
  -H "Content-Type: application/json" \
  -d '{"data":["item1","item2"],"operation":"uppercase"}'

# Response streams back chunks
# {"index":0,"result":"ITEM1"}
# {"index":1,"result":"ITEM2"}
# {"complete":true,"total":2}
```

---

## 🔄 Migration Path

### From Current (v0.4.0-rc) to Multi-Transport (v0.5.0)

**No Breaking Changes for Existing Users:**
- stdio transport remains default
- All existing functionality preserved
- New transports opt-in via feature flags

**CLI Changes:**
```bash
# Old (still works)
./mcp-boilerplate-rust --mode stdio

# New (recommended)
./mcp-boilerplate-rust --transport stdio
./mcp-boilerplate-rust --transport sse
./mcp-boilerplate-rust --transport websocket
./mcp-boilerplate-rust --transport http-stream

# Multiple transports
./mcp-boilerplate-rust --transport stdio,sse,websocket
```

**Config Changes:**
```json
{
  "transports": {
    "stdio": { "enabled": true },
    "sse": { 
      "enabled": true,
      "port": 8025
    },
    "websocket": {
      "enabled": true,
      "port": 8026
    }
  }
}
```

---

## 📋 Pre-Session Checklist

Before starting next session, ensure:

- [ ] Current v0.4.0-rc merged to main
- [ ] All tests passing
- [ ] Documentation reviewed
- [ ] Dependencies identified
- [ ] Architecture planned
- [ ] This document reviewed

---

## 🎓 Learning Resources

### SSE (Server-Sent Events)
- [MDN: Server-Sent Events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events)
- [Axum SSE Example](https://github.com/tokio-rs/axum/tree/main/examples/sse)

### WebSocket
- [MDN: WebSocket API](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)
- [Tokio Tungstenite](https://github.com/snapview/tokio-tungstenite)

### HTTP Streaming
- [MDN: Transfer-Encoding](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Transfer-Encoding)
- [Hyper Streaming](https://hyper.rs/)

### gRPC
- [gRPC Documentation](https://grpc.io/docs/)
- [Tonic Tutorial](https://github.com/hyperium/tonic)

---

## 🚀 Quick Start for Next Session

```bash
# 1. Pull latest main
git checkout main
git pull origin main

# 2. Create feature branch
git checkout -b feature/multi-transport

# 3. Start with transport trait
# Create src/transport/trait.rs
# Define Transport trait

# 4. Refactor stdio
# Update src/transport/stdio.rs to implement Transport trait

# 5. Add SSE
# Create src/transport/sse.rs
# Create src/mcp/sse_server.rs

# 6. Test
cargo build --features sse
./scripts/test_sse.sh

# 7. Iterate on other transports
```

---

## 📊 Timeline Estimate

| Phase | Duration | Effort |
|-------|----------|--------|
| Phase 1: Transport Abstraction | 1 week | Medium |
| Phase 2: SSE Implementation | 1 week | Low |
| Phase 3: WebSocket Implementation | 1-2 weeks | Medium |
| Phase 4: HTTP Streaming | 1-2 weeks | Medium |
| Phase 5: RPC/gRPC (Optional) | 1-2 weeks | High |
| Phase 6: Documentation & Examples | 1-2 weeks | Medium |
| **Total** | **6-10 weeks** | **Medium-High** |

**Recommendation:** Start with Phases 1-3 (SSE + WebSocket) for v0.5.0, defer gRPC to v0.6.0

---

## 🎯 Session Goals Summary

### Primary Goals
1. ✅ Implement transport abstraction layer
2. ✅ Add SSE support with progress notifications
3. ✅ Add WebSocket support for real-time communication
4. ✅ Add HTTP streaming for large data transfers

### Secondary Goals
5. ⚠️ Add gRPC support (optional)
6. ✅ Comprehensive testing for all transports
7. ✅ Documentation and examples
8. ✅ Performance benchmarks

### Success Metrics
- All 11 tools work on all transports
- Progress notifications work across transports
- <10ms latency for WebSocket
- <100ms latency for SSE
- Memory usage <50MB per transport
- 100% test coverage for transport layer

---

**Current Status:** v0.4.0-rc (stdio + basic HTTP)  
**Target Status:** v0.5.0 (stdio + SSE + WebSocket + HTTP Streaming)  
**Timeline:** 6-10 weeks  
**Complexity:** Medium-High  

**Ready to begin!** 🚀

---

**Last Updated:** 2026-01-08  
**Maintained by:** NetAdx AI  
**License:** MIT