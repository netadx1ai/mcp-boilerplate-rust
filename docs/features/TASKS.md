# Tasks

Long-running task management for MCP servers.

**Version:** 0.6.3  
**MCP Spec:** 2025-11-25

---

## Overview

Tasks enable tools to run long-running operations asynchronously. Instead of blocking until completion, tools can return a task ID that clients use to track progress and retrieve results.

Features:
- Async task execution
- Progress tracking
- Task cancellation
- Result retrieval
- Task lifecycle management

---

## Task Lifecycle

```
┌─────────┐     ┌─────────┐     ┌───────────┐
│ Pending │────>│ Running │────>│ Completed │
└─────────┘     └─────────┘     └───────────┘
                    │                  
                    v                  
              ┌───────────┐            
              │  Failed   │            
              └───────────┘            
                    │                  
                    v                  
              ┌───────────┐            
              │ Cancelled │            
              └───────────┘            
```

---

## Task Status

```rust
use crate::mcp::tasks::TaskStatus;

pub enum TaskStatus {
    Pending,    // Queued, not yet started
    Running,    // Currently executing
    Completed,  // Successfully finished
    Failed,     // Execution failed
    Cancelled,  // Cancelled by user/system
}
```

---

## Task Manager

Manage task lifecycle and state.

### Creating Tasks

```rust
use crate::mcp::tasks::{TaskManager, CreateTaskRequest};

let manager = TaskManager::new();

let request = CreateTaskRequest {
    tool_name: "long_process".to_string(),
    arguments: json!({"file": "large.csv"}),
};

let task = manager.create_task(request).await?;
println!("Task ID: {}", task.id);
```

### Retrieving Tasks

```rust
// Get single task
let task = manager.get_task("task_123").await?;

// List all tasks
let tasks = manager.list_tasks(None).await;

// List with status filter
let running = manager.list_tasks(Some(TaskStatus::Running)).await;
```

### Updating Task Status

```rust
// Start task
manager.start_task("task_123").await?;

// Complete task with result
manager.complete_task("task_123", result_value).await?;

// Fail task with error
manager.fail_task("task_123", "Error message".to_string()).await?;

// Cancel task
manager.cancel_task("task_123").await?;
```

### Progress Updates

```rust
// Update progress (0-100)
manager.update_progress("task_123", 50).await?;

// Update progress with message
manager.update_progress_with_message("task_123", 75, "Processing...").await?;
```

---

## Task Endpoints

MCP protocol endpoints for task management.

### tasks/list

List all tasks with optional filtering.

```json
// Request
{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tasks/list",
    "params": {
        "status": "running"  // optional filter
    }
}

// Response
{
    "jsonrpc": "2.0",
    "id": 1,
    "result": {
        "tasks": [
            {
                "id": "task_123",
                "tool_name": "long_process",
                "status": "running",
                "progress": 50,
                "created_at": "2026-01-09T12:00:00Z"
            }
        ]
    }
}
```

### tasks/get

Get details for a specific task.

```json
// Request
{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tasks/get",
    "params": {
        "task_id": "task_123"
    }
}

// Response
{
    "jsonrpc": "2.0",
    "id": 1,
    "result": {
        "task": {
            "id": "task_123",
            "tool_name": "long_process",
            "status": "running",
            "progress": 75,
            "message": "Processing row 7500 of 10000",
            "created_at": "2026-01-09T12:00:00Z",
            "started_at": "2026-01-09T12:00:01Z"
        }
    }
}
```

### tasks/result

Get the result of a completed task.

```json
// Request
{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tasks/result",
    "params": {
        "task_id": "task_123"
    }
}

// Response
{
    "jsonrpc": "2.0",
    "id": 1,
    "result": {
        "task_id": "task_123",
        "status": "completed",
        "result": {
            "processed_rows": 10000,
            "output_file": "processed.csv"
        }
    }
}
```

### tasks/cancel

