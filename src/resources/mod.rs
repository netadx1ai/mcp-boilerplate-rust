use rmcp::model::{
    Annotated, RawResource, ReadResourceResult, ResourceContents,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetadata {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: String,
}

#[derive(Clone)]
pub struct ResourceRegistry {
    resources: HashMap<String, ResourceMetadata>,
}

impl ResourceRegistry {
    pub fn new() -> Self {
        let mut resources = HashMap::new();

        resources.insert(
            "config://server".to_string(),
            ResourceMetadata {
                uri: "config://server".to_string(),
                name: "Server Configuration".to_string(),
                description: "Current MCP server configuration and metadata".to_string(),
                mime_type: "application/json".to_string(),
            },
        );

        resources.insert(
            "info://capabilities".to_string(),
            ResourceMetadata {
                uri: "info://capabilities".to_string(),
                name: "Server Capabilities".to_string(),
                description: "List of enabled MCP capabilities (tools, prompts, resources)".to_string(),
                mime_type: "application/json".to_string(),
            },
        );

        resources.insert(
            "doc://quick-start".to_string(),
            ResourceMetadata {
                uri: "doc://quick-start".to_string(),
                name: "Quick Start Guide".to_string(),
                description: "Quick start guide for using this MCP server".to_string(),
                mime_type: "text/plain".to_string(),
            },
        );

        resources.insert(
            "stats://usage".to_string(),
            ResourceMetadata {
                uri: "stats://usage".to_string(),
                name: "Usage Statistics".to_string(),
                description: "Server usage statistics and metrics".to_string(),
                mime_type: "application/json".to_string(),
            },
        );

        Self { resources }
    }

    pub fn list_resources(&self) -> Vec<Annotated<RawResource>> {
        self.resources
            .values()
            .map(|meta| Annotated {
                raw: RawResource {
                    uri: meta.uri.clone(),
                    name: meta.name.clone(),
                    title: None,
                    description: Some(meta.description.clone()),
                    mime_type: Some(meta.mime_type.clone()),
                    size: None,
                    icons: None,
                    meta: None,
                },
                annotations: None,
            })
            .collect()
    }

    pub fn read_resource(&self, uri: &str) -> Result<ReadResourceResult, String> {
        let meta = self
            .resources
            .get(uri)
            .ok_or_else(|| format!("Resource '{}' not found", uri))?;

        let content = self.get_resource_content(uri)?;

        Ok(ReadResourceResult {
            contents: vec![ResourceContents::TextResourceContents {
                uri: uri.to_string(),
                mime_type: Some(meta.mime_type.clone()),
                text: content,
                meta: None,
            }],
        })
    }

    fn get_resource_content(&self, uri: &str) -> Result<String, String> {
        match uri {
            "config://server" => Ok(serde_json::to_string_pretty(&serde_json::json!({
                "name": "mcp-boilerplate-rust",
                "version": env!("CARGO_PKG_VERSION"),
                "protocol": "MCP 2024-11-05",
                "sdk": "rmcp v0.12.0",
                "mode": "stdio",
                "features": {
                    "tools": true,
                    "prompts": true,
                    "resources": true,
                    "logging": false
                },
                "transport": {
                    "type": "stdio",
                    "encoding": "utf-8"
                }
            }))
            .unwrap()),

            "info://capabilities" => Ok(serde_json::to_string_pretty(&serde_json::json!({
                "capabilities": {
                    "tools": {
                        "enabled": true,
                        "count": 5,
                        "available": ["echo", "ping", "info", "calculate", "evaluate"]
                    },
                    "prompts": {
                        "enabled": true,
                        "count": 3,
                        "available": ["code_review", "explain_code", "debug_help"]
                    },
                    "resources": {
                        "enabled": true,
                        "count": 4,
                        "available": ["config://server", "info://capabilities", "doc://quick-start", "stats://usage"]
                    },
                    "logging": {
                        "enabled": false,
                        "reason": "Disabled in stdio mode to prevent JSON interference"
                    }
                },
                "server_info": {
                    "name": env!("CARGO_PKG_NAME"),
                    "version": env!("CARGO_PKG_VERSION")
                }
            }))
            .unwrap()),

            "doc://quick-start" => Ok(format!(
                "MCP Boilerplate Rust - Quick Start\n\
                =====================================\n\n\
                Version: {}\n\
                Protocol: MCP 2024-11-05\n\
                SDK: rmcp v0.12.0\n\n\
                Available Tools:\n\
                - echo: Echo back a message with validation\n\
                - ping: Simple connectivity test\n\
                - info: Get server information\n\
                - calculate: Perform arithmetic operations\n\
                - evaluate: Evaluate mathematical expressions\n\n\
                Available Prompts:\n\
                - code_review: Generate code review prompts\n\
                - explain_code: Generate code explanation prompts\n\
                - debug_help: Generate debugging assistance prompts\n\n\
                Available Resources:\n\
                - config://server: Server configuration\n\
                - info://capabilities: Server capabilities\n\
                - doc://quick-start: This guide\n\
                - stats://usage: Usage statistics\n\n\
                Usage:\n\
                1. Build: cargo build --release\n\
                2. Run: ./target/release/mcp-boilerplate-rust --mode stdio\n\
                3. Configure in Claude Desktop config\n\
                4. Restart Claude Desktop\n\n\
                For more information, visit:\n\
                https://github.com/netadx1ai/mcp-boilerplate-rust\n",
                env!("CARGO_PKG_VERSION")
            )),

            "stats://usage" => {
                let now = chrono::Utc::now();
                Ok(serde_json::to_string_pretty(&serde_json::json!({
                    "server": {
                        "uptime": "N/A (stateless)",
                        "mode": "stdio",
                        "protocol": "MCP 2024-11-05"
                    },
                    "tools": {
                        "total_calls": "N/A (stateless)",
                        "last_used": "N/A"
                    },
                    "resources": {
                        "total_reads": "N/A (stateless)",
                        "last_accessed": "N/A"
                    },
                    "prompts": {
                        "total_requests": "N/A (stateless)",
                        "last_requested": "N/A"
                    },
                    "timestamp": now.to_rfc3339(),
                    "note": "This server is stateless and does not persist usage data"
                }))
                .unwrap())
            }

            _ => Err(format!("Unknown resource URI: {}", uri)),
        }
    }
}

impl Default for ResourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}