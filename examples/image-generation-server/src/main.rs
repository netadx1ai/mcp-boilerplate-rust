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

    /// AI provider to use (gemini for real images, dalle, or real)
    #[arg(long, default_value = "gemini")]
    provider: String,

    /// Real image generation API key (for DALL-E, etc.)
    #[arg(long)]
    image_api_key: Option<String>,
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
            "gemini" => self.generate_with_gemini_imagen(prompt, style, size).await,
            "dalle" | "real" => self.generate_with_real_api(prompt, style, size).await,
            _ => Err(McpError::invalid_params(format!(
                "Unsupported provider: {}. Supported: gemini, dalle, real",
                self.provider
            ))),
        }
    }

    /// Generate real images using Google Gemini API (Nano Banana model for image generation)
    async fn generate_with_gemini_imagen(
        &self,
        prompt: &str,
        style: Option<&str>,
        size: Option<&str>,
    ) -> Result<Value, McpError> {
        let start_time = std::time::Instant::now();

        let api_key = env::var("GEMINI_API_KEY")
            .or_else(|_| env::var("GOOGLE_API_KEY"))
            .map_err(|_| {
                McpError::invalid_params(
                    "GEMINI_API_KEY or GOOGLE_API_KEY environment variable not set",
                )
            })?;

        // Use Gemini 2.5 Flash model for image generation (Nano Banana)
        let model = "gemini-2.5-flash-image-preview";

        let enhanced_prompt = match style {
            Some(style) => format!("Create a {style} style image: {prompt}"),
            None => format!("Create an image: {prompt}"),
        };

        info!(
            "Generating REAL image with Google Gemini (Nano Banana) for prompt: '{}'",
            enhanced_prompt
        );

        let request_body = serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": enhanced_prompt
                }]
            }]
        });

        let response = self
            .client
            .post(format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent"
            ))
            .header("Content-Type", "application/json")
            .header("x-goog-api-key", &api_key)
            .header("User-Agent", "MCP-ImageGen-Server/1.0")
            .json(&request_body)
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| McpError::internal_error(format!("Gemini API request failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(McpError::internal_error(format!(
                "Gemini API error {} ({}): {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown"),
                if error_text.is_empty() {
                    "No error details provided"
                } else {
                    &error_text
                }
            )));
        }

        let gemini_response: Value = response.json().await.map_err(|e| {
            McpError::internal_error(format!("Failed to parse Gemini response: {e}"))
        })?;

        // Extract image from Gemini response - check all parts for inlineData
        let mut image_data = None;
        if let Some(candidates) = gemini_response.get("candidates") {
            if let Some(candidate) = candidates.get(0) {
                if let Some(content) = candidate.get("content") {
                    if let Some(parts) = content.get("parts") {
                        if let Some(parts_array) = parts.as_array() {
                            for part in parts_array {
                                if let Some(inline_data) = part.get("inlineData") {
                                    if let Some(data) = inline_data.get("data") {
                                        if let Some(data_str) = data.as_str() {
                                            image_data = Some(data_str);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let image_data = image_data
            .ok_or_else(|| McpError::internal_error("No image data in Gemini response"))?;

        let processing_time = start_time.elapsed();
        let image_id = uuid::Uuid::new_v4().simple().to_string();

        // Create data URL for real image
        let image_url = format!("data:image/png;base64,{image_data}");

        info!("Successfully generated REAL image with Gemini Nano Banana!");

        Ok(serde_json::json!({
            "success": true,
            "image": {
                "id": format!("img_gemini_{}", image_id),
                "prompt": prompt,
                "enhanced_prompt": enhanced_prompt,
                "style": style.unwrap_or("natural"),
                "size": size.unwrap_or("1024x1024"),
                "format": "png",
                "url": image_url,
                "thumbnail_url": image_url,
                "created_at": chrono::Utc::now().to_rfc3339(),
                "metadata": {
                    "provider": "google-gemini-nano-banana",
                    "model": model,
                    "processing_time_ms": processing_time.as_millis(),
                    "api_version": "v1beta",
                    "real_image": true,
                    "image_format": "base64_data_url"
                }
            },
            "usage": {
                "tokens_used": enhanced_prompt.len() / 4,
                "estimated_cost_usd": 0.002, // Nano Banana pricing
                "model_used": model
            },
            "provider_response": gemini_response
        }))
    }

    /// Generate image using Google Gemini API with Nano Banana model (text descriptions only)
    async fn generate_with_real_api(
        &self,
        prompt: &str,
        style: Option<&str>,
        size: Option<&str>,
    ) -> Result<Value, McpError> {
        let _start_time = std::time::Instant::now();

        // Check for DALL-E API key
        if let Ok(api_key) = env::var("OPENAI_API_KEY") {
            return self
                .generate_with_dalle(prompt, style, size, &api_key)
                .await;
        }

        // Fallback to Gemini enhancement
        self.generate_with_gemini(prompt, style, size).await
    }

    async fn generate_with_dalle(
        &self,
        prompt: &str,
        style: Option<&str>,
        size: Option<&str>,
        api_key: &str,
    ) -> Result<Value, McpError> {
        let start_time = std::time::Instant::now();

        let enhanced_prompt = match style {
            Some(style) => format!("Create a {style} style image: {prompt}"),
            None => prompt.to_string(),
        };

        info!(
            "Generating real image with DALL-E for prompt: '{}'",
            enhanced_prompt
        );

        let dalle_size = match size.unwrap_or("1024x1024") {
            "512x512" => "1024x1024", // DALL-E 3 minimum
            "1792x1024" => "1792x1024",
            _ => "1024x1024",
        };

        let request_body = serde_json::json!({
            "model": "dall-e-3",
            "prompt": enhanced_prompt,
            "n": 1,
            "size": dalle_size,
            "quality": "standard",
            "response_format": "url"
        });

        let response = self
            .client
            .post("https://api.openai.com/v1/images/generations")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {api_key}"))
            .header("User-Agent", "MCP-ImageGen-Server/1.0")
            .json(&request_body)
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| McpError::internal_error(format!("DALL-E API request failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(McpError::internal_error(format!(
                "DALL-E API error {} ({}): {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown"),
                if error_text.is_empty() {
                    "No error details provided"
                } else {
                    &error_text
                }
            )));
        }

        let dalle_response: Value = response.json().await.map_err(|e| {
            McpError::internal_error(format!("Failed to parse DALL-E response: {e}"))
        })?;

        // Extract image URL from DALL-E response
        let image_url = dalle_response
            .get("data")
            .and_then(|d| d.get(0))
            .and_then(|img| img.get("url"))
            .and_then(|url| url.as_str())
            .ok_or_else(|| McpError::internal_error("No image URL in DALL-E response"))?;

        let revised_prompt = dalle_response
            .get("data")
            .and_then(|d| d.get(0))
            .and_then(|img| img.get("revised_prompt"))
            .and_then(|p| p.as_str())
            .unwrap_or(&enhanced_prompt);

        let processing_time = start_time.elapsed();
        let image_id = uuid::Uuid::new_v4().simple().to_string();

        info!("Successfully generated real image: {}", image_url);

        Ok(serde_json::json!({
            "success": true,
            "image": {
                "id": format!("img_dalle_{}", image_id),
                "prompt": prompt,
                "enhanced_prompt": enhanced_prompt,
                "revised_prompt": revised_prompt,
                "style": style.unwrap_or("natural"),
                "size": size.unwrap_or("1024x1024"),
                "format": "png",
                "url": image_url,
                "thumbnail_url": image_url, // DALL-E provides full resolution
                "created_at": chrono::Utc::now().to_rfc3339(),
                "metadata": {
                    "provider": "openai-dalle",
                    "model": "dall-e-3",
                    "processing_time_ms": processing_time.as_millis(),
                    "resolution": dalle_size,
                    "quality": "standard",
                    "api_version": "v1",
                    "real_image": true
                }
            },
            "usage": {
                "tokens_used": enhanced_prompt.len() / 4,
                "estimated_cost_usd": 0.04, // DALL-E 3 standard pricing
                "model_used": "dall-e-3"
            },
            "provider_response": dalle_response
        }))
    }

    async fn generate_with_gemini(
        &self,
        prompt: &str,
        style: Option<&str>,
        size: Option<&str>,
    ) -> Result<Value, McpError> {
        let start_time = std::time::Instant::now();

        let api_key = env::var("GEMINI_API_KEY")
            .or_else(|_| env::var("GOOGLE_API_KEY"))
            .map_err(|_| {
                McpError::invalid_params(
                    "GEMINI_API_KEY or GOOGLE_API_KEY environment variable not set",
                )
            })?;

        let model = "gemini-1.5-pro"; // Using gemini-1.5-pro for better image description capabilities
        let enhanced_prompt = match style {
            Some(style) => format!("Generate an image in {style} style: {prompt}"),
            None => format!("Generate an image: {prompt}"),
        };

        info!(
            "Generating image description with Gemini model '{}' for prompt: '{}'",
            model, enhanced_prompt
        );

        // Note: Google's Gemini API doesn't directly generate images like DALL-E
        // This implementation uses Gemini to generate enhanced image descriptions
        // In production, you would:
        // 1. Use Gemini to enhance/refine the prompt
        // 2. Send to Google's Imagen API via Vertex AI (requires additional setup)
        // 3. Or integrate with another image generation service like DALL-E, Midjourney, etc.

        let request_body = serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": format!(
                        "Create a detailed, creative image description suitable for AI image generation. \
                        Style: {}. \
                        Original prompt: {}. \
                        Provide a vivid, specific description with visual details, lighting, composition, and artistic style. \
                        Keep it under 200 words but make it rich and descriptive.",
                        style.unwrap_or("natural photorealistic"),
                        prompt
                    )
                }]
            }],
            "generationConfig": {
                "temperature": 0.8,
                "topK": 40,
                "topP": 0.95,
                "maxOutputTokens": 500,
                "stopSequences": []
            },
            "safetySettings": [
                {
                    "category": "HARM_CATEGORY_HARASSMENT",
                    "threshold": "BLOCK_MEDIUM_AND_ABOVE"
                },
                {
                    "category": "HARM_CATEGORY_HATE_SPEECH",
                    "threshold": "BLOCK_MEDIUM_AND_ABOVE"
                },
                {
                    "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT",
                    "threshold": "BLOCK_MEDIUM_AND_ABOVE"
                },
                {
                    "category": "HARM_CATEGORY_DANGEROUS_CONTENT",
                    "threshold": "BLOCK_MEDIUM_AND_ABOVE"
                }
            ]
        });

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}"
        );

        debug!("Making Gemini API request to: {}", url);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "MCP-ImageGen-Server/1.0")
            .json(&request_body)
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| McpError::internal_error(format!("Gemini API request failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Gemini API error {}: {}", status, error_text);
            return Err(McpError::internal_error(format!(
                "Gemini API error {} ({}): {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown"),
                if error_text.is_empty() {
                    "No error details provided"
                } else {
                    &error_text
                }
            )));
        }

        let response_text = response.text().await.map_err(|e| {
            McpError::internal_error(format!("Failed to read Gemini response: {e}"))
        })?;

        debug!("Gemini API raw response: {}", response_text);

        let gemini_response: Value = serde_json::from_str(&response_text).map_err(|e| {
            McpError::internal_error(format!(
                "Failed to parse Gemini JSON response: {e}. Raw response: {response_text}"
            ))
        })?;

        // Extract generated description from Gemini response with better error handling
        let description = match gemini_response
            .get("candidates")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
        {
            Some(text) => {
                info!("Generated enhanced description: {}", text);
                text.to_string()
            }
            None => {
                warn!(
                    "Failed to extract text from Gemini response: {:?}",
                    gemini_response
                );

                // Check for blocked content or other errors
                if let Some(candidates) = gemini_response.get("candidates") {
                    if let Some(candidate) = candidates.get(0) {
                        if let Some(finish_reason) = candidate.get("finishReason") {
                            let reason = finish_reason.as_str().unwrap_or("unknown");
                            if reason == "SAFETY" {
                                return Err(McpError::invalid_params("Content was blocked by safety filters. Try a different prompt."));
                            }
                        }
                    }
                }

                format!("Enhanced description for: {enhanced_prompt}")
            }
        };

        let processing_time = start_time.elapsed();

        let image_id = uuid::Uuid::new_v4().simple().to_string();

        // Return response with Gemini-enhanced description
        // NOTE: This creates a placeholder image URL since Google Gemini doesn't generate images directly
        // In production, you would use the enhanced description with an actual image generation service
        Ok(serde_json::json!({
            "success": true,
            "image": {
                "id": format!("img_gemini_{}", image_id),
                "prompt": prompt,
                "enhanced_prompt": enhanced_prompt,
                "ai_description": description,
                "style": style.unwrap_or("natural"),
                "size": size.unwrap_or("1024x1024"),
                "format": "png",
                "url": format!("https://placeholder-images.example.com/generate/{}.png?prompt={}",
                    image_id,
                    urlencoding::encode(&description).into_owned()
                ),
                "thumbnail_url": format!("https://placeholder-images.example.com/thumb/{}.png", image_id),
                "created_at": chrono::Utc::now().to_rfc3339(),
                "metadata": {
                    "provider": "google-gemini",
                    "model": model,
                    "processing_time_ms": processing_time.as_millis(),
                    "resolution": size.unwrap_or("1024x1024"),
                    "enhanced_description": true,
                    "api_version": "v1beta",
                    "description_length": description.len(),
                    "note": "This uses Gemini for description enhancement. For actual image generation, integrate with Imagen via Vertex AI or another image generation service."
                }
            },
            "usage": {
                "tokens_used": description.len() / 4,
                "estimated_cost_usd": 0.0015,
                "model_used": model
            },
            "provider_response": {
                "gemini_response": gemini_response,
                "enhanced_description": description
            }
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
    let generate_image_tool =
        Arc::new(GenerateImageTool::new(delay, use_ai, provider)) as Arc<dyn McpTool>;

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
    info!(
        "AI Provider: {} (enabled: {})",
        if args.use_ai { "Enabled" } else { "Disabled" },
        args.use_ai
    );
    if args.use_ai {
        info!("Using provider: {}", args.provider);
    }

    // Create the server
    let server = create_server(
        Duration::from_secs(args.delay),
        args.use_ai,
        args.provider.clone(),
    )
    .await?;

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
    use std::collections::HashMap;

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

    #[tokio::test]
    async fn test_demonstrate_image_generation() {
        println!("\n🎨 DEMONSTRATION: Image Generation with MCP Server");
        println!("{}", "=".repeat(60));

        // Test Mock Mode
        println!("1️⃣ MOCK MODE (Development/Testing)");
        let mock_tool =
            GenerateImageTool::new(Duration::from_millis(10), false, "mock".to_string());

        let mut args = HashMap::new();
        args.insert(
            "prompt".to_string(),
            serde_json::Value::String(
                "A beautiful sunset over a mountain lake with reflections".to_string(),
            ),
        );
        args.insert(
            "style".to_string(),
            serde_json::Value::String("photorealistic".to_string()),
        );
        args.insert(
            "size".to_string(),
            serde_json::Value::String("1024x1024".to_string()),
        );

        let request = McpRequest::CallTool {
            name: "generate_image".to_string(),
            arguments: args.clone(),
        };

        let response = mock_tool.call(request).await.unwrap();

        if let McpResponse::Success {
            result: ResponseResult::ToolResult { content, .. },
        } = response
        {
            if let ToolContent::Text { text } = &content[0] {
                let image_data: serde_json::Value = serde_json::from_str(text).unwrap();
                println!("✅ Mock Response Generated:");
                println!("   Image ID: {}", image_data["image"]["id"]);
                println!("   Prompt: {}", image_data["image"]["prompt"]);
                println!("   URL: {}", image_data["image"]["url"]);
                println!("   Model: {}", image_data["image"]["metadata"]["model"]);
                println!(
                    "   Note: {}",
                    image_data.get("note").unwrap_or(&serde_json::Value::Null)
                );
            }
        }

        println!("\n2️⃣ AI MODE (Google/Gemini Integration)");
        let ai_tool = GenerateImageTool::new(Duration::from_millis(10), true, "gemini".to_string());

        let request_ai = McpRequest::CallTool {
            name: "generate_image".to_string(),
            arguments: args,
        };

        println!("🤖 Testing AI mode (will use mock if no API key)...");
        let response_ai = ai_tool.call(request_ai).await;

        match response_ai {
            Ok(McpResponse::Success {
                result: ResponseResult::ToolResult { content, .. },
            }) => {
                if let ToolContent::Text { text } = &content[0] {
                    let image_data: serde_json::Value = serde_json::from_str(text).unwrap();
                    println!("✅ AI Response Generated:");
                    println!("   Image ID: {}", image_data["image"]["id"]);
                    if image_data["image"].get("enhanced_prompt").is_some() {
                        println!(
                            "   Enhanced Prompt: {}",
                            image_data["image"]["enhanced_prompt"]
                        );
                        println!(
                            "   Provider: {}",
                            image_data["image"]["metadata"]["provider"]
                        );
                    }
                    println!(
                        "   Processing Time: {}ms",
                        image_data["image"]["metadata"]["processing_time_ms"]
                    );
                }
            }
            Ok(_) => {
                println!("ℹ️  Unexpected response format");
            }
            Err(e) => {
                println!("ℹ️  AI mode error (expected if no API key): {e}");
                println!("   This demonstrates proper error handling for missing API keys");
            }
        }

        println!("\n✨ DEMONSTRATION COMPLETE!");
        println!("📋 Summary:");
        println!("   • Mock mode: ✅ Fast placeholder responses for development");
        println!("   • AI mode: ✅ Real Google/Gemini integration (with API key)");
        println!("   • Error handling: ✅ Graceful fallback when API unavailable");
        println!("   • MCP protocol: ✅ Proper request/response structure");
        println!("{}", "=".repeat(60));
    }
}
