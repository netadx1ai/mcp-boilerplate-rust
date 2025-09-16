//! Creative content server example for MCP
//! 
//! This example demonstrates an AI-powered MCP server that provides multiple creative content generation capabilities.
//! The server includes tools for story generation, poetry creation, character development, and creative writing assistance.

use anyhow::Result;
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
#[command(name = "creative-content-server")]
#[command(about = "MCP creative content generation server with multiple tools")]
#[command(version = "0.1.0")]
struct Args {
    /// Transport type to use
    #[arg(short, long, value_enum, default_value_t = TransportType::Stdio)]
    transport: TransportType,

    /// Port for HTTP transport
    #[arg(short, long, default_value_t = 3003)]
    port: u16,

    /// Host for HTTP transport  
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Artificial delay in seconds for content generation (simulates AI processing time)
    #[arg(long, default_value_t = 1)]
    delay: u64,
}

#[derive(Clone, ValueEnum, Debug)]
enum TransportType {
    /// Use STDIO transport
    Stdio,
    /// Use HTTP transport
    Http,
}

/// Story generation tool that creates creative stories based on user requirements
pub struct GenerateStoryTool {
    processing_delay: Duration,
}

impl GenerateStoryTool {
    pub fn new(processing_delay: Duration) -> Self {
        Self { processing_delay }
    }

    async fn generate_placeholder_story(&self, parameters: &HashMap<String, Value>) -> Result<Value, McpError> {
        info!("Generating story with parameters: {:?}", parameters);
        
        sleep(self.processing_delay).await;
        
        let genre = parameters.get("genre")
            .and_then(|v| v.as_str())
            .unwrap_or("adventure");
            
        let length = parameters.get("length")
            .and_then(|v| v.as_str())
            .unwrap_or("short");
            
        let theme = parameters.get("theme")
            .and_then(|v| v.as_str())
            .unwrap_or("friendship");

        let (title, content) = match genre {
            "fantasy" => generate_fantasy_story(length, theme),
            "sci-fi" => generate_scifi_story(length, theme),
            "mystery" => generate_mystery_story(length, theme),
            "romance" => generate_romance_story(length, theme),
            "adventure" => generate_adventure_story(length, theme),
            _ => generate_generic_story(genre, length, theme),
        };

        Ok(json!({
            "story": {
                "title": title,
                "content": content,
                "metadata": {
                    "genre": genre,
                    "length": length,
                    "theme": theme,
                    "word_count": estimate_word_count(length),
                    "generated_at": chrono::Utc::now().to_rfc3339(),
                    "reading_time_minutes": estimate_reading_time(length)
                }
            },
            "generation_stats": {
                "processing_time_seconds": self.processing_delay.as_secs(),
                "creativity_score": 0.94,
                "originality_rating": 0.87
            }
        }))
    }

