# Authentication (JWT)

Simple, effective JWT authentication for HTTP transport.

**Last Updated:** 2026-01-09 HCMC  
**Feature Flag:** `auth`  
**Requires:** `http` feature (automatically included)

## Quick Start

### 1. Build with Auth

```bash
cargo build --release --features "http,auth"
```

### 2. Set Environment Variables

```bash
export JWT_SECRET="your-secret-key-min-32-chars-recommended"
export JWT_EXPIRY_SECONDS=86400  # Optional, default 24 hours
```

### 3. Run Server

```bash
./target/release/mcp-boilerplate-rust --mode http
```

## Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/auth/login` | No | Get JWT token |
| GET | `/auth/verify` | No | Verify token validity |
| GET | `/auth/me` | Yes | Get current user info |
| GET | `/protected/tools` | Yes | Protected tools list |

## Usage

### Login

```bash
curl -X POST http://127.0.0.1:8025/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}'
```

Response:
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

### Verify Token

```bash
curl http://127.0.0.1:8025/auth/verify \
  -H "Authorization: Bearer <token>"
```

Response:
```json
{
  "valid": true,
  "claims": {
    "sub": "admin",
    "exp": 1768024810,
    "iat": 1767938410,
    "uid": "user_admin",
    "role": "admin"
  }
}
```

### Get Current User

```bash
curl http://127.0.0.1:8025/auth/me \
  -H "x-access-token: <token>"
```

Response:
```json
{
  "user": "admin",
  "user_id": "user_admin",
  "role": "admin"
}
```

### Access Protected Endpoint

```bash
# Without token - 401
curl http://127.0.0.1:8025/protected/tools
# {"success":false,"error":"Missing token..."}

# With token - 200
curl http://127.0.0.1:8025/protected/tools \
  -H "Authorization: Bearer <token>"
```

## Token Headers

Two methods supported:

1. **Authorization Header** (standard)
   ```
   Authorization: Bearer <token>
   ```

2. **Custom Header** (alternative)
   ```
   x-access-token: <token>
   ```

## JWT Claims

```rust
pub struct Claims {
    pub sub: String,           // Subject (username)
    pub exp: usize,            // Expiration (unix timestamp)
    pub iat: usize,            // Issued at (unix timestamp)
    pub user_id: Option<String>, // Custom: user object id
    pub role: Option<String>,    // Custom: role
}
```

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `JWT_SECRET` | Yes | - | Secret key for signing tokens |
| `JWT_EXPIRY_SECONDS` | No | 86400 | Token expiration (seconds) |
| `AUTH_USERNAME` | No | admin | Demo username |
| `AUTH_PASSWORD_HASH` | No | (hash of admin123) | Password hash |
| `PASSWORD_SALT` | No | mcp_salt_2026 | Salt for password hashing |

## Security Notes

### Production Checklist

1. **Set JWT_SECRET** - Never use default, use 32+ character random string
2. **Use HTTPS** - Always use TLS in production
3. **Set proper expiry** - Don't use overly long token lifetimes
4. **Rotate secrets** - Periodically rotate JWT_SECRET
5. **Use proper password hashing** - Replace demo hash with argon2/bcrypt

### Generate Secure Secret

```bash
# Generate 64 character random secret
openssl rand -base64 48
```

### Demo vs Production

The current implementation uses a simple hash for demo purposes. For production:

1. Replace `Auth::hash_password()` with argon2 or bcrypt
2. Store users in database (enable `database` feature)
3. Implement refresh tokens
4. Add rate limiting

## Code Examples

### Protect a Route

```rust
use axum::{Router, routing::get, middleware};
use crate::middleware::auth_middleware;

let app = Router::new()
    .route("/api/data", get(handler))
    .layer(middleware::from_fn(auth_middleware));
```

### Optional Auth

```rust
use crate::middleware::optional_auth_middleware;

let app = Router::new()
    .route("/api/public", get(handler))
    .layer(middleware::from_fn(optional_auth_middleware));
```

### Extract Claims in Handler

```rust
use axum::extract::Extension;
use crate::middleware::Claims;

async fn handler(Extension(claims): Extension<Claims>) -> String {
    format!("Hello, {}!", claims.sub)
}
```

### Generate Token Programmatically

```rust
use crate::middleware::Auth;

let token = Auth::generate_token(
    "user123",           // subject
    Some("uid_123".into()), // user_id
    Some("admin".into()),   // role
)?;

println!("Token: {}", token.token);
```

### Verify Token

```rust
use crate::middleware::Auth;

match Auth::verify_token(&token_string) {
    Ok(claims) => println!("Valid: user={}", claims.sub),
    Err(e) => println!("Invalid: {}", e.message),
}
```

## Error Responses

### 400 Bad Request
```json
{"success": false, "error": "Username and password required"}
```

### 401 Unauthorized
```json
{"success": false, "error": "Invalid credentials"}
{"success": false, "error": "Missing token. Use x-access-token header or Authorization: Bearer <token>"}
{"success": false, "error": "Invalid token: ExpiredSignature"}
```

### 500 Internal Server Error
```json
{"success": false, "error": "JWT_SECRET not configured. Set JWT_SECRET env var."}
```

## Testing

```bash
# Run auth tests
cargo test --features "http,auth" middleware::auth

# Output
test middleware::auth::tests::test_password_hash ... ok
test middleware::auth::tests::test_token_generation_requires_secret ... ok
test middleware::auth::tests::test_token_roundtrip ... ok
```

## File Structure

```
src/middleware/
├── mod.rs          # Module exports
└── auth.rs         # Auth implementation (395 lines)
    ├── Types       # Claims, LoginRequest, TokenResponse, AuthError
    ├── Config      # get_jwt_secret(), get_token_expiry()
    ├── Auth        # generate_token(), verify_token(), extract_token()
    ├── Middleware  # auth_middleware(), optional_auth_middleware()
    ├── Handlers    # login_handler(), verify_handler(), me_handler()
    └── Router      # auth_router()
```

## Integration with Other Transports

Auth is designed for HTTP transport. For other transports:

- **Stdio:** Not applicable (local only)
- **SSE/WebSocket:** Pass token in connection query string
- **gRPC:** Use metadata headers

Example for WebSocket:
```javascript
const ws = new WebSocket('ws://localhost:9001/ws?token=' + token);
```

## Changelog

### v0.5.1 (2026-01-09)
- Initial auth implementation
- JWT token generation and verification
- Auth middleware (required/optional)
- Demo login endpoint
- Protected route support