//! Template Server using official RMCP SDK
//!
//! This is a production-ready MCP server for content templates and structured outputs.
//! Provides template management, rendering, validation, and content generation capabilities.

use anyhow::Result;
use clap::Parser;
use handlebars::Handlebars;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    service::RequestContext,
    tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

/// Command line arguments
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

/// Template rendering arguments
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RenderTemplateArgs {
    /// Template ID to render
    pub template_id: String,
    /// Parameters for template rendering
    pub parameters: HashMap<String, serde_json::Value>,
    /// Output format (default: markdown)
    #[serde(default = "default_format")]
    pub format: String,
}

/// Template listing arguments
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListTemplatesArgs {
    /// Category filter (optional)
    pub category: Option<String>,
    /// Include template preview (default: false)
    #[serde(default)]
    pub include_preview: bool,
}

/// Template validation arguments
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ValidateTemplateArgs {
    /// Template ID to validate
    pub template_id: String,
    /// Parameters to validate against template
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Template creation arguments
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateTemplateArgs {
    /// Template name
    pub name: String,
    /// Template category
    pub category: String,
    /// Template content (Handlebars format)
    pub content: String,
    /// Required parameters description
    pub required_params: Vec<String>,
    /// Template description
    pub description: String,
}

/// Template metadata
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Template {
    /// Unique template ID
    pub id: String,
    /// Template name
    pub name: String,
    /// Template category
    pub category: String,
    /// Template description
    pub description: String,
    /// Template content (Handlebars)
    pub content: String,
    /// Required parameters
    pub required_params: Vec<String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last modified timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Template server
#[derive(Clone)]
pub struct TemplateServer {
    /// Handlebars template engine
    handlebars: Arc<Handlebars<'static>>,
    /// Template storage
    templates: Arc<Mutex<HashMap<String, Template>>>,
    /// Tool router for handling tool calls
    tool_router: ToolRouter<TemplateServer>,
    /// Request statistics
    stats: Arc<Mutex<HashMap<String, u64>>>,
}

// Default value functions
fn default_format() -> String {
    "markdown".to_string()
}

impl TemplateServer {
    /// Create new template server
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        let server = Self {
            handlebars: Arc::new(handlebars),
            templates: Arc::new(Mutex::new(HashMap::new())),
            tool_router: Self::tool_router(),
            stats: Arc::new(Mutex::new(HashMap::new())),
        };

        // Initialize with built-in templates
        server.init_builtin_templates();
        server
    }

    /// Initialize built-in templates
    fn init_builtin_templates(&self) {
        let builtin_templates = vec![
            Template {
                id: "blog-post".to_string(),
                name: "Blog Post".to_string(),
                category: "content".to_string(),
                description:
                    "Standard blog post template with title, introduction, sections, and conclusion"
                        .to_string(),
                content: r#"# {{title}}

{{#if author}}*By {{author}}*{{/if}}
{{#if date}}*Published: {{date}}*{{/if}}

## Introduction

{{introduction}}

{{#each sections}}
## {{title}}

{{content}}

{{/each}}

## Conclusion

{{conclusion}}

{{#if tags}}
**Tags:** {{#each tags}}#{{this}} {{/each}}
{{/if}}"#
                    .to_string(),
                required_params: vec![
                    "title".to_string(),
                    "introduction".to_string(),
                    "sections".to_string(),
                    "conclusion".to_string(),
                ],
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Template {
                id: "email-template".to_string(),
                name: "Email Template".to_string(),
                category: "communication".to_string(),
                description: "Professional email template with greeting, body, and signature"
                    .to_string(),
                content: r#"Subject: {{subject}}

{{#if recipient_name}}Dear {{recipient_name}},{{else}}Hello,{{/if}}

{{body}}

{{#each action_items}}
- {{this}}
{{/each}}

{{#if closing}}{{closing}}{{else}}Best regards,{{/if}}
{{sender_name}}
{{#if sender_title}}{{sender_title}}{{/if}}
{{#if company}}{{company}}{{/if}}"#
                    .to_string(),
                required_params: vec![
                    "subject".to_string(),
                    "body".to_string(),
                    "sender_name".to_string(),
                ],
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Template {
                id: "api-documentation".to_string(),
                name: "API Documentation".to_string(),
                category: "technical".to_string(),
                description: "API endpoint documentation template".to_string(),
                content: r#"# {{endpoint_name}} API

## Endpoint
`{{method}} {{path}}`

## Description
{{description}}

## Parameters

{{#each parameters}}
### {{name}}
- **Type:** {{type}}
- **Required:** {{required}}
- **Description:** {{description}}
{{#if example}}
- **Example:** `{{example}}`
{{/if}}

{{/each}}

## Response

```json
{{response_example}}
```

{{#if error_codes}}
## Error Codes

{{#each error_codes}}
- **{{code}}:** {{description}}
{{/each}}
{{/if}}

## Example Request

```{{request_language}}
{{request_example}}
```"#
                    .to_string(),
                required_params: vec![
                    "endpoint_name".to_string(),
                    "method".to_string(),
                    "path".to_string(),
                    "description".to_string(),
                ],
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Template {
                id: "project-readme".to_string(),
                name: "Project README".to_string(),
                category: "documentation".to_string(),
                description: "Comprehensive project README template".to_string(),
                content: r#"# {{project_name}}

{{description}}

{{#if badges}}
{{#each badges}}
![{{name}}]({{url}})
{{/each}}
{{/if}}

## Features

{{#each features}}
- {{this}}
{{/each}}

## Installation

```bash
{{installation_command}}
```

## Usage

{{usage_description}}

```{{code_language}}
{{usage_example}}
```

## API Reference

{{#each api_sections}}
### {{title}}

{{description}}

{{/each}}

## Contributing

{{#if contributing_guidelines}}
{{contributing_guidelines}}
{{else}}
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request
{{/if}}

## License

{{#if license}}
This project is licensed under the {{license}} License.
{{else}}
MIT License
{{/if}}"#
                    .to_string(),
                required_params: vec![
                    "project_name".to_string(),
                    "description".to_string(),
                    "features".to_string(),
                    "installation_command".to_string(),
                ],
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ];

        // Store built-in templates
        if let Ok(mut templates) = self.templates.try_lock() {
            for template in builtin_templates {
                templates.insert(template.id.clone(), template);
            }
        }
    }

    /// Update request statistics
    async fn update_stats(&self, tool_name: &str) {
        let mut stats = self.stats.lock().await;
        *stats.entry(tool_name.to_string()).or_insert(0) += 1;
    }

    /// Validate template parameters
    fn validate_params_internal(
        &self,
        template: &Template,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<(), String> {
        for required_param in &template.required_params {
            if !params.contains_key(required_param) {
                return Err(format!("Missing required parameter: {required_param}"));
            }
        }
        Ok(())
    }

    /// Render template with parameters
    fn render_template_internal(
        &self,
        template: &Template,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<String, String> {
        self.handlebars
            .render_template(&template.content, params)
            .map_err(|e| format!("Template rendering error: {e}"))
    }

    /// Format template list for display
    fn format_template_list(&self, templates: &[&Template], include_preview: bool) -> String {
        let mut result = String::new();

        // Group by category
        let mut categories: HashMap<String, Vec<&Template>> = HashMap::new();
        for template in templates {
            categories
                .entry(template.category.clone())
                .or_default()
                .push(template);
        }

        for (category, category_templates) in categories {
            result.push_str(&format!("## {} Templates\n\n", category.to_uppercase()));

            for template in category_templates {
                result.push_str(&format!("### {} ({})\n", template.name, template.id));
                result.push_str(&format!("**Description:** {}\n", template.description));
                result.push_str(&format!(
                    "**Required Parameters:** {}\n",
                    template.required_params.join(", ")
                ));
                result.push_str(&format!(
                    "**Updated:** {}\n",
                    template.updated_at.format("%Y-%m-%d %H:%M UTC")
                ));

                if include_preview {
                    let preview = template
                        .content
                        .lines()
                        .take(3)
                        .collect::<Vec<_>>()
                        .join("\n");
                    result.push_str(&format!("**Preview:**\n```\n{preview}...\n```\n"));
                }

                result.push_str("\n---\n\n");
            }
        }

        result
    }
}

#[tool_router]
impl TemplateServer {
    /// List available templates
    #[tool(description = "List available content templates, optionally filtered by category")]
    async fn list_templates(
        &self,
        Parameters(args): Parameters<ListTemplatesArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.update_stats("list_templates").await;

        info!(
            "📋 Template list request: category={:?}, preview={}",
            args.category, args.include_preview
        );

        let templates = self.templates.lock().await;
        let filtered_templates: Vec<&Template> = templates
            .values()
            .filter(|t| args.category.as_ref().is_none_or(|cat| &t.category == cat))
            .collect();

        let formatted_list = self.format_template_list(&filtered_templates, args.include_preview);

        let summary = format!(
            "📋 **Available Templates**\n\n**Total Found:** {}\n**Category Filter:** {}\n\n{}",
            filtered_templates.len(),
            args.category.unwrap_or_else(|| "All".to_string()),
            formatted_list
        );

        Ok(CallToolResult::success(vec![Content::text(summary)]))
    }

    /// Get a specific template
    #[tool(description = "Get details of a specific template by ID")]
    async fn get_template(
        &self,
        Parameters(args): Parameters<serde_json::Value>,
    ) -> Result<CallToolResult, McpError> {
        self.update_stats("get_template").await;

        let template_id = args
            .get("template_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                McpError::new(
                    rmcp::model::ErrorCode(-32602),
                    "Missing 'template_id' parameter",
                    None,
                )
            })?;

        info!("📄 Template details request: id='{}'", template_id);

        let templates = self.templates.lock().await;
        let template = templates.get(template_id).ok_or_else(|| {
            McpError::new(
                rmcp::model::ErrorCode(-32602),
                format!("Template '{template_id}' not found"),
                None,
            )
        })?;

        let template_info = format!(
            "📄 **Template: {}**\n\n**ID:** {}\n**Category:** {}\n**Description:** {}\n\n**Required Parameters:** {}\n\n**Created:** {}\n**Updated:** {}\n\n**Content:**\n```handlebars\n{}\n```",
            template.name,
            template.id,
            template.category,
            template.description,
            template.required_params.join(", "),
            template.created_at.format("%Y-%m-%d %H:%M UTC"),
            template.updated_at.format("%Y-%m-%d %H:%M UTC"),
            template.content
        );

        Ok(CallToolResult::success(vec![Content::text(template_info)]))
    }

    /// Render a template with provided parameters
    #[tool(description = "Render a template with provided parameters")]
    async fn render_template(
        &self,
        Parameters(args): Parameters<RenderTemplateArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.update_stats("render_template").await;

        info!(
            "🎨 Template render request: id='{}', params={}",
            args.template_id,
            args.parameters.len()
        );

        let templates = self.templates.lock().await;
        let template = templates.get(&args.template_id).ok_or_else(|| {
            McpError::new(
                rmcp::model::ErrorCode(-32602),
                format!("Template '{}' not found", args.template_id),
                None,
            )
        })?;

        // Validate required parameters
        if let Err(validation_error) = self.validate_params_internal(template, &args.parameters) {
            return Err(McpError::new(
                rmcp::model::ErrorCode(-32602),
                validation_error,
                None,
            ));
        }

        // Render template
        let rendered_content = self
            .render_template_internal(template, &args.parameters)
            .map_err(|e| McpError::new(rmcp::model::ErrorCode(-32603), e, None))?;

        let result = format!(
            "🎨 **Rendered Template: {}**\n\n**Format:** {}\n**Parameters Used:** {}\n\n**Output:**\n\n{}",
            template.name,
            args.format,
            args.parameters.keys().cloned().collect::<Vec<_>>().join(", "),
            rendered_content
        );

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Validate template parameters
    #[tool(description = "Validate parameters against a template's requirements")]
    async fn validate_template_params(
        &self,
        Parameters(args): Parameters<ValidateTemplateArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.update_stats("validate_template_params").await;

        info!(
            "✅ Template validation request: id='{}', params={}",
            args.template_id,
            args.parameters.len()
        );

        let templates = self.templates.lock().await;
        let template = templates.get(&args.template_id).ok_or_else(|| {
            McpError::new(
                rmcp::model::ErrorCode(-32602),
                format!("Template '{}' not found", args.template_id),
                None,
            )
        })?;

        let validation_result = self.validate_params_internal(template, &args.parameters);

        let result = match validation_result {
            Ok(()) => {
                format!(
                    "✅ **Template Validation: PASSED**\n\n**Template:** {}\n**Parameters Provided:** {}\n**Required Parameters:** {}\n\n**Status:** All required parameters are present and valid.",
                    template.name,
                    args.parameters.keys().cloned().collect::<Vec<_>>().join(", "),
                    template.required_params.join(", ")
                )
            }
            Err(error) => {
                format!(
                    "❌ **Template Validation: FAILED**\n\n**Template:** {}\n**Error:** {}\n**Required Parameters:** {}\n**Provided Parameters:** {}",
                    template.name,
                    error,
                    template.required_params.join(", "),
                    args.parameters.keys().cloned().collect::<Vec<_>>().join(", ")
                )
            }
        };

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Create a new custom template
    #[tool(description = "Create a new custom template")]
    async fn create_template(
        &self,
        Parameters(args): Parameters<CreateTemplateArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.update_stats("create_template").await;

        info!("🆕 Template creation request: name='{}'", args.name);

        // Generate unique ID
        let template_id = format!(
            "custom-{}",
            uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
        );

        // Test template compilation with dummy parameters
        let mut test_params = HashMap::new();
        for param in &args.required_params {
            test_params.insert(param.clone(), serde_json::Value::String("test".to_string()));
        }

        if let Err(e) = self.handlebars.render_template(&args.content, &test_params) {
            return Err(McpError::new(
                rmcp::model::ErrorCode(-32602),
                format!("Invalid template syntax: {e}"),
                None,
            ));
        }

        let template = Template {
            id: template_id.clone(),
            name: args.name.clone(),
            category: args.category.clone(),
            description: args.description.clone(),
            content: args.content.clone(),
            required_params: args.required_params.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Store template
        let mut templates = self.templates.lock().await;
        templates.insert(template_id.clone(), template);

        let result = format!(
            "🆕 **Template Created Successfully**\n\n**ID:** {}\n**Name:** {}\n**Category:** {}\n**Description:** {}\n**Required Parameters:** {}\n\nTemplate is now available for use with `render_template` tool.",
            template_id,
            args.name,
            args.category,
            args.description,
            args.required_params.join(", ")
        );

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get available template categories
    #[tool(description = "Get list of available template categories")]
    async fn get_categories(&self) -> Result<CallToolResult, McpError> {
        self.update_stats("get_categories").await;

        let templates = self.templates.lock().await;
        let categories: std::collections::HashSet<String> =
            templates.values().map(|t| t.category.clone()).collect();

        let mut category_list: Vec<String> = categories.into_iter().collect();
        category_list.sort();

        let category_display = category_list
            .iter()
            .enumerate()
            .map(|(i, cat)| format!("{}. {}", i + 1, cat))
            .collect::<Vec<_>>()
            .join("\n");

        let result = format!(
            "📚 **Available Template Categories**\n\n{category_display}\n\nUse any of these categories with the `list_templates` tool."
        );

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get server status
    #[tool(description = "Get server health status and usage statistics")]
    async fn get_server_status(&self) -> Result<CallToolResult, McpError> {
        self.update_stats("get_server_status").await;

        let stats = self.stats.lock().await;
        let templates = self.templates.lock().await;

        let mut status_parts = vec![
            "🟢 **Template Server Status: HEALTHY**".to_string(),
            format!("📄 **Templates Available:** {}", templates.len()),
            format!(
                "⏱️ **Uptime:** Server Running (Started: {})",
                chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
            ),
        ];

        if !stats.is_empty() {
            status_parts.push("📊 **Usage Statistics:**".to_string());
            for (tool, count) in stats.iter() {
                status_parts.push(format!("  - {tool}: {count} requests"));
            }
        }

        let status_report = status_parts.join("\n");

        Ok(CallToolResult::success(vec![Content::text(status_report)]))
    }
}

#[tool_handler]
impl ServerHandler for TemplateServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "📄 Template Server - Content generation and structured outputs:\n\
                • list_templates: List available templates with optional category filter\n\
                • get_template: Get detailed information about a specific template\n\
                • render_template: Render template with provided parameters\n\
                • validate_template_params: Validate parameters against template requirements\n\
                • create_template: Create new custom templates\n\
                • get_categories: List available template categories\n\
                • get_server_status: Health check and usage statistics\n\n\
                🎨 Built-in templates: blog-post, email-template, api-documentation, project-readme\n\
                🚀 Fast, lightweight implementation using official RMCP SDK"
                .to_string()
            ),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        info!("📄 Template Server initialized successfully");
        info!("🎨 Built-in templates loaded and ready");
        Ok(self.get_info())
    }
}

impl Default for TemplateServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Setup logging
    let log_level = if args.debug { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(format!("template_server={log_level}").parse()?)
                .add_directive(format!("rmcp={log_level}").parse()?),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    info!("🚀 Starting Template Server using official RMCP SDK");
    info!("📄 Content generation and structured outputs ready");

    // Create server instance
    let server = TemplateServer::new();

    // Start the server with STDIO transport
    let service = server.serve(stdio()).await.inspect_err(|e| {
        error!("Failed to start server: {:?}", e);
    })?;

    info!("✅ Template Server started and ready for MCP connections");
    info!("🎨 Built-in templates available for immediate use");

    // Wait for the service to complete
    service.waiting().await?;

    info!("Server shutdown complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let server = TemplateServer::new();
        let info = server.get_info();

        assert!(info.capabilities.tools.is_some());
        assert!(info.instructions.is_some());
        assert!(info.instructions.unwrap().contains("Template Server"));
    }

    #[tokio::test]
    async fn test_builtin_templates_loaded() {
        let server = TemplateServer::new();
        let templates = server.templates.lock().await;

        assert!(templates.contains_key("blog-post"));
        assert!(templates.contains_key("email-template"));
        assert!(templates.contains_key("api-documentation"));
        assert!(templates.contains_key("project-readme"));
    }

    #[tokio::test]
    async fn test_list_templates_tool() {
        let server = TemplateServer::new();
        let args = ListTemplatesArgs {
            category: None,
            include_preview: false,
        };

        let result = server.list_templates(Parameters(args)).await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("Available Templates"));
                assert!(text.text.contains("blog-post"));
            }
        }
    }

    #[tokio::test]
    async fn test_get_template_tool() {
        let server = TemplateServer::new();
        let args = serde_json::json!({
            "template_id": "blog-post"
        });

        let result = server.get_template(Parameters(args)).await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("Template: Blog Post"));
                assert!(text.text.contains("handlebars"));
            }
        }
    }

    #[tokio::test]
    async fn test_render_template_tool() {
        let server = TemplateServer::new();
        let mut params = HashMap::new();
        params.insert(
            "title".to_string(),
            serde_json::Value::String("Test Blog".to_string()),
        );
        params.insert(
            "introduction".to_string(),
            serde_json::Value::String("This is a test".to_string()),
        );
        params.insert(
            "sections".to_string(),
            serde_json::json!([{"title": "Section 1", "content": "Test content"}]),
        );
        params.insert(
            "conclusion".to_string(),
            serde_json::Value::String("Test conclusion".to_string()),
        );

        let args = RenderTemplateArgs {
            template_id: "blog-post".to_string(),
            parameters: params,
            format: "markdown".to_string(),
        };

        let result = server.render_template(Parameters(args)).await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("Rendered Template"));
                assert!(text.text.contains("# Test Blog"));
            }
        }
    }

    #[tokio::test]
    async fn test_validate_template_params_tool() {
        let server = TemplateServer::new();
        let mut params = HashMap::new();
        params.insert(
            "title".to_string(),
            serde_json::Value::String("Test".to_string()),
        );

        let args = ValidateTemplateArgs {
            template_id: "blog-post".to_string(),
            parameters: params,
        };

        let result = server
            .validate_template_params(Parameters(args))
            .await
            .unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("Template Validation"));
            }
        }
    }

    #[tokio::test]
    async fn test_get_categories_tool() {
        let server = TemplateServer::new();

        let result = server.get_categories().await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("Available Template Categories"));
                assert!(text.text.contains("content"));
                assert!(text.text.contains("communication"));
            }
        }
    }

    #[tokio::test]
    async fn test_server_status_tool() {
        let server = TemplateServer::new();

        let result = server.get_server_status().await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("Template Server Status: HEALTHY"));
                assert!(text.text.contains("Templates Available"));
            }
        }
    }

    #[tokio::test]
    async fn test_create_template_tool() {
        let server = TemplateServer::new();
        let args = CreateTemplateArgs {
            name: "Test Template".to_string(),
            category: "test".to_string(),
            content: "Hello {{name}}!".to_string(),
            required_params: vec!["name".to_string()],
            description: "A simple test template".to_string(),
        };

        let result = server.create_template(Parameters(args)).await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("Template Created Successfully"));
                assert!(text.text.contains("Test Template"));
            }
        }
    }

    #[tokio::test]
    async fn test_template_validation() {
        let server = TemplateServer::new();
        let templates = server.templates.lock().await;
        let template = templates.get("blog-post").unwrap();

        // Test with missing required parameters
        let incomplete_params = HashMap::new();
        let validation_result = server.validate_params_internal(template, &incomplete_params);
        assert!(validation_result.is_err());

        // Test with all required parameters
        let mut complete_params = HashMap::new();
        complete_params.insert(
            "title".to_string(),
            serde_json::Value::String("Test".to_string()),
        );
        complete_params.insert(
            "introduction".to_string(),
            serde_json::Value::String("Test".to_string()),
        );
        complete_params.insert("sections".to_string(), serde_json::json!([]));
        complete_params.insert(
            "conclusion".to_string(),
            serde_json::Value::String("Test".to_string()),
        );

        let validation_result = server.validate_params_internal(template, &complete_params);
        assert!(validation_result.is_ok());
    }
}
