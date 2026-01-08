# Start Here - MCP Boilerplate Rust

**Quick start guide for the multi-transport MCP server**

## What is This?

A production-ready Rust implementation of the Model Context Protocol (MCP) with 6 transport modes:
- **stdio** - Desktop apps (Claude Desktop)
- **SSE** - Browser push notifications
- **WebSocket** - Real-time bidirectional
- **HTTP** - REST APIs
- **HTTP Streaming** - Large file transfers
- **gRPC** - High-performance microservices

## Quick Start (5 minutes)

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Build & Run

```bash
# Clone
git clone https://github.com/netadx/mcp-boilerplate-rust
cd mcp-boilerplate-rust

# Build (minimal)
cargo build --release

# Run
cargo run --release -- --mode stdio
```

### 3. Test with MCP Inspector

```bash
npx @modelcontextprotocol/inspector cargo run --release -- --mode stdio
```

Open http://localhost:5173 to interact with all 11 tools.

## Use with Claude Desktop

### Step 1: Build

```bash
cargo build --release
```

### Step 2: Configure

Edit `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "mcp-boilerplate-rust": {
      "command": "/absolute/path/to/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust",
      "args": ["--mode", "stdio"]
    }
  }
}
```

### Step 3: Restart Claude Desktop

The 11 tools will appear in Claude's interface.

## Available Tools

1. **ping** - Health check
2. **echo** - Message validation
3. **info** - Server metadata
4. **calculate** - Math operations (add, subtract, multiply, divide)
5. **evaluate** - Expression evaluation
6. **process_with_progress** - Data processing with progress notifications
7. **batch_process** - Batch operations with logging
8. **transform_data** - Array transformations
9. **simulate_upload** - File upload simulation
10. **health_check** - System health monitoring
11. **long_task** - Long-running operation demo

## Transport Modes

### Stdio (Default)
```bash
cargo run --release -- --mode stdio
```

### SSE (Browser Push)
```bash
cargo run --release --features sse -- --mode sse --bind 127.0.0.1:8025
# Test: open examples/sse_test_client.html
```

### WebSocket (Real-time)
```bash
cargo run --release --features websocket -- --mode websocket --bind 127.0.0.1:9001
# Test: open examples/websocket_test_client.html
```

### HTTP Streaming (Large Files)
```bash
cargo run --release --features http-stream -- --mode http-stream --bind 127.0.0.1:8026
# Test: curl -N http://127.0.0.1:8026/stream
```

### gRPC (High Performance)
```bash
cargo run --release --features grpc -- --mode grpc --bind 127.0.0.1:50051
# Test: grpcurl -plaintext 127.0.0.1:50051 list
```

### All Features
```bash
cargo build --release --features full
```

## Build Options

| Configuration | Size | Command |
|--------------|------|---------|
| Minimal (stdio) | 2.4 MB | `cargo build --release` |
| Web transports | 3.3 MB | `cargo build --release --features "sse,websocket"` |
| Streaming | 3.2 MB | `cargo build --release --features http-stream` |
| High performance | 3.9 MB | `cargo build --release --features grpc` |
| Everything | 4.2 MB | `cargo build --release --features full` |

## Testing

```bash
# All tests
cargo test --features full

# Integration tests
./scripts/integration_test.sh

# Specific transport
cargo test --features sse -- transport::sse
```

**Result:** 89 tests passing

## Next Steps

### For Developers
1. Read [README.md](README.md) for complete documentation
2. Check [docs/TRANSPORT_QUICK_REFERENCE.md](docs/TRANSPORT_QUICK_REFERENCE.md) for API reference
3. Explore [examples/](examples/) for browser clients
4. Review [docs/guides/TESTING_GUIDE.md](docs/guides/TESTING_GUIDE.md) for testing

### For Production
1. Choose your transport mode
2. Build with appropriate features
3. Configure environment variables
4. Deploy (Docker, systemd, or PM2)
5. Monitor logs and metrics

## Common Tasks

### Add a New Tool

1. Create `src/tools/my_tool.rs`
2. Implement the tool logic
3. Register in `src/mcp/protocol_handler.rs`
4. Add tests
5. Rebuild and test

### Change Default Port

```bash
cargo run --release --features sse -- --mode sse --bind 127.0.0.1:9999
```

### Enable Debug Logging

```bash
RUST_LOG=debug cargo run --release --features sse -- --mode sse
```

### Deploy with Docker

```bash
docker build -t mcp-server .
docker run -p 8025:8025 mcp-server
```

## Troubleshooting

### Port Already in Use
```bash
lsof -i :8025
kill -9 <PID>
```

### Build Errors
```bash
cargo clean
cargo build --release --features full
```

### Claude Desktop Not Seeing Tools
1. Check config file path
2. Use absolute path to binary
3. Restart Claude Desktop
4. Check logs: `~/Library/Logs/Claude/mcp*.log`

## Documentation

- **[README.md](README.md)** - Complete project documentation
- **[CHANGELOG.md](CHANGELOG.md)** - Version history
- **[docs/TRANSPORT_QUICK_REFERENCE.md](docs/TRANSPORT_QUICK_REFERENCE.md)** - API reference
- **[docs/TRANSPORT_QUICK_GUIDE.md](docs/TRANSPORT_QUICK_GUIDE.md)** - Detailed transport guide
- **[docs/guides/TESTING_GUIDE.md](docs/guides/TESTING_GUIDE.md)** - Testing guide

## Support

- **GitHub:** https://github.com/netadx/mcp-boilerplate-rust
- **Email:** hello@netadx.ai
- **Website:** https://netadx.ai

## License

MIT License - see [LICENSE](LICENSE)

---

**You're ready to go!** 🚀

Choose your transport mode, build, and start developing with MCP.