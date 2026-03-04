//! Auth tool for Đấu Trường Vui
//!
//! All actions authenticate against PostgreSQL `dtv_users` table via PostgREST.
//! Passwords hashed with bcrypt (cost 12). Tokens signed with HS256 JWT.
//!
//! Actions: login, register, google_auth, get_user_info, check_role

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use tracing::{error, info, warn};

use crate::auth::jwt;
use crate::auth::middleware::extract_claims;

// ==================== Types ====================

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub action: String,
    // login / register
    pub email: Option<String>,
    pub password: Option<String>,
    pub name: Option<String>,
    // google_auth
    pub google_token: Option<String>,
    // get_user_info / check_role
    pub token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub metadata: AuthMetadata,
}

#[derive(Debug, Serialize)]
pub struct AuthMetadata {
    #[serde(rename = "executionTime")]
    pub execution_time: u64,
    pub timestamp: String,
}

impl AuthResponse {
    fn ok(token: Option<String>, data: Value) -> Self {
        Self {
            success: true,
            token,
            data: Some(data),
            error: None,
            metadata: AuthMetadata {
                execution_time: 0,
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        }
    }

    fn err(message: &str) -> Self {
        Self {
            success: false,
            token: None,
            data: None,
            error: Some(message.to_string()),
            metadata: AuthMetadata {
                execution_time: 0,
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        }
    }
}

// ==================== PostgREST helpers ====================

fn postgrest_url() -> String {
    env::var("POSTGREST_URL").unwrap_or_else(|_| "http://localhost:3001".to_string())
}

fn table_name(name: &str) -> String {
    let prefix = env::var("DB_TABLE_PREFIX").unwrap_or_else(|_| "dtv_".to_string());
    if name.starts_with(&prefix) {
        name.to_string()
    } else {
        format!("{prefix}{name}")
    }
}

fn http_client() -> reqwest::Client {
    reqwest::Client::new()
}

/// Query PostgREST for users matching filters, returning JSON array
async fn query_users(filter_query: &str, select: &str) -> Result<Vec<Value>, String> {
    let url = format!(
        "{}/{}?{}&select={}",
        postgrest_url(),
        table_name("users"),
        filter_query,
        select
    );

    let resp = http_client()
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("PostgREST request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("PostgREST error {status}: {body}"));
    }

    resp.json::<Vec<Value>>()
        .await
        .map_err(|e| format!("Failed to parse PostgREST response: {e}"))
}

/// Insert a row into a PostgREST table, returning the created row
async fn insert_row(table: &str, data: &Value) -> Result<Value, String> {
    let url = format!("{}/{}", postgrest_url(), table_name(table));

    let resp = http_client()
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Prefer", "return=representation")
        .json(data)
        .send()
        .await
        .map_err(|e| format!("PostgREST insert failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("PostgREST insert error {status}: {body}"));
    }

    let rows: Vec<Value> = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse insert response: {e}"))?;

    rows.into_iter()
        .next()
        .ok_or_else(|| "Insert returned no rows".to_string())
}

/// PATCH a row in PostgREST
async fn patch_row(table: &str, filter_query: &str, data: &Value) -> Result<(), String> {
    let url = format!(
        "{}/{}?{}",
        postgrest_url(),
        table_name(table),
        filter_query
    );

    let resp = http_client()
        .patch(&url)
        .header("Content-Type", "application/json")
        .header("Prefer", "return=minimal")
        .json(data)
        .send()
        .await
        .map_err(|e| format!("PostgREST patch failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("PostgREST patch error {status}: {body}"));
    }

    Ok(())
}

// ==================== Public execute entry point ====================

/// Execute an auth action. Called from protocol_handler and stdio_server.
pub async fn execute(args: Value) -> AuthResponse {
    let start = std::time::Instant::now();

    let req: AuthRequest = match serde_json::from_value(args) {
        Ok(r) => r,
        Err(e) => return AuthResponse::err(&format!("Invalid auth request: {e}")),
    };

    let mut resp = match req.action.as_str() {
        "login" => handle_login(&req).await,
        "register" => handle_register(&req).await,
        "google_auth" => handle_google_auth(&req).await,
        "get_user_info" => handle_get_user_info(&req).await,
        "check_role" => handle_check_role(&req).await,
        other => AuthResponse::err(&format!("Unknown auth action: {other}")),
    };

    resp.metadata.execution_time = start.elapsed().as_millis() as u64;
    resp
}

// ==================== Action handlers ====================

/// Login with email + password
async fn handle_login(req: &AuthRequest) -> AuthResponse {
    let email = match &req.email {
        Some(e) if !e.is_empty() => e.trim().to_lowercase(),
        _ => return AuthResponse::err("Email là bắt buộc"),
    };
    let password = match &req.password {
        Some(p) if !p.is_empty() => p,
        _ => return AuthResponse::err("Mật khẩu là bắt buộc"),
    };

    info!("Auth login attempt: {}", email);

    // Query user by email
    let filter = format!("email=eq.{}", urlencoding_encode(&email));
    let select = "id,email,name,password_hash,role,avatar_url,provider";
    let users = match query_users(&filter, select).await {
        Ok(u) => u,
        Err(e) => {
            error!("Login query failed: {e}");
            return AuthResponse::err("Lỗi hệ thống khi đăng nhập");
        }
    };

    let user = match users.first() {
        Some(u) => u,
        None => {
            warn!("Login failed: email not found: {}", email);
            return AuthResponse::err("Email hoặc mật khẩu không đúng");
        }
    };

    // Verify password
    let hash = user["password_hash"].as_str().unwrap_or("");
    if hash.is_empty() {
        // User registered via Google, no password set
        return AuthResponse::err("Tài khoản này sử dụng đăng nhập Google");
    }

    let password_valid = match bcrypt::verify(password, hash) {
        Ok(v) => v,
        Err(e) => {
            error!("bcrypt verify error: {e}");
            return AuthResponse::err("Lỗi xác thực mật khẩu");
        }
    };

    if !password_valid {
        warn!("Login failed: wrong password for {}", email);
        return AuthResponse::err("Email hoặc mật khẩu không đúng");
    }

    let user_id = user["id"].as_str().unwrap_or("");
    let role = user["role"].as_str().unwrap_or("user");

    // Sign JWT
    let token = match jwt::sign_jwt(user_id, &email, role) {
        Ok(t) => t,
        Err(e) => {
            error!("JWT sign error: {e}");
            return AuthResponse::err("Lỗi tạo token xác thực");
        }
    };

    // Update last_login_at (fire and forget)
    let filter = format!("id=eq.{user_id}");
    let patch_data = json!({ "last_login_at": chrono::Utc::now().to_rfc3339() });
    if let Err(e) = patch_row("users", &filter, &patch_data).await {
        warn!("Failed to update last_login_at: {e}");
    }

    info!("Login success: {} ({})", email, user_id);

    AuthResponse::ok(
        Some(token),
        json!({
            "user": {
                "id": user_id,
                "email": email,
                "name": user.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                "avatar_url": user.get("avatar_url").and_then(|v| v.as_str()),
                "role": role
            }
        }),
    )
}

/// Register a new user with email + password
async fn handle_register(req: &AuthRequest) -> AuthResponse {
    let email = match &req.email {
        Some(e) if !e.is_empty() => e.trim().to_lowercase(),
        _ => return AuthResponse::err("Email là bắt buộc"),
    };
    let password = match &req.password {
        Some(p) if p.len() >= 6 => p,
        Some(_) => return AuthResponse::err("Mật khẩu phải có ít nhất 6 ký tự"),
        None => return AuthResponse::err("Mật khẩu là bắt buộc"),
    };
    let name = req.name.as_deref().unwrap_or("").trim();

    info!("Auth register attempt: {}", email);

    // Check if email already exists
    let filter = format!("email=eq.{}", urlencoding_encode(&email));
    let existing = match query_users(&filter, "id").await {
        Ok(u) => u,
        Err(e) => {
            error!("Register check query failed: {e}");
            return AuthResponse::err("Lỗi hệ thống khi đăng ký");
        }
    };

    if !existing.is_empty() {
        warn!("Register failed: email already exists: {}", email);
        return AuthResponse::err("Email đã được sử dụng");
    }

    // Hash password with bcrypt cost 12
    let password_hash = match bcrypt::hash(password, 12) {
        Ok(h) => h,
        Err(e) => {
            error!("bcrypt hash error: {e}");
            return AuthResponse::err("Lỗi mã hóa mật khẩu");
        }
    };

    // Insert user
    let user_data = json!({
        "email": email,
        "password_hash": password_hash,
        "name": if name.is_empty() { None } else { Some(name) },
        "role": "user",
        "provider": "email"
    });

    let user = match insert_row("users", &user_data).await {
        Ok(u) => u,
        Err(e) => {
            error!("Register insert failed: {e}");
            return AuthResponse::err("Lỗi tạo tài khoản");
        }
    };

    let user_id = user["id"].as_str().unwrap_or("");

    // Credit wallet is auto-created by database trigger (dtv_auto_create_wallet).
    // But as a safety net, also try to insert one here in case trigger doesn't exist yet.
    let wallet_data = json!({
        "user_id": user_id,
        "paid_credits": 0,
        "referral_credits": 0,
        "bonus_credits": 10
    });
    if let Err(e) = insert_row("credit_wallets", &wallet_data).await {
        // Not fatal -- trigger may have already created it
        warn!("Wallet insert (may be duplicate): {e}");
    }

    // Sign JWT
    let token = match jwt::sign_jwt(user_id, &email, "user") {
        Ok(t) => t,
        Err(e) => {
            error!("JWT sign error after register: {e}");
            return AuthResponse::err("Đăng ký thành công nhưng lỗi tạo token");
        }
    };

    info!("Register success: {} ({})", email, user_id);

    AuthResponse::ok(
        Some(token),
        json!({
            "user": {
                "id": user_id,
                "email": email,
                "name": name,
                "avatar_url": null,
                "role": "user"
            }
        }),
    )
}

/// Google OAuth: verify token, find-or-create user by google_id
async fn handle_google_auth(req: &AuthRequest) -> AuthResponse {
    let google_token = match &req.google_token {
        Some(t) if !t.is_empty() => t,
        _ => return AuthResponse::err("Google token là bắt buộc"),
    };

    info!("Auth google_auth attempt");

    // Verify Google token by calling Google's tokeninfo endpoint
    let google_info = match verify_google_token(google_token).await {
        Ok(info) => info,
        Err(e) => {
            warn!("Google token verification failed: {e}");
            return AuthResponse::err("Token Google không hợp lệ");
        }
    };

    let google_id = google_info["sub"].as_str().unwrap_or("");
    let email = google_info["email"].as_str().unwrap_or("");
    let name = google_info["name"].as_str().unwrap_or("");
    let picture = google_info["picture"].as_str();

    if google_id.is_empty() || email.is_empty() {
        return AuthResponse::err("Không lấy được thông tin từ Google");
    }

    // Try to find existing user by google_id
    let filter = format!("google_id=eq.{}", urlencoding_encode(google_id));
    let select = "id,email,name,role,avatar_url,google_id";
    let users = match query_users(&filter, select).await {
        Ok(u) => u,
        Err(e) => {
            error!("Google auth query by google_id failed: {e}");
            return AuthResponse::err("Lỗi hệ thống");
        }
    };

    let (user_id, user_email, user_role, user_name);

    if let Some(existing) = users.first() {
        // Existing user
        user_id = existing["id"].as_str().unwrap_or("").to_string();
        user_email = existing["email"].as_str().unwrap_or(email).to_string();
        user_role = existing["role"].as_str().unwrap_or("user").to_string();
        user_name = existing["name"].as_str().unwrap_or(name).to_string();

        // Update last_login_at
        let patch_filter = format!("id=eq.{}", &user_id);
        let patch = json!({ "last_login_at": chrono::Utc::now().to_rfc3339() });
        let _ = patch_row("users", &patch_filter, &patch).await;
    } else {
        // Also check by email (user may have registered with email first)
        let email_filter = format!("email=eq.{}", urlencoding_encode(email));
        let email_users = match query_users(&email_filter, select).await {
            Ok(u) => u,
            Err(e) => {
                error!("Google auth query by email failed: {e}");
                return AuthResponse::err("Lỗi hệ thống");
            }
        };

        if let Some(existing) = email_users.first() {
            // Link google_id to existing email account
            user_id = existing["id"].as_str().unwrap_or("").to_string();
            user_email = existing["email"].as_str().unwrap_or(email).to_string();
            user_role = existing["role"].as_str().unwrap_or("user").to_string();
            user_name = existing["name"].as_str().unwrap_or(name).to_string();

            let patch_filter = format!("id=eq.{}", &user_id);
            let patch = json!({
                "google_id": google_id,
                "avatar_url": picture,
                "last_login_at": chrono::Utc::now().to_rfc3339()
            });
            let _ = patch_row("users", &patch_filter, &patch).await;
        } else {
            // Create new user
            let user_data = json!({
                "email": email,
                "name": if name.is_empty() { None } else { Some(name) },
                "google_id": google_id,
                "avatar_url": picture,
                "role": "user",
                "provider": "google"
            });

            let new_user = match insert_row("users", &user_data).await {
                Ok(u) => u,
                Err(e) => {
                    error!("Google auth user insert failed: {e}");
                    return AuthResponse::err("Lỗi tạo tài khoản Google");
                }
            };

            user_id = new_user["id"].as_str().unwrap_or("").to_string();
            user_email = email.to_string();
            user_role = "user".to_string();
            user_name = name.to_string();

            // Wallet auto-created by trigger, safety net insert
            let wallet_data = json!({
                "user_id": &user_id,
                "paid_credits": 0,
                "referral_credits": 0,
                "bonus_credits": 10
            });
            let _ = insert_row("credit_wallets", &wallet_data).await;
        }
    }

    // Sign JWT
    let token = match jwt::sign_jwt(&user_id, &user_email, &user_role) {
        Ok(t) => t,
        Err(e) => {
            error!("JWT sign error for Google auth: {e}");
            return AuthResponse::err("Lỗi tạo token");
        }
    };

    info!("Google auth success: {} ({})", user_email, user_id);

    AuthResponse::ok(
        Some(token),
        json!({
            "user": {
                "id": user_id,
                "email": user_email,
                "name": user_name,
                "avatar_url": picture,
                "role": user_role
            }
        }),
    )
}

/// Get user info from JWT token
async fn handle_get_user_info(req: &AuthRequest) -> AuthResponse {
    let token = match &req.token {
        Some(t) if !t.is_empty() => t,
        _ => return AuthResponse::err("Token là bắt buộc"),
    };

    let claims = match extract_claims(token) {
        Ok(c) => c,
        Err(e) => return AuthResponse::err(&e),
    };

    // Query fresh user data from DB
    let filter = format!("id=eq.{}", &claims.sub);
    let select = "id,email,name,avatar_url,role,provider,last_login_at,created_at";
    let users = match query_users(&filter, select).await {
        Ok(u) => u,
        Err(e) => {
            error!("get_user_info query failed: {e}");
            return AuthResponse::err("Lỗi tải thông tin người dùng");
        }
    };

    match users.first() {
        Some(user) => AuthResponse::ok(None, user.clone()),
        None => AuthResponse::err("Người dùng không tồn tại"),
    }
}

/// Check if user has admin role
async fn handle_check_role(req: &AuthRequest) -> AuthResponse {
    let token = match &req.token {
        Some(t) if !t.is_empty() => t,
        _ => return AuthResponse::err("Token là bắt buộc"),
    };

    let claims = match extract_claims(token) {
        Ok(c) => c,
        Err(e) => return AuthResponse::err(&e),
    };

    // Query role from DB (not just from JWT, in case it changed)
    let filter = format!("id=eq.{}", &claims.sub);
    let select = "id,role";
    let users = match query_users(&filter, select).await {
        Ok(u) => u,
        Err(e) => {
            error!("check_role query failed: {e}");
            // Fallback to JWT role
            return AuthResponse::ok(
                None,
                json!({ "isAdmin": claims.role == "admin" }),
            );
        }
    };

    let is_admin = users
        .first()
        .and_then(|u| u["role"].as_str())
        .map(|r| r == "admin")
        .unwrap_or(false);

    AuthResponse::ok(None, json!({ "isAdmin": is_admin }))
}

// ==================== Google token verification ====================

/// Verify a Google ID token via Google's tokeninfo API
async fn verify_google_token(token: &str) -> Result<Value, String> {
    let url = format!(
        "https://oauth2.googleapis.com/tokeninfo?id_token={token}"
    );

    let resp = http_client()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Google API request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err("Google token invalid or expired".to_string());
    }

    resp.json::<Value>()
        .await
        .map_err(|e| format!("Failed to parse Google response: {e}"))
}

// ==================== URL encoding helper ====================

/// Simple URL-encode for PostgREST filter values.
/// Handles the common characters that need escaping.
fn urlencoding_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            '@' => "%40".to_string(),
            '+' => "%2B".to_string(),
            '#' => "%23".to_string(),
            '&' => "%26".to_string(),
            '=' => "%3D".to_string(),
            '?' => "%3F".to_string(),
            '%' => "%25".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_request_deserialize() {
        let json = json!({
            "action": "login",
            "email": "test@example.com",
            "password": "pass123"
        });
        let req: AuthRequest = serde_json::from_value(json).unwrap();
        assert_eq!(req.action, "login");
        assert_eq!(req.email.as_deref(), Some("test@example.com"));
        assert_eq!(req.password.as_deref(), Some("pass123"));
    }

    #[test]
    fn test_auth_request_register() {
        let json = json!({
            "action": "register",
            "email": "new@user.com",
            "password": "secret123",
            "name": "Nguyễn Văn A"
        });
        let req: AuthRequest = serde_json::from_value(json).unwrap();
        assert_eq!(req.action, "register");
        assert_eq!(req.name.as_deref(), Some("Nguyễn Văn A"));
    }

    #[test]
    fn test_auth_response_ok() {
        let resp = AuthResponse::ok(
            Some("jwt_token".to_string()),
            json!({"user": {"id": "123"}}),
        );
        assert!(resp.success);
        assert_eq!(resp.token.as_deref(), Some("jwt_token"));
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_auth_response_err() {
        let resp = AuthResponse::err("Lỗi đăng nhập");
        assert!(!resp.success);
        assert!(resp.token.is_none());
        assert_eq!(resp.error.as_deref(), Some("Lỗi đăng nhập"));
    }

    #[test]
    fn test_urlencoding() {
        assert_eq!(urlencoding_encode("test@email.com"), "test%40email.com");
        assert_eq!(urlencoding_encode("a+b"), "a%2Bb");
        assert_eq!(urlencoding_encode("hello world"), "hello%20world");
        assert_eq!(urlencoding_encode("simple"), "simple");
    }

    #[test]
    fn test_table_name_with_prefix() {
        env::set_var("DB_TABLE_PREFIX", "dtv_");
        assert_eq!(table_name("users"), "dtv_users");
        assert_eq!(table_name("dtv_users"), "dtv_users");
    }

    #[test]
    fn test_table_name_no_duplicate_prefix() {
        env::set_var("DB_TABLE_PREFIX", "dtv_");
        let name = table_name("dtv_credit_wallets");
        assert_eq!(name, "dtv_credit_wallets");
    }

    #[tokio::test]
    async fn test_execute_unknown_action() {
        let resp = execute(json!({ "action": "nonexistent" })).await;
        assert!(!resp.success);
        assert!(resp.error.unwrap().contains("Unknown auth action"));
    }

    #[tokio::test]
    async fn test_execute_login_missing_email() {
        let resp = execute(json!({ "action": "login" })).await;
        assert!(!resp.success);
        assert!(resp.error.unwrap().contains("Email"));
    }

    #[tokio::test]
    async fn test_execute_login_missing_password() {
        let resp = execute(json!({
            "action": "login",
            "email": "test@test.com"
        }))
        .await;
        assert!(!resp.success);
        assert!(resp.error.unwrap().contains("Mật khẩu"));
    }

    #[tokio::test]
    async fn test_execute_register_short_password() {
        let resp = execute(json!({
            "action": "register",
            "email": "test@test.com",
            "password": "abc"
        }))
        .await;
        assert!(!resp.success);
        assert!(resp.error.unwrap().contains("6 ký tự"));
    }

    #[tokio::test]
    async fn test_execute_get_user_info_no_token() {
        let resp = execute(json!({ "action": "get_user_info" })).await;
        assert!(!resp.success);
        assert!(resp.error.unwrap().contains("Token"));
    }

    #[tokio::test]
    async fn test_execute_check_role_no_token() {
        let resp = execute(json!({ "action": "check_role" })).await;
        assert!(!resp.success);
        assert!(resp.error.unwrap().contains("Token"));
    }

    #[tokio::test]
    async fn test_execute_google_auth_no_token() {
        let resp = execute(json!({ "action": "google_auth" })).await;
        assert!(!resp.success);
        assert!(resp.error.unwrap().contains("Google token"));
    }

    #[test]
    fn test_auth_response_serializes_cleanly() {
        let resp = AuthResponse::ok(None, json!({"isAdmin": true}));
        let serialized = serde_json::to_string(&resp).unwrap();
        // token should be absent (skip_serializing_if)
        assert!(!serialized.contains("\"token\""));
        assert!(serialized.contains("\"isAdmin\""));
    }
}