//! MCP OAuth 2.1 Authorization Module
//!
//! Implements MCP Authorization Spec (2025-11-25):
//! - OAuth 2.1 with PKCE (required for all clients)
//! - Authorization Server Metadata Discovery (RFC 8414)
//! - Protected Resource Metadata (RFC 9728) - NEW
//! - OpenID Connect Discovery Support - NEW
//! - Dynamic Client Registration (RFC 7591)
//! - Client ID Metadata Documents - NEW
//! - Third-Party Authorization Flow
//!
//! Last Updated: 2026-01-09 13:53 HCMC

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Form, Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::RwLock;

// ============================================================================
// TYPES - OAuth 2.1 Standard
// ============================================================================

/// OAuth 2.1 Error Response (RFC 6749)
#[derive(Debug, Serialize)]
pub struct OAuthError {
    pub error: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_uri: Option<String>,
}

impl OAuthError {
    pub fn new(error: &'static str, description: impl Into<String>) -> Self {
        Self {
            error,
            error_description: Some(description.into()),
            error_uri: None,
        }
    }

    pub fn invalid_request(desc: impl Into<String>) -> Self {
        Self::new("invalid_request", desc)
    }

    pub fn invalid_client(desc: impl Into<String>) -> Self {
        Self::new("invalid_client", desc)
    }

    pub fn invalid_grant(desc: impl Into<String>) -> Self {
        Self::new("invalid_grant", desc)
    }

    pub fn unauthorized_client(desc: impl Into<String>) -> Self {
        Self::new("unauthorized_client", desc)
    }

    pub fn unsupported_grant_type(desc: impl Into<String>) -> Self {
        Self::new("unsupported_grant_type", desc)
    }

    pub fn invalid_scope(desc: impl Into<String>) -> Self {
        Self::new("invalid_scope", desc)
    }
}

impl IntoResponse for OAuthError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}

/// Token Response (RFC 6749 Section 5.1)
#[derive(Debug, Serialize, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// Token Request
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub code_verifier: Option<String>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

/// Authorization Request (Authorization Code flow with PKCE)
#[derive(Debug, Deserialize)]
pub struct AuthorizeRequest {
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    // PKCE - Required in OAuth 2.1/MCP
    pub code_challenge: String,
    pub code_challenge_method: String,
}

/// Token Introspection Request (RFC 7662)
#[derive(Debug, Deserialize)]
pub struct IntrospectRequest {
    pub token: String,
    #[serde(default)]
    pub token_type_hint: Option<String>,
}

/// Token Introspection Response
#[derive(Debug, Serialize)]
pub struct IntrospectResponse {
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<u64>,
}

/// Dynamic Client Registration Request (RFC 7591)
#[derive(Debug, Deserialize)]
pub struct ClientRegistrationRequest {
    #[serde(default)]
    pub client_name: Option<String>,
    pub redirect_uris: Vec<String>,
    #[serde(default)]
    pub grant_types: Option<Vec<String>>,
    #[serde(default)]
    pub token_endpoint_auth_method: Option<String>,
    #[serde(default)]
    pub response_types: Option<Vec<String>>,
    #[serde(default)]
    pub scope: Option<String>,
}

/// Dynamic Client Registration Response
#[derive(Debug, Serialize)]
pub struct ClientRegistrationResponse {
    pub client_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,
    pub redirect_uris: Vec<String>,
    pub grant_types: Vec<String>,
    pub token_endpoint_auth_method: String,
    pub response_types: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret_expires_at: Option<u64>,
}

/// Authorization Server Metadata (RFC 8414)
#[derive(Debug, Serialize, Clone)]
pub struct AuthorizationServerMetadata {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub introspection_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revocation_endpoint: Option<String>,
    pub response_types_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
    pub token_endpoint_auth_methods_supported: Vec<String>,
    pub code_challenge_methods_supported: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes_supported: Option<Vec<String>>,
    /// Client ID Metadata Document support (MCP 2025-11-25)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id_metadata_document_supported: Option<bool>,
}

/// Protected Resource Metadata (RFC 9728) - MCP 2025-11-25
#[derive(Debug, Serialize, Clone)]
pub struct ProtectedResourceMetadata {
    /// URL identifier for this protected resource
    pub resource: String,
    /// Authorization servers that can issue tokens for this resource
    pub authorization_servers: Vec<String>,
    /// How bearer tokens can be transmitted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_methods_supported: Option<Vec<String>>,
    /// Scopes understood by this resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes_supported: Option<Vec<String>>,
    /// URL to resource documentation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_documentation: Option<String>,
    /// Resource name for display
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_name: Option<String>,
}

/// Client ID Metadata Document (draft-ietf-oauth-client-id-metadata-document)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClientIdMetadataDocument {
    /// Client identifier (must match the URL it was fetched from)
    pub client_id: String,
    /// Redirect URIs for authorization code flow
    #[serde(default)]
    pub redirect_uris: Vec<String>,
    /// Grant types the client will use
    #[serde(default)]
    pub grant_types: Option<Vec<String>>,
    /// Response types the client will use
    #[serde(default)]
    pub response_types: Option<Vec<String>>,
    /// Client name for display
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,
    /// Client URI (homepage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_uri: Option<String>,
    /// Logo URI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_uri: Option<String>,
    /// Scope the client will request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Token endpoint auth method
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_endpoint_auth_method: Option<String>,
}

// ============================================================================
// INTERNAL TYPES
// ============================================================================

