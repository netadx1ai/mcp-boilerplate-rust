# Filesystem Server Example

A complete MCP server implementation that provides file system operations, demonstrating the core MCP architecture with both STDIO and HTTP transports.

## Overview

This example showcases:
- **ReadFileTool**: A tool for reading file contents with security constraints
- **Dual Transport Support**: Both STDIO and HTTP transport layers
- **Security Features**: Path traversal protection and base directory constraints
- **Error Handling**: Comprehensive error responses for various failure scenarios
- **CLI Interface**: Command-line configuration with clap

## Features

### ReadFileTool
- **Secure File Reading**: Reads text files with path validation
- **Base Directory Constraint**: All file operations are restricted to a configurable base directory
- **Path Traversal Protection**: Prevents access to files outside the base directory
- **Comprehensive Error Handling**: Proper MCP error codes for different failure scenarios

### Transport Support
- **STDIO Transport**: Traditional pipe-based communication for integration with other tools
- **HTTP Transport**: RESTful API with JSON payloads for web-based integration

## Usage

### Command Line Options

```bash
filesystem-server [OPTIONS]

Options:
  -t, --transport <TRANSPORT>  Transport type to use [default: stdio] [possible values: stdio, http]
  -p, --port <PORT>           Port for HTTP transport [default: 3000]
      --host <HOST>           Host for HTTP transport [default: 127.0.0.1]
  -d, --debug                 Enable debug logging
  -b, --base-dir <BASE_DIR>   Base directory for file operations [default: .]
  -h, --help                  Print help
  -V, --version               Print version
```

### STDIO Transport

Run with STDIO transport for pipe-based communication:

```bash
cargo run --bin filesystem-server -- --transport stdio
```

Send JSON-formatted MCP requests via stdin:

```json
{"method": "tools/call", "params": {"name": "read_file", "arguments": {"path": "README.md"}}}
```

### HTTP Transport

Run with HTTP transport for RESTful API:

```bash
cargo run --bin filesystem-server -- --transport http --port 3000
```

The server will start on `http://127.0.0.1:3000` with the following endpoints:

#### Available Endpoints

- **GET /health** - Health check endpoint
- **POST /mcp/tools/call** - Call a specific tool
- **GET /mcp/tools/list** - List available tools
- **POST /mcp/request** - Generic MCP request endpoint

#### Example HTTP Requests

**Read a file:**
```bash
curl -X POST http://127.0.0.1:3000/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{"name": "read_file", "arguments": {"path": "README.md"}}'
```

**List available tools:**
```bash
curl http://127.0.0.1:3000/mcp/tools/list
```

**Health check:**
```bash
curl http://127.0.0.1:3000/health
```

## Security

### Path Security
The server implements several security measures:

1. **Base Directory Constraint**: All file operations are restricted to the specified base directory
2. **Path Traversal Protection**: Attempts to access files outside the base directory (e.g., `../../../etc/passwd`) are blocked
3. **Path Canonicalization**: All paths are resolved and validated before file operations

### Error Handling
The server provides appropriate MCP error codes:

- **ResourceNotFound**: When the requested file doesn't exist
- **PermissionDenied**: When trying to access files outside the base directory
- **InvalidParams**: When required parameters are missing
- **InternalError**: For other file system errors

## Example Responses

### Successful File Read
```json
{
  "result": {
    "_type": "toolResult",
    "content": [
      {
        "type": "text",
        "text": "# File Contents\nThis is the content of the file..."
      }
    ],
    "isError": false
  }
}
```

### Error Response
```json
{
  "error": {
    "code": -32001,
    "message": "Resource 'nonexistent.txt' not found"
  }
}
```

## Development

### Building
```bash
cargo build --bin filesystem-server
```

### Testing
```bash
cargo test --package filesystem-server
```

The test suite includes:
- File reading functionality
- Path traversal protection
- Error handling scenarios
- Tool metadata validation

### Debug Mode
Enable debug logging for development:

```bash
cargo run --bin filesystem-server -- --debug --transport stdio
```

## Architecture

This example demonstrates key MCP concepts:

1. **Tool Implementation**: Shows how to implement the `McpTool` trait
2. **Server Configuration**: Uses the builder pattern for server setup
3. **Transport Abstraction**: Works with multiple transport types
4. **Error Handling**: Proper MCP error code usage
5. **Security**: Input validation and access control

The implementation follows the project's architectural principles:
- Tools are discrete business logic units
- Server acts as an orchestrator
- Transport layer is just message passing
- Core protocol logic is transport-agnostic

## Integration

This server can be integrated with MCP clients or other tools that support the MCP protocol. The dual transport support makes it suitable for:

- **CLI Integration**: Use STDIO transport with shell scripts and command-line tools
- **Web Integration**: Use HTTP transport for browser-based or web service integration
- **Development**: HTTP transport provides easy testing with curl and other HTTP clients

## Files

- `src/main.rs` - Complete server implementation with ReadFileTool
- `Cargo.toml` - Dependencies and configuration
- `README.md` - This documentation