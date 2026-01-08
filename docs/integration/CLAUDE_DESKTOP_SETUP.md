# Claude Desktop Integration Setup Guide

**Version:** 0.3.1  
**Last Updated:** 2026-01-08 17:10:00 +07:00 (HCMC)  
**Status:** Ready for Integration Testing

---

## Overview

This guide walks you through integrating MCP Boilerplate Rust v0.3.1 with Claude Desktop. The server is production-ready with security hardening, input validation, and comprehensive testing.

## Prerequisites Checklist

Before starting, ensure you have:

- [ ] Claude Desktop installed (macOS)
- [ ] Rust 1.70+ installed (`rustc --version`)
- [ ] Release binary built (2.4MB)
- [ ] All tests passing (stdio, HTTP, validation)

**Verify prerequisites:**

```bash
cd /Users/hoangiso/Desktop/mcp-boilerplate-rust

# Check binary exists
ls -lh target/release/mcp-boilerplate-rust
# Expected: -rwxr-xr-x  2.4M mcp-boilerplate-rust

# Run test suite
./test_mcp.sh
# Expected: All Tests Passed

./test_validation.sh
# Expected: Validation Tests Complete
```

---

## Step 1: Locate Claude Desktop Config

Claude Desktop stores its configuration at:

```
~/Library/Application Support/Claude/claude_desktop_config.json
```

**Create directory if it doesn't exist:**

```bash
mkdir -p ~/Library/Application\ Support/Claude
```

---

## Step 2: Configure MCP Server

### Option A: Use Provided Config (Recommended)

Copy the pre-configured file:

```bash
cd /Users/hoangiso/Desktop/mcp-boilerplate-rust

# Backup existing config (if any)
cp ~/Library/Application\ Support/Claude/claude_desktop_config.json \
   ~/Library/Application\ Support/Claude/claude_desktop_config.json.backup 2>/dev/null || true

# Copy new config
cp claude_desktop_config_stdio.json \
   ~/Library/Application\ Support/Claude/claude_desktop_config.json
```

### Option B: Manual Configuration

Edit the config file manually:

```bash
nano ~/Library/Application\ Support/Claude/claude_desktop_config.json
```

Add this configuration:

```json
{
  "mcpServers": {
    "mcp-boilerplate-rust": {
      "command": "/Users/hoangiso/Desktop/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust",
      "args": ["--mode", "stdio"],
      "env": {
        "RUST_LOG": "info,mcp_boilerplate_rust=debug"
      }
    }
  }
}
```

**Important:**
- Use absolute path for `command`
- Binary must be executable (`chmod +x`)
- `args` specifies stdio mode
- `RUST_LOG` enables debug logging

### Option C: Add to Existing Config

If you already have other MCP servers configured:

```json
{
  "mcpServers": {
    "existing-server": {
      "command": "...",
      "args": ["..."]
    },
    "mcp-boilerplate-rust": {
      "command": "/Users/hoangiso/Desktop/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust",
      "args": ["--mode", "stdio"],
      "env": {
        "RUST_LOG": "info,mcp_boilerplate_rust=debug"
      }
    }
  }
}
```

---

## Step 3: Restart Claude Desktop

**Complete restart required:**

```bash
# Kill all Claude processes
killall Claude

# Wait 2 seconds
sleep 2

# Reopen Claude Desktop
open -a Claude
```

**Why full restart?**
- Claude Desktop only reads config on startup
- Ensures clean state for MCP server initialization
- Clears any cached connections

---

## Step 4: Verify Tools are Available

Once Claude Desktop reopens:

1. **Check for MCP icon/indicator** - Look for tool integration UI
2. **Try a simple prompt** - "What tools do you have access to?"
3. **Look for these tools:**
   - `echo` - Echo back messages (with validation)
   - `ping` - Simple connectivity test
   - `info` - Server information

---

## Step 5: Test Tool Functionality

### Test 1: Echo Tool (Basic)

**Prompt in Claude Desktop:**
```
Use the echo tool to say "Hello from v0.3.1"
```

**Expected behavior:**
- Tool executes successfully
- Response contains the echoed message
- Timestamp in RFC3339 format

