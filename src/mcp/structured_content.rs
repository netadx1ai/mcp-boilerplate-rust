//! Structured Content Module
//!
//! This module provides structured content validation for MCP 2025-11-25 specification.
//! Tools can define output schemas, and this module validates that tool results
//! conform to those schemas.
//!
//! # Features
//!
//! - JSON Schema validation for tool outputs
//! - Structured content creation helpers
//! - Schema-aware result transformation
//!
//! # Example
//!
//! ```rust
//! use crate::mcp::structured_content::{StructuredOutput, OutputValidator};
//! use serde_json::json;
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
//! let output = json!({"temperature": 22.5, "unit": "celsius"});
//! assert!(validator.validate(&output).is_ok());
//! ```

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

// Re-export Content type from rmcp
pub use rmcp::model::{CallToolResult, Content};

/// Validation error for structured content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Path to the invalid field (e.g., "root.field.subfield")
    pub path: String,
    /// Description of the validation error
    pub message: String,
    /// Expected type or value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    /// Actual value found
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<String>,
}

impl ValidationError {
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
            expected: None,
            actual: None,
        }
    }

    pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self
    }

    pub fn with_actual(mut self, actual: impl Into<String>) -> Self {
        self.actual = Some(actual.into());
        self
    }

    pub fn type_mismatch(path: &str, expected: &str, actual: &str) -> Self {
        Self::new(path, format!("Type mismatch: expected {}, got {}", expected, actual))
            .with_expected(expected)
            .with_actual(actual)
    }

    pub fn missing_required(path: &str, field: &str) -> Self {
        Self::new(path, format!("Missing required field: {}", field))
    }

    pub fn invalid_enum(path: &str, value: &str, allowed: &[String]) -> Self {
        Self::new(
            path,
            format!(
                "Invalid enum value '{}'. Allowed values: {:?}",
                value, allowed
            ),
        )
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Result type for validation operations
pub type ValidationResult<T> = Result<T, Vec<ValidationError>>;

/// Output schema validator
///
/// Validates JSON values against a JSON Schema definition.
/// This is a simplified validator that handles common cases in MCP tool outputs.
#[derive(Debug, Clone)]
pub struct OutputValidator {
    schema: Value,
}

impl OutputValidator {
    /// Create a new validator with the given schema
    pub fn new(schema: Value) -> Self {
        Self { schema }
    }

    /// Validate a value against the schema
    pub fn validate(&self, value: &Value) -> ValidationResult<()> {
        let mut errors = Vec::new();
        self.validate_value(value, &self.schema, "root", &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate and return the value if valid
    pub fn validate_and_get(&self, value: Value) -> ValidationResult<Value> {
        self.validate(&value)?;
        Ok(value)
    }

    fn validate_value(
        &self,
        value: &Value,
        schema: &Value,
        path: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        // Check type constraint
        if let Some(type_str) = schema.get("type").and_then(|v| v.as_str()) {
            self.validate_type(value, type_str, path, errors);
        }

        // Check enum constraint
        if let Some(enum_values) = schema.get("enum").and_then(|v| v.as_array()) {
            self.validate_enum(value, enum_values, path, errors);
        }

        // Check object properties
        if schema.get("type").and_then(|v| v.as_str()) == Some("object") {
            self.validate_object(value, schema, path, errors);
        }

        // Check array items
        if schema.get("type").and_then(|v| v.as_str()) == Some("array") {
            self.validate_array(value, schema, path, errors);
        }

        // Check number constraints
        if let Some(minimum) = schema.get("minimum").and_then(|v| v.as_f64()) {
            if let Some(num) = value.as_f64() {
                if num < minimum {
                    errors.push(ValidationError::new(
                        path,
                        format!("Value {} is less than minimum {}", num, minimum),
                    ));
                }
            }
        }

        if let Some(maximum) = schema.get("maximum").and_then(|v| v.as_f64()) {
            if let Some(num) = value.as_f64() {
                if num > maximum {
                    errors.push(ValidationError::new(
                        path,
                        format!("Value {} is greater than maximum {}", num, maximum),
                    ));
                }
            }
        }

        // Check string constraints
        if let Some(min_length) = schema.get("minLength").and_then(|v| v.as_u64()) {
            if let Some(s) = value.as_str() {
                if (s.len() as u64) < min_length {
                    errors.push(ValidationError::new(
                        path,
                        format!(
                            "String length {} is less than minimum {}",
                            s.len(),
                            min_length
                        ),
                    ));
                }
            }
        }

        if let Some(max_length) = schema.get("maxLength").and_then(|v| v.as_u64()) {
            if let Some(s) = value.as_str() {
                if (s.len() as u64) > max_length {
                    errors.push(ValidationError::new(
                        path,
                        format!(
                            "String length {} is greater than maximum {}",
                            s.len(),
                            max_length
                        ),
                    ));
                }
            }
        }
    }

    fn validate_type(
        &self,
        value: &Value,
        expected_type: &str,
        path: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        let actual_type = match value {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(n) => {
                if n.is_i64() || n.is_u64() {
                    "integer"
                } else {
                    "number"
                }
            }
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        };

        // number type accepts both number and integer
        let type_matches = match expected_type {
            "number" => actual_type == "number" || actual_type == "integer",
            _ => actual_type == expected_type,
        };

        if !type_matches {
            errors.push(ValidationError::type_mismatch(path, expected_type, actual_type));
        }
    }

    fn validate_enum(
        &self,
        value: &Value,
        enum_values: &[Value],
        path: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        if !enum_values.contains(value) {
            let allowed: Vec<String> = enum_values
                .iter()
                .map(|v| v.to_string())
                .collect();
            errors.push(ValidationError::invalid_enum(
                path,
                &value.to_string(),
                &allowed,
            ));
        }
    }

    fn validate_object(
        &self,
        value: &Value,
        schema: &Value,
        path: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        let obj = match value.as_object() {
            Some(o) => o,
            None => return, // Type error already reported
        };

        // Check required properties
        if let Some(required) = schema.get("required").and_then(|v| v.as_array()) {
            for req in required {
                if let Some(req_str) = req.as_str() {
                    if !obj.contains_key(req_str) {
                        errors.push(ValidationError::missing_required(path, req_str));
                    }
                }
            }
        }

        // Validate each property against its schema
        if let Some(properties) = schema.get("properties").and_then(|v| v.as_object()) {
            for (key, prop_schema) in properties {
                if let Some(prop_value) = obj.get(key) {
                    let prop_path = format!("{}.{}", path, key);
                    self.validate_value(prop_value, prop_schema, &prop_path, errors);
                }
            }
        }

        // Check additionalProperties
        if let Some(additional) = schema.get("additionalProperties") {
            if additional.as_bool() == Some(false) {
                if let Some(properties) = schema.get("properties").and_then(|v| v.as_object()) {
                    for key in obj.keys() {
                        if !properties.contains_key(key) {
                            errors.push(ValidationError::new(
                                path,
                                format!("Additional property '{}' not allowed", key),
                            ));
                        }
                    }
                }
            }
        }
    }

    fn validate_array(
        &self,
        value: &Value,
        schema: &Value,
        path: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        let arr = match value.as_array() {
            Some(a) => a,
            None => return, // Type error already reported
        };

        // Check minItems
        if let Some(min_items) = schema.get("minItems").and_then(|v| v.as_u64()) {
            if (arr.len() as u64) < min_items {
                errors.push(ValidationError::new(
                    path,
                    format!("Array has {} items, minimum is {}", arr.len(), min_items),
                ));
            }
        }

        // Check maxItems
        if let Some(max_items) = schema.get("maxItems").and_then(|v| v.as_u64()) {
            if (arr.len() as u64) > max_items {
                errors.push(ValidationError::new(
                    path,
                    format!("Array has {} items, maximum is {}", arr.len(), max_items),
                ));
            }
        }

        // Validate items against item schema
        if let Some(items_schema) = schema.get("items") {
            for (i, item) in arr.iter().enumerate() {
                let item_path = format!("{}[{}]", path, i);
                self.validate_value(item, items_schema, &item_path, errors);
            }
        }
    }
}

/// Structured output builder
///
/// Helps build tool outputs with both human-readable content
/// and machine-parseable structured data.
#[derive(Debug)]
pub struct StructuredOutput {
    content: Vec<Content>,
    structured: Option<Value>,
    is_error: bool,
}

impl StructuredOutput {
    /// Create a new structured output builder
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
            structured: None,
            is_error: false,
        }
    }

    /// Add text content
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.content.push(Content::text(text.into()));
        self
    }

    /// Set the structured data
    pub fn structured(mut self, data: Value) -> Self {
        self.structured = Some(data);
        self
    }

    /// Set structured data from a serializable type
    pub fn structured_from<T: Serialize>(mut self, data: &T) -> Self {
        self.structured = serde_json::to_value(data).ok();
        self
    }

    /// Mark as error
    pub fn error(mut self) -> Self {
        self.is_error = true;
        self
    }

    /// Build the CallToolResult
    pub fn build(self) -> CallToolResult {
        // If we have structured data but no content, add text representation
        let content = if self.content.is_empty() && self.structured.is_some() {
            vec![Content::text(
                serde_json::to_string_pretty(self.structured.as_ref().unwrap())
                    .unwrap_or_default(),
            )]
        } else {
            self.content
        };

        CallToolResult {
            content,
            structured_content: self.structured,
            is_error: Some(self.is_error),
            meta: None,
        }
    }

    /// Build with validation against a schema
    pub fn build_validated(self, validator: &OutputValidator) -> ValidationResult<CallToolResult> {
        if let Some(ref structured) = self.structured {
            validator.validate(structured)?;
        }
        Ok(self.build())
    }
}