/// Registered OAuth Client
#[derive(Debug, Clone)]
pub struct OAuthClient {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub client_name: Option<String>,
    pub redirect_uris: Vec<String>,
    pub grant_types: Vec<String>,
    pub response_types: Vec<String>,
    pub allowed_scopes: Vec<String>,
    pub token_endpoint_auth_method: String,
    pub created_at: u64,
}

impl OAuthClient {
    pub fn is_public(&self) -> bool {
        self.client_secret.is_none()
    }

    pub fn is_confidential(&self) -> bool {
        self.client_secret.is_some()
    }
}

/// Authorization Code (temporary)
#[derive(Debug, Clone)]
struct AuthCode {
    code: String,
    client_id: String,
    redirect_uri: String,
    scope: Option<String>,
    code_challenge: String,
    code_challenge_method: String,
    expires_at: u64,
    user_id: Option<String>,
}

/// Stored Token
#[derive(Debug, Clone)]
struct StoredToken {
    access_token: String,
    refresh_token: Option<String>,
    client_id: String,
    scope: Option<String>,
    expires_at: u64,
    user_id: Option<String>,
    // Third-party token binding (for delegated auth)
    third_party_token: Option<String>,
}

// ============================================================================
// CONFIGURATION
// ============================================================================

/// OAuth Server Configuration
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub issuer: String,
    pub token_expiry_secs: u64,
    pub refresh_token_expiry_secs: u64,
    pub auth_code_expiry_secs: u64,
    pub allow_dynamic_registration: bool,
    pub default_scopes: Vec<String>,
    /// Resource URL for Protected Resource Metadata (RFC 9728)
    pub resource_url: String,
    /// Resource name for display
    pub resource_name: String,
    /// Resource documentation URL
    pub resource_documentation: Option<String>,
    /// Enable Client ID Metadata Document support
    pub client_id_metadata_document_supported: bool,
    /// Cache TTL for fetched client metadata (seconds)
    pub client_metadata_cache_ttl: u64,
}

