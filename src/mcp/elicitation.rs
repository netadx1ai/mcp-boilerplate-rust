//! Elicitation Module
//!
//! This module provides elicitation support for MCP 2025-11-25 specification.
//! Elicitation allows servers to request interactive input from users through
//! the client, supporting both form-based and URL-based modes.
//!
//! # Modes
//!
//! - **Form Mode**: Present a structured form to collect user input with validation
//! - **URL Mode**: Direct users to an external URL for sensitive data collection
//!
//! # Example
//!
//! ```rust
//! use crate::mcp::elicitation::{ElicitationRequest, ElicitationMode};
//!
//! // Form mode
//! let form_request = ElicitationRequest::form("Please enter your details")
//!     .with_string_field("name", "Your full name", true)
//!     .with_email_field("email", "Contact email", true)
//!     .build();
//!
//! // URL mode
//! let url_request = ElicitationRequest::url(
//!     "Please complete authentication",
//!     "https://auth.example.com/oauth/authorize?state=abc123"
//! );
//! ```

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// Re-export types from rmcp for convenience
pub use rmcp::model::{
    BooleanSchema, CreateElicitationRequestParam, CreateElicitationResult, ElicitationAction,
    ElicitationSchema, ElicitationSchemaBuilder, EnumSchema, IntegerSchema, NumberSchema,
    StringSchema,
};

/// Elicitation mode determines how user input is collected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ElicitationMode {
    /// Form-based elicitation with JSON Schema validation
    Form,
    /// URL-based elicitation for sensitive data (e.g., OAuth, payments)
    Url,
}

impl Default for ElicitationMode {
    fn default() -> Self {
        Self::Form
    }
}

/// URL mode elicitation request
///
/// Used when sensitive data should not pass through the MCP client.
/// The client opens the URL and waits for a callback or user confirmation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlElicitationRequest {
    /// Human-readable message explaining what the URL is for
    pub message: String,
    /// The URL to open for data collection
    pub url: String,
    /// Optional callback URL for completion notification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
    /// Optional timeout in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

/// Combined elicitation request supporting both form and URL modes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElicitationRequest {
    /// The mode of elicitation
    pub mode: ElicitationMode,
    /// Human-readable message
    pub message: String,
    /// Form schema (only for form mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_schema: Option<ElicitationSchema>,
    /// URL for data collection (only for URL mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Optional callback URL (only for URL mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
    /// Optional timeout in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

impl ElicitationRequest {
    /// Create a form-mode elicitation request builder
    pub fn form(message: impl Into<String>) -> ElicitationFormBuilder {
        ElicitationFormBuilder::new(message.into())
    }

    /// Create a URL-mode elicitation request
    pub fn url(message: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            mode: ElicitationMode::Url,
            message: message.into(),
            requested_schema: None,
            url: Some(url.into()),
            callback_url: None,
            timeout_ms: None,
        }
    }

    /// Create a URL-mode elicitation request with callback
    pub fn url_with_callback(
        message: impl Into<String>,
        url: impl Into<String>,
        callback_url: impl Into<String>,
    ) -> Self {
        Self {
            mode: ElicitationMode::Url,
            message: message.into(),
            requested_schema: None,
            url: Some(url.into()),
            callback_url: Some(callback_url.into()),
            timeout_ms: None,
        }
    }

    /// Set timeout for the elicitation
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// Convert to rmcp CreateElicitationRequestParam (form mode only)
    pub fn to_create_param(&self) -> Option<CreateElicitationRequestParam> {
        match self.mode {
            ElicitationMode::Form => self.requested_schema.clone().map(|schema| {
                CreateElicitationRequestParam {
                    message: self.message.clone(),
                    requested_schema: schema,
                }
            }),
            ElicitationMode::Url => None,
        }
    }

    /// Convert to JSON Value for protocol transmission
    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(json!({}))
    }
}

/// Builder for form-mode elicitation requests
#[derive(Debug)]
pub struct ElicitationFormBuilder {
    message: String,
    schema_builder: ElicitationSchemaBuilder,
    timeout_ms: Option<u64>,
}

impl ElicitationFormBuilder {
    fn new(message: String) -> Self {
        Self {
            message,
            schema_builder: ElicitationSchemaBuilder::new(),
            timeout_ms: None,
        }
    }

    /// Add a required string field
    pub fn with_string_field(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        required: bool,
    ) -> Self {
        let name = name.into();
        let desc = description.into();
        if required {
            self.schema_builder =
                self.schema_builder
                    .required_string_with(name, |s| s.description(desc));
        } else {
            self.schema_builder =
                self.schema_builder
                    .optional_string_with(name, |s| s.description(desc));
        }
        self
    }