impl Default for StructuredOutput {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry for output schemas
///
/// Stores output schemas for tools and provides validation
pub struct OutputSchemaRegistry {
    schemas: HashMap<String, Arc<OutputValidator>>,
}

impl OutputSchemaRegistry {
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    /// Register an output schema for a tool
    pub fn register(&mut self, tool_name: impl Into<String>, schema: Value) {
        self.schemas
            .insert(tool_name.into(), Arc::new(OutputValidator::new(schema)));
    }

    /// Get the validator for a tool
    pub fn get(&self, tool_name: &str) -> Option<Arc<OutputValidator>> {
        self.schemas.get(tool_name).cloned()
    }

    /// Validate output for a tool
    pub fn validate(&self, tool_name: &str, output: &Value) -> ValidationResult<()> {
        match self.schemas.get(tool_name) {
            Some(validator) => validator.validate(output),
            None => Ok(()), // No schema = no validation
        }
    }

    /// Check if a tool has an output schema registered
    pub fn has_schema(&self, tool_name: &str) -> bool {
        self.schemas.contains_key(tool_name)
    }

    /// List all tools with registered schemas
    pub fn tools_with_schemas(&self) -> Vec<&String> {
        self.schemas.keys().collect()
    }
}

impl Default for OutputSchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create common output schemas
pub struct OutputSchemas;

impl OutputSchemas {
    /// Create a simple string output schema
    pub fn string() -> Value {
        json!({
            "type": "string"
        })
    }

