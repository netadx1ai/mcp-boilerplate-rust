use anyhow::Result;
use rmcp::{
    ErrorData as McpError,
    RoleServer,
    ServerHandler,
    ServiceExt,
    handler::server::{
        tool::ToolRouter,
        wrapper::{Json, Parameters},
    },
    model::*,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use tracing::info;

use crate::tools::{
    calculator::{CalculateRequest, CalculateResponse, EvaluateRequest, EvaluateResponse},
    shared::*,
};

#[derive(Clone)]
pub struct McpServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl McpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Echo back a message")]
    async fn echo(
        &self,
        Parameters(req): Parameters<EchoRequest>,
    ) -> Result<Json<EchoResponse>, McpError> {
        info!("Echo: {}", req.message);
        let response = create_echo_response(req.message)
            .map_err(|e| McpError::invalid_params(format!("{e}"), None))?;
        Ok(Json(response))
    }

    #[tool(description = "Simple ping-pong test to verify connection")]
    async fn ping(&self) -> Result<Json<PingResponse>, McpError> {
        info!("Ping received");
        Ok(Json(create_ping_response()))
    }

    #[tool(description = "Get information about the server capabilities")]
    async fn info(&self) -> Result<Json<InfoResponse>, McpError> {
        info!("Info requested");
        Ok(Json(create_info_response()))
    }

    #[tool(description = "Perform basic arithmetic operations (add, subtract, multiply, divide, modulo, power)")]
    async fn calculate(
        &self,
        Parameters(req): Parameters<CalculateRequest>,
    ) -> Result<Json<CalculateResponse>, McpError> {
        info!("Calculate: {} {} {}", req.a, req.operation, req.b);
        
        let result = match req.operation.to_lowercase().as_str() {
            "add" | "+" => req.a + req.b,
            "subtract" | "-" => req.a - req.b,
            "multiply" | "*" => req.a * req.b,
            "divide" | "/" => {
                if req.b == 0.0 {
                    return Err(McpError::invalid_params(
                        "Division by zero is not allowed".to_string(),
                        None,
                    ));
                }
                req.a / req.b
            }
            "modulo" | "%" => {
                if req.b == 0.0 {
                    return Err(McpError::invalid_params(
                        "Modulo by zero is not allowed".to_string(),
                        None,
                    ));
                }
                req.a % req.b
            }
            "power" | "pow" | "^" => req.a.powf(req.b),
            _ => {
                return Err(McpError::invalid_params(
                    format!(
                        "Unknown operation: '{}'. Supported: add, subtract, multiply, divide, modulo, power",
                        req.operation
                    ),
                    None,
                ));
            }
        };

        if !result.is_finite() {
            return Err(McpError::invalid_params(
                "Result is not a finite number (overflow or invalid operation)".to_string(),
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
    ) -> Result<Json<EvaluateResponse>, McpError> {
        let expression = req.expression.trim();
        
        if expression.is_empty() {
            return Err(McpError::invalid_params(
                "Expression cannot be empty".to_string(),
                None,
            ));
        }

        if expression.len() > 1000 {
            return Err(McpError::invalid_params(
                "Expression too long (max 1000 characters)".to_string(),
                None,
            ));
        }

        info!("Evaluate: {}", expression);

        let result = Self::evaluate_expression(expression).map_err(|e| {
            McpError::invalid_params(format!("Failed to evaluate expression: {}", e), None)
        })?;

        if !result.is_finite() {
            return Err(McpError::invalid_params(
                "Result is not a finite number".to_string(),
                None,
            ));
        }

        Ok(Json(EvaluateResponse {
            expression: expression.to_string(),
            result,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }))
    }

    pub async fn run(self) -> Result<()> {
        info!("Starting MCP stdio server");
        info!("Protocol: MCP v2024-11-05");
        info!("Using rmcp SDK v0.12");
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
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "MCP Boilerplate Rust Server. Available tools: echo, ping, info, calculate, evaluate.".to_string(),
            ),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        info!("MCP: Listing resources (none available)");
        Ok(ListResourcesResult::default())
    }

    async fn read_resource(
        &self,
        params: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        info!("MCP: Read resource requested: {}", params.uri);
        Err(McpError::resource_not_found(
            "Resources are not supported in this server",
            None,
        ))
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
        info!("MCP: Listing prompts (none available)");
        Ok(ListPromptsResult::default())
    }

    async fn get_prompt(
        &self,
        params: GetPromptRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        info!("MCP: Get prompt requested: {}", params.name);
        Err(McpError::invalid_request(
            "Prompts are not supported in this server",
            None,
        ))
    }
}

impl McpServer {
    fn evaluate_expression(expr: &str) -> Result<f64, String> {
        let expr = expr.replace(" ", "");
        
        for c in expr.chars() {
            if !c.is_ascii_digit() && !matches!(c, '+' | '-' | '*' | '/' | '(' | ')' | '.') {
                return Err(format!("Invalid character in expression: '{}'", c));
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
                _ => return Err(format!("Unexpected character at position {}: '{}'", pos, op)),
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
                _ => return Err(format!("Unexpected character at position {}: '{}'", pos, op)),
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
            Err(format!("Unexpected character at position {}: '{}'", pos, ch))
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
            return Err(format!("Expected number at position {}", pos));
        }
        
        let num_str = &expr[pos..end];
        let num = num_str.parse::<f64>()
            .map_err(|_| format!("Invalid number: '{}'", num_str))?;
        
        Ok((num, end))
    }
}