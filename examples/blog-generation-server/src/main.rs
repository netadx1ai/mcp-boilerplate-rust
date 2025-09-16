//! Blog generation server example for MCP
//! 
//! This example demonstrates an AI-powered MCP server that provides blog generation capabilities.
//! The server can generate blog posts with various topics, styles, and lengths.

use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::{Parser, ValueEnum};
use mcp_core::{McpError, McpRequest, McpResponse, McpTool, McpServer, ResponseResult, ToolContent};
use mcp_server::McpServerBuilder;
use mcp_transport::{HttpTransport, StdioTransport, Transport};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info};

#[derive(Parser)]
#[command(name = "blog-generation-server")]
#[command(about = "MCP blog generation server example")]
#[command(version = "0.1.0")]
struct Args {
    /// Transport type to use
    #[arg(short, long, value_enum, default_value_t = TransportType::Stdio)]
    transport: TransportType,

    /// Port for HTTP transport
    #[arg(short, long, default_value_t = 3002)]
    port: u16,

    /// Host for HTTP transport  
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Artificial delay in seconds for blog generation (simulates AI processing time)
    #[arg(long, default_value_t = 2)]
    delay: u64,
}

#[derive(Clone, ValueEnum, Debug)]
enum TransportType {
    /// Use STDIO transport
    Stdio,
    /// Use HTTP transport
    Http,
}

/// Blog generation tool that creates blog posts based on user requirements
pub struct CreateBlogPostTool {
    processing_delay: Duration,
}

impl CreateBlogPostTool {
    pub fn new(processing_delay: Duration) -> Self {
        Self { processing_delay }
    }

