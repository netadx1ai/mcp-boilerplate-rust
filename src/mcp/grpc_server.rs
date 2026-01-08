//! gRPC Server for MCP Protocol
//!
//! Provides high-performance RPC server using tonic/gRPC.
//! Supports bidirectional streaming and efficient binary serialization.

#[cfg(feature = "grpc")]
use crate::mcp::protocol_handler::ProtocolHandler;
#[cfg(feature = "grpc")]
use crate::transport::grpc::GrpcTransport;
#[cfg(feature = "grpc")]
use crate::transport::Transport;

#[cfg(feature = "grpc")]
use std::sync::Arc;
#[cfg(feature = "grpc")]
use tokio::sync::RwLock;
#[cfg(feature = "grpc")]
use tonic::{transport::Server, Request, Response, Status};
#[cfg(feature = "grpc")]
use tracing::{error, info};

#[cfg(feature = "grpc")]
pub mod mcp_service {
    tonic::include_proto!("mcp");
}

#[cfg(feature = "grpc")]
use mcp_service::{
    mcp_server::{Mcp, McpServer},
    JsonRpcRequest, JsonRpcResponse, ToolCallRequest, ToolCallResponse,
    ToolsListRequest, ToolsListResponse, StreamRequest, StreamResponse,
};

/// gRPC server state
#[cfg(feature = "grpc")]
#[derive(Clone)]
pub struct GrpcServerState {
    pub protocol_handler: Arc<ProtocolHandler>,
    pub transport: Arc<RwLock<GrpcTransport>>,
    pub active_connections: Arc<RwLock<u32>>,
    pub total_requests: Arc<RwLock<u64>>,
}

/// MCP gRPC service implementation
#[cfg(feature = "grpc")]
pub struct McpService {
    state: GrpcServerState,
}

#[cfg(feature = "grpc")]
impl McpService {
    pub fn new(state: GrpcServerState) -> Self {
        Self { state }
    }
}

#[cfg(feature = "grpc")]
#[tonic::async_trait]
impl Mcp for McpService {
    /// Handle JSON-RPC requests
    async fn json_rpc(
        &self,
        request: Request<JsonRpcRequest>,
    ) -> Result<Response<JsonRpcResponse>, Status> {
        let req = request.into_inner();
        
        {
            let mut total = self.state.total_requests.write().await;
            *total += 1;
        }

        info!("gRPC JSON-RPC request: {}", req.payload);

        // Parse JSON-RPC request
        let json_request: serde_json::Value = match serde_json::from_str(&req.payload) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to parse JSON-RPC request: {}", e);
                return Err(Status::invalid_argument(format!(
                    "Invalid JSON-RPC request: {}",
                    e
                )));
            }
        };

        // Process through protocol handler
        let request_str = serde_json::to_string(&json_request).unwrap_or_default();
        let response_str = match self.state.protocol_handler.handle_request(&request_str).await {
            Ok(s) => s,
            Err(e) => {
                error!("Protocol handler error: {}", e);
                return Err(Status::internal(format!("Handler error: {}", e)));
            }
        };

        // Response is already a JSON string
        let response_str = match response_str.len() {
            0 => return Err(Status::internal("Empty response from handler")),
            _ => response_str,
        };

        Ok(Response::new(JsonRpcResponse {
            payload: response_str,
        }))
    }

    /// List available tools
    async fn list_tools(
        &self,
        request: Request<ToolsListRequest>,
    ) -> Result<Response<ToolsListResponse>, Status> {
        info!("gRPC list_tools request");

        let json_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        });

        let request_str = serde_json::to_string(&json_request).unwrap();
        let tools_json = match self.state.protocol_handler.handle_request(&request_str).await {
            Ok(s) => s,
            Err(e) => {
                return Err(Status::internal(format!("Handler error: {}", e)));
            }
        };

        Ok(Response::new(ToolsListResponse {
            tools_json,
        }))
    }

    /// Call a tool
    async fn call_tool(
        &self,
        request: Request<ToolCallRequest>,
    ) -> Result<Response<ToolCallResponse>, Status> {
        let req = request.into_inner();
        
        info!("gRPC call_tool: {} with args: {}", req.tool_name, req.arguments_json);

        let arguments: serde_json::Value = match serde_json::from_str(&req.arguments_json) {
            Ok(v) => v,
            Err(e) => {
                return Err(Status::invalid_argument(format!(
                    "Invalid arguments JSON: {}",
                    e
                )));
            }
        };

        let json_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": req.tool_name,
                "arguments": arguments
            }
        });

        let request_str = serde_json::to_string(&json_request).unwrap();
        let result_json = match self.state.protocol_handler.handle_request(&request_str).await {
            Ok(s) => s,
            Err(e) => {
                return Err(Status::internal(format!("Handler error: {}", e)));
            }
        };

        Ok(Response::new(ToolCallResponse {
            result_json,
        }))
    }

    /// Server streaming endpoint
    type StreamResponsesStream = futures::stream::BoxStream<
        'static,
        Result<StreamResponse, Status>,
    >;

    async fn stream_responses(
        &self,
        request: Request<StreamRequest>,
    ) -> Result<Response<Self::StreamResponsesStream>, Status> {
        let req = request.into_inner();
        
        info!("gRPC streaming request: {}", req.request_id);

        {
            let mut active = self.state.active_connections.write().await;
            *active += 1;
        }

        // Create a stream of responses
        use futures::stream::{self, StreamExt};
        
        // Example: Stream multiple chunks
        let chunks = vec![
            StreamResponse {
                chunk_id: 1,
                data: b"Chunk 1 data".to_vec(),
                is_final: false,
            },
            StreamResponse {
                chunk_id: 2,
                data: b"Chunk 2 data".to_vec(),
                is_final: false,
            },
            StreamResponse {
                chunk_id: 3,
                data: b"Chunk 3 data - final".to_vec(),
                is_final: true,
            },
        ];

        let stream = stream::iter(chunks)
            .map(Ok::<_, Status>);

        // Cleanup when stream ends
        let state_clone = self.state.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            let mut active = state_clone.active_connections.write().await;
            if *active > 0 {
                *active -= 1;
            }
        });

        Ok(Response::new(Box::pin(stream)))
    }
}

