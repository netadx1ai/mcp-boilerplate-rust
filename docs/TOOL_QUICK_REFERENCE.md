# Tool Development Quick Reference

Quick reference for creating new MCP tools in Rust.

## CRITICAL RULE: File Size Limit

**Each file MUST be under 500 lines**

- Split files when approaching 400 lines
- Use modules for large tools
- Move complex logic to services
- Extract types to separate files
- See `docs/CODE_ORGANIZATION.md` for details

## Quick Start

1. Create file: `src/tools/my_tool.rs`
2. Register in: `src/tools/mod.rs`
3. Add route in: `src/main.rs`
4. Add handler in: `src/main.rs`
5. Test with curl

## Minimal Tool Template

```rust
use serde_json::json;
use tracing::info;
use crate::types::{McpError, McpResult, ToolRequest, ToolResult};

pub struct MyTool;

impl MyTool {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, request: ToolRequest) -> McpResult<ToolResult> {
        match request.action.as_str() {
            "my_action" => self.handle_my_action(request).await,
            _ => Err(McpError::InvalidAction(request.action)),
        }
    }

    async fn handle_my_action(&self, request: ToolRequest) -> McpResult<ToolResult> {
        info!("Handling my_action");

        Ok(ToolResult {
            success: true,
            data: json!({
                "action": "my_action",
                "result": "success",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        })
    }
}
```

## Register Tool

`src/tools/mod.rs`:
```rust
pub mod my_tool;
pub use my_tool::MyTool;
```

## Add Route

`src/main.rs`:
```rust
use tools::my_tool::MyTool;

// In app builder:
.route("/tools/my_tool", post(handle_my_tool))
```

## Add Handler

`src/main.rs`:
```rust
async fn handle_my_tool(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ToolRequest>,
) -> impl IntoResponse {
    let start_time = std::time::Instant::now();
    let tool = MyTool::new();
    
    match tool.execute(payload).await {
        Ok(result) => {
            (StatusCode::OK, Json(McpResponse {
                success: true,
                data: Some(result.data),
                error: None,
                metadata: Some(json!({
                    "executionTime": start_time.elapsed().as_millis(),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })),
            })).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(McpResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
                metadata: Some(json!({
                    "executionTime": start_time.elapsed().as_millis(),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })),
            })).into_response()
        }
    }
}
```

## Parameter Extraction

### Required String
```rust
let name = request
    .params
    .get("name")
    .and_then(|v| v.as_str())
    .ok_or_else(|| McpError::MissingParameter("name".to_string()))?;
```

### Optional String
```rust
let name = request.params.get("name").and_then(|v| v.as_str());
```

### Required Number
```rust
let count = request
    .params
    .get("count")
    .and_then(|v| v.as_u64())
    .ok_or_else(|| McpError::MissingParameter("count".to_string()))?;
```

### Required Boolean
```rust
let enabled = request
    .params
    .get("enabled")
    .and_then(|v| v.as_bool())
    .ok_or_else(|| McpError::MissingParameter("enabled".to_string()))?;
```

## Error Types

```rust
McpError::InvalidAction("action_name".to_string())
McpError::MissingParameter("param_name".to_string())
McpError::Database("error message".to_string())
McpError::Authentication("error message".to_string())
McpError::Internal("error message".to_string())
McpError::ToolExecution("error message".to_string())
```

## Response Format

```rust
Ok(ToolResult {
    success: true,
    data: json!({
        "action": "action_name",
        "result": value,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }),
})
```

## Multiple Actions

```rust
pub async fn execute(&self, request: ToolRequest) -> McpResult<ToolResult> {
    match request.action.as_str() {
        "create" => self.handle_create(request).await,
        "read" => self.handle_read(request).await,
        "update" => self.handle_update(request).await,
        "delete" => self.handle_delete(request).await,
        _ => Err(McpError::InvalidAction(format!(
            "Unknown action: {}. Available: create, read, update, delete",
            request.action
        ))),
    }
}
```