    fn validate_parameters(&self, parameters: &HashMap<String, Value>) -> Result<(), McpError> {
        if let Some(genre) = parameters.get("genre") {
            if let Some(genre_str) = genre.as_str() {
                let valid_genres = ["fantasy", "sci-fi", "mystery", "romance", "adventure", "horror", "comedy", "drama"];
                if !valid_genres.contains(&genre_str.to_lowercase().as_str()) {
                    return Err(McpError::invalid_params(
                        format!("genre must be one of: {}", valid_genres.join(", "))
                    ));
                }
            }
        }

        if let Some(length) = parameters.get("length") {
            if let Some(length_str) = length.as_str() {
                let valid_lengths = ["flash", "short", "medium", "long"];
                if !valid_lengths.contains(&length_str.to_lowercase().as_str()) {
                    return Err(McpError::invalid_params(
                        format!("length must be one of: {}", valid_lengths.join(", "))
                    ));
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl McpTool for GenerateStoryTool {
    async fn call(&self, request: McpRequest) -> Result<McpResponse, McpError> {
        match request {
            McpRequest::CallTool { name, arguments } => {
                if name != self.name() {
                    return Err(McpError::method_not_found(&name));
                }
                
                debug!("GenerateStoryTool called with arguments: {:?}", arguments);
                self.validate_parameters(&arguments)?;
                
                match self.generate_placeholder_story(&arguments).await {
                    Ok(result) => {
                        info!("Successfully generated story");
                        let content = ToolContent::Text { 
                            text: serde_json::to_string_pretty(&result)
                                .unwrap_or_else(|_| "Error serializing story".to_string())
                        };
                        let response_result = ResponseResult::ToolResult {
                            content: vec![content],
                            is_error: false,
                        };
                        Ok(McpResponse::success(response_result))
                    }
                    Err(e) => {
                        error!("Failed to generate story: {}", e);
                        Err(e)
                    }
                }
            }
            _ => Err(McpError::invalid_request("Expected CallTool request")),
        }
    }

    fn name(&self) -> &str {
        "generate_story"
    }

    fn description(&self) -> &str {
        "Generate creative stories in various genres and lengths. Perfect for creative writing, entertainment, and storytelling applications."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "genre": {
                    "type": "string",
                    "description": "Genre of the story",
                    "enum": ["fantasy", "sci-fi", "mystery", "romance", "adventure", "horror", "comedy", "drama"],
                    "default": "adventure"
                },
                "length": {
                    "type": "string",
                    "description": "Length of the story",
                    "enum": ["flash", "short", "medium", "long"],
                    "default": "short"
                },
                "theme": {
                    "type": "string",
                    "description": "Main theme or message of the story",
                    "default": "friendship"
                },
                "characters": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Optional list of character names or types"
                }
            }
        })
    }
}

/// Poetry creation tool that generates poems in various styles and forms
pub struct CreatePoemTool {
    processing_delay: Duration,
}

impl CreatePoemTool {
    pub fn new(processing_delay: Duration) -> Self {
        Self { processing_delay }
    }

    async fn generate_placeholder_poem(&self, parameters: &HashMap<String, Value>) -> Result<Value, McpError> {
        info!("Generating poem with parameters: {:?}", parameters);
        
        sleep(self.processing_delay).await;
        
        let style = parameters.get("style")
            .and_then(|v| v.as_str())
            .unwrap_or("free_verse");
            
        let theme = parameters.get("theme")
            .and_then(|v| v.as_str())
            .unwrap_or("nature");

        let (title, content, structure) = match style {
            "sonnet" => generate_sonnet(theme),
            "haiku" => generate_haiku(theme),
            "limerick" => generate_limerick(theme),
            "ballad" => generate_ballad(theme),
            "free_verse" => generate_free_verse(theme),
            _ => generate_free_verse(theme),
        };

        Ok(json!({
            "poem": {
                "title": title,
                "content": content,
                "metadata": {
                    "style": style,
                    "theme": theme,
                    "structure": structure,
                    "generated_at": chrono::Utc::now().to_rfc3339(),
                    "line_count": content.lines().count()
                }
            },
            "generation_stats": {
                "processing_time_seconds": self.processing_delay.as_secs(),
                "lyrical_quality": 0.91,
                "emotional_resonance": 0.88
            }
        }))
    }