Cancel a pending or running task.

```json
// Request
{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tasks/cancel",
    "params": {
        "task_id": "task_123"
    }
}

// Response
{
    "jsonrpc": "2.0",
    "id": 1,
    "result": {
        "task_id": "task_123",
        "status": "cancelled"
    }
}
```

---

## Tool Task Support

Tools declare their task support level.

```rust
use crate::mcp::tasks::TaskSupport;

pub enum TaskSupport {
    Required,   // Tool always returns task ID
    Optional,   // Tool may return task ID for long operations
    Forbidden,  // Tool never uses tasks (synchronous only)
}
```

### Tool Metadata

```rust
use crate::tools::metadata::ToolMetadata;

let metadata = ToolMetadata::new()
    .with_task_support(TaskSupport::Optional)
    .with_supports_progress(true)
    .with_supports_cancellation(true)
    .with_estimated_duration(30000);  // 30 seconds
```

### In tools/list Response

```json
{
    "name": "process_file",
    "description": "Process a large file",
    "inputSchema": {...},
    "outputSchema": {...},
    "_meta": {
        "taskSupport": "optional",
        "supportsProgress": true,
        "supportsCancellation": true,
        "estimatedDurationMs": 30000
    }
}
```

---

## Example: Long-Running Tool

```rust
async fn execute_process_file(
    manager: &TaskManager,
    file_path: String,
) -> Result<String, Error> {
    // Create task
    let task = manager.create_task(CreateTaskRequest {
        tool_name: "process_file".to_string(),
        arguments: json!({"file": file_path}),
    }).await?;
    
    let task_id = task.id.clone();
    
    // Start async processing
    tokio::spawn(async move {
        manager.start_task(&task_id).await.ok();
        
        // Simulate processing
        for i in 0..100 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            manager.update_progress(&task_id, i).await.ok();
        }
        
        // Complete
        let result = json!({"processed": true});
        manager.complete_task(&task_id, result).await.ok();
    });
    
    // Return task ID immediately
    Ok(task.id)
}
```

---

## Task Configuration

```rust
use crate::mcp::tasks::ToolExecutionConfig;

let config = ToolExecutionConfig {
    timeout_ms: Some(60000),      // 1 minute timeout
    max_retries: Some(3),         // Retry up to 3 times
    priority: Some(1),            // High priority
};
```

---

## API Reference

### TaskManager

| Method | Description |
|--------|-------------|
| `new()` | Create new manager |
| `create_task(request)` | Create new task |
| `get_task(id)` | Get task by ID |
| `list_tasks(status)` | List tasks with optional filter |
| `start_task(id)` | Mark task as running |
| `complete_task(id, result)` | Complete with result |
| `fail_task(id, error)` | Fail with error |
| `cancel_task(id)` | Cancel task |
| `update_progress(id, percent)` | Update progress (0-100) |

### Task

| Field | Type | Description |
|-------|------|-------------|
| `id` | String | Unique task ID |
| `tool_name` | String | Tool that created the task |
| `status` | TaskStatus | Current status |
| `progress` | Option<u8> | Progress percentage |
| `message` | Option<String> | Status message |
| `result` | Option<Value> | Completion result |
| `error` | Option<String> | Error message |
| `created_at` | DateTime | Creation timestamp |
| `started_at` | Option<DateTime> | Start timestamp |
| `completed_at` | Option<DateTime> | Completion timestamp |

### CreateTaskRequest

| Field | Type | Description |
|-------|------|-------------|
| `tool_name` | String | Tool name |
| `arguments` | Value | Tool arguments |

### TaskSupport

| Value | Description |
|-------|-------------|
| `Required` | Always returns task ID |
| `Optional` | May return task ID |
| `Forbidden` | Synchronous only |

---

## References

- [MCP Tasks Spec](https://modelcontextprotocol.io/specification/2025-11-25/server/utilities/tasks)
- Source: `src/mcp/tasks.rs`
