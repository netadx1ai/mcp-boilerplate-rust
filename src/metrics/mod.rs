//! Metrics module - no-op stubs for DTV backend
//!
//! Prometheus/OpenTelemetry removed. These functions exist so callers
//! don't need conditional compilation.
#![allow(dead_code)]

#[inline]
pub fn record_request(_transport: &str, _method: &str, _status: &str, _duration_secs: f64) {}

#[inline]
pub fn record_tool_invocation(_tool_name: &str, _status: &str, _duration_secs: f64) {}

#[inline]
pub fn increment_active_connections() {}

#[inline]
pub fn decrement_active_connections() {}

#[inline]
pub fn record_connection(_transport: &str) {}

#[inline]
pub fn record_error(_transport: &str, _error_type: &str) {}

#[inline]
pub fn record_notification(_notification_type: &str) {}

#[inline]
pub fn record_bytes_sent(_transport: &str, _bytes: u64) {}

#[inline]
pub fn record_bytes_received(_transport: &str, _bytes: u64) {}

pub fn gather_metrics() -> Result<String, Box<dyn std::error::Error>> {
    Ok("# Metrics not enabled\n".to_string())
}