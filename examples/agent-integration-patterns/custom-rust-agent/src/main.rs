//! Custom Rust Agent with MCP Server Integration
//!
//! This example demonstrates how to build a custom AI agent in Rust that integrates
//! with our MCP server ecosystem. The agent provides:
//!
//! 1. Natural language processing with OpenAI integration
//! 2. MCP tool execution with intelligent routing
//! 3. Task planning and execution workflows
//! 4. Error handling and resilience patterns
//! 5. Performance monitoring and metrics
//!
//! ## Features
//! - Direct MCP server integration with connection pooling
//! - AI-powered task planning and execution
//! - Streaming responses for real-time interaction
//! - Circuit breaker pattern for server resilience
//! - Comprehensive logging and monitoring
//! - Configurable retry logic and fallback mechanisms

use anyhow::{Context, Result};
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessage, CreateChatCompletionRequest, Role,
    },
    Client as OpenAIClient,
};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use dashmap::DashMap;
use futures::future::try_join_all;
use handlebars::Handlebars;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

/// Agent execution errors
#[derive(Error, Debug)]
pub enum AgentError {
    #[error("MCP server error: {server} - {message}")]
    MCPServer { server: String, message: String },
    
    #[error("OpenAI API error: {0}")]
    OpenAI(String),
    
    #[error("Task planning error: {0}")]
    Planning(String),
    
    #[error("Task execution error: {0}")]
    Execution(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Circuit breaker open for server: {0}")]
    CircuitBreakerOpen(String),
    
    #[error("All retry attempts failed: {0}")]
    RetryExhausted(String),
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerConfig {
    pub name: String,
    pub url: String,
    pub timeout_seconds: u64,
    pub max_retries: usize,
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_timeout_seconds: u64,
    pub capabilities: Vec<String>,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub openai_model: String,
    pub openai_temperature: f32,
    pub max_planning_iterations: usize,
    pub task_timeout_seconds: u64,
    pub enable_streaming: bool,
    pub enable_metrics: bool,
    pub log_level: String,
    pub servers: Vec<MCPServerConfig>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            openai_model: "gpt-4-turbo-preview".to_string(),
            openai_temperature: 0.7,
            max_planning_iterations: 5,
            task_timeout_seconds: 60,
            enable_streaming: false,
            enable_metrics: true,
            log_level: "info".to_string(),
            servers: vec![
                MCPServerConfig {
                    name: "news-data-server".to_string(),
                    url: "http://localhost:3001".to_string(),
                    timeout_seconds: 30,
                    max_retries: 3,
                    circuit_breaker_threshold: 5,
                    circuit_breaker_timeout_seconds: 60,
                    capabilities: vec![
                        "search_news".to_string(),
                        "get_trending_news".to_string(),
                        "get_categories".to_string(),
                    ],
                },
                MCPServerConfig {
                    name: "template-server".to_string(),
                    url: "http://localhost:3002".to_string(),
                    timeout_seconds: 15,
                    max_retries: 2,
                    circuit_breaker_threshold: 3,
                    circuit_breaker_timeout_seconds: 30,
                    capabilities: vec![
                        "render_template".to_string(),
                        "list_templates".to_string(),
                        "create_template".to_string(),
                    ],
                },
                MCPServerConfig {
                    name: "database-server".to_string(),
                    url: "http://localhost:3003".to_string(),
                    timeout_seconds: 45,
                    max_retries: 3,
                    circuit_breaker_threshold: 3,
                    circuit_breaker_timeout_seconds: 30,
                    capabilities: vec![
                        "execute_query".to_string(),
                        "list_tables".to_string(),
                        "get_table_schema".to_string(),
                    ],
                },
                MCPServerConfig {
                    name: "analytics-server".to_string(),
                    url: "http://localhost:3004".to_string(),
                    timeout_seconds: 20,
                    max_retries: 2,
                    circuit_breaker_threshold: 5,
                    circuit_breaker_timeout_seconds: 60,
                    capabilities: vec![
                        "track_content_metrics".to_string(),
                        "get_engagement_trends".to_string(),
                        "generate_analytics_report".to_string(),
                    ],
                },
            ],
        }
    }
}

/// Task execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    pub id: Uuid,
    pub description: String,
    pub steps: Vec<TaskStep>,
    pub estimated_duration: Duration,
    pub created_at: DateTime<Utc>,
}