**Example response:**
```json
{
  "message": "Hello from v0.3.1",
  "timestamp": "2026-01-08T10:15:30.123Z"
}
```

### Test 2: Ping Tool

**Prompt:**
```
Ping the MCP server
```

**Expected response:**
```json
{
  "status": "ok",
  "timestamp": "2026-01-08T10:16:00.456Z"
}
```

### Test 3: Info Tool

**Prompt:**
```
Get information about the MCP server
```

**Expected response:**
```json
{
  "name": "mcp-boilerplate-rust",
  "version": "0.3.1",
  "mode": "stdio",
  "features": ["input_validation", "security_hardened"]
}
```

### Test 4: Input Validation (Empty Message)

**Prompt:**
```
Use the echo tool with an empty message
```

**Expected behavior:**
- Tool execution fails
- Error message: "Message cannot be empty"
- Claude handles error gracefully

### Test 5: Input Validation (Large Message)

**Prompt:**
```
Use the echo tool to repeat this 1000 times: "This is a very long message that will exceed the 10KB limit when repeated many times..."
```

**Expected behavior:**
- Tool execution fails
- Error message: "Message too long: X bytes (max: 10240 bytes)"
- Claude provides user-friendly explanation

---

## Step 6: Monitor Logs (Optional)

### Claude Desktop Logs

```bash
# Follow Claude Desktop logs in real-time
tail -f ~/Library/Logs/Claude/mcp*.log
```

**Look for:**
- Server initialization messages
- Tool registration
- Request/response cycles
- Any errors or warnings

### MCP Server Logs

If you need more detailed server-side logging:

```bash
# Run server manually with verbose logging
cd /Users/hoangiso/Desktop/mcp-boilerplate-rust

RUST_LOG=debug ./target/release/mcp-boilerplate-rust --mode stdio \
  2> server-debug.log &

# Watch logs
tail -f server-debug.log
```

**Note:** Only do this for debugging. Claude Desktop manages the server automatically.

---

## Troubleshooting

### Tools Don't Appear in Claude Desktop

**Check 1: Config File Location**
```bash
cat ~/Library/Application\ Support/Claude/claude_desktop_config.json
```
Expected: Valid JSON with mcp-boilerplate-rust entry

**Check 2: Binary Path**
```bash
/Users/hoangiso/Desktop/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust --version
```
Expected: Version info or help text

**Check 3: Binary Permissions**
```bash
ls -l /Users/hoangiso/Desktop/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust
```
Expected: `-rwxr-xr-x` (executable)

If not executable:
```bash
chmod +x /Users/hoangiso/Desktop/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust
```

**Check 4: JSON Syntax**
```bash
# Validate JSON
python3 -m json.tool ~/Library/Application\ Support/Claude/claude_desktop_config.json
```
Expected: Pretty-printed JSON (no errors)

**Check 5: Complete Restart**
```bash
# Force quit and restart
killall -9 Claude
sleep 2
open -a Claude
```

### Tools Appear But Fail to Execute

**Check 1: Server Logs**
```bash
tail -50 ~/Library/Logs/Claude/mcp*.log | grep -i error
```

**Check 2: Test Server Manually**
```bash
cd /Users/hoangiso/Desktop/mcp-boilerplate-rust
./test_mcp.sh
```
Expected: All Tests Passed

**Check 3: Run Server Interactively**
```bash
./target/release/mcp-boilerplate-rust --mode stdio --verbose
```
Then try sending a test request:
```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}
```

### Validation Errors Expected

These are **not bugs** - validation is working correctly:

**Empty message:**
```
Error: Message cannot be empty
```

**Message too large (>10KB):**
```
Error: Message too long: 15360 bytes (max: 10240 bytes)
```

**What to do:** Use valid input (1-10,240 bytes)

### Performance Issues

**Symptom:** Slow tool responses

**Check 1: Using Release Build?**
```bash
# Should see "release" in path
which ./target/release/mcp-boilerplate-rust
```

**Check 2: CPU Usage**
```bash
# While using tools
top | grep mcp-boilerplate-rust
```

**Check 3: Rebuild if Needed**
```bash
cd /Users/hoangiso/Desktop/mcp-boilerplate-rust
cargo clean
cargo build --release
```

---

## Configuration Options

### Reduce Logging Verbosity

