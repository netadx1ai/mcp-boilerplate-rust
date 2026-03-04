//! JWT sign/verify for Đấu Trường Vui auth
//!
//! HS256, JWT_SECRET env, 30-day expiry.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;

#[cfg(feature = "auth")]
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

/// JWT claims payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user id as UUID string)
    pub sub: String,
    /// User email
    pub email: String,
    /// User role (user, admin)
    pub role: String,
    /// Issued at (unix timestamp)
    pub iat: u64,
    /// Expiration (unix timestamp)
    pub exp: u64,
}

/// Default expiry: 30 days in seconds
const DEFAULT_EXPIRY_SECS: u64 = 30 * 24 * 60 * 60;

/// Get JWT secret from env, fallback to "aivaAPI" (shared across NetADX apps)
fn get_secret() -> String {
    env::var("JWT_SECRET").unwrap_or_else(|_| "aivaAPI".to_string())
}

/// Sign a JWT token for the given user
#[cfg(feature = "auth")]
pub fn sign_jwt(user_id: &str, email: &str, role: &str) -> Result<String> {
    let secret = get_secret();
    let now = chrono::Utc::now().timestamp() as u64;

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        iat: now,
        exp: now + DEFAULT_EXPIRY_SECS,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .context("Failed to encode JWT")?;

    Ok(token)
}

/// Verify and decode a JWT token, returning claims
#[cfg(feature = "auth")]
pub fn verify_jwt(token: &str) -> Result<Claims> {
    let secret = get_secret();

    let mut validation = Validation::default();
    validation.validate_exp = true;
    validation.leeway = 60; // 60s clock skew tolerance

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .context("Invalid or expired JWT token")?;

    Ok(token_data.claims)
}

/// Stub when auth feature is disabled
#[cfg(not(feature = "auth"))]
pub fn sign_jwt(_user_id: &str, _email: &str, _role: &str) -> Result<String> {
    anyhow::bail!("Auth feature not enabled. Rebuild with: cargo build --features auth")
}

/// Stub when auth feature is disabled
#[cfg(not(feature = "auth"))]
pub fn verify_jwt(_token: &str) -> Result<Claims> {
    anyhow::bail!("Auth feature not enabled. Rebuild with: cargo build --features auth")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sign + verify using explicit secret (bypasses env var entirely)
    fn sign_with_secret(user_id: &str, email: &str, role: &str, secret: &str) -> String {
        use jsonwebtoken::{encode, EncodingKey, Header};
        let now = chrono::Utc::now().timestamp() as u64;
        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            role: role.to_string(),
            iat: now,
            exp: now + DEFAULT_EXPIRY_SECS,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    fn verify_with_secret(token: &str, secret: &str) -> Result<Claims> {
        use jsonwebtoken::{decode, DecodingKey, Validation};
        let mut validation = Validation::default();
        validation.validate_exp = true;
        validation.leeway = 60;
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        )
        .context("Invalid or expired JWT token")?;
        Ok(data.claims)
    }

    #[test]
    #[cfg(feature = "auth")]
    fn test_sign_verify_roundtrip() {
        let secret = "roundtrip_test_secret_unique_1";
        let token = sign_with_secret("user-uuid-123", "test@example.com", "user", secret);
        assert!(!token.is_empty());

        let claims = verify_with_secret(&token, secret).unwrap();
        assert_eq!(claims.sub, "user-uuid-123");
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.role, "user");
        assert!(claims.exp > claims.iat);
        assert_eq!(claims.exp - claims.iat, DEFAULT_EXPIRY_SECS);
    }

    #[test]
    #[cfg(feature = "auth")]
    fn test_expired_token_rejected() {
        use jsonwebtoken::{encode, EncodingKey, Header};
        let secret = "expired_test_secret_unique_2";

        let now = chrono::Utc::now().timestamp() as u64;
        let claims = Claims {
            sub: "user-1".to_string(),
            email: "expired@test.com".to_string(),
            role: "user".to_string(),
            iat: now - 100_000,
            exp: now - 1000,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let result = verify_with_secret(&token, secret);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("expired") || err_msg.contains("Invalid"),
            "Expected expiry error, got: {err_msg}"
        );
    }

    #[test]
    #[cfg(feature = "auth")]
    fn test_invalid_secret_rejected() {
        let token = sign_with_secret("user-1", "a@b.com", "user", "secret_a");
        let result = verify_with_secret(&token, "secret_b");
        assert!(result.is_err());
    }

    #[test]
    #[cfg(feature = "auth")]
    fn test_admin_role_in_token() {
        let secret = "admin_test_secret_unique_4";
        let token = sign_with_secret("admin-uuid", "admin@netadx.ai", "admin", secret);
        let claims = verify_with_secret(&token, secret).unwrap();
        assert_eq!(claims.role, "admin");
    }

    #[test]
    #[cfg(feature = "auth")]
    #[ignore] // Races with parallel tests that mutate JWT_SECRET env var. Run: cargo test test_sign_jwt_uses_env -- --test-threads=1 --ignored
    fn test_sign_jwt_uses_env() {
        env::set_var("JWT_SECRET", "env_integration_test_secret");
        let token = sign_jwt("u1", "e@e.com", "user").unwrap();
        let claims = verify_jwt(&token).unwrap();
        assert_eq!(claims.sub, "u1");
    }
}