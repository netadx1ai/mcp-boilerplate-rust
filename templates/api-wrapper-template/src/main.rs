//! API Wrapper Template
//!
//! A production-ready MCP server template for wrapping external APIs with
//! authentication, resilience patterns, and comprehensive error handling.
//!
//! ## Features
//!
//! - Multiple authentication methods (API Key, Bearer Token, Basic Auth, OAuth)
//! - Rate limiting with token bucket algorithm
//! - Circuit breaker pattern for fault tolerance
//! - Exponential backoff retry logic
//! - Request/response transformation
//! - Comprehensive error handling and logging
//! - Statistics tracking and monitoring
//!
//! ## Customization Points
//!
//! 1. **API Configuration**: Modify `create_default_apis()` to add your APIs
//! 2. **Authentication**: Extend `AuthMethod` enum for custom auth schemes
//! 3. **Transformations**: Implement custom request/response transformations
//! 4. **Error Handling**: Add domain-specific error types
//! 5. **Rate Limits**: Adjust rate limiting strategies per API
//!
//! ## Usage
//!
//! ```bash
//! cargo run --bin api-wrapper-server
//! ```

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Parser;
use dashmap::DashMap;
use reqwest::{Client, Method, RequestBuilder, Response};
use rmcp::{
    handler::server::wrapper::Parameters, model::*, tool, tool_router, transport::stdio,
    ErrorData as McpError, ServiceExt, schemars::JsonSchema,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{sync::Mutex, time::{sleep, timeout}};
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;
use url::Url;

/// Command line arguments
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

/// API Wrapper Server errors
#[derive(thiserror::Error, Debug)]
pub enum ApiWrapperError {
    #[error("API not found: {0}")]
    ApiNotFound(String),
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Rate limit exceeded for API: {0}")]
    RateLimitExceeded(String),
    #[error("API request failed: {0}")]
    RequestFailed(String),
    #[error("Response transformation error: {0}")]
    TransformationError(String),
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    #[error("Circuit breaker open for API: {0}")]
    CircuitBreakerOpen(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
}

/// Authentication methods supported by the wrapper
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum AuthMethod {
    #[serde(rename = "api_key")]
    ApiKey {
        header: String,
        value: String,
    },
    #[serde(rename = "bearer_token")]
    BearerToken {
        token: String,
    },
    #[serde(rename = "basic_auth")]
    BasicAuth {
        username: String,
        password: String,
    },
    #[serde(rename = "oauth")]
    OAuth {
        access_token: String,
        refresh_token: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    },
    #[serde(rename = "custom_header")]
    CustomHeader {
        header_name: String,
        header_value: String,
    },
}

/// External API configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ApiConfig {
    pub name: String,
    pub base_url: String,
    pub description: String,
    pub auth_method: Option<AuthMethod>,
    pub rate_limit_per_minute: u32,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub default_headers: HashMap<String, String>,
}

/// Tool argument schemas
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CallApiArgs {
    /// Name of the configured API to call
    pub api_name: String,
    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    #[serde(default = "default_method")]
    pub method: String,
    /// API path/endpoint to call
    pub path: String,
    /// Query parameters (optional)
    #[serde(default)]
    pub params: Option<HashMap<String, serde_json::Value>>,
    /// Request body (optional)
    #[serde(default)]
    pub body: Option<serde_json::Value>,
    /// Additional headers (optional)
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetStatsArgs {
    /// API name to get stats for (optional - if not provided, returns all stats)
    #[serde(default)]
    pub api_name: Option<String>,
}

fn default_method() -> String {
    "GET".to_string()
}

/// Rate limiting implementation using token bucket algorithm
#[derive(Debug)]
struct RateLimiter {
    tokens: AtomicU64,
    last_refill: AtomicU64,
    capacity: u64,
    refill_rate: u64, // tokens per second
}

impl RateLimiter {
    fn new(rate_per_minute: u32) -> Self {
        let capacity = rate_per_minute as u64;
        let refill_rate = capacity / 60; // tokens per second
        
        Self {
            tokens: AtomicU64::new(capacity),
            last_refill: AtomicU64::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ),
            capacity,
            refill_rate: refill_rate.max(1), // At least 1 token per second
        }
    }

    fn try_acquire(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let last_refill = self.last_refill.load(Ordering::Relaxed);
        let elapsed = now - last_refill;
        
        // Refill tokens based on elapsed time
        if elapsed > 0 {
            let tokens_to_add = (elapsed * self.refill_rate).min(self.capacity);
            let current_tokens = self.tokens.load(Ordering::Relaxed);
            let new_tokens = (current_tokens + tokens_to_add).min(self.capacity);
            
            self.tokens.store(new_tokens, Ordering::Relaxed);
            self.last_refill.store(now, Ordering::Relaxed);
        }

        // Try to acquire a token
        let current_tokens = self.tokens.load(Ordering::Relaxed);
        if current_tokens > 0 {
            self.tokens.fetch_sub(1, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
}

/// Circuit breaker for handling API failures
#[derive(Debug)]
struct CircuitBreaker {
    failure_count: AtomicU64,
    success_count: AtomicU64,
    last_failure_time: AtomicU64,
    state: std::sync::RwLock<CircuitBreakerState>,
    failure_threshold: u64,
    success_threshold: u64,
    timeout_seconds: u64,
}

#[derive(Debug, Clone, PartialEq)]
enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    fn new() -> Self {
        Self {
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            last_failure_time: AtomicU64::new(0),
            state: std::sync::RwLock::new(CircuitBreakerState::Closed),
            failure_threshold: 5,
            success_threshold: 3,
            timeout_seconds: 60,
        }
    }

    fn can_proceed(&self) -> bool {
        let state = self.state.read().unwrap();
        match *state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let last_failure = self.last_failure_time.load(Ordering::Relaxed);
                
                if now - last_failure > self.timeout_seconds {
                    drop(state);
                    *self.state.write().unwrap() = CircuitBreakerState::HalfOpen;
                    self.success_count.store(0, Ordering::Relaxed);
                    true
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    fn record_success(&self) {
        let state = self.state.read().unwrap();
        match *state {
            CircuitBreakerState::Closed => {
                self.failure_count.store(0, Ordering::Relaxed);
            }
            CircuitBreakerState::HalfOpen => {
                let successes = self.success_count.fetch_add(1, Ordering::Relaxed);
                if successes >= self.success_threshold - 1 {
                    drop(state);
                    *self.state.write().unwrap() = CircuitBreakerState::Closed;
                    self.failure_count.store(0, Ordering::Relaxed);
                    self.success_count.store(0, Ordering::Relaxed);
                }
            }
            _ => {}
        }
    }

    fn record_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::Relaxed);
        self.last_failure_time.store(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            Ordering::Relaxed,
        );

        if failures >= self.failure_threshold - 1 {
            *self.state.write().unwrap() = CircuitBreakerState::Open;
        }
    }

    fn get_state(&self) -> String {
        let state = self.state.read().unwrap();
        format!("{:?}", *state)
    }
}

/// API call statistics
#[derive(Debug, Default, Serialize, Clone)]
struct ApiStats {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    average_response_time_ms: f64,
    last_request_time: Option<DateTime<Utc>>,
    circuit_breaker_state: String,
    rate_limit_hits: u64,
}

/// API Wrapper Server implementation
#[derive(Clone)]
pub struct ApiWrapperServer {
    apis: Arc<Mutex<HashMap<String, ApiConfig>>>,
    http_client: Client,
    rate_limiters: Arc<DashMap<String, RateLimiter>>,
    circuit_breakers: Arc<DashMap<String, CircuitBreaker>>,
    stats: Arc<DashMap<String, ApiStats>>,
    start_time: SystemTime,
}

impl ApiWrapperServer {
    /// Create a new API Wrapper Server
    pub fn new() -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("MCP-API-Wrapper/1.0")
            .build()
            .expect("Failed to create HTTP client");

        let server = Self {
            apis: Arc::new(Mutex::new(HashMap::new())),
            http_client,
            rate_limiters: Arc::new(DashMap::new()),
            circuit_breakers: Arc::new(DashMap::new()),
            stats: Arc::new(DashMap::new()),
            start_time: SystemTime::now(),
        };

        // Initialize with example APIs (customize this for your use case)
        server.init_default_apis();
        server
    }

    /// Initialize with default API configurations
    fn init_default_apis(&self) {
        let default_apis = create_default_apis();
        for api in default_apis {
            self.add_api_sync(api);
        }
    }

    /// Add a single API configuration (synchronous)
    fn add_api_sync(&self, api: ApiConfig) {
        let name = api.name.clone();
        
        // Initialize rate limiter
        self.rate_limiters.insert(
            name.clone(),
            RateLimiter::new(api.rate_limit_per_minute),
        );
        
        // Initialize circuit breaker
        self.circuit_breakers.insert(
            name.clone(),
            CircuitBreaker::new(),
        );
        
        // Initialize stats
        self.stats.insert(name.clone(), ApiStats::default());
        
        // Store API config (blocking for initialization)
        if let Ok(mut apis) = self.apis.try_lock() {
            apis.insert(name, api);
        }
    }

    /// Execute an API call with full resilience patterns
    async fn execute_api_call(
        &self,
        api_name: &str,
        method: &str,
        path: &str,
        params: Option<HashMap<String, serde_json::Value>>,
        body: Option<serde_json::Value>,
        headers: Option<HashMap<String, String>>,
    ) -> Result<serde_json::Value, ApiWrapperError> {
        let apis = self.apis.lock().await;
        let config = apis.get(api_name)
            .ok_or_else(|| ApiWrapperError::ApiNotFound(api_name.to_string()))?
            .clone();
        drop(apis);

        // Check rate limiting
        if let Some(rate_limiter) = self.rate_limiters.get(api_name) {
            if !rate_limiter.try_acquire() {
                // Update stats
                if let Some(mut stats) = self.stats.get_mut(api_name) {
                    stats.rate_limit_hits += 1;
                }
                return Err(ApiWrapperError::RateLimitExceeded(api_name.to_string()));
            }
        }

        // Check circuit breaker
        if let Some(circuit_breaker) = self.circuit_breakers.get(api_name) {
            if !circuit_breaker.can_proceed() {
                return Err(ApiWrapperError::CircuitBreakerOpen(api_name.to_string()));
            }
        }

        let start_time = std::time::Instant::now();
        let mut last_error = None;

        // Retry logic with exponential backoff
        for attempt in 0..=config.retry_attempts {
            let result = timeout(
                Duration::from_secs(config.timeout_seconds),
                self.make_http_request(&config, method, path, &params, &body, &headers)
            ).await;

            match result {
                Ok(Ok(response)) => {
                    // Record success
                    if let Some(circuit_breaker) = self.circuit_breakers.get(api_name) {
                        circuit_breaker.record_success();
                    }

                    // Update statistics
                    self.update_success_stats(api_name, start_time);
                    return Ok(response);
                },
                Ok(Err(e)) => {
                    last_error = Some(e);
                },
                Err(_) => {
                    last_error = Some(ApiWrapperError::TimeoutError(
                        format!("Request to {} timed out after {}s", api_name, config.timeout_seconds)
                    ));
                }
            }

            // Exponential backoff before retry
            if attempt < config.retry_attempts {
                let delay_ms = 100 * (2_u64.pow(attempt));
                debug!("Retrying {} in {}ms (attempt {}/{})", api_name, delay_ms, attempt + 1, config.retry_attempts);
                sleep(Duration::from_millis(delay_ms)).await;
            }
        }

        // Record failure
        if let Some(circuit_breaker) = self.circuit_breakers.get(api_name) {
            circuit_breaker.record_failure();
        }

        // Update failure statistics
        self.update_failure_stats(api_name);

        Err(last_error.unwrap_or_else(|| {
            ApiWrapperError::RequestFailed("Unknown error".to_string())
        }))
    }

    /// Make the actual HTTP request
    async fn make_http_request(
        &self,
        config: &ApiConfig,
        method: &str,
        path: &str,
        params: &Option<HashMap<String, serde_json::Value>>,
        body: &Option<serde_json::Value>,
        extra_headers: &Option<HashMap<String, String>>,
    ) -> Result<serde_json::Value, ApiWrapperError> {
        // Build URL
        let mut url = Url::parse(&config.base_url)
            .map_err(|e| ApiWrapperError::ConfigurationError(format!("Invalid base URL: {}", e)))?;
        
        url.set_path(path);

        // Add query parameters
        if let Some(params) = params {
            let mut query_pairs = url.query_pairs_mut();
            for (key, value) in params {
                let value_str = match value {
                    serde_json::Value::String(s) => s.clone(),
                    _ => value.to_string(),
                };
                query_pairs.append_pair(key, &value_str);
            }
        }

        // Build request
        let http_method = Method::from_bytes(method.as_bytes())
            .map_err(|_| ApiWrapperError::ConfigurationError(format!("Invalid HTTP method: {}", method)))?;
        
        let mut request = self.http_client.request(http_method, url);

        // Add authentication
        request = self.add_authentication(request, &config.auth_method)?;

        // Add default headers
        for (key, value) in &config.default_headers {
            request = request.header(key, value);
        }

        // Add extra headers
        if let Some(headers) = extra_headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }

        // Add body for non-GET requests
        if let Some(body) = body {
            request = request.json(body);
        }

        // Execute request
        let response = request.send().await
            .map_err(|e| ApiWrapperError::NetworkError(e.to_string()))?;

        // Handle response
        self.handle_response(response).await
    }

    /// Add authentication to the request
    fn add_authentication(
        &self,
        mut request: RequestBuilder,
        auth_method: &Option<AuthMethod>,
    ) -> Result<RequestBuilder, ApiWrapperError> {
        if let Some(auth) = auth_method {
            match auth {
                AuthMethod::ApiKey { header, value } => {
                    request = request.header(header, value);
                }
                AuthMethod::BearerToken { token } => {
                    request = request.bearer_auth(token);
                }
                AuthMethod::BasicAuth { username, password } => {
                    request = request.basic_auth(username, Some(password));
                }
                AuthMethod::OAuth { access_token, .. } => {
                    request = request.bearer_auth(access_token);
                }
                AuthMethod::CustomHeader { header_name, header_value } => {
                    request = request.header(header_name, header_value);
                }
            }
        }
        Ok(request)
    }

    /// Handle HTTP response
    async fn handle_response(&self, response: Response) -> Result<serde_json::Value, ApiWrapperError> {
        let status = response.status();
        
        if status.is_success() {
            let text = response.text().await
                .map_err(|e| ApiWrapperError::NetworkError(e.to_string()))?;
            
            // Try to parse as JSON, fallback to string
            match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(json) => Ok(json),
                Err(_) => Ok(serde_json::Value::String(text)),
            }
        } else {
            Err(ApiWrapperError::RequestFailed(
                format!("HTTP {}: {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown"))
            ))
        }
    }

    /// Update success statistics
    fn update_success_stats(&self, api_name: &str, start_time: std::time::Instant) {
        if let Some(mut stats) = self.stats.get_mut(api_name) {
            stats.total_requests += 1;
            stats.successful_requests += 1;
            
            let elapsed_ms = start_time.elapsed().as_millis() as f64;
            stats.average_response_time_ms = 
                (stats.average_response_time_ms * (stats.total_requests - 1) as f64 + elapsed_ms) 
                / stats.total_requests as f64;
            
            stats.last_request_time = Some(Utc::now());
            
            // Update circuit breaker state
            if let Some(cb) = self.circuit_breakers.get(api_name) {
                stats.circuit_breaker_state = cb.get_state();
            }
        }
    }

    /// Update failure statistics
    fn update_failure_stats(&self, api_name: &str) {
        if let Some(mut stats) = self.stats.get_mut(api_name) {
            stats.total_requests += 1;
            stats.failed_requests += 1;
            stats.last_request_time = Some(Utc::now());
            
            // Update circuit breaker state
            if let Some(cb) = self.circuit_breakers.get(api_name) {
                stats.circuit_breaker_state = cb.get_state();
            }
        }
    }

    /// List all configured APIs
    async fn list_configured_apis(&self) -> serde_json::Value {
        let apis = self.apis.lock().await;
        let apis: Vec<_> = apis.iter().map(|(name, config)| {
            serde_json::json!({
                "name": name,
                "description": config.description,
                "base_url": config.base_url,
                "rate_limit_per_minute": config.rate_limit_per_minute,
                "timeout_seconds": config.timeout_seconds,
                "retry_attempts": config.retry_attempts
            })
        }).collect();
        
        serde_json::json!({
            "configured_apis": apis,
            "total_count": apis.len()
        })
    }

    /// Get API statistics
    fn get_api_statistics(&self, api_name: Option<&str>) -> serde_json::Value {
        if let Some(name) = api_name {
            if let Some(stats) = self.stats.get(name) {
                serde_json::to_value(&*stats).unwrap_or_else(|_| serde_json::json!({}))
            } else {
                serde_json::json!({ "error": format!("API '{}' not found", name) })
            }
        } else {
            let all_stats: HashMap<String, ApiStats> = self.stats.iter()
                .map(|entry| (entry.key().clone(), entry.value().clone()))
                .collect();
            serde_json::to_value(all_stats).unwrap_or_else(|_| serde_json::json!({}))
        }
    }

    /// Get server status
    fn get_server_status_info(&self) -> serde_json::Value {
        let uptime_seconds = self.start_time.elapsed().unwrap_or_default().as_secs();
        
        serde_json::json!({
            "status": "running",
            "uptime_seconds": uptime_seconds,
            "configured_apis": self.stats.len(),
            "total_requests": self.stats.iter().map(|s| s.total_requests).sum::<u64>(),
            "start_time": chrono::DateTime::<Utc>::from(self.start_time).to_rfc3339()
        })
    }
}

