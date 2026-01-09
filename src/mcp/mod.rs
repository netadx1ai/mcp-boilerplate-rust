pub mod protocol_handler;
pub mod stdio_server;
pub mod tasks;
pub mod elicitation;
pub mod sampling;
pub mod structured_content;

#[cfg(test)]
mod integration_tests;

#[cfg(feature = "sse")]
pub mod sse_server;

#[cfg(feature = "websocket")]
pub mod websocket_server;

#[cfg(feature = "http-stream")]
pub mod http_stream_server;

#[cfg(feature = "grpc")]
pub mod grpc_server;

pub use stdio_server::McpServer;

#[cfg(feature = "sse")]
pub use sse_server::run_sse_server;

#[cfg(feature = "websocket")]
pub use websocket_server::create_router as create_websocket_router;

#[cfg(feature = "http-stream")]
pub use http_stream_server::run_http_stream_server;

#[cfg(feature = "grpc")]
pub use grpc_server::run_grpc_server;

pub use tasks::{
    CreateTaskRequest,
    Task,
    TaskError,
    TaskManager,
    TaskStatus,
    TaskSupport,
    TasksCancelRequest,
    TasksCancelResponse,
    TasksGetRequest,
    TasksGetResponse,
    TasksListRequest,
    TasksListResponse,
    TasksResultRequest,
    TasksResultResponse,
    ToolExecutionConfig,
};

pub use elicitation::{
    ElicitationFormBuilder,
    ElicitationManager,
    ElicitationMode,
    ElicitationRequest,
    ElicitationResponse,
    ElicitationStatus,
    PendingElicitation,
};

pub use sampling::{
    SamplingRequest,
    SamplingRequestBuilder,
    SamplingResponse,
    SamplingSession,
    SamplingTool,
    ToolCall,
    ToolCallResult,
    ToolChoice,
    ToolExecutor,
    ToolExecutorRegistry,
};

pub use structured_content::{
    OutputSchemaRegistry,
    OutputSchemas,
    OutputValidator,
    StructuredOutput,
    ValidationError,
    ValidationResult,
};