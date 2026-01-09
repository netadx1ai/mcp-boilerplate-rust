//! Load balancer types for public API
//!
//! Note: Many types are defined for public API extensibility but may not be used internally.
#![allow(dead_code)]

use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Load balancing strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    /// Round-robin: Distribute requests evenly across all backends
    RoundRobin,
    /// Least connections: Send to backend with fewest active connections
    LeastConnections,
    /// Random: Randomly select a backend
    Random,
    /// Weighted round-robin: Distribute based on backend weights
    WeightedRoundRobin,
    /// IP hash: Consistent backend selection based on client IP
    IpHash,
}

impl fmt::Display for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Strategy::RoundRobin => write!(f, "round-robin"),
            Strategy::LeastConnections => write!(f, "least-connections"),
            Strategy::Random => write!(f, "random"),
            Strategy::WeightedRoundRobin => write!(f, "weighted-round-robin"),
            Strategy::IpHash => write!(f, "ip-hash"),
        }
    }
}

/// Backend server configuration
#[derive(Debug, Clone)]
pub struct Backend {
    /// Unique identifier for the backend
    pub id: String,
    /// Backend address (host:port)
    pub address: String,
    /// Weight for weighted strategies (default: 1)
    pub weight: u32,
    /// Maximum concurrent connections (0 = unlimited)
    pub max_connections: u32,
    /// Backend-specific timeout in seconds
    pub timeout_secs: u64,
    /// Whether this backend is enabled
    pub enabled: bool,
}

impl Backend {
    pub fn new(id: String, address: String) -> Self {
        Self {
            id,
            address,
            weight: 1,
            max_connections: 0,
            timeout_secs: 30,
            enabled: true,
        }
    }

    pub fn with_weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }

    pub fn with_max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }
}

/// Backend health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Backend is healthy and accepting connections
    Healthy,
    /// Backend is unhealthy and should not receive traffic
    Unhealthy,
    /// Backend health is unknown or being checked
    Unknown,
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
            HealthStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Backend runtime state
#[derive(Debug, Clone)]
pub struct BackendState {
    /// Backend configuration
    pub backend: Backend,
    /// Current health status
    pub health: HealthStatus,
    /// Number of active connections
    pub active_connections: u32,
    /// Total number of requests handled
    pub total_requests: u64,
    /// Total number of failed requests
    pub failed_requests: u64,
    /// Last health check time
    pub last_health_check: Option<Instant>,
    /// Last successful request time
    pub last_success: Option<Instant>,
    /// Last failure time
    pub last_failure: Option<Instant>,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
}

impl BackendState {
    pub fn new(backend: Backend) -> Self {
        Self {
            backend,
            health: HealthStatus::Unknown,
            active_connections: 0,
            total_requests: 0,
            failed_requests: 0,
            last_health_check: None,
            last_success: None,
            last_failure: None,
            avg_response_time_ms: 0.0,
        }
    }

    /// Check if backend can accept new connections
    pub fn can_accept_connection(&self) -> bool {
        if !self.backend.enabled {
            return false;
        }

        if self.health != HealthStatus::Healthy {
            return false;
        }

        if self.backend.max_connections > 0 
            && self.active_connections >= self.backend.max_connections {
            return false;
        }

        true
    }

    /// Record a successful request
    pub fn record_success(&mut self, response_time_ms: f64) {
        self.total_requests += 1;
        self.last_success = Some(Instant::now());
        
        // Update moving average
        if self.avg_response_time_ms == 0.0 {
            self.avg_response_time_ms = response_time_ms;
        } else {
            self.avg_response_time_ms = 
                (self.avg_response_time_ms * 0.9) + (response_time_ms * 0.1);
        }
    }

    /// Record a failed request
    pub fn record_failure(&mut self) {
        self.total_requests += 1;
        self.failed_requests += 1;
        self.last_failure = Some(Instant::now());
    }

    /// Get failure rate (0.0 to 1.0)
    pub fn failure_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        self.failed_requests as f64 / self.total_requests as f64
    }
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Interval between health checks
    pub interval: Duration,
    /// Timeout for health check requests
    pub timeout: Duration,
    /// Number of consecutive failures before marking unhealthy
    pub unhealthy_threshold: u32,
    /// Number of consecutive successes before marking healthy
    pub healthy_threshold: u32,
    /// Health check endpoint path
    pub path: String,
    /// Expected HTTP status code
    pub expected_status: u16,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(10),
            timeout: Duration::from_secs(5),
            unhealthy_threshold: 3,
            healthy_threshold: 2,
            path: "/health".to_string(),
            expected_status: 200,
        }
    }
}

/// Load balancer configuration
#[derive(Debug, Clone)]
pub struct LoadBalancerConfig {
    /// Load balancing strategy
    pub strategy: Strategy,
    /// Backends to load balance across
    pub backends: Vec<Backend>,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Enable sticky sessions
    pub sticky_sessions: bool,
    /// Session cookie name (for sticky sessions)
    pub session_cookie_name: String,
    /// Enable failover to healthy backends
    pub failover_enabled: bool,
    /// Maximum retries on failure
    pub max_retries: u32,
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            strategy: Strategy::RoundRobin,
            backends: Vec::new(),
            health_check: HealthCheckConfig::default(),
            sticky_sessions: false,
            session_cookie_name: "mcp_session".to_string(),
            failover_enabled: true,
            max_retries: 3,
        }
    }
}

impl LoadBalancerConfig {
    pub fn new(strategy: Strategy) -> Self {
        Self {
            strategy,
            ..Default::default()
        }
    }

    pub fn add_backend(mut self, backend: Backend) -> Self {
        self.backends.push(backend);
        self
    }

    pub fn with_health_check(mut self, config: HealthCheckConfig) -> Self {
        self.health_check = config;
        self
    }

    pub fn with_sticky_sessions(mut self, enabled: bool) -> Self {
        self.sticky_sessions = enabled;
        self
    }

    pub fn with_failover(mut self, enabled: bool) -> Self {
        self.failover_enabled = enabled;
        self
    }

    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }
}

/// Load balancer statistics
#[derive(Debug, Clone, Default)]
pub struct LoadBalancerStats {
    /// Total requests processed
    pub total_requests: u64,
    /// Total failed requests
    pub total_failures: u64,
    /// Total successful requests
    pub total_successes: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Number of currently active connections
    pub active_connections: u32,
    /// Uptime in seconds
    pub uptime_secs: u64,
}

impl LoadBalancerStats {
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        self.total_successes as f64 / self.total_requests as f64
    }

    pub fn failure_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        self.total_failures as f64 / self.total_requests as f64
    }
}

/// Shared backend state for thread-safe access
pub type SharedBackendState = Arc<RwLock<BackendState>>;