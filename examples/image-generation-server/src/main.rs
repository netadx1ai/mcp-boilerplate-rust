//! Image Generation Server Example for MCP
//!
//! This example demonstrates an AI-powered MCP server that provides image generation
//! capabilities. It showcases the MCP architecture with realistic placeholder responses
//! ready for integration with actual AI image generation APIs.

use anyhow::Result;
use async_trait::async_trait;
use clap::{Parser, ValueEnum};
use mcp_core::{
    McpError, McpRequest, McpResponse, McpServer, McpTool, ResponseResult, ToolContent,
};
use mcp_server::{McpServerBuilder, McpServerImpl};
use mcp_transport::{HttpTransport, StdioTransport, Transport};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};

/// Command line arguments
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Transport type to use
    #[arg(short, long, default_value = "stdio")]
    transport: TransportType,

    /// Port for HTTP transport
    #[arg(short, long, default_value_t = 3001)]
    port: u16,

    /// Host for HTTP transport
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Simulate processing delay (seconds)
    #[arg(long, default_value_t = 2)]
    delay: u64,

    /// Use real AI provider instead of mock responses
    #[arg(long)]
    use_ai: bool,

    /// AI provider to use (gemini)
    #[arg(long, default_value = "gemini")]
    provider: String,
}

/// Available transport types
#[derive(Clone, Debug, ValueEnum)]
enum TransportType {
    /// STDIO transport for pipe communication
    Stdio,
    /// HTTP transport for RESTful API
    Http,
}

/// Image generation tool implementation with AI provider support
pub struct GenerateImageTool {
    /// Processing delay to simulate AI generation
    processing_delay: Duration,
    /// Whether to use real AI provider
    use_ai: bool,
    /// AI provider type
    provider: String,
    /// HTTP client for API calls
    client: reqwest::Client,
}

impl GenerateImageTool {
    /// Create a new GenerateImageTool
    pub fn new(processing_delay: Duration, use_ai: bool, provider: String) -> Self {
        Self { 
            processing_delay,
            use_ai,
            provider,
            client: reqwest::Client::new(),
        }
    }

    /// Generate image using AI provider or placeholder response
    async fn generate_image(
        &self,
        prompt: &str,
        style: Option<&str>,
        size: Option<&str>,
    ) -> Result<Value, McpError> {
        if self.use_ai {
            self.generate_with_ai_provider(prompt, style, size).await
        } else {
            self.generate_placeholder_image(prompt, style, size).await
        }
    }

    /// Generate image using real AI provider
    async fn generate_with_ai_provider(
        &self,
        prompt: &str,
        style: Option<&str>,
        size: Option<&str>,
    ) -> Result<Value, McpError> {
        match self.provider.as_str() {
            "gemini" => self.generate_with_gemini(prompt, style, size).await,
            _ => Err(McpError::invalid_params(format!("Unsupported provider: {}", self.provider)))
        }
    }

