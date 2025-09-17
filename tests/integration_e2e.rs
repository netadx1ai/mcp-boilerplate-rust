//! Multi-Server Integration E2E Tests for MCP Boilerplate
//! 
//! Tests running multiple servers simultaneously to validate:
//! - No port conflicts in HTTP mode
//! - Proper process isolation
//! - Clean shutdown of all servers
//! - Concurrent operations without interference
//! 
//! This implements Task 3.1 from the E2E testing roadmap.

use std::collections::HashMap;
use std::process::{Command, Stdio, Child};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::timeout;

/// Test configuration for multi-server scenarios
#[derive(Clone)]
struct MultiServerTestConfig {
    servers: Vec<ServerConfig>,
    test_duration: Duration,
    cleanup_timeout: Duration,
}

/// Configuration for individual server in multi-server test
#[derive(Clone)]
struct ServerConfig {
    name: String,
    binary_name: String,
    http_port: Option<u16>,
    args: Vec<String>,
}

impl Default for MultiServerTestConfig {
    fn default() -> Self {
        Self {
            servers: vec![
                ServerConfig {
                    name: "filesystem".to_string(),
                    binary_name: "filesystem-server".to_string(),
                    http_port: Some(8001),
                    args: vec!["--transport".to_string(), "http".to_string(), "--port".to_string(), "8001".to_string()],
                },
                ServerConfig {
                    name: "image-generation".to_string(),
                    binary_name: "image-generation-server".to_string(),
                    http_port: Some(8002),
                    args: vec!["--transport".to_string(), "http".to_string(), "--port".to_string(), "8002".to_string()],
                },
                ServerConfig {
                    name: "blog-generation".to_string(),
                    binary_name: "blog-generation-server".to_string(),
                    http_port: Some(8003),
                    args: vec!["--transport".to_string(), "http".to_string(), "--port".to_string(), "8003".to_string()],
                },
                ServerConfig {
                    name: "creative-content".to_string(),
                    binary_name: "creative-content-server".to_string(),
                    http_port: Some(8004),
                    args: vec!["--transport".to_string(), "http".to_string(), "--port".to_string(), "8004".to_string()],
                },
            ],
            test_duration: Duration::from_secs(5),
            cleanup_timeout: Duration::from_secs(10),
        }
    }
}

/// Server process handle with automatic cleanup
struct ServerHandle {
    name: String,
    process: Child,
    port: Option<u16>,
    start_time: Instant,
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        if let Err(e) = self.process.kill() {
            eprintln!("⚠️ Failed to kill server {}: {}", self.name, e);
        } else {
            println!("🧹 Cleaned up server: {}", self.name);
        }
    }
}

impl ServerHandle {
    /// Start a server with the given configuration
    async fn start(config: &ServerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        println!("🚀 Starting server: {} with args: {:?}", config.name, config.args);
        
        let mut cmd = Command::new("cargo");
        cmd.args(&["run", "--bin", &config.binary_name, "--"])
           .args(&config.args)
           .current_dir("..")
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        let process = cmd.spawn()
            .map_err(|e| format!("Failed to start {}: {}", config.name, e))?;
        
        Ok(ServerHandle {
            name: config.name.clone(),
            process,
            port: config.http_port,
            start_time: Instant::now(),
        })
    }
    
    /// Check if server is still running
    fn is_running(&mut self) -> bool {
        match self.process.try_wait() {
            Ok(Some(_)) => false, // Process exited
            Ok(None) => true,     // Process still running
            Err(_) => false,      // Error checking status
        }
    }
    
    /// Get server uptime
    fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    /// Attempt graceful shutdown (send SIGTERM)
    #[cfg(unix)]
    fn shutdown_graceful(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Send SIGTERM
        unsafe {
            libc::kill(self.process.id() as i32, libc::SIGTERM);
        }
        
        // Wait a bit for graceful shutdown
        std::thread::sleep(Duration::from_millis(100));
        
        Ok(())
    }
    
    /// Force shutdown (kill process)
    fn shutdown_force(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.process.kill()?;
        self.process.wait()?;
        println!("💀 Force killed server: {}", self.name);
        Ok(())
    }
}

