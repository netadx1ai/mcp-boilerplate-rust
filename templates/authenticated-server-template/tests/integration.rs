//! Integration tests for the authenticated MCP server template
//!
//! These tests verify the complete authentication flow and MCP tool functionality.

use authenticated_server_template::{
    AuthCredentials, AuthenticatedServer, AuthConfig, PermissionCheck, User, UserRole,
};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

/// Create a test server with default configuration
fn create_test_server() -> AuthenticatedServer {
    let config = AuthConfig {
        jwt_secret: "test-secret-key-for-integration-tests".to_string(),
        oauth_client_id: Some("test-client-id".to_string()),
        oauth_client_secret: Some("test-client-secret".to_string()),
        oauth_auth_url: Some("https://test.example.com/oauth/authorize".to_string()),
        oauth_token_url: Some("https://test.example.com/oauth/token".to_string()),
        oauth_redirect_url: Some("http://localhost:8080/auth/callback".to_string()),
        session_timeout_minutes: 30,
        token_timeout_minutes: 15,
    };
    
    AuthenticatedServer::new(config)
}

#[tokio::test]
async fn test_complete_api_key_flow() {
    let server = create_test_server();
    
    // Test API key authentication
    let credentials = AuthCredentials::ApiKey {
        key: "admin-key-12345".to_string(),
    };
    
    let auth_result = server.authenticate_user(credentials).await;
    assert!(auth_result.is_ok(), "API key authentication should succeed");
    
    let auth_response = auth_result.unwrap();
    assert!(auth_response.success, "Authentication should be successful");
    assert!(auth_response.token.is_some(), "Token should be provided");
    assert!(auth_response.user.is_some(), "User info should be provided");
    
    let user = auth_response.user.unwrap();
    assert_eq!(user.username, "admin");
    assert_eq!(user.role, UserRole::Admin);
    
    // Test token validation
    let token = auth_response.token.unwrap();
    let validation_result = server.validate_token(token.clone()).await;
    assert!(validation_result.is_ok(), "Token validation should succeed");
    
    let validation = validation_result.unwrap();
    assert!(validation.valid, "Token should be valid");
    assert_eq!(validation.username.unwrap(), "admin");
    
    // Test user profile retrieval
    let profile_result = server.get_user_profile(token.clone()).await;
    assert!(profile_result.is_ok(), "Profile retrieval should succeed");
    
    let profile = profile_result.unwrap();
    assert_eq!(profile.username, "admin");
    assert_eq!(profile.role, UserRole::Admin);
    
    // Test permission check
    let permission_check = PermissionCheck {
        user_id: profile.id,
        required_permission: "admin".to_string(),
    };
    
    let permission_result = server.check_permission(permission_check).await;
    assert!(permission_result.is_ok(), "Permission check should succeed");
    assert!(permission_result.unwrap(), "Admin should have admin permission");
    
    // Test logout
    let logout_result = server.logout(token).await;
    assert!(logout_result.is_ok(), "Logout should succeed");
    assert!(logout_result.unwrap(), "Logout should return true");
}

#[tokio::test]
async fn test_username_password_authentication() {
    let server = create_test_server();
    
    let credentials = AuthCredentials::UsernamePassword {
        username: "user".to_string(),
        password: "password".to_string(),
    };
    
    let auth_result = server.authenticate_user(credentials).await;
    assert!(auth_result.is_ok(), "Username/password authentication should succeed");
    
    let auth_response = auth_result.unwrap();
    assert!(auth_response.success);
    assert!(auth_response.token.is_some());
    assert!(auth_response.refresh_token.is_some());
    
    let user = auth_response.user.unwrap();
    assert_eq!(user.username, "user");
    assert_eq!(user.role, UserRole::User);
    assert!(user.permissions.contains(&"read".to_string()));
    assert!(user.permissions.contains(&"write".to_string()));
    assert!(!user.permissions.contains(&"admin".to_string()));
}

#[tokio::test]
async fn test_token_refresh_flow() {
    let server = create_test_server();
    
    // First authenticate
    let credentials = AuthCredentials::ApiKey {
        key: "user-key-67890".to_string(),
    };
    
    let auth_result = server.authenticate_user(credentials).await;
    assert!(auth_result.is_ok());
    
    let auth_response = auth_result.unwrap();
    let refresh_token = auth_response.refresh_token.unwrap();
    
    // Test token refresh
    let refresh_result = server.refresh_token(refresh_token.clone()).await;
    assert!(refresh_result.is_ok(), "Token refresh should succeed");
    
    let new_auth = refresh_result.unwrap();
    assert!(new_auth.success);
    assert!(new_auth.token.is_some());
    assert_eq!(new_auth.refresh_token.unwrap(), refresh_token); // Should keep same refresh token
    
    // Verify new token works
    let new_token = new_auth.token.unwrap();
    let validation_result = server.validate_token(new_token).await;
    assert!(validation_result.is_ok());
    assert!(validation_result.unwrap().valid);
}

