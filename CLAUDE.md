# CLAUDE.md

This file provides guidance to Claude (AI assistant) when working with code in this repository.

## Project Overview

**MCP Boilerplate Rust** is a production-ready Model Context Protocol (MCP) server implementation in Rust with 6 transport modes. This is a reference implementation and starting template for Rust-based MCP servers.

**Version:** 0.4.0  
**Protocol:** MCP 2025-03-26  
**SDK:** rmcp v0.12.0 (official Rust SDK)  
**Status:** Production ready

## Quick Reference

### Build & Test Commands

```bash
# Build minimal (stdio only, 2.4MB binary)
cargo build --release

# Build with all features (4.2MB binary)
cargo build --release --features full

# Run specific transport modes
cargo run --release -- --mode stdio
cargo run --release --features sse -- --mode sse --bind 127.0.0.1:8025
cargo run --release --features websocket -- --mode websocket --bind 127.0.0.1:9001
cargo run --release --features http-stream -- --mode http-stream --bind 127.0.0.1:8026
cargo run --release --features grpc -- --mode grpc --bind 127.0.0.1:50051

# Run all tests
cargo test --features full

# Code quality
cargo clippy --release --all-features
cargo fmt
cargo audit

# Integration tests
./scripts/integration_test.sh
```

### Project Structure

```
mcp-boilerplate-rust/
├── src/
│   ├── main.rs                     # Entry point with CLI args
│   ├── mcp/
│   │   ├── mod.rs                  # MCP module exports
│   │   ├── protocol_handler.rs     # Shared protocol logic (969 lines)
│   │   ├── stdio_server.rs         # Stdio transport server
│   │   ├── sse_server.rs           # SSE transport server (573 lines)
│   │   ├── websocket_server.rs     # WebSocket transport server (395 lines)
│   │   ├── http_stream_server.rs   # HTTP streaming server (397 lines)
│   │   └── grpc_server.rs          # gRPC server (317 lines)
│   ├── transport/
│   │   ├── trait.rs                # Transport trait definition
│   │   ├── stdio.rs                # Stdio transport
│   │   ├── sse.rs                  # SSE transport (435 lines)
│   │   ├── websocket.rs            # WebSocket transport (398 lines)
│   │   ├── http_stream.rs          # HTTP streaming (358 lines)
│   │   └── grpc.rs                 # gRPC transport (358 lines)
│   ├── tools/                      # 11 tool implementations
│   │   ├── mod.rs                  # Tool registry
│   │   ├── shared.rs               # Shared types
│   │   ├── echo.rs                 # Basic tools (echo, ping, info)
│   │   ├── calculator.rs           # Math tools (calculate, evaluate)
│   │   └── advanced.rs             # Advanced tools (6 tools)
│   ├── prompts/                    # Prompt templates
│   ├── resources/                  # Resource providers
│   ├── middleware/
│   │   └── auth.rs                 # JWT authentication
│   └── utils/
│       ├── types.rs                # Error types (McpError)
│       └── logger.rs               # Logging
├── proto/
│   └── mcp.proto                   # gRPC service definition (158 lines)
├── scripts/                        # Test scripts
├── examples/                       # Browser test clients
│   ├── sse_test_client.html        # SSE browser client (684 lines)
│   └── websocket_test_client.html  # WebSocket client (747 lines)
└── docs/                           # Documentation

Target binaries:
  - stdio only: target/release/mcp-boilerplate-rust (2.4MB)
  - full features: target/release/mcp-boilerplate-rust (4.2MB)
```

## Code Style & Architecture

### Rust Guidelines

- **Edition:** Rust 2021
- **Style:** Follow standard Rust conventions
- **Async:** Tokio runtime for async operations
- **Error Handling:** Custom `McpError` enum with `thiserror`
- **Serialization:** `serde` and `serde_json` for JSON
- **Validation:** Input validation using `schemars`
- **Logging:** `tracing` with ANSI disabled in stdio mode

### Key Patterns

**1. Tool Implementation Pattern (v0.4.0):**
```rust
// src/tools/example.rs
use crate::tools::shared::{ToolInput, ToolOutput};
use crate::utils::types::McpResult;
use rmcp::RequestContext;
use rmcp::role::server::RoleServer;
use serde_json::json;

pub async fn example_tool(
    params: Parameters<Request>,
    ctx: RequestContext<RoleServer>,  // Required in v0.4.0
) -> Result<Json<Response>, McpError> {
    // Extract input
    let message = params.arguments.get("message")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams("Missing message".to_string()))?;
    
    // Validate input
    if message.is_empty() {
        return Err(McpError::InvalidParams("Message cannot be empty".to_string()));
    }
    
    // Send progress notification (optional)
    ctx.notify(Progress {
        progress_token: params.progress_token,
        progress: 50,
        total: Some(100),
    }).await?;
    
    // Execute logic
    let result = json!({
        "data": message,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    // Return response
    Ok(Json(Response {
        content: vec![TextContent { text: result.to_string() }],
        is_error: Some(false),
    }))
}
```

