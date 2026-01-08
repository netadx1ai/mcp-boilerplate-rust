# CLAUDE.md

This file provides guidance to Claude (AI assistant) when working with code in this repository.

## Project Overview

**MCP Boilerplate Rust** is a production-ready Model Context Protocol (MCP) server implementation in Rust with dual transport support (stdio + HTTP). This is a reference implementation and starting template for Rust-based MCP servers.

**Version:** 0.3.1+  
**Protocol:** MCP 2025-03-26  
**SDK:** rmcp v0.12.0 (local development build)  
**Status:** Production ready with advanced features

## Quick Reference

### Build & Test Commands

```bash
# Build stdio mode (2.4MB binary)
cargo build --release

# Build HTTP mode (3.1MB binary)
cargo build --release --features http

# Run stdio server
./target/release/mcp-boilerplate-rust --mode stdio

# Run HTTP server
./target/release/mcp-boilerplate-rust --mode http

# Run all tests
./scripts/test_mcp.sh           # Stdio tests
./scripts/test_http.sh          # HTTP tests
./scripts/test_validation.sh    # Validation tests
./scripts/verify_claude_ready.sh # Pre-flight checks (10 checks)

# Code quality
cargo clippy --release --all-features
cargo fmt
cargo audit
```

### Project Structure

```
mcp-boilerplate-rust/
├── src/
│   ├── main.rs              # Entry point with CLI args
│   ├── mcp/
│   │   ├── mod.rs          # MCP module exports
│   │   └── stdio_server.rs # Stdio transport implementation
│   ├── tools/
│   │   ├── mod.rs          # Tool registry
│   │   ├── shared.rs       # Shared types (EchoRequest, etc.)
│   │   ├── echo.rs         # Echo tool with validation
│   │   ├── calculator.rs   # Math operations
│   │   └── advanced.rs     # Advanced features (progress, batch, etc.)
│   ├── transport/
│   │   ├── mod.rs          # Transport module
│   │   └── stdio.rs        # Stdio transport
│   ├── middleware/
│   │   └── auth.rs         # JWT authentication
│   └── utils/
│       ├── types.rs        # Error types (McpError)
│       └── logger.rs       # Logging (ANSI disabled for stdio)
├── scripts/                # Test scripts
├── examples/              # Config templates
└── docs/                  # Documentation

Target binary: target/release/mcp-boilerplate-rust
```

## Code Style & Architecture

### Rust Guidelines

- **Edition:** Rust 2021
- **Style:** Follow standard Rust conventions
- **Async:** Tokio runtime for async operations
- **Error Handling:** Custom `McpError` enum with `thiserror`
- **Serialization:** `serde` and `serde_json` for JSON
- **Validation:** Input validation on all tool parameters
- **Logging:** `tracing` with ANSI disabled in stdio mode
- **Tasks:** SEP-1686 task lifecycle support for long-running operations
- **Progress:** Real-time progress notifications during tool execution

### Key Patterns

**1. Tool Implementation Pattern (Modern with RequestContext):**
```rust
// src/tools/example.rs
use rmcp::{
    handler::server::wrapper::{Json, Parameters},
    service::RequestContext,
    ErrorData as McpError, RoleServer,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ExampleRequest {
    pub message: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ExampleResponse {
    pub result: String,
    pub timestamp: String,
}

pub struct ExampleTool;

impl ExampleTool {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(
        params: Parameters<ExampleRequest>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<Json<ExampleResponse>, McpError> {
        // Access peer for notifications
        let peer = ctx.peer.clone();
        
        // Validate input
        if params.0.message.is_empty() {
            return Err(McpError::invalid_params(
                "Message cannot be empty",
                None,
            ));
        }
        
        // Send progress notification
        let _ = peer.notify_progress(ProgressNotificationParam {
            progress_token: ProgressToken::String("example".into()),
            progress: 50.0,
            total: Some(100.0),
        }).await;
        
        // Process and return
        Ok(Json(ExampleResponse {
            result: params.0.message,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }))
    }
}
        let param = input.get_string("param")?;
        
        // Execute logic
        let result = json!({
            "data": param,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        Ok(ToolOutput::json(result))
    }
}
```

