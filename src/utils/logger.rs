use tracing::{info, warn, error, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct Logger;

impl Logger {
    pub fn init() {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "error".into()),
            )
            .with(
                tracing_subscriber::fmt::layer()
                    .with_ansi(false)
                    .with_target(false)
            )
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
}