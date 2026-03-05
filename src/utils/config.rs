#![allow(dead_code)]

use std::env;

pub struct Config {
    pub host: String,
    pub port: u16,
    #[allow(dead_code)]
    pub rust_log: String,
    #[allow(dead_code)]
    pub jwt_secret: Option<String>,
    #[allow(dead_code)]
    pub postgrest_url: Option<String>,
    #[allow(dead_code)]
    pub db_table_prefix: Option<String>,
    /// MCP V5 API base URL (AI text generation + S3 upload proxy)
    pub v5_api_url: String,
    /// MCP V5 server-to-server API key (X-API-Key header, bypassConsume: true)
    pub v5_api_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8030".to_string())
                .parse()
                .unwrap_or(8030),
            rust_log: env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,mcp_dautruongvui_be=debug".to_string()),
            jwt_secret: env::var("JWT_SECRET").ok(),
            postgrest_url: env::var("POSTGREST_URL").ok(),
            db_table_prefix: env::var("DB_TABLE_PREFIX").ok(),
            v5_api_url: env::var("V5_API_URL")
                .unwrap_or_else(|_| "http://api_v5.ainext.vn".to_string()),
            v5_api_key: env::var("V5_API_KEY").ok(),
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
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
            port: 8030,
            rust_log: "info".to_string(),
            jwt_secret: None,
            postgrest_url: None,
            db_table_prefix: None,
            v5_api_url: "http://api_v5.ainext.vn".to_string(),
            v5_api_key: None,
        }
    }
}