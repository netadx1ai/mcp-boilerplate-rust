//! Integration Tests for MCP Modules
//!
//! This module provides integration tests that verify the interaction
//! between elicitation, sampling, and structured content modules.

#[cfg(test)]
mod tests {
    use crate::mcp::elicitation::{
        ElicitationAction, ElicitationManager, ElicitationMode, ElicitationRequest,
        ElicitationResponse, ElicitationStatus,
    };
    use crate::mcp::sampling::{
        SamplingRequest, SamplingResponse, SamplingSession, SamplingTool, ToolCall,
        ToolCallResult, ToolChoice, ToolExecutorRegistry,
    };
    use crate::mcp::structured_content::{
        OutputSchemaRegistry, OutputSchemas, OutputValidator, StructuredOutput, ValidationError,
    };
    use rmcp::model::{Content, Role, SamplingMessage};
    use serde_json::json;

    // =========================================================================
    // Elicitation Integration Tests
    // =========================================================================

    #[tokio::test]
    async fn test_elicitation_form_workflow() {
        let manager = ElicitationManager::new();

        // Create a form elicitation request
        let request = ElicitationRequest::form("Please provide your contact information")
            .with_string_field("name", "Your full name", true)
            .with_email_field("email", "Contact email", true)
            .with_boolean_field("subscribe", "Subscribe to newsletter", false, Some(false))
            .with_timeout(30000)
            .build();

        assert_eq!(request.mode, ElicitationMode::Form);
        assert!(request.requested_schema.is_some());

        // Create pending elicitation
        let elicitation_id = manager.create(request).await;
        assert!(!elicitation_id.is_empty());

        // Verify it's pending
        let pending = manager.get(&elicitation_id).await;
        assert!(pending.is_some());
        assert_eq!(pending.unwrap().status, ElicitationStatus::Pending);

        // List pending should include our elicitation
        let pending_list = manager.list_pending().await;
        assert_eq!(pending_list.len(), 1);

        // Complete with response
        let response = ElicitationResponse::accept(json!({
            "name": "John Doe",
            "email": "john@example.com",
            "subscribe": true
        }));

        let completed = manager.complete(&elicitation_id, response).await;
        assert!(completed.is_some());
        assert_eq!(completed.unwrap().status, ElicitationStatus::Completed);

        // Should no longer be in pending list
        let pending_list = manager.list_pending().await;
        assert_eq!(pending_list.len(), 0);
    }

    #[tokio::test]
    async fn test_elicitation_url_workflow() {
        let manager = ElicitationManager::new();

        // Create URL elicitation for OAuth
        let request = ElicitationRequest::url_with_callback(
            "Please authenticate with your provider",
            "https://auth.example.com/oauth/authorize?client_id=abc&state=xyz",
            "https://api.example.com/oauth/callback",
        )
        .with_timeout(120000);

        assert_eq!(request.mode, ElicitationMode::Url);
        assert!(request.url.is_some());
        assert!(request.callback_url.is_some());

        let id = manager.create(request).await;

        // Simulate URL completion
        let response = ElicitationResponse::url_completed();
        assert!(response.is_accepted());

        manager.complete(&id, response).await;
    }