**2. Error Handling Pattern:**
```rust
use crate::utils::types::{McpError, McpResult};

// Use ? operator for propagation
let value = some_operation()
    .map_err(|e| McpError::InvalidParams(e.to_string()))?;

// Return descriptive errors
if input.is_empty() {
    return Err(McpError::InvalidParams("Input cannot be empty".to_string()));
}

// Validate ranges
if value > MAX_VALUE {
    return Err(McpError::InvalidParams(
        format!("Value {} exceeds maximum {}", value, MAX_VALUE)
    ));
}
```

**3. Stdio Mode - No Logging:**
```rust
// In main.rs - logging disabled for stdio to prevent JSON interference
match args.mode {
    ServerMode::Stdio => {
        std::env::set_var("RUST_LOG", "off");  // Critical!
        Logger::init();
        run_stdio_server().await?;
    }
    ServerMode::Sse | ServerMode::WebSocket | ServerMode::HttpStream | ServerMode::Grpc => {
        std::env::set_var("RUST_LOG", "debug");
        Logger::init();
        // Run appropriate server
    }
}
```

**4. Progress Notifications Pattern:**
```rust
use rmcp::{RequestContext, ProgressNotificationParam};
use rmcp::role::server::RoleServer;

pub async fn long_running_tool(
    params: Parameters<Request>,
    ctx: RequestContext<RoleServer>,
) -> Result<Json<Response>, McpError> {
    let total_steps = 10;
    
    for step in 0..total_steps {
        // Do work
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Send progress
        if let Some(token) = &params.progress_token {
            ctx.notify(ProgressNotificationParam {
                progress_token: token.clone(),
                progress: step + 1,
                total: Some(total_steps),
            }).await.ok(); // Ignore notification errors
        }
    }
    
    Ok(Json(Response { /* ... */ }))
}
```

**5. Input Validation Pattern:**
```rust
// Validate required fields
let message = params.arguments.get("message")
    .and_then(|v| v.as_str())
    .ok_or_else(|| McpError::InvalidParams("Missing required field: message".to_string()))?;

// Validate length
if message.is_empty() {
    return Err(McpError::InvalidParams("Message cannot be empty".to_string()));
}
if message.len() > MAX_MESSAGE_SIZE {
    return Err(McpError::InvalidParams(
        format!("Message exceeds maximum length of {} bytes", MAX_MESSAGE_SIZE)
    ));
}

// Validate numeric ranges
let count = params.arguments.get("count")
    .and_then(|v| v.as_u64())
    .unwrap_or(1);
if count > 1000 {
    return Err(McpError::InvalidParams("Count cannot exceed 1000".to_string()));
}
```

## Transport Modes (6 Total)

### 1. Stdio (Default)
**Best for:** Desktop apps, Claude Desktop, CLI tools  
**Port:** N/A  
**Performance:** 2ms latency  
**Build:** `cargo build --release`  
**Run:** `cargo run --release -- --mode stdio`

### 2. SSE (Server-Sent Events)
**Best for:** Browser push notifications, real-time updates  
**Port:** 8025 (default)  
**Performance:** 15ms latency  
**Build:** `cargo build --release --features sse`  
**Run:** `cargo run --release --features sse -- --mode sse --bind 127.0.0.1:8025`

### 3. WebSocket
**Best for:** Real-time bidirectional communication, chat apps  
**Port:** 9001 (default)  
**Performance:** 8ms latency  
**Build:** `cargo build --release --features websocket`  
**Run:** `cargo run --release --features websocket -- --mode websocket --bind 127.0.0.1:9001`

### 4. HTTP Streaming
**Best for:** Large file transfers, progressive data delivery  
**Port:** 8026 (default)  
**Performance:** 12ms latency, 150 MB/s throughput  
**Build:** `cargo build --release --features http-stream`  
**Run:** `cargo run --release --features http-stream -- --mode http-stream --bind 127.0.0.1:8026`

### 5. gRPC
**Best for:** Microservices, high-performance APIs, internal services  
**Port:** 50051 (default)  
**Performance:** 4ms latency, 200 MB/s throughput  
**Build:** `cargo build --release --features grpc`  
**Run:** `cargo run --release --features grpc -- --mode grpc --bind 127.0.0.1:50051`

### 6. HTTP (REST API)
**Best for:** Standard REST APIs, public APIs  
**Port:** 8080 (default)  
**Performance:** 20ms latency  
**Build:** `cargo build --release --features http`  
**Run:** `cargo run --release --features http -- --mode http`

