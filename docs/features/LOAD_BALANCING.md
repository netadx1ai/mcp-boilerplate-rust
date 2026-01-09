# Load Balancing Guide

Complete guide to using load balancing features in MCP Boilerplate Rust.

## Overview

The load balancer distributes incoming MCP requests across multiple backend servers, providing:

- High availability through automatic failover
- Improved performance via request distribution
- Health monitoring and automatic recovery
- Multiple load balancing strategies
- Real-time statistics and monitoring

## Quick Start

### Basic Setup

```rust
use mcp_boilerplate_rust::loadbalancer::{
    LoadBalancer, LoadBalancerConfig, Backend, Strategy
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let config = LoadBalancerConfig::new(Strategy::RoundRobin)
        .add_backend(Backend::new(
            "backend1".to_string(),
            "127.0.0.1:8081".to_string()
        ))
        .add_backend(Backend::new(
            "backend2".to_string(),
            "127.0.0.1:8082".to_string()
        ))
        .add_backend(Backend::new(
            "backend3".to_string(),
            "127.0.0.1:8083".to_string()
        ));

    // Create load balancer
    let lb = LoadBalancer::new(config);

    // Start health checks
    lb.start_health_checks().await;

    // Select a backend
    let backend = lb.select_backend(None).await?;
    let state = backend.read().await;
    println!("Selected: {}", state.backend.address);

    Ok(())
}
```

## Load Balancing Strategies

### 1. Round-Robin

Distributes requests evenly across all healthy backends in rotation.

**Use Case:** Equal backends, uniform request distribution

```rust
let config = LoadBalancerConfig::new(Strategy::RoundRobin)
    .add_backend(Backend::new("b1".to_string(), "127.0.0.1:8081".to_string()))
    .add_backend(Backend::new("b2".to_string(), "127.0.0.1:8082".to_string()))
    .add_backend(Backend::new("b3".to_string(), "127.0.0.1:8083".to_string()));

let lb = LoadBalancer::new(config);
```

**Characteristics:**
- Simple and predictable
- Equal load distribution
- No state tracking required
- Best for homogeneous backends

### 2. Least Connections

Routes requests to the backend with the fewest active connections.

**Use Case:** Long-lived connections, varying request durations

```rust
let config = LoadBalancerConfig::new(Strategy::LeastConnections)
    .add_backend(Backend::new("b1".to_string(), "127.0.0.1:8081".to_string()))
    .add_backend(Backend::new("b2".to_string(), "127.0.0.1:8082".to_string()));

let lb = LoadBalancer::new(config);
```

**Characteristics:**
- Dynamic load balancing
- Handles varying request durations
- Prevents overloading slow backends
- Best for mixed workloads

### 3. Random

Randomly selects a healthy backend for each request.

**Use Case:** Simple distribution, no state required

```rust
let config = LoadBalancerConfig::new(Strategy::Random)
    .add_backend(Backend::new("b1".to_string(), "127.0.0.1:8081".to_string()))
    .add_backend(Backend::new("b2".to_string(), "127.0.0.1:8082".to_string()));

let lb = LoadBalancer::new(config);
```

**Characteristics:**
- Stateless
- Good distribution over time
- Low overhead
- Best for simple scenarios

### 4. Weighted Round-Robin

Distributes requests based on backend weights (higher weight = more requests).

**Use Case:** Heterogeneous backends with different capacities

```rust
let config = LoadBalancerConfig::new(Strategy::WeightedRoundRobin)
    .add_backend(
        Backend::new("powerful".to_string(), "127.0.0.1:8081".to_string())
            .with_weight(5)
    )
    .add_backend(
        Backend::new("standard".to_string(), "127.0.0.1:8082".to_string())
            .with_weight(3)
    )
    .add_backend(
        Backend::new("light".to_string(), "127.0.0.1:8083".to_string())
            .with_weight(1)
    );

let lb = LoadBalancer::new(config);
```

**Characteristics:**
- Respects backend capacity differences
- Configurable distribution
- Good for mixed hardware
- Best for heterogeneous environments

### 5. IP Hash

Routes requests from the same client IP to the same backend (sticky sessions).

**Use Case:** Session affinity, stateful backends