/// Individual task step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep {
    pub id: Uuid,
    pub description: String,
    pub server: String,
    pub tool: String,
    pub parameters: serde_json::Value,
    pub dependencies: Vec<Uuid>,
    pub status: TaskStepStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Task step execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub plan_id: Uuid,
    pub success: bool,
    pub completed_steps: usize,
    pub failed_steps: usize,
    pub total_duration: Duration,
    pub results: Vec<serde_json::Value>,
    pub errors: Vec<String>,
    pub performance_metrics: PerformanceMetrics,
}

/// Performance metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_mcp_calls: u64,
    pub successful_mcp_calls: u64,
    pub failed_mcp_calls: u64,
    pub average_response_time: Duration,
    pub server_response_times: HashMap<String, Duration>,
    pub circuit_breaker_activations: u64,
    pub retry_attempts: u64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_mcp_calls: 0,
            successful_mcp_calls: 0,
            failed_mcp_calls: 0,
            average_response_time: Duration::from_secs(0),
            server_response_times: HashMap::new(),
            circuit_breaker_activations: 0,
            retry_attempts: 0,
        }
    }
}

/// Circuit breaker for server resilience
#[derive(Debug)]
pub struct CircuitBreaker {
    failure_count: AtomicU64,
    last_failure: RwLock<Option<Instant>>,
    threshold: u32,
    timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(threshold: u32, timeout: Duration) -> Self {
        Self {
            failure_count: AtomicU64::new(0),
            last_failure: RwLock::new(None),
            threshold,
            timeout,
        }
    }
    
