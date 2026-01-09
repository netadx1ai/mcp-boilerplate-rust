# MCP Specification Review & Enhancement Roadmap

**Date**: 2026-01-08  
**Current Implementation**: v0.3.1 (Protocol 2024-11-05)  
**Latest MCP Spec**: 2025-11-25  
**Gap Analysis**: Protocol version behind by ~1 year

## Executive Summary

After reviewing the official MCP specification repository, our Rust boilerplate implementation is **functionally complete** for the 2024-11-05 protocol version but **missing newer features** from the 2025-11-25 specification. This document outlines the gaps and provides a prioritized roadmap for upgrades.

## Current Implementation Status

### ✅ Fully Implemented (2024-11-05 Compliant)

| Feature | Status | Notes |
|---------|--------|-------|
| Tools (5) | ✅ Complete | echo, ping, info, calculate, evaluate |
| Prompts (3) | ✅ Complete | code_review, explain_code, debug_help |
| Resources (4) | ✅ Complete | config, capabilities, docs, stats |
| Stdio Transport | ✅ Complete | Primary transport mode |
| HTTP Transport | ✅ Complete | Feature-gated, optional |
| Logging | ✅ Complete | Context-aware (off in stdio) |
| Input Validation | ✅ Complete | All tools validated |
| Error Handling | ✅ Complete | Protocol + Tool errors |

### ⚠️ Missing Features (2025-11-25 Spec)

Based on the changelog review, here are features we're missing:

## Gap Analysis: Missing Features

### 1. Icons Support (SEP-973) - HIGH PRIORITY
**Spec Version**: 2025-11-25  
**Status**: ❌ Not Implemented  
**Impact**: Moderate (UX enhancement)

**What's Missing**:
- Tools, resources, prompts, and resource templates can now include `icons` array
- Icons have `src` (URL), `mimeType`, and `sizes` fields
- Enhances UI/UX in clients like Claude Desktop

**Implementation Effort**: Low (1-2 hours)

**Example**:
```rust
// Add to tool definition
icons: Some(vec![Icon {
    src: "https://example.com/calculator-icon.svg".to_string(),
    mime_type: Some("image/svg+xml".to_string()),
    sizes: Some(vec!["any".to_string()]),
}])
```

**Files to Update**:
- `src/tools/mod.rs` - Add icons to tool definitions
- `src/prompts/mod.rs` - Add icons to prompt definitions  
- `src/resources/mod.rs` - Add icons to resource definitions

---

### 2. Tool Output Schema (SEP-1577) - MEDIUM PRIORITY
**Spec Version**: 2025-11-25  
**Status**: ❌ Not Implemented  
**Impact**: Moderate (better type safety)

**What's Missing**:
- Tools can declare `outputSchema` to validate structured results
- Enables `structuredContent` field in tool results
- Better integration with LLMs and type-safe clients

**Implementation Effort**: Medium (3-4 hours)

**Example**:
```rust
Tool {
    name: "calculate".to_string(),
    input_schema: json!({...}),
    output_schema: Some(json!({
        "type": "object",
        "properties": {
            "result": {"type": "number"},
            "operation": {"type": "string"}
        },
        "required": ["result", "operation"]
    })),
}
```

**Files to Update**:
- `src/tools/calculator.rs` - Add output schemas
- `src/tools/echo.rs` - Add output schemas
- `src/mcp/stdio_server.rs` - Return `structuredContent` in results

---

### 3. Resource Annotations (2025-11-25) - MEDIUM PRIORITY
**Spec Version**: 2025-11-25  
**Status**: ❌ Not Implemented  
**Impact**: Moderate (better resource metadata)

**What's Missing**:
- Resources support `annotations` with:
  - `audience`: `["user", "assistant"]` - intended audience
  - `priority`: 0.0-1.0 - importance level
  - `lastModified`: ISO 8601 timestamp

**Implementation Effort**: Low (1-2 hours)

**Example**:
```rust
ResourceContents::TextResourceContents {
    uri: uri.to_string(),
    mime_type: Some("application/json".to_string()),
    text: content,
    annotations: Some(Annotations {
        audience: Some(vec!["user".to_string(), "assistant".to_string()]),
        priority: Some(0.8),
        last_modified: Some(chrono::Utc::now().to_rfc3339()),
    }),
}
```

