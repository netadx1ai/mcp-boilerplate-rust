# MCP Boilerplate Rust

**Version 0.3.1** | ✅ Stdio Mode | ✅ HTTP Mode | 🚀 Production Ready | 🔒 Security Hardened

A production-ready Rust implementation of the Model Context Protocol (MCP) using the official `rmcp` SDK from ModelContextProtocol. Both stdio and HTTP modes fully functional with shared tool implementation and input validation.

## Status

✅ **v0.3.1** - Both modes working perfectly  
✅ **0 errors, 0 warnings** in both stdio and HTTP builds  
✅ **All tests passing** - Automated test suites included  
✅ **Shared tool types** - Single source of truth  
✅ **Input validation** - 10KB limit, empty checks  
✅ **Security reviewed** - No vulnerabilities found  

## Key Features

- **Dual Protocol** - Stdio (primary) + HTTP (optional) with shared implementation
- **MCP Standard Compliant** - Uses official `rmcp` v0.12 SDK
- **Zero Warnings** - Clean builds with proper feature gates
- **Type-Safe** - Full Rust type safety with auto-generated schemas
- **Production Ready** - Async/await, error handling, comprehensive logging
- **Extensible** - Easy to add new tools via shared types
- **Well Tested** - Automated test scripts for both modes
- **Security Hardened** - Input validation, comprehensive security documentation

## Architecture

```
┌─────────────────┐         ┌─────────────────┐
│  Claude Desktop │         │   HTTP Client   │
└────────┬────────┘         └────────┬────────┘
         │ stdio                     │ REST API
         │                           │
    ┌────▼───────────────────────────▼────┐
    │      MCP Boilerplate Rust v0.3.1    │
    │        (rmcp SDK v0.12)             │
    ├─────────────────────────────────────┤
    │          Shared Tools               │
    │  ┌─────────────────────────────┐   │
    │  │ EchoRequest/Response        │   │
    │  │ PingResponse                │   │
    │  │ InfoResponse                │   │
    │  └─────────────────────────────┘   │
    └─────────────────────────────────────┘
```

## Quick Start

See **[QUICK_START.md](QUICK_START.md)** for detailed guide and **[SECURITY.md](SECURITY.md)** for security guidelines.

### 5-Second Test

```bash
# Test stdio mode
./test_mcp.sh

# Test HTTP mode  
./test_http.sh
```

### Prerequisites

