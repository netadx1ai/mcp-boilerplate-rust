# Sampling

LLM completion with tool calling support.

**Version:** 0.6.3  
**MCP Spec:** 2025-11-25

---

## Overview

Sampling allows servers to request LLM completions from the client, with support for:

- Tool definitions and execution
- Tool choice modes (auto, none, required, specific)
- Multi-turn conversation sessions
- Model preferences (cost, speed, intelligence)

---

## Sampling Requests

### Basic Request

```rust
use crate::mcp::sampling::{SamplingRequest, SamplingTool, ToolChoice};

let request = SamplingRequest::new("You are a helpful assistant")
    .add_user_message("What is the capital of France?")
    .with_max_tokens(1000)
    .with_temperature(0.7)
    .build();
```

### With Tools

```rust
let weather_tool = SamplingTool::new(
    "get_weather",
    json!({
        "type": "object",
        "properties": {
            "location": { "type": "string" },
            "units": { "type": "string", "enum": ["celsius", "fahrenheit"] }
        },
        "required": ["location"]
    }),
).with_description("Get current weather for a location");

let request = SamplingRequest::new("You are a weather assistant")
    .add_user_message("What's the weather in Tokyo?")
    .with_tools(vec![weather_tool])
    .with_tool_choice(ToolChoice::Auto)
    .build();
```

---

## Tool Choice

Control how the model uses tools.

```rust
use crate::mcp::sampling::ToolChoice;

// Let model decide
ToolChoice::Auto

// Don't use any tools
ToolChoice::None

// Force tool use
ToolChoice::Required

// Use specific tool
ToolChoice::Tool("get_weather".to_string())
```

### JSON Serialization

```rust
ToolChoice::Auto.to_json()           // {"type": "auto"}
ToolChoice::None.to_json()           // {"type": "none"}
ToolChoice::Required.to_json()       // {"type": "required"}
ToolChoice::Tool("x".into()).to_json() // {"type": "tool", "name": "x"}
```

---

## Model Preferences

Guide model selection.

```rust
let request = SamplingRequest::new("System prompt")
    .add_user_message("Hello")
    .prefer_cost(0.8)        // Prefer cheaper models
    .prefer_speed(0.5)       // Balance speed
    .prefer_intelligence(0.9) // Prefer capable models
    .build();
```

Values are 0.0 to 1.0, where higher means stronger preference.

---

## Tool Executor Registry

Register and execute tool handlers.

### Registration

```rust
use crate::mcp::sampling::{ToolExecutorRegistry, ToolCall, ToolCallResult};

let mut registry = ToolExecutorRegistry::new();

registry.register("get_weather", |args| {
    let location = args.get("location")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    
    // Call actual weather API...
    let result = json!({
        "temperature": 22.5,
        "unit": "celsius",
        "condition": "Sunny"
    });
    
    ToolCallResult::success("call_id", serde_json::to_string(&result).unwrap())
});

registry.register("web_search", |args| {
    let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
    // Perform search...
    ToolCallResult::success("call_id", format!("Results for: {}", query))
});
```

### Execution

```rust
let call = ToolCall {
    id: "call_123".to_string(),
    name: "get_weather".to_string(),
    arguments: json!({"location": "Tokyo", "units": "celsius"}),
};

let result = registry.execute(&call);

if result.is_error == Some(false) {
    // Success
}
```

### Registry Methods

```rust
registry.has_tool("get_weather")  // Check if registered
registry.tool_names()             // List all tools
```

---

## Tool Results

### Success

```rust
ToolCallResult::success("call_id", "Result text")

ToolCallResult::with_content("call_id", vec![
    Content::text("Text result"),
    Content::image(base64_data, "image/png"),
])
```

### Error

```rust
ToolCallResult::error("call_id", "Error message")
```

---

## Multi-turn Sessions

Manage conversation state across multiple turns.

### Basic Session

```rust
use crate::mcp::sampling::SamplingSession;

let mut session = SamplingSession::new("You are a helpful assistant");

// Add tools
session.add_tool(weather_tool);
session.set_tool_choice(ToolChoice::Auto);

// Turn 1
session.add_user_message("What's the weather in Tokyo?");
let request1 = session.build_request();

// Process response
session.process_response(&response1);

// Turn 2
session.add_user_message("And what about tomorrow?");
let request2 = session.build_request();
```

