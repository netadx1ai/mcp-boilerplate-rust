# MCP Rust Boilerplate - Deep Research Analysis & Improvements

**Analysis Date:** 2026-01-13 (HCMC Timezone)  
**Rust SDK Version:** 0.12.0  
**MCP Protocol:** 2024-11-05 / 2025-03-26  
**Current Boilerplate Version:** 0.3.1

## Executive Summary

After analyzing the official `rust-sdk` repository and `modelcontextprotocol` specification, identified 12 critical improvements and 8 optional enhancements for the MCP Rust boilerplate.

**Priority Status:**
- 🔴 Critical: 5 features (Task support, Better macros, Progress notifications, RequestContext, Elicitation)
- 🟡 High: 4 features (Resource templates, Multi-transport, Better examples, OAuth)
- 🟢 Medium: 3 features (Metrics, Benchmarks, WASI support)

---

## 1. CRITICAL MISSING FEATURES

### 1.1 Task Lifecycle Support (SEP-1686) 🔴

**Status:** NOT IMPLEMENTED  
**Impact:** HIGH - Blocks long-running operations

**What is it:**
- Queue-based system for long-running/async tool calls
- Prevents timeout on operations >30s
- Enables progress tracking and cancellation

**Official Implementation Pattern:**
```rust
// From rust-sdk/examples/servers/src/common/counter.rs

use rmcp::task_manager::OperationProcessor;

#[derive(Clone)]
pub struct McpServer {
    tool_router: ToolRouter<Self>,
    processor: Arc<Mutex<OperationProcessor>>,
}

#[task_handler]
impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_tasks()  // ← Enable task capability
                .build(),
            ..Default::default()
        }
    }
}

// Long-running tool example
#[tool(description = "Long running task example")]
async fn long_task(&self) -> Result<CallToolResult, McpError> {
    tokio::time::sleep(Duration::from_secs(60)).await;
    Ok(CallToolResult::success(vec![Content::text("Completed")]))
}
```

**Client Usage:**
```rust
// Create task instead of immediate execution
let params = CallToolRequestParam {
    name: "long_task".into(),
    arguments: None,
    task: Some(serde_json::Map::new()),  // ← Enable task mode
};

let response = service.send_request(params).await?;
// Response: CreateTaskResult with task_id

// Poll for result
let result = service.tasks_result(task_id).await?;

// Or cancel
service.tasks_cancel(task_id).await?;
```

**Required Changes:**
1. Add `OperationProcessor` to `McpServer` struct
2. Implement `#[task_handler]` macro
3. Enable `.enable_tasks()` in capabilities
4. Add task endpoints: list, get, result, cancel
5. Update tests to verify task lifecycle

**Files to Modify:**
- `src/mcp/stdio_server.rs` - Add processor field
- `Cargo.toml` - Verify rmcp task features
- `scripts/test_mcp.sh` - Add task tests

---

### 1.2 Improved Macro Usage 🔴

**Status:** PARTIAL - Using old patterns  
**Impact:** HIGH - Code maintainability

**Current Pattern (Boilerplate):**
```rust
#[tool_router]
impl McpServer {
    #[tool(description = "Echo back a message")]
    async fn echo(&self, Parameters(req): Parameters<EchoRequest>) 
        -> Result<Json<EchoResponse>, McpError> {
        // Manual implementation
    }
}

#[tool_handler]
impl ServerHandler for McpServer {
    // Manual tool routing
}
```

**Better Pattern (rust-sdk examples):**
```rust
#[tool_router]
impl Counter {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
            tool_router: Self::tool_router(),  // ← Auto-generated
        }
    }
}

#[tool_handler(meta = Meta(rmcp::object!({"version": "1.0"})))]
#[prompt_handler(meta = Meta(rmcp::object!({"prompts": "enabled"})))]
#[task_handler]
impl ServerHandler for Counter {}  // ← All routing auto-wired
```

**Advantages:**
- Less boilerplate code
- Automatic tool discovery
- Type-safe metadata
- Better error messages
- Consistent API surface

**Required Changes:**
1. Use `Self::tool_router()` pattern
2. Add `#[prompt_handler]` for prompts
3. Remove manual routing code
4. Add metadata to handlers

---

### 1.3 Progress Notifications 🔴

