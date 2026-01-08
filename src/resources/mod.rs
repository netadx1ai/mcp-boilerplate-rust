use rmcp::model::{
    Annotated, Annotations, Icon, RawResource, ReadResourceResult, ResourceContents, Role,
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
                description: "List of enabled MCP capabilities (tools, prompts, resources)"
                    .to_string(),
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
            .map(|meta| {
                let (icons, annotations) = match meta.uri.as_str() {
                    "config://server" => (
                        Some(vec![Icon {
                            src: "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiPjxjaXJjbGUgY3g9IjEyIiBjeT0iMTIiIHI9IjMiLz48cGF0aCBkPSJNMTIgMXYyIi8+PHBhdGggZD0iTTEyIDIxdjIiLz48cGF0aCBkPSJNNC4yMiA0LjIybDEuNDIgMS40MiIvPjxwYXRoIGQ9Ik0xOC4zNiAxOC4zNmwxLjQyIDEuNDIiLz48cGF0aCBkPSJNMSAxMmgyIi8+PHBhdGggZD0iTTIxIDEyaDIiLz48cGF0aCBkPSJNNC4yMiAxOS43OGwxLjQyLTEuNDIiLz48cGF0aCBkPSJNMTguMzYgNS42NGwxLjQyLTEuNDIiLz48L3N2Zz4=".to_string(),
                            mime_type: Some("image/svg+xml".to_string()),
                            sizes: Some(vec!["any".to_string()]),
                        }]),
                        Some(Annotations {
                            audience: Some(vec![Role::User]),
                            priority: Some(0.9),
                            last_modified: Some(chrono::Utc::now()),
                        }),
                    ),
                    "info://capabilities" => (
                        Some(vec![Icon {
                            src: "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiPjxjaXJjbGUgY3g9IjEyIiBjeT0iMTIiIHI9IjEwIi8+PGxpbmUgeDE9IjEyIiB5MT0iMTYiIHgyPSIxMiIgeTI9IjEyIi8+PGxpbmUgeDE9IjEyIiB5MT0iOCIgeDI9IjEyLjAxIiB5Mj0iOCIvPjwvc3ZnPg==".to_string(),
                            mime_type: Some("image/svg+xml".to_string()),
                            sizes: Some(vec!["any".to_string()]),
                        }]),
                        Some(Annotations {
                            audience: Some(vec![Role::User, Role::Assistant]),
                            priority: Some(0.8),
                            last_modified: Some(chrono::Utc::now()),
                        }),
                    ),
                    "doc://quick-start" => (
                        Some(vec![Icon {
                            src: "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiPjxwYXRoIGQ9Ik0yIDNoNmEyIDIgMCAwIDEgMiAydjE0YTIgMiAwIDAgMS0yIDJIMnYtMnYtMnYtMnYtMnYtMnYtMnYtMnYtMnoiLz48cGF0aCBkPSJNMjIgM2gtNmEyIDIgMCAwIDAtMiAydjE0YTIgMiAwIDAgMCAyIDJoNnYtMnYtMnYtMnYtMnYtMnYtMnYtMnYtMnoiLz48L3N2Zz4=".to_string(),
                            mime_type: Some("image/svg+xml".to_string()),
                            sizes: Some(vec!["any".to_string()]),
                        }]),
                        Some(Annotations {
                            audience: Some(vec![Role::User]),
                            priority: Some(0.7),
                            last_modified: Some(chrono::Utc::now()),
                        }),
                    ),
                    "stats://usage" => (
                        Some(vec![Icon {
                            src: "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiPjxsaW5lIHgxPSIxMiIgeTE9IjIwIiB4Mj0iMTIiIHkyPSIxMCIvPjxsaW5lIHgxPSIxOCIgeTE9IjIwIiB4Mj0iMTgiIHkyPSI0Ii8+PGxpbmUgeDE9IjYiIHkxPSIyMCIgeDI9IjYiIHkyPSIxNiIvPjwvc3ZnPg==".to_string(),
                            mime_type: Some("image/svg+xml".to_string()),
                            sizes: Some(vec!["any".to_string()]),
                        }]),
                        Some(Annotations {
                            audience: Some(vec![Role::User]),
                            priority: Some(0.5),
                            last_modified: Some(chrono::Utc::now()),
                        }),
                    ),
                    _ => (None, None),
                };

                Annotated {
                    raw: RawResource {
                        uri: meta.uri.clone(),
                        name: meta.name.clone(),
                        title: None,
                        description: Some(meta.description.clone()),
                        mime_type: Some(meta.mime_type.clone()),
                        size: None,
                        icons,
                        meta: None,
                    },
                    annotations,
                }
            })
            .collect()
    }

    pub fn read_resource(&self, uri: &str) -> Result<ReadResourceResult, String> {
        let meta = self
            .resources
            .get(uri)
            .ok_or_else(|| format!("Resource '{uri}' not found"))?;

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
                "protocol": "MCP 2025-03-26",
                "sdk": "rmcp (local)",
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
                Protocol: MCP 2025-03-26\n\
                SDK: rmcp (local)\n\n\
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
                        "protocol": "MCP 2025-03-26"
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

            _ => Err(format!("Unknown resource URI: {uri}")),
        }
    }
}

impl Default for ResourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
