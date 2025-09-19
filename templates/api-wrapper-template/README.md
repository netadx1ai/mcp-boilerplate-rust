# API Wrapper Template

A production-ready MCP server template for wrapping external APIs with authentication, resilience patterns, and comprehensive error handling.

## 🚀 Features

- **Multiple Authentication Methods**: API Key, Bearer Token, Basic Auth, OAuth, Custom Headers
- **Resilience Patterns**: Rate limiting, circuit breaker, exponential backoff retry
- **HTTP Client Integration**: Built on reqwest with timeout and compression support
- **Request/Response Transformation**: Flexible data mapping and validation
- **Comprehensive Error Handling**: Structured error types with detailed context
- **Statistics & Monitoring**: Real-time metrics and performance tracking
- **Production Ready**: Logging, tracing, and observability built-in

## 📋 Quick Start

### 1. Copy the Template

```bash
# Copy this template to your project
cp -r templates/api-wrapper-template my-api-wrapper
cd my-api-wrapper

# Update Cargo.toml with your project details
vim Cargo.toml
```

### 2. Configure Your APIs

Edit the `create_default_apis()` function in `src/main.rs`:

```rust
fn create_default_apis() -> Vec<ApiConfig> {
    vec![
        ApiConfig {
            name: "my-api".to_string(),
            base_url: "https://api.myservice.com".to_string(),
            description: "My external API service".to_string(),
            auth_method: Some(AuthMethod::ApiKey {
                header: "X-API-Key".to_string(),
                value: std::env::var("MY_API_KEY").unwrap_or_default(),
            }),
            rate_limit_per_minute: 100,
            timeout_seconds: 10,
            retry_attempts: 3,
            default_headers: {
                let mut headers = HashMap::new();
                headers.insert("Accept".to_string(), "application/json".to_string());
                headers
            },
        },
        // Add more APIs as needed...
    ]
}
```

### 3. Set Environment Variables

```bash
# Set your API credentials
export MY_API_KEY="your-actual-api-key"
export RUST_LOG="info"
```

### 4. Build and Run

```bash
# Install dependencies and build
cargo build

# Run the server
cargo run --bin api-wrapper-server

# Or run with debug logging
RUST_LOG=debug cargo run --bin api-wrapper-server
```

## 🛠️ Customization Guide

### Adding New APIs

1. **Extend the configuration** in `create_default_apis()`:
```rust
ApiConfig {
    name: "weather-api".to_string(),
    base_url: "https://api.openweathermap.org/data/2.5".to_string(),
    description: "OpenWeather API for weather data".to_string(),
    auth_method: Some(AuthMethod::ApiKey {
        header: "appid".to_string(), // Query parameter
        value: std::env::var("OPENWEATHER_API_KEY").unwrap_or_default(),
    }),
    rate_limit_per_minute: 60,
    timeout_seconds: 5,
    retry_attempts: 2,
    default_headers: HashMap::new(),
}
```

2. **Add API-specific methods** (optional):
```rust
impl ApiWrapperServer {
    async fn get_weather(&self, city: &str) -> Result<serde_json::Value, ApiWrapperError> {
        self.call_api(
            "weather-api",
            "GET",
            "/weather",
            Some([("q".to_string(), serde_json::Value::String(city.to_string()))].into()),
            None,
            None,
        ).await
    }
}
```

### Authentication Methods

The template supports multiple authentication patterns:

```rust
// API Key in header
AuthMethod::ApiKey {
    header: "X-API-Key".to_string(),
    value: "your-key".to_string(),
}

// Bearer token
AuthMethod::BearerToken {
    token: "your-jwt-token".to_string(),
}

// Basic authentication
AuthMethod::BasicAuth {
    username: "user".to_string(),
    password: "pass".to_string(),
}

// OAuth with refresh token
AuthMethod::OAuth {
    access_token: "access-token".to_string(),
    refresh_token: Some("refresh-token".to_string()),
    expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
}

// Custom header
AuthMethod::CustomHeader {
    header_name: "Authorization".to_string(),
    header_value: "Custom token".to_string(),
}
```

### Rate Limiting Configuration

Adjust rate limits per API:

```rust
// Conservative rate limiting
rate_limit_per_minute: 60,  // 1 request per second

// Aggressive rate limiting
rate_limit_per_minute: 6000, // 100 requests per second

// Very conservative for expensive APIs
rate_limit_per_minute: 10,   // 1 request every 6 seconds
```

### Circuit Breaker Tuning

Modify circuit breaker thresholds in the `CircuitBreaker::new()` method:

```rust
impl CircuitBreaker {
    fn new() -> Self {
        Self {
            // ... other fields
            failure_threshold: 3,    // Open after 3 failures
            success_threshold: 5,    // Close after 5 successes in half-open
            timeout_seconds: 30,     // Stay open for 30 seconds
        }
    }
}
```

### Error Handling

Extend the `ApiWrapperError` enum for domain-specific errors:

