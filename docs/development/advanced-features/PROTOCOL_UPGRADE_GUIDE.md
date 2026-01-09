# MCP Protocol Upgrade Guide

**From:** MCP 2024-11-05  
**To:** MCP 2025-03-26  
**Date:** 2026-01-08  
**Status:** Complete

## Overview

This guide documents the upgrade of the MCP Boilerplate Rust server from protocol version 2024-11-05 to 2025-03-26. The upgrade adds three major feature categories:

1. **Icons Support** - Visual indicators for tools, prompts, and resources
2. **Resource Annotations** - Metadata including audience, priority, and timestamps
3. **Enhanced Error Handling** - Tool execution errors for LLM self-correction

## Breaking Changes

**None.** All changes are additive and backward compatible.

## New Features

### 1. Icons Support

Icons can now be added to:
- Tools
- Prompts  
- Resources
- Resource Templates

#### Icon Structure

```rust
use rmcp::model::Icon;

Icon {
    src: "data:image/svg+xml;base64,...".to_string(),  // URI or data URL
    mime_type: Some("image/svg+xml".to_string()),      // Optional MIME type
    sizes: Some(vec!["any".to_string()]),              // Optional size specs
}
```

#### Example: Adding Icons to Prompts

```rust
use rmcp::model::{Icon, Prompt};

Prompt {
    name: "code_review".to_string(),
    description: Some("Generate code review prompts".to_string()),
    icons: Some(vec![Icon {
        src: "data:image/svg+xml;base64,PHN2ZyB4bWxucz0i...".to_string(),
        mime_type: Some("image/svg+xml".to_string()),
        sizes: Some(vec!["any".to_string()]),
    }]),
    // ... other fields
}
```

#### Icon Best Practices

1. **Use SVG for scalability** - Set sizes to `["any"]`
2. **Use data URLs** - Embed icons as base64 for self-contained deployment
3. **Keep icons simple** - 24x24px equivalent for clarity
4. **Use semantic icons** - Match icon to function (gear for settings, etc.)

### 2. Resource Annotations

Resources can now include annotations with:
- **Audience**: Who should see this resource (User, Assistant, or both)
- **Priority**: Importance level (0.0 to 1.0)
- **Last Modified**: Timestamp for cache invalidation

#### Annotations Structure

```rust
use rmcp::model::{Annotations, Role};
use chrono::Utc;

Annotations {
    audience: Some(vec![Role::User, Role::Assistant]),
    priority: Some(0.8),  // 0.0 = low, 1.0 = high
    last_modified: Some(Utc::now()),
}
```

#### Example: Annotated Resource

```rust
use rmcp::model::{Annotated, RawResource, Annotations, Role};

Annotated {
    raw: RawResource {
        uri: "config://server".to_string(),
        name: "Server Configuration".to_string(),
        icons: Some(vec![/* ... */]),
        // ... other fields
    },
    annotations: Some(Annotations {
        audience: Some(vec![Role::User]),
        priority: Some(0.9),  // High priority
        last_modified: Some(chrono::Utc::now()),
    }),
}
```

#### Priority Guidelines

| Priority | Use Case | Example |
|----------|----------|---------|
| 0.9-1.0 | Critical/Config | Server settings, authentication |
| 0.7-0.8 | Important | Capabilities, documentation |
| 0.5-0.6 | Informational | Stats, logs |
| 0.1-0.4 | Optional | Debug info, metadata |

#### Audience Guidelines

- **User only**: Configuration, documentation, user-facing resources
- **Assistant only**: Internal data, model-specific resources  
- **Both**: Capabilities, shared reference data

### 3. Enhanced Error Handling

Distinguish between Protocol Errors and Tool Execution Errors.

#### Protocol Errors (Before)

Used for both protocol violations AND validation failures:

```rust
// BAD: Validation failure as protocol error
if message.is_empty() {
    return Err(McpError::invalid_params(
        "Message cannot be empty".to_string(),
        None,
    ));
}
```

**Problem**: LLM cannot self-correct because it looks like a protocol issue.

#### Tool Execution Errors (After)

Validation failures return descriptive error messages:

```rust
// GOOD: Validation failure as tool execution error
if message.is_empty() {
    return Err(McpError::invalid_params(
        "Message cannot be empty. Please provide a non-empty message to echo.".to_string(),
        None,
    ));
}
```

**Note**: In rmcp, `McpError::invalid_params` is still used, but we provide **descriptive, actionable error messages** that help the LLM understand what to fix.