    /// Add an email field
    pub fn with_email_field(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        required: bool,
    ) -> Self {
        let name = name.into();
        let desc = description.into();
        if required {
            self.schema_builder = self.schema_builder.required_string_with(name, |s| {
                s.format(rmcp::model::StringFormat::Email).description(desc)
            });
        } else {
            self.schema_builder = self.schema_builder.optional_string_with(name, |s| {
                s.format(rmcp::model::StringFormat::Email).description(desc)
            });
        }
        self
    }

    /// Add a number field
    pub fn with_number_field(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        required: bool,
        min: Option<f64>,
        max: Option<f64>,
    ) -> Self {
        let name = name.into();
        let desc = description.into();

        let configure = move |mut schema: NumberSchema| -> NumberSchema {
            schema = schema.description(desc);
            if let (Some(min_val), Some(max_val)) = (min, max) {
                schema = schema.range(min_val, max_val);
            } else {
                if let Some(min_val) = min {
                    schema = schema.minimum(min_val);
                }
                if let Some(max_val) = max {
                    schema = schema.maximum(max_val);
                }
            }
            schema
        };

        if required {
            self.schema_builder = self.schema_builder.required_number_with(name, configure);
        } else {
            self.schema_builder = self.schema_builder.optional_number_with(name, configure);
        }
        self
    }

    /// Add an integer field
    pub fn with_integer_field(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        required: bool,
        min: Option<i64>,
        max: Option<i64>,
    ) -> Self {
        let name = name.into();
        let desc = description.into();

        let configure = move |mut schema: IntegerSchema| -> IntegerSchema {
            schema = schema.description(desc);
            if let (Some(min_val), Some(max_val)) = (min, max) {
                schema = schema.range(min_val, max_val);
            } else {
                if let Some(min_val) = min {
                    schema = schema.minimum(min_val);
                }
                if let Some(max_val) = max {
                    schema = schema.maximum(max_val);
                }
            }
            schema
        };

        if required {
            self.schema_builder = self.schema_builder.required_integer_with(name, configure);
        } else {
            self.schema_builder = self.schema_builder.optional_integer_with(name, configure);
        }
        self
    }

    /// Add a boolean field
    pub fn with_boolean_field(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        required: bool,
        default: Option<bool>,
    ) -> Self {
        let name = name.into();
        let desc = description.into();

        let configure = move |mut schema: BooleanSchema| -> BooleanSchema {
            schema = schema.description(desc);
            if let Some(default_val) = default {
                schema = schema.with_default(default_val);
            }
            schema
        };

        if required {
            self.schema_builder = self.schema_builder.required_bool_with(name, configure);
        } else {
            self.schema_builder = self.schema_builder.optional_bool_with(name, configure);
        }
        self
    }

    /// Add an enum field (single select, untitled)
    pub fn with_enum_field(
        mut self,
        name: impl Into<String>,
        values: Vec<String>,
        required: bool,
    ) -> Self {
        let name = name.into();
        let enum_schema = EnumSchema::builder(values).single_select().build();

        if required {
            self.schema_builder = self.schema_builder.required_enum_schema(name, enum_schema);
        } else {
            self.schema_builder = self.schema_builder.optional_enum_schema(name, enum_schema);
        }
        self
    }

    /// Add an enum field with titles (single select, titled)
    pub fn with_titled_enum_field(
        mut self,
        name: impl Into<String>,
        values: Vec<String>,
        titles: Vec<String>,
        required: bool,
    ) -> Self {
        let name = name.into();
        let enum_schema = EnumSchema::builder(values.clone())
            .single_select()
            .enum_titles(titles)
            .map(|b| b.build())
            .unwrap_or_else(|_| EnumSchema::builder(values).build());

        if required {
            self.schema_builder = self.schema_builder.required_enum_schema(name, enum_schema);
        } else {
            self.schema_builder = self.schema_builder.optional_enum_schema(name, enum_schema);
        }
        self
    }

    /// Add a multi-select enum field
    pub fn with_multiselect_enum_field(
        mut self,
        name: impl Into<String>,
        values: Vec<String>,
        required: bool,
        min_items: Option<u64>,
        max_items: Option<u64>,
    ) -> Self {
        let name = name.into();
        let builder = EnumSchema::builder(values).multiselect();

        // Chain the builder methods, handling ownership properly
        let builder = match min_items {
            Some(min) => builder.min_items(min).unwrap_or_else(|_| EnumSchema::builder(vec![]).multiselect()),
            None => builder,
        };

        let builder = match max_items {
            Some(max) => builder.max_items(max).unwrap_or_else(|_| EnumSchema::builder(vec![]).multiselect()),
            None => builder,
        };

        let enum_schema = builder.build();

        if required {
            self.schema_builder = self.schema_builder.required_enum_schema(name, enum_schema);
        } else {
            self.schema_builder = self.schema_builder.optional_enum_schema(name, enum_schema);
        }
        self
    }

