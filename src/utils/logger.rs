use tracing::{debug, error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();
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

    pub fn shutdown() {
        // No-op without OpenTelemetry
    }
}