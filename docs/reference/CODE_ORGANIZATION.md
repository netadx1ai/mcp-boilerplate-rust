# Code Organization Guidelines

Best practices for keeping code files under 500 lines in MCP Boilerplate Rust.

## Rule: Maximum 500 Lines Per File

Each Rust source file should be **under 500 lines** to maintain readability and modularity.

## File Organization Strategies

### 1. Split by Feature/Action

Instead of one large tool file, split into multiple modules.

**Bad** (800+ lines):
```
src/tools/user_management.rs  # 800 lines
```

**Good** (multiple files):
```
src/tools/user_management/
├── mod.rs           # 50 lines - exports & tool struct
├── get.rs           # 100 lines - get operations
├── create.rs        # 120 lines - create operations
├── update.rs        # 100 lines - update operations
├── delete.rs        # 80 lines - delete operations
└── types.rs         # 80 lines - shared types
```

### 2. Separate Business Logic into Services

Move complex logic to services layer.

**Tool File** (~150 lines):
```rust
// src/tools/payment.rs
use crate::services::payment_service::PaymentService;

pub struct PaymentTool {
    service: PaymentService,
}

impl PaymentTool {
    pub async fn execute(&self, request: ToolRequest) -> McpResult<ToolResult> {
        match request.action.as_str() {
            "process" => self.handle_process(request).await,
            _ => Err(McpError::InvalidAction(request.action)),
        }
    }

    async fn handle_process(&self, request: ToolRequest) -> McpResult<ToolResult> {
        // Simple validation only
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

**Service File** (~300 lines):
```rust
// src/services/payment_service.rs
pub struct PaymentService;

impl PaymentService {
    pub async fn process_payment(&self, params: Params) -> Result<Data, Error> {
        // Complex business logic here
        Ok(data)
    }
}
```

### 3. Extract Types and Models

Move type definitions to separate files.

**Before** (600 lines in one file):
```rust
// src/tools/analytics.rs - 600 lines
pub struct AnalyticsTool { }
pub struct MetricsData { }
pub struct ReportData { }
// ... all code together
```

**After** (split into modules):
```rust
// src/tools/analytics/mod.rs - 150 lines
mod types;
mod metrics;
mod reports;

pub use types::*;
use metrics::MetricsHandler;
use reports::ReportsHandler;

pub struct AnalyticsTool {
    metrics: MetricsHandler,
    reports: ReportsHandler,
}

// src/tools/analytics/types.rs - 100 lines
pub struct MetricsData { }
pub struct ReportData { }

// src/tools/analytics/metrics.rs - 200 lines
pub struct MetricsHandler;
impl MetricsHandler { }

// src/tools/analytics/reports.rs - 150 lines
pub struct ReportsHandler;
impl ReportsHandler { }
```

### 4. Use Trait-Based Organization

Split implementations using traits.

```rust
// src/tools/database/mod.rs - 80 lines
mod query;
mod mutation;
mod types;

pub use query::QueryHandler;
pub use mutation::MutationHandler;
pub use types::*;

pub struct DatabaseTool {
    query: QueryHandler,
    mutation: MutationHandler,
}

// src/tools/database/query.rs - 250 lines
pub struct QueryHandler;
impl QueryHandler {
    pub async fn find_one(&self) -> Result<T> { }
    pub async fn find_many(&self) -> Result<Vec<T>> { }
}

// src/tools/database/mutation.rs - 300 lines
pub struct MutationHandler;
impl MutationHandler {
    pub async fn insert(&self) -> Result<T> { }
    pub async fn update(&self) -> Result<T> { }
    pub async fn delete(&self) -> Result<T> { }
}
```

## Module Organization Patterns

### Pattern 1: Action-Based Modules

For CRUD tools with many actions:

```
src/tools/resource_name/
├── mod.rs          # Main tool struct and execute()
├── create.rs       # Create actions
├── read.rs         # Read/Get/List actions
├── update.rs       # Update actions
├── delete.rs       # Delete actions
└── types.rs        # Shared types
```

**mod.rs** (~100 lines):
```rust
mod create;
mod read;
mod update;
mod delete;
mod types;

