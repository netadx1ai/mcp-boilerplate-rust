# MCP Boilerplate Rust

A comprehensive Rust implementation of the Model Context Protocol (MCP), providing a robust framework for building MCP servers with multiple transport layers and example implementations.

## Overview

This project serves as both a reference implementation and a boilerplate template for creating MCP servers in Rust. It demonstrates best practices for protocol implementation, transport abstraction, and tool integration.

## Architecture

The project is organized as a Cargo workspace with the following structure:

```
mcp-boilerplate-rust/
├── crates/
│   ├── mcp-core/          # Core protocol types and traits
│   ├── mcp-transport/     # Transport layer implementations
│   └── mcp-server/        # Server framework and orchestration
├── examples/
│   ├── filesystem-server/     # File system operations example
│   ├── image-generation-server/   # AI image generation example
│   ├── blog-generation-server/    # AI blog writing example
│   └── creative-content-server/   # Multi-tool creative AI example
└── sessions/              # Development session logs
```

### Core Components

- **mcp-core**: Defines the fundamental MCP protocol types, message structures, and traits
- **mcp-transport**: Provides transport layer implementations (STDIO, HTTP)
- **mcp-server**: Server framework for orchestrating tools and handling requests

### Design Principles

1. **Transport Agnostic**: Core protocol logic is independent of transport mechanisms
2. **Tool-Based Architecture**: All business logic implemented as discrete `McpTool` implementations
3. **Async First**: Built on Tokio with async/await throughout
4. **Type Safety**: Leverages Rust's type system for protocol correctness
5. **Extensible**: Easy to add new tools and transport methods

## Quick Start

### Prerequisites

- Rust 1.70+ with Cargo
- (Optional) GitHub CLI for development workflow

### Building

```bash
# Clone the repository
git clone https://github.com/netadx1ai/mcp-boilerplate-rust.git
cd mcp-boilerplate-rust

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Check code formatting and linting
cargo fmt --check
cargo clippy --workspace --all-targets
```

### Running Examples

#### Filesystem Server (STDIO Transport)
```bash
cargo run --bin filesystem-server -- --transport stdio
```

#### Filesystem Server (HTTP Transport)
```bash
cargo run --bin filesystem-server -- --transport http --port 3000
```

#### Test HTTP endpoint
```bash
curl -X POST http://localhost:3000/tools/read_file \
  -H "Content-Type: application/json" \
  -d '{"path": "README.md"}'
```

## Example Servers

### 1. Filesystem Server
Demonstrates basic file operations with proper error handling:
- `read_file`: Read file contents
- Supports both STDIO and HTTP transports
- Shows real-world tool implementation patterns

### 2. AI-Powered Servers (Scaffolded)
Ready-to-integrate examples with placeholder responses:
- **Image Generation**: Generate images from text prompts
- **Blog Generation**: Create blog posts from topics
- **Creative Content**: Multi-tool server with stories, poems, etc.

Each AI server includes:
- Realistic JSON response structures
- Multiple tool implementations
- Transport configuration
- Integration points for actual AI APIs

## Development

### Adding New Tools

1. Implement the `McpTool` trait:
```rust
use mcp_core::{McpTool, McpRequest, McpResponse, McpError};
use async_trait::async_trait;

#[derive(Default)]
pub struct MyCustomTool;

#[async_trait]
impl McpTool for MyCustomTool {
    async fn call(&self, request: McpRequest) -> Result<McpResponse, McpError> {
        // Your tool logic here
        todo!()
    }
    
    fn name(&self) -> &str {
        "my_custom_tool"
    }
    
    fn description(&self) -> &str {
        "Description of what this tool does"
    }
}
```

2. Register with the server:
```rust
use mcp_server::McpServerBuilder;

let server = McpServerBuilder::new()
    .add_tool(Box::new(MyCustomTool::default()))
    .build();
```

### Transport Support

Both STDIO and HTTP transports are supported:
- **STDIO**: Traditional pipe-based communication
- **HTTP**: RESTful API with JSON payloads

### Project Rules

This project follows strict development rules defined in `.rules`:
- **Verification Mandate**: All code must compile, test, and be verified before completion
- **Git Workflow**: Feature branches, conventional commits, GitHub issue integration
- **Documentation First**: All public APIs documented with examples
- **Quality Assurance**: Format, lint, and test before commit

## Contributing

1. Create feature branch: `git checkout -b feature/my-feature`
2. Follow conventional commit format: `feat(scope): description [#issue]`
3. Ensure all tests pass: `cargo test --workspace`
4. Update documentation as needed
5. Create pull request with verification proof

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Links

- [Repository](https://github.com/netadx1ai/mcp-boilerplate-rust)
- [Documentation](https://docs.rs/mcp-boilerplate-rust)
- [Issues](https://github.com/netadx1ai/mcp-boilerplate-rust/issues)