**Status:** NOT IMPLEMENTED  
**Impact:** HIGH - User experience for long operations

**Official Pattern:**
```rust
#[tool(description = "Process large dataset with progress updates")]
async fn stream_processor(&self, ctx: RequestContext<RoleServer>) 
    -> Result<CallToolResult, McpError> {
    
    let peer = ctx.peer;
    
    for i in 0..100 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Send progress notification
        peer.notify_progress(ProgressNotificationParam {
            progress_token: ProgressToken::String("task_123".into()),
            progress: i as f64,
            total: Some(100.0),
        }).await?;
    }
    
    Ok(CallToolResult::success(vec![Content::text("Done")]))
}
```

**Client Side:**
```rust
impl ClientHandler for MyClient {
    async fn on_progress(
        &self,
        notification: ProgressNotificationParam,
        context: NotificationContext<RoleClient>,
    ) {
        println!("Progress: {}/{}", 
            notification.progress, 
            notification.total.unwrap_or(0.0)
        );
    }
}
```

**Use Cases:**
- File processing (upload/download)
- Data transformation
- Model inference
- Batch operations

**Required Changes:**
1. Add `RequestContext<RoleServer>` to tool signatures
2. Implement `peer.notify_progress()` calls
3. Add progress capability to ServerInfo
4. Create example tool with progress

---

### 1.4 RequestContext Access 🔴

**Status:** NOT USED  
**Impact:** MEDIUM - Missing peer communication

**What It Provides:**
```rust
pub struct RequestContext<Role> {
    pub peer: Arc<Peer>,           // ← Send notifications/requests back
    pub extensions: Extensions,     // ← HTTP headers, metadata
    // ... internal fields
}
```

**Current Signature:**
```rust
async fn echo(&self, Parameters(req): Parameters<EchoRequest>) 
    -> Result<Json<EchoResponse>, McpError>
```

**Better Signature:**
```rust
async fn echo(
    &self, 
    Parameters(req): Parameters<EchoRequest>,
    ctx: RequestContext<RoleServer>,  // ← Add context
) -> Result<Json<EchoResponse>, McpError> {
    // Access peer for notifications
    ctx.peer.notify_logging_message(LoggingMessageNotificationParam {
        level: LoggingLevel::Info,
        logger: Some("echo".into()),
        data: json!({"message": "Processing echo request"}),
    }).await?;
    
    // Access HTTP headers (if HTTP transport)
    if let Some(parts) = ctx.extensions.get::<axum::http::request::Parts>() {
        let auth = parts.headers.get("authorization");
        tracing::info!(?auth, "Auth header");
    }
    
    // Process request
    Ok(Json(response))
}
```

**Benefits:**
- Bidirectional communication
- Access to transport metadata
- HTTP header inspection
- Better logging/debugging

---

### 1.5 Elicitation Support 🔴

**Status:** NOT IMPLEMENTED  
**Impact:** MEDIUM - Interactive workflows

**What is Elicitation:**
Safely collect user input during tool execution with type-safe validation.

**Official Pattern:**
```rust
use rmcp::{elicit_safe, schemars::JsonSchema};

#[derive(Serialize, Deserialize, JsonSchema)]
struct UserInfo {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
}

#[tool(description = "Greet user by name")]
async fn greet_user(
    &self,
    ctx: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    // Elicit user info with type validation
    let user_info: UserInfo = elicit_safe!(ctx.peer, {
        "prompt": "What is your name?",
        "description": "Please provide your information"
    })?;
    
    let greeting = format!("Hello, {}!", user_info.name);
    Ok(CallToolResult::success(vec![Content::text(greeting)]))
}
```

**Use Cases:**
- Authentication flows
- Configuration wizards
- Interactive data collection
- Multi-step workflows

**Required Changes:**
1. Add elicitation capability to ServerInfo
2. Import `elicit_safe!` macro
3. Create example interactive tool
4. Add schemars for validation

---

## 2. HIGH PRIORITY IMPROVEMENTS

### 2.1 Resource Templates 🟡

**Status:** STUB ONLY  
**Current:**
```rust
async fn list_resource_templates(&self) -> Result<...> {
    Ok(ListResourceTemplatesResult::default())  // Empty!
}
```

