# Changelog

All notable changes to MCP Boilerplate Rust will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

---

## [0.6.3] - 2026-01-09

### Added

#### Integration Tests
- Cross-module integration tests for elicitation, sampling, and structured content
- End-to-end workflow tests:
  - `test_elicitation_form_workflow` - Complete form elicitation lifecycle
  - `test_elicitation_url_workflow` - URL-based OAuth flow
  - `test_elicitation_with_enums` - Single/multi-select enum handling
  - `test_sampling_with_tools_workflow` - Tool-enabled sampling
  - `test_sampling_session_multi_turn` - Multi-turn conversation
  - `test_tool_executor_registry` - Tool registration and execution
  - `test_structured_output_with_validation` - Schema validation
  - `test_complex_nested_validation` - Nested object validation
  - `test_elicitation_to_sampling_workflow` - Cross-module flow
  - `test_sampling_with_structured_output` - Sampling + validation
  - `test_error_handling_across_modules` - Error scenarios
  - `test_tool_choice_variations` - All ToolChoice modes

### Testing
- 108 unit tests passing (up from 94)
- Integration test module: `src/mcp/integration_tests.rs`
- Full coverage of module interactions

---

## [0.6.2] - 2026-01-09

### Added

#### Elicitation Support (Priority 4)

**URL Mode Elicitation**
- `ElicitationMode::Form` - Form-based data collection with JSON Schema
- `ElicitationMode::Url` - URL-based collection for sensitive data (OAuth, payments)
- `ElicitationRequest::url()` and `ElicitationRequest::url_with_callback()`
- Callback URL support for completion notifications
- Timeout configuration for elicitation requests

**Enhanced Enum Support**
- Single-select enums (titled and untitled)
- Multi-select enums with min/max items constraints
- `with_enum_field()`, `with_titled_enum_field()`, `with_multiselect_enum_field()`
- Full integration with rmcp's `EnumSchema` and `EnumSchemaBuilder`

**Form Builder API**
- `ElicitationFormBuilder` for fluent form construction
- `with_string_field()`, `with_email_field()`, `with_number_field()`
- `with_integer_field()`, `with_boolean_field()`
- All fields support required/optional and description
- Number/integer fields support min/max constraints

**Elicitation Manager**
- Track pending elicitations by ID
- Complete/cancel elicitation workflows
- List pending elicitations
- Automatic timeout cleanup

#### Sampling with Tool Support (Priority 5)

**Tool Calling in Sampling**
- `SamplingTool` - Define tools available during sampling
- `ToolChoice` enum: Auto, None, Required, Tool(name)
- `ToolCall` and `ToolCallResult` types for tool execution
- Extended metadata in sampling requests for tools/toolChoice

**Sampling Request Builder**
- `SamplingRequest::new()` with fluent builder API
- `add_user_message()`, `add_assistant_message()`, `add_message()`
- `with_tools()`, `add_tool()`, `with_tool_choice()`
- `with_max_tokens()`, `with_temperature()`, `with_stop_sequences()`
- Model preferences: `prefer_cost()`, `prefer_speed()`, `prefer_intelligence()`

**Sampling Session**
- Multi-turn conversation management
- Tool result integration
- Request building from session state
- Response processing and history tracking

**Tool Executor Registry**
- Register tool handlers by name
- Execute tool calls with argument validation
- Function-based tool executors

#### Structured Content Validation (Priority 7)

**Output Schema Validator**
- JSON Schema validation for tool outputs
- Type checking: string, number, integer, boolean, array, object
- Constraint validation: min/max, minLength/maxLength, enum, required
- Nested object and array validation
- Clear validation error messages with paths

**Structured Output Builder**
- `StructuredOutput::new()` fluent builder
- Combine human-readable text with machine-parseable structured data
- Automatic text representation for structured-only outputs
- `build_validated()` for schema-enforced outputs

**Output Schema Registry**
- Register schemas per tool name
- Validate outputs against registered schemas
- Pre-built schemas: `OutputSchemas::weather()`, `OutputSchemas::api_response()`

### Changed
- `ProtocolHandler` now includes `ElicitationManager`
- Module exports updated in `mcp/mod.rs`
- 94 tests (up from 77 in v0.6.1)

### Technical Details
- Elicitation uses rmcp's `ElicitationSchema`, `ElicitationSchemaBuilder`
- Sampling extends `CreateMessageRequestParam` with tool metadata
- Structured content validation is a lightweight JSON Schema subset
- All new modules follow existing code patterns

### Testing
- 94 unit tests passing
- Elicitation: form builder, URL mode, manager lifecycle
- Sampling: request builder, tool choice serialization, session management
- Structured content: type validation, constraints, registry

---

## [0.6.1] - 2026-01-09

### Added

#### Integration Work Completed (Priority 6)

**OAuth/Well-Known Routers Mounted in main.rs**
- OAuth 2.1 routes mounted at `/oauth` (authorize, token, register, introspect, revoke)
- Well-known metadata routes mounted at `/.well-known`:
  - `/.well-known/oauth-authorization-server` (RFC 8414)
  - `/.well-known/openid-configuration` (OIDC Discovery)
  - `/.well-known/oauth-protected-resource` (RFC 9728)