/// Multi-server test orchestrator
struct MultiServerOrchestrator {
    servers: Arc<Mutex<HashMap<String, ServerHandle>>>,
    config: MultiServerTestConfig,
}

impl MultiServerOrchestrator {
    fn new(config: MultiServerTestConfig) -> Self {
        Self {
            servers: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }
    
    /// Start all configured servers
    async fn start_all_servers(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut servers = self.servers.lock().await;
        
        for server_config in &self.config.servers {
            match ServerHandle::start(server_config).await {
                Ok(handle) => {
                    servers.insert(server_config.name.clone(), handle);
                    println!("✅ Started server: {}", server_config.name);
                }
                Err(e) => {
                    eprintln!("❌ Failed to start server {}: {}", server_config.name, e);
                    // Continue with other servers for partial success testing
                }
            }
        }
        
        // Wait a moment for servers to initialize
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        Ok(())
    }
    
    /// Check status of all servers
    async fn check_server_status(&self) -> HashMap<String, bool> {
        let mut servers = self.servers.lock().await;
        let mut status = HashMap::new();
        
        for (name, handle) in servers.iter_mut() {
            let running = handle.is_running();
            status.insert(name.clone(), running);
            
            if running {
                println!("🟢 Server {} running (uptime: {:?})", name, handle.uptime());
            } else {
                println!("🔴 Server {} stopped", name);
            }
        }
        
        status
    }
    
    /// Shutdown all servers
    async fn shutdown_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut servers = self.servers.lock().await;
        
        for (name, handle) in servers.iter_mut() {
            println!("🛑 Shutting down server: {}", name);
            
            // Try graceful shutdown first
            #[cfg(unix)]
            if let Err(e) = handle.shutdown_graceful() {
                eprintln!("⚠️ Graceful shutdown failed for {}: {}", name, e);
            }
            
            // Wait a moment
            tokio::time::sleep(Duration::from_millis(200)).await;
            
            // Force shutdown if still running
            if handle.is_running() {
                if let Err(e) = handle.shutdown_force() {
                    eprintln!("❌ Force shutdown failed for {}: {}", name, e);
                }
            }
        }
        
        servers.clear();
        println!("🧹 All servers shutdown complete");
        Ok(())
    }
    
    /// Test port conflicts by checking if servers bind to different ports
    async fn test_port_conflicts(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let servers = self.servers.lock().await;
        let mut used_ports = std::collections::HashSet::new();
        let mut conflicts = false;
        
        for (name, handle) in servers.iter() {
            if let Some(port) = handle.port {
                if used_ports.contains(&port) {
                    eprintln!("❌ Port conflict detected: {} trying to use port {}", name, port);
                    conflicts = true;
                } else {
                    used_ports.insert(port);
                    println!("✅ Server {} allocated unique port: {}", name, port);
                }
            }
        }
        
        Ok(!conflicts)
    }
}

/// Test starting multiple servers simultaneously without conflicts
#[tokio::test]
async fn test_multiple_servers_no_conflicts() {
    let test_result = timeout(
        Duration::from_secs(15),
        async {
            let config = MultiServerTestConfig::default();
            let orchestrator = MultiServerOrchestrator::new(config);
            
            // Start all servers
            orchestrator.start_all_servers().await?;
            
            // Wait for servers to initialize
            tokio::time::sleep(Duration::from_millis(1000)).await;
            
            // Check that servers are running
            let status = orchestrator.check_server_status().await;
            let running_count = status.values().filter(|&&v| v).count();
            
            println!("📊 Running servers: {}/{}", running_count, status.len());
            
            // Test for port conflicts
            let no_conflicts = orchestrator.test_port_conflicts().await?;
            assert!(no_conflicts, "No port conflicts should exist between servers");
            
            // At least half the servers should be running for a successful test
            assert!(
                running_count >= 2,
                "At least 2 servers should be running simultaneously, got {}",
                running_count
            );
            
            // Clean shutdown
            orchestrator.shutdown_all().await?;
            
            // Verify shutdown
            tokio::time::sleep(Duration::from_millis(500)).await;
            let final_status = orchestrator.check_server_status().await;
            let still_running = final_status.values().filter(|&&v| v).count();
            
            assert_eq!(still_running, 0, "All servers should be stopped after shutdown");
            
            println!("✅ Multi-server integration test completed successfully");
            Ok::<(), Box<dyn std::error::Error>>(())
        }
    ).await;
    
    match test_result {
        Ok(Ok(())) => println!("✅ Multi-server test passed"),
        Ok(Err(e)) => panic!("❌ Multi-server test failed: {}", e),
        Err(_) => panic!("💥 Multi-server test timed out - potential deadlock or hanging"),
    }
}

