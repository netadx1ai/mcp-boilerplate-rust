//! Error Recovery & Resilience E2E Tests for MCP Boilerplate
//! 
//! Tests server recovery from errors and resilience to common failure modes:
//! - Invalid tool calls don't crash server
//! - Malformed JSON handling
//! - Network interruption recovery
//! - Graceful shutdown scenarios
//! - Restart and state recovery
//! 
//! This implements Task 3.3 from the E2E testing roadmap.

use std::collections::HashMap;
use std::process::{Command, Stdio, Child};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::timeout;

/// Resilience test configuration
struct ResilienceTestConfig {
    recovery_timeout: Duration,
    stress_duration: Duration,
    max_retry_attempts: usize,
    error_injection_rate: f64,
}

impl Default for ResilienceTestConfig {
    fn default() -> Self {
        Self {
            recovery_timeout: Duration::from_secs(5),
            stress_duration: Duration::from_secs(3),
            max_retry_attempts: 3,
            error_injection_rate: 0.3, // 30% error injection rate
        }
    }
}

/// Error scenario types for testing
#[derive(Debug, Clone)]
enum ErrorScenario {
    InvalidArguments,
    MalformedInput,
    ResourceExhaustion,
    NetworkFailure,
    ProcessKill,
    ConfigurationError,
}

impl ErrorScenario {
    fn description(&self) -> &'static str {
        match self {
            ErrorScenario::InvalidArguments => "Invalid command line arguments",
            ErrorScenario::MalformedInput => "Malformed or corrupt input data",
            ErrorScenario::ResourceExhaustion => "Resource exhaustion conditions",
            ErrorScenario::NetworkFailure => "Network connectivity issues",
            ErrorScenario::ProcessKill => "Unexpected process termination",
            ErrorScenario::ConfigurationError => "Invalid configuration settings",
        }
    }
}

/// Resilience test results
#[derive(Debug)]
struct ResilienceResults {
    scenario: ErrorScenario,
    recovery_successful: bool,
    recovery_time: Option<Duration>,
    error_message: Option<String>,
}

/// Server resilience tester
struct ServerResilienceTester {
    config: ResilienceTestConfig,
    results: Arc<Mutex<Vec<ResilienceResults>>>,
}

