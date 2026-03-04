# Setup Guide

Complete setup guide for MCP Boilerplate Rust.

**Version:** 0.6.3
**Last Updated:** 2026-03-04 HCMC

---

## Prerequisites

- Rust 1.75+ (`rustup`, `cargo`)
- macOS / Linux / Windows
- Docker + Docker Compose (for PostgreSQL db tool only)
- `../rust-sdk` repo must exist adjacent (rmcp local path dependency)

---

## Installation

```bash
git clone https://github.com/netadx1ai/mcp-boilerplate-rust.git
cd mcp-boilerplate-rust
cargo build --release
```

---

## Quick Test

```bash
# Test stdio mode
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1.0"}}}' | \
  RUST_LOG=off ./target/release/mcp-boilerplate-rust --mode stdio

# Test with MCP Inspector
npx @modelcontextprotocol/inspector ./target/release/mcp-boilerplate-rust --mode stdio
```

---

## Claude Desktop Setup

### Stdio (default)

**1. Build:**
```bash
cargo build --release
```

**2. Configure:**

Edit `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "mcp-rust": {
      "command": "/absolute/path/to/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust",
      "args": ["--mode", "stdio"],
      "env": {
        "RUST_LOG": "off"
      }
    }
  }
}
```

**3. Restart Claude Desktop:**
```bash
killall Claude
open -a Claude
```

**4. Test:**
Ask Claude: "What MCP tools are available?"

### Stdio with PostgreSQL db Tool

**1. Build with postgres feature:**
```bash
cargo build --release --features postgres
```

**2. Start PostgREST stack:**
```bash
docker compose -f docker-compose.postgrest.yml up -d
docker compose -f docker-compose.postgrest.yml exec -T postgres \
  psql -U postgres -d myapp < scripts/postgrest-setup.sql
```

**3. Configure Claude Desktop:**

```json
{
  "mcpServers": {
    "mcp-rust-db": {
      "command": "/absolute/path/to/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust",
      "args": ["--mode", "stdio"],
      "env": {
        "RUST_LOG": "off",
        "POSTGREST_URL": "http://localhost:3000",
        "POSTGREST_ANON_KEY": "",
        "POSTGREST_TIMEOUT": "30",
        "DB_ALLOWED_TABLES": "",
        "DB_TABLE_PREFIX": ""
      }
    }
  }
}
```

**4. Restart Claude Desktop and test:**
Ask Claude: "List all database tables" or "Query the test_mcp_db table"

See `examples/claude_desktop_config_db.json` for a ready-to-use config.

---

## Build Options

```bash
# Minimal (stdio only) - ~2.4 MB
cargo build --release

# PostgreSQL db tool - ~2.5 MB
cargo build --release --features postgres

# HTTP transport - ~3.1 MB
cargo build --release --features http

# HTTP with auth
cargo build --release --features "http,auth"

# All transports (except gRPC) + db tool
cargo build --release --features "http,sse,websocket,http-stream,database,auth,metrics,postgres"

# All features (requires protoc for gRPC) - ~4.2 MB
cargo build --release --features full
```

---

## Feature Flags

| Flag | Description | Extra Deps |
|------|-------------|------------|
| (none) | Stdio only | core |
| `http` | HTTP REST API | axum, tower |
| `sse` | Server-Sent Events | axum, tokio-stream |
| `websocket` | WebSocket | axum, tokio-tungstenite |
| `http-stream` | HTTP streaming | axum |
| `grpc` | gRPC (requires protoc) | tonic, prost |
| `postgres` | PostgreSQL db tool via PostgREST | none (uses existing reqwest) |
| `auth` | JWT authentication | jsonwebtoken |
| `metrics` | Prometheus metrics | prometheus |
| `otel` | OpenTelemetry tracing | opentelemetry |
| `database` | MongoDB (future) | mongodb |
| `full` | All features | everything |

---

## Running