    /// Set timeout for the elicitation
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// Build the elicitation request
    pub fn build(self) -> ElicitationRequest {
        ElicitationRequest {
            mode: ElicitationMode::Form,
            message: self.message,
            requested_schema: self.schema_builder.build().ok(),
            url: None,
            callback_url: None,
            timeout_ms: self.timeout_ms,
        }
    }
}

/// Elicitation response from the client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElicitationResponse {
    /// The user's action
    pub action: ElicitationAction,
    /// The collected data (if action is Accept)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Value>,
    /// For URL mode: indicates if the URL was successfully opened/completed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_completed: Option<bool>,
}

impl ElicitationResponse {
    /// Create an accept response with data
    pub fn accept(content: Value) -> Self {
        Self {
            action: ElicitationAction::Accept,
            content: Some(content),
            url_completed: None,
        }
    }

    /// Create a decline response
    pub fn decline() -> Self {
        Self {
            action: ElicitationAction::Decline,
            content: None,
            url_completed: None,
        }
    }

    /// Create a cancel response
    pub fn cancel() -> Self {
        Self {
            action: ElicitationAction::Cancel,
            content: None,
            url_completed: None,
        }
    }

    /// Create a URL completion response
    pub fn url_completed() -> Self {
        Self {
            action: ElicitationAction::Accept,
            content: None,
            url_completed: Some(true),
        }
    }

    /// Check if the user accepted
    pub fn is_accepted(&self) -> bool {
        matches!(self.action, ElicitationAction::Accept)
    }

    /// Check if the user declined
    pub fn is_declined(&self) -> bool {
        matches!(self.action, ElicitationAction::Decline)
    }

    /// Check if the user cancelled
    pub fn is_cancelled(&self) -> bool {
        matches!(self.action, ElicitationAction::Cancel)
    }

