use anyhow::Result;
use rmcp::{
    handler::server::{
        tool::ToolRouter,
        wrapper::{Json, Parameters},
    },
    model::*,
    service::RequestContext,
    // task_handler, // TODO: Enable when task lifecycle is fully implemented
    task_manager::OperationProcessor,
    tool, tool_handler, tool_router, ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use crate::prompts::PromptRegistry;
use crate::resources::ResourceRegistry;
use crate::tools::{
    advanced::*,
    calculator::{CalculateRequest, CalculateResponse, EvaluateRequest, EvaluateResponse},
    shared::*,
};

#[derive(Clone)]
pub struct McpServer {
    tool_router: ToolRouter<Self>,
    prompt_registry: PromptRegistry,
    resource_registry: ResourceRegistry,
    #[allow(dead_code)] // Reserved for future task lifecycle implementation (SEP-1686)
    processor: Arc<Mutex<OperationProcessor>>,
}

#[tool_router]
impl McpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            prompt_registry: PromptRegistry::new(),
            resource_registry: ResourceRegistry::new(),
            processor: Arc::new(Mutex::new(OperationProcessor::new())),
        }
    }

    #[tool(description = "Echo back a message")]
    async fn echo(
        &self,
        Parameters(req): Parameters<EchoRequest>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<Json<EchoResponse>, McpError> {
        info!("Echo: {}", req.message);

        // Validation: Return tool execution error for LLM self-correction
        if req.message.is_empty() {
            return Err(McpError::invalid_params(
                "Message cannot be empty. Please provide a non-empty message to echo.".to_string(),
                None,
            ));
        }

        if req.message.len() > 10240 {
            return Err(McpError::invalid_params(
                format!(
                    "Message exceeds maximum length of 10,240 bytes (got {} bytes). Please shorten your message.",
                    req.message.len()
                ),
                None,
            ));
        }

        let response = create_echo_response(req.message)
            .map_err(|e| McpError::invalid_params(format!("{e}"), None))?;
        Ok(Json(response))
    }

    #[tool(description = "Simple ping-pong test to verify connection")]
    async fn ping(
        &self,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<Json<PingResponse>, McpError> {
        info!("Ping received");
        Ok(Json(create_ping_response()))
    }

    #[tool(description = "Get information about the server capabilities")]
    async fn info(
        &self,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<Json<InfoResponse>, McpError> {
        info!("Info requested");
        Ok(Json(create_info_response()))
    }

    #[tool(
        description = "Perform basic arithmetic operations (add, subtract, multiply, divide, modulo, power)"
    )]
    async fn calculate(
        &self,
        Parameters(req): Parameters<CalculateRequest>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<Json<CalculateResponse>, McpError> {
        info!("Calculate: {} {} {}", req.a, req.operation, req.b);

        let result = match req.operation.to_lowercase().as_str() {
            "add" | "+" => req.a + req.b,
            "subtract" | "-" => req.a - req.b,
            "multiply" | "*" => req.a * req.b,
            "divide" | "/" => {
                if req.b == 0.0 {
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
            return Err(McpError::invalid_params(
                "Result is not a finite number (overflow or invalid operation). Please check your input values and try again with smaller numbers.".to_string(),
                None,
            ));
        }

        Ok(Json(CalculateResponse {
            operation: req.operation,
            a: req.a,
            b: req.b,
            result,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }))
    }

    #[tool(description = "Evaluate a mathematical expression (supports +, -, *, /, parentheses)")]
    async fn evaluate(
        &self,
        Parameters(req): Parameters<EvaluateRequest>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<Json<EvaluateResponse>, McpError> {
        let expression = req.expression.trim();

        if expression.is_empty() {
            return Err(McpError::invalid_params(
                "Expression cannot be empty. Please provide a mathematical expression to evaluate."
                    .to_string(),
                None,
            ));
        }

        if expression.len() > 1000 {
            return Err(McpError::invalid_params(
                format!(
                    "Expression too long (maximum 1000 characters, got {}). Please shorten your expression.",
                    expression.len()
                ),
                None,
            ));
        }

        info!("Evaluate: {}", expression);

        let result = Self::evaluate_expression(expression).map_err(|e| {
            McpError::invalid_params(format!("Failed to evaluate expression: {e}"), None)
        })?;

        if !result.is_finite() {
            return Err(McpError::invalid_params(
                "Result is not a finite number (overflow or invalid operation). Please check your expression and try again with smaller numbers.".to_string(),
                None,
            ));
        }

        Ok(Json(EvaluateResponse {
            expression: expression.to_string(),
            result,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }))
    }

    #[tool(description = "Long running task example with progress notifications")]
    async fn long_task(
        &self,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        info!("Long task started");
        
        let peer = ctx.peer.clone();
        
        for i in 0..10 {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            
            let _ = peer.notify_progress(ProgressNotificationParam {
                progress_token: ProgressToken(rmcp::model::NumberOrString::String("long_task".into())),
                progress: i as f64,
                total: Some(10.0),
                message: None,
            }).await;
            
            info!("Long task progress: {}/10", i);
        }
        
        info!("Long task completed");
        Ok(CallToolResult::success(vec![Content::text(
            "Long task completed successfully",
        )]))
    }

    #[tool(description = "Process data with real-time progress notifications")]
    async fn process_with_progress(
        &self,
        params: Parameters<ProcessDataRequest>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<Json<ProcessDataResponse>, McpError> {
        AdvancedTool::process_with_progress(params, ctx).await
    }

    #[tool(description = "Batch processing with status updates and logging")]
    async fn batch_process(
        &self,
        params: Parameters<BatchRequest>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<Json<BatchResponse>, McpError> {
        AdvancedTool::batch_process(params, ctx).await
    }

    #[tool(description = "Transform data array with specified operation")]
    async fn transform_data(
        &self,
        params: Parameters<TransformRequest>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<Json<TransformResponse>, McpError> {
        AdvancedTool::transform_data(params, ctx).await
    }

    #[tool(description = "Simulate file upload with progress tracking")]
    async fn simulate_upload(
        &self,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        AdvancedTool::simulate_upload(ctx).await
    }

    #[tool(description = "Health check with system information")]
    async fn health_check(
        &self,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        AdvancedTool::health_check(ctx).await
    }

    pub async fn run(self) -> Result<()> {
        info!("Starting MCP stdio server");
        info!("Protocol: MCP 2025-03-26");
        info!("Using rmcp SDK (local)");
        info!("Capabilities: Tools (11) | Prompts (3) | Resources (4) | Progress");
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
impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_03_26,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .enable_resources()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "MCP Boilerplate Rust Server with advanced features. Tools: echo, ping, info, calculate, evaluate, long_task, process_with_progress, batch_process, transform_data, simulate_upload, health_check. Prompts: code_review, explain_code, debug_help. Resources: config, capabilities, docs, stats. Supports progress notifications and real-time logging.".to_string(),
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

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        info!("MCP: Listing prompts");
        let prompts = self.prompt_registry.list_prompts();
        Ok(ListPromptsResult {
            prompts,
            next_cursor: None,
            meta: None,
        })
    }

    async fn get_prompt(
        &self,
        params: GetPromptRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        info!("MCP: Get prompt requested: {}", params.name);

        let arguments: HashMap<String, String> = params
            .arguments
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(k, v)| {
                if let serde_json::Value::String(s) = v {
                    Some((k, s))
                } else {
                    None
                }
            })
            .collect();

        self.prompt_registry
            .get_prompt(&params.name, &arguments)
            .map_err(|e| McpError::invalid_params(e, None))
    }
}

impl McpServer {
    fn evaluate_expression(expr: &str) -> Result<f64, String> {
        let expr = expr.replace(" ", "");

        for c in expr.chars() {
            if !c.is_ascii_digit() && !matches!(c, '+' | '-' | '*' | '/' | '(' | ')' | '.') {
                return Err(format!("Invalid character in expression: '{c}'"));
            }
        }

        Self::parse_expression(&expr, 0).map(|(result, _)| result)
    }

    fn parse_expression(expr: &str, pos: usize) -> Result<(f64, usize), String> {
        let (mut left, mut pos) = Self::parse_term(expr, pos)?;

        while pos < expr.len() {
            let op = expr.chars().nth(pos).unwrap();
            match op {
                '+' | '-' => {
                    let (right, new_pos) = Self::parse_term(expr, pos + 1)?;
                    if op == '+' {
                        left += right;
                    } else {
                        left -= right;
                    }
                    pos = new_pos;
                }
                ')' => break,
                _ => return Err(format!("Unexpected character at position {pos}: '{op}'")),
            }
        }

        Ok((left, pos))
    }

    fn parse_term(expr: &str, pos: usize) -> Result<(f64, usize), String> {
        let (mut left, mut pos) = Self::parse_factor(expr, pos)?;

        while pos < expr.len() {
            let op = expr.chars().nth(pos).unwrap();
            match op {
                '*' | '/' => {
                    let (right, new_pos) = Self::parse_factor(expr, pos + 1)?;
                    if op == '*' {
                        left *= right;
                    } else {
                        if right == 0.0 {
                            return Err("Division by zero".to_string());
                        }
                        left /= right;
                    }
                    pos = new_pos;
                }
                '+' | '-' | ')' => break,
                _ => return Err(format!("Unexpected character at position {pos}: '{op}'")),
            }
        }

        Ok((left, pos))
    }

    fn parse_factor(expr: &str, pos: usize) -> Result<(f64, usize), String> {
        if pos >= expr.len() {
            return Err("Unexpected end of expression".to_string());
        }

        let ch = expr.chars().nth(pos).unwrap();

        if ch == '(' {
            let (result, new_pos) = Self::parse_expression(expr, pos + 1)?;
            if new_pos >= expr.len() || expr.chars().nth(new_pos).unwrap() != ')' {
                return Err("Missing closing parenthesis".to_string());
            }
            Ok((result, new_pos + 1))
        } else if ch == '-' || ch == '+' {
            let (result, new_pos) = Self::parse_factor(expr, pos + 1)?;
            if ch == '-' {
                Ok((-result, new_pos))
            } else {
                Ok((result, new_pos))
            }
        } else if ch.is_ascii_digit() || ch == '.' {
            Self::parse_number(expr, pos)
        } else {
            Err(format!("Unexpected character at position {pos}: '{ch}'"))
        }
    }

    fn parse_number(expr: &str, pos: usize) -> Result<(f64, usize), String> {
        let mut end = pos;
        let mut has_dot = false;

        while end < expr.len() {
            let ch = expr.chars().nth(end).unwrap();
            if ch.is_ascii_digit() {
                end += 1;
            } else if ch == '.' && !has_dot {
                has_dot = true;
                end += 1;
            } else {
                break;
            }
        }

        if end == pos {
            return Err(format!("Expected number at position {pos}"));
        }

        let num_str = &expr[pos..end];
        let num = num_str
            .parse::<f64>()
            .map_err(|_| format!("Invalid number: '{num_str}'"))?;

        Ok((num, end))
    }
}
