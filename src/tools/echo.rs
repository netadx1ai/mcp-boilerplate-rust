use rmcp::{
    ErrorData as McpError,
    handler::server::tool::ToolRouter,
    handler::server::wrapper::{Json, Parameters},
    tool, tool_router,
};
use tracing::info;

use super::shared::*;

#[derive(Clone)]
pub struct EchoTool {
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl EchoTool {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[allow(dead_code)]
    pub fn router(&self) -> &ToolRouter<Self> {
        &self.tool_router
    }

    #[tool(description = "Echo back a message")]
    pub async fn echo(
        &self,
        params: Parameters<EchoRequest>,
    ) -> Result<Json<EchoResponse>, McpError> {
        let message = &params.0.message;
        info!("Echo: {}", message);
        let response = create_echo_response(message.clone())
            .map_err(|e| McpError::invalid_params(format!("{e}"), None))?;
        Ok(Json(response))
    }

    #[tool(description = "Simple ping-pong test to verify connection")]
    pub async fn ping(
        &self,
    ) -> Result<Json<PingResponse>, McpError> {
        info!("Ping received");
        Ok(Json(create_ping_response()))
    }

    #[tool(description = "Get information about the echo tool capabilities")]
    pub async fn info(
        &self,
    ) -> Result<Json<InfoResponse>, McpError> {
        info!("Info requested");
        Ok(Json(create_info_response()))
    }
}

impl Default for EchoTool {
    fn default() -> Self {
        Self::new()
    }
}