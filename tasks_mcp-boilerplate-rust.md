# MCP Boilerplate Rust - MVP Tasks

## Project Overview
Implementation of a functional Minimum Viable Product (MVP) of the MCP boilerplate template in Rust, serving as a "vertical slice" demonstrating end-to-end working application with both simple and advanced use cases.

## Task Breakdown

### Phase 1: Core Foundation (120 minutes) [#1]

#### Task 1.1: Project Structure & Workspace Setup (20 minutes) [#2]
- [ ] Create workspace root structure [commit: ]
  - [ ] Initialize Cargo.toml workspace configuration
  - [ ] Set up workspace members: mcp-core, mcp-transport, mcp-server
  - [ ] Create examples directory structure
  - [ ] Add basic .gitignore and README.md
- [ ] Set up development environment [commit: ]
  - [ ] Configure workspace dependencies and features
  - [ ] Add common development dependencies (serde, tokio, anyhow, thiserror)
  - [ ] Verify workspace compiles with `cargo check --workspace`

#### Task 1.2: mcp-core Crate Implementation (45 minutes) [#3]
- [ ] Create core protocol message types [commit: ]
  - [ ] Implement McpRequest enum with all protocol message variants
  - [ ] Implement McpResponse enum with success/error variants
  - [ ] Implement McpError struct with proper error handling
  - [ ] Add serde serialization/deserialization support
- [ ] Define core traits [commit: ]
  - [ ] Create McpServer trait with async method signatures
  - [ ] Create McpTool trait for tool implementations
  - [ ] Add proper error handling and Result types
  - [ ] Add comprehensive documentation with examples
- [ ] Verification [commit: ]
  - [ ] Run `cargo check --package mcp-core`
  - [ ] Run `cargo test --package mcp-core`
  - [ ] Run `cargo doc --package mcp-core --no-deps`

#### Task 1.3: mcp-transport Crate Implementation (55 minutes) [#4]
- [ ] STDIO transport implementation [commit: ]
  - [ ] Create StdioTransport struct
  - [ ] Implement async read/write for stdin/stdout
  - [ ] Add JSON message parsing and serialization
  - [ ] Handle transport-level error cases
- [ ] HTTP transport implementation [commit: ]
  - [ ] Set up axum-based HTTP server
  - [ ] Create RESTful endpoints for MCP protocol
  - [ ] Implement JSON request/response handling
  - [ ] Add proper HTTP status code mapping
- [ ] Transport abstraction [commit: ]
  - [ ] Create Transport trait for common interface
  - [ ] Implement transport factory/builder pattern
  - [ ] Add configuration structures
  - [ ] Add comprehensive error handling
- [ ] Verification [commit: ]
  - [ ] Run `cargo check --package mcp-transport`
  - [ ] Run `cargo test --package mcp-transport`
  - [ ] Test STDIO transport with sample messages

### Phase 2: Server Framework (60 minutes) [#5]

#### Task 2.1: mcp-server Crate Implementation (45 minutes) [#6]
- [ ] Basic server framework [commit: ]
  - [ ] Create McpServerBuilder struct
  - [ ] Implement server configuration and setup
  - [ ] Add tool registration and management
  - [ ] Create server runtime with tokio
- [ ] Request routing and handling [commit: ]
  - [ ] Implement request dispatching to appropriate tools
  - [ ] Add proper error propagation and response handling
  - [ ] Create middleware-ready architecture (future extensibility)
  - [ ] Add logging and basic observability
- [ ] Verification [commit: ]
  - [ ] Run `cargo check --package mcp-server`
  - [ ] Run `cargo test --package mcp-server`
  - [ ] Verify server can start with no tools registered

#### Task 2.2: Integration Testing (15 minutes) [#7]
- [ ] Cross-crate integration verification [commit: ]
  - [ ] Test mcp-core types work with mcp-transport
  - [ ] Test mcp-server can use both transport types
  - [ ] Run `cargo check --workspace`
  - [ ] Run `cargo build --workspace`

### Phase 3: Example Implementations (90 minutes) [#8]

