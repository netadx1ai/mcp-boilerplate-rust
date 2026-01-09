# Native Stdio Protocol Guide

**Timestamp:** 2025-01-08 15:30:00 +07:00 (HCMC)

This guide explains how to use the native stdio protocol implementation using the official `rmcp` Rust SDK.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Usage Modes](#usage-modes)
- [Claude Desktop Integration](#claude-desktop-integration)
- [Development](#development)
- [Troubleshooting](#troubleshooting)

## Overview

The MCP Boilerplate Rust now supports **two protocol modes**:

1. **HTTP Mode** - REST API server (port 8025)
2. **Stdio Mode** - Native stdio protocol using `rmcp` SDK

Both modes use the same tool implementations, ensuring consistent behavior.

### Why Native Stdio?

- **Direct Integration** - No HTTP wrapper needed
- **Better Performance** - No HTTP overhead
- **Official SDK** - Uses `rmcp` v0.12 from ModelContextProtocol
- **Claude Desktop Ready** - Direct stdio communication

## Architecture

### Protocol Flow

```
Claude Desktop → stdin/stdout → MCP Server (Rust)
```

### Component Structure

```
src/
├── main.rs                    # CLI entry point with mode selection
├── mcp/
│   ├── mod.rs                # MCP module exports
│   ├── stdio_server.rs       # Stdio server implementation
│   └── tool_handler.rs       # RMCP ServerHandler implementation
└── tools/
    └── echo.rs               # Sample tool (shared by both modes)
```

### Key Components

1. **StdioServer** (`stdio_server.rs`)
   - Initializes stdio transport
   - Configures server capabilities
   - Manages server lifecycle

2. **McpToolHandler** (`tool_handler.rs`)
   - Implements `rmcp::ServerHandler` trait
   - Maps MCP tool calls to internal tools
   - Handles tool discovery and execution

3. **Tool Implementations** (`tools/`)
   - Shared between HTTP and stdio modes
   - Action-based pattern
   - Consistent error handling

## Quick Start

### 1. Build the Project

```bash
# Development build
cargo build

# Production build (recommended for Claude Desktop)
cargo build --release
```

### 2. Test Stdio Mode

```bash
# Run in stdio mode
cargo run -- --mode stdio

# Or use Makefile
make run-stdio
```

### 3. Test with MCP Inspector

```bash
# Install MCP Inspector (if not installed)
npm install -g @modelcontextprotocol/inspector

# Test the server
mcp-inspector cargo run -- --mode stdio
```

### 4. Configure Claude Desktop

See [Claude Desktop Integration](#claude-desktop-integration) section.

## Configuration

### Environment Variables

Create `.env` file (copy from `.env.example`):

```bash
# Server configuration (only for HTTP mode)
HOST=0.0.0.0
PORT=8025

# Logging
RUST_LOG=info,mcp_boilerplate_rust=debug

# Optional: JWT secret (for HTTP mode)
JWT_SECRET=your-secret-key
```

### Server Capabilities

Defined in `src/mcp/stdio_server.rs`:

```rust
let capabilities = ServerCapabilities {
    tools: Some(rmcp::ToolsCapability {
        list_changed: Some(false),
    }),
    prompts: None,      // Not supported
    resources: None,    // Not supported
    logging: None,      // Optional
};
```

## Usage Modes

### HTTP Mode (Default)

```bash
# Using cargo
cargo run -- --mode http

# Using Makefile
make run-http

# Using run script
./run.sh dev http
```

**Endpoints:**
- `GET /health` - Health check
- `GET /tools` - List tools
- `POST /tools/{name}` - Execute tool

### Stdio Mode

```bash
# Using cargo
cargo run -- --mode stdio

# Using Makefile
make run-stdio

# Using run script
./run.sh dev stdio
```

**Protocol:** MCP v5 native stdio

### Both Mode (Planned)

```bash
cargo run -- --mode both
```

Run both HTTP and stdio simultaneously (not yet implemented).

## Claude Desktop Integration

### Option 1: Using Development Build

**Configuration File:** `claude_desktop_config_stdio.json`

```json
{
  "mcpServers": {
    "mcp-boilerplate-rust": {
      "command": "cargo",
      "args": [
        "run",
        "--manifest-path",
        "/Users/hoangiso/Desktop/mcp-boilerplate-rust/Cargo.toml",
        "--",
        "--mode",
        "stdio"
      ],
      "env": {
        "RUST_LOG": "info,mcp_boilerplate_rust=debug"
      }
    }
  }
}
```

### Option 2: Using Release Binary (Recommended)

**Step 1:** Build release binary

```bash
cargo build --release
```

**Step 2:** Configuration file: `claude_desktop_config_binary.json`

```json
{
  "mcpServers": {
    "mcp-boilerplate-rust": {
      "command": "/Users/hoangiso/Desktop/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust",
      "args": [
        "--mode",
        "stdio"
      ],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Installation Paths

**macOS:**
```
~/Library/Application Support/Claude/claude_desktop_config.json
```

**Windows:**
```
%APPDATA%\Claude\claude_desktop_config.json
```

**Linux:**
```
~/.config/Claude/claude_desktop_config.json
```

### Setup Steps

1. **Build Release Binary:**
   ```bash
   make release
   ```

2. **Copy Configuration:**
   ```bash
   # macOS
   cp claude_desktop_config_binary.json ~/Library/Application\ Support/Claude/claude_desktop_config.json
   ```

3. **Restart Claude Desktop**

4. **Verify:**
   - Open Claude Desktop
   - Look for MCP icon
   - Check available tools

## Development

### Adding New Tools

**Step 1:** Create tool implementation

```rust
// src/tools/my_tool.rs
use crate::types::{McpResult, ToolRequest, ToolResult};
use serde_json::json;

pub struct MyTool;

impl MyTool {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, request: ToolRequest) -> McpResult<ToolResult> {
        match request.action.as_str() {
            "do_something" => self.handle_action(request).await,
            _ => Err(McpError::InvalidAction(request.action))
        }
    }

    async fn handle_action(&self, request: ToolRequest) -> McpResult<ToolResult> {
        Ok(ToolResult {
            success: true,
            data: json!({"result": "success"}),
        })
    }
}
```

**Step 2:** Register in tool handler

```rust
// src/mcp/tool_handler.rs

// Add to struct
pub struct McpToolHandler {
    echo_tool: Arc<EchoTool>,
    my_tool: Arc<MyTool>,  // Add this
}

// Add to list_tools
async fn list_tools(&self) -> Result<ListToolsResult, String> {
    let tools = vec![
        // ... existing tools ...
        Tool {
            name: "my_action".to_string(),
            description: Some("My tool description".to_string()),
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
        },
    ];
    Ok(ListToolsResult { tools })
}

// Add to call_tool
async fn call_tool(&self, request: CallToolRequest) -> Result<CallToolResult, String> {
    let result = match request.name.as_str() {
        "my_action" => {
            let req = ToolRequest {
                action: "do_something".to_string(),
                params: /* convert args */,
            };
            self.my_tool.execute(req).await?
        }
        // ... existing cases ...
    };
    // ... rest of implementation
}
```

**Step 3:** Test both modes

```bash
# Test HTTP mode
make run-http
curl -X POST http://localhost:8025/tools/my_action -d '{"action":"do_something"}'

# Test stdio mode
make run-stdio
```

### File Size Control

**RULE: Every file MUST be under 500 lines**

Check file sizes:

```bash
make check-size
```

If file exceeds 500 lines, split it:

```bash
# Example: Split large tool
src/tools/my_tool/
├── mod.rs          # Main interface
├── actions.rs      # Action handlers
├── validators.rs   # Validation logic
└── helpers.rs      # Helper functions
```

### Testing

**Unit Tests:**

```bash
cargo test
```

**Integration Tests:**

```bash
# HTTP mode
make test-curl

# Stdio mode (requires MCP Inspector)
mcp-inspector cargo run -- --mode stdio
```

**Watch Mode:**

```bash
# HTTP mode
make watch

# Stdio mode
make watch-stdio
```

## Troubleshooting

### Stdio Mode Not Starting

**Problem:** Server starts but no output

**Solution:** Stdio mode uses stdin/stdout for protocol communication. Logging goes to stderr.

```bash
# See logs
RUST_LOG=debug cargo run -- --mode stdio 2> server.log
```

### Claude Desktop Not Seeing Tools

**Check 1:** Verify binary path in config

```bash
# Test binary exists
ls -la target/release/mcp-boilerplate-rust

# Test binary runs
./target/release/mcp-boilerplate-rust --mode stdio
```

**Check 2:** Verify logs

```bash
# macOS logs location
tail -f ~/Library/Logs/Claude/mcp*.log
```

**Check 3:** Restart Claude Desktop

```bash
# macOS
killall Claude
open -a Claude
```

### Tool Not Found Error

**Problem:** Tool exists but not registered

**Solution:** Check `list_tools` and `call_tool` in `tool_handler.rs`:

1. Tool defined in `list_tools()`
2. Tool case added in `call_tool()`
3. Tool name matches exactly

### Performance Issues

**Problem:** Slow tool execution

**Solutions:**

1. **Use Release Build:**
   ```bash
   cargo build --release
   ```

2. **Enable LTO:**
   Already enabled in `Cargo.toml`:
   ```toml
   [profile.release]
   opt-level = 3
   lto = true
   codegen-units = 1
   ```

3. **Check Logging:**
   ```bash
   # Reduce log level
   RUST_LOG=info cargo run -- --mode stdio
   ```

### Build Errors

**Problem:** rmcp dependency error

**Solution:**

```bash
# Update dependencies
cargo update

# Clean and rebuild
cargo clean
cargo build
```

**Problem:** Missing features

**Solution:** Check `Cargo.toml` features:

```toml
rmcp = { version = "0.12", features = ["server"] }
```

## Advanced Topics

### Custom Server Info

```rust
// src/mcp/stdio_server.rs
let server_info = rmcp::Implementation {
    name: "your-server-name".to_string(),
    version: "1.0.0".to_string(),
};
```

### Adding Logging Support

```rust
// Add to capabilities
let capabilities = ServerCapabilities {
    tools: Some(rmcp::ToolsCapability {
        list_changed: Some(false),
    }),
    logging: Some(rmcp::LoggingCapability {}),
    // ...
};

// Implement in handler
async fn set_logging_level(&self, level: LogLevel) -> Result<(), String> {
    // Implementation
}
```

### Adding Prompts Support

```rust
// Enable in capabilities
prompts: Some(rmcp::PromptsCapability {
    list_changed: Some(false),
}),

// Implement in handler
async fn list_prompts(&self) -> Result<ListPromptsResult, String> {
    let prompts = vec![
        Prompt {
            name: "greeting".to_string(),
            description: Some("Friendly greeting".to_string()),
            arguments: vec![],
        },
    ];
    Ok(ListPromptsResult { prompts })
}

async fn get_prompt(&self, request: GetPromptRequest) -> Result<GetPromptResult, String> {
    // Return prompt content
}
```

### Adding Resources Support

```rust
// Enable in capabilities
resources: Some(rmcp::ResourcesCapability {
    subscribe: Some(false),
    list_changed: Some(false),
}),

// Implement in handler
async fn list_resources(&self) -> Result<ListResourcesResult, String> {
    // Return available resources
}

async fn read_resource(&self, request: ReadResourceRequest) -> Result<ReadResourceResult, String> {
    // Return resource content
}
```

## Best Practices

1. **Use Release Builds for Production**
   - Faster execution
   - Smaller binary size
   - Better performance

2. **Keep Tools Small**
   - Max 500 lines per file
   - Split large tools into modules
   - Use helper functions

3. **Error Handling**
   - Return descriptive errors
   - Use `McpError` types
   - Log errors with context

4. **Testing**
   - Test both HTTP and stdio modes
   - Use MCP Inspector for stdio
   - Write unit tests

5. **Documentation**
   - Document tool schemas
   - Provide examples
   - Keep README updated

## Resources

- **RMCP SDK:** https://github.com/modelcontextprotocol/rust-sdk
- **MCP Spec:** https://modelcontextprotocol.io/specification/2025-11-25
- **Crates.io:** https://crates.io/crates/rmcp
- **MCP Inspector:** https://github.com/modelcontextprotocol/inspector

## Next Steps

1. **Build and Test:**
   ```bash
   make release
   make run-stdio
   ```

2. **Configure Claude Desktop:**
   - Copy config file
   - Restart Claude
   - Test tools

3. **Add Your Tools:**
   - Follow tool pattern
   - Register in handler
   - Test thoroughly

4. **Deploy:**
   - Build release binary
   - Update paths in config
   - Monitor logs

---

**Ready to use native stdio protocol with Claude Desktop!**