    fn validate_parameters(&self, parameters: &HashMap<String, Value>) -> Result<(), McpError> {
        if let Some(style) = parameters.get("style") {
            if let Some(style_str) = style.as_str() {
                let valid_styles = ["sonnet", "haiku", "limerick", "ballad", "free_verse", "acrostic", "cinquain"];
                if !valid_styles.contains(&style_str.to_lowercase().as_str()) {
                    return Err(McpError::invalid_params(
                        format!("style must be one of: {}", valid_styles.join(", "))
                    ));
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl McpTool for CreatePoemTool {
    async fn call(&self, request: McpRequest) -> Result<McpResponse, McpError> {
        match request {
            McpRequest::CallTool { name, arguments } => {
                if name != self.name() {
                    return Err(McpError::method_not_found(&name));
                }
                
                debug!("CreatePoemTool called with arguments: {:?}", arguments);
                self.validate_parameters(&arguments)?;
                
                match self.generate_placeholder_poem(&arguments).await {
                    Ok(result) => {
                        info!("Successfully generated poem");
                        let content = ToolContent::Text { 
                            text: serde_json::to_string_pretty(&result)
                                .unwrap_or_else(|_| "Error serializing poem".to_string())
                        };
                        let response_result = ResponseResult::ToolResult {
                            content: vec![content],
                            is_error: false,
                        };
                        Ok(McpResponse::success(response_result))
                    }
                    Err(e) => {
                        error!("Failed to generate poem: {}", e);
                        Err(e)
                    }
                }
            }
            _ => Err(McpError::invalid_request("Expected CallTool request")),
        }
    }

    fn name(&self) -> &str {
        "create_poem"
    }

    fn description(&self) -> &str {
        "Create beautiful poetry in various styles and forms. Express emotions, themes, and ideas through the art of verse."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "style": {
                    "type": "string",
                    "description": "Style or form of the poem",
                    "enum": ["sonnet", "haiku", "limerick", "ballad", "free_verse", "acrostic", "cinquain"],
                    "default": "free_verse"
                },
                "theme": {
                    "type": "string",
                    "description": "Theme or subject of the poem",
                    "default": "nature"
                },
                "mood": {
                    "type": "string",
                    "description": "Desired mood or tone",
                    "enum": ["joyful", "melancholic", "romantic", "contemplative", "energetic", "peaceful"]
                }
            }
        })
    }
}

/// Character development tool for creating detailed fictional characters
pub struct DevelopCharacterTool {
    processing_delay: Duration,
}

impl DevelopCharacterTool {
    pub fn new(processing_delay: Duration) -> Self {
        Self { processing_delay }
    }

    async fn generate_character_profile(&self, parameters: &HashMap<String, Value>) -> Result<Value, McpError> {
        info!("Developing character with parameters: {:?}", parameters);
        
        sleep(self.processing_delay).await;
        
        let name = parameters.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Alex");
            
        let archetype = parameters.get("archetype")
            .and_then(|v| v.as_str())
            .unwrap_or("hero");

        let character = generate_character_details(name, archetype);

        Ok(json!({
            "character": character,
            "generation_stats": {
                "processing_time_seconds": self.processing_delay.as_secs(),
                "personality_depth": 0.93,
                "believability_score": 0.89
            }
        }))
    }
}

#[async_trait]
impl McpTool for DevelopCharacterTool {
    async fn call(&self, request: McpRequest) -> Result<McpResponse, McpError> {
        match request {
            McpRequest::CallTool { name, arguments } => {
                if name != self.name() {
                    return Err(McpError::method_not_found(&name));
                }
                
                debug!("DevelopCharacterTool called with arguments: {:?}", arguments);
                
                match self.generate_character_profile(&arguments).await {
                    Ok(result) => {
                        info!("Successfully developed character");
                        let content = ToolContent::Text { 
                            text: serde_json::to_string_pretty(&result)
                                .unwrap_or_else(|_| "Error serializing character".to_string())
                        };
                        let response_result = ResponseResult::ToolResult {
                            content: vec![content],
                            is_error: false,
                        };
                        Ok(McpResponse::success(response_result))
                    }
                    Err(e) => {
                        error!("Failed to develop character: {}", e);
                        Err(e)
                    }
                }
            }
            _ => Err(McpError::invalid_request("Expected CallTool request")),
        }
    }

    fn name(&self) -> &str {
        "develop_character"
    }

    fn description(&self) -> &str {
        "Develop detailed fictional characters with personalities, backstories, and motivations for creative writing."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Character's name",
                    "default": "Alex"
                },
                "archetype": {
                    "type": "string",
                    "description": "Character archetype or role",
                    "enum": ["hero", "mentor", "villain", "sidekick", "trickster", "innocent", "explorer"],
                    "default": "hero"
                },
                "genre": {
                    "type": "string",
                    "description": "Genre context for the character",
                    "default": "fantasy"
                }
            }
        })
    }
}

