# MCP Tool Output Schemas

**Version**: 0.3.1  
**Protocol**: MCP 2025-03-26  
**Status**: ✅ Fully Implemented  
**Last Updated**: 2026-01-08

## Overview

Tool output schemas define the structure of data returned by MCP tools. They enable:
- **Type Safety**: Clients know what to expect from tool responses
- **Validation**: Automatic verification of tool outputs
- **IDE Support**: Autocomplete and type checking
- **Self-Documentation**: Tools describe their own output format
- **LLM Integration**: Better understanding of structured data

## Implementation Status

All 5 tools in this server have output schemas automatically generated:

| Tool | Output Schema | Status |
|------|---------------|--------|
| echo | `EchoResponse` | ✅ Complete |
| ping | `PingResponse` | ✅ Complete |
| info | `InfoResponse` | ✅ Complete |
| calculate | `CalculateResponse` | ✅ Complete |
| evaluate | `EvaluateResponse` | ✅ Complete |

## How It Works

### Automatic Schema Generation

The rmcp SDK automatically generates output schemas when tools return `Json<T>` where `T` implements `JsonSchema`:

```rust
use rmcp::{Json, tool};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct EchoResponse {
    pub message: String,
    pub timestamp: String,
}

#[tool(description = "Echo back a message")]
async fn echo(&self, params: Parameters<EchoRequest>) -> Result<Json<EchoResponse>, McpError> {
    Ok(Json(EchoResponse {
        message: params.0.message,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}
```

The `#[tool]` macro detects `Json<EchoResponse>` and automatically:
1. Generates JSON Schema from `EchoResponse`
2. Adds `outputSchema` field to tool definition
3. Validates schema has root type "object" (MCP requirement)

## Output Schema Definitions

### 1. Echo Tool

**Purpose**: Echo back a message with timestamp

**Output Schema**:
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "message": {
      "type": "string"
    },
    "timestamp": {
      "type": "string"
    }
  },
  "required": ["message", "timestamp"],
  "title": "EchoResponse"
}
```

**Example Output**:
```json
{
  "message": "Hello, World!",
  "timestamp": "2026-01-08T19:30:00.123456Z"
}
```

### 2. Ping Tool

**Purpose**: Simple connectivity test

**Output Schema**:
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "response": {
      "type": "string"
    },
    "timestamp": {
      "type": "string"
    }
  },
  "required": ["response", "timestamp"],
  "title": "PingResponse"
}
```

**Example Output**:
```json
{
  "response": "pong",
  "timestamp": "2026-01-08T19:30:00.123456Z"
}
```

### 3. Info Tool

**Purpose**: Server metadata and version information

**Output Schema**:
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "tool": {
      "type": "string"
    },
    "version": {
      "type": "string"
    },
    "description": {
      "type": "string"
    },
    "timestamp": {
      "type": "string"
    }
  },
  "required": ["tool", "version", "description", "timestamp"],
  "title": "InfoResponse"
}
```

**Example Output**:
```json
{
  "tool": "mcp-boilerplate-rust",
  "version": "0.3.1",
  "description": "MCP Boilerplate Rust Server",
  "timestamp": "2026-01-08T19:30:00.123456Z"
}
```

### 4. Calculate Tool

**Purpose**: Perform basic arithmetic operations

**Output Schema**:
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "operation": {
      "type": "string"
    },
    "a": {
      "type": "number",
      "format": "double"
    },
    "b": {
      "type": "number",
      "format": "double"
    },
    "result": {
      "type": "number",
      "format": "double"
    },
    "timestamp": {
      "type": "string"
    }
  },
  "required": ["operation", "a", "b", "result", "timestamp"],
  "title": "CalculateResponse"
}
```

**Example Output**:
```json
{
  "operation": "add",
  "a": 10.0,
  "b": 5.0,
  "result": 15.0,
  "timestamp": "2026-01-08T19:30:00.123456Z"
}
```

### 5. Evaluate Tool

**Purpose**: Evaluate mathematical expressions