```bash
# Stdio (default)
./target/release/mcp-boilerplate-rust --mode stdio

# Stdio with PostgreSQL db tool
POSTGREST_URL=http://localhost:3000 ./target/release/mcp-boilerplate-rust --mode stdio

# HTTP
./target/release/mcp-boilerplate-rust --mode http

# SSE
./target/release/mcp-boilerplate-rust --mode sse --bind 127.0.0.1:8025

# WebSocket
./target/release/mcp-boilerplate-rust --mode websocket --bind 127.0.0.1:9001

# HTTP with auth
JWT_SECRET="your-secret-key" ./target/release/mcp-boilerplate-rust --mode http

# Verbose logging (non-stdio modes only)
./target/release/mcp-boilerplate-rust --mode http --verbose
```

---

## Environment Variables

### Core

```bash
RUST_LOG=off                     # MUST be off for stdio mode
HOST=127.0.0.1                   # Bind host (HTTP mode)
PORT=8080                        # Bind port (HTTP mode)
JWT_SECRET=your-secret-key       # Required for auth feature
JWT_EXPIRY_SECONDS=86400         # Optional, default 24h
```

### PostgreSQL db Tool

```bash
POSTGREST_URL=http://localhost:3000   # PostgREST base URL
POSTGREST_ANON_KEY=                   # Bearer token for anonymous access
POSTGREST_TIMEOUT=30                  # Request timeout in seconds
DB_ALLOWED_TABLES=users,orders        # Comma-separated table whitelist
DB_TABLE_PREFIX=app_                  # Only allow tables with this prefix
```

If neither `DB_ALLOWED_TABLES` nor `DB_TABLE_PREFIX` is set, all tables are accessible.

---

## PostgREST Setup (for db tool)

### Start

```bash
# Start PostgreSQL 16 + PostgREST v12.2.0
docker compose -f docker-compose.postgrest.yml up -d

# Verify both containers are healthy
docker compose -f docker-compose.postgrest.yml ps

# Seed database (creates web_anon role, test table, test function)
docker compose -f docker-compose.postgrest.yml exec -T postgres \
  psql -U postgres -d myapp < scripts/postgrest-setup.sql

# Verify PostgREST responds
curl -s http://localhost:3000/test_mcp_db?limit=1
```

### Stop

```bash
# Stop, keep data
docker compose -f docker-compose.postgrest.yml down

# Stop and remove data volumes
docker compose -f docker-compose.postgrest.yml down -v
```

### Services

| Service | Image | Port | Credentials |
|---------|-------|------|-------------|
| postgres | postgres:16 | 5432 | user: `postgres`, pass: `postgres`, db: `myapp` |
| postgrest | postgrest/postgrest:v12.2.0 | 3000 | anon role: `web_anon` |

### Schema Changes

After modifying tables or functions, reload the PostgREST schema cache:

```bash
docker compose -f docker-compose.postgrest.yml exec -T postgres \
  psql -U postgres -d myapp -c "NOTIFY pgrst, 'reload schema';"
```

---

## Available Tools (12)

### Built-in (11)

| Tool | Description |
|------|-------------|
| `echo` | Echo back a message |
| `ping` | Health check |
| `info` | Server info |
| `calculate` | Arithmetic operations |
| `evaluate` | Math expression evaluation |
| `process_with_progress` | Progress demo |
| `batch_process` | Batch operations |
| `transform_data` | Data transformation |
| `simulate_upload` | Upload simulation |
| `health_check` | System health |
| `long_task` | Long-running task demo |

### Database (requires `postgres` feature)

| Tool | Description |
|------|-------------|
| `db` | PostgreSQL CRUD via PostgREST |

Actions: `query`, `insert`, `update`, `delete`, `upsert`, `rpc`, `list_tables`, `describe`

14 Supabase-compatible filter operators: `eq`, `neq`, `gt`, `gte`, `lt`, `lte`, `like`, `ilike`, `is`, `in`, `not`, `contains`, `containedBy`, `overlaps`

See [features/POSTGRESQL.md](features/POSTGRESQL.md) for full documentation.

---

## Testing

```bash
# Unit tests (default features, 95 tests)
cargo test

# Unit tests with postgres (151 tests, includes 56 db tests)
cargo test --features postgres

# Only db tool tests
cargo test --features postgres -- db::

# PostgREST integration tests (requires running PostgREST, 23 tests)
./scripts/test-db-integration.sh

# MCP stdio end-to-end tests (requires running PostgREST, 34 tests)
./scripts/test-db-mcp-smoke.sh

# General integration tests
./scripts/integration_test.sh

# Code quality
cargo clippy --features postgres
cargo fmt
```