impl Default for ApiWrapperServer {
    fn default() -> Self {
        Self::new()
    }
}

/// MCP tool router implementation
#[tool_router]
impl ApiWrapperServer {
    /// Make an API call to a configured external service with resilience patterns
    #[tool(description = "Make an API call to a configured external service with resilience patterns")]
    async fn call_api(
        &self,
        Parameters(args): Parameters<CallApiArgs>,
    ) -> Result<CallToolResult, McpError> {
        info!("Making API call to {} {} {}", args.api_name, args.method, args.path);

        match self.execute_api_call(
            &args.api_name,
            &args.method,
            &args.path,
            args.params,
            args.body,
            args.headers,
        ).await {
            Ok(response) => {
                let response_text = serde_json::to_string_pretty(&response)
                    .unwrap_or_else(|_| "Failed to format response".to_string());
                Ok(CallToolResult::success(vec![Content::text(response_text)]))
            },
            Err(e) => {
                let error_msg = format!("API call failed: {}", e);
                warn!("{}", error_msg);
                Ok(CallToolResult::error(vec![Content::text(error_msg)]))
            }
        }
    }

    /// List all configured external APIs
    #[tool(description = "List all configured external APIs")]
    async fn list_apis(&self) -> Result<CallToolResult, McpError> {
        let apis = self.list_configured_apis().await;
        let apis_text = serde_json::to_string_pretty(&apis)
            .unwrap_or_else(|_| "Failed to format APIs list".to_string());
        Ok(CallToolResult::success(vec![Content::text(apis_text)]))
    }

