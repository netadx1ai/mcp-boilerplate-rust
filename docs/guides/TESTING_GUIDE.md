# Testing Guide - Advanced MCP Features

**Created:** 2026-01-13 (HCMC Timezone)  
**Version:** 0.3.1+  
**Purpose:** Practical guide for testing new advanced features

---

## Quick Start

### 1. Build & Verify

```bash
# Build release binary
cargo build --release

# Run all tests
./scripts/test_mcp.sh

# Expected output:
# ✓ All 11 tools operational
# ✓ Advanced features working
# ✓ Server ready for Claude Desktop
```

### 2. Test with MCP Inspector

```bash
# Start inspector (best for development)
npx @modelcontextprotocol/inspector cargo run --release -- --mode stdio

# Browser will open at http://localhost:5173
# You can see all 11 tools and test them interactively
```

### 3. Integrate with Claude Desktop

```bash
# Copy config
cp examples/claude_desktop_config_binary.json ~/Library/Application\ Support/Claude/claude_desktop_config.json

# Edit config to update path:
{
  "mcpServers": {
    "mcp-boilerplate-rust": {
      "command": "/Users/YOUR_USERNAME/Desktop/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust",
      "args": ["--mode", "stdio"]
    }
  }
}

# Restart Claude Desktop
# Look for hammer icon in chat - should show 11 tools
```

---

## Testing New Features

### Feature 1: Progress Notifications

**Tool:** `process_with_progress`

**Test in MCP Inspector:**

```json
{
  "name": "process_with_progress",
  "arguments": {
    "items": 100,
    "delay_ms": 50
  }
}
```

**Expected behavior:**
- 10 progress notifications (every 10 items)
- Each notification shows: progress (0-1.0), total, message
- Final result with completion status

**Test in Claude Desktop:**

Ask Claude:
> "Use the process_with_progress tool to process 100 items with 50ms delay"

You should see Claude report progress updates in real-time.

---

### Feature 2: Batch Processing

**Tool:** `batch_process`

**Test in MCP Inspector:**

```json
{
  "name": "batch_process",
  "arguments": {
    "batch_size": 50,
    "total_batches": 10
  }
}
```

**Expected behavior:**
- 10 batches processed (50 items each)
- Progress notifications per batch
- Logging notifications with batch details
- Total: 500 items processed

**Test in Claude Desktop:**

Ask Claude:
> "Process 10 batches of 50 items each using batch_process"

---

### Feature 3: Data Transformation

**Tool:** `transform_data`

**Test in MCP Inspector:**

```json
{
  "name": "transform_data",
  "arguments": {
    "data": ["hello", "world", "rust", "mcp"],
    "operation": "uppercase"
  }
}
```

**Operations available:**
- `uppercase` - HELLO, WORLD, RUST, MCP
- `lowercase` - hello, world, rust, mcp
- `reverse` - olleh, dlrow, tsur, pcm
- `double` - hellohello, worldworld, rustrust, mcpmcp

**Large dataset test:**

```json
{
  "name": "transform_data",
  "arguments": {
    "data": ["item1", "item2", ..., "item1000"],
    "operation": "uppercase"
  }
}
```

Expected: Progress notifications every 100 items

---

### Feature 4: File Upload Simulation

**Tool:** `simulate_upload`

**Test in MCP Inspector:**

```json
{
  "name": "simulate_upload",
  "arguments": {
    "filename": "test.pdf",
    "size_kb": 1024
  }
}
```

**Expected behavior:**
- 20 chunks uploaded (1024kb / 20 = ~51kb per chunk)
- Progress notifications for each chunk
- Logging notifications with chunk details
- Final upload confirmation

---

### Feature 5: Health Check

**Tool:** `health_check`

**Test in MCP Inspector:**

```json
{
  "name": "health_check",
  "arguments": {}
}
```

**Expected response:**

```json
{
  "status": "healthy",
  "version": "0.3.1",
  "uptime_seconds": 42,
  "timestamp": "2026-01-13T10:00:00Z",
  "features": {
    "progress_notifications": true,
    "task_lifecycle": true,
    "request_context": true
  }
}
```

---

### Feature 6: Long Running Task

**Tool:** `long_task`

