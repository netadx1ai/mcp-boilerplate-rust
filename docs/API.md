# API Reference

Complete API reference for MCP Boilerplate Rust.

**Version:** 0.5.2  
**Last Updated:** 2026-01-09 HCMC

---

## JSON-RPC Methods

All transports use JSON-RPC 2.0 protocol.

### initialize

Initialize MCP session.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2025-03-26",
    "capabilities": {},
    "clientInfo": {
      "name": "client",
      "version": "1.0.0"
    }
  }
}
```

### tools/list

List available tools.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/list",
  "params": {}
}
```

### tools/call

Call a tool.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "echo",
    "arguments": {
      "message": "Hello"
    }
  }
}
```

### prompts/list

List available prompts.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "prompts/list",
  "params": {}
}
```

### prompts/get

Get a prompt.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "prompts/get",
  "params": {
    "name": "code_review",
    "arguments": {
      "code": "fn main() {}",
      "language": "rust"
    }
  }
}
```

---

## Tools

### echo

Echo back a message.

**Input:**
```json
{
  "message": "string (required, 1-10240 bytes)"
}
```

**Output:**
```json
{
  "message": "Hello",
  "timestamp": "2026-01-09T12:00:00Z"
}
```

### ping

Health check.

**Input:** None

**Output:**
```json
{
  "response": "pong",
  "timestamp": "2026-01-09T12:00:00Z"
}
```

### info

Server information.

**Input:** None

**Output:**
```json
{
  "tool": "mcp-boilerplate-rust",
  "version": "0.5.2",
  "description": "MCP server"
}
```

### calculate

Arithmetic operations.

**Input:**
```json
{
  "operation": "add|subtract|multiply|divide|modulo|power",
  "a": "number",
  "b": "number"
}
```

**Output:**
```json
{
  "result": 10,
  "operation": "add",
  "a": 5,
  "b": 5
}
```

### evaluate

Evaluate math expression.

**Input:**
```json
{
  "expression": "2 + 3 * 4"
}
```

**Output:**
```json
{
  "result": 14,
  "expression": "2 + 3 * 4"
}
```

### transform_data

Transform data.

**Input:**
```json
{
  "data": "string",
  "transformation": "uppercase|lowercase|reverse"
}
```

**Output:**
```json
{
  "original": "hello",
  "transformed": "HELLO",
  "transformation": "uppercase"
}
```

### health_check

System health.

**Input:** None

**Output:**
```json
{
  "status": "healthy",
  "version": "0.5.2",
  "uptime": 3600,
  "timestamp": "2026-01-09T12:00:00Z"
}
```

### long_task

Long-running task with progress.

**Input:**
```json
{
  "duration_seconds": "number (1-60)",
  "task_name": "string"
}
```

**Output:**
```json
{
  "completed": true,
  "duration_seconds": 10,
  "steps": 10
}
```

---

## HTTP Endpoints

When running with `--features http`:

| Method | Path | Description |
|--------|------|-------------|
| GET | / | Server info |
| GET | /health | Health check |
| GET | /tools | List tools |
| POST | /tools/call | Call tool |
| POST | /rpc | JSON-RPC |

### GET /health

```bash
curl http://127.0.0.1:8025/health
```

```json
{
  "status": "healthy",
  "service": "mcp-boilerplate-rust",
  "version": "0.5.2",
  "protocol": "MCP v5",
  "timestamp": "2026-01-09T12:00:00Z"
}
```

### GET /tools

```bash
curl http://127.0.0.1:8025/tools
```

```json
{
  "tools": [
    {
      "name": "echo",
      "description": "Echo back a message",
      "inputSchema": {...}
    }
  ]
}
```

### POST /tools/call

```bash
curl -X POST http://127.0.0.1:8025/tools/call \
  -H "Content-Type: application/json" \
  -d '{"name":"echo","arguments":{"message":"hello"}}'
```

### POST /rpc

```bash
curl -X POST http://127.0.0.1:8025/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
```

---

## Auth Endpoints

When running with `--features "http,auth"`:

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | /auth/login | No | Get token |
| GET | /auth/verify | No | Verify token |
| GET | /auth/me | Yes | Current user |
| GET | /protected/tools | Yes | Protected tools |

### POST /auth/login

```bash
curl -X POST http://127.0.0.1:8025/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}'
```

```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

### GET /auth/verify

```bash
curl http://127.0.0.1:8025/auth/verify \
  -H "Authorization: Bearer <token>"
```

```json
{
  "valid": true,
  "claims": {
    "sub": "admin",
    "exp": 1768024810,
    "role": "admin"
  }
}
```

### GET /auth/me

```bash
curl http://127.0.0.1:8025/auth/me \
  -H "x-access-token: <token>"
```

```json
{
  "user": "admin",
  "user_id": "user_admin",
  "role": "admin"
}
```

---

## Authentication

Two methods supported:

**Authorization Header:**
```
Authorization: Bearer <token>
```

**Custom Header:**
```
x-access-token: <token>
```

---

## Error Responses

### JSON-RPC Error

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32600,
    "message": "Invalid Request"
  }
}
```

### HTTP Error

```json
{
  "success": false,
  "error": "Error message"
}
```

### Error Codes

| Code | Description |
|------|-------------|
| -32700 | Parse error |
| -32600 | Invalid Request |
| -32601 | Method not found |
| -32602 | Invalid params |
| -32603 | Internal error |

---

## HTTP Status Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 400 | Bad Request |
| 401 | Unauthorized |
| 404 | Not Found |
| 500 | Internal Error |

---

## CORS

CORS enabled for all origins by default.

---

## Rate Limiting

No rate limiting implemented. Add via middleware if needed.

---

## Examples

### curl

```bash
# List tools
curl -X POST http://127.0.0.1:8025/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'

# Call echo
curl -X POST http://127.0.0.1:8025/tools/call \
  -H "Content-Type: application/json" \
  -d '{"name":"echo","arguments":{"message":"hello"}}'
```

### JavaScript

```javascript
const response = await fetch('http://127.0.0.1:8025/rpc', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    jsonrpc: '2.0',
    id: 1,
    method: 'tools/list',
    params: {}
  })
});
const data = await response.json();
```

### Python

```python
import requests

response = requests.post(
    'http://127.0.0.1:8025/rpc',
    json={
        'jsonrpc': '2.0',
        'id': 1,
        'method': 'tools/list',
        'params': {}
    }
)
print(response.json())
```

---

## Performance

| Operation | Latency |
|-----------|---------|
| Health check | <1ms |
| Tool list | <2ms |
| Echo tool | 2-5ms |
| Calculate | 1-3ms |