#[tokio::test]
async fn test_permission_system() {
    let server = create_test_server();
    
    // Test different user roles and permissions
    let test_cases = vec![
        ("admin-key-12345", "admin", UserRole::Admin, vec!["read", "write", "delete", "admin"]),
        ("user-key-67890", "user", UserRole::User, vec!["read", "write"]),
        ("readonly-key-11111", "readonly", UserRole::ReadOnly, vec!["read"]),
    ];
    
    for (api_key, username, expected_role, expected_permissions) in test_cases {
        let credentials = AuthCredentials::ApiKey {
            key: api_key.to_string(),
        };
        
        let auth_result = server.authenticate_user(credentials).await;
        assert!(auth_result.is_ok(), "Authentication should succeed for {}", username);
        
        let auth_response = auth_result.unwrap();
        let user = auth_response.user.unwrap();
        
        assert_eq!(user.username, username);
        assert_eq!(user.role, expected_role);
        
        // Test permissions
        let permissions_result = server.list_permissions(user.id).await;
        assert!(permissions_result.is_ok());
        
        let permissions = permissions_result.unwrap();
        for expected_perm in expected_permissions {
            assert!(permissions.contains(&expected_perm.to_string()), 
                   "{} should have {} permission", username, expected_perm);
        }
        
        // Test specific permission checks
        for permission in &permissions {
            let check = PermissionCheck {
                user_id: user.id,
                required_permission: permission.clone(),
            };
            
            let check_result = server.check_permission(check).await;
            assert!(check_result.is_ok());
            assert!(check_result.unwrap(), "{} should have {} permission", username, permission);
        }
        
        // Test permission user doesn't have
        if username != "admin" {
            let check = PermissionCheck {
                user_id: user.id,
                required_permission: "admin".to_string(),
            };
            
            let check_result = server.check_permission(check).await;
            assert!(check_result.is_ok());
            assert!(!check_result.unwrap(), "{} should NOT have admin permission", username);
        }
    }
}

#[tokio::test]
async fn test_invalid_credentials() {
    let server = create_test_server();
    
    // Test invalid API key
    let credentials = AuthCredentials::ApiKey {
        key: "invalid-key".to_string(),
    };
    
    let auth_result = server.authenticate_user(credentials).await;
    assert!(auth_result.is_err(), "Invalid API key should fail");
    
    // Test invalid username
    let credentials = AuthCredentials::UsernamePassword {
        username: "nonexistent".to_string(),
        password: "password".to_string(),
    };
    
    let auth_result = server.authenticate_user(credentials).await;
    assert!(auth_result.is_err(), "Invalid username should fail");
    
    // Test invalid JWT token
    let credentials = AuthCredentials::Jwt {
        token: "invalid.jwt.token".to_string(),
    };
    
    let auth_result = server.authenticate_user(credentials).await;
    assert!(auth_result.is_err(), "Invalid JWT should fail");
}

#[tokio::test]
async fn test_token_validation_edge_cases() {
    let server = create_test_server();
    
    // Test completely invalid token
    let validation_result = server.validate_token("not-a-jwt-token".to_string()).await;
    assert!(validation_result.is_ok()); // Should return TokenValidation with valid=false
    
    let validation = validation_result.unwrap();
    assert!(!validation.valid);
    assert!(validation.user_id.is_none());
    assert!(validation.username.is_none());
    
    // Test empty token
    let validation_result = server.validate_token("".to_string()).await;
    assert!(validation_result.is_ok());
    assert!(!validation_result.unwrap().valid);
    
    // Test malformed JWT
    let validation_result = server.validate_token("eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.invalid.signature".to_string()).await;
    assert!(validation_result.is_ok());
    assert!(!validation_result.unwrap().valid);
}

#[tokio::test]
async fn test_session_management() {
    let server = create_test_server();
    
    // Create multiple sessions
    let mut session_ids = Vec::new();
    
    for i in 0..3 {
        let credentials = AuthCredentials::ApiKey {
            key: format!("{}-key-{}", if i == 0 { "admin" } else { "user" }, if i == 0 { "12345" } else { "67890" }),
        };
        
        let auth_result = server.authenticate_user(credentials).await;
        assert!(auth_result.is_ok());
        
        let auth_response = auth_result.unwrap();
        if let Some(session_id) = auth_response.session_id {
            session_ids.push(session_id);
        }
    }
    
    // Check server status shows active sessions
    let status_result = server.get_server_status().await;
    assert!(status_result.is_ok());
    
    let status = status_result.unwrap();
    assert!(status.active_sessions > 0);
    assert!(status.successful_authentications >= 3);
    assert_eq!(status.total_users, 3); // admin, user, readonly
}

