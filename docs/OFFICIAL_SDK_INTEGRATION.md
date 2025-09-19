# Official MCP SDK Integration Guide

**Version**: 1.0  
**Last Updated**: January 18, 2025  
**SDK Version**: RMCP v0.6.3  
**Project**: mcp-boilerplate-rust

This guide provides comprehensive instructions for integrating the official RMCP (Rust MCP) SDK into your MCP server projects, based on our successful strategic pivot from custom implementation to the official foundation.

---

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Quick Start](#quick-start)
4. [SDK Architecture](#sdk-architecture)
5. [Creating Your First Server](#creating-your-first-server)
6. [Advanced Integration Patterns](#advanced-integration-patterns)
7. [Migration from Custom Implementation](#migration-from-custom-implementation)
8. [Testing and Validation](#testing-and-validation)
9. [Performance Optimization](#performance-optimization)
10. [Troubleshooting](#troubleshooting)
11. [Best Practices](#best-practices)

---

## Overview

The official RMCP SDK provides a robust, production-ready foundation for building MCP (Model Context Protocol) servers in Rust. This project demonstrates the strategic value of leveraging the official SDK rather than building custom implementations.

### Why Use the Official SDK?

- **Battle-Tested**: 2.2k+ stars on GitHub, proven in production
- **Standards Compliant**: Full MCP protocol compliance out of the box
- **Performance Optimized**: Efficient async implementation with minimal overhead
- **Extensible**: Rich macro system for rapid server development
- **Maintained**: Active development and community support

### Project Achievement

This project successfully pivoted from custom MCP implementation to official SDK, delivering:
- 6 production-ready MCP servers
- 4 reusable server templates
- Complete deployment infrastructure
- Comprehensive testing and documentation

---

## Prerequisites

### System Requirements

```bash
# Rust toolchain (1.70+)
rustc --version
# rustc 1.70.0 or higher

# Cargo package manager
cargo --version
# cargo 1.70.0 or higher
```

### Development Tools

```bash
# Install essential development tools
cargo install just cargo-watch cargo-audit

# Optional but recommended
cargo install cargo-edit cargo-expand
```

### Verification

```bash
# Clone and verify the project
git clone https://github.com/netadx1ai/mcp-boilerplate-rust.git
cd mcp-boilerplate-rust

# Verify workspace compilation
cargo check --workspace --all-targets
# Should complete in < 30 seconds

# Run test suite
cargo test --workspace
# Should pass 70+ tests in < 5 seconds
```

---

## Quick Start

### 1. Add SDK Dependencies

Add the official RMCP SDK to your `Cargo.toml`:

```toml
[dependencies]
rmcp = { version = "0.6.3", features = ["server", "client"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
```

### 2. Basic Server Structure

```rust
use rmcp::ServiceExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default)]
pub struct MyMcpServer {
    // Server state here
}

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("Tool execution failed: {0}")]
    ToolError(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

#[rmcp::server]
impl MyMcpServer {
    #[tool]
    async fn hello_world(&self, name: String) -> Result<String, ServerError> {
        Ok(format!("Hello, {}! Welcome to MCP.", name))
    }
    
    #[tool]
    async fn get_status(&self) -> Result<HashMap<String, String>, ServerError> {
        let mut status = HashMap::new();
        status.insert("status".to_string(), "running".to_string());
        status.insert("version".to_string(), "1.0.0".to_string());
        Ok(status)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::init();
    
    let server = MyMcpServer::default();
    server.serve().await?;
    
    Ok(())
}
```

### 3. Test Your Server

```bash
# Build and run
cargo build --release
cargo run

# Test with MCP client
# Server listens on stdio by default
echo '{"jsonrpc": "2.0", "method": "tools/list", "id": 1}' | ./target/release/my-mcp-server
```

---

## SDK Architecture

### Core Components

```rust
// Server trait - implement your business logic
#[rmcp::server]
impl MyServer {
    #[tool]
    async fn my_tool(&self) -> Result<String, Error> { /* ... */ }
}

// Transport layer - handles communication
// Built-in: stdio, HTTP, WebSocket
use rmcp::transport::{stdio, http, websocket};

// Protocol layer - MCP message handling
// Automatic serialization/deserialization
use rmcp::protocol::{Request, Response, Tool, Resource};
```

### Request Lifecycle

1. **Transport** receives raw message
2. **Protocol** deserializes to MCP message
3. **Server** routes to appropriate tool method
4. **Tool** executes business logic
5. **Server** serializes response
6. **Transport** sends response back

### Async Architecture

The SDK is built on Tokio async runtime:

```rust
// All tool methods are async
#[tool]
async fn process_data(&self, data: Vec<u8>) -> Result<String, Error> {
    // Async operations supported
    let result = tokio::fs::read_to_string("config.json").await?;
    let processed = self.heavy_computation(data).await?;
    Ok(processed)
}
```

---

## Creating Your First Server

### Step 1: Initialize Project

```bash
# Create new MCP server project
cargo new my-mcp-server --bin
cd my-mcp-server

# Or use our template
cp -r templates/basic-server-template my-custom-server
cd my-custom-server
```

### Step 2: Define Your Domain

```rust
// Define your business domain
pub struct WeatherServer {
    api_key: String,
    cache: Arc<RwLock<HashMap<String, WeatherData>>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WeatherData {
    temperature: f64,
    humidity: f64,
    conditions: String,
    timestamp: i64,
}

#[derive(thiserror::Error, Debug)]
pub enum WeatherError {
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Invalid location: {0}")]
    InvalidLocation(String),
    #[error("Cache error: {0}")]
    CacheError(String),
}
```

### Step 3: Implement MCP Tools

```rust
#[rmcp::server]
impl WeatherServer {
    #[tool]
    async fn get_current_weather(
        &self, 
        location: String
    ) -> Result<WeatherData, WeatherError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&location) {
                if self.is_cache_valid(cached) {
                    return Ok(cached.clone());
                }
            }
        }
        
        // Fetch from API
        let weather_data = self.fetch_weather_api(&location).await?;
        
        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(location, weather_data.clone());
        }
        
        Ok(weather_data)
    }
    
    #[tool]
    async fn get_forecast(
        &self,
        location: String,
        days: Option<u8>
    ) -> Result<Vec<WeatherData>, WeatherError> {
        let days = days.unwrap_or(7).min(14); // Max 14 days
        self.fetch_forecast_api(&location, days).await
    }
    
    #[tool]
    async fn list_supported_locations(&self) -> Result<Vec<String>, WeatherError> {
        Ok(vec![
            "New York".to_string(),
            "London".to_string(),
            "Tokyo".to_string(),
            // ... more locations
        ])
    }
}
```

### Step 4: Add Business Logic

```rust
impl WeatherServer {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    async fn fetch_weather_api(&self, location: &str) -> Result<WeatherData, WeatherError> {
        // Implementation using reqwest or similar
        let client = reqwest::Client::new();
        let response = client
            .get(&format!("https://api.weather.com/v1/current?location={}&key={}", 
                location, self.api_key))
            .send()
            .await
            .map_err(|e| WeatherError::ApiError(e.to_string()))?;
            
        let weather_data: WeatherData = response
            .json()
            .await
            .map_err(|e| WeatherError::ApiError(e.to_string()))?;
            
        Ok(weather_data)
    }
    
    fn is_cache_valid(&self, data: &WeatherData) -> bool {
        let now = chrono::Utc::now().timestamp();
        now - data.timestamp < 300 // 5 minutes cache
    }
}
```

### Step 5: Main Function and Configuration

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();
    
    // Load configuration
    let api_key = std::env::var("WEATHER_API_KEY")
        .map_err(|_| anyhow::anyhow!("WEATHER_API_KEY not set"))?;
    
    // Create and start server
    let server = WeatherServer::new(api_key);
    
    tracing::info!("Starting Weather MCP Server");
    server.serve().await?;
    
    Ok(())
}
```

---

## Advanced Integration Patterns

### State Management

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct StatefulServer {
    state: Arc<RwLock<ServerState>>,
    config: ServerConfig,
}

#[derive(Default)]
struct ServerState {
    request_count: u64,
    active_sessions: HashMap<String, Session>,
    last_health_check: Option<Instant>,
}

#[rmcp::server]
impl StatefulServer {
    #[tool]
    async fn get_stats(&self) -> Result<Stats, ServerError> {
        let state = self.state.read().await;
        Ok(Stats {
            request_count: state.request_count,
            active_sessions: state.active_sessions.len(),
            uptime: state.last_health_check
                .map(|t| t.elapsed().as_secs())
                .unwrap_or(0),
        })
    }
    
    // Increment request count for all tools
    async fn increment_counter(&self) {
        let mut state = self.state.write().await;
        state.request_count += 1;
    }
}
```

### Error Handling Patterns

```rust
// Custom error hierarchy
#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("Business logic error: {0}")]
    BusinessError(#[from] BusinessError),
    #[error("External API error: {0}")]
    ExternalApiError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

// Error conversion and context
impl From<reqwest::Error> for ServerError {
    fn from(err: reqwest::Error) -> Self {
        ServerError::ExternalApiError(err.to_string())
    }
}

// Structured error responses
#[tool]
async fn handle_with_context(&self, input: String) -> Result<String, ServerError> {
    self.validate_input(&input)
        .map_err(|e| ServerError::BusinessError(e))?;
        
    let result = self.process_external_api(&input).await
        .with_context(|| format!("Failed to process input: {}", input))?;
        
    Ok(result)
}
```

### Resource Management

```rust
#[rmcp::server]
impl ResourceServer {
    #[resource]
    async fn get_file_content(&self, uri: String) -> Result<Vec<u8>, ServerError> {
        // Validate URI format
        let path = self.parse_uri(&uri)?;
        
        // Security validation
        self.validate_file_access(&path)?;
        
        // Read with async I/O
        let content = tokio::fs::read(&path).await
            .map_err(|e| ServerError::FileError(e.to_string()))?;
            
        Ok(content)
    }
    
    #[resource]
    async fn list_files(&self, directory: String) -> Result<Vec<FileInfo>, ServerError> {
        let mut files = Vec::new();
        let mut dir = tokio::fs::read_dir(&directory).await?;
        
        while let Some(entry) = dir.next_entry().await? {
            let metadata = entry.metadata().await?;
            files.push(FileInfo {
                name: entry.file_name().to_string_lossy().to_string(),
                size: metadata.len(),
                modified: metadata.modified()?.into(),
            });
        }
        
        Ok(files)
    }
}
```

### Middleware and Hooks

```rust
// Request middleware
#[rmcp::server]
impl MiddlewareServer {
    // Before all tool executions
    async fn before_request(&self, method: &str) {
        tracing::info!("Executing tool: {}", method);
        self.metrics.increment_counter(method).await;
    }
    
    // After all tool executions
    async fn after_request(&self, method: &str, duration: Duration) {
        tracing::info!("Completed tool: {} in {:?}", method, duration);
        self.metrics.record_duration(method, duration).await;
    }
    
    // Error handling middleware
    async fn on_error(&self, method: &str, error: &dyn std::error::Error) {
        tracing::error!("Tool {} failed: {}", method, error);
        self.metrics.increment_error_counter(method).await;
    }
}
```

---

## Migration from Custom Implementation

### Assessment Checklist

Before migrating, assess your current implementation:

- [ ] **Protocol Compliance**: Are you implementing full MCP specification?
- [ ] **Transport Support**: Do you support multiple transport layers?
- [ ] **Error Handling**: Is error handling comprehensive and standardized?
- [ ] **Testing**: Do you have comprehensive test coverage?
- [ ] **Performance**: Are you meeting performance requirements?
- [ ] **Maintenance**: Is the codebase maintainable and documented?

### Migration Strategy

#### 1. Backup and Archive

```bash
# Archive your custom implementation
mkdir archive/custom-implementation-$(date +%Y%m%d)
cp -r src/ tests/ Cargo.toml archive/custom-implementation-$(date +%Y%m%d)/

# Document custom features
echo "# Custom Implementation Features" > archive/CUSTOM_FEATURES.md
echo "- Feature 1: ..." >> archive/CUSTOM_FEATURES.md
echo "- Feature 2: ..." >> archive/CUSTOM_FEATURES.md
```

#### 2. Dependency Migration

```toml
# Replace custom dependencies
[dependencies]
# Remove these
# mcp-core = { path = "../mcp-core" }
# mcp-transport = { path = "../mcp-transport" }

# Add official SDK
rmcp = { version = "0.6.3", features = ["server", "client"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
thiserror = "1.0"
```

#### 3. Code Migration Patterns

```rust
// OLD: Custom implementation pattern
pub struct CustomMcpServer {
    transport: Box<dyn Transport>,
    protocol: McpProtocol,
}

impl CustomMcpServer {
    pub async fn handle_request(&self, req: Request) -> Response {
        match req.method.as_str() {
            "tools/list" => self.list_tools().await,
            "tools/call" => self.call_tool(req.params).await,
            _ => Response::error("Unknown method"),
        }
    }
}

// NEW: Official SDK pattern
#[rmcp::server]
impl MigratedServer {
    #[tool]
    async fn my_tool(&self, params: MyParams) -> Result<MyResult, MyError> {
        // Business logic moved here
        self.original_business_logic(params).await
    }
}
```

#### 4. Testing Migration

```rust
// Update tests to use official SDK patterns
#[tokio::test]
async fn test_migrated_functionality() {
    let server = MigratedServer::new();
    
    // Test tool execution directly
    let result = server.my_tool(MyParams::default()).await;
    assert!(result.is_ok());
    
    // Or test via MCP protocol
    let client = rmcp::Client::new();
    let response = client.call_tool("my_tool", MyParams::default()).await;
    assert!(response.is_ok());
}
```

### Common Migration Issues

#### Issue 1: Custom Transport Layers

**Problem**: You have custom transport implementations
**Solution**: Use official SDK transports or implement Transport trait

```rust
// Use built-in transports
use rmcp::transport::{stdio, http, websocket};

// Or implement custom transport
#[async_trait]
impl Transport for MyCustomTransport {
    async fn send(&self, message: Message) -> Result<(), Error> {
        // Your custom transport logic
    }
    
    async fn receive(&self) -> Result<Message, Error> {
        // Your custom receive logic
    }
}
```

#### Issue 2: Custom Protocol Extensions

**Problem**: You have protocol extensions not in MCP spec
**Solution**: Use tool parameters or implement as resources

```rust
// Instead of custom protocol method
#[tool]
async fn custom_feature(&self, params: CustomParams) -> Result<CustomResult, Error> {
    // Implement as standard MCP tool
}

// Or as resource if it's data-oriented
#[resource]
async fn custom_data(&self, uri: String) -> Result<Vec<u8>, Error> {
    // Implement as MCP resource
}
```

#### Issue 3: State Management Differences

**Problem**: Different state management patterns
**Solution**: Adapt to SDK patterns using Arc<RwLock<State>>

```rust
// SDK-compatible state management
pub struct MigratedServer {
    state: Arc<RwLock<ServerState>>,
    // Migrate your state here
}

#[rmcp::server]
impl MigratedServer {
    #[tool]
    async fn stateful_operation(&self) -> Result<String, Error> {
        let state = self.state.read().await;
        // Use state safely
        Ok(format!("State: {:?}", state))
    }
}
```

---

## Testing and Validation

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tool_execution() {
        let server = MyServer::new();
        
        let result = server.my_tool("test_input".to_string()).await;
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert_eq!(output, "expected_output");
    }
    
    #[tokio::test]
    async fn test_error_handling() {
        let server = MyServer::new();
        
        let result = server.my_tool("invalid_input".to_string()).await;
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(matches!(error, MyError::InvalidInput(_)));
    }
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_mcp_protocol_compliance() {
    let server = MyServer::new();
    
    // Test tools/list
    let tools = server.list_tools().await.unwrap();
    assert!(!tools.is_empty());
    
    // Test tool execution via protocol
    let request = Request {
        method: "tools/call".to_string(),
        params: json!({
            "name": "my_tool",
            "arguments": {"input": "test"}
        }),
    };
    
    let response = server.handle_request(request).await;
    assert!(response.is_ok());
}
```

### Performance Testing

```rust
#[tokio::test]
async fn test_performance_requirements() {
    let server = MyServer::new();
    
    let start = Instant::now();
    let result = server.my_tool("performance_test".to_string()).await;
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    assert!(duration < Duration::from_millis(50)); // < 50ms requirement
}

#[tokio::test]
async fn test_concurrent_requests() {
    let server = Arc::new(MyServer::new());
    let mut handles = Vec::new();
    
    for i in 0..100 {
        let server_clone = server.clone();
        let handle = tokio::spawn(async move {
            server_clone.my_tool(format!("request_{}", i)).await
        });
        handles.push(handle);
    }
    
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}
```

### Compliance Testing

```rust
#[tokio::test]
async fn test_mcp_specification_compliance() {
    let server = MyServer::new();
    
    // Test required methods
    assert!(server.supports_method("tools/list"));
    assert!(server.supports_method("tools/call"));
    
    // Test error responses format
    let invalid_request = Request {
        method: "invalid/method".to_string(),
        params: json!({}),
    };
    
    let response = server.handle_request(invalid_request).await;
    assert!(response.is_err());
    
    let error = response.unwrap_err();
    assert!(error.code() == -32601); // Method not found
}
```

---

## Performance Optimization

### Async Best Practices

```rust
// Use efficient async patterns
#[tool]
async fn optimized_tool(&self, batch: Vec<String>) -> Result<Vec<String>, Error> {
    // Process in parallel
    let futures: Vec<_> = batch.into_iter()
        .map(|item| self.process_item(item))
        .collect();
    
    // Wait for all to complete
    let results = futures::future::try_join_all(futures).await?;
    Ok(results)
}

// Avoid blocking operations
#[tool]
async fn avoid_blocking(&self, file_path: String) -> Result<String, Error> {
    // ❌ Don't do this - blocks async runtime
    // let content = std::fs::read_to_string(&file_path)?;
    
    // ✅ Use async I/O
    let content = tokio::fs::read_to_string(&file_path).await?;
    Ok(content)
}
```

### Memory Management

```rust
// Use efficient data structures
use std::collections::HashMap;
use lru::LruCache;

pub struct OptimizedServer {
    // LRU cache for bounded memory usage
    cache: Arc<RwLock<LruCache<String, CachedData>>>,
    
    // Use Arc for shared data
    shared_config: Arc<Config>,
}

#[tool]
async fn cached_operation(&self, key: String) -> Result<String, Error> {
    // Check cache first
    {
        let mut cache = self.cache.write().await;
        if let Some(cached) = cache.get(&key) {
            return Ok(cached.data.clone());
        }
    }
    
    // Compute and cache
    let result = self.expensive_computation(&key).await?;
    
    {
        let mut cache = self.cache.write().await;
        cache.put(key, CachedData { data: result.clone() });
    }
    
    Ok(result)
}
```

### Connection Pooling

```rust
use sqlx::PgPool;

pub struct DatabaseServer {
    pool: PgPool,
}

impl DatabaseServer {
    pub async fn new(database_url: &str) -> Result<Self, Error> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }
}

#[rmcp::server]
impl DatabaseServer {
    #[tool]
    async fn query_database(&self, sql: String) -> Result<Vec<Row>, Error> {
        // Use connection pool efficiently
        let rows = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await?;
        
        Ok(rows)
    }
}
```

### Benchmarking

```rust
// Add to Cargo.toml
[dev-dependencies]
criterion = "0.5"

// benches/performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_tool_execution(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let server = MyServer::new();
    
    c.bench_function("tool_execution", |b| {
        b.iter(|| {
            rt.block_on(async {
                server.my_tool(black_box("benchmark_input".to_string())).await
            })
        })
    });
}

criterion_group!(benches, benchmark_tool_execution);
criterion_main!(benches);
```

---

## Troubleshooting

### Common Issues

#### Issue 1: Server Won't Start

**Symptoms**: Server exits immediately or panics on startup
**Debug Steps**:

```bash
# Check dependencies
cargo check --workspace

# Enable debug logging
RUST_LOG=debug cargo run

# Check port availability (if using HTTP transport)
netstat -tlnp | grep :8080
```

**Solutions**:
- Verify all required dependencies are installed
- Check environment variables are set
- Ensure ports are available
- Validate configuration files

#### Issue 2: Tool Not Found

**Symptoms**: MCP client receives "Method not found" errors
**Debug Steps**:

```rust
// Add debugging to server
#[rmcp::server]
impl DebugServer {
    fn list_available_tools(&self) -> Vec<String> {
        // List all registered tools
        self.get_tool_registry().keys().collect()
    }
    
    #[tool]
    async fn debug_info(&self) -> Result<HashMap<String, Value>, Error> {
        let mut info = HashMap::new();
        info.insert("available_tools".to_string(), 
                   json!(self.list_available_tools()));
        Ok(info)
    }
}
```

**Solutions**:
- Verify tool is marked with `#[tool]` attribute
- Check tool name matches client requests
- Ensure server is properly compiled
- Validate parameter types match

#### Issue 3: Performance Issues

**Symptoms**: Slow response times, high CPU/memory usage
**Debug Steps**:

```rust
// Add performance monitoring
use std::time::Instant;

#[tool]
async fn monitored_tool(&self, input: String) -> Result<String, Error> {
    let start = Instant::now();
    
    let result = self.actual_work(input).await?;
    
    let duration = start.elapsed();
    if duration > Duration::from_millis(100) {
        tracing::warn!("Slow tool execution: {:?}", duration);
    }
    
    Ok(result)
}
```

**Solutions**:
- Profile with `cargo flamegraph`
- Check for blocking operations
- Optimize database queries
- Implement caching
- Use connection pooling

#### Issue 4: Memory Leaks

**Symptoms**: Increasing memory usage over time
**Debug Steps**:

```bash
# Monitor memory usage
ps aux | grep my-mcp-server

# Use valgrind (Linux)
valgrind --tool=massif cargo run

# Use instruments (macOS)
instruments -t "Allocations" cargo run
```

**Solutions**:
- Check for unclosed resources
- Implement bounded caches
- Use `weak` references where appropriate
- Profile with memory tools

### Debugging Tools

#### Logging Configuration

```rust
// Configure structured logging
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn setup_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "my_server=debug,rmcp=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

// Use in tools
#[tool]
async fn logged_tool(&self, input: String) -> Result<String, Error> {
    tracing::info!("Processing input: {}", input);
    
    let result = self.process(input).await?;
    
    tracing::debug!("Generated result: {}", result);
    Ok(result)
}
```

#### Testing Framework

```rust
// Test helpers
pub mod test_utils {
    use super::*;
    
    pub fn create_test_server() -> MyServer {
        MyServer::new_with_config(TestConfig::default())
    }
    
    pub async fn assert_tool_success<T, E>(
        result: Result<T, E>
    ) where 
        E: std::fmt::Debug 
    {
        match result {
            Ok(_) => (),
            Err(e) => panic!("Tool execution failed: {:?}", e),
        }
    }
}
```

---

## Best Practices

### Security

```rust
// Input validation
#[tool]
async fn secure_tool(&self, user_input: String) -> Result<String, Error> {
    // Validate input length
    if user_input.len() > 1000 {
        return Err(Error::InvalidInput("Input too long".to_string()));
    }
    
    // Sanitize input
    let sanitized = user_input
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>();
    
    // Validate business rules
    if !self.is_valid_business_input(&sanitized) {
        return Err(Error::InvalidInput("Invalid format".to_string()));
    }
    
    self.process_safely(sanitized).await
}

// Rate limiting
use governor::{Quota, RateLimiter};

pub struct RateLimitedServer {
    rate_limiter: RateLimiter<String, DefaultHasher, DefaultClock>,
}

#[tool]
async fn rate_limited_tool(&self, client_id: String) -> Result<String, Error> {
    // Check rate limit
    if self.rate_limiter.check_key(&client_id).is_err() {
        return Err(Error::RateLimitExceeded);
    }
    
    self.process_request().await
}
```

### Error Handling

```rust
// Comprehensive error types
#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("Validation failed: {field}")]
    ValidationError { field: String },
    
    #[error("External service error: {service} - {message}")]
    ExternalServiceError { service: String, message: String },
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Resource not found: {resource_type} with id {id}")]
    NotFound { resource_type: String, id: String },
    
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

// Error context
#[tool]
async fn well_handled_tool(&self, id: String) -> Result<Data, ServerError> {
    let data = self.fetch_data(&id).await
        .map_err(|e| ServerError::ExternalServiceError {
            service: "data_api".to_string(),
            message: e.to_string(),
        })?;
    
    self.validate_data(&data)
        .map_err(|field| ServerError::ValidationError { field })?;
    
    Ok(data)
}
```

### Code Organization

```rust
// Modular structure
mod tools {
    pub mod user_management;
    pub mod data_processing;
    pub mod reporting;
}

mod services {
    pub mod database;
    pub mod external_api;
    pub mod cache;
}

mod config {
    pub mod settings;
    pub mod validation;
}

// Clean interfaces
#[async_trait]
pub trait DataService {
    async fn fetch_data(&self, id: &str) -> Result<Data, DataError>;
    async fn store_data(&self, data: Data) -> Result<String, DataError>;
}

pub struct DatabaseDataService {
    pool: PgPool,
}

#[async_trait]
impl DataService for DatabaseDataService {
    async fn fetch_data(&self, id: &str) -> Result<Data, DataError> {
        // Implementation
    }
}

// Dependency injection
pub struct ServerBuilder {
    data_service: Option<Box<dyn DataService>>,
    config: Option<Config>,
}

impl ServerBuilder {
    pub fn with_data_service(mut self, service: Box<dyn DataService>) -> Self {
        self.data_service = Some(service);
        self
    }
    
    pub fn build(self) -> Result<MyServer, BuildError> {
        Ok(MyServer {
            data_service: self.data_service.ok_or(BuildError::MissingDataService)?,
            config: self.config.ok_or(BuildError::MissingConfig)?,
        })
    }
}
```

### Testing Strategy

```rust
// Comprehensive test structure
#[cfg(test)]
mod tests {
    use super::*;
    
    mod unit_tests {
        // Test individual functions
        #[test]
        fn test_validation_logic() {
            assert!(validate_input("valid").is_ok());
            assert!(validate_input("").is_err());
        }
    }
    
    mod integration_tests {
        // Test tool execution
        #[tokio::test]
        async fn test_full_tool_flow() {
            let server = create_test_server().await;
            let result = server.my_tool("test_input".to_string()).await;
            assert!(result.is_ok());
        }
    }
    
    mod performance_tests {
        #[tokio::test]
        async fn test_response_time() {
            let server = create_test_server().await;
            let start = Instant::now();
            let _ = server.my_tool("benchmark".to_string()).await;
            assert!(start.elapsed() < Duration::from_millis(50));
        }
    }
    
    mod property_tests {
        use proptest::prelude::*;
        
        proptest! {
            #[test]
            fn test_input_handling(input in "\\PC*") {
                let server = create_test_server();
                // Should not panic with any input
                let _ = server.validate_input(&input);
            }
        }
    }
}
```

---

## Conclusion

The official RMCP SDK provides a robust foundation for building production-ready MCP servers. This guide has shown you how to:

1. **Integrate the SDK** into your Rust projects
2. **Create performant servers** with proper async patterns
3. **Migrate from custom implementations** safely
4. **Test and validate** your implementations
5. **Optimize for production** use

### Next Steps

1. **Explore the Templates**: Check out the 4 server templates in this project
2. **Review Example Servers**: Study the 6 production servers for patterns
3. **Deploy to Production**: Use the provided Docker and Kubernetes configs
4. **Join the Community**: Contribute back to the ecosystem

### Resources

- **Project Repository**: https://github.com/netadx1ai/mcp-boilerplate-rust
- **Official RMCP SDK**: https://github.com/zed-industries/rmcp
- **MCP Specification**: https://modelcontextprotocol.io/
- **Community Discord**: [Join the discussion]

---

**Happy building! 🚀**

The MCP ecosystem is growing rapidly, and your contributions make it stronger. Whether you're building internal tools, public services, or contributing to the core SDK, you're part of shaping the future of AI-human collaboration.