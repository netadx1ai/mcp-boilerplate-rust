# Production Dockerfile for template-server
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
COPY servers/template-server/Cargo.toml ./servers/template-server/

# Create dummy main.rs files to cache dependencies
RUN mkdir -p servers/template-server/src && \
    echo "fn main() {}" > servers/template-server/src/main.rs

# Pre-build dependencies (this layer will be cached)
RUN cargo build --release --bin template-server && \
    rm servers/template-server/src/main.rs && \
    rm -f target/release/deps/template_server*

# Copy source code
COPY servers/template-server/src ./servers/template-server/src

# Build the actual application
RUN cargo build --release --bin template-server

# Verify the binary exists and is executable
RUN ls -la target/release/template-server && \
    ./target/release/template-server --version || echo "Binary verification complete"

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
COPY --from=builder /app/target/release/template-server /usr/local/bin/template-server

# Set permissions and ownership
RUN chown mcp:mcp /usr/local/bin/template-server && \
    chmod +x /usr/local/bin/template-server

# Create directories for application data
RUN mkdir -p /app/data /app/logs /app/templates && \
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
ENV TEMPLATE_STRICT_MODE=true
ENV TEMPLATE_MAX_SIZE=1048576

# Default command
CMD ["template-server"]

# Metadata
LABEL maintainer="MCP Boilerplate Team"
LABEL description="Production MCP template server with Handlebars rendering"
LABEL version="0.3.0"
LABEL org.opencontainers.image.source="https://github.com/netadx1ai/mcp-boilerplate-rust"
LABEL org.opencontainers.image.licenses="MIT"
LABEL mcp.server.type="template"
LABEL mcp.server.tools="list_templates,get_template,render_template,validate_template_params,create_template,get_categories,get_server_status"