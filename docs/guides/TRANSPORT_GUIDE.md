# Transport Layer Guide

**Version:** v0.5.0-dev  
**Phase:** 1 Complete - Multi-transport architecture ready  
**Last Updated:** 2026-01-08

---

## Overview

The MCP Boilerplate Rust server now supports multiple transport methods through a unified `Transport` trait. This guide shows you how to use and extend the transport layer.

---

## Quick Start

### Using the Global Registry

```rust
use mcp_boilerplate_rust::transport::{
    TransportRegistry, TransportConfig, Transport
};

// Initialize registry with default transports
mcp_boilerplate_rust::transport::init_registry();

// Create a transport from configuration
let config = TransportConfig {
    transport_type: "stdio".to_string(),
    max_message_size: 10 * 1024 * 1024, // 10MB
    timeout_seconds: 30,
    ..Default::default()
};

let registry = TransportRegistry::global();
let mut transport = registry.create(config)?;

// Use the transport
transport.initialize().await?;
transport.send(message).await?;
let response = transport.receive().await?;
transport.shutdown().await?;
```

---

## Available Transports

### 1. stdio (✅ Available)

**Use Cases:**
- CLI tools and desktop applications
- Claude Desktop integration
- Process spawning and IPC

**Configuration:**
```rust
let config = TransportConfig {
    transport_type: "stdio".to_string(),
    max_message_size: 10 * 1024 * 1024,
    timeout_seconds: 30,
    ..Default::default()
};
```

**Capabilities:**
- ✅ Bidirectional (request/response)
- ❌ Server Push (synchronous only)
- ❌ Multi-connection (single process)
- ❌ Streaming
- ❌ Browser Compatible

**Example:**
```rust
use mcp_boilerplate_rust::transport::{StdioTransport, Transport};

let mut transport = StdioTransport::new();
transport.initialize().await?;

// Read from stdin
let request = transport.receive().await?;

// Write to stdout
transport.send(response).await?;

transport.shutdown().await?;
```

---

### 2. SSE - Server-Sent Events (🚧 Coming in Phase 2)

**Use Cases:**
- Real-time browser notifications
- Live progress updates
- Server-to-client push events

**Capabilities:**
- ❌ Bidirectional (one-way only)
- ✅ Server Push
- ✅ Multi-connection
- ✅ Streaming
- ✅ Browser Compatible

**Example (Future):**
```rust
// Will be available in Phase 2
let config = TransportConfig {
    transport_type: "sse".to_string(),
    bind_address: Some("127.0.0.1".to_string()),
    port: Some(8025),
    ..Default::default()
};

let mut transport = registry.create(config)?;
transport.initialize().await?;

// Broadcast to all connected clients
transport.broadcast(progress_update).await?;
```

---

### 3. WebSocket (🚧 Coming in Phase 3)

**Use Cases:**
- Real-time bidirectional communication
- Live collaboration features
- Chat-like interactions

**Capabilities:**
- ✅ Bidirectional
- ✅ Server Push
- ✅ Multi-connection
- ✅ Streaming
- ✅ Browser Compatible

---

### 4. HTTP Streaming (🚧 Coming in Phase 4)

**Use Cases:**
- Large file transfers
- Chunked data processing
- Download progress tracking

**Capabilities:**
- ✅ Bidirectional
- ❌ Server Push
- ✅ Multi-connection
- ✅ Streaming
- ✅ Browser Compatible

---

### 5. RPC/gRPC (🚧 Coming in Phase 5)

**Use Cases:**
- Microservices communication
- High-performance APIs
- Service mesh integration

**Capabilities:**
- ✅ Bidirectional
- ✅ Server Push (gRPC streaming)
- ✅ Multi-connection
- ✅ Streaming
- ⚠️ Browser Compatible (requires gRPC-Web)

---

## Transport Trait

### Core Interface