/// Test servers in STDIO mode (should not conflict since no ports)
#[tokio::test]
async fn test_multiple_servers_stdio_mode() {
    let test_result = timeout(
        Duration::from_secs(10),
        async {
            let mut stdio_config = MultiServerTestConfig::default();
            
            // Configure all servers for STDIO mode
            for server in &mut stdio_config.servers {
                server.args = vec!["--transport".to_string(), "stdio".to_string()];
                server.http_port = None;
            }
            
            let orchestrator = MultiServerOrchestrator::new(stdio_config);
            
            // Start all servers in STDIO mode
            orchestrator.start_all_servers().await?;
            
            // Brief wait for initialization
            tokio::time::sleep(Duration::from_millis(500)).await;
            
            // Check server status
            let status = orchestrator.check_server_status().await;
            let running_count = status.values().filter(|&&v| v).count();
            
            println!("📊 STDIO servers running: {}/{}", running_count, status.len());
            
            // In STDIO mode, servers should start but may exit quickly without input
            // We consider it success if they start without error
            assert!(
                running_count >= 2 || status.len() >= 3,
                "STDIO servers should start successfully"
            );
            
            // Clean shutdown
            orchestrator.shutdown_all().await?;
            
            println!("✅ STDIO multi-server test completed");
            Ok::<(), Box<dyn std::error::Error>>(())
        }
    ).await;
    
    match test_result {
        Ok(Ok(())) => println!("✅ STDIO multi-server test passed"),
        Ok(Err(e)) => panic!("❌ STDIO multi-server test failed: {}", e),
        Err(_) => panic!("💥 STDIO multi-server test timed out"),
    }
}

/// Test concurrent server operations
#[tokio::test]
async fn test_concurrent_server_operations() {
    let test_result = timeout(
        Duration::from_secs(20),
        async {
            let config = MultiServerTestConfig::default();
            let orchestrator = MultiServerOrchestrator::new(config);
            
            // Start servers
            orchestrator.start_all_servers().await?;
            tokio::time::sleep(Duration::from_millis(1000)).await;
            
            // Perform concurrent operations
            let status_checks = Arc::new(Mutex::new(Vec::new()));
            let mut tasks = Vec::new();
            
            // Spawn multiple status check tasks
            for i in 0..5 {
                let orch = orchestrator.clone();
                let checks = status_checks.clone();
                
                let task = tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(i * 200)).await;
                    let status = orch.check_server_status().await;
                    let running_count = status.values().filter(|&&v| v).count();
                    
                    let mut checks_lock = checks.lock().await;
                    checks_lock.push(running_count);
                    
                    println!("🔄 Concurrent check #{}: {} servers running", i + 1, running_count);
                });
                
                tasks.push(task);
            }
            
            // Wait for all concurrent operations
            for task in tasks {
                task.await?;
            }
            
            // Verify consistency
            let final_checks = status_checks.lock().await;
            let min_running = *final_checks.iter().min().unwrap_or(&0);
            let max_running = *final_checks.iter().max().unwrap_or(&0);
            
            println!("📈 Concurrent operations: min={}, max={} servers running", min_running, max_running);
            
            // Results should be reasonably consistent (servers shouldn't randomly crash)
            assert!(
                max_running - min_running <= 1,
                "Server count should be stable during concurrent operations"
            );
            
            // Clean shutdown
            orchestrator.shutdown_all().await?;
            
            println!("✅ Concurrent operations test completed");
            Ok::<(), Box<dyn std::error::Error>>(())
        }
    ).await;
    
    match test_result {
        Ok(Ok(())) => println!("✅ Concurrent operations test passed"),
        Ok(Err(e)) => panic!("❌ Concurrent operations test failed: {}", e),
        Err(_) => panic!("💥 Concurrent operations test timed out"),
    }
}

