# MCP Boilerplate Rust - API Documentation

Complete API reference for the MCP v5 Rust boilerplate server.

## Base URL

```
http://localhost:8025
```

## Response Format

All endpoints return JSON with the following structure:

### Success Response

```json
{
  "success": true,
  "data": { ... },
  "metadata": {
    "executionTime": 10,
    "timestamp": "2025-01-08T10:30:00Z"
  }
}
```

### Error Response

```json
{
  "success": false,
  "error": "Error message",
  "metadata": {
    "executionTime": 5,
    "timestamp": "2025-01-08T10:30:00Z"
  }
}
```

## Health Check Endpoints

### GET /health

Check server health status.

**Request:**
```bash
curl http://localhost:8025/health
```

**Response:**
```json
{
  "status": "healthy",
  "service": "mcp-boilerplate-rust",
  "version": "0.1.0",
  "protocol": "MCP v5",
  "timestamp": "2025-01-08T10:30:00.123Z"
}
```

### GET /

Root health check endpoint (same as /health).

## Tool Endpoints

### POST /tools/echo

Echo tool with multiple actions for testing and debugging.

#### Echo Action

Echo back a message.

**Request:**
```bash
curl -X POST http://localhost:8025/tools/echo \
  -H "Content-Type: application/json" \
  -d '{
    "action": "echo",
    "message": "Hello MCP!"
  }'
```

**Parameters:**
- `action` (string, required): Must be "echo"
- `message` (string, required): Message to echo back

**Response:**
```json
{
  "success": true,
  "data": {
    "action": "echo",
    "message": "Hello MCP!",
    "echoed_at": "2025-01-08T10:30:00.123Z"
  },
  "metadata": {
    "executionTime": 2,
    "timestamp": "2025-01-08T10:30:00.123Z"
  }
}
```

#### Ping Action

Simple ping-pong test.

**Request:**
```bash
curl -X POST http://localhost:8025/tools/echo \
  -H "Content-Type: application/json" \
  -d '{
    "action": "ping"
  }'
```

**Parameters:**
- `action` (string, required): Must be "ping"

**Response:**
```json
{
  "success": true,
  "data": {
    "action": "ping",
    "response": "pong",
    "timestamp": "2025-01-08T10:30:00.123Z"
  },
  "metadata": {
    "executionTime": 1,
    "timestamp": "2025-01-08T10:30:00.123Z"
  }
}
```

#### Info Action

Get tool information and available actions.

**Request:**
```bash
curl -X POST http://localhost:8025/tools/echo \
  -H "Content-Type: application/json" \
  -d '{
    "action": "info"
  }'
```

**Parameters:**
- `action` (string, required): Must be "info"

**Response:**
```json
{
  "success": true,
  "data": {
    "action": "info",
    "tool": "echo",
    "version": "0.1.0",
    "description": "Simple echo tool for MCP v5",
    "available_actions": [
      {
        "name": "echo",
        "description": "Echo back a message",
        "parameters": {
          "message": "string (required)"
        }
      },
      {
        "name": "ping",
        "description": "Simple ping-pong test",
        "parameters": {}
      },
      {
        "name": "info",
        "description": "Get tool information",
        "parameters": {}
      }
    ],
    "timestamp": "2025-01-08T10:30:00.123Z"
  },
  "metadata": {
    "executionTime": 3,
    "timestamp": "2025-01-08T10:30:00.123Z"
  }
}
```

## Error Responses

### Invalid Action

**Request:**
```bash
curl -X POST http://localhost:8025/tools/echo \
  -H "Content-Type: application/json" \
  -d '{
    "action": "invalid_action"
  }'
```

**Response:**
```json
{
  "success": false,
  "error": "Invalid action: invalid_action. Available actions: echo, ping, info",
  "metadata": {
    "executionTime": 1,
    "timestamp": "2025-01-08T10:30:00.123Z"
  }
}
```

**Status Code:** 500

### Missing Parameter

