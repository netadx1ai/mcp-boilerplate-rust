//! Content Generation Pipeline Example
//! 
//! This example demonstrates coordinating multiple MCP servers to create a complete
//! content generation workflow: News Data → Template Rendering → Database Storage → Analytics Tracking
//! 
//! ## Workflow
//! 1. Fetch trending news from news-data-server
//! 2. Generate article content using template-server
//! 3. Store the content in database-server
//! 4. Track performance metrics with analytics-server
//! 
//! ## Features
//! - Error handling and graceful degradation
//! - Performance monitoring and optimization
//! - Parallel processing where possible
//! - Circuit breaker pattern for resilience
//! - Comprehensive logging and tracing

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use futures::future::{join_all, try_join_all};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

/// Pipeline execution errors
#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("News server error: {0}")]
    NewsServer(String),
    
    #[error("Template server error: {0}")]
    TemplateServer(String),
    
    #[error("Database server error: {0}")]
    DatabaseServer(String),
    
    #[error("Analytics server error: {0}")]
    AnalyticsServer(String),
    
    #[error("Pipeline configuration error: {0}")]
    Configuration(String),
    
    #[error("Pipeline timeout after {timeout:?}")]
    Timeout { timeout: Duration },
    
    #[error("All servers failed: primary={primary}, fallback={fallback}")]
    AllServersFailed { primary: String, fallback: String },
    
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
}

/// News item from news-data-server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub content: String,
    pub category: String,
    pub published_at: DateTime<Utc>,
    pub source: String,
    pub url: String,
    pub engagement_score: f64,
}

/// Generated article content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedArticle {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub template_id: String,
    pub source_news_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub word_count: usize,
    pub estimated_read_time: u32, // minutes
}

/// Database storage result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageResult {
    pub article_id: Uuid,
    pub database_id: i64,
    pub stored_at: DateTime<Utc>,
    pub storage_location: String,
}

/// Analytics tracking result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsResult {
    pub tracking_id: Uuid,
    pub article_id: Uuid,
    pub metrics_recorded: Vec<String>,
    pub performance_score: f64,
    pub tracked_at: DateTime<Utc>,
}

/// Complete pipeline execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub execution_id: Uuid,
    pub article: GeneratedArticle,
    pub storage: StorageResult,
    pub analytics: AnalyticsResult,
    pub execution_time: Duration,
    pub success: bool,
    pub errors: Vec<String>,
}

/// Pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub news_server_url: String,
    pub template_server_url: String,
    pub database_server_url: String,
    pub analytics_server_url: String,
    pub max_news_items: usize,
    pub timeout_seconds: u64,
    pub retry_attempts: usize,
    pub enable_caching: bool,
    pub enable_parallel_processing: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            news_server_url: "http://localhost:3001".to_string(),
            template_server_url: "http://localhost:3002".to_string(),
            database_server_url: "http://localhost:3003".to_string(),
            analytics_server_url: "http://localhost:3004".to_string(),
            max_news_items: 5,
            timeout_seconds: 30,
            retry_attempts: 3,
            enable_caching: true,
            enable_parallel_processing: true,
        }
    }
}

/// Pipeline statistics
#[derive(Debug, Default)]
pub struct PipelineStats {
    pub total_executions: AtomicU64,
    pub successful_executions: AtomicU64,
    pub failed_executions: AtomicU64,
    pub total_execution_time: AtomicU64, // milliseconds
    pub news_fetch_time: AtomicU64,
    pub template_render_time: AtomicU64,
    pub database_store_time: AtomicU64,
    pub analytics_track_time: AtomicU64,
}

impl PipelineStats {
    pub fn record_execution(&self, result: &PipelineResult) {
        self.total_executions.fetch_add(1, Ordering::Relaxed);
        
        if result.success {
            self.successful_executions.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_executions.fetch_add(1, Ordering::Relaxed);
        }
        
        self.total_execution_time.fetch_add(
            result.execution_time.as_millis() as u64,
            Ordering::Relaxed,
        );
    }
    
    pub fn success_rate(&self) -> f64 {
        let total = self.total_executions.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        
        let successful = self.successful_executions.load(Ordering::Relaxed);
        successful as f64 / total as f64
    }
    
    pub fn average_execution_time(&self) -> Duration {
        let total = self.total_executions.load(Ordering::Relaxed);
        if total == 0 {
            return Duration::from_secs(0);
        }
        
        let total_time = self.total_execution_time.load(Ordering::Relaxed);
        Duration::from_millis(total_time / total)
    }
}