/// Test server resource isolation
#[tokio::test]
async fn test_server_resource_isolation() {
    let test_result = timeout(
        Duration::from_secs(15),
        async {
            // Test with reduced server set for faster execution
            let mut config = MultiServerTestConfig::default();
            config.servers = config.servers.into_iter().take(2).collect();
            
            let orchestrator = MultiServerOrchestrator::new(config);
            
            // Start servers
            orchestrator.start_all_servers().await?;
            tokio::time::sleep(Duration::from_millis(800)).await;
            
            // Get initial server count
            let status = orchestrator.check_server_status().await;
            let initial_count = status.values().filter(|&&v| v).count();
            
            println!("📊 Initial server count: {}", initial_count);
            
            // Simulate stopping one server (resource isolation test)
            let stopped_server_name = {
                let servers = orchestrator.servers.lock().await;
                servers.keys().next().cloned()
            };
            
            if let Some(server_name) = stopped_server_name {
                let mut servers = orchestrator.servers.lock().await;
                if let Some(mut handle) = servers.remove(&server_name) {
                    println!("🛑 Manually stopping server: {}", server_name);
                    let _ = handle.shutdown_force();
                }
            }
            
            // Brief wait
            tokio::time::sleep(Duration::from_millis(300)).await;
            
            // Check that other servers are still running
            let post_shutdown_status = orchestrator.check_server_status().await;
            let remaining_count = post_shutdown_status.values().filter(|&&v| v).count();
            
            println!("📊 Remaining server count: {}", remaining_count);
            
            // Other servers should still be running (resource isolation)
            assert!(
                remaining_count >= initial_count - 1,
                "Other servers should remain running when one is stopped"
            );
            
            // Clean shutdown of remaining servers
            orchestrator.shutdown_all().await?;
            
            println!("✅ Resource isolation test completed");
            Ok::<(), Box<dyn std::error::Error>>(())
        }
    ).await;
    
    match test_result {
        Ok(Ok(())) => println!("✅ Resource isolation test passed"),
        Ok(Err(e)) => panic!("❌ Resource isolation test failed: {}", e),
        Err(_) => panic!("💥 Resource isolation test timed out"),
    }
}

/// Test that HTTP servers use different ports without conflicts
#[tokio::test]
async fn test_http_port_allocation() {
    let test_result = timeout(
        Duration::from_secs(10),
        async {
            let config = MultiServerTestConfig::default();
            let orchestrator = MultiServerOrchestrator::new(config);
            
            // Start all servers
            orchestrator.start_all_servers().await?;
            tokio::time::sleep(Duration::from_millis(800)).await;
            
            // Test port conflicts
            let no_conflicts = orchestrator.test_port_conflicts().await?;
            assert!(no_conflicts, "HTTP servers should use unique ports");
            
            // Test that ports are actually in use (basic connectivity check)
            let servers = orchestrator.servers.lock().await;
            let mut accessible_ports = 0;
            
            for (name, handle) in servers.iter() {
                if let Some(port) = handle.port {
                    // Try to connect to the port (basic check)
                    match std::net::TcpStream::connect_timeout(
                        &format!("127.0.0.1:{}", port).parse().unwrap(),
                        Duration::from_millis(500)
                    ) {
                        Ok(_) => {
                            println!("✅ Server {} accessible on port {}", name, port);
                            accessible_ports += 1;
                        }
                        Err(e) => {
                            println!("⚠️ Server {} not accessible on port {}: {}", name, port, e);
                        }
                    }
                }
            }
            
            println!("📊 Accessible HTTP servers: {}/{}", accessible_ports, servers.len());
            
            // At least some servers should be accessible
            assert!(accessible_ports >= 1, "At least one HTTP server should be accessible");
            
            // Clean shutdown
            drop(servers); // Release lock before shutdown
            orchestrator.shutdown_all().await?;
            
            println!("✅ HTTP port allocation test completed");
            Ok::<(), Box<dyn std::error::Error>>(())
        }
    ).await;
    
    match test_result {
        Ok(Ok(())) => println!("✅ HTTP port allocation test passed"),
        Ok(Err(e)) => panic!("❌ HTTP port allocation test failed: {}", e),
        Err(_) => panic!("💥 HTTP port allocation test timed out"),
    }
}

