# MCP Server Development Guide

**Version**: 1.0  
**Last Updated**: January 18, 2025  
**SDK Version**: RMCP v0.6.3  
**Project**: mcp-boilerplate-rust

A comprehensive guide for building production-ready MCP servers using the official RMCP SDK and proven patterns from our 6-server ecosystem.

---

## Table of Contents

1. [Development Philosophy](#development-philosophy)
2. [Server Architecture](#server-architecture)
3. [Development Workflow](#development-workflow)
4. [Core Patterns](#core-patterns)
5. [Tool Implementation](#tool-implementation)
6. [State Management](#state-management)
7. [Error Handling](#error-handling)
8. [Testing Strategy](#testing-strategy)
9. [Performance Guidelines](#performance-guidelines)
10. [Security Best Practices](#security-best-practices)
11. [Deployment Preparation](#deployment-preparation)
12. [Real-World Examples](#real-world-examples)

---

## Development Philosophy

### Production-Ready from Day One

Our development approach prioritizes production readiness over rapid prototyping:

- **Security First**: Input validation, rate limiting, and secure defaults
- **Performance Focused**: < 50ms response times, efficient async patterns
- **Reliability Oriented**: Comprehensive error handling and graceful degradation
- **Maintainability**: Clean code, comprehensive tests, and documentation

### Proven Patterns

This guide is based on patterns proven across our 6 production servers:
- **news-data-server**: External API integration with caching
- **template-server**: Content generation with security validation
- **analytics-server**: Business intelligence with mock data patterns
- **database-server**: SQL operations with injection protection
- **api-gateway-server**: External service orchestration with resilience
- **workflow-server**: Multi-server coordination and task automation

---

## Server Architecture

### Standard Structure

Every MCP server follows this proven structure:

```
my-mcp-server/
├── src/
│   ├── main.rs          # Entry point and server configuration
│   ├── lib.rs           # Server implementation and tools
│   ├── errors.rs        # Error types and handling
│   ├── config.rs        # Configuration management
│   └── utils.rs         # Shared utilities
├── tests/
│   ├── integration.rs   # Integration tests
│   ├── performance.rs   # Performance benchmarks
│   └── security.rs      # Security validation
├── Cargo.toml           # Dependencies and metadata
├── README.md            # Usage and development guide
└── justfile             # Development commands
```

### Core Components

#### 1. Server Struct

```rust
use rmcp::ServiceExt;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MyMcpServer {
    // Business logic state
    state: Arc<RwLock<ServerState>>,
    
    // External service clients
    external_client: reqwest::Client,
    
    // Configuration
    config: ServerConfig,
    
    // Metrics and monitoring
    metrics: Arc<ServerMetrics>,
}

#[derive(Default)]
struct ServerState {
    request_count: u64,
    last_activity: Option<std::time::Instant>,
    cache: std::collections::HashMap<String, CachedData>,
}
```

#### 2. Configuration Management

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    // Server identification
    pub name: String,
    pub version: String,
    
    // Feature flags
    pub enable_caching: bool,
    pub enable_rate_limiting: bool,
    
    // External services
    pub api_endpoints: HashMap<String, String>,
    pub api_keys: HashMap<String, String>,
    
    // Performance tuning
    pub cache_ttl_seconds: u64,
    pub max_concurrent_requests: usize,
    pub request_timeout_ms: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: "My MCP Server".to_string(),
            version: "1.0.0".to_string(),
            enable_caching: true,
            enable_rate_limiting: true,
            api_endpoints: HashMap::new(),
            api_keys: HashMap::new(),
            cache_ttl_seconds: 300, // 5 minutes
            max_concurrent_requests: 100,
            request_timeout_ms: 5000, // 5 seconds
        }
    }
}
```

#### 3. Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Configuration error: {message}")]
    ConfigError { message: String },
    
    #[error("Validation error in field '{field}': {message}")]
    ValidationError { field: String, message: String },
    
    #[error("External service '{service}' error: {message}")]
    ExternalServiceError { service: String, message: String },
    
    #[error("Rate limit exceeded for operation '{operation}'")]
    RateLimitExceeded { operation: String },
    
    #[error("Resource not found: {resource_type} with identifier '{id}'")]
    NotFound { resource_type: String, id: String },
    
    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),
}

// Implement common conversions
impl From<reqwest::Error> for ServerError {
    fn from(err: reqwest::Error) -> Self {
        ServerError::ExternalServiceError {
            service: "HTTP Client".to_string(),
            message: err.to_string(),
        }
    }
}
```

---

## Development Workflow

### Step 1: Project Setup

```bash
# Use our template for rapid setup
cp -r templates/basic-server-template my-new-server
cd my-new-server

# Update project metadata
sed -i 's/basic-server-template/my-new-server/g' Cargo.toml
sed -i 's/Basic Server Template/My New Server/g' README.md

# Initialize development environment
just setup
```

### Step 2: Define Your Domain

Start by clearly defining your server's purpose and tools:

```rust
// Define your business domain
pub struct WeatherServer {
    state: Arc<RwLock<WeatherState>>,
    weather_client: WeatherApiClient,
    config: WeatherConfig,
}

struct WeatherState {
    request_count: u64,
    cache: HashMap<String, CachedWeatherData>,
    rate_limiter: HashMap<String, Instant>,
}

// Define your data models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub location: String,
    pub temperature: f64,
    pub humidity: f64,
    pub conditions: String,
    pub timestamp: i64,
    pub forecast: Option<Vec<DailyForecast>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyForecast {
    pub date: String,
    pub high_temp: f64,
    pub low_temp: f64,
    pub conditions: String,
    pub precipitation_chance: f64,
}
```

### Step 3: Implement Core Business Logic

Separate business logic from MCP concerns:

```rust
impl WeatherServer {
    pub fn new(config: WeatherConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(WeatherState::default())),
            weather_client: WeatherApiClient::new(&config.api_key),
            config,
        }
    }
    
    async fn fetch_weather_data(&self, location: &str) -> Result<WeatherData, WeatherError> {
        // Check cache first
        if let Some(cached) = self.get_cached_data(location).await? {
            return Ok(cached);
        }
        
        // Validate location
        self.validate_location(location)?;
        
        // Fetch from external API
        let weather_data = self.weather_client
            .get_current_weather(location)
            .await?;
        
        // Cache the result
        self.cache_weather_data(location, &weather_data).await?;
        
        // Update metrics
        self.increment_request_count().await;
        
        Ok(weather_data)
    }
    
    async fn validate_location(&self, location: &str) -> Result<(), WeatherError> {
        if location.is_empty() {
            return Err(WeatherError::InvalidLocation("Location cannot be empty".to_string()));
        }
        
        if location.len() > 100 {
            return Err(WeatherError::InvalidLocation("Location name too long".to_string()));
        }
        
        // Additional validation logic...
        Ok(())
    }
}
```

### Step 4: Implement MCP Tools

```rust
#[rmcp::server]
impl WeatherServer {
    #[tool]
    async fn get_current_weather(&self, location: String) -> Result<WeatherData, ServerError> {
        let weather_data = self.fetch_weather_data(&location).await
            .map_err(|e| ServerError::ExternalServiceError {
                service: "Weather API".to_string(),
                message: e.to_string(),
            })?;
        
        Ok(weather_data)
    }
    
    #[tool]
    async fn get_forecast(
        &self, 
        location: String, 
        days: Option<u8>
    ) -> Result<Vec<DailyForecast>, ServerError> {
        let days = days.unwrap_or(7).min(14); // Default 7, max 14 days
        
        let forecast_data = self.weather_client
            .get_forecast(&location, days)
            .await?;
        
        Ok(forecast_data)
    }
    
    #[tool]
    async fn get_server_status(&self) -> Result<HashMap<String, serde_json::Value>, ServerError> {
        let state = self.state.read().await;
        
        let mut status = HashMap::new();
        status.insert("status".to_string(), json!("running"));
        status.insert("version".to_string(), json!(self.config.version));
        status.insert("request_count".to_string(), json!(state.request_count));
        status.insert("cache_size".to_string(), json!(state.cache.len()));
        status.insert("uptime".to_string(), json!(self.get_uptime_seconds()));
        
        Ok(status)
    }
}
```

### Step 5: Add Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_server() -> WeatherServer {
        let config = WeatherConfig {
            api_key: "test_key".to_string(),
            ..Default::default()
        };
        WeatherServer::new(config)
    }
    
    #[tokio::test]
    async fn test_get_current_weather() {
        let server = create_test_server();
        
        let result = server.get_current_weather("New York".to_string()).await;
        assert!(result.is_ok());
        
        let weather = result.unwrap();
        assert_eq!(weather.location, "New York");
        assert!(weather.temperature > -100.0 && weather.temperature < 100.0);
    }
    
    #[tokio::test]
    async fn test_invalid_location() {
        let server = create_test_server();
        
        let result = server.get_current_weather("".to_string()).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ServerError::ValidationError { field, .. } => {
                assert_eq!(field, "location");
            }
            _ => panic!("Expected validation error"),
        }
    }
    
    #[tokio::test]
    async fn test_performance_requirement() {
        let server = create_test_server();
        
        let start = std::time::Instant::now();
        let result = server.get_current_weather("London".to_string()).await;
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        assert!(duration < std::time::Duration::from_millis(50));
    }
}
```

---

## Core Patterns

### Pattern 1: Scoped Lock Management

**Problem**: Async deadlocks from holding locks across await points
**Solution**: Use scoped blocks to control lock lifetime

```rust
// ❌ DEADLOCK RISK - Don't do this
#[tool]
async fn dangerous_pattern(&self) -> Result<String, Error> {
    let mut state = self.state.write().await; // Lock acquired
    let result = self.external_api_call().await?; // Might need same lock = DEADLOCK
    state.update(result);
    Ok("done".to_string())
}

// ✅ SAFE PATTERN - Use scoped locks
#[tool]
async fn safe_pattern(&self) -> Result<String, Error> {
    // Scoped read lock for validation
    {
        let state = self.state.read().await;
        if !state.is_ready() {
            return Err(Error::NotReady);
        }
    } // Lock released here
    
    // External call without holding locks
    let result = self.external_api_call().await?;
    
    // Scoped write lock for update
    {
        let mut state = self.state.write().await;
        state.update(result.clone());
    } // Lock released here
    
    Ok(result)
}
```

### Pattern 2: Error Context Propagation

```rust
use anyhow::Context;

#[tool]
async fn contextual_error_handling(&self, user_id: String) -> Result<UserData, ServerError> {
    // Add context at each step
    let user = self.fetch_user(&user_id).await
        .with_context(|| format!("Failed to fetch user with ID: {}", user_id))?;
    
    let preferences = self.fetch_preferences(&user_id).await
        .with_context(|| format!("Failed to fetch preferences for user: {}", user_id))?;
    
    let enriched_data = self.enrich_user_data(user, preferences).await
        .with_context(|| "Failed to enrich user data")?;
    
    Ok(enriched_data)
}
```

### Pattern 3: Resource Cleanup

```rust
#[tool]
async fn resource_management(&self, file_path: String) -> Result<String, ServerError> {
    // Use RAII patterns for cleanup
    let _span = tracing::info_span!("file_processing", path = %file_path).entered();
    
    // Acquire resource with automatic cleanup
    let temp_file = TempFile::new().await?;
    
    // Process with guaranteed cleanup
    let result = async {
        let content = tokio::fs::read_to_string(&file_path).await?;
        let processed = self.process_content(content).await?;
        tokio::fs::write(temp_file.path(), &processed).await?;
        Ok(processed)
    }.await;
    
    // temp_file automatically cleaned up when dropped
    result
}
```

### Pattern 4: Circuit Breaker for External Services

```rust
use std::sync::atomic::{AtomicU64, Ordering};

pub struct CircuitBreaker {
    failure_count: AtomicU64,
    last_failure: Arc<RwLock<Option<Instant>>>,
    threshold: u64,
    timeout: Duration,
}

impl CircuitBreaker {
    pub async fn call<F, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: Future<Output = Result<T, E>>,
    {
        // Check if circuit is open
        if self.is_open().await {
            return Err(CircuitBreakerError::CircuitOpen);
        }
        
        match operation.await {
            Ok(result) => {
                self.reset().await;
                Ok(result)
            }
            Err(e) => {
                self.record_failure().await;
                Err(CircuitBreakerError::OperationFailed(e))
            }
        }
    }
    
    async fn is_open(&self) -> bool {
        let failure_count = self.failure_count.load(Ordering::Relaxed);
        if failure_count < self.threshold {
            return false;
        }
        
        let last_failure = self.last_failure.read().await;
        if let Some(last_failure_time) = *last_failure {
            last_failure_time.elapsed() < self.timeout
        } else {
            false
        }
    }
}

// Use in MCP tools
#[tool]
async fn resilient_external_call(&self, query: String) -> Result<ApiResponse, ServerError> {
    let operation = async {
        self.external_client
            .get(&format!("/api/search?q={}", query))
            .send()
            .await?
            .json::<ApiResponse>()
            .await
    };
    
    self.circuit_breaker
        .call(operation)
        .await
        .map_err(|e| match e {
            CircuitBreakerError::CircuitOpen => {
                ServerError::ExternalServiceError {
                    service: "Search API".to_string(),
                    message: "Service temporarily unavailable".to_string(),
                }
            }
            CircuitBreakerError::OperationFailed(req_err) => {
                ServerError::ExternalServiceError {
                    service: "Search API".to_string(),
                    message: req_err.to_string(),
                }
            }
        })
}
```

---

## Tool Implementation

### Tool Design Principles

1. **Single Responsibility**: Each tool should have one clear purpose
2. **Idempotent**: Same input should produce same output
3. **Fast**: Target < 50ms response time
4. **Safe**: Comprehensive input validation
5. **Documented**: Clear parameter descriptions and examples

### Tool Categories

#### 1. Query Tools

For retrieving information:

```rust
#[tool]
async fn search_articles(
    &self,
    query: String,
    limit: Option<u32>,
    category: Option<String>
) -> Result<Vec<Article>, ServerError> {
    // Validate parameters
    let limit = limit.unwrap_or(10).min(100); // Default 10, max 100
    self.validate_search_query(&query)?;
    
    // Build search parameters
    let mut search_params = SearchParams {
        query: query.clone(),
        limit,
        category: category.unwrap_or_else(|| "all".to_string()),
    };
    
    // Execute search with caching
    let cache_key = format!("search:{}:{}:{}", search_params.query, 
                           search_params.limit, search_params.category);
    
    if let Some(cached) = self.get_cached_result(&cache_key).await? {
        return Ok(cached);
    }
    
    let results = self.execute_search(search_params).await?;
    self.cache_result(&cache_key, &results).await?;
    
    Ok(results)
}
```

#### 2. Action Tools

For performing operations:

```rust
#[tool]
async fn create_document(
    &self,
    title: String,
    content: String,
    tags: Option<Vec<String>>
) -> Result<DocumentCreationResult, ServerError> {
    // Validate input
    self.validate_document_title(&title)?;
    self.validate_document_content(&content)?;
    
    let tags = tags.unwrap_or_default();
    self.validate_tags(&tags)?;
    
    // Create document
    let document = Document {
        id: uuid::Uuid::new_v4().to_string(),
        title,
        content,
        tags,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    // Store document
    let document_id = self.store_document(document).await?;
    
    // Update metrics
    self.metrics.increment_documents_created().await;
    
    Ok(DocumentCreationResult {
        id: document_id,
        status: "created".to_string(),
        url: format!("/documents/{}", document_id),
    })
}
```

#### 3. Status Tools

For monitoring and health checks:

```rust
#[tool]
async fn get_server_status(&self) -> Result<ServerStatus, ServerError> {
    let state = self.state.read().await;
    
    // Gather system metrics
    let memory_usage = self.get_memory_usage().await;
    let disk_usage = self.get_disk_usage().await;
    
    // Check external dependencies
    let mut dependency_status = HashMap::new();
    for (service, client) in &self.external_clients {
        let status = match client.health_check().await {
            Ok(_) => "healthy".to_string(),
            Err(_) => "unhealthy".to_string(),
        };
        dependency_status.insert(service.clone(), status);
    }
    
    Ok(ServerStatus {
        status: "running".to_string(),
        version: self.config.version.clone(),
        uptime_seconds: self.get_uptime_seconds(),
        request_count: state.request_count,
        memory_usage_mb: memory_usage,
        disk_usage_percent: disk_usage,
        dependency_status,
        last_health_check: chrono::Utc::now(),
    })
}
```

### Parameter Validation

```rust
trait Validator {
    fn validate(&self) -> Result<(), ValidationError>;
}

impl Validator for String {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.is_empty() {
            return Err(ValidationError::EmptyString);
        }
        if self.len() > 1000 {
            return Err(ValidationError::StringTooLong(self.len()));
        }
        Ok(())
    }
}

// Use in tools
#[tool]
async fn validated_tool(&self, text: String, count: u32) -> Result<String, ServerError> {
    // Validate parameters
    text.validate().map_err(|e| ServerError::ValidationError {
        field: "text".to_string(),
        message: e.to_string(),
    })?;
    
    if count == 0 || count > 1000 {
        return Err(ServerError::ValidationError {
            field: "count".to_string(),
            message: "Count must be between 1 and 1000".to_string(),
        });
    }
    
    // Process validated input
    self.process_text(text, count).await
}
```

---

## State Management

### Thread-Safe State Patterns

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

// Central state management
pub struct ServerState {
    // Application data
    documents: HashMap<String, Document>,
    user_sessions: HashMap<String, UserSession>,
    
    // Metrics and monitoring
    request_count: u64,
    error_count: u64,
    last_activity: Option<Instant>,
    
    // Caching
    cache: lru::LruCache<String, CachedData>,
    
    // Configuration
    runtime_config: RuntimeConfig,
}

impl ServerState {
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            documents: HashMap::new(),
            user_sessions: HashMap::new(),
            request_count: 0,
            error_count: 0,
            last_activity: None,
            cache: lru::LruCache::new(max_cache_size),
            runtime_config: RuntimeConfig::default(),
        }
    }
    
    // Safe state updates
    pub fn increment_request_count(&mut self) {
        self.request_count += 1;
        self.last_activity = Some(Instant::now());
    }
    
    pub fn add_document(&mut self, id: String, document: Document) {
        self.documents.insert(id, document);
    }
    
    pub fn get_document(&self, id: &str) -> Option<&Document> {
        self.documents.get(id)
    }
}
```

### Cache Management

```rust
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct CachedData {
    pub data: serde_json::Value,
    pub created_at: Instant,
    pub ttl: Duration,
}

impl CachedData {
    pub fn new(data: serde_json::Value, ttl: Duration) -> Self {
        Self {
            data,
            created_at: Instant::now(),
            ttl,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

// Cache management in server
impl MyMcpServer {
    async fn get_cached<T>(&self, key: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let state = self.state.read().await;
        if let Some(cached) = state.cache.peek(key) {
            if !cached.is_expired() {
                if let Ok(data) = serde_json::from_value(cached.data.clone()) {
                    return Some(data);
                }
            }
        }
        None
    }
    
    async fn set_cache<T>(&self, key: &str, data: &T, ttl: Duration) -> Result<(), ServerError>
    where
        T: serde::Serialize,
    {
        let serialized = serde_json::to_value(data)?;
        let cached = CachedData::new(serialized, ttl);
        
        let mut state = self.state.write().await;
        state.cache.put(key.to_string(), cached);
        
        Ok(())
    }
}
```

### Metrics Collection

```rust
#[derive(Default)]
pub struct ServerMetrics {
    pub tools_executed: HashMap<String, u64>,
    pub response_times: HashMap<String, Vec<Duration>>,
    pub error_counts: HashMap<String, u64>,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl ServerMetrics {
    pub async fn record_tool_execution(&mut self, tool_name: &str, duration: Duration) {
        *self.tools_executed.entry(tool_name.to_string()).or_insert(0) += 1;
        self.response_times
            .entry(tool_name.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
    }
    
    pub async fn record_error(&mut self, tool_name: &str) {
        *self.error_counts.entry(tool_name.to_string()).or_insert(0) += 1;
    }
    
    pub fn get_average_response_time(&self, tool_name: &str) -> Option<Duration> {
        self.response_times.get(tool_name).map(|times| {
            let total: Duration = times.iter().sum();
            total / times.len() as u32
        })
    }
}
```

---

## Error Handling

### Structured Error Types

```rust
#[derive(Error, Debug)]
pub enum ServerError {
    // Client errors (4xx equivalent)
    #[error("Validation failed: {field} - {message}")]
    ValidationError { field: String, message: String },
    
    #[error("Resource not found: {resource_type} '{id}'")]
    NotFound { resource_type: String, id: String },
    
    #[error("Rate limit exceeded: {operation}")]
    RateLimitExceeded { operation: String },
    
    #[error("Permission denied: {action}")]
    PermissionDenied { action: String },
    
    // Server errors (5xx equivalent)
    #[error("External service error: {service} - {message}")]
    ExternalServiceError { service: String, message: String },
    
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
    
    #[error("Internal error: {message}")]
    InternalError { message: String },
    
    // Specific business errors
    #[error("Database error: {operation} failed - {message}")]
    DatabaseError { operation: String, message: String },
    
    #[error("Authentication error: {message}")]
    AuthenticationError { message: String },
}

// Error metadata for monitoring
impl ServerError {
    pub fn error_code(&self) -> &'static str {
        match self {
            ServerError::ValidationError { .. } => "VALIDATION_ERROR",
            ServerError::NotFound { .. } => "NOT_FOUND",
            ServerError::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
            ServerError::PermissionDenied { .. } => "PERMISSION_DENIED",
            ServerError::ExternalServiceError { .. } => "EXTERNAL_SERVICE_ERROR",
            ServerError::ConfigurationError { .. } => "CONFIGURATION_ERROR",
            ServerError::InternalError { .. } => "INTERNAL_ERROR",
            ServerError::DatabaseError { .. } => "DATABASE_ERROR",
            ServerError::AuthenticationError { .. } => "AUTHENTICATION_ERROR",
        }
    }
    
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            ServerError::ValidationError { .. }
                | ServerError::NotFound { .. }
                | ServerError::RateLimitExceeded { .. }
                | ServerError::PermissionDenied { .. }
                | ServerError::AuthenticationError { .. }
        )
    }
}
```

### Error Context and Recovery

```rust
// Error context propagation
#[tool]
async fn complex_operation(&self, input: ComplexInput) -> Result<ComplexOutput, ServerError> {
    let context = format!("Processing complex operation for input ID: {}", input.id);
    
    // Step 1: Validation with context
    self.validate_complex_input(&input)
        .map_err(|e| ServerError::ValidationError {
            field: "input".to_string(),
            message: format!("{} - {}", context, e),
        })?;
    
    // Step 2: External service call with retry
    let external_data = self
        .fetch_external_data(&input.external_id)
        .await
        .or_else(|_| {
            // Fallback strategy
            tracing::warn!("External service failed, using cached data");
            self.get_cached_external_data(&input.external_id)
        })
        .map_err(|e| ServerError::ExternalServiceError {
            service: "External Data Provider".to_string(),
            message: format!("{} - {}", context, e),
        })?;
    
    // Step 3: Business logic with error recovery
    let result = self
        .process_with_business_logic(&input, &external_data)
        .await
        .or_else(|e| {
            tracing::error!("Business logic failed: {}, attempting recovery", e);
            self.recover_from_business_logic_error(&input, e)
        })
        .map_err(|e| ServerError::InternalError {
            message: format!("{} - Business logic failed: {}", context, e),
        })?;
    
    Ok(result)
}

// Graceful degradation
#[tool]
async fn get_enhanced_data(&self, id: String) -> Result<EnhancedData, ServerError> {
    // Core data (required)
    let core_data = self.get_core_data(&id).await?;
    
    // Enhancement data (optional)
    let mut enhanced_data = EnhancedData::from_core(core_data);
    
    // Try to add enhancements, but don't fail if they're unavailable
    if let Ok(metadata) = self.get_metadata(&id).await {
        enhanced_data.metadata = Some(metadata);
    } else {
        tracing::warn!("Metadata unavailable for ID: {}", id);
    }
    
    if let Ok(related) = self.get_related_items(&id).await {
        enhanced_data.related_items = related;
    } else {
        tracing::warn!("Related items unavailable for ID: {}", id);
    }
    
    // Always succeed if we have core data
    Ok(enhanced_data)
}
```

### Error Monitoring

```rust
// Error tracking for monitoring
pub struct ErrorTracker {
    error_counts: HashMap<String, u64>,
    error_rates: HashMap<String, Vec<Instant>>,
    alert_thresholds: HashMap<String, u64>,
}

impl ErrorTracker {
    pub fn record_error(&mut self, error_code: &str) {
        // Count errors
        *self.error_counts.entry(error_code.to_string()).or_insert(0) += 1;
        
        // Track error rate (last hour)
        let now = Instant::now();
        let error_times = self.error_rates
            .entry(error_code.to_string())
            .or_insert_with(Vec::new);
        
        error_times.push(now);
        
        // Keep only errors from last hour
        error_times.retain(|&time| now.duration_since(time) < Duration::from_secs(3600));
        
        // Check alert thresholds
        if let Some(&threshold) = self.alert_thresholds.get(error_code) {
            if error_times.len() as u64 > threshold {
                self.trigger_alert(error_code, error_times.len());
            }
        }
    }
    
    fn trigger_alert(&self, error_code: &str, error_count: usize) {
        tracing::error!(
            "Error rate alert: {} errors of type {} in the last hour",
            error_count,
            error_code
        );
        
        // Send to monitoring system
        // self.monitoring_client.send_alert(...);
    }
}

// Integration with MCP tools
#[tool]
async fn monitored_tool(&self, input: String) -> Result<String, ServerError> {
    let result = self.risky_operation(input).await;
    
    // Record error for monitoring
    if let Err(ref error) = result {
        self.error_tracker
            .write()
            .await
            .record_error(error.error_code());
    }
    
    result
}
```

---

## Testing Strategy

### Test Pyramid

Our testing follows the proven test pyramid:

1. **Unit Tests** (70%): Fast, isolated, focused
2. **Integration Tests** (20%): Component interaction
3. **End-to-End Tests** (10%): Full system validation

### Unit Testing

```rust
#[cfg(test)]
mod unit_tests {
    use super::*;
    use tokio_test;
    
    #[tokio::test]
    async fn test_validation_logic() {
        let server = create_test_server();
        
        // Test valid input
        let valid_result = server.validate_input("valid_input").await;
        assert!(valid_result.is_ok());
        
        // Test invalid inputs
        let empty_result = server.validate_input("").await;
        assert!(empty_result.is_err());
        
        let too_long_result = server.validate_input(&"x".repeat(2000)).await;
        assert!(too_long_result.is_err());
    }
    
    #[tokio::test]
    async fn test_caching_behavior() {
        let server = create_test_server();
        
        // First call should hit external service
        let start = Instant::now();
        let result1 = server.get_data("test_key").await.unwrap();
        let first_duration = start.elapsed();
        
        // Second call should hit cache
        let start = Instant::now();
        let result2 = server.get_data("test_key").await.unwrap();
        let second_duration = start.elapsed();
        
        assert_eq!(result1, result2);
        assert!(second_duration < first_duration / 2); // Cache should be much faster
    }
    
    #[tokio::test]
    async fn test_error_handling() {
        let server = create_test_server();
        
        // Test specific error types
        let validation_error = server.process_invalid_data().await;
        assert!(matches!(validation_error.unwrap_err(), 
                        ServerError::ValidationError { .. }));
        
        let not_found_error = server.get_nonexistent_resource("missing").await;
        assert!(matches!(not_found_error.unwrap_err(), 
                        ServerError::NotFound { .. }));
    }
}
```

### Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_full_workflow() {
        let server = create_test_server_with_real_dependencies().await;
        
        // Test complete workflow
        let create_result = server.create_document(
            "Test Document".to_string(),
            "Test content".to_string(),
            Some(vec!["test".to_string()]),
        ).await;
        
        assert!(create_result.is_ok());
        let document_id = create_result.unwrap().id;
        
        // Verify document was created
        let get_result = server.get_document(document_id.clone()).await;
        assert!(get_result.is_ok());
        
        let document = get_result.unwrap();
        assert_eq!(document.title, "Test Document");
        assert_eq!(document.content, "Test content");
        
        // Test search finds the document
        let search_result = server.search_documents(
            "Test".to_string(),
            None,
            None,
        ).await;
        
        assert!(search_result.is_ok());
        let found_documents = search_result.unwrap();
        assert!(!found_documents.is_empty());
        assert!(found_documents.iter().any(|d| d.id == document_id));
    }
    
    #[tokio::test]
    async fn test_concurrent_operations() {
        let server = Arc::new(create_test_server_with_real_dependencies().await);
        let mut handles = Vec::new();
        
        // Spawn concurrent operations
        for i in 0..10 {
            let server_clone = server.clone();
            let handle = tokio::spawn(async move {
                server_clone.create_document(
                    format!("Document {}", i),
                    format!("Content {}", i),
                    None,
                ).await
            });
            handles.push(handle);
        }
        
        // Wait for all operations to complete
        let mut results = Vec::new();
        for handle in handles {
            let result = handle.await.unwrap();
            results.push(result);
        }
        
        // Verify all operations succeeded
        assert_eq!(results.len(), 10);
        for result in results {
            assert!(result.is_ok());
        }
        
        // Verify state consistency
        let status = server.get_server_status().await.unwrap();
        assert!(status.request_count >= 10);
    }
}
```

### Performance Testing

```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::{Duration, Instant};
    
    #[tokio::test]
    async fn test_response_time_requirements() {
        let server = create_test_server();
        
        // Test multiple operations for consistent performance
        for _ in 0..100 {
            let start = Instant::now();
            let result = server.fast_operation("test_input".to_string()).await;
            let duration = start.elapsed();
            
            assert!(result.is_ok());
            assert!(duration < Duration::from_millis(50), 
                   "Operation took {:?}, expected < 50ms", duration);
        }
    }
    
    #[tokio::test]
    async fn test_throughput() {
        let server = Arc::new(create_test_server());
        let operations_count = 1000;
        let start = Instant::now();
        
        // Execute operations concurrently
        let futures: Vec<_> = (0..operations_count)
            .map(|i| {
                let server_clone = server.clone();
                tokio::spawn(async move {
                    server_clone.lightweight_operation(format!("input_{}", i)).await
                })
            })
            .collect();
        
        // Wait for all operations
        let results = futures::future::join_all(futures).await;
        let total_duration = start.elapsed();
        
        // Verify all succeeded
        for result in results {
            assert!(result.unwrap().is_ok());
        }
        
        // Calculate throughput
        let ops_per_second = operations_count as f64 / total_duration.as_secs_f64();
        assert!(ops_per_second > 100.0, 
               "Throughput {} ops/sec, expected > 100 ops/sec", ops_per_second);
    }
    
    #[tokio::test]
    async fn test_memory_usage() {
        let server = create_test_server();
        
        // Measure baseline memory
        let baseline_memory = get_memory_usage();
        
        // Perform memory-intensive operations
        for i in 0..1000 {
            let result = server.memory_intensive_operation(
                format!("large_data_{}", i)
            ).await;
            assert!(result.is_ok());
        }
        
        // Force garbage collection
        for _ in 0..3 {
            tokio::task::yield_now().await;
        }
        
        // Measure final memory
        let final_memory = get_memory_usage();
        let memory_increase = final_memory - baseline_memory;
        
        // Memory increase should be reasonable
        assert!(memory_increase < 100_000_000, // 100MB
               "Memory increased by {} bytes, expected < 100MB", memory_increase);
    }
}
```

### Security Testing

```rust
#[cfg(test)]
mod security_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_input_sanitization() {
        let server = create_test_server();
        
        // Test SQL injection attempts
        let sql_injection_inputs = vec![
            "'; DROP TABLE users; --",
            "1' OR '1'='1",
            "admin'/*",
            "1; INSERT INTO users (username) VALUES ('hacker'); --",
        ];
        
        for malicious_input in sql_injection_inputs {
            let result = server.database_query(malicious_input.to_string()).await;
            // Should either sanitize or reject
            match result {
                Ok(data) => {
                    // If successful, verify no malicious effects
                    assert!(!data.contains("DROP TABLE"));
                    assert!(!data.contains("INSERT INTO"));
                }
                Err(ServerError::ValidationError { .. }) => {
                    // Acceptable to reject malicious input
                }
                Err(other) => {
                    panic!("Unexpected error for malicious input: {:?}", other);
                }
            }
        }
    }
    
    #[tokio::test]
    async fn test_rate_limiting() {
        let server = create_test_server();
        let client_id = "test_client".to_string();
        
        // Rapidly send requests
        let mut success_count = 0;
        let mut rate_limited_count = 0;
        
        for _ in 0..100 {
            match server.rate_limited_operation(client_id.clone()).await {
                Ok(_) => success_count += 1,
                Err(ServerError::RateLimitExceeded { .. }) => rate_limited_count += 1,
                Err(other) => panic!("Unexpected error: {:?}", other),
            }
        }
        
        // Should have some rate limiting
        assert!(rate_limited_count > 0, "Rate limiting not working");
        assert!(success_count > 0, "No requests should succeed");
    }
    
    #[tokio::test]
    async fn test_authentication() {
        let server = create_test_server();
        
        // Test without authentication
        let unauth_result = server.protected_operation(
            "sensitive_data".to_string(),
            None, // No auth token
        ).await;
        
        assert!(matches!(unauth_result.unwrap_err(), 
                        ServerError::AuthenticationError { .. }));
        
        // Test with invalid authentication
        let invalid_auth_result = server.protected_operation(
            "sensitive_data".to_string(),
            Some("invalid_token".to_string()),
        ).await;
        
        assert!(matches!(invalid_auth_result.unwrap_err(), 
                        ServerError::AuthenticationError { .. }));
        
        // Test with valid authentication
        let valid_token = server.generate_test_token().await;
        let auth_result = server.protected_operation(
            "sensitive_data".to_string(),
            Some(valid_token),
        ).await;
        
        assert!(auth_result.is_ok());
    }
}
```

---

## Performance Guidelines

### Response Time Targets

- **Query Tools**: < 50ms (cached) / < 200ms (uncached)
- **Action Tools**: < 100ms (simple) / < 500ms (complex)
- **Status Tools**: < 10ms (in-memory) / < 50ms (with health checks)

### Memory Management

```rust
// Efficient data structures
use std::collections::HashMap;
use lru::LruCache;
use dashmap::DashMap; // For concurrent access

pub struct OptimizedServer {
    // Use LRU cache for bounded memory
    cache: Arc<RwLock<LruCache<String, CachedData>>>,
    
    // Use DashMap for concurrent access without locks
    session_data: DashMap<String, SessionData>,
    
    // Pool expensive resources
    http_client: reqwest::Client,
    database_pool: sqlx::PgPool,
}

// Memory-efficient operations
#[tool]
async fn stream_large_data(&self, query: String) -> Result<Vec<DataChunk>, ServerError> {
    // Process data in chunks to avoid loading everything into memory
    let mut chunks = Vec::new();
    let mut offset = 0;
    const CHUNK_SIZE: usize = 1000;
    
    loop {
        let chunk = self.fetch_data_chunk(&query, offset, CHUNK_SIZE).await?;
        if chunk.is_empty() {
            break;
        }
        
        // Process chunk immediately to free memory
        let processed_chunk = self.process_chunk(chunk).await?;
        chunks.push(processed_chunk);
        
        offset += CHUNK_SIZE;
    }
    
    Ok(chunks)
}
```

### Connection Pooling

```rust
use sqlx::{Pool, Postgres};
use reqwest::Client;

pub struct ConnectionPooledServer {
    // Database connection pool
    db_pool: Pool<Postgres>,
    
    // HTTP client with connection pooling
    http_client: Client,
}

impl ConnectionPooledServer {
    pub async fn new(database_url: &str) -> Result<Self, ServerError> {
        // Configure database pool
        let db_pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .min_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(1800))
            .connect(database_url)
            .await?;
        
        // Configure HTTP client with connection pooling
        let http_client = reqwest::Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .timeout(Duration::from_secs(30))
            .build()?;
        
        Ok(Self {
            db_pool,
            http_client,
        })
    }
}

#[rmcp::server]
impl ConnectionPooledServer {
    #[tool]
    async fn efficient_database_query(&self, sql: String) -> Result<Vec<Row>, ServerError> {
        // Use connection pool efficiently
        let query_result = sqlx::query(&sql)
            .fetch_all(&self.db_pool)
            .await?;
        
        Ok(query_result)
    }
    
    #[tool]
    async fn efficient_http_request(&self, url: String) -> Result<String, ServerError> {
        // Reuse HTTP client connections
        let response = self.http_client
            .get(&url)
            .send()
            .await?
            .text()
            .await?;
        
        Ok(response)
    }
}
```

### Async Optimization

```rust
// Parallel processing
#[tool]
async fn parallel_processing(&self, items: Vec<String>) -> Result<Vec<ProcessedItem>, ServerError> {
    // Process items in parallel
    let futures: Vec<_> = items
        .into_iter()
        .map(|item| self.process_single_item(item))
        .collect();
    
    // Wait for all to complete
    let results = futures::future::try_join_all(futures).await?;
    Ok(results)
}

// Batched operations
#[tool]
async fn batched_operations(&self, ids: Vec<String>) -> Result<Vec<DataItem>, ServerError> {
    // Group IDs into batches for efficient processing
    const BATCH_SIZE: usize = 50;
    let mut all_results = Vec::new();
    
    for batch in ids.chunks(BATCH_SIZE) {
        let batch_results = self.fetch_batch(batch.to_vec()).await?;
        all_results.extend(batch_results);
    }
    
    Ok(all_results)
}

// Streaming responses for large datasets
#[tool]
async fn stream_results(&self, query: String) -> Result<StreamingResult, ServerError> {
    // Return a streaming result instead of loading everything
    let stream = self.create_result_stream(query).await?;
    
    Ok(StreamingResult {
        total_count: stream.estimated_size(),
        stream_id: stream.id(),
        first_batch: stream.next_batch(100).await?,
    })
}
```

---

## Security Best Practices

### Input Validation

```rust
use regex::Regex;
use once_cell::sync::Lazy;

// Pre-compiled regex patterns for efficiency
static SQL_INJECTION_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(union|select|insert|update|delete|drop|create|alter|exec|script)").unwrap()
});

