# SDK Generator Quick Start

Generate client SDKs for MCP Boilerplate Rust in TypeScript, Python, and Go.

## 5-Minute Setup

### Step 1: Generate SDKs

```bash
cd sdk-generators
cargo run --release
```

Output:
```
SDK generation complete!
  - TypeScript: output/typescript/mcp-client.ts
  - Python:     output/python/mcp_client.py
  - Go:         output/go/mcpclient/client.go
```

### Step 2: Start MCP Server

```bash
cd ..
cargo run --release --features http -- --mode http
```

Server will start on `http://127.0.0.1:8080`

### Step 3: Use Generated SDK

**TypeScript:**

```typescript
import { McpClient } from './mcp-client';

const client = new McpClient({ transport: 'http', port: 8080 });
const result = await client.echo({ message: 'Hello' });
console.log(result.data);
```

**Python:**

```python
from mcp_client import McpClient, McpClientConfig

config = McpClientConfig(transport='http', port=8080)
client = McpClient(config)
result = client.echo(message='Hello from Python')
print(result.data)
```

**Go:**

```go
import "your-project/mcpclient"

config := mcpclient.Config{Transport: "http", Port: 8080}
client := mcpclient.NewClient(config)
result, _ := client.Echo(map[string]interface{}{"message": "Hello"})
fmt.Println(result.Data)
```

## What Gets Generated

### All 11 Tools Supported

- `ping` - Health check
- `echo` - Message validation
- `info` - Server metadata
- `calculate` - Math operations
- `evaluate` - Expression evaluation
- `process_with_progress` - Progress tracking
- `batch_process` - Batch operations
- `transform_data` - Data transformation
- `simulate_upload` - Upload simulation
- `health_check` - Health monitoring
- `long_task` - Long-running operations

### All 6 Transports Supported

| Transport | Port | Browser | Use Case |
|-----------|------|---------|----------|
| stdio | N/A | No | Desktop apps |
| sse | 8025 | Yes | Server push |
| websocket | 9001 | Yes | Real-time |
| http | 8080 | Yes | REST API |
| http-stream | 8026 | Yes | Large files |
| grpc | 50051 | No | Microservices |

## Quick Examples

### TypeScript Example

```typescript
// Basic usage
const client = new McpClient({ transport: 'http' });

// Health check
await client.ping();

// Calculator
await client.calculate({ operation: 'add', a: 10, b: 5 });

// Transform data
await client.transform_data({
  data: ['hello', 'world'],
  operation: 'uppercase'
});
```

### Python Example

```python
config = McpClientConfig(transport='websocket', port=9001)
client = McpClient(config)

# Health check
client.ping()

# Process with progress
client.process_with_progress(
    data=['item1', 'item2', 'item3'],
    delay_ms=100
)

# Batch processing
client.batch_process(
    items=['task1', 'task2'],
    operation='process'
)
```

### Go Example

```go
config := mcpclient.Config{Transport: "grpc", Port: 50051}
client := mcpclient.NewClient(config)

// Health check
client.Ping(map[string]interface{}{})

// Evaluate expression
client.Evaluate(map[string]interface{}{
    "expression": "2 * (3 + 4)",
})

// Simulate upload
client.SimulateUpload(map[string]interface{}{
    "filename": "test.pdf",
    "size_bytes": 1024000,
})
```

## Transport Configuration

### SSE Transport

```typescript
const client = new McpClient({
  transport: 'sse',
  port: 8025
});
```

Server command:
```bash
cargo run --release --features sse -- --mode sse
```

### WebSocket Transport

```python
config = McpClientConfig(transport='websocket', port=9001)
```

Server command:
```bash
cargo run --release --features websocket -- --mode websocket
```

### gRPC Transport

```go
config := mcpclient.Config{Transport: "grpc", Port: 50051}
```

Server command:
```bash
cargo run --release --features grpc -- --mode grpc
```

## Installation

### TypeScript

```bash
# Copy generated file
cp output/typescript/mcp-client.ts your-project/src/

# No dependencies needed (uses native fetch)
```

### Python

```bash
# Copy generated file
cp output/python/mcp_client.py your-project/

# Install dependency
pip install requests
```

### Go

```bash
# Copy generated package
cp -r output/go/mcpclient your-project/

# No dependencies needed
```

## Error Handling

### TypeScript

```typescript
try {
  const result = await client.echo({ message: 'test' });
  if (result.success) {
    console.log(result.data);
  } else {
    console.error(result.error);
  }
} catch (error) {
  console.error('Request failed:', error);
}
```

### Python

```python
result = client.echo(message='test')
if result.success:
    print(result.data)
else:
    print(f'Error: {result.error}')
```

### Go

```go
result, err := client.Echo(map[string]interface{}{"message": "test"})
if err != nil {
    log.Printf("Request failed: %v", err)
    return
}
if !result.Success {
    log.Printf("Tool error: %s", result.Error)
    return
}
fmt.Println(result.Data)
```

## Testing

### Run Example Files

```bash
# TypeScript
cd templates
ts-node typescript-example.ts

# Python
python3 python-example.py

# Go
go run go-example.go
```

### Run All Transports

```bash
# Start servers in separate terminals
cargo run --release --features http -- --mode http
cargo run --release --features sse -- --mode sse
cargo run --release --features websocket -- --mode websocket
cargo run --release --features grpc -- --mode grpc
```

## Customization

### Add Custom Tool

Edit `src/main.rs` in `load_tool_schemas()`:

```rust
ToolSchema {
    name: "custom_tool".to_string(),
    description: "My custom tool".to_string(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "param": {
                "type": "string",
                "description": "Parameter description"
            }
        },
        "required": ["param"]
    }),
}
```

Regenerate:
```bash
cargo run --release
```

## Performance

- Generation time: <500ms total
- TypeScript SDK: ~15KB
- Python SDK: ~12KB
- Go SDK: ~18KB

## Common Issues

### Port Already in Use

```bash
# Find process
lsof -i :8080
kill -9 <PID>

# Or use different port
cargo run --release --features http -- --mode http --bind 127.0.0.1:9999
```

### Server Not Running

Ensure MCP server is started before using client:
```bash
cargo run --release --features http -- --mode http
```

### Import Errors

**TypeScript:**
```bash
npm install -g typescript ts-node
```

**Python:**
```bash
pip install requests
```

**Go:**
```bash
go mod init your-project
```

## Next Steps

1. Review full README: `sdk-generators/README.md`
2. Check examples: `templates/`
3. Read MCP docs: `../docs/`
4. Test all transports
5. Integrate into your project

## Support

- GitHub: https://github.com/netadx/mcp-boilerplate-rust
- Email: hello@netadx.ai

---

**Version:** 0.4.0  
**Generated:** 2026-01-09 HCMC  
**Status:** Production Ready