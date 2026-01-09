# Documentation

MCP Boilerplate Rust documentation.

**Version:** 0.6.3  
**Last Updated:** 2026-01-09 HCMC  
**MCP Spec:** 2025-11-25

---

## Quick Links

| Document | Description |
|----------|-------------|
| [SETUP.md](SETUP.md) | Installation and configuration |
| [TRANSPORTS.md](TRANSPORTS.md) | All 6 transport modes |
| [API.md](API.md) | API reference |

---

## Features

### Core Features

| Document | Description |
|----------|-------------|
| [features/AUTH.md](features/AUTH.md) | JWT authentication |
| [features/OAUTH.md](features/OAUTH.md) | OAuth 2.1 authorization |
| [features/LOAD_BALANCING.md](features/LOAD_BALANCING.md) | Load balancer |

### MCP 2025-11-25 Features

| Document | Description |
|----------|-------------|
| [features/ELICITATION.md](features/ELICITATION.md) | User input collection (form/URL modes) |
| [features/SAMPLING.md](features/SAMPLING.md) | LLM sampling with tool calling |
| [features/STRUCTURED_CONTENT.md](features/STRUCTURED_CONTENT.md) | Output schema validation |
| [features/TASKS.md](features/TASKS.md) | Long-running task management |

### SDK & Tools

| Document | Description |
|----------|-------------|
| [features/SDK_GENERATORS.md](features/SDK_GENERATORS.md) | Client SDK generators |
| [features/RUST_SDK.md](features/RUST_SDK.md) | Rust SDK details |

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    MCP Server v0.6.3                     │
├─────────────────────────────────────────────────────────┤
│  ProtocolHandler                                         │
│  ├── TaskManager                                        │
│  ├── ToolMetadataRegistry                               │
│  └── ElicitationManager                                 │
├─────────────────────────────────────────────────────────┤
│  Core Modules                                            │
│  ├── tasks.rs         - Long-running task management    │
│  ├── elicitation.rs   - User input collection           │
│  ├── sampling.rs      - LLM completion with tools       │
│  └── structured_content.rs - Output validation          │
├─────────────────────────────────────────────────────────┤
│  Transport Layer                                         │
│  ├── stdio (default)                                    │
│  ├── HTTP/SSE (optional)                                │
│  ├── WebSocket (optional)                               │
│  └── gRPC (optional)                                    │
├─────────────────────────────────────────────────────────┤
│  Security                                                │
│  ├── OAuth 2.1 (RFC 8414, RFC 9728)                     │
│  ├── JWT Authentication                                  │
│  └── Well-known metadata endpoints                      │
└─────────────────────────────────────────────────────────┘
```

---

## Project Files

| File | Description |
|------|-------------|
| [../README.md](../README.md) | Project overview |
| [../CHANGELOG.md](../CHANGELOG.md) | Version history |
| [../CLAUDE.md](../CLAUDE.md) | AI assistant guide |
| [../NEXT_SESSION.md](../NEXT_SESSION.md) | Implementation status |

---

## Examples

### Browser Test Clients

Located in `../examples/`:
- `sse_test_client.html` - SSE transport test
- `websocket_test_client.html` - WebSocket transport test

### Code Examples

- `mcp_2025_11_25_features.rs` - MCP 2025-11-25 features usage

---

## Getting Started

```bash
# 1. Clone
git clone https://github.com/netadx/mcp-boilerplate-rust.git
cd mcp-boilerplate-rust

# 2. Build
cargo build --release --features "http,auth"

# 3. Run stdio server
./target/release/mcp-boilerplate-rust

# 4. Run HTTP server
./target/release/mcp-boilerplate-rust --mode http

# 5. Test
cargo test --features "http,auth"

# 6. Inspect
npx @modelcontextprotocol/inspector ./target/release/mcp-boilerplate-rust
```

---

## MCP 2025-11-25 Spec Coverage

- [x] Task management (tasks/list, tasks/get, tasks/result, tasks/cancel)
- [x] Tool metadata (outputSchema, taskSupport, progress, cancellation)
- [x] Elicitation form mode with JSON Schema
- [x] Elicitation URL mode for sensitive data
- [x] Enhanced enum support (titled, multi-select)
- [x] Sampling with tool calling
- [x] Tool choice (auto, none, required, specific)
- [x] Structured content output
- [x] Output schema validation
- [x] OAuth 2.1 authorization
- [x] Well-known metadata endpoints

---

## Test Results

```
108 tests passing

Modules:
- mcp/elicitation.rs (7 tests)
- mcp/sampling.rs (7 tests)
- mcp/structured_content.rs (10 tests)
- mcp/integration_tests.rs (14 tests)
- mcp/tasks.rs (5 tests)
- Other modules (65 tests)
```

---

## Support

- Repository: https://github.com/netadx/mcp-boilerplate-rust
- MCP Spec: https://modelcontextprotocol.io/specification/2025-11-25