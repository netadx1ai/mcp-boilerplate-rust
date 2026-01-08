# MCP Boilerplate Rust - Quick Start Guide

**Version:** 0.3.1  
**Last Updated:** 2025-01-08

---

## Prerequisites

- Rust 1.70+ (`rustup`, `cargo`)
- macOS/Linux/Windows
- For HTTP mode: `curl` or Postman for testing

---

## Installation

```bash
# Clone repository
git clone https://github.com/netadx/mcp-boilerplate-rust.git
cd mcp-boilerplate-rust

# Build stdio mode (default)
cargo build --release

# Build HTTP mode
cargo build --release --features http
```

---

## Quick Test

### Stdio Mode (5 seconds)

```bash
./test_mcp.sh
```

Expected output:
```
✅ [1/4] Build successful
✅ [2/4] Initialize response valid
✅ [3/4] Tools list returns 3 tools
✅ [4/4] Echo tool call successful
```

### HTTP Mode (10 seconds)

```bash
./test_http.sh
```

Expected output:
```
✅ [1/5] Build with HTTP feature
✅ [2/5] Server started
✅ [3/5] Health check passed
✅ [4/5] Tools list passed
✅ [5/5] Echo tool passed
```

---

## Usage

### Stdio Mode (Claude Desktop)

**1. Build binary:**
```bash
cargo build --release
```

**2. Configure Claude Desktop:**

Edit `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "boilerplate": {
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

**4. Test in Claude:**
Ask: "What MCP tools are available?"

You should see: echo, ping, info

### HTTP Mode (REST API)

**1. Start server:**
```bash
./target/release/mcp-boilerplate-rust --mode http
```

**2. Test endpoints:**

```bash
# Health check
curl http://localhost:8025/health

# List tools
curl http://localhost:8025/tools

# Call echo tool
curl -X POST http://localhost:8025/tools/echo \
  -H "Content-Type: application/json" \
  -d '{"message":"Hello World!"}'

# Call ping tool
curl -X POST http://localhost:8025/tools/ping \
  -H "Content-Type: application/json" \
  -d '{}'

# Call info tool
curl -X POST http://localhost:8025/tools/info \
  -H "Content-Type: application/json" \
  -d '{}'
```

---

## Available Tools

| Tool | Description | Input | Output |
|------|-------------|-------|--------|
| `echo` | Echo back a message | `{"message": "text"}` | `{"message": "text", "timestamp": "..."}` |
| `ping` | Simple ping-pong test | `{}` | `{"response": "pong", "timestamp": "..."}` |
| `info` | Get server information | `{}` | `{"tool": "...", "version": "...", ...}` |

---

## Development

### Project Structure

```
mcp-boilerplate-rust/
├── src/
│   ├── main.rs              # Entry point
│   ├── mcp/
│   │   └── stdio_server.rs  # Stdio mode (McpServer)
│   ├── tools/
│   │   ├── shared.rs        # Shared types (NEW!)
│   │   └── echo.rs          # HTTP mode tools
│   ├── types.rs             # Common types
│   └── utils/               # Logger, config
├── test_mcp.sh              # Stdio tests
├── test_http.sh             # HTTP tests
└── Cargo.toml               # Dependencies
```

### Adding a New Tool

**1. Define types in `src/tools/shared.rs`:**

```rust
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct GreetRequest {
    pub name: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct GreetResponse {
    pub greeting: String,
    pub timestamp: String,
}

pub fn create_greet_response(name: String) -> GreetResponse {
    GreetResponse {
        greeting: format!("Hello, {}!", name),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}
```

**2. Add to stdio server (`src/mcp/stdio_server.rs`):**

```rust
#[tool(description = "Greet a person by name")]
async fn greet(
    &self,
    Parameters(req): Parameters<GreetRequest>,
) -> Result<Json<GreetResponse>, McpError> {
    info!("Greet: {}", req.name);
    Ok(Json(create_greet_response(req.name)))
}
```

**3. Add to HTTP tools (`src/tools/echo.rs`):**

```rust
#[tool(description = "Greet a person by name")]
pub async fn greet(
    &self,
    params: Parameters<GreetRequest>,
) -> Result<Json<GreetResponse>, McpError> {
    info!("Greet: {}", params.0.name);
    Ok(Json(create_greet_response(params.0.name)))
}
```

**4. Add HTTP endpoint (`src/main.rs`):**

```rust
.route("/tools/greet", post({
    let tool = Arc::clone(&echo_tool);
    move |payload| handle_greet_tool(tool, payload)
}))

// Add handler function
#[cfg(feature = "http")]
async fn handle_greet_tool(
    tool: Arc<EchoTool>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    match serde_json::from_value::<GreetRequest>(payload) {
        Ok(req) => {
            let params = rmcp::handler::server::wrapper::Parameters(req);
            match tool.greet(params).await {
                Ok(result) => (StatusCode::OK, Json(json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&result.0).unwrap()
                    }],
                    "is_error": false,
                }))).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                    "content": [{"type": "text", "text": e.message}],
                    "is_error": true,
                }))).into_response(),
            }
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({
            "content": [{"type": "text", "text": format!("Invalid request: {}", e)}],
            "is_error": true,
        }))).into_response(),
    }
}
```

**5. Test:**

```bash
# Build and test stdio
cargo build --release
./test_mcp.sh

