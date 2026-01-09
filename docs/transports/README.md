# Transport Documentation

Complete guides for all 6 transport modes in MCP Boilerplate Rust.

**Version:** 0.5.0  
**Last Updated:** 2026-01-09 HCMC

---

## 📚 Transport Guides

### Quick Reference
**[QUICK_REFERENCE.md](QUICK_REFERENCE.md)** - API cheat sheet

- All 6 transports at a glance
- Port configurations
- Command-line usage
- Quick examples
- Performance comparison

### Complete Guide
**[GUIDE.md](GUIDE.md)** - Detailed transport guide

- Step-by-step setup for each transport
- Configuration options
- Client examples
- Best practices
- Troubleshooting

### Quick Start
**[QUICK_START.md](QUICK_START.md)** - Get started fast

- 5-minute setup for each transport
- Basic examples
- Common use cases
- Testing commands

### Advanced Features
**[ADVANCED.md](ADVANCED.md)** - Advanced patterns

- Connection pooling
- TLS/SSL configuration
- Performance optimization
- Production deployment
- Security best practices

---

## 🚀 Transport Modes

### 1. Stdio (Default)
**Best for:** Desktop apps, Claude Desktop, CLI tools  
**Port:** N/A  
**Latency:** 2ms

```bash
cargo run --release -- --mode stdio
```

### 2. SSE (Server-Sent Events)
**Best for:** Browser push notifications, live updates  
**Port:** 8025  
**Latency:** 15ms

```bash
cargo run --release --features sse -- --mode sse --bind 127.0.0.1:8025
```

### 3. WebSocket
**Best for:** Real-time bidirectional communication  
**Port:** 9001  
**Latency:** 8ms

```bash
cargo run --release --features websocket -- --mode websocket --bind 127.0.0.1:9001
```

### 4. HTTP
**Best for:** Standard REST APIs  
**Port:** 8080  
**Latency:** 20ms

```bash
cargo run --release --features http -- --mode http
```

### 5. HTTP Streaming
**Best for:** Large file transfers  
**Port:** 8026  
**Latency:** 12ms  
**Throughput:** 150 MB/s

```bash
cargo run --release --features http-stream -- --mode http-stream --bind 127.0.0.1:8026
```

### 6. gRPC
**Best for:** Microservices, high-performance APIs  
**Port:** 50051  
**Latency:** 4ms  
**Throughput:** 200 MB/s

```bash
cargo run --release --features grpc -- --mode grpc --bind 127.0.0.1:50051
```

---

## 📊 Performance Comparison

| Transport | Latency (P50) | Throughput | Memory | Best For |
|-----------|---------------|------------|--------|----------|
| Stdio | 2ms | High | <5MB | Desktop apps |
| gRPC | 4ms | 200 MB/s | <10MB | Microservices |
| WebSocket | 8ms | High | <8MB | Real-time chat |
| HTTP Stream | 12ms | 150 MB/s | <10MB | Large files |
| SSE | 15ms | Medium | <8MB | Browser push |
| HTTP | 20ms | Medium | <8MB | REST APIs |

---

## 🔧 Build Commands

```bash
# Minimal (Stdio only)
cargo build --release

# Web transports
cargo build --release --features "sse,websocket,http"

# High performance
cargo build --release --features "grpc,http-stream"

# Everything
cargo build --release --features full
```

---

## 🧪 Testing

```bash
# Test with MCP Inspector (Stdio)
npx @modelcontextprotocol/inspector ./target/release/mcp-boilerplate-rust --mode stdio

# Test SSE endpoint
curl -N http://127.0.0.1:8025/sse

# Test WebSocket (use browser client)
open ../examples/websocket_test_client.html

# Test HTTP
curl -X POST http://127.0.0.1:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```

---

## 📖 Related Documentation

- [../guides/TRANSPORT_GUIDE.md](../guides/TRANSPORT_GUIDE.md) - Main transport guide
- [../guides/TESTING_GUIDE.md](../guides/TESTING_GUIDE.md) - Testing strategies
- [../reference/API.md](../reference/API.md) - API reference
- [../../examples/](../../examples/) - Browser test clients

---

## 🎯 Quick Links by Use Case

### Desktop Applications
→ Use **Stdio** transport  
→ See [QUICK_START.md](QUICK_START.md#stdio)

### Web Applications
→ Use **SSE** or **WebSocket**  
→ See [GUIDE.md](GUIDE.md#web-transports)

### Microservices
→ Use **gRPC**  
→ See [ADVANCED.md](ADVANCED.md#grpc-production)

### File Transfers
→ Use **HTTP Streaming**  
→ See [GUIDE.md](GUIDE.md#http-streaming)

### Public APIs
→ Use **HTTP**  
→ See [QUICK_START.md](QUICK_START.md#http)

---

**Version:** 0.5.0  
**Maintained by:** NetADX Team