static XSS_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)<script|javascript:|on\w+\s*=").unwrap()
});

pub trait SecurityValidator {
    fn validate_sql_safety(&self) -> Result<(), SecurityError>;
    fn validate_xss_safety(&self) -> Result<(), SecurityError>;
    fn sanitize(&self) -> String;
}

impl SecurityValidator for String {
    fn validate_sql_safety(&self) -> Result<(), SecurityError> {
        if SQL_INJECTION_PATTERN.is_match(self) {
            return Err(SecurityError::SqlInjectionAttempt(self.clone()));
        }
        Ok(())
    }
    
    fn validate_xss_safety(&self) -> Result<(), SecurityError> {
        if XSS_PATTERN.is_match(self) {
            return Err(SecurityError::XssAttempt(self.clone()));
        }
        Ok(())
    }
    
    fn sanitize(&self) -> String {
        // Remove dangerous characters
        self.chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || ".,!?-_@".contains(*c))
            .collect()
    }
}

// Use in tools
#[tool]
async fn secure_database_query(&self, query: String) -> Result<QueryResult, ServerError> {
    // Validate input for security
    query.validate_sql_safety()
        .map_err(|e| ServerError::ValidationError {
            field: "query".to_string(),
            message: format!("Security validation failed: {}", e),
        })?;
    
    // Additional length validation
    if query.len() > 1000 {
        return Err(ServerError::ValidationError {
            field: "query".to_string(),
            message: "Query too long".to_string(),
        });
    }
    
    // Execute with prepared statements
    self.execute_prepared_query(query).await
}
```

### Authentication and Authorization

```rust
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // Subject (user ID)
    pub exp: usize,   // Expiration time
    pub iat: usize,   // Issued at
    pub roles: Vec<String>,  // User roles
}