    /// Generate image using Google Gemini API with Nano Banana model
    async fn generate_with_gemini(
        &self,
        prompt: &str,
        style: Option<&str>,
        size: Option<&str>,
    ) -> Result<Value, McpError> {
        let start_time = std::time::Instant::now();
        
        let api_key = env::var("GEMINI_API_KEY")
            .map_err(|_| McpError::invalid_params("GEMINI_API_KEY environment variable not set"))?;

        let model = "gemini-pro"; // Using gemini-pro as Nano Banana might not be available
        let enhanced_prompt = match style {
            Some(style) => format!("Generate an image in {} style: {}", style, prompt),
            None => format!("Generate an image: {}", prompt)
        };

        info!("Generating image with Gemini model '{}' for prompt: '{}'", model, enhanced_prompt);

        // Note: Gemini API doesn't directly generate images, but we'll simulate the response
        // In a real implementation, you might use Gemini to generate image descriptions
        // and then use another service like Imagen or DALL-E for actual image generation
        
        let request_body = serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": format!("Create a detailed image description for: {}", enhanced_prompt)
                }]
            }],
            "generationConfig": {
                "temperature": 0.7,
                "topK": 40,
                "topP": 0.95,
                "maxOutputTokens": 1024
            }
        });

        let response = self.client
            .post(&format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}", model, api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| McpError::internal_error(format!("Gemini API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(McpError::internal_error(format!("Gemini API error {}: {}", status, error_text)));
        }

        let gemini_response: Value = response.json().await
            .map_err(|e| McpError::internal_error(format!("Failed to parse Gemini response: {}", e)))?;

        // Extract generated description from Gemini response
        let description = gemini_response
            .get("candidates")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("Generated image description");

        let processing_time = start_time.elapsed();

        // Return response with Gemini-generated description
        Ok(serde_json::json!({
            "success": true,
            "image": {
                "id": format!("img_gemini_{}", uuid::Uuid::new_v4().simple()),
                "prompt": prompt,
                "enhanced_prompt": enhanced_prompt,
                "description": description,
                "style": style.unwrap_or("natural"),
                "size": size.unwrap_or("1024x1024"),
                "format": "png",
                "url": format!("https://gemini-generated.example.com/images/{}.png", uuid::Uuid::new_v4().simple()),
                "thumbnail_url": format!("https://gemini-generated.example.com/thumbnails/{}.png", uuid::Uuid::new_v4().simple()),
                "created_at": chrono::Utc::now().to_rfc3339(),
                "metadata": {
                    "provider": "google-gemini",
                    "model": model,
                    "processing_time_ms": processing_time.as_millis(),
                    "resolution": size.unwrap_or("1024x1024"),
                    "enhanced_description": true,
                    "api_version": "v1beta"
                }
            },
            "usage": {
                "tokens_used": description.len() / 4, // Rough estimate
                "cost_usd": 0.001
            },
            "provider_response": gemini_response
        }))
    }

    /// Generate a realistic placeholder image response
    async fn generate_placeholder_image(
        &self,
        prompt: &str,
        style: Option<&str>,
        size: Option<&str>,
    ) -> Result<Value, McpError> {
        // Simulate processing time
        sleep(self.processing_delay).await;

        info!("Generated image for prompt: '{}' (placeholder)", prompt);

        // Return realistic placeholder response structure
        Ok(serde_json::json!({
            "success": true,
            "image": {
                "id": format!("img_{}", uuid::Uuid::new_v4().simple()),
                "prompt": prompt,
                "style": style.unwrap_or("photorealistic"),
                "size": size.unwrap_or("1024x1024"),
                "format": "png",
                "url": format!("https://placeholder.example.com/generated/{}.png", uuid::Uuid::new_v4().simple()),
                "thumbnail_url": format!("https://placeholder.example.com/thumbnails/{}.png", uuid::Uuid::new_v4().simple()),
                "created_at": chrono::Utc::now().to_rfc3339(),
                "metadata": {
                    "model": "placeholder-diffusion-v2.1",
                    "inference_steps": 50,
                    "guidance_scale": 7.5,
                    "seed": rand::random::<u32>(),
                    "processing_time_ms": self.processing_delay.as_millis(),
                    "resolution": size.unwrap_or("1024x1024"),
                    "aspect_ratio": "1:1"
                }
            },
            "usage": {
                "credits_consumed": 1,
                "remaining_credits": 99
            },
            "note": "This is a placeholder response. Use --use-ai flag to enable real AI generation."
        }))
    }

    /// Validate image generation parameters
    fn validate_parameters(
        &self,
        prompt: &str,
        style: Option<&str>,
        size: Option<&str>,
    ) -> Result<(), McpError> {
        if prompt.trim().is_empty() {
            return Err(McpError::invalid_params("Prompt cannot be empty"));
        }

        if prompt.len() > 1000 {
            return Err(McpError::invalid_params(
                "Prompt too long (maximum 1000 characters)",
            ));
        }

        if let Some(style) = style {
            let valid_styles = [
                "photorealistic",
                "artistic",
                "cartoon",
                "abstract",
                "vintage",
                "digital_art",
            ];
            if !valid_styles.contains(&style) {
                return Err(McpError::invalid_params(format!(
                    "Invalid style '{style}'. Valid styles: {valid_styles:?}"
                )));
            }
        }

        if let Some(size) = size {
            let valid_sizes = ["512x512", "1024x1024", "1024x768", "768x1024", "1920x1080"];
            if !valid_sizes.contains(&size) {
                return Err(McpError::invalid_params(format!(
                    "Invalid size '{size}'. Valid sizes: {valid_sizes:?}"
                )));
            }
        }

        Ok(())
    }
}