#[tokio::test]
async fn test_server_statistics() {
    let server = create_test_server();
    
    // Perform several operations to generate statistics
    let operations = vec![
        ("admin-key-12345", true),
        ("user-key-67890", true),
        ("invalid-key", false),
        ("readonly-key-11111", true),
        ("another-invalid-key", false),
    ];
    
    for (key, should_succeed) in operations {
        let credentials = AuthCredentials::ApiKey {
            key: key.to_string(),
        };
        
        let auth_result = server.authenticate_user(credentials).await;
        assert_eq!(auth_result.is_ok(), should_succeed);
    }
    
    // Check statistics
    let status_result = server.get_server_status().await;
    assert!(status_result.is_ok());
    
    let status = status_result.unwrap();
    assert_eq!(status.total_users, 3);
    assert_eq!(status.api_keys, 3);
    assert_eq!(status.authentication_attempts, 5);
    assert_eq!(status.successful_authentications, 3);
    assert_eq!(status.failed_authentications, 2);
    assert!(status.uptime_seconds >= 0);
}

#[tokio::test]
async fn test_concurrent_authentication() {
    let server = create_test_server();
    
    // Test concurrent authentication requests
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let server_clone = server.clone();
        let handle = tokio::spawn(async move {
            let key = if i % 3 == 0 {
                "admin-key-12345"
            } else if i % 3 == 1 {
                "user-key-67890"
            } else {
                "readonly-key-11111"
            };
            
            let credentials = AuthCredentials::ApiKey {
                key: key.to_string(),
            };
            
            server_clone.authenticate_user(credentials).await
        });
        
        handles.push(handle);
    }
    
    // Wait for all authentications to complete
    let mut successful = 0;
    for handle in handles {
        let result = timeout(Duration::from_secs(5), handle).await;
        assert!(result.is_ok(), "Authentication should not timeout");
        
        let auth_result = result.unwrap().unwrap();
        if auth_result.is_ok() {
            successful += 1;
        }
    }
    
    assert_eq!(successful, 10, "All concurrent authentications should succeed");
    
    // Verify server state is consistent
    let status = server.get_server_status().await.unwrap();
    assert!(status.active_sessions > 0);
    assert!(status.successful_authentications >= 10);
}

#[tokio::test]
async fn test_oauth_placeholder() {
    let server = create_test_server();
    
    // Test OAuth flow (currently returns error as it's not fully implemented)
    let credentials = AuthCredentials::OAuth {
        authorization_code: "test-code".to_string(),
        state: "test-state".to_string(),
    };
    
    let auth_result = server.authenticate_user(credentials).await;
    assert!(auth_result.is_err(), "OAuth flow should return error (not implemented)");
}

#[tokio::test]
async fn test_refresh_token_edge_cases() {
    let server = create_test_server();
    
    // Test with invalid refresh token
    let refresh_result = server.refresh_token("invalid-refresh-token".to_string()).await;
    assert!(refresh_result.is_err(), "Invalid refresh token should fail");
    
    // Test with empty refresh token
    let refresh_result = server.refresh_token("".to_string()).await;
    assert!(refresh_result.is_err(), "Empty refresh token should fail");
}

#[tokio::test]
async fn test_user_profile_with_invalid_token() {
    let server = create_test_server();
    
    // Test with invalid token
    let profile_result = server.get_user_profile("invalid-token".to_string()).await;
    assert!(profile_result.is_err(), "Invalid token should fail profile retrieval");
    
    // Test with empty token
    let profile_result = server.get_user_profile("".to_string()).await;
    assert!(profile_result.is_err(), "Empty token should fail profile retrieval");
}

#[tokio::test]
async fn test_permission_check_with_invalid_user() {
    let server = create_test_server();
    
    // Test permission check with non-existent user
    let check = PermissionCheck {
        user_id: Uuid::new_v4(), // Random UUID that doesn't exist
        required_permission: "read".to_string(),
    };
    
    let check_result = server.check_permission(check).await;
    assert!(check_result.is_err(), "Permission check with invalid user should fail");
}

#[tokio::test]
async fn test_list_permissions_with_invalid_user() {
    let server = create_test_server();
    
    // Test listing permissions for non-existent user
    let permissions_result = server.list_permissions(Uuid::new_v4()).await;
    assert!(permissions_result.is_err(), "Listing permissions for invalid user should fail");
}

#[tokio::test]
async fn test_performance_under_load() {
    let server = create_test_server();
    
    // Measure authentication performance
    let start = std::time::Instant::now();
    
    for _ in 0..100 {
        let credentials = AuthCredentials::ApiKey {
            key: "admin-key-12345".to_string(),
        };
        
        let result = server.authenticate_user(credentials).await;
        assert!(result.is_ok());
    }
    
    let duration = start.elapsed();
    assert!(duration.as_millis() < 1000, "100 authentications should complete in < 1s, took {}ms", duration.as_millis());
    
    // Measure token validation performance
    let credentials = AuthCredentials::ApiKey {
        key: "admin-key-12345".to_string(),
    };
    let auth_result = server.authenticate_user(credentials).await.unwrap();
    let token = auth_result.token.unwrap();
    
    let start = std::time::Instant::now();
    
    for _ in 0..100 {
        let result = server.validate_token(token.clone()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().valid);
    }
    
    let duration = start.elapsed();
    assert!(duration.as_millis() < 500, "100 token validations should complete in < 0.5s, took {}ms", duration.as_millis());
}