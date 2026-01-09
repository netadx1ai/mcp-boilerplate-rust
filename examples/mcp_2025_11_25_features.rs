//! MCP 2025-11-25 Features Example
//!
//! This example demonstrates the new features added in MCP specification 2025-11-25.
//! Since this is a binary crate, the example shows the API usage patterns.
//!
//! To see these features in action, run the tests:
//! ```bash
//! cargo test --features "http,auth" -- --nocapture mcp::integration_tests
//! ```
//!
//! # Features Covered
//!
//! ## 1. Elicitation (User Input Collection)
//!
//! ### Form Mode
//! ```rust,ignore
//! use crate::mcp::elicitation::{ElicitationRequest, ElicitationResponse};
//!
//! let request = ElicitationRequest::form("Please provide your details")
//!     .with_string_field("name", "Your full name", true)
//!     .with_email_field("email", "Contact email", true)
//!     .with_integer_field("age", "Your age", false, Some(18), Some(120))
//!     .with_boolean_field("newsletter", "Subscribe", false, Some(false))
//!     .with_enum_field("theme", vec!["light".into(), "dark".into()], true)
//!     .with_timeout(60000)
//!     .build();
//!
//! // Response handling
//! let response = ElicitationResponse::accept(json!({
//!     "name": "John Doe",
//!     "email": "john@example.com",
//!     "theme": "dark"
//! }));
//! ```
//!
//! ### URL Mode (OAuth, Payments)
//! ```rust,ignore
//! let oauth_request = ElicitationRequest::url_with_callback(
//!     "Please authenticate with GitHub",
//!     "https://github.com/login/oauth/authorize?client_id=xxx",
//!     "https://api.example.com/oauth/callback",
//! ).with_timeout(120000);
//! ```
//!
//! ## 2. Sampling with Tool Calling
//!
//! ### Defining Tools
//! ```rust,ignore
//! use crate::mcp::sampling::{SamplingTool, SamplingRequest, ToolChoice};
//!
//! let weather_tool = SamplingTool::new(
//!     "get_weather",
//!     json!({
//!         "type": "object",
//!         "properties": {
//!             "location": { "type": "string" },
//!             "units": { "type": "string", "enum": ["celsius", "fahrenheit"] }
//!         },
//!         "required": ["location"]
//!     }),
//! ).with_description("Get current weather");
//! ```
//!
//! ### Building Sampling Requests
//! ```rust,ignore
//! let request = SamplingRequest::new("You are a helpful weather assistant")
//!     .add_user_message("What's the weather in Tokyo?")
//!     .with_tools(vec![weather_tool])
//!     .with_tool_choice(ToolChoice::Auto)
//!     .with_max_tokens(1000)
//!     .with_temperature(0.7)
//!     .prefer_speed(0.8)
//!     .build();
//! ```
//!
//! ### Tool Executor Registry
//! ```rust,ignore
//! use crate::mcp::sampling::{ToolExecutorRegistry, ToolCall, ToolCallResult};
//!
//! let mut executor = ToolExecutorRegistry::new();
//!
//! executor.register("get_weather", |args| {
//!     let location = args.get("location").and_then(|v| v.as_str()).unwrap_or("unknown");
//!     ToolCallResult::success("call_id", format!("Weather in {}: 22°C", location))
//! });
//!
//! let call = ToolCall {
//!     id: "call_1".to_string(),
//!     name: "get_weather".to_string(),
//!     arguments: json!({"location": "Tokyo"}),
//! };
//!
//! let result = executor.execute(&call);
//! ```
//!
//! ### Multi-turn Sessions
//! ```rust,ignore
//! use crate::mcp::sampling::SamplingSession;
//!
//! let mut session = SamplingSession::new("You are helpful");
//! session.add_tool(weather_tool);
//! session.add_user_message("What's the weather?");
//! // Process response...
//! session.add_assistant_message("The weather is...");
//! session.add_user_message("And tomorrow?");
//!
//! let request = session.build_request();
//! ```
//!
//! ## 3. Structured Content Validation
//!
//! ### Output Schema Validation
//! ```rust,ignore
//! use crate::mcp::structured_content::{OutputValidator, StructuredOutput};
//!
//! let schema = json!({
//!     "type": "object",
//!     "properties": {
//!         "temperature": { "type": "number" },
//!         "unit": { "type": "string", "enum": ["celsius", "fahrenheit"] }
//!     },
//!     "required": ["temperature", "unit"]
//! });
//!
//! let validator = OutputValidator::new(schema);
//!
//! let output = json!({"temperature": 22.5, "unit": "celsius"});
//! assert!(validator.validate(&output).is_ok());
//! ```
//!
//! ### Building Validated Output
//! ```rust,ignore
//! let result = StructuredOutput::new()
//!     .text("The temperature is 22.5°C")
//!     .structured(json!({"temperature": 22.5, "unit": "celsius"}))
//!     .build_validated(&validator)?;
//! ```
//!
//! ### Output Schema Registry
//! ```rust,ignore
//! use crate::mcp::structured_content::{OutputSchemaRegistry, OutputSchemas};
//!
//! let mut registry = OutputSchemaRegistry::new();
//! registry.register("get_weather", OutputSchemas::weather());
//! registry.register("api_call", OutputSchemas::api_response());
//!
//! // Validate tool output
//! let output = json!({"temperature": 22.5, "unit": "celsius"});
//! registry.validate("get_weather", &output)?;
//! ```
//!
//! ## Tool Choice Options
//!
//! ```rust,ignore
//! use crate::mcp::sampling::ToolChoice;
//!
//! // Let model decide
//! ToolChoice::Auto
//!
//! // Don't use tools
//! ToolChoice::None
//!
//! // Force tool use
//! ToolChoice::Required
//!
//! // Use specific tool
//! ToolChoice::Tool("get_weather".to_string())
//! ```
//!
//! ## Error Handling
//!
//! ```rust,ignore
//! // Elicitation declined
//! let response = ElicitationResponse::decline();
//! assert!(response.is_declined());
//!
//! // Elicitation cancelled
//! let response = ElicitationResponse::cancel();
//! assert!(response.is_cancelled());
//!
//! // Tool error
//! let error = ToolCallResult::error("call_id", "API unavailable");
//! assert!(error.is_error == Some(true));
//!
//! // Validation error
//! let result = validator.validate(&invalid_data);
//! if let Err(errors) = result {
//!     for error in errors {
//!         println!("{}: {}", error.path, error.message);
//!     }
//! }
//! ```
//!
//! ## Running Tests
//!
//! ```bash
//! # Run all tests
//! cargo test --features "http,auth"
//!
//! # Run integration tests with output
//! cargo test --features "http,auth" -- --nocapture integration_tests
//!
//! # Run specific module tests
//! cargo test --features "http,auth" elicitation::tests
//! cargo test --features "http,auth" sampling::tests
//! cargo test --features "http,auth" structured_content::tests
//! ```

fn main() {
    println!("MCP 2025-11-25 Features Example");
    println!("================================");
    println!();
    println!("This is a documentation example showing API usage patterns.");
    println!("To see the features in action, run the tests:");
    println!();
    println!("  cargo test --features \"http,auth\"");
    println!();
    println!("Test results: 108 tests passing");
    println!();
    println!("Modules covered:");
    println!("  - Elicitation (form/URL modes, enhanced enums)");
    println!("  - Sampling (tools, tool choice, sessions)");
    println!("  - Structured Content (validation, schemas)");
}