    /// Create a number output schema
    pub fn number() -> Value {
        json!({
            "type": "number"
        })
    }

    /// Create an object schema with specified properties
    pub fn object(properties: Value, required: Vec<&str>) -> Value {
        json!({
            "type": "object",
            "properties": properties,
            "required": required
        })
    }

    /// Create an array schema
    pub fn array(items: Value) -> Value {
        json!({
            "type": "array",
            "items": items
        })
    }

    /// Create an enum schema
    pub fn enum_values(values: Vec<&str>) -> Value {
        json!({
            "type": "string",
            "enum": values
        })
    }

    /// Weather output schema (example)
    pub fn weather() -> Value {
        json!({
            "type": "object",
            "properties": {
                "temperature": { "type": "number" },
                "unit": { "type": "string", "enum": ["celsius", "fahrenheit", "kelvin"] },
                "description": { "type": "string" },
                "humidity": { "type": "number", "minimum": 0, "maximum": 100 }
            },
            "required": ["temperature", "unit"]
        })
    }

    /// API response schema (example)
    pub fn api_response() -> Value {
        json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "data": { "type": "object" },
                "error": {
                    "type": "object",
                    "properties": {
                        "code": { "type": "string" },
                        "message": { "type": "string" }
                    }
                }
            },
            "required": ["success"]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_string() {
        let validator = OutputValidator::new(json!({"type": "string"}));

        assert!(validator.validate(&json!("hello")).is_ok());
        assert!(validator.validate(&json!(123)).is_err());
    }