    /// Generate a realistic placeholder blog post response
    /// In a real implementation, this would integrate with AI APIs like OpenAI, Claude, etc.
    async fn generate_placeholder_blog(&self, parameters: &HashMap<String, Value>) -> Result<Value, McpError> {
        info!("Generating blog post with parameters: {:?}", parameters);
        
        // Simulate processing time
        sleep(self.processing_delay).await;
        
        let topic = parameters.get("topic")
            .and_then(|v| v.as_str())
            .unwrap_or("Technology");
            
        let style = parameters.get("style")
            .and_then(|v| v.as_str())
            .unwrap_or("professional");
            
        let word_count = parameters.get("word_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(800);

        // Generate realistic placeholder content based on parameters
        let (title, content) = match topic.to_lowercase().as_str() {
            "technology" => (
                "The Future of Artificial Intelligence: Transforming Industries in 2024".to_string(),
                generate_tech_content(word_count, style)
            ),
            "business" => (
                "Strategic Innovation: Building Resilient Business Models for Tomorrow".to_string(),
                generate_business_content(word_count, style)
            ),
            "health" => (
                "Wellness in the Digital Age: Balancing Technology and Human Well-being".to_string(),
                generate_health_content(word_count, style)
            ),
            "education" => (
                "Educational Technology: Reshaping Learning in the 21st Century".to_string(),
                generate_education_content(word_count, style)
            ),
            _ => (
                format!("Exploring {}: A Comprehensive Guide", topic),
                generate_generic_content(topic, word_count, style)
            ),
        };

        Ok(json!({
            "blog_post": {
                "title": title,
                "content": content,
                "metadata": {
                    "topic": topic,
                    "style": style,
                    "estimated_word_count": word_count,
                    "generated_at": chrono::Utc::now().to_rfc3339(),
                    "ai_model": "placeholder-blog-generator-v1.0",
                    "content_type": "blog_post"
                },
                "seo": {
                    "meta_description": format!("Discover insights about {} in this comprehensive blog post covering key trends and practical applications.", topic),
                    "keywords": generate_keywords(topic),
                    "reading_time_minutes": (word_count / 200).max(1) // Average reading speed
                }
            },
            "generation_stats": {
                "processing_time_seconds": self.processing_delay.as_secs(),
                "model_confidence": 0.92,
                "content_quality_score": 0.89
            }
        }))
    }

    /// Validate input parameters for blog generation
    fn validate_parameters(&self, parameters: &HashMap<String, Value>) -> Result<(), McpError> {
        // Validate word count if provided
        if let Some(word_count) = parameters.get("word_count") {
            match word_count.as_u64() {
                Some(count) if count < 100 => {
                    return Err(McpError::invalid_params(
                        "word_count must be at least 100 words for a meaningful blog post"
                    ));
                }
                Some(count) if count > 5000 => {
                    return Err(McpError::invalid_params(
                        "word_count must be 5000 words or less to ensure readability"
                    ));
                }
                Some(_) => {} // Valid word count
                None => {
                    return Err(McpError::invalid_params(
                        "word_count must be a positive integer"
                    ));
                }
            }
        }

        // Validate style if provided
        if let Some(style) = parameters.get("style") {
            if let Some(style_str) = style.as_str() {
                let valid_styles = ["professional", "casual", "academic", "creative", "technical", "conversational"];
                if !valid_styles.contains(&style_str.to_lowercase().as_str()) {
                    return Err(McpError::invalid_params(
                        format!("style must be one of: {}", valid_styles.join(", "))
                    ));
                }
            } else {
                return Err(McpError::invalid_params(
                    "style must be a string"
                ));
            }
        }

        Ok(())
    }
}

#[async_trait]
impl McpTool for CreateBlogPostTool {
    async fn call(&self, request: McpRequest) -> Result<McpResponse, McpError> {
        match request {
            McpRequest::CallTool { name, arguments } => {
                if name != self.name() {
                    return Err(McpError::method_not_found(&name));
                }
                
                debug!("CreateBlogPostTool called with arguments: {:?}", arguments);

                // Validate parameters
                self.validate_parameters(&arguments)?;

                // Check for required topic parameter
                if arguments.get("topic").is_none() {
                    return Err(McpError::invalid_params(
                        "Missing required parameter: 'topic' - please specify what the blog post should be about"
                    ));
                }

                // Generate the blog post
                match self.generate_placeholder_blog(&arguments).await {
                    Ok(result) => {
                        info!("Successfully generated blog post");
                        let content = ToolContent::Text { 
                            text: serde_json::to_string_pretty(&result)
                                .unwrap_or_else(|_| "Error serializing blog post".to_string())
                        };
                        let response_result = ResponseResult::ToolResult {
                            content: vec![content],
                            is_error: false,
                        };
                        Ok(McpResponse::success(response_result))
                    }
                    Err(e) => {
                        error!("Failed to generate blog post: {}", e);
                        Err(e)
                    }
                }
            }
            _ => Err(McpError::invalid_request("Expected CallTool request")),
        }
    }

    fn name(&self) -> &'static str {
        "create_blog_post"
    }

    fn description(&self) -> &'static str {
        "Generate a blog post on a specified topic with customizable style and length. Perfect for content creation, SEO optimization, and thought leadership."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "topic": {
                    "type": "string",
                    "description": "The main topic or subject for the blog post (required)"
                },
                "style": {
                    "type": "string",
                    "description": "Writing style for the blog post",
                    "enum": ["professional", "casual", "academic", "creative", "technical", "conversational"],
                    "default": "professional"
                },
                "word_count": {
                    "type": "integer",
                    "description": "Target word count for the blog post (100-5000 words)",
                    "minimum": 100,
                    "maximum": 5000,
                    "default": 800
                },
                "target_audience": {
                    "type": "string",
                    "description": "Target audience for the blog post (optional)",
                    "default": "general"
                }
            },
            "required": ["topic"]
        })
    }
}

