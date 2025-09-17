# Creative Content Server Example

A comprehensive AI-powered MCP server that provides multiple creative content generation capabilities, demonstrating the MCP architecture with realistic placeholder responses ready for integration with actual AI content generation APIs.

## Overview

This example showcases:
- **Multiple Creative Tools**: Story generation, poetry creation, character development, and creative writing assistance
- **Realistic Placeholder Responses**: Structured JSON responses mimicking real AI creative services
- **Dual Transport Support**: Both STDIO and HTTP transport layers
- **AI Integration Ready**: Clear TODO markers for actual API integration
- **Multi-Tool Architecture**: Demonstrates how to build servers with multiple specialized tools

## Features

### Available Tools

#### 1. GenerateStoryTool
- **Creative Story Generation**: Full short stories with plot, characters, and narrative structure
- **Genre Support**: Fantasy, sci-fi, mystery, romance, horror, adventure, and more
- **Length Options**: Flash fiction, short story, novelette configurations
- **Character Development**: Automatic character creation and development
- **Plot Structure**: Beginning, middle, end with conflict resolution

#### 2. CreatePoemTool
- **Poetry Generation**: Various poetic forms and styles
- **Form Support**: Haiku, sonnet, free verse, limerick, ballad, and custom forms
- **Theme Integration**: Love, nature, existential, social, personal themes
- **Meter and Rhyme**: Structured rhyme schemes and meter patterns
- **Emotional Tone**: Happy, melancholic, contemplative, energetic moods

#### 3. DevelopCharacterTool
- **Character Creation**: Comprehensive character profiles and backstories
- **Personality Traits**: MBTI-based personality development
- **Background Generation**: History, motivations, goals, and conflicts
- **Relationship Mapping**: Character connections and dynamics
- **Visual Descriptions**: Physical appearance and mannerisms

#### 4. CreativeWritingAssistTool
- **Writing Enhancement**: Improve existing text with creative suggestions
- **Style Analysis**: Identify and enhance writing style elements
- **Plot Suggestions**: Story development and plot advancement ideas
- **Dialogue Enhancement**: Improve conversation and character voice
- **Creative Prompts**: Generate writing prompts and inspiration

### Ready for AI API Integration
The server includes structured TODO comments for easy integration with:
- **OpenAI GPT**: Direct API integration points for all tools
- **Claude**: Anthropic API integration for creative writing
- **Google Gemini**: Google AI integration
- **Custom APIs**: Generic integration patterns for each tool

## Usage

### Command Line Options

```bash
creative-content-server [OPTIONS]

Options:
  -t, --transport <TRANSPORT>  Transport type to use [default: stdio] [possible values: stdio, http]
  -p, --port <PORT>           Port for HTTP transport [default: 3003]
      --host <HOST>           Host for HTTP transport [default: 127.0.0.1]
  -d, --debug                 Enable debug logging
      --delay <DELAY>         Simulate processing delay in seconds [default: 1]
  -h, --help                  Print help
  -V, --version               Print version
```

### STDIO Transport

Run with STDIO transport for pipe-based communication:

```bash
cargo run --bin creative-content-server -- --transport stdio
```

Send JSON-formatted MCP requests via stdin:

```json
{
  "method": "tools/call",
  "params": {
    "name": "generate_story",
    "arguments": {
      "theme": "A lonely robot discovers the meaning of friendship",
      "genre": "sci-fi",
      "length": "short",
      "tone": "heartwarming"
    }
  }
}
```

### HTTP Transport

Run with HTTP transport for RESTful API:

```bash
cargo run --bin creative-content-server -- --transport http --port 3003
```

The server will start on `http://127.0.0.1:3003` with the following endpoints:

#### Available Endpoints

- **GET /health** - Health check endpoint
- **POST /mcp/tools/call** - Call a specific tool
- **GET /mcp/tools/list** - List available tools
- **POST /mcp/request** - Generic MCP request endpoint

#### Example HTTP Requests

**Generate a story:**
```bash
curl -X POST http://127.0.0.1:3003/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "generate_story",
    "arguments": {
      "theme": "A magical library that exists between dimensions",
      "genre": "fantasy",
      "length": "short",
      "tone": "mysterious"
    }
  }'
```

**Create a poem:**
```bash
curl -X POST http://127.0.0.1:3003/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "create_poem",
    "arguments": {
      "theme": "ocean waves at sunset",
      "form": "haiku",
      "mood": "peaceful"
    }
  }'
```

**Develop a character:**
```bash
curl -X POST http://127.0.0.1:3003/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "develop_character",
    "arguments": {
      "character_type": "protagonist",
      "genre": "mystery",
      "age_range": "middle-aged",
      "background": "retired detective"
    }
  }'
```

**Get creative writing assistance:**
```bash
curl -X POST http://127.0.0.1:3003/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "creative_writing_assist",
    "arguments": {
      "text": "The old house creaked in the wind...",
      "assistance_type": "enhance_description",
      "style": "gothic"
    }
  }'
```

## Tool Parameters