```rust
#[async_trait]
pub trait Transport: Send + Sync {
    // Get transport type name
    fn transport_type(&self) -> &str;
    
    // Get capabilities
    fn capabilities(&self) -> TransportCapabilities;
    
    // Send a message
    async fn send(&self, message: TransportMessage) 
        -> Result<(), TransportError>;
    
    // Receive a message
    async fn receive(&self) 
        -> Result<TransportMessage, TransportError>;
    
    // Initialize transport
    async fn initialize(&mut self) -> Result<(), TransportError>;
    
    // Shutdown transport
    async fn shutdown(&mut self) -> Result<(), TransportError>;
    
    // Check if ready
    fn is_ready(&self) -> bool;
    
    // Get connection count (multi-connection transports)
    fn connection_count(&self) -> usize;
    
    // Broadcast to all connections
    async fn broadcast(&self, message: TransportMessage) 
        -> Result<(), TransportError>;
}
```

---

## Creating Custom Transports

### Step 1: Implement Transport Trait

```rust
use mcp_boilerplate_rust::transport::{
    Transport, TransportCapabilities, TransportMessage,
    TransportError
};
use async_trait::async_trait;

pub struct MyCustomTransport {
    // Your transport state
}

#[async_trait]
impl Transport for MyCustomTransport {
    fn transport_type(&self) -> &str {
        "custom"
    }
    
    fn capabilities(&self) -> TransportCapabilities {
        TransportCapabilities {
            bidirectional: true,
            server_push: true,
            multi_connection: true,
            streaming: false,
            browser_compatible: true,
        }
    }
    
    async fn send(&self, message: TransportMessage) 
        -> Result<(), TransportError> 
    {
        // Implement sending logic
        Ok(())
    }
    
    async fn receive(&self) 
        -> Result<TransportMessage, TransportError> 
    {
        // Implement receiving logic
        todo!()
    }
    
    async fn initialize(&mut self) -> Result<(), TransportError> {
        // Initialize connections, bind ports, etc.
        Ok(())
    }
    
    async fn shutdown(&mut self) -> Result<(), TransportError> {
        // Clean up resources
        Ok(())
    }
    
    fn is_ready(&self) -> bool {
        // Check if transport is ready
        true
    }
}
```

### Step 2: Create Factory

```rust
use mcp_boilerplate_rust::transport::{
    TransportFactory, TransportConfig, Transport, TransportError
};

pub struct MyCustomTransportFactory;

impl TransportFactory for MyCustomTransportFactory {
    fn create(&self, config: TransportConfig) 
        -> Result<Box<dyn Transport>, TransportError> 
    {
        if config.transport_type != "custom" {
            return Err(TransportError::ConfigError(
                format!("Invalid transport type: {}", config.transport_type)
            ));
        }
        
        Ok(Box::new(MyCustomTransport::new(config)))
    }
    
    fn supported_types(&self) -> Vec<String> {
        vec!["custom".to_string()]
    }
}
```

### Step 3: Register with Registry

```rust
use std::sync::Arc;
use mcp_boilerplate_rust::transport::TransportRegistry;

let registry = TransportRegistry::global();
let factory = Arc::new(MyCustomTransportFactory);
registry.register("custom", factory)?;
```

---

## Transport Messages

### Creating Messages

```rust
use mcp_boilerplate_rust::transport::TransportMessage;

// Simple message
let msg = TransportMessage::new(json_content);

// Message with metadata
let msg = TransportMessage::with_metadata(json_content, "stdio");

// Message with correlation ID
let msg = TransportMessage::new(json_content)
    .with_correlation_id("req-123");
```

### Message Structure

```rust
pub struct TransportMessage {
    // JSON-RPC message content
    pub content: String,
    
    // Optional metadata
    pub metadata: Option<TransportMetadata>,
}

pub struct TransportMetadata {
    pub id: Option<String>,
    pub timestamp: String,
    pub transport_type: String,
    pub correlation_id: Option<String>,
}
```