// Helper functions for generating content
fn generate_tech_content(word_count: u64, style: &str) -> String {
    let base_content = match style {
        "academic" => "Artificial Intelligence represents a paradigm shift in computational capabilities, fundamentally altering the landscape of technological innovation. Recent advances in machine learning algorithms, particularly in deep learning architectures, have demonstrated unprecedented capabilities in pattern recognition, natural language processing, and predictive analytics.",
        "casual" => "AI is everywhere these days! From your smartphone's voice assistant to recommendation systems on streaming platforms, artificial intelligence is quietly revolutionizing how we interact with technology. It's pretty amazing when you think about it - machines that can learn and adapt just like humans do.",
        _ => "Artificial Intelligence continues to reshape industries across the globe, offering unprecedented opportunities for innovation and efficiency. Organizations that embrace AI technologies are discovering new ways to enhance customer experiences, streamline operations, and drive competitive advantage in an increasingly digital marketplace."
    };
    
    // Simulate content expansion based on word count
    let paragraphs = (word_count / 150).max(1);
    let mut content = String::new();
    
    for i in 0..paragraphs {
        if i > 0 { content.push_str("\n\n"); }
        content.push_str(&format!("{} [Content continues with detailed analysis and insights...]", base_content));
    }
    
    content
}

fn generate_business_content(word_count: u64, style: &str) -> String {
    let base_content = match style {
        "academic" => "Contemporary business environments are characterized by unprecedented volatility, uncertainty, complexity, and ambiguity (VUCA). Strategic frameworks must therefore incorporate adaptive mechanisms that enable organizational resilience and sustainable competitive positioning.",
        "casual" => "Running a business today feels like navigating through a storm sometimes. The market changes so quickly that what worked yesterday might not work tomorrow. But here's the thing - the companies that thrive are the ones that stay flexible and keep their customers at the heart of everything they do.",
        _ => "Modern businesses face unique challenges that require innovative solutions and strategic thinking. Success in today's marketplace demands agility, customer-centricity, and a deep understanding of emerging trends that shape consumer behavior and market dynamics."
    };
    
    let paragraphs = (word_count / 150).max(1);
    let mut content = String::new();
    
    for i in 0..paragraphs {
        if i > 0 { content.push_str("\n\n"); }
        content.push_str(&format!("{} [Content continues with strategic insights and practical applications...]", base_content));
    }
    
    content
}

fn generate_health_content(word_count: u64, style: &str) -> String {
    let base_content = match style {
        "academic" => "The intersection of digital technology and human wellness presents both opportunities and challenges for public health outcomes. Evidence-based research indicates that technology-mediated interventions can significantly improve health behaviors when designed with user-centered principles.",
        "casual" => "Staying healthy in our digital world is all about finding the right balance. Sure, we're spending more time looking at screens, but we're also using apps to track our fitness, connect with healthcare providers, and learn about wellness in ways that weren't possible before.",
        _ => "Digital wellness has become a critical component of overall health as we navigate an increasingly connected world. The key lies in leveraging technology to enhance rather than diminish our physical and mental well-being through mindful integration of digital tools."
    };
    
    let paragraphs = (word_count / 150).max(1);
    let mut content = String::new();
    
    for i in 0..paragraphs {
        if i > 0 { content.push_str("\n\n"); }
        content.push_str(&format!("{} [Content continues with health insights and practical recommendations...]", base_content));
    }
    
    content
}

fn generate_education_content(word_count: u64, style: &str) -> String {
    let base_content = match style {
        "academic" => "Educational technology integration represents a fundamental transformation in pedagogical approaches, necessitating evidence-based methodologies that enhance learning outcomes while addressing diverse learner needs and technological accessibility challenges.",
        "casual" => "Education is getting a major tech makeover! From online learning platforms to interactive whiteboards, students today have access to learning tools that make education more engaging and personalized than ever before. It's pretty exciting to see how technology is opening up new possibilities for learners everywhere.",
        _ => "The integration of technology in education is transforming how students learn and teachers teach. Modern educational approaches leverage digital tools to create more engaging, personalized, and effective learning experiences that prepare students for success in a technology-driven world."
    };
    
    let paragraphs = (word_count / 150).max(1);
    let mut content = String::new();
    
    for i in 0..paragraphs {
        if i > 0 { content.push_str("\n\n"); }
        content.push_str(&format!("{} [Content continues with educational insights and innovative approaches...]", base_content));
    }
    
    content
}