**Should Be:**
```rust
async fn list_resource_templates(&self) -> Result<...> {
    Ok(ListResourceTemplatesResult {
        resource_templates: vec![
            ResourceTemplate {
                uri_template: "file://{path}".into(),
                name: "File Reader".into(),
                description: Some("Read any file by path".into()),
                mime_type: Some("text/plain".into()),
            },
            ResourceTemplate {
                uri_template: "db://{table}/{id}".into(),
                name: "Database Record".into(),
                description: Some("Fetch record by table and ID".into()),
                mime_type: Some("application/json".into()),
            },
        ],
        next_cursor: None,
        meta: None,
    })
}
```

---

### 2.2 Multiple Transport Examples 🟡

**Status:** STDIO + HTTP only  
**Missing:** WebSocket, TCP, Unix Socket, SSE

**Official Examples Available:**
```
rust-sdk/examples/transport/
├── tcp.rs
├── http_upgrade.rs
├── unix_socket.rs
└── websocket.rs
```

**Recommendation:**
Create `examples/transports/` with:
1. `stdio_example.rs` - Current default
2. `http_streamable.rs` - SSE-based streaming
3. `websocket_example.rs` - WebSocket bidirectional
4. `tcp_example.rs` - Raw TCP transport

---

### 2.3 Better Example Tools 🟡

**Current Tools:**
- echo, ping, info (basic)
- calculate, evaluate (math)

**Add These Examples:**
1. **File Operations**
   - read_file, write_file, list_directory
   - Shows resource management

2. **Data Processing**
   - json_transform, csv_parse, xml_parse
   - Shows structured data handling

3. **External APIs**
   - fetch_url, github_search, weather_api
   - Shows async HTTP clients

4. **State Management**
   - counter, memory_store, session_manager
   - Shows stateful operations

**Reference:** `rust-sdk/examples/servers/`

---

### 2.4 OAuth2 Integration 🟡

**Status:** NOT IMPLEMENTED  
**Available in SDK:** YES (full OAuth2 flow)

**Official Example:**
```
rust-sdk/examples/servers/src/complex_auth_streamhttp.rs
```

**Features:**
- Client registration
- Authorization code flow
- Token validation
- Metadata discovery (`/.well-known/oauth-authorization-server`)

**Implementation Pattern:**
```rust
use rmcp::auth::oauth2::{
    AuthorizationServer,
    ClientRegistration,
    TokenResponse,
};

// See rust-sdk/examples/servers/src/complex_auth_streamhttp.rs
// for full 400+ line implementation
```

**Required for Production:**
- Secure token storage
- Client credential management
- Scope-based authorization
- Token refresh flows

---

## 3. MEDIUM PRIORITY ENHANCEMENTS

### 3.1 Metrics & Observability 🟢

**Add:**
- Request counting
- Latency tracking
- Error rate monitoring
- Tool usage statistics

**Pattern:**
```rust
use std::sync::atomic::{AtomicU64, Ordering};

pub struct Metrics {
    requests_total: AtomicU64,
    requests_success: AtomicU64,
    requests_error: AtomicU64,
}

impl Metrics {
    pub fn record_request(&self, success: bool) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        if success {
            self.requests_success.fetch_add(1, Ordering::Relaxed);
        } else {
            self.requests_error.fetch_add(1, Ordering::Relaxed);
        }
    }
}
```

---

### 3.2 Benchmarks 🟢

**Current:** None  
**Add:** `benches/` directory with criterion.rs

**Benchmark Suite:**
```rust
// benches/tool_performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_echo(c: &mut Criterion) {
    c.bench_function("echo_tool", |b| {
        b.iter(|| {
            // Tool execution
        });
    });
}

criterion_group!(benches, bench_echo);
criterion_main!(benches);
```

---

### 3.3 WASI Support 🟢

**What:** WebAssembly System Interface for portable execution  
**Reference:** `rust-sdk/examples/wasi/`

**Use Cases:**
- Sandboxed tool execution
- Edge deployment
- Plugin systems
- Security isolation

---

## 4. IMPLEMENTATION ROADMAP

### Phase 1: Critical (Week 1)
- [ ] Task lifecycle support (SEP-1686)
- [ ] Improved macro patterns
- [ ] Progress notifications
- [ ] RequestContext in all tools
- [ ] Update documentation

