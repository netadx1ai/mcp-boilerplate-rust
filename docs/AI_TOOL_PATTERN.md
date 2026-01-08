# AI Tool Development Pattern - MCP Boilerplate Rust

Complete guide for AI assistants to create new MCP tools in this Rust boilerplate.

## Context

Project: MCP Boilerplate Rust
Language: Rust
Framework: Axum + Tokio
Protocol: MCP v5
Architecture: HTTP REST API with tool pattern

## Project Structure

```
src/
├── main.rs              # Server entry point, routes, handlers
├── types.rs             # Core types, errors, request/response
├── utils/
│   ├── mod.rs          # Utils module exports
│   ├── config.rs       # Configuration helper
│   └── logger.rs       # Logging utility
├── middleware/
│   ├── mod.rs          # Middleware exports
│   └── auth.rs         # JWT authentication
├── services/
│   └── mod.rs          # Business logic services
├── models/
│   └── mod.rs          # Data models
└── tools/
    ├── mod.rs          # Tools module exports
    └── echo.rs         # Example tool
```

## Tool Development Pattern

### Step 1: Create Tool File

File: `src/tools/{tool_name}.rs`

```rust
use async_trait::async_trait;
use serde_json::{json, Value};
use tracing::{info, warn, error};

use crate::types::{McpError, McpResult, ToolRequest, ToolResult};

pub struct {ToolName}Tool;

impl {ToolName}Tool {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, request: ToolRequest) -> McpResult<ToolResult> {
        let action = request.action.as_str();

        match action {
            "action1" => self.handle_action1(request).await,
            "action2" => self.handle_action2(request).await,
            _ => Err(McpError::InvalidAction(format!(
                "Unknown action: {}. Available actions: action1, action2",
                action
            ))),
        }
    }

    async fn handle_action1(&self, request: ToolRequest) -> McpResult<ToolResult> {
        info!("Handling action1");

        // Extract parameters
        let param = request
            .params
            .get("param_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::MissingParameter("param_name".to_string()))?;

        // Business logic here

        Ok(ToolResult {
            success: true,
            data: json!({
                "action": "action1",
                "result": "success",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        })
    }

    async fn handle_action2(&self, request: ToolRequest) -> McpResult<ToolResult> {
        info!("Handling action2");

        // Business logic here

        Ok(ToolResult {
            success: true,
            data: json!({
                "action": "action2",
                "result": "success",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        })
    }
}
```

### Step 2: Register Tool in Module

File: `src/tools/mod.rs`

```rust
pub mod echo;
pub mod {tool_name};  // Add this line

pub use echo::EchoTool;
pub use {tool_name}::{ToolName}Tool;  // Add this line
```

### Step 3: Add Route in Main

File: `src/main.rs`

Add to imports:
```rust
use tools::{tool_name}::{ToolName}Tool;
```

Add route:
```rust
let app = Router::new()
    .route("/", get(health_check))
    .route("/health", get(health_check))
    .route("/tools/echo", post(handle_echo_tool))
    .route("/tools/{tool_name}", post(handle_{tool_name}_tool))  // Add this
    .layer(cors)
    .with_state(state);
```

### Step 4: Add Handler Function

File: `src/main.rs`

```rust
async fn handle_{tool_name}_tool(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ToolRequest>,
) -> impl IntoResponse {
    let start_time = std::time::Instant::now();

    info!("{ToolName} tool called with action: {:?}", payload.action);

    let tool = {ToolName}Tool::new();
    
    match tool.execute(payload).await {
        Ok(result) => {
            let execution_time = start_time.elapsed().as_millis();
            
            let response = McpResponse {
                success: true,
                data: Some(result.data),
                error: None,
                metadata: Some(json!({
                    "executionTime": execution_time,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })),
            };

            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("{ToolName} tool error: {}", e);
            let execution_time = start_time.elapsed().as_millis();
            
            let response = McpResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
                metadata: Some(json!({
                    "executionTime": execution_time,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })),
            };

            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}
```

## Parameter Extraction Patterns

### Required String Parameter
```rust
let param = request
    .params
    .get("param_name")
    .and_then(|v| v.as_str())
    .ok_or_else(|| McpError::MissingParameter("param_name".to_string()))?;
```

### Optional String Parameter
```rust
let param = request
    .params
    .get("param_name")
    .and_then(|v| v.as_str())
    .map(|s| s.to_string());
```