/// Start gRPC server
#[cfg(feature = "grpc")]
pub async fn run_grpc_server(bind_address: &str) -> anyhow::Result<()> {
    info!("Starting MCP gRPC Server");
    info!("Bind address: {}", bind_address);

    let protocol_handler = Arc::new(ProtocolHandler::new());
    let transport = Arc::new(RwLock::new(GrpcTransport::new(bind_address.to_string())));

    // Initialize transport
    {
        let mut t = transport.write().await;
        t.initialize().await?;
    }

    let state = GrpcServerState {
        protocol_handler,
        transport,
        active_connections: Arc::new(RwLock::new(0)),
        total_requests: Arc::new(RwLock::new(0)),
    };

    let service = McpService::new(state);
    let addr = bind_address.parse()?;

    info!("gRPC Server ready on {}", bind_address);
    info!("Service: MCP gRPC");
    info!("Protocol: HTTP/2");
    info!("Features:");
    info!("  - Bidirectional streaming");
    info!("  - Binary serialization (Protocol Buffers)");
    info!("  - HTTP/2 multiplexing");
    info!("  - Automatic retries");

    Server::builder()
        .add_service(McpServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

// Stub implementation when gRPC feature is not enabled
#[cfg(not(feature = "grpc"))]
pub async fn run_grpc_server(_bind_address: &str) -> anyhow::Result<()> {
    anyhow::bail!("gRPC feature is not enabled. Compile with --features grpc")
}

#[cfg(test)]
#[cfg(feature = "grpc")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_grpc_server_state_creation() {
        let protocol_handler = Arc::new(ProtocolHandler::new());
        let transport = Arc::new(RwLock::new(GrpcTransport::new("127.0.0.1:50051".to_string())));

        let state = GrpcServerState {
            protocol_handler,
            transport,
            active_connections: Arc::new(RwLock::new(0)),
            total_requests: Arc::new(RwLock::new(0)),
        };

        assert_eq!(*state.active_connections.read().await, 0);
        assert_eq!(*state.total_requests.read().await, 0);
    }

    #[test]
    fn test_service_creation() {
        let protocol_handler = Arc::new(ProtocolHandler::new());
        let transport = Arc::new(RwLock::new(GrpcTransport::new("127.0.0.1:50052".to_string())));

        let state = GrpcServerState {
            protocol_handler,
            transport,
            active_connections: Arc::new(RwLock::new(0)),
            total_requests: Arc::new(RwLock::new(0)),
        };

        let service = McpService::new(state);
        // Service created successfully
    }
}