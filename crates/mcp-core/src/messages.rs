//! Core message types for the MCP protocol.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// MCP protocol request message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "method", content = "params")]
pub enum McpRequest {
    /// Initialize connection with client
    #[serde(rename = "initialize")]
    Initialize {
        /// Protocol version
        protocol_version: String,
        /// Client capabilities
        capabilities: ClientCapabilities,
        /// Client information
        client_info: ClientInfo,
    },

    /// List available tools
    #[serde(rename = "tools/list")]
    ListTools {
        /// Optional cursor for pagination
        cursor: Option<String>,
    },

    /// Call a specific tool
    #[serde(rename = "tools/call")]
    CallTool {
        /// Name of the tool to call
        name: String,
        /// Arguments to pass to the tool
        #[serde(default)]
        arguments: HashMap<String, serde_json::Value>,
    },

    /// List available resources
    #[serde(rename = "resources/list")]
    ListResources {
        /// Optional cursor for pagination
        cursor: Option<String>,
    },

    /// Read a specific resource
    #[serde(rename = "resources/read")]
    ReadResource {
        /// URI of the resource to read
        uri: String,
    },

    /// Subscribe to resource changes
    #[serde(rename = "resources/subscribe")]
    SubscribeResource {
        /// URI of the resource to subscribe to
        uri: String,
    },

    /// Unsubscribe from resource changes
    #[serde(rename = "resources/unsubscribe")]
    UnsubscribeResource {
        /// URI of the resource to unsubscribe from
        uri: String,
    },

    /// List available prompts
    #[serde(rename = "prompts/list")]
    ListPrompts {
        /// Optional cursor for pagination
        cursor: Option<String>,
    },

    /// Get a specific prompt
    #[serde(rename = "prompts/get")]
    GetPrompt {
        /// Name of the prompt to get
        name: String,
        /// Arguments for the prompt
        #[serde(default)]
        arguments: HashMap<String, serde_json::Value>,
    },

    /// Complete text using the server
    #[serde(rename = "completion/complete")]
    Complete {
        /// Reference to the completion request
        #[serde(rename = "ref")]
        reference: CompletionReference,
        /// Completion argument
        argument: CompletionArgument,
    },

    /// Set logging level
    #[serde(rename = "logging/setLevel")]
    SetLoggingLevel {
        /// New logging level
        level: LoggingLevel,
    },

    /// Generic ping for connection health
    #[serde(rename = "ping")]
    Ping,
}

/// MCP protocol response message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum McpResponse {
    /// Successful response
    Success {
        /// Response data
        #[serde(flatten)]
        result: ResponseResult,
    },
    /// Error response
    Error {
        /// Error details
        error: crate::error::McpError,
    },
}

/// Response result variants
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "_type")]
pub enum ResponseResult {
    /// Initialize response
    #[serde(rename = "initialize")]
    Initialize {
        /// Protocol version
        #[serde(rename = "protocolVersion")]
        protocol_version: String,
        /// Server capabilities
        capabilities: ServerCapabilities,
        /// Server information
        #[serde(rename = "serverInfo")]
        server_info: ServerInfo,
    },

    /// Tools list response
    #[serde(rename = "tools")]
    Tools {
        /// List of available tools
        tools: Vec<Tool>,
        /// Optional cursor for pagination
        #[serde(skip_serializing_if = "Option::is_none")]
        next_cursor: Option<String>,
    },

    /// Tool call response
    #[serde(rename = "toolResult")]
    ToolResult {
        /// Tool result content
        content: Vec<ToolContent>,
        /// Whether the tool call was successful
        #[serde(rename = "isError")]
        is_error: bool,
    },

    /// Resources list response
    #[serde(rename = "resources")]
    Resources {
        /// List of available resources
        resources: Vec<Resource>,
        /// Optional cursor for pagination
        #[serde(skip_serializing_if = "Option::is_none")]
        next_cursor: Option<String>,
    },

    /// Resource content response
    #[serde(rename = "resourceContents")]
    ResourceContents {
        /// Resource contents
        contents: Vec<ResourceContent>,
    },

