# Quick Reference - Advanced MCP Features

**Version:** 0.3.1+  
**Date:** 2026-01-13  
**Purpose:** Fast lookup for common testing scenarios

---

## 🚀 Quick Start (30 seconds)

```bash
# Build
cargo build --release

# Test all features
./scripts/test_mcp.sh

# Open inspector
npx @modelcontextprotocol/inspector cargo run --release -- --mode stdio
```

---

## 🛠️ All 11 Tools

| Tool | Purpose | Key Feature |
|------|---------|-------------|
| `echo` | Message validation | Input validation (1-10KB) |
| `ping` | Health check | Connectivity test |
| `info` | Server metadata | Version info |
| `calculate` | Math operations | Basic calculator |
| `evaluate` | Expression eval | Formula evaluation |
| `process_with_progress` | Data processing | Progress notifications |
| `batch_process` | Batch operations | Batch + logging |
| `transform_data` | Array transformation | 4 operations |
| `simulate_upload` | File upload demo | Upload simulation |
| `long_task` | Long operation | 10s with progress |
| `health_check` | System status | Health monitoring |

---

## 📊 New Advanced Tools (Quick Test)

### 1. Progress Notifications

```json
{
  "name": "process_with_progress",
  "arguments": {
    "items": 100,
    "delay_ms": 50
  }
}
```
**Result:** 10 progress updates + final result (5 seconds)

---

### 2. Batch Processing

```json
{
  "name": "batch_process",
  "arguments": {
    "batch_size": 50,
    "total_batches": 10
  }
}
```
**Result:** 500 items processed, 10 progress updates (10 seconds)

---

### 3. Data Transformation

```json
{
  "name": "transform_data",
  "arguments": {
    "data": ["hello", "world", "rust"],
    "operation": "uppercase"
  }
}
```
**Operations:** `uppercase`, `lowercase`, `reverse`, `double`

---

### 4. Upload Simulation

```json
{
  "name": "simulate_upload",
  "arguments": {
    "filename": "data.zip",
    "size_kb": 2048
  }
}
```
**Result:** 20 chunks uploaded with progress (2 seconds)

---

### 5. Health Check

```json
{
  "name": "health_check",
  "arguments": {}
}
```
**Result:** Status, version, uptime, features enabled

---

### 6. Long Running Task

```json
{
  "name": "long_task",
  "arguments": {}
}
```
**Result:** 10 second task with progress every second

---

## 🔧 Claude Desktop Setup

### Step 1: Copy config

```bash
cp examples/claude_desktop_config_binary.json \
   ~/Library/Application\ Support/Claude/claude_desktop_config.json
```

### Step 2: Edit path

```json
{
  "mcpServers": {
    "mcp-boilerplate-rust": {
      "command": "/FULL/PATH/TO/target/release/mcp-boilerplate-rust",
      "args": ["--mode", "stdio"]
    }
  }
}
```

### Step 3: Restart Claude Desktop

Look for hammer icon → Should show 11 tools

---

## 💬 Test Prompts for Claude

```
1. "Check server health"
2. "Process 100 items with progress tracking"
3. "Transform ['hello', 'world'] to uppercase"
4. "Simulate uploading a 1024kb file called test.pdf"
5. "Run batch process with 10 batches of 50 items"
6. "Execute the long task and show progress"
```

---

## 🧪 MCP Inspector Testing

```bash
# Start inspector
npx @modelcontextprotocol/inspector cargo run --release -- --mode stdio

# Opens http://localhost:5173
# Click any tool → Fill arguments → Execute
# Watch console for progress notifications
```

---

## 📈 Performance Expectations

| Metric | Value |
|--------|-------|
| Response time | 2-7ms (simple), 2-10s (advanced) |
| Memory usage | <5MB idle, <10MB active |
| Binary size | 2.4MB (stdio), 3.1MB (HTTP) |
| Tools count | 11 total (5 basic + 6 advanced) |
| Progress overhead | ~1ms per notification |

---

## 🎯 Key Features Implemented

### ✅ Progress Notifications
- Real-time updates during execution
- `ProgressNotificationParam` with proper format
- Works in MCP Inspector and Claude Desktop

### ✅ RequestContext Integration
- All 11 tools use `RequestContext<RoleServer>`
- Bidirectional communication with `ctx.peer`
- Access to HTTP headers via `ctx.extensions`

### ✅ Logging Notifications
- Structured logging during operations
- `LoggingNotificationParam` with levels
- Info, warn, error, debug levels supported

### ✅ Advanced Tool Suite
- 6 new tools demonstrating modern patterns
- Input validation with `schemars`
- Comprehensive error handling

---

## 🐛 Troubleshooting (30 second fixes)

### Tools not in Claude Desktop?
```bash
# 1. Check binary exists
ls -la target/release/mcp-boilerplate-rust

# 2. Verify config path is absolute
cat ~/Library/Application\ Support/Claude/claude_desktop_config.json

# 3. Restart Claude Desktop (CMD+Q, reopen)
```

### Build warnings?
```bash
# Safe to ignore - for future task lifecycle
# Or fix with:
cargo clippy --fix --allow-dirty
```

### No progress notifications?
```bash
# Check MCP Inspector console (F12)
# Should see notification messages
# Ensure tool uses RequestContext parameter
```

---

## 📚 Documentation Files

| File | Lines | Purpose |
|------|-------|---------|
| DEEP_RESEARCH_IMPROVEMENTS.md | 674 | Complete rust-sdk analysis |
| examples/advanced_features_demo.md | 507 | Usage guide & examples |
| IMPLEMENTATION_SUMMARY.md | 438 | Session summary |
| TESTING_GUIDE.md | 620 | Comprehensive test guide |
| claude.md | Updated | Modern patterns |

---

## 🔄 Git Workflow

### Option 1: Direct commit
```bash
git add .
git commit -F COMMIT_MESSAGE.txt
git push origin main
```

### Option 2: Feature branch (recommended)
```bash
git checkout -b feature/advanced-mcp-features
git add .
git commit -F COMMIT_MESSAGE.txt
git push origin feature/advanced-mcp-features
# Create PR
```

---

## 📋 Next Steps Checklist

- [ ] Build: `cargo build --release`
- [ ] Test: `./scripts/test_mcp.sh`
- [ ] Inspector: Test all 11 tools
- [ ] Claude Desktop: Update config
- [ ] Claude Desktop: Restart & verify
- [ ] Test prompts: Try all 6 sample prompts
- [ ] Review: Read TESTING_GUIDE.md
- [ ] Optional: Fix task_handler macro issue
- [ ] Optional: Commit changes

---

## 🎓 Learn More

**Deep dive:**
- DEEP_RESEARCH_IMPROVEMENTS.md - rust-sdk analysis
- examples/advanced_features_demo.md - Complete examples
- TESTING_GUIDE.md - Detailed testing

**Official docs:**
- Desktop/rust-sdk/examples/ - Official examples
- https://modelcontextprotocol.io - MCP spec

---

## 💡 Pro Tips

1. **Use MCP Inspector for development** - Better debugging than Claude Desktop
2. **Watch console for notifications** - Progress updates visible in browser console
3. **Start with small tests** - Test 10 items before testing 1000
4. **Check health_check first** - Verify server is healthy
5. **Read error messages** - All errors have helpful descriptions

---

## 📞 Support

- **Email:** hello@netadx.ai
- **Docs:** See documentation files above
- **Examples:** Desktop/rust-sdk/examples/

---

**Last Updated:** 2026-01-13  
**Maintained by:** NetAdx AI  
**License:** MIT