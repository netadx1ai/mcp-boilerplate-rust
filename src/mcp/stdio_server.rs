use anyhow::Result;
use rmcp::{
    handler::server::{
        router::prompt::PromptRouter,
        tool::ToolRouter,
        wrapper::{Json, Parameters},
    },
    model::*,
    prompt, prompt_handler, prompt_router,
    service::RequestContext,
    task_handler,
    task_manager::OperationProcessor,
    tool, tool_handler, tool_router,
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, instrument};

use crate::metrics;
use crate::resources::ResourceRegistry;
use crate::tools::{
    advanced::*,
    calculator::{CalculateRequest, CalculateResponse, EvaluateRequest, EvaluateResponse},
    shared::*,
};

// Prompt argument types
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CodeReviewArgs {
    /// Programming language (e.g., rust, python, javascript)
    pub language: String,
    /// Review focus area (e.g., security, performance, style)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ExplainCodeArgs {
    /// Explanation level (beginner, intermediate, advanced)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DebugHelpArgs {
    /// Type of error (compile, runtime, logic)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
}

#[derive(Clone)]
pub struct McpServer {
    tool_router: ToolRouter<Self>,
    prompt_router: PromptRouter<Self>,
    resource_registry: ResourceRegistry,
    processor: Arc<Mutex<OperationProcessor>>,
}

