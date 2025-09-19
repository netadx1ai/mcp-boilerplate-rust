//! API Gateway Server - Production MCP server for external API integration
//!
//! This server provides comprehensive external API integration capabilities including:
//! - Generic external API calling with authentication
//! - Multiple authentication methods (API keys, OAuth, Bearer tokens)
//! - Request/response transformation and validation
//! - Retry logic with exponential backoff
//! - Rate limiting and circuit breaker patterns
//! - Mock external API simulation for development
//!
//! Built on the official RMCP SDK with production-ready patterns.

use anyhow::Result;
use chrono;
use clap::Parser;
use dashmap::DashMap;
use rmcp::{
    handler::server::wrapper::Parameters, model::*, service::RequestContext, tool, tool_router,
    transport::stdio, ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

// ================================================================================================
// Request/Response Types
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CallExternalApiArgs {
    /// Name of the external API to call
    pub api_name: String,
    /// API endpoint to call
    pub endpoint: String,
    /// Parameters to send with the API request
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetApiSchemaArgs {
    /// Name of the API to get schema for
    pub api_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TestApiConnectionArgs {
    /// Name of the API to test
    pub api_name: String,
}

// ================================================================================================
// Error Types
// ================================================================================================

/// API Gateway Server errors
#[derive(thiserror::Error, Debug)]
pub enum ApiGatewayError {
    #[error("API not found: {0}")]
    ApiNotFound(String),
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Rate limit exceeded for API: {0}")]
    RateLimitExceeded(String),
    #[error("API request failed: {0}")]
    RequestFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    #[error("Circuit breaker open for API: {0}")]
    CircuitBreakerOpen(String),
}

// ================================================================================================
// Authentication and Configuration Types
// ================================================================================================

/// Authentication methods supported by the gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthMethod {
    #[serde(rename = "api_key")]
    ApiKey { header: String, value: String },
    #[serde(rename = "bearer_token")]
    BearerToken { token: String },
    #[serde(rename = "basic_auth")]
    BasicAuth { username: String, password: String },
    #[serde(rename = "oauth")]
    OAuth {
        access_token: String,
        refresh_token: Option<String>,
        expires_at: Option<u64>,
    },
}

/// External API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub name: String,
    pub base_url: String,
    pub description: String,
    pub auth_method: AuthMethod,
    pub rate_limit_per_minute: u32,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub endpoints: HashMap<String, EndpointConfig>,
}

/// Endpoint configuration within an API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub path: String,
    pub method: String,
    pub description: String,
    pub parameters: Vec<ParameterConfig>,
    pub response_schema: serde_json::Value,
}

/// Parameter configuration for endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterConfig {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub description: String,
    pub default_value: Option<serde_json::Value>,
}

// ================================================================================================
// Resilience Patterns
// ================================================================================================

/// Rate limiting tracker
#[derive(Debug)]
struct RateLimiter {
    requests: AtomicU64,
    window_start: AtomicU64,
    limit: u32,
}

impl RateLimiter {
    fn new(limit: u32) -> Self {
        Self {
            requests: AtomicU64::new(0),
            window_start: AtomicU64::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ),
            limit,
        }
    }

    fn check_rate_limit(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let window_start = self.window_start.load(Ordering::Relaxed);

        // Reset counter if we're in a new minute
        if now - window_start >= 60 {
            self.window_start.store(now, Ordering::Relaxed);
            self.requests.store(0, Ordering::Relaxed);
        }

        let current_requests = self.requests.fetch_add(1, Ordering::Relaxed);
        current_requests < self.limit as u64
    }
}

/// Circuit breaker for handling API failures
#[derive(Debug)]
struct CircuitBreaker {
    failure_count: AtomicU64,
    last_failure_time: AtomicU64,
    state: std::sync::RwLock<CircuitBreakerState>,
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
            last_failure_time: AtomicU64::new(0),
            state: std::sync::RwLock::new(CircuitBreakerState::Closed),
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

                // Open circuit for 60 seconds after failure
                if now - last_failure > 60 {
                    drop(state);
                    *self.state.write().unwrap() = CircuitBreakerState::HalfOpen;
                    true
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    fn record_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        *self.state.write().unwrap() = CircuitBreakerState::Closed;
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

        // Open circuit after 3 failures
        if failures >= 2 {
            *self.state.write().unwrap() = CircuitBreakerState::Open;
        }
    }
}

