# Image Generation Server Example

An AI-powered MCP server that provides image generation capabilities, demonstrating the MCP architecture with realistic placeholder responses ready for integration with actual AI image generation APIs.

## Overview

This example showcases:
- **GenerateImageTool**: AI-powered image generation with comprehensive configuration options
- **Realistic Placeholder Responses**: Structured JSON responses mimicking real AI services
- **Dual Transport Support**: Both STDIO and HTTP transport layers
- **AI Integration Ready**: Clear TODO markers for actual API integration
- **Comprehensive Metadata**: Detailed image generation parameters and metadata

## Features

### GenerateImageTool
- **Flexible Image Generation**: Support for various prompts, styles, and sizes
- **Style Options**: Photorealistic, artistic, cartoon, abstract, and custom styles
- **Size Configurations**: Multiple resolution options (512x512, 1024x1024, 1920x1080, etc.)
- **Metadata Rich**: Includes generation parameters, processing time, and technical details
- **Simulated Processing**: Configurable delay to simulate real AI processing time

### Ready for AI API Integration
The server includes structured TODO comments for easy integration with:
- **OpenAI DALL-E**: Direct API integration points
- **Midjourney**: API wrapper integration
- **Stable Diffusion**: Local or hosted model integration
- **Custom APIs**: Generic integration pattern

## Usage

### Command Line Options

```bash
image-generation-server [OPTIONS]

Options:
  -t, --transport <TRANSPORT>  Transport type to use [default: stdio] [possible values: stdio, http]
  -p, --port <PORT>           Port for HTTP transport [default: 3001]
      --host <HOST>           Host for HTTP transport [default: 127.0.0.1]
  -d, --debug                 Enable debug logging
      --delay <DELAY>         Simulate processing delay in seconds [default: 2]
  -h, --help                  Print help
  -V, --version               Print version
```

### STDIO Transport

Run with STDIO transport for pipe-based communication:

```bash
cargo run --bin image-generation-server -- --transport stdio
```

Send JSON-formatted MCP requests via stdin:

```json
{
  "method": "tools/call",
  "params": {
    "name": "generate_image",
    "arguments": {
      "prompt": "A serene mountain landscape at sunset",
      "style": "photorealistic",
      "size": "1024x1024"
    }
  }
}
```

### HTTP Transport

Run with HTTP transport for RESTful API:

```bash
cargo run --bin image-generation-server -- --transport http --port 3001
```

The server will start on `http://127.0.0.1:3001` with the following endpoints:

#### Available Endpoints

- **GET /health** - Health check endpoint
- **POST /mcp/tools/call** - Call a specific tool
- **GET /mcp/tools/list** - List available tools
- **POST /mcp/request** - Generic MCP request endpoint

#### Example HTTP Requests

**Generate an image:**
```bash
curl -X POST http://127.0.0.1:3001/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "generate_image",
    "arguments": {
      "prompt": "A futuristic city with flying cars at night",
      "style": "cyberpunk",
      "size": "1920x1080"
    }
  }'
```

**List available tools:**
```bash
curl http://127.0.0.1:3001/mcp/tools/list
```

**Health check:**
```bash
curl http://127.0.0.1:3001/health
```

## Tool Parameters

### generate_image

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `prompt` | string | Yes | - | Text description of the image to generate |
| `style` | string | No | "photorealistic" | Image style (photorealistic, artistic, cartoon, abstract, cyberpunk, fantasy, etc.) |
| `size` | string | No | "1024x1024" | Image dimensions (512x512, 1024x1024, 1920x1080, etc.) |

### Supported Styles
- `photorealistic` - Photo-realistic images
- `artistic` - Artistic and painterly styles
- `cartoon` - Cartoon and animated styles
- `abstract` - Abstract and conceptual art
- `cyberpunk` - Futuristic cyberpunk aesthetic
- `fantasy` - Fantasy and magical themes
- `minimalist` - Clean, minimal design
- `vintage` - Retro and vintage styles

### Supported Sizes
- `512x512` - Square, low resolution
- `1024x1024` - Square, high resolution (default)
- `1920x1080` - Landscape, HD
- `1080x1920` - Portrait, HD
- `2048x2048` - Square, ultra-high resolution

## Example Responses