- Rust 1.70+ (install from https://rustup.rs/)
- Claude Desktop (for stdio integration)

### Build & Run

```bash
# Stdio mode (for Claude Desktop)
cargo build --release
./target/release/mcp-boilerplate-rust --mode stdio

# HTTP mode (REST API)
cargo build --release --features http
./target/release/mcp-boilerplate-rust --mode http
```

### Claude Desktop Integration

```bash
# Production build
cargo build --release

# Binary location
./target/release/mcp-boilerplate-rust
```

## Usage

### Stdio Mode (Primary)

```bash
# Run stdio server
cargo run -- --mode stdio

# Or use Makefile
make run-stdio

# With debug logging
cargo run -- --mode stdio --verbose
```

### HTTP Mode (Optional)

Requires `http` feature:

```bash
# Build with HTTP support
cargo build --features http

# Run HTTP server
cargo run --features http -- --mode http
```

### Claude Desktop Integration

**Step 1:** Build release binary

```bash
cargo build --release
```

**Step 2:** Configure Claude Desktop

Edit config file (macOS):
```bash
~/Library/Application Support/Claude/claude_desktop_config.json
```

Add server:
```json
{
  "mcpServers": {
    "mcp-boilerplate-rust": {
      "command": "/Users/hoangiso/Desktop/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust",
      "args": ["--mode", "stdio"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

**Step 3:** Restart Claude Desktop

Available tools will appear in Claude interface.

## Project Structure

```
mcp-boilerplate-rust/
├── src/
│   ├── main.rs              # Entry point with CLI args
│   ├── types.rs             # MCP types (ToolInput, ToolOutput, etc.)
│   ├── mcp/
│   │   ├── mod.rs          # MCP module exports
│   │   ├── stdio_server.rs # Stdio server implementation
│   │   └── tool_handler.rs # ServerHandler implementation
│   ├── tools/
│   │   ├── mod.rs          # Tool exports
│   │   └── echo.rs         # Sample echo tool
│   ├── utils/
│   │   ├── config.rs       # Configuration
│   │   └── logger.rs       # Logging
│   ├── middleware/         # HTTP middleware (optional)
│   ├── models/             # Data models
│   └── services/           # Business logic
├── docs/
│   ├── NATIVE_STDIO_GUIDE.md    # Complete stdio guide
│   ├── AI_TOOL_PATTERN.md       # AI tool development patterns
│   └── API.md                   # API documentation
├── Cargo.toml                    # Dependencies
├── Makefile                      # Development commands
└── README.md                     # This file
```

## Available Tools

### Echo Tool

Three capabilities with input validation:

1. **echo** - Echo back a message (max 10KB, non-empty)
   ```json
   {
     "name": "echo",
     "arguments": {
       "message": "Hello, MCP!"
     }
   }
   ```

2. **ping** - Simple connectivity test
   ```json
   {
     "name": "ping",
     "arguments": {}
   }
   ```

3. **info** - Get tool information
   ```json
   {
     "name": "info",
     "arguments": {}
   }
   ```

## Development

### Adding New Tools

**Step 1:** Create tool implementation

```rust
// src/tools/my_tool.rs
use crate::types::{McpResult, ToolInput, ToolOutput};
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

**Step 2:** Register in tool handler

```rust
// src/mcp/tool_handler.rs

// Add to struct
pub struct McpToolHandler {
    echo_tool: Arc<EchoTool>,
    my_tool: Arc<MyTool>,  // Add this
}

// Add to list_tools()
Tool {
    name: "my_tool".to_string(),
    description: Some("My custom tool".to_string()),
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
}

// Add to call_tool()
"my_tool" => {
    let input = self.convert_arguments_to_input(request.arguments);
    let output = self.my_tool.execute(input).await?;
    Ok(self.convert_output_to_result(output))
}
```

**Step 3:** Test

```bash
# Rebuild
cargo build --release

# Test with Claude Desktop or MCP Inspector
mcp-inspector ./target/release/mcp-boilerplate-rust --mode stdio
```

### File Size Control

**RULE: Every file MUST be under 500 lines**

```bash
# Check file sizes
make check-size

# Script checks all .rs files
./scripts/check-file-sizes.sh
```

If file exceeds limit, split into modules:
```
src/tools/large_tool/
├── mod.rs       # Public interface
├── actions.rs   # Action handlers
└── helpers.rs   # Helper functions
```

### Testing

```bash
# Run tests
cargo test

# Run with coverage
make test

# Check code
make check

# Lint
make lint

# Format
make fmt
```

### Development Commands

```bash
# Run stdio mode
make run-stdio

# Run with debug logging
make dev-stdio

# Watch mode (auto-reload)
make watch-stdio

# Check file sizes
make check-size

# Full quality check
make all
```

## Configuration

### Environment Variables

Create `.env` file:

```bash
# Logging
RUST_LOG=info,mcp_boilerplate_rust=debug

# HTTP mode only (optional)
HOST=0.0.0.0
PORT=8025

# Optional features
JWT_SECRET=your-secret-key
MONGODB_URI=mongodb://localhost:27017
```

### Cargo Features

```toml
# Default: stdio only
cargo build

# With HTTP support
cargo build --features http

# With database
cargo build --features database

# All features
cargo build --features full
```

## MCP Standard Compliance

This project follows MCP specification v2025-11-25:

- **Protocol:** Native stdio transport using `rmcp` SDK
- **Server Handler:** Implements `rmcp::ServerHandler` trait
- **Capabilities:** Tools, Prompts, Resources support
- **Types:** MCP-compliant request/response types
- **Error Handling:** Proper error propagation

### Implemented Capabilities

- ✅ **Tools** - Tool listing and execution
- ⏳ **Prompts** - Template support (placeholder)
- ⏳ **Resources** - Resource access (placeholder)
- ⏳ **Logging** - Structured logging (optional)

## Documentation

- **[QUICK_START.md](QUICK_START.md)** - 5-minute setup guide
- **[SECURITY.md](SECURITY.md)** - Security guidelines and best practices
- **[SIMPLIFICATION_COMPLETE.md](SIMPLIFICATION_COMPLETE.md)** - v0.3.1 changes
- **[CLEANUP_HTTP_FIX_COMPLETE.md](CLEANUP_HTTP_FIX_COMPLETE.md)** - HTTP mode fix details
- **[REFACTORING_COMPLETE.md](REFACTORING_COMPLETE.md)** - Stdio implementation details

## Best Practices

1. **Use stdio mode for production** - Direct Claude Desktop integration
2. **Keep files under 500 lines** - Better maintainability
3. **Follow MCP patterns** - Use official SDK types and patterns
4. **Write tests** - Unit and integration tests
5. **Handle errors properly** - Use McpError types
6. **Log appropriately** - Use tracing with proper levels
7. **Document tools** - Clear descriptions and schemas

## Troubleshooting

### Stdio mode not working

```bash
# Check binary exists
ls -la target/release/mcp-boilerplate-rust

# Test binary
./target/release/mcp-boilerplate-rust --mode stdio

# Check logs (stderr)
RUST_LOG=debug cargo run -- --mode stdio 2> server.log
```

### Claude Desktop not seeing tools

1. Check config file path (macOS):
   ```bash
   ~/Library/Application Support/Claude/claude_desktop_config.json
   ```

2. Verify binary path is absolute

3. Restart Claude Desktop:
   ```bash
   killall Claude
   open -a Claude
   ```

4. Check Claude logs:
   ```bash
   tail -f ~/Library/Logs/Claude/mcp*.log
   ```

### Build errors

```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update
```

## Performance

Release build optimizations in `Cargo.toml`:

```toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
strip = true         # Smaller binary
```

## Resources

- **MCP Specification:** https://modelcontextprotocol.io/specification/2025-11-25
- **Rust SDK:** https://github.com/modelcontextprotocol/rust-sdk
- **rmcp Crate:** https://crates.io/crates/rmcp
- **MCP Inspector:** https://github.com/modelcontextprotocol/inspector

## License

MIT

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## Changelog

### v0.3.1 (2026-01-08)
- Added input validation (10KB limit, empty checks)
- Created comprehensive SECURITY.md
- Removed unused modules (models/, services/)
- Simplified error types (removed 2 unused variants)
- Updated .env.example with security settings
- Zero warnings in strict builds
- Full security audit completed

### v0.3.0
- Refactored to focus on MCP standard and rust-sdk
- Stdio mode as primary protocol
- HTTP mode as optional feature
- Simplified types aligned with rmcp SDK
- Improved tool handler implementation

### v0.2.0
- Added native stdio support using rmcp SDK
- Dual protocol support (HTTP + stdio)
- CLI mode selection

### v0.1.0
- Initial release with HTTP support
- Sample echo tool
- Basic MCP v5 implementation

---

## Get Started Today

Ready to unlock the power of AI for your organization?

🌐 Visit: https://netadx.ai  
📧 Contact: hello@netadx.ai  
📅 Book Consultation: Free 30-minute discovery call available

"Empowering businesses through intelligent automation and custom AI solutions"

---

**Ready for production use with Claude Desktop!**