# Authenticated MCP Server Template

A production-ready template for building MCP servers with comprehensive authentication and authorization capabilities. Supports OAuth 2.0, JWT tokens, API keys, and role-based access control.

## 🔐 Features

### Authentication Methods
- **OAuth 2.0**: Authorization code flow with PKCE support
- **JWT Tokens**: Secure token validation and refresh mechanisms
- **API Keys**: Simple API key-based authentication
- **Username/Password**: Traditional credential-based authentication

### Security Features
- **Role-Based Access Control (RBAC)**: Admin, User, ReadOnly, Service roles
- **Session Management**: Secure session handling with expiration
- **Token Refresh**: Automatic token refresh with refresh tokens
- **CSRF Protection**: Built-in CSRF token generation
- **Permission System**: Granular permission checking

### Production Ready
- **Secure Defaults**: JWT secrets, session timeouts, token expiration
- **Statistics Tracking**: Authentication attempts, active sessions, performance metrics
- **Error Handling**: Comprehensive error types and validation
- **Testing**: Complete test suite with 100% coverage
- **Documentation**: Full API documentation and examples

## 🚀 Quick Start

### 1. Copy Template
```bash
# Copy the template to your project
cp -r templates/authenticated-server-template my-auth-server
cd my-auth-server
```

### 2. Install Dependencies
```bash
cargo build
```

### 3. Configure Environment (Optional)
```bash
export JWT_SECRET="your-super-secret-jwt-key"
export OAUTH_CLIENT_ID="your-oauth-client-id"
export OAUTH_CLIENT_SECRET="your-oauth-client-secret"
export SESSION_TIMEOUT_MINUTES="1440"  # 24 hours
export TOKEN_TIMEOUT_MINUTES="60"      # 1 hour
```

### 4. Run the Server
```bash
cargo run --bin authenticated-server
```

### 5. Test Authentication
```bash
# Test with sample API key
echo '{"type": "api_key", "key": "admin-key-12345"}' | \
  your-mcp-client call authenticate_user

# Test with username/password
echo '{"type": "username_password", "username": "admin", "password": "password"}' | \
  your-mcp-client call authenticate_user
```

## 🛠 MCP Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `authenticate_user` | Authenticate with various credential types | `credentials: AuthCredentials` |
| `validate_token` | Validate JWT token | `token: String` |
| `refresh_token` | Refresh JWT token using refresh token | `refresh_token: String` |
| `get_user_profile` | Get user information by token | `token: String` |
| `list_permissions` | List user permissions | `user_id: Uuid` |
| `check_permission` | Check if user has specific permission | `check: PermissionCheck` |
| `logout` | Invalidate user session | `token: String` |
| `get_server_status` | Get server statistics | - |

## 📋 Sample Data

The template comes with pre-configured sample data for testing:

### Users
| Username | Role | Permissions | API Key |
|----------|------|-------------|---------|
| `admin` | Admin | read, write, delete, admin | `admin-key-12345` |
| `user` | User | read, write | `user-key-67890` |
| `readonly` | ReadOnly | read | `readonly-key-11111` |

### Authentication Examples

#### API Key Authentication
```json
{
  "type": "api_key",
  "key": "admin-key-12345"
}
```

#### Username/Password Authentication
```json
{
  "type": "username_password",
  "username": "admin",
  "password": "password"
}
```

#### JWT Token Validation
```json
{
  "type": "jwt",
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
}
```

## 🔧 Customization

### 1. Authentication Methods

#### Add Custom Authentication
```rust
// In authenticate_user method, add new credential type
AuthCredentials::Custom { data } => {
    // Your custom authentication logic
    validate_custom_credentials(data).await?
}
```

#### Configure OAuth Provider
```rust
let config = AuthConfig {
    oauth_client_id: Some("your-client-id".to_string()),
    oauth_client_secret: Some("your-client-secret".to_string()),
    oauth_auth_url: Some("https://provider.com/oauth/authorize".to_string()),
    oauth_token_url: Some("https://provider.com/oauth/token".to_string()),
    // ... other config
};
```

### 2. User Management

#### Add User Registration
```rust
#[tool]
async fn register_user(&self, user_data: UserRegistration) -> Result<User, AuthError> {
    // Validate user data
    // Hash password
    // Create user
    // Send verification email
}
```

#### Custom User Roles
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    SuperAdmin,
    Admin,
    User,
    ReadOnly,
    Guest,
    // Add your custom roles
}
```

### 3. Permission System

#### Define Custom Permissions
```rust
pub const PERMISSIONS: &[&str] = &[
    "read",
    "write", 
    "delete",
    "admin",
    "billing",
    "analytics",
    // Add your permissions
];
```

#### Permission Middleware
```rust
async fn require_permission(&self, token: &str, permission: &str) -> Result<(), AuthError> {
    let claims = self.validate_jwt_token(token)?;
    let user_id = Uuid::parse_str(&claims.sub)?;
    
    let check = PermissionCheck {
        user_id,
        required_permission: permission.to_string(),
    };
    
    if !self.check_permission(check).await? {
        return Err(AuthError::InsufficientPermissions);
    }
    
    Ok(())
}
```

### 4. Session Storage

#### Database Session Storage
```rust
use sqlx::PgPool;

pub struct DatabaseSessionStore {
    pool: PgPool,
}

