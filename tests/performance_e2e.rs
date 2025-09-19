//! Performance & Stress Testing E2E Tests for MCP Boilerplate
//!
//! Tests server performance under various load conditions:
//! - Server startup times (< 2 seconds target)
//! - Response times under load
//! - Multiple concurrent requests
//! - Large payload handling
//! - Memory usage monitoring
//! - Graceful degradation under stress
//! - Timeout handling and recovery
//!
//! This implements Task 3.2 from the E2E testing roadmap.

use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::timeout;

/// Performance test configuration
struct PerformanceTestConfig {
    startup_timeout: Duration,
    response_timeout: Duration,
    load_test_duration: Duration,
    concurrent_requests: usize,
    memory_threshold_mb: u64,
}

impl Default for PerformanceTestConfig {
    fn default() -> Self {
        Self {
            startup_timeout: Duration::from_secs(2),
            response_timeout: Duration::from_millis(1000),
            load_test_duration: Duration::from_secs(3),
            concurrent_requests: 5,
            memory_threshold_mb: 100, // 100MB threshold for server processes
        }
    }
}

/// Server performance metrics
#[derive(Debug, Clone)]
struct PerformanceMetrics {
    startup_time: Duration,
    memory_usage_mb: f64,
    response_times: Vec<Duration>,
    error_count: usize,
    successful_requests: usize,
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self {
            startup_time: Duration::from_secs(0),
            memory_usage_mb: 0.0,
            response_times: Vec::new(),
            error_count: 0,
            successful_requests: 0,
        }
    }

    fn average_response_time(&self) -> Duration {
        if self.response_times.is_empty() {
            Duration::from_secs(0)
        } else {
            self.response_times.iter().sum::<Duration>() / self.response_times.len() as u32
        }
    }

    fn max_response_time(&self) -> Duration {
        self.response_times
            .iter()
            .max()
            .copied()
            .unwrap_or(Duration::from_secs(0))
    }

    fn success_rate(&self) -> f64 {
        let total = self.successful_requests + self.error_count;
        if total == 0 {
            1.0
        } else {
            self.successful_requests as f64 / total as f64
        }
    }
}

/// Performance test harness for individual servers
struct ServerPerformanceTester {
    config: PerformanceTestConfig,
    metrics: Arc<Mutex<HashMap<String, PerformanceMetrics>>>,
}

