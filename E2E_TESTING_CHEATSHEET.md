# MCP Boilerplate Rust - E2E Testing Cheat Sheet

## Quick Reference for End-to-End Testing Patterns

**Version**: 1.0  
**Created**: 2025-09-17T08:38:14+07:00  
**Purpose**: Quick reference for implementing and maintaining E2E tests

---

## 🚀 Essential Test Patterns

### Timeout Pattern (MANDATORY)
```rust
#[tokio::test]
async fn test_server_operation() {
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        actual_test_logic()
    ).await.expect("Test should not hang - investigate if this fails");
    
    assert!(result.is_ok());
}
```

### Resource Cleanup Pattern
```rust
struct TestContext {
    server_handle: Option<JoinHandle<()>>,
    temp_dir: TempDir,
    http_client: reqwest::Client,
}

impl Drop for TestContext {
    fn drop(&mut self) {
        if let Some(handle) = self.server_handle.take() {
            handle.abort(); // Force cleanup
        }
    }
}

#[tokio::test]
async fn test_with_cleanup() {
    let _ctx = TestContext::new().await;
    // Test logic here - cleanup happens automatically
}
```

### Process Spawning for STDIO Tests
```rust
async fn spawn_stdio_server(server_name: &str) -> Result<Child, Error> {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--bin", server_name])
       .stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped())
       .kill_on_drop(true);
    
    let child = cmd.spawn()?;
    
    // Wait for server ready signal
    tokio::time::sleep(Duration::from_millis(100)).await;
    Ok(child)
}
```

### HTTP Server Test Pattern
```rust
async fn test_http_server(port: u16) -> Result<(), Error> {
    // Start server on specific port
    let server_handle = tokio::spawn(async move {
        start_server_on_port(port).await
    });
    
    // Wait for startup
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Test HTTP endpoint
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("http://localhost:{}/mcp", port))
        .json(&test_request)
        .send()
        .await?;
        
    assert_eq!(response.status(), 200);
    
    // Cleanup
    server_handle.abort();
    Ok(())
}
```

---

## 🔧 MCP Protocol Testing Patterns

### MCP Handshake Test
```rust
async fn test_mcp_handshake(transport: &mut dyn Transport) -> Result<(), Error> {
    // Send initialize request
    let init_request = McpRequest::Initialize {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ClientCapabilities::default(),
        client_info: ClientInfo {
            name: "test-client".to_string(),
            version: "1.0.0".to_string(),
        },
    };
    
    transport.send_message(&init_request).await?;
    
    // Expect initialize response
    let response = timeout(
        Duration::from_secs(2),
        transport.receive_message()
    ).await??;
    
    match response {
        McpResponse::Initialize { capabilities, .. } => {
            assert!(!capabilities.tools.is_empty());
        }
        _ => panic!("Expected Initialize response"),
    }
    
    // Send initialized notification
    transport.send_message(&McpRequest::Initialized).await?;
    Ok(())
}
```

### Tool List Validation
```rust
async fn test_tool_list(transport: &mut dyn Transport) -> Result<(), Error> {
    let request = McpRequest::ListTools;
    transport.send_message(&request).await?;
    
    let response = timeout(
        Duration::from_secs(1),
        transport.receive_message()
    ).await??;
    
    match response {
        McpResponse::ListTools { tools } => {
            assert!(!tools.is_empty(), "Server should expose at least one tool");
            
            // Validate each tool has required fields
            for tool in tools {
                assert!(!tool.name.is_empty(), "Tool must have name");
                assert!(!tool.description.is_empty(), "Tool must have description");
            }
        }
        _ => panic!("Expected ListTools response"),
    }
    Ok(())
}
```

### Tool Call Test Pattern
```rust
async fn test_tool_call(
    transport: &mut dyn Transport,
    tool_name: &str,
    arguments: Value
) -> Result<ToolResult, Error> {
    let call_id = Uuid::new_v4().to_string();
    let request = McpRequest::CallTool {
        name: tool_name.to_string(),
        arguments,
        call_id: call_id.clone(),
    };
    
    transport.send_message(&request).await?;
    
    let response = timeout(
        Duration::from_secs(3),
        transport.receive_message()
    ).await??;
    
    match response {
        McpResponse::CallTool { result, call_id: resp_id } => {
            assert_eq!(call_id, resp_id, "Call ID mismatch");
            Ok(result)
        }
        McpResponse::Error { error } => {
            Err(Error::ToolError(error.message))
        }
        _ => panic!("Expected CallTool response"),
    }
}
```