pub use types::*;
use create::CreateHandler;
use read::ReadHandler;
use update::UpdateHandler;
use delete::DeleteHandler;

pub struct ResourceTool {
    create: CreateHandler,
    read: ReadHandler,
    update: UpdateHandler,
    delete: DeleteHandler,
}

impl ResourceTool {
    pub async fn execute(&self, request: ToolRequest) -> McpResult<ToolResult> {
        match request.action.as_str() {
            "create" => self.create.handle(request).await,
            "get" => self.read.get(request).await,
            "list" => self.read.list(request).await,
            "update" => self.update.handle(request).await,
            "delete" => self.delete.handle(request).await,
            _ => Err(McpError::InvalidAction(request.action)),
        }
    }
}
```

### Pattern 2: Service-Based Organization

For tools with complex business logic:

```
src/
├── tools/
│   └── payment.rs              # ~200 lines - tool layer only
├── services/
│   ├── payment_service/
│   │   ├── mod.rs             # ~100 lines - service interface
│   │   ├── processor.rs       # ~300 lines - payment processing
│   │   ├── validator.rs       # ~200 lines - validation logic
│   │   └── gateway.rs         # ~400 lines - gateway integration
│   └── mod.rs
└── models/
    └── payment.rs              # ~150 lines - data models
```

### Pattern 3: Handler Pattern

For tools with many related actions:

```
src/tools/notification/
├── mod.rs              # ~80 lines - tool struct
├── handlers/
│   ├── mod.rs         # ~30 lines - exports
│   ├── email.rs       # ~250 lines - email notifications
│   ├── sms.rs         # ~200 lines - SMS notifications
│   └── push.rs        # ~250 lines - push notifications
└── types.rs           # ~100 lines - shared types
```

## When to Split a File

Split when:
- File exceeds 400 lines (approaching limit)
- Multiple distinct responsibilities
- Code duplication appears
- Hard to navigate/find code
- Testing becomes difficult

### Checklist for Splitting

1. **Identify Responsibilities**
   - What are the distinct tasks?
   - Can they work independently?

2. **Group Related Code**
   - Related actions together
   - Shared types in one place
   - Common utilities grouped

3. **Create Clear Boundaries**
   - Each file has one purpose
   - Clear naming conventions
   - Logical organization

4. **Maintain Cohesion**
   - Related code stays together
   - Dependencies are clear
   - Easy to understand flow

## Code Size Guidelines

| File Type | Max Lines | Recommended |
|-----------|-----------|-------------|
| Tool main file | 500 | 200-300 |
| Service file | 500 | 300-400 |
| Handler file | 500 | 200-300 |
| Types file | 500 | 100-200 |
| Utility file | 500 | 100-200 |
| Test file | No limit | Keep reasonable |

## Example: Splitting Large Tool

### Before (650 lines)

```rust
// src/tools/content_management.rs - 650 lines

pub struct ContentTool;

impl ContentTool {
    // 50+ methods in one file
    async fn create_post() { }
    async fn update_post() { }
    async fn delete_post() { }
    async fn get_post() { }
    async fn list_posts() { }
    async fn create_category() { }
    async fn update_category() { }
    // ... many more
}
```

### After (split into modules)

```rust
// src/tools/content_management/mod.rs - 120 lines
mod posts;
mod categories;
mod tags;
mod media;
mod types;

pub use types::*;
use posts::PostsHandler;
use categories::CategoriesHandler;
use tags::TagsHandler;
use media::MediaHandler;

pub struct ContentTool {
    posts: PostsHandler,
    categories: CategoriesHandler,
    tags: TagsHandler,
    media: MediaHandler,
}

