# Features Documentation

Complete guides for all major features in MCP Boilerplate Rust.

**Version:** 0.5.0  
**Last Updated:** 2026-01-09 HCMC

---

## 📚 Available Features

### Load Balancing
**[LOAD_BALANCING.md](LOAD_BALANCING.md)** - Enterprise-grade load balancing (659 lines)

**What it provides:**
- 5 load balancing strategies (Round-robin, Least connections, Random, Weighted, IP hash)
- Automatic health checking with configurable intervals
- Auto failover to healthy backends
- Per-backend connection limits and tracking
- Sticky sessions support
- Real-time statistics and monitoring
- Dynamic backend management (add/remove at runtime)

**Quick Start:**
```rust
use mcp_boilerplate_rust::loadbalancer::{LoadBalancer, LoadBalancerConfig, Backend, Strategy};

let config = LoadBalancerConfig::new(Strategy::RoundRobin)
    .add_backend(Backend::new("b1".to_string(), "127.0.0.1:8081".to_string()))
    .add_backend(Backend::new("b2".to_string(), "127.0.0.1:8082".to_string()))
    .with_failover(true);

let lb = LoadBalancer::new(config);
lb.start_health_checks().await;
```

---

### SDK Generators
**[SDK_GENERATORS.md](SDK_GENERATORS.md)** - Auto-generate client SDKs (607 lines)

**What it provides:**
- Auto-generate client libraries in 4 languages
- TypeScript SDK (209 lines, zero dependencies)
- Python SDK (111 lines, type hints)
- Go SDK (172 lines, idiomatic Go)
- Rust SDK (470 lines, race car quality)
- All tools automatically included
- Complete type safety
- Consistent APIs across languages

**Quick Start:**
```bash
cd sdk-generators
cargo run --release

# Generates:
# - output/typescript/mcp-client.ts
# - output/python/mcp_client.py
# - output/go/mcpclient/client.go
# - output/rust/mcp_client.rs (Race Car Edition 🏎️)
```

---

### Rust SDK (Race Car Edition 🏎️)
**[RUST_SDK.md](RUST_SDK.md)** - Generated Rust client (386 lines)

**What it provides:**
- High-performance auto-generated Rust client
- Custom error types (not `Box<dyn Error>`)
- Borrowing optimizations (`&str` vs `String`)
- Zero-cost abstractions with generics
- Pattern matching on enums
- Async/await optimized for Tokio
- Type-safe API with compile-time guarantees
- Auto-generated, stays in sync with server

**Quick Start:**
```rust
use mcp_client::{McpClient, HttpTransport, Result, McpError};

let transport = HttpTransport::new("http://127.0.0.1:8080");
let mut client = McpClient::new(transport);

client.connect().await?;
let result = client.echo("Hello, MCP!").await?;

// Pattern matching on custom errors
match client.ping().await {
    Ok(r) => println!("Success: {}", r),
    Err(McpError::Connection(e)) => eprintln!("Connection: {}", e),
    Err(e) => eprintln!("Error: {}", e),
}
```

---

## 🚀 Feature Comparison

| Feature | Lines | Language | Purpose | Quality |
|---------|-------|----------|---------|---------|
| Load Balancing | 876 | Rust | Enterprise HA | Production Ready |
| SDK Generators | ~4,400 | Rust | Client generation | Production Ready |
| TypeScript SDK | 209 | TypeScript | Web clients | Generated |
| Python SDK | 111 | Python | Python apps | Generated |
| Go SDK | 172 | Go | Go services | Generated |
| Rust SDK | 470 | Rust | Rust apps | Race Car 🏎️ |

---

## 📊 Feature Matrix

### Load Balancing

| Strategy | Use Case | Complexity | Performance |
|----------|----------|------------|-------------|
| Round-Robin | Equal backends | Simple | High |
| Least Connections | Varying loads | Medium | High |
| Random | Stateless | Simple | High |
| Weighted | Mixed capacity | Medium | High |
| IP Hash | Session affinity | Medium | High |

**Best for:** High availability, distributed systems, microservices

### SDK Generators

