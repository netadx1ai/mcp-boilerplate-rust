# Multi-Server Coordination Examples

This directory contains practical examples of coordinating multiple MCP servers to accomplish complex workflows. These examples demonstrate real-world patterns for building sophisticated applications using our MCP server ecosystem.

## Overview

Multi-server coordination allows you to:
- **Chain Operations**: Output from one server becomes input to another
- **Parallel Processing**: Execute multiple server operations simultaneously
- **Error Recovery**: Handle failures gracefully across server boundaries
- **Performance Optimization**: Cache results and optimize data flow
- **Resource Management**: Coordinate shared resources across servers

## Available Examples

### 1. Content Generation Pipeline (`content-pipeline/`)
**Servers Used**: news-data → template → database → analytics  
**Use Case**: Automated content creation from news data to published articles

```
News Data → Template Rendering → Database Storage → Analytics Tracking
```

**Features**:
- Fetches trending news from news-data-server
- Generates articles using template-server
- Stores content in database-server
- Tracks performance with analytics-server
- Error handling and rollback capabilities
- Performance monitoring and optimization

### 2. Business Intelligence Dashboard (`bi-dashboard/`)
**Servers Used**: database → analytics → template → workflow  
**Use Case**: Generate comprehensive business reports

```
Database Query → Analytics Processing → Report Template → Workflow Automation
```

**Features**:
- Multi-table data aggregation
- Statistical analysis and KPI calculation
- Professional report generation
- Automated distribution workflow
- Performance benchmarking

### 3. API Integration Chain (`api-integration/`)
**Servers Used**: api-gateway → database → template → analytics  
**Use Case**: External API data processing and reporting

```
External API → Data Processing → Content Generation → Performance Tracking
```

**Features**:
- External API data fetching with resilience
- Data transformation and validation
- Template-based report generation
- Comprehensive analytics tracking
- Circuit breaker and retry patterns

### 4. Real-time Monitoring System (`monitoring-system/`)
**Servers Used**: All 6 servers in coordinated fashion  
**Use Case**: Complete system health and performance monitoring

```
All Servers → Health Checks → Alert Generation → Dashboard Updates
```

**Features**:
- Multi-server health monitoring
- Performance metrics aggregation
- Alert template generation
- Workflow-based incident response
- Real-time dashboard updates

## Coordination Patterns

### 1. Sequential Coordination (Pipeline)
```rust
// Example: Content generation pipeline
let news = news_server.search_news("AI technology", 5).await?;
let article = template_server.render_template("blog_post", news_data).await?;
let stored_id = database_server.execute_query("INSERT INTO articles...", params).await?;
let analytics = analytics_server.track_content_creation(stored_id).await?;
```

### 2. Parallel Coordination (Fan-out)
```rust
// Example: Multi-source data aggregation
let (news_future, analytics_future, db_future) = tokio::join!(
    news_server.get_trending_news("technology", 10),
    analytics_server.get_engagement_trends("last_week"),
    database_server.execute_query("SELECT * FROM popular_articles", vec![])
);
```

### 3. Conditional Coordination (Decision Trees)
```rust
// Example: Adaptive content generation
let audience_data = analytics_server.get_audience_insights("tech_enthusiasts").await?;
let template_id = if audience_data.engagement_level > 0.8 {
    "advanced_technical_post"
} else {
    "beginner_friendly_post"
};
let content = template_server.render_template(template_id, data).await?;
```

### 4. Error Recovery Coordination
```rust
// Example: Graceful degradation
let result = match news_server.search_news(query, limit).await {
    Ok(news) => news,
    Err(_) => {
        // Fallback to cached data from database
        database_server.execute_query("SELECT * FROM cached_news WHERE...", params).await?
    }
};
```

## Error Handling Strategies

### 1. Circuit Breaker Pattern
```rust
pub struct ServerCircuitBreaker {
    failure_count: AtomicU32,
    last_failure: Mutex<Option<Instant>>,
    failure_threshold: u32,
    recovery_timeout: Duration,
}

impl ServerCircuitBreaker {
    pub async fn call<T, E>(&self, operation: impl Future<Output = Result<T, E>>) -> Result<T, CircuitBreakerError<E>> {
        if self.is_open().await {
            return Err(CircuitBreakerError::CircuitOpen);
        }
        
        match operation.await {
            Ok(result) => {
                self.reset_failures().await;
                Ok(result)
            }
            Err(error) => {
                self.record_failure().await;
                Err(CircuitBreakerError::OperationFailed(error))
            }
        }
    }
}
```

### 2. Retry with Exponential Backoff
```rust
pub async fn retry_with_backoff<T, E>(
    operation: impl Fn() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>,
    max_attempts: usize,
    base_delay: Duration,
) -> Result<T, E> {
    let mut attempt = 0;
    let mut delay = base_delay;
    
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) if attempt >= max_attempts - 1 => return Err(error),
            Err(_) => {
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
                attempt += 1;
            }
        }
    }
}
```

