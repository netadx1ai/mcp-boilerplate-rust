//! Sampling Module
//!
//! This module provides sampling support for MCP 2025-11-25 specification.
//! Sampling allows servers to request LLM completions from the client,
//! with support for tool calling during the sampling process.
//!
//! # Features
//!
//! - Create message requests with conversation history
//! - Tool calling within sampling (tools/toolChoice parameters)
//! - Model preferences (cost, speed, intelligence priorities)
//! - Progress tracking for long-running generations
//!
//! # Example
//!
//! ```rust
//! use crate::mcp::sampling::{SamplingRequest, ToolChoice};
//!
//! let request = SamplingRequest::new("You are a helpful assistant")
//!     .add_user_message("What's the weather like?")
//!     .with_tools(vec![weather_tool])
//!     .with_tool_choice(ToolChoice::Auto)
//!     .with_max_tokens(1000)
//!     .build();
//! ```

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

// Re-export types from rmcp for convenience
pub use rmcp::model::{
    Content, CreateMessageRequestParam, CreateMessageResult, ModelHint, ModelPreferences,
    Role, SamplingMessage,
};

/// Tool choice options for sampling requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoice {
    /// Let the model decide whether to use tools
    Auto,
    /// Don't use any tools
    None,
    /// Force the model to use a tool
    Required,
    /// Force the model to use a specific tool
    Tool(String),
}

impl Default for ToolChoice {
    fn default() -> Self {
        Self::Auto
    }
}

impl ToolChoice {
    pub fn to_json(&self) -> Value {
        match self {
            ToolChoice::Auto => json!({ "type": "auto" }),
            ToolChoice::None => json!({ "type": "none" }),
            ToolChoice::Required => json!({ "type": "required" }),
            ToolChoice::Tool(name) => json!({
                "type": "tool",
                "name": name
            }),
        }
    }
}

/// Tool definition for sampling requests
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplingTool {
    /// Unique name of the tool
    pub name: String,
    /// Human-readable description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema for the tool's input parameters
    pub input_schema: Value,
}

impl SamplingTool {
    pub fn new(name: impl Into<String>, input_schema: Value) -> Self {
        Self {
            name: name.into(),
            description: None,
            input_schema,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Tool call request from the model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCall {
    /// Unique ID for this tool call
    pub id: String,
    /// Name of the tool to call
    pub name: String,
    /// Arguments to pass to the tool
    pub arguments: Value,
}

/// Tool call result to return to the model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallResult {
    /// ID of the tool call this is responding to
    pub tool_call_id: String,
    /// Result content
    pub content: Vec<Content>,
    /// Whether this represents an error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

impl ToolCallResult {
    pub fn success(tool_call_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content: vec![Content::text(text.into())],
            is_error: Some(false),
        }
    }

    pub fn error(tool_call_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content: vec![Content::text(message.into())],
            is_error: Some(true),
        }
    }

    pub fn with_content(tool_call_id: impl Into<String>, content: Vec<Content>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content,
            is_error: Some(false),
        }
    }
}