    pub async fn is_open(&self) -> bool {
        let count = self.failure_count.load(Ordering::Relaxed);
        if count < self.threshold as u64 {
            return false;
        }
        
        let last_failure = self.last_failure.read().await;
        match *last_failure {
            Some(time) => time.elapsed() < self.timeout,
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

/// MCP client for server communication
#[derive(Debug)]
pub struct MCPClient {
    config: MCPServerConfig,
    http_client: HttpClient,
    circuit_breaker: Arc<CircuitBreaker>,
    metrics: Arc<RwLock<PerformanceMetrics>>,
}

impl MCPClient {
    pub fn new(config: MCPServerConfig) -> Self {
        let timeout = Duration::from_secs(config.timeout_seconds);
        let http_client = HttpClient::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");
        
        let circuit_breaker = Arc::new(CircuitBreaker::new(
            config.circuit_breaker_threshold,
            Duration::from_secs(config.circuit_breaker_timeout_seconds),
        ));
        
        Self {
            config,
            http_client,
            circuit_breaker,
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
        }
    }
    
    #[instrument(skip(self, parameters))]
    pub async fn call_tool(&self, tool: &str, parameters: serde_json::Value) -> Result<serde_json::Value, AgentError> {
        let start_time = Instant::now();
        
        // Check circuit breaker
        if self.circuit_breaker.is_open().await {
            self.metrics.write().await.circuit_breaker_activations += 1;
            return Err(AgentError::CircuitBreakerOpen(self.config.name.clone()));
        }
        
        let url = format!("{}/{}", self.config.url, tool);
        let mut last_error = None;
        
        for attempt in 1..=self.config.max_retries {
            match self.http_client
                .post(&url)
                .json(&parameters)
                .send()
                .await
            {
                Ok(response) => {
                    let elapsed = start_time.elapsed();
                    
                    if response.status().is_success() {
                        match response.json::<serde_json::Value>().await {
                            Ok(result) => {
                                self.circuit_breaker.record_success().await;
                                self.update_metrics(true, elapsed).await;
                                debug!("MCP call successful: {} -> {}", tool, self.config.name);
                                return Ok(result);
                            }
                            Err(e) => {
                                last_error = Some(format!("Invalid JSON response: {}", e));
                            }
                        }
                    } else {
                        last_error = Some(format!("HTTP {}: {}", 
                            response.status(), 
                            response.text().await.unwrap_or_default()
                        ));
                    }
                }
                Err(e) => {
                    last_error = Some(format!("Request failed: {}", e));
                }
            }
            
            if attempt < self.config.max_retries {
                let delay = Duration::from_millis(100 * (1 << (attempt - 1))); // Exponential backoff
                debug!("Retrying MCP call in {:?} (attempt {}/{})", delay, attempt, self.config.max_retries);
                tokio::time::sleep(delay).await;
                self.metrics.write().await.retry_attempts += 1;
            }
        }
        
        // All retries failed
        self.circuit_breaker.record_failure().await;
        self.update_metrics(false, start_time.elapsed()).await;
        
        let error_msg = last_error.unwrap_or_else(|| "Unknown error".to_string());
        Err(AgentError::MCPServer {
            server: self.config.name.clone(),
            message: error_msg,
        })
    }
    
    async fn update_metrics(&self, success: bool, response_time: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.total_mcp_calls += 1;
        
        if success {
            metrics.successful_mcp_calls += 1;
        } else {
            metrics.failed_mcp_calls += 1;
        }
        
        // Update average response time
        let total_time = metrics.average_response_time.as_millis() as u64 * metrics.total_mcp_calls;
        let new_total = total_time + response_time.as_millis() as u64;
        metrics.average_response_time = Duration::from_millis(new_total / (metrics.total_mcp_calls + 1));
        
        // Update server-specific response time
        metrics.server_response_times.insert(self.config.name.clone(), response_time);
    }
    
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/health", self.config.url);
        match self.http_client.get(&url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
    
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }
}

/// Main agent implementation
pub struct RustAgent {
    config: AgentConfig,
    openai_client: OpenAIClient<OpenAIConfig>,
    mcp_clients: HashMap<String, Arc<MCPClient>>,
    template_engine: Handlebars<'static>,
    task_cache: Arc<DashMap<Uuid, TaskResult>>,
}

impl RustAgent {
    pub async fn new(config: AgentConfig, openai_api_key: String) -> Result<Self> {
        let openai_config = OpenAIConfig::new().with_api_key(openai_api_key);
        let openai_client = OpenAIClient::with_config(openai_config);
        
        let mut mcp_clients = HashMap::new();
        for server_config in &config.servers {
            let client = Arc::new(MCPClient::new(server_config.clone()));
            mcp_clients.insert(server_config.name.clone(), client);
        }
        
        let mut template_engine = Handlebars::new();
        template_engine.register_template_string("task_prompt", include_str!("../templates/task_prompt.hbs"))
            .context("Failed to register task prompt template")?;
        
        Ok(Self {
            config,
            openai_client,
            mcp_clients,
            template_engine,
            task_cache: Arc::new(DashMap::new()),
        })
    }
    
    #[instrument(skip(self))]
    pub async fn execute_task(&self, user_input: &str) -> Result<TaskResult> {
        info!("Executing task: {}", user_input);
        
        // Step 1: Create task plan
        let plan = self.create_task_plan(user_input).await
            .context("Failed to create task plan")?;
        
        info!("Created task plan with {} steps", plan.steps.len());
        
        // Step 2: Execute task plan
        let result = self.execute_task_plan(plan).await
            .context("Failed to execute task plan")?;
        
        // Step 3: Cache result
        self.task_cache.insert(result.plan_id, result.clone());
        
        info!("Task completed: success={}, steps={}/{}", 
            result.success, result.completed_steps, result.completed_steps + result.failed_steps);
        
        Ok(result)
    }
    
    async fn create_task_plan(&self, user_input: &str) -> Result<TaskPlan, AgentError> {
        let available_tools = self.get_available_tools().await;
        let server_status = self.check_server_health().await;
        
        let context = serde_json::json!({
            "user_input": user_input,
            "available_tools": available_tools,
            "server_status": server_status,
            "current_time": Utc::now().to_rfc3339(),
        });
        
        let prompt = self.template_engine
            .render("task_prompt", &context)
            .map_err(|e| AgentError::Planning(format!("Template rendering failed: {}", e)))?;
        
        let messages = vec![
            ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: prompt,
                role: Role::System,
                name: None,
            }),
            ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                content: user_input.to_string(),
                role: Role::User,
                name: None,
            }),
        ];
        
        let request = CreateChatCompletionRequest {
            model: self.config.openai_model.clone(),
            messages,
            temperature: Some(self.config.openai_temperature),
            max_tokens: Some(2000),
            ..Default::default()
        };
        
        let response = self.openai_client
            .chat()
            .create(request)
            .await
            .map_err(|e| AgentError::OpenAI(format!("OpenAI API call failed: {}", e)))?;
        
        let plan_text = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or_else(|| AgentError::Planning("No response from OpenAI".to_string()))?;
        
