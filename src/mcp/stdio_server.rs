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

use crate::tools::shared::*;

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
                "MCP Boilerplate Rust Server. Available tools: echo, ping, info.".to_string(),
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