pub struct AuthService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl AuthService {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
        }
    }
    
    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        let token_data = decode::<Claims>(
            token,
            &self.decoding_key,
            &Validation::default(),
        )?;
        
        Ok(token_data.claims)
    }
    
    pub fn check_permission(&self, claims: &Claims, required_role: &str) -> Result<(), AuthError> {
        if !claims.roles.contains(&required_role.to_string()) {
            return Err(AuthError::InsufficientPermissions {
                required: required_role.to_string(),
                available: claims.roles.clone(),
            });
        }
        Ok(())
    }
}

// Protected tool example
#[tool]
async fn protected_operation(
    &self,
    auth_token: String,
    sensitive_data: String,
) -> Result<String, ServerError> {
    // Validate authentication
    let claims = self.auth_service
        .validate_token(&auth_token)
        .map_err(|e| ServerError::AuthenticationError {
            message: e.to_string(),
        })?;
    
    // Check authorization
    self.auth_service
        .check_permission(&claims, "data_access")
        .map_err(|e| ServerError::PermissionDenied {
            action: format!("access sensitive data: {}", e),
        })?;
    
    // Log access for audit
    tracing::info!(
        "User {} accessed sensitive data",
        claims.sub
    );
    
    // Process the request
    self.process_sensitive_data(sensitive_data).await
}
```

### Rate Limiting

```rust
use governor::{Quota, RateLimiter, Jitter};
use nonzero_ext::*;
use std::collections::HashMap;
use std::net::IpAddr;