/// Extended sampling request with tool support
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplingRequest {
    /// Conversation messages
    pub messages: Vec<SamplingMessage>,
    /// System prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    /// Model preferences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_preferences: Option<ModelPreferences>,
    /// Maximum tokens to generate
    pub max_tokens: u32,
    /// Temperature (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    /// Tools available for the model to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<SamplingTool>>,
    /// How the model should choose tools
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

impl SamplingRequest {
    /// Create a new sampling request with system prompt
    pub fn new(system_prompt: impl Into<String>) -> SamplingRequestBuilder {
        SamplingRequestBuilder::new(system_prompt.into())
    }

    /// Create a new sampling request without system prompt
    pub fn without_system() -> SamplingRequestBuilder {
        SamplingRequestBuilder::without_system()
    }

    /// Convert to base CreateMessageRequestParam (without tool extensions)
    pub fn to_base_param(&self) -> CreateMessageRequestParam {
        CreateMessageRequestParam {
            messages: self.messages.clone(),
            model_preferences: self.model_preferences.clone(),
            system_prompt: self.system_prompt.clone(),
            include_context: None,
            temperature: self.temperature,
            max_tokens: self.max_tokens,
            stop_sequences: self.stop_sequences.clone(),
            metadata: self.to_extended_metadata(),
        }
    }

    /// Convert to extended metadata including tools
    fn to_extended_metadata(&self) -> Option<Value> {
        let mut meta = self.metadata.clone().unwrap_or(json!({}));

        if let Some(tools) = &self.tools {
            if let Value::Object(ref mut map) = meta {
                map.insert("tools".to_string(), json!(tools));
            }
        }

        if let Some(tool_choice) = &self.tool_choice {
            if let Value::Object(ref mut map) = meta {
                map.insert("toolChoice".to_string(), tool_choice.to_json());
            }
        }

        if meta == json!({}) {
            None
        } else {
            Some(meta)
        }
    }
}

/// Builder for sampling requests
#[derive(Debug)]
pub struct SamplingRequestBuilder {
    system_prompt: Option<String>,
    messages: Vec<SamplingMessage>,
    model_preferences: Option<ModelPreferences>,
    max_tokens: u32,
    temperature: Option<f32>,
    stop_sequences: Option<Vec<String>>,
    tools: Option<Vec<SamplingTool>>,
    tool_choice: Option<ToolChoice>,
    metadata: Option<Value>,
}

impl SamplingRequestBuilder {
    fn new(system_prompt: String) -> Self {
        Self {
            system_prompt: Some(system_prompt),
            messages: Vec::new(),
            model_preferences: None,
            max_tokens: 1024,
            temperature: None,
            stop_sequences: None,
            tools: None,
            tool_choice: None,
            metadata: None,
        }
    }

    fn without_system() -> Self {
        Self {
            system_prompt: None,
            messages: Vec::new(),
            model_preferences: None,
            max_tokens: 1024,
            temperature: None,
            stop_sequences: None,
            tools: None,
            tool_choice: None,
            metadata: None,
        }
    }

    /// Add a user message
    pub fn add_user_message(mut self, text: impl Into<String>) -> Self {
        self.messages.push(SamplingMessage {
            role: Role::User,
            content: Content::text(text.into()),
        });
        self
    }

    /// Add an assistant message
    pub fn add_assistant_message(mut self, text: impl Into<String>) -> Self {
        self.messages.push(SamplingMessage {
            role: Role::Assistant,
            content: Content::text(text.into()),
        });
        self
    }

    /// Add a custom message
    pub fn add_message(mut self, message: SamplingMessage) -> Self {
        self.messages.push(message);
        self
    }

    /// Set model preferences
    pub fn with_model_preferences(mut self, preferences: ModelPreferences) -> Self {
        self.model_preferences = Some(preferences);
        self
    }

    /// Prefer cheaper models
    pub fn prefer_cost(mut self, priority: f32) -> Self {
        let prefs = self.model_preferences.get_or_insert_with(|| ModelPreferences {
            hints: None,
            cost_priority: None,
            speed_priority: None,
            intelligence_priority: None,
        });
        prefs.cost_priority = Some(priority.clamp(0.0, 1.0));
        self
    }

    /// Prefer faster models
    pub fn prefer_speed(mut self, priority: f32) -> Self {
        let prefs = self.model_preferences.get_or_insert_with(|| ModelPreferences {
            hints: None,
            cost_priority: None,
            speed_priority: None,
            intelligence_priority: None,
        });
        prefs.speed_priority = Some(priority.clamp(0.0, 1.0));
        self
    }

    /// Prefer more intelligent models
    pub fn prefer_intelligence(mut self, priority: f32) -> Self {
        let prefs = self.model_preferences.get_or_insert_with(|| ModelPreferences {
            hints: None,
            cost_priority: None,
            speed_priority: None,
            intelligence_priority: None,
        });
        prefs.intelligence_priority = Some(priority.clamp(0.0, 1.0));
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature.clamp(0.0, 1.0));
        self
    }

    /// Set stop sequences
    pub fn with_stop_sequences(mut self, sequences: Vec<String>) -> Self {
        self.stop_sequences = Some(sequences);
        self
    }

    /// Set available tools
    pub fn with_tools(mut self, tools: Vec<SamplingTool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Add a single tool
    pub fn add_tool(mut self, tool: SamplingTool) -> Self {
        self.tools.get_or_insert_with(Vec::new).push(tool);
        self
    }

    /// Set tool choice
    pub fn with_tool_choice(mut self, choice: ToolChoice) -> Self {
        self.tool_choice = Some(choice);
        self
    }

    /// Set additional metadata
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Build the sampling request
    pub fn build(self) -> SamplingRequest {
        SamplingRequest {
            messages: self.messages,
            system_prompt: self.system_prompt,
            model_preferences: self.model_preferences,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            stop_sequences: self.stop_sequences,
            tools: self.tools,
            tool_choice: self.tool_choice,
            metadata: self.metadata,
        }
    }
}

/// Extended sampling response with tool call support
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplingResponse {
    /// The model that generated the response
    pub model: String,
    /// Stop reason
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    /// Generated message
    pub message: SamplingMessage,
    /// Tool calls requested by the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl SamplingResponse {
    /// Check if the model requested tool calls
    pub fn has_tool_calls(&self) -> bool {
        self.tool_calls
            .as_ref()
            .map(|t| !t.is_empty())
            .unwrap_or(false)
    }

    /// Get the text content if available
    pub fn get_text(&self) -> Option<String> {
        self.message.content.raw.as_text().map(|t| t.text.clone())
    }

    /// Check if generation stopped due to max tokens
    pub fn is_max_tokens(&self) -> bool {
        self.stop_reason
            .as_ref()
            .map(|r| r == "maxTokens")
            .unwrap_or(false)
    }

    /// Check if generation completed naturally
    pub fn is_end_turn(&self) -> bool {
        self.stop_reason
            .as_ref()
            .map(|r| r == "endTurn")
            .unwrap_or(false)
    }
}

impl From<CreateMessageResult> for SamplingResponse {
    fn from(result: CreateMessageResult) -> Self {
        Self {
            model: result.model,
            stop_reason: result.stop_reason,
            message: result.message,
            tool_calls: None, // Parse from message content if needed
        }
    }
}

/// Sampling session for multi-turn conversations with tool use
#[derive(Debug)]
pub struct SamplingSession {
    system_prompt: Option<String>,
    messages: Vec<SamplingMessage>,
    tools: Vec<SamplingTool>,
    tool_choice: ToolChoice,
    max_tokens: u32,
    temperature: Option<f32>,
}

impl SamplingSession {
    /// Create a new sampling session
    pub fn new(system_prompt: impl Into<String>) -> Self {
        Self {
            system_prompt: Some(system_prompt.into()),
            messages: Vec::new(),
            tools: Vec::new(),
            tool_choice: ToolChoice::Auto,
            max_tokens: 1024,
            temperature: None,
        }
    }

    /// Create a session without system prompt
    pub fn without_system() -> Self {
        Self {
            system_prompt: None,
            messages: Vec::new(),
            tools: Vec::new(),
            tool_choice: ToolChoice::Auto,
            max_tokens: 1024,
            temperature: None,
        }
    }

    /// Add a tool to the session
    pub fn add_tool(&mut self, tool: SamplingTool) {
        self.tools.push(tool);
    }

    /// Set tools for the session
    pub fn set_tools(&mut self, tools: Vec<SamplingTool>) {
        self.tools = tools;
    }

    /// Set tool choice
    pub fn set_tool_choice(&mut self, choice: ToolChoice) {
        self.tool_choice = choice;
    }

    /// Set max tokens
    pub fn set_max_tokens(&mut self, max_tokens: u32) {
        self.max_tokens = max_tokens;
    }

    /// Set temperature
    pub fn set_temperature(&mut self, temperature: f32) {
        self.temperature = Some(temperature);
    }

    /// Add a user message
    pub fn add_user_message(&mut self, text: impl Into<String>) {
        self.messages.push(SamplingMessage {
            role: Role::User,
            content: Content::text(text.into()),
        });
    }

    /// Add an assistant message
    pub fn add_assistant_message(&mut self, text: impl Into<String>) {
        self.messages.push(SamplingMessage {
            role: Role::Assistant,
            content: Content::text(text.into()),
        });
    }

    /// Add a message
    pub fn add_message(&mut self, message: SamplingMessage) {
        self.messages.push(message);
    }

    /// Add a tool result to the conversation
    pub fn add_tool_result(&mut self, result: ToolCallResult) {
        // Tool results are typically added as user messages with special content
        let content = json!({
            "type": "tool_result",
            "tool_call_id": result.tool_call_id,
            "content": result.content,
            "is_error": result.is_error
        });
        self.messages.push(SamplingMessage {
            role: Role::User,
            content: Content::text(content.to_string()),
        });
    }

    /// Build a request for the current session state
    pub fn build_request(&self) -> SamplingRequest {
        SamplingRequest {
            messages: self.messages.clone(),
            system_prompt: self.system_prompt.clone(),
            model_preferences: None,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            stop_sequences: None,
            tools: if self.tools.is_empty() {
                None
            } else {
                Some(self.tools.clone())
            },
            tool_choice: Some(self.tool_choice.clone()),
            metadata: None,
        }
    }

    /// Process a response and update the session
    pub fn process_response(&mut self, response: &SamplingResponse) {
        self.messages.push(response.message.clone());
    }

    /// Get the conversation history
    pub fn messages(&self) -> &[SamplingMessage] {
        &self.messages
    }

    /// Clear the conversation history
    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }
}

