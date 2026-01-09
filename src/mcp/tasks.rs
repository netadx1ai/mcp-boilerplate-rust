//! MCP Tasks Module (Experimental)
//!
//! Implements MCP 2025-11-25 Task lifecycle management.
//! Tasks allow long-running operations to be managed asynchronously.
//!
//! Endpoints:
//! - tasks/list - List active tasks
//! - tasks/get - Get task status
//! - tasks/result - Retrieve completed task result
//! - tasks/cancel - Cancel a running task
//!
//! Last Updated: 2026-01-09 13:53 HCMC

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Task status enum per MCP 2025-11-25 spec
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    /// Task is queued but not yet started
    Pending,
    /// Task is currently executing
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed with an error
    Failed,
    /// Task was cancelled by request
    Cancelled,
    /// Task requires additional input (for interactive tasks)
    InputRequired,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Task execution support modes for tool definitions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskSupport {
    /// Task execution is required for this tool
    Required,
    /// Task execution is optional (client choice)
    Optional,
    /// Task execution is not allowed
    Forbidden,
}

impl Default for TaskSupport {
    fn default() -> Self {
        Self::Optional
    }
}

/// Task definition per MCP 2025-11-25
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier
    pub task_id: String,
    /// Current task status
    pub status: TaskStatus,
    /// Optional human-readable status message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
    /// Task creation timestamp (ISO 8601)
    pub created_at: String,
    /// Last update timestamp (ISO 8601)
    pub last_updated_at: String,
    /// Time-to-live in seconds (task expiry)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u64>,
    /// Suggested poll interval in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poll_interval: Option<u64>,
    /// Progress (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<f64>,
    /// Total work units (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<f64>,
}

/// Internal task data with result storage
#[derive(Debug, Clone)]
struct TaskData {
    task: Task,
    /// Tool name that created this task
    tool_name: String,
    /// Original arguments
    arguments: Value,
    /// Task result (when completed)
    result: Option<Value>,
    /// Error message (when failed)
    error: Option<String>,
    /// Creation timestamp (unix epoch)
    created_epoch: u64,
}

/// Request to list tasks
#[derive(Debug, Clone, Deserialize, Default)]
pub struct TasksListRequest {
    /// Filter by status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskStatus>,
    /// Filter by tool name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    /// Pagination cursor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Maximum results to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// Response for tasks/list
#[derive(Debug, Clone, Serialize)]
pub struct TasksListResponse {
    pub tasks: Vec<Task>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Request to get task status
#[derive(Debug, Clone, Deserialize)]
pub struct TasksGetRequest {
    pub task_id: String,
}

/// Response for tasks/get
#[derive(Debug, Clone, Serialize)]
pub struct TasksGetResponse {
    pub task: Task,
}

/// Request to get task result
#[derive(Debug, Clone, Deserialize)]
pub struct TasksResultRequest {
    pub task_id: String,
}

/// Response for tasks/result
#[derive(Debug, Clone, Serialize)]
pub struct TasksResultResponse {
    pub task: Task,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Request to cancel a task
#[derive(Debug, Clone, Deserialize)]
pub struct TasksCancelRequest {
    pub task_id: String,
}

/// Response for tasks/cancel
#[derive(Debug, Clone, Serialize)]
pub struct TasksCancelResponse {
    pub task: Task,
    pub cancelled: bool,
}

/// Task creation request (internal)
#[derive(Debug, Clone)]
pub struct CreateTaskRequest {
    pub tool_name: String,
    pub arguments: Value,
    pub ttl: Option<u64>,
    pub poll_interval: Option<u64>,
}

/// Task Manager handles all task lifecycle operations
#[derive(Clone)]
pub struct TaskManager {
    tasks: Arc<RwLock<HashMap<String, TaskData>>>,
    default_ttl: u64,
    default_poll_interval: u64,
    max_tasks: usize,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: 3600,           // 1 hour
            default_poll_interval: 1000, // 1 second
            max_tasks: 1000,
        }
    }