**Files to Update**:
- `src/resources/mod.rs` - Add annotations to all resources

---

### 4. Tasks (SEP-1686) - LOW PRIORITY (Experimental)
**Spec Version**: 2025-11-25  
**Status**: ❌ Not Implemented  
**Impact**: Low (advanced feature, experimental)

**What's Missing**:
- Task-augmented requests for long-running operations
- Task lifecycle management (list, cancel, poll)
- Deferred result retrieval
- Batch processing support

**Implementation Effort**: High (8-12 hours)

**Why Low Priority**:
- Marked as **EXPERIMENTAL** in spec
- Complex state management (conflicts with stateless design)
- Not required for basic MCP functionality
- Low adoption in current clients

**Recommendation**: Wait for spec stabilization and client adoption

---

### 5. Enhanced Error Handling (SEP-1303) - HIGH PRIORITY
**Spec Version**: 2025-11-25  
**Status**: ⚠️ Partially Implemented  
**Impact**: High (better LLM self-correction)

**What's Missing**:
- Clear distinction between Protocol Errors vs Tool Execution Errors
- Input validation errors should be Tool Execution Errors (not Protocol Errors)
- Enables model self-correction on validation failures

**Current Implementation**: We mix Protocol and Tool errors

**Implementation Effort**: Low (1 hour)

**Example**:
```rust
// BEFORE (Protocol Error - bad)
if message.is_empty() {
    return Err(McpError::invalid_params("Message cannot be empty", None));
}

// AFTER (Tool Execution Error - good)
if message.is_empty() {
    return Ok(CallToolResult {
        content: vec![TextContent {
            text: "Error: Message cannot be empty. Please provide a non-empty message.".to_string()
        }],
        is_error: true,
    });
}
```

**Files to Update**:
- `src/tools/echo.rs` - Return Tool Execution Errors for validation
- `src/tools/calculator.rs` - Return Tool Execution Errors for validation

---

### 6. Tool Names Guidance (SEP-986) - LOW PRIORITY
**Spec Version**: 2025-11-25  
**Status**: ✅ Already Compliant  
**Impact**: None (documentation only)

**What Changed**:
- Tool names SHOULD be 1-128 characters
- SHOULD be case-sensitive
- SHOULD use only: A-Z, a-z, 0-9, _, -, .
- SHOULD NOT contain spaces or special chars

**Our Tools**:
- ✅ `echo` - compliant
- ✅ `ping` - compliant
- ✅ `info` - compliant
- ✅ `calculate` - compliant
- ✅ `evaluate` - compliant

**Action**: No changes needed, update docs to reference guidelines

---

### 7. Resource Templates & URI Templates (RFC 6570) - LOW PRIORITY
**Spec Version**: 2025-11-25  
**Status**: ❌ Not Implemented  
**Impact**: Low (advanced resource feature)

**What's Missing**:
- `resources/templates/list` endpoint
- URI template support (e.g., `file:///{path}`)
- Parameterized resources

**Implementation Effort**: Medium (4-6 hours)

**Example**:
```rust
ResourceTemplate {
    uri_template: "file:///{path}".to_string(),
    name: "Project Files".to_string(),
    description: Some("Access files in the project".to_string()),
    mime_type: Some("application/octet-stream".to_string()),
}
```

**Recommendation**: Implement only if use case emerges

---

### 8. Resource Subscriptions - LOW PRIORITY
**Spec Version**: 2024-11-05 (already in spec)  
**Status**: ❌ Not Implemented  
**Impact**: Low (our resources are stateless)

**What's Missing**:
- `resources/subscribe` request
- `notifications/resources/updated` notification
- Real-time resource change notifications

**Why Not Implemented**:
- Our resources are **dynamically generated** (stateless)
- No file watching or external state changes
- Low value for boilerplate/template project

**Recommendation**: Document as "not applicable for stateless servers"

---

### 9. OAuth 2.0 / Authorization - LOW PRIORITY
**Spec Version**: 2025-11-25  
**Status**: ❌ Not Implemented  
**Impact**: Low (security feature, optional)

**What's Missing**:
- OAuth 2.0 Protected Resource Metadata (RFC 9728)
- OpenID Connect Discovery 1.0
- Incremental scope consent
- `WWW-Authenticate` header support

