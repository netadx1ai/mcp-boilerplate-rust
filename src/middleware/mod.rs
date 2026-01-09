//! Middleware Module
//!
//! Authentication and authorization middleware for MCP server.
//!
//! Features:
//! - JWT Authentication (simple, for basic use cases)
//! - OAuth 2.1 Authorization (MCP spec compliant)
//! - Protected Resource Metadata (RFC 9728) - MCP 2025-11-25
//! - OpenID Connect Discovery Support
//! - Client ID Metadata Documents
//!
//! Last Updated: 2026-01-09 13:53 HCMC

// JWT Authentication (simple)
#[cfg(feature = "auth")]
pub mod auth;

#[cfg(feature = "auth")]
pub use auth::{
    Auth,
    AuthError,
    AuthMiddleware,
    Claims,
    LoginRequest,
    TokenResponse,
    auth_middleware,
    auth_router,
    login_handler,
    me_handler,
    optional_auth_middleware,
    verify_handler,
};

// OAuth 2.1 Authorization (MCP spec)
#[cfg(feature = "auth")]
pub mod oauth;

#[cfg(feature = "auth")]
pub use oauth::{
    // State
    OAuthState,
    OAuthConfig,
    OAuthClient,
    // Request/Response types
    AuthorizeRequest,
    TokenRequest,
    TokenResponse as OAuthTokenResponse,
    IntrospectRequest,
    IntrospectResponse,
    ClientRegistrationRequest,
    ClientRegistrationResponse,
    AuthorizationServerMetadata,
    OAuthError,
    OAuthTokenInfo,
    // RFC 9728 - Protected Resource Metadata (MCP 2025-11-25)
    ProtectedResourceMetadata,
    // Client ID Metadata Document (MCP 2025-11-25)
    ClientIdMetadataDocument,
    // Handlers
    authorize_handler,
    token_handler,
    register_handler,
    introspect_handler,
    revoke_handler,
    metadata_handler,
    oidc_metadata_handler,
    protected_resource_metadata_handler,
    // Middleware
    oauth_middleware,
    require_scope,
    // Routers
    oauth_router,
    wellknown_router,
};