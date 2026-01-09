# Common Issues & Solutions

**MCP Boilerplate Rust v0.4.0-rc**  
**Last Updated:** 2026-01-08 (HCMC Timezone)

---

## Quick Troubleshooting Guide

This document consolidates all known issues and their solutions.

---

## 🔧 Build Issues

### Issue: Build Warnings

**Symptoms:**
```
warning: unused import: `task_handler`
warning: field `processor` is never read
```

**Solution:**
These warnings are safe to ignore. They're reserved for future task lifecycle implementation (SEP-1686).

**To suppress:**
```rust
// In src/mcp/stdio_server.rs
#[allow(dead_code)]
processor: Arc<Mutex<OperationProcessor>>,
```

**Status:** Already fixed in v0.4.0-rc

---

### Issue: Compilation Errors

**Symptoms:**
```
error: could not compile `mcp-boilerplate-rust`
```

**Solutions:**

1. **Update Rust:**
```bash
rustup update
rustc --version  # Should be 1.88.0+
```

2. **Clean build:**
```bash
cargo clean
cargo build --release
```

3. **Check dependencies:**
```bash
cargo update
```

---

## 🔌 Claude Desktop Integration Issues

### Issue: Tools Not Appearing

**Symptoms:**
- Claude Desktop shows no tools
- Hammer icon missing
- "No MCP servers connected" message

**Solutions:**

1. **Verify config path is absolute:**
```json
{
  "mcpServers": {
    "mcp-boilerplate-rust": {
      "command": "/Users/YOUR_USERNAME/Desktop/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust",
      "args": ["--mode", "stdio"]
    }
  }
}
```

❌ **Wrong:** `./target/release/mcp-boilerplate-rust`  
✅ **Correct:** `/Users/username/Desktop/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust`

2. **Verify binary exists:**
```bash
ls -la target/release/mcp-boilerplate-rust
# Should show ~2.4MB file
```

3. **Rebuild if needed:**
```bash
cargo build --release
```

4. **Restart Claude Desktop:**
```bash
# Fully quit (CMD+Q)
# Then reopen
```

5. **Check Claude Desktop logs:**
```bash
tail -f ~/Library/Logs/Claude/mcp*.log
```

---

### Issue: ANSI Escape Codes / JSON Parse Error

**Symptoms:**
```
Error: Unexpected token '\x1B' in JSON
SyntaxError: Unexpected token in JSON
```

**Cause:** Logging output interfering with JSON-RPC communication

**Solution:**
Ensure `RUST_LOG=off` in stdio mode (already fixed in v0.4.0-rc):

```rust
// In main.rs
match args.mode {
    ServerMode::Stdio => {
        std::env::set_var("RUST_LOG", "off");  // Critical!
        Logger::init();
    }
}
```

**Verify fix:**
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | \
  ./target/release/mcp-boilerplate-rust --mode stdio | jq
```

Should output clean JSON, no ANSI codes.

---

### Issue: Tools Execute but No Response

**Symptoms:**
- Tools appear in Claude Desktop
- Execution starts but hangs
- No response received

**Solutions:**

1. **Check for blocking operations:**
```rust
// Use tokio::time::sleep for async
tokio::time::sleep(Duration::from_millis(100)).await;

// Not thread::sleep!
```

2. **Verify RequestContext usage:**
```rust
async fn my_tool(
    params: Parameters<Request>,
    ctx: RequestContext<RoleServer>,  // Must be present
) -> Result<Json<Response>, McpError>
```

3. **Test with MCP Inspector:**
```bash
npx @modelcontextprotocol/inspector cargo run --release -- --mode stdio
```

---

## 📊 Progress Notifications Issues

### Issue: Progress Notifications Not Visible

**Symptoms:**
- Tool executes successfully
- No progress updates shown
- Claude doesn't report progress

**Solutions:**

1. **Check browser console in MCP Inspector:**
```bash
# Open inspector
npx @modelcontextprotocol/inspector cargo run --release -- --mode stdio

# Press F12 to open console
# Look for notification messages
```

2. **Verify notification code:**
```rust
ctx.peer.send_notification(
    Notification::progress(ProgressNotificationParam {
        progress_token: NumberOrString::Number(1),
        progress: 0.5,
        total: Some(100.0),
    })
).await?;
```

3. **Test with advanced tools:**
```bash
# Use process_with_progress tool
# Should show 10 progress updates
```

---

## 🌐 HTTP Mode Issues

### Issue: Port Already in Use

**Symptoms:**
```
Error: Address already in use (os error 48)
```

**Solution:**

1. **Find process using port 8025:**
```bash
lsof -i :8025
```

2. **Kill the process:**
```bash
kill -9 <PID>
```

3. **Or use different port:**
```bash
./target/release/mcp-boilerplate-rust --mode http --port 8026
```

---

### Issue: CORS Errors

**Symptoms:**
```
Access to fetch blocked by CORS policy
```

**Solution:**
HTTP mode includes CORS support (already configured in v0.4.0-rc)

Verify CORS headers:
```bash
curl -v http://localhost:8025/health
# Should see Access-Control-Allow-Origin header
```

---

## 🧪 Testing Issues

### Issue: Tests Fail

**Symptoms:**
```bash
./scripts/test_mcp.sh
# Some tests failing
```

**Solutions:**

1. **Rebuild binary:**
```bash
cargo build --release
```

2. **Check binary is executable:**
```bash
chmod +x target/release/mcp-boilerplate-rust
```

3. **Verify jq is installed:**
```bash
which jq
# If not found:
brew install jq
```

4. **Run individual test:**
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | \
  ./target/release/mcp-boilerplate-rust --mode stdio
```