// Helper functions for content generation
fn generate_fantasy_story(length: &str, theme: &str) -> (String, String) {
    let title = format!("The Chronicles of {}", theme.to_title_case());
    let content = match length {
        "flash" => format!("In the mystical realm of Eldoria, {} would prove to be the key to salvation. The ancient prophecy spoke of a chosen one who would...", theme),
        "short" => format!("The moonlight filtered through the ancient oak trees as Lyra discovered the truth about {}. Her journey had led her through dark forests and forgotten ruins, but nothing had prepared her for this revelation. The crystal in her hand pulsed with an otherworldly light...", theme),
        _ => format!("Long ago, in a world where magic flowed like rivers through the land, the concept of {} held power beyond imagination. The great wizard Aldric had spent decades studying its mysteries, but even he could not have predicted what would unfold when the young apprentice Maya stumbled upon the forbidden tome...", theme)
    };
    (title, content)
}

fn generate_scifi_story(length: &str, theme: &str) -> (String, String) {
    let title = format!("Quantum {}", theme.to_title_case());
    let content = match length {
        "flash" => format!("The space station's AI announced: 'Anomaly detected in sector 7.' Captain Chen knew {} would be crucial to humanity's survival.", theme),
        "short" => format!("Commander Sarah Rodriguez stared at the readings from the deep space probe. The data suggested that {} wasn't just a concept—it was a measurable force that could revolutionize their understanding of the universe. The implications were staggering...", theme),
        _ => format!("In the year 2387, humanity had spread across the galaxy, but they had never encountered anything quite like this. The discovery on Kepler-442b challenged everything they knew about {}. Dr. Elena Vasquez, the lead xenobiologist, carefully examined the alien artifact that seemed to embody the very essence of what they had been searching for...", theme)
    };
    (title, content)
}

fn generate_mystery_story(length: &str, theme: &str) -> (String, String) {
    let title = format!("The {} Enigma", theme.to_title_case());
    let content = match length {
        "flash" => format!("Detective Morrison found the note: '{}' is not what it seems. Follow the clues, but trust no one.'", theme),
        "short" => format!("The rain drummed against Detective Kate Sullivan's office window as she reviewed the case files. Three victims, seemingly unconnected, except for one thing: they all had written about {} in their final days. The pattern was too precise to be coincidental...", theme),
        _ => format!("The foggy streets of Victorian London concealed many secrets, but none as perplexing as the case that had stumped Inspector James Whitmore for months. Each victim had been found with a single word carved into the scene: '{}'. The connection eluded him until he received an anonymous letter that would change everything...", theme)
    };
    (title, content)
}

fn generate_romance_story(length: &str, theme: &str) -> (String, String) {
    let title = format!("Love and {}", theme.to_title_case());
    let content = match length {
        "flash" => format!("Emma's heart skipped when she saw the message: 'Meet me where {} began.' She knew exactly where to go.", theme),
        "short" => format!("The coffee shop where Emma first met David held so many memories. As she sat at their usual table, she reflected on how {} had brought them together and now seemed to be pulling them apart. The letter in her hands contained words that would determine their future...", theme),
        _ => format!("Sophie had always believed that {} was just a fairy tale, something that happened to other people in romantic movies. But when she collided with the handsome stranger outside the bookstore, scattering her research papers about medieval literature across the sidewalk, she began to wonder if fate had other plans...", theme)
    };
    (title, content)
}

fn generate_adventure_story(length: &str, theme: &str) -> (String, String) {
    let title = format!("Quest for {}", theme.to_title_case());
    let content = match length {
        "flash" => format!("The map led to the temple where {} awaited. Jack checked his gear one last time before entering the ancient ruins.", theme),
        "short" => format!("The jungle was alive with sounds as archaeologist Dr. Maya Chen pushed through the dense vegetation. The legends spoke of a lost civilization that had mastered the secrets of {}. Her expedition had already faced numerous perils, but the real danger lay ahead...", theme),
        _ => format!("The call came at midnight: 'We've found it, Professor Martinez. The expedition to find the source of {} can finally begin.' Within hours, Maria was on a plane to the Amazon, her heart racing with anticipation. The local guides spoke in hushed tones about the cursed temple, but Maria knew that some discoveries were worth any risk...", theme)
    };
    (title, content)
}

