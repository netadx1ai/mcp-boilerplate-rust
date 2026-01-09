//! Tool Metadata Module (MCP 2025-11-25)
//!
//! Provides icons, output schemas, and execution configuration for tools.
//! This module extends tool definitions with optional metadata per spec.
//!
//! Last Updated: 2026-01-09 13:53 HCMC

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Tool icon definition per MCP 2025-11-25
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolIcon {
    /// Icon source (URL or data URI)
    pub src: String,
    /// MIME type of the icon
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Icon sizes (e.g., "16x16", "32x32", "any")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sizes: Option<String>,
}

impl ToolIcon {
    pub fn new(src: impl Into<String>) -> Self {
        Self {
            src: src.into(),
            mime_type: None,
            sizes: None,
        }
    }

    pub fn svg(src: impl Into<String>) -> Self {
        Self {
            src: src.into(),
            mime_type: Some("image/svg+xml".into()),
            sizes: Some("any".into()),
        }
    }

    pub fn png(src: impl Into<String>, size: u32) -> Self {
        Self {
            src: src.into(),
            mime_type: Some("image/png".into()),
            sizes: Some(format!("{}x{}", size, size)),
        }
    }
}

/// Tool execution configuration for task support
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolExecution {
    /// Task support mode: "required", "optional", "forbidden"
    #[serde(rename = "taskSupport", skip_serializing_if = "Option::is_none")]
    pub task_support: Option<String>,
    /// Whether this tool supports progress reporting
    #[serde(rename = "supportsProgress", skip_serializing_if = "Option::is_none")]
    pub supports_progress: Option<bool>,
    /// Whether this tool can be cancelled
    #[serde(rename = "supportsCancellation", skip_serializing_if = "Option::is_none")]
    pub supports_cancellation: Option<bool>,
    /// Estimated execution time in milliseconds
    #[serde(rename = "estimatedDurationMs", skip_serializing_if = "Option::is_none")]
    pub estimated_duration_ms: Option<u64>,
}

impl ToolExecution {
    pub fn required() -> Self {
        Self {
            task_support: Some("required".into()),
            ..Default::default()
        }
    }

    pub fn optional() -> Self {
        Self {
            task_support: Some("optional".into()),
            ..Default::default()
        }
    }

    pub fn forbidden() -> Self {
        Self {
            task_support: Some("forbidden".into()),
            ..Default::default()
        }
    }

    pub fn with_progress(mut self) -> Self {
        self.supports_progress = Some(true);
        self
    }

    pub fn with_cancellation(mut self) -> Self {
        self.supports_cancellation = Some(true);
        self
    }

    pub fn with_estimated_duration(mut self, ms: u64) -> Self {
        self.estimated_duration_ms = Some(ms);
        self
    }
}

/// Extended tool metadata per MCP 2025-11-25
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolMetadata {
    /// Tool icons for UI display
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icons: Option<Vec<ToolIcon>>,
    /// JSON Schema for structured output validation
    #[serde(rename = "outputSchema", skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,
    /// Execution configuration (task support)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution: Option<ToolExecution>,
}

impl ToolMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_icon(mut self, icon: ToolIcon) -> Self {
        self.icons.get_or_insert_with(Vec::new).push(icon);
        self
    }

    pub fn with_output_schema(mut self, schema: Value) -> Self {
        self.output_schema = Some(schema);
        self
    }

    pub fn with_execution(mut self, execution: ToolExecution) -> Self {
        self.execution = Some(execution);
        self
    }
}

/// Registry of tool metadata
#[derive(Default)]
pub struct ToolMetadataRegistry {
    metadata: HashMap<String, ToolMetadata>,
}