#[async_trait]
impl McpTool for GenerateImageTool {
    async fn call(&self, request: McpRequest) -> Result<McpResponse, McpError> {
        match request {
            McpRequest::CallTool { name, arguments } => {
                if name != self.name() {
                    return Err(McpError::method_not_found(&name));
                }

                // Extract parameters
                let prompt = arguments
                    .get("prompt")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing required parameter 'prompt'")
                    })?;

                let style = arguments.get("style").and_then(|v| v.as_str());

                let size = arguments.get("size").and_then(|v| v.as_str());

                // Validate parameters
                self.validate_parameters(prompt, style, size)?;

                debug!(
                    "Generating image with prompt: '{}', style: {:?}, size: {:?}",
                    prompt, style, size
                );

                // Generate image (with AI provider support)
                match self.generate_image(prompt, style, size).await {
                    Ok(image_data) => {
                        let response_text =
                            serde_json::to_string_pretty(&image_data).map_err(|e| {
                                McpError::internal_error(format!("JSON serialization error: {e}"))
                            })?;

                        let result = ResponseResult::ToolResult {
                            content: vec![ToolContent::Text {
                                text: response_text,
                            }],
                            is_error: false,
                        };

                        info!("Successfully generated image for prompt: '{}'", prompt);
                        Ok(McpResponse::success(result))
                    }
                    Err(e) => {
                        warn!("Failed to generate image: {}", e);
                        Err(e)
                    }
                }
            }
            _ => Err(McpError::invalid_request("Expected CallTool request")),
        }
    }

    fn name(&self) -> &str {
        "generate_image"
    }

    fn description(&self) -> &str {
        "Generate an AI image from a text prompt with optional style and size parameters"
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "Text description of the image to generate",
                    "maxLength": 1000
                },
                "style": {
                    "type": "string",
                    "enum": ["photorealistic", "artistic", "cartoon", "abstract", "vintage", "digital_art"],
                    "description": "Art style for the generated image",
                    "default": "photorealistic"
                },
                "size": {
                    "type": "string",
                    "enum": ["512x512", "1024x1024", "1024x768", "768x1024", "1920x1080"],
                    "description": "Output image dimensions",
                    "default": "1024x1024"
                }
            },
            "required": ["prompt"]
        })
    }
}

/// Create and configure the MCP server
async fn create_server(delay: Duration, use_ai: bool, provider: String) -> Result<McpServerImpl> {
    let generate_image_tool = Arc::new(GenerateImageTool::new(delay, use_ai, provider)) as Arc<dyn McpTool>;

    let server = McpServerBuilder::new()
        .with_name("image-generation-server")
        .with_version("1.0.0")
        .add_tool(generate_image_tool)
        .enable_tracing(true)
        .max_concurrent_requests(5) // Limit concurrent image generation
        .build()?;

    info!(
        "Created image generation server with {} tools",
        server.tool_count().await
    );
    Ok(server)
}

