//! # Authenticated MCP Server Template Library
//!
//! This library provides reusable components for building MCP servers with authentication.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Authentication errors
#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    #[error("Session not found")]
    SessionNotFound,
    #[error("OAuth error: {0}")]
    OAuthError(String),
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// User roles for authorization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Admin,
    User,
    ReadOnly,
    Service,
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,         // Subject (user ID)
    pub username: String,    // Username
    pub role: UserRole,      // User role
    pub permissions: Vec<String>, // User permissions
    pub exp: i64,           // Expiration time
    pub iat: i64,           // Issued at
    pub jti: String,        // JWT ID
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub refresh_token: String,
    pub csrf_token: String,
}

/// OAuth state for PKCE flow
#[derive(Debug)]
pub struct OAuthState {
    pub csrf_token: CsrfToken,
    pub pkce_verifier: PkceCodeVerifier,
    pub redirect_uri: String,
}

/// Authentication configuration
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub oauth_client_id: Option<String>,
    pub oauth_client_secret: Option<String>,
    pub oauth_auth_url: Option<String>,
    pub oauth_token_url: Option<String>,
    pub oauth_redirect_url: Option<String>,
    pub session_timeout_minutes: i64,
    pub token_timeout_minutes: i64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "your-super-secret-jwt-key-change-in-production".to_string(),
            oauth_client_id: None,
            oauth_client_secret: None,
            oauth_auth_url: Some("https://auth.example.com/oauth/authorize".to_string()),
            oauth_token_url: Some("https://auth.example.com/oauth/token".to_string()),
            oauth_redirect_url: Some("http://localhost:8080/auth/callback".to_string()),
            session_timeout_minutes: 60 * 24, // 24 hours
            token_timeout_minutes: 60,        // 1 hour
        }
    }
}

/// Authentication credentials
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthCredentials {
    #[serde(rename = "oauth")]
    OAuth {
        authorization_code: String,
        state: String,
    },
    #[serde(rename = "api_key")]
    ApiKey {
        key: String,
    },
    #[serde(rename = "jwt")]
    Jwt {
        token: String,
    },
    #[serde(rename = "username_password")]
    UsernamePassword {
        username: String,
        password: String,
    },
}

/// Authentication response
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    pub user: Option<User>,
    pub session_id: Option<Uuid>,
}

/// Token validation response
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenValidation {
    pub valid: bool,
    pub user_id: Option<Uuid>,
    pub username: Option<String>,
    pub role: Option<UserRole>,
    pub permissions: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// OAuth authorization URL response
#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthUrlResponse {
    pub auth_url: String,
    pub state: String,
    pub pkce_challenge: String,
}

/// Permission check parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionCheck {
    pub user_id: Uuid,
    pub required_permission: String,
}

/// Server statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthServerStats {
    pub active_sessions: usize,
    pub total_users: usize,
    pub oauth_states: usize,
    pub api_keys: usize,
    pub uptime_seconds: u64,
    pub authentication_attempts: u64,
    pub successful_authentications: u64,
    pub failed_authentications: u64,
    pub token_validations: u64,
    pub session_timeouts: u64,
}

/// Main authenticated MCP server
#[derive(Clone)]
pub struct AuthenticatedServer {
    config: AuthConfig,
    users: Arc<DashMap<Uuid, User>>,
    sessions: Arc<DashMap<Uuid, Session>>,
    api_keys: Arc<DashMap<String, Uuid>>, // API key -> User ID
    oauth_states: Arc<DashMap<String, OAuthState>>,
    stats: Arc<RwLock<AuthServerStats>>,
    start_time: DateTime<Utc>,
}

impl Default for AuthenticatedServer {
    fn default() -> Self {
        Self::new(AuthConfig::default())
    }
}

