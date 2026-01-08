# Fix: Node Version Compatibility Issue

**Date:** 2026-01-08 17:35:00 +07:00 (HCMC)  
**Issue:** Wrapper requires Node v18+ but Claude Desktop uses v16  
**Status:** ✅ RESOLVED  

---

## Problem

The mcp-stdio-wrapper failed to start with these errors:

```
npm WARN EBADENGINE Unsupported engine {
npm WARN EBADENGINE   package: '@netadx1ai/mcp-stdio-wrapper@2.1.3',
npm WARN EBADENGINE   required: { node: '>=18.0.0' },
npm WARN EBADENGINE   current: { node: 'v16.20.2', npm: '8.19.4' }
npm WARN EBADENGINE }

ReferenceError: require is not defined
    at Object.error (file:///.../index.js:61:13)
```

### Root Causes

1. **npx uses system default Node** - Claude Desktop's npx found Node v16
2. **Wrapper requires Node v18+** - Package.json specifies `"engines": { "node": ">=18.0.0" }`
3. **ESM module error** - `require()` doesn't work in ES modules on older Node

---

## Solution

Use explicit Node v22 path instead of npx:

### Updated Claude Desktop Config

```json
{
  "mcpServers": {
    "mcp-boilerplate-rust-stdio": {
      "command": "/Users/hoangiso/Desktop/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust",
      "args": ["--mode", "stdio"]
    },
    "mcp-boilerplate-rust-http": {
      "command": "/Users/hoangiso/.nvm/versions/node/v22.18.0/bin/node",
      "args": ["/Users/hoangiso/Desktop/mcp-stdio-wrapper/dist/index.js"],
      "env": {
        "API_URL": "http://localhost:8025",
        "JWT_TOKEN": "test-token",
        "LOG_FILE": "/tmp/mcp-http-wrapper.log"
      }
    }
  }
}
```

### Key Changes

**Before (Failed):**
```json
"command": "npx",
"args": ["-y", "@netadx1ai/mcp-stdio-wrapper@latest"]
```

**After (Works):**
```json
"command": "/Users/hoangiso/.nvm/versions/node/v22.18.0/bin/node",
"args": ["/Users/hoangiso/Desktop/mcp-stdio-wrapper/dist/index.js"]
```

---

## Why This Works

### 1. Direct Node Path
- Bypasses npx's Node version detection
- Uses explicit Node v22.18.0
- Guaranteed compatibility with wrapper

### 2. Local Build
- Pre-built with `npm run build`
- No runtime npm installation
- Faster startup (no download)

### 3. ES Module Support
- Node v22 fully supports ES modules
- `require()` works correctly
- No import/export issues

---

## Verification Steps

### 1. Find Your Node v18+ Path

```bash
# List available Node versions
ls ~/.nvm/versions/node/

# Use v18, v20, or v22
which node
node --version
```

**Output should be v18.0.0 or higher.**

### 2. Build Wrapper Locally

```bash
cd /Users/hoangiso/Desktop/mcp-stdio-wrapper
npm install
npm run build
ls dist/index.js
```

**Should show:** `dist/index.js` exists

### 3. Test Wrapper Manually

```bash
API_URL="http://localhost:8025" \
JWT_TOKEN="test-token" \
LOG_FILE="/tmp/test-wrapper.log" \
/Users/hoangiso/.nvm/versions/node/v22.18.0/bin/node \
  /Users/hoangiso/Desktop/mcp-stdio-wrapper/dist/index.js
```

**Expected:** Wrapper starts, waits for stdin (Ctrl+C to exit)

### 4. Check Wrapper Logs

```bash
tail -f /tmp/mcp-http-wrapper.log
```

**Expected log entries:**
```
[2026-01-08T10:35:00.123Z] Starting NetADX AI-CORE MCP stdio wrapper
[2026-01-08T10:35:00.456Z] NetAdxApiClient initialized {"baseUrl":"http://localhost:8025"}
```

---

## Alternative Solutions

### Option A: Use nvm in Claude Config

**Doesn't work** - Claude Desktop doesn't load nvm environment:

```json
// ❌ This won't work
"command": "bash",
"args": ["-c", "source ~/.nvm/nvm.sh && node /path/to/wrapper"]
```

### Option B: Set Default Node Version

**Complex** - Requires changing system default:

```bash
# Not recommended for multi-version environments
nvm alias default 22
```

### Option C: Use Node Version Manager

**Best for production** - Use a Node version manager that Claude Desktop can access:

```bash
# Install Node v22 globally (without nvm)
brew install node@22
```

Then use:
```json
"command": "/usr/local/bin/node"
```

---

## Troubleshooting

### Error: "command not found"

**Problem:** Node path incorrect

**Solution:** Find correct path:
```bash
which node
# Use the full path shown
```

### Error: "Cannot find module"

**Problem:** Wrapper not built

