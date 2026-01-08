# MCP Boilerplate Rust - Project Overview

Simple, clean Rust implementation of Model Context Protocol (MCP) v5.

## What This Is

A minimal boilerplate for building MCP v5 servers in Rust with HTTP transport. Designed for developers who want to create production-ready API tools quickly.

## Key Features

- **Simple Architecture** - Clean separation of concerns
- **HTTP Transport** - REST API using Axum framework
- **Tool Pattern** - Easy-to-follow pattern for creating tools
- **Async/Await** - Built on Tokio for high performance
- **Type Safe** - Full Rust type safety
- **Production Ready** - Error handling, logging, Docker support

## Project Structure

```
mcp-boilerplate-rust/
├── src/
│   ├── main.rs              # Server setup, HTTP routes, handlers
│   ├── types.rs             # Core types, errors, request/response
│   └── tools/
│       ├── mod.rs           # Tools module exports
│       └── echo.rs          # Sample echo tool (ping, echo, info)
│
├── docs/
│   └── API.md               # Complete API documentation
│
├── Cargo.toml               # Rust dependencies
├── .env.example             # Environment configuration template
├── Dockerfile               # Multi-stage production build
├── docker-compose.yml       # Docker Compose with MongoDB
├── Makefile                 # Common development commands
├── run.sh                   # Server run script
├── test.sh                  # Endpoint testing script
├── README.md                # Full documentation
├── QUICKSTART.md            # 5-minute getting started guide
├── CONTRIBUTING.md          # Development guide
└── LICENSE                  # MIT license

```

## Core Components

### 1. Server (`src/main.rs`)

- Axum HTTP server setup
- CORS middleware
- Route registration
- Health check endpoints
- Tool request handlers

### 2. Types (`src/types.rs`)

- `AppState` - Application state
- `ToolRequest` - Incoming tool requests
- `ToolResult` - Tool execution results
- `McpResponse` - Standard API response
- `McpError` - Error types
- `McpResult<T>` - Result type alias

### 3. Tools (`src/tools/`)

Each tool follows this pattern:
- Struct with `new()` constructor
- `execute()` method handling actions
- Action-specific handler methods
- Returns `McpResult<ToolResult>`

## Technology Stack

| Component | Technology |
|-----------|------------|
| Runtime | Tokio (async) |
| HTTP Server | Axum 0.7 |
| Middleware | Tower/Tower-HTTP |
| Serialization | Serde/serde_json |
| Logging | tracing/tracing-subscriber |
| Error Handling | thiserror/anyhow |
| Database | MongoDB (optional) |
| Auth | jsonwebtoken (optional) |

## API Endpoints

### Health Check
- `GET /health` - Server health status
- `GET /` - Root health check

### Tools
- `POST /tools/echo` - Echo tool with multiple actions
  - `action=echo` - Echo back a message
  - `action=ping` - Simple ping-pong test
  - `action=info` - Get tool information

## Response Format

All responses follow MCP v5 standard:

```json
{
  "success": true,
  "data": { ... },
  "metadata": {
    "executionTime": 10,
    "timestamp": "2025-01-08T10:30:00Z"
  }
}
```

## Quick Commands

```bash
# Setup
make setup

# Run server
make run

# Run with debug logs
make dev

# Run tests
make test

# Test with curl
make test-curl

# Format code
make fmt

# Lint code
make lint

# Build release
make release

# Docker
docker-compose up
```

## Tool Development Pattern

1. Create tool file: `src/tools/my_tool.rs`
2. Implement tool struct with `execute()` method
3. Register in `src/tools/mod.rs`
4. Add route in `src/main.rs`
5. Add handler function
6. Test with curl

Example tool structure:

```rust
pub struct MyTool;

impl MyTool {
    pub fn new() -> Self { Self }
    
    pub async fn execute(&self, request: ToolRequest) -> McpResult<ToolResult> {
        match request.action.as_str() {
            "action1" => self.handle_action1(request).await,
            "action2" => self.handle_action2(request).await,
            _ => Err(McpError::InvalidAction(request.action)),
        }
    }
    
    async fn handle_action1(&self, request: ToolRequest) -> McpResult<ToolResult> {
        Ok(ToolResult {
            success: true,
            data: json!({ "result": "success" }),
        })
    }
}
```

## Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `HOST` | 0.0.0.0 | Server bind address |
| `PORT` | 8025 | Server port |
| `RUST_LOG` | info | Log level |
| `MONGODB_URI` | - | MongoDB connection (optional) |
| `JWT_SECRET` | - | JWT secret (optional) |

## Error Handling

Built-in error types:
- `ToolExecution` - Tool execution errors
- `InvalidAction` - Unknown action
- `MissingParameter` - Required parameter missing
- `Database` - Database errors
- `Authentication` - Auth errors
- `Internal` - Internal server errors

## Performance

- Async I/O with Tokio
- Zero-copy JSON parsing
- Optimized release builds (LTO enabled)
- Typical response time: 1-5ms

## Production Deployment

1. Build release binary:
   ```bash
   cargo build --release
   ```

2. Binary location:
   ```
   target/release/mcp-boilerplate-rust
   ```

3. Run:
   ```bash
   ./target/release/mcp-boilerplate-rust
   ```

Or use Docker:
```bash
docker-compose up -d
```

## Documentation

- `README.md` - Complete project documentation
- `QUICKSTART.md` - 5-minute getting started guide
- `CONTRIBUTING.md` - Development and contribution guide
- `docs/API.md` - Comprehensive API reference

## Testing

Manual testing:
```bash
./test.sh
```

Unit tests:
```bash
cargo test
```

Integration testing:
```bash
curl -X POST http://localhost:8025/tools/echo \
  -H "Content-Type: application/json" \
  -d '{"action":"ping"}'
```

## Design Philosophy

1. **Simple First** - Keep it minimal and understandable
2. **Type Safe** - Leverage Rust's type system
3. **Clean Code** - Self-explanatory, maintainable code
4. **Production Ready** - Proper error handling and logging
5. **Easy Extension** - Clear patterns for adding features

## Use Cases

- MCP v5 tool servers
- REST API backends
- Microservices
- API gateways
- Integration services
- Development prototypes

## Next Steps

1. Clone/copy this boilerplate
2. Follow `QUICKSTART.md`
3. Create your first tool
4. Read `docs/API.md` for details
5. Deploy to production

## License

MIT License - See LICENSE file

## Author

NetADX MCP Team

## Version

0.1.0 - Initial release with echo tool example

---

**Note**: This is a boilerplate project. Customize and extend based on your needs.