### Phase 2: High Priority (Week 2)
- [ ] Resource templates implementation
- [ ] Elicitation support
- [ ] Multi-transport examples
- [ ] Additional tool examples

### Phase 3: Medium Priority (Week 3)
- [ ] OAuth2 integration
- [ ] Metrics & observability
- [ ] Benchmark suite
- [ ] WASI exploration

### Phase 4: Polish (Week 4)
- [ ] Performance optimization
- [ ] Security audit
- [ ] Documentation updates
- [ ] Example gallery

---

## 5. BREAKING CHANGES ASSESSMENT

### Required Breaking Changes:
1. **Tool signatures** - Add `RequestContext` parameter
2. **Server struct** - Add `OperationProcessor` field
3. **Capabilities** - Enable tasks, progress

### Migration Path:
```rust
// Old
async fn echo(&self, params: Parameters<EchoRequest>) 
    -> Result<Json<EchoResponse>, McpError>

// New
async fn echo(
    &self, 
    params: Parameters<EchoRequest>,
    _ctx: RequestContext<RoleServer>,  // ← Add (can ignore with _)
) -> Result<Json<EchoResponse>, McpError>
```

**Impact:** LOW - Parameter addition is backward compatible

---

## 6. COMPARATIVE ANALYSIS

### Current Boilerplate (0.3.1)
```
Strengths:
✓ Clean architecture
✓ Good documentation
✓ Validation examples
✓ HTTP + stdio support
✓ Production-ready error handling

Gaps:
✗ No task support
✗ No progress notifications
✗ Limited macro usage
✗ No elicitation
✗ No OAuth
✗ Minimal examples
```

### rust-sdk Examples
```
Strengths:
✓ Task lifecycle complete
✓ Progress demo server
✓ Full macro coverage
✓ Elicitation examples
✓ OAuth2 server
✓ 15+ server examples
✓ Multiple transports
✓ WASI support

Educational Value:
✓ counter_stdio.rs - Basic pattern
✓ progress_demo.rs - Progress tracking
✓ elicitation_stdio.rs - User input
✓ complex_auth_streamhttp.rs - OAuth2
✓ memory_stdio.rs - State management
```

---

## 7. RECOMMENDED NEXT STEPS

### Immediate (This Session):
1. Add task support to McpServer
2. Update tool signatures with RequestContext
3. Implement progress_demo tool
4. Add task tests

### Short Term (Next Session):
1. Create elicitation example
2. Implement resource templates
3. Add multiple transport examples
4. Update CLAUDE.md

### Long Term (Future):
1. OAuth2 integration
2. Metrics dashboard
3. Benchmark suite
4. WASI investigation

---

## 8. CODE REFERENCES

### Key Files to Study:
```
rust-sdk/crates/rmcp/README.md
  └─ Task lifecycle documentation

rust-sdk/examples/servers/src/common/counter.rs
  └─ Full-featured server with tasks, prompts, resources

rust-sdk/examples/servers/src/progress_demo.rs
  └─ Progress notification implementation

rust-sdk/examples/servers/src/elicitation_stdio.rs
  └─ Type-safe user input collection

rust-sdk/examples/servers/src/complex_auth_streamhttp.rs
  └─ Complete OAuth2 authorization server
```

### Macro Documentation:
```
rust-sdk/crates/rmcp-macros/README.md
  └─ #[tool], #[tool_router], #[tool_handler]
  └─ #[prompt], #[prompt_router], #[prompt_handler]
  └─ #[task_handler] - Task lifecycle wiring
```

---

## 9. CONCLUSION

The boilerplate is production-ready but missing modern MCP features. Priority focus:

1. **Task support** - Enables long-running operations
2. **Progress notifications** - Better UX
3. **RequestContext** - Bidirectional communication
4. **Better examples** - Learning resources

All features are proven in rust-sdk examples. Implementation is straightforward with clear patterns to follow.

**Estimated Effort:** 3-4 sessions for critical features  
**Risk Level:** LOW - All patterns proven in official SDK  
**User Impact:** HIGH - Unlocks advanced MCP capabilities

---

**Document Version:** 1.0  
**Last Updated:** 2026-01-13  
**Next Review:** After Phase 1 implementation