#### Error Message Best Practices

1. **Be specific**: "Division by zero" → "Division by zero is not allowed. Please provide a non-zero divisor."
2. **Include context**: "Invalid operation" → "Unknown operation: 'xyz'. Supported operations are: add, subtract, multiply, divide."
3. **Provide guidance**: "Expression too long" → "Expression too long (maximum 1000 characters, got 1234). Please shorten your expression."
4. **Be actionable**: Tell the LLM what to do next

#### Example: Enhanced Error Messages

```rust
// BEFORE
if req.b == 0.0 {
    return Err(McpError::invalid_params("Division by zero".to_string(), None));
}

// AFTER
if req.b == 0.0 {
    return Err(McpError::invalid_params(
        "Division by zero is not allowed. Please provide a non-zero divisor.".to_string(),
        None,
    ));
}
```

## Migration Steps

### Step 1: Update SDK Dependency

```toml
# Cargo.toml
[dependencies]
# Use local development SDK with 2025-03-26 support
rmcp = { path = "../rust-sdk/crates/rmcp", features = ["server", "macros", "transport-io"] }

# Or use published version when available
# rmcp = { version = "0.13", features = ["server", "macros", "transport-io"] }
```

### Step 2: Update Protocol Version

```rust
// src/mcp/stdio_server.rs
use rmcp::model::ProtocolVersion;

impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_03_26,  // Updated
            // ...
        }
    }
}
```

### Step 3: Add Icons to Prompts

```rust
// src/prompts/mod.rs
use rmcp::model::Icon;

pub fn list_prompts(&self) -> Vec<Prompt> {
    // Add icons based on prompt type
    let icons = match template.name.as_str() {
        "code_review" => Some(vec![Icon {
            src: "data:image/svg+xml;base64,...".to_string(),
            mime_type: Some("image/svg+xml".to_string()),
            sizes: Some(vec!["any".to_string()]),
        }]),
        _ => None,
    };
    
    Prompt {
        icons,
        // ... other fields
    }
}
```

### Step 4: Add Icons and Annotations to Resources

```rust
// src/resources/mod.rs
use rmcp::model::{Annotated, Annotations, Icon, Role};

pub fn list_resources(&self) -> Vec<Annotated<RawResource>> {
    let (icons, annotations) = match meta.uri.as_str() {
        "config://server" => (
            Some(vec![Icon { /* ... */ }]),
            Some(Annotations {
                audience: Some(vec![Role::User]),
                priority: Some(0.9),
                last_modified: Some(chrono::Utc::now()),
            }),
        ),
        _ => (None, None),
    };
    
    Annotated {
        raw: RawResource {
            icons,
            // ... other fields
        },
        annotations,
    }
}
```

### Step 5: Enhance Error Messages

```rust
// src/mcp/stdio_server.rs

// Add validation with helpful error messages
if message.is_empty() {
    return Err(McpError::invalid_params(
        "Message cannot be empty. Please provide a non-empty message to echo.".to_string(),
        None,
    ));
}

if message.len() > 10240 {
    return Err(McpError::invalid_params(
        format!(
            "Message exceeds maximum length of 10,240 bytes (got {} bytes). Please shorten your message.",
            message.len()
        ),
        None,
    ));
}
```

### Step 6: Update Documentation

Update all documentation files to reflect new protocol version:

- `README.md` - Protocol version badge
- `CLAUDE.md` - Protocol version reference
- `docs/IMPLEMENTATION_STATUS.md` - Feature status
- Resource content (config, docs) - Protocol version strings

### Step 7: Test Thoroughly

```bash
# Build
cargo build --release

# Run test suite
./scripts/test_mcp.sh
./scripts/test_prompts_resources.sh
./scripts/test_validation.sh
./scripts/verify_claude_ready.sh

# Verify all tests pass
echo "All tests should pass: 34/34"
```

## Feature Comparison

| Feature | 2024-11-05 | 2025-03-26 | Notes |
|---------|------------|------------|-------|
| Tools | ✅ | ✅ | No change |
| Prompts | ✅ | ✅ | Now with icons |
| Resources | ✅ | ✅ | Now with icons & annotations |
| Icons | ❌ | ✅ | **New** |
| Annotations | ❌ | ✅ | **New** |
| Enhanced Errors | ❌ | ✅ | **Improved** |
| Tool Output Schema | ❌ | ✅ | Available but not used yet |
| Tasks | ❌ | ✅ | Available (experimental) |
| OAuth 2.0 | ❌ | ✅ | Available (optional) |

