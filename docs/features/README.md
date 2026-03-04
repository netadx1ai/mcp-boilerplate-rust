# Features

Feature documentation for MCP Boilerplate Rust.

**Version:** 0.6.3
**Last Updated:** 2026-03-04 HCMC

---

## Available Features

| Feature | Description |
|---------|-------------|
| [AUTH.md](AUTH.md) | Simple JWT authentication |
| [OAUTH.md](OAUTH.md) | OAuth 2.1 authorization (MCP spec) |
| [POSTGRESQL.md](POSTGRESQL.md) | PostgreSQL db tool via PostgREST |
| [LOAD_BALANCING.md](LOAD_BALANCING.md) | Enterprise load balancer with 5 strategies |
| [SDK_GENERATORS.md](SDK_GENERATORS.md) | Auto-generate client SDKs (4 languages) |
| [RUST_SDK.md](RUST_SDK.md) | Generated Rust client SDK |

---

## Feature Flags

| Flag | Description |
|------|-------------|
| `postgres` | PostgreSQL db tool via PostgREST (no new deps) |
| `auth` | JWT + OAuth 2.1 authentication |
| `metrics` | Prometheus metrics |
| `otel` | OpenTelemetry tracing |
| `database` | MongoDB integration (future) |

---

## Quick Start

```bash
# PostgreSQL db tool
cargo build --release --features postgres
docker compose -f docker-compose.postgrest.yml up -d
docker compose -f docker-compose.postgrest.yml exec -T postgres \
  psql -U postgres -d myapp < scripts/postgrest-setup.sql
POSTGREST_URL=http://localhost:3000 ./target/release/mcp-boilerplate-rust --mode stdio

# Simple JWT Auth
cargo build --release --features "http,auth"
JWT_SECRET="secret" ./target/release/mcp-boilerplate-rust --mode http

# OAuth 2.1 (MCP spec)
OAUTH_ISSUER="http://localhost:8080" ./target/release/mcp-boilerplate-rust --mode http

# Generate SDKs
cd sdk-generators && cargo run --release
```