**Output Schema**:
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "expression": {
      "type": "string"
    },
    "result": {
      "type": "number",
      "format": "double"
    },
    "timestamp": {
      "type": "string"
    }
  },
  "required": ["expression", "result", "timestamp"],
  "title": "EvaluateResponse"
}
```

**Example Output**:
```json
{
  "expression": "2 + 3 * 4",
  "result": 14.0,
  "timestamp": "2026-01-08T19:30:00.123456Z"
}
```

## Adding Output Schemas to New Tools

### Step 1: Define Response Type

```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MyToolResponse {
    pub data: String,
    pub count: u32,
    pub timestamp: String,
}
```

**Requirements**:
- Implement `Serialize` for JSON encoding
- Implement `Deserialize` for JSON decoding
- Implement `JsonSchema` for schema generation
- Use descriptive field names
- Include `timestamp` for auditability

### Step 2: Return Json<T> from Tool

```rust
use rmcp::{Json, tool, ErrorData as McpError};
use rmcp::handler::server::wrapper::Parameters;

#[tool(description = "My custom tool")]
async fn my_tool(
    &self,
    params: Parameters<MyToolRequest>,
) -> Result<Json<MyToolResponse>, McpError> {
    let response = MyToolResponse {
        data: params.0.input,
        count: 42,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    Ok(Json(response))
}
```

**The `Json<T>` wrapper**:
- Automatically generates `outputSchema` for the tool
- Validates schema has root type "object"
- Serializes response to JSON in tool result
- No manual schema definition needed!

### Step 3: Verify Schema Generation

Run the test suite:
```bash
./scripts/test_output_schemas.sh
```

Or manually check:
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | ./target/release/mcp-boilerplate-rust --mode stdio | jq '.result.tools[] | select(.name == "my_tool") | .outputSchema'
```

## Best Practices

### 1. Always Include Timestamps

```rust
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Response {
    pub data: String,
    pub timestamp: String,  // ✅ Good - enables auditability
}
```

### 2. Use Descriptive Field Names

```rust
// ❌ Bad
pub struct Response {
    pub d: String,
    pub c: u32,
}

// ✅ Good
pub struct Response {
    pub data: String,
    pub count: u32,
}
```

### 3. Document Complex Types

```rust
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ComplexResponse {
    /// The main result data
    pub result: String,
    
    /// Number of items processed
    pub items_count: u32,
    
    /// ISO 8601 timestamp
    pub timestamp: String,
}
```

### 4. Use Appropriate Types

```rust
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Response {
    pub count: u32,           // ✅ Integers for counts
    pub price: f64,           // ✅ Floats for decimals
    pub enabled: bool,        // ✅ Booleans for flags
    pub timestamp: String,    // ✅ Strings for timestamps (ISO 8601)
}
```

### 5. Validate Output Matches Schema

```rust
#[tool(description = "My tool")]
async fn my_tool(&self) -> Result<Json<MyResponse>, McpError> {
    let response = MyResponse {
        data: calculate_data(),
        count: count_items(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    // The Json<T> wrapper ensures the response matches the schema
    Ok(Json(response))
}
```

## Schema Validation

### Automatic Validation

The rmcp SDK validates:
1. Schema has root type "object" (MCP requirement)
2. Schema is valid JSON Schema (draft 2020-12)
3. Response can be serialized to JSON

**At Compile Time**:
```rust
// This will fail to compile if MyResponse doesn't implement JsonSchema
async fn my_tool(&self) -> Result<Json<MyResponse>, McpError> {
    // ...
}
```

**At Runtime**:
```rust
// This will panic if schema is invalid
let schema = schema_for_output::<MyResponse>()?;
```

### Manual Testing

Use the provided test script:
```bash
# Test all tools have valid output schemas
./scripts/test_output_schemas.sh

# Expected output:
# ✓ All 5 tools have output schemas
# ✓ Echo output schema valid (message, timestamp)
# ✓ Ping output schema valid (response, timestamp)
# ✓ Calculate output schema valid (operation, a, b, result, timestamp)
# ✓ Evaluate output schema valid (expression, result, timestamp)
# ✓ Actual outputs match declared schemas
```

## Advanced Patterns

### Optional Fields

```rust
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Response {
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
    pub timestamp: String,
}
```

**Generated Schema**:
```json
{
  "type": "object",
  "properties": {
    "data": { "type": "string" },
    "metadata": { "type": "string" },
    "timestamp": { "type": "string" }
  },
  "required": ["data", "timestamp"]
}
```

### Nested Objects

```rust
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct UserInfo {
    pub name: String,
    pub age: u32,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Response {
    pub user: UserInfo,
    pub timestamp: String,
}
```

**Generated Schema**:
```json
{
  "type": "object",
  "properties": {
    "user": {
      "type": "object",
      "properties": {
        "name": { "type": "string" },
        "age": { "type": "integer" }
      },
      "required": ["name", "age"]
    },
    "timestamp": { "type": "string" }
  },
  "required": ["user", "timestamp"]
}
```

### Arrays

```rust
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Response {
    pub items: Vec<String>,
    pub count: usize,
    pub timestamp: String,
}
```

**Generated Schema**:
```json
{
  "type": "object",
  "properties": {
    "items": {
      "type": "array",
      "items": { "type": "string" }
    },
    "count": { "type": "integer" },
    "timestamp": { "type": "string" }
  },
  "required": ["items", "count", "timestamp"]
}
```

## Troubleshooting

### Issue: Schema Not Generated

**Problem**: Tool doesn't have `outputSchema` field

**Solution**: Ensure you're returning `Json<T>`:
```rust
// ❌ No schema
async fn my_tool(&self) -> Result<String, McpError> { }

// ✅ Has schema
async fn my_tool(&self) -> Result<Json<MyResponse>, McpError> { }
```

### Issue: Schema Validation Error

**Problem**: "outputSchema must have root type 'object'"

**Solution**: Your response type must be a struct, not a primitive:
```rust
// ❌ Invalid - String is not an object
async fn my_tool(&self) -> Result<Json<String>, McpError> { }

// ✅ Valid - MyResponse is a struct (object)
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MyResponse { pub data: String }

async fn my_tool(&self) -> Result<Json<MyResponse>, McpError> { }
```

### Issue: Missing JsonSchema Derive

**Problem**: Compile error "JsonSchema is not implemented"

**Solution**: Add `JsonSchema` to derive:
```rust
// ❌ Missing JsonSchema
#[derive(Serialize, Deserialize)]
pub struct MyResponse { }

// ✅ Has JsonSchema
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MyResponse { }
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use schemars::schema_for;

    #[test]
    fn test_response_schema() {
        let schema = schema_for!(MyResponse);
        
        // Verify schema properties
        assert!(schema.schema.object.is_some());
        assert!(schema.schema.object.unwrap().properties.contains_key("data"));
    }
}
```

### Integration Tests

Run the output schema test suite:
```bash
./scripts/test_output_schemas.sh
```

## Performance

**Schema Generation**: Happens once at server startup
**Runtime Overhead**: None - schemas are cached
**Binary Size**: +~50KB for schemars dependency
**Memory**: Negligible - schemas are Arc-wrapped

## Migration Guide

### From Non-Schema Tools

**Before**:
```rust
async fn my_tool(&self) -> Result<String, McpError> {
    Ok("result".to_string())
}
```

**After**:
```rust
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MyResponse {
    pub result: String,
    pub timestamp: String,
}

async fn my_tool(&self) -> Result<Json<MyResponse>, McpError> {
    Ok(Json(MyResponse {
        result: "result".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}
```

## References

- [MCP Specification - Output Schemas](https://modelcontextprotocol.io/specification/2025-03-26)
- [JSON Schema 2020-12](https://json-schema.org/draft/2020-12/schema)
- [schemars Documentation](https://docs.rs/schemars)
- [rmcp SDK Documentation](https://docs.rs/rmcp)

## Summary

✅ **All 5 tools have output schemas**  
✅ **Automatic generation via `Json<T>`**  
✅ **Full JSON Schema 2020-12 compliance**  
✅ **Validated with test suite**  
✅ **No manual schema writing needed**  

Output schemas provide type safety, self-documentation, and better LLM integration with zero boilerplate!

---

**Last Updated**: 2026-01-08 19:30 HCMC  
**Status**: Production Ready  
**Testing**: 7/7 tests passing