# MCP Client - Generated Rust SDK (Race Car Edition 🏎️)

High-performance, idiomatic Rust SDK for Model Context Protocol.

## Features

- ✅ Zero-cost abstractions
- ✅ Custom error types (no `Box<dyn Error>`)
- ✅ Borrowing optimizations (`&str` vs `String`)
- ✅ Async/await on Tokio
- ✅ Type-safe pattern matching
- ✅ Production-ready performance

## Installation

```toml
[dependencies]
mcp-client = { path = "./rust" }
tokio = { version = "1.35", features = ["full"] }
```

## Usage

```rust
use mcp_client::{McpClient, HttpTransport, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let transport = HttpTransport::new("http://127.0.0.1:8080");
    let mut client = McpClient::new(transport);
    
    // Connect
    let server_info = client.connect().await?;
    println!("Connected to: {}", server_info.server_info.name);
    
    // Echo (with borrowing)
    let result = client.echo("Hello, MCP!").await?;
    println!("Echo: {}", result);
    
    // Calculate (zero-copy)
    let result = client.calculate("add", 10.0, 5.0).await?;
    println!("10 + 5 = {}", result);
    
    // Evaluate
    let result = client.evaluate("2 * (3 + 4)").await?;
    println!("Result: {}", result);
    
    client.close().await?;
    Ok(())
}
```

## Performance

This SDK is optimized for:
- Zero allocations where possible
- Borrowing instead of cloning
- Custom error types for pattern matching
- Efficient async patterns

## Generated

Auto-generated from MCP Boilerplate Rust v0.4.0

**Quality:** Race Car 🏎️ (not sedan!)