    /// Prompts list response
    #[serde(rename = "prompts")]
    Prompts {
        /// List of available prompts
        prompts: Vec<Prompt>,
        /// Optional cursor for pagination
        #[serde(skip_serializing_if = "Option::is_none")]
        next_cursor: Option<String>,
    },

    /// Prompt response
    #[serde(rename = "getPrompt")]
    GetPrompt {
        /// Prompt description
        description: Option<String>,
        /// Prompt messages
        messages: Vec<PromptMessage>,
    },

    /// Completion response
    #[serde(rename = "completion")]
    Completion {
        /// Completion result
        completion: CompletionResult,
    },

    /// Simple success response
    #[serde(rename = "success")]
    Success {
        /// Success message
        message: String,
    },

    /// Pong response
    #[serde(rename = "pong")]
    Pong,
}

impl McpResponse {
    /// Create a successful response
    pub fn success(result: ResponseResult) -> Self {
        McpResponse::Success { result }
    }

    /// Create an error response
    pub fn error(error: crate::error::McpError) -> Self {
        McpResponse::Error { error }
    }

    /// Create a simple success response
    pub fn simple_success(message: impl Into<String>) -> Self {
        McpResponse::Success {
            result: ResponseResult::Success {
                message: message.into(),
            },
        }
    }

    /// Create a pong response
    pub fn pong() -> Self {
        McpResponse::Success {
            result: ResponseResult::Pong,
        }
    }
}

/// Client capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ClientCapabilities {
    /// Experimental capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, serde_json::Value>>,
    /// Sampling capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<SamplingCapabilities>,
}

/// Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ServerCapabilities {
    /// Experimental capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, serde_json::Value>>,
    /// Logging capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<LoggingCapabilities>,
    /// Prompts capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapabilities>,
    /// Resources capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapabilities>,
    /// Tools capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapabilities>,
}

/// Client information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClientInfo {
    /// Client name
    pub name: String,
    /// Client version
    pub version: String,
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerInfo {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tool {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Input schema for the tool
    #[serde(rename = "inputSchema")]
    pub input_schema: ToolInputSchema,
}

/// Tool input schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolInputSchema {
    /// Schema type
    #[serde(rename = "type")]
    pub schema_type: String,
    /// Schema properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, serde_json::Value>>,
    /// Required properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

/// Tool content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ToolContent {
    /// Text content
    #[serde(rename = "text")]
    Text {
        /// Text content
        text: String,
    },
    /// Image content
    #[serde(rename = "image")]
    Image {
        /// Image data (base64 encoded)
        data: String,
        /// MIME type
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    /// Resource content
    #[serde(rename = "resource")]
    Resource {
        /// Resource URI
        uri: String,
        /// Optional MIME type
        #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
    },
}

/// Resource definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resource {
    /// Resource URI
    pub uri: String,
    /// Resource name
    pub name: String,
    /// Resource description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// MIME type
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Resource content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceContent {
    /// Resource URI
    pub uri: String,
    /// MIME type
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    /// Content data
    #[serde(flatten)]
    pub content: ResourceContentData,
}

/// Resource content data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ResourceContentData {
    /// Text content
    Text { text: String },
    /// Binary content (base64 encoded)
    Blob { blob: String },
}

/// Prompt definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Prompt {
    /// Prompt name
    pub name: String,
    /// Prompt description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Prompt arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
}

/// Prompt argument
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptArgument {
    /// Argument name
    pub name: String,
    /// Argument description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the argument is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

/// Prompt message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptMessage {
    /// Message role
    pub role: MessageRole,
    /// Message content
    pub content: PromptContent,
}

/// Message role
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// User message
    User,
    /// Assistant message
    Assistant,
    /// System message
    System,
}

/// Prompt content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum PromptContent {
    /// Text content
    #[serde(rename = "text")]
    Text {
        /// Text content
        text: String,
    },
    /// Image content
    #[serde(rename = "image")]
    Image {
        /// Image data (base64 encoded)
        data: String,
        /// MIME type
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
}

