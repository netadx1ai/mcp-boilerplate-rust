//! Basic MCP Server Template using official RMCP SDK
//!
//! This is a minimal, production-ready template for creating MCP servers.
//! Copy this template and customize the business logic for your specific use case.
//!
//! Features demonstrated:
//! - Official RMCP SDK v0.6.3 integration
//! - Multiple MCP tools with different argument patterns
//! - Comprehensive error handling
//! - Request statistics and server health monitoring
//! - Async state management with proper locking patterns
//! - Structured logging and CLI configuration
//! - Production-ready testing patterns

use anyhow::Result;
use clap::Parser;
use rmcp::{
    handler::server::wrapper::Parameters, model::*, tool, tool_router, transport::stdio,
    ErrorData as McpError, ServerHandler, ServiceExt,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

/// Command line arguments
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

/// Simple tool arguments example
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProcessDataArgs {
    /// Input data to process
    pub input: String,
    /// Processing options (optional)
    pub options: Option<HashMap<String, serde_json::Value>>,
}

/// List items arguments example
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListItemsArgs {
    /// Category filter (optional)
    pub category: Option<String>,
    /// Maximum number of items to return
    #[serde(default = "default_limit")]
    pub limit: u32,
}

/// Get item arguments example
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetItemArgs {
    /// Item ID to retrieve
    pub item_id: String,
}

/// Data item structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataItem {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Basic server implementation
#[derive(Clone)]
pub struct BasicServer {
    /// In-memory data storage (replace with real database/API in production)
    data_store: Arc<Mutex<HashMap<String, DataItem>>>,
    /// Request statistics
    stats: Arc<Mutex<HashMap<String, u64>>>,
}

// Default value functions
fn default_limit() -> u32 {
    10
}

impl BasicServer {
    /// Create new basic server
    pub fn new() -> Self {
        let server = Self {
            data_store: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(HashMap::new())),
        };

        // Initialize with sample data
        server.init_sample_data();
        server
    }

    /// Initialize sample data (replace with real data loading in production)
    fn init_sample_data(&self) {
        let sample_items = vec![
            DataItem {
                id: "item-1".to_string(),
                name: "Sample Item 1".to_string(),
                category: "example".to_string(),
                description: "This is a sample data item for demonstration".to_string(),
                created_at: chrono::Utc::now(),
                metadata: HashMap::new(),
            },
            DataItem {
                id: "item-2".to_string(),
                name: "Sample Item 2".to_string(),
                category: "demo".to_string(),
                description: "Another sample item with different category".to_string(),
                created_at: chrono::Utc::now(),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert(
                        "priority".to_string(),
                        serde_json::Value::String("high".to_string()),
                    );
                    meta
                },
            },
            DataItem {
                id: "item-3".to_string(),
                name: "Sample Item 3".to_string(),
                category: "example".to_string(),
                description: "Third sample item for testing list operations".to_string(),
                created_at: chrono::Utc::now(),
                metadata: HashMap::new(),
            },
        ];

        // Populate data store
        tokio::spawn({
            let data_store = self.data_store.clone();
            async move {
                let mut store = data_store.lock().await;
                for item in sample_items {
                    store.insert(item.id.clone(), item);
                }
            }
        });
    }

    /// Increment statistics counter
    async fn increment_stat(&self, key: &str) {
        let mut stats = self.stats.lock().await;
        *stats.entry(key.to_string()).or_insert(0) += 1;
    }

    /// Generate server information
    async fn get_server_info(&self) -> HashMap<String, serde_json::Value> {
        let stats = self.stats.lock().await;
        let data_store = self.data_store.lock().await;

        let mut info = HashMap::new();
        info.insert(
            "server_name".to_string(),
            serde_json::Value::String("Basic MCP Server".to_string()),
        );
        info.insert(
            "version".to_string(),
            serde_json::Value::String("1.0.0".to_string()),
        );
        info.insert(
            "status".to_string(),
            serde_json::Value::String("healthy".to_string()),
        );
        info.insert(
            "uptime_seconds".to_string(),
            serde_json::Value::Number(serde_json::Number::from(300)),
        ); // Mock uptime
        info.insert(
            "total_items".to_string(),
            serde_json::Value::Number(serde_json::Number::from(data_store.len())),
        );
        info.insert(
            "request_stats".to_string(),
            serde_json::to_value(&*stats).unwrap_or_default(),
        );

        info
    }
}