**2. Error Handling Pattern:**
```rust
use crate::utils::types::{McpError, McpResult};

// Use ? operator for propagation
let value = some_operation().map_err(|e| McpError::InvalidParams(e.to_string()))?;

// Return descriptive errors
Err(McpError::InvalidParams("Message cannot be empty".to_string()))
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
    ServerMode::Http => {
        std::env::set_var("RUST_LOG", "debug");
        Logger::init();
        run_http_server().await?;
    }
}
```

**4. Input Validation Pattern:**
```rust
// Validate message length and content
if message.is_empty() {
    return Err(McpError::InvalidParams("Message cannot be empty".to_string()));
}
if message.len() > MAX_MESSAGE_SIZE {
    return Err(McpError::InvalidParams(
        format!("Message exceeds maximum length of {} bytes", MAX_MESSAGE_SIZE)
    ));
}
```

## Available Tools

### Basic Tools

**1. echo** - Message validation and timestamping  
- Input: `message` (string, 1-10,240 bytes, non-empty)  
- Output: `{ message, timestamp }`  
- File: `src/tools/echo.rs`

**2. ping** - Health check and connectivity test  
- Input: None  
- Output: `{ response: "pong", timestamp }`

**3. info** - Server metadata  
- Input: None  
- Output: `{ tool, version, description }`

### Calculator Tools

**4. calculate** - Basic arithmetic operations  
- Input: `a` (number), `b` (number), `operation` (string)  
- Operations: add, subtract, multiply, divide, modulo, power  
- Output: `{ operation, a, b, result, timestamp }`

**5. evaluate** - Mathematical expression evaluation  
- Input: `expression` (string, max 1000 chars)  
- Supports: +, -, *, /, parentheses  
- Output: `{ expression, result, timestamp }`

### Advanced Tools (New!)

**6. long_task** - Long-running operation with progress  
- Duration: ~10 seconds  
- Progress: 10 notifications  
- Supports: Task lifecycle (SEP-1686)

**7. process_with_progress** - Data processing with real-time progress  
- Input: `items` (1-1000), `delay_ms` (optional)  
- Progress: Every 10 items  
- Output: `{ items_processed, total_time_ms, timestamp }`

**8. batch_process** - Batch processing with status updates  
- Input: `batch_size`, `total_batches`  
- Notifications: Progress + logging per batch  
- Output: `{ batches_completed, items_processed, status, timestamp }`

**9. transform_data** - Array data transformation  
- Input: `data` (array), `operation` (uppercase/lowercase/reverse/double)  
- Max items: 10,000  
- Progress: Every 100 items  
- Output: `{ original_count, transformed_count, operation, result, timestamp }`

**10. simulate_upload** - File upload simulation  
- Chunks: 20  
- Progress: Per chunk  
- Demonstrates: Progress + logging notifications

**11. health_check** - System health information  
- Output: `{ status, uptime_seconds, memory_mb, timestamp, version }`

### 4. calculate
**Purpose:** Basic arithmetic operations  
**Input:** `a` (number), `b` (number), `operation` (add/subtract/multiply/divide/modulo/power)  
**Output:** `{ operation, a, b, result, timestamp }`  
**Error Handling:** Returns helpful error messages for division by zero, invalid operations  
**File:** `src/tools/calculator.rs`

### 5. evaluate
**Purpose:** Mathematical expression evaluator  
**Input:** `expression` (string, mathematical expression with +, -, *, /, parentheses)  
**Output:** `{ expression, result, timestamp }`  
**Error Handling:** Returns detailed error messages for parsing failures  
**File:** `src/tools/calculator.rs`

## Adding New Tools

### Step 1: Define Request/Response Types

Create types in `src/tools/my_tool.rs`:
```rust
use rmcp::{
    handler::server::wrapper::{Json, Parameters},
    service::RequestContext,
    ErrorData as McpError, RoleServer,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MyToolRequest {
    pub param: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MyToolResponse {
    pub result: String,
    pub timestamp: String,
}

pub struct MyTool;

impl MyTool {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(
        params: Parameters<MyToolRequest>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<Json<MyToolResponse>, McpError> {
        // Validate
        if params.0.param.is_empty() {
            return Err(McpError::invalid_params(
                "Param cannot be empty",
                None,
            ));
        }
        
        // Optional: Send progress
        let _ = ctx.peer.notify_progress(ProgressNotificationParam {
            progress_token: ProgressToken::String("my_tool".into()),
            progress: 50.0,
            total: Some(100.0),
        }).await;
        
        Ok(Json(MyToolResponse {
            result: params.0.param,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }))
    }
}
```