```rust
use std::net::IpAddr;

let config = LoadBalancerConfig::new(Strategy::IpHash)
    .add_backend(Backend::new("b1".to_string(), "127.0.0.1:8081".to_string()))
    .add_backend(Backend::new("b2".to_string(), "127.0.0.1:8082".to_string()));

let lb = LoadBalancer::new(config);

// Select with client IP
let client_ip: IpAddr = "192.168.1.100".parse().unwrap();
let backend = lb.select_backend(Some(client_ip)).await?;
```

**Characteristics:**
- Consistent backend selection per client
- Session affinity
- Cache-friendly
- Best for stateful applications

## Backend Configuration

### Basic Backend

```rust
let backend = Backend::new(
    "backend1".to_string(),
    "127.0.0.1:8081".to_string()
);
```

### Backend with Weight

```rust
let backend = Backend::new("backend1".to_string(), "127.0.0.1:8081".to_string())
    .with_weight(5);
```

### Backend with Connection Limit

```rust
let backend = Backend::new("backend1".to_string(), "127.0.0.1:8081".to_string())
    .with_max_connections(100);
```

### Backend with Custom Timeout

```rust
let backend = Backend::new("backend1".to_string(), "127.0.0.1:8081".to_string())
    .with_timeout(60);
```

### Complete Configuration

```rust
let backend = Backend::new("backend1".to_string(), "127.0.0.1:8081".to_string())
    .with_weight(3)
    .with_max_connections(200)
    .with_timeout(30);
```

## Health Checking

### Default Health Check

```rust
let config = LoadBalancerConfig::new(Strategy::RoundRobin)
    .add_backend(Backend::new("b1".to_string(), "127.0.0.1:8081".to_string()));

let lb = LoadBalancer::new(config);
lb.start_health_checks().await;
```

Default settings:
- Interval: 10 seconds
- Timeout: 5 seconds
- Unhealthy threshold: 3 failures
- Healthy threshold: 2 successes
- Path: `/health`
- Expected status: 200

### Custom Health Check

```rust
use mcp_boilerplate_rust::loadbalancer::HealthCheckConfig;
use std::time::Duration;

let health_config = HealthCheckConfig {
    interval: Duration::from_secs(5),
    timeout: Duration::from_secs(3),
    unhealthy_threshold: 2,
    healthy_threshold: 3,
    path: "/api/health".to_string(),
    expected_status: 200,
};

let config = LoadBalancerConfig::new(Strategy::RoundRobin)
    .add_backend(Backend::new("b1".to_string(), "127.0.0.1:8081".to_string()))
    .with_health_check(health_config);

let lb = LoadBalancer::new(config);
lb.start_health_checks().await;
```

### Health Status

Health checks automatically mark backends as:
- **Healthy**: Passing health checks, accepting traffic
- **Unhealthy**: Failing health checks, no traffic
- **Unknown**: Initial state or being checked

## Advanced Features

### Sticky Sessions

Enable session affinity using cookies:

```rust
let config = LoadBalancerConfig::new(Strategy::RoundRobin)
    .add_backend(Backend::new("b1".to_string(), "127.0.0.1:8081".to_string()))
    .add_backend(Backend::new("b2".to_string(), "127.0.0.1:8082".to_string()))
    .with_sticky_sessions(true);

let lb = LoadBalancer::new(config);

// Set sticky session
lb.set_sticky_backend(
    "session_id_123".to_string(),
    "b1".to_string()
).await;

// Get sticky backend
if let Some(backend_id) = lb.get_sticky_backend("session_id_123").await {
    println!("Session pinned to: {}", backend_id);
}
```

### Failover

Enable automatic failover to healthy backends:

```rust
let config = LoadBalancerConfig::new(Strategy::RoundRobin)
    .add_backend(Backend::new("b1".to_string(), "127.0.0.1:8081".to_string()))
    .add_backend(Backend::new("b2".to_string(), "127.0.0.1:8082".to_string()))
    .with_failover(true)
    .with_max_retries(3);

let lb = LoadBalancer::new(config);
```

### Dynamic Backend Management

Add backends at runtime:

```rust
let mut lb = LoadBalancer::new(config);

// Add new backend
lb.add_backend(Backend::new(
    "backend4".to_string(),
    "127.0.0.1:8084".to_string()
)).await;

// Remove backend
lb.remove_backend("backend1").await;

// Enable/disable backend
lb.set_backend_enabled("backend2", false).await;
lb.set_backend_enabled("backend2", true).await;
```

### Connection Tracking

Track connections for monitoring and limits:

