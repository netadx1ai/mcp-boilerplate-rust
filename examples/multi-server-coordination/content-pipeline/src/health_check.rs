//! Health Check Utility for Content Pipeline Servers
//! 
//! This utility performs comprehensive health checks on all servers in the content
//! generation pipeline, providing detailed status information and diagnostics.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{error, info, warn};

/// Server health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unreachable,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "✅ Healthy"),
            HealthStatus::Degraded => write!(f, "⚠️ Degraded"),
            HealthStatus::Unhealthy => write!(f, "❌ Unhealthy"),
            HealthStatus::Unreachable => write!(f, "🔌 Unreachable"),
        }
    }
}

/// Detailed health information for a server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHealth {
    pub name: String,
    pub url: String,
    pub status: HealthStatus,
    pub response_time: Duration,
    pub last_checked: DateTime<Utc>,
    pub version: Option<String>,
    pub uptime: Option<Duration>,
    pub error_message: Option<String>,
    pub endpoints_checked: Vec<EndpointHealth>,
}

/// Health information for individual endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointHealth {
    pub path: String,
    pub status: HealthStatus,
    pub response_time: Duration,
    pub http_status: Option<u16>,
    pub error_message: Option<String>,
}

/// Comprehensive health check report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub overall_status: HealthStatus,
    pub checked_at: DateTime<Utc>,
    pub total_servers: usize,
    pub healthy_servers: usize,
    pub degraded_servers: usize,
    pub unhealthy_servers: usize,
    pub unreachable_servers: usize,
    pub servers: Vec<ServerHealth>,
    pub recommendations: Vec<String>,
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    pub timeout: Duration,
    pub retry_attempts: usize,
    pub retry_delay: Duration,
    pub detailed_checks: bool,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            retry_attempts: 3,
            retry_delay: Duration::from_millis(500),
            detailed_checks: true,
        }
    }
}

/// Server configuration for health checks
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub name: String,
    pub base_url: String,
    pub health_endpoint: String,
    pub critical_endpoints: Vec<String>,
}

/// Health checker for pipeline servers
pub struct PipelineHealthChecker {
    client: Client,
    config: HealthCheckConfig,
    servers: Vec<ServerConfig>,
}

impl PipelineHealthChecker {
    pub fn new(config: HealthCheckConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");

        let servers = vec![
            ServerConfig {
                name: "News Data Server".to_string(),
                base_url: "http://localhost:3001".to_string(),
                health_endpoint: "/health".to_string(),
                critical_endpoints: vec![
                    "/search_news".to_string(),
                    "/get_trending_news".to_string(),
                    "/get_categories".to_string(),
                ],
            },
            ServerConfig {
                name: "Template Server".to_string(),
                base_url: "http://localhost:3002".to_string(),
                health_endpoint: "/health".to_string(),
                critical_endpoints: vec![
                    "/list_templates".to_string(),
                    "/render_template".to_string(),
                    "/get_template".to_string(),
                ],
            },
            ServerConfig {
                name: "Database Server".to_string(),
                base_url: "http://localhost:3003".to_string(),
                health_endpoint: "/health".to_string(),
                critical_endpoints: vec![
                    "/execute_query".to_string(),
                    "/list_tables".to_string(),
                    "/get_table_schema".to_string(),
                ],
            },
            ServerConfig {
                name: "Analytics Server".to_string(),
                base_url: "http://localhost:3004".to_string(),
                health_endpoint: "/health".to_string(),
                critical_endpoints: vec![
                    "/get_content_metrics".to_string(),
                    "/get_engagement_trends".to_string(),
                    "/generate_analytics_report".to_string(),
                ],
            },
        ];

        Self {
            client,
            config,
            servers,
        }
    }

