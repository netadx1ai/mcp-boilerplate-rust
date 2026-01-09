# Changelog

All notable changes to MCP Boilerplate Rust will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

---

## [0.5.1] - 2026-01-09

### Changed

#### HTTP Server Fix (2026-01-09)
- **HTTP Server Now Has 11 Tools** - Refactored HTTP server to use ProtocolHandler instead of hardcoded 3 tools
- **Consistent Tool Count** - Both Stdio and HTTP modes now expose all 11 tools
- **New HTTP Endpoints** - `/tools/call` for tool execution, `/rpc` for JSON-RPC

#### Code Cleanup (2026-01-09)
- **Zero Compiler Warnings** - Cleaned up all 48 warnings (unused imports, dead code)
- **Protocol Handler Simplified** - Removed unused `tool_router` and `processor` fields
- **Public API Annotations** - Added `#![allow(dead_code)]` for intentional public API types
- **Modules Updated** - transport/, loadbalancer/, metrics/ cleaned for feature-gated usage

#### Official SDK Patterns
- **Prompt Router Macros** - Refactored to use `#[prompt_router]` and `#[prompt_handler]` macros from rmcp SDK
- **Task Handler Enabled** - Enabled `#[task_handler]` macro for long-running task lifecycle support (SEP-1686)
- **Simplified Prompts** - Prompts now defined directly in McpServer using `#[prompt]` macro instead of manual PromptRegistry
- **Cleaner Architecture** - Single McpServer struct with both `tool_router` and `prompt_router` fields
- **ServerCapabilities** - Added `.enable_tasks()` to server capabilities

#### Code Quality
- **Removed PromptRegistry dependency** - Protocol handler now generates prompts inline
- **Updated prompts module** - Simplified to reference types only, actual prompts use macros
- **Fixed test issue** - Fixed temporary value borrowed error in loadbalancer tests
- **48 tests passing** - All unit tests pass

### Technical Details
- Uses `#[tool_router]` + `#[prompt_router]` on McpServer impl block
- Uses `#[tool_handler]` + `#[prompt_handler]` + `#[task_handler]` on ServerHandler impl
- Prompt argument types defined as structs with `JsonSchema` derive
- Binary size: 2.9MB (stdio only)
- Zero warnings in release build
- All 48 unit tests passing