```rust
let backend = lb.select_backend(None).await?;

// Mark connection start
lb.mark_connection_start(&backend).await;

// Process request...
let start = std::time::Instant::now();
// ... do work ...
let duration_ms = start.elapsed().as_millis() as f64;

// Mark connection end
lb.mark_connection_end(&backend, true, duration_ms).await;
```

## Monitoring and Statistics

### Get Statistics

```rust
let stats = lb.get_stats().await;

println!("Total requests: {}", stats.total_requests);
println!("Success rate: {:.2}%", stats.success_rate() * 100.0);
println!("Failure rate: {:.2}%", stats.failure_rate() * 100.0);
println!("Avg response time: {:.2}ms", stats.avg_response_time_ms);
println!("Active connections: {}", stats.active_connections);
println!("Uptime: {}s", stats.uptime_secs);
```

### Get Backend States

```rust
let backend_states = lb.get_backend_states().await;

for state in backend_states {
    println!("\nBackend: {}", state.backend.id);
    println!("  Address: {}", state.backend.address);
    println!("  Health: {}", state.health);
    println!("  Active connections: {}", state.active_connections);
    println!("  Total requests: {}", state.total_requests);
    println!("  Failed requests: {}", state.failed_requests);
    println!("  Failure rate: {:.2}%", state.failure_rate() * 100.0);
    println!("  Avg response time: {:.2}ms", state.avg_response_time_ms);
}
```

### Check Healthy Backends

```rust
let healthy_count = lb.healthy_backend_count().await;
println!("Healthy backends: {}/{}", healthy_count, lb.backends.len());
```

## Complete Example

```rust
use mcp_boilerplate_rust::loadbalancer::{
    LoadBalancer, LoadBalancerConfig, Backend, Strategy, HealthCheckConfig
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure health checks
    let health_config = HealthCheckConfig {
        interval: Duration::from_secs(10),
        timeout: Duration::from_secs(5),
        unhealthy_threshold: 3,
        healthy_threshold: 2,
        path: "/health".to_string(),
        expected_status: 200,
    };

    // Create load balancer configuration
    let config = LoadBalancerConfig::new(Strategy::WeightedRoundRobin)
        .add_backend(
            Backend::new("backend1".to_string(), "127.0.0.1:8081".to_string())
                .with_weight(5)
                .with_max_connections(100)
        )
        .add_backend(
            Backend::new("backend2".to_string(), "127.0.0.1:8082".to_string())
                .with_weight(3)
                .with_max_connections(75)
        )
        .add_backend(
            Backend::new("backend3".to_string(), "127.0.0.1:8083".to_string())
                .with_weight(2)
                .with_max_connections(50)
        )
        .with_health_check(health_config)
        .with_sticky_sessions(true)
        .with_failover(true)
        .with_max_retries(3);

    // Create load balancer
    let lb = LoadBalancer::new(config);

    // Start health checks
    lb.start_health_checks().await;

    // Wait for initial health checks
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Process requests
    for i in 0..10 {
        match lb.select_backend(None).await {
            Ok(backend) => {
                let start = std::time::Instant::now();
                
                // Mark connection start
                lb.mark_connection_start(&backend).await;

                // Simulate request processing
                let state = backend.read().await;
                println!("Request {} -> {}", i, state.backend.address);
                drop(state);

                tokio::time::sleep(Duration::from_millis(100)).await;

                // Mark connection end
                let duration_ms = start.elapsed().as_millis() as f64;
                lb.mark_connection_end(&backend, true, duration_ms).await;
            }
            Err(e) => eprintln!("Failed to select backend: {}", e),
        }
    }

    // Print statistics
    println!("\n=== Load Balancer Statistics ===");
    let stats = lb.get_stats().await;
    println!("Total requests: {}", stats.total_requests);
    println!("Success rate: {:.2}%", stats.success_rate() * 100.0);
    println!("Avg response time: {:.2}ms", stats.avg_response_time_ms);
    println!("Uptime: {}s", stats.uptime_secs);

    println!("\n=== Backend States ===");
    let backend_states = lb.get_backend_states().await;
    for state in backend_states {
        println!("\n{} ({})", state.backend.id, state.backend.address);
        println!("  Health: {}", state.health);
        println!("  Weight: {}", state.backend.weight);
        println!("  Requests: {}", state.total_requests);
        println!("  Failures: {}", state.failed_requests);
        println!("  Active connections: {}", state.active_connections);
    }

    Ok(())
}
```

