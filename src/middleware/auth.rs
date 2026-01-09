//! JWT Authentication Module
//! Simple, effective, hacker-style auth
//! 
//! Features:
//! - Token generation (login)
//! - Token verification
//! - Auth middleware (required/optional)
//! - Secure defaults
//!
//! Last Updated: 2026-01-09 HCMC



use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;

// ============================================================================
// TYPES
// ============================================================================

/// JWT Claims - keep it minimal
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (user id or username)
    pub sub: String,
    /// Expiration time (unix timestamp)
    pub exp: usize,
    /// Issued at (unix timestamp)
    pub iat: usize,
    /// Custom: user object id (optional)
    #[serde(rename = "uid", skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// Custom: role (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Token response
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: usize,
}

/// Auth error
#[derive(Debug)]
pub struct AuthError {
    pub code: StatusCode,
    pub message: String,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (
            self.code,
            Json(json!({
                "success": false,
                "error": self.message
            })),
        )
            .into_response()
    }
}

// ============================================================================
// CONFIG
// ============================================================================

/// Get JWT secret from env - MUST be set in production
fn get_jwt_secret() -> Result<String, AuthError> {
    std::env::var("JWT_SECRET").map_err(|_| AuthError {
        code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "JWT_SECRET not configured. Set JWT_SECRET env var.".to_string(),
    })
}

/// Token expiration in seconds (default: 24 hours)
fn get_token_expiry() -> usize {
    std::env::var("JWT_EXPIRY_SECONDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(86400) // 24 hours
}

// ============================================================================
// CORE AUTH FUNCTIONS
// ============================================================================

pub struct Auth;

impl Auth {
    /// Generate JWT token
    pub fn generate_token(
        subject: &str,
        user_id: Option<String>,
        role: Option<String>,
    ) -> Result<TokenResponse, AuthError> {
        let secret = get_jwt_secret()?;
        let expiry = get_token_expiry();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| AuthError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                message: "System time error".to_string(),
            })?
            .as_secs() as usize;

        let claims = Claims {
            sub: subject.to_string(),
            exp: now + expiry,
            iat: now,
            user_id,
            role,
        };

        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|e| AuthError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("Token generation failed: {}", e),
        })?;

        Ok(TokenResponse {
            token,
            token_type: "Bearer".to_string(),
            expires_in: expiry,
        })
    }

    /// Verify JWT token and extract claims
    pub fn verify_token(token: &str) -> Result<Claims, AuthError> {
        let secret = get_jwt_secret()?;

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map(|data| data.claims)
        .map_err(|e| AuthError {
            code: StatusCode::UNAUTHORIZED,
            message: format!("Invalid token: {}", e),
        })
    }

    /// Extract token from headers (x-access-token or Authorization: Bearer)
    pub fn extract_token(headers: &HeaderMap) -> Option<String> {
        // Try x-access-token first
        headers
            .get("x-access-token")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .or_else(|| {
                // Fall back to Authorization: Bearer <token>
                headers
                    .get("authorization")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.strip_prefix("Bearer "))
                    .map(|s| s.to_string())
            })
    }

    /// Simple password hash (for demo - use argon2/bcrypt in production)
    pub fn hash_password(password: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        // Add salt from env or default
        let salt = std::env::var("PASSWORD_SALT").unwrap_or_else(|_| "mcp_salt_2026".to_string());
        format!("{}:{}", salt, password).hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Verify password (for demo)
    pub fn verify_password(password: &str, hash: &str) -> bool {
        Self::hash_password(password) == hash
    }
}

// ============================================================================
// MIDDLEWARE
// ============================================================================

/// Required auth middleware - returns 401 if no valid token
pub async fn auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let token = Auth::extract_token(&headers).ok_or_else(|| {
        AuthError {
            code: StatusCode::UNAUTHORIZED,
            message: "Missing token. Use x-access-token header or Authorization: Bearer <token>"
                .to_string(),
        }
        .into_response()
    })?;

    let claims = Auth::verify_token(&token).map_err(|e| e.into_response())?;

    // Inject claims into request extensions
    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}

