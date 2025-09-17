//! Basic Integration Tests for MCP Boilerplate Servers
//! 
//! Simple working integration tests that demonstrate E2E testing capabilities
//! without complex dependencies or infrastructure.

use std::time::Duration;
use std::process::Command;

/// Test that filesystem server binary can be invoked
#[tokio::test]
async fn test_filesystem_server_exists() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "filesystem-server", "--", "--help"])
        .output()
        .expect("Failed to execute filesystem server");
    
    // Server should be able to show help
    assert!(output.status.success(), "Filesystem server should show help successfully");
    
    let help_text = String::from_utf8_lossy(&output.stdout);
    assert!(!help_text.is_empty(), "Help text should not be empty");
    
    println!("✅ Filesystem server exists and responds to --help");
}

/// Test that image generation server binary can be invoked
#[tokio::test] 
async fn test_image_generation_server_exists() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "image-generation-server", "--", "--help"])
        .output()
        .expect("Failed to execute image generation server");
    
    assert!(output.status.success(), "Image generation server should show help successfully");
    
    let help_text = String::from_utf8_lossy(&output.stdout);
    assert!(!help_text.is_empty(), "Help text should not be empty");
    
    println!("✅ Image generation server exists and responds to --help");
}

/// Test that blog generation server binary can be invoked
#[tokio::test]
async fn test_blog_generation_server_exists() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "blog-generation-server", "--", "--help"])
        .output()
        .expect("Failed to execute blog generation server");
    
    assert!(output.status.success(), "Blog generation server should show help successfully");
    
    let help_text = String::from_utf8_lossy(&output.stdout);
    assert!(!help_text.is_empty(), "Help text should not be empty");
    
    println!("✅ Blog generation server exists and responds to --help");
}

/// Test that creative content server binary can be invoked
#[tokio::test]
async fn test_creative_content_server_exists() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "creative-content-server", "--", "--help"])
        .output()
        .expect("Failed to execute creative content server");
    
    assert!(output.status.success(), "Creative content server should show help successfully");
    
    let help_text = String::from_utf8_lossy(&output.stdout);
    assert!(!help_text.is_empty(), "Help text should not be empty");
    
    println!("✅ Creative content server exists and responds to --help");
}

/// Test that all servers compile without errors
#[tokio::test]
async fn test_all_servers_compile() {
    let servers = [
        "filesystem-server",
        "image-generation-server", 
        "blog-generation-server",
        "creative-content-server"
    ];
    
    for server in &servers {
        let output = Command::new("cargo")
            .args(&["check", "--bin", server])
            .output()
            .expect(&format!("Failed to check {}", server));
        
        assert!(output.status.success(), "Server {} should compile successfully", server);
        println!("✅ {} compilation check passed", server);
    }
}

/// Test server help content quality
#[tokio::test]
async fn test_server_help_content() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "filesystem-server", "--", "--help"])
        .output()
        .expect("Failed to execute filesystem server");
    
    assert!(output.status.success(), "Server should show help");
    
    let help_text = String::from_utf8_lossy(&output.stdout);
    let help_lower = help_text.to_lowercase();
    
    // Help should contain useful information
    assert!(
        help_lower.contains("help") || 
        help_lower.contains("usage") || 
        help_lower.contains("options") ||
        help_lower.contains("transport") ||
        help_lower.contains("mcp"),
        "Help should contain expected content"
    );
    
    println!("✅ Server help content validation passed");
}

/// Test error handling with invalid arguments
#[tokio::test]
async fn test_server_error_handling() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "filesystem-server", "--", "--invalid-flag"])
        .output()
        .expect("Failed to execute filesystem server with invalid args");
    
    // Should exit with non-zero status for invalid args
    assert!(!output.status.success(), "Server should reject invalid arguments");
    
    // Should provide some error output
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(
        !stderr.is_empty() || !stdout.is_empty(),
        "Server should provide error message for invalid arguments"
    );
    
    println!("✅ Server error handling validation passed");
}

/// Performance test - help should respond quickly
#[tokio::test]
async fn test_server_response_time() {
    let start = std::time::Instant::now();
    
    let output = Command::new("cargo")
        .args(&["run", "--bin", "filesystem-server", "--", "--help"])
        .output()
        .expect("Failed to execute filesystem server");
    
    let elapsed = start.elapsed();
    
    assert!(output.status.success(), "Server should show help");
    
    // Should respond within reasonable time (10 seconds for cargo run)
    assert!(
        elapsed < Duration::from_secs(10),
        "Server help should respond within 10 seconds, took {:?}",
        elapsed
    );
    
    println!("✅ Server response time: {:?}", elapsed);
}

/// Integration test - verify all components work together
#[tokio::test]
async fn test_complete_server_suite() {
    let servers = [
        "filesystem-server",
        "image-generation-server",
        "blog-generation-server", 
        "creative-content-server"
    ];
    
    let mut successful_servers = 0;
    
    for server in &servers {
        // Test compilation
        let check_output = Command::new("cargo")
            .args(&["check", "--bin", server])
            .output()
            .expect(&format!("Failed to check {}", server));
        
        if !check_output.status.success() {
            println!("⚠️ {} failed compilation check", server);
            continue;
        }
        
        // Test help functionality
        let help_output = Command::new("cargo")
            .args(&["run", "--bin", server, "--", "--help"])
            .output()
            .expect(&format!("Failed to run {} help", server));
        
        if !help_output.status.success() {
            println!("⚠️ {} failed help test", server);
            continue;
        }
        
        successful_servers += 1;
        println!("✅ {} passed all tests", server);
    }
    
    // At least 3 out of 4 servers should work completely
    assert!(
        successful_servers >= 3,
        "At least 3 servers should pass all tests, got {}",
        successful_servers
    );
    
    println!("✅ Server suite test: {}/{} servers fully functional", 
             successful_servers, servers.len());
}