Edit config to use `info` level only:

```json
{
  "mcpServers": {
    "mcp-boilerplate-rust": {
      "command": "/Users/hoangiso/Desktop/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust",
      "args": ["--mode", "stdio"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Increase Logging Detail

For debugging, use trace level:

```json
{
  "env": {
    "RUST_LOG": "trace,mcp_boilerplate_rust=trace"
  }
}
```

### Disable Logging

Minimal logging:

```json
{
  "env": {
    "RUST_LOG": "error"
  }
}
```

---

## Verification Checklist

After setup, verify:

- [ ] Config file exists at correct path
- [ ] Binary path in config is absolute
- [ ] Binary is executable
- [ ] Claude Desktop fully restarted
- [ ] Tools appear in Claude UI
- [ ] Echo tool works with valid input
- [ ] Ping tool returns success
- [ ] Info tool shows server details
- [ ] Empty message rejected with clear error
- [ ] Large message (>10KB) rejected
- [ ] No errors in Claude logs

---

## Success Criteria

Integration is successful when:

1. **Tool Discovery**
   - All 3 tools visible in Claude Desktop
   - Tool descriptions are clear
   - Tool schemas are correct

2. **Tool Execution**
   - Echo tool returns input with timestamp
   - Ping tool confirms connectivity
   - Info tool provides server metadata

3. **Input Validation**
   - Empty messages rejected
   - Large messages (>10KB) rejected
   - Valid messages (1-10,240 bytes) processed

4. **Error Handling**
   - Clear error messages
   - No crashes or hangs
   - Claude handles errors gracefully

5. **Performance**
   - Tool responses < 100ms
   - No memory leaks
   - No CPU spikes

---

## Next Steps After Successful Integration

### Document Results

Create `INTEGRATION_TEST_RESULTS.md`:

```bash
cd /Users/hoangiso/Desktop/mcp-boilerplate-rust
touch INTEGRATION_TEST_RESULTS.md
```

Include:
- Screenshots of Claude UI with tools
- Example successful tool calls
- Example validation errors
- Performance observations
- Any issues encountered

### Optional Enhancements

Once integration works:

1. **Add More Tools** - File operations, calculations, etc.
2. **Custom Validation** - Per-tool size limits
3. **Monitoring** - Usage metrics and analytics
4. **Advanced Features** - Tool chaining, async operations

### Production Deployment

See `SECURITY.md` for production checklist:
- Review security settings
- Configure CORS (if using HTTP mode)
- Set up monitoring
- Enable rate limiting
- Use strong secrets

---

## Quick Reference

### File Locations

| Item | Path |
|------|------|
| Binary | `/Users/hoangiso/Desktop/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust` |
| Claude Config | `~/Library/Application Support/Claude/claude_desktop_config.json` |
| Claude Logs | `~/Library/Logs/Claude/mcp*.log` |
| Project Root | `/Users/hoangiso/Desktop/mcp-boilerplate-rust` |

### Commands

| Action | Command |
|--------|---------|
| Test stdio | `./test_mcp.sh` |
| Test validation | `./test_validation.sh` |
| Restart Claude | `killall Claude && open -a Claude` |
| View logs | `tail -f ~/Library/Logs/Claude/mcp*.log` |
| Rebuild | `cargo build --release` |

### Tools Available

| Tool | Purpose | Validation |
|------|---------|------------|
| echo | Echo message back | 1-10,240 bytes |
| ping | Connectivity test | None |
| info | Server metadata | None |

---

## Support

### Documentation

- `README.md` - Main documentation
- `QUICK_START.md` - 5-minute setup
- `SECURITY.md` - Security guidelines
- `SIMPLIFICATION_COMPLETE.md` - v0.3.1 changes

### Testing

- `test_mcp.sh` - Stdio mode tests
- `test_http.sh` - HTTP mode tests
- `test_validation.sh` - Input validation tests

### Logs

Check logs for detailed error information:

```bash
# Claude Desktop logs
~/Library/Logs/Claude/mcp*.log

# Server logs (if running manually)
RUST_LOG=debug ./target/release/mcp-boilerplate-rust --mode stdio 2> debug.log
```

---

**Ready to integrate!** Follow the steps above and document your results.