**Implementation Effort**: High (12-16 hours)

**Why Low Priority**:
- Stdio mode doesn't support authentication (by design)
- HTTP mode has basic JWT middleware
- Complex enterprise feature
- Not required for basic MCP

**Recommendation**: Implement in separate "enterprise" branch if needed

---

### 10. Audio Content Type - LOW PRIORITY
**Spec Version**: 2025-11-25  
**Status**: ❌ Not Implemented  
**Impact**: Low (multimodal feature)

**What's Missing**:
- Audio content in tool results
- Audio content in prompt messages
- Base64-encoded audio data support

**Implementation Effort**: Low (1 hour)

**Example**:
```rust
Content::Audio {
    data: base64_encoded_audio,
    mime_type: "audio/wav".to_string(),
}
```

**Recommendation**: Add when use case emerges

---

## Prioritized Roadmap

### Phase 1: Quick Wins (Next Session - 2-4 hours)

1. **Enhanced Error Handling** (1 hour)
   - Convert validation errors to Tool Execution Errors
   - Enable LLM self-correction
   - High impact, low effort

2. **Icons Support** (1-2 hours)
   - Add icons to tools, prompts, resources
   - Better UX in Claude Desktop
   - Easy implementation

3. **Resource Annotations** (1 hour)
   - Add audience, priority, lastModified
   - Richer metadata for clients
   - Simple addition

### Phase 2: Type Safety (Future - 3-4 hours)

4. **Tool Output Schema** (3-4 hours)
   - Add outputSchema to all tools
   - Support structuredContent responses
   - Better type safety and validation

### Phase 3: Advanced Features (Optional - 8-12 hours)

5. **Resource Templates** (4-6 hours)
   - URI template support
   - Parameterized resources
   - Only if use case emerges

6. **Tasks Support** (8-12 hours)
   - Wait for spec stabilization
   - Requires state management redesign
   - Experimental feature

### Phase 4: Enterprise Features (Future Branch)

7. **OAuth 2.0 / Authorization** (12-16 hours)
   - Separate enterprise branch
   - Complex security implementation
   - Optional for most users

---

## Protocol Version Upgrade Path

### Option A: Stay on 2024-11-05 (Conservative)
**Pros**:
- Fully compliant and tested
- Stable, production-ready
- No breaking changes

**Cons**:
- Missing newer features
- May not support latest clients

### Option B: Upgrade to 2025-11-25 (Recommended)
**Pros**:
- Access to new features (icons, annotations, etc.)
- Better UX and type safety
- Future-proof

**Cons**:
- Need to update rmcp SDK (check compatibility)
- Some features experimental (tasks)
- More testing required

**Recommendation**: Upgrade incrementally
1. Update to 2025-11-25 protocol version
2. Implement Phase 1 features (quick wins)
3. Add Phase 2 features (type safety)
4. Leave Phase 3/4 for future/enterprise

---

## SDK Compatibility Check

**Action Required**: Check if `rmcp` SDK supports 2025-11-25

```bash
# Check current SDK version
grep "rmcp" Cargo.toml

# Check for updates
cargo search rmcp

# Review SDK changelog
```

**If rmcp doesn't support 2025-11-25 yet**:
- Implement features that don't require SDK changes (icons, annotations)
- Wait for SDK update for protocol-level changes
- Consider contributing to rmcp SDK

---

## Code Structure Improvements

### Suggested Refactoring (Separate from Spec)

1. **Shared Annotations Module**
   ```rust
   // src/common/annotations.rs
   pub struct Annotations {
       pub audience: Option<Vec<String>>,
       pub priority: Option<f64>,
       pub last_modified: Option<String>,
   }
   ```

2. **Icon Helper Module**
   ```rust
   // src/common/icons.rs
   pub fn default_tool_icon() -> Icon { ... }
   pub fn default_prompt_icon() -> Icon { ... }
   ```

3. **Error Type Refinement**
   ```rust
   // src/utils/types.rs
   pub enum McpError {
       ProtocolError(String),     // For protocol violations
       ToolExecutionError(String), // For tool-level errors
       ResourceError(String),      // For resource errors
   }
   ```

---

## Testing Requirements

### New Tests Needed

1. **Icons Tests**
   - Validate icon URLs
   - Test icon rendering (if applicable)
   - Ensure proper serialization

