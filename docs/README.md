# MCP Boilerplate Rust - Documentation

Complete documentation for the MCP Boilerplate Rust project.

**Version:** 0.5.0  
**Status:** Production Ready  
**Last Updated:** 2026-01-09 HCMC

---

## 📚 Quick Navigation

### Getting Started
- [START_HERE](../START_HERE.md) - 5-minute quick start
- [Installation Guide](guides/INSTALLATION.md) - Detailed setup
- [Quick Start](guides/QUICK_START.md) - First steps

### Transports
- [Transports Overview](transports/) - All 6 transport modes
- [Quick Reference](transports/QUICK_REFERENCE.md) - API cheat sheet
- [Complete Guide](transports/GUIDE.md) - Detailed setup
- [Advanced Features](transports/ADVANCED.md) - Production patterns

### Features
- [Features Overview](features/) - All major features
- [Load Balancing](features/LOAD_BALANCING.md) - Enterprise load balancing
- [SDK Generators](features/SDK_GENERATORS.md) - Client SDK generation
- [Rust SDK](features/RUST_SDK.md) - Generated Rust client (Race Car Edition)

### Guides
- [Testing Guide](guides/TESTING_GUIDE.md) - Testing strategies
- [Metrics Guide](guides/METRICS_GUIDE.md) - Prometheus metrics
- [Transport Guide](guides/TRANSPORT_GUIDE.md) - Transport setup
- [Integration Guide](guides/integration/INTEGRATION_GUIDE.md) - Integration patterns
- [Claude Desktop Setup](guides/integration/CLAUDE_DESKTOP_SETUP.md) - Claude integration

### Reference
- [API Reference](reference/API.md) - Complete API documentation
- [Quick Reference](reference/QUICK_REFERENCE.md) - Command cheat sheet
- [Security Guide](reference/SECURITY.md) - Production security
- [Project Structure](reference/PROJECT_STRUCTURE.md) - Code organization

### Architecture
- [SDK Comparison](architecture/SDK_COMPARISON.md) - Generated vs hand-written
- [Rust SDK Architecture](architecture/RUST_SDK_ARCHITECTURE.md) - Design decisions

---

## 📖 Documentation Structure

```
docs/
├── README.md (this file)
│
├── transports/                      # Transport documentation
│   ├── README.md                   # Transport overview
│   ├── QUICK_REFERENCE.md          # API cheat sheet
│   ├── GUIDE.md                    # Complete guide
│   ├── ADVANCED.md                 # Advanced features
│   └── QUICK_START.md              # Quick start
│
├── features/                        # Feature documentation
│   ├── README.md                   # Features overview
│   ├── LOAD_BALANCING.md           # Load balancing (659 lines)
│   ├── SDK_GENERATORS.md           # SDK generation (607 lines)
│   └── RUST_SDK.md                 # Rust SDK (386 lines)
│
├── guides/                          # How-to guides
│   ├── QUICK_START.md
│   ├── INSTALLATION.md
│   ├── TESTING_GUIDE.md
│   ├── TRANSPORT_GUIDE.md
│   ├── METRICS_GUIDE.md
│   ├── integration/                # Integration guides
│   │   ├── CLAUDE_DESKTOP_SETUP.md
│   │   └── INTEGRATION_GUIDE.md
│   └── troubleshooting/            # Problem solving
│       └── COMMON_ISSUES.md
│
├── reference/                       # Reference documentation
│   ├── API.md
│   ├── QUICK_REFERENCE.md
│   ├── SECURITY.md
│   └── PROJECT_STRUCTURE.md
│
├── architecture/                    # Architectural decisions
│   ├── SDK_COMPARISON.md           # SDK comparison
│   └── RUST_SDK_ARCHITECTURE.md    # Rust SDK design
│
├── development/                     # Development notes
│   └── SESSION_*.md                # Development sessions
│
└── archive/                         # Historical documentation
    └── sessions/                   # Past development sessions
```

---

## 🚀 Key Features Documented

### 1. Multi-Transport Support (6 Modes)
- **Stdio** - Desktop applications, Claude Desktop
- **SSE** - Server-Sent Events for browser push
- **WebSocket** - Real-time bidirectional communication
- **HTTP** - Standard REST APIs
- **HTTP Streaming** - Large file transfers
- **gRPC** - High-performance microservices

📖 [Transports Overview](transports/) | [Quick Reference](transports/QUICK_REFERENCE.md)