pub struct RateLimitingService {
    // Per-client rate limiters
    client_limiters: DashMap<String, RateLimiter<String, DashHasher, DefaultClock>>,
    
    // Global rate limiter
    global_limiter: RateLimiter<(), DashHasher, DefaultClock>,
    
    // Configuration
    per_client_quota: Quota,
    global_quota: Quota,
}

impl RateLimitingService {
    pub fn new() -> Self {
        // 100 requests per minute per client
        let per_client_quota = Quota::per_minute(nonzero!(100u32));
        
        // 10,000 requests per minute globally
        let global_quota = Quota::per_minute(nonzero!(10_000u32));
        
        Self {
            client_limiters: DashMap::new(),
            global_limiter: RateLimiter::direct(global_quota),
            per_client_quota,
            global_quota,
        }
    }
    
    pub async fn check_rate_limit(&self, client_id: &str) -> Result<(), RateLimitError> {
        // Check global rate limit first
        self.global_limiter
            .check()
            .map_err(|_| RateLimitError::GlobalLimitExceeded)?;
        
        // Check per-client rate limit
        let client_limiter = self.client_limiters
            .entry(client_id.to_string())
            .or_insert_with(|| RateLimiter::keyed(self.per_client_quota));
        
        client_limiter
            .check_key(&client_id.to_string())
            .map_err(|_| RateLimitError::ClientLimitExceeded {
                client_id: client_id.to_string(),
            })?;
        
        Ok(())
    }
}

