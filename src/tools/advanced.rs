use chrono::Utc;
use rmcp::{
    handler::server::wrapper::{Json, Parameters},
    model::*,
    service::RequestContext,
    ErrorData as McpError, RoleServer,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::info;

// Progress Example Types
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ProcessDataRequest {
    /// Number of items to process
    pub items: u32,
    /// Delay in milliseconds per item
    #[serde(default = "default_delay")]
    pub delay_ms: u64,
}

fn default_delay() -> u64 {
    100
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ProcessDataResponse {
    pub items_processed: u32,
    pub total_time_ms: u64,
    pub timestamp: String,
}

// Batch Processing Types
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct BatchRequest {
    pub batch_size: u32,
    pub total_batches: u32,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct BatchResponse {
    pub batches_completed: u32,
    pub items_processed: u32,
    pub status: String,
    pub timestamp: String,
}

// Data Transform Types
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct TransformRequest {
    pub data: Vec<serde_json::Value>,
    pub operation: String, // "uppercase", "lowercase", "reverse", "sort"
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct TransformResponse {
    pub original_count: usize,
    pub transformed_count: usize,
    pub operation: String,
    pub result: Vec<serde_json::Value>,
    pub timestamp: String,
}

// Advanced tool implementations
pub struct AdvancedTool;

impl AdvancedTool {
    pub fn new() -> Self {
        Self
    }

    /// Process data with progress notifications
    pub async fn process_with_progress(
        params: Parameters<ProcessDataRequest>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<Json<ProcessDataResponse>, McpError> {
        let req = params.0;

        if req.items == 0 {
            return Err(McpError::invalid_params(
                "Items must be greater than 0",
                None,
            ));
        }

        if req.items > 1000 {
            return Err(McpError::invalid_params(
                "Items cannot exceed 1000 (too many items to process)",
                None,
            ));
        }

        info!(
            "Processing {} items with {}ms delay",
            req.items, req.delay_ms
        );

        let peer = ctx.peer.clone();
        let start_time = std::time::Instant::now();

        // Send progress notifications
        for i in 0..req.items {
            tokio::time::sleep(tokio::time::Duration::from_millis(req.delay_ms)).await;

            // Notify progress every 10 items or on last item
            if i % 10 == 0 || i == req.items - 1 {
                let _ = peer
                    .notify_progress(ProgressNotificationParam {
                        progress_token: ProgressToken(rmcp::model::NumberOrString::String(
                            "process_data".into(),
                        )),
                        progress: (i + 1) as f64,
                        total: Some(req.items as f64),
                        message: None,
                    })
                    .await;

                info!("Progress: {}/{} items", i + 1, req.items);
            }
        }

        let elapsed = start_time.elapsed().as_millis() as u64;

        Ok(Json(ProcessDataResponse {
            items_processed: req.items,
            total_time_ms: elapsed,
            timestamp: Utc::now().to_rfc3339(),
        }))
    }

    /// Batch processing with status updates
    pub async fn batch_process(
        params: Parameters<BatchRequest>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<Json<BatchResponse>, McpError> {
        let req = params.0;

        if req.batch_size == 0 || req.total_batches == 0 {
            return Err(McpError::invalid_params(
                "Batch size and total batches must be greater than 0",
                None,
            ));
        }

        info!(
            "Processing {} batches of {} items",
            req.total_batches, req.batch_size
        );

        let peer = ctx.peer.clone();

        for batch in 0..req.total_batches {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

            let _ = peer
                .notify_progress(ProgressNotificationParam {
                    progress_token: ProgressToken(rmcp::model::NumberOrString::String(
                        "batch_process".into(),
                    )),
                    progress: (batch + 1) as f64,
                    total: Some(req.total_batches as f64),
                    message: None,
                })
                .await;

            let _ = peer
                .notify_logging_message(LoggingMessageNotificationParam {
                    level: LoggingLevel::Info,
                    logger: Some("batch_processor".into()),
                    data: serde_json::json!({
                        "batch": batch + 1,
                        "items": req.batch_size,
                        "status": "processing"
                    }),
                })
                .await;

            info!("Batch {}/{} completed", batch + 1, req.total_batches);
        }

        Ok(Json(BatchResponse {
            batches_completed: req.total_batches,
            items_processed: req.batch_size * req.total_batches,
            status: "completed".to_string(),
            timestamp: Utc::now().to_rfc3339(),
        }))
    }

    /// Transform data with validation
    pub async fn transform_data(
        params: Parameters<TransformRequest>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<Json<TransformResponse>, McpError> {
        let req = params.0;

        if req.data.is_empty() {
            return Err(McpError::invalid_params("Data array cannot be empty", None));
        }

        if req.data.len() > 10000 {
            return Err(McpError::invalid_params(
                format!("Data array too large: {} items (max 10000)", req.data.len()),
                None,
            ));
        }

        info!(
            "Transforming {} items with operation: {}",
            req.data.len(),
            req.operation
        );

        let peer = ctx.peer.clone();
        let original_count = req.data.len();

        let _ = peer
            .notify_logging_message(LoggingMessageNotificationParam {
                level: LoggingLevel::Debug,
                logger: Some("transform".into()),
                data: serde_json::json!({
                    "operation": req.operation,
                    "count": original_count
                }),
            })
            .await;

        let mut result = Vec::new();

        for (idx, item) in req.data.iter().enumerate() {
            if idx % 100 == 0 && idx > 0 {
                let _ = peer
                    .notify_progress(ProgressNotificationParam {
                        progress_token: ProgressToken(rmcp::model::NumberOrString::String(
                            "transform".into(),
                        )),
                        progress: idx as f64,
                        total: Some(original_count as f64),
                        message: None,
                    })
                    .await;
            }

            let transformed = match req.operation.as_str() {
                "uppercase" => {
                    if let Some(s) = item.as_str() {
                        serde_json::Value::String(s.to_uppercase())
                    } else {
                        item.clone()
                    }
                }
                "lowercase" => {
                    if let Some(s) = item.as_str() {
                        serde_json::Value::String(s.to_lowercase())
                    } else {
                        item.clone()
                    }
                }
                "reverse" => {
                    if let Some(s) = item.as_str() {
                        serde_json::Value::String(s.chars().rev().collect())
                    } else {
                        item.clone()
                    }
                }
                "double" => {
                    if let Some(n) = item.as_f64() {
                        serde_json::Value::Number(
                            serde_json::Number::from_f64(n * 2.0)
                                .unwrap_or(serde_json::Number::from(0)),
                        )
                    } else {
                        item.clone()
                    }
                }
                _ => {
                    return Err(McpError::invalid_params(
                        format!(
                            "Unknown operation: '{}'. Supported: uppercase, lowercase, reverse, double",
                            req.operation
                        ),
                        None,
                    ));
                }
            };

            result.push(transformed);
        }

        Ok(Json(TransformResponse {
            original_count,
            transformed_count: result.len(),
            operation: req.operation,
            result,
            timestamp: Utc::now().to_rfc3339(),
        }))
    }

    /// Simulate file upload with progress
    pub async fn simulate_upload(
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        info!("Simulating file upload");

        let peer = ctx.peer.clone();
        let total_chunks = 20;

        for chunk in 0..total_chunks {
            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

            let progress = ((chunk + 1) as f64 / total_chunks as f64) * 100.0;

            let _ = peer
                .notify_progress(ProgressNotificationParam {
                    progress_token: ProgressToken(rmcp::model::NumberOrString::String(
                        "upload".into(),
                    )),
                    progress: (chunk + 1) as f64,
                    total: Some(total_chunks as f64),
                    message: None,
                })
                .await;

            let _ = peer
                .notify_logging_message(LoggingMessageNotificationParam {
                    level: LoggingLevel::Info,
                    logger: Some("uploader".into()),
                    data: serde_json::json!({
                        "chunk": chunk + 1,
                        "total": total_chunks,
                        "progress_percent": progress as u32
                    }),
                })
                .await;

            info!("Upload progress: {:.1}%", progress);
        }

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::json!({
                "status": "uploaded",
                "chunks": total_chunks,
                "timestamp": Utc::now().to_rfc3339()
            })
            .to_string(),
        )]))
    }

    /// Health check with system info
    pub async fn health_check(ctx: RequestContext<RoleServer>) -> Result<CallToolResult, McpError> {
        info!("Health check requested");

        let peer = ctx.peer.clone();

        let _ = peer
            .notify_logging_message(LoggingMessageNotificationParam {
                level: LoggingLevel::Debug,
                logger: Some("health".into()),
                data: serde_json::json!({"check": "starting"}),
            })
            .await;

        let health_data = serde_json::json!({
            "status": "healthy",
            "uptime_seconds": 0,
            "memory_mb": 0,
            "timestamp": Utc::now().to_rfc3339(),
            "version": env!("CARGO_PKG_VERSION")
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&health_data).unwrap(),
        )]))
    }
}

impl Default for AdvancedTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_data_request_validation() {
        let valid = ProcessDataRequest {
            items: 10,
            delay_ms: 100,
        };
        assert_eq!(valid.items, 10);
        assert_eq!(valid.delay_ms, 100);
    }

    #[test]
    fn test_transform_operations() {
        let data = [serde_json::Value::String("hello".to_string()),
            serde_json::Value::String("world".to_string())];

        assert_eq!(data.len(), 2);
    }
}