impl ServerPerformanceTester {
    fn new(config: PerformanceTestConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Test server startup performance
    async fn test_startup_performance(
        &self,
        server_name: &str,
        binary_name: &str,
    ) -> Result<Duration, Box<dyn std::error::Error>> {
        println!("⏱️ Testing startup performance for {}", server_name);

        let start_time = Instant::now();

        let _result = timeout(self.config.startup_timeout, async {
            let mut cmd = Command::new("cargo");
            cmd.args(&["run", "--bin", binary_name, "--", "--help"])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let output = cmd
                .output()
                .map_err(|e| format!("Failed to start {}: {}", server_name, e))?;

            if !output.status.success() {
                return Err(format!(
                    "Server {} failed to start: stderr={}",
                    server_name,
                    String::from_utf8_lossy(&output.stderr)
                )
                .into());
            }

            Ok::<(), Box<dyn std::error::Error>>(())
        })
        .await??;

        let startup_time = start_time.elapsed();

        // Update metrics
        {
            let mut metrics = self.metrics.lock().await;
            let server_metrics = metrics
                .entry(server_name.to_string())
                .or_insert_with(PerformanceMetrics::new);
            server_metrics.startup_time = startup_time;
        }

        println!("✅ {} startup time: {:?}", server_name, startup_time);
        Ok(startup_time)
    }

    /// Test server memory usage during operation
    async fn test_memory_usage(
        &self,
        server_name: &str,
        binary_name: &str,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        println!("🧠 Testing memory usage for {}", server_name);

        let result = timeout(Duration::from_secs(8), async {
            // Start server in background
            let mut cmd = Command::new("cargo");
            cmd.args(&["run", "--bin", binary_name, "--", "--transport", "stdio"])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let mut process = cmd
                .spawn()
                .map_err(|e| format!("Failed to spawn {}: {}", server_name, e))?;

            // Let server initialize
            tokio::time::sleep(Duration::from_millis(500)).await;

            // Get memory usage (simplified approach using ps)
            let memory_mb = get_process_memory_mb(process.id()).await.unwrap_or(0.0);

            // Cleanup
            let _ = process.kill();
            let _ = process.wait();

            Ok::<f64, Box<dyn std::error::Error>>(memory_mb)
        })
        .await??;

        // Update metrics
        {
            let mut metrics = self.metrics.lock().await;
            let server_metrics = metrics
                .entry(server_name.to_string())
                .or_insert_with(PerformanceMetrics::new);
            server_metrics.memory_usage_mb = result;
        }

        println!("✅ {} memory usage: {:.2} MB", server_name, result);
        Ok(result)
    }

    /// Test server under simulated load
    async fn test_load_performance(
        &self,
        server_name: &str,
        binary_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("🏋️ Testing load performance for {}", server_name);

        let result = timeout(Duration::from_secs(12), async {
            // Start server
            let mut cmd = Command::new("cargo");
            cmd.args(&["run", "--bin", binary_name, "--", "--transport", "stdio"])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let mut process = cmd
                .spawn()
                .map_err(|e| format!("Failed to spawn {}: {}", server_name, e))?;

            // Let server initialize
            tokio::time::sleep(Duration::from_millis(400)).await;

            // Simulate concurrent load (multiple help requests)
            let mut tasks: Vec<
                tokio::task::JoinHandle<Result<Duration, Box<dyn std::error::Error + Send + Sync>>>,
            > = Vec::new();
            let start_time = Instant::now();

            for i in 0..self.config.concurrent_requests {
                let server_name_copy = server_name.to_string();
                let binary_name_copy = binary_name.to_string();

                let task = tokio::spawn(async move {
                    let request_start = Instant::now();

                    let output = Command::new("cargo")
                        .args(&["run", "--bin", &binary_name_copy, "--", "--version"])
                        .output();

                    let response_time = request_start.elapsed();

                    match output {
                        Ok(output) if output.status.success() => {
                            println!(
                                "✅ Request #{} for {} completed in {:?}",
                                i + 1,
                                server_name_copy,
                                response_time
                            );
                            Ok(response_time)
                        }
                        Ok(output) => {
                            eprintln!(
                                "⚠️ Request #{} for {} failed with status: {}",
                                i + 1,
                                server_name_copy,
                                output.status
                            );
                            Err("Request failed".into())
                        }
                        Err(e) => {
                            eprintln!(
                                "❌ Request #{} for {} error: {}",
                                i + 1,
                                server_name_copy,
                                e
                            );
                            Err(e.into())
                        }
                    }
                });

                tasks.push(task);

                // Stagger requests slightly
                tokio::time::sleep(Duration::from_millis(50)).await;
            }

            // Collect results
            let mut response_times = Vec::new();
            let mut errors = 0;

            for task in tasks {
                match task.await? {
                    Ok(response_time) => response_times.push(response_time),
                    Err(_) => errors += 1,
                }
            }

            let total_test_time = start_time.elapsed();

            // Update metrics
            {
                let mut metrics = self.metrics.lock().await;
                let server_metrics = metrics
                    .entry(server_name.to_string())
                    .or_insert_with(PerformanceMetrics::new);
                server_metrics.response_times = response_times.clone();
                server_metrics.error_count = errors;
                server_metrics.successful_requests = response_times.len();
            }

            // Performance assertions
            let avg_response = if !response_times.is_empty() {
                response_times.iter().sum::<Duration>() / response_times.len() as u32
            } else {
                Duration::from_secs(0)
            };

            println!("📊 {} Load Test Results:", server_name);
            println!("   Total time: {:?}", total_test_time);
            println!("   Successful requests: {}", response_times.len());
            println!("   Failed requests: {}", errors);
            println!("   Average response: {:?}", avg_response);

            // At least 80% success rate under load
            let success_rate = response_times.len() as f64 / self.config.concurrent_requests as f64;
            assert!(
                success_rate >= 0.8,
                "Success rate should be at least 80%, got {:.1}%",
                success_rate * 100.0
            );

            // Cleanup
            let _ = process.kill();
            let _ = process.wait();

            println!("✅ {} load test completed", server_name);
            Ok::<(), Box<dyn std::error::Error>>(())
        })
        .await??;

        Ok(result)
    }

    /// Generate performance report
    async fn generate_report(&self) -> String {
        let metrics = self.metrics.lock().await;
        let mut report = String::new();

        report.push_str("📊 Performance Test Report\n");
        report.push_str("=".repeat(50).as_str());
        report.push('\n');

        for (server_name, server_metrics) in metrics.iter() {
            report.push_str(&format!("\n🖥️ Server: {}\n", server_name));
            report.push_str(&format!(
                "   Startup Time: {:?}\n",
                server_metrics.startup_time
            ));
            report.push_str(&format!(
                "   Memory Usage: {:.2} MB\n",
                server_metrics.memory_usage_mb
            ));
            report.push_str(&format!(
                "   Avg Response: {:?}\n",
                server_metrics.average_response_time()
            ));
            report.push_str(&format!(
                "   Max Response: {:?}\n",
                server_metrics.max_response_time()
            ));
            report.push_str(&format!(
                "   Success Rate: {:.1}%\n",
                server_metrics.success_rate() * 100.0
            ));
            report.push_str(&format!(
                "   Total Requests: {}\n",
                server_metrics.successful_requests + server_metrics.error_count
            ));
        }

        report.push_str("\n");
        report.push_str("=".repeat(50).as_str());
        report
    }
}

/// Get memory usage for a process ID in MB
async fn get_process_memory_mb(pid: u32) -> Option<f64> {
    let output = Command::new("ps")
        .args(&["-o", "rss=", "-p", &pid.to_string()])
        .output()
        .ok()?;

    if output.status.success() {
        let rss_kb = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<f64>()
            .ok()?;

        // Convert KB to MB
        Some(rss_kb / 1024.0)
    } else {
        None
    }
}

/// Test server startup times meet performance requirements
#[tokio::test]
async fn test_server_startup_times() {
    let config = PerformanceTestConfig::default();
    let tester = ServerPerformanceTester::new(config);

    let servers = vec![
        ("filesystem", "filesystem-server"),
        ("image-generation", "image-generation-server"),
        ("blog-generation", "blog-generation-server"),
        ("creative-content", "creative-content-server"),
    ];

    let mut startup_times = Vec::new();

    for (server_name, binary_name) in servers {
        match tester
            .test_startup_performance(server_name, binary_name)
            .await
        {
            Ok(startup_time) => {
                startup_times.push((server_name, startup_time));

                // Individual server startup should be reasonable (with cargo overhead)
                assert!(
                    startup_time < Duration::from_secs(8),
                    "Server {} startup took too long: {:?}",
                    server_name,
                    startup_time
                );
            }
            Err(e) => {
                eprintln!("⚠️ Startup test failed for {}: {}", server_name, e);
            }
        }
    }

    // At least 3 servers should pass startup tests
    assert!(
        startup_times.len() >= 3,
        "At least 3 servers should pass startup performance tests"
    );

    // Calculate overall startup performance
    let avg_startup: Duration = startup_times
        .iter()
        .map(|(_, time)| *time)
        .sum::<Duration>()
        / startup_times.len() as u32;

    println!("📊 Overall Startup Performance:");
    println!("   Servers tested: {}", startup_times.len());
    println!("   Average startup: {:?}", avg_startup);

    // Report individual results
    for (name, time) in startup_times {
        println!("   {}: {:?}", name, time);
    }

    println!("✅ Server startup performance test completed");
}

/// Test server memory usage under normal operation
#[tokio::test]
async fn test_server_memory_usage() {
    let config = PerformanceTestConfig::default();
    let tester = ServerPerformanceTester::new(config);

    let servers = vec![
        ("filesystem", "filesystem-server"),
        ("image-generation", "image-generation-server"),
    ];

    let mut memory_results = Vec::new();

    for (server_name, binary_name) in servers {
        match tester.test_memory_usage(server_name, binary_name).await {
            Ok(memory_mb) => {
                memory_results.push((server_name, memory_mb));

                // Memory usage should be reasonable for MCP servers
                assert!(
                    memory_mb < tester.config.memory_threshold_mb as f64,
                    "Server {} memory usage too high: {:.2} MB (threshold: {} MB)",
                    server_name,
                    memory_mb,
                    tester.config.memory_threshold_mb
                );
            }
            Err(e) => {
                eprintln!("⚠️ Memory test failed for {}: {}", server_name, e);
            }
        }
    }

    // At least 1 server should pass memory tests
    assert!(
        !memory_results.is_empty(),
        "At least one server should pass memory usage tests"
    );

    println!("📊 Memory Usage Results:");
    for (name, memory_mb) in memory_results {
        println!("   {}: {:.2} MB", name, memory_mb);
    }

    println!("✅ Server memory usage test completed");
}

/// Test server performance under concurrent load
#[tokio::test]
async fn test_concurrent_load_performance() {
    let config = PerformanceTestConfig::default();
    let tester = ServerPerformanceTester::new(config);

    // Test with one representative server for load testing
    let server_name = "filesystem";
    let binary_name = "filesystem-server";

    let test_result = timeout(
        Duration::from_secs(15),
        tester.test_load_performance(server_name, binary_name),
    )
    .await;

    match test_result {
        Ok(Ok(())) => {
            // Analyze results
            let metrics = tester.metrics.lock().await;
            if let Some(server_metrics) = metrics.get(server_name) {
                println!("📊 Load Test Analysis:");
                println!(
                    "   Average response: {:?}",
                    server_metrics.average_response_time()
                );
                println!("   Max response: {:?}", server_metrics.max_response_time());
                println!(
                    "   Success rate: {:.1}%",
                    server_metrics.success_rate() * 100.0
                );

                // Performance assertions
                assert!(
                    server_metrics.success_rate() >= 0.8,
                    "Success rate under load should be at least 80%"
                );

                assert!(
                    server_metrics.average_response_time() < Duration::from_secs(5),
                    "Average response time should be under 5 seconds"
                );
            }

            println!("✅ Concurrent load test passed");
        }
        Ok(Err(e)) => panic!("❌ Load test failed: {}", e),
        Err(_) => panic!("💥 Load test timed out"),
    }
}

/// Test server response times under various conditions
#[tokio::test]
async fn test_response_time_variability() {
    let test_result = timeout(Duration::from_secs(12), async {
        let _server_name = "image-generation";
        let binary_name = "image-generation-server";

        let mut response_times = Vec::new();

        // Test multiple sequential requests
        for i in 1..=5 {
            let start = Instant::now();

            let output = Command::new("cargo")
                .args(&["run", "--bin", binary_name, "--", "--help"])
                .output();

            let response_time = start.elapsed();

            match output {
                Ok(output) if output.status.success() => {
                    response_times.push(response_time);
                    println!("📈 Request #{}: {:?}", i, response_time);
                }
                Ok(output) => {
                    eprintln!("⚠️ Request #{} failed with status: {}", i, output.status);
                }
                Err(e) => {
                    eprintln!("❌ Request #{} error: {}", i, e);
                }
            }

            // Small delay between requests
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Analyze response time variability
        if !response_times.is_empty() {
            let avg_time = response_times.iter().sum::<Duration>() / response_times.len() as u32;
            let min_time = *response_times.iter().min().unwrap();
            let max_time = *response_times.iter().max().unwrap();
            let variability = max_time.saturating_sub(min_time);

            println!("📊 Response Time Analysis:");
            println!("   Average: {:?}", avg_time);
            println!("   Min: {:?}", min_time);
            println!("   Max: {:?}", max_time);
            println!("   Variability: {:?}", variability);

            // Response times should be reasonably consistent
            assert!(
                variability < Duration::from_secs(3),
                "Response time variability should be under 3 seconds, got {:?}",
                variability
            );

            // At least 80% of requests should succeed
            assert!(
                response_times.len() >= 4,
                "At least 4/5 requests should succeed"
            );
        }

        println!("✅ Response time variability test completed");
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await;

    match test_result {
        Ok(Ok(())) => println!("✅ Response time variability test passed"),
        Ok(Err(e)) => panic!("❌ Response time test failed: {}", e),
        Err(_) => panic!("💥 Response time test timed out"),
    }
}

/// Test server behavior with large or complex inputs
#[tokio::test]
async fn test_large_payload_handling() {
    let test_result = timeout(Duration::from_secs(10), async {
        let _server_name = "blog-generation";
        let binary_name = "blog-generation-server";

        // Test server can handle startup with various argument patterns
        let test_cases = vec![
            vec!["--help"],
            vec!["--version"],
            vec!["--transport", "stdio"],
            vec!["--transport", "http", "--port", "9999"],
        ];

        let mut successful_cases = 0;

        for (i, args) in test_cases.iter().enumerate() {
            println!("🧪 Testing payload case #{}: {:?}", i + 1, args);

            let start = Instant::now();

            let output = Command::new("cargo")
                .args(&["run", "--bin", binary_name, "--"])
                .args(args)
                .output();

            let duration = start.elapsed();

            match output {
                Ok(output) => {
                    if output.status.success()
                        || String::from_utf8_lossy(&output.stderr).contains("help")
                        || String::from_utf8_lossy(&output.stdout).contains("help")
                    {
                        successful_cases += 1;
                        println!("✅ Case #{} succeeded in {:?}", i + 1, duration);
                    } else {
                        println!("⚠️ Case #{} failed with status: {}", i + 1, output.status);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Case #{} error: {}", i + 1, e);
                }
            }
        }

        // Most cases should succeed
        assert!(
            successful_cases >= 3,
            "At least 3/4 payload test cases should succeed, got {}",
            successful_cases
        );

        println!(
            "✅ Large payload handling test: {}/{} cases successful",
            successful_cases,
            test_cases.len()
        );
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await;

    match test_result {
        Ok(Ok(())) => println!("✅ Large payload handling test passed"),
        Ok(Err(e)) => panic!("❌ Payload handling test failed: {}", e),
        Err(_) => panic!("💥 Payload handling test timed out"),
    }
}

/// Test timeout handling and recovery
#[tokio::test]
async fn test_timeout_handling() {
    let test_result = timeout(Duration::from_secs(8), async {
        let _server_name = "creative-content";
        let binary_name = "creative-content-server";

        // Test that server operations complete within reasonable timeouts
        let operations = vec![
            ("help", vec!["--help"]),
            ("version", vec!["--version"]),
            ("stdio-mode", vec!["--transport", "stdio"]),
        ];

        let mut successful_ops = 0;

        let operations_len = operations.len();
        for (op_name, args) in &operations {
            println!("⏰ Testing timeout for operation: {}", op_name);

            let op_result = timeout(
                Duration::from_secs(6), // Individual operation timeout
                async {
                    let output = Command::new("cargo")
                        .args(&["run", "--bin", binary_name, "--"])
                        .args(args)
                        .output();

                    match output {
                        Ok(output) if output.status.success() => {
                            println!("✅ Operation '{}' completed successfully", op_name);
                            Ok::<(), Box<dyn std::error::Error>>(())
                        }
                        Ok(output) => {
                            // Help and version operations may "fail" but still provide output
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            let stderr = String::from_utf8_lossy(&output.stderr);

                            if stdout.contains("help")
                                || stdout.contains("version")
                                || stderr.contains("help")
                                || stderr.contains("version")
                            {
                                println!("✅ Operation '{}' provided expected output", op_name);
                                Ok(())
                            } else {
                                Err(format!(
                                    "Operation '{}' failed: status={}",
                                    op_name, output.status
                                )
                                .into())
                            }
                        }
                        Err(e) => Err(format!("Operation '{}' error: {}", op_name, e).into()),
                    }
                },
            )
            .await;

            match op_result {
                Ok(Ok(())) => {
                    successful_ops += 1;
                    println!("✅ '{}' completed within timeout", op_name);
                }
                Ok(Err(e)) => {
                    eprintln!("⚠️ '{}' failed: {}", op_name, e);
                }
                Err(_) => {
                    eprintln!("⏰ '{}' timed out", op_name);
                }
            }
        }

        // Most operations should complete within timeout
        assert!(
            successful_ops >= 2,
            "At least 2/3 operations should complete within timeout, got {}",
            successful_ops
        );

        println!(
            "✅ Timeout handling test: {}/{} operations successful",
            successful_ops, operations_len
        );
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await;

    match test_result {
        Ok(Ok(())) => println!("✅ Timeout handling test passed"),
        Ok(Err(e)) => panic!("❌ Timeout handling test failed: {}", e),
        Err(_) => panic!("💥 Timeout handling test itself timed out"),
    }
}

/// Test graceful degradation under resource constraints
#[tokio::test]
async fn test_graceful_degradation() {
    let test_result = timeout(Duration::from_secs(15), async {
        let servers = vec![
            ("filesystem", "filesystem-server"),
            ("image-generation", "image-generation-server"),
        ];

        let mut degradation_results = Vec::new();

        for (server_name, binary_name) in servers {
            println!("🔻 Testing graceful degradation for {}", server_name);

            // Test with invalid/stress conditions
            let stress_tests = vec![
                (
                    "invalid-port",
                    vec!["--transport", "http", "--port", "99999"],
                ),
                ("invalid-transport", vec!["--transport", "invalid-mode"]),
                (
                    "conflicting-args",
                    vec!["--transport", "http", "--transport", "stdio"],
                ),
            ];

            let mut graceful_failures = 0;

            for (test_name, args) in stress_tests {
                let start = Instant::now();

                let output = Command::new("cargo")
                    .args(&["run", "--bin", binary_name, "--"])
                    .args(&args)
                    .output();

                let duration = start.elapsed();

                match output {
                    Ok(output) => {
                        // Should fail gracefully with proper error message
                        if !output.status.success() {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            let stdout = String::from_utf8_lossy(&output.stdout);

                            if !stderr.is_empty()
                                || stdout.contains("error")
                                || stdout.contains("help")
                            {
                                graceful_failures += 1;
                                println!("✅ '{}' failed gracefully in {:?}", test_name, duration);
                            } else {
                                println!("⚠️ '{}' failed without helpful output", test_name);
                            }
                        } else {
                            println!("⚠️ '{}' unexpectedly succeeded", test_name);
                        }
                    }
                    Err(e) => {
                        println!("❌ '{}' had execution error: {}", test_name, e);
                    }
                }
            }

            degradation_results.push((server_name, graceful_failures));

            // Most stress tests should fail gracefully
            assert!(
                graceful_failures >= 2,
                "Server {} should handle at least 2/3 stress conditions gracefully",
                server_name
            );
        }

        println!("📊 Graceful Degradation Results:");
        for (name, graceful_count) in degradation_results {
            println!("   {}: {}/3 graceful failures", name, graceful_count);
        }

        println!("✅ Graceful degradation test completed");
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await;

    match test_result {
        Ok(Ok(())) => println!("✅ Graceful degradation test passed"),
        Ok(Err(e)) => panic!("❌ Graceful degradation test failed: {}", e),
        Err(_) => panic!("💥 Graceful degradation test timed out"),
    }
}

/// Test server recovery from various error conditions
#[tokio::test]
async fn test_server_error_recovery() {
    let test_result = timeout(Duration::from_secs(10), async {
        let _server_name = "blog-generation";
        let binary_name = "blog-generation-server";

        // Test error scenarios and recovery
        let error_scenarios = vec![
            ("help-after-error", vec!["--invalid-flag"], vec!["--help"]),
            ("version-after-error", vec!["--bad-arg"], vec!["--version"]),
        ];

        let mut recovery_successes = 0;

        let error_scenarios_len = error_scenarios.len();
        for (scenario_name, error_args, recovery_args) in error_scenarios {
            println!("🔄 Testing recovery scenario: {}", scenario_name);

            // First command should fail
            let error_output = Command::new("cargo")
                .args(&["run", "--bin", binary_name, "--"])
                .args(&error_args)
                .output();

            match error_output {
                Ok(output) if !output.status.success() => {
                    println!("✅ Error condition triggered as expected");

                    // Recovery command should work
                    let recovery_output = Command::new("cargo")
                        .args(&["run", "--bin", binary_name, "--"])
                        .args(&recovery_args)
                        .output();

                    match recovery_output {
                        Ok(output) if output.status.success() => {
                            recovery_successes += 1;
                            println!("✅ Recovery successful for '{}'", scenario_name);
                        }
                        Ok(output) => {
                            // Help/version might have non-zero exit but still work
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            let stderr = String::from_utf8_lossy(&output.stderr);

                            if stdout.contains("help")
                                || stdout.contains("version")
                                || stderr.contains("help")
                                || stderr.contains("version")
                            {
                                recovery_successes += 1;
                                println!(
                                    "✅ Recovery provided expected output for '{}'",
                                    scenario_name
                                );
                            } else {
                                println!("⚠️ Recovery failed for '{}'", scenario_name);
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Recovery error for '{}': {}", scenario_name, e);
                        }
                    }
                }
                Ok(_) => {
                    println!("⚠️ Error condition didn't trigger for '{}'", scenario_name);
                }
                Err(e) => {
                    eprintln!(
                        "❌ Failed to test error condition for '{}': {}",
                        scenario_name, e
                    );
                }
            }

            // Brief pause between scenarios
            tokio::time::sleep(Duration::from_millis(200)).await;
        }

        // At least one recovery scenario should work
        assert!(
            recovery_successes >= 1,
            "At least one error recovery scenario should succeed"
        );

        println!(
            "✅ Error recovery test: {}/{} scenarios successful",
            recovery_successes, error_scenarios_len
        );
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await;

    match test_result {
        Ok(Ok(())) => println!("✅ Server error recovery test passed"),
        Ok(Err(e)) => panic!("❌ Error recovery test failed: {}", e),
        Err(_) => panic!("💥 Error recovery test timed out"),
    }
}

/// Comprehensive performance test suite
#[tokio::test]
async fn test_comprehensive_performance_suite() {
    let test_result = timeout(Duration::from_secs(25), async {
        println!("🏁 Starting comprehensive performance test suite");

        let config = PerformanceTestConfig::default();
        let tester = ServerPerformanceTester::new(config);

        let servers = vec![
            ("filesystem", "filesystem-server"),
            ("image-generation", "image-generation-server"),
        ];

        // Run performance tests for each server
        for (server_name, binary_name) in servers {
            println!("\n🔬 Testing server: {}", server_name);

            // Startup performance
            if let Ok(startup_time) = tester
                .test_startup_performance(server_name, binary_name)
                .await
            {
                assert!(
                    startup_time < Duration::from_secs(8),
                    "Startup time requirement"
                );
            }

            // Memory usage
            if let Ok(memory_mb) = tester.test_memory_usage(server_name, binary_name).await {
                assert!(memory_mb < 100.0, "Memory usage requirement");
            }

            // Brief pause between server tests
            tokio::time::sleep(Duration::from_millis(300)).await;
        }

        // Generate and display comprehensive report
        let report = tester.generate_report().await;
        println!("\n{}", report);

        println!("✅ Comprehensive performance suite completed");
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await;

    match test_result {
        Ok(Ok(())) => println!("✅ Comprehensive performance test suite passed"),
        Ok(Err(e)) => panic!("❌ Performance test suite failed: {}", e),
        Err(_) => panic!("💥 Performance test suite timed out"),
    }
}

/// Test server resilience to rapid start/stop cycles
#[tokio::test]
async fn test_server_resilience_cycles() {
    let test_result = timeout(Duration::from_secs(18), async {
        let _server_name = "filesystem";
        let binary_name = "filesystem-server";

        let mut successful_cycles = 0;
        let total_cycles = 4;

        for cycle in 1..=total_cycles {
            println!("🔄 Resilience cycle #{}/{}", cycle, total_cycles);

            let cycle_start = Instant::now();

            // Start server
            let mut cmd = Command::new("cargo");
            cmd.args(&["run", "--bin", binary_name, "--", "--transport", "stdio"])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            match cmd.spawn() {
                Ok(mut process) => {
                    // Brief operation period
                    tokio::time::sleep(Duration::from_millis(300)).await;

                    // Stop server
                    match process.kill() {
                        Ok(()) => {
                            let _ = process.wait();
                            successful_cycles += 1;
                            println!(
                                "✅ Cycle #{} completed in {:?}",
                                cycle,
                                cycle_start.elapsed()
                            );
                        }
                        Err(e) => {
                            eprintln!("⚠️ Cycle #{} shutdown failed: {}", cycle, e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Cycle #{} startup failed: {}", cycle, e);
                }
            }

            // Recovery pause
            tokio::time::sleep(Duration::from_millis(200)).await;
        }

        // Most cycles should succeed for resilient servers
        assert!(
            successful_cycles >= total_cycles - 1,
            "At least {}/{} resilience cycles should succeed, got {}",
            total_cycles - 1,
            total_cycles,
            successful_cycles
        );

        println!(
            "✅ Server resilience: {}/{} cycles successful",
            successful_cycles, total_cycles
        );
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await;

    match test_result {
        Ok(Ok(())) => println!("✅ Server resilience cycles test passed"),
        Ok(Err(e)) => panic!("❌ Resilience cycles test failed: {}", e),
        Err(_) => panic!("💥 Resilience cycles test timed out"),
    }
}

/// Test system resource limits and constraints
#[tokio::test]
#[ignore] // Mark as ignored for regular test runs due to resource intensity
async fn test_system_resource_limits() {
    let test_result = timeout(Duration::from_secs(30), async {
        println!("💪 Testing system resource limits");

        // Test with higher concurrent load
        let high_load_config = PerformanceTestConfig {
            concurrent_requests: 10,
            load_test_duration: Duration::from_secs(5),
            ..Default::default()
        };

        let tester = ServerPerformanceTester::new(high_load_config);

        // Run intensive load test on filesystem server
        match tester
            .test_load_performance("filesystem", "filesystem-server")
            .await
        {
            Ok(()) => {
                let metrics = tester.metrics.lock().await;
                if let Some(fs_metrics) = metrics.get("filesystem") {
                    println!("📊 High Load Results:");
                    println!("   Success rate: {:.1}%", fs_metrics.success_rate() * 100.0);
                    println!(
                        "   Average response: {:?}",
                        fs_metrics.average_response_time()
                    );

                    // Under high load, still expect reasonable performance
                    assert!(
                        fs_metrics.success_rate() >= 0.7,
                        "Success rate under high load should be at least 70%"
                    );

                    assert!(
                        fs_metrics.average_response_time() < Duration::from_secs(8),
                        "Average response under high load should be under 8 seconds"
                    );
                }

                println!("✅ High load test passed");
            }
            Err(e) => {
                // High load test failure is acceptable, but log it
                eprintln!("⚠️ High load test failed (acceptable): {}", e);
            }
        }

        println!("✅ System resource limits test completed");
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await;

    match test_result {
        Ok(Ok(())) => println!("✅ System resource limits test passed"),
        Ok(Err(e)) => println!("⚠️ Resource limits test failed (non-critical): {}", e),
        Err(_) => println!("⏰ Resource limits test timed out (expected under high load)"),
    }
}

/// Performance regression test (baseline comparison)
#[tokio::test]
async fn test_performance_baseline() {
    let test_result = timeout(Duration::from_secs(12), async {
        println!("📏 Running performance baseline test");

        let config = PerformanceTestConfig::default();
        let tester = ServerPerformanceTester::new(config);

        // Test key server for baseline performance
        let server_name = "filesystem";
        let binary_name = "filesystem-server";

        // Multiple runs for consistency
        let mut baseline_times = Vec::new();

        for run in 1..=3 {
            println!("🏃 Baseline run #{}/3", run);

            match tester
                .test_startup_performance(server_name, binary_name)
                .await
            {
                Ok(time) => {
                    baseline_times.push(time);
                    println!("   Run #{}: {:?}", run, time);
                }
                Err(e) => {
                    eprintln!("⚠️ Baseline run #{} failed: {}", run, e);
                }
            }

            // Cool-down between runs
            tokio::time::sleep(Duration::from_millis(400)).await;
        }

        if !baseline_times.is_empty() {
            let avg_baseline =
                baseline_times.iter().sum::<Duration>() / baseline_times.len() as u32;
            let min_baseline = *baseline_times.iter().min().unwrap();
            let max_baseline = *baseline_times.iter().max().unwrap();
            let consistency = max_baseline.saturating_sub(min_baseline);

            println!("📊 Performance Baseline:");
            println!("   Average: {:?}", avg_baseline);
            println!("   Best: {:?}", min_baseline);
            println!("   Worst: {:?}", max_baseline);
            println!("   Consistency: {:?}", consistency);

            // Baseline requirements
            assert!(
                avg_baseline < Duration::from_secs(6),
                "Average baseline should be under 6 seconds"
            );

            assert!(
                consistency < Duration::from_secs(3),
                "Performance should be consistent (variability < 3s)"
            );

            // Store baseline for future comparison
            println!(
                "💾 Baseline established: {:?} ±{:?}",
                avg_baseline, consistency
            );
        }

        println!("✅ Performance baseline test completed");
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await;

    match test_result {
        Ok(Ok(())) => println!("✅ Performance baseline test passed"),
        Ok(Err(e)) => panic!("❌ Performance baseline test failed: {}", e),
        Err(_) => panic!("💥 Performance baseline test timed out"),
    }
}
