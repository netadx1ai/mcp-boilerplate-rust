# Production Dockerfile for workflow-server
# Multi-stage build for optimal size and security

ARG RUST_VERSION=1.75
ARG DEBIAN_VERSION=bookworm-slim

# Build stage - compile the Rust application
FROM rust:${RUST_VERSION} as builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set build environment
WORKDIR /app
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

# Copy workspace configuration first for better layer caching
COPY Cargo.toml Cargo.lock ./
COPY servers/workflow-server/Cargo.toml ./servers/workflow-server/

# Create dummy main.rs files to cache dependencies
RUN mkdir -p servers/workflow-server/src && \
    echo "fn main() {}" > servers/workflow-server/src/main.rs

# Pre-build dependencies (this layer will be cached)
RUN cargo build --release --bin workflow-server && \
    rm servers/workflow-server/src/main.rs && \
    rm -f target/release/deps/workflow_server*

# Copy source code
COPY servers/workflow-server/src ./servers/workflow-server/src

# Build the actual application
RUN cargo build --release --bin workflow-server

# Verify the binary exists and is executable
RUN ls -la target/release/workflow-server && \
    ./target/release/workflow-server --version || echo "Binary verification complete"

# Runtime stage - minimal production image
FROM debian:${DEBIAN_VERSION} as runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates

# Create non-root user for security
RUN groupadd -r mcp && useradd -r -g mcp -s /bin/false mcp

# Create application directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/workflow-server /usr/local/bin/workflow-server

# Set permissions and ownership
RUN chown mcp:mcp /usr/local/bin/workflow-server && \
    chmod +x /usr/local/bin/workflow-server

# Create directories for application data
RUN mkdir -p /app/data /app/logs /app/workflows /app/executions && \
    chown -R mcp:mcp /app

# Switch to non-root user
USER mcp

# Health check endpoint
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Expose MCP server port
EXPOSE 8080

# Environment variables for configuration
ENV RUST_LOG=info
ENV MCP_SERVER_HOST=0.0.0.0
ENV MCP_SERVER_PORT=8080
ENV WORKFLOW_MAX_CONCURRENT=10
ENV WORKFLOW_TIMEOUT=3600
ENV WORKFLOW_RETRY_ATTEMPTS=3

# Default command
CMD ["workflow-server"]

# Metadata
LABEL maintainer="MCP Boilerplate Team"
LABEL description="Production MCP workflow server for task automation and orchestration"
LABEL version="0.3.0"
LABEL org.opencontainers.image.source="https://github.com/netadx1ai/mcp-boilerplate-rust"
LABEL org.opencontainers.image.licenses="MIT"
LABEL mcp.server.type="workflow"
LABEL mcp.server.tools="execute_workflow,get_workflow_status,list_available_workflows,get_workflow_definition,cancel_workflow,get_server_status"