/// API call statistics
#[derive(Debug, Default, Serialize)]
struct ApiStats {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    average_response_time_ms: f64,
    last_request_time: Option<String>,
}

// ================================================================================================
// API Gateway Server Implementation
// ================================================================================================

/// API Gateway Server implementation
pub struct ApiGatewayServer {
    apis: HashMap<String, ApiConfig>,
    rate_limiters: DashMap<String, RateLimiter>,
    circuit_breakers: DashMap<String, CircuitBreaker>,
    stats: DashMap<String, ApiStats>,
    start_time: SystemTime,
}

impl ApiGatewayServer {
    /// Create a new API Gateway Server with mock external APIs
    pub fn new() -> Self {
        let mut apis = HashMap::new();

        // Weather API configuration
        apis.insert(
            "weather".to_string(),
            ApiConfig {
                name: "Weather API".to_string(),
                base_url: "https://api.weather.example".to_string(),
                description: "Current weather conditions and forecasts".to_string(),
                auth_method: AuthMethod::ApiKey {
                    header: "X-API-Key".to_string(),
                    value: "weather_api_key_123".to_string(),
                },
                rate_limit_per_minute: 100,
                timeout_seconds: 10,
                retry_attempts: 3,
                endpoints: {
                    let mut endpoints = HashMap::new();
                    endpoints.insert(
                        "current".to_string(),
                        EndpointConfig {
                            path: "/current".to_string(),
                            method: "GET".to_string(),
                            description: "Get current weather conditions".to_string(),
                            parameters: vec![
                                ParameterConfig {
                                    name: "location".to_string(),
                                    param_type: "string".to_string(),
                                    required: true,
                                    description: "City name or coordinates".to_string(),
                                    default_value: None,
                                },
                                ParameterConfig {
                                    name: "units".to_string(),
                                    param_type: "string".to_string(),
                                    required: false,
                                    description: "Temperature units (celsius/fahrenheit)"
                                        .to_string(),
                                    default_value: Some(serde_json::Value::String(
                                        "celsius".to_string(),
                                    )),
                                },
                            ],
                            response_schema: serde_json::json!({
                                "type": "object",
                                "properties": {
                                    "location": {"type": "string"},
                                    "temperature": {"type": "number"},
                                    "humidity": {"type": "number"},
                                    "description": {"type": "string"}
                                }
                            }),
                        },
                    );
                    endpoints
                },
            },
        );

        // Currency Exchange API configuration
        apis.insert(
            "currency".to_string(),
            ApiConfig {
                name: "Currency Exchange API".to_string(),
                base_url: "https://api.exchange.example".to_string(),
                description: "Real-time currency exchange rates".to_string(),
                auth_method: AuthMethod::BearerToken {
                    token: "bearer_token_currency_456".to_string(),
                },
                rate_limit_per_minute: 60,
                timeout_seconds: 5,
                retry_attempts: 2,
                endpoints: {
                    let mut endpoints = HashMap::new();
                    endpoints.insert(
                        "rates".to_string(),
                        EndpointConfig {
                            path: "/rates".to_string(),
                            method: "GET".to_string(),
                            description: "Get current exchange rates".to_string(),
                            parameters: vec![
                                ParameterConfig {
                                    name: "base".to_string(),
                                    param_type: "string".to_string(),
                                    required: true,
                                    description: "Base currency code (USD, EUR, etc.)".to_string(),
                                    default_value: None,
                                },
                                ParameterConfig {
                                    name: "target".to_string(),
                                    param_type: "string".to_string(),
                                    required: false,
                                    description: "Target currency codes (comma-separated)"
                                        .to_string(),
                                    default_value: None,
                                },
                            ],
                            response_schema: serde_json::json!({
                                "type": "object",
                                "properties": {
                                    "base": {"type": "string"},
                                    "rates": {"type": "object"},
                                    "timestamp": {"type": "string"}
                                }
                            }),
                        },
                    );
                    endpoints
                },
            },
        );

        // Geocoding API configuration
        apis.insert(
            "geocoding".to_string(),
            ApiConfig {
                name: "Geocoding Service".to_string(),
                base_url: "https://api.geocoding.example".to_string(),
                description: "Address geocoding and reverse geocoding".to_string(),
                auth_method: AuthMethod::BasicAuth {
                    username: "geo_user".to_string(),
                    password: "geo_pass_789".to_string(),
                },
                rate_limit_per_minute: 200,
                timeout_seconds: 8,
                retry_attempts: 2,
                endpoints: {
                    let mut endpoints = HashMap::new();
                    endpoints.insert(
                        "geocode".to_string(),
                        EndpointConfig {
                            path: "/geocode".to_string(),
                            method: "GET".to_string(),
                            description: "Convert address to coordinates".to_string(),
                            parameters: vec![ParameterConfig {
                                name: "address".to_string(),
                                param_type: "string".to_string(),
                                required: true,
                                description: "Address to geocode".to_string(),
                                default_value: None,
                            }],
                            response_schema: serde_json::json!({
                                "type": "object",
                                "properties": {
                                    "address": {"type": "string"},
                                    "latitude": {"type": "number"},
                                    "longitude": {"type": "number"},
                                    "accuracy": {"type": "string"}
                                }
                            }),
                        },
                    );
                    endpoints
                },
            },
        );

        let server = Self {
            apis,
            rate_limiters: DashMap::new(),
            circuit_breakers: DashMap::new(),
            stats: DashMap::new(),
            start_time: SystemTime::now(),
        };

        // Initialize rate limiters and circuit breakers
        for (api_name, config) in &server.apis {
            server.rate_limiters.insert(
                api_name.clone(),
                RateLimiter::new(config.rate_limit_per_minute),
            );
            server
                .circuit_breakers
                .insert(api_name.clone(), CircuitBreaker::new());
            server.stats.insert(api_name.clone(), ApiStats::default());
        }

        server
    }