/// Circuit breaker for server resilience
#[derive(Debug)]
pub struct CircuitBreaker {
    failure_count: AtomicU32,
    last_failure: RwLock<Option<Instant>>,
    failure_threshold: u32,
    recovery_timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            failure_count: AtomicU32::new(0),
            last_failure: RwLock::new(None),
            failure_threshold,
            recovery_timeout,
        }
    }
    
    pub async fn is_open(&self) -> bool {
        let failure_count = self.failure_count.load(Ordering::Relaxed);
        if failure_count < self.failure_threshold {
            return false;
        }
        
        let last_failure = self.last_failure.read().await;
        match *last_failure {
            Some(time) => time.elapsed() < self.recovery_timeout,
            None => false,
        }
    }
    
    pub async fn record_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        *self.last_failure.write().await = None;
    }
    
    pub async fn record_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
        *self.last_failure.write().await = Some(Instant::now());
    }
}

/// Content generation pipeline orchestrator
pub struct ContentPipeline {
    config: PipelineConfig,
    client: Client,
    stats: Arc<PipelineStats>,
    cache: Arc<DashMap<String, (NewsItem, Instant)>>,
    news_circuit_breaker: Arc<CircuitBreaker>,
    template_circuit_breaker: Arc<CircuitBreaker>,
    database_circuit_breaker: Arc<CircuitBreaker>,
    analytics_circuit_breaker: Arc<CircuitBreaker>,
}

