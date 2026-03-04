//! Shared types for mcp-dautruongvui-be

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

#[allow(dead_code)]
pub type McpResult<T> = Result<T, McpError>;