        // Parse the plan from the AI response
        self.parse_task_plan(plan_text, user_input).await
    }
    
    async fn parse_task_plan(&self, plan_text: &str, description: &str) -> Result<TaskPlan, AgentError> {
        // This is a simplified parser. In a real implementation, you might use
        // more sophisticated parsing or have the AI return structured JSON.
        
        let plan_id = Uuid::new_v4();
        let mut steps = Vec::new();
        
        // Extract steps from the plan text (simplified parsing)
        for (i, line) in plan_text.lines().enumerate() {
            if line.trim().starts_with("Step") || line.trim().starts_with(&format!("{}.", i + 1)) {
                if let Some(step) = self.parse_task_step(line, i).await {
                    steps.push(step);
                }
            }
        }
        
        // If no structured steps found, create a default step
        if steps.is_empty() {
            steps.push(TaskStep {
                id: Uuid::new_v4(),
                description: "Execute user request".to_string(),
                server: "news-data-server".to_string(),
                tool: "search_news".to_string(),
                parameters: serde_json::json!({
                    "query": description,
                    "limit": 5
                }),
                dependencies: vec![],
                status: TaskStepStatus::Pending,
                result: None,
                error: None,
                started_at: None,
                completed_at: None,
            });
        }
        
        Ok(TaskPlan {
            id: plan_id,
            description: description.to_string(),
            steps,
            estimated_duration: Duration::from_secs(30),
            created_at: Utc::now(),
        })
    }
    
    async fn parse_task_step(&self, step_text: &str, index: usize) -> Option<TaskStep> {
        // Simplified step parsing - in reality, you'd have more sophisticated parsing
        let step_id = Uuid::new_v4();
        
        // Try to identify server and tool from step text
        let (server, tool) = if step_text.to_lowercase().contains("news") || step_text.to_lowercase().contains("search") {
            ("news-data-server", "search_news")
        } else if step_text.to_lowercase().contains("template") || step_text.to_lowercase().contains("create") {
            ("template-server", "render_template")
        } else if step_text.to_lowercase().contains("store") || step_text.to_lowercase().contains("database") {
            ("database-server", "execute_query")
        } else if step_text.to_lowercase().contains("analytics") || step_text.to_lowercase().contains("track") {
            ("analytics-server", "track_content_metrics")
        } else {
            ("news-data-server", "search_news") // Default
        };
        
        Some(TaskStep {
            id: step_id,
            description: step_text.trim().to_string(),
            server: server.to_string(),
            tool: tool.to_string(),
            parameters: serde_json::json!({
                "query": "default query",
                "limit": 5
            }),
            dependencies: if index > 0 { vec![step_id] } else { vec![] },
            status: TaskStepStatus::Pending,
            result: None,
            error: None,
            started_at: None,
            completed_at: None,
        })
    }
    
    async fn execute_task_plan(&self, mut plan: TaskPlan) -> Result<TaskResult> {
        let start_time = Instant::now();
        let mut completed_steps = 0;
        let mut failed_steps = 0;
        let mut results = Vec::new();
        let mut errors = Vec::new();
        
        // Execute steps in dependency order
        let mut remaining_steps = plan.steps.clone();
        
        while !remaining_steps.is_empty() {
            let mut executed_any = false;
            let mut new_remaining = Vec::new();
            
            for mut step in remaining_steps {
                // Check if all dependencies are completed
                let dependencies_ready = step.dependencies.iter().all(|dep_id| {
                    plan.steps.iter().any(|s| s.id == *dep_id && s.status == TaskStepStatus::Completed)
                });
                
                if dependencies_ready {
                    step.status = TaskStepStatus::Running;
                    step.started_at = Some(Utc::now());
                    
                    match self.execute_task_step(&step).await {
                        Ok(result) => {
                            step.status = TaskStepStatus::Completed;
                            step.result = Some(result.clone());
                            step.completed_at = Some(Utc::now());
                            results.push(result);
                            completed_steps += 1;
                            info!("Step completed: {}", step.description);
                        }
                        Err(e) => {
                            step.status = TaskStepStatus::Failed;
                            step.error = Some(e.to_string());
                            step.completed_at = Some(Utc::now());
                            errors.push(e.to_string());
                            failed_steps += 1;
                            warn!("Step failed: {} - {}", step.description, e);
                        }
                    }
                    
                    // Update step in plan
                    if let Some(plan_step) = plan.steps.iter_mut().find(|s| s.id == step.id) {
                        *plan_step = step;
                    }
                    
                    executed_any = true;
                } else {
                    new_remaining.push(step);
                }
            }
            
            remaining_steps = new_remaining;
            
            // Prevent infinite loops
            if !executed_any && !remaining_steps.is_empty() {
                error!("Deadlock detected in task execution - some dependencies cannot be satisfied");
                break;
            }
        }
        
        let total_duration = start_time.elapsed();
        let success = failed_steps == 0 && completed_steps > 0;
        
        // Aggregate performance metrics
        let mut performance_metrics = PerformanceMetrics::default();
        for client in self.mcp_clients.values() {
            let client_metrics = client.get_metrics().await;
            performance_metrics.total_mcp_calls += client_metrics.total_mcp_calls;
            performance_metrics.successful_mcp_calls += client_metrics.successful_mcp_calls;
            performance_metrics.failed_mcp_calls += client_metrics.failed_mcp_calls;
            performance_metrics.circuit_breaker_activations += client_metrics.circuit_breaker_activations;
            performance_metrics.retry_attempts += client_metrics.retry_attempts;
            
            for (server, response_time) in client_metrics.server_response_times {
                performance_metrics.server_response_times.insert(server, response_time);
            }
        }
        
        Ok(TaskResult {
            plan_id: plan.id,
            success,
            completed_steps,
            failed_steps,
            total_duration,
            results,
            errors,
            performance_metrics,
        })
    }
    
    async fn execute_task_step(&self, step: &TaskStep) -> Result<serde_json::Value, AgentError> {
        let client = self.mcp_clients.get(&step.server)
            .ok_or_else(|| AgentError::Execution(format!("Unknown server: {}", step.server)))?;
        
        client.call_tool(&step.tool, step.parameters.clone()).await
    }
    
    async fn get_available_tools(&self) -> Vec<serde_json::Value> {
        let mut tools = Vec::new();
        
        for (server_name, config) in self.config.servers.iter().enumerate() {
            for capability in &config.capabilities {
                tools.push(serde_json::json!({
                    "server": config.name,
                    "tool": capability,
                    "description": format!("Tool {} on server {}", capability, config.name)
                }));
            }
        }
        
        tools
    }
    
    async fn check_server_health(&self) -> HashMap<String, bool> {
        let mut health_status = HashMap::new();
        
        let health_checks: Vec<_> = self.mcp_clients.iter().map(|(name, client)| {
            let name = name.clone();
            let client = client.clone();
            async move {
                let healthy = client.health_check().await;
                (name, healthy)
            }
        }).collect();
        
        let results = try_join_all(health_checks).await.unwrap_or_default();
        
        for (name, healthy) in results {
            health_status.insert(name, healthy);
        }
        
        health_status
    }
    
    pub async fn get_task_history(&self) -> Vec<TaskResult> {
        self.task_cache.iter().map(|entry| entry.value().clone()).collect()
    }
    
    pub async fn get_performance_summary(&self) -> PerformanceMetrics {
        let mut summary = PerformanceMetrics::default();
        
        for client in self.mcp_clients.values() {
            let metrics = client.get_metrics().await;
            summary.total_mcp_calls += metrics.total_mcp_calls;
            summary.successful_mcp_calls += metrics.successful_mcp_calls;
            summary.failed_mcp_calls += metrics.failed_mcp_calls;
            summary.circuit_breaker_activations += metrics.circuit_breaker_activations;
            summary.retry_attempts += metrics.retry_attempts;
        }
        
        summary
    }
}

