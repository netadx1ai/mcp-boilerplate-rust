# Rust SDK Architecture Decision: Hand-Written vs Generated

**Question:** Why not use sdk-generators for Rust client too?

**Short Answer:** We could! But hand-written Rust has significant advantages for this use case.

---

## The Trade-off

### Option A: Generated Rust SDK (Consistent Approach)

```
sdk-generators/
  └─ generates TypeScript, Python, Go, Rust ← Add Rust here
```

**Pros:**
- ✅ Consistent approach across ALL languages
- ✅ Single source of truth
- ✅ Auto-updates when server changes
- ✅ Less manual maintenance
- ✅ Guaranteed API parity

**Cons:**
- ❌ Generated Rust code is often verbose
- ❌ Can't leverage advanced Rust features (traits, lifetimes, zero-cost abstractions)
- ❌ Less idiomatic Rust patterns
- ❌ Generic async/await (not optimized for Tokio)
- ❌ Limited customization
- ❌ Harder to extend
- ❌ Generated code harder to debug

### Option B: Hand-Written Rust SDK (Current Approach)

```
mcp-client/
  └─ Native Rust library with idiomatic code
```

**Pros:**
- ✅ Fully idiomatic Rust
- ✅ Optimized for performance
- ✅ Proper Tokio async/await patterns
- ✅ Advanced type safety (traits, generics, lifetimes)
- ✅ Zero-cost abstractions
- ✅ Better error handling (Result, custom error types)
- ✅ Extensible and flexible
- ✅ Full control over implementation
- ✅ Easier to debug
- ✅ Better IDE support

**Cons:**
- ❌ More code to maintain (~1,400 lines)
- ❌ Manual updates when server changes
- ❌ Duplication of effort
- ❌ Inconsistent with other SDKs

---

## Why Hand-Written Wins for Rust

### 1. Rust is Special

Unlike TypeScript/Python/Go, Rust has unique features that generated code can't fully leverage:

**Generated Rust (typical):**
```rust
// Generic, safe, but not optimal
pub fn echo(&self, message: String) -> Result<String, Box<dyn Error>> {
    let response = self.client.post(&self.url)
        .json(&json!({ "message": message }))
        .send()
        .await?;
    Ok(response.text().await?)
}
```

**Hand-Written Rust:**
```rust
// Idiomatic, optimized, type-safe
pub async fn echo(&self, message: &str) -> Result<String> {
    let mut args = HashMap::new();
    args.insert("message".to_string(), json!(message));
    
    let result = self.call_tool("echo", Some(args)).await?;
    
    if let Some(Content::Text { text }) = result.content.first() {
        Ok(text.clone())
    } else {
        Err(McpClientError::InvalidResponse("Invalid echo response".into()))
    }
}
```

**Differences:**
- Custom error types (not `Box<dyn Error>`)
- Borrows instead of owned strings
- Pattern matching on enum variants
- Type-safe content extraction

### 2. Performance Matters

**Generated:**
- Generic HTTP client
- JSON serialization overhead
- String allocations
- Boxing errors

**Hand-Written:**
- Optimized Tokio usage
- Minimal allocations
- Zero-cost abstractions
- Efficient error handling

### 3. The Server is Already Rust

Since the MCP server is written in Rust:
- We already have Rust expertise
- Can share types/patterns
- Better integration potential
- Can evolve together

### 4. Rust Developers Expect Quality

Rust community has high standards:
- Idiomatic code is expected
- Performance optimization is valued
- Proper async/await patterns required
- Documentation quality matters

Generated code often falls short here.

---

## Real-World Comparison

### Generated SDK Example (TypeScript)

```typescript
// Simple, works well for TypeScript
export class McpClient {
  async echo(message: string): Promise<string> {
    const response = await fetch(`${this.url}/rpc`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method: 'tools/call',
        params: { name: 'echo', arguments: { message } }
      })
    });
    return response.json();
  }
}
```

This is great for TypeScript! Simple, clean, works.

### Generated Rust Would Look Like:

```rust
pub struct McpClient {
    url: String,
    client: reqwest::Client,
}

impl McpClient {
    pub async fn echo(&self, message: String) -> Result<String, Box<dyn std::error::Error>> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "echo",
                "arguments": { "message": message }
            }
        });
        
        let response = self.client
            .post(&format!("{}/rpc", self.url))
            .json(&body)
            .send()
            .await?;
            
        let json: serde_json::Value = response.json().await?;
        Ok(json.to_string())
    }
}
```

**Problems:**
- Takes `String` instead of `&str` (unnecessary allocation)
- Returns `Box<dyn Error>` (type erasure, harder to match)
- Direct JSON handling (no type safety)
- No validation
- Inefficient

