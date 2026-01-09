# START TESTING NOW - Claude Desktop Integration

**Version:** 0.3.1  
**Status:** ✅ READY  
**Time to test:** 2 minutes

---

## Pre-Flight Status

All checks passed:
- ✅ Binary built (2.4MB)
- ✅ Tests passing
- ✅ Config installed
- ✅ Claude Desktop found

**You're ready to test!**

---

## Step 1: Restart Claude Desktop

```bash
killall Claude && sleep 2 && open -a Claude
```

---

## Step 2: Test in Claude Desktop

Open Claude Desktop and try these prompts:

### Test 1: Basic Echo
```
Use the echo tool to say "Hello from v0.3.1"
```

**Expected:** Message echoed back with timestamp

### Test 2: Ping
```
Ping the MCP server
```

**Expected:** Status "ok" with timestamp

### Test 3: Server Info
```
Get info about the MCP server
```

**Expected:** Name, version, mode, features

### Test 4: Validation - Empty (should fail)
```
Use the echo tool with an empty message
```

**Expected:** Error "Message cannot be empty"

### Test 5: Validation - Large (should fail)
```
Echo this 500 times: "This will exceed 10KB limit when repeated. "
```

**Expected:** Error "Message too long"

---

## Monitor Logs (Optional)

```bash
tail -f ~/Library/Logs/Claude/mcp*.log
```

---

## Success = All 5 Tests Work

- ✅ Echo returns message + timestamp
- ✅ Ping returns status ok
- ✅ Info shows version 0.3.1
- ✅ Empty message rejected
- ✅ Large message rejected

---

## Troubleshooting

**Tools don't appear?**
```bash
# Check config
cat ~/Library/Application\ Support/Claude/claude_desktop_config.json

# Complete restart
killall -9 Claude && sleep 2 && open -a Claude
```

**Tools fail?**
```bash
# Test manually
cd /Users/hoangiso/Desktop/mcp-boilerplate-rust
./test_mcp.sh
```

---

## After Testing

Document results in `INTEGRATION_TEST_RESULTS.md`

Fill in:
- Which tests passed/failed
- Screenshots
- Performance observations
- Any issues

---

## Quick Commands

```bash
# Restart Claude
killall Claude && sleep 2 && open -a Claude

# View logs
tail -f ~/Library/Logs/Claude/mcp*.log

# Test stdio
./test_mcp.sh

# Verify setup
./verify_claude_ready.sh
```

---

**NOW: Restart Claude Desktop and start testing!**

See `CLAUDE_DESKTOP_SETUP.md` for detailed guide.