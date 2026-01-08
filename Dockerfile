# MCP Boilerplate Rust - Dockerfile
# Multi-stage build for optimized production image

# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY src ./src

# Build the application
RUN touch src/main.rs && \
    cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/mcp-boilerplate-rust /usr/local/bin/mcp-server

# Create non-root user
RUN useradd -m -u 1000 mcp && \
    chown -R mcp:mcp /app

USER mcp

# Environment
ENV RUST_LOG=info
ENV HOST=0.0.0.0
ENV PORT=8025

# Expose port
EXPOSE 8025

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8025/health || exit 1

# Run the binary
CMD ["mcp-server"]