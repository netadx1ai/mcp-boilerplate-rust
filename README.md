# MCP Server Ecosystem - Official RMCP SDK

[![Rust](https://img.shields.io/badge/rust-1.75+-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![RMCP SDK](https://img.shields.io/badge/RMCP-v0.6.3-green.svg)](https://github.com/modelcontextprotocol/rust-sdk)
[![NetADX](https://img.shields.io/badge/Powered%20by-NetADX.ai-purple.svg)](https://netadx.ai)

A production-ready MCP (Model Context Protocol) server ecosystem built on the official RMCP SDK, delivering specialized servers, reusable templates, and complete deployment infrastructure.

> 🚀 **Professional AI Solutions**: Need custom AI integrations? [NetADX.ai](https://netadx.ai) offers enterprise AI customizer services, from proof-of-concept to production deployment. Transform your business with tailored AI solutions!

> 💡 **Looking for TypeScript?** Check out our [TypeScript version](https://github.com/netadx1ai/mcp-boilerplate-ts) for Node.js environments with the same features and API compatibility!

## 🚀 Quick Start

```bash
# Clone and build
git clone https://github.com/netadx1ai/mcp-boilerplate-rust.git
cd mcp-boilerplate-rust
cargo build --workspace --release

# Run a server
./target/release/news-data-server --port 8001

# Test the API
curl http://localhost:8001/mcp/tools/list
```

## 📋 Production Servers

| Server | Purpose | Tools | Port |
|--------|---------|-------|------|
| **news-data-server** | Real-time news & trends | 5 tools | 8001 |
| **template-server** | Content templates & rendering | 7 tools | 8002 |
| **analytics-server** | Metrics & performance data | 7 tools | 8003 |
| **database-server** | Query & data access | 7 tools | 8004 |
| **api-gateway-server** | External API integration | 5 tools | 8005 |
| **workflow-server** | Task automation | 7 tools | 8006 |

### News Data Server
Real-time news and trends data provider with multi-language support.

```bash
./target/release/news-data-server --port 8001
curl -X POST http://localhost:8001/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{"name": "search_news", "arguments": {"query": "AI", "limit": 5}}'
```

**Available Tools**: `search_news`, `get_category_news`, `get_trending_news`, `get_categories`, `get_server_status`

### Template Server
Content templates and structure provider with Handlebars rendering.

```bash
./target/release/template-server --port 8002
curl -X POST http://localhost:8002/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{"name": "render_template", "arguments": {"template_id": "blog_post", "params": {"title": "My Blog"}}}'
```

**Available Tools**: `list_templates`, `get_template`, `render_template`, `validate_template_params`, `create_template`, `get_categories`

### Analytics Server
Metrics and performance data provider with business intelligence.

```bash
./target/release/analytics-server --port 8003
curl -X POST http://localhost:8003/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{"name": "get_content_metrics", "arguments": {"content_id": "blog_123", "period": "week"}}'
```

**Available Tools**: `get_content_metrics`, `get_audience_insights`, `get_engagement_trends`, `generate_analytics_report`, `get_available_metrics`

## 🛠️ Server Templates

Copy-paste ready templates for rapid MCP server development:

### Basic Server Template
```bash
cd templates/basic-server-template
cargo run -- --help
```
- Minimal MCP server implementation
- 4 example tools with async patterns
- Complete development setup

### API Wrapper Template
```bash
cd templates/api-wrapper-template
cargo run -- --help
```
- External API integration patterns
- 5 authentication methods (API Key, OAuth, Bearer, Basic, Custom)
- Rate limiting and circuit breaker

### Database Integration Template
```bash
cd templates/database-integration-template
cargo run -- --help
```
- Multi-database support (PostgreSQL, MySQL, SQLite)
- SQL injection protection
- Connection pooling patterns

### Authenticated Server Template
```bash
cd templates/authenticated-server-template
cargo run -- --help
```
- OAuth integration examples
- Session management
- Authorization middleware

## 🚀 Deployment

### Docker
```bash
cd deployment/docker
./build.sh --build-all
./build.sh --dev-up
```

### Kubernetes
```bash
cd deployment/kubernetes
./deploy.sh --apply-all
```

### Monitoring
```bash
cd deployment/monitoring
./deploy.sh --monitoring
```

## 📖 Documentation

- **[Server Development Guide](docs/SERVER_DEVELOPMENT_GUIDE.md)** - Build custom servers
- **[Deployment Guide](docs/DEPLOYMENT_GUIDE.md)** - Docker + Kubernetes
- **[Performance Guide](docs/PERFORMANCE_GUIDE.md)** - Optimization tips
- **[Security Guide](docs/SECURITY_GUIDE.md)** - Security best practices
- **[API Reference](docs/API_REFERENCE.md)** - Complete API documentation

## 🔧 Development

### Prerequisites
- Rust 1.75+
- Docker (for deployment)
- Kubernetes (optional, for production)

### Build
```bash
# Build all servers
cargo build --workspace --release

# Run tests
cargo test --workspace

# Development commands
just dev-server news-data-server
just test-all
just quality-check
```

### Create Your Own Server
1. Copy a template:
```bash
cp -r templates/basic-server-template my-server
cd my-server
```

2. Implement your tools:
```rust
use mcp_core::Tool;

#[derive(Default)]
pub struct MyTool;

impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }
    fn description(&self) -> &str { "My custom tool" }
    // ... implement tool logic
}
```

3. Register with server:
```rust
let server = ServerBuilder::new()
    .with_tool(Box::new(MyTool::default()))
    .build();
```

## 🏗️ Architecture

Built on the official RMCP SDK with clean separation of concerns:

```
mcp-boilerplate-rust/
├── servers/           # Production MCP servers
├── templates/         # Reusable server templates
├── deployment/        # Docker + K8s + monitoring
├── examples/          # Integration examples
├── docs/             # Comprehensive documentation
└── tests/            # Integration test suite
```

### Key Design Principles
- **Official SDK**: Built on RMCP SDK v0.6.3
- **Production Ready**: Enterprise-grade quality and security
- **Template Driven**: Rapid development through templates
- **Deployment Focused**: Complete automation and monitoring
- **Community First**: Open source and contribution-friendly

## 📊 Performance

- **Response Times**: < 50ms (production verified)
- **Build Times**: 4-9 seconds per server
- **Memory Usage**: < 50MB per server
- **Startup Time**: < 2 seconds
- **Test Suite**: 100% pass rate, < 5 seconds execution

## 🔒 Security

- ✅ SQL injection protection
- ✅ Input validation and sanitization
- ✅ Rate limiting and circuit breakers
- ✅ Secure authentication patterns
- ✅ Container security (non-root, read-only)
- ✅ Kubernetes Pod Security Standards

## 🔄 TypeScript Version Available

This Rust implementation has a companion [TypeScript version](https://github.com/netadx1ai/mcp-boilerplate-ts) that provides:

- **Same API**: Identical tool interfaces and responses
- **Same Architecture**: Equivalent server templates and deployment options
- **Same Performance Class**: Comparable response times for most use cases
- **Enhanced DX**: Better debugging and IDE integration with TypeScript
- **Node.js Ecosystem**: Access to npm packages and familiar tooling

### Performance Comparison

| Feature | Rust Version | TypeScript Version |
|---------|-------------|-------------------|
| **Response Time** | ~20ms | ~50ms |
| **Memory Usage** | ~30MB per server | ~100MB per server |
| **Build Time** | 4-9 seconds | 3-8 seconds |
| **Type Safety** | Compile-time | Compile-time |
| **Runtime** | Native binary | Node.js |
| **Hot Reload** | Manual restart | Automatic |
| **IDE Support** | Good | Excellent |
| **Learning Curve** | Steep | Moderate |

**Choose Rust for**: Maximum performance, minimal resource usage, systems programming teams
**Choose TypeScript for**: Rapid development, web development teams, Node.js ecosystem integration

## 🚀 Enterprise AI Solutions by NetADX.ai

Looking to accelerate your AI journey? [**NetADX.ai**](https://netadx.ai) offers comprehensive AI customizer services to transform your business:

### 🎯 Our Services
- **Custom AI Model Development** - Tailored models for your specific use cases
- **AI Integration Consulting** - Seamless integration with existing systems
- **Proof-of-Concept Development** - Rapid prototyping and validation
- **Production Deployment** - Enterprise-grade AI solutions at scale
- **Training & Support** - Comprehensive team training and ongoing support

### 🌟 Why Choose NetADX.ai?
- **Proven Expertise** - Deep experience in AI/ML and enterprise software
- **Open Source First** - Building on solid, community-driven foundations
- **End-to-End Solutions** - From concept to production deployment
- **Industry Agnostic** - Serving healthcare, finance, retail, manufacturing, and more
- **Scalable Architecture** - Solutions that grow with your business

### 📞 Get Started Today
Ready to unlock the power of AI for your organization? 

**🌐 Visit**: [https://netadx.ai](https://netadx.ai)  
**📧 Contact**: [hello@netadx.ai](mailto:hello@netadx.ai)  
**📅 Book Consultation**: Free 30-minute discovery call available

*"Empowering businesses through intelligent automation and custom AI solutions"*

---

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes and test: `just test-all`
4. Submit a pull request

### Development Standards
- All code must pass `cargo clippy` with zero warnings
- All tests must pass: `cargo test --workspace`
- Format code: `cargo fmt --all`
- Document public APIs with examples

## 📄 License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- [RMCP SDK Team](https://github.com/modelcontextprotocol/rust-sdk) for the excellent official implementation
- [Model Context Protocol](https://modelcontextprotocol.io/) for the specification
- [TypeScript Version](https://github.com/netadx1ai/mcp-boilerplate-ts) for cross-language compatibility validation
- [NetADX.ai](https://netadx.ai) for sponsoring open source development and enterprise AI innovation
- Rust community for the incredible ecosystem

---

**Ready for Production** | **Enterprise Quality** | **Community Driven** | **Powered by [NetADX.ai](https://netadx.ai)**

Start building your MCP integration today! 🚀

### 🌟 Open Source Commitment
This project is part of NetADX.ai's commitment to open source innovation. We believe in:
- **Transparent Development** - All code is open and community-driven
- **Knowledge Sharing** - Contributing to the global AI ecosystem
- **Collaborative Growth** - Building better solutions together
- **Accessible Technology** - Making enterprise-grade AI tools available to everyone

Join our mission to democratize AI technology while offering professional services for those who need customized solutions.