impl ContentPipeline {
    pub fn new(config: PipelineConfig) -> Self {
        let timeout = Duration::from_secs(config.timeout_seconds);
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            config,
            client,
            stats: Arc::new(PipelineStats::default()),
            cache: Arc::new(DashMap::new()),
            news_circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(60))),
            template_circuit_breaker: Arc::new(CircuitBreaker::new(3, Duration::from_secs(30))),
            database_circuit_breaker: Arc::new(CircuitBreaker::new(3, Duration::from_secs(30))),
            analytics_circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(60))),
        }
    }
    
    /// Execute the complete content generation pipeline
    #[instrument(skip(self), fields(execution_id = %Uuid::new_v4()))]
    pub async fn execute_pipeline(&self, query: &str, category: &str) -> Result<PipelineResult> {
        let execution_id = Uuid::new_v4();
        let start_time = Instant::now();
        let mut errors = Vec::new();
        
        info!("Starting content pipeline execution for query: {}", query);
        
        // Step 1: Fetch news data
        let news_items = match self.fetch_news_data(query, category).await {
            Ok(items) => items,
            Err(e) => {
                error!("Failed to fetch news data: {}", e);
                errors.push(format!("News fetch failed: {}", e));
                
                // Try fallback to cached data
                match self.get_cached_news(query).await {
                    Ok(cached_items) => {
                        warn!("Using cached news data as fallback");
                        cached_items
                    }
                    Err(cache_error) => {
                        errors.push(format!("Cache fallback failed: {}", cache_error));
                        return Ok(PipelineResult {
                            execution_id,
                            article: self.create_error_article(&errors).await,
                            storage: self.create_error_storage().await,
                            analytics: self.create_error_analytics().await,
                            execution_time: start_time.elapsed(),
                            success: false,
                            errors,
                        });
                    }
                }
            }
        };
        
        // Step 2: Generate article content
        let article = match self.generate_article_content(&news_items, category).await {
            Ok(article) => article,
            Err(e) => {
                error!("Failed to generate article content: {}", e);
                errors.push(format!("Content generation failed: {}", e));
                return Ok(PipelineResult {
                    execution_id,
                    article: self.create_error_article(&errors).await,
                    storage: self.create_error_storage().await,
                    analytics: self.create_error_analytics().await,
                    execution_time: start_time.elapsed(),
                    success: false,
                    errors,
                });
            }
        };
        
        // Step 3 & 4: Store in database and track analytics (parallel)
        let (storage_result, analytics_result) = if self.config.enable_parallel_processing {
            // Execute storage and analytics in parallel for better performance
            let storage_future = self.store_article_content(&article);
            let analytics_future = self.track_article_analytics(&article);
            
            match tokio::try_join!(storage_future, analytics_future) {
                Ok((storage, analytics)) => (storage, analytics),
                Err(e) => {
                    error!("Failed parallel processing: {}", e);
                    errors.push(format!("Parallel processing failed: {}", e));
                    return Ok(PipelineResult {
                        execution_id,
                        article,
                        storage: self.create_error_storage().await,
                        analytics: self.create_error_analytics().await,
                        execution_time: start_time.elapsed(),
                        success: false,
                        errors,
                    });
                }
            }
        } else {
            // Sequential execution
            let storage = match self.store_article_content(&article).await {
                Ok(storage) => storage,
                Err(e) => {
                    error!("Failed to store article: {}", e);
                    errors.push(format!("Storage failed: {}", e));
                    return Ok(PipelineResult {
                        execution_id,
                        article,
                        storage: self.create_error_storage().await,
                        analytics: self.create_error_analytics().await,
                        execution_time: start_time.elapsed(),
                        success: false,
                        errors,
                    });
                }
            };
            
            let analytics = match self.track_article_analytics(&article).await {
                Ok(analytics) => analytics,
                Err(e) => {
                    error!("Failed to track analytics: {}", e);
                    errors.push(format!("Analytics tracking failed: {}", e));
                    // Analytics failure is not critical, continue with warning
                    self.create_error_analytics().await
                }
            };
            
            (storage, analytics)
        };
        
        let execution_time = start_time.elapsed();
        let success = errors.is_empty();
        
        let result = PipelineResult {
            execution_id,
            article,
            storage: storage_result,
            analytics: analytics_result,
            execution_time,
            success,
            errors,
        };
        
        // Record statistics
        self.stats.record_execution(&result);
        
        if success {
            info!("Pipeline execution completed successfully in {:?}", execution_time);
        } else {
            warn!("Pipeline execution completed with errors in {:?}", execution_time);
        }
        
        Ok(result)
    }
    
    /// Fetch news data from news-data-server
    #[instrument(skip(self))]
    async fn fetch_news_data(&self, query: &str, category: &str) -> Result<Vec<NewsItem>, PipelineError> {
        let start_time = Instant::now();
        
        // Check circuit breaker
        if self.news_circuit_breaker.is_open().await {
            return Err(PipelineError::NewsServer("Circuit breaker open".to_string()));
        }
        
        // Check cache first if enabled
        if self.config.enable_caching {
            let cache_key = format!("{}:{}", query, category);
            if let Some((cached_item, cached_at)) = self.cache.get(&cache_key) {
                if cached_at.elapsed() < Duration::from_secs(300) { // 5 minute cache
                    debug!("Using cached news data for query: {}", query);
                    return Ok(vec![cached_item.clone()]);
                }
            }
        }
        
        let url = format!(
            "{}/search_news?query={}&category={}&limit={}",
            self.config.news_server_url,
            urlencoding::encode(query),
            urlencoding::encode(category),
            self.config.max_news_items
        );
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| PipelineError::NewsServer(e.to_string()))?;
        
        if !response.status().is_success() {
            self.news_circuit_breaker.record_failure().await;
            return Err(PipelineError::NewsServer(
                format!("HTTP {}: {}", response.status(), response.text().await.unwrap_or_default())
            ));
        }
        
        let news_items: Vec<NewsItem> = response
            .json()
            .await
            .map_err(|e| PipelineError::NewsServer(format!("Invalid JSON response: {}", e)))?;
        
        // Cache the results if enabled
        if self.config.enable_caching && !news_items.is_empty() {
            let cache_key = format!("{}:{}", query, category);
            self.cache.insert(cache_key, (news_items[0].clone(), Instant::now()));
        }
        
        self.news_circuit_breaker.record_success().await;
        self.stats.news_fetch_time.fetch_add(
            start_time.elapsed().as_millis() as u64,
            Ordering::Relaxed,
        );
        
        debug!("Fetched {} news items in {:?}", news_items.len(), start_time.elapsed());
        Ok(news_items)
    }
    
    /// Generate article content using template-server
    #[instrument(skip(self, news_items))]
    async fn generate_article_content(&self, news_items: &[NewsItem], category: &str) -> Result<GeneratedArticle, PipelineError> {
        let start_time = Instant::now();
        
        // Check circuit breaker
        if self.template_circuit_breaker.is_open().await {
            return Err(PipelineError::TemplateServer("Circuit breaker open".to_string()));
        }
        
        // Select appropriate template based on category
        let template_id = match category.to_lowercase().as_str() {
            "technology" | "tech" => "technical_article",
            "business" | "finance" => "business_article",
            "health" | "medical" => "health_article",
            _ => "general_article",
        };
        
        // Prepare template parameters
        let template_params = serde_json::json!({
            "title": format!("Latest {} News Roundup", category),
            "news_items": news_items,
            "generated_at": Utc::now().to_rfc3339(),
            "category": category,
            "summary": self.generate_summary(news_items).await,
        });
        
        let url = format!("{}/render_template", self.config.template_server_url);
        let request_body = serde_json::json!({
            "template_id": template_id,
            "parameters": template_params,
        });
        
        let response = self.client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| PipelineError::TemplateServer(e.to_string()))?;
        
        if !response.status().is_success() {
            self.template_circuit_breaker.record_failure().await;
            return Err(PipelineError::TemplateServer(
                format!("HTTP {}: {}", response.status(), response.text().await.unwrap_or_default())
            ));
        }
        
        let rendered_content: serde_json::Value = response
            .json()
            .await
            .map_err(|e| PipelineError::TemplateServer(format!("Invalid JSON response: {}", e)))?;
        
        let content = rendered_content
            .get("rendered_content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PipelineError::TemplateServer("Missing rendered_content field".to_string()))?;
        
        let word_count = content.split_whitespace().count();
        let estimated_read_time = (word_count / 200).max(1) as u32; // Assume 200 WPM reading speed
        
        let article = GeneratedArticle {
            id: Uuid::new_v4(),
            title: template_params["title"].as_str().unwrap_or("Untitled").to_string(),
            content: content.to_string(),
            template_id: template_id.to_string(),
            source_news_ids: news_items.iter().map(|item| item.id.clone()).collect(),
            created_at: Utc::now(),
            word_count,
            estimated_read_time,
        };
        
        self.template_circuit_breaker.record_success().await;
        self.stats.template_render_time.fetch_add(
            start_time.elapsed().as_millis() as u64,
            Ordering::Relaxed,
        );
        
        debug!("Generated article content in {:?}, {} words", start_time.elapsed(), word_count);
        Ok(article)
    }
    
    /// Store article content in database-server
    #[instrument(skip(self, article))]
    async fn store_article_content(&self, article: &GeneratedArticle) -> Result<StorageResult, PipelineError> {
        let start_time = Instant::now();
        
        // Check circuit breaker
        if self.database_circuit_breaker.is_open().await {
            return Err(PipelineError::DatabaseServer("Circuit breaker open".to_string()));
        }
        
        let url = format!("{}/execute_query", self.config.database_server_url);
        let insert_query = r#"
            INSERT INTO articles (id, title, content, template_id, source_news_ids, created_at, word_count, estimated_read_time)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#;
        
        let request_body = serde_json::json!({
            "query": insert_query,
            "parameters": [
                article.id.to_string(),
                article.title,
                article.content,
                article.template_id,
                serde_json::to_string(&article.source_news_ids).unwrap(),
                article.created_at.to_rfc3339(),
                article.word_count,
                article.estimated_read_time,
            ]
        });
        
        let response = self.client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| PipelineError::DatabaseServer(e.to_string()))?;
        
        if !response.status().is_success() {
            self.database_circuit_breaker.record_failure().await;
            return Err(PipelineError::DatabaseServer(
                format!("HTTP {}: {}", response.status(), response.text().await.unwrap_or_default())
            ));
        }
        
        let db_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| PipelineError::DatabaseServer(format!("Invalid JSON response: {}", e)))?;
        
        let database_id = db_response
            .get("inserted_id")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        
        let storage_result = StorageResult {
            article_id: article.id,
            database_id,
            stored_at: Utc::now(),
            storage_location: format!("articles_table_row_{}", database_id),
        };
        
        self.database_circuit_breaker.record_success().await;
        self.stats.database_store_time.fetch_add(
            start_time.elapsed().as_millis() as u64,
            Ordering::Relaxed,
        );
        
        debug!("Stored article in database in {:?}", start_time.elapsed());
        Ok(storage_result)
    }
    
    /// Track article analytics using analytics-server
    #[instrument(skip(self, article))]
    async fn track_article_analytics(&self, article: &GeneratedArticle) -> Result<AnalyticsResult, PipelineError> {
        let start_time = Instant::now();
        
        // Check circuit breaker
        if self.analytics_circuit_breaker.is_open().await {
            return Err(PipelineError::AnalyticsServer("Circuit breaker open".to_string()));
        }
        
        let url = format!("{}/track_content_creation", self.config.analytics_server_url);
        let metrics_data = serde_json::json!({
            "content_id": article.id.to_string(),
            "content_type": "generated_article",
            "template_id": article.template_id,
            "word_count": article.word_count,
            "estimated_read_time": article.estimated_read_time,
            "source_count": article.source_news_ids.len(),
            "created_at": article.created_at.to_rfc3339(),
        });
        
        let response = self.client
            .post(&url)
            .json(&metrics_data)
            .send()
            .await
            .map_err(|e| PipelineError::AnalyticsServer(e.to_string()))?;
        
        if !response.status().is_success() {
            self.analytics_circuit_breaker.record_failure().await;
            return Err(PipelineError::AnalyticsServer(
                format!("HTTP {}: {}", response.status(), response.text().await.unwrap_or_default())
            ));
        }
        
        let analytics_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| PipelineError::AnalyticsServer(format!("Invalid JSON response: {}", e)))?;
        
        let performance_score = analytics_response
            .get("performance_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        
        let analytics_result = AnalyticsResult {
            tracking_id: Uuid::new_v4(),
            article_id: article.id,
            metrics_recorded: vec![
                "content_creation".to_string(),
                "word_count".to_string(),
                "read_time".to_string(),
                "source_analysis".to_string(),
            ],
            performance_score,
            tracked_at: Utc::now(),
        };
        
        self.analytics_circuit_breaker.record_success().await;
        self.stats.analytics_track_time.fetch_add(
            start_time.elapsed().as_millis() as u64,
            Ordering::Relaxed,
        );
        
        debug!("Tracked analytics in {:?}", start_time.elapsed());
        Ok(analytics_result)
    }
    
    /// Get cached news data as fallback
    async fn get_cached_news(&self, query: &str) -> Result<Vec<NewsItem>, PipelineError> {
        for entry in self.cache.iter() {
            if entry.key().contains(query) {
                let (news_item, _) = entry.value();
                return Ok(vec![news_item.clone()]);
            }
        }
        
        Err(PipelineError::NewsServer("No cached data available".to_string()))
    }
    
    /// Generate summary from news items
    async fn generate_summary(&self, news_items: &[NewsItem]) -> String {
        if news_items.is_empty() {
            return "No news items available for summary.".to_string();
        }
        
        let titles: Vec<&str> = news_items.iter().map(|item| item.title.as_str()).collect();
        format!("This article covers {} recent developments: {}", news_items.len(), titles.join(", "))
    }
    
    /// Create error article for failed pipeline execution
    async fn create_error_article(&self, errors: &[String]) -> GeneratedArticle {
        GeneratedArticle {
            id: Uuid::new_v4(),
            title: "Pipeline Execution Failed".to_string(),
            content: format!("Content generation failed due to: {}", errors.join("; ")),
            template_id: "error_template".to_string(),
            source_news_ids: vec![],
            created_at: Utc::now(),
            word_count: 0,
            estimated_read_time: 0,
        }
    }
    
    /// Create error storage result
    async fn create_error_storage(&self) -> StorageResult {
        StorageResult {
            article_id: Uuid::new_v4(),
            database_id: -1,
            stored_at: Utc::now(),
            storage_location: "error".to_string(),
        }
    }
    
    /// Create error analytics result
    async fn create_error_analytics(&self) -> AnalyticsResult {
        AnalyticsResult {
            tracking_id: Uuid::new_v4(),
            article_id: Uuid::new_v4(),
            metrics_recorded: vec!["error".to_string()],
            performance_score: 0.0,
            tracked_at: Utc::now(),
        }
    }
    
    /// Get pipeline statistics
    pub fn get_stats(&self) -> &PipelineStats {
        &self.stats
    }
    
    /// Health check for all servers
    pub async fn health_check(&self) -> Result<serde_json::Value> {
        let servers = vec![
            ("news", &self.config.news_server_url),
            ("template", &self.config.template_server_url),
            ("database", &self.config.database_server_url),
            ("analytics", &self.config.analytics_server_url),
        ];
        
        let health_futures: Vec<_> = servers.iter().map(|(name, url)| {
            let client = &self.client;
            async move {
                let health_url = format!("{}/health", url);
                let result = client.get(&health_url).send().await;
                
                let status = match result {
                    Ok(response) if response.status().is_success() => "healthy",
                    Ok(_) => "unhealthy",
                    Err(_) => "unreachable",
                };
                
                (name.to_string(), status)
            }
        }).collect();
        
        let results = join_all(health_futures).await;
        let health_status: serde_json::Map<String, serde_json::Value> = results
            .into_iter()
            .map(|(name, status)| (name, serde_json::Value::String(status.to_string())))
            .collect();
        
        Ok(serde_json::Value::Object(health_status))
    }
}