    /// Get the content as a specific type
    pub fn get_content<T: for<'de> Deserialize<'de>>(&self) -> Option<T> {
        self.content
            .as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Get a string field from the content
    pub fn get_string(&self, field: &str) -> Option<String> {
        self.content
            .as_ref()
            .and_then(|v| v.get(field))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Get an integer field from the content
    pub fn get_integer(&self, field: &str) -> Option<i64> {
        self.content
            .as_ref()
            .and_then(|v| v.get(field))
            .and_then(|v| v.as_i64())
    }

    /// Get a boolean field from the content
    pub fn get_bool(&self, field: &str) -> Option<bool> {
        self.content
            .as_ref()
            .and_then(|v| v.get(field))
            .and_then(|v| v.as_bool())
    }
}

impl From<CreateElicitationResult> for ElicitationResponse {
    fn from(result: CreateElicitationResult) -> Self {
        Self {
            action: result.action,
            content: result.content,
            url_completed: None,
        }
    }
}

/// Pending elicitation tracking
#[derive(Debug, Clone)]
pub struct PendingElicitation {
    pub id: String,
    pub request: ElicitationRequest,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub status: ElicitationStatus,
}

/// Status of a pending elicitation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ElicitationStatus {
    Pending,
    Completed,
    TimedOut,
    Cancelled,
}

/// Manager for tracking elicitation requests
#[derive(Debug, Clone)]
pub struct ElicitationManager {
    pending: Arc<RwLock<HashMap<String, PendingElicitation>>>,
}

impl ElicitationManager {
    pub fn new() -> Self {
        Self {
            pending: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new elicitation request and track it
    pub async fn create(&self, request: ElicitationRequest) -> String {
        let id = Uuid::new_v4().to_string();
        let pending = PendingElicitation {
            id: id.clone(),
            request,
            created_at: chrono::Utc::now(),
            status: ElicitationStatus::Pending,
        };

        let mut pending_map = self.pending.write().await;
        pending_map.insert(id.clone(), pending);
        id
    }

    /// Get a pending elicitation by ID
    pub async fn get(&self, id: &str) -> Option<PendingElicitation> {
        let pending_map = self.pending.read().await;
        pending_map.get(id).cloned()
    }

    /// Complete an elicitation with a response
    pub async fn complete(
        &self,
        id: &str,
        _response: ElicitationResponse,
    ) -> Option<PendingElicitation> {
        let mut pending_map = self.pending.write().await;
        if let Some(mut elicitation) = pending_map.remove(id) {
            elicitation.status = ElicitationStatus::Completed;
            Some(elicitation)
        } else {
            None
        }
    }

    /// Cancel an elicitation
    pub async fn cancel(&self, id: &str) -> Option<PendingElicitation> {
        let mut pending_map = self.pending.write().await;
        if let Some(mut elicitation) = pending_map.remove(id) {
            elicitation.status = ElicitationStatus::Cancelled;
            Some(elicitation)
        } else {
            None
        }
    }

    /// List all pending elicitations
    pub async fn list_pending(&self) -> Vec<PendingElicitation> {
        let pending_map = self.pending.read().await;
        pending_map
            .values()
            .filter(|e| e.status == ElicitationStatus::Pending)
            .cloned()
            .collect()
    }

    /// Clean up timed out elicitations
    pub async fn cleanup_timed_out(&self, timeout_ms: u64) {
        let now = chrono::Utc::now();
        let mut pending_map = self.pending.write().await;

        let timed_out: Vec<String> = pending_map
            .iter()
            .filter(|(_, e)| {
                let elapsed = now.signed_duration_since(e.created_at);
                elapsed.num_milliseconds() as u64 > e.request.timeout_ms.unwrap_or(timeout_ms)
            })
            .map(|(id, _)| id.clone())
            .collect();

        for id in timed_out {
            if let Some(e) = pending_map.get_mut(&id) {
                e.status = ElicitationStatus::TimedOut;
            }
        }
    }
}

impl Default for ElicitationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_elicitation_builder() {
        let request = ElicitationRequest::form("Enter your information")
            .with_string_field("name", "Your full name", true)
            .with_email_field("email", "Your email", true)
            .with_boolean_field("subscribe", "Subscribe to newsletter", false, Some(false))
            .build();

        assert_eq!(request.mode, ElicitationMode::Form);
        assert!(request.requested_schema.is_some());
        assert!(request.url.is_none());
    }

    #[test]
    fn test_url_elicitation() {
        let request =
            ElicitationRequest::url("Please authenticate", "https://auth.example.com/login");

        assert_eq!(request.mode, ElicitationMode::Url);
        assert!(request.url.is_some());
        assert!(request.requested_schema.is_none());
    }

    #[test]
    fn test_url_elicitation_with_callback() {
        let request = ElicitationRequest::url_with_callback(
            "Complete payment",
            "https://payment.example.com/checkout",
            "https://api.example.com/callback",
        );

        assert_eq!(request.mode, ElicitationMode::Url);
        assert_eq!(
            request.url,
            Some("https://payment.example.com/checkout".to_string())
        );
        assert_eq!(
            request.callback_url,
            Some("https://api.example.com/callback".to_string())
        );
    }

    #[test]
    fn test_elicitation_response() {
        let response = ElicitationResponse::accept(json!({
            "name": "John Doe",
            "email": "john@example.com"
        }));

        assert!(response.is_accepted());
        assert_eq!(response.get_string("name"), Some("John Doe".to_string()));
        assert_eq!(
            response.get_string("email"),
            Some("john@example.com".to_string())
        );
    }

    #[test]
    fn test_enum_field() {
        let request = ElicitationRequest::form("Select options")
            .with_enum_field(
                "color",
                vec!["red".into(), "green".into(), "blue".into()],
                true,
            )
            .with_titled_enum_field(
                "size",
                vec!["s".into(), "m".into(), "l".into()],
                vec!["Small".into(), "Medium".into(), "Large".into()],
                true,
            )
            .build();

        assert!(request.requested_schema.is_some());
    }

    #[test]
    fn test_multiselect_enum_field() {
        let request = ElicitationRequest::form("Select multiple")
            .with_multiselect_enum_field(
                "tags",
                vec!["rust".into(), "python".into(), "go".into()],
                true,
                Some(1),
                Some(3),
            )
            .build();

        assert!(request.requested_schema.is_some());
    }

    #[tokio::test]
    async fn test_elicitation_manager() {
        let manager = ElicitationManager::new();

        let request = ElicitationRequest::form("Test")
            .with_string_field("field", "A field", true)
            .build();

        let id = manager.create(request).await;
        assert!(!id.is_empty());

        let pending = manager.get(&id).await;
        assert!(pending.is_some());
        assert_eq!(pending.unwrap().status, ElicitationStatus::Pending);

        let response = ElicitationResponse::accept(json!({"field": "value"}));
        let completed = manager.complete(&id, response).await;
        assert!(completed.is_some());
        assert_eq!(completed.unwrap().status, ElicitationStatus::Completed);
    }
}