//! Minimal News Data Server using official RMCP SDK
//!
//! This is a lightweight MCP server for news data with core functionality.
//! Security features will be added incrementally in future versions.

use anyhow::{Context, Result};
use clap::Parser;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    service::RequestContext,
    tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
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

/// Search news arguments
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchNewsArgs {
    /// Search query
    pub query: String,
    /// Number of articles (default: 10)
    #[serde(default = "default_limit")]
    pub limit: u32,
    /// Language code (default: en)
    #[serde(default = "default_language")]
    pub language: String,
}

/// Category news arguments
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetCategoryArgs {
    /// News category
    pub category: String,
    /// Number of articles (default: 10)
    #[serde(default = "default_limit")]
    pub limit: u32,
}

/// Trending news arguments
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetTrendingArgs {
    /// Country code (default: us)
    #[serde(default = "default_country")]
    pub country: String,
    /// Number of articles (default: 10)
    #[serde(default = "default_limit")]
    pub limit: u32,
}

/// News article structure
#[derive(Debug, Serialize, Deserialize)]
struct NewsArticle {
    title: String,
    description: Option<String>,
    url: String,
    source: String,
    published_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// News data server
#[derive(Clone)]
pub struct NewsDataServer {
    /// Tool router for handling tool calls
    tool_router: ToolRouter<NewsDataServer>,
    /// Request statistics
    stats: Arc<Mutex<HashMap<String, u64>>>,
}

// Default value functions
fn default_limit() -> u32 {
    10
}
fn default_language() -> String {
    "en".to_string()
}
fn default_country() -> String {
    "us".to_string()
}

impl NewsDataServer {
    /// Create new news data server
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            stats: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Update request statistics
    async fn update_stats(&self, tool_name: &str) {
        let mut stats = self.stats.lock().await;
        *stats.entry(tool_name.to_string()).or_insert(0) += 1;
    }

    /// Mock news search for demonstration
    async fn mock_search_news(&self, args: &SearchNewsArgs) -> Vec<NewsArticle> {
        // Simulate API delay
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let articles = vec![
            NewsArticle {
                title: format!("Breaking: {} Latest Updates", args.query),
                description: Some("Mock news article for demonstration purposes.".to_string()),
                url: "https://example.com/news/1".to_string(),
                source: "Example News".to_string(),
                published_at: Some(chrono::Utc::now()),
            },
            NewsArticle {
                title: format!("Analysis: {} Market Impact", args.query),
                description: Some("Expert analysis on recent developments.".to_string()),
                url: "https://example.com/news/2".to_string(),
                source: "Market Analysis".to_string(),
                published_at: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
            },
        ];

        articles.into_iter().take(args.limit as usize).collect()
    }

    /// Format news articles for display
    fn format_articles(&self, articles: &[NewsArticle]) -> String {
        let mut result = String::new();

        for (i, article) in articles.iter().enumerate() {
            result.push_str(&format!("## {} - {}\n\n", i + 1, article.title));

            if let Some(description) = &article.description {
                result.push_str(&format!("**Description:** {}\n\n", description));
            }

            result.push_str(&format!("**Source:** {}\n", article.source));
            result.push_str(&format!("**URL:** {}\n", article.url));

            if let Some(published) = article.published_at {
                result.push_str(&format!(
                    "**Published:** {}\n",
                    published.format("%Y-%m-%d %H:%M UTC")
                ));
            }

            result.push_str("\n---\n\n");
        }

        result
    }
}

#[tool_router]
impl NewsDataServer {
    /// Search for news articles
    #[tool(description = "Search for news articles by query")]
    async fn search_news(
        &self,
        Parameters(args): Parameters<SearchNewsArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.update_stats("search_news").await;

        info!(
            "🔍 News search: query='{}', limit={}",
            args.query, args.limit
        );

        let articles = self.mock_search_news(&args).await;
        let formatted_results = self.format_articles(&articles);

        let summary = format!(
            "🔍 **News Search Results**\n\n**Query:** {}\n**Articles Found:** {}\n**Language:** {}\n\n{}",
            args.query, articles.len(), args.language, formatted_results
        );

        Ok(CallToolResult::success(vec![Content::text(summary)]))
    }

    /// Get news by category
    #[tool(description = "Get news articles by category")]
    async fn get_category_news(
        &self,
        Parameters(args): Parameters<GetCategoryArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.update_stats("get_category_news").await;

        info!(
            "📂 Category news: category='{}', limit={}",
            args.category, args.limit
        );

        let mock_articles = vec![NewsArticle {
            title: format!("Top {} News Today", args.category.to_uppercase()),
            description: Some(format!("Latest developments in {} sector.", args.category)),
            url: "https://example.com/category/1".to_string(),
            source: "Category News".to_string(),
            published_at: Some(chrono::Utc::now()),
        }];

        let limited_articles: Vec<NewsArticle> = mock_articles
            .into_iter()
            .take(args.limit as usize)
            .collect();

        let formatted_results = self.format_articles(&limited_articles);
        let summary = format!(
            "📂 **Category News: {}**\n\n**Articles Found:** {}\n\n{}",
            args.category.to_uppercase(),
            limited_articles.len(),
            formatted_results
        );

        Ok(CallToolResult::success(vec![Content::text(summary)]))
    }