/// Completion reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum CompletionReference {
    /// Prompt reference
    #[serde(rename = "ref/prompt")]
    Prompt {
        /// Prompt name
        name: String,
    },
    /// Resource reference
    #[serde(rename = "ref/resource")]
    Resource {
        /// Resource URI
        uri: String,
    },
}

/// Completion argument
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletionArgument {
    /// Argument name
    pub name: String,
    /// Argument value
    pub value: String,
}

/// Completion result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletionResult {
    /// Completion values
    pub values: Vec<String>,
    /// Total number of completions available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u32>,
    /// Whether there are more completions available
    #[serde(rename = "hasMore", skip_serializing_if = "Option::is_none")]
    pub has_more: Option<bool>,
}

/// Logging level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LoggingLevel {
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Notice level
    Notice,
    /// Warning level
    Warning,
    /// Error level
    Error,
    /// Critical level
    Critical,
    /// Alert level
    Alert,
    /// Emergency level
    Emergency,
}

/// Sampling capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SamplingCapabilities {}

/// Logging capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct LoggingCapabilities {}

/// Prompts capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PromptsCapabilities {
    /// Whether the server supports prompt listing
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Resources capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ResourcesCapabilities {
    /// Whether the server supports resource subscription
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,
    /// Whether the server supports resource listing changes
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Tools capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ToolsCapabilities {
    /// Whether the server supports tool listing changes
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Tool call wrapper for convenience
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    /// Unique call identifier
    pub id: String,
    /// Tool name
    pub name: String,
    /// Tool arguments
    #[serde(default)]
    pub arguments: HashMap<String, serde_json::Value>,
}

impl ToolCall {
    /// Create a new tool call
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            arguments: HashMap::new(),
        }
    }

    /// Add an argument to the tool call
    pub fn with_argument(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.arguments.insert(key.into(), value);
        self
    }
}

/// Tool result wrapper for convenience
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolResult {
    /// Call ID this result corresponds to
    pub call_id: String,
    /// Result content
    pub content: Vec<ToolContent>,
    /// Whether the call resulted in an error
    pub is_error: bool,
}

impl ToolResult {
    /// Create a successful tool result
    pub fn success(call_id: impl Into<String>, content: Vec<ToolContent>) -> Self {
        Self {
            call_id: call_id.into(),
            content,
            is_error: false,
        }
    }

    /// Create an error tool result
    pub fn error(call_id: impl Into<String>, error_message: impl Into<String>) -> Self {
        Self {
            call_id: call_id.into(),
            content: vec![ToolContent::Text {
                text: error_message.into(),
            }],
            is_error: true,
        }
    }

    /// Create a text result
    pub fn text(call_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self::success(
            call_id,
            vec![ToolContent::Text {
                text: text.into(),
            }],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_call_creation() {
        let call = ToolCall::new("test_tool")
            .with_argument("param1", serde_json::json!("value1"))
            .with_argument("param2", serde_json::json!(42));

        assert_eq!(call.name, "test_tool");
        assert_eq!(call.arguments.len(), 2);
        assert_eq!(call.arguments["param1"], serde_json::json!("value1"));
        assert_eq!(call.arguments["param2"], serde_json::json!(42));
    }

    #[test]
    fn test_tool_result_creation() {
        let result = ToolResult::text("call-123", "Hello, world!");
        assert_eq!(result.call_id, "call-123");
        assert!(!result.is_error);
        assert_eq!(result.content.len(), 1);

        if let ToolContent::Text { text } = &result.content[0] {
            assert_eq!(text, "Hello, world!");
        } else {
            panic!("Expected text content");
        }
    }

    #[test]
    fn test_response_creation() {
        let response = McpResponse::simple_success("Operation completed");
        match response {
            McpResponse::Success {
                result: ResponseResult::Success { message },
            } => {
                assert_eq!(message, "Operation completed");
            }
            _ => panic!("Expected success response"),
        }
    }

    #[test]
    fn test_message_serialization() {
        let request = McpRequest::Ping;
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: McpRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, deserialized);
    }
}