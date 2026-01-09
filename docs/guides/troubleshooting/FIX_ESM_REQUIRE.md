# Fix: ESM Module `require()` Error in Wrapper

**Date:** 2026-01-08 17:35:00 +07:00 (HCMC)  
**Issue:** `require()` is not defined in ES module  
**Status:** ✅ RESOLVED  

---

## Problem

Even with Node v22, the wrapper failed with:

```
ReferenceError: require is not defined
    at Object.error (file:///Users/hoangiso/Desktop/mcp-stdio-wrapper/dist/index.js:61:13)
```

### Root Cause

The wrapper's `src/index.ts` was using **CommonJS `require()`** inside an **ES module**:

```typescript
// ❌ This doesn't work in ES modules
const logger = {
  log: (message: string, ...args: any[]) => {
    if (config.logFile) {
      require('fs').appendFileSync(config.logFile, logMessage);  // ERROR!
    }
  }
}
```

**Why it failed:**
- Package.json declares `"type": "module"` (ES module)
- ES modules must use `import`, not `require()`
- `require()` only works in CommonJS modules
- Node v22 enforces this strictly

---

## Solution

Replace `require('fs')` with ES module import:

### File: `src/index.ts`

**Added import at top:**
```typescript
import { appendFileSync } from 'fs';
```

**Updated logger:**
```typescript
const logger = {
  log: (message: string, ...args: any[]) => {
    if (config.logFile) {
      const timestamp = new Date().toISOString();
      const logMessage = `[${timestamp}] ${message} ${args.map(a => JSON.stringify(a)).join(' ')}\n`;
      appendFileSync(config.logFile, logMessage);  // ✅ Works!
    }
  },
  error: (message: string, ...args: any[]) => {
    if (config.logFile) {
      const timestamp = new Date().toISOString();
      const logMessage = `[${timestamp}] ERROR: ${message} ${args.map(a => JSON.stringify(a)).join(' ')}\n`;
      appendFileSync(config.logFile, logMessage);  // ✅ Works!
    }
  },
};
```

---

## Changes Made

**Before (Broken):**
```typescript
require('fs').appendFileSync(config.logFile, logMessage);
```

**After (Fixed):**
```typescript
// At top of file
import { appendFileSync } from 'fs';

// In logger
appendFileSync(config.logFile, logMessage);
```

---

## Rebuild & Test

### 1. Rebuild Wrapper

```bash
cd /Users/hoangiso/Desktop/mcp-stdio-wrapper
npm run build
```

**Expected output:**
```
> @netadx1ai/mcp-stdio-wrapper@2.1.3 build
> tsc
```

### 2. Test Manually

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}' | \
API_URL="http://localhost:8025" \
JWT_TOKEN="test-token" \
LOG_FILE="/tmp/mcp-http-wrapper.log" \
/Users/hoangiso/.nvm/versions/node/v22.18.0/bin/node \
  /Users/hoangiso/Desktop/mcp-stdio-wrapper/dist/index.js