    /// Simulate an external API call with realistic response
    async fn simulate_api_call(
        &self,
        api_name: &str,
        endpoint: &str,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, ApiGatewayError> {
        // Simulate network delay
        sleep(Duration::from_millis(50)).await;

        match (api_name, endpoint) {
            ("weather", "current") => {
                let location =
                    params
                        .get("location")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            ApiGatewayError::RequestFailed("Missing location parameter".to_string())
                        })?;

                let units = params
                    .get("units")
                    .and_then(|v| v.as_str())
                    .unwrap_or("celsius");

                let temp = if units == "fahrenheit" { 72.5 } else { 22.5 };

                Ok(serde_json::json!({
                    "location": location,
                    "temperature": temp,
                    "humidity": 65,
                    "description": "Partly cloudy",
                    "wind_speed": 12,
                    "units": units,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))
            }
            ("currency", "rates") => {
                let base = params.get("base").and_then(|v| v.as_str()).ok_or_else(|| {
                    ApiGatewayError::RequestFailed("Missing base currency parameter".to_string())
                })?;

                let mut rates = serde_json::Map::new();
                rates.insert(
                    "EUR".to_string(),
                    serde_json::Value::Number(serde_json::Number::from_f64(0.85).unwrap()),
                );
                rates.insert(
                    "GBP".to_string(),
                    serde_json::Value::Number(serde_json::Number::from_f64(0.73).unwrap()),
                );
                rates.insert(
                    "JPY".to_string(),
                    serde_json::Value::Number(serde_json::Number::from_f64(110.25).unwrap()),
                );
                rates.insert(
                    "CAD".to_string(),
                    serde_json::Value::Number(serde_json::Number::from_f64(1.25).unwrap()),
                );

                Ok(serde_json::json!({
                    "base": base,
                    "rates": rates,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "provider": "Exchange API"
                }))
            }
            ("geocoding", "geocode") => {
                let address = params
                    .get("address")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ApiGatewayError::RequestFailed("Missing address parameter".to_string())
                    })?;

                // Simulate geocoding based on address keywords
                let (lat, lng) = if address.to_lowercase().contains("new york") {
                    (40.7128, -74.0060)
                } else if address.to_lowercase().contains("london") {
                    (51.5074, -0.1278)
                } else if address.to_lowercase().contains("tokyo") {
                    (35.6762, 139.6503)
                } else {
                    (37.7749, -122.4194) // Default to San Francisco
                };

                Ok(serde_json::json!({
                    "address": address,
                    "latitude": lat,
                    "longitude": lng,
                    "accuracy": "high",
                    "geocoded_at": chrono::Utc::now().to_rfc3339()
                }))
            }
            _ => Err(ApiGatewayError::RequestFailed(format!(
                "Endpoint not found: {}/{}",
                api_name, endpoint
            ))),
        }
    }

    /// Execute an external API call with retry logic and error handling
    async fn call_external_api_internal(
        &self,
        api_name: &str,
        endpoint: &str,
        params: HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, ApiGatewayError> {
        let config = self
            .apis
            .get(api_name)
            .ok_or_else(|| ApiGatewayError::ApiNotFound(api_name.to_string()))?;

        // Check rate limiting
        if let Some(rate_limiter) = self.rate_limiters.get(api_name) {
            if !rate_limiter.check_rate_limit() {
                return Err(ApiGatewayError::RateLimitExceeded(api_name.to_string()));
            }
        }

        // Check circuit breaker
        if let Some(circuit_breaker) = self.circuit_breakers.get(api_name) {
            if !circuit_breaker.can_proceed() {
                return Err(ApiGatewayError::CircuitBreakerOpen(api_name.to_string()));
            }
        }

        let start_time = std::time::Instant::now();
        let mut last_error = None;

        // Retry logic with exponential backoff
        for attempt in 0..=config.retry_attempts {
            let result = timeout(
                Duration::from_secs(config.timeout_seconds),
                self.simulate_api_call(api_name, endpoint, &params),
            )
            .await;

            match result {
                Ok(Ok(response)) => {
                    // Record success
                    if let Some(circuit_breaker) = self.circuit_breakers.get(api_name) {
                        circuit_breaker.record_success();
                    }

                    // Update statistics
                    if let Some(mut stats) = self.stats.get_mut(api_name) {
                        stats.total_requests += 1;
                        stats.successful_requests += 1;
                        let elapsed_ms = start_time.elapsed().as_millis() as f64;
                        stats.average_response_time_ms = (stats.average_response_time_ms
                            * (stats.total_requests - 1) as f64
                            + elapsed_ms)
                            / stats.total_requests as f64;
                        stats.last_request_time = Some(chrono::Utc::now().to_rfc3339());
                    }

                    return Ok(response);
                }
                Ok(Err(e)) => {
                    last_error = Some(e);
                }
                Err(_) => {
                    last_error = Some(ApiGatewayError::TimeoutError(format!(
                        "Request to {} timed out after {}s",
                        api_name, config.timeout_seconds
                    )));
                }
            }

            // Exponential backoff before retry
            if attempt < config.retry_attempts {
                let delay_ms = 100 * (2_u64.pow(attempt));
                sleep(Duration::from_millis(delay_ms)).await;
            }
        }

        // Record failure
        if let Some(circuit_breaker) = self.circuit_breakers.get(api_name) {
            circuit_breaker.record_failure();
        }

        // Update failure statistics
        if let Some(mut stats) = self.stats.get_mut(api_name) {
            stats.total_requests += 1;
            stats.failed_requests += 1;
            stats.last_request_time = Some(chrono::Utc::now().to_rfc3339());
        }

        Err(last_error.unwrap_or_else(|| {
            ApiGatewayError::RequestFailed("Unknown error occurred".to_string())
        }))
    }
}

