# Prompts and Resources Guide

This guide covers the implementation of MCP Prompts and Resources in the Rust boilerplate server.

## Table of Contents

- [Overview](#overview)
- [Prompts](#prompts)
  - [Available Prompts](#available-prompts)
  - [Using Prompts](#using-prompts)
  - [Adding Custom Prompts](#adding-custom-prompts)
- [Resources](#resources)
  - [Available Resources](#available-resources)
  - [Accessing Resources](#accessing-resources)
  - [Adding Custom Resources](#adding-custom-resources)
- [Testing](#testing)

## Overview

The MCP Boilerplate Rust server implements both Prompts and Resources capabilities:

- **Prompts**: Pre-defined message templates with customizable parameters
- **Resources**: Server-side data that can be accessed by clients

Both features are fully integrated with the MCP protocol and tested automatically.

## Prompts

Prompts are reusable templates that generate structured messages based on input parameters. They're useful for creating consistent AI interactions.

### Available Prompts

| Name | Description | Arguments | Required |
|------|-------------|-----------|----------|
| `code_review` | Generate code review prompts | `language` (string)<br>`focus` (string) | `language` required<br>`focus` optional |
| `explain_code` | Generate code explanation prompts | `complexity` (beginner/intermediate/advanced) | optional |
| `debug_help` | Generate debugging assistance prompts | `error_type` (compile/runtime/logic) | optional |

### Using Prompts

#### List Available Prompts

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "prompts/list"
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "prompts": [
      {
        "name": "code_review",
        "description": "Generate a code review prompt for analyzing code quality",
        "arguments": [
          {
            "name": "language",
            "description": "Programming language (e.g., rust, python, javascript)",
            "required": true
          },
          {
            "name": "focus",
            "description": "Review focus area (e.g., security, performance, style)",
            "required": false
          }
        ]
      }
    ]
  }
}
```

#### Get a Prompt

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "prompts/get",
  "params": {
    "name": "code_review",
    "arguments": {
      "language": "rust",
      "focus": "security"
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "description": "Generate a code review prompt for analyzing code quality",
    "messages": [
      {
        "role": "user",
        "content": {
          "type": "text",
          "text": "Please review the following rust code with a focus on security..."
        }
      }
    ]
  }
}
```

### Adding Custom Prompts

To add a new prompt, edit `src/prompts/mod.rs`:

**Step 1: Add Template Definition**

```rust
templates.insert(
    "my_prompt".to_string(),
    PromptTemplate {
        name: "my_prompt".to_string(),
        description: "Description of my prompt".to_string(),
        arguments: vec![
            PromptArgument {
                name: "param1".to_string(),
                description: "Description of param1".to_string(),
                required: true,
            },
        ],
    },
);
```

**Step 2: Add Message Generator**

```rust
fn generate_prompt_messages(
    &self,
    name: &str,
    arguments: &HashMap<String, String>,
) -> Result<Vec<PromptMessage>, String> {
    match name {
        "my_prompt" => {
            let param1 = arguments
                .get("param1")
                .ok_or("param1 argument required")?;

            Ok(vec![PromptMessage::new_text(
                PromptMessageRole::User,
                format!("Your prompt text with {}", param1),
            )])
        }
        // ... existing prompts
    }
}
```

**Step 3: Test**

```bash
cargo build --release
./scripts/test_prompts_resources.sh
```

## Resources

Resources provide access to server-side data and metadata. They use URI-based addressing.

### Available Resources

| URI | Name | Description | MIME Type |
|-----|------|-------------|-----------|
| `config://server` | Server Configuration | Current MCP server config and metadata | application/json |
| `info://capabilities` | Server Capabilities | List of enabled MCP capabilities | application/json |
| `doc://quick-start` | Quick Start Guide | Quick start guide for using this server | text/plain |
| `stats://usage` | Usage Statistics | Server usage statistics (stateless) | application/json |

### Accessing Resources

#### List Available Resources

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "resources/list"
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "resources": [
      {
        "uri": "config://server",
        "name": "Server Configuration",
        "description": "Current MCP server configuration and metadata",
        "mimeType": "application/json"
      }
    ]
  }
}
```

#### Read a Resource

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "resources/read",
  "params": {
    "uri": "config://server"
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "contents": [
      {
        "uri": "config://server",
        "mimeType": "application/json",
        "text": "{\"name\":\"mcp-boilerplate-rust\",\"version\":\"0.3.1\",...}"
      }
    ]
  }
}
```

### Adding Custom Resources

To add a new resource, edit `src/resources/mod.rs`:

**Step 1: Add Resource Metadata**

```rust
resources.insert(
    "myscheme://myresource".to_string(),
    ResourceMetadata {
        uri: "myscheme://myresource".to_string(),
        name: "My Resource".to_string(),
        description: "Description of my resource".to_string(),
        mime_type: "application/json".to_string(),
    },
);
```

**Step 2: Add Content Generator**

```rust
fn get_resource_content(&self, uri: &str) -> Result<String, String> {
    match uri {
        "myscheme://myresource" => Ok(serde_json::to_string_pretty(&serde_json::json!({
            "data": "your data here",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
        .unwrap()),
        // ... existing resources
    }
}
```

**Step 3: Test**

```bash
cargo build --release
./scripts/test_prompts_resources.sh
```

## Testing

### Run All Tests

```bash
# Test tools
./scripts/test_mcp.sh

# Test prompts and resources
./scripts/test_prompts_resources.sh

# Or run comprehensive tests
./scripts/verify_claude_ready.sh
```

### Manual Testing with curl (HTTP mode)

```bash
# Build HTTP mode
cargo build --release --features http

# Run server
./target/release/mcp-boilerplate-rust --mode http

# In another terminal:

# List prompts
curl -X GET http://localhost:8025/prompts

# List resources
curl -X GET http://localhost:8025/resources

# Get specific resource
curl -X POST http://localhost:8025/resources/read \
  -H "Content-Type: application/json" \
  -d '{"uri": "config://server"}'
```

### Example Client Code (Rust)

```rust
use serde_json::json;

// List prompts
let request = json!({
    "jsonrpc": "2.0",
    "id": 1,
    "method": "prompts/list"
});

// Get prompt with arguments
let request = json!({
    "jsonrpc": "2.0",
    "id": 2,
    "method": "prompts/get",
    "params": {
        "name": "code_review",
        "arguments": {
            "language": "rust",
            "focus": "performance"
        }
    }
});

// Read resource
let request = json!({
    "jsonrpc": "2.0",
    "id": 3,
    "method": "resources/read",
    "params": {
        "uri": "info://capabilities"
    }
});
```

## Architecture

### Prompts Registry

Located in `src/prompts/mod.rs`:

- **PromptRegistry**: Manages prompt templates
- **PromptTemplate**: Defines prompt structure
- **PromptArgument**: Defines required/optional parameters
- **generate_prompt_messages()**: Generates messages from templates

### Resources Registry

Located in `src/resources/mod.rs`:

- **ResourceRegistry**: Manages available resources
- **ResourceMetadata**: Defines resource metadata
- **get_resource_content()**: Generates resource content dynamically

### Integration

Both registries are integrated into `McpServer` in `src/mcp/stdio_server.rs`:

```rust
pub struct McpServer {
    tool_router: ToolRouter<Self>,
    prompt_registry: PromptRegistry,
    resource_registry: ResourceRegistry,
}
```

## Best Practices

### Prompts

1. **Keep prompts focused**: Each prompt should have a clear, single purpose
2. **Use meaningful arguments**: Argument names should be descriptive
3. **Validate required arguments**: Check for required arguments before generating
4. **Provide clear descriptions**: Help users understand what each prompt does
5. **Use consistent formatting**: Keep prompt text structured and readable

### Resources

1. **Use descriptive URIs**: Choose URI schemes that clearly indicate resource type
2. **Set correct MIME types**: Help clients interpret resource content
3. **Keep resources stateless**: Resources should be dynamically generated
4. **Provide metadata**: Include descriptions to help users understand resources
5. **Handle errors gracefully**: Return meaningful error messages for invalid URIs

## Common Issues

### Issue: Prompt Arguments Not Working

**Solution**: Ensure required arguments are provided and correctly validated in `get_prompt()`.

### Issue: Resource Not Found

**Solution**: Check that the URI exactly matches the registered resource URI in `ResourceRegistry`.

### Issue: JSON Serialization Errors

**Solution**: Verify that all resource content is valid JSON when using `application/json` MIME type.

## Performance Considerations

- Prompts and resources are generated on-demand
- No caching is implemented (stateless design)
- Memory usage is minimal as templates are small
- Response times typically < 10ms for both prompts and resources

## Security Notes

- Prompts do not execute code or access files
- Resources do not expose sensitive system information
- All content is statically defined or safely generated
- No external network calls or file system access
- Input validation on all prompt arguments

## Future Enhancements

Potential improvements:

1. **Dynamic Prompts**: Load prompts from configuration files
2. **Resource Templates**: Support URI templates with parameters
3. **Caching**: Cache frequently accessed resources
4. **Streaming**: Support streaming large resource content
5. **Pagination**: Implement pagination for large resource lists

## References

- [MCP Protocol Specification](https://spec.modelcontextprotocol.io/)
- [rmcp SDK Documentation](https://docs.rs/rmcp/)
- [Project CLAUDE.md](../CLAUDE.md)
- [Main README](../README.md)

---

**Last Updated**: 2026-01-08  
**Version**: 0.3.1  
**Status**: Production Ready