    /// Perform comprehensive health check on all servers
    pub async fn check_all_servers(&self) -> Result<HealthReport> {
        info!("Starting comprehensive health check for {} servers", self.servers.len());
        let start_time = Instant::now();

        let mut server_health_futures = Vec::new();

        for server in &self.servers {
            let server_clone = server.clone();
            let client = self.client.clone();
            let config = self.config.clone();

            let future = async move {
                Self::check_single_server(&client, &server_clone, &config).await
            };

            server_health_futures.push(future);
        }

        // Execute all health checks in parallel
        let server_health_results = futures::future::join_all(server_health_futures).await;

        let mut servers = Vec::new();
        let mut healthy_count = 0;
        let mut degraded_count = 0;
        let mut unhealthy_count = 0;
        let mut unreachable_count = 0;

        for result in server_health_results {
            match result {
                Ok(health) => {
                    match health.status {
                        HealthStatus::Healthy => healthy_count += 1,
                        HealthStatus::Degraded => degraded_count += 1,
                        HealthStatus::Unhealthy => unhealthy_count += 1,
                        HealthStatus::Unreachable => unreachable_count += 1,
                    }
                    servers.push(health);
                }
                Err(e) => {
                    error!("Failed to check server health: {}", e);
                    unreachable_count += 1;
                }
            }
        }

        let total_servers = servers.len();
        let overall_status = Self::determine_overall_status(
            healthy_count,
            degraded_count,
            unhealthy_count,
            unreachable_count,
        );

        let recommendations = Self::generate_recommendations(&servers);

        let report = HealthReport {
            overall_status,
            checked_at: Utc::now(),
            total_servers,
            healthy_servers: healthy_count,
            degraded_servers: degraded_count,
            unhealthy_servers: unhealthy_count,
            unreachable_servers: unreachable_count,
            servers,
            recommendations,
        };

        info!(
            "Health check completed in {:?}. Overall status: {}",
            start_time.elapsed(),
            overall_status
        );

        Ok(report)
    }

    /// Check health of a single server
    async fn check_single_server(
        client: &Client,
        server: &ServerConfig,
        config: &HealthCheckConfig,
    ) -> Result<ServerHealth> {
        let start_time = Instant::now();
        info!("Checking health of server: {}", server.name);

        // Primary health endpoint check
        let health_url = format!("{}{}", server.base_url, server.health_endpoint);
        let (primary_status, primary_response_time, error_message, version, uptime) =
            Self::check_endpoint_with_retries(client, &health_url, config).await;

        // Detailed endpoint checks if enabled
        let mut endpoints_checked = Vec::new();
        if config.detailed_checks {
            for endpoint in &server.critical_endpoints {
                let endpoint_url = format!("{}{}", server.base_url, endpoint);
                let (status, response_time, _, http_status, endpoint_error) =
                    Self::check_single_endpoint(client, &endpoint_url, config.timeout).await;

                endpoints_checked.push(EndpointHealth {
                    path: endpoint.clone(),
                    status,
                    response_time,
                    http_status,
                    error_message: endpoint_error,
                });
            }
        }

        // Determine overall server status
        let server_status = if primary_status == HealthStatus::Healthy {
            if endpoints_checked.iter().all(|e| e.status == HealthStatus::Healthy) {
                HealthStatus::Healthy
            } else if endpoints_checked.iter().any(|e| e.status == HealthStatus::Unhealthy) {
                HealthStatus::Degraded
            } else {
                HealthStatus::Healthy
            }
        } else {
            primary_status
        };

        let server_health = ServerHealth {
            name: server.name.clone(),
            url: server.base_url.clone(),
            status: server_status,
            response_time: primary_response_time,
            last_checked: Utc::now(),
            version,
            uptime,
            error_message,
            endpoints_checked,
        };

        info!(
            "Server {} health check completed in {:?}: {}",
            server.name,
            start_time.elapsed(),
            server_health.status
        );

        Ok(server_health)
    }

    /// Check endpoint with retry logic
    async fn check_endpoint_with_retries(
        client: &Client,
        url: &str,
        config: &HealthCheckConfig,
    ) -> (HealthStatus, Duration, Option<String>, Option<String>, Option<Duration>) {
        let mut last_error = None;

        for attempt in 1..=config.retry_attempts {
            let (status, response_time, error, http_status, extra_data) =
                Self::check_single_endpoint(client, url, config.timeout).await;

            match status {
                HealthStatus::Healthy => {
                    // Extract version and uptime from extra_data if available
                    let (version, uptime) = if let Some(data) = extra_data {
                        Self::parse_health_response(&data)
                    } else {
                        (None, None)
                    };
                    return (status, response_time, None, version, uptime);
                }
                HealthStatus::Unreachable if attempt < config.retry_attempts => {
                    last_error = error;
                    tokio::time::sleep(config.retry_delay).await;
                    continue;
                }
                _ => {
                    return (status, response_time, error, None, None);
                }
            }
        }

        (HealthStatus::Unreachable, Duration::from_secs(0), last_error, None, None)
    }

