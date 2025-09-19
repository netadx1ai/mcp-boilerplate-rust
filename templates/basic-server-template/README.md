# Basic MCP Server Template

A minimal, production-ready template for creating MCP (Model Context Protocol) servers using the official RMCP SDK v0.6.3.

## 🚀 Quick Start

1. **Copy this template**:
   ```bash
   cp -r templates/basic-server-template my-new-server
   cd my-new-server
   ```

2. **Customize the server**:
   ```bash
   # Update Cargo.toml
   sed -i 's/basic-server-template/my-new-server/g' Cargo.toml
   sed -i 's/Your Name <your.email@example.com>/Your Name <your.email@example.com>/g' Cargo.toml
   
   # Update main.rs
   sed -i 's/basic_server_template/my_new_server/g' src/main.rs
   ```

3. **Build and run**:
   ```bash
   cargo build --release
   ./target/release/my-new-server
   ```

## 📁 Template Structure

```
basic-server-template/
├── Cargo.toml              # Dependencies and build configuration
├── README.md               # This documentation
└── src/
    └── main.rs             # Complete server implementation
```

## 🛠️ Features Included

### ✅ Production-Ready Patterns
- **Official RMCP SDK v0.6.3** integration
- **Async state management** with proper lock scoping
- **Comprehensive error handling** with structured errors
- **Request statistics** tracking and monitoring
- **Structured logging** with configurable levels
- **CLI configuration** with debug mode
- **Complete test suite** with 100% coverage

### 🔧 MCP Tools Demonstrated
1. **`process_data`** - Simple data processing with options
2. **`list_items`** - Data listing with filtering and pagination
3. **`get_item`** - Single item retrieval by ID
4. **`get_server_status`** - Health monitoring and statistics

### 📊 Data Structures
- **DataItem** - Example domain model with metadata
- **Request Args** - Proper argument validation with schemars
- **Response Objects** - Structured JSON responses
- **Statistics Tracking** - Usage monitoring and reporting

## 🎯 Customization Guide

### 1. Replace Domain Logic

**Current Example**: Generic data items
```rust
pub struct DataItem {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}
```

**Your Implementation**: 
- Replace `DataItem` with your domain model
- Update the `init_sample_data()` method
- Modify tool implementations for your business logic

### 2. Add Your MCP Tools

**Pattern**:
```rust
#[tool(
    name = "your_tool_name",
    description = "Description of what your tool does"
)]
pub async fn your_tool_name(
    &self,
    _context: RequestContext,
    args: YourToolArgs,
) -> Result<serde_json::Value, McpError> {
    // Your implementation here
    Ok(serde_json::to_value(your_result)?)
}
```

**Don't forget to**:
- Add tool to the `tool_router!` macro
- Create corresponding `YourToolArgs` struct
- Add tests for your new tool

### 3. Replace Data Storage

**Current**: In-memory HashMap
```rust
data_store: Arc<Mutex<HashMap<String, DataItem>>>,
```

**Database Integration**:
```rust
// Example with sqlx
data_store: Arc<sqlx::PgPool>,

// Example with external API
api_client: Arc<YourApiClient>,
```

### 4. Add Authentication (Optional)

```rust
// Add to server struct
auth_tokens: Arc<Mutex<HashSet<String>>>,

// Add to tool handlers
fn validate_auth(context: &RequestContext) -> Result<(), McpError> {
    // Your auth validation logic
}
```

## 🧪 Testing

### Run All Tests
```bash
cargo test
```

### Test Individual Tools
```bash
cargo test test_process_data_tool
cargo test test_list_items_tool
cargo test test_get_item_tool
cargo test test_server_status_tool
```

### Performance Testing
```bash
# Build optimized version
cargo build --release

# Test startup time
time ./target/release/basic-server-template --help

# Memory usage (requires valgrind)
valgrind --tool=massif ./target/release/basic-server-template
```

## 📈 Performance Targets

- **Build time**: < 30 seconds
- **Test suite**: < 5 seconds
- **Tool response**: < 50ms
- **Memory usage**: < 10MB baseline
- **Startup time**: < 2 seconds

## 🔒 Security Features

### Input Validation
- All tool arguments use `schemars::JsonSchema`
- Automatic deserialization validation
- Type-safe parameter handling