impl ContentTool {
    pub async fn execute(&self, request: ToolRequest) -> McpResult<ToolResult> {
        match request.action.as_str() {
            // Posts
            "create_post" => self.posts.create(request).await,
            "update_post" => self.posts.update(request).await,
            "delete_post" => self.posts.delete(request).await,
            "get_post" => self.posts.get(request).await,
            "list_posts" => self.posts.list(request).await,
            
            // Categories
            "create_category" => self.categories.create(request).await,
            "update_category" => self.categories.update(request).await,
            
            // Tags
            "create_tag" => self.tags.create(request).await,
            
            // Media
            "upload_media" => self.media.upload(request).await,
            
            _ => Err(McpError::InvalidAction(request.action)),
        }
    }
}

// src/tools/content_management/posts.rs - 280 lines
pub struct PostsHandler;
impl PostsHandler {
    pub async fn create(&self, request: ToolRequest) -> McpResult<ToolResult> { }
    pub async fn update(&self, request: ToolRequest) -> McpResult<ToolResult> { }
    pub async fn delete(&self, request: ToolRequest) -> McpResult<ToolResult> { }
    pub async fn get(&self, request: ToolRequest) -> McpResult<ToolResult> { }
    pub async fn list(&self, request: ToolRequest) -> McpResult<ToolResult> { }
}

// src/tools/content_management/categories.rs - 180 lines
pub struct CategoriesHandler;
impl CategoriesHandler {
    pub async fn create(&self, request: ToolRequest) -> McpResult<ToolResult> { }
    pub async fn update(&self, request: ToolRequest) -> McpResult<ToolResult> { }
}

// ... other handlers
```

## Naming Conventions

### File Names
- `mod.rs` - Module entry point
- `types.rs` - Type definitions
- `{action}.rs` - Action handlers (create.rs, read.rs)
- `{feature}_handler.rs` - Feature handlers
- `{entity}_service.rs` - Services

### Module Structure
```
tool_name/
├── mod.rs           # Public interface
├── types.rs         # Shared types
├── handlers/        # Action handlers
├── utils.rs         # Internal utilities
└── tests.rs         # Tests (optional)
```

## Tools Registration Pattern

Keep registration simple in main.rs by grouping:

```rust
// src/main.rs
mod tools {
    pub mod echo;
    pub mod user_management;
    pub mod content_management;
    pub mod payment;
}

// In router setup
.route("/tools/echo", post(handlers::echo::handle))
.route("/tools/users", post(handlers::users::handle))
.route("/tools/content", post(handlers::content::handle))
.route("/tools/payment", post(handlers::payment::handle))
```

## Refactoring Workflow

1. **Measure Current Size**
   ```bash
   wc -l src/tools/my_tool.rs
   ```

2. **Identify Split Points**
   - Group related functions
   - Find natural boundaries
   - Look for duplicate code

3. **Create Directory Structure**
   ```bash
   mkdir -p src/tools/my_tool
   touch src/tools/my_tool/mod.rs
   ```

4. **Move Code Gradually**
   - Start with types
   - Move one handler at a time
   - Update imports
   - Test after each move

5. **Verify**
   ```bash
   cargo check
   cargo test
   ```

## Quick Reference

### When File is Too Large

1. Create module directory
2. Split by responsibility
3. Keep mod.rs as coordinator
4. Move types to types.rs
5. Create handler files
6. Update imports
7. Test thoroughly

### Target File Sizes

- Simple tool: 150-250 lines
- Complex tool main: 200-300 lines
- Handler: 200-350 lines
- Service: 300-450 lines
- Types: 100-200 lines

## Best Practices

1. **One Responsibility Per File**
   - Each file does one thing well
   - Clear purpose and naming

2. **Clear Dependencies**
   - Explicit imports
   - Avoid circular dependencies

3. **Consistent Organization**
   - Follow same pattern across tools
   - Predictable structure

4. **Documentation**
   - Comment complex splits
   - Explain file organization

5. **Testing**
   - Test each module
   - Integration tests for tool

## Summary

- Max 500 lines per file
- Split by feature/action/responsibility
- Use services for complex logic
- Extract types and models
- Clear module organization
- Consistent naming conventions
- Gradual refactoring
- Test after splitting

---

Version: 1.0.0
Last Updated: 2025-01-08