### Hand-Written Rust:

```rust
pub struct McpClient {
    transport: Arc<Mutex<Box<dyn Transport>>>,
    request_id: AtomicU64,
    initialized: Arc<Mutex<bool>>,
}

impl McpClient {
    pub async fn echo(&self, message: &str) -> Result<String> {
        self.ensure_initialized().await?;
        
        let mut args = HashMap::new();
        args.insert("message".to_string(), json!(message));
        
        let result = self.call_tool("echo", Some(args)).await?;
        
        if let Some(Content::Text { text }) = result.content.first() {
            Ok(text.clone())
        } else {
            Err(McpClientError::InvalidResponse(
                "Invalid echo response".to_string()
            ))
        }
    }
}
```

**Advantages:**
- Borrows `message` (zero-copy)
- Custom error type (pattern matching)
- Type-safe content extraction
- Connection state management
- Proper async patterns

---

## Best of Both Worlds?

### Hybrid Approach (Possible Future)

We COULD offer both:

```
sdk-generators/
  └─ Generates simple Rust SDK (basic use cases)

mcp-client/
  └─ Advanced hand-written SDK (production use)
```

Users choose:
- **Generated Rust SDK**: Quick prototyping, simple scripts
- **Hand-Written SDK**: Production apps, performance-critical code

---

## Decision Matrix

| Criteria | Generated | Hand-Written | Winner |
|----------|-----------|--------------|--------|
| Idiomatic Code | ⭐⭐ | ⭐⭐⭐⭐⭐ | Hand-Written |
| Performance | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | Hand-Written |
| Maintenance | ⭐⭐⭐⭐⭐ | ⭐⭐ | Generated |
| Type Safety | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | Hand-Written |
| Extensibility | ⭐⭐ | ⭐⭐⭐⭐⭐ | Hand-Written |
| Consistency | ⭐⭐⭐⭐⭐ | ⭐⭐ | Generated |
| Learning Curve | ⭐⭐⭐⭐ | ⭐⭐⭐ | Generated |
| Production Ready | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | Hand-Written |

**Overall:** Hand-Written wins for production Rust use cases

---

## Recommendation

### Current Approach (v0.5.0): Keep Both Separate

**For non-Rust languages:** Use sdk-generators
- TypeScript, Python, Go don't need the same level of optimization
- Simple HTTP clients work great
- Generated code is clean enough
- Auto-updates are valuable

**For Rust:** Use hand-written mcp-client
- Leverage Rust's unique strengths
- Production-grade quality
- Performance optimization
- Idiomatic patterns

### Future Option: Add Generated Rust Too

We COULD add Rust to sdk-generators as a "simple/quick-start" option:

```
sdk-generators/output/
  ├── typescript/mcp-client.ts    (Simple, auto-generated)
  ├── python/mcp_client.py        (Simple, auto-generated)
  ├── go/mcp_client.go            (Simple, auto-generated)
  └── rust/mcp_client.rs          (Simple, auto-generated) ← NEW

mcp-client/                        (Advanced, hand-written) ← KEEP
```

Users choose based on needs:
- **Quick script?** Use generated Rust SDK
- **Production app?** Use mcp-client library

---

## Analogy

Think of it like cars:

**Generated SDKs (TypeScript/Python/Go):**
- Like a standard sedan
- Gets you from A to B
- Good enough for most use cases

**Generated Rust SDK:**
- Like a standard sedan with racing stripes
- Looks like a race car, but isn't optimized

**Hand-Written Rust SDK:**
- Like a hand-built race car
- Optimized for performance
- Custom-tuned for the track
- Worth the extra effort

For Rust developers building production systems, the "hand-built race car" is worth it.

---

## Conclusion

**Why not generate Rust SDK?**

We COULD, but hand-written is better because:

1. **Rust is unique** - Has features (traits, lifetimes, zero-cost) that generated code can't leverage
2. **Performance matters** - Hand-written can be optimized
3. **Quality expectations** - Rust community expects idiomatic code
4. **We have expertise** - Server is already Rust
5. **Production-ready** - Worth the investment for serious use

**Bottom Line:**
- Generated works great for TypeScript/Python/Go (simpler languages)
- Hand-written works better for Rust (complex, performance-focused language)
- Both approaches are valid - we chose based on language characteristics

**Could we add generated Rust too?** Yes! As a "simple/quick-start" option alongside the hand-written library.

---

**Version:** 0.5.0  
**Date:** 2026-01-09 HCMC  
**Decision:** Hand-written for production quality, generated could complement for simplicity