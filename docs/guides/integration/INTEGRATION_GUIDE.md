# Integration Guide

**MCP Boilerplate Rust v0.4.0-rc**  
**Last Updated:** 2026-01-08 (HCMC Timezone)

---

## Overview

This guide covers all integration methods for the MCP Boilerplate Rust server.

---

## 🖥️ Claude Desktop Integration

### Prerequisites

- Claude Desktop app installed
- MCP Boilerplate Rust built (`cargo build --release`)
- Binary path known

### Step 1: Build the Server

```bash
cd /path/to/mcp-boilerplate-rust
cargo build --release
```

**Expected output:**
- Binary: `target/release/mcp-boilerplate-rust` (~2.4MB)

### Step 2: Configure Claude Desktop

**Config location:**
```
~/Library/Application Support/Claude/claude_desktop_config.json
```

**Config content:**
```json
{
  "mcpServers": {
    "mcp-boilerplate-rust": {
      "command": "/ABSOLUTE/PATH/TO/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust",
      "args": ["--mode", "stdio"]
    }
  }
}
```

**Important:**
- Use **absolute path** (starts with `/Users/` or `/`)
- Not relative path (`./` or `../`)

### Step 3: Restart Claude Desktop

```bash
# Fully quit (CMD+Q on macOS)
# Then reopen Claude Desktop
```

### Step 4: Verify

1. Look for hammer icon (🔨) in chat
2. Should show 11 tools available
3. Test with: "Check server health"

### Example Config Templates

**Binary mode (recommended):**
```json
{
  "mcpServers": {
    "mcp-boilerplate-rust": {
      "command": "/Users/username/Desktop/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust",
      "args": ["--mode", "stdio"]
    }
  }
}
```

**With environment variables:**
```json
{
  "mcpServers": {
    "mcp-boilerplate-rust": {
      "command": "/path/to/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust",
      "args": ["--mode", "stdio"],
      "env": {
        "RUST_LOG": "off"
      }
    }
  }
}
```

---

## 🔍 MCP Inspector Integration

### What is MCP Inspector?

Interactive tool for testing MCP servers during development.

### Step 1: Start Inspector

```bash
npx @modelcontextprotocol/inspector cargo run --release -- --mode stdio
```

### Step 2: Access Web Interface

**Opens automatically at:**
```
http://localhost:5173
```

### Step 3: Test Tools

1. Click on any tool in the list
2. Fill in arguments (JSON format)
3. Click "Execute"
4. View response in output panel

### Step 4: Monitor Progress

1. Open browser console (F12)
2. Watch for notification messages
3. See progress updates in real-time

### Example Test Cases

**Health Check:**
```json
{
  "name": "health_check",
  "arguments": {}
}
```

**Process with Progress:**
```json
{
  "name": "process_with_progress",
  "arguments": {
    "items": 100,
    "delay_ms": 50
  }
}
```

**Transform Data:**
```json
{
  "name": "transform_data",
  "arguments": {
    "data": ["hello", "world", "rust"],
    "operation": "uppercase"
  }
}
```

---

## 🌐 HTTP Mode Integration

### Enable HTTP Mode

```bash
cargo build --release --features http
./target/release/mcp-boilerplate-rust --mode http
```

**Default port:** 8025

### Available Endpoints

**Health check:**
```bash
curl http://localhost:8025/health
```

**List tools:**
```bash
curl http://localhost:8025/tools
```

**Get tool details:**
```bash
curl http://localhost:8025/tools/echo
```

**Call tool:**
```bash
curl -X POST http://localhost:8025/tools/echo \
  -H "Content-Type: application/json" \
  -d '{"message": "Hello World"}'
```

### Custom Port

```bash
./target/release/mcp-boilerplate-rust --mode http --port 8026
```

### CORS Support

HTTP mode includes CORS headers by default:
- `Access-Control-Allow-Origin: *`
- `Access-Control-Allow-Methods: GET, POST, OPTIONS`
- `Access-Control-Allow-Headers: Content-Type`

---

## 🔧 Wrapper Integration (Optional)

### Using mcp-stdio-wrapper

If you need HTTP-to-stdio bridge:

**Install wrapper:**
```bash
cd /path/to/mcp-stdio-wrapper
npm install
```

