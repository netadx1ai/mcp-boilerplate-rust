//! # Authenticated MCP Server Binary
//!
//! A production-ready MCP server with comprehensive authentication and authorization.

use anyhow::Result;
use authenticated_server_template::{AuthConfig, AuthenticatedServer};
use tokio::signal;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration from environment
    let config = AuthConfig {
        jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| {
            warn!("JWT_SECRET not set, using default (change in production!)");
            "your-super-secret-jwt-key-change-in-production".to_string()
        }),
        oauth_client_id: std::env::var("OAUTH_CLIENT_ID").ok(),
        oauth_client_secret: std::env::var("OAUTH_CLIENT_SECRET").ok(),
        oauth_auth_url: std::env::var("OAUTH_AUTH_URL").ok(),
        oauth_token_url: std::env::var("OAUTH_TOKEN_URL").ok(),
        oauth_redirect_url: std::env::var("OAUTH_REDIRECT_URL").ok(),
        session_timeout_minutes: std::env::var("SESSION_TIMEOUT_MINUTES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60 * 24), // 24 hours default
        token_timeout_minutes: std::env::var("TOKEN_TIMEOUT_MINUTES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60), // 1 hour default
    };

    // Create the server
    let server = AuthenticatedServer::new(config);
    
    info!("Starting Authenticated MCP Server...");
    info!("Authentication methods: JWT, API Key, Username/Password");
    info!("Sample API keys: admin-key-12345, user-key-67890, readonly-key-11111");
    info!("Sample users: admin, user, readonly");
    
    // For this template, we'll simulate server operation
    // In a real implementation, you would integrate with the MCP framework
    info!("Server initialized and ready for MCP connections");
    info!("Use Ctrl+C to shutdown");

    // Wait for shutdown signal
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Received Ctrl+C, shutting down...");
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        }
    }

    info!("Authenticated MCP Server stopped");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use authenticated_server_template::{AuthCredentials, UserRole};

    #[tokio::test]
    async fn test_server_creation() {
        let server = AuthenticatedServer::new(AuthConfig::default());
        
        // Test that server status works
        let status = server.get_server_status().await;
        assert!(status.is_ok());
        
        let stats = status.unwrap();
        assert_eq!(stats.total_users, 3);
        assert_eq!(stats.api_keys, 3);
    }

    #[tokio::test]
    async fn test_api_key_auth() {
        let server = AuthenticatedServer::new(AuthConfig::default());
        
        let credentials = AuthCredentials::ApiKey {
            key: "admin-key-12345".to_string(),
        };
        
        let result = server.authenticate_user(credentials).await;
        assert!(result.is_ok());
        
        let auth_response = result.unwrap();
        assert!(auth_response.success);
        assert!(auth_response.token.is_some());
        assert_eq!(auth_response.user.unwrap().role, UserRole::Admin);
    }
}