### 2. Load Balancing
- 5 strategies (Round-robin, Least connections, Random, Weighted, IP hash)
- Automatic health checking
- Auto failover
- Connection management
- Real-time statistics

📖 [Load Balancing Guide](features/LOAD_BALANCING.md)

### 3. Client SDK Generators
- **TypeScript SDK** - Zero dependencies, Browser + Node.js
- **Python SDK** - Type hints, dataclasses
- **Go SDK** - Idiomatic Go, stdlib only
- **Rust SDK** - Race car edition with zero-cost abstractions

📖 [SDK Generators](features/SDK_GENERATORS.md) | [Rust SDK](features/RUST_SDK.md)

### 4. Observability
- Prometheus metrics
- OpenTelemetry tracing
- Structured logging
- Health endpoints

📖 [Metrics Guide](guides/METRICS_GUIDE.md)

---

## 🛠️ Common Tasks

### Build and Run

```bash
# Build minimal (stdio only)
cargo build --release

# Build with all features
cargo build --release --features full

# Run with specific transport
cargo run --release -- --mode stdio
cargo run --release --features sse -- --mode sse --bind 127.0.0.1:8025
```

### Generate Client SDKs

```bash
cd sdk-generators
cargo run --release

# Generates:
# - TypeScript: output/typescript/mcp-client.ts
# - Python: output/python/mcp_client.py
# - Go: output/go/mcpclient/client.go
# - Rust: output/rust/mcp_client.rs (Race Car Edition 🏎️)
```

### Testing

```bash
# Unit tests
cargo test --features full

# Integration tests
./scripts/integration_test.sh

# Test with MCP Inspector
npx @modelcontextprotocol/inspector ./target/release/mcp-boilerplate-rust --mode stdio
```

---

## 📊 Project Statistics

- **Transport Modes:** 6
- **Tools:** 11 production-ready
- **Client SDKs:** 4 (Rust, TypeScript, Python, Go)
- **Code:** ~17,500 lines
- **Documentation:** ~12,000 lines
- **Tests:** 89+ passing
- **Binary Size:** 2.4MB (minimal) to 4.2MB (full)

---

## 🎯 Quick Links

### Essential Docs
- [Project Status](../PROJECT_STATUS.md) - Complete project overview
- [Changelog](../CHANGELOG.md) - Version history
- [Claude Integration](../CLAUDE.md) - AI assistant guide

### Transport Docs
- [Transports Overview](transports/) - All transport documentation
- [Quick Reference](transports/QUICK_REFERENCE.md) - Transport API cheat sheet
- [Complete Guide](transports/GUIDE.md) - Transport setup guide
- [Advanced Features](transports/ADVANCED.md) - Advanced patterns

### Examples
- [Advanced Features Demo](../examples/advanced_features_demo.md)
- [SSE Test Client](../examples/sse_test_client.html)
- [WebSocket Test Client](../examples/websocket_test_client.html)

---

## 💡 Tips for Developers

### New to MCP?
1. Start with [START_HERE](../START_HERE.md)
2. Follow [Quick Start](guides/QUICK_START.md)
3. Try [Claude Desktop Setup](guides/integration/CLAUDE_DESKTOP_SETUP.md)

### Adding Features?
1. Read [Code Organization](reference/CODE_ORGANIZATION.md)
2. Check [API Reference](reference/API.md)
3. Review [Testing Guide](guides/TESTING_GUIDE.md)

### Deploying to Production?
1. Review [Security Guide](reference/SECURITY.md)
2. Setup [Load Balancing](features/LOAD_BALANCING.md)
3. Configure [Metrics](guides/METRICS_GUIDE.md)
4. Choose [Transport](transports/) based on your needs

### Troubleshooting?
1. Check [Common Issues](guides/troubleshooting/COMMON_ISSUES.md)
2. Review transport-specific guides
3. Check GitHub issues

---

## 🤝 Contributing

See [CONTRIBUTING](reference/CONTRIBUTING.md) for guidelines.

---

## 📝 Documentation Standards

- All guides include practical examples
- Code blocks show actual usage
- Performance metrics when relevant
- Troubleshooting sections included
- Last updated dates maintained

---

## 🔗 External Resources

- [MCP Specification](https://spec.modelcontextprotocol.io/)
- [Official Rust SDK](https://github.com/modelcontextprotocol/rust-sdk)
- [MCP Documentation](https://modelcontextprotocol.io/)

---

**Maintained by:** NetADX Team  
**Contact:** hello@netadx.ai  
**Website:** https://netadx.ai