# Structured Content

Output schema validation for MCP tool results.

**Version:** 0.6.3  
**MCP Spec:** 2025-11-25

---

## Overview

Structured content allows tools to return both human-readable text and machine-parseable structured data. The structured data can be validated against JSON Schema.

Features:
- JSON Schema validation for tool outputs
- Combined text + structured output
- Schema registry for per-tool validation
- Pre-built schemas for common patterns

---

## Output Validation

### Basic Validation

```rust
use crate::mcp::structured_content::OutputValidator;

let schema = json!({
    "type": "object",
    "properties": {
        "temperature": { "type": "number" },
        "unit": { "type": "string", "enum": ["celsius", "fahrenheit"] }
    },
    "required": ["temperature", "unit"]
});

let validator = OutputValidator::new(schema);

// Valid output
let output = json!({"temperature": 22.5, "unit": "celsius"});
assert!(validator.validate(&output).is_ok());

// Invalid output
let invalid = json!({"temperature": "hot", "unit": "celsius"});
assert!(validator.validate(&invalid).is_err());
```

### Supported Validations

#### Type Checking

```rust
// String
{"type": "string"}

// Number (accepts integers too)
{"type": "number"}

// Integer
{"type": "integer"}

// Boolean
{"type": "boolean"}

// Array
{"type": "array"}

// Object
{"type": "object"}

// Null
{"type": "null"}
```

#### String Constraints

```rust
{
    "type": "string",
    "minLength": 1,
    "maxLength": 100
}
```

#### Number Constraints

```rust
{
    "type": "number",
    "minimum": 0,
    "maximum": 100
}
```

#### Enum Values

```rust
{
    "type": "string",
    "enum": ["red", "green", "blue"]
}
```

#### Array Constraints

```rust
{
    "type": "array",
    "items": { "type": "number" },
    "minItems": 1,
    "maxItems": 10
}
```

#### Object Properties

```rust
{
    "type": "object",
    "properties": {
        "name": { "type": "string" },
        "age": { "type": "integer" }
    },
    "required": ["name"],
    "additionalProperties": false
}
```

---

## Structured Output Builder

Build tool results with both text and structured content.

### Basic Usage

```rust
use crate::mcp::structured_content::StructuredOutput;

let result = StructuredOutput::new()
    .text("The temperature is 22.5°C with sunny skies.")
    .structured(json!({
        "temperature": 22.5,
        "unit": "celsius",
        "condition": "sunny"
    }))
    .build();

// result.content contains text
// result.structured_content contains JSON
```

### From Serializable Type

```rust
#[derive(Serialize)]
struct Weather {
    temperature: f64,
    unit: String,
}

let data = Weather { temperature: 22.5, unit: "celsius".into() };

let result = StructuredOutput::new()
    .text("Weather report")
    .structured_from(&data)
    .build();
```

### Error Output

```rust
let result = StructuredOutput::new()
    .text("Failed to fetch weather data")
    .structured(json!({
        "error": "API_ERROR",
        "message": "Service unavailable"
    }))
    .error()
    .build();

// result.is_error == Some(true)
```

### Validated Output

```rust
let validator = OutputValidator::new(weather_schema);

let result = StructuredOutput::new()
    .structured(json!({"temperature": 22.5, "unit": "celsius"}))
    .build_validated(&validator);

match result {
    Ok(output) => // Valid output
    Err(errors) => // Validation failed
}
```

---

## Output Schema Registry

Store and retrieve schemas per tool.

### Registration

```rust
use crate::mcp::structured_content::{OutputSchemaRegistry, OutputSchemas};

let mut registry = OutputSchemaRegistry::new();

// Register custom schema
registry.register("get_weather", json!({
    "type": "object",
    "properties": {
        "temperature": { "type": "number" },
        "unit": { "type": "string" }
    },
    "required": ["temperature", "unit"]
}));

// Use pre-built schema
registry.register("weather_tool", OutputSchemas::weather());
registry.register("api_tool", OutputSchemas::api_response());
```

### Validation

```rust
let output = json!({"temperature": 22.5, "unit": "celsius"});

match registry.validate("get_weather", &output) {
    Ok(()) => println!("Valid"),
    Err(errors) => {
        for err in errors {
            println!("{}: {}", err.path, err.message);
        }
    }
}
```

### Registry Methods

```rust
registry.has_schema("tool_name")      // Check if registered
registry.get("tool_name")             // Get validator
registry.tools_with_schemas()         // List all tools
```

---