#[tool_router]
#[prompt_router]
impl McpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
            resource_registry: ResourceRegistry::new(),
            processor: Arc::new(Mutex::new(OperationProcessor::new())),
        }
    }

    // ==================== TOOLS ====================

    #[tool(description = "Echo back a message")]
    #[instrument(skip(self, _ctx), fields(tool = "echo"))]
    async fn echo(
        &self,
        Parameters(req): Parameters<EchoRequest>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<Json<EchoResponse>, McpError> {
        let start_time = std::time::Instant::now();
        info!("Echo: {}", req.message);

        if req.message.is_empty() {
            metrics::record_tool_invocation("echo", "error", start_time.elapsed().as_secs_f64());
            return Err(McpError::invalid_params(
                "Message cannot be empty. Please provide a non-empty message to echo.".to_string(),
                None,
            ));
        }

        if req.message.len() > 10240 {
            metrics::record_tool_invocation("echo", "error", start_time.elapsed().as_secs_f64());
            return Err(McpError::invalid_params(
                format!(
                    "Message exceeds maximum length of 10,240 bytes (got {} bytes). Please shorten your message.",
                    req.message.len()
                ),
                None,
            ));
        }

        let response = match create_echo_response(req.message) {
            Ok(resp) => resp,
            Err(e) => {
                metrics::record_tool_invocation(
                    "echo",
                    "error",
                    start_time.elapsed().as_secs_f64(),
                );
                return Err(McpError::invalid_params(format!("{e}"), None));
            }
        };

        metrics::record_tool_invocation("echo", "success", start_time.elapsed().as_secs_f64());
        Ok(Json(response))
    }

    #[tool(description = "Simple ping-pong test to verify connection")]
    #[instrument(skip(self, _ctx), fields(tool = "ping"))]
    async fn ping(&self, _ctx: RequestContext<RoleServer>) -> Result<Json<PingResponse>, McpError> {
        let start_time = std::time::Instant::now();
        info!("Ping received");
        metrics::record_tool_invocation("ping", "success", start_time.elapsed().as_secs_f64());
        Ok(Json(create_ping_response()))
    }

    #[tool(description = "Get information about the server capabilities")]
    #[instrument(skip(self, _ctx), fields(tool = "info"))]
    async fn info(&self, _ctx: RequestContext<RoleServer>) -> Result<Json<InfoResponse>, McpError> {
        let start_time = std::time::Instant::now();
        info!("Info requested");
        metrics::record_tool_invocation("info", "success", start_time.elapsed().as_secs_f64());
        Ok(Json(create_info_response()))
    }

    #[tool(
        description = "Perform basic arithmetic operations (add, subtract, multiply, divide, modulo, power)"
    )]
    #[instrument(skip(self, _ctx), fields(tool = "calculate"))]
    async fn calculate(
        &self,
        Parameters(req): Parameters<CalculateRequest>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<Json<CalculateResponse>, McpError> {
        let start_time = std::time::Instant::now();
        info!("Calculate: {} {} {}", req.a, req.operation, req.b);

        let result = match req.operation.to_lowercase().as_str() {
            "add" | "+" => req.a + req.b,
            "subtract" | "-" => req.a - req.b,
            "multiply" | "*" => req.a * req.b,
            "divide" | "/" => {
                if req.b == 0.0 {
                    metrics::record_tool_invocation(
                        "calculate",
                        "error",
                        start_time.elapsed().as_secs_f64(),
                    );
                    return Err(McpError::invalid_params(
                        "Division by zero is not allowed. Please provide a non-zero divisor."
                            .to_string(),
                        None,
                    ));
                }
                req.a / req.b
            }
            "modulo" | "%" => {
                if req.b == 0.0 {
                    metrics::record_tool_invocation(
                        "calculate",
                        "error",
                        start_time.elapsed().as_secs_f64(),
                    );
                    return Err(McpError::invalid_params(
                        "Modulo by zero is not allowed. Please provide a non-zero divisor."
                            .to_string(),
                        None,
                    ));
                }
                req.a % req.b
            }
            "power" | "pow" | "^" => req.a.powf(req.b),
            _ => {
                metrics::record_tool_invocation(
                    "calculate",
                    "error",
                    start_time.elapsed().as_secs_f64(),
                );
                return Err(McpError::invalid_params(
                    format!(
                        "Unknown operation: '{}'. Supported operations are: add (+), subtract (-), multiply (*), divide (/), modulo (%), power (pow/^). Please use one of these.",
                        req.operation
                    ),
                    None,
                ));
            }
        };

        if !result.is_finite() {
            metrics::record_tool_invocation(
                "calculate",
                "error",
                start_time.elapsed().as_secs_f64(),
            );
            return Err(McpError::invalid_params(
                "Result is not a finite number (overflow or invalid operation). Please check your input values and try again with smaller numbers.".to_string(),
                None,
            ));
        }

        metrics::record_tool_invocation("calculate", "success", start_time.elapsed().as_secs_f64());
        Ok(Json(CalculateResponse {
            operation: req.operation,
            a: req.a,
            b: req.b,
            result,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }))
    }

    #[tool(description = "Evaluate a mathematical expression (supports +, -, *, /, parentheses)")]
    #[instrument(skip(self, _ctx), fields(tool = "evaluate"))]
    async fn evaluate(
        &self,
        Parameters(req): Parameters<EvaluateRequest>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<Json<EvaluateResponse>, McpError> {
        let expression = req.expression.trim();

        if expression.is_empty() {
            return Err(McpError::invalid_params(
                "Expression cannot be empty".to_string(),
                None,
            ));
        }

        match self.evaluate_expression(expression) {
            Ok(result) => Ok(Json(EvaluateResponse {
                expression: req.expression,
                result,
                timestamp: chrono::Utc::now().to_rfc3339(),
            })),
            Err(e) => Err(McpError::invalid_params(e, None)),
        }
    }

    #[tool(description = "Long running task example with progress notifications")]
    #[instrument(skip(self, ctx), fields(tool = "long_task"))]
    async fn long_task(&self, ctx: RequestContext<RoleServer>) -> Result<CallToolResult, McpError> {
        info!("Long task started");

        let peer = ctx.peer.clone();

        for i in 0..10 {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            let _ = peer
                .notify_progress(ProgressNotificationParam {
                    progress_token: ProgressToken(rmcp::model::NumberOrString::String(
                        "long_task".into(),
                    )),
                    progress: i as f64,
                    total: Some(10.0),
                    message: None,
                })
                .await;

            info!("Long task progress: {}/10", i);
        }

        info!("Long task completed");
        Ok(CallToolResult::success(vec![Content::text(
            "Long task completed successfully",
        )]))
    }

    #[tool(description = "Process items with progress updates (demonstrates progress reporting)")]
    #[instrument(skip(self, ctx), fields(tool = "process_with_progress"))]
    async fn process_with_progress(
        &self,
        Parameters(req): Parameters<ProcessDataRequest>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<Json<ProcessDataResponse>, McpError> {
        AdvancedTool::process_with_progress(Parameters(req), ctx).await
    }

    #[tool(description = "Process multiple operations in batch")]
    #[instrument(skip(self, ctx), fields(tool = "batch_process"))]
    async fn batch_process(
        &self,
        Parameters(req): Parameters<BatchRequest>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<Json<BatchResponse>, McpError> {
        AdvancedTool::batch_process(Parameters(req), ctx).await
    }

    #[tool(description = "Transform data using specified transformation")]
    #[instrument(skip(self, ctx), fields(tool = "transform_data"))]
    async fn transform_data(
        &self,
        Parameters(req): Parameters<TransformRequest>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<Json<TransformResponse>, McpError> {
        AdvancedTool::transform_data(Parameters(req), ctx).await
    }

    #[tool(description = "Simulate file upload with size validation")]
    #[instrument(skip(self, ctx), fields(tool = "simulate_upload"))]
    async fn simulate_upload(
        &self,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        AdvancedTool::simulate_upload(ctx).await
    }

    #[tool(description = "Check server health status")]
    #[instrument(skip(self, ctx), fields(tool = "health_check"))]
    async fn health_check(
        &self,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        AdvancedTool::health_check(ctx).await
    }

    // ==================== PROMPTS ====================

    /// Generate a code review prompt for analyzing code quality
    #[prompt(name = "code_review")]
    async fn code_review_prompt(
        &self,
        Parameters(args): Parameters<CodeReviewArgs>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let focus = args.focus.unwrap_or_else(|| "general".to_string());

        let messages = vec![PromptMessage::new_text(
            PromptMessageRole::User,
            format!(
                "Please review the following {} code with a focus on {}. \
                Provide detailed feedback on:\n\
                1. Code quality and best practices\n\
                2. Potential bugs or issues\n\
                3. Performance considerations\n\
                4. Security vulnerabilities\n\
                5. Suggestions for improvement",
                args.language, focus
            ),
        )];

        Ok(GetPromptResult {
            description: Some(format!(
                "Code review prompt for {} with focus on {}",
                args.language, focus
            )),
            messages,
        })
    }

    /// Generate a prompt to explain code functionality
    #[prompt(name = "explain_code")]
    async fn explain_code_prompt(
        &self,
        Parameters(args): Parameters<ExplainCodeArgs>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let complexity = args.complexity.unwrap_or_else(|| "intermediate".to_string());

        let level_desc = match complexity.as_str() {
            "beginner" => "in simple terms suitable for beginners",
            "advanced" => "with technical depth for experienced developers",
            _ => "at an intermediate level",
        };

        let messages = vec![PromptMessage::new_text(
            PromptMessageRole::User,
            format!(
                "Please explain the following code {level_desc}. Include:\n\
                1. What the code does\n\
                2. How it works step by step\n\
                3. Key concepts and patterns used\n\
                4. Any important considerations"
            ),
        )];

        Ok(GetPromptResult {
            description: Some(format!("Code explanation prompt at {complexity} level")),
            messages,
        })
    }

    /// Generate a debugging assistance prompt
    #[prompt(name = "debug_help")]
    async fn debug_help_prompt(
        &self,
        Parameters(args): Parameters<DebugHelpArgs>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let error_type = args.error_type.unwrap_or_else(|| "general".to_string());

        let messages = vec![PromptMessage::new_text(
            PromptMessageRole::User,
            format!(
                "Help me debug this {error_type} error. Please:\n\
                1. Analyze the error message and code\n\
                2. Identify the root cause\n\
                3. Suggest specific fixes\n\
                4. Explain why the error occurred\n\
                5. Recommend preventive measures"
            ),
        )];

        Ok(GetPromptResult {
            description: Some(format!("Debug assistance prompt for {error_type} errors")),
            messages,
        })
    }

    // ==================== SERVER ====================

    #[instrument(skip(self))]
    pub async fn run(self) -> Result<()> {
        info!("Starting MCP stdio server");
        info!("Protocol: MCP 2025-03-26");
        info!("Using rmcp SDK");
        info!("Capabilities: Tools (11) | Prompts (3) | Resources (4) | Tasks");
        info!("Ready to receive MCP requests");

        let service = self.serve(rmcp::transport::stdio()).await?;
        service.waiting().await?;

        Ok(())
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_handler]
#[prompt_handler]
#[task_handler]
impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .enable_resources()
                .enable_tasks()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "MCP Boilerplate Rust Server with advanced features. \
                Tools: echo, ping, info, calculate, evaluate, long_task, process_with_progress, batch_process, transform_data, simulate_upload, health_check. \
                Prompts: code_review, explain_code, debug_help. \
                Resources: config, capabilities, docs, stats. \
                Supports progress notifications, task lifecycle, and real-time logging."
                    .to_string(),
            ),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        info!("MCP: Listing resources");
        let resources = self.resource_registry.list_resources();
        Ok(ListResourcesResult {
            resources,
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        params: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        info!("MCP: Read resource requested: {}", params.uri);
        self.resource_registry
            .read_resource(&params.uri)
            .map_err(|e| McpError::resource_not_found(e, None))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        info!("MCP: Listing resource templates (none available)");
        Ok(ListResourceTemplatesResult::default())
    }
}

// Expression evaluator implementation
impl McpServer {
    fn evaluate_expression(&self, expr: &str) -> Result<f64, String> {
        let mut pos = 0;
        self.parse_expression(expr, &mut pos)
    }

    fn parse_expression(&self, expr: &str, pos: &mut usize) -> Result<f64, String> {
        let mut left = self.parse_term(expr, pos)?;

        loop {
            while *pos < expr.len() && expr.chars().nth(*pos).unwrap().is_whitespace() {
                *pos += 1;
            }

            if *pos >= expr.len() {
                break;
            }

            match expr.chars().nth(*pos).unwrap() {
                '+' => {
                    *pos += 1;
                    let right = self.parse_term(expr, pos)?;
                    left += right;
                }
                '-' => {
                    *pos += 1;
                    let right = self.parse_term(expr, pos)?;
                    left -= right;
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_term(&self, expr: &str, pos: &mut usize) -> Result<f64, String> {
        let mut left = self.parse_factor(expr, pos)?;

        loop {
            while *pos < expr.len() && expr.chars().nth(*pos).unwrap().is_whitespace() {
                *pos += 1;
            }

            if *pos >= expr.len() {
                break;
            }

            match expr.chars().nth(*pos).unwrap() {
                '*' => {
                    *pos += 1;
                    let right = self.parse_factor(expr, pos)?;
                    left *= right;
                }
                '/' => {
                    *pos += 1;
                    let right = self.parse_factor(expr, pos)?;
                    if right == 0.0 {
                        return Err("Division by zero".to_string());
                    }
                    left /= right;
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_factor(&self, expr: &str, pos: &mut usize) -> Result<f64, String> {
        while *pos < expr.len() && expr.chars().nth(*pos).unwrap().is_whitespace() {
            *pos += 1;
        }

        if *pos >= expr.len() {
            return Err("Unexpected end of expression".to_string());
        }

        let char = expr.chars().nth(*pos).unwrap();

        if char == '(' {
            *pos += 1;
            let result = self.parse_expression(expr, pos)?;

            while *pos < expr.len() && expr.chars().nth(*pos).unwrap().is_whitespace() {
                *pos += 1;
            }

            if *pos >= expr.len() || expr.chars().nth(*pos).unwrap() != ')' {
                return Err("Missing closing parenthesis".to_string());
            }
            *pos += 1;
            Ok(result)
        } else if char == '-' {
            *pos += 1;
            Ok(-self.parse_factor(expr, pos)?)
        } else if char.is_ascii_digit() || char == '.' {
            self.parse_number(expr, pos)
        } else {
            Err(format!("Unexpected character: {char}"))
        }
    }

    fn parse_number(&self, expr: &str, pos: &mut usize) -> Result<f64, String> {
        let start = *pos;
        let mut has_dot = false;

        while *pos < expr.len() {
            let c = expr.chars().nth(*pos).unwrap();
            if c.is_ascii_digit() {
                *pos += 1;
            } else if c == '.' {
                if has_dot {
                    return Err("Invalid number format (multiple dots)".to_string());
                }
                has_dot = true;
                *pos += 1;
            } else {
                break;
            }
        }

        if start == *pos {
            return Err("Expected number".to_string());
        }

        let num_str = &expr[start..*pos];
        num_str.parse::<f64>().map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_simple() {
        let server = McpServer::new();
        assert_eq!(server.evaluate_expression("2+2").unwrap(), 4.0);
        assert_eq!(server.evaluate_expression("10-5").unwrap(), 5.0);
        assert_eq!(server.evaluate_expression("3*4").unwrap(), 12.0);
        assert_eq!(server.evaluate_expression("15/3").unwrap(), 5.0);
    }

    #[test]
    fn test_evaluate_precedence() {
        let server = McpServer::new();
        assert_eq!(server.evaluate_expression("2+3*4").unwrap(), 14.0);
        assert_eq!(server.evaluate_expression("10-2*3").unwrap(), 4.0);
    }

    #[test]
    fn test_evaluate_parentheses() {
        let server = McpServer::new();
        assert_eq!(server.evaluate_expression("(2+3)*4").unwrap(), 20.0);
        assert_eq!(server.evaluate_expression("2*(3+4)").unwrap(), 14.0);
    }
}