impl Default for BasicServer {
    fn default() -> Self {
        Self::new()
    }
}

/// MCP tool router implementation
#[tool_router]
impl BasicServer {
    /// Process data tool - demonstrates simple data processing
    #[tool(description = "Process input data and return transformed result")]
    async fn process_data(
        &self,
        Parameters(args): Parameters<ProcessDataArgs>,
    ) -> Result<CallToolResult, McpError> {
        info!("Processing data: {}", args.input);
        self.increment_stat("process_data_calls").await;

        // TODO: Replace with your actual data processing logic
        let processed_result = format!("Processed: {}", args.input);
        let timestamp = chrono::Utc::now();

        let mut result = HashMap::new();
        result.insert(
            "original_input".to_string(),
            serde_json::Value::String(args.input),
        );
        result.insert(
            "processed_output".to_string(),
            serde_json::Value::String(processed_result),
        );
        result.insert(
            "processing_timestamp".to_string(),
            serde_json::Value::String(timestamp.to_rfc3339()),
        );

        if let Some(options) = args.options {
            result.insert(
                "options_used".to_string(),
                serde_json::to_value(options).unwrap(),
            );
        }

        let result_text = serde_json::to_string_pretty(&result).unwrap();
        Ok(CallToolResult::success(vec![Content::text(result_text)]))
    }

    /// List items tool - demonstrates data listing with filtering
    #[tool(description = "List available data items with optional category filtering")]
    async fn list_items(
        &self,
        Parameters(args): Parameters<ListItemsArgs>,
    ) -> Result<CallToolResult, McpError> {
        info!(
            "Listing items with category filter: {:?}, limit: {}",
            args.category, args.limit
        );
        self.increment_stat("list_items_calls").await;

        let data_store = self.data_store.lock().await;
        let mut items: Vec<&DataItem> = data_store.values().collect();

        // Apply category filter if specified
        if let Some(category) = &args.category {
            items.retain(|item| item.category == *category);
        }

        // Apply limit
        items.truncate(args.limit as usize);

        let mut result = HashMap::new();
        result.insert(
            "total_available".to_string(),
            serde_json::Value::Number(serde_json::Number::from(data_store.len())),
        );
        result.insert(
            "filtered_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(items.len())),
        );
        result.insert(
            "category_filter".to_string(),
            serde_json::Value::String(args.category.unwrap_or_else(|| "none".to_string())),
        );
        result.insert("items".to_string(), serde_json::to_value(items).unwrap());

        let result_text = serde_json::to_string_pretty(&result).unwrap();
        Ok(CallToolResult::success(vec![Content::text(result_text)]))
    }

    /// Get item tool - demonstrates single item retrieval
    #[tool(description = "Retrieve a specific item by ID")]
    async fn get_item(
        &self,
        Parameters(args): Parameters<GetItemArgs>,
    ) -> Result<CallToolResult, McpError> {
        info!("Getting item: {}", args.item_id);
        self.increment_stat("get_item_calls").await;

        let data_store = self.data_store.lock().await;

        match data_store.get(&args.item_id) {
            Some(item) => {
                let mut result = HashMap::new();
                result.insert("found".to_string(), serde_json::Value::Bool(true));
                result.insert("item".to_string(), serde_json::to_value(item).unwrap());
                result.insert(
                    "retrieved_at".to_string(),
                    serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
                );

                let result_text = serde_json::to_string_pretty(&result).unwrap();
                Ok(CallToolResult::success(vec![Content::text(result_text)]))
            }
            None => {
                warn!("Item not found: {}", args.item_id);
                let mut result = HashMap::new();
                result.insert("found".to_string(), serde_json::Value::Bool(false));
                result.insert(
                    "item_id".to_string(),
                    serde_json::Value::String(args.item_id),
                );
                result.insert(
                    "error".to_string(),
                    serde_json::Value::String("Item not found".to_string()),
                );

                let result_text = serde_json::to_string_pretty(&result).unwrap();
                Ok(CallToolResult::success(vec![Content::text(result_text)]))
            }
        }
    }

    /// Server status tool - demonstrates health monitoring
    #[tool(description = "Get server health and usage statistics")]
    async fn get_server_status(&self) -> Result<CallToolResult, McpError> {
        info!("Getting server status");
        self.increment_stat("get_server_status_calls").await;

        let server_info = self.get_server_info().await;
        let result_text = serde_json::to_string_pretty(&server_info).unwrap();
        Ok(CallToolResult::success(vec![Content::text(result_text)]))
    }
}