Total: 208 tests (151 unit + 23 integration + 34 MCP smoke).

---

## Directory Structure

```
mcp-boilerplate-rust/
├── src/
│   ├── main.rs                     # Entry point
│   ├── mcp/                        # MCP servers (stdio, HTTP, SSE, WS, gRPC)
│   ├── tools/                      # 12 tool implementations
│   │   ├── echo.rs                 # echo, ping, info
│   │   ├── calculator.rs           # calculate, evaluate
│   │   ├── advanced.rs             # progress, batch, transform, upload, health, long_task
│   │   └── db.rs                   # PostgreSQL via PostgREST [postgres feature]
│   ├── middleware/                  # Auth middleware [auth feature]
│   ├── transport/                  # Transport abstraction
│   ├── loadbalancer/               # Load balancing
│   └── utils/                      # Logger, config, types
├── docker-compose.postgrest.yml    # PostgREST dev environment
├── scripts/
│   ├── postgrest-setup.sql         # DB seed
│   ├── test-db-integration.sh      # PostgREST integration tests
│   ├── test-db-mcp-smoke.sh        # MCP stdio e2e tests
│   └── integration_test.sh         # General integration tests
├── examples/                       # Claude Desktop configs, browser clients
├── docs/                           # Documentation
└── proto/                          # gRPC definitions
```

---

## Troubleshooting

### Binary not found
```bash
cargo build --release
ls -la target/release/mcp-boilerplate-rust
```

### Port in use
```bash
lsof -i :8025
lsof -i :3000
lsof -i :5432
kill -9 <PID>
```

### Claude Desktop not connecting
1. Check absolute path in config
2. Verify JSON syntax
3. Ensure `RUST_LOG` is `off` for stdio mode
4. Restart Claude completely (`killall Claude`)
5. Check Claude logs

### db tool returns "not enabled"
Binary was built without the `postgres` feature. Rebuild:
```bash
cargo build --release --features postgres
```

### db tool returns connection error
PostgREST is not running or `POSTGREST_URL` is wrong:
```bash
# Check PostgREST is up
curl -s http://localhost:3000/
# Start if needed
docker compose -f docker-compose.postgrest.yml up -d
```

### PostgREST returns 404 for a table
Table doesn't exist or `web_anon` role has no access:
```bash
# Check table exists
docker compose -f docker-compose.postgrest.yml exec -T postgres \
  psql -U postgres -d myapp -c "\dt"

# Re-run setup SQL
docker compose -f docker-compose.postgrest.yml exec -T postgres \
  psql -U postgres -d myapp < scripts/postgrest-setup.sql
```

### PostgREST not reflecting schema changes
Reload the schema cache:
```bash
docker compose -f docker-compose.postgrest.yml exec -T postgres \
  psql -U postgres -d myapp -c "NOTIFY pgrst, 'reload schema';"
```

### Colima / Docker won't start
```bash
colima stop --force
sleep 2
colima start --cpu 2 --memory 4
```

### Auth not working
```bash
echo $JWT_SECRET
curl -X POST http://127.0.0.1:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}'
```

---

## Performance

| Mode | Binary | Memory | Latency |
|------|--------|--------|---------|
| Stdio | 2.4 MB | <5 MB | 2ms |
| Stdio + postgres | 2.5 MB | <5 MB | 2ms (+5-50ms per db call) |
| HTTP | 3.1 MB | <8 MB | 20ms |
| SSE | 3.3 MB | <8 MB | 15ms |
| WebSocket | 3.3 MB | <8 MB | 8ms |
| gRPC | 3.9 MB | <10 MB | 4ms |

---

## Next Steps

- [TRANSPORTS.md](TRANSPORTS.md) - Transport details
- [features/POSTGRESQL.md](features/POSTGRESQL.md) - PostgreSQL db tool
- [features/AUTH.md](features/AUTH.md) - Authentication
- [API.md](API.md) - API reference