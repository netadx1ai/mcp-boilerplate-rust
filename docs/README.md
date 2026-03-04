# Documentation

MCP Boilerplate Rust documentation.

**Version:** 0.6.3
**Last Updated:** 2026-03-04 HCMC
**MCP Spec:** 2025-11-25

---

## Quick Links

| Document | Description |
|----------|-------------|
| [SETUP.md](SETUP.md) | Installation and configuration |
| [TRANSPORTS.md](TRANSPORTS.md) | All 6 transport modes |
| [API.md](API.md) | API reference (tools, HTTP endpoints, auth) |

---

## Features

### Core Features

| Document | Description |
|----------|-------------|
| [features/AUTH.md](features/AUTH.md) | JWT authentication |
| [features/OAUTH.md](features/OAUTH.md) | OAuth 2.1 authorization |
| [features/LOAD_BALANCING.md](features/LOAD_BALANCING.md) | Load balancer |
| [features/POSTGRESQL.md](features/POSTGRESQL.md) | PostgreSQL db tool via PostgREST |

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
│  ├── structured_content.rs - Output validation          │
│  └── db.rs            - PostgreSQL via PostgREST [postgres] │
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
├─────────────────────────────────────────────────────────┤
│  External Services                                       │
│  └── PostgREST v12.2.0 -> PostgreSQL 16 [postgres]      │
└─────────────────────────────────────────────────────────┘
```

---

## Tools (12 Total)

### Built-in Tools (11)

| Tool | Description | File |
|------|-------------|------|
| `echo` | Message validation and timestamping | `src/tools/echo.rs` |
| `ping` | Health check | `src/tools/echo.rs` |
| `info` | Server metadata | `src/tools/echo.rs` |
| `calculate` | Arithmetic operations | `src/tools/calculator.rs` |
| `evaluate` | Math expression evaluation | `src/tools/calculator.rs` |
| `process_with_progress` | Data processing with progress | `src/tools/advanced.rs` |
| `batch_process` | Batch operations with logging | `src/tools/advanced.rs` |
| `transform_data` | Array transformation | `src/tools/advanced.rs` |
| `simulate_upload` | File upload simulation | `src/tools/advanced.rs` |
| `health_check` | System health monitoring | `src/tools/advanced.rs` |
| `long_task` | Long-running operation demo | `src/tools/advanced.rs` |

### Database Tool (requires `postgres` feature)

| Tool | Description | File |
|------|-------------|------|
| `db` | PostgreSQL CRUD via PostgREST | `src/tools/db.rs` |

Actions: `query`, `insert`, `update`, `delete`, `upsert`, `rpc`, `list_tables`, `describe`

See [features/POSTGRESQL.md](features/POSTGRESQL.md) for full documentation.

---

## Project Files

| File | Description |
|------|-------------|
| [../README.md](../README.md) | Project overview |
| [../CHANGELOG.md](../CHANGELOG.md) | Version history |
| [../CLAUDE.md](../CLAUDE.md) | AI assistant guide |
| [../Cargo.toml](../Cargo.toml) | Dependencies and feature flags |
| [../docker-compose.postgrest.yml](../docker-compose.postgrest.yml) | PostgREST dev environment |
| [../scripts/postgrest-setup.sql](../scripts/postgrest-setup.sql) | DB seed for PostgREST |
| [../scripts/test-db-integration.sh](../scripts/test-db-integration.sh) | PostgREST integration tests |
| [../scripts/test-db-mcp-smoke.sh](../scripts/test-db-mcp-smoke.sh) | MCP stdio e2e db tests |
| [../examples/claude_desktop_config_db.json](../examples/claude_desktop_config_db.json) | Claude Desktop config with db tool |

---

## Examples

### Browser Test Clients

Located in `../examples/`:
- `sse_test_client.html` - SSE transport test
- `websocket_test_client.html` - WebSocket transport test

### Claude Desktop Configs

Located in `../examples/`:
- `claude_desktop_config_stdio.json` - Stdio mode (default)
- `claude_desktop_config_binary.json` - Pre-built binary
- `claude_desktop_config_db.json` - Stdio with PostgreSQL db tool
- `claude_desktop_config_http_wrapper.json` - HTTP wrapper

### Code Examples

- `mcp_2025_11_25_features.rs` - MCP 2025-11-25 features usage

---

## Getting Started

```bash
# 1. Clone
git clone https://github.com/netadx1ai/mcp-boilerplate-rust.git
cd mcp-boilerplate-rust

# 2. Build (stdio only)
cargo build --release

# 3. Build with PostgreSQL db tool
cargo build --release --features postgres

# 4. Run stdio server
./target/release/mcp-boilerplate-rust

# 5. Run HTTP server
cargo build --release --features "http,auth"
./target/release/mcp-boilerplate-rust --mode http

# 6. Test
cargo test --features postgres    # 151 tests

# 7. Inspect
npx @modelcontextprotocol/inspector ./target/release/mcp-boilerplate-rust
```

### PostgreSQL db Tool Quick Start

```bash
# Start PostgREST stack
docker compose -f docker-compose.postgrest.yml up -d
docker compose -f docker-compose.postgrest.yml exec -T postgres \
  psql -U postgres -d myapp < scripts/postgrest-setup.sql

# Build with postgres feature
cargo build --features postgres

# Run integration tests
./scripts/test-db-integration.sh     # 23 tests
./scripts/test-db-mcp-smoke.sh       # 34 tests

# Stop
docker compose -f docker-compose.postgrest.yml down -v
```

---

## Feature Flags

| Flag | Description | New Deps |
|------|-------------|----------|
| (default) | Stdio only | core |
| `http` | REST API transport | axum, tower |
| `sse` | Server-Sent Events | axum, tokio-stream |
| `websocket` | WebSocket transport | axum, tokio-tungstenite |
| `http-stream` | HTTP streaming | axum |
| `grpc` | gRPC transport | tonic, prost |
| `postgres` | PostgreSQL db tool via PostgREST | none (uses existing reqwest) |
| `auth` | JWT authentication | jsonwebtoken |
| `metrics` | Prometheus metrics | prometheus |
| `otel` | OpenTelemetry tracing | opentelemetry |
| `database` | MongoDB (future) | mongodb |
| `full` | All features | everything |

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
208 tests passing (151 unit + 23 integration + 34 MCP smoke)

Unit tests by module:
- tools/db.rs           (56 tests) -- config, filters, builders, types, normalizer
- mcp/elicitation.rs     (7 tests)
- mcp/sampling.rs        (7 tests)
- mcp/structured_content.rs (10 tests)
- mcp/integration_tests.rs  (14 tests)
- mcp/tasks.rs           (5 tests)
- Other modules         (52 tests)

Integration tests:
- scripts/test-db-integration.sh  (23 tests) -- PostgREST HTTP level
- scripts/test-db-mcp-smoke.sh    (34 tests) -- MCP stdio end-to-end
```

---

## Support

- Repository: https://github.com/netadx1ai/mcp-boilerplate-rust
- MCP Spec: https://modelcontextprotocol.io/specification/2025-11-25