    /// Check a single endpoint
    async fn check_single_endpoint(
        client: &Client,
        url: &str,
        timeout_duration: Duration,
    ) -> (HealthStatus, Duration, Option<String>, Option<u16>, Option<String>) {
        let start_time = Instant::now();

        let result = timeout(timeout_duration, client.get(url).send()).await;

        let response_time = start_time.elapsed();

        match result {
            Ok(Ok(response)) => {
                let status_code = response.status().as_u16();
                
                if response.status().is_success() {
                    // Try to get response body for additional health info
                    let body = response.text().await.ok();
                    (HealthStatus::Healthy, response_time, None, Some(status_code), body)
                } else {
                    let error_msg = format!("HTTP {}", status_code);
                    (HealthStatus::Unhealthy, response_time, Some(error_msg), Some(status_code), None)
                }
            }
            Ok(Err(e)) => {
                let error_msg = format!("Request failed: {}", e);
                (HealthStatus::Unreachable, response_time, Some(error_msg), None, None)
            }
            Err(_) => {
                let error_msg = format!("Request timeout after {:?}", timeout_duration);
                (HealthStatus::Unreachable, response_time, Some(error_msg), None, None)
            }
        }
    }

    /// Parse health response for additional information
    fn parse_health_response(response_body: &str) -> (Option<String>, Option<Duration>) {
        // Try to parse JSON response for version and uptime
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(response_body) {
            let version = json.get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let uptime = json.get("uptime_seconds")
                .and_then(|v| v.as_u64())
                .map(Duration::from_secs);

            (version, uptime)
        } else {
            (None, None)
        }
    }

    /// Determine overall system health status
    fn determine_overall_status(
        healthy: usize,
        degraded: usize,
        unhealthy: usize,
        unreachable: usize,
    ) -> HealthStatus {
        let total = healthy + degraded + unhealthy + unreachable;
        
        if total == 0 {
            return HealthStatus::Unreachable;
        }

        let healthy_percentage = (healthy as f64 / total as f64) * 100.0;

        match (unhealthy, unreachable, healthy_percentage) {
            (0, 0, p) if p >= 100.0 => HealthStatus::Healthy,
            (0, 0, p) if p >= 80.0 => HealthStatus::Degraded,
            (u, r, _) if u + r == total => HealthStatus::Unreachable,
            _ => HealthStatus::Unhealthy,
        }
    }

    /// Generate actionable recommendations based on health status
    fn generate_recommendations(servers: &[ServerHealth]) -> Vec<String> {
        let mut recommendations = Vec::new();

        let unreachable_servers: Vec<&str> = servers
            .iter()
            .filter(|s| s.status == HealthStatus::Unreachable)
            .map(|s| s.name.as_str())
            .collect();

        let unhealthy_servers: Vec<&str> = servers
            .iter()
            .filter(|s| s.status == HealthStatus::Unhealthy)
            .map(|s| s.name.as_str())
            .collect();

        let degraded_servers: Vec<&str> = servers
            .iter()
            .filter(|s| s.status == HealthStatus::Degraded)
            .map(|s| s.name.as_str())
            .collect();

        let slow_servers: Vec<&str> = servers
            .iter()
            .filter(|s| s.response_time > Duration::from_secs(5))
            .map(|s| s.name.as_str())
            .collect();

        if !unreachable_servers.is_empty() {
            recommendations.push(format!(
                "🔌 Check network connectivity and restart these servers: {}",
                unreachable_servers.join(", ")
            ));
        }

        if !unhealthy_servers.is_empty() {
            recommendations.push(format!(
                "❌ Investigate and fix critical issues in: {}",
                unhealthy_servers.join(", ")
            ));
        }

        if !degraded_servers.is_empty() {
            recommendations.push(format!(
                "⚠️ Monitor and consider maintenance for: {}",
                degraded_servers.join(", ")
            ));
        }

        if !slow_servers.is_empty() {
            recommendations.push(format!(
                "🐌 Performance optimization needed for: {}",
                slow_servers.join(", ")
            ));
        }

        if recommendations.is_empty() {
            recommendations.push("✅ All systems are operating normally. Continue monitoring.".to_string());
        }

        recommendations
    }