/// Tool executor trait for handling tool calls
pub trait ToolExecutor: Send + Sync {
    /// Execute a tool call and return the result
    fn execute(
        &self,
        tool_call: &ToolCall,
    ) -> impl std::future::Future<Output = ToolCallResult> + Send;
}

/// Simple function-based tool executor
pub struct FnToolExecutor<F>
where
    F: Fn(&str, &Value) -> ToolCallResult + Send + Sync,
{
    executor: F,
}

impl<F> FnToolExecutor<F>
where
    F: Fn(&str, &Value) -> ToolCallResult + Send + Sync,
{
    pub fn new(executor: F) -> Self {
        Self { executor }
    }
}

impl<F> ToolExecutor for FnToolExecutor<F>
where
    F: Fn(&str, &Value) -> ToolCallResult + Send + Sync,
{
    async fn execute(&self, tool_call: &ToolCall) -> ToolCallResult {
        (self.executor)(&tool_call.name, &tool_call.arguments)
    }
}

/// Registry for tool executors
pub struct ToolExecutorRegistry {
    executors: HashMap<String, Box<dyn Fn(&Value) -> ToolCallResult + Send + Sync>>,
}

impl ToolExecutorRegistry {
    pub fn new() -> Self {
        Self {
            executors: HashMap::new(),
        }
    }

