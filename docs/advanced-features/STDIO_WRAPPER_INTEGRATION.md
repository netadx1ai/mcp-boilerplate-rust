# MCP Stdio Wrapper Integration Guide

Complete guide for integrating MCP Boilerplate Rust with Claude Desktop via stdio wrapper.

## Overview

The MCP Stdio Wrapper (`@netadx1ai/mcp-stdio-wrapper`) enables Claude Desktop to connect to your Rust MCP server via HTTP.

```
┌─────────────────┐
│ Claude Desktop  │
│   (stdio MCP)   │
└────────┬────────┘
         │ stdio protocol (JSON-RPC 2.0)
         ▼
┌─────────────────┐
│  Stdio Wrapper  │
│  stdio → HTTP   │
└────────┬────────┘
         │ HTTP/REST (x-access-token)
         ▼
┌─────────────────┐
│  MCP Rust Server│
│  (This Project) │
└─────────────────┘
```

## Prerequisites

- MCP Rust server running (this project)
- Node.js installed (for stdio wrapper)
- Claude Desktop installed
- JWT token for authentication

## Quick Start

### 1. Start Rust MCP Server

```bash
cd Desktop/mcp-boilerplate-rust

# Start server
make run

# Or with cargo
cargo run
```

Server starts on `http://localhost:8025`

### 2. Generate JWT Token

```bash
# Install jsonwebtoken if needed
# npm install -g jsonwebtoken-cli

# Or use Node.js
node -e "
const jwt = require('jsonwebtoken');
const token = jwt.sign(
  { userObjId: 'test-user-123' },
  'aivaAPI',  // Match JWT_SECRET in .env
  { algorithm: 'HS256', expiresIn: '24h' }
);
console.log(token);
"
```

Save the token for next step.

### 3. Configure Claude Desktop

Add to Claude Desktop config:

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows:** `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "rust-mcp-local": {
      "command": "npx",
      "args": ["-y", "@netadx1ai/mcp-stdio-wrapper@latest"],
      "env": {
        "API_URL": "http://localhost:8025",
        "JWT_TOKEN": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
        "LOG_FILE": "/tmp/mcp-rust-wrapper.log"
      }
    }
  }
}
```

### 4. Restart Claude Desktop

Restart Claude Desktop to load the MCP server.

### 5. Test in Claude Desktop

Ask Claude:

```
Can you list the available tools?
```

Claude should respond with:
- `echo` - Echo tool with actions (echo, ping, info)

Try the echo tool:

```
Can you use the echo tool to ping?
```

Claude will execute:
```json
{
  "action": "ping"
}
```

## Required Endpoints

Your Rust MCP server MUST implement these endpoints:

### GET /tools

List all available tools.

**Response:**
```json
{
  "tools": [
    {
      "name": "tool_name",
      "description": "Tool description",
      "parameters": {
        "type": "object",
        "properties": {
          "action": {
            "type": "string",
            "description": "Action to perform"
          }
        },
        "required": ["action"]
      }
    }
  ]
}
```

### POST /tools/{tool_name}

Execute a specific tool.

**Request:**
```json
{
  "action": "action_name",
  "param1": "value1"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "result": "..."
  },
  "metadata": {
    "executionTime": 10,
    "timestamp": "2025-01-08T10:30:00Z"
  }
}
```

## Authentication

The stdio wrapper sends JWT token in `x-access-token` header.

### Optional Authentication

Current implementation (no auth required):

```rust
async fn handle_tool(
    headers: HeaderMap,
    Json(payload): Json<ToolRequest>,
) -> impl IntoResponse {
    // No authentication check
}
```

### Required Authentication

Add authentication middleware:

```rust
use crate::middleware::auth_middleware;

// In main.rs
.route("/tools/protected", post(handle_protected_tool))
    .layer(middleware::from_fn(auth_middleware))
```

Handler with user claims:

```rust
async fn handle_protected_tool(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<ToolRequest>,
) -> impl IntoResponse {
    let user_id = claims.user_obj_id;
    // Tool logic with authenticated user
}
```

## Configuration

### Environment Variables

**.env file:**
```bash
# Server
HOST=0.0.0.0
PORT=8025

# JWT (if using authentication)
JWT_SECRET=aivaAPI

# Logging
RUST_LOG=info,mcp_boilerplate_rust=debug
```

### Claude Desktop Config

```json
{
  "mcpServers": {
    "rust-mcp-local": {
      "command": "npx",
      "args": ["-y", "@netadx1ai/mcp-stdio-wrapper@latest"],
      "env": {
        "API_URL": "http://localhost:8025",
        "JWT_TOKEN": "your-token-here",
        "LOG_FILE": "/tmp/mcp-rust-wrapper.log"
      }
    }
  }
}
```

## Adding New Tools

When you add a new tool, update the `/tools` endpoint:

### 1. Create Tool

```rust
// src/tools/my_tool.rs
pub struct MyTool;

impl MyTool {
    pub async fn execute(&self, request: ToolRequest) -> McpResult<ToolResult> {
        // Tool logic
    }
}
```

### 2. Register Tool

```rust
// src/tools/mod.rs
pub mod my_tool;
pub use my_tool::MyTool;
```

### 3. Add Route

```rust
// src/main.rs
.route("/tools/my_tool", post(handle_my_tool))
```

### 4. Update /tools Endpoint

