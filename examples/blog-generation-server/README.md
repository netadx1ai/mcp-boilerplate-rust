# Blog Generation Server Example

An AI-powered MCP server that provides blog generation capabilities, demonstrating the MCP architecture with realistic placeholder responses ready for integration with actual AI content generation APIs.

## Overview

This example showcases:
- **CreateBlogPostTool**: AI-powered blog post generation with comprehensive configuration options
- **Realistic Placeholder Responses**: Structured JSON responses mimicking real AI content services
- **Dual Transport Support**: Both STDIO and HTTP transport layers
- **AI Integration Ready**: Clear TODO markers for actual API integration
- **Comprehensive Metadata**: Detailed generation parameters and content statistics

## Features

### CreateBlogPostTool
- **Flexible Content Generation**: Support for various topics, styles, and lengths
- **Writing Styles**: Professional, casual, technical, creative, and academic styles
- **Length Options**: Short, medium, long, and custom word count configurations
- **SEO Features**: Automatic title generation, meta descriptions, and keyword optimization
- **Metadata Rich**: Includes generation parameters, word count, reading time, and technical details
- **Simulated Processing**: Configurable delay to simulate real AI processing time

### Ready for AI API Integration
The server includes structured TODO comments for easy integration with:
- **OpenAI GPT**: Direct API integration points
- **Claude**: Anthropic API integration
- **Google Gemini**: Google AI integration
- **Custom APIs**: Generic integration pattern

## Usage

### Command Line Options

```bash
blog-generation-server [OPTIONS]

Options:
  -t, --transport <TRANSPORT>  Transport type to use [default: stdio] [possible values: stdio, http]
  -p, --port <PORT>           Port for HTTP transport [default: 3002]
      --host <HOST>           Host for HTTP transport [default: 127.0.0.1]
  -d, --debug                 Enable debug logging
      --delay <DELAY>         Simulate processing delay in seconds [default: 2]
  -h, --help                  Print help
  -V, --version               Print version
```

### STDIO Transport

Run with STDIO transport for pipe-based communication:

```bash
cargo run --bin blog-generation-server -- --transport stdio
```

Send JSON-formatted MCP requests via stdin:

```json
{
  "method": "tools/call",
  "params": {
    "name": "create_blog_post",
    "arguments": {
      "topic": "The Future of Artificial Intelligence",
      "style": "professional",
      "length": "medium",
      "keywords": ["AI", "machine learning", "technology", "future"]
    }
  }
}
```

### HTTP Transport

Run with HTTP transport for RESTful API:

```bash
cargo run --bin blog-generation-server -- --transport http --port 3002
```

The server will start on `http://127.0.0.1:3002` with the following endpoints:

#### Available Endpoints

- **GET /health** - Health check endpoint
- **POST /mcp/tools/call** - Call a specific tool
- **GET /mcp/tools/list** - List available tools
- **POST /mcp/request** - Generic MCP request endpoint

#### Example HTTP Requests

**Generate a blog post:**
```bash
curl -X POST http://127.0.0.1:3002/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "create_blog_post",
    "arguments": {
      "topic": "Sustainable Energy Solutions",
      "style": "technical",
      "length": "long",
      "keywords": ["renewable energy", "solar power", "sustainability"]
    }
  }'
```

**List available tools:**
```bash
curl http://127.0.0.1:3002/mcp/tools/list
```

**Health check:**
```bash
curl http://127.0.0.1:3002/health
```

## Tool Parameters

### create_blog_post

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `topic` | string | Yes | - | Main topic or subject for the blog post |
| `style` | string | No | "professional" | Writing style (professional, casual, technical, creative, academic) |
| `length` | string | No | "medium" | Content length (short, medium, long, custom) |
| `keywords` | array | No | [] | SEO keywords to incorporate into the content |
| `word_count` | number | No | - | Specific word count (overrides length parameter) |
| `include_outline` | boolean | No | true | Whether to include a content outline |

### Supported Writing Styles
- `professional` - Business and corporate tone
- `casual` - Conversational and friendly tone
- `technical` - Detailed and precise technical writing
- `creative` - Artistic and engaging storytelling
- `academic` - Formal and research-oriented
- `journalistic` - News and reporting style

### Content Length Options
- `short` - 300-500 words
- `medium` - 800-1200 words (default)
- `long` - 1500-2500 words
- `custom` - Use word_count parameter for specific length

## Example Responses

### Successful Blog Post Generation
```json
{
  "result": {
    "_type": "toolResult",
    "content": [
      {
        "type": "text",
        "text": "{
          \"success\": true,
          \"blog_post\": {
            \"id\": \"post_a1b2c3d4e5f6\",
            \"title\": \"The Future of Artificial Intelligence: Transforming Industries and Society\",
            \"content\": \"# The Future of Artificial Intelligence\\n\\nArtificial Intelligence (AI) has emerged as one of the most transformative technologies of our time...\\n\\n## Current State of AI\\n\\nToday's AI landscape is characterized by...\",
            \"meta_description\": \"Explore how artificial intelligence is reshaping industries and society, from machine learning breakthroughs to ethical considerations in AI development.\",
            \"outline\": [
              \"Introduction to AI's Impact\",
              \"Current State of AI Technology\",
              \"Industry Applications\",
              \"Ethical Considerations\",
              \"Future Predictions\"
            ],
            \"keywords\": [\"AI\", \"machine learning\", \"technology\", \"future\"],
            \"word_count\": 1150,
            \"estimated_reading_time\": \"5 minutes\",
            \"style\": \"professional\",
            \"topic\": \"The Future of Artificial Intelligence\",
            \"created_at\": \"2025-01-17T12:34:56Z\",
            \"metadata\": {
              \"model\": \"placeholder-gpt-v3.5\",
              \"processing_time_ms\": 2000,
              \"language\": \"en\",
              \"readability_score\": 8.2,
              \"seo_score\": 9.1,
              \"sentiment\": \"positive\"
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
    "message": "Missing required parameter 'topic'"
  }
}
```