**Test in MCP Inspector:**

```json
{
  "name": "long_task",
  "arguments": {}
}
```

**Expected behavior:**
- 10 second execution
- Progress notifications every second (0.1, 0.2, ..., 1.0)
- Logging notifications with step details
- Final completion message

**Note:** This demonstrates how RequestContext enables real-time updates during execution.

---

## RequestContext Testing

All tools now accept `RequestContext<RoleServer>` parameter. This enables:

### 1. Progress Notifications

```rust
// In any tool
ctx.peer.send_notification(
    Notification::progress(ProgressNotificationParam {
        progress_token: NumberOrString::Number(1),
        progress: 0.5,
        total: Some(100.0),
    })
).await?;
```

### 2. Logging Notifications

```rust
// Log important events
ctx.peer.send_notification(
    Notification::logging(LoggingNotificationParam {
        level: LoggingLevel::Info,
        data: serde_json::json!({
            "message": "Processing batch 5/10",
            "items": 50
        }),
        logger: Some("batch_processor".to_string()),
    })
).await?;
```

### 3. HTTP Headers (HTTP mode only)

```rust
// Access request headers
if let Some(headers) = ctx.extensions.get::<HeaderMap>() {
    if let Some(auth) = headers.get("authorization") {
        // Validate auth token
    }
}
```

---

## Performance Testing

### Baseline Tests

```bash
# Test all tools for performance
time ./scripts/test_mcp.sh

# Expected: <5 seconds total
# Each tool: <100ms
```

### Load Tests

**1. High-volume transformation:**

```json
{
  "name": "transform_data",
  "arguments": {
    "data": [/* 10,000 items */],
    "operation": "uppercase"
  }
}
```

Expected: ~5-10 seconds with progress notifications

**2. Many small batches:**

```json
{
  "name": "batch_process",
  "arguments": {
    "batch_size": 10,
    "total_batches": 100
  }
}
```

Expected: ~10-15 seconds with 100 progress notifications

### Memory Testing

```bash
# Monitor memory during long task
cargo build --release
./target/release/mcp-boilerplate-rust --mode stdio &
PID=$!

# Call long_task via inspector
# Monitor memory:
ps aux | grep mcp-boilerplate-rust

# Expected: <10MB memory usage
kill $PID
```

---

## Error Handling Tests

### 1. Invalid Parameters

```json
{
  "name": "process_with_progress",
  "arguments": {
    "items": 0  // Invalid: must be 1-1000
  }
}
```

Expected error:
```json
{
  "error": {
    "code": -32602,
    "message": "Items must be between 1 and 1000"
  }
}
```

### 2. Type Validation

```json
{
  "name": "transform_data",
  "arguments": {
    "data": "not_an_array"  // Invalid type
  }
}
```

Expected error: Type mismatch

### 3. Operation Validation

```json
{
  "name": "transform_data",
  "arguments": {
    "data": ["test"],
    "operation": "invalid_op"  // Invalid operation
  }
}
```

Expected error:
```json
{
  "error": {
    "message": "Invalid operation. Must be one of: uppercase, lowercase, reverse, double"
  }
}
```

---

## Integration Tests

### Test Suite Execution

```bash
# Full verification
./scripts/verify_claude_ready.sh

# Should pass all 10 checks:
# ✓ Binary exists
# ✓ Stdio communication
# ✓ All 11 tools working
# ✓ Progress notifications
# ✓ Error handling
# ✓ Health check
```

### Claude Desktop Integration

**Test checklist:**

1. Config file updated with correct path
2. Claude Desktop restarted
3. Hammer icon shows 11 tools
4. Each tool tested with Claude
5. Progress updates visible in Claude's responses

**Sample prompts for Claude:**

1. "Check server health using health_check tool"
2. "Process 100 items with progress tracking"
3. "Transform this array to uppercase: ['hello', 'world']"
4. "Simulate uploading a 2048kb file called data.zip"
5. "Run a batch process with 20 batches of 100 items each"
6. "Execute the long_task to see progress updates"

---

## Troubleshooting

### Issue: No progress notifications visible

**Check:**
- MCP Inspector console for notification messages
- Server logs (if HTTP mode)
- RequestContext properly passed to tool

