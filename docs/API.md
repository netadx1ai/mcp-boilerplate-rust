# API Reference

Complete API reference for MCP Boilerplate Rust.

**Version:** 0.6.3
**Last Updated:** 2026-03-04 HCMC

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
    "protocolVersion": "2025-11-25",
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
  "version": "0.6.3",
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

### db (requires `postgres` feature)

PostgreSQL database operations via PostgREST.

**Input:**
```json
{
  "action": "string (required: query|insert|update|delete|upsert|rpc|list_tables|describe)",
  "table": "string (required for most actions)",
  "select": "string or array (optional, column projection: \"id,name\" or [\"id\",\"name\"])",
  "filters": "object (optional, Supabase-compatible: { \"col\": { \"op\": value } })",
  "data": "object or array (required for insert/update/upsert)",
  "order": "array (optional: [{ \"column\": \"id\", \"ascending\": true }])",
  "limit": "number (optional)",
  "offset": "number (optional)",
  "options": {
    "count": "string (optional: \"exact\")",
    "single": "boolean (optional: true for single object instead of array)",
    "return_pref": "string (optional: \"representation\" or \"minimal\")"
  },
  "function_name": "string (required for rpc action)",
  "params": "object (optional, rpc parameters)",
  "conflict": "string (optional, conflict column for upsert)",
  "token": "string (optional, JWT token override)"
}
```

**Output:**
```json
{
  "success": true,
  "data": [],
  "count": 5,
  "metadata": {
    "execution_time_ms": 12,
    "timestamp": "2026-03-04T06:17:16Z",
    "action": "query",
    "table": "users",
    "affected_rows": 5
  }
}
```

**Error output:**
```json
{
  "success": false,
  "error": "Descriptive error message",
  "metadata": {
    "execution_time_ms": 1,
    "timestamp": "2026-03-04T06:17:16Z",
    "action": "update",
    "table": "users"
  }
}
```

**Filter operators (14):**

| Operator | Example | PostgREST equivalent |
|----------|---------|---------------------|
| `eq` | `{ "name": { "eq": "alice" } }` | `name=eq.alice` |
| `neq` | `{ "status": { "neq": "deleted" } }` | `status=neq.deleted` |
| `gt` | `{ "age": { "gt": 18 } }` | `age=gt.18` |
| `gte` | `{ "age": { "gte": 18 } }` | `age=gte.18` |
| `lt` | `{ "price": { "lt": 100 } }` | `price=lt.100` |
| `lte` | `{ "price": { "lte": 100 } }` | `price=lte.100` |
| `like` | `{ "name": { "like": "%test%" } }` | `name=like.*test*` |
| `ilike` | `{ "name": { "ilike": "%test%" } }` | `name=ilike.*test*` |
| `is` | `{ "deleted": { "is": null } }` | `deleted=is.null` |
| `in` | `{ "status": { "in": ["a","b"] } }` | `status=in.(a,b)` |
| `not` | `{ "status": { "not": "deleted" } }` | `status=not.eq.deleted` |
| `contains` | `{ "tags": { "contains": ["a"] } }` | `tags=cs.{a}` |
| `containedBy` | `{ "tags": { "containedBy": ["a","b"] } }` | `tags=cd.{a,b}` |
| `overlaps` | `{ "tags": { "overlaps": ["a"] } }` | `tags=ov.{a}` |

**Action examples:**

Query with filters, ordering, limit:
```json
{
  "jsonrpc": "2.0", "id": 1,
  "method": "tools/call",
  "params": {
    "name": "db",
    "arguments": {
      "action": "query",
      "table": "users",
      "select": "id,name,email",
      "filters": { "is_active": { "eq": true } },
      "order": [{ "column": "created_at", "ascending": false }],
      "limit": 10,
      "options": { "count": "exact" }
    }
  }
}
```

Insert:
```json
{
  "jsonrpc": "2.0", "id": 2,
  "method": "tools/call",
  "params": {
    "name": "db",
    "arguments": {
      "action": "insert",
      "table": "users",
      "data": { "name": "Alice", "email": "alice@example.com" }
    }
  }
}
```

Batch insert:
```json
{
  "jsonrpc": "2.0", "id": 3,
  "method": "tools/call",
  "params": {
    "name": "db",
    "arguments": {
      "action": "insert",
      "table": "users",
      "data": [
        { "name": "Alice", "email": "alice@example.com" },
        { "name": "Bob", "email": "bob@example.com" }
      ]
    }
  }
}
```

Update (filters required):
```json
{
  "jsonrpc": "2.0", "id": 4,
  "method": "tools/call",
  "params": {
    "name": "db",
    "arguments": {
      "action": "update",
      "table": "users",
      "filters": { "id": { "eq": 42 } },
      "data": { "name": "Bob" }
    }
  }
}
```

Delete (filters required):
```json
{
  "jsonrpc": "2.0", "id": 5,
  "method": "tools/call",
  "params": {
    "name": "db",
    "arguments": {
      "action": "delete",
      "table": "users",
      "filters": { "id": { "eq": 42 } }
    }
  }
}
```

Upsert:
```json
{
  "jsonrpc": "2.0", "id": 6,
  "method": "tools/call",
  "params": {
    "name": "db",
    "arguments": {
      "action": "upsert",
      "table": "users",
      "data": { "id": 42, "name": "Bob", "email": "bob@example.com" },
      "conflict": "id"
    }
  }
}
```

