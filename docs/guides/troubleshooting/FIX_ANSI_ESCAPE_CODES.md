# Fix: ANSI Escape Codes Breaking Claude Desktop Integration

**Date:** 2026-01-08 17:20:00 +07:00 (HCMC)  
**Version:** 0.3.1  
**Issue:** Logging output interfering with JSON-RPC protocol  
**Status:** ✅ RESOLVED

---

## Problem Description

### Error Message

Claude Desktop showed repeated errors:
```
Unexpected token '\x1B', "\x1B[2m2026-0"... is not valid JSON
```

### Root Cause

The Rust server was outputting **ANSI color escape codes** in log messages to stderr. These escape codes (`\x1B[2m` for dimming text, etc.) were being mixed with JSON-RPC responses on stdout, causing Claude Desktop's JSON parser to fail.

**Technical Details:**
- `\x1B` = ESC character (ASCII 27)
- `\x1B[2m` = ANSI code for dim/faint text
- tracing-subscriber was automatically adding color codes
- Logs written to stderr, JSON to stdout
- Claude Desktop reads both streams and expects only JSON

### Why It Happened

The `tracing-subscriber` crate defaults to colored output when it detects a TTY. Even though stdio mode uses pipes (not TTY), the logger was still formatting with ANSI codes.

---

## Solution Implemented

### 1. Disabled Logging for Stdio Mode

**File:** `src/main.rs`

Changed initialization to disable logging in stdio mode:

```rust
match args.mode {
    ServerMode::Stdio => {
        if args.verbose {
            std::env::set_var("RUST_LOG", "error");
        } else {
            std::env::set_var("RUST_LOG", "off");
        }
        Logger::init();
        run_stdio_server().await?;
    }
    #[cfg(feature = "http")]
    ServerMode::Http => {
        if args.verbose {
            std::env::set_var("RUST_LOG", "debug,mcp_boilerplate_rust=trace");
        }
        Logger::init();
        info!("MCP Boilerplate Rust v{}", env!("CARGO_PKG_VERSION"));
        info!("Using official rmcp SDK v0.12");
        info!("Starting MCP server in HTTP mode");
        run_http_server().await?;
    }
}
```

**Changes:**
- Stdio mode: Logging set to "off" (or "error" if --verbose)
- HTTP mode: Full logging enabled (unchanged)
- Logger initialized AFTER mode check
- Prevents any log output from interfering with JSON-RPC

### 2. Disabled ANSI Colors in Logger

**File:** `src/utils/logger.rs`

Added `.with_ansi(false)` to disable color codes:

```rust
pub fn init() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "error".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)      // Disable ANSI color codes
                .with_target(false)    // Clean output format
        )
        .init();
}
```

### 3. Removed Unused Import

**File:** `src/main.rs`

Moved `tracing::info` import to HTTP-only:

```rust
#[cfg(feature = "http")]
use tracing::info;
```

---

## Verification

### Before Fix

**Output contained logs:**
```
2026-01-08T10:13:15.462Z  INFO MCP Boilerplate Rust v0.3.1
2026-01-08T10:13:15.471Z  INFO Using official rmcp SDK v0.12
{"jsonrpc":"2.0","id":1,"result":{...}}
```

**Result:** JSON parser failed on ANSI codes

### After Fix

**Output is pure JSON:**
```
{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"rmcp","version":"0.12.0"},"instructions":"MCP Boilerplate Rust Server. Available tools: echo, ping, info."}}
```

**Result:** ✅ Clean JSON, no errors

### Test Results

```bash
$ ./test_mcp.sh

=== MCP Boilerplate Rust - Protocol Test ===

[1/4] Building release binary...
✓ Build complete

[2/4] Testing initialize...
✓ Initialize successful

[3/4] Testing tools/list...
✓ Tools list successful
  Found 3 tools

[4/4] Testing tools/call (echo)...
✓ Echo tool call successful

=== All Tests Passed ===
```

---

## Technical Lessons

### 1. Stdio Protocol Requirements

**MCP Stdio Mode:**
- stdout: JSON-RPC messages ONLY
- stderr: Can be used for logs, but...
- Claude Desktop reads both streams
- Any non-JSON on stdout breaks parsing

**Best Practice:**
- Disable logging completely for stdio mode
- Or redirect logs to a file
- Never mix logs with JSON output

### 2. ANSI Escape Codes

**Common escape codes:**
- `\x1B[0m` - Reset
- `\x1B[1m` - Bold
- `\x1B[2m` - Dim
- `\x1B[31m` - Red color
- `\x1B[32m` - Green color

**Detection:**
- `\x1B` = ASCII 27 (ESC character)
- Visible in hex dumps
- Invisible in terminal (rendered as colors)
- Breaks JSON parsing

### 3. Logger Configuration

**tracing-subscriber defaults:**
- Auto-detects TTY
- Enables colors by default
- Includes timestamps, levels, targets

**Disable colors:**
```rust
.with_ansi(false)
```

**Disable completely:**
```rust
std::env::set_var("RUST_LOG", "off");
```

---

## Files Modified

1. **src/main.rs**
   - Moved logger init after mode check
   - Disabled logging for stdio mode
   - Moved `tracing::info` to HTTP-only

2. **src/utils/logger.rs**
   - Added `.with_ansi(false)`
   - Added `.with_target(false)`
   - Changed default level to "error"

3. **Claude Desktop config**
   - Removed `RUST_LOG` env var
   - Removed `NO_COLOR` env var
   - Simplified to minimal config

---

## Configuration

### Final Claude Desktop Config

```json
{
  "mcpServers": {
    "mcp-boilerplate-rust": {
      "command": "/Users/hoangiso/Desktop/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust",
      "args": ["--mode", "stdio"]
    }
  }
}
```

**Clean and minimal - no env vars needed.**

---

## Build Commands

```bash
# Rebuild after fix
cd /Users/hoangiso/Desktop/mcp-boilerplate-rust
cargo build --release

# Test stdio mode
./test_mcp.sh

# Test manually (should see only JSON)
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}' | ./target/release/mcp-boilerplate-rust --mode stdio
```

---

## Next Steps

1. **Restart Claude Desktop:**
   ```bash
   killall Claude && sleep 2 && open -a Claude
   ```

2. **Test in Claude Desktop:**
   - "Use the echo tool to say hello"
   - "Ping the MCP server"
   - "Get info about the server"

3. **Expected Result:**
   - No JSON parsing errors
   - All 3 tools available
   - Tool execution successful

---

## Debug Mode (If Needed)

If you need to debug issues later:

```bash
# Enable error-level logging for stdio
./target/release/mcp-boilerplate-rust --mode stdio --verbose

# Logs will show errors only, won't break JSON
```

---

## Prevention for Future Tools

When adding new tools, remember:

1. **Never log to stdout in stdio mode**
2. **Keep stderr clean or redirect to file**
3. **Test with test_mcp.sh before integrating**
4. **Use `--verbose` flag sparingly**

**Best practice:** Disable logging entirely for stdio mode, enable full logging for HTTP mode.

---

## Status

- ✅ ANSI codes removed
- ✅ Logging disabled for stdio mode
- ✅ HTTP mode logging preserved
- ✅ All tests passing
- ✅ Clean build (0 warnings)
- ✅ Ready for Claude Desktop

**Issue RESOLVED - Ready to test with Claude Desktop!**