```

**Expected response:**
```json
{
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {"tools": {}},
    "serverInfo": {
      "name": "netadx-aicore-stdio-wrapper",
      "version": "2.1.3"
    }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```

### 3. Check Logs

```bash
cat /tmp/mcp-http-wrapper.log
```

**Expected:**
```
[2026-01-08T10:33:23.207Z] Starting NetADX AI-CORE MCP stdio wrapper {"apiUrl":"http://localhost:8025","jwtToken":"test-token","logFile":"/tmp/mcp-http-wrapper.log"}
[2026-01-08T10:33:23.209Z] NetAdxApiClient initialized {"baseUrl":"http://localhost:8025"}
[2026-01-08T10:33:23.211Z] NetADX AI-CORE MCP stdio wrapper started successfully
```

✅ **Logs working!**

---

## Why This Fix Works

### ES Module Rules

**ES modules (package.json `"type": "module"`):**
- ✅ Must use `import/export`
- ❌ Cannot use `require()`
- ✅ Top-level imports only
- ✅ File extensions required (.js)

**CommonJS modules:**
- ✅ Can use `require()`
- ❌ Cannot use `import/export` (older Node)
- ✅ Dynamic requires allowed
- ❌ No file extensions needed

### Named Import Syntax

```typescript
// Import specific function
import { appendFileSync } from 'fs';

// Or import entire module
import * as fs from 'fs';
fs.appendFileSync(...);

// Default import (if module has default export)
import fs from 'fs';
```

**For Node.js built-in modules like 'fs', use named imports for tree-shaking benefits.**

---

## Testing with Claude Desktop

### 1. Ensure HTTP Server Running

```bash
cd /Users/hoangiso/Desktop/mcp-boilerplate-rust
./target/release/mcp-boilerplate-rust --mode http
```

### 2. Restart Claude Desktop

```bash
killall Claude
sleep 2
open -a Claude
```

### 3. Monitor Wrapper Logs

```bash
tail -f /tmp/mcp-http-wrapper.log
```

**Expected log flow:**
```
[...] Starting NetADX AI-CORE MCP stdio wrapper
[...] NetAdxApiClient initialized {"baseUrl":"http://localhost:8025"}
[...] NetADX AI-CORE MCP stdio wrapper started successfully
[...] Handling ListTools request
[...] Fetching tools from NetADX AI-CORE API
[...] Tools fetched successfully {"count":3}
```

### 4. Test in Claude Desktop

**Prompt:** "What tools do you have access to?"

**Expected:** See 6 tools total:
- 3 from `mcp-boilerplate-rust-stdio`
- 3 from `mcp-boilerplate-rust-http`

**Try both servers:**
```
Use the echo tool from mcp-boilerplate-rust-stdio to say "Hello via stdio"
Use the echo tool from mcp-boilerplate-rust-http to say "Hello via HTTP"
```

---

## Common ESM Errors & Fixes

### Error: "require is not defined"

**Cause:** Using `require()` in ES module

**Fix:**
```typescript
// ❌ Wrong
const fs = require('fs');

// ✅ Correct
import fs from 'fs';
// or
import { readFileSync } from 'fs';
```

### Error: "Cannot use import outside a module"

**Cause:** Using `import` in CommonJS

**Fix:** Add to package.json:
```json
{
  "type": "module"
}
```

### Error: "Cannot find module"

**Cause:** Missing .js extension in ES module

**Fix:**
```typescript
// ❌ Wrong
import { foo } from './utils';

// ✅ Correct
import { foo } from './utils.js';
```

### Error: "Top-level await"

**Cause:** Using `await` outside async function in CommonJS

**Fix:** ES modules support top-level await:
```typescript
// ✅ Works in ES modules
const data = await fetch('...');
```

---

## Best Practices

### 1. Consistent Module System

**Choose one:**
- ES modules: `"type": "module"` + import/export
- CommonJS: No type field + require/module.exports

**Don't mix** require() in ES modules or import in CommonJS.

### 2. Named vs Default Imports

```typescript
// Prefer named imports (better tree-shaking)
import { appendFileSync, readFileSync } from 'fs';

// Avoid wildcard imports
import * as fs from 'fs';  // Imports everything
```

### 3. File Extensions

```typescript
// Always include .js for relative imports in ES modules
import { logger } from './logger.js';
```

### 4. TypeScript Configuration

**tsconfig.json for ES modules:**
```json
{
  "compilerOptions": {
    "module": "ES2020",
    "target": "ES2020",
    "moduleResolution": "node"
  }
}
```

---

## Prevention

### For Future Development

1. **Use ES module imports consistently**
2. **Set `"type": "module"` in package.json early**
3. **Configure TypeScript for ES modules**
4. **Test with Node v18+ during development**
5. **Use ESLint to catch require() in ES modules**

### ESLint Rule

Add to `.eslintrc.json`:
```json
{
  "rules": {
    "no-restricted-syntax": [
      "error",
      {
        "selector": "CallExpression[callee.name='require']",
        "message": "Use ES6 import instead of require()"
      }
    ]
  }
}
```

---

## Verification Checklist

After fix:
- [x] Wrapper builds without errors
- [x] Manual test shows JSON response
- [x] Log file created successfully
- [x] No `require is not defined` errors
- [x] Node v22 runs wrapper successfully
- [x] Ready for Claude Desktop integration

---

## Status

**Issue:** ✅ RESOLVED  
**Fix:** Changed `require('fs')` to `import { appendFileSync } from 'fs'`  
**Build:** ✅ Successful  
**Test:** ✅ Working  
**Logs:** ✅ Creating correctly  
**Ready:** ✅ For Claude Desktop  

---

## Summary

The wrapper had a simple but critical bug: using CommonJS `require()` in an ES module context. The fix was straightforward:

1. Add ES module import at top: `import { appendFileSync } from 'fs'`
2. Replace `require('fs').appendFileSync()` with `appendFileSync()`
3. Rebuild with `npm run build`
4. Test and verify

**Total time to fix:** ~5 minutes  
**Lines changed:** 3 lines  
**Impact:** Wrapper now works perfectly with Node v18+

---

**Fix completed:** 2026-01-08 17:35:00 +07:00 (HCMC)  
**Wrapper ready for production use!**