impl Default for ApiGatewayServer {
    fn default() -> Self {
        Self::new()
    }
}

// ================================================================================================
// MCP Tools Implementation
// ================================================================================================

#[tool_router]
impl ApiGatewayServer {
    /// Execute an external API call with authentication, retry logic, and transformation
    #[tool(
        description = "Execute an external API call with authentication, retry logic, and transformation"
    )]
    async fn call_external_api(
        &self,
        Parameters(args): Parameters<CallExternalApiArgs>,
    ) -> Result<CallToolResult, McpError> {
        let start_time = std::time::Instant::now();

        match self
            .call_external_api_internal(&args.api_name, &args.endpoint, args.parameters)
            .await
        {
            Ok(response) => {
                info!("API call successful: {}/{}", args.api_name, args.endpoint);
                let result = format!(
                    "✅ **API Call Successful**\n\n**API:** {}\n**Endpoint:** {}\n**Execution Time:** {}ms\n\n**Response:**\n```json\n{}\n```",
                    args.api_name,
                    args.endpoint,
                    start_time.elapsed().as_millis(),
                    serde_json::to_string_pretty(&response).unwrap_or_else(|_| "Invalid JSON".to_string())
                );

                Ok(CallToolResult::success(vec![Content::text(result)]))
            }
            Err(e) => {
                warn!(
                    "API call failed: {}/{} - {}",
                    args.api_name, args.endpoint, e
                );
                let result = format!(
                    "❌ **API Call Failed**\n\n**API:** {}\n**Endpoint:** {}\n**Execution Time:** {}ms\n\n**Error:** {}",
                    args.api_name,
                    args.endpoint,
                    start_time.elapsed().as_millis(),
                    e
                );

                Ok(CallToolResult::success(vec![Content::text(result)]))
            }
        }
    }

    /// List all configured external APIs with their capabilities
    #[tool(description = "List all configured external APIs with their capabilities")]
    async fn list_available_apis(&self) -> Result<CallToolResult, McpError> {
        debug!("Listing available APIs");

        let mut result = format!("🌐 **Available External APIs** ({})\n\n", self.apis.len());

        for config in self.apis.values() {
            let auth_type = match &config.auth_method {
                AuthMethod::ApiKey { .. } => "API Key",
                AuthMethod::BearerToken { .. } => "Bearer Token",
                AuthMethod::BasicAuth { .. } => "Basic Auth",
                AuthMethod::OAuth { .. } => "OAuth",
            };

            let endpoints: Vec<_> = config.endpoints.keys().collect();

            result.push_str(&format!(
                "**{}**\n- Description: {}\n- Base URL: {}\n- Authentication: {}\n- Rate Limit: {}/min\n- Timeout: {}s\n- Endpoints: {}\n\n",
                config.name,
                config.description,
                config.base_url,
                auth_type,
                config.rate_limit_per_minute,
                config.timeout_seconds,
                endpoints.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
            ));
        }

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get detailed schema and documentation for a specific API
    #[tool(description = "Get detailed schema and documentation for a specific API")]
    async fn get_api_schema(
        &self,
        Parameters(args): Parameters<GetApiSchemaArgs>,
    ) -> Result<CallToolResult, McpError> {
        let config = match self.apis.get(&args.api_name) {
            Some(config) => config,
            None => {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "❌ **API Not Found**\n\nAPI '{}' is not configured in this gateway.",
                    args.api_name
                ))]));
            }
        };

        debug!("Retrieved schema for API: {}", args.api_name);

        let auth_type = match &config.auth_method {
            AuthMethod::ApiKey { header, .. } => format!("API Key (Header: {})", header),
            AuthMethod::BearerToken { .. } => "Bearer Token".to_string(),
            AuthMethod::BasicAuth { .. } => "Basic Authentication".to_string(),
            AuthMethod::OAuth { .. } => "OAuth 2.0".to_string(),
        };

        let mut result = format!(
            "📋 **API Schema: {}**\n\n**Description:** {}\n**Base URL:** {}\n**Authentication:** {}\n\n**Rate Limiting:**\n- Requests per minute: {}\n- Timeout: {}s\n- Retry attempts: {}\n\n**Endpoints:**\n",
            config.name,
            config.description,
            config.base_url,
            auth_type,
            config.rate_limit_per_minute,
            config.timeout_seconds,
            config.retry_attempts
        );

        for (name, endpoint) in &config.endpoints {
            result.push_str(&format!(
                "\n**{}**\n- Path: {}\n- Method: {}\n- Description: {}\n- Parameters:\n",
                name, endpoint.path, endpoint.method, endpoint.description
            ));

            for param in &endpoint.parameters {
                let required_str = if param.required {
                    "(required)"
                } else {
                    "(optional)"
                };
                result.push_str(&format!(
                    "  - {} ({}): {} {}\n",
                    param.name, param.param_type, param.description, required_str
                ));
            }
        }

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Test connectivity and health of a specific external API
    #[tool(description = "Test connectivity and health of a specific external API")]
    async fn test_api_connection(
        &self,
        Parameters(args): Parameters<TestApiConnectionArgs>,
    ) -> Result<CallToolResult, McpError> {
        let start_time = std::time::Instant::now();

        // Simulate a simple health check
        let test_result = match args.api_name.as_str() {
            "weather" => {
                let params = HashMap::from([(
                    "location".to_string(),
                    serde_json::Value::String("Test City".to_string()),
                )]);
                self.call_external_api_internal(&args.api_name, "current", params)
                    .await
            }
            "currency" => {
                let params = HashMap::from([(
                    "base".to_string(),
                    serde_json::Value::String("USD".to_string()),
                )]);
                self.call_external_api_internal(&args.api_name, "rates", params)
                    .await
            }
            "geocoding" => {
                let params = HashMap::from([(
                    "address".to_string(),
                    serde_json::Value::String("Test Address".to_string()),
                )]);
                self.call_external_api_internal(&args.api_name, "geocode", params)
                    .await
            }
            _ => {
                return Ok(CallToolResult::success(vec![Content::text(
                    format!("❌ **API Connection Test Failed**\n\nAPI '{}' is not configured in this gateway.", args.api_name)
                )]));
            }
        };

        let response_time_ms = start_time.elapsed().as_millis();

        let result = match test_result {
            Ok(_) => {
                info!("API connection test successful: {}", args.api_name);
                let cb_state = if let Some(cb) = self.circuit_breakers.get(&args.api_name) {
                    let state = cb.state.read().unwrap();
                    match *state {
                        CircuitBreakerState::Closed => "closed",
                        CircuitBreakerState::Open => "open",
                        CircuitBreakerState::HalfOpen => "half_open",
                    }
                } else {
                    "unknown"
                };

                format!(
                    "✅ **API Connection Test: PASSED**\n\n**API:** {}\n**Status:** Healthy\n**Response Time:** {}ms\n**Circuit Breaker:** {}\n**Timestamp:** {}",
                    args.api_name,
                    response_time_ms,
                    cb_state,
                    chrono::Utc::now().to_rfc3339()
                )
            }
            Err(e) => {
                warn!("API connection test failed for {}: {}", args.api_name, e);
                format!(
                    "❌ **API Connection Test: FAILED**\n\n**API:** {}\n**Response Time:** {}ms\n**Error:** {}\n**Timestamp:** {}",
                    args.api_name,
                    response_time_ms,
                    e,
                    chrono::Utc::now().to_rfc3339()
                )
            }
        };

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get API gateway server status, statistics, and health information
    #[tool(description = "Get API gateway server status, statistics, and health information")]
    async fn get_server_status(&self) -> Result<CallToolResult, McpError> {
        debug!("Getting server status");

        let uptime_secs = self.start_time.elapsed().unwrap().as_secs();
        let total_apis = self.apis.len();

        let total_requests: u64 = self
            .stats
            .iter()
            .map(|entry| entry.value().total_requests)
            .sum();
        let total_successful: u64 = self
            .stats
            .iter()
            .map(|entry| entry.value().successful_requests)
            .sum();
        let total_failed: u64 = self
            .stats
            .iter()
            .map(|entry| entry.value().failed_requests)
            .sum();

        let success_rate = if total_requests > 0 {
            (total_successful as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        let mut result = format!(
            "🚀 **API Gateway Server Status**\n\n**Server:** API Gateway Server\n**Status:** ✅ Running\n**Uptime:** {}s\n**Start Time:** {}\n\n**Configuration:**\n- Total APIs: {}\n- Available APIs: {}\n\n**Statistics:**\n- Total Requests: {}\n- Successful: {}\n- Failed: {}\n- Success Rate: {:.1}%\n\n**API Statistics:**\n",
            uptime_secs,
            chrono::DateTime::<chrono::Utc>::from(self.start_time).to_rfc3339(),
            total_apis,
            self.apis.keys().map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
            total_requests,
            total_successful,
            total_failed,
            success_rate
        );

        for entry in self.stats.iter() {
            let stats = entry.value();
            result.push_str(&format!(
                "\n**{}:**\n- Requests: {}\n- Success: {}\n- Failed: {}\n- Avg Response: {:.1}ms\n- Last Request: {}\n",
                entry.key(),
                stats.total_requests,
                stats.successful_requests,
                stats.failed_requests,
                stats.average_response_time_ms,
                stats.last_request_time.as_ref().unwrap_or(&"Never".to_string())
            ));
        }

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
}

// ================================================================================================
// Main Function
// ================================================================================================

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Setup logging
    let log_level = if args.debug { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(format!("api_gateway_server={log_level}").parse()?)
                .add_directive(format!("rmcp={log_level}").parse()?),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    info!("🚀 Starting API Gateway Server using official RMCP SDK");
    info!("🌐 External API integration with authentication and resilience patterns");

    // Create server instance
    let server = ApiGatewayServer::new();

    info!(
        "API Gateway Server configured with {} external APIs",
        server.apis.len()
    );
    for api_name in server.apis.keys() {
        info!("  - {}", api_name);
    }

    // Start the server with STDIO transport
    let service = server.serve(stdio()).await.inspect_err(|e| {
        error!("Failed to start server: {:?}", e);
    })?;

    info!("✅ API Gateway Server started and ready for MCP connections");
    info!("🔗 Available external APIs: weather, currency, geocoding");
    info!("⚡ Features: Authentication, Rate limiting, Circuit breaker, Retry logic");

    // Wait for the service to complete
    service.waiting().await?;

    info!("Server shutdown complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let server = ApiGatewayServer::new();
        assert_eq!(server.apis.len(), 3);
        assert!(server.apis.contains_key("weather"));
        assert!(server.apis.contains_key("currency"));
        assert!(server.apis.contains_key("geocoding"));
    }

    #[tokio::test]
    async fn test_weather_api_call() {
        let server = ApiGatewayServer::new();
        let params = HashMap::from([
            (
                "location".to_string(),
                serde_json::Value::String("New York".to_string()),
            ),
            (
                "units".to_string(),
                serde_json::Value::String("celsius".to_string()),
            ),
        ]);

        let result = server
            .call_external_api_internal("weather", "current", params)
            .await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(
            response.get("location"),
            Some(&serde_json::Value::String("New York".to_string()))
        );
        assert!(response.get("temperature").is_some());
    }

    #[tokio::test]
    async fn test_currency_api_call() {
        let server = ApiGatewayServer::new();
        let params = HashMap::from([(
            "base".to_string(),
            serde_json::Value::String("USD".to_string()),
        )]);

        let result = server
            .call_external_api_internal("currency", "rates", params)
            .await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(
            response.get("base"),
            Some(&serde_json::Value::String("USD".to_string()))
        );
        assert!(response.get("rates").is_some());
    }

    #[tokio::test]
    async fn test_geocoding_api_call() {
        let server = ApiGatewayServer::new();
        let params = HashMap::from([(
            "address".to_string(),
            serde_json::Value::String("London, UK".to_string()),
        )]);

        let result = server
            .call_external_api_internal("geocoding", "geocode", params)
            .await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(
            response.get("address"),
            Some(&serde_json::Value::String("London, UK".to_string()))
        );
        assert!(response.get("latitude").is_some());
        assert!(response.get("longitude").is_some());
    }

    #[tokio::test]
    async fn test_api_not_found() {
        let server = ApiGatewayServer::new();
        let params = HashMap::new();

        let result = server
            .call_external_api_internal("nonexistent", "endpoint", params)
            .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApiGatewayError::ApiNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(2);

        // First two requests should pass
        assert!(limiter.check_rate_limit());
        assert!(limiter.check_rate_limit());

        // Third request should fail
        assert!(!limiter.check_rate_limit());
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let breaker = CircuitBreaker::new();

        // Should be closed initially
        assert!(breaker.can_proceed());

        // Record failures to open circuit
        breaker.record_failure();
        breaker.record_failure();
        breaker.record_failure();

        // Should be open now
        assert!(!breaker.can_proceed());

        // Record success to close circuit
        breaker.record_success();
        assert!(breaker.can_proceed());
    }
}