### generate_story

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `theme` | string | Yes | - | Main theme or premise for the story |
| `genre` | string | No | "general" | Story genre (fantasy, sci-fi, mystery, romance, horror, etc.) |
| `length` | string | No | "short" | Story length (flash, short, novelette) |
| `tone` | string | No | "neutral" | Emotional tone (happy, sad, mysterious, suspenseful, etc.) |
| `characters` | array | No | [] | Specific character names or types to include |

### create_poem

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `theme` | string | Yes | - | Subject or theme for the poem |
| `form` | string | No | "free_verse" | Poetic form (haiku, sonnet, limerick, free_verse, etc.) |
| `mood` | string | No | "contemplative" | Emotional mood of the poem |
| `rhyme_scheme` | string | No | - | Specific rhyme scheme (ABAB, AABA, etc.) |

### develop_character

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `character_type` | string | Yes | - | Character role (protagonist, antagonist, supporting, etc.) |
| `genre` | string | No | "general" | Story genre context |
| `age_range` | string | No | "adult" | Character age range |
| `background` | string | No | - | Brief background or profession |
| `personality_traits` | array | No | [] | Specific traits to emphasize |

### creative_writing_assist

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `text` | string | Yes | - | Existing text to enhance or analyze |
| `assistance_type` | string | No | "general" | Type of assistance (enhance_description, improve_dialogue, plot_suggestions, etc.) |
| `style` | string | No | "neutral" | Writing style preference |
| `target_audience` | string | No | "general" | Intended audience |

## Supported Options

### Story Genres
- `fantasy` - Magical and fantastical elements
- `sci-fi` - Science fiction and futuristic themes
- `mystery` - Detective and puzzle-solving narratives
- `romance` - Love and relationship stories
- `horror` - Scary and suspenseful content
- `adventure` - Action and exploration themes
- `drama` - Character-driven emotional stories
- `comedy` - Humorous and light-hearted content

### Poem Forms
- `haiku` - Traditional Japanese 5-7-5 syllable structure
- `sonnet` - 14-line structured poem with specific rhyme scheme
- `limerick` - Humorous 5-line poem with AABBA rhyme
- `free_verse` - No specific structure or rhyme requirements
- `ballad` - Narrative poem with musical qualities
- `acrostic` - First letters spell out a word or phrase

### Character Types
- `protagonist` - Main hero or central character
- `antagonist` - Opposition or villain character
- `supporting` - Secondary characters who aid the story
- `comic_relief` - Characters who provide humor
- `mentor` - Wise guide or teacher characters
- `love_interest` - Romantic partner characters

## Example Responses

### Successful Story Generation
```json
{
  "result": {
    "_type": "toolResult",
    "content": [
      {
        "type": "text",
        "text": "{
          \"success\": true,
          \"story\": {
            \"id\": \"story_a1b2c3d4e5f6\",
            \"title\": \"The Dimensional Library\",
            \"content\": \"Maya stumbled through the shimmering portal, her heart racing as she found herself in a vast library that seemed to stretch infinitely in all directions. The shelves towered impossibly high, filled with books that glowed with an inner light...\\n\\nAs she explored deeper into the mystical repository, she discovered that each book contained not just stories, but entire worlds waiting to be explored. The librarian, an ancient being with starlight in their eyes, explained that she had been chosen as the new Guardian of Stories...\\n\\nWith newfound purpose, Maya embraced her role, knowing that she would spend eternity protecting the dreams and imagination of countless worlds.\",
            \"genre\": \"fantasy\",
            \"theme\": \"A magical library that exists between dimensions\",
            \"word_count\": 847,
            \"characters\": [
              {
                \"name\": \"Maya\",
                \"role\": \"protagonist\",
                \"description\": \"Young woman who discovers her destiny as Guardian of Stories\"
              },
              {
                \"name\": \"The Librarian\",
                \"role\": \"mentor\",
                \"description\": \"Ancient being who guides Maya to her purpose\"
              }
            ],
            \"plot_structure\": {
              \"exposition\": \"Maya discovers the dimensional library\",
              \"rising_action\": \"Exploration and discovery of the library's true nature\",
              \"climax\": \"Meeting the Librarian and learning about her destiny\",
              \"resolution\": \"Accepting the role of Guardian of Stories\"
            },
            \"tone\": \"mysterious\",
            \"estimated_reading_time\": \"3 minutes\",
            \"created_at\": \"2025-01-17T12:34:56Z\",
            \"metadata\": {
              \"model\": \"placeholder-creative-ai-v2.1\",
              \"processing_time_ms\": 1500,
              \"creativity_score\": 8.7,
              \"originality_score\": 9.2,
              \"narrative_coherence\": 8.9
            }
          }
        }"
      }
    ],
    "isError": false
  }
}
```