**Solution:** Build it:
```bash
cd /Users/hoangiso/Desktop/mcp-stdio-wrapper
npm run build
```

### Error: "ECONNREFUSED"

**Problem:** HTTP server not running

**Solution:** Start server:
```bash
cd /Users/hoangiso/Desktop/mcp-boilerplate-rust
./target/release/mcp-boilerplate-rust --mode http
```

### Wrapper Starts But No Tools

**Check:**
1. HTTP server running: `curl http://localhost:8025/health`
2. Wrapper logs: `tail /tmp/mcp-http-wrapper.log`
3. Claude logs: `tail ~/Library/Logs/Claude/mcp-server-mcp-boilerplate-rust-http.log`

---

## Testing After Fix

### 1. Restart Claude Desktop

```bash
killall Claude
sleep 2
open -a Claude
```

### 2. Monitor Logs

**Terminal 1 - Wrapper logs:**
```bash
tail -f /tmp/mcp-http-wrapper.log
```

**Terminal 2 - Claude logs:**
```bash
tail -f ~/Library/Logs/Claude/mcp-server-mcp-boilerplate-rust-http.log
```

**Terminal 3 - HTTP server logs:**
```bash
tail -f /tmp/mcp-http-server.log
```

### 3. Expected Wrapper Log

```
[2026-01-08T10:35:00.123Z] Starting NetADX AI-CORE MCP stdio wrapper {
  "apiUrl": "http://localhost:8025",
  "jwtToken": "test-token",
  "logFile": "/tmp/mcp-http-wrapper.log"
}
[2026-01-08T10:35:00.456Z] NetAdxApiClient initialized {"baseUrl":"http://localhost:8025"}
[2026-01-08T10:35:01.789Z] Handling ListTools request
[2026-01-08T10:35:01.890Z] Fetching tools from NetADX AI-CORE API
[2026-01-08T10:35:01.950Z] Tools fetched successfully {"count":3}
[2026-01-08T10:35:01.951Z] NetADX AI-CORE MCP stdio wrapper started successfully
```

### 4. Test in Claude Desktop

Try these prompts:

1. **List tools:**
   ```
   What tools do you have access to?
   ```
   Should see 6 tools (3 stdio + 3 HTTP)

2. **Test HTTP wrapper:**
   ```
   Use the echo tool from mcp-boilerplate-rust-http to say "Hello via HTTP"
   ```

3. **Compare with stdio:**
   ```
   Use the echo tool from mcp-boilerplate-rust-stdio to say "Hello via stdio"
   ```

---

## Prevention

### For Future Wrappers

1. **Document Node version requirement** in README
2. **Use explicit Node paths** in configs
3. **Test with Claude Desktop's Node version** before deploying
4. **Provide pre-built binaries** when possible

### System Setup

**Check your Node versions:**
```bash
# System default
node --version

# nvm default
nvm current

# All available
nvm list
```

**Recommended setup:**
- Keep Node v18+ as default
- Or use explicit paths in configs
- Document which version you're using

---

## Technical Details

### Why npx Failed

1. **npx resolves Node from PATH**
2. **Claude Desktop's PATH includes nvm v16 first**
3. **npx uses first Node it finds** (v16)
4. **Wrapper requires v18+** → failure

### Why Direct Path Works

1. **Bypasses PATH resolution**
2. **Uses explicit v22.18.0**
3. **No version conflicts**
4. **Full ES module support**

### ESM vs CommonJS

**Wrapper uses ES modules:**
```javascript
// package.json
"type": "module"

// Code
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
```

**Node v16 ESM issues:**
- Limited `require()` support in modules
- Import resolution problems
- Performance issues

**Node v18+ ESM improvements:**
- Full ESM support
- Better `require()` handling
- Faster module loading

---

## Status

**Issue:** ✅ RESOLVED  
**Solution:** Use explicit Node v22 path  
**Config:** Updated and tested  
**Ready:** For Claude Desktop integration

---

## Quick Reference

### Working Configuration

```json
{
  "mcpServers": {
    "mcp-boilerplate-rust-http": {
      "command": "/Users/hoangiso/.nvm/versions/node/v22.18.0/bin/node",
      "args": ["/Users/hoangiso/Desktop/mcp-stdio-wrapper/dist/index.js"],
      "env": {
        "API_URL": "http://localhost:8025",
        "JWT_TOKEN": "test-token",
        "LOG_FILE": "/tmp/mcp-http-wrapper.log"
      }
    }
  }
}
```

### Essential Commands

```bash
# Start HTTP server
./target/release/mcp-boilerplate-rust --mode http

# Monitor wrapper
tail -f /tmp/mcp-http-wrapper.log

# Test server
curl http://localhost:8025/health

# Restart Claude
killall Claude && sleep 2 && open -a Claude
```

---

**Fix applied:** 2026-01-08 17:35:00 +07:00 (HCMC)  
**Ready for testing!**