    /// Get trending news
    #[tool(description = "Get trending news articles by country")]
    async fn get_trending_news(
        &self,
        Parameters(args): Parameters<GetTrendingArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.update_stats("get_trending_news").await;

        info!(
            "📈 Trending news: country='{}', limit={}",
            args.country, args.limit
        );

        let mock_articles = vec![NewsArticle {
            title: format!(
                "Trending in {}: Major Development",
                args.country.to_uppercase()
            ),
            description: Some("This story is currently trending nationwide.".to_string()),
            url: "https://example.com/trending/1".to_string(),
            source: "Trending News".to_string(),
            published_at: Some(chrono::Utc::now()),
        }];

        let limited_articles: Vec<NewsArticle> = mock_articles
            .into_iter()
            .take(args.limit as usize)
            .collect();

        let formatted_results = self.format_articles(&limited_articles);
        let summary = format!(
            "📈 **Trending News: {}**\n\n**Articles Found:** {}\n\n{}",
            args.country.to_uppercase(),
            limited_articles.len(),
            formatted_results
        );

        Ok(CallToolResult::success(vec![Content::text(summary)]))
    }

    /// Get server status
    #[tool(description = "Get server health status and usage statistics")]
    async fn get_server_status(&self) -> Result<CallToolResult, McpError> {
        self.update_stats("get_server_status").await;

        let stats = self.stats.lock().await;

        let mut status_parts = vec![
            "🟢 **Server Status: HEALTHY**".to_string(),
            format!(
                "⏱️ **Uptime:** Server Running (Started: {})",
                chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
            ),
        ];

        if !stats.is_empty() {
            status_parts.push("📊 **Usage Statistics:**".to_string());
            for (tool, count) in stats.iter() {
                status_parts.push(format!("  - {}: {} requests", tool, count));
            }
        }

        let status_report = status_parts.join("\n");

        Ok(CallToolResult::success(vec![Content::text(status_report)]))
    }

    /// Get available news categories
    #[tool(description = "Get list of available news categories")]
    async fn get_categories(&self) -> Result<CallToolResult, McpError> {
        self.update_stats("get_categories").await;

        let categories = vec![
            "general",
            "business",
            "entertainment",
            "health",
            "science",
            "sports",
            "technology",
            "politics",
            "world",
            "national",
        ];

        let category_list = categories
            .iter()
            .enumerate()
            .map(|(i, cat)| format!("{}. {}", i + 1, cat))
            .collect::<Vec<_>>()
            .join("\n");

        let result = format!(
            "📚 **Available News Categories**\n\n{}\n\nUse any of these categories with the `get_category_news` tool.",
            category_list
        );

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
}

#[tool_handler]
impl ServerHandler for NewsDataServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "📰 News Data Server - Real-time news and information:\n\
                • search_news: Search articles by query\n\
                • get_category_news: Get articles by category\n\
                • get_trending_news: Get trending articles by country\n\
                • get_categories: List available news categories\n\
                • get_server_status: Health check and usage statistics\n\n\
                🚀 Fast, lightweight implementation using official RMCP SDK"
                    .to_string(),
            ),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        info!("📰 News Data Server initialized successfully");
        info!("🚀 Lightweight build ready for MCP connections");
        Ok(self.get_info())
    }
}

impl Default for NewsDataServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Setup logging
    let log_level = if args.debug { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(format!("news_data_server={log_level}").parse()?)
                .add_directive(format!("rmcp={log_level}").parse()?),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    info!("🚀 Starting News Data Server using official RMCP SDK");
    info!("📰 Lightweight build - SDK Version: 0.6.3+");

    // Create server instance
    let server = NewsDataServer::new();

    // Start the server with STDIO transport
    let service = server.serve(stdio()).await.inspect_err(|e| {
        error!("Failed to start server: {:?}", e);
    })?;

    info!("✅ News Data Server started and ready for MCP connections");
    info!("🔗 Connect via MCP client using STDIO transport");

    // Wait for the service to complete
    service.waiting().await?;

    info!("Server shutdown complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let server = NewsDataServer::new();
        let info = server.get_info();

        assert!(info.capabilities.tools.is_some());
        assert!(info.instructions.is_some());
        assert!(info.instructions.unwrap().contains("News Data Server"));
    }

    #[tokio::test]
    async fn test_search_news_tool() {
        let server = NewsDataServer::new();
        let args = SearchNewsArgs {
            query: "technology".to_string(),
            limit: 5,
            language: "en".to_string(),
        };

        let result = server.search_news(Parameters(args)).await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("News Search Results"));
                assert!(text.text.contains("technology"));
            }
        }
    }

    #[tokio::test]
    async fn test_category_news_tool() {
        let server = NewsDataServer::new();
        let args = GetCategoryArgs {
            category: "technology".to_string(),
            limit: 3,
        };

        let result = server.get_category_news(Parameters(args)).await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("Category News: TECHNOLOGY"));
            }
        }
    }

    #[tokio::test]
    async fn test_trending_news_tool() {
        let server = NewsDataServer::new();
        let args = GetTrendingArgs {
            country: "us".to_string(),
            limit: 5,
        };

        let result = server.get_trending_news(Parameters(args)).await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("Trending News: US"));
            }
        }
    }

    #[tokio::test]
    async fn test_server_status_tool() {
        let server = NewsDataServer::new();

        let result = server.get_server_status().await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("Server Status: HEALTHY"));
            }
        }
    }

    #[tokio::test]
    async fn test_get_categories_tool() {
        let server = NewsDataServer::new();

        let result = server.get_categories().await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("Available News Categories"));
                assert!(text.text.contains("technology"));
                assert!(text.text.contains("business"));
            }
        }
    }

    #[tokio::test]
    async fn test_mock_search_functionality() {
        let server = NewsDataServer::new();
        let args = SearchNewsArgs {
            query: "test".to_string(),
            limit: 2,
            language: "en".to_string(),
        };

        let articles = server.mock_search_news(&args).await;
        assert_eq!(articles.len(), 2);
        assert!(articles[0].title.contains("test"));
    }
}