**Fix:**
```rust
// Ensure tool signature includes ctx
async fn my_tool(
    params: Parameters<Request>,
    ctx: RequestContext<RoleServer>,  // Must be present
) -> Result<Json<Response>, McpError>
```

### Issue: Tools not appearing in Claude Desktop

**Check:**
1. Binary path in config is correct
2. Binary built in release mode
3. `--mode stdio` argument present
4. Claude Desktop restarted

**Fix:**
```bash
# Rebuild
cargo build --release

# Verify path
ls -la target/release/mcp-boilerplate-rust

# Test standalone
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | \
  ./target/release/mcp-boilerplate-rust --mode stdio
```

### Issue: Build warnings

**Warnings present:**
```
warning: unused import: `task_handler`
warning: field `processor` is never read
```

**Status:** Safe to ignore - these are for future task lifecycle implementation

**To fix:**
```bash
# Allow dead code for processor
#[allow(dead_code)]
processor: Arc<Mutex<OperationProcessor>>,

# Remove unused import
// use rmcp::task_handler;  // Commented out for now
```

---

## Next Steps

### 1. Immediate Actions

- [ ] Test all 11 tools in MCP Inspector
- [ ] Verify progress notifications working
- [ ] Test integration with Claude Desktop
- [ ] Review advanced_features_demo.md for examples

### 2. Optional Improvements

- [ ] Fix task_handler macro compatibility issue
- [ ] Add elicitation support for interactive workflows
- [ ] Implement OAuth2 for production auth
- [ ] Add resource templates for dynamic URIs
- [ ] Create benchmark suite with Criterion.rs

### 3. Production Checklist

- [ ] Review SECURITY.md
- [ ] Enable metrics/instrumentation
- [ ] Add rate limiting (HTTP mode)
- [ ] Implement proper error recovery
- [ ] Add circuit breakers for external calls
- [ ] Set up monitoring/alerting

### 4. Git Workflow

**Option 1: Commit implementation**

```bash
# Use prepared commit message
git add .
git commit -F COMMIT_MESSAGE.txt
git push origin main
```

**Option 2: Feature branch**

```bash
git checkout -b feature/advanced-mcp-features
git add .
git commit -F COMMIT_MESSAGE.txt
git push origin feature/advanced-mcp-features
# Create PR for review
```

---

## Documentation

**Key files to review:**

1. **DEEP_RESEARCH_IMPROVEMENTS.md** (674 lines)
   - Complete analysis of rust-sdk
   - 12 improvements identified
   - 5 critical features implemented
   - Future roadmap

2. **examples/advanced_features_demo.md** (507 lines)
   - Complete usage guide
   - All tool examples
   - Integration guide
   - Performance benchmarks

3. **IMPLEMENTATION_SUMMARY.md** (438 lines)
   - Session summary
   - Architecture changes
   - Breaking changes
   - Statistics

4. **claude.md** (updated)
   - Modern tool patterns
   - RequestContext guide
   - Best practices

---

## Benchmarks

### Expected Performance

| Tool | Items | Time | Throughput | Notifications |
|------|-------|------|-----------|---------------|
| process_with_progress | 100 | 5s | 20/s | 10 |
| batch_process | 500 (10x50) | 10s | 50/s | 10 |
| transform_data | 1000 | 2s | 500/s | 10 |
| simulate_upload | 1024kb | 2s | 512kb/s | 20 |
| long_task | - | 10s | - | 10 |
| health_check | - | <10ms | - | 0 |

### Memory Profile

- Idle: ~2-3MB
- Processing 1000 items: ~4-5MB
- Long task: ~3-4MB
- Peak: <10MB

### CPU Usage

- Idle: <1%
- Active processing: 5-15%
- Long task: 2-5%

---

## Support

**Issues?**
1. Check this guide first
2. Review DEEP_RESEARCH_IMPROVEMENTS.md
3. Check rust-sdk examples: Desktop/rust-sdk/examples/
4. Contact: hello@netadx.ai

**Feature requests?**
See "Future Work" section in IMPLEMENTATION_SUMMARY.md

---

**Last Updated:** 2026-01-13  
**Maintained by:** NetAdx AI  
**License:** MIT