## AI Integration Points

### TODO: Integrate with Real AI APIs

The server includes clear integration points for actual AI services:

#### OpenAI GPT Integration
```rust
// TODO: Replace placeholder with OpenAI API call
// use openai_api_rs::v1::chat_completion::{ChatCompletionRequest, ChatCompletionMessage};
// let response = client.chat_completion(ChatCompletionRequest {
//     model: "gpt-4".to_string(),
//     messages: vec![ChatCompletionMessage {
//         role: "user".to_string(),
//         content: format!("Write a {} blog post about: {}", style, topic),
//     }],
//     max_tokens: Some(word_count),
//     temperature: Some(0.7),
// }).await?;
```

#### Claude Integration
```rust
// TODO: Replace placeholder with Anthropic Claude API call
// use anthropic::{Client, messages::CreateMessageRequest};
// let response = client.messages().create(CreateMessageRequest {
//     model: "claude-3-opus-20240229".to_string(),
//     max_tokens: word_count,
//     messages: vec![Message {
//         role: "user".to_string(),
//         content: format!("Create a {} style blog post about: {}", style, topic),
//     }],
// }).await?;
```

#### Custom API Integration
```rust
// TODO: Replace placeholder with your custom content generation API
// let response = reqwest::Client::new()
//     .post("https://your-api.com/generate-content")
//     .json(&request_body)
//     .send()
//     .await?;
```

## Development

### Building
```bash
cargo build --bin blog-generation-server
```

### Testing
```bash
cargo test --package blog-generation-server
```

### Debug Mode
Enable debug logging and reduce processing delay for development:

```bash
cargo run --bin blog-generation-server -- --debug --delay 0 --transport http
```

## Architecture

This example demonstrates key MCP concepts for AI content generation:

1. **AI Content Tool Pattern**: Shows how to structure content generation tools
2. **Async Processing**: Handles long-running AI operations
3. **Rich Content Metadata**: Provides comprehensive generation details
4. **SEO Integration**: Built-in SEO optimization features
5. **Placeholder Pattern**: Ready-to-replace placeholder responses

The implementation follows content generation best practices:
- Structured request/response formats
- Comprehensive metadata tracking
- SEO-friendly output
- Multiple style and length options
- Clear integration points

## Production Considerations

When integrating with real AI APIs:

1. **API Keys**: Store API keys securely using environment variables
2. **Rate Limiting**: Implement proper rate limiting and request queuing
3. **Content Caching**: Cache generated content to reduce API costs
4. **Error Handling**: Handle API failures gracefully with retries
5. **Content Filtering**: Implement content policy enforcement
6. **Quality Control**: Add content quality scoring and validation
7. **Cost Management**: Track API usage and implement cost controls
8. **Plagiarism Detection**: Implement originality checking

## Integration Examples

### Environment Variables
```bash
# For OpenAI GPT
export OPENAI_API_KEY="your-api-key"

# For Anthropic Claude
export ANTHROPIC_API_KEY="your-api-key"

# For Google Gemini
export GOOGLE_API_KEY="your-api-key"

# For custom APIs
export CUSTOM_AI_API_KEY="your-key"
export CUSTOM_AI_API_URL="https://your-api.com"
```

### Docker Deployment
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin blog-generation-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/blog-generation-server /usr/local/bin/
EXPOSE 3002
CMD ["blog-generation-server", "--transport", "http", "--host", "0.0.0.0"]
```

## Content Quality Features

### SEO Optimization
- Automatic title generation with keyword optimization
- Meta description creation
- Keyword density analysis
- Readability scoring
- Internal linking suggestions

### Content Analysis
- Word count and reading time estimation
- Sentiment analysis
- Readability scoring (Flesch-Kincaid)
- SEO score calculation
- Topic relevance assessment

## Files

- `src/main.rs` - Complete server implementation with CreateBlogPostTool
- `Cargo.toml` - Dependencies and configuration
- `README.md` - This documentation

## Related Examples

- [filesystem-server](../filesystem-server/) - Basic file operations example
- [image-generation-server](../image-generation-server/) - AI image generation
- [creative-content-server](../creative-content-server/) - Multi-tool creative AI server

## Use Cases

### Content Marketing
- Automated blog post generation for marketing campaigns
- SEO-optimized content creation
- Topic research and content planning
- Multi-language content generation

### Technical Documentation
- API documentation generation
- Tutorial and guide creation
- Code documentation writing
- Technical specification documents

### Educational Content
- Course material creation
- Study guide generation
- Quiz and assessment content
- Research paper assistance

### Creative Writing
- Story and narrative generation
- Creative essay writing
- Poetry and creative content
- Script and dialogue writing