| SDK | Size | Dependencies | Type Safety | Quality |
|-----|------|--------------|-------------|---------|
| TypeScript | 209 lines | Zero | Full | Production |
| Python | 111 lines | requests | Type hints | Production |
| Go | 172 lines | stdlib | Full | Production |
| Rust | 470 lines | 6 crates | Full + Zero-cost | Race Car 🏎️ |

**Best for:** Multi-language environments, client distribution, API consistency

---

## 🎯 Use Cases

### High Availability Setup
**Use:** Load Balancing + Health Checks
```rust
let config = LoadBalancerConfig::new(Strategy::LeastConnections)
    .add_backend(Backend::new("primary".into(), "10.0.1.10:8080".into()))
    .add_backend(Backend::new("secondary".into(), "10.0.1.11:8080".into()))
    .add_backend(Backend::new("tertiary".into(), "10.0.1.12:8080".into()))
    .with_health_check(health_config)
    .with_failover(true);
```

### Multi-Language Client Support
**Use:** SDK Generators
```bash
# Generate all client SDKs
cd sdk-generators && cargo run --release

# Distribute to teams:
# - Frontend team: TypeScript SDK
# - Data team: Python SDK
# - Backend team: Go SDK
# - Systems team: Rust SDK
```

### Production Deployment
**Use:** Load Balancing + Rust SDK + Monitoring
```rust
// Server side: Load balanced backends
let lb = LoadBalancer::new(config);
lb.start_health_checks().await;

// Client side: High-performance Rust client
let client = McpClient::new(transport);
let stats = lb.get_stats().await;
```

---

## 🔧 Getting Started

### 1. Choose Your Feature

**Need high availability?**  
→ Start with [LOAD_BALANCING.md](LOAD_BALANCING.md)

**Need client libraries?**  
→ Start with [SDK_GENERATORS.md](SDK_GENERATORS.md)

**Building Rust apps?**  
→ Start with [RUST_SDK.md](RUST_SDK.md)

### 2. Follow the Guide

Each feature guide includes:
- Complete setup instructions
- Configuration examples
- Best practices
- Production deployment tips
- Troubleshooting

### 3. Test It Out

```bash
# Test load balancing
cargo test loadbalancer

# Generate SDKs
cd sdk-generators && cargo run --release

# Test Rust SDK
cd output/rust && cargo build
```

---

## 📖 Related Documentation

### Architecture
- [../architecture/SDK_COMPARISON.md](../architecture/SDK_COMPARISON.md) - SDK design decisions
- [../architecture/RUST_SDK_ARCHITECTURE.md](../architecture/RUST_SDK_ARCHITECTURE.md) - Why race car quality

### Guides
- [../guides/TESTING_GUIDE.md](../guides/TESTING_GUIDE.md) - Testing strategies
- [../guides/METRICS_GUIDE.md](../guides/METRICS_GUIDE.md) - Monitoring and metrics
- [../transports/](../transports/) - Transport documentation

### Reference
- [../reference/API.md](../reference/API.md) - Complete API reference
- [../reference/QUICK_REFERENCE.md](../reference/QUICK_REFERENCE.md) - Quick command reference

---

## 💡 Feature Combinations

### Enterprise Stack
```
Load Balancing + Multiple Backends + Health Checks + Metrics
→ Production-ready high availability
```

### Client Distribution
```
SDK Generators → TypeScript + Python + Go + Rust SDKs
→ Multi-language support out of the box
```

### Performance Optimization
```
Rust SDK + gRPC Transport + Load Balancing
→ Sub-5ms latency, 200 MB/s throughput
```

---

## 🎨 Examples

### Complete Load Balancing Setup
See [LOAD_BALANCING.md](LOAD_BALANCING.md#complete-example)

### Generate All SDKs
See [SDK_GENERATORS.md](SDK_GENERATORS.md#quick-start)

### Use Generated Rust SDK
See [RUST_SDK.md](RUST_SDK.md#usage-example)

---

## 🚦 Status

| Feature | Version | Status | Tests |
|---------|---------|--------|-------|
| Load Balancing | 0.5.0 | Production Ready | ✅ Passing |
| SDK Generators | 0.5.0 | Production Ready | ✅ Passing |
| Rust SDK | 0.5.0 | Production Ready | ✅ Generated |

---

**Version:** 0.5.0  
**Status:** All features production-ready  
**Maintained by:** NetADX Team