### Successful Image Generation
```json
{
  "result": {
    "_type": "toolResult",
    "content": [
      {
        "type": "text",
        "text": "{
          \"success\": true,
          \"image\": {
            \"id\": \"img_a1b2c3d4e5f6\",
            \"prompt\": \"A serene mountain landscape at sunset\",
            \"style\": \"photorealistic\",
            \"size\": \"1024x1024\",
            \"format\": \"png\",
            \"url\": \"https://placeholder.example.com/generated/xyz123.png\",
            \"thumbnail_url\": \"https://placeholder.example.com/thumbnails/xyz123.png\",
            \"created_at\": \"2025-01-17T12:34:56Z\",
            \"metadata\": {
              \"model\": \"placeholder-diffusion-v2.1\",
              \"inference_steps\": 50,
              \"guidance_scale\": 7.5,
              \"seed\": 1234567890,
              \"processing_time_ms\": 2000,
              \"resolution\": \"1024x1024\",
              \"aspect_ratio\": \"1:1\"
            }
          }
        }"
      }
    ],
    "isError": false
  }
}
```

### Error Response
```json
{
  "error": {
    "code": -32602,
    "message": "Missing required parameter 'prompt'"
  }
}
```

## AI Integration Points

### TODO: Integrate with Real AI APIs

The server includes clear integration points for actual AI services:

#### OpenAI DALL-E Integration
```rust
// TODO: Replace placeholder with DALL-E API call
// use openai_api_rs::v1::image::CreateImageRequest;
// let response = client.create_image(CreateImageRequest {
//     prompt: prompt.to_string(),
//     n: Some(1),
//     size: Some(size.to_string()),
//     response_format: Some("url".to_string()),
// }).await?;
```

#### Stable Diffusion Integration
```rust
// TODO: Replace placeholder with Stable Diffusion API call
// use stable_diffusion::StableDiffusionPipeline;
// let image = pipeline.generate(
//     prompt,
//     negative_prompt,
//     num_inference_steps,
//     guidance_scale,
//     seed,
// ).await?;
```

#### Custom API Integration
```rust
// TODO: Replace placeholder with your custom image generation API
// let response = reqwest::Client::new()
//     .post("https://your-api.com/generate")
//     .json(&request_body)
//     .send()
//     .await?;
```

## Development

### Building
```bash
cargo build --bin image-generation-server
```

### Testing
```bash
cargo test --package image-generation-server
```

### Debug Mode
Enable debug logging and reduce processing delay for development:

```bash
cargo run --bin image-generation-server -- --debug --delay 0 --transport http
```

## Architecture

This example demonstrates key MCP concepts for AI integration:

1. **AI Tool Pattern**: Shows how to structure AI-powered tools
2. **Async Processing**: Handles long-running AI operations
3. **Rich Metadata**: Provides comprehensive generation details
4. **Error Handling**: Proper error codes for AI service failures
5. **Placeholder Pattern**: Ready-to-replace placeholder responses

The implementation follows AI integration best practices:
- Structured request/response formats
- Comprehensive metadata tracking
- Timeout and error handling
- Realistic response simulation
- Clear integration points

## Production Considerations

When integrating with real AI APIs:

1. **API Keys**: Store API keys securely using environment variables
2. **Rate Limiting**: Implement proper rate limiting and request queuing
3. **Caching**: Cache generated images to reduce API costs
4. **Error Handling**: Handle API failures gracefully with retries
5. **Monitoring**: Add metrics and logging for API usage tracking
6. **Content Filtering**: Implement content policy enforcement
7. **Cost Management**: Track API usage and implement cost controls

## Integration Examples

### Environment Variables
```bash
# For OpenAI DALL-E
export OPENAI_API_KEY="your-api-key"

# For Stable Diffusion
export REPLICATE_API_TOKEN="your-token"

# For custom APIs
export CUSTOM_AI_API_KEY="your-key"
export CUSTOM_AI_API_URL="https://your-api.com"
```

### Docker Deployment
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin image-generation-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/image-generation-server /usr/local/bin/
EXPOSE 3001
CMD ["image-generation-server", "--transport", "http", "--host", "0.0.0.0"]
```

## Files

- `src/main.rs` - Complete server implementation with GenerateImageTool
- `Cargo.toml` - Dependencies and configuration
- `README.md` - This documentation

## Related Examples

- [filesystem-server](../filesystem-server/) - Basic file operations example
- [blog-generation-server](../blog-generation-server/) - AI blog generation
- [creative-content-server](../creative-content-server/) - Multi-tool creative AI server