### Session Methods

```rust
// Configuration
session.add_tool(tool);
session.set_tools(vec![tool1, tool2]);
session.set_tool_choice(ToolChoice::Auto);
session.set_max_tokens(2000);
session.set_temperature(0.7);

// Messages
session.add_user_message("Hello");
session.add_assistant_message("Hi there!");
session.add_message(custom_message);
session.add_tool_result(tool_result);

// State
session.messages();       // Get history
session.clear_messages(); // Reset

// Build
let request = session.build_request();
```

---

## Response Handling

```rust
use crate::mcp::sampling::SamplingResponse;

let response: SamplingResponse = // from client...

// Check for tool calls
if response.has_tool_calls() {
    for call in response.tool_calls.unwrap() {
        let result = executor.execute(&call);
        session.add_tool_result(result);
    }
}

// Get text content
if let Some(text) = response.get_text() {
    println!("Response: {}", text);
}

// Check stop reason
if response.is_end_turn() {
    // Natural completion
}
if response.is_max_tokens() {
    // Hit token limit
}
```

---

## Example: Weather Assistant

```rust
async fn weather_assistant() {
    // Set up tools
    let mut executor = ToolExecutorRegistry::new();
    executor.register("get_weather", |args| {
        let location = args.get("location").and_then(|v| v.as_str()).unwrap();
        // Call weather API...
        ToolCallResult::success("", format!("22°C in {}", location))
    });

    // Create session
    let mut session = SamplingSession::new("You are a weather assistant");
    session.add_tool(SamplingTool::new("get_weather", json!({
        "type": "object",
        "properties": { "location": { "type": "string" } },
        "required": ["location"]
    })));

    // User query
    session.add_user_message("What's the weather in Paris and London?");
    
    // Build and send request
    let request = session.build_request();
    
    // Process response (would come from client)
    // If model requests tool calls, execute them
    // Continue conversation as needed
}
```

---

## API Reference

### SamplingRequest Builder

| Method | Description |
|--------|-------------|
| `new(system_prompt)` | Create with system prompt |
| `without_system()` | Create without system prompt |
| `add_user_message(text)` | Add user message |
| `add_assistant_message(text)` | Add assistant message |
| `add_message(message)` | Add custom message |
| `with_tools(tools)` | Set available tools |
| `add_tool(tool)` | Add single tool |
| `with_tool_choice(choice)` | Set tool choice mode |
| `with_max_tokens(n)` | Set max tokens |
| `with_temperature(t)` | Set temperature (0.0-1.0) |
| `with_stop_sequences(seqs)` | Set stop sequences |
| `prefer_cost(priority)` | Prefer cheaper models |
| `prefer_speed(priority)` | Prefer faster models |
| `prefer_intelligence(priority)` | Prefer capable models |
| `with_metadata(value)` | Set custom metadata |
| `build()` | Build the request |

### SamplingTool

| Method | Description |
|--------|-------------|
| `new(name, schema)` | Create tool with input schema |
| `with_description(desc)` | Add description |

### ToolExecutorRegistry

| Method | Description |
|--------|-------------|
| `new()` | Create empty registry |
| `register(name, handler)` | Register tool handler |
| `execute(call)` | Execute tool call |
| `has_tool(name)` | Check if tool exists |
| `tool_names()` | List registered tools |

### SamplingSession

| Method | Description |
|--------|-------------|
| `new(system_prompt)` | Create session |
| `without_system()` | Create without system |
| `add_tool(tool)` | Add tool |
| `set_tools(tools)` | Set all tools |
| `set_tool_choice(choice)` | Set tool choice |
| `add_user_message(text)` | Add user message |
| `add_assistant_message(text)` | Add assistant message |
| `add_tool_result(result)` | Add tool result |
| `process_response(response)` | Process model response |
| `build_request()` | Build current request |
| `messages()` | Get message history |
| `clear_messages()` | Clear history |

---

## References

- [MCP Sampling Spec](https://modelcontextprotocol.io/specification/2025-11-25/client/sampling)
- Source: `src/mcp/sampling.rs`
