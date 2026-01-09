//! Load balancer implementation for distributing requests across multiple backends
//!
//! Note: Many methods are defined for public API extensibility but may not be used internally.
#![allow(dead_code)]

use super::types::{
    Backend, BackendState, HealthCheckConfig, HealthStatus, LoadBalancerConfig,
    LoadBalancerStats, SharedBackendState, Strategy,
};
use crate::types::McpError;
use rand::Rng;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::sleep;

type Result<T> = std::result::Result<T, McpError>;

/// Load balancer for distributing requests across multiple backends
pub struct LoadBalancer {
    config: LoadBalancerConfig,
    backends: Vec<SharedBackendState>,
    round_robin_index: AtomicUsize,
    total_requests: AtomicU64,
    total_failures: AtomicU64,
    total_successes: AtomicU64,
    sticky_sessions: Arc<RwLock<HashMap<String, String>>>,
    start_time: Instant,
}

impl LoadBalancer {
    /// Create a new load balancer with the given configuration
    pub fn new(config: LoadBalancerConfig) -> Self {
        let backends: Vec<SharedBackendState> = config
            .backends
            .iter()
            .map(|b| Arc::new(RwLock::new(BackendState::new(b.clone()))))
            .collect();

        Self {
            config,
            backends,
            round_robin_index: AtomicUsize::new(0),
            total_requests: AtomicU64::new(0),
            total_failures: AtomicU64::new(0),
            total_successes: AtomicU64::new(0),
            sticky_sessions: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    /// Start health checking for all backends
    pub async fn start_health_checks(&self) {
        for backend_state in &self.backends {
            let backend_state = Arc::clone(backend_state);
            let health_config = self.config.health_check.clone();

            tokio::spawn(async move {
                Self::health_check_loop(backend_state, health_config).await;
            });
        }
    }

    /// Select a backend based on the configured strategy
    pub async fn select_backend(&self, client_ip: Option<IpAddr>) -> Result<SharedBackendState> {
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        let selected = match self.config.strategy {
            Strategy::RoundRobin => self.select_round_robin().await,
            Strategy::LeastConnections => self.select_least_connections().await,
            Strategy::Random => self.select_random().await,
            Strategy::WeightedRoundRobin => self.select_weighted_round_robin().await,
            Strategy::IpHash => {
                if let Some(ip) = client_ip {
                    self.select_ip_hash(ip).await
                } else {
                    self.select_round_robin().await
                }
            }
        };

        match selected {
            Some(backend) => Ok(backend),
            None => {
                self.total_failures.fetch_add(1, Ordering::Relaxed);
                Err(McpError::InternalError("No healthy backends available".to_string()))
            }
        }
    }

    /// Select backend using round-robin strategy
    async fn select_round_robin(&self) -> Option<SharedBackendState> {
        let backends_count = self.backends.len();
        if backends_count == 0 {
            return None;
        }

        for _ in 0..backends_count {
            let index = self.round_robin_index.fetch_add(1, Ordering::Relaxed) % backends_count;
            let backend = &self.backends[index];
            let state = backend.read().await;

            if state.can_accept_connection() {
                drop(state);
                return Some(Arc::clone(backend));
            }
        }

        None
    }

    /// Select backend with least connections
    async fn select_least_connections(&self) -> Option<SharedBackendState> {
        let mut min_connections = u32::MAX;
        let mut selected: Option<SharedBackendState> = None;

        for backend in &self.backends {
            let state = backend.read().await;

            if state.can_accept_connection()
                && state.active_connections < min_connections {
                    min_connections = state.active_connections;
                    drop(state);
                    selected = Some(Arc::clone(backend));
                }
        }

        selected
    }

    /// Select backend randomly
    async fn select_random(&self) -> Option<SharedBackendState> {
        let healthy_backends: Vec<_> = {
            let mut healthy = Vec::new();
            for backend in &self.backends {
                let state = backend.read().await;
                if state.can_accept_connection() {
                    healthy.push(Arc::clone(backend));
                }
            }
            healthy
        };

        if healthy_backends.is_empty() {
            return None;
        }

        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..healthy_backends.len());
        Some(healthy_backends[index].clone())
    }

    /// Select backend using weighted round-robin
    async fn select_weighted_round_robin(&self) -> Option<SharedBackendState> {
        let mut total_weight = 0u64;
        let mut weighted_backends = Vec::new();

        for backend in &self.backends {
            let state = backend.read().await;
            if state.can_accept_connection() {
                total_weight += state.backend.weight as u64;
                weighted_backends.push((Arc::clone(backend), state.backend.weight));
            }
        }

        if weighted_backends.is_empty() {
            return None;
        }

        let mut rng = rand::thread_rng();
        let mut random_weight = rng.gen_range(0..total_weight);

        for (backend, weight) in weighted_backends {
            if random_weight < weight as u64 {
                return Some(backend);
            }
            random_weight -= weight as u64;
        }

        None
    }

    /// Select backend using IP hash (consistent hashing)
    async fn select_ip_hash(&self, client_ip: IpAddr) -> Option<SharedBackendState> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let healthy_backends: Vec<_> = {
            let mut healthy = Vec::new();
            for backend in &self.backends {
                let state = backend.read().await;
                if state.can_accept_connection() {
                    healthy.push(Arc::clone(backend));
                }
            }
            healthy
        };

        if healthy_backends.is_empty() {
            return None;
        }

        let mut hasher = DefaultHasher::new();
        client_ip.hash(&mut hasher);
        let hash = hasher.finish();

        let index = (hash as usize) % healthy_backends.len();
        Some(healthy_backends[index].clone())
    }