## Available Tools (11 Total)

### Basic Tools (5)

**1. echo**
- **Purpose:** Message validation and timestamping
- **Input:** `message` (string, 1-10,240 bytes, non-empty)
- **Output:** `{ message, timestamp }`
- **File:** `src/tools/echo.rs`

**2. ping**
- **Purpose:** Health check and connectivity test
- **Input:** None
- **Output:** `{ response: "pong", timestamp }`
- **File:** `src/tools/echo.rs`

**3. info**
- **Purpose:** Server metadata
- **Input:** None
- **Output:** `{ tool, version, description }`
- **File:** `src/tools/echo.rs`

**4. calculate**
- **Purpose:** Arithmetic operations
- **Input:** `operation` (add/subtract/multiply/divide), `a`, `b` (numbers)
- **Output:** `{ result, operation, a, b }`
- **File:** `src/tools/calculator.rs`

**5. evaluate**
- **Purpose:** Math expression evaluation
- **Input:** `expression` (string, e.g., "2 * (3 + 4)")
- **Output:** `{ result, expression }`
- **File:** `src/tools/calculator.rs`

### Advanced Tools (6)

**6. process_with_progress**
- **Purpose:** Data processing with progress notifications
- **Input:** `data` (array), `delay_ms` (number, optional)
- **Output:** `{ processed_count, items, duration_ms }`
- **Features:** 10 progress updates
- **File:** `src/tools/advanced.rs`

**7. batch_process**
- **Purpose:** Batch operations with logging
- **Input:** `items` (array), `operation` (string)
- **Output:** `{ processed_count, results }`
- **Features:** Progress + logging notifications
- **File:** `src/tools/advanced.rs`

**8. transform_data**
- **Purpose:** Array transformation
- **Input:** `data` (array, max 10K items), `operation` (uppercase/lowercase/reverse/double)
- **Output:** `{ original_count, transformed_count, operation, result }`
- **File:** `src/tools/advanced.rs`

**9. simulate_upload**
- **Purpose:** File upload simulation
- **Input:** `filename` (string), `size_bytes` (number)
- **Output:** `{ filename, size_bytes, chunks, duration_ms }`
- **Features:** 20 chunks with progress
- **File:** `src/tools/advanced.rs`

**10. health_check**
- **Purpose:** System health monitoring
- **Input:** None
- **Output:** `{ status, version, uptime, features, timestamp }`
- **File:** `src/tools/advanced.rs`

**11. long_task**
- **Purpose:** Long-running operation demo
- **Input:** `duration_seconds` (number, 1-60, default 10)
- **Output:** `{ completed, duration_seconds, steps }`
- **Features:** Step-by-step progress updates
- **File:** `src/tools/advanced.rs`

## Adding New Tools

### Step 1: Define Tool Function

Create or update file in `src/tools/`:
```rust
use rmcp::{RequestContext, Parameters, Request, Response, TextContent, Json};
use rmcp::role::server::RoleServer;
use crate::utils::types::McpError;
use serde_json::json;

pub async fn my_new_tool(
    params: Parameters<Request>,
    ctx: RequestContext<RoleServer>,
) -> Result<Json<Response>, McpError> {
    // Extract parameters
    let param = params.arguments.get("param")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams("Missing param".to_string()))?;
    
    // Validate
    if param.is_empty() {
        return Err(McpError::InvalidParams("Param cannot be empty".to_string()));
    }
    
    // Execute logic
    let result = json!({
        "data": param,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    // Return response
    Ok(Json(Response {
        content: vec![TextContent { 
            text: serde_json::to_string_pretty(&result).unwrap() 
        }],
        is_error: Some(false),
    }))
}
```

### Step 2: Register in ProtocolHandler

Add to `src/mcp/protocol_handler.rs`:
```rust
// In list_tools() method
tools.push(Tool {
    name: "my_new_tool".to_string(),
    description: Some("Description of my tool".to_string()),
    input_schema: json!({
        "type": "object",
        "properties": {
            "param": {
                "type": "string",
                "description": "Parameter description"
            }
        },
        "required": ["param"]
    }),
});

// In call_tool() method
"my_new_tool" => my_new_tool(params, ctx).await,
```

### Step 3: Export and Test

Update `src/tools/mod.rs`:
```rust
pub use my_file::my_new_tool;
```

Test:
```bash
cargo build --release
cargo test --features full
./scripts/integration_test.sh
```

## Critical Implementation Details

### Stdio Mode Requirements

**MUST NOT output to stdout/stderr:**
- Logging disabled with `RUST_LOG=off`
- ANSI colors disabled with `.with_ansi(false)`
- Only pure JSON-RPC messages on stdout