### 3. Graceful Degradation
```rust
pub async fn fetch_with_fallback(
    primary_server: &NewsDataServer,
    fallback_server: &DatabaseServer,
    query: &str,
) -> Result<Vec<NewsItem>, CoordinationError> {
    // Try primary server
    match primary_server.search_news(query, 10).await {
        Ok(news) => Ok(news),
        Err(primary_error) => {
            tracing::warn!("Primary server failed: {}, using fallback", primary_error);
            
            // Try fallback from cached data
            match fallback_server.execute_query(
                "SELECT * FROM cached_news WHERE category LIKE ?",
                vec![query.to_string()]
            ).await {
                Ok(cached_data) => Ok(parse_cached_news(cached_data)?),
                Err(fallback_error) => {
                    Err(CoordinationError::AllServersFailed {
                        primary: primary_error,
                        fallback: fallback_error,
                    })
                }
            }
        }
    }
}
```

## Performance Optimization

### 1. Request Batching
```rust
// Batch multiple operations for efficiency
pub async fn batch_content_creation(
    news_items: Vec<NewsItem>,
    template_server: &TemplateServer,
    database_server: &DatabaseServer,
) -> Result<Vec<String>, CoordinationError> {
    // Process in parallel batches
    let batch_size = 5;
    let batches: Vec<_> = news_items.chunks(batch_size).collect();
    
    let mut all_results = Vec::new();
    for batch in batches {
        let futures: Vec<_> = batch.iter().map(|news_item| {
            create_single_content(news_item, template_server, database_server)
        }).collect();
        
        let batch_results = futures::future::try_join_all(futures).await?;
        all_results.extend(batch_results);
    }
    
    Ok(all_results)
}
```

### 2. Response Caching
```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct CoordinationCache {
    cache: Arc<RwLock<HashMap<String, CachedResponse>>>,
    ttl: Duration,
}

impl CoordinationCache {
    pub async fn get_or_compute<T>(
        &self,
        key: &str,
        compute: impl Future<Output = Result<T, CoordinationError>>,
    ) -> Result<T, CoordinationError> 
    where
        T: Clone + Send + Sync + 'static,
    {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(key) {
                if cached.expires_at > Instant::now() {
                    if let Some(value) = cached.value.downcast_ref::<T>() {
                        return Ok(value.clone());
                    }
                }
            }
        }
        
        // Compute and cache
        let result = compute.await?;
        let cached_response = CachedResponse {
            value: Box::new(result.clone()),
            expires_at: Instant::now() + self.ttl,
        };
        
        {
            let mut cache = self.cache.write().await;
            cache.insert(key.to_string(), cached_response);
        }
        
        Ok(result)
    }
}
```

### 3. Connection Pooling
```rust
pub struct ServerPool<T> {
    servers: Vec<Arc<T>>,
    current: AtomicUsize,
}

impl<T> ServerPool<T> {
    pub fn new(servers: Vec<T>) -> Self {
        Self {
            servers: servers.into_iter().map(Arc::new).collect(),
            current: AtomicUsize::new(0),
        }
    }
    
    pub fn get_server(&self) -> Arc<T> {
        let index = self.current.fetch_add(1, Ordering::Relaxed) % self.servers.len();
        self.servers[index].clone()
    }
}
```

## Getting Started

1. **Setup Development Environment**:
   ```bash
   cd examples/multi-server-coordination
   cargo build
   ```

2. **Run Individual Examples**:
   ```bash
   # Content generation pipeline
   cargo run --bin content-pipeline
   
   # Business intelligence dashboard
   cargo run --bin bi-dashboard
   
   # API integration chain
   cargo run --bin api-integration
   
   # Real-time monitoring
   cargo run --bin monitoring-system
   ```

3. **Run All Examples**:
   ```bash
   just run-coordination-examples
   ```

## Testing

Each example includes comprehensive tests demonstrating:
- **Unit Tests**: Individual coordination functions
- **Integration Tests**: Full multi-server workflows
- **Performance Tests**: Latency and throughput benchmarks
- **Failure Tests**: Error handling and recovery scenarios

```bash
# Run all coordination tests
cargo test

# Run specific example tests
cargo test --bin content-pipeline

# Run performance benchmarks
cargo bench
```

## Monitoring and Debugging

### Built-in Observability
- **Structured Logging**: JSON logs with correlation IDs
- **Metrics Collection**: Performance and error metrics
- **Distributed Tracing**: Request flow across servers
- **Health Checks**: Server availability monitoring

### Debug Tools
```bash
# Enable debug logging
RUST_LOG=debug cargo run --bin content-pipeline

# Performance profiling
RUST_LOG=trace cargo run --bin content-pipeline

# Health check all servers
cargo run --bin health-check
```

## Security Considerations

### 1. Input Validation
- Validate all inputs before server coordination
- Sanitize data passed between servers
- Implement rate limiting for coordination operations

### 2. Authentication & Authorization
- Consistent authentication across server chain
- Role-based access control for coordination operations
- Secure credential management

### 3. Data Privacy
- Minimize data exposure across server boundaries
- Implement data encryption for sensitive operations
- Audit trail for coordination activities

## Best Practices

1. **Design for Failure**: Every coordination should handle server failures gracefully
2. **Idempotent Operations**: Ensure operations can be safely retried
3. **Timeout Management**: Set appropriate timeouts for each coordination step
4. **Resource Cleanup**: Always clean up resources on failures
5. **Monitoring**: Instrument all coordination points for observability
6. **Documentation**: Document coordination flows and error scenarios

## Contributing

When adding new coordination examples:

1. Follow the established patterns in existing examples
2. Include comprehensive error handling
3. Add performance benchmarks
4. Write integration tests
5. Update this README with new patterns
6. Add security considerations for new workflows

## License

This project is licensed under the MIT License - see the LICENSE file for details.