### Required Number Parameter
```rust
let count = request
    .params
    .get("count")
    .and_then(|v| v.as_u64())
    .ok_or_else(|| McpError::MissingParameter("count".to_string()))?;
```

### Required Boolean Parameter
```rust
let enabled = request
    .params
    .get("enabled")
    .and_then(|v| v.as_bool())
    .ok_or_else(|| McpError::MissingParameter("enabled".to_string()))?;
```

### Required Object Parameter
```rust
let config = request
    .params
    .get("config")
    .and_then(|v| v.as_object())
    .ok_or_else(|| McpError::MissingParameter("config".to_string()))?;
```

### Required Array Parameter
```rust
let items = request
    .params
    .get("items")
    .and_then(|v| v.as_array())
    .ok_or_else(|| McpError::MissingParameter("items".to_string()))?;
```

## Error Handling Patterns

### Invalid Action
```rust
Err(McpError::InvalidAction(format!(
    "Unknown action: {}. Available actions: action1, action2",
    action
)))
```

### Missing Parameter
```rust
Err(McpError::MissingParameter("param_name".to_string()))
```

### Database Error
```rust
Err(McpError::Database(format!("Failed to query: {}", err)))
```

### Authentication Error
```rust
Err(McpError::Authentication("Invalid credentials".to_string()))
```

### Internal Error
```rust
Err(McpError::Internal(format!("Unexpected error: {}", err)))
```

### Tool Execution Error
```rust
Err(McpError::ToolExecution(format!("Operation failed: {}", err)))
```

## Response Patterns

### Simple Success
```rust
Ok(ToolResult {
    success: true,
    data: json!({
        "action": "action_name",
        "result": "success"
    }),
})
```

### Success with Data
```rust
Ok(ToolResult {
    success: true,
    data: json!({
        "action": "get_user",
        "user": {
            "id": user_id,
            "name": user_name,
            "email": email
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }),
})
```

### Success with List
```rust
Ok(ToolResult {
    success: true,
    data: json!({
        "action": "list_items",
        "items": items,
        "count": items.len(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }),
})
```

## Authentication Patterns

### Optional Authentication
```rust
async fn handle_tool(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ToolRequest>,
) -> impl IntoResponse {
    // Extract optional user claims
    let user_claims = headers
        .get("x-access-token")
        .and_then(|token| {
            // Verify token logic
            None
        });
    
    // Tool logic here
}
```

### Required Authentication
Use middleware in route:
```rust
.route("/tools/protected", post(handle_protected_tool))
    .layer(middleware::from_fn(auth_middleware))
```

Then extract claims in handler:
```rust
async fn handle_protected_tool(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<ToolRequest>,
) -> impl IntoResponse {
    let user_id = claims.user_obj_id;
    // Tool logic with authenticated user
}
```

## Database Integration Patterns

### MongoDB Example
```rust
use mongodb::{Client, Collection, bson::doc};

// In tool struct
pub struct DatabaseTool {
    collection: Collection<Document>,
}

impl DatabaseTool {
    pub fn new(mongo_uri: &str, db_name: &str, collection_name: &str) -> Self {
        let client = Client::with_uri_str(mongo_uri).await?;
        let db = client.database(db_name);
        let collection = db.collection(collection_name);
        
        Self { collection }
    }

    async fn query_data(&self, filter: Document) -> McpResult<Vec<Document>> {
        let cursor = self.collection
            .find(filter, None)
            .await
            .map_err(|e| McpError::Database(e.to_string()))?;
        
        // Process cursor
        Ok(results)
    }
}
```

## Service Integration Pattern

Create service in `src/services/{service_name}.rs`:

```rust
pub struct {ServiceName}Service;

impl {ServiceName}Service {
    pub fn new() -> Self {
        Self
    }

    pub async fn perform_operation(&self, params: Params) -> Result<Output, Error> {
        // Service logic
        Ok(output)
    }
}
```

Use in tool:
```rust
use crate::services::{ServiceName}Service;

impl {ToolName}Tool {
    async fn handle_action(&self, request: ToolRequest) -> McpResult<ToolResult> {
        let service = {ServiceName}Service::new();
        
        let result = service.perform_operation(params)
            .await
            .map_err(|e| McpError::Internal(e.to_string()))?;
        
        Ok(ToolResult {
            success: true,
            data: json!({ "result": result }),
        })
    }
}
```

## Testing Pattern