    pub fn with_config(default_ttl: u64, default_poll_interval: u64, max_tasks: usize) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
            default_poll_interval,
            max_tasks,
        }
    }

    fn now_iso() -> String {
        chrono::Utc::now().to_rfc3339()
    }

    fn now_epoch() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    fn generate_task_id() -> String {
        format!("task_{}", uuid::Uuid::new_v4().to_string().replace("-", ""))
    }

    /// Create a new task
    pub async fn create_task(&self, req: CreateTaskRequest) -> Result<Task, TaskError> {
        let mut tasks = self.tasks.write().await;

        // Cleanup expired tasks first
        self.cleanup_expired_tasks_internal(&mut tasks);

        // Check max tasks limit
        if tasks.len() >= self.max_tasks {
            return Err(TaskError::TooManyTasks(self.max_tasks));
        }

        let task_id = Self::generate_task_id();
        let now = Self::now_iso();
        let now_epoch = Self::now_epoch();

        let task = Task {
            task_id: task_id.clone(),
            status: TaskStatus::Pending,
            status_message: Some("Task created".into()),
            created_at: now.clone(),
            last_updated_at: now,
            ttl: Some(req.ttl.unwrap_or(self.default_ttl)),
            poll_interval: Some(req.poll_interval.unwrap_or(self.default_poll_interval)),
            progress: Some(0.0),
            total: None,
        };

        let task_data = TaskData {
            task: task.clone(),
            tool_name: req.tool_name,
            arguments: req.arguments,
            result: None,
            error: None,
            created_epoch: now_epoch,
        };

        tasks.insert(task_id.clone(), task_data);
        info!("Created task: {}", task_id);

        Ok(task)
    }

    /// Get task by ID
    pub async fn get_task(&self, task_id: &str) -> Result<Task, TaskError> {
        let tasks = self.tasks.read().await;
        tasks
            .get(task_id)
            .map(|d| d.task.clone())
            .ok_or_else(|| TaskError::NotFound(task_id.to_string()))
    }

    /// List tasks with optional filters
    pub async fn list_tasks(&self, req: TasksListRequest) -> TasksListResponse {
        let tasks = self.tasks.read().await;
        let limit = req.limit.unwrap_or(50).min(100);

        let mut result: Vec<Task> = tasks
            .values()
            .filter(|d| {
                // Filter by status
                if let Some(ref status) = req.status {
                    if &d.task.status != status {
                        return false;
                    }
                }
                // Filter by tool name
                if let Some(ref tool_name) = req.tool_name {
                    if &d.tool_name != tool_name {
                        return false;
                    }
                }
                true
            })
            .map(|d| d.task.clone())
            .collect();

        // Sort by created_at descending (newest first)
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Pagination
        let has_more = result.len() > limit;
        result.truncate(limit);

        TasksListResponse {
            tasks: result,
            next_cursor: if has_more {
                Some("not_implemented".into())
            } else {
                None
            },
        }
    }

    /// Get task result (for completed tasks)
    pub async fn get_task_result(&self, task_id: &str) -> Result<TasksResultResponse, TaskError> {
        let tasks = self.tasks.read().await;
        let data = tasks
            .get(task_id)
            .ok_or_else(|| TaskError::NotFound(task_id.to_string()))?;

        Ok(TasksResultResponse {
            task: data.task.clone(),
            result: data.result.clone(),
            error: data.error.clone(),
        })
    }

    /// Cancel a task
    pub async fn cancel_task(&self, task_id: &str) -> Result<TasksCancelResponse, TaskError> {
        let mut tasks = self.tasks.write().await;
        let data = tasks
            .get_mut(task_id)
            .ok_or_else(|| TaskError::NotFound(task_id.to_string()))?;

        let can_cancel = matches!(
            data.task.status,
            TaskStatus::Pending | TaskStatus::Running | TaskStatus::InputRequired
        );

        if can_cancel {
            data.task.status = TaskStatus::Cancelled;
            data.task.status_message = Some("Task cancelled by user".into());
            data.task.last_updated_at = Self::now_iso();
            info!("Cancelled task: {}", task_id);
        } else {
            warn!(
                "Cannot cancel task {} in status {:?}",
                task_id, data.task.status
            );
        }

        Ok(TasksCancelResponse {
            task: data.task.clone(),
            cancelled: can_cancel,
        })
    }

    /// Update task status (for task executors)
    pub async fn update_task_status(
        &self,
        task_id: &str,
        status: TaskStatus,
        message: Option<String>,
        progress: Option<f64>,
    ) -> Result<Task, TaskError> {
        let mut tasks = self.tasks.write().await;
        let data = tasks
            .get_mut(task_id)
            .ok_or_else(|| TaskError::NotFound(task_id.to_string()))?;

        data.task.status = status;
        data.task.status_message = message;
        data.task.last_updated_at = Self::now_iso();
        if let Some(p) = progress {
            data.task.progress = Some(p.clamp(0.0, 1.0));
        }

        Ok(data.task.clone())
    }

    /// Complete task with result
    pub async fn complete_task(&self, task_id: &str, result: Value) -> Result<Task, TaskError> {
        let mut tasks = self.tasks.write().await;
        let data = tasks
            .get_mut(task_id)
            .ok_or_else(|| TaskError::NotFound(task_id.to_string()))?;

        data.task.status = TaskStatus::Completed;
        data.task.status_message = Some("Task completed successfully".into());
        data.task.last_updated_at = Self::now_iso();
        data.task.progress = Some(1.0);
        data.result = Some(result);

        info!("Completed task: {}", task_id);
        Ok(data.task.clone())
    }

    /// Fail task with error
    pub async fn fail_task(&self, task_id: &str, error: String) -> Result<Task, TaskError> {
        let mut tasks = self.tasks.write().await;
        let data = tasks
            .get_mut(task_id)
            .ok_or_else(|| TaskError::NotFound(task_id.to_string()))?;

        data.task.status = TaskStatus::Failed;
        data.task.status_message = Some(format!("Task failed: {}", error));
        data.task.last_updated_at = Self::now_iso();
        data.error = Some(error);

        warn!("Failed task: {}", task_id);
        Ok(data.task.clone())
    }

    /// Start a task (transition from Pending to Running)
    pub async fn start_task(&self, task_id: &str) -> Result<Task, TaskError> {
        self.update_task_status(task_id, TaskStatus::Running, Some("Task started".into()), None)
            .await
    }

    /// Cleanup expired tasks
    pub async fn cleanup_expired_tasks(&self) -> usize {
        let mut tasks = self.tasks.write().await;
        self.cleanup_expired_tasks_internal(&mut tasks)
    }

    fn cleanup_expired_tasks_internal(&self, tasks: &mut HashMap<String, TaskData>) -> usize {
        let now = Self::now_epoch();
        let before = tasks.len();

        tasks.retain(|_, data| {
            if let Some(ttl) = data.task.ttl {
                let expires_at = data.created_epoch + ttl;
                if now > expires_at {
                    return false;
                }
            }
            true
        });

        let removed = before - tasks.len();
        if removed > 0 {
            info!("Cleaned up {} expired tasks", removed);
        }
        removed
    }

    /// Get task capabilities for server initialization
    pub fn get_capabilities() -> Value {
        json!({
            "tasks": {
                "supported": true,
                "listChanged": true,
                "statusNotifications": true
            }
        })
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Task errors
#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("Task not found: {0}")]
    NotFound(String),

    #[error("Task already exists: {0}")]
    AlreadyExists(String),

    #[error("Too many tasks (max: {0})")]
    TooManyTasks(usize),

    #[error("Invalid task state transition")]
    InvalidStateTransition,

    #[error("Task execution error: {0}")]
    ExecutionError(String),
}