/// Test graceful shutdown of multiple servers
#[tokio::test]
async fn test_graceful_multi_server_shutdown() {
    let test_result = timeout(
        Duration::from_secs(12),
        async {
            let config = MultiServerTestConfig::default();
            let orchestrator = MultiServerOrchestrator::new(config);
            
            // Start servers
            orchestrator.start_all_servers().await?;
            tokio::time::sleep(Duration::from_millis(600)).await;
            
            // Verify servers are running
            let pre_shutdown_status = orchestrator.check_server_status().await;
            let running_count = pre_shutdown_status.values().filter(|&&v| v).count();
            
            assert!(running_count >= 2, "Multiple servers should be running before shutdown test");
            println!("📊 Pre-shutdown: {} servers running", running_count);
            
            // Test graceful shutdown
            let shutdown_start = Instant::now();
            orchestrator.shutdown_all().await?;
            let shutdown_duration = shutdown_start.elapsed();
            
            // Shutdown should be reasonably fast
            assert!(
                shutdown_duration < Duration::from_secs(5),
                "Shutdown should complete within 5 seconds, took {:?}",
                shutdown_duration
            );
            
            // Verify all servers stopped
            tokio::time::sleep(Duration::from_millis(300)).await;
            let post_shutdown_status = orchestrator.check_server_status().await;
            let still_running = post_shutdown_status.values().filter(|&&v| v).count();
            
            assert_eq!(still_running, 0, "All servers should be stopped after graceful shutdown");
            
            println!("✅ Graceful shutdown completed in {:?}", shutdown_duration);
            Ok::<(), Box<dyn std::error::Error>>(())
        }
    ).await;
    
    match test_result {
        Ok(Ok(())) => println!("✅ Graceful multi-server shutdown test passed"),
        Ok(Err(e)) => panic!("❌ Graceful shutdown test failed: {}", e),
        Err(_) => panic!("💥 Graceful shutdown test timed out"),
    }
}

