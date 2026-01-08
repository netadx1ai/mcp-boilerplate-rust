#[cfg(feature = "metrics")]
use lazy_static::lazy_static;
#[cfg(feature = "metrics")]
use prometheus::{
    register_counter_vec, register_gauge, register_histogram_vec, CounterVec, Encoder, Gauge,
    HistogramVec, TextEncoder,
};

#[cfg(feature = "metrics")]
lazy_static! {
    // Request metrics
    pub static ref REQUEST_COUNTER: CounterVec = register_counter_vec!(
        "mcp_requests_total",
        "Total number of MCP requests",
        &["transport", "method", "status"]
    )
    .expect("Failed to create REQUEST_COUNTER metric");

    pub static ref REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "mcp_request_duration_seconds",
        "MCP request duration in seconds",
        &["transport", "method"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    )
    .expect("Failed to create REQUEST_DURATION metric");

    // Tool metrics
    pub static ref TOOL_INVOCATIONS: CounterVec = register_counter_vec!(
        "mcp_tool_invocations_total",
        "Total number of tool invocations",
        &["tool_name", "status"]
    )
    .expect("Failed to create TOOL_INVOCATIONS metric");

    pub static ref TOOL_DURATION: HistogramVec = register_histogram_vec!(
        "mcp_tool_duration_seconds",
        "Tool execution duration in seconds",
        &["tool_name"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    )
    .expect("Failed to create TOOL_DURATION metric");

    // Connection metrics
    pub static ref ACTIVE_CONNECTIONS: Gauge = register_gauge!(
        "mcp_active_connections",
        "Number of active connections"
    )
    .expect("Failed to create ACTIVE_CONNECTIONS metric");

    pub static ref TOTAL_CONNECTIONS: CounterVec = register_counter_vec!(
        "mcp_connections_total",
        "Total number of connections",
        &["transport"]
    )
    .expect("Failed to create TOTAL_CONNECTIONS metric");

    // Error metrics
    pub static ref ERROR_COUNTER: CounterVec = register_counter_vec!(
        "mcp_errors_total",
        "Total number of errors",
        &["transport", "error_type"]
    )
    .expect("Failed to create ERROR_COUNTER metric");

    // Notification metrics
    pub static ref NOTIFICATION_COUNTER: CounterVec = register_counter_vec!(
        "mcp_notifications_total",
        "Total number of notifications sent",
        &["notification_type"]
    )
    .expect("Failed to create NOTIFICATION_COUNTER metric");

    // Transport-specific metrics
    pub static ref TRANSPORT_BYTES_SENT: CounterVec = register_counter_vec!(
        "mcp_transport_bytes_sent_total",
        "Total bytes sent by transport",
        &["transport"]
    )
    .expect("Failed to create TRANSPORT_BYTES_SENT metric");

    pub static ref TRANSPORT_BYTES_RECEIVED: CounterVec = register_counter_vec!(
        "mcp_transport_bytes_received_total",
        "Total bytes received by transport",
        &["transport"]
    )
    .expect("Failed to create TRANSPORT_BYTES_RECEIVED metric");
}

#[cfg(feature = "metrics")]
pub fn record_request(transport: &str, method: &str, status: &str, duration_secs: f64) {
    REQUEST_COUNTER
        .with_label_values(&[transport, method, status])
        .inc();
    REQUEST_DURATION
        .with_label_values(&[transport, method])
        .observe(duration_secs);
}

#[cfg(feature = "metrics")]
pub fn record_tool_invocation(tool_name: &str, status: &str, duration_secs: f64) {
    TOOL_INVOCATIONS
        .with_label_values(&[tool_name, status])
        .inc();
    TOOL_DURATION
        .with_label_values(&[tool_name])
        .observe(duration_secs);
}

#[cfg(feature = "metrics")]
pub fn increment_active_connections() {
    ACTIVE_CONNECTIONS.inc();
}

#[cfg(feature = "metrics")]
pub fn decrement_active_connections() {
    ACTIVE_CONNECTIONS.dec();
}

#[cfg(feature = "metrics")]
pub fn record_connection(transport: &str) {
    TOTAL_CONNECTIONS.with_label_values(&[transport]).inc();
}

#[cfg(feature = "metrics")]
pub fn record_error(transport: &str, error_type: &str) {
    ERROR_COUNTER
        .with_label_values(&[transport, error_type])
        .inc();
}

#[cfg(feature = "metrics")]
pub fn record_notification(notification_type: &str) {
    NOTIFICATION_COUNTER
        .with_label_values(&[notification_type])
        .inc();
}

#[cfg(feature = "metrics")]
pub fn record_bytes_sent(transport: &str, bytes: u64) {
    TRANSPORT_BYTES_SENT
        .with_label_values(&[transport])
        .inc_by(bytes as f64);
}

#[cfg(feature = "metrics")]
pub fn record_bytes_received(transport: &str, bytes: u64) {
    TRANSPORT_BYTES_RECEIVED
        .with_label_values(&[transport])
        .inc_by(bytes as f64);
}

#[cfg(feature = "metrics")]
pub fn gather_metrics() -> Result<String, Box<dyn std::error::Error>> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;
    Ok(String::from_utf8(buffer)?)
}

// No-op implementations when metrics feature is disabled
#[cfg(not(feature = "metrics"))]
pub fn record_request(_transport: &str, _method: &str, _status: &str, _duration_secs: f64) {}

#[cfg(not(feature = "metrics"))]
pub fn record_tool_invocation(_tool_name: &str, _status: &str, _duration_secs: f64) {}

#[cfg(not(feature = "metrics"))]
pub fn increment_active_connections() {}

#[cfg(not(feature = "metrics"))]
pub fn decrement_active_connections() {}

#[cfg(not(feature = "metrics"))]
pub fn record_connection(_transport: &str) {}

#[cfg(not(feature = "metrics"))]
pub fn record_error(_transport: &str, _error_type: &str) {}

#[cfg(not(feature = "metrics"))]
pub fn record_notification(_notification_type: &str) {}

#[cfg(not(feature = "metrics"))]
pub fn record_bytes_sent(_transport: &str, _bytes: u64) {}

#[cfg(not(feature = "metrics"))]
pub fn record_bytes_received(_transport: &str, _bytes: u64) {}

#[cfg(not(feature = "metrics"))]
pub fn gather_metrics() -> Result<String, Box<dyn std::error::Error>> {
    Ok("# Metrics not enabled\n".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "metrics")]
    fn test_record_request() {
        record_request("stdio", "tools/call", "success", 0.015);
        let metrics = gather_metrics().unwrap();
        assert!(metrics.contains("mcp_requests_total"));
        assert!(metrics.contains("mcp_request_duration_seconds"));
    }

    #[test]
    #[cfg(feature = "metrics")]
    fn test_record_tool_invocation() {
        record_tool_invocation("echo", "success", 0.002);
        let metrics = gather_metrics().unwrap();
        assert!(metrics.contains("mcp_tool_invocations_total"));
        assert!(metrics.contains("mcp_tool_duration_seconds"));
    }

    #[test]
    #[cfg(feature = "metrics")]
    fn test_connection_metrics() {
        increment_active_connections();
        record_connection("sse");
        let metrics = gather_metrics().unwrap();
        assert!(metrics.contains("mcp_active_connections"));
        assert!(metrics.contains("mcp_connections_total"));
        decrement_active_connections();
    }

    #[test]
    fn test_metrics_disabled() {
        record_request("stdio", "tools/call", "success", 0.015);
        let result = gather_metrics();
        assert!(result.is_ok());
    }
}