## Pre-built Schemas

Common schemas for quick use.

### OutputSchemas::weather()

```rust
{
    "type": "object",
    "properties": {
        "temperature": { "type": "number" },
        "unit": { "type": "string", "enum": ["celsius", "fahrenheit", "kelvin"] },
        "description": { "type": "string" },
        "humidity": { "type": "number", "minimum": 0, "maximum": 100 }
    },
    "required": ["temperature", "unit"]
}
```

### OutputSchemas::api_response()

```rust
{
    "type": "object",
    "properties": {
        "success": { "type": "boolean" },
        "data": { "type": "object" },
        "error": {
            "type": "object",
            "properties": {
                "code": { "type": "string" },
                "message": { "type": "string" }
            }
        }
    },
    "required": ["success"]
}
```

### Helper Methods

```rust
OutputSchemas::string()           // {"type": "string"}
OutputSchemas::number()           // {"type": "number"}
OutputSchemas::array(items)       // {"type": "array", "items": ...}
OutputSchemas::object(props, req) // {"type": "object", ...}
OutputSchemas::enum_values(vals)  // {"type": "string", "enum": [...]}
```

---

## Validation Errors

### Error Structure

```rust
pub struct ValidationError {
    pub path: String,      // e.g., "root.user.email"
    pub message: String,   // Human-readable error
    pub expected: Option<String>,
    pub actual: Option<String>,
}
```

### Error Types

```rust
// Type mismatch
ValidationError::type_mismatch("root.age", "number", "string")

// Missing required field
ValidationError::missing_required("root", "email")

// Invalid enum value
ValidationError::invalid_enum("root.status", "unknown", &allowed_values)
```

### Handling Errors

```rust
match validator.validate(&output) {
    Ok(()) => println!("Valid"),
    Err(errors) => {
        for error in errors {
            println!("Error at {}: {}", error.path, error.message);
            if let Some(expected) = &error.expected {
                println!("  Expected: {}", expected);
            }
            if let Some(actual) = &error.actual {
                println!("  Got: {}", actual);
            }
        }
    }
}
```

---

## Example: Tool with Validated Output

```rust
fn execute_weather_tool(location: &str) -> CallToolResult {
    // Define schema
    let schema = json!({
        "type": "object",
        "properties": {
            "location": { "type": "string" },
            "temperature": { "type": "number" },
            "unit": { "type": "string" },
            "conditions": {
                "type": "array",
                "items": { "type": "string" }
            }
        },
        "required": ["location", "temperature", "unit"]
    });
    
    let validator = OutputValidator::new(schema);
    
    // Fetch weather data...
    let data = json!({
        "location": location,
        "temperature": 22.5,
        "unit": "celsius",
        "conditions": ["sunny", "mild"]
    });
    
    // Build validated output
    StructuredOutput::new()
        .text(format!("Weather in {}: 22.5°C, sunny and mild", location))
        .structured(data)
        .build_validated(&validator)
        .unwrap_or_else(|_| {
            StructuredOutput::new()
                .text("Failed to validate weather data")
                .error()
                .build()
        })
}
```

---

## API Reference

### OutputValidator

| Method | Description |
|--------|-------------|
| `new(schema)` | Create validator with JSON Schema |
| `validate(&value)` | Validate value, return errors if invalid |
| `validate_and_get(value)` | Validate and return value if valid |

### StructuredOutput

| Method | Description |
|--------|-------------|
| `new()` | Create empty builder |
| `text(string)` | Add text content |
| `structured(value)` | Set structured JSON |
| `structured_from(&T)` | Set from serializable type |
| `error()` | Mark as error |
| `build()` | Build CallToolResult |
| `build_validated(&validator)` | Build with validation |

### OutputSchemaRegistry

| Method | Description |
|--------|-------------|
| `new()` | Create empty registry |
| `register(name, schema)` | Register schema for tool |
| `get(name)` | Get validator for tool |
| `validate(name, value)` | Validate output for tool |
| `has_schema(name)` | Check if tool has schema |
| `tools_with_schemas()` | List all registered tools |

### ValidationError

| Field | Type | Description |
|-------|------|-------------|
| `path` | String | JSON path to error |
| `message` | String | Error description |
| `expected` | Option<String> | Expected value/type |
| `actual` | Option<String> | Actual value/type |

---

## References

- [MCP Tool Output Schema](https://modelcontextprotocol.io/specification/2025-11-25/server/tools#output-schemas)
- Source: `src/mcp/structured_content.rs`