// Use in tools
#[tool]
async fn rate_limited_tool(
    &self,
    client_id: String,
    operation_data: String,
) -> Result<String, ServerError> {
    // Check rate limit
    self.rate_limiting_service
        .check_rate_limit(&client_id)
        .await
        .map_err(|e| match e {
            RateLimitError::ClientLimitExceeded { .. } => {
                ServerError::RateLimitExceeded {
                    operation: "rate_limited_tool".to_string(),
                }
            }
            RateLimitError::GlobalLimitExceeded => {
                ServerError::InternalError {
                    message: "System overloaded".to_string(),
                }
            }
        })?;
    
    // Process the operation
    self.process_operation(operation_data).await
}
```

### Data Encryption

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, NewAead}};
use rand::RngCore;

pub struct EncryptionService {
    cipher: Aes256Gcm,
}

impl EncryptionService {
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        
        Self { cipher }
    }
    
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt data
        let ciphertext = self.cipher
            .encrypt(nonce, data)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;
        
        // Prepend nonce to ciphertext
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }
    
    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if encrypted_data.len() < 12 {
            return Err(EncryptionError::InvalidData("Data too short".to_string()));
        }
        
        // Extract nonce and ciphertext
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        // Decrypt data
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;
        
        Ok(plaintext)
    }
}

// Use for sensitive data
#[tool]
async fn store_sensitive_data(
    &self,
    auth_token: String,
    sensitive_info: String,
) -> Result<String, ServerError> {
    // Authenticate first
    let claims = self.authenticate(&auth_token).await?;
    
    // Encrypt sensitive data
    let encrypted_data = self.encryption_service
        .encrypt(sensitive_info.as_bytes())
        .map_err(|e| ServerError::InternalError {
            message: format!("Encryption failed: {}", e),
        })?;
    
    // Store encrypted data
    let storage_id = self.store_encrypted_data(encrypted_data).await?;
    
    // Log access (without sensitive data)
    tracing::info!(
        "User {} stored sensitive data with ID {}",
        claims.sub,
        storage_id
    );
    
    Ok(storage_id)
}
```

---

## Deployment Preparation

### Configuration Management

```rust
use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
    pub monitoring: MonitoringConfig,
    pub external_services: ExternalServicesConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub name: String,
    pub version: String,
    pub bind_address: String,
    pub port: u16,
    pub worker_threads: usize,
    pub max_connections: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    pub