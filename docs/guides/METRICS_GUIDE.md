# Prometheus Metrics Guide

**Version 0.4.0**

This guide details the Prometheus metrics implementation in the MCP Boilerplate Rust server. The system provides comprehensive observability into server performance, tool usage, and transport statistics.

## Overview

The metrics system tracks key performance indicators (KPIs) across all transport modes:
- **Request Latency**: Duration of JSON-RPC requests.
- **Tool Performance**: Execution time and success/failure rates for all 11 tools.
- **Connection Tracking**: Active connections for stateful transports (WebSocket, SSE, HTTP Streaming).
- **Traffic Stats**: Bytes sent and received.
- **Error Rates**: Classification of errors by transport.

## Configuration

Metrics support is hidden behind the `metrics` feature flag to keep the base binary small. It is included in the `full` feature set.

### Build with Metrics

```bash
# Build with all features (includes metrics)
cargo build --release --features full

# Build specifically with metrics
cargo build --release --features "sse,metrics"
```

## Accessing Metrics

The metrics endpoint is available at `/metrics` on all HTTP-based transports.

### Endpoints

| Transport | Default Port | Metrics URL |
|-----------|--------------|-------------|
| SSE | 8025 | `http://127.0.0.1:8025/metrics` |
| WebSocket | 9001 | `http://127.0.0.1:9001/metrics` |
| HTTP Stream | 8026 | `http://127.0.0.1:8026/metrics` |
| gRPC | 50051 | N/A (Uses gRPC interceptors, metrics logic shared) |
| Stdio | N/A | Metrics recorded internally but requires external exporter |

### Response Format

The endpoint returns standard Prometheus text format:

```text
# HELP mcp_requests_total Total number of MCP requests
# TYPE mcp_requests_total counter
mcp_requests_total{method="tools/call",status="success",transport="json_rpc"} 12
# HELP mcp_active_connections Number of active connections
# TYPE mcp_active_connections gauge
mcp_active_connections 5
```

## Metric Reference

### Request Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mcp_requests_total` | Counter | `transport`, `method`, `status` | Total JSON-RPC requests processed. |
| `mcp_request_duration_seconds` | Histogram | `transport`, `method` | Latency distribution of requests. |

### Tool Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mcp_tool_invocations_total` | Counter | `tool_name`, `status` | Total execution count per tool. |
| `mcp_tool_duration_seconds` | Histogram | `tool_name` | Execution time distribution per tool. |

### Connection Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mcp_active_connections` | Gauge | None | Current number of active WebSocket/SSE/Stream connections. |
| `mcp_connections_total` | Counter | `transport` | Total cumulative connections accepted. |

### Traffic & Errors

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mcp_errors_total` | Counter | `transport`, `error_type` | Total errors encountered. |
| `mcp_transport_bytes_sent_total` | Counter | `transport` | Total bytes sent (gRPC only). |
| `mcp_transport_bytes_received_total` | Counter | `transport` | Total bytes received (gRPC only). |

## Integration

### Prometheus Configuration

Add the following job to your `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'mcp_server'
    scrape_interval: 15s
    static_configs:
      - targets: ['localhost:8025']  # SSE port
      - targets: ['localhost:9001']  # WebSocket port
      - targets: ['localhost:8026']  # HTTP Stream port
```

### Grafana Dashboard Queries

**Request Rate (RPS):**
```promql
rate(mcp_requests_total[5m])
```

**99th Percentile Latency:**
```promql
histogram_quantile(0.99, rate(mcp_request_duration_seconds_bucket[5m]))
```

**Tool Error Rate:**
```promql
sum(rate(mcp_tool_invocations_total{status="error"}[5m])) by (tool_name)
```

**Active Connections:**
```promql
mcp_active_connections
```

## Testing

To verify metrics are working:

1. **Start the server:**
   ```bash
   cargo run --features full -- --mode sse
   ```

2. **Generate some traffic:**
   ```bash
   curl -X POST http://localhost:8025/rpc -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","id":1,"method":"ping"}'
   ```

3. **Check metrics:**
   ```bash
   curl http://localhost:8025/metrics
   ```

## Implementation Details

The metrics system uses the `prometheus` crate with `lazy_static` for global registry access. Instrumentation is applied at three layers:

1. **Transport Layer**: Tracks connections and raw bytes (where applicable).
2. **Protocol Handler**: Tracks JSON-RPC methods and generic request flow.
3. **Tool Implementation**: Tracks specific tool execution times and results.

This layered approach ensures that even if a request fails at the protocol level (e.g., invalid JSON), it is still captured in request metrics, while tool metrics specifically track successful dispatch to tools.