    #[tokio::test]
    async fn test_elicitation_cancel() {
        let manager = ElicitationManager::new();

        let request = ElicitationRequest::form("Test form")
            .with_string_field("field", "A field", true)
            .build();

        let id = manager.create(request).await;

        // Cancel the elicitation
        let cancelled = manager.cancel(&id).await;
        assert!(cancelled.is_some());
        assert_eq!(cancelled.unwrap().status, ElicitationStatus::Cancelled);

        // Should not be retrievable after cancel
        let not_found = manager.get(&id).await;
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_elicitation_with_enums() {
        let request = ElicitationRequest::form("Configure your preferences")
            .with_enum_field(
                "priority",
                vec!["low".into(), "medium".into(), "high".into()],
                true,
            )
            .with_titled_enum_field(
                "region",
                vec!["us-east".into(), "us-west".into(), "eu-central".into()],
                vec![
                    "US East (Virginia)".into(),
                    "US West (Oregon)".into(),
                    "EU Central (Frankfurt)".into(),
                ],
                true,
            )
            .with_multiselect_enum_field(
                "features",
                vec![
                    "analytics".into(),
                    "notifications".into(),
                    "api_access".into(),
                ],
                false,
                Some(1),
                Some(3),
            )
            .build();

        assert!(request.requested_schema.is_some());
        let schema = request.requested_schema.unwrap();
        assert!(!schema.properties.is_empty());
    }

    // =========================================================================
    // Sampling Integration Tests
    // =========================================================================

    #[test]
    fn test_sampling_with_tools_workflow() {
        // Define tools for weather assistant
        let weather_tool = SamplingTool::new(
            "get_weather",
            json!({
                "type": "object",
                "properties": {
                    "location": { "type": "string", "description": "City name" },
                    "units": { "type": "string", "enum": ["celsius", "fahrenheit"] }
                },
                "required": ["location"]
            }),
        )
        .with_description("Get current weather for a location");

        let forecast_tool = SamplingTool::new(
            "get_forecast",
            json!({
                "type": "object",
                "properties": {
                    "location": { "type": "string" },
                    "days": { "type": "integer", "minimum": 1, "maximum": 7 }
                },
                "required": ["location", "days"]
            }),
        )
        .with_description("Get weather forecast");

        // Build sampling request
        let request = SamplingRequest::new("You are a helpful weather assistant")
            .add_user_message("What's the weather like in Tokyo?")
            .with_tools(vec![weather_tool, forecast_tool])
            .with_tool_choice(ToolChoice::Auto)
            .with_max_tokens(1000)
            .with_temperature(0.7)
            .prefer_speed(0.8)
            .build();

        assert!(request.tools.is_some());
        assert_eq!(request.tools.as_ref().unwrap().len(), 2);
        assert_eq!(request.tool_choice, Some(ToolChoice::Auto));
        assert_eq!(request.max_tokens, 1000);

        // Convert to base param (for rmcp compatibility)
        let base_param = request.to_base_param();
        assert!(base_param.metadata.is_some());
    }

    #[test]
    fn test_sampling_session_multi_turn() {
        let mut session = SamplingSession::new("You are a coding assistant");

        // Add weather tool
        session.add_tool(SamplingTool::new(
            "run_code",
            json!({
                "type": "object",
                "properties": {
                    "language": { "type": "string" },
                    "code": { "type": "string" }
                }
            }),
        ));

        // Turn 1: User asks question
        session.add_user_message("Can you help me write a Python function to calculate fibonacci?");

        // Build request for turn 1
        let request1 = session.build_request();
        assert_eq!(request1.messages.len(), 1);

        // Simulate assistant response
        let response1 = SamplingResponse {
            model: "claude-3".to_string(),
            stop_reason: Some("endTurn".to_string()),
            message: SamplingMessage {
                role: Role::Assistant,
                content: Content::text("Here's a fibonacci function..."),
            },
            tool_calls: None,
        };

        session.process_response(&response1);
        assert_eq!(session.messages().len(), 2);

        // Turn 2: User follow-up
        session.add_user_message("Can you make it recursive?");

        let request2 = session.build_request();
        assert_eq!(request2.messages.len(), 3);
    }

    #[test]
    fn test_tool_executor_registry() {
        let mut registry = ToolExecutorRegistry::new();

        // Register echo tool
        registry.register("echo", |args| {
            let text = args
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("(empty)");
            ToolCallResult::success("", format!("Echo: {}", text))
        });

        // Register add tool
        registry.register("add", |args| {
            let a = args.get("a").and_then(|v| v.as_i64()).unwrap_or(0);
            let b = args.get("b").and_then(|v| v.as_i64()).unwrap_or(0);
            ToolCallResult::success("", format!("{}", a + b))
        });

        assert!(registry.has_tool("echo"));
        assert!(registry.has_tool("add"));
        assert!(!registry.has_tool("unknown"));

        // Execute echo
        let echo_call = ToolCall {
            id: "call_1".to_string(),
            name: "echo".to_string(),
            arguments: json!({"text": "Hello, World!"}),
        };
        let echo_result = registry.execute(&echo_call);
        assert_eq!(echo_result.is_error, Some(false));

        // Execute add
        let add_call = ToolCall {
            id: "call_2".to_string(),
            name: "add".to_string(),
            arguments: json!({"a": 5, "b": 3}),
        };
        let add_result = registry.execute(&add_call);
        assert_eq!(add_result.is_error, Some(false));

        // Execute unknown tool
        let unknown_call = ToolCall {
            id: "call_3".to_string(),
            name: "unknown".to_string(),
            arguments: json!({}),
        };
        let unknown_result = registry.execute(&unknown_call);
        assert_eq!(unknown_result.is_error, Some(true));
    }

    // =========================================================================
    // Structured Content Integration Tests
    // =========================================================================

    #[test]
    fn test_structured_output_with_validation() {
        let schema = json!({
            "type": "object",
            "properties": {
                "temperature": { "type": "number" },
                "unit": { "type": "string", "enum": ["celsius", "fahrenheit"] },
                "description": { "type": "string" }
            },
            "required": ["temperature", "unit"]
        });

        let validator = OutputValidator::new(schema);

        // Valid output
        let valid_output = StructuredOutput::new()
            .text("The temperature is 22.5°C - partly cloudy")
            .structured(json!({
                "temperature": 22.5,
                "unit": "celsius",
                "description": "Partly cloudy"
            }))
            .build_validated(&validator);

        assert!(valid_output.is_ok());
        let result = valid_output.unwrap();
        assert!(result.structured_content.is_some());

        // Invalid output (wrong type)
        let invalid_output = StructuredOutput::new()
            .structured(json!({
                "temperature": "hot", // Should be number
                "unit": "celsius"
            }))
            .build_validated(&validator);

        assert!(invalid_output.is_err());
    }

    #[test]
    fn test_output_schema_registry_workflow() {
        let mut registry = OutputSchemaRegistry::new();

        // Register schemas for different tools
        registry.register("get_weather", OutputSchemas::weather());
        registry.register(
            "list_items",
            json!({
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "integer" },
                        "name": { "type": "string" }
                    },
                    "required": ["id", "name"]
                }
            }),
        );

        assert!(registry.has_schema("get_weather"));
        assert!(registry.has_schema("list_items"));

        // Validate weather output
        let weather_output = json!({
            "temperature": 25.0,
            "unit": "celsius",
            "humidity": 65
        });
        assert!(registry.validate("get_weather", &weather_output).is_ok());

        // Validate list output
        let list_output = json!([
            {"id": 1, "name": "Item 1"},
            {"id": 2, "name": "Item 2"}
        ]);
        assert!(registry.validate("list_items", &list_output).is_ok());

        // Invalid list output (missing required field)
        let invalid_list = json!([
            {"id": 1}, // Missing "name"
            {"id": 2, "name": "Item 2"}
        ]);
        assert!(registry.validate("list_items", &invalid_list).is_err());
    }

    #[test]
    fn test_complex_nested_validation() {
        let schema = json!({
            "type": "object",
            "properties": {
                "user": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "minLength": 1 },
                        "age": { "type": "integer", "minimum": 0, "maximum": 150 },
                        "email": { "type": "string" }
                    },
                    "required": ["name", "email"]
                },
                "preferences": {
                    "type": "object",
                    "properties": {
                        "theme": { "type": "string", "enum": ["light", "dark", "system"] },
                        "notifications": { "type": "boolean" }
                    }
                },
                "tags": {
                    "type": "array",
                    "items": { "type": "string" },
                    "minItems": 0,
                    "maxItems": 10
                }
            },
            "required": ["user"]
        });

        let validator = OutputValidator::new(schema);

        // Valid complex object
        let valid = json!({
            "user": {
                "name": "Alice",
                "age": 30,
                "email": "alice@example.com"
            },
            "preferences": {
                "theme": "dark",
                "notifications": true
            },
            "tags": ["developer", "rust"]
        });
        assert!(validator.validate(&valid).is_ok());

        // Invalid: missing required nested field
        let missing_email = json!({
            "user": {
                "name": "Bob",
                "age": 25
            }
        });
        let result = validator.validate(&missing_email);
        assert!(result.is_err());

        // Invalid: age out of range
        let invalid_age = json!({
            "user": {
                "name": "Charlie",
                "age": 200,
                "email": "charlie@example.com"
            }
        });
        let result = validator.validate(&invalid_age);
        assert!(result.is_err());

        // Invalid: wrong enum value
        let invalid_theme = json!({
            "user": {
                "name": "Dave",
                "email": "dave@example.com"
            },
            "preferences": {
                "theme": "purple"
            }
        });
        let result = validator.validate(&invalid_theme);
        assert!(result.is_err());
    }

    // =========================================================================
    // Cross-Module Integration Tests
    // =========================================================================

    #[tokio::test]
    async fn test_elicitation_to_sampling_workflow() {
        // Scenario: User provides preferences via elicitation, then uses sampling

        let manager = ElicitationManager::new();

        // Step 1: Collect user preferences via elicitation
        let elicit_request = ElicitationRequest::form("Configure AI assistant")
            .with_enum_field(
                "model_preference",
                vec!["fast".into(), "balanced".into(), "powerful".into()],
                true,
            )
            .with_integer_field("max_tokens", "Maximum response length", false, Some(100), Some(4000))
            .build();

        let elicit_id = manager.create(elicit_request).await;

        // Simulate user response
        let user_prefs = ElicitationResponse::accept(json!({
            "model_preference": "balanced",
            "max_tokens": 2000
        }));

        assert!(user_prefs.is_accepted());
        let model_pref = user_prefs.get_string("model_preference");
        let max_tokens = user_prefs.get_integer("max_tokens");

        assert_eq!(model_pref, Some("balanced".to_string()));
        assert_eq!(max_tokens, Some(2000));

        manager.complete(&elicit_id, user_prefs).await;

        // Step 2: Use preferences in sampling request
        let mut request_builder = SamplingRequest::new("You are a helpful assistant")
            .add_user_message("Explain quantum computing")
            .with_max_tokens(max_tokens.unwrap_or(1000) as u32);

        // Apply model preference
        request_builder = match model_pref.as_deref() {
            Some("fast") => request_builder.prefer_speed(0.9),
            Some("powerful") => request_builder.prefer_intelligence(0.9),
            _ => request_builder.prefer_speed(0.5).prefer_intelligence(0.5),
        };

        let sampling_request = request_builder.build();

        assert_eq!(sampling_request.max_tokens, 2000);
        assert!(sampling_request.model_preferences.is_some());
    }

    #[test]
    fn test_sampling_with_structured_output() {
        // Scenario: Sampling response is validated against output schema

        let mut registry = OutputSchemaRegistry::new();
        registry.register(
            "analyze_sentiment",
            json!({
                "type": "object",
                "properties": {
                    "sentiment": { "type": "string", "enum": ["positive", "negative", "neutral"] },
                    "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
                    "keywords": {
                        "type": "array",
                        "items": { "type": "string" }
                    }
                },
                "required": ["sentiment", "confidence"]
            }),
        );

        // Simulate tool execution within sampling
        let mut executor = ToolExecutorRegistry::new();
        executor.register("analyze_sentiment", |args| {
            let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");

            // Simulate sentiment analysis
            let sentiment = if text.contains("great") || text.contains("love") {
                "positive"
            } else if text.contains("bad") || text.contains("hate") {
                "negative"
            } else {
                "neutral"
            };

            let result = json!({
                "sentiment": sentiment,
                "confidence": 0.85,
                "keywords": ["test", "analysis"]
            });

            ToolCallResult::success("", serde_json::to_string(&result).unwrap())
        });

        // Execute tool
        let call = ToolCall {
            id: "call_1".to_string(),
            name: "analyze_sentiment".to_string(),
            arguments: json!({"text": "I love this product, it's great!"}),
        };

        let result = executor.execute(&call);
        assert_eq!(result.is_error, Some(false));

        // Parse and validate the structured output
        if let Some(content) = result.content.first() {
            if let Some(text) = content.raw.as_text() {
                let parsed: serde_json::Value = serde_json::from_str(&text.text).unwrap();
                assert!(registry.validate("analyze_sentiment", &parsed).is_ok());
                assert_eq!(parsed["sentiment"], "positive");
            }
        }
    }

    #[test]
    fn test_error_handling_across_modules() {
        // Test graceful error handling

        // Elicitation: declined response
        let declined = ElicitationResponse::decline();
        assert!(declined.is_declined());
        assert!(declined.content.is_none());

        // Elicitation: cancelled response
        let cancelled = ElicitationResponse::cancel();
        assert!(cancelled.is_cancelled());

        // Sampling: tool error
        let tool_error = ToolCallResult::error("call_1", "Tool execution failed: API unavailable");
        assert_eq!(tool_error.is_error, Some(true));

        // Structured: validation errors
        let validator = OutputValidator::new(json!({
            "type": "object",
            "properties": {
                "required_field": { "type": "string" }
            },
            "required": ["required_field"]
        }));

        let result = validator.validate(&json!({}));
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("required"));
    }

    #[test]
    fn test_tool_choice_variations() {
        // Test all tool choice modes

        let tool = SamplingTool::new("test_tool", json!({}));

        // Auto mode
        let auto_request = SamplingRequest::new("Test")
            .with_tools(vec![tool.clone()])
            .with_tool_choice(ToolChoice::Auto)
            .build();
        assert_eq!(
            auto_request.tool_choice.unwrap().to_json(),
            json!({"type": "auto"})
        );

        // None mode
        let none_request = SamplingRequest::new("Test")
            .with_tools(vec![tool.clone()])
            .with_tool_choice(ToolChoice::None)
            .build();
        assert_eq!(
            none_request.tool_choice.unwrap().to_json(),
            json!({"type": "none"})
        );

        // Required mode
        let required_request = SamplingRequest::new("Test")
            .with_tools(vec![tool.clone()])
            .with_tool_choice(ToolChoice::Required)
            .build();
        assert_eq!(
            required_request.tool_choice.unwrap().to_json(),
            json!({"type": "required"})
        );

        // Specific tool
        let specific_request = SamplingRequest::new("Test")
            .with_tools(vec![tool])
            .with_tool_choice(ToolChoice::Tool("test_tool".to_string()))
            .build();
        assert_eq!(
            specific_request.tool_choice.unwrap().to_json(),
            json!({"type": "tool", "name": "test_tool"})
        );
    }
}