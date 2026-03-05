use anyhow::Result;
use rmcp::{
    handler::server::{
        router::prompt::PromptRouter,
        tool::ToolRouter,
        wrapper::Parameters,
    },
    model::*,
    prompt_handler, prompt_router,
    service::RequestContext,
    task_handler,
    task_manager::OperationProcessor,
    tool, tool_handler, tool_router,
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, instrument};

#[derive(Clone)]
pub struct McpServer {
    tool_router: ToolRouter<Self>,
    prompt_router: PromptRouter<Self>,
    processor: Arc<Mutex<OperationProcessor>>,
}

#[tool_router]
#[prompt_router]
impl McpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
            processor: Arc::new(Mutex::new(OperationProcessor::new())),
        }
    }

    // ==================== DATABASE (PostgREST) ====================

    #[tool(
        description = "PostgreSQL database tool via PostgREST. Actions: query, insert, update, delete, upsert, rpc, list_tables, describe. Supports filters (eq, neq, gt, gte, lt, lte, like, ilike, is, in, not, contains, containedBy, overlaps). Env: POSTGREST_URL, DB_TABLE_PREFIX."
    )]
    async fn db(
        &self,
        Parameters(req): Parameters<serde_json::Value>,
    ) -> Result<String, McpError> {
        #[cfg(feature = "postgres")]
        {
            use crate::tools::db;
            let db_req: db::DbRequest = serde_json::from_value(req)
                .map_err(|e| McpError::invalid_params(format!("Invalid db request: {e}"), None))?;
            let client = db::get_client();
            let config = db::get_config();
            let response = db::execute_db(client, config, &db_req).await;
            serde_json::to_string_pretty(&response)
                .map_err(|e| McpError::internal_error(format!("Serialization error: {e}"), None))
        }
        #[cfg(not(feature = "postgres"))]
        {
            let _ = req;
            Err(McpError::invalid_params(
                "PostgreSQL feature not enabled. Rebuild with: cargo build --features postgres",
                None,
            ))
        }
    }

    // ==================== AUTH (PostgreSQL dtv_users) ====================

    #[tool(
        description = "Authentication tool for Đấu Trường Vui. Actions: login, register, google_auth, get_user_info, check_role. Uses PostgreSQL dtv_users table via PostgREST, bcrypt passwords, JWT tokens."
    )]
    async fn auth(
        &self,
        Parameters(req): Parameters<serde_json::Value>,
    ) -> Result<String, McpError> {
        #[cfg(feature = "auth")]
        {
            use crate::tools::auth;
            let response = auth::execute(req).await;
            serde_json::to_string_pretty(&response)
                .map_err(|e| McpError::internal_error(format!("Serialization error: {e}"), None))
        }
        #[cfg(not(feature = "auth"))]
        {
            let _ = req;
            Err(McpError::invalid_params(
                "Auth feature not enabled. Rebuild with: cargo build --features auth",
                None,
            ))
        }
    }

    // ==================== TEXTGEN (V5 Proxy) ====================

    #[tool(
        description = "AI text generation via MCP V5 proxy. Supports JSON mode, structured output (json_schema), vision attachments. Credit-gated per toolId. bypassConsume=true (DTV manages credits in PostgreSQL)."
    )]
    async fn textgen(
        &self,
        Parameters(req): Parameters<serde_json::Value>,
    ) -> Result<String, McpError> {
        #[cfg(feature = "auth")]
        {
            use crate::tools::textgen;
            let response = textgen::execute(req).await;
            serde_json::to_string_pretty(&response)
                .map_err(|e| McpError::internal_error(format!("Serialization error: {e}"), None))
        }
        #[cfg(not(feature = "auth"))]
        {
            let _ = req;
            Err(McpError::invalid_params(
                "Auth feature not enabled (required for textgen). Rebuild with: cargo build --features auth",
                None,
            ))
        }
    }

    // ==================== SERVER ====================

    #[instrument(skip(self))]
    pub async fn run(self) -> Result<()> {
        info!("Starting mcp-dautruongvui-be stdio server");
        info!("Protocol: MCP 2025-03-26");
        info!("Tools: auth (PostgreSQL auth), db (PostgreSQL via PostgREST), textgen (AI via V5 proxy)");
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
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "Đấu Trường Vui MCP Backend. Tools: auth (PostgreSQL auth), db (PostgreSQL via PostgREST), textgen (AI via V5 proxy).".to_string(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let _server = McpServer::new();
    }

    #[test]
    fn test_server_default() {
        let _server = McpServer::default();
    }
}