impl AuthenticatedServer {
    /// Create a new authenticated server
    pub fn new(config: AuthConfig) -> Self {
        let server = Self {
            config,
            users: Arc::new(DashMap::new()),
            sessions: Arc::new(DashMap::new()),
            api_keys: Arc::new(DashMap::new()),
            oauth_states: Arc::new(DashMap::new()),
            stats: Arc::new(RwLock::new(AuthServerStats {
                active_sessions: 0,
                total_users: 0,
                oauth_states: 0,
                api_keys: 0,
                uptime_seconds: 0,
                authentication_attempts: 0,
                successful_authentications: 0,
                failed_authentications: 0,
                token_validations: 0,
                session_timeouts: 0,
            })),
            start_time: Utc::now(),
        };

        // Initialize with sample users
        server.initialize_sample_data();
        server
    }

    /// Initialize with sample users and API keys
    fn initialize_sample_data(&self) {
        // Create sample users
        let admin_user = User {
            id: Uuid::new_v4(),
            username: "admin".to_string(),
            email: "admin@example.com".to_string(),
            role: UserRole::Admin,
            permissions: vec![
                "read".to_string(),
                "write".to_string(),
                "delete".to_string(),
                "admin".to_string(),
            ],
            created_at: Utc::now(),
            last_login: None,
        };

        let regular_user = User {
            id: Uuid::new_v4(),
            username: "user".to_string(),
            email: "user@example.com".to_string(),
            role: UserRole::User,
            permissions: vec!["read".to_string(), "write".to_string()],
            created_at: Utc::now(),
            last_login: None,
        };

        let readonly_user = User {
            id: Uuid::new_v4(),
            username: "readonly".to_string(),
            email: "readonly@example.com".to_string(),
            role: UserRole::ReadOnly,
            permissions: vec!["read".to_string()],
            created_at: Utc::now(),
            last_login: None,
        };

        // Store users
        let admin_id = admin_user.id;
        let user_id = regular_user.id;
        let readonly_id = readonly_user.id;
        
        self.users.insert(admin_id, admin_user);
        self.users.insert(user_id, regular_user);
        self.users.insert(readonly_id, readonly_user);

        // Create sample API keys
        self.api_keys.insert("admin-key-12345".to_string(), admin_id);
        self.api_keys.insert("user-key-67890".to_string(), user_id);
        self.api_keys.insert("readonly-key-11111".to_string(), readonly_id);

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.total_users = 3;
            stats.api_keys = 3;
        }

