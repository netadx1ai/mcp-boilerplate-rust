# MCP Server Performance Guide

**Version**: 1.0  
**Last Updated**: January 18, 2025  
**SDK Version**: RMCP v0.6.3  
**Project**: mcp-boilerplate-rust

A comprehensive guide for optimizing MCP server performance, based on proven patterns from our 6-server production ecosystem achieving consistent sub-50ms response times.

---

## Table of Contents

1. [Performance Philosophy](#performance-philosophy)
2. [Benchmarking and Metrics](#benchmarking-and-metrics)
3. [Async Optimization](#async-optimization)
4. [Memory Management](#memory-management)
5. [Database Performance](#database-performance)
6. [Caching Strategies](#caching-strategies)
7. [Network Optimization](#network-optimization)
8. [Resource Management](#resource-management)
9. [Profiling and Debugging](#profiling-and-debugging)
10. [Production Optimization](#production-optimization)

---

## Performance Philosophy

### Target Metrics

Based on our 6-server ecosystem, we've established these proven performance targets:

- **Query Tools**: < 50ms (cached) / < 200ms (uncached)
- **Action Tools**: < 100ms (simple) / < 500ms (complex)
- **Status Tools**: < 10ms (in-memory) / < 50ms (with health checks)
- **Memory Usage**: < 512MB per server instance
- **CPU Usage**: < 70% under normal load
- **Throughput**: > 1000 requests/second per instance

### Performance-First Design

Every architectural decision considers performance impact:

```rust
// ✅ Performance-optimized pattern
#[tool]
async fn optimized_tool(&self, input: String) -> Result<String, ServerError> {
    // 1. Early validation (fail fast)
    if input.is_empty() {
        return Err(ServerError::ValidationError {
            field: "input".to_string(),
            message: "Input cannot be empty".to_string(),
        });
    }
    
    // 2. Check cache first
    let cache_key = format!("tool_result:{}", hash(&input));
    if let Some(cached) = self.get_cached(&cache_key).await {
        return Ok(cached);
    }
    
    // 3. Scoped async work
    let result = {
        let start = Instant::now();
        let result = self.expensive_computation(input).await?;
        
        // Log performance
        let duration = start.elapsed();
        if duration > Duration::from_millis(50) {
            tracing::warn!("Slow operation: {:?}", duration);
        }
        
        result
    };
    
    // 4. Cache result
    self.set_cached(&cache_key, &result, Duration::from_secs(300)).await;
    
    Ok(result)
}
```

---

## Benchmarking and Metrics

### Performance Testing Framework

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;

fn benchmark_mcp_tools(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let server = rt.block_on(async { create_test_server().await });
    
    c.bench_function("search_tool", |b| {
        b.iter(|| {
            rt.block_on(async {
                server.search_content(black_box("test query".to_string())).await
            })
        })
    });
    
    c.bench_function("create_tool", |b| {
        b.iter(|| {
            rt.block_on(async {
                server.create_content(
                    black_box("title".to_string()),
                    black_box("content".to_string()),
                ).await
            })
        })
    });
}

criterion_group!(benches, benchmark_mcp_tools);
criterion_main!(benches);
```

### Metrics Collection

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct PerformanceMetrics {
    // Request metrics
    request_count: AtomicU64,
    error_count: AtomicU64,
    
    // Response time tracking
    response_times: Arc<RwLock<HashMap<String, Vec<Duration>>>>,
    
    // Cache metrics
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    
    // Resource metrics
    memory_usage: AtomicU64,
    cpu_usage: AtomicU64,
}

impl PerformanceMetrics {
    pub async fn record_request(&self, tool_name: &str, duration: Duration) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
        
        let mut times = self.response_times.write().await;
        times.entry(tool_name.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
        
        // Keep only recent measurements (sliding window)
        let cutoff = Instant::now() - Duration::from_secs(300); // 5 minutes
        for times in times.values_mut() {
            times.retain(|&d| d > cutoff.elapsed());
        }
    }
    
    pub async fn get_average_response_time(&self, tool_name: &str) -> Option<Duration> {
        let times = self.response_times.read().await;
        times.get(tool_name).map(|durations| {
            if durations.is_empty() {
                Duration::ZERO
            } else {
                let total: Duration = durations.iter().sum();
                total / durations.len() as u32
            }
        })
    }
    
    pub fn get_cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;
        
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }
}
```

### Automated Performance Testing

```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::{Duration, Instant};
    
    #[tokio::test]
    async fn test_response_time_requirements() {
        let server = create_test_server().await;
        
        // Test 100 requests for consistent performance
        for i in 0..100 {
            let start = Instant::now();
            let result = server.fast_tool(format!("input_{}", i)).await;
            let duration = start.elapsed();
            
            assert!(result.is_ok(), "Request {} failed: {:?}", i, result.err());
            assert!(
                duration < Duration::from_millis(50),
                "Request {} took {:?}, expected < 50ms",
                i,
                duration
            );
        }
    }
    
    #[tokio::test]
    async fn test_throughput_requirements() {
        let server = Arc::new(create_test_server().await);
        let operations_count = 1000;
        let start = Instant::now();
        
        // Execute operations concurrently
        let futures: Vec<_> = (0..operations_count)
            .map(|i| {
                let server_clone = server.clone();
                tokio::spawn(async move {
                    server_clone.lightweight_tool(format!("input_{}", i)).await
                })
            })
            .collect();
        
        // Wait for all operations
        let results = futures::future::join_all(futures).await;
        let total_duration = start.elapsed();
        
        // Verify all succeeded
        for (i, result) in results.into_iter().enumerate() {
            assert!(result.unwrap().is_ok(), "Operation {} failed", i);
        }
        
        // Calculate throughput
        let ops_per_second = operations_count as f64 / total_duration.as_secs_f64();
        assert!(
            ops_per_second > 1000.0,
            "Throughput {} ops/sec, expected > 1000 ops/sec",
            ops_per_second
        );
    }
    
    #[tokio::test]
    async fn test_memory_usage() {
        let server = create_test_server().await;
        
        // Get baseline memory usage
        let baseline_memory = get_memory_usage();
        
        // Perform memory-intensive operations
        for i in 0..1000 {
            let _ = server.memory_intensive_tool(format!("data_{}", i)).await;
        }
        
        // Force garbage collection
        for _ in 0..3 {
            tokio::task::yield_now().await;
        }
        
        // Check memory increase
        let current_memory = get_memory_usage();
        let memory_increase = current_memory - baseline_memory;
        
        assert!(
            memory_increase < 100_000_000, // 100MB
            "Memory increased by {} bytes, expected < 100MB",
            memory_increase
        );
    }
}

fn get_memory_usage() -> u64 {
    // Implementation depends on platform
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/self/status")
            .unwrap()
            .lines()
            .find(|line| line.starts_with("VmRSS:"))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0) * 1024 // Convert KB to bytes
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        // Fallback for other platforms
        0
    }
}
```

---

## Async Optimization

### Efficient Async Patterns

```rust
// ✅ Parallel processing pattern
#[tool]
async fn parallel_processing(&self, items: Vec<String>) -> Result<Vec<ProcessedItem>, ServerError> {
    // Process items in parallel with controlled concurrency
    use futures::stream::{self, StreamExt};
    
    const MAX_CONCURRENT: usize = 10;
    
    let results: Result<Vec<_>, _> = stream::iter(items)
        .map(|item| self.process_single_item(item))
        .buffer_unordered(MAX_CONCURRENT)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect();
    
    results
}

// ✅ Batched operations pattern
#[tool]
async fn batched_database_operations(&self, ids: Vec<String>) -> Result<Vec<DataItem>, ServerError> {
    const BATCH_SIZE: usize = 100;
    let mut all_results = Vec::new();
    
    for chunk in ids.chunks(BATCH_SIZE) {
        let batch_results = self.fetch_batch(chunk.to_vec()).await?;
        all_results.extend(batch_results);
    }
    
    Ok(all_results)
}

// ✅ Timeout and cancellation pattern
#[tool]
async fn timeout_protected_operation(&self, input: String) -> Result<String, ServerError> {
    use tokio::time::{timeout, Duration};
    
    let operation = self.potentially_slow_operation(input);
    
    match timeout(Duration::from_secs(5), operation).await {
        Ok(result) => result,
        Err(_) => Err(ServerError::Timeout {
            operation: "slow_operation".to_string(),
            timeout_seconds: 5,
        }),
    }
}
```

### Lock-Free Programming

```rust
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use crossbeam::queue::SegQueue;

pub struct LockFreeCounter {
    count: AtomicU64,
    last_reset: AtomicU64,
}

impl LockFreeCounter {
    pub fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
            last_reset: AtomicU64::new(Self::current_timestamp()),
        }
    }
    
    pub fn increment(&self) -> u64 {
        self.count.fetch_add(1, Ordering::Relaxed)
    }
    
    pub fn get_rate_per_second(&self) -> f64 {
        let now = Self::current_timestamp();
        let last_reset = self.last_reset.load(Ordering::Relaxed);
        let count = self.count.load(Ordering::Relaxed);
        
        let duration_secs = (now - last_reset) as f64 / 1000.0;
        if duration_secs > 0.0 {
            count as f64 / duration_secs
        } else {
            0.0
        }
    }
    
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

// Lock-free message queue
pub struct LockFreeMessageQueue<T> {
    queue: SegQueue<T>,
    size: AtomicU64,
    max_size: u64,
}

impl<T> LockFreeMessageQueue<T> {
    pub fn new(max_size: u64) -> Self {
        Self {
            queue: SegQueue::new(),
            size: AtomicU64::new(0),
            max_size,
        }
    }
    
    pub fn try_push(&self, item: T) -> Result<(), T> {
        if self.size.load(Ordering::Relaxed) >= self.max_size {
            return Err(item);
        }
        
        self.queue.push(item);
        self.size.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    
    pub fn try_pop(&self) -> Option<T> {
        match self.queue.pop() {
            Some(item) => {
                self.size.fetch_sub(1, Ordering::Relaxed);
                Some(item)
            }
            None => None,
        }
    }
}
```

### Async Runtime Optimization

```rust
// Optimized Tokio runtime configuration
use tokio::runtime::{Builder, Runtime};

pub fn create_optimized_runtime() -> Result<Runtime, Box<dyn std::error::Error>> {
    let num_cpus = num_cpus::get();
    let worker_threads = std::env::var("TOKIO_WORKER_THREADS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(num_cpus);
    
    let runtime = Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .max_blocking_threads(64)
        .thread_name("mcp-worker")
        .thread_stack_size(4 * 1024 * 1024) // 4MB stack size
        .enable_all()
        .build()?;
    
    Ok(runtime)
}

// Runtime metrics collection
pub struct RuntimeMetrics {
    runtime_handle: tokio::runtime::Handle,
}

impl RuntimeMetrics {
    pub fn new(handle: tokio::runtime::Handle) -> Self {
        Self {
            runtime_handle: handle,
        }
    }
    
    pub fn get_metrics(&self) -> RuntimeStats {
        let metrics = self.runtime_handle.metrics();
        
        RuntimeStats {
            worker_threads: metrics.num_workers(),
            blocking_threads: metrics.num_blocking_threads(),
            active_tasks: metrics.active_tasks_count(),
            pending_tasks: metrics.scheduled_tasks_count(),
        }
    }
}

#[derive(Debug)]
pub struct RuntimeStats {
    pub worker_threads: usize,
    pub blocking_threads: usize,
    pub active_tasks: usize,
    pub pending_tasks: usize,
}
```

---

## Memory Management

### Efficient Data Structures

```rust
use std::collections::HashMap;
use lru::LruCache;
use smallvec::SmallVec;

// Use SmallVec for collections that are usually small
type SmallStringVec = SmallVec<[String; 4]>;

// Efficient cache with memory bounds
pub struct BoundedCache<K, V> {
    cache: LruCache<K, CachedItem<V>>,
    memory_usage: AtomicU64,
    max_memory: u64,
}

#[derive(Clone)]
struct CachedItem<V> {
    value: V,
    size: u64,
    created_at: Instant,
    ttl: Duration,
}

impl<K, V> BoundedCache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    pub fn new(max_entries: usize, max_memory: u64) -> Self {
        Self {
            cache: LruCache::new(max_entries),
            memory_usage: AtomicU64::new(0),
            max_memory,
        }
    }
    
    pub fn insert(&mut self, key: K, value: V, ttl: Duration) -> Option<V> {
        let size = std::mem::size_of_val(&value) as u64;
        
        // Check memory bounds
        while self.memory_usage.load(Ordering::Relaxed) + size > self.max_memory {
            if let Some((_, old_item)) = self.cache.pop_lru() {
                self.memory_usage.fetch_sub(old_item.size, Ordering::Relaxed);
            } else {
                break;
            }
        }
        
        let item = CachedItem {
            value: value.clone(),
            size,
            created_at: Instant::now(),
            ttl,
        };
        
        let old_value = self.cache.put(key, item).map(|old_item| {
            self.memory_usage.fetch_sub(old_item.size, Ordering::Relaxed);
            old_item.value
        });
        
        self.memory_usage.fetch_add(size, Ordering::Relaxed);
        old_value
    }
    
    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(item) = self.cache.get(key) {
            if item.created_at.elapsed() <= item.ttl {
                return Some(item.value.clone());
            } else {
                // Item expired
                self.cache.pop(key);
                self.memory_usage.fetch_sub(item.size, Ordering::Relaxed);
            }
        }
        None
    }
}
```

### Memory Pool Pattern

```rust
use std::sync::Mutex;

pub struct BufferPool {
    pools: Vec<Mutex<Vec<Vec<u8>>>>,
    sizes: Vec<usize>,
}

impl BufferPool {
    pub fn new() -> Self {
        // Common buffer sizes: 1KB, 4KB, 16KB, 64KB, 256KB
        let sizes = vec![1024, 4096, 16384, 65536, 262144];
        let pools = sizes.iter().map(|_| Mutex::new(Vec::new())).collect();
        
        Self { pools, sizes }
    }
    
    pub fn get_buffer(&self, min_size: usize) -> PooledBuffer {
        for (i, &size) in self.sizes.iter().enumerate() {
            if size >= min_size {
                let mut pool = self.pools[i].lock().unwrap();
                if let Some(mut buffer) = pool.pop() {
                    buffer.clear();
                    return PooledBuffer {
                        buffer,
                        pool_index: Some(i),
                        pool: self,
                    };
                } else {
                    return PooledBuffer {
                        buffer: Vec::with_capacity(size),
                        pool_index: Some(i),
                        pool: self,
                    };
                }
            }
        }
        
        // Fallback for very large buffers
        PooledBuffer {
            buffer: Vec::with_capacity(min_size),
            pool_index: None,
            pool: self,
        }
    }
}

pub struct PooledBuffer<'a> {
    buffer: Vec<u8>,
    pool_index: Option<usize>,
    pool: &'a BufferPool,
}

impl<'a> std::ops::Deref for PooledBuffer<'a> {
    type Target = Vec<u8>;
    
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl<'a> std::ops::DerefMut for PooledBuffer<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

impl<'a> Drop for PooledBuffer<'a> {
    fn drop(&mut self) {
        if let Some(pool_index) = self.pool_index {
            if self.buffer.capacity() <= self.pool.sizes[pool_index] * 2 {
                // Return to pool if not too oversized
                let mut pool = self.pool.pools[pool_index].lock().unwrap();
                if pool.len() < 16 { // Limit pool size
                    pool.push(std::mem::take(&mut self.buffer));
                }
            }
        }
    }
}
```

### Memory Monitoring

```rust
#[cfg(target_os = "linux")]
pub mod memory_monitor {
    use std::fs;
    use std::time::{Duration, Instant};
    
    #[derive(Debug, Clone)]
    pub struct MemoryStats {
        pub rss_bytes: u64,
        pub vms_bytes: u64,
        pub peak_rss_bytes: u64,
        pub timestamp: Instant,
    }
    
    pub fn get_memory_stats() -> Result<MemoryStats, Box<dyn std::error::Error>> {
        let status = fs::read_to_string("/proc/self/status")?;
        
        let mut rss_bytes = 0;
        let mut vms_bytes = 0;
        let mut peak_rss_bytes = 0;
        
        for line in status.lines() {
            if let Some(value) = line.strip_prefix("VmRSS:") {
                rss_bytes = parse_memory_line(value)? * 1024;
            } else if let Some(value) = line.strip_prefix("VmSize:") {
                vms_bytes = parse_memory_line(value)? * 1024;
            } else if let Some(value) = line.strip_prefix("VmHWM:") {
                peak_rss_bytes = parse_memory_line(value)? * 1024;
            }
        }
        
        Ok(MemoryStats {
            rss_bytes,
            vms_bytes,
            peak_rss_bytes,
            timestamp: Instant::now(),
        })
    }
    
    fn parse_memory_line(line: &str) -> Result<u64, Box<dyn std::error::Error>> {
        let kb_str = line.trim().split_whitespace().next()
            .ok_or("Invalid memory line format")?;
        Ok(kb_str.parse::<u64>()?)
    }
    
    pub struct MemoryMonitor {
        last_stats: Option<MemoryStats>,
        alert_threshold_mb: u64,
    }
    
    impl MemoryMonitor {
        pub fn new(alert_threshold_mb: u64) -> Self {
            Self {
                last_stats: None,
                alert_threshold_mb,
            }
        }
        
        pub fn check_memory(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            let current_stats = get_memory_stats()?;
            
            // Check if memory usage exceeds threshold
            let rss_mb = current_stats.rss_bytes / 1024 / 1024;
            if rss_mb > self.alert_threshold_mb {
                tracing::warn!(
                    "High memory usage: {}MB (threshold: {}MB)",
                    rss_mb,
                    self.alert_threshold_mb
                );
            }
            
            // Check for memory leaks (growing trend)
            if let Some(ref last_stats) = self.last_stats {
                let time_diff = current_stats.timestamp.duration_since(last_stats.timestamp);
                let memory_diff = current_stats.rss_bytes as i64 - last_stats.rss_bytes as i64;
                
                if time_diff > Duration::from_secs(60) && memory_diff > 10 * 1024 * 1024 {
                    tracing::warn!(
                        "Potential memory leak detected: {}MB increase in {:?}",
                        memory_diff / 1024 / 1024,
                        time_diff
                    );
                }
            }
            
            self.last_stats = Some(current_stats);
            Ok(())
        }
    }
}
```

---

## Database Performance

### Connection Pool Optimization

```rust
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::time::Duration;

pub async fn create_optimized_database_pool(
    database_url: &str,
    max_connections: u32,
) -> Result<Pool<Postgres>, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(max_connections / 4)  // Keep 25% as minimum
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(600)) // 10 minutes
        .max_lifetime(Duration::from_secs(1800)) // 30 minutes
        .test_before_acquire(true)
        .connect(database_url)
        .await
}

// Database performance monitoring
pub struct DatabaseMetrics {
    pool: Pool<Postgres>,
    query_times: Arc<RwLock<HashMap<String, Vec<Duration>>>>,
}

impl DatabaseMetrics {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            pool,
            query_times: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn execute_with_metrics<T>(
        &self,
        query_name: &str,
        query: impl FnOnce() -> T,
    ) -> T {
        let start = Instant::now();
        let result = query();
        let duration = start.elapsed();
        
        // Record query time
        {
            let mut times = self.query_times.write().await;
            times.entry(query_name.to_string())
                .or_insert_with(Vec::new)
                .push(duration);
            
            // Keep only recent measurements
            for times in times.values_mut() {
                if times.len() > 1000 {
                    times.drain(0..500); // Keep most recent 500
                }
            }
        }
        
        // Log slow queries
        if duration > Duration::from_millis(100) {
            tracing::warn!(
                "Slow database query '{}' took {:?}",
                query_name,
                duration
            );
        }
        
        result
    }
    
    pub async fn get_pool_stats(&self) -> PoolStats {
        PoolStats {
            total_connections: self.pool.size(),
            idle_connections: self.pool.num_idle(),
            active_connections: self.pool.size() - self.pool.num_idle(),
        }
    }
}

#[derive(Debug)]
pub struct PoolStats {
    pub total_connections: u32,
    pub idle_connections: usize,
    pub active_connections: usize,
}
```

### Query Optimization

```rust
// Efficient query patterns
impl DatabaseServer {
    // ✅ Use prepared statements with parameters
    pub async fn search_users_optimized(
        &self,
        search_term: &str,
        limit: i32,
    ) -> Result<Vec<User>, DatabaseError> {
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, created_at
            FROM users 
            WHERE username ILIKE $1 OR email ILIKE $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            format!("%{}%", search_term),
            limit
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(users)
    }
    
    // ✅ Batch operations for better performance
    pub async fn insert_users_batch(
        &self,
        users: Vec<NewUser>,
    ) -> Result<Vec<i64>, DatabaseError> {
        let mut tx = self.pool.begin().await?;
        let mut user_ids = Vec::new();
        
        for user in users {
            let id = sqlx::query!(
                "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING id",
                user.username,
                user.email
            )
            .fetch_one(&mut *tx)
            .await?
            .id;
            
            user_ids.push(id);
        }
        
        tx.commit().await?;
        Ok(user_ids)
    }
    
    // ✅ Use EXPLAIN ANALYZE for query optimization
    pub async fn analyze_query_performance(
        &self,
        query: &str,
    ) -> Result<String, DatabaseError> {
        let explain_query = format!("EXPLAIN (ANALYZE, BUFFERS) {}", query);
        
        let rows = sqlx::query(&explain_query)
            .fetch_all(&self.pool)
            .await?;
        
        let plan = rows
            .into_iter()
            .map(|row| row.get::<String, _>(0))
            .collect::<Vec<_>>()
            .join("\n");
        
        Ok(plan)
    }
}
```

### Transaction Management

```rust
use sqlx::{Transaction, Postgres};

pub struct TransactionManager {
    pool: Pool<Postgres>,
}

impl TransactionManager {
    pub async fn execute_in_transaction<T, F, Fut>(
        &self,
        operation: F,
    ) -> Result<T, DatabaseError>
    where
        F: FnOnce(Transaction<'_, Postgres>) -> Fut,
        Fut: std::future::Future<Output = Result<T, DatabaseError>>,
    {
        let mut tx = self.pool.begin().await?;
        
        match operation(tx).await {
            Ok(result) => {
                // Transaction succeeds, commit
                if let Err(e) = tx.commit().await {
                    tracing::error!("Failed to commit transaction: {}", e);
                    return Err(DatabaseError::TransactionFailed(e.to_string()));
                }
                Ok(result)
            }
            Err(e) => {
                // Transaction fails, rollback
                if let Err(rollback_error) = tx.rollback().await {
                    tracing::error!("Failed to rollback transaction: {}", rollback_error);
                }
                Err(e)
            }
        }
    }
    
    // Optimistic locking pattern
    pub async fn update_with_version(
        &self,
        id: i64,
        expected_version: i32,
        update_data: UpdateData,
    ) -> Result<bool, DatabaseError> {
        let result = sqlx::query!(
            r#"
            UPDATE documents 
            SET content = $1, version = version + 1, updated_at = NOW()
            WHERE id = $2 AND version = $3
            "#,
            update_data.content,
            id,
            expected_version
        )
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }
}
```

---

## Caching Strategies

### Multi-Level Caching

```rust
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

pub struct MultiLevelCache {
    // L1: In-memory LRU cache (fastest)
    l1_cache: Arc<RwLock<LruCache<String, CachedData>>>,
    
    // L2: Redis cache (fast, shared)
    redis_client: redis::Client,
    
    // L3: Database cache (persistent)
    database_pool: Pool<Postgres>,
}

impl MultiLevelCache {
    pub async fn get<T>(&self, key: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned + Clone,
    {
        // Try L1 cache first
        {
            let cache = self.l1_cache.read().await;
            if let Some(data) = cache.peek(key) {
                if !data.is_expired() {
                    if let Ok(value) = serde_json::from_value(data.value.clone()) {
                        return Some(value);
                    }
                }
            }
        }
        
        // Try L2 cache (Redis)
        if let Ok(mut conn) = self.redis_client.get_connection() {
            if let Ok(data) = conn.get::<_, String>(key) {
                if let Ok(value) = serde_json::from_str::<T>(&data) {
                    // Promote to L1 cache
                    self.set_l1_cache(key, &value, Duration::from_secs(300)).await;
                    return Some(value);
                }
            }
        }
        
        // Try L3 cache (Database)
        if let Ok(data) = self.get_from_database_cache(key).await {
            if let Ok(value) = serde_json::from_str::<T>(&data) {
                // Promote to L2 and L1 caches
                self.set_redis_cache(key, &value, Duration::from_secs(3600)).await;
                self.set_l1_cache(key, &value, Duration::from_secs(300)).await;
                return Some(value);
            }
        }
        
        None
    }
    
    pub async fn set<T>(&self, key: &str, value: &T, ttl: Duration)
    where
        T: serde::Serialize + Clone,
    {
        // Set in all cache levels
        tokio::join!(
            self.set_l1_cache(key, value, ttl),
            self.set_redis_cache(key, value, ttl),
            self.set_database_cache(key, value, ttl)
        );
    }
    
    async fn set_l1_cache<T>(&self, key: &str, value: &T, ttl: Duration)
    where
        T: serde::Serialize,
    {
        if let Ok(json_value) = serde_json::to_value(value) {
            let mut cache = self.l1_cache.write().await;
            cache.put(key.to_string(), CachedData {
                value: json_value,
                created_at: Instant::now(),
                ttl,
            });
        }
    }
    
    async fn set_redis_cache<T>(&self, key: &str, value: &T, ttl: Duration)
    where
        T: serde::Serialize,
    {
        if let Ok(mut conn) = self.redis_client.get_connection() {
            if let Ok(serialized) = serde_json::to_string(value) {
                let _ = conn.set_ex::<_, _, ()>(key, serialized, ttl.as_secs());
            }
        }
    }
    
    async fn set_database_cache<T>(&self, key: &str, value: &T, ttl: Duration)
    where
        T: serde::Serialize,
    {
        if let Ok(serialized) = serde_json::to_string(value) {
            let expires_at = chrono::Utc::now() + chrono::Duration::from_std(ttl).unwrap();
            
            let _ = sqlx::query!(
                r#"
                INSERT INTO cache_entries (key, value, expires_at)
                VALUES ($1, $2, $3)
                ON CONFLICT (key) DO UPDATE SET
                    value = EXCLUDED.value,
                    expires_at = EXCLUDED.expires_at
                "#,
                key,
                serialized,
                expires_at
            )
            .execute(&self.database_pool)
            .await;
        }
    }
}
```

### Cache Warming and Preloading

```rust
pub struct CacheWarmer {
    cache: Arc<MultiLevelCache>,
    background_tasks: JoinSet<()>,
}

impl CacheWarmer {
    pub fn new(cache: Arc<MultiLevelCache>) -> Self {
        Self {
            cache,
            background_tasks: JoinSet::new(),
        }
    }
    
    pub async fn start_warming_tasks(&mut self) {
        // Warm frequently accessed data
        self.background_tasks.spawn({
            let cache = self.cache.clone();
            async move {
                loop {
                    Self::warm_popular_content(&cache).await;
                    tokio::time::sleep(Duration::from_secs(300)).await; // Every 5 minutes
                }
            }
        });
        
        // Preload user-specific data
        self.background_tasks.spawn({
            let cache = self.cache.clone();
            async move {
                loop {
                    Self::preload_user_data(&cache).await;
                    tokio::time::sleep(Duration::from_secs(600)).await; // Every 10 minutes
                }
            }
        });
    }
    
    async fn warm_popular_content(cache: &MultiLevelCache) {
        // Get list of popular content that should be cached
        let popular_items = get_popular_content_ids().await;
        
        for item_id in popular_items {
            let cache_key = format!("content:{}", item_id);
            
            // Check if already cached
            if cache.get::<Content>(&cache_key).await.is_none() {
                // Load and cache the content
                if let Ok(content) = load_content_from_source(&item_id).await {
                    cache.set(&cache_key, &content, Duration::from_secs(3600)).await;
                }
            }
        }
    }
    
    async fn preload_user_data(cache: &MultiLevelCache) {
        // Preload data for recently active users
        let active_users = get_recently_active_users().await;
        
        for user_id in active_users {
            let preferences_key = format!("user:{}:preferences", user_id);
            
            if cache.get::<UserPreferences>(&preferences_key).await.is_none() {
                if let Ok(preferences) = load_user_preferences(&user_id).await {
                    cache.set(&preferences_key, &preferences, Duration::from_secs(1800)).await;
                }
            }
        }
    }
}

async fn get_popular_content_ids() -> Vec<String> {
    // Implementation to get popular content IDs
    // This could be from analytics, view counts, etc.
    vec![]
}

async fn get_recently_active_users() -> Vec<String> {
    // Implementation to get recently active user IDs
    vec![]
}
```

### Cache Invalidation Strategies

```rust
pub struct CacheInvalidationManager {
    cache: Arc<MultiLevelCache>,
    invalidation_patterns: HashMap<String, Vec<String>>,
}

impl CacheInvalidationManager {
    pub fn new(cache: Arc<MultiLevelCache>) -> Self {
        let mut invalidation_patterns = HashMap::new();
        
        // Define invalidation patterns
        invalidation_patterns.insert(
            "user_updated".to_string(),
            vec![
                "user:{user_id}:profile".to_string(),
                "user:{user_id}:preferences".to_string(),
                "user:{user_id}:permissions".to_string(),
            ],
        );
        
        invalidation_patterns.insert(
            "content_updated".to_string(),
            vec![
                "content:{content_id}".to_string(),
                "content:{content_id}:metadata".to_string(),
                "search:*".to_string(), // Invalidate all search results
            ],
        );
        
        Self {
            cache,
            invalidation_patterns,
        }
    }
    
    pub async fn invalidate(&self, event: &str, context: &HashMap<String, String>) {
        if let Some(patterns) = self.invalidation_patterns.get(event) {
            for pattern in patterns {
                let cache_key = self.substitute_pattern(pattern, context);
                
                if cache_key.contains('*') {
                    // Pattern invalidation (e.g., "search:*")
                    self.invalidate_pattern(&cache_key).await;
                } else {
                    // Exact key invalidation
                    self.cache.delete(&cache_key).await;
                }
            }
        }
    }
    
    fn substitute_pattern(&self, pattern: &str, context: &HashMap<String, String>) -> String {
        let mut result = pattern.to_string();
        
        for (key, value) in context {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        
        result
    }
    
    async fn invalidate_pattern(&self, pattern: &str) {
        // For patterns with wildcards, we need to find matching keys
        // This is expensive, so use sparingly
        
        // Get all keys from Redis matching pattern
        if let Ok(mut conn) = self.cache.redis_client.get_connection() {
            if let Ok(keys) = conn.keys::<_, Vec<String>>(pattern) {
                for key in keys {
                    self.cache.delete(&key).await;
                }
            }
        }
    }
}

// Usage in MCP tools
#[tool]
async fn update_user_profile(
    &self,
    user_id: String,
    profile_data: UserProfile,
) -> Result<UserProfile, ServerError> {
    // Update user profile
    let updated_profile = self.database.update_user_profile(&user_id, profile_data).await?;
    
    // Invalidate related caches
    let mut context = HashMap::new();
    context.insert("user_id".to_string(), user_id);
    
    self.cache_invalidation
        .invalidate("user_updated", &context)
        .await;
    
    Ok(updated_profile)
}
```

---

## Network Optimization

### HTTP Client Optimization

```rust
use reqwest::{Client, ClientBuilder};
use std::time::Duration;

pub fn create_optimized_http_client() -> Result<Client, reqwest::Error> {
    ClientBuilder::new()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .pool_max_idle_per_host(20)
        .pool_idle_timeout(Duration::from_secs(90))
        .tcp_keepalive(Duration::from_secs(60))
        .tcp_nodelay(true)
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .user_agent("MCP-Server/1.0")
        .build()
}

// Connection pool monitoring
pub struct HttpClientMetrics {
    client: Client,
    request_metrics: Arc<RwLock<HashMap<String, RequestMetrics>>>,
}

#[derive(Default)]
struct RequestMetrics {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    total_duration: Duration,
    min_duration: Duration,
    max_duration: Duration,
}

impl HttpClientMetrics {
    pub async fn execute_request<T>(
        &self,
        name: &str,
        request: impl Future<Output = Result<T, reqwest::Error>>,
    ) -> Result<T, reqwest::Error> {
        let start = Instant::now();
        let result = request.await;
        let duration = start.elapsed();
        
        // Update metrics
        {
            let mut metrics = self.request_metrics.write().await;
            let request_metrics = metrics.entry(name.to_string()).or_default();
            
            request_metrics.total_requests += 1;
            request_metrics.total_duration += duration;
            
            if request_metrics.min_duration == Duration::ZERO || duration < request_metrics.min_duration {
                request_metrics.min_duration = duration;
            }
            if duration > request_metrics.max_duration {
                request_metrics.max_duration = duration;
            }
            
            match &result {
                Ok(_) => request_metrics.successful_requests += 1,
                Err(_) => request_metrics.failed_requests += 1,
            }
        }
        
        result
    }
    
    pub async fn get_metrics(&self, name: &str) -> Option<RequestMetrics> {
        let metrics = self.request_metrics.read().await;
        metrics.get(name).cloned()
    }
}
```

### Request Batching and Compression

```rust
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;

pub struct BatchRequestManager {
    pending_requests: Arc<RwLock<HashMap<String, Vec<PendingRequest>>>>,
    batch_size: usize,
    batch_timeout: Duration,
}

struct PendingRequest {
    data: serde_json::Value,
    response_sender: oneshot::Sender<Result<serde_json::Value, BatchError>>,
    created_at: Instant,
}

impl BatchRequestManager {
    pub fn new(batch_size: usize, batch_timeout: Duration) -> Self {
        Self {
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            batch_size,
            batch_timeout,
        }
    }
    
    pub async fn submit_request(
        &self,
        endpoint: &str,
        data: serde_json::Value,
    ) -> Result<serde_json::Value, BatchError> {
        let (tx, rx) = oneshot::channel();
        
        let request = PendingRequest {
            data,
            response_sender: tx,
            created_at: Instant::now(),
        };
        
        // Add to pending requests
        {
            let mut pending = self.pending_requests.write().await;
            let endpoint_requests = pending.entry(endpoint.to_string()).or_insert_with(Vec::new);
            endpoint_requests.push(request);
            
            // Check if we should flush this batch
            if endpoint_requests.len() >= self.batch_size {
                let batch = std::mem::take(endpoint_requests);
                tokio::spawn({
                    let endpoint = endpoint.to_string();
                    async move {
                        Self::process_batch(&endpoint, batch).await;
                    }
                });
            }
        }
        
        // Wait for response
        rx.await.map_err(|_| BatchError::Cancelled)?
    }
    
    async fn process_batch(endpoint: &str, requests: Vec<PendingRequest>) {
        // Compress batch data
        let batch_data: Vec<_> = requests.iter().map(|r| &r.data).collect();
        let compressed_data = Self::compress_data(&batch_data).unwrap_or_default();
        
        // Send batch request
        let client = reqwest::Client::new();
        let response = client
            .post(endpoint)
            .header("Content-Encoding", "gzip")
            .header("Content-Type", "application/json")
            .body(compressed_data)
            .send()
            .await;
        
        match response {
            Ok(resp) if resp.status().is_success() => {
                // Parse batch response
                if let Ok(batch_response) = resp.json::<Vec<serde_json::Value>>().await {
                    // Send individual responses
                    for (request, response) in requests.into_iter().zip(batch_response) {
                        let _ = request.response_sender.send(Ok(response));
                    }
                } else {
                    // Send error to all requests
                    for request in requests {
                        let _ = request.response_sender.send(Err(BatchError::ParseError));
                    }
                }
            }
            Ok(resp) => {
                // HTTP error
                for request in requests {
                    let _ = request.response_sender.send(Err(BatchError::HttpError(resp.status())));
                }
            }
            Err(e) => {
                // Network error
                for request in requests {
                    let _ = request.response_sender.send(Err(BatchError::NetworkError(e.to_string())));
                }
            }
        }
    }
    
    fn compress_data(data: &[&serde_json::Value]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let json_data = serde_json::to_vec(data)?;
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&json_data)?;
        let compressed = encoder.finish()?;
        
        Ok(compressed)
    }
    
    // Background task to flush timed-out batches
    pub async fn start_timeout_flusher(&self) {
        let pending_requests = self.pending_requests.clone();
        let batch_timeout = self.batch_timeout;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            
            loop {
                interval.tick().await;
                
                let mut to_flush = Vec::new();
                
                // Check for timed-out batches
                {
                    let mut pending = pending_requests.write().await;
                    let now = Instant::now();
                    
                    for (endpoint, requests) in pending.iter_mut() {
                        if let Some(oldest) = requests.first() {
                            if now.duration_since(oldest.created_at) >= batch_timeout {
                                to_flush.push((endpoint.clone(), std::mem::take(requests)));
                            }
                        }
                    }
                }
                
                // Process timed-out batches
                for (endpoint, batch) in to_flush {
                    tokio::spawn(async move {
                        Self::process_batch(&endpoint, batch).await;
                    });
                }
            }
        });
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BatchError {
    #[error("Request was cancelled")]
    Cancelled,
    #[error("Failed to parse response")]
    ParseError,
    #[error("HTTP error: {0}")]
    HttpError(reqwest::StatusCode),
    #[error("Network error: {0}")]
    NetworkError(String),
}
```

---

## Resource Management

### Resource Pool Pattern

```rust
use tokio::sync::Semaphore;

pub struct ResourcePool<T> {
    resources: Arc<Mutex<Vec<T>>>,
    semaphore: Arc<Semaphore>,
    factory: Box<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
}

impl<T> ResourcePool<T>
where
    T: Send + 'static,
{
    pub fn new<F>(max_size: usize, factory: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            resources: Arc::new(Mutex::new(Vec::new())),
            semaphore: Arc::new(Semaphore::new(max_size)),
            factory: Box::new(factory),
            max_size,
        }
    }
    
    pub async fn acquire(&self) -> Result<PooledResource<T>, ResourceError> {
        // Acquire semaphore permit
        let permit = self.semaphore.acquire().await.map_err(|_| ResourceError::PoolClosed)?;
        
        // Try to get resource from pool
        let resource = {
            let mut pool = self.resources.lock().await;
            pool.pop()
        };
        
        let resource = match resource {
            Some(resource) => resource,
            None => {
                // Create new resource
                (self.factory)()
            }
        };
        
        Ok(PooledResource {
            resource: Some(resource),
            pool: self.resources.clone(),
            _permit: permit,
        })
    }
    
    pub async fn size(&self) -> usize {
        self.resources.lock().await.len()
    }
}

pub struct PooledResource<T> {
    resource: Option<T>,
    pool: Arc<Mutex<Vec<T>>>,
    _permit: tokio::sync::SemaphorePermit<'_>,
}

impl<T> std::ops::Deref for PooledResource<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        self.resource.as_ref().unwrap()
    }
}

impl<T> std::ops::DerefMut for PooledResource<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.resource.as_mut().unwrap()
    }
}

impl<T> Drop for PooledResource<T> {
    fn drop(&mut self) {
        if let Some(resource) = self.resource.take() {
            // Return resource to pool
            if let Ok(mut pool) = self.pool.try_lock() {
                pool.push(resource);
            }
            // If pool is locked, resource is simply dropped
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    #[error("Resource pool is closed")]
    PoolClosed,
    #[error("Resource acquisition timeout")]
    Timeout,
}
```

### CPU Intensive Task Management

```rust
use tokio::task;
use rayon::prelude::*;

pub struct CpuTaskManager {
    thread_pool: rayon::ThreadPool,
    semaphore: Arc<Semaphore>,
}

impl CpuTaskManager {
    pub fn new(max_concurrent_tasks: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let num_threads = num_cpus::get();
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .thread_name(|index| format!("cpu-worker-{}", index))
            .build()?;
        
        Ok(Self {
            thread_pool,
            semaphore: Arc::new(Semaphore::new(max_concurrent_tasks)),
        })
    }
    
    pub async fn execute_cpu_intensive<T, F>(&self, task: F) -> Result<T, TaskError>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        // Acquire semaphore to limit concurrent CPU tasks
        let _permit = self.semaphore.acquire().await.map_err(|_| TaskError::PoolClosed)?;
        
        // Execute on dedicated thread pool
        let (tx, rx) = oneshot::channel();
        
        self.thread_pool.spawn(move || {
            let result = task();
            let _ = tx.send(result);
        });
        
        rx.await.map_err(|_| TaskError::TaskCancelled)
    }
    
    pub async fn parallel_map<T, U, F>(&self, items: Vec<T>, map_fn: F) -> Result<Vec<U>, TaskError>
    where
        T: Send + 'static,
        U: Send + 'static,
        F: Fn(T) -> U + Send + Sync + 'static,
    {
        let _permit = self.semaphore.acquire().await.map_err(|_| TaskError::PoolClosed)?;
        
        let map_fn = Arc::new(map_fn);
        let (tx, rx) = oneshot::channel();
        
        self.thread_pool.spawn(move || {
            let results: Vec<U> = items.into_par_iter().map(|item| map_fn(item)).collect();
            let _ = tx.send(results);
        });
        
        rx.await.map_err(|_| TaskError::TaskCancelled)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("Task pool is closed")]
    PoolClosed,
    #[error("Task was cancelled")]
    TaskCancelled,
}

// Usage in MCP tools
#[tool]
async fn process_large_dataset(&self, data: Vec<DataItem>) -> Result<ProcessedData, ServerError> {
    // Use CPU task manager for intensive processing
    let processed_items = self.cpu_task_manager
        .parallel_map(data, |item| {
            // CPU-intensive processing
            expensive_computation(item)
        })
        .await
        .map_err(|e| ServerError::ProcessingError(e.to_string()))?;
    
    Ok(ProcessedData { items: processed_items })
}
```

---

## Profiling and Debugging

### Performance Profiling

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct PerformanceProfiler {
    spans: Arc<RwLock<HashMap<String, SpanData>>>,
    active_spans: Arc<RwLock<HashMap<u64, ActiveSpan>>>,
}

struct SpanData {
    name: String,
    total_duration: Duration,
    count: u64,
    min_duration: Duration,
    max_duration: Duration,
}

struct ActiveSpan {
    name: String,
    start_time: Instant,
    span_id: u64,
}

impl PerformanceProfiler {
    pub async fn start_span(&self, name: &str) -> SpanGuard {
        let span_id = self.generate_span_id();
        let start_time = Instant::now();
        
        {
            let mut active = self.active_spans.write().await;
            active.insert(span_id, ActiveSpan {
                name: name.to_string(),
                start_time,
                span_id,
            });
        }
        
        SpanGuard {
            span_id,
            profiler: self,
        }
    }
    
    async fn end_span(&self, span_id: u64) {
        let span_data = {
            let mut active = self.active_spans.write().await;
            active.remove(&span_id)
        };
        
        if let Some(span) = span_data {
            let duration = span.start_time.elapsed();
            
            let mut spans = self.spans.write().await;
            let entry = spans.entry(span.name).or_insert_with(|| SpanData {
                name: span.name.clone(),
                total_duration: Duration::ZERO,
                count: 0,
                min_duration: Duration::MAX,
                max_duration: Duration::ZERO,
            });
            
            entry.total_duration += duration;
            entry.count += 1;
            entry.min_duration = entry.min_duration.min(duration);
            entry.max_duration = entry.max_duration.max(duration);
        }
    }
    
    pub async fn get_profile_report(&self) -> ProfileReport {
        let spans = self.spans.read().await;
        let mut report_entries = Vec::new();
        
        for span_data in spans.values() {
            let avg_duration = if span_data.count > 0 {
                span_data.total_duration / span_data.count as u32
            } else {
                Duration::ZERO
            };
            
            report_entries.push(ProfileReportEntry {
                name: span_data.name.clone(),
                total_duration: span_data.total_duration,
                count: span_data.count,
                avg_duration,
                min_duration: span_data.min_duration,
                max_duration: span_data.max_duration,
            });
        }
        
        // Sort by total duration descending
        report_entries.sort_by(|a, b| b.total_duration.cmp(&a.total_duration));
        
        ProfileReport { entries: report_entries }
    }
    
    fn generate_span_id(&self) -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }
}

pub struct SpanGuard<'a> {
    span_id: u64,
    profiler: &'a PerformanceProfiler,
}

impl<'a> Drop for SpanGuard<'a> {
    fn drop(&mut self) {
        // Note: This blocks, but spans should be short-lived
        let profiler = self.profiler;
        let span_id = self.span_id;
        tokio::spawn(async move {
            profiler.end_span(span_id).await;
        });
    }
}

#[derive(Debug)]
pub struct ProfileReport {
    pub entries: Vec<ProfileReportEntry>,
}

#[derive(Debug)]
pub struct ProfileReportEntry {
    pub name: String,
    pub total_duration: Duration,
    pub count: u64,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
}

// Usage in MCP tools