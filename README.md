# MCP Boilerplate Rust

**Version 0.6.3** | Production-Ready Multi-Transport MCP Server

A production-ready Rust implementation of the Model Context Protocol (MCP) 2025-11-25 specification featuring 6 transport modes, comprehensive observability, and enterprise-grade tooling.

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-108%20passing-brightgreen.svg)]()
[![MCP](https://img.shields.io/badge/MCP-2025--11--25-purple.svg)](https://modelcontextprotocol.io)

## Features

- **MCP 2025-11-25 Compliant** - Full spec implementation
- **6 Transport Modes** - Stdio, SSE, WebSocket, HTTP, HTTP Streaming, gRPC
- **Elicitation** - Form and URL modes for user input collection
- **Sampling with Tools** - LLM completion with tool calling
- **Structured Content** - Output schema validation
- **Task Management** - Long-running async operations
- **OAuth 2.1** - RFC 8414, RFC 9728 compliant
- **JWT Authentication** - Complete auth system
- **4 Auto-Generated SDKs** - TypeScript, Python, Go, Rust
- **Load Balancing** - 5 strategies, health checks, auto-failover
- **Observability** - OpenTelemetry + Prometheus

## Quick Start

### Prerequisites

```bash
# Rust 1.75+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install & Run

```bash
# Clone
git clone https://github.com/netadx/mcp-boilerplate-rust
cd mcp-boilerplate-rust

# Build
cargo build --release --features "http,auth"

# Run stdio server
./target/release/mcp-boilerplate-rust

# Run HTTP server
./target/release/mcp-boilerplate-rust --mode http

# Run tests
cargo test --features "http,auth"
```

### Test with MCP Inspector

```bash
npx @modelcontextprotocol/inspector ./target/release/mcp-boilerplate-rust
```

## MCP 2025-11-25 Features

### Elicitation

Collect user input via forms or external URLs.

```rust
// Form mode
let request = ElicitationRequest::form("Enter your details")
    .with_string_field("name", "Your name", true)
    .with_email_field("email", "Contact email", true)
    .with_enum_field("plan", vec!["free", "pro", "enterprise"], true)
    .build();

// URL mode (OAuth, payments)
let request = ElicitationRequest::url_with_callback(
    "Authenticate with GitHub",
    "https://github.com/login/oauth/authorize",
    "https://api.example.com/callback"
);
```

### Sampling with Tools

Request LLM completions with tool calling support.

```rust
let request = SamplingRequest::new("You are a helpful assistant")
    .add_user_message("What's the weather in Tokyo?")
    .with_tools(vec![weather_tool])
    .with_tool_choice(ToolChoice::Auto)
    .with_max_tokens(1000)
    .build();
```

### Structured Content

Validate tool outputs against JSON Schema.

```rust
let validator = OutputValidator::new(schema);
let result = StructuredOutput::new()
    .text("Temperature is 22.5°C")
    .structured(json!({"temperature": 22.5, "unit": "celsius"}))
    .build_validated(&validator)?;
```

### Task Management

Handle long-running operations asynchronously.

```rust
let task = manager.create_task(CreateTaskRequest {
    tool_name: "process_file".to_string(),
    arguments: json!({"file": "large.csv"}),
}).await?;

// Track progress
manager.update_progress(&task.id, 50).await?;

// Complete
manager.complete_task(&task.id, result).await?;
```

## Transport Modes

| Mode | Use Case | Command |
|------|----------|---------|
| Stdio | Desktop apps, Claude Desktop | `cargo run --release` |
| HTTP | REST APIs | `cargo run --release --features http -- -m http` |
| SSE | Browser push, live updates | `cargo run --release --features sse -- -m sse` |
| WebSocket | Real-time bidirectional | `cargo run --release --features websocket -- -m websocket` |
| HTTP Streaming | Large file transfers | `cargo run --release --features http-stream -- -m http-stream` |
| gRPC | Microservices | `cargo run --release --features grpc -- -m grpc` |

## Tools

| Tool | Description |
|------|-------------|
| `ping` | Health check |
| `echo` | Message validation |
| `info` | Server metadata |
| `calculate` | Math operations |
| `evaluate` | Expression evaluation |
| `process_with_progress` | Data processing with progress |
| `batch_process` | Batch operations |
| `transform_data` | Array transformations |
| `simulate_upload` | File upload simulation |
| `health_check` | System health status |
| `long_task` | Long operation simulation |

## OAuth 2.1 & Security

```bash
# OAuth endpoints
GET  /.well-known/oauth-authorization-server
GET  /.well-known/openid-configuration
GET  /.well-known/oauth-protected-resource
POST /oauth/authorize
POST /oauth/token
POST /oauth/register
POST /oauth/introspect
POST /oauth/revoke
```

## Client SDKs

Auto-generate type-safe client libraries:

```bash
cd sdk-generators
cargo run --release

# Generates:
# - TypeScript: output/typescript/mcp-client.ts
# - Python: output/python/mcp_client.py
# - Go: output/go/mcpclient/client.go
# - Rust: output/rust/mcp_client.rs
```

## Build Options

| Feature | Command | Size |
|---------|---------|------|
| Minimal (Stdio) | `cargo build --release` | ~2.4 MB |
| HTTP + Auth | `cargo build --release --features "http,auth"` | ~3.0 MB |
| Web (SSE/WS) | `cargo build --release --features "sse,websocket"` | ~3.3 MB |
| gRPC | `cargo build --release --features grpc` | ~3.9 MB |
| Full | `cargo build --release --features full` | ~4.2 MB |

## Testing

```bash
# Run all tests (108 passing)
cargo test --features "http,auth"

# Run specific module tests
cargo test --features "http,auth" elicitation::tests
cargo test --features "http,auth" sampling::tests
cargo test --features "http,auth" structured_content::tests
cargo test --features "http,auth" integration_tests
```

## Documentation

| Document | Description |
|----------|-------------|
| [docs/README.md](docs/README.md) | Documentation index |
| [docs/features/ELICITATION.md](docs/features/ELICITATION.md) | User input collection |
| [docs/features/SAMPLING.md](docs/features/SAMPLING.md) | LLM sampling with tools |
| [docs/features/STRUCTURED_CONTENT.md](docs/features/STRUCTURED_CONTENT.md) | Output validation |
| [docs/features/TASKS.md](docs/features/TASKS.md) | Task management |
| [docs/features/OAUTH.md](docs/features/OAUTH.md) | OAuth 2.1 |
| [CHANGELOG.md](CHANGELOG.md) | Version history |
| [NEXT_SESSION.md](NEXT_SESSION.md) | Implementation status |

## Project Statistics

| Metric | Value |
|--------|-------|
| MCP Spec Version | 2025-11-25 |
| Transport Modes | 6 |
| Production Tools | 11 |
| Client SDKs | 4 |
| Tests | 108 passing |
| Code | ~20,000 lines |
| Binary Size | 2.4MB - 4.2MB |

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

## License

MIT License - see [LICENSE](LICENSE) file.

## Support

- **GitHub:** https://github.com/netadx/mcp-boilerplate-rust
- **MCP Spec:** https://modelcontextprotocol.io/specification/2025-11-25
- **Website:** https://netadx.ai

---

**Version:** 0.6.3  
**Status:** Production Ready  
**MCP Spec:** 2025-11-25  
**Maintained by:** NetADX Team