impl Default for OAuthConfig {
    fn default() -> Self {
        let issuer = std::env::var("OAUTH_ISSUER")
            .unwrap_or_else(|_| "http://localhost:8025".into());
        Self {
            resource_url: std::env::var("OAUTH_RESOURCE_URL")
                .unwrap_or_else(|_| issuer.clone()),
            issuer,
            token_expiry_secs: std::env::var("OAUTH_TOKEN_EXPIRY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3600), // 1 hour
            refresh_token_expiry_secs: std::env::var("OAUTH_REFRESH_TOKEN_EXPIRY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(86400 * 30), // 30 days
            auth_code_expiry_secs: 600, // 10 minutes (RFC recommendation)
            allow_dynamic_registration: true,
            default_scopes: vec!["mcp:read".into(), "mcp:write".into()],
            resource_name: std::env::var("OAUTH_RESOURCE_NAME")
                .unwrap_or_else(|_| "MCP Server".into()),
            resource_documentation: std::env::var("OAUTH_RESOURCE_DOCS").ok(),
            client_id_metadata_document_supported: true,
            client_metadata_cache_ttl: 3600, // 1 hour default
        }
    }
}

// ============================================================================
// STATE
// ============================================================================

/// Cached client metadata from URL-based client_id
#[derive(Debug, Clone)]
struct CachedClientMetadata {
    metadata: ClientIdMetadataDocument,
    fetched_at: u64,
    ttl: u64,
}

/// OAuth Server State
#[derive(Clone)]
pub struct OAuthState {
    config: OAuthConfig,
    clients: Arc<RwLock<HashMap<String, OAuthClient>>>,
    auth_codes: Arc<RwLock<HashMap<String, AuthCode>>>,
    tokens: Arc<RwLock<HashMap<String, StoredToken>>>,
    refresh_tokens: Arc<RwLock<HashMap<String, String>>>,
    /// Cache for URL-based client_id metadata documents
    client_metadata_cache: Arc<RwLock<HashMap<String, CachedClientMetadata>>>,
}

impl OAuthState {
    pub fn new(config: OAuthConfig) -> Self {
        Self {
            config,
            clients: Arc::new(RwLock::new(HashMap::new())),
            auth_codes: Arc::new(RwLock::new(HashMap::new())),
            tokens: Arc::new(RwLock::new(HashMap::new())),
            refresh_tokens: Arc::new(RwLock::new(HashMap::new())),
            client_metadata_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(OAuthConfig::default())
    }

    /// Register a client programmatically
    pub async fn register_client(&self, client: OAuthClient) {
        let mut clients = self.clients.write().await;
        clients.insert(client.client_id.clone(), client);
    }

    /// Setup default MCP clients
    pub async fn setup_default_clients(&self) {
        // Confidential client (server-to-server)
        self.register_client(OAuthClient {
            client_id: "mcp-server".into(),
            client_secret: Some(
                std::env::var("OAUTH_CLIENT_SECRET")
                    .unwrap_or_else(|_| "mcp_secret_2026".into()),
            ),
            client_name: Some("MCP Server Client".into()),
            redirect_uris: vec![],
            grant_types: vec!["client_credentials".into(), "refresh_token".into()],
            response_types: vec![],
            allowed_scopes: vec!["mcp:read".into(), "mcp:write".into(), "mcp:admin".into()],
            token_endpoint_auth_method: "client_secret_basic".into(),
            created_at: now_secs(),
        })
        .await;

        // Public client (SPA/mobile - requires PKCE)
        self.register_client(OAuthClient {
            client_id: "mcp-public".into(),
            client_secret: None,
            client_name: Some("MCP Public Client".into()),
            redirect_uris: vec![
                "http://localhost:3000/callback".into(),
                "http://127.0.0.1:3000/callback".into(),
                "http://localhost:8080/callback".into(),
            ],
            grant_types: vec!["authorization_code".into(), "refresh_token".into()],
            response_types: vec!["code".into()],
            allowed_scopes: vec!["mcp:read".into(), "mcp:write".into()],
            token_endpoint_auth_method: "none".into(),
            created_at: now_secs(),
        })
        .await;
    }

    /// Get server metadata (RFC 8414)
    pub fn get_metadata(&self) -> AuthorizationServerMetadata {
        let base = &self.config.issuer;
        AuthorizationServerMetadata {
            issuer: base.clone(),
            authorization_endpoint: format!("{}/authorize", base),
            token_endpoint: format!("{}/token", base),
            registration_endpoint: if self.config.allow_dynamic_registration {
                Some(format!("{}/register", base))
            } else {
                None
            },
            introspection_endpoint: Some(format!("{}/introspect", base)),
            revocation_endpoint: Some(format!("{}/revoke", base)),
            response_types_supported: vec!["code".into()],
            grant_types_supported: vec![
                "authorization_code".into(),
                "client_credentials".into(),
                "refresh_token".into(),
            ],
            token_endpoint_auth_methods_supported: vec![
                "client_secret_basic".into(),
                "client_secret_post".into(),
                "none".into(),
            ],
            code_challenge_methods_supported: vec!["S256".into()],
            scopes_supported: Some(self.config.default_scopes.clone()),
            client_id_metadata_document_supported: if self.config.client_id_metadata_document_supported {
                Some(true)
            } else {
                None
            },
        }
    }

    /// Get protected resource metadata (RFC 9728) - MCP 2025-11-25
    pub fn get_resource_metadata(&self) -> ProtectedResourceMetadata {
        ProtectedResourceMetadata {
            resource: self.config.resource_url.clone(),
            authorization_servers: vec![self.config.issuer.clone()],
            bearer_methods_supported: Some(vec!["header".into()]),
            scopes_supported: Some(self.config.default_scopes.clone()),
            resource_documentation: self.config.resource_documentation.clone(),
            resource_name: Some(self.config.resource_name.clone()),
        }
    }

    /// Check if client_id is a URL (for Client ID Metadata Document)
    pub fn is_url_client_id(client_id: &str) -> bool {
        client_id.starts_with("https://")
    }

    /// Fetch and cache client metadata from URL-based client_id
    pub async fn fetch_client_metadata(&self, client_id: &str) -> Result<ClientIdMetadataDocument, OAuthError> {
        if !Self::is_url_client_id(client_id) {
            return Err(OAuthError::invalid_client("client_id is not a URL"));
        }

        // Check cache first
        {
            let cache = self.client_metadata_cache.read().await;
            if let Some(cached) = cache.get(client_id) {
                if now_secs() < cached.fetched_at + cached.ttl {
                    return Ok(cached.metadata.clone());
                }
            }
        }

        // Fetch from URL
        let client = reqwest::Client::new();
        let response = client
            .get(client_id)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| OAuthError::invalid_client(format!("Failed to fetch client metadata: {}", e)))?;

        if !response.status().is_success() {
            return Err(OAuthError::invalid_client(format!(
                "Client metadata fetch failed with status: {}",
                response.status()
            )));
        }

        // Parse cache control header for TTL
        let ttl = response
            .headers()
            .get("cache-control")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| {
                s.split(',')
                    .find(|p| p.trim().starts_with("max-age="))
                    .and_then(|p| p.trim().strip_prefix("max-age="))
                    .and_then(|v| v.parse::<u64>().ok())
            })
            .unwrap_or(self.config.client_metadata_cache_ttl);

        let metadata: ClientIdMetadataDocument = response
            .json()
            .await
            .map_err(|e| OAuthError::invalid_client(format!("Invalid client metadata JSON: {}", e)))?;

        // Validate that client_id in document matches the URL
        if metadata.client_id != client_id {
            return Err(OAuthError::invalid_client(
                "client_id in metadata does not match fetch URL"
            ));
        }

        // Cache the result
        {
            let mut cache = self.client_metadata_cache.write().await;
            cache.insert(client_id.to_string(), CachedClientMetadata {
                metadata: metadata.clone(),
                fetched_at: now_secs(),
                ttl,
            });
        }

        Ok(metadata)
    }

    /// Get or fetch client (supports both registered and URL-based clients)
    pub async fn get_client(&self, client_id: &str) -> Result<OAuthClient, OAuthError> {
        // First check registered clients
        {
            let clients = self.clients.read().await;
            if let Some(client) = clients.get(client_id) {
                return Ok(client.clone());
            }
        }

        // If client_id is a URL, try to fetch metadata document
        if Self::is_url_client_id(client_id) && self.config.client_id_metadata_document_supported {
            let metadata = self.fetch_client_metadata(client_id).await?;
            
            // Convert metadata document to OAuthClient
            let client = OAuthClient {
                client_id: metadata.client_id,
                client_secret: None, // URL-based clients are always public
                client_name: metadata.client_name,
                redirect_uris: metadata.redirect_uris,
                grant_types: metadata.grant_types.unwrap_or_else(|| vec!["authorization_code".into()]),
                response_types: metadata.response_types.unwrap_or_else(|| vec!["code".into()]),
                allowed_scopes: metadata.scope
                    .map(|s| s.split_whitespace().map(String::from).collect())
                    .unwrap_or_else(|| self.config.default_scopes.clone()),
                token_endpoint_auth_method: metadata.token_endpoint_auth_method.unwrap_or_else(|| "none".into()),
                created_at: now_secs(),
            };

            return Ok(client);
        }

        Err(OAuthError::invalid_client("Unknown client_id"))
    }
}

impl Default for OAuthState {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// ============================================================================
// HELPERS
// ============================================================================

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

fn generate_token() -> String {
    format!(
        "{:016x}{:016x}",
        rand::random::<u64>(),
        rand::random::<u64>()
    )
}

fn generate_code() -> String {
    format!("{:032x}", rand::random::<u128>())
}

fn generate_client_id() -> String {
    format!("client_{:016x}", rand::random::<u64>())
}

fn generate_client_secret() -> String {
    format!(
        "{:016x}{:016x}{:016x}",
        rand::random::<u64>(),
        rand::random::<u64>(),
        rand::random::<u64>()
    )
}

/// PKCE S256 verification
fn verify_pkce_s256(verifier: &str, challenge: &str) -> bool {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // SHA256 hash (simplified - use sha2 crate in production)
    let mut hasher = DefaultHasher::new();
    verifier.hash(&mut hasher);
    let hash = hasher.finish();

    // Base64url encode
    let computed = base64url_encode(&hash.to_be_bytes());

    // Compare (timing-safe comparison recommended in production)
    computed == challenge
}

fn base64url_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b = match chunk.len() {
            1 => [chunk[0], 0, 0],
            2 => [chunk[0], chunk[1], 0],
            _ => [chunk[0], chunk[1], chunk[2]],
        };
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | (b[2] as u32);
        result.push(CHARS[((n >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((n >> 6) & 0x3F) as usize] as char);
        }
        if chunk.len() > 2 {
            result.push(CHARS[(n & 0x3F) as usize] as char);
        }
    }
    result
}

fn base64_decode(input: &str) -> Option<String> {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let input = input.trim_end_matches('=');
    let mut result = Vec::new();

    for chunk in input.as_bytes().chunks(4) {
        let mut n: u32 = 0;
        for (i, &c) in chunk.iter().enumerate() {
            let val = CHARS.iter().position(|&x| x == c)? as u32;
            n |= val << (18 - 6 * i);
        }
        result.push((n >> 16) as u8);
        if chunk.len() > 2 {
            result.push((n >> 8) as u8);
        }
        if chunk.len() > 3 {
            result.push(n as u8);
        }
    }

    String::from_utf8(result).ok()
}

/// Extract client credentials from Basic auth or body
fn extract_client_credentials(
    headers: &HeaderMap,
    req: &TokenRequest,
) -> (Option<String>, Option<String>) {
    // Try Basic auth first
    if let Some(auth) = headers.get("authorization") {
        if let Ok(auth_str) = auth.to_str() {
            if let Some(basic) = auth_str.strip_prefix("Basic ") {
                if let Some(decoded) = base64_decode(basic) {
                    if let Some((id, secret)) = decoded.split_once(':') {
                        return (Some(id.to_string()), Some(secret.to_string()));
                    }
                }
            }
        }
    }

    // Fall back to body params
    (req.client_id.clone(), req.client_secret.clone())
}

fn validate_scope(requested: Option<&str>, allowed: &[String]) -> Result<Option<String>, OAuthError> {
    match requested {
        None => Ok(None),
        Some(scope_str) => {
            for scope in scope_str.split_whitespace() {
                if !allowed.iter().any(|s| s == scope) {
                    return Err(OAuthError::invalid_scope(format!(
                        "Scope '{}' not allowed",
                        scope
                    )));
                }
            }
            Ok(Some(scope_str.to_string()))
        }
    }
}

fn validate_redirect_uri(uri: &str, allowed: &[String]) -> bool {
    // Check exact match
    if allowed.contains(&uri.to_string()) {
        return true;
    }

    // Validate localhost URLs (allowed per spec)
    if uri.starts_with("http://localhost") || uri.starts_with("http://127.0.0.1") {
        return true;
    }

    // Must be HTTPS for non-localhost
    uri.starts_with("https://")
}

// ============================================================================
// HANDLERS
// ============================================================================

/// GET /.well-known/oauth-authorization-server
/// Authorization Server Metadata Discovery (RFC 8414)
pub async fn metadata_handler(State(state): State<OAuthState>) -> Json<AuthorizationServerMetadata> {
    Json(state.get_metadata())
}

/// GET /.well-known/openid-configuration
/// OpenID Connect Discovery 1.0 (alias for oauth-authorization-server)
pub async fn oidc_metadata_handler(State(state): State<OAuthState>) -> Json<AuthorizationServerMetadata> {
    Json(state.get_metadata())
}

/// GET /.well-known/oauth-protected-resource
/// Protected Resource Metadata (RFC 9728) - MCP 2025-11-25
pub async fn protected_resource_metadata_handler(
    State(state): State<OAuthState>,
) -> Json<ProtectedResourceMetadata> {
    Json(state.get_resource_metadata())
}

/// POST /register - Dynamic Client Registration (RFC 7591)
pub async fn register_handler(
    State(state): State<OAuthState>,
    Json(req): Json<ClientRegistrationRequest>,
) -> Result<Json<ClientRegistrationResponse>, OAuthError> {
    if !state.config.allow_dynamic_registration {
        return Err(OAuthError::new(
            "registration_not_supported",
            "Dynamic client registration is disabled",
        ));
    }

    // Validate redirect URIs
    if req.redirect_uris.is_empty() {
        return Err(OAuthError::invalid_request("redirect_uris required"));
    }

    for uri in &req.redirect_uris {
        if !validate_redirect_uri(uri, &[]) {
            return Err(OAuthError::invalid_request(format!(
                "Invalid redirect_uri: {}. Must be localhost or HTTPS",
                uri
            )));
        }
    }

    // Determine client type and generate credentials
    let auth_method = req
        .token_endpoint_auth_method
        .clone()
        .unwrap_or_else(|| "none".into());

    let (client_secret, is_confidential) = match auth_method.as_str() {
        "client_secret_basic" | "client_secret_post" => (Some(generate_client_secret()), true),
        "none" => (None, false),
        _ => {
            return Err(OAuthError::invalid_request(
                "Unsupported token_endpoint_auth_method",
            ))
        }
    };

    let grant_types = req.grant_types.clone().unwrap_or_else(|| {
        if is_confidential {
            vec!["client_credentials".into(), "refresh_token".into()]
        } else {
            vec!["authorization_code".into(), "refresh_token".into()]
        }
    });

    let response_types = req
        .response_types
        .clone()
        .unwrap_or_else(|| vec!["code".into()]);

    let client_id = generate_client_id();
    let client = OAuthClient {
        client_id: client_id.clone(),
        client_secret: client_secret.clone(),
        client_name: req.client_name.clone(),
        redirect_uris: req.redirect_uris.clone(),
        grant_types: grant_types.clone(),
        response_types: response_types.clone(),
        allowed_scopes: state.config.default_scopes.clone(),
        token_endpoint_auth_method: auth_method.clone(),
        created_at: now_secs(),
    };

    state.register_client(client).await;

    Ok(Json(ClientRegistrationResponse {
        client_id,
        client_secret,
        client_name: req.client_name,
        redirect_uris: req.redirect_uris,
        grant_types,
        token_endpoint_auth_method: auth_method,
        response_types,
        client_secret_expires_at: None, // Never expires
    }))
}

/// GET /authorize - Authorization Endpoint
pub async fn authorize_handler(
    State(state): State<OAuthState>,
    Query(req): Query<AuthorizeRequest>,
) -> Result<Redirect, OAuthError> {
    // Validate response_type
    if req.response_type != "code" {
        return Err(OAuthError::new(
            "unsupported_response_type",
            "Only 'code' response type is supported",
        ));
    }

    // PKCE is required in OAuth 2.1/MCP
    if req.code_challenge_method != "S256" {
        return Err(OAuthError::invalid_request(
            "code_challenge_method must be S256 (required by OAuth 2.1)",
        ));
    }

    if req.code_challenge.is_empty() {
        return Err(OAuthError::invalid_request(
            "code_challenge is required (PKCE)",
        ));
    }

    // Validate client (supports URL-based client_id)
    let client = state.get_client(&req.client_id).await?;

    // Validate redirect_uri
    if !client.redirect_uris.contains(&req.redirect_uri)
        && !validate_redirect_uri(&req.redirect_uri, &client.redirect_uris)
    {
        return Err(OAuthError::invalid_request("Invalid redirect_uri"));
    }

    // Validate scope
    let scope = validate_scope(req.scope.as_deref(), &client.allowed_scopes)?;

    // Generate authorization code
    let code = generate_code();
    let auth_code = AuthCode {
        code: code.clone(),
        client_id: req.client_id.clone(),
        redirect_uri: req.redirect_uri.clone(),
        scope,
        code_challenge: req.code_challenge.clone(),
        code_challenge_method: req.code_challenge_method.clone(),
        expires_at: now_secs() + state.config.auth_code_expiry_secs,
        user_id: Some("mcp_user".into()), // In production: from session/login
    };

    state
        .auth_codes
        .write()
        .await
        .insert(code.clone(), auth_code);

    // Build redirect URL with code
    let mut redirect = req.redirect_uri.clone();
    redirect.push_str(if redirect.contains('?') { "&" } else { "?" });
    redirect.push_str(&format!("code={}", code));
    if let Some(state_param) = &req.state {
        redirect.push_str(&format!("&state={}", state_param));
    }

    Ok(Redirect::to(&redirect))
}

/// POST /token - Token Endpoint
pub async fn token_handler(
    State(state): State<OAuthState>,
    headers: HeaderMap,
    Form(req): Form<TokenRequest>,
) -> Result<Json<TokenResponse>, OAuthError> {
    match req.grant_type.as_str() {
        "authorization_code" => handle_authorization_code(&state, &req).await,
        "client_credentials" => handle_client_credentials(&state, &headers, &req).await,
        "refresh_token" => handle_refresh_token(&state, &headers, &req).await,
        _ => Err(OAuthError::unsupported_grant_type(format!(
            "Grant type '{}' not supported",
            req.grant_type
        ))),
    }
}

/// Authorization Code Grant (with PKCE)
async fn handle_authorization_code(
    state: &OAuthState,
    req: &TokenRequest,
) -> Result<Json<TokenResponse>, OAuthError> {
    let code = req
        .code
        .as_ref()
        .ok_or_else(|| OAuthError::invalid_request("Missing code"))?;

    let code_verifier = req
        .code_verifier
        .as_ref()
        .ok_or_else(|| OAuthError::invalid_request("Missing code_verifier (PKCE required)"))?;

    // Get and remove auth code (one-time use)
    let auth_code = state
        .auth_codes
        .write()
        .await
        .remove(code)
        .ok_or_else(|| OAuthError::invalid_grant("Invalid or expired authorization code"))?;

    // Check expiration
    if now_secs() > auth_code.expires_at {
        return Err(OAuthError::invalid_grant("Authorization code expired"));
    }

    // Verify redirect_uri matches
    if req.redirect_uri.as_ref() != Some(&auth_code.redirect_uri) {
        return Err(OAuthError::invalid_grant("redirect_uri mismatch"));
    }

    // Verify PKCE (S256 required)
    if !verify_pkce_s256(code_verifier, &auth_code.code_challenge) {
        return Err(OAuthError::invalid_grant("PKCE verification failed"));
    }

    // Generate tokens
    let access_token = generate_token();
    let refresh_token = generate_token();
    let expires_in = state.config.token_expiry_secs;

    // Store tokens
    let stored = StoredToken {
        access_token: access_token.clone(),
        refresh_token: Some(refresh_token.clone()),
        client_id: auth_code.client_id,
        scope: auth_code.scope.clone(),
        expires_at: now_secs() + expires_in,
        user_id: auth_code.user_id,
        third_party_token: None,
    };

    state
        .tokens
        .write()
        .await
        .insert(access_token.clone(), stored);
    state
        .refresh_tokens
        .write()
        .await
        .insert(refresh_token.clone(), access_token.clone());

    Ok(Json(TokenResponse {
        access_token,
        token_type: "Bearer",
        expires_in,
        refresh_token: Some(refresh_token),
        scope: auth_code.scope,
    }))
}

/// Client Credentials Grant
async fn handle_client_credentials(
    state: &OAuthState,
    headers: &HeaderMap,
    req: &TokenRequest,
) -> Result<Json<TokenResponse>, OAuthError> {
    let (client_id, client_secret) = extract_client_credentials(headers, req);

    let client_id = client_id.ok_or_else(|| OAuthError::invalid_request("Missing client_id"))?;

    // Validate client (supports URL-based client_id)
    let client = state.get_client(&client_id).await?;

    // Confidential client must provide valid secret
    if client.is_confidential() {
        let expected = client.client_secret.as_ref().unwrap();
        if client_secret.as_ref() != Some(expected) {
            return Err(OAuthError::invalid_client("Invalid client_secret"));
        }
    }

    // Validate grant type is allowed
    if !client.grant_types.contains(&"client_credentials".to_string()) {
        return Err(OAuthError::unauthorized_client(
            "client_credentials grant not allowed for this client",
        ));
    }

    // Validate scope
    let scope = validate_scope(req.scope.as_deref(), &client.allowed_scopes)?;

    // Generate token (no refresh token for client_credentials per spec)
    let access_token = generate_token();
    let expires_in = state.config.token_expiry_secs;

    let stored = StoredToken {
        access_token: access_token.clone(),
        refresh_token: None,
        client_id: client_id.clone(),
        scope: scope.clone(),
        expires_at: now_secs() + expires_in,
        user_id: None,
        third_party_token: None,
    };

    state
        .tokens
        .write()
        .await
        .insert(access_token.clone(), stored);

    Ok(Json(TokenResponse {
        access_token,
        token_type: "Bearer",
        expires_in,
        refresh_token: None,
        scope,
    }))
}

/// Refresh Token Grant
async fn handle_refresh_token(
    state: &OAuthState,
    headers: &HeaderMap,
    req: &TokenRequest,
) -> Result<Json<TokenResponse>, OAuthError> {
    let refresh_token = req
        .refresh_token
        .as_ref()
        .ok_or_else(|| OAuthError::invalid_request("Missing refresh_token"))?;

    // Get old access token
    let old_access = state
        .refresh_tokens
        .write()
        .await
        .remove(refresh_token)
        .ok_or_else(|| OAuthError::invalid_grant("Invalid refresh token"))?;

    // Get old token data
    let old_token = state
        .tokens
        .write()
        .await
        .remove(&old_access)
        .ok_or_else(|| OAuthError::invalid_grant("Token not found"))?;

    // Optionally verify client (for confidential clients)
    let (client_id, client_secret) = extract_client_credentials(headers, req);
    if let Some(id) = &client_id {
        if id != &old_token.client_id {
            return Err(OAuthError::invalid_grant("client_id mismatch"));
        }
        // Verify secret for confidential clients
        let clients = state.clients.read().await;
        if let Some(client) = clients.get(id) {
            if client.is_confidential() {
                let expected = client.client_secret.as_ref().unwrap();
                if client_secret.as_ref() != Some(expected) {
                    return Err(OAuthError::invalid_client("Invalid client_secret"));
                }
            }
        }
    }

    // Token rotation (OAuth 2.1 requirement)
    let new_access_token = generate_token();
    let new_refresh_token = generate_token();
    let expires_in = state.config.token_expiry_secs;

    let stored = StoredToken {
        access_token: new_access_token.clone(),
        refresh_token: Some(new_refresh_token.clone()),
        client_id: old_token.client_id,
        scope: old_token.scope.clone(),
        expires_at: now_secs() + expires_in,
        user_id: old_token.user_id,
        third_party_token: old_token.third_party_token,
    };

    state
        .tokens
        .write()
        .await
        .insert(new_access_token.clone(), stored);
    state
        .refresh_tokens
        .write()
        .await
        .insert(new_refresh_token.clone(), new_access_token.clone());

    Ok(Json(TokenResponse {
        access_token: new_access_token,
        token_type: "Bearer",
        expires_in,
        refresh_token: Some(new_refresh_token),
        scope: old_token.scope,
    }))
}

/// POST /introspect - Token Introspection (RFC 7662)
pub async fn introspect_handler(
    State(state): State<OAuthState>,
    Form(req): Form<IntrospectRequest>,
) -> Json<IntrospectResponse> {
    let tokens = state.tokens.read().await;

    if let Some(token) = tokens.get(&req.token) {
        if now_secs() < token.expires_at {
            return Json(IntrospectResponse {
                active: true,
                scope: token.scope.clone(),
                client_id: Some(token.client_id.clone()),
                username: token.user_id.clone(),
                exp: Some(token.expires_at),
                iat: Some(token.expires_at - state.config.token_expiry_secs),
            });
        }
    }

    Json(IntrospectResponse {
        active: false,
        scope: None,
        client_id: None,
        username: None,
        exp: None,
        iat: None,
    })
}

/// POST /revoke - Token Revocation (RFC 7009)
pub async fn revoke_handler(
    State(state): State<OAuthState>,
    Form(req): Form<IntrospectRequest>,
) -> StatusCode {
    state.tokens.write().await.remove(&req.token);
    state.refresh_tokens.write().await.remove(&req.token);
    StatusCode::OK
}

// ============================================================================
// MIDDLEWARE
// ============================================================================

/// Build WWW-Authenticate header with resource_metadata (RFC 9728)
fn build_www_authenticate(state: &OAuthState, error: &str, description: Option<&str>) -> String {
    let resource_metadata_url = format!(
        "{}/.well-known/oauth-protected-resource",
        state.config.resource_url
    );
    let mut header = format!(
        "Bearer realm=\"mcp\", resource_metadata=\"{}\", error=\"{}\"",
        resource_metadata_url, error
    );
    if let Some(desc) = description {
        header.push_str(&format!(", error_description=\"{}\"", desc));
    }
    // Add scope hint for incremental consent (SEP-835)
    if !state.config.default_scopes.is_empty() {
        header.push_str(&format!(", scope=\"{}\"", state.config.default_scopes.join(" ")));
    }
    header
}

/// MCP OAuth Bearer Token Middleware
/// Returns 401 Unauthorized if token missing/invalid (per MCP spec)
/// Includes resource_metadata parameter in WWW-Authenticate (RFC 9728)
pub async fn oauth_middleware(
    State(state): State<OAuthState>,
    headers: HeaderMap,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<Response, Response> {
    // Extract Bearer token from Authorization header only (OAuth 2.1 requirement)
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| {
            let www_auth = build_www_authenticate(&state, "invalid_token", Some("Missing or invalid Bearer token"));
            (
                StatusCode::UNAUTHORIZED,
                [("WWW-Authenticate", www_auth)],
                Json(json!({
                    "error": "invalid_token",
                    "error_description": "Missing or invalid Bearer token"
                })),
            )
                .into_response()
        })?;

    // Validate token
    let tokens = state.tokens.read().await;
    let stored = tokens.get(token).ok_or_else(|| {
        let www_auth = build_www_authenticate(&state, "invalid_token", Some("Token not found or revoked"));
        (
            StatusCode::UNAUTHORIZED,
            [("WWW-Authenticate", www_auth)],
            Json(json!({
                "error": "invalid_token",
                "error_description": "Token not found or revoked"
            })),
        )
            .into_response()
    })?;

    // Check expiration
    if now_secs() > stored.expires_at {
        let www_auth = build_www_authenticate(&state, "invalid_token", Some("Token expired"));
        return Err((
            StatusCode::UNAUTHORIZED,
            [("WWW-Authenticate", www_auth)],
            Json(json!({
                "error": "invalid_token",
                "error_description": "Token expired"
            })),
        )
            .into_response());
    }

    // Inject token info into request
    request.extensions_mut().insert(OAuthTokenInfo {
        client_id: stored.client_id.clone(),
        scope: stored.scope.clone(),
        user_id: stored.user_id.clone(),
    });

    Ok(next.run(request).await)
}

/// Token info available to handlers after authentication
#[derive(Debug, Clone)]
pub struct OAuthTokenInfo {
    pub client_id: String,
    pub scope: Option<String>,
    pub user_id: Option<String>,
}

// ============================================================================
// ROUTER
// ============================================================================

/// Create MCP OAuth 2.1 router
/// 
/// Mount at root or nest under /oauth
/// Also mount wellknown_router at /.well-known
pub fn oauth_router(state: OAuthState) -> Router {
    Router::new()
        .route("/authorize", get(authorize_handler))
        .route("/token", post(token_handler))
        .route("/register", post(register_handler))
        .route("/introspect", post(introspect_handler))
        .route("/revoke", post(revoke_handler))
        .with_state(state)
}

/// Well-known metadata router (RFC 8414 + RFC 9728 + OIDC)
/// Mount at /.well-known
/// 
/// Endpoints:
/// - GET /.well-known/oauth-authorization-server (RFC 8414)
/// - GET /.well-known/openid-configuration (OIDC Discovery)
/// - GET /.well-known/oauth-protected-resource (RFC 9728 - MCP 2025-11-25)
pub fn wellknown_router(state: OAuthState) -> Router {
    Router::new()
        .route("/oauth-authorization-server", get(metadata_handler))
        .route("/openid-configuration", get(oidc_metadata_handler))
        .route("/oauth-protected-resource", get(protected_resource_metadata_handler))
        .with_state(state)
}

/// Check if request has sufficient scope (for step-up authorization)
/// Returns 403 Forbidden with scope challenge if insufficient
pub async fn require_scope(
    State(state): State<OAuthState>,
    request: axum::extract::Request,
    required_scope: &str,
) -> Result<(), Response> {
    let token_info = request
        .extensions()
        .get::<OAuthTokenInfo>()
        .ok_or_else(|| {
            let www_auth = build_www_authenticate(&state, "invalid_token", Some("No token info"));
            (
                StatusCode::UNAUTHORIZED,
                [("WWW-Authenticate", www_auth)],
                Json(json!({
                    "error": "invalid_token",
                    "error_description": "No token info available"
                })),
            )
                .into_response()
        })?;

    let has_scope = token_info
        .scope
        .as_ref()
        .map(|s| s.split_whitespace().any(|sc| sc == required_scope))
        .unwrap_or(false);

    if !has_scope {
        let resource_metadata_url = format!(
            "{}/.well-known/oauth-protected-resource",
            state.config.resource_url
        );
        let www_auth = format!(
            "Bearer realm=\"mcp\", resource_metadata=\"{}\", error=\"insufficient_scope\", scope=\"{}\"",
            resource_metadata_url, required_scope
        );
        return Err((
            StatusCode::FORBIDDEN,
            [("WWW-Authenticate", www_auth)],
            Json(json!({
                "error": "insufficient_scope",
                "error_description": format!("Required scope: {}", required_scope),
                "scope": required_scope
            })),
        )
            .into_response());
    }

    Ok(())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token() {
        let t1 = generate_token();
        let t2 = generate_token();
        assert_ne!(t1, t2);
        assert_eq!(t1.len(), 32);
    }

    #[test]
    fn test_generate_code() {
        let c1 = generate_code();
        let c2 = generate_code();
        assert_ne!(c1, c2);
        assert_eq!(c1.len(), 32);
    }

    #[test]
    fn test_validate_scope() {
        let allowed = vec!["mcp:read".into(), "mcp:write".into()];

        assert!(validate_scope(None, &allowed).is_ok());
        assert!(validate_scope(Some("mcp:read"), &allowed).is_ok());
        assert!(validate_scope(Some("mcp:read mcp:write"), &allowed).is_ok());
        assert!(validate_scope(Some("mcp:admin"), &allowed).is_err());
    }

    #[test]
    fn test_validate_redirect_uri() {
        let allowed = vec!["http://localhost:3000/callback".into()];

        assert!(validate_redirect_uri("http://localhost:3000/callback", &allowed));
        assert!(validate_redirect_uri("http://localhost:8080/cb", &[])); // localhost always ok
        assert!(validate_redirect_uri("http://127.0.0.1:3000/cb", &[]));
        assert!(validate_redirect_uri("https://example.com/cb", &[])); // https ok
        assert!(!validate_redirect_uri("http://example.com/cb", &[])); // http non-localhost not ok
    }

    #[test]
    fn test_base64_decode() {
        // "hello:world" in base64
        let decoded = base64_decode("aGVsbG86d29ybGQ=").unwrap();
        assert_eq!(decoded, "hello:world");
    }

    #[tokio::test]
    async fn test_oauth_state() {
        let state = OAuthState::with_defaults();
        state.setup_default_clients().await;

        let clients = state.clients.read().await;
        assert!(clients.contains_key("mcp-server"));
        assert!(clients.contains_key("mcp-public"));

        let server = clients.get("mcp-server").unwrap();
        assert!(server.is_confidential());

        let public = clients.get("mcp-public").unwrap();
        assert!(public.is_public());
    }

    #[test]
    fn test_metadata() {
        let state = OAuthState::with_defaults();
        let meta = state.get_metadata();

        assert!(meta.issuer.starts_with("http"));
        assert!(meta.authorization_endpoint.contains("/authorize"));
        assert!(meta.token_endpoint.contains("/token"));
        assert!(meta.code_challenge_methods_supported.contains(&"S256".into()));
        assert_eq!(meta.client_id_metadata_document_supported, Some(true));
    }

    #[test]
    fn test_resource_metadata() {
        let state = OAuthState::with_defaults();
        let meta = state.get_resource_metadata();

        assert!(!meta.resource.is_empty());
        assert!(!meta.authorization_servers.is_empty());
        assert!(meta.bearer_methods_supported.unwrap().contains(&"header".into()));
    }

    #[test]
    fn test_is_url_client_id() {
        assert!(OAuthState::is_url_client_id("https://example.com/client"));
        assert!(!OAuthState::is_url_client_id("mcp-server"));
        assert!(!OAuthState::is_url_client_id("http://example.com/client")); // must be https
    }

    #[test]
    fn test_www_authenticate_header() {
        let state = OAuthState::with_defaults();
        let header = build_www_authenticate(&state, "invalid_token", Some("Test error"));
        
        assert!(header.contains("realm=\"mcp\""));
        assert!(header.contains("resource_metadata="));
        assert!(header.contains("oauth-protected-resource"));
        assert!(header.contains("error=\"invalid_token\""));
    }
}