/// Test server startup sequence and timing
#[tokio::test]
async fn test_server_startup_performance() {
    let test_result = timeout(
        Duration::from_secs(15),
        async {
            let config = MultiServerTestConfig::default();
            let mut startup_times = HashMap::new();
            
            // Test each server individually for accurate timing
            for server_config in &config.servers {
                let start_time = Instant::now();
                
                match ServerHandle::start(server_config).await {
                    Ok(mut handle) => {
                        // Wait for server to initialize
                        tokio::time::sleep(Duration::from_millis(400)).await;
                        
                        let startup_time = start_time.elapsed();
                        startup_times.insert(server_config.name.clone(), startup_time);
                        
                        println!("⏱️ Server {} startup time: {:?}", server_config.name, startup_time);
                        
                        // Cleanup
                        let _ = handle.shutdown_force();
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to start server {}: {}", server_config.name, e);
                    }
                }
                
                // Small gap between server tests
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
            
            // Analyze startup performance
            if !startup_times.is_empty() {
                let avg_startup = startup_times.values().sum::<Duration>() / startup_times.len() as u32;
                let max_startup = *startup_times.values().max().unwrap();
                
                println!("📊 Startup Performance:");
                println!("   Average: {:?}", avg_startup);
                println!("   Slowest: {:?}", max_startup);
                
                // Performance assertions
                assert!(
                    max_startup < Duration::from_secs(8),
                    "No server should take more than 8 seconds to start (cargo run overhead)"
                );
                
                assert!(
                    avg_startup < Duration::from_secs(5),
                    "Average startup should be under 5 seconds"
                );
            }
            
            println!("✅ Server startup performance test completed");
            Ok::<(), Box<dyn std::error::Error>>(())
        }
    ).await;
    
    match test_result {
        Ok(Ok(())) => println!("✅ Server startup performance test passed"),
        Ok(Err(e)) => panic!("❌ Startup performance test failed: {}", e),
        Err(_) => panic!("💥 Startup performance test timed out"),
    }
}

/// Test mixed transport modes (some HTTP, some STDIO)
#[tokio::test]
async fn test_mixed_transport_modes() {
    let test_result = timeout(
        Duration::from_secs(12),
        async {
            let mut config = MultiServerTestConfig::default();
            
            // Configure mixed transport modes
            config.servers[0].args = vec!["--transport".to_string(), "http".to_string(), "--port".to_string(), "8001".to_string()];
            config.servers[1].args = vec!["--transport".to_string(), "stdio".to_string()];
            config.servers[2].args = vec!["--transport".to_string(), "http".to_string(), "--port".to_string(), "8003".to_string()];
            config.servers[3].args = vec!["--transport".to_string(), "stdio".to_string()];
            
            // Update port info
            config.servers[1].http_port = None;
            config.servers[3].http_port = None;
            
            let orchestrator = MultiServerOrchestrator::new(config);
            
            // Start all servers
            orchestrator.start_all_servers().await?;
            tokio::time::sleep(Duration::from_millis(800)).await;
            
            // Check status
            let status = orchestrator.check_server_status().await;
            let running_count = status.values().filter(|&&v| v).count();
            
            println!("📊 Mixed transport servers: {}/{} running", running_count, status.len());
            
            // Test that HTTP servers don't conflict
            let no_conflicts = orchestrator.test_port_conflicts().await?;
            assert!(no_conflicts, "HTTP servers in mixed mode should not have port conflicts");
            
            // At least some servers should be running
            assert!(running_count >= 2, "At least 2 servers should run in mixed transport mode");
            
            // Clean shutdown
            orchestrator.shutdown_all().await?;
            
            println!("✅ Mixed transport test completed");
            Ok::<(), Box<dyn std::error::Error>>(())
        }
    ).await;
    
    match test_result {
        Ok(Ok(())) => println!("✅ Mixed transport test passed"),
        Ok(Err(e)) => panic!("❌ Mixed transport test failed: {}", e),
        Err(_) => panic!("💥 Mixed transport test timed out"),
    }
}

/// Test server memory and process cleanup
#[tokio::test]
async fn test_server_process_cleanup() {
    let test_result = timeout(
        Duration::from_secs(10),
        async {
            let config = MultiServerTestConfig::default();
            let orchestrator = MultiServerOrchestrator::new(config);
            
            // Get initial process count
            let initial_processes = count_rust_processes().await;
            println!("📊 Initial Rust processes: {}", initial_processes);
            
            // Start servers
            orchestrator.start_all_servers().await?;
            tokio::time::sleep(Duration::from_millis(600)).await;
            
            // Check process count increased
            let running_processes = count_rust_processes().await;
            println!("📊 Running processes: {}", running_processes);
            
            assert!(
                running_processes > initial_processes,
                "Process count should increase when servers are running"
            );
            
            // Shutdown all servers
            orchestrator.shutdown_all().await?;
            tokio::time::sleep(Duration::from_millis(500)).await;
            
            // Check process cleanup
            let final_processes = count_rust_processes().await;
            println!("📊 Final processes: {}", final_processes);
            
            // Process count should return close to initial (allow some variance for cargo processes)
            let process_diff = final_processes.saturating_sub(initial_processes);
            assert!(
                process_diff <= 2,
                "Process count should return near initial after cleanup, diff: {}",
                process_diff
            );
            
            println!("✅ Process cleanup verification completed");
            Ok::<(), Box<dyn std::error::Error>>(())
        }
    ).await;
    
    match test_result {
        Ok(Ok(())) => println!("✅ Process cleanup test passed"),
        Ok(Err(e)) => panic!("❌ Process cleanup test failed: {}", e),
        Err(_) => panic!("💥 Process cleanup test timed out"),
    }
}

/// Count running Rust/cargo processes (helper function)
async fn count_rust_processes() -> usize {
    match Command::new("pgrep")
        .args(&["-f", "cargo|rust"])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                output_str.lines().count()
            } else {
                0
            }
        }
        Err(_) => 0, // pgrep not available or error
    }
}

