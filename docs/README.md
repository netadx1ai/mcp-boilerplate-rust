# MCP Boilerplate Rust - Documentation

**Version:** 0.4.0  
**Last Updated:** 2026-01-09 HCMC

## Quick Navigation

### Getting Started
- [Quick Start Guide](guides/QUICK_START.md) - Get up and running in 5 minutes
- [Installation](guides/INSTALLATION.md) - Detailed installation instructions
- [Testing Guide](guides/TESTING_GUIDE.md) - How to test the server

### Core Features
- [Transport Guide](guides/TRANSPORT_GUIDE.md) - All 6 transport modes
- [SDK Generators](SDK_GENERATORS.md) - Auto-generate client SDKs (TypeScript, Python, Go)
- [Metrics Guide](guides/METRICS_GUIDE.md) - Prometheus metrics and monitoring

### User Guides
- **Integration**
  - [Claude Desktop Setup](guides/integration/CLAUDE_DESKTOP_SETUP.md)
  - [Integration Guide](guides/integration/INTEGRATION_GUIDE.md)
  - [HTTP Wrapper Integration](guides/integration/HTTP_WRAPPER_INTEGRATION.md)

- **Troubleshooting**
  - [Common Issues](guides/troubleshooting/COMMON_ISSUES.md)
  - [JSON Error Fixes](guides/troubleshooting/TROUBLESHOOTING_JSON_ERROR.md)
  - [ANSI Escape Codes](guides/troubleshooting/FIX_ANSI_ESCAPE_CODES.md)

### Reference
- [API Reference](reference/API.md) - Complete API documentation
- [Quick Reference](reference/QUICK_REFERENCE.md) - Command cheat sheet
- [Code Organization](reference/CODE_ORGANIZATION.md) - Project structure
- [Security](reference/SECURITY.md) - Security best practices
- [Output Schemas](reference/OUTPUT_SCHEMAS.md) - Response schemas
- [Contributing](reference/CONTRIBUTING.md) - How to contribute

### Transport Documentation
- [Transport Quick Start](TRANSPORT_QUICK_START.md) - Get started with transports
- [Transport Quick Reference](TRANSPORT_QUICK_REFERENCE.md) - Transport API reference
- [Transport Quick Guide](TRANSPORT_QUICK_GUIDE.md) - Detailed transport guide
- [Transport Advanced Summary](TRANSPORT_ADVANCED_SUMMARY.md) - Advanced features

### Development
- [Project Structure](PROJECT_STRUCTURE.md) - Architecture overview
- [Development Sessions](development/) - Implementation notes
- [Git Workflow](guides/GIT_WORKFLOW.md) - Git best practices
- [Action Plan](guides/ACTION_PLAN.md) - Development roadmap

## Documentation Structure

```
docs/
├── README.md                          # This file
├── SDK_GENERATORS.md                  # SDK generator documentation
├── TRANSPORT_*.md                     # Transport documentation
├── PROJECT_STRUCTURE.md               # Architecture
├── INDEX.md                           # Legacy index
│
├── guides/                            # User guides
│   ├── QUICK_START.md
│   ├── INSTALLATION.md
│   ├── TESTING_GUIDE.md
│   ├── TRANSPORT_GUIDE.md
│   ├── METRICS_GUIDE.md
│   ├── GIT_WORKFLOW.md
│   ├── ACTION_PLAN.md
│   │
│   ├── integration/                   # Integration guides
│   │   ├── CLAUDE_DESKTOP_SETUP.md
│   │   ├── INTEGRATION_GUIDE.md
│   │   └── HTTP_WRAPPER_INTEGRATION.md
│   │
│   └── troubleshooting/              # Troubleshooting guides
│       ├── COMMON_ISSUES.md
│       ├── TROUBLESHOOTING_JSON_ERROR.md
│       ├── FIX_ANSI_ESCAPE_CODES.md
│       ├── FIX_ESM_REQUIRE.md
│       └── FIX_NODE_VERSION.md
│
├── reference/                         # API reference
│   ├── API.md
│   ├── QUICK_REFERENCE.md
│   ├── CODE_ORGANIZATION.md
│   ├── SECURITY.md
│   ├── OUTPUT_SCHEMAS.md
│   ├── CONTRIBUTING.md
│   ├── AI_TOOL_PATTERN.md
│   ├── PROMPTS_AND_RESOURCES.md
│   └── FILE_SIZE_ENFORCEMENT.md
│
├── development/                       # Development documentation
│   ├── SDK_GENERATORS_COMPLETE.md
│   ├── advanced-features/
│   └── session-notes/
│
└── archive/                          # Archived documentation
    └── old-sessions/
```

## Quick Links by Task

### I want to...

**Get started quickly**
→ [Quick Start Guide](guides/QUICK_START.md)

**Set up for Claude Desktop**
→ [Claude Desktop Setup](guides/integration/CLAUDE_DESKTOP_SETUP.md)

**Use a different transport (SSE, WebSocket, gRPC, etc.)**
→ [Transport Guide](guides/TRANSPORT_GUIDE.md)

**Generate client SDKs**
→ [SDK Generators](SDK_GENERATORS.md)

**Fix JSON parsing errors**
→ [Troubleshooting JSON Error](guides/troubleshooting/TROUBLESHOOTING_JSON_ERROR.md)

**Monitor performance**
→ [Metrics Guide](guides/METRICS_GUIDE.md)

**Add new tools**
→ [AI Tool Pattern](reference/AI_TOOL_PATTERN.md)

**Contribute to the project**
→ [Contributing Guide](reference/CONTRIBUTING.md)

**Deploy to production**
→ [Security Guide](reference/SECURITY.md)

## Transport Modes (6 Total)

| Transport | Port | Use Case | Documentation |
|-----------|------|----------|---------------|
| stdio | N/A | Desktop apps, Claude | [Quick Start](guides/QUICK_START.md) |
| SSE | 8025 | Browser push | [Transport Guide](guides/TRANSPORT_GUIDE.md) |
| WebSocket | 9001 | Real-time chat | [Transport Guide](guides/TRANSPORT_GUIDE.md) |
| HTTP | 8080 | REST APIs | [Transport Guide](guides/TRANSPORT_GUIDE.md) |
| HTTP Stream | 8026 | Large files | [Transport Guide](guides/TRANSPORT_GUIDE.md) |
| gRPC | 50051 | Microservices | [Transport Guide](guides/TRANSPORT_GUIDE.md) |

## Client SDKs

Auto-generated client libraries available:
- **TypeScript** - Full type safety, zero dependencies
- **Python** - Type hints, minimal dependencies
- **Go** - Idiomatic Go, stdlib only

See [SDK Generators](SDK_GENERATORS.md) for details.

## Tools (11 Total)

### Basic Tools (5)
- `ping` - Health check
- `echo` - Message validation
- `info` - Server metadata
- `calculate` - Arithmetic operations
- `evaluate` - Expression evaluation

### Advanced Tools (6)
- `process_with_progress` - Progress notifications
- `batch_process` - Batch operations
- `transform_data` - Data transformations
- `simulate_upload` - Upload simulation
- `health_check` - System health
- `long_task` - Long-running tasks

## Support

- **GitHub:** https://github.com/netadx/mcp-boilerplate-rust
- **Issues:** https://github.com/netadx/mcp-boilerplate-rust/issues
- **Email:** hello@netadx.ai
- **Website:** https://netadx.ai

## License

MIT License - see [LICENSE](../LICENSE) file for details.

---

**Maintained by:** NetADX Team  
**Status:** Production Ready  
**Version:** 0.4.0