RPC (call PostgreSQL function):
```json
{
  "jsonrpc": "2.0", "id": 7,
  "method": "tools/call",
  "params": {
    "name": "db",
    "arguments": {
      "action": "rpc",
      "function_name": "test_add",
      "params": { "a": 17, "b": 25 }
    }
  }
}
```

List tables:
```json
{
  "jsonrpc": "2.0", "id": 8,
  "method": "tools/call",
  "params": { "name": "db", "arguments": { "action": "list_tables" } }
}
```

Describe table schema:
```json
{
  "jsonrpc": "2.0", "id": 9,
  "method": "tools/call",
  "params": {
    "name": "db",
    "arguments": { "action": "describe", "table": "users" }
  }
}
```

**Security notes:**
- Table names validated against `^[A-Za-z_][A-Za-z0-9_]*$`
- `DB_ALLOWED_TABLES` / `DB_TABLE_PREFIX` env vars restrict accessible tables
- Update and delete require non-empty filters (prevents mass operations)
- `raw_sql` action is explicitly rejected (use `rpc` with DB functions instead)

**Environment variables:**

| Variable | Default | Description |
|----------|---------|-------------|
| `POSTGREST_URL` | `http://localhost:3000` | PostgREST base URL |
| `POSTGREST_ANON_KEY` | (none) | Bearer token for anonymous access |
| `POSTGREST_TIMEOUT` | `30` | Request timeout in seconds |
| `DB_ALLOWED_TABLES` | (none) | Comma-separated table whitelist |
| `DB_TABLE_PREFIX` | (none) | Only allow tables with this prefix |

---

## HTTP Endpoints

When running with `--features http` (default port 8080):

| Method | Path | Description |
|--------|------|-------------|
| GET | / | Server info |
| GET | /health | Health check |
| GET | /tools | List tools |
| POST | /tools/call | Call tool |
| POST | /rpc | JSON-RPC |

### GET /health

```bash
curl http://127.0.0.1:8080/health
```

```json
{
  "status": "healthy",
  "service": "mcp-boilerplate-rust",
  "version": "0.6.3",
  "protocol": "MCP v5",
  "timestamp": "2026-03-04T12:00:00Z"
}
```

### GET /tools

```bash
curl http://127.0.0.1:8080/tools
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
# Echo tool
curl -X POST http://127.0.0.1:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{"name":"echo","arguments":{"message":"hello"}}'

# db tool (requires postgres feature)
curl -X POST http://127.0.0.1:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{"name":"db","arguments":{"action":"query","table":"users","limit":5}}'
```

### POST /rpc

```bash
curl -X POST http://127.0.0.1:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
```

---

## Auth Endpoints

When running with `--features "http,auth"` (default port 8080):

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | /auth/login | No | Get token |
| GET | /auth/verify | No | Verify token |
| GET | /auth/me | Yes | Current user |
| GET | /protected/tools | Yes | Protected tools |
| GET | /.well-known/oauth-authorization-server | No | OAuth metadata (RFC 8414) |
| GET | /.well-known/openid-configuration | No | OIDC discovery |
| GET | /.well-known/oauth-protected-resource | No | Protected resource metadata (RFC 9728) |
| POST | /oauth/authorize | No | OAuth authorization |
| POST | /oauth/token | No | Token exchange |
| POST | /oauth/register | No | Dynamic client registration |
| POST | /oauth/introspect | No | Token introspection |
| POST | /oauth/revoke | No | Token revocation |

### POST /auth/login

```bash
curl -X POST http://127.0.0.1:8080/auth/login \
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
curl http://127.0.0.1:8080/auth/verify \
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
curl http://127.0.0.1:8080/auth/me \
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
# List tools (HTTP mode)
curl -X POST http://127.0.0.1:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'

# Call echo (HTTP mode)
curl -X POST http://127.0.0.1:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{"name":"echo","arguments":{"message":"hello"}}'

# Call db tool (HTTP mode, requires postgres feature)
curl -X POST http://127.0.0.1:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{"name":"db","arguments":{"action":"query","table":"test_mcp_db","filters":{"is_active":{"eq":true}},"limit":5}}'

# Call db tool via stdio
printf '{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"curl","version":"0.1.0"}}}\n' | \
  POSTGREST_URL=http://localhost:3000 RUST_LOG=off ./target/release/mcp-boilerplate-rust --mode stdio
```

### JavaScript

```javascript
const response = await fetch('http://127.0.0.1:8080/rpc', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    jsonrpc: '2.0',
    id: 1,
    method: 'tools/call',
    params: {
      name: 'db',
      arguments: { action: 'query', table: 'users', limit: 10 }
    }
  })
});
const data = await response.json();
```

### Python

```python
import requests

# List tools
response = requests.post(
    'http://127.0.0.1:8080/rpc',
    json={
        'jsonrpc': '2.0',
        'id': 1,
        'method': 'tools/list',
        'params': {}
    }
)
print(response.json())

# Query db tool
response = requests.post(
    'http://127.0.0.1:8080/tools/call',
    json={
        'name': 'db',
        'arguments': {
            'action': 'query',
            'table': 'users',
            'filters': {'is_active': {'eq': True}},
            'limit': 10
        }
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
| db query (PostgREST) | 5-50ms |
| db insert (PostgREST) | 5-30ms |
| db rpc (PostgREST) | 5-20ms |

db tool latency depends on PostgREST query complexity. The `metadata.execution_time_ms` field in each DbResponse tracks actual per-request timing.