/// Optional auth middleware - continues even without token
pub async fn optional_auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(token) = Auth::extract_token(&headers) {
        if let Ok(claims) = Auth::verify_token(&token) {
            request.extensions_mut().insert(claims);
        }
    }
    next.run(request).await
}

// ============================================================================
// ROUTES
// ============================================================================

/// Login handler - simple demo implementation
/// In production: validate against database
pub async fn login_handler(Json(payload): Json<LoginRequest>) -> Result<impl IntoResponse, AuthError> {
    // Validate input
    if payload.username.is_empty() || payload.password.is_empty() {
        return Err(AuthError {
            code: StatusCode::BAD_REQUEST,
            message: "Username and password required".to_string(),
        });
    }

    // Demo: check against env vars or hardcoded demo user
    // In production: query database
    let valid_user = std::env::var("AUTH_USERNAME").unwrap_or_else(|_| "admin".to_string());
    let valid_pass_hash = std::env::var("AUTH_PASSWORD_HASH")
        .unwrap_or_else(|_| Auth::hash_password("admin123"));

    if payload.username != valid_user || !Auth::verify_password(&payload.password, &valid_pass_hash) {
        return Err(AuthError {
            code: StatusCode::UNAUTHORIZED,
            message: "Invalid credentials".to_string(),
        });
    }

    // Generate token
    let token_response = Auth::generate_token(
        &payload.username,
        Some(format!("user_{}", payload.username)),
        Some("admin".to_string()),
    )?;

    Ok((StatusCode::OK, Json(token_response)))
}

/// Verify token endpoint - check if token is valid
pub async fn verify_handler(headers: HeaderMap) -> Result<impl IntoResponse, AuthError> {
    let token = Auth::extract_token(&headers).ok_or_else(|| AuthError {
        code: StatusCode::BAD_REQUEST,
        message: "No token provided".to_string(),
    })?;

    let claims = Auth::verify_token(&token)?;

    Ok(Json(json!({
        "valid": true,
        "claims": {
            "sub": claims.sub,
            "exp": claims.exp,
            "iat": claims.iat,
            "uid": claims.user_id,
            "role": claims.role
        }
    })))
}

/// Me endpoint - get current user info from token
pub async fn me_handler(
    axum::extract::Extension(claims): axum::extract::Extension<Claims>,
) -> impl IntoResponse {
    Json(json!({
        "user": claims.sub,
        "user_id": claims.user_id,
        "role": claims.role
    }))
}

/// Create auth router with all endpoints
/// Generic over state to allow nesting in any router
pub fn auth_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    use axum::routing::get;

    Router::new()
        .route("/login", post(login_handler))
        .route("/verify", get(verify_handler))
        .route(
            "/me",
            get(me_handler).layer(axum::middleware::from_fn(auth_middleware)),
        )
}

// ============================================================================
// RE-EXPORTS FOR CONVENIENCE
// ============================================================================

pub use Auth as AuthMiddleware; // Backward compatibility

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Mutex to ensure JWT tests run serially (env vars are global)
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_password_hash() {
        let hash = Auth::hash_password("test123");
        assert!(!hash.is_empty());
        assert!(Auth::verify_password("test123", &hash));
        assert!(!Auth::verify_password("wrong", &hash));
    }

    #[test]
    fn test_token_generation_requires_secret() {
        let _lock = TEST_MUTEX.lock().unwrap();
        
        // Clear env var
        std::env::remove_var("JWT_SECRET");

        let result = Auth::generate_token("test", None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_token_roundtrip() {
        let _lock = TEST_MUTEX.lock().unwrap();
        
        std::env::set_var("JWT_SECRET", "test_secret_key_12345_roundtrip");

        let token_resp = Auth::generate_token("user1", Some("uid1".into()), Some("admin".into()))
            .expect("token generation");

        let claims = Auth::verify_token(&token_resp.token).expect("token verification");

        assert_eq!(claims.sub, "user1");
        assert_eq!(claims.user_id, Some("uid1".to_string()));
        assert_eq!(claims.role, Some("admin".to_string()));

        // Cleanup
        std::env::remove_var("JWT_SECRET");
    }
}