impl TaskError {
    pub fn to_json_rpc_error(&self) -> Value {
        let (code, message) = match self {
            TaskError::NotFound(_) => (-32001, self.to_string()),
            TaskError::AlreadyExists(_) => (-32002, self.to_string()),
            TaskError::TooManyTasks(_) => (-32003, self.to_string()),
            TaskError::InvalidStateTransition => (-32004, self.to_string()),
            TaskError::ExecutionError(_) => (-32005, self.to_string()),
        };

        json!({
            "code": code,
            "message": message
        })
    }
}

/// Tool execution configuration for task support
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolExecutionConfig {
    /// Task support mode
    #[serde(default)]
    pub task_support: TaskSupport,
    /// Estimated execution time in milliseconds (for UI hints)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_duration_ms: Option<u64>,
    /// Whether this tool supports progress reporting
    #[serde(default)]
    pub supports_progress: bool,
    /// Whether this tool can be cancelled
    #[serde(default)]
    pub supports_cancellation: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_task() {
        let manager = TaskManager::new();
        let task = manager
            .create_task(CreateTaskRequest {
                tool_name: "test_tool".into(),
                arguments: json!({"key": "value"}),
                ttl: None,
                poll_interval: None,
            })
            .await
            .unwrap();

        assert!(task.task_id.starts_with("task_"));
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[tokio::test]
    async fn test_task_lifecycle() {
        let manager = TaskManager::new();

        // Create
        let task = manager
            .create_task(CreateTaskRequest {
                tool_name: "long_task".into(),
                arguments: json!({}),
                ttl: Some(60),
                poll_interval: Some(500),
            })
            .await
            .unwrap();

        let task_id = task.task_id.clone();

        // Start
        let task = manager.start_task(&task_id).await.unwrap();
        assert_eq!(task.status, TaskStatus::Running);

        // Update progress
        let task = manager
            .update_task_status(&task_id, TaskStatus::Running, Some("50%".into()), Some(0.5))
            .await
            .unwrap();
        assert_eq!(task.progress, Some(0.5));

        // Complete
        let task = manager
            .complete_task(&task_id, json!({"result": "done"}))
            .await
            .unwrap();
        assert_eq!(task.status, TaskStatus::Completed);

        // Get result
        let result = manager.get_task_result(&task_id).await.unwrap();
        assert!(result.result.is_some());
    }

    #[tokio::test]
    async fn test_cancel_task() {
        let manager = TaskManager::new();

        let task = manager
            .create_task(CreateTaskRequest {
                tool_name: "test".into(),
                arguments: json!({}),
                ttl: None,
                poll_interval: None,
            })
            .await
            .unwrap();

        let response = manager.cancel_task(&task.task_id).await.unwrap();
        assert!(response.cancelled);
        assert_eq!(response.task.status, TaskStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_list_tasks() {
        let manager = TaskManager::new();

        // Create multiple tasks
        for i in 0..5 {
            manager
                .create_task(CreateTaskRequest {
                    tool_name: format!("tool_{}", i),
                    arguments: json!({}),
                    ttl: None,
                    poll_interval: None,
                })
                .await
                .unwrap();
        }

        let list = manager.list_tasks(TasksListRequest::default()).await;
        assert_eq!(list.tasks.len(), 5);
    }

    #[test]
    fn test_task_status_serialization() {
        let status = TaskStatus::Running;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"running\"");

        let deserialized: TaskStatus = serde_json::from_str("\"completed\"").unwrap();
        assert_eq!(deserialized, TaskStatus::Completed);
    }
}