```rust
async fn list_tools() -> impl IntoResponse {
    Json(json!({
        "tools": [
            {
                "name": "echo",
                "description": "Echo tool",
                "parameters": { /* ... */ }
            },
            {
                "name": "my_tool",
                "description": "My custom tool",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["action1", "action2"]
                        }
                    },
                    "required": ["action"]
                }
            }
        ]
    }))
}
```

### 5. Restart Server

```bash
cargo run
```

### 6. Restart Claude Desktop

Claude will now see the new tool!

## Production Deployment

### 1. Build Release Binary

```bash
cargo build --release
```

### 2. Deploy to Server

```bash
# Copy binary to server
scp target/release/mcp-boilerplate-rust user@server:/opt/mcp-server/

# Run on server
./mcp-boilerplate-rust
```

### 3. Update Claude Desktop Config

```json
{
  "mcpServers": {
    "rust-mcp-production": {
      "command": "npx",
      "args": ["-y", "@netadx1ai/mcp-stdio-wrapper@latest"],
      "env": {
        "API_URL": "https://api.yourdomain.com",
        "JWT_TOKEN": "production-token-here",
        "LOG_FILE": "/tmp/mcp-rust-prod.log"
      }
    }
  }
}
```

## Troubleshooting

### Tools Not Appearing in Claude

**Check 1:** Verify server is running
```bash
curl http://localhost:8025/health
```

**Check 2:** Verify /tools endpoint
```bash
curl http://localhost:8025/tools
```

**Check 3:** Check wrapper logs
```bash
tail -f /tmp/mcp-rust-wrapper.log
```

**Check 4:** Verify Claude config
```bash
cat ~/Library/Application\ Support/Claude/claude_desktop_config.json
```

### Authentication Errors

**Error:** "Authentication failed: Invalid or expired JWT token"

**Solution:** Generate new token with correct secret
```bash
node -e "
const jwt = require('jsonwebtoken');
const token = jwt.sign(
  { userObjId: 'user-123' },
  'aivaAPI',  // Must match JWT_SECRET in .env
  { algorithm: 'HS256', expiresIn: '24h' }
);
console.log(token);
"
```

### Connection Errors

**Error:** "Failed to fetch tools from NetADX AI-CORE API"

**Solutions:**
1. Check server is running: `make run`
2. Verify API_URL is correct in Claude config
3. Check CORS is enabled (already enabled in boilerplate)
4. Check firewall/network settings

### Tool Execution Errors

**Check server logs:**
```bash
RUST_LOG=debug cargo run
```

**Check tool response format:**
```bash
curl -X POST http://localhost:8025/tools/echo \
  -H "Content-Type: application/json" \
  -d '{"action":"ping"}'
```

Expected response:
```json
{
  "success": true,
  "data": {
    "action": "ping",
    "response": "pong"
  },
  "metadata": {
    "executionTime": 1,
    "timestamp": "2025-01-08T10:30:00Z"
  }
}
```

## Testing

### Test Server Directly

```bash
# Health check
curl http://localhost:8025/health

# List tools
curl http://localhost:8025/tools

# Execute tool
curl -X POST http://localhost:8025/tools/echo \
  -H "Content-Type: application/json" \
  -d '{"action":"ping"}'
```

### Test with Wrapper Locally

```bash
# Set environment variables
export API_URL="http://localhost:8025"
export JWT_TOKEN="your-token"
export LOG_FILE="/tmp/mcp-test.log"

# Run wrapper
npx -y @netadx1ai/mcp-stdio-wrapper@latest
```

### Test in Claude Desktop

Ask Claude:
```
Can you list available tools?
Can you use the echo tool to ping?
Can you echo the message "Hello from Claude"?
```

## Example Tools Configuration

### Multiple Tools

```rust
async fn list_tools() -> impl IntoResponse {
    Json(json!({
        "tools": [
            {
                "name": "echo",
                "description": "Echo tool with ping and info",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["echo", "ping", "info"]
                        },
                        "message": {
                            "type": "string"
                        }
                    },
                    "required": ["action"]
                }
            },
            {
                "name": "user_management",
                "description": "User CRUD operations",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["create", "get", "list", "update", "delete"]
                        },
                        "userId": {
                            "type": "string"
                        },
                        "data": {
                            "type": "object"
                        }
                    },
                    "required": ["action"]
                }
            }
        ]
    }))
}
```

## Security Best Practices

1. **Use HTTPS in Production**
   ```json
   "API_URL": "https://api.yourdomain.com"
   ```

2. **Rotate JWT Tokens Regularly**
   - Generate new tokens every 24 hours
   - Use short expiration times

3. **Never Commit Tokens**
   - Add to .gitignore
   - Use environment variables

4. **Enable Authentication**
   - Use JWT middleware for sensitive tools
   - Validate user permissions

5. **Monitor Access**
   - Check wrapper logs
   - Monitor server logs
   - Track API usage

## Summary

1. Start Rust MCP server: `make run`
2. Generate JWT token
3. Configure Claude Desktop with stdio wrapper
4. Restart Claude Desktop
5. Test tools in Claude

**Required Endpoints:**
- `GET /tools` - List tools
- `POST /tools/{tool_name}` - Execute tool

**Environment Variables:**
- `API_URL` - Server URL
- `JWT_TOKEN` - Authentication token
- `LOG_FILE` - Debug logs

**Verification:**
```bash
# Server running
curl http://localhost:8025/health

# Tools listed
curl http://localhost:8025/tools

# Tool works
curl -X POST http://localhost:8025/tools/echo \
  -H "Content-Type: application/json" \
  -d '{"action":"ping"}'
```

---

Version: 1.0.0
Last Updated: 2025-01-08
Author: NetADX MCP Team