fn generate_generic_story(genre: &str, length: &str, theme: &str) -> (String, String) {
    let title = format!("A Tale of {}", theme.to_title_case());
    let content = format!("This {} story explores the profound meaning of {} in ways that will captivate readers and leave them pondering long after the final page...", genre, theme);
    (title, content)
}

fn generate_sonnet(theme: &str) -> (String, String, String) {
    let title = format!("Sonnet on {}", theme.to_title_case());
    let content = format!(
        "When {} blooms bright in morning's golden light,\n\
         And shadows dance beneath the azure sky,\n\
         The world awakens to a wondrous sight,\n\
         As nature's beauty makes the spirit fly.\n\n\
         Through seasons past and futures yet to come,\n\
         This truth remains as constant as the sun,\n\
         That {} makes the heart's true rhythm hum,\n\
         And shows us how all souls become as one.\n\n\
         So let us pause and contemplate this day,\n\
         The gifts that {} has brought to every heart,\n\
         And find in simple moments grand display\n\
         Of how such beauty plays its vital part.\n\n\
         For in {}'s embrace we truly see\n\
         The path to our own immortality.", 
        theme, theme, theme, theme
    );
    (title, content, "Shakespearean sonnet (ABAB CDCD EFEF GG)".to_string())
}

fn generate_haiku(theme: &str) -> (String, String, String) {
    let title = format!("{} Haiku", theme.to_title_case());
    let content = match theme.to_lowercase().as_str() {
        "nature" => "Cherry blossoms fall\nGentle breeze carries petals\nSpring's fleeting beauty",
        "friendship" => "Laughter shared between\nTwo souls walking side by side\nBonds that time cannot break",
        "love" => "Hearts beating as one\nStars witness our whispered vows\nEternal promise",
        _ => &format!("{} whispers soft\nIn moments of quiet peace\nTruth reveals itself", theme.to_title_case())[..]
    };
    (title, content.to_string(), "Traditional haiku (5-7-5 syllable pattern)".to_string())
}

fn generate_limerick(theme: &str) -> (String, String, String) {
    let title = format!("A Limerick About {}", theme.to_title_case());
    let content = format!(
        "There once was a thing about {},\nWhich made people happy and free,\n    It would dance and would play,\n    Throughout night and day,\nAnd fill hearts with such sweet harmony!", 
        theme.to_lowercase()
    );
    (title, content, "Limerick (AABBA rhyme scheme)".to_string())
}

fn generate_ballad(theme: &str) -> (String, String, String) {
    let title = format!("Ballad of {}", theme.to_title_case());
    let content = format!(
        "Oh, listen to the tale I tell,\nOf {} so true and bright,\nThat guided souls through darkest dell,\nAnd brought them to the light.\n\n\
         The minstrels sang of days of old,\nWhen {} was young and new,\nAnd heroes brave and knights so bold\nWould seek it through and through.\n\n\
         So raise your voice and sing along,\nOf {}'s enduring fame,\nFor in this ancient, timeless song,\nWe honor its sweet name.",
        theme, theme, theme
    );
    (title, content, "Traditional ballad (ABAB rhyme scheme)".to_string())
}

fn generate_free_verse(theme: &str) -> (String, String, String) {
    let title = format!("Meditation on {}", theme.to_title_case());
    let content = format!(
        "{} flows like water\nthrough the spaces between\nour carefully constructed thoughts,\n\n\
         reminding us\nthat some truths\ncannot be captured\nin neat definitions\n\n\
         but only felt\nin the quiet moments\nwhen we stop\ntrying so hard\n\n\
         and simply\nallow ourselves\nto be\nwith what is.",
        theme.to_title_case()
    );
    (title, content, "Free verse (no fixed rhyme or meter)".to_string())
}

