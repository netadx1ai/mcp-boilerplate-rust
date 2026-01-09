//! Load balancing module for distributing requests across multiple MCP server backends
//!
//! Note: Many types are re-exported for public API but may not be used internally.
#![allow(unused_imports)]
//!
//! This module provides load balancing capabilities with multiple strategies:
//! - Round-robin: Even distribution across backends
//! - Least connections: Send to backend with fewest active connections
//! - Random: Random backend selection
//! - Weighted round-robin: Distribution based on backend weights
//! - IP hash: Consistent backend selection based on client IP
//!
//! ## Features
//!
//! - Multiple load balancing strategies
//! - Health checking with automatic failover
//! - Connection pooling and limits
//! - Sticky sessions support
//! - Real-time statistics and monitoring
//! - Dynamic backend addition/removal
//!
//! ## Example
//!
//! ```rust,no_run
//! use mcp_boilerplate_rust::loadbalancer::{LoadBalancer, LoadBalancerConfig, Backend, Strategy};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create configuration
//!     let config = LoadBalancerConfig::new(Strategy::RoundRobin)
//!         .add_backend(Backend::new(
//!             "backend1".to_string(),
//!             "127.0.0.1:8081".to_string()
//!         ))
//!         .add_backend(Backend::new(
//!             "backend2".to_string(),
//!             "127.0.0.1:8082".to_string()
//!         ))
//!         .with_failover(true)
//!         .with_max_retries(3);
//!
//!     // Create load balancer
//!     let lb = LoadBalancer::new(config);
//!
//!     // Start health checks
//!     lb.start_health_checks().await;
//!
//!     // Select a backend
//!     match lb.select_backend(None).await {
//!         Ok(backend) => {
//!             let state = backend.read().await;
//!             println!("Selected backend: {}", state.backend.address);
//!         }
//!         Err(e) => eprintln!("No backends available: {}", e),
//!     }
//!
//!     // Get statistics
//!     let stats = lb.get_stats().await;
//!     println!("Total requests: {}", stats.total_requests);
//!     println!("Success rate: {:.2}%", stats.success_rate() * 100.0);
//! }
//! ```

pub mod types;
pub mod balancer;

// Re-export types for public API
#[allow(unused_imports)]
pub use types::{
    Backend,
    BackendState,
    HealthCheckConfig,
    HealthStatus,
    LoadBalancerConfig,
    LoadBalancerStats,
    SharedBackendState,
    Strategy,
};

#[allow(unused_imports)]
pub use balancer::LoadBalancer;