### Successful Poem Creation
```json
{
  "result": {
    "_type": "toolResult",
    "content": [
      {
        "type": "text",
        "text": "{
          \"success\": true,
          \"poem\": {
            \"id\": \"poem_x1y2z3a4b5c6\",
            \"title\": \"Ocean's Evening\",
            \"content\": \"Waves embrace the shore\\nSunset paints the endless sea\\nPeace in golden light\",
            \"form\": \"haiku\",
            \"theme\": \"ocean waves at sunset\",
            \"mood\": \"peaceful\",
            \"structure\": {
              \"lines\": 3,
              \"syllable_pattern\": \"5-7-5\",
              \"rhyme_scheme\": \"none\"
            },
            \"literary_devices\": [
              \"personification\",
              \"imagery\",
              \"metaphor\"
            ],
            \"created_at\": \"2025-01-17T12:34:56Z\",
            \"metadata\": {
              \"model\": \"placeholder-poetry-ai-v1.5\",
              \"processing_time_ms\": 800,
              \"artistic_quality\": 8.5,
              \"adherence_to_form\": 10.0,
              \"emotional_resonance\": 8.8
            }
          }
        }"
      }
    ],
    "isError": false
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
//         role: "system".to_string(),
//         content: format!("You are a creative writing assistant specializing in {}", genre),
//     }, ChatCompletionMessage {
//         role: "user".to_string(),
//         content: format!("Write a {} {} story about: {}", tone, genre, theme),
//     }],
//     max_tokens: Some(1000),
//     temperature: Some(0.8),
// }).await?;
```

#### Claude Integration for Creative Writing
```rust
// TODO: Replace placeholder with Anthropic Claude API call
// use anthropic::{Client, messages::CreateMessageRequest};
// let response = client.messages().create(CreateMessageRequest {
//     model: "claude-3-opus-20240229".to_string(),
//     max_tokens: 1000,
//     messages: vec![Message {
//         role: "user".to_string(),
//         content: format!("Create a {} in {} form about: {}", "poem", form, theme),
//     }],
// }).await?;
```

## Development

### Building
```bash
cargo build --bin creative-content-server
```

### Testing
```bash
cargo test --package creative-content-server
```

### Debug Mode
Enable debug logging and reduce processing delay for development:

```bash
cargo run --bin creative-content-server -- --debug --delay 0 --transport http
```

## Architecture

This example demonstrates key MCP concepts for multi-tool creative AI:

1. **Multi-Tool Pattern**: Shows how to implement multiple related tools in one server
2. **Creative AI Architecture**: Specialized tools for different creative tasks
3. **Rich Creative Metadata**: Comprehensive analysis and scoring of creative content
4. **Tool Specialization**: Each tool optimized for specific creative tasks
5. **Consistent API Design**: Uniform request/response patterns across all tools

## Production Considerations

When integrating with real AI APIs:

1. **API Key Management**: Secure storage and rotation of API keys
2. **Rate Limiting**: Implement per-tool and per-user rate limiting
3. **Content Filtering**: Ensure generated content meets content policies
4. **Quality Control**: Implement content quality scoring and validation
5. **Caching Strategy**: Cache similar requests to reduce API costs
6. **User Preferences**: Store and apply user style preferences
7. **Content Moderation**: Implement automated content review systems
8. **Performance Optimization**: Optimize for creative generation latency

## Integration Examples

### Environment Variables
```bash
# For OpenAI GPT
export OPENAI_API_KEY="your-api-key"

# For Anthropic Claude
export ANTHROPIC_API_KEY="your-api-key"

# For specialized creative APIs
export CREATIVE_AI_API_KEY="your-key"
export POETRY_API_KEY="your-key"
```

### Docker Deployment
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin creative-content-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/creative-content-server /usr/local/bin/
EXPOSE 3003
CMD ["creative-content-server", "--transport", "http", "--host", "0.0.0.0"]
```

## Creative Features

### Story Generation
- Plot structure analysis and generation
- Character development integration
- Genre-specific tropes and elements
- Narrative arc optimization
- Dialogue and description balance

### Poetry Creation
- Meter and rhythm analysis
- Rhyme scheme adherence
- Literary device integration
- Emotional tone consistency
- Form structure validation

### Character Development
- Personality consistency checking
- Backstory coherence
- Character arc planning
- Relationship mapping
- Motivation alignment

### Writing Enhancement
- Style consistency analysis
- Voice and tone optimization
- Pacing improvement suggestions
- Plot hole identification
- Character development guidance

## Files

- `src/main.rs` - Complete server implementation with all creative tools
- `Cargo.toml` - Dependencies and configuration
- `README.md` - This documentation

## Related Examples

- [filesystem-server](../filesystem-server/) - Basic file operations example
- [image-generation-server](../image-generation-server/) - AI image generation
- [blog-generation-server](../blog-generation-server/) - AI blog generation

## Use Cases

### Creative Writing
- Novel and story development
- Poetry collection creation
- Character development for fiction
- Writing workshop assistance
- Creative writing education

### Content Creation
- Marketing copy generation
- Social media content
- Creative campaign development
- Brand storytelling
- Interactive fiction development

### Educational Applications
- Creative writing instruction
- Literature analysis tools
- Writing prompt generation
- Character analysis exercises
- Poetry appreciation and creation

### Entertainment
- Game narrative development
- Interactive storytelling
- Creative writing challenges
- Community writing projects
- Personal creative expression tools