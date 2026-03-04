//! Axum middleware for x-access-token JWT extraction
//!
//! Extracts and verifies JWT from the `x-access-token` HTTP header.

use crate::auth::jwt::{verify_jwt, Claims};

#[cfg(feature = "http-stream")]
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

#[cfg(feature = "http-stream")]
use serde_json::json;

/// Extractor that pulls Claims from the x-access-token header.
///
/// Usage in an Axum handler:
/// ```ignore
/// async fn my_handler(AuthToken(claims): AuthToken) -> impl IntoResponse {
///     // claims.sub is the user_id
///     // claims.email is the email
///     // claims.role is "user" or "admin"
/// }
/// ```
#[cfg(feature = "http-stream")]
pub struct AuthToken(pub Claims);

#[cfg(feature = "http-stream")]
impl<S> FromRequestParts<S> for AuthToken
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        _state: &'life1 S,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            // Try x-access-token header first, then Authorization: Bearer
            let token = parts
                .headers
                .get("x-access-token")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
                .or_else(|| {
                    parts
                        .headers
                        .get("authorization")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.strip_prefix("Bearer "))
                        .map(|s| s.to_string())
                });

            let token = match token {
                Some(ref t) if !t.is_empty() => t.as_str(),
                _ => return Err(AuthError::MissingToken),
            };

            match verify_jwt(token) {
                Ok(claims) => Ok(AuthToken(claims)),
                Err(_) => Err(AuthError::InvalidToken),
            }
        })
    }
}

/// Auth error types for middleware rejection
#[cfg(feature = "http-stream")]
#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
}

#[cfg(feature = "http-stream")]
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Thiếu token xác thực"),
            AuthError::InvalidToken => {
                (StatusCode::UNAUTHORIZED, "Token không hợp lệ hoặc đã hết hạn")
            }
        };

        let body = Json(json!({
            "success": false,
            "error": message,
            "metadata": {
                "executionTime": 0,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        }));

        (status, body).into_response()
    }
}

/// Optional auth extractor -- does NOT reject if token is missing,
/// but still validates if one is present.
#[cfg(feature = "http-stream")]
pub struct OptionalAuthToken(pub Option<Claims>);

#[cfg(feature = "http-stream")]
impl<S> FromRequestParts<S> for OptionalAuthToken
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        _state: &'life1 S,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let token = parts
                .headers
                .get("x-access-token")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
                .or_else(|| {
                    parts
                        .headers
                        .get("authorization")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.strip_prefix("Bearer "))
                        .map(|s| s.to_string())
                });

            let claims = token
                .as_deref()
                .filter(|t| !t.is_empty())
                .and_then(|t| verify_jwt(t).ok());

            Ok(OptionalAuthToken(claims))
        })
    }
}

/// Helper: extract user_id from token string without the Axum extractor.
/// Useful inside tool handlers where you receive the token as a parameter.
pub fn extract_user_id(token: &str) -> Result<String, String> {
    verify_jwt(token)
        .map(|claims| claims.sub)
        .map_err(|e| format!("Token không hợp lệ: {e}"))
}

/// Helper: extract full claims from token string.
pub fn extract_claims(token: &str) -> Result<Claims, String> {
    verify_jwt(token).map_err(|e| format!("Token không hợp lệ: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::jwt::sign_jwt;

    #[test]
    #[cfg(feature = "auth")]
    fn test_extract_user_id() {
        std::env::set_var("JWT_SECRET", "test_middleware_secret");
        let token = sign_jwt("uuid-abc-123", "test@example.com", "user").unwrap();

        let user_id = extract_user_id(&token).unwrap();
        assert_eq!(user_id, "uuid-abc-123");
    }

    #[test]
    #[cfg(feature = "auth")]
    fn test_extract_claims() {
        std::env::set_var("JWT_SECRET", "test_middleware_secret");
        let token = sign_jwt("uuid-xyz", "admin@netadx.ai", "admin").unwrap();

        let claims = extract_claims(&token).unwrap();
        assert_eq!(claims.sub, "uuid-xyz");
        assert_eq!(claims.email, "admin@netadx.ai");
        assert_eq!(claims.role, "admin");
    }

    #[test]
    #[cfg(feature = "auth")]
    fn test_extract_invalid_token() {
        std::env::set_var("JWT_SECRET", "test_middleware_secret");
        let result = extract_user_id("garbage.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_empty_token() {
        let result = extract_user_id("");
        assert!(result.is_err());
    }
}