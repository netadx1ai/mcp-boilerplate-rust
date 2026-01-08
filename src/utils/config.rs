#[cfg(feature = "http")]
use std::env;
#[cfg(feature = "http")]
use anyhow::Result;

pub struct Config {
    pub host: String,
    pub port: u16,
    #[allow(dead_code)]
    pub rust_log: String,
    #[allow(dead_code)]
    pub mongodb_uri: Option<String>,
    #[allow(dead_code)]
    pub mongodb_database: Option<String>,
    #[allow(dead_code)]
    pub jwt_secret: Option<String>,
}

impl Config {
    #[cfg(feature = "http")]
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8025".to_string())
                .parse()
                .unwrap_or(8025),
            rust_log: env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,mcp_boilerplate_rust=debug".to_string()),
            mongodb_uri: env::var("MONGODB_URI").ok(),
            mongodb_database: env::var("MONGODB_DATABASE").ok(),
            jwt_secret: env::var("JWT_SECRET").ok(),
        }
    }

    #[cfg(feature = "http")]
    pub fn validate(&self) -> Result<()> {
        if self.port == 0 {
            anyhow::bail!("Invalid port number");
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn server_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8025,
            rust_log: "info".to_string(),
            mongodb_uri: None,
            mongodb_database: None,
            jwt_secret: None,
        }
    }
}