# Elicitation

User input collection for MCP servers.

**Version:** 0.6.3  
**MCP Spec:** 2025-11-25

---

## Overview

Elicitation allows servers to request interactive input from users through the client. Two modes are supported:

- **Form Mode**: Structured forms with JSON Schema validation
- **URL Mode**: External URLs for sensitive data (OAuth, payments)

---

## Form Mode

Collect structured user input with validation.

### Basic Usage

```rust
use crate::mcp::elicitation::{ElicitationRequest, ElicitationResponse};

let request = ElicitationRequest::form("Please provide your details")
    .with_string_field("name", "Your full name", true)
    .with_email_field("email", "Contact email", true)
    .with_timeout(60000)
    .build();
```

### Field Types

#### String Fields

```rust
// Required string
.with_string_field("name", "Your name", true)

// Optional string
.with_string_field("nickname", "Optional nickname", false)

// Email field (with format validation)
.with_email_field("email", "Contact email", true)
```

#### Number Fields

```rust
// Integer with range
.with_integer_field("age", "Your age", true, Some(18), Some(120))

// Number with range
.with_number_field("rating", "Rating", false, Some(0.0), Some(5.0))
```

#### Boolean Fields

```rust
// With default value
.with_boolean_field("subscribe", "Subscribe to newsletter", false, Some(false))
```

#### Enum Fields

```rust
// Single select (untitled)
.with_enum_field(
    "theme",
    vec!["light".into(), "dark".into(), "system".into()],
    true
)

// Single select with titles
.with_titled_enum_field(
    "region",
    vec!["us-east".into(), "us-west".into(), "eu".into()],
    vec!["US East".into(), "US West".into(), "Europe".into()],
    true
)

// Multi-select with constraints
.with_multiselect_enum_field(
    "interests",
    vec!["tech".into(), "science".into(), "art".into()],
    false,
    Some(1),  // min items
    Some(3)   // max items
)
```

---

## URL Mode

Direct users to external URLs for sensitive data collection.

### Basic Usage

```rust
let request = ElicitationRequest::url(
    "Please authenticate",
    "https://auth.example.com/login"
);
```

### With Callback

```rust
let request = ElicitationRequest::url_with_callback(
    "Complete authentication",
    "https://github.com/login/oauth/authorize?client_id=xxx",
    "https://api.example.com/oauth/callback"
).with_timeout(120000);
```

---

## Response Handling

### Accept

```rust
let response = ElicitationResponse::accept(json!({
    "name": "John Doe",
    "email": "john@example.com",
    "theme": "dark"
}));

if response.is_accepted() {
    let name = response.get_string("name");
    let age = response.get_integer("age");
    let subscribe = response.get_bool("subscribe");
}
```

### Decline

```rust
let response = ElicitationResponse::decline();
assert!(response.is_declined());
```

### Cancel

```rust
let response = ElicitationResponse::cancel();
assert!(response.is_cancelled());
```

### URL Completion

```rust
let response = ElicitationResponse::url_completed();
```

---

## Elicitation Manager

Track pending elicitations.

```rust
use crate::mcp::elicitation::ElicitationManager;

let manager = ElicitationManager::new();

// Create elicitation
let id = manager.create(request).await;

// Check status
let pending = manager.get(&id).await;

// List all pending
let list = manager.list_pending().await;

// Complete
manager.complete(&id, response).await;

// Cancel
manager.cancel(&id).await;
```

---

## Example: User Registration

```rust
async fn collect_user_info() -> Option<UserInfo> {
    let manager = ElicitationManager::new();
    
    let request = ElicitationRequest::form("Create your account")
        .with_string_field("username", "Choose a username", true)
        .with_email_field("email", "Your email address", true)
        .with_integer_field("age", "Your age (optional)", false, Some(13), Some(120))
        .with_enum_field(
            "plan",
            vec!["free".into(), "pro".into(), "enterprise".into()],
            true
        )
        .with_boolean_field("terms", "Accept terms of service", true, None)
        .with_timeout(300000)
        .build();
    
    let id = manager.create(request).await;
    
    // In real implementation, send to client and wait for response
    // ...
    
    None
}
```

---

## API Reference

### ElicitationRequest

| Method | Description |
|--------|-------------|
| `form(message)` | Create form-mode request builder |
| `url(message, url)` | Create URL-mode request |
| `url_with_callback(message, url, callback)` | URL with callback |
| `with_timeout(ms)` | Set timeout in milliseconds |

### ElicitationFormBuilder

| Method | Description |
|--------|-------------|
| `with_string_field(name, desc, required)` | Add string field |
| `with_email_field(name, desc, required)` | Add email field |
| `with_number_field(name, desc, req, min, max)` | Add number field |
| `with_integer_field(name, desc, req, min, max)` | Add integer field |
| `with_boolean_field(name, desc, req, default)` | Add boolean field |
| `with_enum_field(name, values, required)` | Add enum field |
| `with_titled_enum_field(name, values, titles, req)` | Add titled enum |
| `with_multiselect_enum_field(name, values, req, min, max)` | Add multi-select |
| `build()` | Build the request |

### ElicitationResponse

| Method | Description |
|--------|-------------|
| `accept(content)` | Create accept response |
| `decline()` | Create decline response |
| `cancel()` | Create cancel response |
| `url_completed()` | Create URL completion response |
| `is_accepted()` | Check if accepted |
| `is_declined()` | Check if declined |
| `is_cancelled()` | Check if cancelled |
| `get_string(field)` | Get string value |
| `get_integer(field)` | Get integer value |
| `get_bool(field)` | Get boolean value |
| `get_content<T>()` | Deserialize content |

---

## References

- [MCP Elicitation Spec](https://modelcontextprotocol.io/specification/2025-11-25/client/elicitation)
- Source: `src/mcp/elicitation.rs`