### Step 2: Register in McpServer

Add to `src/mcp/stdio_server.rs`:
```rust
use crate::tools::my_tool::*;

#[tool_router]
impl McpServer {
    // ... existing tools ...
    
    #[tool(description = "My custom tool")]
    async fn my_tool(
        &self,
        params: Parameters<MyToolRequest>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<Json<MyToolResponse>, McpError> {
        MyTool::execute(params, ctx).await
    }
}
```

### Step 3: Export Module

Add to `src/tools/mod.rs`:
```rust
pub mod my_tool;
```

### Step 4: Test

```bash
cargo build --release
./scripts/test_mcp.sh

# Or test with MCP Inspector
npx @modelcontextprotocol/inspector cargo run --release
```

## Advanced Features

### 1. Task Lifecycle (SEP-1686)

Long-running operations support queuing, polling, and cancellation:

```rust
#[tool(description = "Long running task")]
async fn long_task(
    &self,
    ctx: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    // Task automatically queued if client sends task metadata
    tokio::time::sleep(Duration::from_secs(60)).await;
    Ok(CallToolResult::success(vec![Content::text("Done")]))
}
```

Enable in server:
```rust
#[task_handler(processor = self.processor.clone())]
impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tasks()  // ← Enable task support
                .build(),
            ..Default::default()
        }
    }
}
```

### 2. Progress Notifications

Send real-time progress updates:

```rust
async fn process_data(
    params: Parameters<Request>,
    ctx: RequestContext<RoleServer>,
) -> Result<Json<Response>, McpError> {
    let peer = ctx.peer.clone();
    
    for i in 0..100 {
        // Process item
        
        // Send progress
        let _ = peer.notify_progress(ProgressNotificationParam {
            progress_token: ProgressToken::String("process".into()),
            progress: i as f64,
            total: Some(100.0),
        }).await;
    }
    
    Ok(Json(response))
}
```

### 3. Logging Notifications

Send structured logs to client:

```rust
let _ = peer.notify_logging_message(LoggingMessageNotificationParam {
    level: LoggingLevel::Info,
    logger: Some("my_tool".into()),
    data: serde_json::json!({
        "status": "processing",
        "items": 100
    }),
}).await;
```

### 4. RequestContext Usage

Access peer and transport metadata:

```rust
async fn my_tool(
    params: Parameters<Request>,
    ctx: RequestContext<RoleServer>,
) -> Result<Json<Response>, McpError> {
    // Access peer for notifications
    let peer = ctx.peer.clone();
    
    // Access HTTP headers (if HTTP transport)
    if let Some(parts) = ctx.extensions.get::<axum::http::request::Parts>() {
        let auth = parts.headers.get("authorization");
        tracing::info!(?auth, "Auth header");
    }
    
    Ok(Json(response))
}
```

## Critical Implementation Details

### Stdio Mode Requirements

**MUST NOT output to stdout/stderr:**
- Logging disabled with `RUST_LOG=off`
- ANSI colors disabled with `.with_ansi(false)`
- Only pure JSON-RPC messages on stdout

**Why:** Claude Desktop and MCP clients parse stdout as JSON. Any non-JSON output (logs, debug messages, ANSI codes) breaks the protocol.

### HTTP Mode Configuration

**Port:** 8025 (default)  
**Endpoints:**
- `/health` - Health check
- `/tools` - List tools with parameters field (wrapper compatibility)
- `/tools/{name}` - Tool metadata

**Parameters Field:** Both `input_schema` and `parameters` included for wrapper compatibility.

### Validation Rules

**Echo Tool:**
- Min length: 1 byte
- Max length: 10,240 bytes (10KB)
- Non-empty validation
- UTF-8 encoding required

**All Tools:**
- Type validation via JSON schema
- Descriptive error messages
- No data persistence
- Stateless execution

## Performance Expectations

| Metric | Stdio | HTTP | Notes |
|--------|-------|------|-------|
| Response time | 2-7ms | 8-12ms | Average (basic tools) |
| Memory | <5MB | <8MB | Typical |
| CPU | <1% | <2% | Idle |
| Binary size | 2.4MB | 3.1MB | Release |
| Task overhead | +10ms | +15ms | Task lifecycle |
| Progress notifications | ~1ms each | ~2ms each | Per notification |