## Testing

```bash
# Build
cargo build

# Run
cargo run

# Test endpoint
curl -X POST http://localhost:8025/tools/my_tool \
  -H "Content-Type: application/json" \
  -d '{"action":"my_action","param":"value"}'
```

## With Authentication

Route with middleware:
```rust
.route("/tools/protected", post(handle_protected_tool))
    .layer(middleware::from_fn(auth_middleware))
```

Handler with claims:
```rust
async fn handle_protected_tool(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<ToolRequest>,
) -> impl IntoResponse {
    let user_id = claims.user_obj_id;
    // Tool logic
}
```

## Logging

```rust
use tracing::{info, warn, error, debug};

info!("Processing request");
warn!("Invalid input: {}", value);
error!("Operation failed: {}", err);
debug!("Debug info: {:?}", data);
```

## Common Patterns

### List with Pagination
```rust
let limit = request.params.get("limit").and_then(|v| v.as_u64()).unwrap_or(10);
let offset = request.params.get("offset").and_then(|v| v.as_u64()).unwrap_or(0);

Ok(ToolResult {
    success: true,
    data: json!({
        "items": items,
        "count": items.len(),
        "limit": limit,
        "offset": offset
    }),
})
```

### Get by ID
```rust
let id = request
    .params
    .get("id")
    .and_then(|v| v.as_str())
    .ok_or_else(|| McpError::MissingParameter("id".to_string()))?;

Ok(ToolResult {
    success: true,
    data: json!({
        "id": id,
        "data": item
    }),
})
```

### Create Operation
```rust
let name = request.params.get("name").and_then(|v| v.as_str())
    .ok_or_else(|| McpError::MissingParameter("name".to_string()))?;

let new_id = "generated_id";

Ok(ToolResult {
    success: true,
    data: json!({
        "id": new_id,
        "name": name,
        "created": true
    }),
})
```

### Update Operation
```rust
let id = request.params.get("id").and_then(|v| v.as_str())
    .ok_or_else(|| McpError::MissingParameter("id".to_string()))?;

Ok(ToolResult {
    success: true,
    data: json!({
        "id": id,
        "updated": true
    }),
})
```

### Delete Operation
```rust
let id = request.params.get("id").and_then(|v| v.as_str())
    .ok_or_else(|| McpError::MissingParameter("id".to_string()))?;

Ok(ToolResult {
    success: true,
    data: json!({
        "id": id,
        "deleted": true
    }),
})
```

## File Size Control

### When File Exceeds 400 Lines

Create module structure:
```bash
mkdir -p src/tools/my_tool
touch src/tools/my_tool/mod.rs
touch src/tools/my_tool/handlers.rs
touch src/tools/my_tool/types.rs
```

Split by responsibility:
- `mod.rs` - Tool struct and execute() (100-200 lines)
- `handlers.rs` - Action handlers (200-300 lines)
- `types.rs` - Type definitions (100-200 lines)

Or use service layer:
```rust
// Tool stays small (150 lines)
// src/tools/my_tool.rs
use crate::services::my_service::MyService;

// Service has complex logic (300-400 lines)
// src/services/my_service.rs
pub struct MyService;
impl MyService { }
```

## Checklist

- [ ] **Check file size** - Keep under 500 lines
- [ ] Create `src/tools/{tool_name}.rs` or module
- [ ] Register in `src/tools/mod.rs`
- [ ] Add route in `src/main.rs`
- [ ] Add handler in `src/main.rs`
- [ ] Test with curl
- [ ] Add proper error handling
- [ ] Add logging
- [ ] Return timestamp in response
- [ ] **Split if file > 400 lines**

## Commands

```bash
make run          # Run server
make dev          # Run with debug logs
make test-curl    # Test all endpoints
cargo fmt         # Format code
cargo clippy      # Lint code
```

---

Version: 1.0.0