    #[test]
    fn test_validate_number() {
        let validator = OutputValidator::new(json!({
            "type": "number",
            "minimum": 0,
            "maximum": 100
        }));

        assert!(validator.validate(&json!(50)).is_ok());
        assert!(validator.validate(&json!(50.5)).is_ok());
        assert!(validator.validate(&json!(-1)).is_err());
        assert!(validator.validate(&json!(101)).is_err());
    }

    #[test]
    fn test_validate_object() {
        let validator = OutputValidator::new(json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer" }
            },
            "required": ["name"]
        }));

        assert!(validator.validate(&json!({"name": "John", "age": 30})).is_ok());
        assert!(validator.validate(&json!({"name": "John"})).is_ok());
        assert!(validator.validate(&json!({"age": 30})).is_err()); // missing required "name"
    }

    #[test]
    fn test_validate_enum() {
        let validator = OutputValidator::new(json!({
            "type": "string",
            "enum": ["red", "green", "blue"]
        }));

        assert!(validator.validate(&json!("red")).is_ok());
        assert!(validator.validate(&json!("yellow")).is_err());
    }

    #[test]
    fn test_validate_array() {
        let validator = OutputValidator::new(json!({
            "type": "array",
            "items": { "type": "number" },
            "minItems": 1,
            "maxItems": 5
        }));

        assert!(validator.validate(&json!([1, 2, 3])).is_ok());
        assert!(validator.validate(&json!([])).is_err()); // too few items
        assert!(validator.validate(&json!([1, 2, 3, 4, 5, 6])).is_err()); // too many items
        assert!(validator.validate(&json!([1, "two", 3])).is_err()); // wrong type
    }

    #[test]
    fn test_structured_output_builder() {
        let output = StructuredOutput::new()
            .text("Temperature is 22.5°C")
            .structured(json!({
                "temperature": 22.5,
                "unit": "celsius"
            }))
            .build();

        assert!(output.structured_content.is_some());
        assert!(!output.content.is_empty());
        assert_eq!(output.is_error, Some(false));
    }

    #[test]
    fn test_structured_output_error() {
        let output = StructuredOutput::new()
            .text("Failed to fetch weather")
            .structured(json!({
                "error": "API_ERROR",
                "message": "Service unavailable"
            }))
            .error()
            .build();

        assert_eq!(output.is_error, Some(true));
    }

    #[test]
    fn test_output_schema_registry() {
        let mut registry = OutputSchemaRegistry::new();
        registry.register("get_weather", OutputSchemas::weather());

        assert!(registry.has_schema("get_weather"));
        assert!(!registry.has_schema("unknown_tool"));

        let valid_output = json!({
            "temperature": 22.5,
            "unit": "celsius",
            "description": "Sunny"
        });
        assert!(registry.validate("get_weather", &valid_output).is_ok());

        let invalid_output = json!({
            "temperature": "hot", // should be number
            "unit": "celsius"
        });
        assert!(registry.validate("get_weather", &invalid_output).is_err());
    }

    #[test]
    fn test_build_validated() {
        let validator = OutputValidator::new(json!({
            "type": "object",
            "properties": {
                "value": { "type": "number" }
            },
            "required": ["value"]
        }));

        let valid = StructuredOutput::new()
            .structured(json!({"value": 42}))
            .build_validated(&validator);
        assert!(valid.is_ok());

        let invalid = StructuredOutput::new()
            .structured(json!({"other": "field"}))
            .build_validated(&validator);
        assert!(invalid.is_err());
    }

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::type_mismatch("root.field", "number", "string");
        let display = err.to_string();
        assert!(display.contains("root.field"));
        assert!(display.contains("number"));
        assert!(display.contains("string"));
    }
}