    /// Get statistics for API calls (all APIs or specific API)
    #[tool(description = "Get statistics for API calls (all APIs or specific API)")]
    async fn get_api_stats(
        &self,
        Parameters(args): Parameters<GetStatsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let stats = self.get_api_statistics(args.api_name.as_deref());
        let stats_text = serde_json::to_string_pretty(&stats)
            .unwrap_or_else(|_| "Failed to format statistics".to_string());
        Ok(CallToolResult::success(vec![Content::text(stats_text)]))
    }

    /// Get the current status of the API wrapper server
    #[tool(description = "Get the current status of the API wrapper server")]
    async fn get_server_status(&self) -> Result<CallToolResult, McpError> {
        let status = self.get_server_status_info();
        let status_text = serde_json::to_string_pretty(&status)
            .unwrap_or_else(|_| "Failed to format server status".to_string());
        Ok(CallToolResult::success(vec![Content::text(status_text)]))
    }
}

/// Create default API configurations (customize this for your use case)
fn create_default_apis() -> Vec<ApiConfig> {
    vec![
        ApiConfig {
            name: "example-rest-api".to_string(),
            base_url: "https://api.example.com".to_string(),
            description: "Example REST API with API key authentication".to_string(),
            auth_method: Some(AuthMethod::ApiKey {
                header: "X-API-Key".to_string(),
                value: "your-api-key-here".to_string(),
            }),
            rate_limit_per_minute: 60,
            timeout_seconds: 10,
            retry_attempts: 3,
            default_headers: {
                let mut headers = HashMap::new();
                headers.insert("Accept".to_string(), "application/json".to_string());
                headers.insert("Content-Type".to_string(), "application/json".to_string());
                headers
            },
        },
        ApiConfig {
            name: "bearer-token-api".to_string(),
            base_url: "https://secure-api.example.com".to_string(),
            description: "API requiring Bearer token authentication".to_string(),
            auth_method: Some(AuthMethod::BearerToken {
                token: "your-bearer-token-here".to_string(),
            }),
            rate_limit_per_minute: 100,
            timeout_seconds: 15,
            retry_attempts: 2,
            default_headers: HashMap::new(),
        },
        ApiConfig {
            name: "public-api".to_string(),
            base_url: "https://public.example.com".to_string(),
            description: "Public API with no authentication required".to_string(),
            auth_method: None,
            rate_limit_per_minute: 1000,
            timeout_seconds: 5,
            retry_attempts: 1,
            default_headers: HashMap::new(),
        },
    ]
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize tracing
    let env_filter = if args.debug {
        EnvFilter::new("debug")
    } else {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"))
    };

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .init();

    info!("Starting API Wrapper Server...");

    // Create the server
    let server = ApiWrapperServer::new();
    
    info!("Configured {} APIs", server.stats.len());
    for entry in server.stats.iter() {
        info!("  - {}", entry.key());
    }

    // Start the MCP server with stdio transport
    let service = server.serve(stdio()).await.inspect_err(|e| {
        error!("❌ Failed to start server: {}", e);
    })?;

    info!("✅ API Wrapper Server ready for MCP connections");
    service.waiting().await.inspect_err(|e| {
        error!("❌ Server error: {}", e);
    })?;

    info!("Server shutdown complete");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let server = ApiWrapperServer::new();
        assert!(!server.stats.is_empty());
        assert_eq!(server.stats.len(), 3); // Default APIs
    }

    #[tokio::test]
    async fn test_list_apis() {
        let server = ApiWrapperServer::new();
        let result = server.list_configured_apis().await;
        
        assert!(result.get("configured_apis").is_some());
        assert_eq!(result.get("total_count").unwrap().as_u64().unwrap(), 3);
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(60); // 60 per minute = 1 per second
        
        // Should be able to acquire initially
        assert!(limiter.try_acquire());
        
        // Should be limited after exhausting tokens
        for _ in 0..59 {
            limiter.try_acquire();
        }
        assert!(!limiter.try_acquire());
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let breaker = CircuitBreaker::new();
        
        // Should be closed initially
        assert!(breaker.can_proceed());
        
        // Record failures to open circuit
        for _ in 0..5 {
            breaker.record_failure();
        }
        
        // Should be open now
        assert!(!breaker.can_proceed());
    }

    #[tokio::test]
    async fn test_server_status() {
        let server = ApiWrapperServer::new();
        let status = server.get_server_status_info();
        
        assert_eq!(status.get("status").unwrap().as_str().unwrap(), "running");
        assert!(status.get("uptime_seconds").is_some());
        assert_eq!(status.get("configured_apis").unwrap().as_u64().unwrap(), 3);
    }
}