**Request:**
```bash
curl -X POST http://localhost:8025/tools/echo \
  -H "Content-Type: application/json" \
  -d '{
    "action": "echo"
  }'
```

**Response:**
```json
{
  "success": false,
  "error": "Missing parameter: message",
  "metadata": {
    "executionTime": 1,
    "timestamp": "2025-01-08T10:30:00.123Z"
  }
}
```

**Status Code:** 500

### Invalid JSON

**Request:**
```bash
curl -X POST http://localhost:8025/tools/echo \
  -H "Content-Type: application/json" \
  -d 'invalid json'
```

**Response:**
```json
{
  "error": "Failed to deserialize the JSON body into the target type"
}
```

**Status Code:** 400

## HTTP Status Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 400 | Bad Request (invalid JSON) |
| 404 | Not Found |
| 500 | Internal Server Error |

## Headers

### Request Headers

| Header | Required | Description |
|--------|----------|-------------|
| `Content-Type` | Yes | Must be `application/json` |
| `x-access-token` | No | JWT token for authenticated endpoints |

### Response Headers

| Header | Description |
|--------|-------------|
| `Content-Type` | Always `application/json` |
| `Access-Control-Allow-Origin` | CORS header |

## Authentication

Authentication is optional by default. When enabled, include JWT token in header:

```bash
curl -X POST http://localhost:8025/tools/echo \
  -H "Content-Type: application/json" \
  -H "x-access-token: YOUR_JWT_TOKEN" \
  -d '{
    "action": "ping"
  }'
```

## Rate Limiting

Currently no rate limiting is implemented. Can be added using tower middleware.

## CORS

CORS is enabled for all origins by default. Configure in `src/main.rs`:

```rust
let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);
```

## Examples

### Using curl

```bash
# Health check
curl http://localhost:8025/health

# Echo message
curl -X POST http://localhost:8025/tools/echo \
  -H "Content-Type: application/json" \
  -d '{"action":"echo","message":"Hello!"}'

# Ping
curl -X POST http://localhost:8025/tools/echo \
  -H "Content-Type: application/json" \
  -d '{"action":"ping"}'
```

### Using HTTPie

```bash
# Health check
http GET localhost:8025/health

# Echo message
http POST localhost:8025/tools/echo \
  action=echo \
  message="Hello!"

# Ping
http POST localhost:8025/tools/echo \
  action=ping
```

### Using JavaScript/Fetch

```javascript
// Health check
const health = await fetch('http://localhost:8025/health');
const healthData = await health.json();

// Echo message
const response = await fetch('http://localhost:8025/tools/echo', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    action: 'echo',
    message: 'Hello from JavaScript!'
  })
});
const data = await response.json();
console.log(data);
```

### Using Python/Requests

```python
import requests

# Health check
response = requests.get('http://localhost:8025/health')
print(response.json())

# Echo message
response = requests.post(
    'http://localhost:8025/tools/echo',
    json={
        'action': 'echo',
        'message': 'Hello from Python!'
    }
)
print(response.json())
```

### Using Rust/Reqwest

```rust
use reqwest;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    // Health check
    let health = client
        .get("http://localhost:8025/health")
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    println!("{:#?}", health);
    
    // Echo message
    let response = client
        .post("http://localhost:8025/tools/echo")
        .json(&json!({
            "action": "echo",
            "message": "Hello from Rust!"
        }))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    println!("{:#?}", response);
    
    Ok(())
}
```

## Performance

Typical response times:
- Health check: < 1ms
- Echo tool: 1-5ms
- With database: 10-50ms

## Monitoring

Check server logs for request details:

```bash
RUST_LOG=debug cargo run
```

Log output:
```
[INFO] MCP Server starting on 0.0.0.0:8025
[INFO] Available endpoints:
[INFO]   GET  /health - Health check
[INFO]   POST /tools/echo - Echo tool
[INFO] Echo tool called with action: "ping"
```

## Version

API Version: 1.0.0
MCP Protocol: v5
Server Version: 0.1.0

---

Last updated: 2025-01-08