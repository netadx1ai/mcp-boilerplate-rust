#[cfg(all(feature = "http", feature = "auth"))]
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
#[cfg(all(feature = "http", feature = "auth"))]
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
#[cfg(all(feature = "http", feature = "auth"))]
use serde::{Deserialize, Serialize};
#[cfg(all(feature = "http", feature = "auth"))]
use serde_json::json;

#[cfg(all(feature = "http", feature = "auth"))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    #[serde(rename = "userObjId")]
    pub user_obj_id: Option<String>,
    #[serde(rename = "historyLoginObjId")]
    pub history_login_obj_id: Option<String>,
}

#[cfg(all(feature = "http", feature = "auth"))]
pub struct AuthMiddleware;

#[cfg(all(feature = "http", feature = "auth"))]
impl AuthMiddleware {
    pub fn extract_token(headers: &HeaderMap) -> Option<String> {
        headers
            .get("x-access-token")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .or_else(|| {
                headers
                    .get("authorization")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.strip_prefix("Bearer "))
                    .map(|s| s.to_string())
            })
    }

    pub fn verify_token(token: &str, secret: &str) -> Result<Claims, String> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map(|data| data.claims)
        .map_err(|e| format!("Token verification failed: {}", e))
    }
}

#[cfg(all(feature = "http", feature = "auth"))]
pub async fn auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let token = AuthMiddleware::extract_token(&headers);

    if token.is_none() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "success": false,
                "error": "Missing authentication token",
                "message": "Provide x-access-token header or Authorization: Bearer <token>"
            })),
        )
            .into_response());
    }

    let token = token.unwrap();
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string());

    match AuthMiddleware::verify_token(&token, &secret) {
        Ok(claims) => {
            request.extensions_mut().insert(claims);
            Ok(next.run(request).await)
        }
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "success": false,
                "error": "Invalid or expired token",
                "message": e
            })),
        )
            .into_response()),
    }
}

#[cfg(all(feature = "http", feature = "auth"))]
pub async fn optional_auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(token) = AuthMiddleware::extract_token(&headers) {
        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string());
        
        if let Ok(claims) = AuthMiddleware::verify_token(&token, &secret) {
            request.extensions_mut().insert(claims);
        }
    }
    
    next.run(request).await
}