**Configure:**
```json
{
  "servers": {
    "mcp-boilerplate-rust": {
      "command": "/path/to/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust",
      "args": ["--mode", "stdio"]
    }
  }
}
```

**Start:**
```bash
npm start
```

**Access via HTTP:**
```
http://localhost:3000/mcp-boilerplate-rust/tools
```

---

## 🧪 Testing Integration

### Quick Test

```bash
# Test all features
./scripts/test_mcp.sh

# Expected: All 11 tools passing
```

### Verify Claude Desktop Connection

```bash
# Check Claude Desktop logs
tail -f ~/Library/Logs/Claude/mcp*.log
```

### Test with Sample Prompts

Ask Claude:
1. "Check server health"
2. "Process 100 items with progress"
3. "Transform ['hello', 'world'] to uppercase"
4. "Simulate uploading a 2048kb file"
5. "Run batch process with 10 batches of 50 items"
6. "Execute the long task"

---

## 🐛 Common Integration Issues

### Issue: Tools Not Appearing in Claude Desktop

**Solutions:**
1. Verify absolute path in config
2. Rebuild: `cargo build --release`
3. Restart Claude Desktop (CMD+Q)
4. Check logs: `~/Library/Logs/Claude/mcp*.log`

### Issue: MCP Inspector Can't Connect

**Solutions:**
1. Ensure Node.js 18+ installed
2. Clear cache: `npx clear-npx-cache`
3. Use full command: `npx @modelcontextprotocol/inspector cargo run --release -- --mode stdio`
4. Check port 5173 not blocked

### Issue: HTTP Mode Port Conflict

**Solutions:**
1. Find process: `lsof -i :8025`
2. Kill it: `kill -9 <PID>`
3. Or use different port: `--port 8026`

### Issue: ANSI/JSON Errors

**Solutions:**
1. Ensure `RUST_LOG=off` in stdio mode
2. Rebuild: `cargo clean && cargo build --release`
3. Verify with: `./scripts/test_mcp.sh`

---

## 📋 Integration Checklist

### Before Integration
- [ ] Rust 1.88.0+ installed
- [ ] Project cloned/downloaded
- [ ] Dependencies resolved

### Build Phase
- [ ] `cargo build --release` succeeds
- [ ] Binary size ~2.4MB
- [ ] No warnings/errors
- [ ] Binary executable

### Claude Desktop
- [ ] Config file created
- [ ] Absolute path used
- [ ] Args include `--mode stdio`
- [ ] Claude Desktop restarted
- [ ] Hammer icon visible
- [ ] 11 tools shown

### Testing
- [ ] `./scripts/test_mcp.sh` passes
- [ ] MCP Inspector connects
- [ ] Sample prompts work
- [ ] Progress notifications visible

---

## 🎯 Recommended Integration Flow

### Development
```
1. Build: cargo build --release
2. Test: ./scripts/test_mcp.sh
3. Inspect: npx @modelcontextprotocol/inspector ...
4. Iterate: Make changes, repeat
```

### Production (Claude Desktop)
```
1. Build: cargo build --release
2. Test: ./scripts/verify_claude_ready.sh
3. Configure: claude_desktop_config.json
4. Deploy: Restart Claude Desktop
5. Verify: Test with sample prompts
```

### API/HTTP Integration
```
1. Build: cargo build --release --features http
2. Start: ./target/release/... --mode http
3. Test: curl http://localhost:8025/health
4. Integrate: Use HTTP endpoints
```

---

## 📚 Related Documentation

- [Quick Start Guide](../guides/QUICK_START.md)
- [Testing Guide](../guides/TESTING_GUIDE.md)
- [Troubleshooting](../troubleshooting/COMMON_ISSUES.md)
- [API Reference](../reference/API.md)

---

## 🆘 Need Help?

**Can't integrate?**
1. Review this guide thoroughly
2. Check [Common Issues](../troubleshooting/COMMON_ISSUES.md)
3. Run `./scripts/verify_claude_ready.sh`
4. Contact: hello@netadx.ai

**Example configs:**
- See `examples/claude_desktop_config_binary.json`
- See `examples/claude_desktop_config_stdio.json`
- See `examples/claude_desktop_config_http_wrapper.json`

---

**Last Updated:** 2026-01-08  
**Maintained by:** NetAdx AI  
**License:** MIT