---

## 🏗️ Test Infrastructure Patterns

### Random Port Allocation
```rust
fn get_random_port() -> u16 {
    use std::net::{TcpListener, SocketAddr};
    
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to random port");
    let addr = listener.local_addr()
        .expect("Failed to get local address");
    addr.port()
}

#[tokio::test]
async fn test_http_transport() {
    let port = get_random_port();
    // Use port for test...
}
```

### Temporary Directory Setup
```rust
use tempfile::TempDir;

#[tokio::test]
async fn test_filesystem_operations() {
    let temp_dir = TempDir::new()
        .expect("Failed to create temp directory");
    let temp_path = temp_dir.path();
    
    // Create test files in temp_path
    let test_file = temp_path.join("test.txt");
    tokio::fs::write(&test_file, "test content").await?;
    
    // Run filesystem tests
    // temp_dir automatically cleaned up on drop
}
```

### Test Data Fixtures
```rust
fn create_test_fixtures() -> TestData {
    TestData {
        valid_file_content: "Hello, MCP World!",
        invalid_path: "/invalid/nonexistent/path",
        test_image_prompt: "A beautiful sunset over mountains",
        test_blog_topic: "The Future of AI Development",
        expected_tool_count: 4,
    }
}
```

---

## 📊 Server-Specific Test Patterns

### Filesystem Server Tests
```rust
#[tokio::test]
async fn test_filesystem_server_full_workflow() {
    let temp_dir = TempDir::new()?;
    let mut server = start_filesystem_server(temp_dir.path()).await?;
    
    // Test file operations
    let file_path = temp_dir.path().join("test.txt");
    
    // Write file
    let result = call_tool(&mut server, "write_file", json!({
        "path": file_path.to_string_lossy(),
        "content": "test content"
    })).await?;
    
    assert!(result.is_success());
    
    // Read file back
    let result = call_tool(&mut server, "read_file", json!({
        "path": file_path.to_string_lossy()
    })).await?;
    
    assert_eq!(result.content, "test content");
    
    // Verify file exists on disk
    assert!(file_path.exists());
}
```

### AI Server Mock Response Tests
```rust
#[tokio::test]
async fn test_image_generation_server() {
    let mut server = start_image_server().await?;
    
    let result = call_tool(&mut server, "generate_image", json!({
        "prompt": "A test image",
        "size": "512x512"
    })).await?;
    
    // Validate mock response structure
    assert!(result.contains_key("image_data"));
    assert!(result.contains_key("metadata"));
    assert_eq!(result["metadata"]["size"], "512x512");
    
    // Verify consistent mock responses
    let result2 = call_tool(&mut server, "generate_image", json!({
        "prompt": "A test image",
        "size": "512x512"
    })).await?;
    
    // Should be deterministic for same inputs
    assert_eq!(result, result2);
}
```

---

## ⚡ Performance Testing Patterns

### Startup Time Testing
```rust
#[tokio::test]
async fn test_server_startup_performance() {
    let start_time = Instant::now();
    
    let _server = timeout(
        Duration::from_secs(3), // Max acceptable startup time
        start_server()
    ).await.expect("Server startup too slow")?;
    
    let startup_duration = start_time.elapsed();
    assert!(startup_duration < Duration::from_secs(2), 
           "Server startup took {:?}, should be < 2s", startup_duration);
}
```

### Concurrent Request Testing
```rust
#[tokio::test]
async fn test_concurrent_requests() {
    let server = start_server().await?;
    let client = create_client().await?;
    
    // Spawn multiple concurrent requests
    let mut handles = Vec::new();
    for i in 0..10 {
        let client = client.clone();
        let handle = tokio::spawn(async move {
            call_tool(&client, "test_tool", json!({"id": i})).await
        });
        handles.push(handle);
    }
    
    // Wait for all to complete
    let results = timeout(
        Duration::from_secs(5),
        futures::future::join_all(handles)
    ).await.expect("Concurrent requests took too long");
    
    // Verify all succeeded
    for result in results {
        assert!(result?.is_ok());
    }
}
```