fn generate_character_details(name: &str, archetype: &str) -> Value {
    let (personality, backstory, motivation) = match archetype {
        "hero" => (
            "Brave, compassionate, and determined. Natural leader with a strong moral compass.",
            format!("{} grew up in a small village, witnessing injustice that shaped their desire to protect others.", name),
            "To right wrongs and protect the innocent from harm."
        ),
        "mentor" => (
            "Wise, patient, and experienced. Sees potential in others and guides them to greatness.",
            format!("{} once walked the path of adventure but now passes knowledge to the next generation.", name),
            "To ensure wisdom is preserved and shared with worthy successors."
        ),
        "villain" => (
            "Charismatic, cunning, and ruthlessly ambitious. Believes the ends justify the means.",
            format!("{} was once idealistic but became corrupted by power and betrayal.", name),
            "To reshape the world according to their vision, regardless of cost."
        ),
        _ => (
            "Complex and multifaceted, with both strengths and flaws that make them relatable.",
            format!("{} has lived through experiences that shaped their unique worldview.", name),
            "To find their place in the world and fulfill their personal destiny."
        )
    };

    json!({
        "name": name,
        "archetype": archetype,
        "personality": personality,
        "backstory": backstory,
        "motivation": motivation,
        "physical_description": generate_physical_description(),
        "skills": generate_character_skills(archetype),
        "relationships": generate_character_relationships(),
        "generated_at": chrono::Utc::now().to_rfc3339()
    })
}

fn generate_physical_description() -> String {
    "Medium height with expressive eyes that reflect their inner depth. Carries themselves with quiet confidence.".to_string()
}

fn generate_character_skills(archetype: &str) -> Vec<String> {
    match archetype {
        "hero" => vec!["Leadership".to_string(), "Combat".to_string(), "Diplomacy".to_string()],
        "mentor" => vec!["Ancient Knowledge".to_string(), "Teaching".to_string(), "Wisdom".to_string()],
        "villain" => vec!["Manipulation".to_string(), "Strategy".to_string(), "Dark Magic".to_string()],
        _ => vec!["Adaptability".to_string(), "Observation".to_string(), "Empathy".to_string()]
    }
}

fn generate_character_relationships() -> Vec<Value> {
    vec![
        json!({"type": "ally", "description": "Trusted companion who shares their journey"}),
        json!({"type": "rival", "description": "Someone who challenges them and drives growth"}),
        json!({"type": "family", "description": "Sibling or parent who provides emotional anchor"})
    ]
}

fn estimate_word_count(length: &str) -> u32 {
    match length {
        "flash" => 100,
        "short" => 500,
        "medium" => 1500,
        "long" => 3000,
        _ => 500
    }
}

fn estimate_reading_time(length: &str) -> u32 {
    estimate_word_count(length) / 200 // Average reading speed
}

trait ToTitleCase {
    fn to_title_case(&self) -> String;
}

impl ToTitleCase for str {
    fn to_title_case(&self) -> String {
        self.chars()
            .enumerate()
            .map(|(i, c)| {
                if i == 0 || self.chars().nth(i.saturating_sub(1)) == Some(' ') {
                    c.to_uppercase().collect::<String>()
                } else {
                    c.to_lowercase().collect::<String>()
                }
            })
            .collect()
    }
}

/// Create and configure the MCP server with creative content tools
async fn create_server(processing_delay: Duration) -> Result<mcp_server::McpServerImpl> {
    info!("Creating creative content server with multiple tools...");
    
    let story_tool: Arc<dyn McpTool> = Arc::new(GenerateStoryTool::new(processing_delay));
    let poem_tool: Arc<dyn McpTool> = Arc::new(CreatePoemTool::new(processing_delay));
    let character_tool: Arc<dyn McpTool> = Arc::new(DevelopCharacterTool::new(processing_delay));
    
    let server = McpServerBuilder::new()
        .with_name("creative-content-server")
        .with_version("1.0.0")
        .add_tool(story_tool)
        .add_tool(poem_tool)
        .add_tool(character_tool)
        .enable_tracing(true)
        .max_concurrent_requests(10)
        .build()?;
    
    info!("Created creative content server with {} tools", server.tool_count().await);
    Ok(server)
}