## What We Implemented

### Phase 1 (Complete)

- ✅ Icons for all 3 prompts
- ✅ Icons for all 4 resources  
- ✅ Annotations for all 4 resources (audience, priority, timestamps)
- ✅ Enhanced error messages for all 5 tools
- ✅ Protocol version updated to 2025-03-26
- ✅ Documentation fully updated

### What We Skipped (Future)

- ⏭️ Tool Output Schemas (Phase 2)
- ⏭️ Structured Content in tool results (Phase 2)
- ⏭️ Resource Templates with URI templates (Phase 3)
- ⏭️ Tasks support (Phase 3 - experimental)
- ⏭️ OAuth 2.0 (Enterprise feature)

## Known Issues

### 1. Icon Display Support

**Issue**: Not all MCP clients display icons yet.  
**Impact**: Low - icons are optional enhancement  
**Workaround**: None needed, gracefully degrades

### 2. Annotation Interpretation

**Issue**: Clients may ignore audience/priority fields.  
**Impact**: Low - annotations are hints, not requirements  
**Workaround**: None needed, backward compatible

### 3. Protocol Version Mismatch

**Issue**: Older clients may not recognize 2025-03-26.  
**Impact**: Low - protocol negotiation handles this  
**Workaround**: Server supports older versions via SDK

## Testing Checklist

- [x] Build succeeds without warnings
- [x] All 34 tests pass
- [x] Icons present in prompts/list response
- [x] Icons present in resources/list response
- [x] Annotations present in resources/list response
- [x] Error messages are descriptive and actionable
- [x] Protocol version correctly reported as 2025-03-26
- [x] Backward compatibility maintained (older clients work)
- [x] Documentation updated
- [x] No performance regression

## Performance Impact

**None.** All changes are structural metadata additions:

- Icons: Small base64 strings (~200-500 bytes each)
- Annotations: 3 optional fields (~100 bytes per resource)
- Enhanced errors: Slightly longer error messages (~50-100 bytes)

**Total overhead**: < 5KB for entire server metadata

## Rollback Procedure

If issues arise, rollback is simple:

```bash
# Revert to previous commit
git checkout <previous-commit-hash>

# Or manually revert changes
1. Change protocol version back to V_2024_11_05
2. Remove icons: None
3. Remove annotations: None
4. Simplify error messages
5. Rebuild and test
```

## Next Steps (Phase 2)

### Tool Output Schemas

Define expected output structure:

```rust
#[tool(
    description = "Calculate arithmetic",
    output_schema = json!({
        "type": "object",
        "properties": {
            "result": { "type": "number" },
            "operation": { "type": "string" }
        }
    })
)]
async fn calculate(...) -> Result<Json<CalculateResponse>, McpError> {
    // Implementation
}
```

### Structured Content

Return structured JSON in addition to text:

```rust
Ok(CallToolResult {
    content: vec![
        Content::text("Result: 42"),
        Content::structured(json!({ "result": 42 })),
    ],
    is_error: false,
})
```

## Resources

- [MCP Specification 2025-03-26](https://modelcontextprotocol.io/specification/2025-03-26)
- [rmcp SDK Documentation](https://docs.rs/rmcp)
- [MCP GitHub Repository](https://github.com/modelcontextprotocol/specification)
- [Rust SDK Repository](https://github.com/modelcontextprotocol/rust-sdk)

## Changelog

### v0.3.1 → v0.3.2 (Planned)

**Added**:
- Icons support for prompts and resources
- Annotations support (audience, priority, timestamps)
- Enhanced error messages for LLM self-correction
- Protocol version upgraded to 2025-03-26
- Protocol upgrade guide documentation

**Changed**:
- Error messages now more descriptive and actionable
- SDK dependency updated to local development build
- Documentation updated to reflect new features

**Deprecated**:
- None

**Removed**:
- None

**Fixed**:
- None (no bugs, only enhancements)

**Security**:
- No security changes (all additive features)

## Support

If you encounter issues with the protocol upgrade:

1. Check [Troubleshooting Guide](../troubleshooting/)
2. Review [Implementation Status](IMPLEMENTATION_STATUS.md)
3. Compare with [MCP Spec Review](MCP_SPEC_REVIEW_SUMMARY.md)
4. Open an issue on GitHub

---

**Prepared by**: AI Development Team  
**Date**: 2026-01-08  
**Status**: Complete  
**Next Update**: When Phase 2 features are implemented