---

## 🚨 Error Testing Patterns

### Network Error Simulation
```rust
#[tokio::test]
async fn test_network_error_handling() {
    let mut server = start_server().await?;
    
    // Simulate connection drop
    server.disconnect().await?;
    
    // Attempt operation - should handle gracefully
    let result = call_tool(&mut server, "test_tool", json!({})).await;
    
    match result {
        Err(Error::Transport(_)) => {
            // Expected - server should report transport error
        }
        _ => panic!("Expected transport error"),
    }
}
```

### Invalid Parameter Testing
```rust
#[tokio::test]
async fn test_invalid_parameters() {
    let mut server = start_server().await?;
    
    // Test missing required parameter
    let result = call_tool(&mut server, "write_file", json!({})).await;
    assert!(result.is_err());
    
    // Test invalid parameter type
    let result = call_tool(&mut server, "write_file", json!({
        "path": 12345, // Should be string
        "content": "test"
    })).await;
    assert!(result.is_err());
}
```

---

## 🔍 Debugging & Troubleshooting

### Debug Logging in Tests
```rust
#[tokio::test]
async fn test_with_debug_logging() {
    tracing_subscriber::fmt()
        .with_test_writer()
        .with_max_level(tracing::Level::DEBUG)
        .try_init()
        .ok(); // Ignore if already initialized
    
    // Test code with debug logging enabled
    tracing::debug!("Starting test with debug logging");
    
    // Your test logic here
}
```

### Test Output Capture
```rust
#[tokio::test]
async fn test_with_output_capture() {
    let mut cmd = assert_cmd::Command::cargo_bin("filesystem-server")?;
    
    cmd.arg("--help")
       .assert()
       .success()
       .stdout(predicates::str::contains("filesystem operations"));
}
```

### Hanging Test Investigation
```rust
// If a test hangs, add this pattern to investigate:
#[tokio::test]
async fn investigate_hanging_test() {
    println!("Test starting...");
    
    let result = tokio::time::timeout(
        Duration::from_secs(1),
        potentially_hanging_operation()
    ).await;
    
    match result {
        Ok(val) => println!("Operation completed: {:?}", val),
        Err(_) => {
            println!("Operation timed out - investigating...");
            // Add more specific debugging here
        }
    }
}
```

---

## 📋 Quality Checklist

### Before Committing E2E Tests
- [ ] All tests have timeouts (< 5s per test)
- [ ] No hardcoded ports or file paths
- [ ] Proper resource cleanup (no processes left running)
- [ ] Tests pass consistently (run 3 times)
- [ ] Both STDIO and HTTP transports tested
- [ ] Error scenarios covered
- [ ] Performance requirements met
- [ ] No external dependencies

### Performance Targets
- Individual test: < 5 seconds
- Full E2E suite: < 30 seconds
- Server startup: < 2 seconds
- Tool response: < 1 second

### Common Anti-Patterns to Avoid
- ❌ `tokio::time::sleep()` without timeout wrapper
- ❌ Hardcoded ports (use `get_random_port()`)
- ❌ Global state between tests
- ❌ External API calls
- ❌ Tests that sometimes pass/fail

### Debugging Commands
```bash
# Run specific test with output
cargo test test_name -- --nocapture

# Run tests serially (if parallel issues)
cargo test -- --test-threads=1

# Run with timing info
cargo test -- --report-time

# Run only E2E tests
cargo test --tests

# Run with debug logging
RUST_LOG=debug cargo test
```

---

**Remember**: The goal is reliable, fast, comprehensive testing that gives confidence in production deployment. When in doubt, prefer explicit cleanup and shorter timeouts over convenience.

**Next Steps**: Use this cheat sheet as reference while implementing the tasks in `tasks_mcp-boilerplate-rust-e2e-tests_2025-09-17-083814.md`.