## Production Deployment

### High Availability Setup

```rust
// 3 backends with health checks and failover
let config = LoadBalancerConfig::new(Strategy::LeastConnections)
    .add_backend(
        Backend::new("primary".to_string(), "10.0.1.10:8080".to_string())
            .with_max_connections(500)
    )
    .add_backend(
        Backend::new("secondary".to_string(), "10.0.1.11:8080".to_string())
            .with_max_connections(500)
    )
    .add_backend(
        Backend::new("tertiary".to_string(), "10.0.1.12:8080".to_string())
            .with_max_connections(500)
    )
    .with_health_check(HealthCheckConfig {
        interval: Duration::from_secs(5),
        timeout: Duration::from_secs(3),
        unhealthy_threshold: 2,
        healthy_threshold: 3,
        path: "/health".to_string(),
        expected_status: 200,
    })
    .with_failover(true)
    .with_max_retries(3);
```

### Multi-Region Setup

```rust
// Weighted distribution across regions
let config = LoadBalancerConfig::new(Strategy::WeightedRoundRobin)
    .add_backend(
        Backend::new("us-east".to_string(), "us-east.example.com:8080".to_string())
            .with_weight(5)
    )
    .add_backend(
        Backend::new("us-west".to_string(), "us-west.example.com:8080".to_string())
            .with_weight(3)
    )
    .add_backend(
        Backend::new("eu-central".to_string(), "eu.example.com:8080".to_string())
            .with_weight(2)
    );
```

## Best Practices

### 1. Choose the Right Strategy

- **Round-Robin**: Equal backends, simple distribution
- **Least Connections**: Varying request durations
- **Weighted**: Different backend capacities
- **IP Hash**: Session affinity required
- **Random**: Simple, stateless scenarios

### 2. Configure Health Checks

- Set appropriate intervals (5-30 seconds)
- Use dedicated health endpoints
- Configure thresholds based on reliability needs
- Monitor health check results

### 3. Set Connection Limits

- Prevent backend overload
- Match backend capacity
- Leave headroom for spikes
- Monitor connection usage

### 4. Enable Failover

- Always enable for production
- Set appropriate retry counts (2-5)
- Monitor failover events
- Have sufficient healthy backends

### 5. Monitor Statistics

- Track success/failure rates
- Monitor response times
- Watch active connections
- Alert on unhealthy backends

### 6. Dynamic Management

- Add backends gradually
- Drain connections before removal
- Use health checks for safe updates
- Test configuration changes

## Troubleshooting

### No Healthy Backends

```rust
let healthy = lb.healthy_backend_count().await;
if healthy == 0 {
    eprintln!("No healthy backends!");
    let states = lb.get_backend_states().await;
    for state in states {
        eprintln!("{}: {}", state.backend.id, state.health);
    }
}
```

### High Failure Rate

```rust
let stats = lb.get_stats().await;
if stats.failure_rate() > 0.1 {
    eprintln!("High failure rate: {:.2}%", stats.failure_rate() * 100.0);
    
    let states = lb.get_backend_states().await;
    for state in states {
        if state.failure_rate() > 0.1 {
            eprintln!("Backend {} failing: {:.2}%", 
                state.backend.id, 
                state.failure_rate() * 100.0
            );
        }
    }
}
```

### Connection Limit Reached

```rust
let states = lb.get_backend_states().await;
for state in states {
    if state.backend.max_connections > 0 {
        let usage = (state.active_connections as f64 / 
                    state.backend.max_connections as f64) * 100.0;
        if usage > 80.0 {
            eprintln!("Backend {} at {:.0}% capacity", 
                state.backend.id, usage
            );
        }
    }
}
```

## Performance Tuning

### 1. Health Check Tuning

- Increase interval to reduce overhead
- Decrease timeout for faster detection
- Adjust thresholds for stability

### 2. Connection Pooling

- Set appropriate max_connections
- Monitor and adjust based on load
- Use least connections strategy

### 3. Strategy Selection

- Use IP hash for cache efficiency
- Use least connections for varying loads
- Use weighted for heterogeneous backends

### 4. Monitoring

- Collect metrics regularly
- Track response times
- Monitor backend health
- Alert on anomalies

## Version

Current: 0.5.0  
Last Updated: 2026-01-09 HCMC