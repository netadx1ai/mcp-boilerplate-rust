# Features

Feature documentation for MCP Boilerplate Rust.

**Version:** 0.5.2  
**Last Updated:** 2026-01-09 HCMC

---

## Available Features

| Feature | Description |
|---------|-------------|
| [AUTH.md](AUTH.md) | Simple JWT authentication |
| [OAUTH.md](OAUTH.md) | OAuth 2.1 authorization (MCP spec) |
| [LOAD_BALANCING.md](LOAD_BALANCING.md) | Enterprise load balancer with 5 strategies |
| [SDK_GENERATORS.md](SDK_GENERATORS.md) | Auto-generate client SDKs (4 languages) |
| [RUST_SDK.md](RUST_SDK.md) | Generated Rust client SDK |

---

## Feature Flags

| Flag | Description |
|------|-------------|
| `auth` | JWT + OAuth 2.1 authentication |
| `metrics` | Prometheus metrics |
| `otel` | OpenTelemetry tracing |
| `database` | MongoDB integration |

---

## Quick Start

```bash
# Simple JWT Auth
cargo build --release --features "http,auth"
JWT_SECRET="secret" ./target/release/mcp-boilerplate-rust --mode http

# OAuth 2.1 (MCP spec)
OAUTH_ISSUER="http://localhost:8025" ./target/release/mcp-boilerplate-rust --mode http

# Generate SDKs
cd sdk-generators && cargo run --release
```
