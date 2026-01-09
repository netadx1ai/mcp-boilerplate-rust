# Client SDK Generators

Auto-generate type-safe client libraries for MCP Boilerplate Rust in TypeScript, Python, and Go.

## Overview

The SDK generator automatically creates client libraries from tool schemas, enabling developers to interact with the MCP server in their preferred language with full type safety and IDE support.

**Generated Languages:**
- TypeScript/JavaScript (Node.js & Browser)
- Python 3.7+
- Go 1.21+

**Features:**
- Type-safe interfaces for all 11 tools
- Support for all 6 transport modes
- Zero external dependencies (except Python requests)
- Full documentation in generated code
- Production-ready error handling

## Quick Start

### Generate All SDKs

```bash
cd sdk-generators
cargo run --release
```

**Output:**
```
SDK generation complete!
  - TypeScript: output/typescript/mcp-client.ts
  - Python:     output/python/mcp_client.py
  - Go:         output/go/mcpclient/client.go
```

### Using Make

```bash
cd sdk-generators
make generate    # Generate all SDKs
make test        # Install deps and prepare for testing
make verify      # Check generated files
make clean       # Clean output
```

## Generated SDKs

### TypeScript SDK

**File:** `output/typescript/mcp-client.ts`  
**Size:** ~15KB (~209 lines)

**Features:**
- Full TypeScript type definitions
- Promise-based async API
- Browser and Node.js compatible
- Native fetch (no dependencies)

**Installation:**

```bash
cp output/typescript/mcp-client.ts your-project/src/
```

**Usage:**

```typescript
import { McpClient, McpClientConfig } from './mcp-client';

const client = new McpClient({
  transport: 'http',
  port: 8080
});

// Type-safe calls with autocomplete
const result = await client.echo({ message: 'Hello World' });
if (result.success) {
  console.log(result.data);
}

// Calculator with type checking
const calc = await client.calculate({
  operation: 'add',
  a: 10,
  b: 5
});

// Transform data
const transform = await client.transform_data({
  data: ['hello', 'world'],
  operation: 'uppercase'
});
```

**All Transport Modes:**

```typescript
// SSE
const sseClient = new McpClient({ transport: 'sse', port: 8025 });

// WebSocket
const wsClient = new McpClient({ transport: 'websocket', port: 9001 });

// gRPC
const grpcClient = new McpClient({ transport: 'grpc', port: 50051 });

// HTTP Stream
const streamClient = new McpClient({ transport: 'http-stream', port: 8026 });
```

### Python SDK

**File:** `output/python/mcp_client.py`  
**Size:** ~12KB (~111 lines)

**Features:**
- Type hints with dataclasses
- Keyword arguments for all tools
- Simple error handling
- Requests-based HTTP client

**Installation:**

```bash
cp output/python/mcp_client.py your-project/
pip install requests
```

**Usage:**

```python
from mcp_client import McpClient, McpClientConfig, McpResponse

# Configure client
config = McpClientConfig(
    transport='http',
    port=8080,
    timeout=30
)

client = McpClient(config)

# Call tools with keyword arguments
result = client.echo(message='Hello from Python')
if result.success:
    print(result.data)

# Calculator
result = client.calculate(operation='multiply', a=6, b=7)

# Process with progress
result = client.process_with_progress(
    data=['item1', 'item2', 'item3'],
    delay_ms=100
)

# Batch processing
result = client.batch_process(
    items=['task1', 'task2', 'task3'],
    operation='process'
)
```

**Error Handling:**

```python
result = client.echo(message='test')
if result.success:
    print(f"Success: {result.data}")
else:
    print(f"Error: {result.error}")
```

### Go SDK

**File:** `output/go/mcpclient/client.go`  
**Size:** ~18KB (~172 lines)

**Features:**
- Idiomatic Go interfaces
- Native net/http client
- Proper error handling
- No external dependencies

**Installation:**

```bash
cp -r output/go/mcpclient your-project/
```

**Usage:**