impl ToolMetadataRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, tool_name: impl Into<String>, metadata: ToolMetadata) {
        self.metadata.insert(tool_name.into(), metadata);
    }

    pub fn get(&self, tool_name: &str) -> Option<&ToolMetadata> {
        self.metadata.get(tool_name)
    }

    /// Create default registry with standard tool metadata
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Echo tool
        registry.register("echo", ToolMetadata::new()
            .with_output_schema(json!({
                "type": "object",
                "properties": {
                    "message": {"type": "string"},
                    "timestamp": {"type": "string", "format": "date-time"}
                },
                "required": ["message", "timestamp"]
            }))
            .with_execution(ToolExecution::forbidden())
        );

        // Ping tool
        registry.register("ping", ToolMetadata::new()
            .with_output_schema(json!({
                "type": "object",
                "properties": {
                    "response": {"type": "string"},
                    "timestamp": {"type": "string", "format": "date-time"}
                },
                "required": ["response", "timestamp"]
            }))
            .with_execution(ToolExecution::forbidden())
        );

        // Calculate tool
        registry.register("calculate", ToolMetadata::new()
            .with_output_schema(json!({
                "type": "object",
                "properties": {
                    "operation": {"type": "string"},
                    "a": {"type": "number"},
                    "b": {"type": "number"},
                    "result": {"type": "number"},
                    "timestamp": {"type": "string", "format": "date-time"}
                },
                "required": ["operation", "a", "b", "result", "timestamp"]
            }))
            .with_execution(ToolExecution::forbidden())
        );

        // Long task tool - supports tasks with progress
        registry.register("long_task", ToolMetadata::new()
            .with_output_schema(json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "array",
                        "items": {"type": "object"}
                    }
                },
                "required": ["content"]
            }))
            .with_execution(ToolExecution::optional()
                .with_progress()
                .with_cancellation()
                .with_estimated_duration(10000))
        );

        // Process with progress
        registry.register("process_with_progress", ToolMetadata::new()
            .with_output_schema(json!({
                "type": "object",
                "properties": {
                    "items_processed": {"type": "integer"},
                    "total_time_ms": {"type": "integer"},
                    "timestamp": {"type": "string", "format": "date-time"}
                },
                "required": ["items_processed", "total_time_ms", "timestamp"]
            }))
            .with_execution(ToolExecution::optional()
                .with_progress()
                .with_cancellation())
        );

        // Batch process
        registry.register("batch_process", ToolMetadata::new()
            .with_output_schema(json!({
                "type": "object",
                "properties": {
                    "batches_completed": {"type": "integer"},
                    "items_processed": {"type": "integer"},
                    "status": {"type": "string"},
                    "timestamp": {"type": "string", "format": "date-time"}
                },
                "required": ["batches_completed", "items_processed", "status", "timestamp"]
            }))
            .with_execution(ToolExecution::optional()
                .with_progress()
                .with_cancellation())
        );

        // Health check
        registry.register("health_check", ToolMetadata::new()
            .with_output_schema(json!({
                "type": "object",
                "properties": {
                    "status": {"type": "string"},
                    "version": {"type": "string"},
                    "timestamp": {"type": "string", "format": "date-time"}
                },
                "required": ["status", "version", "timestamp"]
            }))
            .with_execution(ToolExecution::forbidden())
        );

        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_icon() {
        let icon = ToolIcon::svg("data:image/svg+xml,...");
        assert_eq!(icon.mime_type, Some("image/svg+xml".into()));
        assert_eq!(icon.sizes, Some("any".into()));
    }

    #[test]
    fn test_tool_execution() {
        let exec = ToolExecution::optional()
            .with_progress()
            .with_cancellation()
            .with_estimated_duration(5000);
        
        assert_eq!(exec.task_support, Some("optional".into()));
        assert_eq!(exec.supports_progress, Some(true));
        assert_eq!(exec.supports_cancellation, Some(true));
        assert_eq!(exec.estimated_duration_ms, Some(5000));
    }

    #[test]
    fn test_registry() {
        let registry = ToolMetadataRegistry::with_defaults();
        
        assert!(registry.get("echo").is_some());
        assert!(registry.get("long_task").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_serialization() {
        let meta = ToolMetadata::new()
            .with_execution(ToolExecution::optional().with_progress());
        
        let json_str = serde_json::to_string(&meta).unwrap();
        assert!(json_str.contains("taskSupport"));
        assert!(json_str.contains("optional"));
    }
}