/// CLI argument structure
#[derive(Parser)]
#[command(name = "rust-agent")]
#[command(about = "Custom Rust Agent with MCP Server Integration")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute a task using the agent
    Execute {
        /// The task description
        #[arg(short, long)]
        task: String,
        
        /// Configuration file path
        #[arg(short, long)]
        config: Option<String>,
    },
    /// Check the health of all MCP servers
    Health,
    /// Show performance metrics
    Metrics,
    /// Interactive mode
    Interactive,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("rust_agent=info,reqwest=warn")
        .json()
        .init();
    
    let cli = Cli::parse();
    
    // Load configuration
    let config = AgentConfig::default(); // In real app, load from file or env
    
    // Get OpenAI API key
    let openai_api_key = std::env::var("OPENAI_API_KEY")
        .context("OPENAI_API_KEY environment variable not set")?;
    
    // Create agent
    let agent = RustAgent::new(config, openai_api_key).await
        .context("Failed to create agent")?;
    
    match cli.command {
        Commands::Execute { task, config: _ } => {
            info!("🚀 Executing task: {}", task);
            
            match agent.execute_task(&task).await {
                Ok(result) => {
                    println!("✅ Task completed successfully!");
                    println!("   Completed steps: {}", result.completed_steps);
                    println!("   Failed steps: {}", result.failed_steps);
                    println!("   Duration: {:?}", result.total_duration);
                    
                    if !result.results.is_empty() {
                        println!("\n📊 Results:");
                        for (i, result) in result.results.iter().enumerate() {
                            println!("   {}. {}", i + 1, serde_json::to_string_pretty(result)?);
                        }
                    }
                    
                    if !result.errors.is_empty() {
                        println!("\n❌ Errors:");
                        for error in &result.errors {
                            println!("   - {}", error);
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Task execution failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::Health => {
            info!("🏥 Checking server health...");
            let health_status = agent.check_server_health().await;
            
            println!("Server Health Status:");
            for (server, healthy) in health_status {
                let status = if healthy { "✅ Healthy" } else { "❌ Unhealthy" };
                println!("  {}: {}", server, status);
            }
        }
        
        Commands::Metrics => {
            info!("📊 Getting performance metrics...");
            let metrics = agent.get_performance_summary().await;
            
            println!("Performance Metrics:");
            println!("  Total MCP Calls: {}", metrics.total_mcp_calls);
            println!("  Successful Calls: {}", metrics.successful_mcp_calls);
            println!("  Failed Calls: {}", metrics.failed_mcp_calls);
            println!("  Success Rate: {:.2}%", 
                if metrics.total_mcp_calls > 0 {
                    (metrics.successful_mcp_calls as f64 / metrics.total_mcp_calls as f64) * 100.0
                } else {
                    0.0
                }
            );
            println!("  Circuit Breaker Activations: {}", metrics.circuit_breaker_activations);
            println!("  Retry Attempts: {}", metrics.retry_attempts);
        }
        
        Commands::Interactive => {
            info!("🤖 Starting interactive mode...");
            println!("Rust Agent Interactive Mode");
            println!("Type 'quit' to exit, 'help' for commands");
            
            loop {
                print!("\n> ");
                use std::io::{self, Write};
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let input = input.trim();
                
                match input.to_lowercase().as_str() {
                    "quit" | "exit" => break,
                    "help" => {
                        println!("Available commands:");
                        println!("  help     - Show this help");
                        println!("  health   - Check server health");
                        println!("  metrics  - Show performance metrics");
                        println!("  history  - Show task history");
                        println!("  quit     - Exit interactive mode");
                        println!("  Or enter any task description to execute");
                    }
                    "health" => {
                        let health_status = agent.check_server_health().await;
                        for (server, healthy) in health_status {
                            let status = if healthy { "✅" } else { "❌" };
                            println!("  {} {}", status, server);
                        }
                    }
                    "metrics" => {
                        let metrics = agent.get_performance_summary().await;
                        println!("  Total calls: {} ({}% success)", 
                            metrics.total_mcp_calls,
                            if metrics.total_mcp_calls > 0 {
                                (metrics.successful_mcp_calls * 100 / metrics.total_mcp_calls)
                            } else { 0 }
                        );
                    }
                    "history" => {
                        let history = agent.get_task_history().await;
                        if history.is_empty() {
                            println!("  No tasks executed yet");
                        } else {
                            for (i, task) in history.iter().enumerate() {
                                let status = if task.success { "✅" } else { "❌" };
                                println!("  {}. {} ({:?}) {}", i + 1, status, task.total_duration, task.plan_id);
                            }
                        }
                    }
                    "" => continue,
                    task => {
                        println!("🔄 Executing: {}", task);
                        match agent.execute_task(task).await {
                            Ok(result) => {
                                if result.success {
                                    println!("✅ Task completed in {:?}", result.total_duration);
                                } else {
                                    println!("⚠️ Task completed with errors");
                                }
                            }
                            Err(e) => {
                                println!("❌ Task failed: {}", e);
                            }
                        }
                    }
                }
            }
        }
    }
    
    info!("🎉 Agent execution completed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.servers.len(), 4);
        assert_eq!(config.openai_model, "gpt-4-turbo-preview");
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
    
    #[test]
    fn test_task_step_status() {
        let step = TaskStep {
            id: Uuid::new_v4(),
            description: "Test step".to_string(),
            server: "test-server".to_string(),
            tool: "test-tool".to_string(),
            parameters: serde_json::json!({}),
            dependencies: vec![],
            status: TaskStepStatus::Pending,
            result: None,
            error: None,
            started_at: None,
            completed_at: None,
        };
        
        assert_eq!(step.status, TaskStepStatus::Pending);
    }
}