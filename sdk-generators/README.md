# MCP Client SDK Generator

Auto-generate client SDKs for MCP Boilerplate Rust in multiple languages.

## Overview

This tool generates type-safe client libraries for interacting with the MCP server in:
- **TypeScript/JavaScript** - For Node.js and browser applications
- **Python** - For Python applications and scripts
- **Go** - For Go applications and services

All generated SDKs support the 6 transport modes: stdio, SSE, WebSocket, HTTP, HTTP Stream, and gRPC.

## Quick Start

### Generate All SDKs

```bash
cd sdk-generators
cargo run --release
```

Output:
```
sdk-generators/output/
├── typescript/
│   └── mcp-client.ts
├── python/
│   └── mcp_client.py
└── go/
    └── mcpclient/
        └── client.go
```

## Generated SDKs

### TypeScript SDK

**Location:** `output/typescript/mcp-client.ts`

**Features:**
- Full TypeScript type definitions
- All 11 tools with typed interfaces
- Support for SSE, WebSocket, HTTP, HTTP Stream, gRPC
- Browser and Node.js compatible
- Promise-based async API

**Usage:**

```typescript
import { McpClient } from './mcp-client';

const client = new McpClient({
  transport: 'http',
  port: 8080
});

// Type-safe tool calls
const result = await client.echo({ message: 'Hello' });
console.log(result.data);
```

**Installation:**

```bash
# Copy to your TypeScript project
cp output/typescript/mcp-client.ts src/

# No dependencies required (uses native fetch)
```

### Python SDK

**Location:** `output/python/mcp_client.py`

**Features:**
- Type hints with dataclasses
- All 11 tools with keyword arguments
- Support for all transport modes
- Requests-based HTTP client
- Simple error handling

**Usage:**

```python
from mcp_client import McpClient, McpClientConfig

config = McpClientConfig(transport='http', port=8080)
client = McpClient(config)

# Call tools with kwargs
result = client.echo(message='Hello from Python')
if result.success:
    print(result.data)
```

**Installation:**

```bash
# Copy to your Python project
cp output/python/mcp_client.py your_project/

# Install dependency
pip install requests
```

### Go SDK

**Location:** `output/go/mcpclient/client.go`

**Features:**
- Idiomatic Go interfaces
- All 11 tools with map arguments
- Support for all transport modes
- Native net/http client
- Proper error handling

**Usage:**

```go
import "github.com/your/path/mcpclient"

config := mcpclient.Config{
    Transport: "http",
    Port:      8080,
}
client := mcpclient.NewClient(config)

// Call tools with map arguments
result, err := client.Echo(map[string]interface{}{
    "message": "Hello from Go",
})
if err == nil && result.Success {
    fmt.Println(result.Data)
}
```

**Installation:**

```bash
# Copy to your Go project
cp -r output/go/mcpclient your_project/

# No external dependencies required
```

## Tool Coverage

All 11 MCP tools are fully supported:

### Basic Tools
- `ping` - Health check
- `echo` - Message validation with timestamp
- `info` - Server metadata
- `calculate` - Arithmetic operations (add, subtract, multiply, divide)
- `evaluate` - Math expression evaluation

### Advanced Tools
- `process_with_progress` - Data processing with progress notifications
- `batch_process` - Batch operations with logging
- `transform_data` - Array transformations (uppercase, lowercase, reverse, double)
- `simulate_upload` - File upload simulation with progress
- `health_check` - System health monitoring
- `long_task` - Long-running operations with progress updates

## Transport Configuration

### Default Ports

| Transport    | Port  | Browser Support |
|-------------|-------|-----------------|
| stdio       | N/A   | No              |
| sse         | 8025  | Yes             |
| websocket   | 9001  | Yes             |
| http        | 8080  | Yes             |
| http_stream | 8026  | Yes             |
| grpc        | 50051 | No (needs proxy)|

### Configuration Examples

**TypeScript:**
```typescript
const config = {
  transport: 'websocket',
  port: 9001,
  baseUrl: 'http://localhost', // optional
  timeout: 30000 // optional, ms
};
```

**Python:**
```python
config = McpClientConfig(
    transport='sse',
    port=8025,
    base_url='http://localhost',  # optional
    timeout=30  # optional, seconds
)
```

**Go:**
```go
config := Config{
    Transport: "grpc",
    Port:      50051,
    BaseURL:   "http://localhost", // optional
    Timeout:   30 * time.Second,   // optional
}
```

## Examples

Example files demonstrate all features:

```bash
# TypeScript examples
templates/typescript-example.ts

# Python examples
templates/python-example.py

# Go examples
templates/go-example.go
```

Run examples:

```bash
# Ensure MCP server is running
cd ..
cargo run --release --features http -- --mode http

# TypeScript (Node.js)
cd sdk-generators/templates
ts-node typescript-example.ts

# Python
python3 python-example.py

# Go
go run go-example.go
```

## Customization

### Modify Tool Schemas

Edit `src/main.rs` in the `load_tool_schemas()` function:

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

### Add Transport

Edit `src/main.rs` in the `load_transport_configs()` function:

```rust
TransportConfig {
    name: "custom_transport".to_string(),
    port: Some(9999),
    protocol: "http".to_string(),
    supports_browser: true,
}
```

### Regenerate

```bash
cargo run --release
```

## Architecture

### Generator Flow

```
Tool Schemas → Code Generator → Language-Specific Templates → Output Files
     ↓              ↓                    ↓                         ↓
  JSON Schema   TypeScript Gen      TS Template            mcp-client.ts
  Validation    Python Gen          Py Template            mcp_client.py
  Transport     Go Gen              Go Template            client.go
```

### Type Mapping

| JSON Schema | TypeScript | Python | Go |
|------------|------------|--------|-----|
| string     | string     | str    | string |
| number     | number     | float  | float64 |
| boolean    | boolean    | bool   | bool |
| array      | T[]        | List[T]| []T |
| object     | Record     | Dict   | map[string]interface{} |

## Testing

### Unit Tests

```bash
cargo test
```

### Integration Tests

```bash
# Start MCP server
cd ..
cargo run --release --features full -- --mode http

# Test TypeScript SDK
cd sdk-generators/output/typescript
npm install -g ts-node typescript
ts-node ../../templates/typescript-example.ts

# Test Python SDK
cd ../python
python3 ../../templates/python-example.py

# Test Go SDK
cd ../go
go run ../../templates/go-example.go
```

### Validation

Each generated SDK is validated against:
- Type correctness
- API compatibility
- Transport support
- Error handling
- Documentation completeness

## Production Checklist

- [ ] Generated SDKs tested with all transports
- [ ] Error handling verified
- [ ] Type definitions validated
- [ ] Documentation reviewed
- [ ] Examples tested
- [ ] Performance benchmarked
- [ ] Security reviewed

## Troubleshooting

### SDK Generation Fails

**Issue:** Cargo build errors  
**Solution:** 
```bash
cargo clean
cargo build --release
```

### Import Errors

**TypeScript:**
```bash
# Ensure TypeScript is configured
tsc --init
```

**Python:**
```bash
# Install requests
pip install requests
```

**Go:**
```bash
# Initialize module
go mod init your-project
```

### Connection Errors

**Issue:** Cannot connect to server  
**Solution:** Ensure MCP server is running:
```bash
cd ..
cargo run --release --features http -- --mode http --bind 127.0.0.1:8080
```

### Type Errors

**Issue:** TypeScript type mismatches  
**Solution:** Regenerate SDK with latest schema:
```bash
cargo run --release
```

## Performance

### Generation Speed
- TypeScript SDK: <100ms
- Python SDK: <100ms
- Go SDK: <100ms
- Total: <500ms

### Generated Code Size
- TypeScript: ~15KB
- Python: ~12KB
- Go: ~18KB

## Best Practices

### TypeScript
- Use strict mode
- Enable type checking
- Handle promise rejections
- Implement retry logic

### Python
- Use type hints
- Handle exceptions properly
- Close connections
- Use context managers

### Go
- Check errors explicitly
- Use defer for cleanup
- Set timeouts
- Handle context cancellation

## Roadmap

### Version 0.5.0
- [ ] Rust client SDK
- [ ] Java/Kotlin SDK
- [ ] C# SDK
- [ ] Swift SDK
- [ ] OpenAPI spec generation

### Version 1.0.0
- [ ] GraphQL schema generation
- [ ] WebSocket streaming support
- [ ] Batch request optimization
- [ ] SDK versioning system

## Contributing

### Adding a New Language

1. Create generator function in `src/main.rs`:
```rust
fn generate_language(&self) -> String {
    // Generate code
}
```

2. Add to `generate_all()`:
```rust
let code = self.generate_language();
self.write_to_file(&code, &output_dir.join("language/client.ext"))?;
```

3. Create example template
4. Document usage
5. Test thoroughly

### Improving Generators

- Add more type mappings
- Enhance error messages
- Optimize generated code
- Improve documentation

## Support

- **GitHub:** https://github.com/netadx/mcp-boilerplate-rust
- **Email:** hello@netadx.ai
- **Issues:** https://github.com/netadx/mcp-boilerplate-rust/issues

## License

MIT License - see LICENSE file for details

---

**Version:** 0.4.0  
**Generated:** 2026-01-09 HCMC  
**MCP Protocol:** 2025-03-26