```go
package main

import (
    "fmt"
    "log"
    "time"
    
    "your-project/mcpclient"
)

func main() {
    config := mcpclient.Config{
        Transport: "http",
        Port:      8080,
        Timeout:   30 * time.Second,
    }
    
    client := mcpclient.NewClient(config)
    
    // Ping
    result, err := client.Ping(map[string]interface{}{})
    if err != nil {
        log.Fatal(err)
    }
    fmt.Println(result.Data)
    
    // Echo
    result, err = client.Echo(map[string]interface{}{
        "message": "Hello from Go",
    })
    
    // Calculate
    result, err = client.Calculate(map[string]interface{}{
        "operation": "divide",
        "a":         100.0,
        "b":         4.0,
    })
    
    // Transform data
    result, err = client.TransformData(map[string]interface{}{
        "data":      []string{"hello", "world"},
        "operation": "uppercase",
    })
}
```

## Tool Coverage

All 11 MCP tools are fully supported across all SDKs:

### Basic Tools (5)

| Tool | Parameters | Returns |
|------|------------|---------|
| `ping` | None | Server timestamp |
| `echo` | message (string) | Message + timestamp |
| `info` | None | Server metadata |
| `calculate` | operation, a, b | Calculation result |
| `evaluate` | expression (string) | Expression result |

### Advanced Tools (6)

| Tool | Parameters | Returns |
|------|------------|---------|
| `process_with_progress` | data (array), delay_ms | Processed items + duration |
| `batch_process` | items (array), operation | Batch results |
| `transform_data` | data (array), operation | Transformed data |
| `simulate_upload` | filename, size_bytes | Upload simulation results |
| `health_check` | None | System health status |
| `long_task` | duration_seconds | Task completion status |

## Transport Configuration

### Default Ports

| Transport | Port | Browser Support | SDK Support |
|-----------|------|----------------|-------------|
| stdio | N/A | No | No (desktop only) |
| sse | 8025 | Yes | All SDKs |
| websocket | 9001 | Yes | All SDKs |
| http | 8080 | Yes | All SDKs |
| http-stream | 8026 | Yes | All SDKs |
| grpc | 50051 | No (needs proxy) | All SDKs |

### Configuration Examples

**TypeScript:**
```typescript
const config: McpClientConfig = {
  transport: 'websocket',
  port: 9001,
  baseUrl: 'http://localhost',  // optional
  timeout: 30000                // optional, milliseconds
};
```

**Python:**
```python
config = McpClientConfig(
    transport='sse',
    port=8025,
    base_url='http://localhost',  # optional
    timeout=30                    # optional, seconds
)
```

**Go:**
```go
config := mcpclient.Config{
    Transport: "grpc",
    Port:      50051,
    BaseURL:   "http://localhost",  // optional
    Timeout:   30 * time.Second,    // optional
}
```

## Examples

Complete working examples are provided in `sdk-generators/templates/`:

- `typescript-example.ts` - All tools and transports
- `python-example.py` - Comprehensive examples with error handling
- `go-example.go` - Idiomatic Go usage patterns

### Running Examples

```bash
# Ensure MCP server is running
cd mcp-boilerplate-rust
cargo run --release --features http -- --mode http

# TypeScript
cd sdk-generators/templates
ts-node typescript-example.ts

# Python
python3 python-example.py

# Go
go run go-example.go
```

## Customization

### Adding Custom Tools

Edit `sdk-generators/src/main.rs` in the `load_tool_schemas()` function:

```rust
ToolSchema {
    name: "my_custom_tool".to_string(),
    description: "Custom tool description".to_string(),
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

The tool will automatically appear in all generated SDKs.

### Modifying Transport Configuration

Edit `sdk-generators/src/main.rs` in the `load_transport_configs()` function:

```rust
TransportConfig {
    name: "custom_transport".to_string(),
    port: Some(9999),
    protocol: "http".to_string(),
    supports_browser: true,
}
```

## Type Mapping

JSON Schema types are automatically converted to language-specific types:

| JSON Schema | TypeScript | Python | Go |
|------------|------------|--------|-----|
| string | string | str | string |
| number | number | float | float64 |
| integer | number | int | int |
| boolean | boolean | bool | bool |
| array | T[] | List[T] | []T |
| object | Record<string, any> | Dict[str, Any] | map[string]interface{} |
| enum | union type | str (enum values) | string |

## Testing

### Integration Testing

```bash
# Start MCP server (all transports)
cargo run --release --features full -- --mode http