### Integration Testing (2026-01-09)
- Verified stdio transport with manual JSON-RPC testing
- Verified with MCP Inspector CLI (`npx @modelcontextprotocol/inspector`)
- Verified HTTP server with curl testing
- `initialize` - Returns capabilities (tools, prompts, resources, tasks)
- `tools/list` - Returns 11 tools with schemas (MCP Inspector verified)
- `tools/call` - Execute tools correctly (tested calculate, echo, evaluate)
- `prompts/list` - Returns 3 prompts with arguments (MCP Inspector verified)
- `prompts/get` - Returns prompt messages with parameters
- `resources/list` - Returns 4 resources with icons and annotations (MCP Inspector verified)
- `resources/read` - Returns resource content (tested config://server)
- HTTP `/tools` - Returns 11 tools (fixed from 3)
- HTTP `/tools/call` - Execute tools via HTTP POST

---

## [0.5.0] - 2026-01-09

### Added

#### Generated Rust Client SDK (Race Car Edition 🏎️)
- **Auto-Generated Rust SDK** - High-performance generated Rust client (470 lines)
- **Race Car Quality** - Custom error types, borrowing optimizations, zero-cost abstractions
- **Not Generic Templates** - Idiomatic Rust code with `&str`, pattern matching, proper async/await
- **Custom Error Types** - `McpError` enum instead of `Box<dyn Error>` for pattern matching
- **Zero-Copy Optimizations** - Borrows `&str` instead of taking ownership `String`
- **Type-Safe API** - Pattern matching on enums, compile-time guarantees
- **Async/Await** - Optimized for Tokio runtime with proper patterns
- **Auto-Generated** - Stays in sync with server automatically
- **All 11 Tools** - Type-safe methods for all tools
- **Rust SDK Generator** - Code generator in `sdk-generators/src/generators/rust_gen.rs` (716 lines)
- **Location** - `sdk-generators/output/rust/`

#### Load Balancing
- **Enterprise Load Balancing** - Production-ready load balancer (~800 lines)
- **5 Strategies** - Round-robin, least connections, random, weighted round-robin, IP hash
- **Health Checking** - Automatic backend health monitoring with configurable intervals
- **Auto Failover** - Automatic failover to healthy backends with retry logic
- **Connection Management** - Per-backend connection limits and tracking
- **Sticky Sessions** - Session affinity support with cookie-based routing
- **Real-Time Statistics** - Total requests, success/failure rates, response times, active connections
- **Dynamic Management** - Add/remove backends at runtime, enable/disable backends
- **Location** - `src/loadbalancer/` module

#### Documentation Reorganization
- **Documentation Structure** - Reorganized `docs/` with professional hierarchy
- **Transport Docs** - Created `docs/transports/` with README, guides, and reference
- **Feature Docs** - Created `docs/features/` with README and feature guides
- **Load Balancing Guide** - Complete guide with examples and best practices (659 lines)
- **Rust SDK Guide** - Generated SDK documentation (386 lines)
- **SDK Comparison** - Architectural comparison (276 lines)
- **SDK Architecture** - Design decisions document (355 lines)
- **Comprehensive Indexes** - Created README.md for transports/ and features/
- **Removed Redundant** - Deleted hand-written `mcp-client/` (replaced by generated)
- **Removed 11 Files** - Consolidated redundant documentation

### Changed
- **Version** - Bumped to 0.5.0
- **Cargo.toml** - Added `rand` and `reqwest` dependencies
- **PROJECT_STATUS.md** - Updated with all v0.5.0 features
- **README.md** - Added Client SDKs and Load Balancing sections
- **docs/README.md** - Complete reorganization with new structure
- **CLAUDE.md** - Updated with v0.5.0 features and documentation paths
- **Total Code** - Now ~16,500 lines (reduced from 17,500 due to cleanup)
- **Total Documentation** - ~12,000 lines, professionally organized

### Removed
- **mcp-client/** - Hand-written Rust client (1,400 lines) - replaced by generated race car SDK
- **Redundant Docs** - 11 redundant documentation files
- **INDEX.md** - Consolidated into docs/README.md
- **Scattered Files** - Moved to organized structure in docs/

---

## [0.4.0] - 2026-01-09

### Added

#### Client SDK Generators
- **Auto-Generated Client Libraries** - TypeScript, Python, and Go client SDKs
- **TypeScript SDK** - 209 lines, full type safety, zero dependencies, Browser + Node.js compatible
- **Python SDK** - 111 lines, type hints with dataclasses, requests-based HTTP client
- **Go SDK** - 172 lines, idiomatic Go interfaces, stdlib only
- **SDK Generator Tool** - Rust-based code generator (715 lines)
- **Complete Documentation** - 3,700+ lines of docs, guides, and examples
- **Working Examples** - Comprehensive examples for all 3 languages
- **Integration Tests** - Full test suite for SDK generation and validation
- **All Tools Supported** - 11 tools with full type definitions across all SDKs
- **All Transports Supported** - 6 transport modes (SSE, WebSocket, HTTP, HTTP Stream, gRPC)
- **Build System** - Makefile and test scripts for easy SDK generation
- **Fast Generation** - Complete SDK generation in <500ms

#### Documentation Restructure
- **Reorganized docs/** - Cleaner structure with guides/, reference/, and development/
- **Documentation Index** - Comprehensive docs/README.md with navigation
- **SDK Documentation** - Complete SDK generator documentation (607 lines)
- **Quick Start Guides** - SDK-specific quick start documentation

### Changed
- **Project Structure** - Improved organization with sdk-generators/ directory
- **Documentation Layout** - Moved session notes to docs/development/
- **README.md** - Added Client SDK Generators section and updated documentation links

---

## [0.4.0] - 2026-01-09

### Added

#### Observability
- **Prometheus Metrics** - Comprehensive metrics collection (requests, tools, connections, errors)
- **Metrics Endpoint** - `/metrics` endpoint available on SSE, WebSocket, and HTTP Stream transports
- **Instrumentation** - Automated tracking for ProtocolHandler and all transports

#### Advanced Features
- **Progress Notifications** - Real-time updates during tool execution via `ProgressNotificationParam`
- **RequestContext Integration** - All 11 tools now use `RequestContext<RoleServer>` for bidirectional communication
- **Logging Notifications** - Structured logging during operations with `LoggingNotificationParam`
- **OperationProcessor** - Infrastructure for future task lifecycle support (SEP-1686)

#### New Tools (6 total)
- `process_with_progress` - Data processing with real-time progress tracking (10 updates)
- `batch_process` - Batch operations with progress + logging notifications
- `transform_data` - Array transformation (uppercase/lowercase/reverse/double, max 10K items)
- `simulate_upload` - File upload simulation with 20 chunks and progress
- `health_check` - System health monitoring (status, version, uptime, features)
- `long_task` - 10-second operation demonstrating progress notifications

#### Documentation (9,300+ lines)
- `START_HERE.md` - Main entry point for new users
- `docs/INDEX.md` - Comprehensive documentation navigation
- `docs/PROJECT_STRUCTURE.md` - Complete project structure guide
- `docs/guides/QUICK_START.md` - 5-minute setup guide
- `docs/guides/TESTING_GUIDE.md` - Comprehensive testing guide (620 lines)
- `docs/guides/ACTION_PLAN.md` - Step-by-step next actions (411 lines)
- `docs/reference/QUICK_REFERENCE.md` - Fast lookup guide (326 lines)
- `docs/advanced-features/DEEP_RESEARCH_IMPROVEMENTS.md` - Complete rust-sdk analysis (674 lines)
- `docs/advanced-features/SESSION_COMPLETE.md` - Implementation summary (461 lines)
- `docs/advanced-features/VISUAL_SUMMARY.md` - Visual overview (452 lines)
- `docs/advanced-features/IMPLEMENTATION_SUMMARY.md` - Technical details (438 lines)
- `docs/integration/INTEGRATION_GUIDE.md` - Complete integration guide (399 lines)
- `docs/troubleshooting/COMMON_ISSUES.md` - Consolidated troubleshooting (521 lines)
- `examples/advanced_features_demo.md` - Complete usage examples (507 lines)

#### Infrastructure
- Organized documentation structure (`docs/guides/`, `docs/reference/`, `docs/advanced-features/`)
- Enhanced test scripts with feature verification
- Comprehensive error handling with descriptive messages
- Input validation using `schemars` for type safety

### Changed

#### Breaking Changes
- All tool signatures now require `RequestContext<RoleServer>` parameter:
  ```rust
  async fn tool(
      params: Parameters<Request>,
      ctx: RequestContext<RoleServer>,  // NEW - required
  ) -> Result<Json<Response>, McpError>
  ```
  Impact: LOW - can use `_ctx` to ignore if not needed

#### Updated
- `src/mcp/stdio_server.rs` - Added RequestContext to all tools, progress notification support
- `src/tools/mod.rs` - Added advanced tools module
- `docs/reference/claude.md` - Updated with modern patterns and RequestContext guide
- `scripts/test_mcp.sh` - Enhanced with advanced feature verification
- README.md - Updated with v0.4.0-rc features and new documentation structure

#### Project Structure
- Root directory cleaned: 23+ MD files → 2 essential files (README, START_HERE)
- Documentation organized into `/docs/` with 6 categories
- Archived 17 historical session notes to `docs/archive/sessions/`
- Consolidated integration guides into single comprehensive guide
- Consolidated troubleshooting into single comprehensive guide

### Performance
- Memory: <5MB idle, <10MB active (no significant increase)
- CPU: <1% idle, 5-15% active
- Binary: 2.4MB (stdio), 3.1MB (HTTP) - unchanged
- Progress notifications: ~1ms overhead each
- Build time: ~30s clean build

### Fixed
- Build warnings suppressed for future task lifecycle components
- ANSI escape code issues in stdio mode (already fixed)
- Documentation duplication removed
- Confusing navigation paths clarified
- Emoji/icon usage removed for B2B professional style

### Removed
- Duplicate documentation files (6 files)
- Obsolete session checklists
- Redundant tool quick references
- Emojis and icons from all documentation

---

## [0.3.1] - 2026-01-08

### Added
- MCP Protocol 2025-03-26 support
- Icons for prompts and resources (7 SVG icons)
- Resource annotations (audience, priority, timestamps)
- Output schemas for all 5 tools (automatic JSON schema generation)
- Enhanced error handling with LLM-friendly messages
- Comprehensive security documentation (347 lines)

### Tools
- `echo` - Message validation (1-10KB)
- `ping` - Health check / connectivity test
- `info` - Server metadata
- `calculate` - Arithmetic operations
- `evaluate` - Math expression evaluator

### Prompts (3 with icons)
- `code_review` - Code review prompts (document icon)
- `explain_code` - Code explanation (help icon)
- `debug_help` - Debugging prompts (bug icon)

### Resources (4 with icons & annotations)
- `config://server` - Server configuration
- `info://capabilities` - MCP capabilities
- `doc://quick-start` - Quick start guide
- `stats://usage` - Usage statistics

### Testing
- 41 automated tests across 7 test suites
- Input validation tests
- Output schema verification
- Calculator tool tests
- Protocol compliance tests

---

## [0.3.0] - 2025-12-XX

### Added
- Initial production-ready release
- Dual transport support (stdio + HTTP)
- Basic tools (echo, ping, info)
- Type-safe implementation with Rust
- Official rmcp SDK integration

---

## Version Comparison

| Feature | v0.3.0 | v0.3.1 | v0.4.0 |
|---------|--------|--------|--------|
| Tools | 3 | 5 | 11 |
| Protocol | 2024-11-05 | 2025-03-26 | 2025-03-26 |
| Progress Notifications | No | No | Yes |
| RequestContext | No | No | Yes |
| Output Schemas | No | Yes | Yes |
| Icons & Annotations | No | Yes | No |
| Observability | No | No | Yes |
| Documentation Lines | ~1K | ~2K | ~9.3K |
| Test Coverage | Basic | Comprehensive | Enhanced |
| Professional B2B Style | No | No | Yes |

---

## Upgrade Guide

### From v0.3.1 to v0.4.0

**Breaking Changes:**
1. Update tool signatures to include `RequestContext`:
   ```rust
   // Old
   async fn my_tool(params: Parameters<Request>) 
       -> Result<Json<Response>, McpError>
   
   // New
   async fn my_tool(
       params: Parameters<Request>,
       ctx: RequestContext<RoleServer>,
   ) -> Result<Json<Response>, McpError>
   ```

2. Rebuild project:
   ```bash
   cargo clean
   cargo build --release
   ```

3. Update Claude Desktop config (if using relative paths, switch to absolute)

4. Restart Claude Desktop to see new tools

**New Features Available:**
- 6 new advanced tools with progress tracking
- Prometheus metrics collection
- Real-time progress notifications
- System health monitoring
- Comprehensive documentation

---

## Future Roadmap

### v0.4.1 (Next Patch)
- [ ] Resolve task_handler macro compatibility
- [ ] Complete task lifecycle implementation (SEP-1686)
- [ ] Production deployment guide
- [ ] Performance benchmarks (Criterion.rs)

### v0.5.0 (Planned)
- [ ] Elicitation support (interactive workflows)
- [ ] OAuth2 integration (production auth)
- [ ] Resource templates (dynamic URIs)
- [ ] Metrics and instrumentation
- [ ] Multi-transport examples (WebSocket, TCP)

### v1.0.0 (Long-term)
- [ ] WASI support
- [ ] Plugin system
- [ ] Advanced monitoring
- [ ] Production hardening
- [ ] Enterprise features

---

## Contributing

See [docs/reference/CONTRIBUTING.md](docs/reference/CONTRIBUTING.md) for contribution guidelines.

---

## Links

- **Repository:** https://github.com/netadx1ai/mcp-boilerplate-rust
- **Documentation:** See [docs/INDEX.md](docs/INDEX.md)
- **MCP Specification:** https://modelcontextprotocol.io
- **Rust SDK:** https://github.com/modelcontextprotocol/rust-sdk

---

**Maintained by:** NetAdx AI (https://netadx.ai)  
**License:** MIT  
**Contact:** hello@netadx.ai