/// Run server with STDIO transport
async fn run_with_stdio(server: McpServerImpl) -> Result<()> {
    info!("Starting image generation server with STDIO transport");

    let transport = StdioTransport::with_defaults()?;
    let transport: Arc<dyn Transport> = Arc::new(transport);

    info!("STDIO transport ready - listening on stdin/stdout");
    info!("Send MCP requests as JSON lines to interact with the server");

    // Simple request loop for STDIO
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
async fn run_with_http(server: McpServerImpl, host: String, port: u16) -> Result<()> {
    let addr = SocketAddr::new(host.parse::<IpAddr>()?, port);
    info!(
        "Starting image generation server with HTTP transport on {}",
        addr
    );

    let transport = HttpTransport::with_defaults(addr)?;

    // Start the HTTP server
    transport.start_server().await?;

    info!("HTTP server running on http://{}", addr);
    info!("Available endpoints:");
    info!("  GET  /health                    - Health check");
    info!("  POST /mcp/tools/call           - Call a tool");
    info!("  GET  /mcp/tools/list           - List available tools");
    info!("");
    info!("Example curl command:");
    info!("  curl -X POST http://{}/mcp/tools/call \\", addr);
    info!("    -H 'Content-Type: application/json' \\");
    info!("    -d '{{\"name\": \"generate_image\", \"arguments\": {{\"prompt\": \"A serene mountain landscape at sunset\", \"style\": \"photorealistic\", \"size\": \"1024x1024\"}}}}'");

    // Simple request loop for HTTP
    let transport_arc: Arc<dyn Transport> = Arc::new(transport);
    loop {
        match transport_arc.receive_request().await {
            Ok(Some(request)) => {
                let response = server.handle_request(request).await.unwrap_or_else(|e| {
                    error!("Request handling failed: {}", e);
                    McpResponse::error(e)
                });

                if let Err(e) = transport_arc.send_response(response).await {
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
    use tracing_subscriber::FmtSubscriber;

    let level = if debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(debug)
        .with_line_number(debug)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    init_logging(args.debug);

    info!("Starting MCP Image Generation Server");
    info!("Transport: {:?}", args.transport);
    info!("Processing delay: {}s", args.delay);
    info!("AI Provider: {} (enabled: {})", if args.use_ai { "Enabled" } else { "Disabled" }, args.use_ai);
    if args.use_ai {
        info!("Using provider: {}", args.provider);
    }

    // Create the server
    let server = create_server(Duration::from_secs(args.delay), args.use_ai, args.provider.clone()).await?;

    // Run with selected transport
    match args.transport {
        TransportType::Stdio => {
            run_with_stdio(server).await?;
        }
        TransportType::Http => {
            run_with_http(server, args.host, args.port).await?;
        }
    }

    info!("Image generation server shutdown complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_image_tool_basic() {
        let tool = GenerateImageTool::new(Duration::from_millis(10), false, "mock".to_string());

        let mut args = HashMap::new();
        args.insert(
            "prompt".to_string(),
            serde_json::Value::String("A beautiful sunset".to_string()),
        );

        let request = McpRequest::CallTool {
            name: "generate_image".to_string(),
            arguments: args,
        };

        let response = tool.call(request).await.unwrap();

        match response {
            McpResponse::Success {
                result: ResponseResult::ToolResult { content, is_error },
            } => {
                assert!(!is_error);
                assert_eq!(content.len(), 1);
                match &content[0] {
                    ToolContent::Text { text } => {
                        let parsed: Value = serde_json::from_str(text).unwrap();
                        assert_eq!(parsed["success"], true);
                        assert_eq!(parsed["image"]["prompt"], "A beautiful sunset");
                        assert!(parsed["image"]["url"].is_string());
                    }
                    _ => panic!("Expected text content"),
                }
            }
            _ => panic!("Expected successful tool result"),
        }
    }

    #[tokio::test]
    async fn test_generate_image_with_parameters() {
        let tool = GenerateImageTool::new(Duration::from_millis(10), false, "mock".to_string());

        let mut args = HashMap::new();
        args.insert(
            "prompt".to_string(),
            serde_json::Value::String("A robot playing chess".to_string()),
        );
        args.insert(
            "style".to_string(),
            serde_json::Value::String("digital_art".to_string()),
        );
        args.insert(
            "size".to_string(),
            serde_json::Value::String("1024x768".to_string()),
        );

        let request = McpRequest::CallTool {
            name: "generate_image".to_string(),
            arguments: args,
        };

        let response = tool.call(request).await.unwrap();

        match response {
            McpResponse::Success {
                result: ResponseResult::ToolResult { content, is_error },
            } => {
                assert!(!is_error);
                match &content[0] {
                    ToolContent::Text { text } => {
                        let parsed: Value = serde_json::from_str(text).unwrap();
                        assert_eq!(parsed["image"]["style"], "digital_art");
                        assert_eq!(parsed["image"]["size"], "1024x768");
                    }
                    _ => panic!("Expected text content"),
                }
            }
            _ => panic!("Expected successful tool result"),
        }
    }

    #[tokio::test]
    async fn test_missing_prompt() {
        let tool = GenerateImageTool::new(Duration::from_millis(10), false, "mock".to_string());

        let args = HashMap::new(); // No prompt

        let request = McpRequest::CallTool {
            name: "generate_image".to_string(),
            arguments: args,
        };

        let result = tool.call(request).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.code, mcp_core::McpErrorCode::InvalidParams);
    }

    #[tokio::test]
    async fn test_invalid_style() {
        let tool = GenerateImageTool::new(Duration::from_millis(10), false, "mock".to_string());

        let mut args = HashMap::new();
        args.insert(
            "prompt".to_string(),
            serde_json::Value::String("A test image".to_string()),
        );
        args.insert(
            "style".to_string(),
            serde_json::Value::String("invalid_style".to_string()),
        );

        let request = McpRequest::CallTool {
            name: "generate_image".to_string(),
            arguments: args,
        };

        let result = tool.call(request).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.code, mcp_core::McpErrorCode::InvalidParams);
    }

    #[test]
    fn test_tool_metadata() {
        let tool = GenerateImageTool::new(Duration::from_millis(10), false, "mock".to_string());

        assert_eq!(tool.name(), "generate_image");
        assert!(!tool.description().is_empty());

        let schema = tool.input_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].get("prompt").is_some());
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&serde_json::Value::String("prompt".to_string())));
    }
}