/// Server handler implementation - required for RMCP integration
impl ServerHandler for BasicServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "🔧 Basic MCP Server Template - Production-ready foundation:\n\
                • process_data: Process input data with optional configurations\n\
                • list_items: List and filter available data items\n\
                • get_item: Retrieve specific items by ID\n\
                • get_server_status: Monitor server health and usage statistics\n\n\
                💡 Customize this template by:\n\
                1. Replace DataItem with your domain model\n\
                2. Implement your business logic in tool methods\n\
                3. Add authentication/authorization as needed\n\
                4. Connect to real data sources (database, APIs)\n\
                5. Add your specific MCP tools\n\n\
                🚀 Ready for production deployment and scaling!"
                    .to_string(),
            ),
        }
    }
}

/// Main function - server startup and configuration
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.debug { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new(format!(
                "basic_server_template={},rmcp={}",
                log_level, log_level
            ))
        }))
        .init();

    info!("Starting Basic MCP Server Template v1.0.0");
    info!("Debug mode: {}", args.debug);

    // Create server instance
    let server = BasicServer::new();

    info!(
        "📊 Basic server initialized with {} sample items",
        server.data_store.lock().await.len()
    );

    // Start the MCP server with stdio transport
    let service = server.serve(stdio()).await.inspect_err(|e| {
        error!("❌ Failed to start server: {}", e);
    })?;

    info!("✅ Basic server ready for MCP connections");
    service.waiting().await.inspect_err(|e| {
        error!("❌ Server error: {}", e);
    })?;

    info!("Server shutdown complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::handler::server::wrapper::Parameters;

    /// Test server creation
    #[tokio::test]
    async fn test_server_creation() {
        let server = BasicServer::new();

        // Verify initial state
        let stats = server.stats.lock().await;
        assert!(stats.is_empty(), "Stats should be empty on creation");

        // Allow time for sample data initialization
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let data_store = server.data_store.lock().await;
        assert!(!data_store.is_empty(), "Data store should have sample data");
    }

    /// Test process_data tool
    #[tokio::test]
    async fn test_process_data_tool() {
        let server = BasicServer::new();

        let args = ProcessDataArgs {
            input: "test input".to_string(),
            options: Some({
                let mut opts = HashMap::new();
                opts.insert("mode".to_string(), serde_json::json!("test"));
                opts
            }),
        };

        let result = server.process_data(Parameters(args)).await;
        assert!(result.is_ok(), "process_data should succeed");

        // Verify stats were updated
        let stats = server.stats.lock().await;
        assert_eq!(stats.get("process_data_calls"), Some(&1));
    }

    /// Test list_items tool
    #[tokio::test]
    async fn test_list_items_tool() {
        let server = BasicServer::new();

        // Allow time for sample data initialization
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let args = ListItemsArgs {
            category: Some("example".to_string()),
            limit: 5,
        };

        let result = server.list_items(Parameters(args)).await;
        assert!(result.is_ok(), "list_items should succeed");
    }

    /// Test get_item tool
    #[tokio::test]
    async fn test_get_item_tool() {
        let server = BasicServer::new();

        // Allow time for sample data initialization
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Test existing item
        let args = GetItemArgs {
            item_id: "item-1".to_string(),
        };

        let result = server.get_item(Parameters(args)).await;
        assert!(result.is_ok(), "get_item should succeed for existing item");

        // Test non-existing item
        let args = GetItemArgs {
            item_id: "non-existent".to_string(),
        };

        let result = server.get_item(Parameters(args)).await;
        assert!(
            result.is_ok(),
            "get_item should succeed but return not found"
        );
    }

    /// Test server status tool
    #[tokio::test]
    async fn test_server_status_tool() {
        let server = BasicServer::new();

        let result = server.get_server_status().await;
        assert!(result.is_ok(), "get_server_status should succeed");
    }

    /// Test multiple tool calls update statistics
    #[tokio::test]
    async fn test_statistics_tracking() {
        let server = BasicServer::new();

        // Make multiple calls
        let _ = server.get_server_status().await;
        let _ = server.get_server_status().await;
        let _ = server.get_server_status().await;

        // Verify stats
        let stats = server.stats.lock().await;
        assert_eq!(stats.get("get_server_status_calls"), Some(&3));
    }
}