# Build and test HTTP
cargo build --release --features http
./test_http.sh
```

---

## Environment Variables

```bash
# Logging level
export RUST_LOG=info,mcp_boilerplate_rust=debug

# HTTP server (when using --features http)
export HOST=0.0.0.0
export PORT=8025

# Optional features
export MONGODB_URI=mongodb://localhost:27017
export MONGODB_DATABASE=mcp
export JWT_SECRET=your-secret-key
```

---

## Build Options

```bash
# Stdio only (smallest binary)
cargo build --release

# HTTP mode
cargo build --release --features http

# All features
cargo build --release --features full

# Development (with debug symbols)
cargo build
```

---

## Testing

```bash
# Quick tests
./test_mcp.sh        # Stdio mode
./test_http.sh       # HTTP mode

# Cargo tests
cargo test

# Check compilation
cargo check                    # Stdio
cargo check --features http    # HTTP
cargo check --all-features     # All

# Format code
cargo fmt

# Lint
cargo clippy
```

---

## Troubleshooting

### Stdio Mode

**Issue:** Claude Desktop doesn't see the server

**Solution:**
1. Check config path: `~/Library/Application Support/Claude/claude_desktop_config.json`
2. Use absolute paths, not relative
3. Restart Claude Desktop completely
4. Check Claude logs

**Issue:** Tools not appearing

**Solution:**
1. Run `./test_mcp.sh` to verify server works
2. Check the binary has execute permissions
3. Verify JSON config syntax

### HTTP Mode

**Issue:** Port 8025 already in use

**Solution:**
```bash
# Find process using port
lsof -i :8025

# Kill it or use different port
export PORT=8026
```

**Issue:** CORS errors

**Solution:**
Server allows all origins by default. Check client-side configuration.

---

## Performance Tips

### Stdio Mode
- Binary size: 2.4MB (optimized)
- Startup: ~10ms
- Tool call: <5ms
- Memory: ~2MB

### HTTP Mode
- Binary size: 3.1MB (includes HTTP stack)
- Startup: ~15ms
- Request: <10ms
- Memory: ~3MB

### Optimization
```toml
# Already in Cargo.toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

---

## Common Commands

```bash
# Development
cargo watch -x 'run -- --mode stdio'     # Auto-rebuild stdio
cargo watch -x 'run --features http -- --mode http'  # Auto-rebuild HTTP

# Release builds
cargo build --release                     # Stdio
cargo build --release --features http    # HTTP

# Testing
./test_mcp.sh                            # Test stdio
./test_http.sh                           # Test HTTP
cargo test                               # Unit tests

# Cleanup
cargo clean                              # Remove build artifacts
rm -rf target/                           # Full cleanup
```

---

## Documentation

- `REFACTORING_COMPLETE.md` - Stdio implementation details
- `CLEANUP_HTTP_FIX_COMPLETE.md` - HTTP fix and cleanup
- `docs/NATIVE_STDIO_GUIDE.md` - Comprehensive stdio guide
- `README.md` - Project overview

---

## Support

- Issues: https://github.com/netadx/mcp-boilerplate-rust/issues
- Docs: https://modelcontextprotocol.io
- SDK: https://github.com/modelcontextprotocol/rust-sdk

---

## License

MIT License - See LICENSE file for details

---

**Ready to use!** Both stdio and HTTP modes are fully functional and production-ready.