/// Main application entry point
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("content_pipeline=info,reqwest=warn")
        .json()
        .init();
    
    info!("Starting Content Generation Pipeline Example");
    
    // Load configuration
    let config = PipelineConfig::default();
    info!("Loaded configuration: {:?}", config);
    
    // Create pipeline
    let pipeline = ContentPipeline::new(config);
    
    // Health check all servers
    info!("Performing health check on all servers...");
    match pipeline.health_check().await {
        Ok(health_status) => {
            info!("Server health status: {}", serde_json::to_string_pretty(&health_status)?);
        }
        Err(e) => {
            warn!("Health check failed: {}", e);
        }
    }
    
    // Example pipeline executions
    let test_queries = vec![
        ("artificial intelligence", "technology"),
        ("stock market", "business"),
        ("renewable energy", "technology"),
        ("healthcare innovation", "health"),
    ];
    
    for (query, category) in test_queries {
        info!("Executing pipeline for query: '{}' in category: '{}'", query, category);
        
        match pipeline.execute_pipeline(query, category).await {
            Ok(result) => {
                if result.success {
                    info!("✅ Pipeline execution successful!");
                    info!("   Article ID: {}", result.article.id);
                    info!("   Word Count: {}", result.article.word_count);
                    info!("   Execution Time: {:?}", result.execution_time);
                    info!("   Performance Score: {:.2}", result.analytics.performance_score);
                } else {
                    warn!("⚠️ Pipeline execution completed with errors:");
                    for error in &result.errors {
                        warn!("   - {}", error);
                    }
                }
            }
            Err(e) => {
                error!("❌ Pipeline execution failed: {}", e);
            }
        }
        
        // Brief pause between executions
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    
    // Display final statistics
    let stats = pipeline.get_stats();
    info!("Pipeline Statistics:");
    info!("  Total Executions: {}", stats.total_executions.load(Ordering::Relaxed));
    info!("  Success Rate: {:.2}%", stats.success_rate() * 100.0);
    info!("  Average Execution Time: {:?}", stats.average_execution_time());
    
    info!("Content Generation Pipeline Example completed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    
    #[tokio::test]
    async fn test_pipeline_creation() {
        let config = PipelineConfig::default();
        let pipeline = ContentPipeline::new(config);
        assert_eq!(pipeline.stats.total_executions.load(Ordering::Relaxed), 0);
    }
    
    #[tokio::test]
    async fn test_circuit_breaker() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        
        // Initially closed
        assert!(!cb.is_open().await);
        
        // Record failures
        cb.record_failure().await;
        cb.record_failure().await;
        cb.record_failure().await;
        
        // Should be open now
        assert!(cb.is_open().await);
        
        // Record success should close it
        cb.record_success().await;
        assert!(!cb.is_open().await);
    }
    
    #[tokio::test]
    async fn test_pipeline_stats() {
        let stats = PipelineStats::default();
        
        let result = PipelineResult {
            execution_id: Uuid::new_v4(),
            article: GeneratedArticle {
                id: Uuid::new_v4(),
                title: "Test".to_string(),
                content: "Test content".to_string(),
                template_id: "test".to_string(),
                source_news_ids: vec![],
                created_at: Utc::now(),
                word_count: 2,
                estimated_read_time: 1,
            },
            storage: StorageResult {
                article_id: Uuid::new_v4(),
                database_id: 1,
                stored_at: Utc::now(),
                storage_location: "test".to_string(),
            },
            analytics: AnalyticsResult {
                tracking_id: Uuid::new_v4(),
                article_id: Uuid::new_v4(),
                metrics_recorded: vec!["test".to_string()],
                performance_score: 0.85,
                tracked_at: Utc::now(),
            },
            execution_time: Duration::from_millis(1500),
            success: true,
            errors: vec![],
        };
        
        stats.record_execution(&result);
        
        assert_eq!(stats.total_executions.load(Ordering::Relaxed), 1);
        assert_eq!(stats.successful_executions.load(Ordering::Relaxed), 1);
        assert_eq!(stats.success_rate(), 1.0);
    }
}