    /// Mark a connection as started on a backend
    pub async fn mark_connection_start(&self, backend: &SharedBackendState) {
        let mut state = backend.write().await;
        state.active_connections += 1;
    }

    /// Mark a connection as ended on a backend
    pub async fn mark_connection_end(
        &self,
        backend: &SharedBackendState,
        success: bool,
        response_time_ms: f64,
    ) {
        let mut state = backend.write().await;

        if state.active_connections > 0 {
            state.active_connections -= 1;
        }

        if success {
            state.record_success(response_time_ms);
            self.total_successes.fetch_add(1, Ordering::Relaxed);
        } else {
            state.record_failure();
            self.total_failures.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get current load balancer statistics
    pub async fn get_stats(&self) -> LoadBalancerStats {
        let mut active_connections = 0;
        let mut avg_response_times = Vec::new();

        for backend in &self.backends {
            let state = backend.read().await;
            active_connections += state.active_connections;
            if state.avg_response_time_ms > 0.0 {
                avg_response_times.push(state.avg_response_time_ms);
            }
        }

        let avg_response_time_ms = if avg_response_times.is_empty() {
            0.0
        } else {
            avg_response_times.iter().sum::<f64>() / avg_response_times.len() as f64
        };

        LoadBalancerStats {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            total_failures: self.total_failures.load(Ordering::Relaxed),
            total_successes: self.total_successes.load(Ordering::Relaxed),
            avg_response_time_ms,
            active_connections,
            uptime_secs: self.start_time.elapsed().as_secs(),
        }
    }

    /// Get detailed backend states
    pub async fn get_backend_states(&self) -> Vec<BackendState> {
        let mut states = Vec::new();
        for backend in &self.backends {
            let state = backend.read().await;
            states.push(state.clone());
        }
        states
    }

    /// Add a new backend dynamically
    pub async fn add_backend(&mut self, backend: Backend) {
        let backend_state = Arc::new(RwLock::new(BackendState::new(backend.clone())));
        self.backends.push(Arc::clone(&backend_state));

        let health_config = self.config.health_check.clone();
        tokio::spawn(async move {
            Self::health_check_loop(backend_state, health_config).await;
        });
    }

    /// Remove a backend by ID
    pub async fn remove_backend(&mut self, backend_id: &str) -> bool {
        let original_len = self.backends.len();
        self.backends.retain(|backend| {
            let state = backend.try_read();
            match state {
                Ok(s) => s.backend.id != backend_id,
                Err(_) => true,
            }
        });
        self.backends.len() < original_len
    }

    /// Enable or disable a backend
    pub async fn set_backend_enabled(&self, backend_id: &str, enabled: bool) -> bool {
        for backend in &self.backends {
            let mut state = backend.write().await;
            if state.backend.id == backend_id {
                state.backend.enabled = enabled;
                return true;
            }
        }
        false
    }

    /// Health check loop for a single backend
    async fn health_check_loop(
        backend_state: SharedBackendState,
        health_config: HealthCheckConfig,
    ) {
        let mut consecutive_failures = 0u32;
        let mut consecutive_successes = 0u32;

        loop {
            sleep(health_config.interval).await;

            let backend_address = {
                let state = backend_state.read().await;
                if !state.backend.enabled {
                    continue;
                }
                state.backend.address.clone()
            };

            let health_url = format!("http://{}{}", backend_address, health_config.path);

            let health_result = tokio::time::timeout(
                health_config.timeout,
                reqwest::get(&health_url),
            )
            .await;

            let is_healthy = match health_result {
                Ok(Ok(response)) => response.status().as_u16() == health_config.expected_status,
                Ok(Err(_)) | Err(_) => false,
            };

            let mut state = backend_state.write().await;
            state.last_health_check = Some(Instant::now());

            if is_healthy {
                consecutive_failures = 0;
                consecutive_successes += 1;

                if consecutive_successes >= health_config.healthy_threshold {
                    if state.health != HealthStatus::Healthy {
                        tracing::info!(
                            "Backend {} is now healthy",
                            state.backend.id
                        );
                    }
                    state.health = HealthStatus::Healthy;
                }
            } else {
                consecutive_successes = 0;
                consecutive_failures += 1;

                if consecutive_failures >= health_config.unhealthy_threshold {
                    if state.health != HealthStatus::Unhealthy {
                        tracing::warn!(
                            "Backend {} is now unhealthy",
                            state.backend.id
                        );
                    }
                    state.health = HealthStatus::Unhealthy;
                }
            }
        }
    }

    /// Get sticky session backend ID for a session ID
    pub async fn get_sticky_backend(&self, session_id: &str) -> Option<String> {
        let sessions = self.sticky_sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// Set sticky session backend for a session ID
    pub async fn set_sticky_backend(&self, session_id: String, backend_id: String) {
        let mut sessions = self.sticky_sessions.write().await;
        sessions.insert(session_id, backend_id);
    }

    /// Clear sticky session
    pub async fn clear_sticky_session(&self, session_id: &str) {
        let mut sessions = self.sticky_sessions.write().await;
        sessions.remove(session_id);
    }

    /// Get number of healthy backends
    pub async fn healthy_backend_count(&self) -> usize {
        let mut count = 0;
        for backend in &self.backends {
            let state = backend.read().await;
            if state.health == HealthStatus::Healthy && state.backend.enabled {
                count += 1;
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_balancer_creation() {
        let config = LoadBalancerConfig::new(Strategy::RoundRobin)
            .add_backend(Backend::new("backend1".to_string(), "127.0.0.1:8081".to_string()))
            .add_backend(Backend::new("backend2".to_string(), "127.0.0.1:8082".to_string()));

        let lb = LoadBalancer::new(config);
        assert_eq!(lb.backends.len(), 2);
    }

    #[tokio::test]
    async fn test_round_robin_selection() {
        let config = LoadBalancerConfig::new(Strategy::RoundRobin)
            .add_backend(Backend::new("backend1".to_string(), "127.0.0.1:8081".to_string()))
            .add_backend(Backend::new("backend2".to_string(), "127.0.0.1:8082".to_string()));

        let lb = LoadBalancer::new(config);

        for backend in &lb.backends {
            let mut state = backend.write().await;
            state.health = HealthStatus::Healthy;
        }

        let b1 = lb.select_round_robin().await;
        let b2 = lb.select_round_robin().await;

        assert!(b1.is_some());
        assert!(b2.is_some());
    }

    #[tokio::test]
    async fn test_least_connections_selection() {
        let config = LoadBalancerConfig::new(Strategy::LeastConnections)
            .add_backend(Backend::new("backend1".to_string(), "127.0.0.1:8081".to_string()))
            .add_backend(Backend::new("backend2".to_string(), "127.0.0.1:8082".to_string()));

        let lb = LoadBalancer::new(config);

        for backend in &lb.backends {
            let mut state = backend.write().await;
            state.health = HealthStatus::Healthy;
        }

        {
            let mut state = lb.backends[0].write().await;
            state.active_connections = 5;
        }

        let selected = lb.select_least_connections().await;
        assert!(selected.is_some());

        let binding = selected.unwrap();
        let state = binding.read().await;
        assert_eq!(state.backend.id, "backend2");
    }

    #[tokio::test]
    async fn test_statistics() {
        let config = LoadBalancerConfig::new(Strategy::RoundRobin)
            .add_backend(Backend::new("backend1".to_string(), "127.0.0.1:8081".to_string()));

        let lb = LoadBalancer::new(config);

        lb.total_requests.store(100, Ordering::Relaxed);
        lb.total_successes.store(95, Ordering::Relaxed);
        lb.total_failures.store(5, Ordering::Relaxed);

        let stats = lb.get_stats().await;
        assert_eq!(stats.total_requests, 100);
        assert_eq!(stats.total_successes, 95);
        assert_eq!(stats.total_failures, 5);
        assert_eq!(stats.success_rate(), 0.95);
    }
}