# Troubleshooting: Claude Desktop JSON Configuration Error

**Error Message:** "MCP mcp-boilerplate-rust: Unexpected token ',' is not valid JSON"

**Date:** 2026-01-08  
**Status:** RESOLVED

---

## Problem

Claude Desktop shows error:
```
⚠ MCP mcp-boilerplate-rust: Unexpected token ',' '[zm2026-0'...
is not valid JSON
```

---

## Root Cause

The error can occur due to:
1. Invisible characters in the JSON file
2. Incorrect JSON syntax
3. Copying from a file with wrong encoding
4. Hidden BOM (Byte Order Mark) at start of file

---

## Solution

### Step 1: Recreate Config File Cleanly

```bash
cat > ~/Library/Application\ Support/Claude/claude_desktop_config.json << 'EOF'
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
EOF
```

### Step 2: Validate JSON

```bash
python3 -m json.tool ~/Library/Application\ Support/Claude/claude_desktop_config.json
```

Expected: Pretty-printed JSON with no errors.

### Step 3: Verify Binary Path

```bash
/Users/hoangiso/Desktop/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust --help
```

Expected: Help text showing version and options.

### Step 4: Test Binary Works

```bash
cd /Users/hoangiso/Desktop/mcp-boilerplate-rust
./test_mcp.sh
```

Expected: All tests pass.

### Step 5: Restart Claude Desktop

```bash
killall Claude
sleep 2
open -a Claude
```

---

## Verified Working Configuration

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

**Key Points:**
- Use absolute path for `command`
- Use simple logging level (`info` not `info,mcp_boilerplate_rust=debug`)
- No trailing commas
- No comments (JSON doesn't support them)
- UTF-8 encoding without BOM

---

## Alternative: Minimal Config

If issues persist, use this minimal config:

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

This removes the `env` section entirely (uses defaults).

---

## Validation Checklist

- [ ] Config file exists at correct path
- [ ] JSON validates with `python3 -m json.tool`
- [ ] No hidden characters (check with `hexdump -C`)
- [ ] Binary path is absolute
- [ ] Binary is executable (`chmod +x`)
- [ ] Binary works (`--help` shows output)
- [ ] No trailing commas in JSON
- [ ] Claude Desktop fully restarted

---

## Debug Commands

```bash
# Check config exists
ls -la ~/Library/Application\ Support/Claude/claude_desktop_config.json

# Show config content
cat ~/Library/Application\ Support/Claude/claude_desktop_config.json

# Validate JSON
python3 -m json.tool ~/Library/Application\ Support/Claude/claude_desktop_config.json

# Check for hidden characters
hexdump -C ~/Library/Application\ Support/Claude/claude_desktop_config.json | head

# Verify binary
/Users/hoangiso/Desktop/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust --version

# Test binary manually
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}' | /Users/hoangiso/Desktop/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust --mode stdio
```

---

## Common Mistakes

### 1. Trailing Comma
```json
{
  "mcpServers": {
    "mcp-boilerplate-rust": {
      "command": "/path/to/binary",
      "args": ["--mode", "stdio"],  // ❌ Trailing comma
    }
  }
}
```

**Fix:** Remove trailing comma after `"stdio"]`

### 2. Comments in JSON
```json
{
  "mcpServers": {
    // This is my server  ❌ JSON doesn't support comments
    "mcp-boilerplate-rust": {
      ...
    }
  }
}
```

**Fix:** Remove all comments

### 3. Single Quotes
```json
{
  'mcpServers': {  // ❌ Must use double quotes
    ...
  }
}
```

**Fix:** Use double quotes only

### 4. Relative Path
```json
{
  "mcpServers": {
    "mcp-boilerplate-rust": {
      "command": "target/release/mcp-boilerplate-rust",  // ❌ Relative path
      ...
    }
  }
}
```

**Fix:** Use absolute path starting with `/Users/...`

---

## Testing After Fix

Once config is fixed, test with these prompts in Claude Desktop:

1. **Basic test:**
   ```
   Use the echo tool to say "Hello"
   ```

2. **Verify all tools:**
   ```
   What tools do you have access to?
   ```

Expected: Should see echo, ping, and info tools.

---

## Status: RESOLVED

After recreating the config file cleanly:
- ✅ JSON validates successfully
- ✅ Binary path is correct
- ✅ Binary executes without error
- ✅ Ready to restart Claude Desktop

**Next Step:** Restart Claude Desktop and test the tools.

---

## Prevention

To avoid this issue in the future:

1. **Always validate JSON** before copying to Claude config:
   ```bash
   python3 -m json.tool your-config.json
   ```

2. **Use a JSON linter** in your editor

3. **Copy from verified working examples** in the project

4. **Test binary independently** before configuring Claude Desktop

5. **Keep a backup** of working config:
   ```bash
   cp ~/Library/Application\ Support/Claude/claude_desktop_config.json \
      ~/Library/Application\ Support/Claude/claude_desktop_config.json.backup
   ```

---

**Issue Resolved:** Clean config recreated and validated. Ready for testing.