Create test in tool file:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_action1() {
        let tool = {ToolName}Tool::new();
        
        let request = ToolRequest {
            action: "action1".to_string(),
            params: json!({
                "param": "value"
            }).as_object().unwrap().clone(),
        };
        
        let result = tool.execute(request).await;
        assert!(result.is_ok());
        
        let result = result.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_invalid_action() {
        let tool = {ToolName}Tool::new();
        
        let request = ToolRequest {
            action: "invalid".to_string(),
            params: Default::default(),
        };
        
        let result = tool.execute(request).await;
        assert!(result.is_err());
    }
}
```

## Complete Example: User Management Tool

File: `src/tools/user_management.rs`

```rust
use async_trait::async_trait;
use serde_json::{json, Value};
use tracing::{info, warn, error};

use crate::types::{McpError, McpResult, ToolRequest, ToolResult};

pub struct UserManagementTool;

impl UserManagementTool {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, request: ToolRequest) -> McpResult<ToolResult> {
        let action = request.action.as_str();

        match action {
            "get_user" => self.handle_get_user(request).await,
            "list_users" => self.handle_list_users(request).await,
            "create_user" => self.handle_create_user(request).await,
            "update_user" => self.handle_update_user(request).await,
            "delete_user" => self.handle_delete_user(request).await,
            _ => Err(McpError::InvalidAction(format!(
                "Unknown action: {}. Available: get_user, list_users, create_user, update_user, delete_user",
                action
            ))),
        }
    }

    async fn handle_get_user(&self, request: ToolRequest) -> McpResult<ToolResult> {
        info!("Getting user");

        let user_id = request
            .params
            .get("userId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::MissingParameter("userId".to_string()))?;

        // Database query logic here
        // let user = db.get_user(user_id).await?;

        Ok(ToolResult {
            success: true,
            data: json!({
                "action": "get_user",
                "user": {
                    "id": user_id,
                    "name": "John Doe",
                    "email": "john@example.com"
                },
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        })
    }

    async fn handle_list_users(&self, request: ToolRequest) -> McpResult<ToolResult> {
        info!("Listing users");

        let limit = request
            .params
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        let offset = request
            .params
            .get("offset")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        // Database query logic here
        let users = vec![]; // Replace with actual query

        Ok(ToolResult {
            success: true,
            data: json!({
                "action": "list_users",
                "users": users,
                "count": users.len(),
                "limit": limit,
                "offset": offset,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        })
    }

    async fn handle_create_user(&self, request: ToolRequest) -> McpResult<ToolResult> {
        info!("Creating user");

        let name = request
            .params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::MissingParameter("name".to_string()))?;

        let email = request
            .params
            .get("email")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::MissingParameter("email".to_string()))?;

        // Create user logic here

        Ok(ToolResult {
            success: true,
            data: json!({
                "action": "create_user",
                "userId": "new_user_id",
                "name": name,
                "email": email,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        })
    }

    async fn handle_update_user(&self, request: ToolRequest) -> McpResult<ToolResult> {
        info!("Updating user");

        let user_id = request
            .params
            .get("userId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::MissingParameter("userId".to_string()))?;

        // Optional fields
        let name = request.params.get("name").and_then(|v| v.as_str());
        let email = request.params.get("email").and_then(|v| v.as_str());

        // Update logic here

        Ok(ToolResult {
            success: true,
            data: json!({
                "action": "update_user",
                "userId": user_id,
                "updated": true,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        })
    }

    async fn handle_delete_user(&self, request: ToolRequest) -> McpResult<ToolResult> {
        info!("Deleting user");

        let user_id = request
            .params
            .get("userId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::MissingParameter("userId".to_string()))?;

        // Delete logic here

        Ok(ToolResult {
            success: true,
            data: json!({
                "action": "delete_user",
                "userId": user_id,
                "deleted": true,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        })
    }
}
```

## Checklist for New Tool

- [ ] Create tool file: `src/tools/{tool_name}.rs`
- [ ] Implement tool struct with `new()` and `execute()`
- [ ] Add action handlers with proper error handling
- [ ] Register in `src/tools/mod.rs`
- [ ] Add route in `src/main.rs`
- [ ] Add handler function in `src/main.rs`
- [ ] Add logging statements
- [ ] Return proper McpResponse format
- [ ] Test with curl commands
- [ ] Add tests if needed
- [ ] Update documentation

## Quick Command Reference

Build and run:
```bash
cargo run
```

Test endpoint:
```bash
curl -X POST http://localhost:8025/tools/{tool_name} \
  -H "Content-Type: application/json" \
  -d '{"action":"action_name","param":"value"}'
```

With authentication:
```bash
curl -X POST http://localhost:8025/tools/{tool_name} \
  -H "Content-Type: application/json" \
  -H "x-access-token: YOUR_TOKEN" \
  -d '{"action":"action_name","param":"value"}'
```

## File Size Control - CRITICAL

**RULE: Each file MUST be under 500 lines**

### When to Split Files

Split a file when it approaches 400 lines or when:
- Multiple distinct responsibilities exist
- Code becomes hard to navigate
- Testing becomes difficult

### Splitting Strategies

#### 1. Action-Based Split (for tools with many actions)

Instead of one large file:
```
src/tools/user_management.rs  # 800 lines - TOO LARGE
```

Split into module:
```
src/tools/user_management/
├── mod.rs           # 100 lines - main tool struct & execute()
├── create.rs        # 150 lines - create actions
├── read.rs          # 150 lines - get/list actions
├── update.rs        # 120 lines - update actions
├── delete.rs        # 100 lines - delete actions
└── types.rs         # 80 lines - shared types
```

**mod.rs example:**
```rust
mod create;
mod read;
mod update;
mod delete;
mod types;

pub use types::*;
use create::CreateHandler;
use read::ReadHandler;

pub struct UserManagementTool {
    create: CreateHandler,
    read: ReadHandler,
}

impl UserManagementTool {
    pub async fn execute(&self, request: ToolRequest) -> McpResult<ToolResult> {
        match request.action.as_str() {
            "create" => self.create.handle(request).await,
            "get" => self.read.get(request).await,
            _ => Err(McpError::InvalidAction(request.action)),
        }
    }
}
```

#### 2. Service Layer Split (for complex logic)

Move business logic to services:

**Tool (150 lines):**
```rust
// src/tools/payment.rs
use crate::services::payment_service::PaymentService;

pub struct PaymentTool {
    service: PaymentService,
}

impl PaymentTool {
    async fn handle_process(&self, request: ToolRequest) -> McpResult<ToolResult> {
        let result = self.service.process_payment(params)
            .await
            .map_err(|e| McpError::Internal(e.to_string()))?;
        
        Ok(ToolResult {
            success: true,
            data: json!({ "result": result }),
        })
    }
}
```

**Service (350 lines):**
```rust
// src/services/payment_service.rs
pub struct PaymentService;

impl PaymentService {
    pub async fn process_payment(&self, params: Params) -> Result<Data, Error> {
        // Complex business logic here (300+ lines)
        Ok(data)
    }
}
```

#### 3. Extract Types

Move type definitions to separate files:

```rust
// src/tools/analytics/mod.rs - 150 lines
mod types;
mod handlers;

pub use types::*;

// src/tools/analytics/types.rs - 100 lines
pub struct MetricsData { }
pub struct ReportData { }

// src/tools/analytics/handlers.rs - 250 lines
pub struct AnalyticsHandler;
```

### File Size Targets

| File Type | Max Lines | Target |
|-----------|-----------|--------|
| Tool main file | 500 | 200-300 |
| Handler file | 500 | 200-350 |
| Service file | 500 | 300-450 |
| Types file | 500 | 100-200 |

### Refactoring Steps

1. Check file size: `wc -l src/tools/my_tool.rs`
2. If > 400 lines, create directory: `mkdir src/tools/my_tool`
3. Create `mod.rs` and split by responsibility
4. Move related functions to separate files
5. Update imports
6. Run `cargo check` and test

**See `docs/CODE_ORGANIZATION.md` for complete guide**

## Notes for AI

1. **CRITICAL: Keep each file under 500 lines** - split when approaching 400 lines
2. Always use `McpError` types for errors
3. Always return `McpResult<ToolResult>`
4. Always add timestamp to responses
5. Always log actions with `info!`, `error!`, etc.
6. Always validate required parameters
7. Always handle errors gracefully
8. Follow naming conventions: snake_case for actions, PascalCase for structs
9. Keep it simple and clean
10. No emojis in code or logs
11. Split large files into modules immediately
12. Use services layer for complex business logic

---

Version: 1.0.0
Last Updated: 2026-01-08
Author: NetADX MCP Team