---

### Issue: MCP Inspector Not Working

**Symptoms:**
```
npx @modelcontextprotocol/inspector cargo run --release
# Fails to connect
```

**Solutions:**

1. **Ensure Node.js 18+:**
```bash
node --version
# Should be v18.0.0 or higher
```

2. **Clear npx cache:**
```bash
npx clear-npx-cache
```

3. **Use specific mode:**
```bash
npx @modelcontextprotocol/inspector cargo run --release -- --mode stdio
```

4. **Check firewall:**
```bash
# Ensure localhost:5173 is not blocked
```

---

## 🔒 Security Issues

### Issue: Input Validation Errors

**Symptoms:**
```
Error: Message exceeds maximum length
Error: Invalid parameter
```

**Solution:**
This is expected behavior - validation working correctly.

**Check limits:**
- Echo message: 1-10,240 bytes
- Items: 1-1,000 for process_with_progress
- Batch size: 1-1,000 for batch_process
- Data array: max 10,000 items

**Adjust input:**
```json
{
  "name": "echo",
  "arguments": {
    "message": "Keep under 10KB"
  }
}
```

---

## 💾 Performance Issues

### Issue: High Memory Usage

**Symptoms:**
- Memory usage >100MB
- System slowing down

**Solutions:**

1. **Check for memory leaks:**
```bash
cargo build --release
./target/release/mcp-boilerplate-rust --mode stdio &
PID=$!
ps aux | grep mcp-boilerplate-rust
# Should be <10MB
```

2. **Reduce batch sizes:**
```json
{
  "name": "transform_data",
  "arguments": {
    "data": ["Smaller", "array"],  // Not 10,000 items
    "operation": "uppercase"
  }
}
```

3. **Restart server periodically**

---

### Issue: Slow Response Times

**Symptoms:**
- Tools take >5 seconds
- Claude times out

**Solutions:**

1. **Check system resources:**
```bash
top
# Look for CPU/memory usage
```

2. **Reduce processing:**
```json
{
  "name": "process_with_progress",
  "arguments": {
    "items": 50,  // Not 1000
    "delay_ms": 10  // Faster
  }
}
```

3. **Use release build:**
```bash
# Always use --release
cargo build --release
```

---

## 🐛 Debug Tips

### Enable Debug Logging (HTTP mode only)

```bash
RUST_LOG=debug ./target/release/mcp-boilerplate-rust --mode http
```

**Note:** Never use logging in stdio mode - it breaks JSON-RPC!

### Test Standalone

```bash
# Test without Claude Desktop
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}' | \
  ./target/release/mcp-boilerplate-rust --mode stdio
```

### Check Binary Size

```bash
ls -lh target/release/mcp-boilerplate-rust
# Should be ~2.4MB

# If much larger, rebuild:
cargo clean
cargo build --release
```

---

## 📋 Pre-flight Checklist

Before reporting issues, verify:

- [ ] Rust 1.88.0+ installed
- [ ] Clean release build (`cargo build --release`)
- [ ] Binary exists and is executable
- [ ] Config uses absolute paths
- [ ] Claude Desktop fully restarted
- [ ] No ANSI codes in output
- [ ] Tests passing (`./scripts/test_mcp.sh`)
- [ ] MCP Inspector can connect
- [ ] Logs checked (HTTP mode only)

---

## 🆘 Still Having Issues?

### 1. Run Full Verification
```bash
./scripts/verify_claude_ready.sh
```

Should pass all 10 checks.

### 2. Check Documentation
- **Quick Reference:** docs/reference/QUICK_REFERENCE.md
- **Testing Guide:** docs/guides/TESTING_GUIDE.md
- **Action Plan:** docs/guides/ACTION_PLAN.md

### 3. Review Specific Guides
- **ANSI Codes:** See FIX_ANSI_ESCAPE_CODES.md
- **Node/ESM Issues:** See FIX_ESM_REQUIRE.md
- **JSON Errors:** See TROUBLESHOOTING_JSON_ERROR.md

### 4. Contact Support
- **Email:** hello@netadx.ai
- **Include:**
  - Error message
  - Steps to reproduce
  - Output of `cargo --version` and `node --version`
  - Relevant logs

---

## 📚 Related Documentation

- [Installation Guide](../guides/INSTALLATION.md)
- [Testing Guide](../guides/TESTING_GUIDE.md)
- [Security Guidelines](../reference/SECURITY.md)
- [API Reference](../reference/API.md)

---

**Last Updated:** 2026-01-08  
**Maintained by:** NetAdx AI  
**License:** MIT