    /// Register a tool executor
    pub fn register<F>(&mut self, name: impl Into<String>, executor: F)
    where
        F: Fn(&Value) -> ToolCallResult + Send + Sync + 'static,
    {
        self.executors.insert(name.into(), Box::new(executor));
    }

    /// Execute a tool call
    pub fn execute(&self, tool_call: &ToolCall) -> ToolCallResult {
        match self.executors.get(&tool_call.name) {
            Some(executor) => executor(&tool_call.arguments),
            None => ToolCallResult::error(
                &tool_call.id,
                format!("Unknown tool: {}", tool_call.name),
            ),
        }
    }

    /// Check if a tool is registered
    pub fn has_tool(&self, name: &str) -> bool {
        self.executors.contains_key(name)
    }

    /// Get list of registered tool names
    pub fn tool_names(&self) -> Vec<&String> {
        self.executors.keys().collect()
    }
}

impl Default for ToolExecutorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sampling_request_builder() {
        let request = SamplingRequest::new("You are a helpful assistant")
            .add_user_message("Hello!")
            .with_max_tokens(500)
            .with_temperature(0.7)
            .build();

        assert_eq!(
            request.system_prompt,
            Some("You are a helpful assistant".to_string())
        );
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.max_tokens, 500);
        assert_eq!(request.temperature, Some(0.7));
    }

    #[test]
    fn test_sampling_request_with_tools() {
        let tool = SamplingTool::new(
            "get_weather",
            json!({
                "type": "object",
                "properties": {
                    "location": { "type": "string" }
                }
            }),
        )
        .with_description("Get weather for a location");

        let request = SamplingRequest::new("You are a weather assistant")
            .add_user_message("What's the weather in Tokyo?")
            .with_tools(vec![tool])
            .with_tool_choice(ToolChoice::Auto)
            .build();

        assert!(request.tools.is_some());
        assert_eq!(request.tools.unwrap().len(), 1);
        assert_eq!(request.tool_choice, Some(ToolChoice::Auto));
    }

    #[test]
    fn test_tool_choice_serialization() {
        assert_eq!(ToolChoice::Auto.to_json(), json!({"type": "auto"}));
        assert_eq!(ToolChoice::None.to_json(), json!({"type": "none"}));
        assert_eq!(ToolChoice::Required.to_json(), json!({"type": "required"}));
        assert_eq!(
            ToolChoice::Tool("my_tool".to_string()).to_json(),
            json!({"type": "tool", "name": "my_tool"})
        );
    }

    #[test]
    fn test_tool_call_result() {
        let success = ToolCallResult::success("call_1", "Result text");
        assert_eq!(success.tool_call_id, "call_1");
        assert_eq!(success.is_error, Some(false));

        let error = ToolCallResult::error("call_2", "Something went wrong");
        assert_eq!(error.tool_call_id, "call_2");
        assert_eq!(error.is_error, Some(true));
    }

    #[test]
    fn test_sampling_session() {
        let mut session = SamplingSession::new("You are helpful");
        session.add_user_message("Hello");
        session.add_tool(SamplingTool::new("test", json!({})));

        let request = session.build_request();
        assert_eq!(request.messages.len(), 1);
        assert!(request.tools.is_some());
    }

    #[test]
    fn test_tool_executor_registry() {
        let mut registry = ToolExecutorRegistry::new();
        registry.register("echo", |args| {
            let text = args
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("no text");
            ToolCallResult::success("test_id", text)
        });

        assert!(registry.has_tool("echo"));
        assert!(!registry.has_tool("unknown"));

        let call = ToolCall {
            id: "call_1".to_string(),
            name: "echo".to_string(),
            arguments: json!({"text": "hello"}),
        };

        let result = registry.execute(&call);
        assert_eq!(result.is_error, Some(false));
    }

    #[test]
    fn test_model_preferences() {
        let request = SamplingRequest::new("Test")
            .prefer_cost(0.8)
            .prefer_speed(0.5)
            .prefer_intelligence(0.9)
            .build();

        let prefs = request.model_preferences.unwrap();
        assert_eq!(prefs.cost_priority, Some(0.8));
        assert_eq!(prefs.speed_priority, Some(0.5));
        assert_eq!(prefs.intelligence_priority, Some(0.9));
    }
}