**Why:** Claude Desktop and MCP clients parse stdout as JSON. Any non-JSON output (logs, debug messages, ANSI codes) breaks the protocol.

### RequestContext Usage (v0.4.0)

All tools in v0.4.0 require `RequestContext<RoleServer>` parameter:
- Use `_ctx` if not needed
- Use `ctx.notify()` for progress/logging notifications
- Never panic or unwrap in notification sends (use `.ok()`)

### Validation Rules

**General:**
- Validate all required parameters
- Provide descriptive error messages
- Use type-safe extraction
- Check ranges and limits

**Echo Tool:**
- Min length: 1 byte
- Max length: 10,240 bytes (10KB)
- Non-empty validation
- UTF-8 encoding required

**Transform Data Tool:**
- Max items: 10,000
- Valid operations: uppercase, lowercase, reverse, double

**All Tools:**
- Stateless execution
- No data persistence
- No file system access
- No external network calls

## Performance Expectations

| Metric | Stdio | SSE | WebSocket | HTTP Stream | gRPC | HTTP |
|--------|-------|-----|-----------|-------------|------|------|
| Latency (P50) | 2ms | 15ms | 8ms | 12ms | 4ms | 20ms |
| Throughput | High | Medium | High | 150 MB/s | 200 MB/s | Medium |
| Memory | <5MB | <8MB | <8MB | <10MB | <10MB | <8MB |
| CPU (idle) | <1% | <1% | <1% | <1% | <1% | <1% |
| Binary Size | 2.4MB | 3.3MB | 3.3MB | 3.2MB | 3.9MB | 3.1MB |

## Common Issues & Solutions

### Issue: JSON Parse Errors in Claude Desktop

**Symptom:** `Unexpected token '\x1B'` errors  
**Cause:** Logging output interfering with JSON  
**Solution:** Ensure `RUST_LOG=off` in stdio mode (already configured in main.rs)

### Issue: Tools Not Appearing in Claude Desktop

**Check:**
1. Binary built in release mode: `cargo build --release`
2. Correct absolute path in config
3. `--mode stdio` argument present
4. Claude Desktop restarted
5. Config file syntax valid JSON

### Issue: Build Warnings

**Solution:**
```bash
cargo clippy --fix --allow-dirty
cargo fmt
```

### Issue: Port Already in Use

**Solution:**
```bash
# Find process
lsof -i :8025
kill -9 <PID>

# Or use different port
cargo run --features sse -- --mode sse --bind 127.0.0.1:9999
```

## Testing Guidelines

### Pre-Commit Checklist

- [ ] `cargo build --release --features full` succeeds
- [ ] `cargo clippy --all-features` has zero warnings
- [ ] `cargo fmt` applied
- [ ] `cargo test --features full` passes (89 tests)
- [ ] Input validation works
- [ ] Error messages are descriptive
- [ ] No emojis or icons in code/docs

### Integration Testing

```bash
# Run full test suite
./scripts/integration_test.sh

# Should verify:
# - Binary exists and is optimized
# - Stdio communication works
# - SSE server starts and responds
# - WebSocket server runs
# - All tools operational
```

### Manual Testing

```bash
# Test with MCP Inspector
npx @modelcontextprotocol/inspector cargo run --release -- --mode stdio

# Test SSE endpoint
curl -N http://127.0.0.1:8025/sse

# Test WebSocket (use browser client)
open examples/websocket_test_client.html
```

## Documentation

- **Main README:** Project overview, quick start
- **START_HERE.md:** 5-minute setup guide
- **CHANGELOG.md:** Version history
- **PROJECT_STATUS.md:** Current project status
- **docs/TRANSPORT_QUICK_REFERENCE.md:** Transport API reference
- **docs/guides/TESTING_GUIDE.md:** Testing guide
- **examples/:** Browser test clients

## Version Information

- **Current:** v0.4.0 (stable)
- **MCP Protocol:** 2025-03-26
- **SDK:** rmcp v0.12.0
- **Rust:** 1.75.0 or later

## Security Notes

- Input validation on all tools
- No file system access
- No external network calls
- No code execution
- Stateless operation
- See `docs/reference/SECURITY.md` for production checklist

## Feature Flags

- `sse` - Server-Sent Events transport
- `websocket` - WebSocket transport
- `http-stream` - HTTP streaming transport
- `grpc` - gRPC transport
- `http` - HTTP REST API transport
- `database` - MongoDB integration (future)
- `auth` - JWT authentication (future)
- `full` - All features

## Contact

- **Repository:** https://github.com/netadx/mcp-boilerplate-rust
- **Website:** https://netadx.ai
- **Email:** hello@netadx.ai

---

**Last Updated:** 2026-01-09  
**For:** Claude AI Assistant  
**Purpose:** Code assistance and development guidance