/// Run server with STDIO transport
async fn run_with_stdio(server: mcp_server::McpServerImpl) -> Result<()> {
    info!("Starting creative content server with STDIO transport");
    
    let transport = StdioTransport::with_defaults()?;
    let transport: Arc<dyn Transport> = Arc::new(transport);
    
    info!("STDIO transport ready - listening on stdin/stdout");
    info!("Available tools: generate_story, create_poem, develop_character");
    info!("Example: {{\"method\": \"generate_story\", \"params\": {{\"genre\": \"fantasy\", \"theme\": \"courage\", \"length\": \"short\"}}}}");
    
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
    
    info!("Starting creative content server with HTTP transport on {}...", addr);
    
    let transport = HttpTransport::with_defaults(addr)?;
    let transport: Arc<dyn Transport> = Arc::new(transport);
    
    info!("Creative content server is ready!");
    info!("HTTP endpoint: http://{}/mcp", addr);
    info!("Health check: http://{}/health", addr);
    info!("");
    info!("Available tools:");
    info!("  - generate_story: Create stories in various genres");
    info!("  - create_poem: Generate poetry in different styles");
    info!("  - develop_character: Create detailed character profiles");
    info!("");
    info!("Example curl requests:");
    info!("# Generate a fantasy story");
    info!("curl -X POST http://{}/mcp \\", addr);
    info!("  -H 'Content-Type: application/json' \\");
    info!("  -d '{{\"method\": \"generate_story\", \"params\": {{\"genre\": \"fantasy\", \"theme\": \"courage\", \"length\": \"short\"}}}}'");
    info!("");
    info!("# Create a haiku");
    info!("curl -X POST http://{}/mcp \\", addr);
    info!("  -H 'Content-Type: application/json' \\");
    info!("  -d '{{\"method\": \"create_poem\", \"params\": {{\"style\": \"haiku\", \"theme\": \"nature\"}}}}'");
    
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
                    tracing_subscriber::EnvFilter::new(format!("creative_content_server={},mcp_server={},mcp_transport={},mcp_core={}", level, level, level, level))
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
    
    info!("MCP Creative Content Server v0.1.0");
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
    async fn test_generate_story_tool() {
        let tool = GenerateStoryTool::new(Duration::from_millis(10));
        let mut params = HashMap::new();
        params.insert("genre".to_string(), json!("fantasy"));
        params.insert("theme".to_string(), json!("adventure"));
        
        let request = McpRequest::CallTool {
            name: "generate_story".to_string(),
            arguments: params
        };
        
        let result = tool.call(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_poem_tool() {
        let tool = CreatePoemTool::new(Duration::from_millis(10));
        let mut params = HashMap::new();
        params.insert("style".to_string(), json!("haiku"));
        params.insert("theme".to_string(), json!("nature"));
        
        let request = McpRequest::CallTool {
            name: "create_poem".to_string(),
            arguments: params
        };
        
        let result = tool.call(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_develop_character_tool() {
        let tool = DevelopCharacterTool::new(Duration::from_millis(10));
        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("Aragorn"));
        params.insert("archetype".to_string(), json!("hero"));
        
        let request = McpRequest::CallTool {
            name: "develop_character".to_string(),
            arguments: params
        };
        
        let result = tool.call(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_story_genre() {
        let tool = GenerateStoryTool::new(Duration::from_millis(10));
        let mut params = HashMap::new();
        params.insert("genre".to_string(), json!("invalid_genre"));
        
        let request = McpRequest::CallTool {
            name: "generate_story".to_string(),
            arguments: params
        };
        
        let result = tool.call(request).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_metadata() {
        let story_tool = GenerateStoryTool::new(Duration::from_millis(10));
        let poem_tool = CreatePoemTool::new(Duration::from_millis(10));
        let character_tool = DevelopCharacterTool::new(Duration::from_millis(10));
        
        assert_eq!(story_tool.name(), "generate_story");
        assert_eq!(poem_tool.name(), "create_poem");
        assert_eq!(character_tool.name(), "develop_character");
        
        assert!(!story_tool.description().is_empty());
        assert!(!poem_tool.description().is_empty());
        assert!(!character_tool.description().is_empty());
    }
}