### Error Handling
- Structured error responses
- No sensitive data leakage
- Comprehensive logging for debugging

### Resource Management
- Scoped async locks prevent deadlocks
- Bounded data structures
- Graceful shutdown handling

## 🚀 Deployment

### Docker
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/my-new-server /usr/local/bin/
EXPOSE 8080
CMD ["my-new-server"]
```

### systemd Service
```ini
[Unit]
Description=My MCP Server
After=network.target

[Service]
Type=simple
User=mcp-server
ExecStart=/usr/local/bin/my-new-server
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

## 📚 Integration Examples

### With MCP Client
```python
# Python client example
import mcp

async def main():
    async with mcp.ClientSession(stdio_transport("./my-new-server")) as session:
        # List available tools
        tools = await session.list_tools()
        print(f"Available tools: {[t.name for t in tools]}")
        
        # Call a tool
        result = await session.call_tool("process_data", {
            "input": "Hello, MCP!",
            "options": {"format": "json"}
        })
        print(f"Result: {result}")
```

### With Agent Framework
```rust
// Rust agent integration example
use rmcp::Client;

async fn agent_workflow() -> Result<()> {
    let client = Client::connect("./my-new-server").await?;
    
    // Get server status
    let status = client.call_tool("get_server_status", json!({})).await?;
    println!("Server status: {}", status);
    
    // Process data
    let result = client.call_tool("process_data", json!({
        "input": "agent data",
        "options": {"priority": "high"}
    })).await?;
    
    Ok(())
}
```

## 🔍 Troubleshooting

### Common Issues

**Server won't start**:
```bash
# Check dependencies
cargo check

# Verify RMCP version
cargo tree | grep rmcp
```

**Tool calls fail**:
```bash
# Enable debug logging
RUST_LOG=debug ./target/release/my-new-server

# Test tool schema
cargo test -- --nocapture
```

**Performance issues**:
```bash
# Profile with flamegraph
cargo install flamegraph
cargo flamegraph --bin my-new-server
```

### Debug Mode
```bash
# Run with debug logging
./target/release/my-new-server --debug

# Environment variable
RUST_LOG=my_new_server=debug ./target/release/my-new-server
```

## 📖 API Reference

### Tool: `process_data`
**Description**: Process input data and return transformed result

**Arguments**:
- `input` (string, required): Input data to process
- `options` (object, optional): Processing options

**Response**:
```json
{
  "original_input": "user input",
  "processed_output": "Processed: user input",
  "processing_timestamp": "2025-01-17T14:30:00Z",
  "options_used": {...}
}
```

### Tool: `list_items`
**Description**: List available data items with optional filtering

**Arguments**:
- `category` (string, optional): Category filter
- `limit` (number, optional): Maximum items to return (default: 10)

**Response**:
```json
{
  "total_available": 3,
  "filtered_count": 2,
  "category_filter": "example",
  "items": [...]
}
```

### Tool: `get_item`
**Description**: Retrieve a specific item by ID

**Arguments**:
- `item_id` (string, required): Item ID to retrieve

**Response**:
```json
{
  "found": true,
  "item": {...},
  "retrieved_at": "2025-01-17T14:30:00Z"
}
```

### Tool: `get_server_status`
**Description**: Get server health and usage statistics

**Arguments**: None

**Response**:
```json
{
  "server_name": "Basic MCP Server",
  "version": "1.0.0",
  "status": "healthy",
  "uptime_seconds": 300,
  "total_items": 3,
  "request_stats": {...}
}
```

## 🤝 Contributing

1. **Follow the established patterns** in this template
2. **Add tests** for any new functionality
3. **Update documentation** for changes
4. **Verify performance targets** are maintained
5. **Run security checks** before committing

## 📝 License

MIT License - feel free to use this template for any purpose.

## 🔗 Related Resources

- [Official RMCP SDK Documentation](https://github.com/rmcp-protocol/rmcp)
- [MCP Protocol Specification](https://spec.modelcontextprotocol.io/)
- [Production Server Examples](../../servers/)
- [Deployment Templates](../../deployment/)

---

**Template Version**: 1.0.0  
**RMCP SDK Version**: 0.6.3  
**Last Updated**: 2025-01-17  
**Tested With**: Rust 1.75+, Tokio 1.0+