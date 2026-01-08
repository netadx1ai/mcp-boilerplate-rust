# HTTP Mode Status

**Timestamp:** 2025-01-08 17:10:00 +07:00  
**Status:** ⚠️ Needs Refactoring

## Current State

HTTP mode currently has **build errors** after the stdio-first refactoring.

### Issues

1. **Import errors** - Missing `use` statements for Axum types
2. **EchoTool dependency** - HTTP handlers still reference old `EchoTool` struct
3. **Error handling** - `ErrorData` field access incorrect
4. **Architecture mismatch** - HTTP mode wasn't updated during refactoring

### Build Error Summary

```bash
$ cargo build --features http
error[E0432]: unresolved import `jsonwebtoken`
error[E0405]: cannot find trait `IntoResponse` in this scope
error[E0425]: cannot find function `Json` in this scope
error[E0412]: cannot find type `Arc` in this scope
error[E0412]: cannot find type `EchoTool` in this scope
# ... 43 errors total
```

## Why It Broke

During v0.3.0 refactoring:
1. Moved tools from `src/tools/echo.rs` to `src/mcp/stdio_server.rs`
2. Changed tool implementation to use `#[tool_router]` macro on `McpServer`
3. HTTP mode in `src/main.rs` still references old `EchoTool`
4. HTTP code wasn't tested during refactoring

## Fix Options

### Option 1: Quick Fix (Keep EchoTool)

Keep `src/tools/echo.rs` working for HTTP mode:

```rust
// In src/main.rs HTTP section
use tools::echo::EchoTool;

// HTTP handlers stay the same
let echo_tool = Arc::new(EchoTool::new());
```

**Pros:** Minimal changes  
**Cons:** Duplicated tool definitions (HTTP and stdio separate)

### Option 2: Shared Implementation (Recommended)

Refactor HTTP to use `McpServer` tools:

```rust
// In src/main.rs
#[cfg(feature = "http")]
async fn run_http_server() -> Result<()> {
    let mcp_server = Arc::new(McpServer::new());
    
    // Create HTTP wrapper around MCP tools
    let app = Router::new()
        .route("/tools/echo", post({
            let server = Arc::clone(&mcp_server);
            move |payload| handle_mcp_tool(server, "echo", payload)
        }))
        // ...
}
```

**Pros:** Single source of truth for tools  
**Cons:** Requires refactoring HTTP handlers

### Option 3: Remove HTTP Mode

Focus on stdio-only:

```toml
# Remove from Cargo.toml
# http = ["dep:axum", ...]

# Remove from src/main.rs
# #[cfg(feature = "http")] blocks
```

**Pros:** Simpler, focused codebase  
**Cons:** Loses HTTP capability

## Recommendation

**For now:** Focus on stdio mode (working perfectly)  
**Next iteration:** Implement Option 2 (shared implementation)

## Timeline

- **v0.3.0 (current):** Stdio working, HTTP broken
- **v0.3.1 (planned):** Fix HTTP mode with shared tools
- **v0.4.0 (future):** Add SSE transport, unified architecture

## Workaround

If you need HTTP mode now:

1. **Revert to v0.2.0** (before refactoring)
2. **Use stdio-http bridge** (e.g., mcp-stdio-wrapper)
3. **Wait for v0.3.1** with HTTP fixes

## Testing Checklist for HTTP Fix

When fixing HTTP mode, ensure:

- [ ] `cargo build --features http` compiles without errors
- [ ] Health endpoint works: `curl http://localhost:8025/health`
- [ ] List tools works: `curl http://localhost:8025/tools`
- [ ] Echo tool works: `curl -X POST http://localhost:8025/tools/echo -d '{"message":"test"}'`
- [ ] Ping tool works: `curl -X POST http://localhost:8025/tools/ping`
- [ ] Info tool works: `curl -X POST http://localhost:8025/tools/info`
- [ ] CORS headers present
- [ ] Error handling correct
- [ ] Same tools as stdio mode

## Related Files

- `src/main.rs` - HTTP server implementation (lines 70-280)
- `src/tools/echo.rs` - Legacy tool implementation
- `src/mcp/stdio_server.rs` - New tool implementation
- `Cargo.toml` - Feature flags

## Contact

If you need HTTP mode urgently, create an issue or:
1. Focus on stdio mode (fully working)
2. Use HTTP proxy in front of stdio server
3. Wait for next release

---

**Status:** Known issue, will be fixed in v0.3.1  
**Priority:** Medium (stdio mode is primary)  
**Effort:** 2-4 hours to refactor properly