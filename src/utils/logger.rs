use tracing::{debug, error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "otel")]
use opentelemetry::{global, KeyValue};
#[cfg(feature = "otel")]
use opentelemetry_otlp::WithExportConfig;
#[cfg(feature = "otel")]
use opentelemetry_sdk::{trace as sdktrace, Resource};
#[cfg(feature = "otel")]
use tracing_opentelemetry::OpenTelemetryLayer;

pub struct Logger;

impl Logger {
    pub fn init() {
        // Create the fmt layer (logging to stderr to avoid interfering with JSON stdout)
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_target(false)
            .with_writer(std::io::stderr);

        // Create the env filter
        let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "error".into());

        // Base registry
        let registry = tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer);

        // Add OpenTelemetry layer if enabled and configured
        #[cfg(feature = "otel")]
        {
            // Check if OTEL endpoint is configured to decide whether to enable tracing
            if std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_ok() {
                match init_tracer() {
                    Ok(tracer) => {
                        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
                        registry.with(telemetry).init();
                        return;
                    }
                    Err(e) => {
                        eprintln!("Failed to initialize OpenTelemetry tracer: {}", e);
                    }
                }
            }
        }

        registry.init();
    }

    #[allow(dead_code)]
    pub fn info(message: &str) {
        info!("{}", message);
    }

    #[allow(dead_code)]
    pub fn warn(message: &str) {
        warn!("{}", message);
    }

    #[allow(dead_code)]
    pub fn error(message: &str) {
        error!("{}", message);
    }

    #[allow(dead_code)]
    pub fn debug(message: &str) {
        debug!("{}", message);
    }

    // Helper to shutdown tracer provider to ensure spans are exported
    #[cfg(feature = "otel")]
    pub fn shutdown() {
        global::shutdown_tracer_provider();
    }

    #[cfg(not(feature = "otel"))]
    pub fn shutdown() {}
}

#[cfg(feature = "otel")]
fn init_tracer() -> anyhow::Result<opentelemetry_sdk::trace::Tracer> {
    let service_name = std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "mcp-boilerplate-rust".to_string());
    
    // Create the exporter
    // Note: We use tonic (gRPC) as the default protocol
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic();

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            sdktrace::config().with_resource(Resource::new(vec![
                KeyValue::new("service.name", service_name),
            ])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .map_err(|e| anyhow::anyhow!(e))
}