---

## Error Handling

### Transport Errors

```rust
pub enum TransportError {
    SendError(String),
    ReceiveError(String),
    NotInitialized,
    AlreadyShutdown,
    ConnectionError(String),
    ConfigError(String),
    Timeout(u64),
    MessageTooLarge { actual: usize, max: usize },
    SerializationError(String),
    IoError(std::io::Error),
    Other(String),
}
```

### Handling Errors

```rust
match transport.send(message).await {
    Ok(()) => println!("Message sent"),
    Err(TransportError::NotInitialized) => {
        eprintln!("Transport not initialized");
    }
    Err(TransportError::MessageTooLarge { actual, max }) => {
        eprintln!("Message too large: {} > {}", actual, max);
    }
    Err(e) => eprintln!("Transport error: {}", e),
}
```

---

## Statistics Tracking

### Getting Statistics

```rust
use mcp_boilerplate_rust::transport::StdioTransport;

let transport = StdioTransport::new();
let stats = transport.stats();

println!("Messages sent: {}", stats.messages_sent);
println!("Messages received: {}", stats.messages_received);
println!("Bytes sent: {}", stats.bytes_sent);
println!("Bytes received: {}", stats.bytes_received);
println!("Errors: {}", stats.error_count);
println!("Uptime: {}s", stats.uptime_seconds);

// Or use Display trait
println!("{}", stats);
// Output: "Sent: 100 msgs (10240 bytes), Received: 95 msgs (9500 bytes), ..."
```

---

## Transport Capabilities

### Checking Capabilities

```rust
let transport = StdioTransport::new();
let caps = transport.capabilities();

if caps.bidirectional {
    println!("Transport supports bidirectional communication");
}

if caps.server_push {
    println!("Transport supports server push");
}

if caps.multi_connection {
    println!("Transport supports multiple connections");
}

if caps.streaming {
    println!("Transport supports streaming");
}

if caps.browser_compatible {
    println!("Transport is browser compatible");
}
```

---

## Configuration

### TransportConfig Structure

```rust
pub struct TransportConfig {
    // Transport type name
    pub transport_type: String,
    
    // Bind address (for network transports)
    pub bind_address: Option<String>,
    
    // Port number (for network transports)
    pub port: Option<u16>,
    
    // Maximum message size in bytes
    pub max_message_size: usize,
    
    // Connection timeout in seconds
    pub timeout_seconds: u64,
    
    // Enable TLS/SSL
    pub enable_tls: bool,
}
```

### Default Configuration

```rust
let config = TransportConfig::default();
// transport_type: "stdio"
// bind_address: Some("127.0.0.1")
// port: Some(8025)
// max_message_size: 10MB
// timeout_seconds: 30
// enable_tls: false
```

---

## Best Practices

### 1. Always Initialize

```rust
let mut transport = registry.create(config)?;

// WRONG: Using without initialization
// transport.send(message).await?; // Will error!

// RIGHT: Initialize first
transport.initialize().await?;
transport.send(message).await?;
```

### 2. Graceful Shutdown

```rust
// Always shutdown gracefully
if let Err(e) = transport.shutdown().await {
    eprintln!("Shutdown error: {}", e);
}
```

### 3. Check Ready State

```rust
if transport.is_ready() {
    transport.send(message).await?;
} else {
    eprintln!("Transport not ready");
}
```

### 4. Handle Message Size

```rust
let max_size = config.max_message_size;
if message.content.len() > max_size {
    // Split message or return error
    return Err(TransportError::MessageTooLarge {
        actual: message.content.len(),
        max: max_size,
    });
}
```

### 5. Use Correlation IDs

```rust
let request_id = uuid::Uuid::new_v4().to_string();
let message = TransportMessage::new(content)
    .with_correlation_id(request_id.clone());

transport.send(message).await?;

// Match response by correlation_id
let response = transport.receive().await?;
if let Some(metadata) = response.metadata {
    if metadata.correlation_id == Some(request_id) {
        // Process response
    }
}
```