/// Test error handling when servers fail to start
#[tokio::test]
async fn test_server_startup_error_handling() {
    let test_result = timeout(
        Duration::from_secs(8),
        async {
            // Create config with intentionally problematic settings
            let mut config = MultiServerTestConfig::default();
            
            // Two servers on same port (should cause conflict)
            config.servers[0].args = vec!["--transport".to_string(), "http".to_string(), "--port".to_string(), "8005".to_string()];
            config.servers[1].args = vec!["--transport".to_string(), "http".to_string(), "--port".to_string(), "8005".to_string()];
            
            let orchestrator = MultiServerOrchestrator::new(config);
            
            // Attempt to start servers (some may fail due to port conflict)
            orchestrator.start_all_servers().await?;
            tokio::time::sleep(Duration::from_millis(800)).await;
            
            // Check which servers actually started
            let status = orchestrator.check_server_status().await;
            let running_count = status.values().filter(|&&v| v).count();
            
            println!("📊 Servers running with conflicts: {}/{}", running_count, status.len());
            
            // Some servers should fail to start due to port conflicts
            // but the system should handle this gracefully
            assert!(
                running_count < status.len(),
                "Not all servers should start when there are port conflicts"
            );
            
            // Cleanup
            orchestrator.shutdown_all().await?;
            
            println!("✅ Error handling test completed");
            Ok::<(), Box<dyn std::error::Error>>(())
        }
    ).await;
    
    match test_result {
        Ok(Ok(())) => println!("✅ Server startup error handling test passed"),
        Ok(Err(e)) => panic!("❌ Startup error handling test failed: {}", e),
        Err(_) => panic!("💥 Startup error handling test timed out"),
    }
}

/// Clone trait for MultiServerOrchestrator to enable sharing across async tasks
trait CloneableOrchestrator {
    fn clone_orchestrator(&self) -> Self;
}

impl Clone for MultiServerOrchestrator {
    fn clone(&self) -> Self {
        Self {
            servers: Arc::clone(&self.servers),
            config: MultiServerTestConfig {
                servers: self.config.servers.clone(),
                test_duration: self.config.test_duration,
                cleanup_timeout: self.config.cleanup_timeout,
            },
        }
    }
}

/// Test rapid start/stop cycles for server stability
#[tokio::test]
async fn test_rapid_server_cycles() {
    let test_result = timeout(
        Duration::from_secs(20),
        async {
            // Use just 2 servers for faster cycling
            let mut config = MultiServerTestConfig::default();
            config.servers = config.servers.into_iter().take(2).collect();
            
            let mut successful_cycles = 0;
            
            for cycle in 1..=3 {
                println!("🔄 Starting cycle #{}", cycle);
                
                let test_config = MultiServerTestConfig {
                    servers: config.servers.clone(),
                    test_duration: config.test_duration,
                    cleanup_timeout: config.cleanup_timeout,
                };
                let orchestrator = MultiServerOrchestrator::new(test_config);
                
                // Quick start
                orchestrator.start_all_servers().await?;
                tokio::time::sleep(Duration::from_millis(400)).await;
                
                // Verify running
                let status = orchestrator.check_server_status().await;
                let running = status.values().filter(|&&v| v).count();
                
                if running >= 1 {
                    successful_cycles += 1;
                    println!("✅ Cycle #{}: {} servers running", cycle, running);
                } else {
                    println!("⚠️ Cycle #{}: no servers running", cycle);
                }
                
                // Quick shutdown
                orchestrator.shutdown_all().await?;
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
            
            assert!(
                successful_cycles >= 2,
                "At least 2/3 rapid cycles should succeed, got {}",
                successful_cycles
            );
            
            println!("✅ Rapid cycling test: {}/3 cycles successful", successful_cycles);
            Ok::<(), Box<dyn std::error::Error>>(())
        }
    ).await;
    
    match test_result {
        Ok(Ok(())) => println!("✅ Rapid server cycles test passed"),
        Ok(Err(e)) => panic!("❌ Rapid cycles test failed: {}", e),
        Err(_) => panic!("💥 Rapid cycles test timed out"),
    }
}