2. **Output Schema Tests**
   - Validate structured content against schema
   - Test schema validation failures
   - Ensure backward compatibility

3. **Error Handling Tests**
   - Verify Tool Execution Error format
   - Test LLM self-correction scenarios
   - Validate error message clarity

4. **Annotation Tests**
   - Test audience filtering
   - Validate priority ranges (0.0-1.0)
   - Check timestamp formats

### Test Scripts to Update

- `scripts/test_mcp.sh` - Add error handling tests
- `scripts/test_prompts_resources.sh` - Add annotation tests
- `scripts/test_validation.sh` - Add output schema tests

---

## Documentation Updates Needed

1. **CLAUDE.md**
   - Update protocol version to 2025-11-25
   - Document new features (icons, annotations)
   - Update examples with output schemas

2. **README.md**
   - Highlight 2025-11-25 compliance
   - List new features
   - Update architecture diagram if needed

3. **PROMPTS_AND_RESOURCES.md**
   - Add annotation examples
   - Document icon usage
   - Update best practices

4. **New File: PROTOCOL_UPGRADE_GUIDE.md**
   - Migration guide from 2024-11-05 to 2025-11-25
   - Breaking changes (if any)
   - Feature comparison table

---

## Recommended Next Session Plan

### Session Goal: Implement Phase 1 (Quick Wins)

**Time Estimate**: 2-4 hours

**Tasks**:

1. **Check SDK Compatibility** (15 min)
   - Review rmcp changelog
   - Check for 2025-11-25 support
   - Update Cargo.toml if needed

2. **Enhanced Error Handling** (1 hour)
   - Update `echo.rs` validation to return Tool Execution Errors
   - Update `calculator.rs` validation
   - Add tests for error format
   - Document error handling pattern

3. **Icons Support** (1-2 hours)
   - Add Icon struct/type imports
   - Add icons to all 5 tools
   - Add icons to 3 prompts
   - Add icons to 4 resources
   - Test icon serialization
   - Update docs with examples

4. **Resource Annotations** (1 hour)
   - Add Annotations to ResourceContents
   - Set appropriate audience/priority for each resource
   - Add timestamps
   - Test annotation serialization
   - Update docs

5. **Update Protocol Version** (30 min)
   - Change protocol version to 2025-11-25
   - Update all references in code/docs
   - Update README.md
   - Run full test suite

6. **Documentation** (30 min)
   - Update CLAUDE.md with new features
   - Create PROTOCOL_UPGRADE_GUIDE.md
   - Update IMPLEMENTATION_STATUS.md

**Validation**:
- [ ] All tests pass
- [ ] Clippy clean
- [ ] Cargo fmt applied
- [ ] Documentation updated
- [ ] Example requests/responses work

---

## Long-term Considerations

### 1. State Management for Tasks
If implementing tasks in the future:
- Consider Redis for task state
- PostgreSQL for durable task storage
- In-memory for simple use cases
- Design state management interface

### 2. WebSocket Transport
Not in current spec but commonly requested:
- Bidirectional communication
- Real-time updates
- Lower latency
- Consider for future enhancement

### 3. Multi-Server Orchestration
Enterprise feature:
- Server composition
- Capability aggregation
- Request routing
- Consider for enterprise branch

### 4. Monitoring & Observability
Production feature:
- Prometheus metrics
- OpenTelemetry tracing
- Structured logging
- Health checks
- Consider for v0.4.0

---

## Summary

**Current State**: Solid 2024-11-05 implementation, production-ready

**Recommended Path**: 
1. Upgrade to 2025-11-25 protocol
2. Implement Phase 1 features (quick wins)
3. Add Phase 2 features (type safety)
4. Monitor spec evolution for Phase 3/4

**Estimated Effort**:
- Phase 1: 2-4 hours
- Phase 2: 3-4 hours
- Total for modern compliance: ~6-8 hours

**Value Proposition**:
- Better UX (icons)
- Better type safety (output schemas)
- Better error handling (LLM self-correction)
- Future-proof (latest spec)

**Risk**: Low - all changes are additive, no breaking changes

---

**Next Steps**: Start Phase 1 in next development session

**Prepared by**: AI Development Team  
**Review Date**: 2026-01-08  
**Status**: Ready for Implementation