fn generate_generic_content(topic: &str, word_count: u64, style: &str) -> String {
    let base_content = match style {
        "academic" => format!("The study of {} presents multifaceted considerations that require systematic analysis and evidence-based evaluation. Current research in this domain reveals significant implications for both theoretical understanding and practical applications.", topic),
        "casual" => format!("Let's dive into the fascinating world of {}! There's so much to explore and understand about this topic, and I think you'll find some really interesting insights that might change how you think about it.", topic),
        _ => format!("Understanding {} is essential in today's rapidly evolving landscape. This comprehensive exploration examines key aspects, current trends, and practical implications that matter to professionals and enthusiasts alike.", topic)
    };
    
    let paragraphs = (word_count / 150).max(1);
    let mut content = String::new();
    
    for i in 0..paragraphs {
        if i > 0 { content.push_str("\n\n"); }
        content.push_str(&format!("{} [Content continues with detailed analysis and expert perspectives...]", base_content));
    }
    
    content
}

fn generate_keywords(topic: &str) -> Vec<String> {
    let base_keywords = vec![topic.to_lowercase()];
    let additional_keywords = match topic.to_lowercase().as_str() {
        "technology" => vec!["innovation", "digital transformation", "AI", "automation", "future trends"],
        "business" => vec!["strategy", "growth", "leadership", "market analysis", "competitive advantage"],
        "health" => vec!["wellness", "healthcare", "medical", "fitness", "mental health"],
        "education" => vec!["learning", "teaching", "students", "academic", "educational technology"],
        _ => vec!["insights", "analysis", "trends", "best practices", "expert advice"],
    };
    
    [base_keywords, additional_keywords.into_iter().map(String::from).collect()].concat()
}

/// Create and configure the MCP server with blog generation tools
async fn create_server(processing_delay: Duration) -> Result<mcp_server::McpServerImpl> {
    info!("Creating blog generation server...");
    
    let blog_tool: Arc<dyn McpTool> = Arc::new(CreateBlogPostTool::new(processing_delay));
    
    let server = McpServerBuilder::new()
        .with_name("blog-generation-server")
        .with_version("1.0.0")
        .add_tool(blog_tool)
        .enable_tracing(true)
        .max_concurrent_requests(10)
        .build()?;
    
    info!("Created blog generation server with {} tools", server.tool_count().await);
    Ok(server)
}

/// Run server with STDIO transport
async fn run_with_stdio(server: mcp_server::McpServerImpl) -> Result<()> {
    info!("Starting blog generation server with STDIO transport");
    
    let transport = StdioTransport::with_defaults()?;
    let transport: Arc<dyn Transport> = Arc::new(transport);
    
    info!("STDIO transport ready - listening on stdin/stdout");
    info!("Send MCP requests to generate blog posts on various topics");
    info!("Example: {{\"method\": \"create_blog_post\", \"params\": {{\"topic\": \"AI trends\", \"style\": \"professional\", \"word_count\": 1000}}}}");
    
    // Simple request loop
    loop {
        match transport.receive_request().await {
            Ok(Some(request)) => {
                let response = server.handle_request(request).await.unwrap_or_else(|e| {
                    error!("Request handling failed: {}", e);
                    McpResponse::error(e)
                });
                
                if let Err(e) = transport.send_response(response).await {
                    error!("Failed to send response: {}", e);
                    break;
                }
            }
            Ok(None) => {
                info!("Transport closed, stopping server");
                break;
            }
            Err(e) => {
                error!("Transport error: {}", e);
                break;
            }
        }
    }
    Ok(())
}

