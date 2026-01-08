# Changelog

All notable changes to MCP Boilerplate Rust will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.4.0-rc] - 2026-01-08

### Added

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

### Removed
- Duplicate documentation files (6 files)
- Obsolete session checklists
- Redundant tool quick references

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

| Feature | v0.3.0 | v0.3.1 | v0.4.0-rc |
|---------|--------|--------|-----------|
| Tools | 3 | 5 | 11 |
| Protocol | 2024-11-05 | 2025-03-26 | 2025-03-26 |
| Progress Notifications | ❌ | ❌ | ✅ |
| RequestContext | ❌ | ❌ | ✅ |
| Output Schemas | ❌ | ✅ | ✅ |
| Icons & Annotations | ❌ | ✅ | ✅ |
| Documentation Lines | ~1K | ~2K | ~9.3K |
| Test Coverage | Basic | Comprehensive | Enhanced |

---

## Upgrade Guide

### From v0.3.1 to v0.4.0-rc

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
- Real-time progress notifications
- System health monitoring
- Comprehensive documentation

---

## Future Roadmap

### v0.4.0 (Stable Release)
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