#[cfg(feature = "http")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "http")]
use serde_json::Value;
#[cfg(feature = "http")]
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AppState {
    #[allow(dead_code)]
    pub start_time: chrono::DateTime<chrono::Utc>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            start_time: chrono::Utc::now(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "http")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInput {
    #[serde(flatten)]
    pub params: HashMap<String, Value>,
}

#[cfg(feature = "http")]
impl ToolInput {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.params.get(key)
    }

    #[allow(dead_code)]
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.params
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    #[allow(dead_code)]
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.params.get(key).and_then(|v| v.as_i64())
    }

    #[allow(dead_code)]
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.params.get(key).and_then(|v| v.as_bool())
    }
}

#[cfg(feature = "http")]
impl Default for ToolInput {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "http")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    pub content: Vec<ContentItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[cfg(feature = "http")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentItem {
    #[serde(rename = "text")]
    Text { text: String },
}

#[cfg(feature = "http")]
impl ToolOutput {
    #[allow(dead_code)]
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: vec![ContentItem::Text { text: text.into() }],
            is_error: Some(false),
        }
    }

    #[allow(dead_code)]
    pub fn json(data: Value) -> Self {
        let text = serde_json::to_string_pretty(&data)
            .unwrap_or_else(|_| "Error serializing data".to_string());
        Self::text(text)
    }

    #[allow(dead_code)]
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: vec![ContentItem::Text {
                text: message.into(),
            }],
            is_error: Some(true),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Invalid params: {0}")]
    InvalidParams(String),

    #[error("Missing parameter: {0}")]
    MissingParameter(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

#[cfg(feature = "http")]
impl McpError {
    #[allow(dead_code)]
    pub fn to_output(&self) -> ToolOutput {
        ToolOutput::error(self.to_string())
    }
}

#[allow(dead_code)]
pub type McpResult<T> = Result<T, McpError>;