/// Run server with HTTP transport  
async fn run_with_http(server: mcp_server::McpServerImpl, host: String, port: u16) -> Result<()> {
    let addr = SocketAddr::new(host.parse::<IpAddr>()?, port);
    
    info!("Starting blog generation server with HTTP transport on {}...", addr);
    
    let transport = HttpTransport::with_defaults(addr)?;
    let transport: Arc<dyn Transport> = Arc::new(transport);
    
    info!("Blog generation server is ready!");
    info!("HTTP endpoint: http://{}/mcp", addr);
    info!("Health check: http://{}/health", addr);
    info!("");
    info!("Example curl request:");
    info!("curl -X POST http://{}/mcp \\", addr);
    info!("  -H 'Content-Type: application/json' \\");
    info!("  -d '{{");
    info!("    \"method\": \"create_blog_post\",");
    info!("    \"params\": {{");
    info!("      \"topic\": \"sustainable technology\",");
    info!("      \"style\": \"professional\",");
    info!("      \"word_count\": 1200");
    info!("    }}");
    info!("  }}'");
    
    // Simple request loop for HTTP
    loop {
        match transport.receive_request().await {
            Ok(Some(request)) => {
                let response = server.handle_request(request).await.unwrap_or_else(|e| {
                    error!("Request handling failed: {}", e);
                    McpResponse::error(e)
                });
                
                if let Err(e) = transport.send_response(response).await {
                    error!("Failed to send response: {}", e);
                    break;
                }
            }
            Ok(None) => {
                info!("Transport closed, stopping server");
                break;
            }
            Err(e) => {
                error!("Transport error: {}", e);
                break;
            }
        }
    }
    Ok(())
}

/// Initialize logging based on debug flag
fn init_logging(debug: bool) {
    let level = if debug { "debug" } else { "info" };
    
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    tracing_subscriber::EnvFilter::new(format!("blog_generation_server={},mcp_server={},mcp_transport={},mcp_core={}", level, level, level, level))
                })
        )
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    init_logging(args.debug);
    
    info!("MCP Blog Generation Server v0.1.0");
    info!("Transport: {:?}", args.transport);
    
    let processing_delay = Duration::from_secs(args.delay);
    let server = create_server(processing_delay).await?;
    
    match args.transport {
        TransportType::Stdio => run_with_stdio(server).await,
        TransportType::Http => run_with_http(server, args.host, args.port).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_create_blog_post_tool_basic() {
        let tool = CreateBlogPostTool::new(Duration::from_millis(10));
        let mut params = HashMap::new();
        params.insert("topic".to_string(), json!("technology"));
        
        let result = tool.call(params).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        let blog_post = response.get("blog_post").unwrap();
        assert!(blog_post.get("title").is_some());
        assert!(blog_post.get("content").is_some());
        assert!(blog_post.get("metadata").is_some());
    }

    #[tokio::test]
    async fn test_create_blog_post_with_parameters() {
        let tool = CreateBlogPostTool::new(Duration::from_millis(10));
        let mut params = HashMap::new();
        params.insert("topic".to_string(), json!("business"));
        params.insert("style".to_string(), json!("casual"));
        params.insert("word_count".to_string(), json!(500));
        
        let result = tool.call(params).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        let metadata = response.get("blog_post").unwrap().get("metadata").unwrap();
        assert_eq!(metadata.get("topic").unwrap(), "business");
        assert_eq!(metadata.get("style").unwrap(), "casual");
    }

    #[tokio::test]
    async fn test_missing_topic() {
        let tool = CreateBlogPostTool::new(Duration::from_millis(10));
        let params = HashMap::new(); // No topic provided
        
        let result = tool.call(params).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing required parameter: 'topic'"));
    }

    #[tokio::test] 
    async fn test_invalid_word_count() {
        let tool = CreateBlogPostTool::new(Duration::from_millis(10));
        let mut params = HashMap::new();
        params.insert("topic".to_string(), json!("test"));
        params.insert("word_count".to_string(), json!(50)); // Too low
        
        let result = tool.call(params).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("word_count must be at least 100"));
    }

    #[test]
    fn test_tool_metadata() {
        let tool = CreateBlogPostTool::new(Duration::from_millis(10));
        assert_eq!(tool.name(), "create_blog_post");
        assert!(!tool.description().is_empty());
        assert!(tool.input_schema().get("properties").is_some());
    }
}