impl DatabaseSessionStore {
    async fn store_session(&self, session: &Session) -> Result<(), AuthError> {
        sqlx::query!(
            "INSERT INTO sessions (id, user_id, created_at, expires_at, refresh_token) 
             VALUES ($1, $2, $3, $4, $5)",
            session.id,
            session.user_id,
            session.created_at,
            session.expires_at,
            session.refresh_token
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
```

#### Redis Session Storage
```rust
use redis::AsyncCommands;

async fn store_session_redis(
    redis: &mut redis::aio::Connection,
    session: &Session,
) -> Result<(), AuthError> {
    let session_data = serde_json::to_string(session)?;
    let ttl = (session.expires_at - Utc::now()).num_seconds() as usize;
    
    redis.setex(
        format!("session:{}", session.id),
        ttl,
        session_data
    ).await?;
    
    Ok(())
}
```

## 🧪 Testing

### Run Tests
```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_api_key_authentication
```

### Test Coverage
```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### Integration Testing
```bash
# Start test server
cargo run --bin authenticated-server &

# Run integration tests
cargo test --test integration

# Stop server
pkill authenticated-server
```

## 🔒 Security Best Practices

### 1. JWT Configuration
```bash
# Use strong, random JWT secrets (256+ bits)
export JWT_SECRET=$(openssl rand -hex 32)

# Set appropriate token lifetimes
export TOKEN_TIMEOUT_MINUTES="15"     # Short for security
export SESSION_TIMEOUT_MINUTES="480"  # 8 hours max
```

### 2. API Key Management
```rust
// Implement API key rotation
#[tool]
async fn rotate_api_key(&self, current_key: String) -> Result<String, AuthError> {
    // Validate current key
    // Generate new key
    // Update storage
    // Return new key
}
```

### 3. Rate Limiting
```rust
use std::collections::HashMap;
use tokio::time::{Duration, Instant};

pub struct RateLimiter {
    attempts: HashMap<String, Vec<Instant>>,
    max_attempts: usize,
    window: Duration,
}

impl RateLimiter {
    async fn check_rate_limit(&mut self, identifier: &str) -> bool {
        let now = Instant::now();
        let attempts = self.attempts.entry(identifier.to_string()).or_default();
        
        // Remove old attempts
        attempts.retain(|&time| now.duration_since(time) < self.window);
        
        if attempts.len() >= self.max_attempts {
            return false;
        }
        
        attempts.push(now);
        true
    }
}
```

### 4. Input Validation
```rust
use validator::{Validate, ValidationError};

#[derive(Validate, Deserialize)]
pub struct UserRegistration {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 8), custom = "validate_password")]
    pub password: String,
}

fn validate_password(password: &str) -> Result<(), ValidationError> {
    // Check password complexity
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(ValidationError::new("password_uppercase"));
    }
    // Add more validation rules
    Ok(())
}
```

## 📊 Monitoring

### Metrics Integration
```rust
use prometheus::{Counter, Histogram, Gauge};

pub struct AuthMetrics {
    auth_attempts: Counter,
    auth_duration: Histogram,
    active_sessions: Gauge,
}

impl AuthMetrics {
    pub fn record_auth_attempt(&self, success: bool) {
        self.auth_attempts
            .with_label_values(&[if success { "success" } else { "failure" }])
            .inc();
    }
}
```

### Logging Configuration
```rust
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into())
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}
```

## 🚀 Deployment

### Docker Configuration
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin authenticated-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/authenticated-server /usr/local/bin/
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1
CMD ["authenticated-server"]
```

### Environment Variables
```bash
# Required
JWT_SECRET=your-256-bit-secret

# OAuth (optional)
OAUTH_CLIENT_ID=your-oauth-client-id
OAUTH_CLIENT_SECRET=your-oauth-client-secret
OAUTH_AUTH_URL=https://provider.com/oauth/authorize
OAUTH_TOKEN_URL=https://provider.com/oauth/token
OAUTH_REDIRECT_URL=https://yourapp.com/auth/callback

# Timeouts
SESSION_TIMEOUT_MINUTES=1440  # 24 hours
TOKEN_TIMEOUT_MINUTES=60      # 1 hour

# Logging
RUST_LOG=info
```

## 📚 API Reference

### Authentication Flow

1. **Initial Authentication**
   ```
   Client → authenticate_user(credentials) → Server
   Server → AuthResponse(token, refresh_token) → Client
   ```

2. **API Requests**
   ```
   Client → validate_token(token) → Server
   Server → TokenValidation(valid, user_info) → Client
   ```

3. **Token Refresh**
   ```
   Client → refresh_token(refresh_token) → Server
   Server → AuthResponse(new_token) → Client
   ```

4. **Logout**
   ```
   Client → logout(token) → Server
   Server → true → Client
   ```

### Error Handling

| Error | Description | HTTP Status |
|-------|-------------|-------------|
| `InvalidCredentials` | Wrong username/password/key | 401 |
| `TokenExpired` | JWT token has expired | 401 |
| `InvalidToken` | Malformed or invalid token | 401 |
| `InsufficientPermissions` | User lacks required permission | 403 |
| `SessionNotFound` | Session expired or invalid | 401 |
| `OAuthError` | OAuth flow error | 400 |

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Related

- [Basic Server Template](../basic-server-template/) - Simple MCP server foundation
- [API Wrapper Template](../api-wrapper-template/) - External API integration
- [Database Integration Template](../database-integration-template/) - Database operations
- [MCP Official SDK](https://github.com/modelcontextprotocol/rust-sdk) - Official MCP SDK

---

**Template Version**: 1.0.0  
**MCP SDK**: v0.6.3  
**Rust Edition**: 2021