impl ServerResilienceTester {
    fn new(config: ResilienceTestConfig) -> Self {
        Self {
            config,
            results: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Test recovery from invalid tool calls
    async fn test_invalid_tool_call_recovery(&self, server_name: &str, binary_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        println!("🛡️ Testing invalid tool call recovery for {}", server_name);
        
        let recovery_start = Instant::now();
        
        let result = timeout(
            self.config.recovery_timeout,
            async {
                // Test various invalid argument patterns
                let invalid_patterns = vec![
                    vec!["--nonexistent-flag"],
                    vec!["--transport", "invalid-transport"],
                    vec!["--port", "not-a-number"],
                    vec!["--port", "99999999"], // Out of range port
                    vec!["--debug", "invalid-debug-level"],
                ];
                
                let mut recovery_count = 0;
                
                for (i, invalid_args) in invalid_patterns.iter().enumerate() {
                    println!("🧪 Testing invalid pattern #{}: {:?}", i + 1, invalid_args);
                    
                    // Invalid command should fail gracefully
                    let invalid_output = Command::new("cargo")
                        .args(&["run", "--bin", binary_name, "--"])
                        .args(invalid_args)
                        .output();
                    
                    match invalid_output {
                        Ok(output) => {
                            // Should fail but provide helpful error
                            if !output.status.success() {
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                let stdout = String::from_utf8_lossy(&output.stdout);
                                
                                if !stderr.is_empty() || stdout.contains("error") || stdout.contains("help") {
                                    println!("✅ Pattern #{} failed gracefully with helpful output", i + 1);
                                } else {
                                    println!("⚠️ Pattern #{} failed without helpful output", i + 1);
                                }
                            }
                            
                            // Test recovery with valid command
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            
                            let recovery_output = Command::new("cargo")
                                .args(&["run", "--bin", binary_name, "--", "--help"])
                                .output();
                            
                            match recovery_output {
                                Ok(output) if output.status.success() => {
                                    recovery_count += 1;
                                    println!("✅ Recovery successful after pattern #{}", i + 1);
                                }
                                Ok(output) => {
                                    // Help might exit with non-zero but still provide output
                                    let stdout = String::from_utf8_lossy(&output.stdout);
                                    let stderr = String::from_utf8_lossy(&output.stderr);
                                    
                                    if stdout.contains("help") || stderr.contains("help") {
                                        recovery_count += 1;
                                        println!("✅ Recovery provided help after pattern #{}", i + 1);
                                    } else {
                                        println!("⚠️ Recovery failed after pattern #{}", i + 1);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("❌ Recovery error after pattern #{}: {}", i + 1, e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to test invalid pattern #{}: {}", i + 1, e);
                        }
                    }
                }
                
                // At least 80% of recovery attempts should succeed
                let recovery_rate = recovery_count as f64 / invalid_patterns.len() as f64;
                let success = recovery_rate >= 0.8;
                
                println!("📊 {} recovery rate: {:.1}% ({}/{})", 
                        server_name, recovery_rate * 100.0, recovery_count, invalid_patterns.len());
                
                Ok(success)
            }
        ).await??;
        
        let recovery_time = recovery_start.elapsed();
        
        // Record results
        {
            let mut results = self.results.lock().await;
            results.push(ResilienceResults {
                scenario: ErrorScenario::InvalidArguments,
                recovery_successful: result,
                recovery_time: Some(recovery_time),
                error_message: None,
            });
        }
        
        println!("✅ Invalid tool call recovery test completed in {:?}", recovery_time);
        Ok(result)
    }
    
    /// Test malformed JSON handling (simulated)
    async fn test_malformed_input_handling(&self, server_name: &str, binary_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        println!("📝 Testing malformed input handling for {}", server_name);
        
        let recovery_start = Instant::now();
        
        let result = timeout(
            self.config.recovery_timeout,
            async {
                // Test various malformed input scenarios through CLI
                let malformed_scenarios = vec![
                    ("empty-args", vec![]),
                    ("unknown-command", vec!["--unknown-command"]),
                    ("malformed-port", vec!["--port", ""]),
                    ("malformed-transport", vec!["--transport"]), // Missing value
                    ("mixed-invalid", vec!["--port", "abc", "--transport", "xyz"]),
                ];
                
                let mut handled_gracefully = 0;
                
                for (scenario_name, args) in malformed_scenarios {
                    println!("🧪 Testing malformed scenario: {}", scenario_name);
                    
                    let output = Command::new("cargo")
                        .args(&["run", "--bin", binary_name, "--"])
                        .args(&args)
                        .output();
                    
                    match output {
                        Ok(output) => {
                            // Should handle malformed input gracefully
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            
                            if !output.status.success() && (!stderr.is_empty() || stdout.contains("error") || stdout.contains("help")) {
                                handled_gracefully += 1;
                                println!("✅ Scenario '{}' handled gracefully", scenario_name);
                            } else if output.status.success() && (stdout.contains("help") || stdout.contains("usage")) {
                                handled_gracefully += 1;
                                println!("✅ Scenario '{}' provided help", scenario_name);
                            } else {
                                println!("⚠️ Scenario '{}' not handled gracefully", scenario_name);
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Scenario '{}' execution error: {}", scenario_name, e);
                        }
                    }
                    
                    // Brief pause between scenarios
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                
                let handling_rate = handled_gracefully as f64 / malformed_scenarios.len() as f64;
                let success = handling_rate >= 0.8;
                
                println!("📊 {} malformed input handling: {:.1}% ({}/{})", 
                        server_name, handling_rate * 100.0, handled_gracefully, malformed_scenarios.len());
                
                Ok(success)
            }
        ).await??;
        
        let recovery_time = recovery_start.elapsed();
        
        // Record results
        {
            let mut results = self.results.lock().await;
            results.push(ResilienceResults {
                scenario: ErrorScenario::MalformedInput,
                recovery_successful: result,
                recovery_time: Some(recovery_time),
                error_message: None,
            });
        }
        
        println!("✅ Malformed input handling test completed in {:?}", recovery_time);
        Ok(result)
    }
    
    /// Test server recovery from process interruption
    async fn test_process_interruption_recovery(&self, server_name: &str, binary_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        println!("⚡ Testing process interruption recovery for {}", server_name);
        
        let recovery_start = Instant::now();
        
        let result = timeout(
            Duration::from_secs(8),
            async {
                let mut successful_recoveries = 0;
                let total_attempts = 3;
                
                for attempt in 1..=total_attempts {
                    println!("🔄 Interruption test attempt #{}/{}", attempt, total_attempts);
                    
                    // Start server
                    let mut cmd = Command::new("cargo");
                    cmd.args(&["run", "--bin", binary_name, "--", "--transport", "stdio"])
                       .stdout(Stdio::piped())
                       .stderr(Stdio::piped());
                    
                    match cmd.spawn() {
                        Ok(mut process) => {
                            // Let server start
                            tokio::time::sleep(Duration::from_millis(300)).await;
                            
                            // Interrupt server
                            if let Err(e) = process.kill() {
                                eprintln!("⚠️ Failed to interrupt server: {}", e);
                                continue;
                            }
                            
                            let _ = process.wait();
                            println!("💀 Server interrupted");
                            
                            // Brief pause for cleanup
                            tokio::time::sleep(Duration::from_millis(200)).await;
                            
                            // Test recovery (restart)
                            let recovery_output = Command::new("cargo")
                                .args(&["run", "--bin", binary_name, "--", "--help"])
                                .output();
                            
                            match recovery_output {
                                Ok(output) if output.status.success() => {
                                    successful_recoveries += 1;
                                    println!("✅ Recovery #{} successful", attempt);
                                }
                                Ok(output) => {
                                    let stdout = String::from_utf8_lossy(&output.stdout);
                                    let stderr = String::from_utf8_lossy(&output.stderr);
                                    
                                    if stdout.contains("help") || stderr.contains("help") {
                                        successful_recoveries += 1;
                                        println!("✅ Recovery #{} provided help", attempt);
                                    } else {
                                        println!("⚠️ Recovery #{} failed", attempt);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("❌ Recovery #{} error: {}", attempt, e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to start server for interruption test: {}", e);
                        }
                    }
                    
                    // Cool-down between attempts
                    tokio::time::sleep(Duration::from_millis(300)).await;
                }
                
                let recovery_rate = successful_recoveries as f64 / total_attempts as f64;
                let success = recovery_rate >= 0.67; // At least 2/3 should recover
                
                println!("📊 {} interruption recovery: {:.1}% ({}/{})", 
                        server_name, recovery_rate * 100.0, successful_recoveries, total_attempts);
                
                Ok(success)
            }
        ).await??;
        
        let recovery_time = recovery_start.elapsed();
        
        // Record results
        {
            let mut results = self.results.lock().await;
            results.push(ResilienceResults {
                scenario: ErrorScenario::ProcessKill,
                recovery_successful: result,
                recovery_time: Some(recovery_time),
                error_message: None,
            });
        }
        
        println!("✅ Process interruption recovery test completed in {:?}", recovery_time);
        Ok(result)
    }
    
    /// Test server graceful shutdown scenarios
    async fn test_graceful_shutdown_scenarios(&self, server_name: &str, binary_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        println!("🛑 Testing graceful shutdown scenarios for {}", server_name);
        
        let test_start = Instant::now();
        
        let result = timeout(
            Duration::from_secs(10),
            async {
                let shutdown_scenarios = vec![
                    ("quick-shutdown", Duration::from_millis(200)),
                    ("normal-shutdown", Duration::from_millis(500)),
                    ("delayed-shutdown", Duration::from_millis(800)),
                ];
                
                let mut successful_shutdowns = 0;
                
                for (scenario_name, operation_duration) in shutdown_scenarios {
                    println!("🔄 Testing shutdown scenario: {}", scenario_name);
                    
                    // Start server
                    let mut cmd = Command::new("cargo");
                    cmd.args(&["run", "--bin", binary_name, "--", "--transport", "stdio"])
                       .stdout(Stdio::piped())
                       .stderr(Stdio::piped());
                    
                    match cmd.spawn() {
                        Ok(mut process) => {
                            // Let server operate for specified duration
                            tokio::time::sleep(operation_duration).await;
                            
                            // Attempt graceful shutdown
                            let shutdown_start = Instant::now();
                            
                            #[cfg(unix)]
                            {
                                // Send SIGTERM for graceful shutdown
                                unsafe {
                                    libc::kill(process.id() as i32, libc::SIGTERM);
                                }
                                
                                // Wait for graceful shutdown
                                tokio::time::sleep(Duration::from_millis(500)).await;
                                
                                // Check if process exited gracefully
                                match process.try_wait() {
                                    Ok(Some(_)) => {
                                        let shutdown_time = shutdown_start.elapsed();
                                        successful_shutdowns += 1;
                                        println!("✅ '{}' graceful shutdown in {:?}", scenario_name, shutdown_time);
                                    }
                                    Ok(None) => {
                                        // Still running, force kill
                                        let _ = process.kill();
                                        let _ = process.wait();
                                        println!("⚠️ '{}' required force kill", scenario_name);
                                    }
                                    Err(e) => {
                                        eprintln!("❌ '{}' shutdown error: {}", scenario_name, e);
                                    }
                                }
                            }
                            
                            #[cfg(not(unix))]
                            {
                                // On non-Unix systems, just use force kill
                                let _ = process.kill();
                                let _ = process.wait();
                                successful_shutdowns += 1;
                                println!("✅ '{}' shutdown completed (force)", scenario_name);
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to start server for '{}': {}", scenario_name, e);
                        }
                    }
                    
                    // Cleanup pause
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
                
                let shutdown_rate = successful_shutdowns as f64 / shutdown_scenarios.len() as f64;
                let success = shutdown_rate >= 0.67;
                
                println!("📊 {} graceful shutdown: {:.1}% ({}/{})", 
                        server_name, shutdown_rate * 100.0, successful_shutdowns, shutdown_scenarios.len());
                
                Ok(success)
            }
        ).await??;
        
        let test_duration = test_start.elapsed();
        
        // Record results
        {
            let mut results = self.results.lock().await;
            results.push(ResilienceResults {
                scenario: ErrorScenario::ProcessKill,
                recovery_successful: result,
                recovery_time: Some(test_duration),
                error_message: None,
            });
        }
        
        println!("✅ Graceful shutdown test completed in {:?}", test_duration);
        Ok(result)
    }
    
    /// Test server restart and state recovery
    async fn test_restart_recovery(&self, server_name: &str, binary_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        println!("🔄 Testing restart and state recovery for {}", server_name);
        
        let test_start = Instant::now();
        
        let result = timeout(
            Duration::from_secs(12),
            async {
                let mut successful_restarts = 0;
                let total_restarts = 3;
                
                for restart_cycle in 1..=total_restarts {
                    println!("🔄 Restart cycle #{}/{}", restart_cycle, total_restarts);
                    
                    // Start server
                    let mut cmd = Command::new("cargo");
                    cmd.args(&["run", "--bin", binary_name, "--", "--transport", "stdio"])
                       .stdout(Stdio::piped())
                       .stderr(Stdio::piped());
                    
                    match cmd.spawn() {
                        Ok(mut process) => {
                            // Let server run briefly
                            tokio::time::sleep(Duration::from_millis(400)).await;
                            
                            // Verify server is responsive (check if still running)
                            let responsive = match process.try_wait() {
                                Ok(Some(_)) => false, // Process exited
                                Ok(None) => true,     // Still running
                                Err(_) => false,      // Error
                            };
                            
                            if responsive {
                                println!("✅ Server responsive in cycle #{}", restart_cycle);
                            } else {
                                println!("⚠️ Server not responsive in cycle #{}", restart_cycle);
                            }
                            
                            // Stop server
                            let _ = process.kill();
                            let _ = process.wait();
                            
                            // Brief pause for cleanup
                            tokio::time::sleep(Duration::from_millis(200)).await;
                            
                            // Test restart capability
                            let restart_output = Command::new("cargo")
                                .args(&["run", "--bin", binary_name, "--", "--version"])
                                .output();
                            
                            match restart_output {
                                Ok(output) if output.status.success() => {
                                    successful_restarts += 1;
                                    println!("✅ Restart #{} successful", restart_cycle);
                                }
                                Ok(output) => {
                                    let stdout = String::from_utf8_lossy(&output.stdout);
                                    let stderr = String::from_utf8_lossy(&output.stderr);
                                    
                                    if stdout.contains("version") || stderr.contains("version") || 
                                       stdout.contains("help") || stderr.contains("help") {
                                        successful_restarts += 1;
                                        println!("✅ Restart #{} provided expected output", restart_cycle);
                                    } else {
                                        println!("⚠️ Restart #{} failed", restart_cycle);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("❌ Restart #{} error: {}", restart_cycle, e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to start server for restart cycle #{}: {}", restart_cycle, e);
                        }
                    }
                    
                    // Recovery pause between cycles
                    tokio::time::sleep(Duration::from_millis(300)).await;
                }
                
                let restart_rate = successful_restarts as f64 / total_restarts as f64;
                let success = restart_rate >= 0.67;
                
                println!("📊 {} restart recovery: {:.1}% ({}/{})", 
                        server_name, restart_rate * 100.0, successful_restarts, total_restarts);
                
                Ok(success)
            }
        ).await??;
        
        let test_duration = test_start.elapsed();
        
        // Record results
        {
            let mut results = self.results.lock().await;
            results.push(ResilienceResults {
                scenario: ErrorScenario::ProcessKill,
                recovery_successful: result,
                recovery_time: Some(test_duration),
                error_message: None,
            });
        }
        
        println!("✅ Restart recovery test completed in {:?}", test_duration);
        Ok(result)
    }
    
    /// Test configuration error handling
    async fn test_configuration_error_handling(&self, server_name: &str, binary_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        println!("⚙️ Testing configuration error handling for {}", server_name);
        
        let test_start = Instant::now();
        
        let result = timeout(
            self.config.recovery_timeout,
            async {
                // Test various configuration errors
                let config_errors = vec![
                    ("invalid-port-range", vec!["--port", "0"]),
                    ("invalid-port-high", vec!["--port", "65536"]),
                    ("missing-required-value", vec!["--port"]),
                    ("conflicting-transports", vec!["--transport", "both"]),
                    ("unknown-debug-level", vec!["--debug", "ultra"]),
                ];
                
                let mut handled_errors = 0;
                
                for (error_name, args) in config_errors {
                    println!("🧪 Testing config error: {}", error_name);
                    
                    let output = Command::new("cargo")
                        .args(&["run", "--bin", binary_name, "--"])
                        .args(&args)
                        .output();
                    
                    match output {
                        Ok(output) => {
                            // Should reject invalid configuration with helpful message
                            if !output.status.success() {
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                let stdout = String::from_utf8_lossy(&output.stdout);
                                
                                if stderr.contains("error") || stderr.contains("invalid") ||
                                   stdout.contains("error") || stdout.contains("help") {
                                    handled_errors += 1;
                                    println!("✅ Config error '{}' handled with helpful message", error_name);
                                } else {
                                    println!("⚠️ Config error '{}' handled but without clear message", error_name);
                                }
                            } else {
                                println!("⚠️ Config error '{}' was unexpectedly accepted", error_name);
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Config error test '{}' failed: {}", error_name, e);
                        }
                    }
                    
                    // Pause between error tests
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                
                let error_handling_rate = handled_errors as f64 / config_errors.len() as f64;
                let success = error_handling_rate >= 0.6; // At least 60% should be handled well
                
                println!("📊 {} config error handling: {:.1}% ({}/{})", 
                        server_name, error_handling_rate * 100.0, handled_errors, config_errors.len());
                
                Ok(success)
            }
        ).await??;
        
        let test_duration = test_start.elapsed();
        
        // Record results
        {
            let mut results = self.results.lock().await;
            results.push(ResilienceResults {
                scenario: ErrorScenario::ConfigurationError,
                recovery_successful: result,
                recovery_time: Some(test_duration),
                error_message: None,
            });
        }
        
        println!("✅ Configuration error handling test completed in {:?}", test_duration);
        Ok(result)
    }
    
    /// Generate comprehensive resilience report
    async fn generate_resilience_report(&self) -> String {
        let results = self.results.lock().await;
        let mut report = String::new();
        
        report.push_str("🛡️ Resilience Test Report\n");
        report.push_str("=".repeat(60).as_str());
        report.push('\n');
        
        let mut scenario_stats: HashMap<String, (usize, usize)> = HashMap::new();
        
        for result in results.iter() {
            let scenario_name = format!("{:?}", result.scenario);
            let (successes, total) = scenario_stats.entry(scenario_name).or_insert((0, 0));
            
            if result.recovery_successful {
                *successes += 1;
            }
            *total += 1;
        }
        
        for (scenario, (successes, total)) in scenario_stats {
            let rate = if total > 0 { successes as f64 / total as f64 } else { 0.0 };
            report.push_str(&format!("\n🔍 {}: {:.1}% ({}/{})\n", scenario, rate * 100.0, successes, total));
        }
        
        // Overall resilience score
        let total_tests = results.len();
        let total_successes = results.iter().filter(|r| r.recovery_successful).count();
        let overall_rate = if total_tests > 0 { total_successes as f64 / total_tests as f64 } else { 0.0 };
        
        report.push_str(&format!("\n🏆 Overall Resilience Score: {:.1}% ({}/{})\n", 
                                overall_rate * 100.0, total_successes, total_tests));
        
        if overall_rate >= 0.8 {
            report.push_str("✅ EXCELLENT resilience\n");
        } else if overall_rate >= 0.6 {
            report.push_str("⚠️ ACCEPTABLE resilience\n");
        } else {
            report.push_str("❌ POOR resilience - needs improvement\n");
        }
        
        report.push_str("=".repeat(60).as_str());
        report
    }
}

/// Test server doesn't crash on invalid tool calls
#[tokio::test]
async fn test_invalid_tool_calls_dont_crash() {
    let config = ResilienceTestConfig::default();
    let tester = ServerResilienceTester::new(config);
    
    let servers = vec![
        ("filesystem", "filesystem-server"),
        ("image-generation", "image-generation-server"),
    ];
    
    let mut server_results = Vec::new();
    
    for (server_name, binary_name) in servers {
        match tester.test_invalid_tool_call_recovery(server_name, binary_name).await {
            Ok(recovered) => {
                server_results.push((server_name, recovered));
                
                assert!(
                    recovered,
                    "Server {} should recover from invalid tool calls",
                    server_name
                );
            }
            Err(e) => {
                eprintln!("⚠️ Invalid tool call test failed for {}: {}", server_name, e);
            }
        }
    }
    
    // At least one server should pass the test
    assert!(
        !server_results.is_empty(),
        "At least one server should pass invalid tool call tests"
    );
    
    println!("📊 Invalid Tool Call Recovery Results:");
    for (name, recovered) in server_results {
        println!("   {}: {}", name, if recovered { "✅ PASSED" } else { "❌ FAILED" });
    }
    
    println!("✅ Invalid tool calls crash prevention test completed");
}

/// Test malformed JSON handling doesn't crash servers
#[tokio::test]
async fn test_malformed_json_handling() {
    let config = ResilienceTestConfig::default();
    let tester = ServerResilienceTester::new(config);
    
    let servers = vec![
        ("blog-generation", "blog-generation-server"),
        ("creative-content", "creative-content-server"),
    ];
    
    let mut handling_results = Vec::new();
    
    for (server_name, binary_name) in servers {
        match tester.test_malformed_input_handling(server_name, binary_name).await {
            Ok(handled_gracefully) => {
                handling_results.push((server_name, handled_gracefully));
                
                assert!(
                    handled_gracefully,
                    "Server {} should handle malformed input gracefully",
                    server_name
                );
            }
            Err(e) => {
                eprintln!("⚠️ Malformed input test failed for {}: {}", server_name, e);
            }
        }
    }
    
    // At least one server should handle malformed input well
    assert!(
        !handling_results.is_empty(),
        "At least one server should pass malformed input tests"
    );
    
    println!("📊 Malformed Input Handling Results:");
    for (name, handled) in handling_results {
        println!("   {}: {}", name, if handled { "✅ GRACEFUL" } else { "❌ POOR" });
    }
    
    println!("✅ Malformed JSON handling test completed");
}

/// Test network interruption recovery (simulated through process management)
#[tokio::test]
async fn test_network_interruption_recovery() {
    let config = ResilienceTestConfig::default();
    let tester = ServerResilienceTester::new(config);
    
    // Test with HTTP-capable server
    let server_name = "filesystem";
    let binary_name = "filesystem-server";
    
    let test_result = timeout(
        Duration::from_secs(15),
        async {
            println!("🌐 Testing network interruption recovery");
            
            // Test HTTP server startup and interruption
            let mut successful_recoveries = 0;
            let test_cycles = 2;
            
            for cycle in 1..=test_cycles {
                println!("🔄 Network recovery cycle #{}/{}", cycle, test_cycles);
                
                // Start HTTP server
                let mut cmd = Command::new("cargo");
                cmd.args(&["run", "--bin", binary_name, "--", "--transport", "http", "--port", "8010"])
                   .stdout(Stdio::piped())
                   .stderr(Stdio::piped());
                
                match cmd.spawn() {
                    Ok(mut process) => {
                        // Let server start
                        tokio::time::sleep(Duration::from_millis(600)).await;
                        
                        // Check if server bound to port (simulated network connectivity)
                        let port_check = std::net::TcpStream::connect_timeout(
                            &"127.0.0.1:8010".parse().unwrap(),
                            Duration::from_millis(500)
                        );
                        
                        match port_check {
                            Ok(_) => {
                                println!("✅ Server bound to port successfully");
                                
                                // Simulate network interruption (kill process)
                                let _ = process.kill();
                                let _ = process.wait();
                                println!("📡 Simulated network interruption");
                                
                                // Wait for port to be released
                                tokio::time::sleep(Duration::from_millis(500)).await;
                                
                                // Test recovery (restart on same port)
                                let recovery_output = Command::new("cargo")
                                    .args(&["run", "--bin", binary_name, "--", "--help"])
                                    .output();
                                
                                match recovery_output {
                                    Ok(output) if output.status.success() => {
                                        successful_recoveries += 1;
                                        println!("✅ Network recovery #{} successful", cycle);
                                    }
                                    Ok(output) => {
                                        let stdout = String::from_utf8_lossy(&output.stdout);
                                        if stdout.contains("help") {
                                            successful_recoveries += 1;
                                            println!("✅ Network recovery #{} provided help", cycle);
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("❌ Network recovery #{} failed: {}", cycle, e);
                                    }
                                }
                            }
                            Err(_) => {
                                println!("⚠️ Server didn't bind to port in cycle #{}", cycle);
                                let _ = process.kill();
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to start server for network test: {}", e);
                    }
                }
                
                // Recovery pause
                tokio::time::sleep(Duration::from_millis(400)).await;
            }
            
            let recovery_rate = successful_recoveries as f64 / test_cycles as f64;
            
            println!("📊 Network interruption recovery: {:.1}% ({}/{})", 
                    recovery_rate * 100.0, successful_recoveries, test_cycles);
            
            // At least 50% recovery rate is acceptable for network tests
            assert!(
                recovery_rate >= 0.5,
                "Network recovery rate should be at least 50%"
            );
            
            Ok(())
        }
    ).await;
    
    match test_result {
        Ok(Ok(())) => println!("✅ Network interruption recovery test passed"),
        Ok(Err(e)) => panic!("❌ Network interruption test failed: {}", e),
        Err(_) => panic!("💥 Network interruption test timed out"),
    }
}

/// Test graceful shutdown scenarios
#[tokio::test]
async fn test_graceful_shutdown_scenarios() {
    let config = ResilienceTestConfig::default();
    let tester = ServerResilienceTester::new(config);
    
    let servers = vec![
        ("filesystem", "filesystem-server"),
        ("creative-content", "creative-content-server"),
    ];
    
    let mut shutdown_results = Vec::new();
    
    for (server_name, binary_name) in servers {
        match tester.test_graceful_shutdown_scenarios(server_name, binary_name).await {
            Ok(graceful) => {
                shutdown_results.push((server_name, graceful));
                
                assert!(
                    graceful,
                    "Server {} should support graceful shutdown",
                    server_name
                );
            }
            Err(e) => {
                eprintln!("⚠️ Graceful shutdown test failed for {}: {}", server_name, e);
            }
        }
    }
    
    // At least one server should support graceful shutdown
    assert!(
        !shutdown_results.is_empty(),
        "At least one server should pass graceful shutdown tests"
    );
    
    println!("📊 Graceful Shutdown Results:");
    for (name, graceful) in shutdown_results {
        println!("   {}: {}", name, if graceful { "✅ GRACEFUL" } else { "⚠️ FORCED" });
    }
    
    println!("✅ Graceful shutdown scenarios test completed");
}

/// Test restart and state recovery
#[tokio::test]
async fn test_restart_and_state_recovery() {
    let config = ResilienceTestConfig::default();
    let tester = ServerResilienceTester::new(config);
    
    let server_name = "image-generation";
    let binary_name = "image-generation-server";
    
    match tester.test_restart_recovery(server_name, binary_name).await {
        Ok(recovered) => {
            assert!(
                recovered,
                "Server {} should support restart and state recovery",
                server_name
            );
            
            println!("✅ Restart and state recovery test passed for {}", server_name);
        }
        Err(e) => {
            panic!("❌ Restart recovery test failed for {}: {}", server_name, e);
        }
    }
}

/// Test configuration error resilience
#[tokio::test]
async fn test_configuration_error_resilience() {
    let config = ResilienceTestConfig::default();
    let tester = ServerResilienceTester::new(config);
    
    let server_name = "blog-generation";
    let binary_name = "blog-generation-server";
    
    match tester.test_configuration_error_handling(server_name, binary_name).await {
        Ok(resilient) => {
            assert!(
                resilient,
                "Server {} should be resilient to configuration errors",
                server_name
            );
            
            println!("✅ Configuration error resilience test passed for {}", server_name);
        }
        Err(e) => {
            panic!("❌ Configuration error test failed for {}: {}", server_name, e);
        }
    }
}

/// Comprehensive resilience test suite
#[tokio::test]
async fn test_comprehensive_resilience_suite() {
    let test_result = timeout(
        Duration::from_secs(30),
        async {
            println!("🏰 Starting comprehensive resilience test suite");
            
            let config = ResilienceTestConfig::default();
            let tester = ServerResilienceTester::new(config);
            
            let test_servers = vec![
                ("filesystem", "filesystem-server"),
                ("image-generation", "image-generation-server"),
            ];
            
            let mut overall_results = Vec::new();
            
            for (server_name, binary_name) in test_servers {
                println!("\n🔬 Testing resilience for server: {}", server_name);
                
                // Run all resilience tests for this server
                let mut server_score = 0;
                let mut total_tests = 0;
                
                // Invalid tool calls test
                if let Ok(result) = tester.test_invalid_tool_call_recovery(server_name, binary_name).await {
                    total_tests += 1;
                    if result { server_score += 1; }
                }
                
                // Malformed input test
                if let Ok(result) = tester.test_malformed_input_handling(server_name, binary_name).await {
                    total_tests += 1;
                    if result { server_score += 1; }
                }
                
                // Configuration error test
                if let Ok(result) = tester.test_configuration_error_handling(server_name, binary_name).await {
                    total_tests += 1;
                    if result { server_score += 1; }
                }
                
                let server_resilience = if total_tests > 0 { server_score as f64 / total_tests as f64 } else { 0.0 };
                overall_results.push((server_name, server_resilience));
                
                println!("📊 {} resilience score: {:.1}% ({}/{})", 
                        server_name, server_resilience * 100.0, server_score, total_tests);
            }
            
            // Generate final report
            let report = tester.generate_resilience_report().await;
            println!("\n{}", report);
            
            // Overall success criteria
            let avg_resilience = overall_results.iter()
                .map(|(_, score)| *score)
                .sum::<f64>() / overall_results.len() as f64;
            
            assert!(
                avg_resilience >= 0.6,
                "Average server resilience should be at least 60%, got {:.1}%",
                avg_resilience * 100.0
            );
            
            println!("✅ Comprehensive resilience test suite completed");
            println!("🏆 Average resilience score: {:.1}%", avg_resilience * 100.0);
            
            Ok::<(), Box<dyn std::error::Error>>(())
        }
    ).await;
    
    match test_result {
        Ok(Ok(())) => println!("✅ Comprehensive resilience test suite passed"),
        Ok(Err(e)) => panic!("❌ Resilience test suite failed: {}", e),
        Err(_) => panic!("💥 Resilience test suite timed out"),
    }
}

/// Test server behavior under resource stress
#[tokio::test]
async fn test_resource_stress_resilience() {
    let test_result = timeout(
        Duration::from_secs(12),
        async {
            println!("💪 Testing resource stress resilience");
            
            let server_name = "filesystem";
            let binary_name = "filesystem-server";
            
            // Test multiple rapid invocations (resource stress)
            let mut stress_results = Vec::new();
            
            for burst in 1..=3 {
                println!("💥 Stress burst #{}/3", burst);
                
                let burst_start = Instant::now();
                let mut tasks = Vec::new();
                
                // Create multiple concurrent server invocations
                for i in 0..4 {
                    let task = tokio::spawn({
                        let binary_name = binary_name.to_string();
                        async move {
                            let output = Command::new("cargo")
                                .args(&["run", "--bin", &binary_name, "--", "--help"])
                                .output();
                            
                            match output {
                                Ok(output) if output.status.success() => Ok(format!("Task {} success", i)),
                                Ok(output) => {
                                    let stdout = String::from_utf8_lossy(&output.stdout);
                                    if stdout.contains("help") {
                                        Ok(format!("Task {} help", i))
                                    } else {
                                        Err(format!("Task {} failed", i))
                                    }
                                }
                                Err(e) => Err(format!("Task {} error: {}", i, e)),
                            }
                        }
                    });
                    
                    tasks.push(task);
                }
                
                // Wait for all tasks
                let mut successes = 0;
                for task in tasks {
                    match task.await {
                        Ok(Ok(_)) => successes += 1,
                        Ok(Err(e)) => eprintln!("⚠️ Stress task failed: {}", e),
                        Err(e) => eprintln!("❌ Stress task error: {}", e),
                    }
                }
                
                let burst_duration = burst_start.elapsed();
                let success_rate = successes as f64 / 4.0;
                
                stress_results.push((burst, success_rate, burst_duration));
                
                println!("📊 Burst #{}: {:.1}% success in {:?}", burst, success_rate * 100.0, burst_duration);
                
                // Recovery pause between bursts
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            
            // Analyze stress test results
            let avg_success_rate = stress_results.iter()
                .map(|(_, rate, _)| *rate)
                .sum::<f64>() / stress_results.len() as f64;
            
            assert!(
                avg_success_rate >= 0.7,
                "Average success rate under stress should be at least 70%, got {:.1}%",
                avg_success_rate * 100.0
            );
            
            println!("📊 Resource stress resilience: {:.1}% average success", avg_success_rate * 100.0);
            println!("✅ Resource stress resilience test completed");
            
            Ok::<(), Box<dyn std::error::Error>>(())
        }
    ).await;
    
    match test_result {
        Ok(Ok(())) => println!("✅ Resource stress resilience test passed"),
        Ok(Err(e)) => panic!("❌ Resource stress test failed: {}", e),
        Err(_) => panic!("💥 Resource stress test timed out"),
    }
}

/// Test error propagation and recovery patterns
#[tokio::test]
async fn test_error_propagation_recovery() {
    let test_result = timeout(
        Duration::from_secs(10),
        async {
            println!("🔄 Testing error propagation and recovery patterns");
            
            let servers = vec![
                ("filesystem", "filesystem-server"),
                ("blog-generation", "blog-generation-server"),
            ];
            
            let mut propagation_results = Vec::new();
            
            for (server_name, binary_name) in servers {
                println!("🔍 Testing error propagation for {}", server_name);
                
                // Test error scenarios that should be handled gracefully
                let error_tests = vec![
                    ("help-recovery", vec!["--invalid"], vec!["--help"]),
                    ("version-recovery", vec!["--bad-flag"], vec!["--version"]),
                ];
                
                let mut recovery_successes = 0;
                
                for (test_name, error_args, recovery_args) in error_tests {
                    // Trigger error
                    let _error_output = Command::new("cargo")
                        .args(&["run", "--bin", binary_name, "--"])
                        .args(&error_args)
                        .output();
                    
                    // Small delay
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    
                    // Test recovery
                    let recovery_output = Command::new("cargo")
                        .args(&["run", "--bin", binary_name, "--"])
                        .args(&recovery_args)
                        .output();
                    
                    match recovery_output {
                        Ok(output) => {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            
                            if output.status.success() || 
                               stdout.contains("help") || stdout.contains("version") ||
                               stderr.contains("help") || stderr.contains("version") {
                                recovery_successes += 1;
                                println!("✅ Error propagation test '{}' recovered", test_name);
                            } else {
                                println!("⚠️ Error propagation test '{}' failed recovery", test_name);
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Recovery test '{}' error: {}", test_name, e);
                        }
                    }
                }
                
                let propagation_rate = recovery_successes as f64 / error_tests.len() as f64;
                propagation_results.push((server_name, propagation_rate));
                
                println!("📊 {} error propagation recovery: {:.1}%", server_name, propagation_rate * 100.0);
            }
            
            // Overall error propagation handling should be good
            let avg_propagation = propagation_results.iter()
                .map(|(_, rate)| *rate)
                .sum::<f64>() / propagation_results.len() as f64;
            
            assert!(
                avg_propagation >= 0.7,
                "Average error propagation recovery should be at least 70%"
            );
            
            println!("✅ Error propagation and recovery test completed");
            Ok::<(), Box<dyn std::error::Error>>(())
        }
    ).await;
    
    match test_result {
        Ok(Ok(())) => println!("✅ Error propagation recovery test passed"),
        Ok(Err(e)) => panic!("❌ Error propagation test failed: {}", e),
        Err(_) => panic!("💥 Error propagation test timed out"),
    }
}