        info!("Initialized sample data: 3 users, 3 API keys");
    }

    /// Generate JWT token for user
    fn generate_jwt_token(&self, user: &User) -> Result<String, AuthError> {
        let now = Utc::now();
        let exp = now + Duration::minutes(self.config.token_timeout_minutes);

        let claims = Claims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            role: user.role.clone(),
            permissions: user.permissions.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
        };

        let key = EncodingKey::from_secret(self.config.jwt_secret.as_bytes());
        encode(&Header::default(), &claims, &key)
            .map_err(|e| AuthError::Internal(anyhow::anyhow!("JWT encoding error: {}", e)))
    }

    /// Validate JWT token
    fn validate_jwt_token(&self, token: &str) -> Result<Claims, AuthError> {
        let key = DecodingKey::from_secret(self.config.jwt_secret.as_bytes());
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        decode::<Claims>(token, &key, &validation)
            .map(|data| data.claims)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))
    }

    /// Create session for user
    fn create_session(&self, user_id: Uuid) -> Session {
        let session = Session {
            id: Uuid::new_v4(),
            user_id,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(self.config.session_timeout_minutes),
            refresh_token: Uuid::new_v4().to_string(),
            csrf_token: Uuid::new_v4().to_string(),
        };

        self.sessions.insert(session.id, session.clone());
        
        // Update stats
        {
            let mut stats = self.stats.write();
            stats.active_sessions = self.sessions.len();
        }

        session
    }

    /// Clean expired sessions
    fn cleanup_expired_sessions(&self) {
        let now = Utc::now();
        let mut expired_sessions = Vec::new();

        for entry in self.sessions.iter() {
            if entry.value().expires_at < now {
                expired_sessions.push(*entry.key());
            }
        }

        for session_id in expired_sessions {
            self.sessions.remove(&session_id);
            let mut stats = self.stats.write();
            stats.session_timeouts += 1;
        }

        // Update active sessions count
        {
            let mut stats = self.stats.write();
            stats.active_sessions = self.sessions.len();
        }
    }

    /// Update server statistics
    fn update_stats(&self) {
        let mut stats = self.stats.write();
        stats.uptime_seconds = (Utc::now() - self.start_time).num_seconds() as u64;
        stats.total_users = self.users.len();
        stats.oauth_states = self.oauth_states.len();
        stats.api_keys = self.api_keys.len();
    }

    /// Authenticate user with various credential types
    pub async fn authenticate_user(
        &self,
        credentials: AuthCredentials,
    ) -> Result<AuthResponse, AuthError> {
        self.cleanup_expired_sessions();
        
        // Update stats
        {
            let mut stats = self.stats.write();
            stats.authentication_attempts += 1;
        }

        let result = match credentials {
            AuthCredentials::ApiKey { key } => {
                if let Some(user_id) = self.api_keys.get(&key) {
                    if let Some(user) = self.users.get(&user_id) {
                        let session = self.create_session(*user_id);
                        let token = self.generate_jwt_token(&user)?;
                        
                        Ok(AuthResponse {
                            success: true,
                            token: Some(token),
                            refresh_token: Some(session.refresh_token.clone()),
                            expires_in: Some(self.config.token_timeout_minutes * 60),
                            user: Some(user.clone()),
                            session_id: Some(session.id),
                        })
                    } else {
                        Err(AuthError::InvalidCredentials)
                    }
                } else {
                    Err(AuthError::InvalidCredentials)
                }
            }
            AuthCredentials::Jwt { token } => {
                match self.validate_jwt_token(&token) {
                    Ok(claims) => {
                        if let Ok(user_id) = Uuid::parse_str(&claims.sub) {
                            if let Some(user) = self.users.get(&user_id) {
                                Ok(AuthResponse {
                                    success: true,
                                    token: Some(token),
                                    refresh_token: None,
                                    expires_in: Some(claims.exp - Utc::now().timestamp()),
                                    user: Some(user.clone()),
                                    session_id: None,
                                })
                            } else {
                                Err(AuthError::InvalidCredentials)
                            }
                        } else {
                            Err(AuthError::InvalidToken("Invalid user ID in token".to_string()))
                        }
                    }
                    Err(e) => Err(e),
                }
            }
            AuthCredentials::UsernamePassword { username, password: _ } => {
                // Mock password validation (in production, use proper password hashing)
                let user = self.users.iter()
                    .find(|entry| entry.value().username == username)
                    .map(|entry| entry.value().clone());

                if let Some(user) = user {
                    let session = self.create_session(user.id);
                    let token = self.generate_jwt_token(&user)?;
                    
                    Ok(AuthResponse {
                        success: true,
                        token: Some(token),
                        refresh_token: Some(session.refresh_token.clone()),
                        expires_in: Some(self.config.token_timeout_minutes * 60),
                        user: Some(user),
                        session_id: Some(session.id),
                    })
                } else {
                    Err(AuthError::InvalidCredentials)
                }
            }
            AuthCredentials::OAuth { authorization_code: _, state: _ } => {
                // Mock OAuth flow (in production, implement actual OAuth 2.0 flow)
                warn!("OAuth flow not fully implemented in template");
                Err(AuthError::OAuthError("OAuth flow not implemented".to_string()))
            }
        };

        // Update success/failure stats
        match &result {
            Ok(_) => {
                let mut stats = self.stats.write();
                stats.successful_authentications += 1;
            }
            Err(_) => {
                let mut stats = self.stats.write();
                stats.failed_authentications += 1;
            }
        }

        result
    }

    /// Validate JWT token
    pub async fn validate_token(&self, token: String) -> Result<TokenValidation, AuthError> {
        {
            let mut stats = self.stats.write();
            stats.token_validations += 1;
        }

        match self.validate_jwt_token(&token) {
            Ok(claims) => {
                let user_id = Uuid::parse_str(&claims.sub)
                    .map_err(|_| AuthError::InvalidToken("Invalid user ID".to_string()))?;
                
                Ok(TokenValidation {
                    valid: true,
                    user_id: Some(user_id),
                    username: Some(claims.username),
                    role: Some(claims.role),
                    permissions: claims.permissions,
                    expires_at: Some(DateTime::from_timestamp(claims.exp, 0)
                        .unwrap_or_else(|| Utc::now())),
                })
            }
            Err(_e) => {
                Ok(TokenValidation {
                    valid: false,
                    user_id: None,
                    username: None,
                    role: None,
                    permissions: vec![],
                    expires_at: None,
                })
            }
        }
    }

    /// Refresh JWT token using refresh token
    pub async fn refresh_token(&self, refresh_token: String) -> Result<AuthResponse, AuthError> {
        self.cleanup_expired_sessions();

        // Find session by refresh token
        let session = self.sessions.iter()
            .find(|entry| entry.value().refresh_token == refresh_token)
            .map(|entry| entry.value().clone());

        if let Some(session) = session {
            if session.expires_at > Utc::now() {
                if let Some(user) = self.users.get(&session.user_id) {
                    let new_token = self.generate_jwt_token(&user)?;
                    
                    Ok(AuthResponse {
                        success: true,
                        token: Some(new_token),
                        refresh_token: Some(refresh_token), // Keep same refresh token
                        expires_in: Some(self.config.token_timeout_minutes * 60),
                        user: Some(user.clone()),
                        session_id: Some(session.id),
                    })
                } else {
                    Err(AuthError::SessionNotFound)
                }
            } else {
                // Remove expired session
                self.sessions.remove(&session.id);
                Err(AuthError::TokenExpired)
            }
        } else {
            Err(AuthError::SessionNotFound)
        }
    }

    /// Get user profile by token
    pub async fn get_user_profile(&self, token: String) -> Result<User, AuthError> {
        let claims = self.validate_jwt_token(&token)?;
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AuthError::InvalidToken("Invalid user ID".to_string()))?;

        self.users.get(&user_id)
            .map(|user| user.clone())
            .ok_or(AuthError::InvalidCredentials)
    }

    /// List user permissions
    pub async fn list_permissions(&self, user_id: Uuid) -> Result<Vec<String>, AuthError> {
        self.users.get(&user_id)
            .map(|user| user.permissions.clone())
            .ok_or(AuthError::InvalidCredentials)
    }

    /// Check if user has specific permission
    pub async fn check_permission(&self, check: PermissionCheck) -> Result<bool, AuthError> {
        if let Some(user) = self.users.get(&check.user_id) {
            Ok(user.permissions.contains(&check.required_permission) || 
               user.permissions.contains(&"admin".to_string()))
        } else {
            Err(AuthError::InvalidCredentials)
        }
    }

    /// Logout user by invalidating session
    pub async fn logout(&self, token: String) -> Result<bool, AuthError> {
        let claims = self.validate_jwt_token(&token)?;
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AuthError::InvalidToken("Invalid user ID".to_string()))?;

        // Remove all sessions for this user
        let sessions_to_remove: Vec<Uuid> = self.sessions.iter()
            .filter(|entry| entry.value().user_id == user_id)
            .map(|entry| *entry.key())
            .collect();

        for session_id in sessions_to_remove {
            self.sessions.remove(&session_id);
        }

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.active_sessions = self.sessions.len();
        }

        Ok(true)
    }

    /// Get server status and statistics
    pub async fn get_server_status(&self) -> Result<AuthServerStats, AuthError> {
        self.cleanup_expired_sessions();
        self.update_stats();
        
        let stats = self.stats.read();
        Ok(stats.clone())
    }
}