---

## Testing

### Mock Transport for Testing

```rust
use async_trait::async_trait;
use mcp_boilerplate_rust::transport::{Transport, TransportMessage};

struct MockTransport {
    messages: Vec<TransportMessage>,
}

#[async_trait]
impl Transport for MockTransport {
    // Implement required methods for testing
    async fn send(&self, message: TransportMessage) 
        -> Result<(), TransportError> 
    {
        // Store message for verification
        Ok(())
    }
    
    // ... other methods
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_transport_roundtrip() {
    let mut transport = StdioTransport::new();
    transport.initialize().await.unwrap();
    
    let sent = TransportMessage::new("test".to_string());
    transport.send(sent).await.unwrap();
    
    let received = transport.receive().await.unwrap();
    assert_eq!(received.content, "test");
    
    transport.shutdown().await.unwrap();
}
```

---

## Troubleshooting

### Transport Not Initialized

**Error:** `TransportError::NotInitialized`

**Solution:**
```rust
transport.initialize().await?;
```

### Message Too Large

**Error:** `TransportError::MessageTooLarge`

**Solution:**
```rust
// Increase max size in config
let config = TransportConfig {
    max_message_size: 100 * 1024 * 1024, // 100MB
    ..Default::default()
};
```

### Connection Timeout

**Error:** `TransportError::Timeout`

**Solution:**
```rust
// Increase timeout in config
let config = TransportConfig {
    timeout_seconds: 60,
    ..Default::default()
};
```

---

## Performance Considerations

### Memory Usage

- stdio: <5MB per connection
- SSE: ~1-2MB per connection
- WebSocket: ~1-2MB per connection
- HTTP Streaming: ~2-5MB per connection

### Latency

- stdio: 2-7ms
- SSE: 10-20ms
- WebSocket: 5-15ms
- HTTP Streaming: 8-25ms

### Throughput

- stdio: ~100 msg/s
- SSE: ~1000 msg/s
- WebSocket: ~5000 msg/s
- HTTP Streaming: ~2000 msg/s

---

## Examples

### Basic stdio Example

```rust
use mcp_boilerplate_rust::transport::{StdioTransport, Transport};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut transport = StdioTransport::new();
    transport.initialize().await?;
    
    loop {
        let request = transport.receive().await?;
        let response = process_request(request);
        transport.send(response).await?;
    }
}
```

### Multi-Transport Server (Future)

```rust
// Will be available after Phase 2-5
use mcp_boilerplate_rust::transport::{TransportRegistry, TransportConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    mcp_boilerplate_rust::transport::init_registry();
    
    let registry = TransportRegistry::global();
    
    // Start multiple transports
    let stdio_config = TransportConfig {
        transport_type: "stdio".to_string(),
        ..Default::default()
    };
    
    let sse_config = TransportConfig {
        transport_type: "sse".to_string(),
        port: Some(8025),
        ..Default::default()
    };
    
    let mut stdio = registry.create(stdio_config)?;
    let mut sse = registry.create(sse_config)?;
    
    tokio::try_join!(
        stdio.initialize(),
        sse.initialize(),
    )?;
    
    // Handle both transports concurrently
    tokio::select! {
        result = handle_stdio(&mut stdio) => result,
        result = handle_sse(&mut sse) => result,
    }
}
```

---

## References

- **TRANSPORT_PHASE1_PROGRESS.md** - Phase 1 implementation details
- **NEXT_SESSION_TRANSPORT.md** - Complete roadmap
- **docs/PROJECT_STRUCTURE.md** - Project organization
- **src/transport/trait.rs** - Transport trait source code

---

**Last Updated:** 2026-01-08  
**Status:** Phase 1 Complete - Ready for Phase 2 (SSE)  
**Next:** Implement SSE transport with browser client example