#### Task 3.1: Filesystem Example Server (30 minutes) [#9]
- [ ] Create filesystem server structure [commit: ]
  - [ ] Set up example crate with proper dependencies
  - [ ] Create ReadFileTool implementing McpTool trait
  - [ ] Add file reading logic with proper error handling
  - [ ] Add CLI argument parsing for transport selection
- [ ] Integration with server framework [commit: ]
  - [ ] Register ReadFileTool with McpServer
  - [ ] Configure both STDIO and HTTP transport options
  - [ ] Add proper logging and error reporting
  - [ ] Create example README with usage instructions
- [ ] Verification [commit: ]
  - [ ] Run filesystem server with STDIO transport
  - [ ] Test read_file request/response cycle
  - [ ] Run filesystem server with HTTP transport
  - [ ] Test HTTP endpoint with curl

#### Task 3.2: AI Example Servers Scaffolding (45 minutes) [#10]
- [ ] Image Generation Server [commit: ]
  - [ ] Create image-generation example crate
  - [ ] Implement GenerateImageTool with placeholder logic
  - [ ] Add hardcoded realistic JSON response
  - [ ] Configure server with both transports
- [ ] Blog Generation Server [commit: ]
  - [ ] Create blog-generation example crate
  - [ ] Implement CreateBlogPostTool with placeholder logic
  - [ ] Add hardcoded realistic JSON response
  - [ ] Configure server with both transports
- [ ] Creative Content Server [commit: ]
  - [ ] Create creative-content example crate
  - [ ] Implement multiple tools: GenerateStory, CreatePoem, etc.
  - [ ] Add hardcoded realistic JSON responses for each tool
  - [ ] Configure server as multi-tool example
- [ ] Verification [commit: ]
  - [ ] Run each AI example server
  - [ ] Test HTTP endpoints return valid JSON responses
  - [ ] Verify all examples compile and start correctly

#### Task 3.3: End-to-End Testing (15 minutes) [#11]
- [ ] Final integration testing [commit: ]
  - [ ] Test all example servers with both transports
  - [ ] Verify request/response cycles work end-to-end
  - [ ] Run comprehensive workspace checks
  - [ ] Document any known limitations or TODO items

### Phase 4: Documentation & Finalization (30 minutes) [#12]

#### Task 4.1: Project Documentation (20 minutes) [#13]
- [ ] Root project documentation [commit: ]
  - [ ] Update main README.md with architecture overview
  - [ ] Add getting started guide
  - [ ] Document transport options and usage
  - [ ] Add example usage for each server
- [ ] Crate-level documentation [commit: ]
  - [ ] Ensure all public APIs have doc comments
  - [ ] Add crate-level documentation for each library
  - [ ] Verify `cargo doc --workspace --no-deps` builds successfully
  - [ ] Add inline examples that compile and test

#### Task 4.2: Final Verification & Cleanup (10 minutes) [#14]
- [ ] Complete workspace verification [commit: ]
  - [ ] Run `cargo fmt --all`
  - [ ] Run `cargo clippy --workspace --all-targets`
  - [ ] Run `cargo check --workspace --all-targets`
  - [ ] Run `cargo build --workspace --all-targets`
  - [ ] Run `cargo test --workspace`
- [ ] Success criteria validation [commit: ]
  - [ ] Verify project compiles successfully
  - [ ] Confirm filesystem example works with both transports
  - [ ] Verify read_file request/response cycle
  - [ ] Confirm AI examples are runnable and return valid responses
  - [ ] Document completion status and any remaining TODOs

## Success Criteria Checklist

- [ ] Project compiles successfully using `cargo build`
- [ ] Filesystem example is runnable via both STDIO and HTTP transports
- [ ] Filesystem server correctly processes read_file request and returns content/error
- [ ] AI example servers (image-generation, blog-generation, creative-content) are runnable
- [ ] AI example servers return valid hardcoded placeholder responses via HTTP
- [ ] All code is properly formatted, linted, and documented
- [ ] Integration between all crates works correctly
- [ ] Proper Git workflow with meaningful commits and issue tracking

## Estimated Total Time: 300 minutes (5 hours)

## Notes
- Each task should be completed on a feature branch following naming convention
- All commits should follow conventional commit format with issue references
- Verification steps must be run and output provided as proof of completion
- GitHub issues should be created for each major task and updated throughout