# Test TypeScript SDK
cd sdk-generators/output/typescript
npm install
ts-node ../../templates/typescript-example.ts

# Test Python SDK
cd ../python
pip install -r requirements.txt
python3 ../../templates/python-example.py

# Test Go SDK
cd ../go
go mod tidy
go run ../../templates/go-example.go
```

### Validation Checklist

- [ ] All 11 tools generate correctly
- [ ] Type definitions are accurate
- [ ] All 6 transports work
- [ ] Error handling is proper
- [ ] Examples run successfully
- [ ] Documentation is complete

## Performance

### Generation Speed
- TypeScript: <100ms
- Python: <100ms
- Go: <100ms
- Total: <500ms

### Generated Code Size
- TypeScript: ~15KB (209 lines)
- Python: ~12KB (111 lines)
- Go: ~18KB (172 lines)

### Runtime Performance
All SDKs have minimal overhead:
- HTTP latency: 20-25ms
- Memory: <1MB per client
- No runtime code generation

## Best Practices

### TypeScript
- Enable strict mode in tsconfig.json
- Use async/await for all calls
- Handle promise rejections
- Implement retry logic for network errors

### Python
- Use type hints for better IDE support
- Handle exceptions properly
- Close connections when done
- Use context managers where applicable

### Go
- Always check errors explicitly
- Use defer for cleanup
- Set appropriate timeouts
- Handle context cancellation

## Troubleshooting

### Generation Errors

**Issue:** Cargo build fails  
**Solution:**
```bash
cargo clean
cargo build --release
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
go mod tidy
```

### Connection Errors

**Issue:** Cannot connect to MCP server  
**Solution:** Ensure server is running:
```bash
cargo run --release --features http -- --mode http --bind 127.0.0.1:8080
```

### Type Errors

**Issue:** Type mismatches in generated code  
**Solution:** Regenerate SDKs:
```bash
cd sdk-generators
make clean
make generate
```

## Architecture

### Generator Flow

```
Tool Schemas (Rust) → Generator → Language Templates → SDK Files
                         ↓
                    Type Mapping
                         ↓
                  Code Generation
                         ↓
                   File Output
```

### Code Organization

```
sdk-generators/
├── src/
│   └── main.rs           # Generator logic
├── templates/            # Example files
│   ├── typescript-example.ts
│   ├── python-example.py
│   └── go-example.go
├── output/              # Generated SDKs
│   ├── typescript/
│   │   ├── mcp-client.ts
│   │   └── package.json
│   ├── python/
│   │   ├── mcp_client.py
│   │   └── requirements.txt
│   └── go/
│       ├── mcpclient/client.go
│       └── go.mod
└── Cargo.toml
```

## Production Checklist

- [ ] SDKs generated without errors
- [ ] All tests pass
- [ ] Examples work correctly
- [ ] Documentation complete
- [ ] Type definitions validated
- [ ] Error handling tested
- [ ] Performance verified
- [ ] Security reviewed

## Roadmap

### Version 0.5.0
- [x] TypeScript SDK
- [x] Python SDK
- [x] Go SDK
- [ ] Rust client SDK
- [ ] OpenAPI spec generation

### Version 1.0.0
- [ ] Java/Kotlin SDK
- [ ] C# SDK
- [ ] Swift SDK
- [ ] GraphQL schema generation
- [ ] SDK versioning system

## Contributing

### Adding a New Language

1. Create generator function in `src/main.rs`
2. Implement type mapping
3. Generate example code
4. Add to `generate_all()`
5. Create example template
6. Document usage
7. Test thoroughly

## Resources

- **Generator Code:** `sdk-generators/src/main.rs`
- **Examples:** `sdk-generators/templates/`
- **Quick Start:** `sdk-generators/QUICKSTART.md`
- **Full Docs:** `sdk-generators/README.md`

## Support

- **GitHub:** https://github.com/netadx/mcp-boilerplate-rust
- **Email:** hello@netadx.ai
- **Issues:** https://github.com/netadx/mcp-boilerplate-rust/issues

---

**Version:** 0.4.0  
**Last Updated:** 2026-01-09 HCMC  
**Status:** Production Ready