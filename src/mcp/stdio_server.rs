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
    tool,
    tool_handler,
    tool_router,
    ErrorData as McpError,
    RoleServer,
    ServerHandler,
    ServiceExt,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, instrument};

use crate::metrics;
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
    #[instrument(skip(self, _ctx), fields(tool = "echo"))]
    async fn echo(
        &self,
        Parameters(req): Parameters<EchoRequest>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<Json<EchoResponse>, McpError> {
        let start_time = std::time::Instant::now();
        info!("Echo: {}", req.message);

        // Validation: Return tool execution error for LLM self-correction
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

    #[instrument(skip(self))]
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

impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
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
    fn evaluate_expression(&self, expr: &str) -> Result<f64, String> {
        let mut pos = 0;
        self.parse_expression(expr, &mut pos)
    }

    fn parse_expression(&self, expr: &str, pos: &mut usize) -> Result<f64, String> {
        let mut left = self.parse_term(expr, pos)?;

        loop {
            // Skip whitespace
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
            // Skip whitespace
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
        // Skip whitespace
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
            
            // Skip whitespace
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
        } else if char.is_digit(10) || char == '.' {
            self.parse_number(expr, pos)
        } else {
            Err(format!("Unexpected character: {}", char))
        }
    }

    fn parse_number(&self, expr: &str, pos: &mut usize) -> Result<f64, String> {
        let start = *pos;
        let mut has_dot = false;

        while *pos < expr.len() {
            let c = expr.chars().nth(*pos).unwrap();
            if c.is_digit(10) {
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