```rust
#[derive(thiserror::Error, Debug)]
pub enum ApiWrapperError {
    // ... existing errors
    
    #[error("Weather data unavailable: {0}")]
    WeatherUnavailable(String),
    
    #[error("Invalid location: {0}")]
    InvalidLocation(String),
    
    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),
}
```

### Adding Custom Tools

Add new MCP tools by extending the `ServerHandler` implementation:

```rust
impl ServerHandler for ApiWrapperServer {
    async fn list_tools(&self) -> Result<Vec<Tool>, ServerError> {
        Ok(vec![
            // ... existing tools
            Tool {
                name: "get_weather".to_string(),
                description: "Get current weather for a city".to_string(),
                input_schema: schemars::schema_for!(WeatherArgs),
            },
        ])
    }

    async fn call_tool(&self, name: &str, input: ToolInput) -> Result<ToolResult, ServerError> {
        match name {
            // ... existing tools
            "get_weather" => {
                let args: WeatherArgs = serde_json::from_value(input.into())?;
                match self.get_weather(&args.city).await {
                    Ok(weather) => Ok(CallToolResult::success(weather)),
                    Err(e) => Ok(CallToolResult::error(&e.to_string())),
                }
            }
            _ => Err(ServerError::ToolNotFound(name.to_string())),
        }
    }
}
```

## 🔧 MCP Tools

The template provides these MCP tools out of the box:

### `call_api`
Make a direct API call to any configured service.

```json
{
  "name": "call_api",
  "arguments": {
    "api_name": "my-api",
    "method": "GET",
    "path": "/users/123",
    "params": {"include": "profile"},
    "headers": {"Accept": "application/json"}
  }
}
```

### `list_apis`
List all configured external APIs.

```json
{
  "name": "list_apis",
  "arguments": {}
}
```

### `get_api_stats`
Get statistics for API calls (all APIs or specific API).

```json
{
  "name": "get_api_stats", 
  "arguments": {
    "api_name": "my-api"
  }
}
```

### `get_server_status`
Get the current status of the API wrapper server.

```json
{
  "name": "get_server_status",
  "arguments": {}
}
```

## 📊 Monitoring & Statistics

The template tracks comprehensive statistics for each API:

- **Request counts**: Total, successful, failed
- **Response times**: Average response time in milliseconds
- **Rate limiting**: Number of rate limit hits
- **Circuit breaker state**: Current state (Closed/Open/HalfOpen)
- **Last request time**: Timestamp of the most recent request

Access these via the `get_api_stats` tool or implement custom monitoring endpoints.

## 🐛 Debugging

### Enable Debug Logging

```bash
RUST_LOG=debug cargo run --bin api-wrapper-server
```

### Common Issues

1. **Rate Limiting**: Adjust `rate_limit_per_minute` in API configuration
2. **Timeouts**: Increase `timeout_seconds` for slow APIs
3. **Circuit Breaker Open**: Check API health and reduce failure threshold
4. **Authentication Failures**: Verify API keys and auth method configuration

### Testing with Mock APIs

The template includes test support with `wiremock`. Add integration tests:

```rust
#[tokio::test]
async fn test_api_integration() {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "ok"})))
        .mount(&mock_server)
        .await;

    // Test your API wrapper against the mock server
}
```

## 🚀 Production Deployment

### Environment Variables

```bash
# API Credentials
export API_KEY_1="your-api-key"
export BEARER_TOKEN_2="your-bearer-token"

# Logging
export RUST_LOG="info"

# Optional: Custom configuration
export CONFIG_FILE="./config.json"
```

### Docker Support

Create a `Dockerfile`:

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin api-wrapper-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/api-wrapper-server /usr/local/bin/
EXPOSE 8080
CMD ["api-wrapper-server"]
```

### Health Checks

The server provides health information via the `get_server_status` tool. For HTTP health checks, consider adding a simple HTTP endpoint:

```rust
// Add to main.rs for HTTP health endpoint
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    // ... existing setup

    // Optional: HTTP health check endpoint
    tokio::spawn(async {
        let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
        // Implement simple health check HTTP server
    });

    // Start MCP server
    rmcp::run_server(server).await?;
    Ok(())
}
```

## 📚 Further Reading

- [MCP Specification](https://spec.modelcontextprotocol.io/)
- [rmcp Documentation](https://docs.rs/rmcp/)
- [reqwest Documentation](https://docs.rs/reqwest/)
- [Circuit Breaker Pattern](https://martinfowler.com/bliki/CircuitBreaker.html)
- [Rate Limiting Strategies](https://blog.cloudflare.com/counting-things-a-lot-of-different-things/)

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

- **Issues**: [GitHub Issues](https://github.com/your-org/your-repo/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/your-repo/discussions)
- **Documentation**: [Project Wiki](https://github.com/your-org/your-repo/wiki)

---

**Happy API Wrapping!** 🎉

This template provides a solid foundation for integrating external APIs into your MCP server with production-ready patterns. Customize it for your specific use case and deploy with confidence!