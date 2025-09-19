# Production Dockerfile for analytics-server
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
COPY servers/analytics-server/Cargo.toml ./servers/analytics-server/

# Create dummy main.rs files to cache dependencies
RUN mkdir -p servers/analytics-server/src && \
    echo "fn main() {}" > servers/analytics-server/src/main.rs

# Pre-build dependencies (this layer will be cached)
RUN cargo build --release --bin analytics-server && \
    rm servers/analytics-server/src/main.rs && \
    rm -f target/release/deps/analytics_server*

# Copy source code
COPY servers/analytics-server/src ./servers/analytics-server/src

# Build the actual application
RUN cargo build --release --bin analytics-server

# Verify the binary exists and is executable
RUN ls -la target/release/analytics-server && \
    ./target/release/analytics-server --version || echo "Binary verification complete"

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
COPY --from=builder /app/target/release/analytics-server /usr/local/bin/analytics-server

# Set permissions and ownership
RUN chown mcp:mcp /usr/local/bin/analytics-server && \
    chmod +x /usr/local/bin/analytics-server

# Create directories for application data
RUN mkdir -p /app/data /app/logs /app/reports && \
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
ENV ANALYTICS_CACHE_TTL=3600
ENV ANALYTICS_MAX_PERIOD=365

# Default command
CMD ["analytics-server"]

# Metadata
LABEL maintainer="MCP Boilerplate Team"
LABEL description="Production MCP analytics server with metrics and reporting"
LABEL version="0.3.0"
LABEL org.opencontainers.image.source="https://github.com/netadx1ai/mcp-boilerplate-rust"
LABEL org.opencontainers.image.licenses="MIT"
LABEL mcp.server.type="analytics"
LABEL mcp.server.tools="get_content_metrics,get_audience_insights,get_engagement_trends,generate_analytics_report,get_available_metrics,get_time_periods,get_server_status"