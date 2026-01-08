# Contributing to MCP Boilerplate Rust

Simple guide for contributing to this MCP v5 Rust boilerplate project.

## Development Setup

1. Install Rust (1.70+):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Clone and setup:
```bash
cd mcp-boilerplate-rust
cp .env.example .env
cargo build
```

3. Run in development:
```bash
cargo run
```

## Code Style

- Use `cargo fmt` before committing
- Run `cargo clippy` to check for issues
- Keep it simple and clean
- Follow existing patterns

## Adding New Tools

1. Create tool file in `src/tools/`:
```rust
// src/tools/your_tool.rs
use crate::types::{McpError, McpResult, ToolRequest, ToolResult};
use serde_json::json;

pub struct YourTool;

impl YourTool {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, request: ToolRequest) -> McpResult<ToolResult> {
        match request.action.as_str() {
            "your_action" => self.handle_action(request).await,
            _ => Err(McpError::InvalidAction(request.action)),
        }
    }

    async fn handle_action(&self, request: ToolRequest) -> McpResult<ToolResult> {
        Ok(ToolResult {
            success: true,
            data: json!({
                "action": "your_action",
                "result": "success"
            }),
        })
    }
}
```

2. Register in `src/tools/mod.rs`:
```rust
pub mod your_tool;
pub use your_tool::YourTool;
```

3. Add route in `src/main.rs`:
```rust
.route("/tools/your_tool", post(handle_your_tool))
```

4. Add handler:
```rust
async fn handle_your_tool(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ToolRequest>,
) -> impl IntoResponse {
    let start_time = std::time::Instant::now();
    let tool = YourTool::new();
    
    match tool.execute(payload).await {
        Ok(result) => {
            let execution_time = start_time.elapsed().as_millis();
            (StatusCode::OK, Json(McpResponse {
                success: true,
                data: Some(result.data),
                error: None,
                metadata: Some(json!({
                    "executionTime": execution_time,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })),
            })).into_response()
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis();
            (StatusCode::INTERNAL_SERVER_ERROR, Json(McpResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
                metadata: Some(json!({
                    "executionTime": execution_time,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })),
            })).into_response()
        }
    }
}
```

## Testing

Run tests:
```bash
cargo test
```

Test with curl:
```bash
./test.sh
```

Manual test:
```bash
curl -X POST http://localhost:8025/tools/your_tool \
  -H "Content-Type: application/json" \
  -d '{"action":"your_action"}'
```

## Code Quality

Format code:
```bash
cargo fmt
```

Lint code:
```bash
cargo clippy
```

Check for issues:
```bash
cargo check
```

## Error Handling

Use proper error types from `types.rs`:

```rust
// Missing parameter
Err(McpError::MissingParameter("field_name".to_string()))

// Invalid action
Err(McpError::InvalidAction(action.to_string()))

// Database error
Err(McpError::Database("connection failed".to_string()))

// Internal error
Err(McpError::Internal("something went wrong".to_string()))
```

## Logging

Use tracing for logs:

```rust
use tracing::{info, warn, error, debug};

info!("Processing request");
warn!("Invalid input: {}", value);
error!("Operation failed: {}", err);
debug!("Debug info: {:?}", data);
```

## Response Format

All tools must return consistent format:

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

## Pull Requests

1. Fork the repository
2. Create feature branch: `git checkout -b feature/your-tool`
3. Make changes
4. Format and test: `cargo fmt && cargo test`
5. Commit: `git commit -m "Add your_tool"`
6. Push: `git push origin feature/your-tool`
7. Create Pull Request

## Commit Messages

Keep it simple and clear:
- `Add user_management tool`
- `Fix authentication error handling`
- `Update echo tool documentation`
- `Refactor error types`

## Documentation

Update README.md when:
- Adding new tools
- Changing API endpoints
- Adding new features
- Modifying configuration

## Performance

- Use async/await properly
- Avoid blocking operations
- Keep handlers simple
- Profile with `cargo flamegraph` if needed

## Dependencies

Keep dependencies minimal:
- Only add when necessary
- Prefer standard library when possible
- Check compatibility with existing versions

## Environment Variables

Add new variables to:
1. `.env.example`
2. `README.md` environment variables section
3. Load in `main.rs` if needed

## Questions

Open an issue for:
- Bug reports
- Feature requests
- Questions about implementation
- Documentation improvements

## License

By contributing, you agree that your contributions will be licensed under MIT License.