### Tool-Specific Performance

| Tool | Items | Time | Throughput |
|------|-------|------|------------|
| echo | 1 | ~3ms | N/A |
| process_with_progress | 100 | ~500ms | 200/s |
| batch_process | 10 batches | ~2s | 5/s |
| transform_data | 1,000 | ~50ms | 20,000/s |
| long_task | N/A | ~10s | Task queued |

## Common Issues & Solutions

### Issue: JSON Parse Errors in Claude Desktop

**Symptom:** `Unexpected token '\x1B'` errors  
**Cause:** Logging output interfering with JSON  
**Solution:** Ensure `RUST_LOG=off` in stdio mode

**Issue: Tools Not Appearing**

**Check:**
1. Binary built in release mode
2. Correct path in config
3. `--mode stdio` argument present
4. Claude Desktop restarted
5. Protocol version compatibility (2025-03-26)

### Issue: Build Warnings

**Solution:**
```bash
cargo clippy --fix --allow-dirty
cargo fmt
```

## Testing Guidelines

### Pre-Commit Checklist

- [ ] `cargo build --release` succeeds
- [ ] `cargo clippy` has zero warnings
- [ ] `cargo fmt` applied
- [ ] All test scripts pass
- [ ] Input validation works
- [ ] Error messages are descriptive

### Integration Testing

```bash
# Run full test suite
./scripts/verify_claude_ready.sh

# Should see:
# ✓ 10/10 checks passing
# ✓ Binary exists
# ✓ Stdio communication works
# ✓ All tools operational
```

## Documentation

- **Main README:** Project overview, quick start
- **SECURITY.md:** Security guidelines (347 lines)
- **QUICK_START.md:** 5-minute setup
- **DEEP_RESEARCH_IMPROVEMENTS.md:** Advanced features analysis
- **examples/advanced_features_demo.md:** Complete feature demonstrations
- **docs/integration/:** Claude Desktop setup guides
- **docs/troubleshooting/:** Fix guides for known issues
- **docs/sessions/:** Development session notes
- **docs/MCP_SPEC_REVIEW_SUMMARY.md:** Protocol upgrade roadmap
- **docs/PROMPTS_AND_RESOURCES.md:** Prompts and resources documentation

## Git Workflow Best Practices

**DO NOT commit directly to main.**

Use feature branches:
```bash
git checkout -b feature/my-tool
# Make changes
git commit -m "feat: Add my tool with validation"
git push origin feature/my-tool
# Create Pull Request
```

See `docs/GIT_WORKFLOW.md` for complete workflow guidelines.

## Version Information

- **Current:** v0.3.1
- **MCP Protocol:** 2025-03-26 (server)
- **SDK:** rmcp (local development build)
- **Rust:** 1.88.0 (or later)
- **Features:** Icons, Annotations, Enhanced Error Handling

## Security Notes

- Input validation on all tools
- No file system access
- No external network calls
- No code execution
- Stateless operation
- See SECURITY.md for production checklist

## Contact

- **Repository:** https://github.com/netadx1ai/mcp-boilerplate-rust
- **Website:** https://netadx.ai
- **Email:** hello@netadx.ai

---

## Quick Examples

### Test Progress Notifications
```bash
# Run server
cargo run --release -- --mode stdio

# Send request (via MCP Inspector or test script)
{
  "name": "process_with_progress",
  "arguments": {
    "items": 50,
    "delay_ms": 100
  }
}
```

### Test Task Lifecycle
```bash
# Call long_task with task mode enabled
{
  "name": "long_task",
  "arguments": {},
  "task": {}  # Enable task queuing
}

# Response will include task_id for polling
```

### Test Data Transformation
```bash
{
  "name": "transform_data",
  "arguments": {
    "data": ["hello", "world", "rust"],
    "operation": "uppercase"
  }
}

# Output: ["HELLO", "WORLD", "RUST"]
```

---

**Last Updated:** 2026-01-13  
**For:** Claude AI Assistant  
**Purpose:** Code assistance and development guidance with advanced features
**Protocol:** MCP 2025-03-26 with icons, annotations, and enhanced error handling