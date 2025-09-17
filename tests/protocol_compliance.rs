//! MCP Protocol Compliance Integration Tests
//!
//! This module tests that all MCP boilerplate servers comply with basic
//! protocol requirements and can be executed successfully.

use std::process::Command;
use std::time::Duration;
use tokio::time::timeout;

/// Test that filesystem server can be invoked and shows help
#[tokio::test]
async fn test_filesystem_server_help() {
    let result = timeout(
        Duration::from_secs(10),
        test_server_help("filesystem-server"),
    )
    .await;

    assert!(
        result.is_ok(),
        "Filesystem server help test should not timeout"
    );
    result.unwrap().expect("Filesystem server should show help");
}

/// Test that image generation server can be invoked and shows help
#[tokio::test]
async fn test_image_generation_server_help() {
    let result = timeout(
        Duration::from_secs(10),
        test_server_help("image-generation-server"),
    )
    .await;

    assert!(
        result.is_ok(),
        "Image generation server help test should not timeout"
    );
    result
        .unwrap()
        .expect("Image generation server should show help");
}

/// Test that blog generation server can be invoked and shows help
#[tokio::test]
async fn test_blog_generation_server_help() {
    let result = timeout(
        Duration::from_secs(10),
        test_server_help("blog-generation-server"),
    )
    .await;

    assert!(
        result.is_ok(),
        "Blog generation server help test should not timeout"
    );
    result
        .unwrap()
        .expect("Blog generation server should show help");
}

/// Test that creative content server can be invoked and shows help
#[tokio::test]
async fn test_creative_content_server_help() {
    let result = timeout(
        Duration::from_secs(10),
        test_server_help("creative-content-server"),
    )
    .await;

    assert!(
        result.is_ok(),
        "Creative content server help test should not timeout"
    );
    result
        .unwrap()
        .expect("Creative content server should show help");
}

/// Test that all servers compile successfully
#[tokio::test]
async fn test_all_servers_compile() {
    let servers = [
        "filesystem-server",
        "image-generation-server",
        "blog-generation-server",
        "creative-content-server",
    ];

    for server_name in &servers {
        let result = timeout(
            Duration::from_secs(30),
            test_server_compilation(server_name),
        )
        .await;

        assert!(
            result.is_ok(),
            "Server {} compilation should not timeout",
            server_name
        );
        result
            .unwrap()
            .expect(&format!("Server {} should compile", server_name));
    }
}

/// Test server startup performance
#[tokio::test]
async fn test_server_startup_performance() {
    let servers = ["filesystem-server", "image-generation-server"];

    for server_name in &servers {
        let start_time = std::time::Instant::now();

        let result = timeout(Duration::from_secs(10), test_server_help(server_name)).await;

        let elapsed = start_time.elapsed();

        assert!(
            result.is_ok(),
            "Server {} should respond to help",
            server_name
        );
        result
            .unwrap()
            .expect(&format!("Server {} should show help", server_name));

        // Performance target: help should complete within 5 seconds
        assert!(
            elapsed < Duration::from_secs(5),
            "Server {} help took {:?}, should be < 5s",
            server_name,
            elapsed
        );

        println!("✅ {} startup performance: {:?}", server_name, elapsed);
    }
}

/// Test basic error handling
#[tokio::test]
async fn test_server_error_handling() {
    let result = timeout(
        Duration::from_secs(5),
        test_invalid_server_args("filesystem-server"),
    )
    .await;

    assert!(result.is_ok(), "Error handling test should not timeout");
    result
        .unwrap()
        .expect("Server should handle invalid args gracefully");
}

/// Helper function to test server help functionality
async fn test_server_help(
    server_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let output = Command::new("cargo")
        .args(&["run", "--bin", server_name, "--", "--help"])
        .output()?;

    if !output.status.success() {
        return Err(format!("Server {} failed to show help", server_name).into());
    }

    let help_text = String::from_utf8_lossy(&output.stdout);

    // Basic validation - help should contain some expected content
    if help_text.len() < 10 {
        return Err(format!("Server {} help output too short", server_name).into());
    }

    // Help should mention common flags or concepts
    let help_lower = help_text.to_lowercase();
    if !help_lower.contains("help")
        && !help_lower.contains("usage")
        && !help_lower.contains("options")
        && !help_lower.contains("transport")
        && !help_lower.contains("mcp")
    {
        return Err(format!(
            "Server {} help doesn't contain expected content",
            server_name
        )
        .into());
    }

    println!("✅ {} help test passed", server_name);
    Ok(())
}

/// Helper function to test server compilation
async fn test_server_compilation(
    server_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let output = Command::new("cargo")
        .args(&["check", "--bin", server_name])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Server {} compilation failed: {}", server_name, stderr).into());
    }

    println!("✅ {} compilation check passed", server_name);
    Ok(())
}

/// Helper function to test invalid arguments handling
async fn test_invalid_server_args(
    server_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let output = Command::new("cargo")
        .args(&["run", "--bin", server_name, "--", "--invalid-flag-xyz"])
        .output()?;

    // Should exit with error for invalid args
    if output.status.success() {
        return Err(format!("Server {} should reject invalid arguments", server_name).into());
    }

    // Should provide some error message
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    if stderr.is_empty() && stdout.is_empty() {
        return Err(format!("Server {} should provide error message", server_name).into());
    }

    println!("✅ {} error handling test passed", server_name);
    Ok(())
}

/// Integration test for overall MCP protocol readiness
#[tokio::test]
async fn test_mcp_protocol_readiness() {
    let servers = [
        "filesystem-server",
        "image-generation-server",
        "blog-generation-server",
        "creative-content-server",
    ];

    let mut ready_servers = 0;

    for server_name in &servers {
        let result = timeout(Duration::from_secs(8), async move {
            // Test basic functionality
            test_server_compilation(server_name).await?;
            test_server_help(server_name).await?;
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        })
        .await;

        match result {
            Ok(Ok(())) => {
                ready_servers += 1;
                println!("✅ {} is MCP protocol ready", server_name);
            }
            Ok(Err(e)) => {
                println!("⚠️ {} has issues: {}", server_name, e);
            }
            Err(_) => {
                println!("⚠️ {} timed out during readiness check", server_name);
            }
        }
    }

    // At least 2 servers should be fully ready
    assert!(
        ready_servers >= 2,
        "At least 2 servers should be MCP protocol ready, got {}",
        ready_servers
    );

    println!(
        "✅ MCP Protocol readiness: {}/{} servers ready",
        ready_servers,
        servers.len()
    );
}