**Task Endpoints Integrated with Protocol Handler**
- `tasks/list` - List active tasks with optional filters
- `tasks/get` - Get task status by ID
- `tasks/result` - Retrieve completed task result
- `tasks/cancel` - Cancel a running/pending task
- Task capabilities advertised in `initialize` response

**Tool Metadata Integrated with tools/list Response**
- `outputSchema` included for tools with defined output schemas
- `_meta.taskSupport` - "required", "optional", or "forbidden"
- `_meta.supportsProgress` - whether tool reports progress
- `_meta.supportsCancellation` - whether tool can be cancelled
- `_meta.estimatedDurationMs` - execution time hint for UI

### Changed
- `ProtocolHandler` now includes `TaskManager` and `ToolMetadataRegistry`
- `handle_initialize` returns `TasksCapability` with list and cancel support
- `handle_list_tools` populates tool metadata from registry
- Router state management improved for mixed state types (OAuth vs Protocol)

### Technical Details
- TaskManager instance shared across protocol handler methods
- ToolMetadataRegistry loaded with defaults on handler creation
- Meta type properly wrapped using `rmcp::model::Meta`
- OAuth routers use separate state from main protocol handler

### Testing
- 70 tests passing
- Curl tested: health, tools/list, tasks/list, tasks/get
- Curl tested: OAuth well-known endpoints all returning correct metadata

---

## [0.6.0] - 2026-01-09

### Added

#### MCP 2025-11-25 Specification Updates

**Priority 1: Authorization Updates**
- **Protected Resource Metadata (RFC 9728)** - New `/.well-known/oauth-protected-resource` endpoint
- **WWW-Authenticate Enhancement** - Added `resource_metadata` parameter to 401 responses for resource discovery
- **OpenID Connect Discovery** - New `/.well-known/openid-configuration` endpoint (alias for oauth-authorization-server)
- **Client ID Metadata Documents** - Support for URL-based client_id with automatic metadata fetching and caching
- **Incremental Scope Consent** - WWW-Authenticate headers now include scope hints for step-up authorization
- **Token Audience Binding** - Resource parameter support in authorization requests

**Priority 2: Tasks (Experimental)**
- **Task Manager** - Full task lifecycle management module (`src/mcp/tasks.rs`, 633 lines)
- **Task Types** - `Task`, `TaskStatus`, `TaskSupport`, `TaskError` types per spec
- **Task Endpoints** - `tasks/list`, `tasks/get`, `tasks/result`, `tasks/cancel` operations
- **Task Lifecycle** - Create, start, update, complete, fail, cancel task states
- **Task TTL** - Automatic cleanup of expired tasks with configurable TTL
- **Task Capabilities** - Server initialization includes task support capabilities

**Priority 3: Tool Enhancements**
- **Tool Metadata Module** - New `src/tools/metadata.rs` with icons, output schemas, execution config
- **Tool Icons** - `ToolIcon` struct supporting SVG, PNG with mimeType and sizes
- **Tool Output Schema** - JSON Schema definitions for structured output validation
- **Tool Execution Config** - `ToolExecution` with taskSupport (required/optional/forbidden), progress, cancellation

#### New Types
- `ProtectedResourceMetadata` - RFC 9728 resource metadata structure
- `ClientIdMetadataDocument` - Client ID metadata for URL-based clients
- `Task` - Task definition with status, progress, TTL, poll interval
- `TaskStatus` - Pending, Running, Completed, Failed, Cancelled, InputRequired
- `TaskSupport` - Required, Optional, Forbidden for tool-level task negotiation
- `TaskManager` - Thread-safe task storage and lifecycle management
- `TaskError` - NotFound, AlreadyExists, TooManyTasks, InvalidStateTransition
- `ToolIcon` - Icon definition with src, mimeType, sizes
- `ToolExecution` - Task support, progress, cancellation, estimated duration
- `ToolMetadata` - Icons, outputSchema, execution config
- `ToolMetadataRegistry` - Registry for tool metadata with defaults

### Changed
- **OAuth Module** - Extended with RFC 9728, OIDC, Client ID Metadata support (500+ lines added)
- **Middleware Exports** - Added new types to `src/middleware/mod.rs`
- **MCP Module** - Added tasks module export to `src/mcp/mod.rs`
- **Tools Module** - Added metadata module export to `src/tools/mod.rs`
- **Version** - Bumped to 0.6.0
- **reqwest** - Enabled `json` feature for Client ID Metadata Document fetching

### Technical Details
- OAuth config extended with `resource_url`, `resource_name`, `resource_documentation`, `client_id_metadata_document_supported`
- Client metadata cache with HTTP cache-control header respect
- WWW-Authenticate builder with resource_metadata URL and scope hints
- Task manager with configurable TTL, poll interval, max tasks
- Tool metadata registry with sensible defaults for all existing tools

### Testing
- Protected Resource Metadata endpoint tests
- Client ID URL detection tests
- WWW-Authenticate header format tests
- Task lifecycle tests (create, start, progress, complete, cancel)
- Tool metadata serialization tests
- Tool execution config tests

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