# SDK Comparison: sdk-generators vs Rust Client SDK

**Quick Answer:** They serve different purposes!

---

## Overview

| Feature | sdk-generators | mcp-client (Rust SDK) |
|---------|---------------|----------------------|
| **Type** | Code Generator Tool | Client Library |
| **Language** | Rust (the tool itself) | Rust (for use in Rust apps) |
| **Purpose** | Generates SDKs for OTHER languages | Direct use in Rust projects |
| **Output** | TypeScript, Python, Go code | None (it IS the library) |
| **Usage** | Run once to generate files | Import and use in your code |
| **Location** | `sdk-generators/` | `mcp-client/` |

---

## sdk-generators

### What It Is

A **code generation tool** written in Rust that automatically creates client SDKs in TypeScript, Python, and Go.

### Purpose

- Generate client libraries for non-Rust languages
- Auto-create type-safe bindings for all 11 tools
- Keep SDKs in sync with server changes
- Support multi-language ecosystems

### How It Works

```bash
cd sdk-generators
cargo run --release

# Generates:
# - output/typescript/mcp-client.ts
# - output/python/mcp_client.py
# - output/go/mcp_client.go
```

### What It Generates

**TypeScript SDK** (209 lines):
```typescript
export class McpClient {
  async ping(): Promise<string> { ... }
  async echo(message: string): Promise<string> { ... }
  // ... all 11 tools
}
```

**Python SDK** (111 lines):
```python
class McpClient:
    def ping(self) -> str: ...
    def echo(self, message: str) -> str: ...
    # ... all 11 tools
```

**Go SDK** (172 lines):
```go
type McpClient struct { ... }
func (c *McpClient) Ping() (string, error) { ... }
func (c *McpClient) Echo(message string) (string, error) { ... }
// ... all 11 tools
```

### Use Case

**You have a JavaScript/Python/Go application** and need to connect to an MCP server:

1. Run the generator once
2. Copy generated SDK to your project
3. Use the SDK in your app

```javascript
// In your Node.js app
import { McpClient } from './mcp-client';

const client = new McpClient('http://localhost:8080');
const result = await client.echo('Hello');
```

---

## mcp-client (Rust Client SDK)

### What It Is

A **native Rust library** that you import and use directly in Rust applications.

### Purpose

- Provide MCP client functionality for Rust apps
- Type-safe, native Rust implementation
- No code generation needed
- Direct integration with Rust projects

### How It Works

```toml
# In your Cargo.toml
[dependencies]
mcp-client = { path = "../mcp-client" }
```

```rust
// In your Rust app
use mcp_client::{McpClient, TransportFactory};

#[tokio::main]
async fn main() {
    let transport = TransportFactory::http("http://127.0.0.1:8080".to_string(), 30);
    let client = McpClient::new(transport);
    
    client.connect().await?;
    let result = client.echo("Hello").await?;
}
```

### What It Provides

- **Real Rust code** (not generated)
- Complete type definitions
- Multi-transport support
- Async/await on Tokio
- Error handling
- Connection management

### Use Case

**You're building a Rust application** and need to connect to an MCP server:

1. Add mcp-client dependency
2. Import and use directly
3. Full Rust type safety and tooling

---

## Side-by-Side Example

### Scenario: Connect to MCP server and call echo tool

#### Using sdk-generators → TypeScript

```bash
# Step 1: Generate SDK
cd sdk-generators && cargo run --release

# Step 2: Copy to your project
cp output/typescript/mcp-client.ts ../my-node-app/
```

```typescript
// Step 3: Use in your Node.js app
import { McpClient } from './mcp-client';

const client = new McpClient('http://localhost:8080');
await client.connect();
const result = await client.echo('Hello from TypeScript!');
```

#### Using mcp-client → Rust

```toml
# Step 1: Add dependency
[dependencies]
mcp-client = { path = "../mcp-client" }
```

```rust
// Step 2: Use directly
use mcp_client::{McpClient, TransportFactory};

let transport = TransportFactory::http("http://127.0.0.1:8080".to_string(), 30);
let client = McpClient::new(transport);
client.connect().await?;
let result = client.echo("Hello from Rust!").await?;
```

---

## When to Use Each

### Use sdk-generators When:

✅ Building JavaScript/TypeScript applications  
✅ Building Python applications  
✅ Building Go applications  
✅ Need SDKs for multiple languages  
✅ Want auto-generated, consistent APIs  
✅ Need to distribute SDKs to other teams  

### Use mcp-client When:

✅ Building Rust applications  
✅ Need native Rust integration  
✅ Want compile-time type safety  
✅ Building high-performance clients  
✅ Need full Rust ecosystem support  
✅ Want async/await on Tokio  

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────┐
│         MCP Boilerplate Rust Project            │
├─────────────────────────────────────────────────┤
│                                                 │
│  ┌──────────────────┐    ┌──────────────────┐  │
│  │  sdk-generators  │    │   mcp-client     │  │
│  │  (Code Gen Tool) │    │  (Rust Library)  │  │
│  └────────┬─────────┘    └────────┬─────────┘  │
│           │                       │             │
│           │ Generates             │ Used by     │
│           ▼                       ▼             │
│  ┌─────────────────┐    ┌──────────────────┐   │
│  │  TypeScript SDK │    │  Your Rust App   │   │
│  │  Python SDK     │    │                  │   │
│  │  Go SDK         │    │  ┌────────────┐  │   │
│  └─────────────────┘    │  │ use mcp-   │  │   │
│                         │  │   client;  │  │   │
│                         │  └────────────┘  │   │
│                         └──────────────────┘   │
└─────────────────────────────────────────────────┘
```

---

## Complete Picture

The MCP Boilerplate Rust project now provides:

### Server Side
- 6 transport modes (stdio, SSE, WebSocket, HTTP, HTTP-Stream, gRPC)
- 11 production tools
- Load balancing
- Metrics and tracing

### Client Side (4 Options!)

1. **TypeScript SDK** - Generated by sdk-generators
2. **Python SDK** - Generated by sdk-generators
3. **Go SDK** - Generated by sdk-generators
4. **Rust SDK** - Native library (mcp-client)

---

## Summary

| Aspect | sdk-generators | mcp-client |
|--------|---------------|------------|
| **What** | Tool that generates code | Library you import |
| **For** | TypeScript, Python, Go | Rust |
| **How** | Run generator, copy output | Add dependency, use directly |
| **Lines** | Generates 492 lines total | 1,400 lines of library code |
| **Maintenance** | Regenerate when tools change | Update dependency version |
| **Type Safety** | Generated types | Native Rust types |
| **Performance** | Depends on target language | Native Rust performance |

**Bottom Line:**
- **sdk-generators** = Make SDKs FOR other languages
- **mcp-client** = USE directly IN Rust projects

Both are valuable! They serve different ecosystems in the MCP landscape.

---

**Version:** 0.5.0  
**Date:** 2026-01-09 HCMC