    /// Print formatted health report
    pub fn print_health_report(&self, report: &HealthReport) {
        println!("\n🏥 PIPELINE HEALTH REPORT");
        println!("═══════════════════════════════════════════════════");
        println!("📊 Overall Status: {}", report.overall_status);
        println!("🕐 Checked At: {}", report.checked_at.format("%Y-%m-%d %H:%M:%S UTC"));
        println!();

        println!("📈 SUMMARY");
        println!("─────────────────────────────────────────────────");
        println!("Total Servers: {}", report.total_servers);
        println!("✅ Healthy: {}", report.healthy_servers);
        println!("⚠️ Degraded: {}", report.degraded_servers);
        println!("❌ Unhealthy: {}", report.unhealthy_servers);
        println!("🔌 Unreachable: {}", report.unreachable_servers);
        println!();

        println!("🖥️ SERVER DETAILS");
        println!("─────────────────────────────────────────────────");
        for server in &report.servers {
            println!("• {} ({})", server.name, server.url);
            println!("  Status: {}", server.status);
            println!("  Response Time: {:?}", server.response_time);
            
            if let Some(version) = &server.version {
                println!("  Version: {}", version);
            }
            
            if let Some(uptime) = &server.uptime {
                println!("  Uptime: {:?}", uptime);
            }
            
            if let Some(error) = &server.error_message {
                println!("  Error: {}", error);
            }

            if !server.endpoints_checked.is_empty() {
                println!("  Endpoints:");
                for endpoint in &server.endpoints_checked {
                    println!("    {} - {} ({:?})", endpoint.path, endpoint.status, endpoint.response_time);
                    if let Some(error) = &endpoint.error_message {
                        println!("      Error: {}", error);
                    }
                }
            }
            println!();
        }

        if !report.recommendations.is_empty() {
            println!("💡 RECOMMENDATIONS");
            println!("─────────────────────────────────────────────────");
            for (i, recommendation) in report.recommendations.iter().enumerate() {
                println!("{}. {}", i + 1, recommendation);
            }
            println!();
        }

        println!("═══════════════════════════════════════════════════");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("health_check=info")
        .init();

    info!("Starting Pipeline Health Check");

    let config = HealthCheckConfig::default();
    let health_checker = PipelineHealthChecker::new(config);

    match health_checker.check_all_servers().await {
        Ok(report) => {
            health_checker.print_health_report(&report);

            // Exit with appropriate code based on overall health
            match report.overall_status {
                HealthStatus::Healthy => std::process::exit(0),
                HealthStatus::Degraded => {
                    warn!("System is degraded but operational");
                    std::process::exit(1);
                }
                HealthStatus::Unhealthy => {
                    error!("System is unhealthy");
                    std::process::exit(2);
                }
                HealthStatus::Unreachable => {
                    error!("System is unreachable");
                    std::process::exit(3);
                }
            }
        }
        Err(e) => {
            error!("Health check failed: {}", e);
            std::process::exit(4);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_display() {
        assert_eq!(format!("{}", HealthStatus::Healthy), "✅ Healthy");
        assert_eq!(format!("{}", HealthStatus::Degraded), "⚠️ Degraded");
        assert_eq!(format!("{}", HealthStatus::Unhealthy), "❌ Unhealthy");
        assert_eq!(format!("{}", HealthStatus::Unreachable), "🔌 Unreachable");
    }

    #[test]
    fn test_determine_overall_status() {
        assert_eq!(
            PipelineHealthChecker::determine_overall_status(4, 0, 0, 0),
            HealthStatus::Healthy
        );
        assert_eq!(
            PipelineHealthChecker::determine_overall_status(3, 1, 0, 0),
            HealthStatus::Degraded
        );
        assert_eq!(
            PipelineHealthChecker::determine_overall_status(2, 1, 1, 0),
            HealthStatus::Unhealthy
        );
        assert_eq!(
            PipelineHealthChecker::determine_overall_status(0, 0, 0, 4),
            HealthStatus::Unreachable
        );
    }

    #[test]
    fn test_parse_health_response() {
        let json_response = r#"{"version": "1.0.0", "uptime_seconds": 3600}"#;
        let (version, uptime) = PipelineHealthChecker::parse_health_response(json_response);
        
        assert_eq!(version, Some("1.0.0".to_string()));
        assert_eq!(uptime, Some(Duration::from_secs(3600)));
    }

    #[tokio::test]
    async fn test_health_checker_creation() {
        let config = HealthCheckConfig::default();
        let checker = PipelineHealthChecker::new(config);
        assert_eq!(checker.servers.len(), 4);
    }
}