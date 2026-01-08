# CLAUDE.md

This file provides guidance to Claude (AI assistant) when working with code in this repository.

## Project Overview

**MCP Boilerplate Rust** is a production-ready Model Context Protocol (MCP) server implementation in Rust with dual transport support (stdio + HTTP). This is a reference implementation and starting template for Rust-based MCP servers.

**Version:** 0.3.1  
**Protocol:** MCP 2024-11-05  
**SDK:** rmcp v0.12.0 (official Rust SDK)  
**Status:** Production ready

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
│   │   └── echo.rs         # Echo tool with validation
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

### Key Patterns

**1. Tool Implementation Pattern:**
```rust
// src/tools/example.rs
use crate::tools::shared::{ToolInput, ToolOutput};
use crate::utils::types::McpResult;
use serde_json::json;

pub struct ExampleTool;

impl ExampleTool {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, input: ToolInput) -> McpResult<ToolOutput> {
        // Validate input
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

### 1. echo
**Purpose:** Message validation and timestamping  
**Input:** `message` (string, 1-10,240 bytes, non-empty)  
**Output:** `{ message, timestamp }`  
**File:** `src/tools/echo.rs`

### 2. ping
**Purpose:** Health check and connectivity test  
**Input:** None  
**Output:** `{ response: "pong", timestamp }`  
**File:** `src/tools/echo.rs`

### 3. info
**Purpose:** Server metadata  
**Input:** None  
**Output:** `{ tool, version, description }`  
**File:** `src/tools/echo.rs`

## Adding New Tools

### Step 1: Create Tool Module

Create `src/tools/my_tool.rs`:
```rust
use crate::tools::shared::{ToolInput, ToolOutput};
use crate::utils::types::McpResult;
use serde_json::json;

pub struct MyTool;

impl MyTool {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, input: ToolInput) -> McpResult<ToolOutput> {
        let param = input.get_string("param")?;
        
        let data = json!({
            "result": param,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        Ok(ToolOutput::json(data))
    }
}
```

### Step 2: Register in main.rs

Add to tool list in `run_stdio_server()` or `run_http_server()`:
```rust
// Add to tools vector
tools.push(Tool {
    name: "my_tool".to_string(),
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

// Add to tool execution match
"my_tool" => {
    let my_tool = MyTool::new();
    let input = ToolInput::from_json(request.params.arguments)?;
    let output = my_tool.execute(input).await?;
    Ok(output.into_call_tool_result())
}
```

### Step 3: Test

```bash
cargo build --release
./scripts/test_mcp.sh
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
| Response time | 2-7ms | 8-12ms | Average |
| Memory | <5MB | <8MB | Typical |
| CPU | <1% | <2% | Idle |
| Binary size | 2.4MB | 3.1MB | Release |

## Common Issues & Solutions

### Issue: JSON Parse Errors in Claude Desktop

**Symptom:** `Unexpected token '\x1B'` errors  
**Cause:** Logging output interfering with JSON  
**Solution:** Ensure `RUST_LOG=off` in stdio mode

### Issue: Tools Not Appearing

**Check:**
1. Binary built in release mode
2. Correct path in config
3. `--mode stdio` argument present
4. Claude Desktop restarted

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
- **docs/integration/:** Claude Desktop setup guides
- **docs/troubleshooting/:** Fix guides for known issues
- **docs/sessions/:** Development session notes

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
- **MCP Protocol:** 2024-11-05 (server)
- **SDK:** rmcp v0.12.0
- **Rust:** 1.88.0 (or later)

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

**Last Updated:** 2026-01-08  
**For:** Claude AI Assistant  
**Purpose:** Code assistance and development guidance