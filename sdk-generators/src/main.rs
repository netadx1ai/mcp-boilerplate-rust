use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use std::fs;
use std::path::Path;

mod generators {
    pub mod rust_gen;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolSchema {
    name: String,
    description: String,
    input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TransportConfig {
    name: String,
    port: Option<u16>,
    protocol: String,
    supports_browser: bool,
}

#[derive(Debug)]
struct SdkGenerator {
    tools: Vec<ToolSchema>,
    transports: Vec<TransportConfig>,
    project_version: String,
}

impl SdkGenerator {
    fn new() -> Self {
        let tools = Self::load_tool_schemas();
        let transports = Self::load_transport_configs();
        
        Self {
            tools,
            transports,
            project_version: "0.4.0".to_string(),
        }
    }

    fn load_tool_schemas() -> Vec<ToolSchema> {
        vec![
            ToolSchema {
                name: "echo".to_string(),
                description: "Echo a message with timestamp validation".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "Message to echo (1-10240 bytes)",
                            "minLength": 1,
                            "maxLength": 10240
                        }
                    },
                    "required": ["message"]
                }),
            },
            ToolSchema {
                name: "ping".to_string(),
                description: "Health check endpoint".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            ToolSchema {
                name: "info".to_string(),
                description: "Get server metadata".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            ToolSchema {
                name: "calculate".to_string(),
                description: "Perform arithmetic operations".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": ["add", "subtract", "multiply", "divide"],
                            "description": "Arithmetic operation to perform"
                        },
                        "a": {
                            "type": "number",
                            "description": "First operand"
                        },
                        "b": {
                            "type": "number",
                            "description": "Second operand"
                        }
                    },
                    "required": ["operation", "a", "b"]
                }),
            },
            ToolSchema {
                name: "evaluate".to_string(),
                description: "Evaluate mathematical expression".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "expression": {
                            "type": "string",
                            "description": "Mathematical expression to evaluate (e.g., '2 * (3 + 4)')"
                        }
                    },
                    "required": ["expression"]
                }),
            },
            ToolSchema {
                name: "process_with_progress".to_string(),
                description: "Process data with progress notifications".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "data": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Array of data items to process"
                        },
                        "delay_ms": {
                            "type": "number",
                            "description": "Delay between items in milliseconds",
                            "default": 100
                        }
                    },
                    "required": ["data"]
                }),
            },
            ToolSchema {
                name: "batch_process".to_string(),
                description: "Batch process items with logging".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "items": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Items to process"
                        },
                        "operation": {
                            "type": "string",
                            "description": "Operation to perform on each item"
                        }
                    },
                    "required": ["items", "operation"]
                }),
            },
            ToolSchema {
                name: "transform_data".to_string(),
                description: "Transform array data".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "data": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Array of strings to transform (max 10000 items)",
                            "maxItems": 10000
                        },
                        "operation": {
                            "type": "string",
                            "enum": ["uppercase", "lowercase", "reverse", "double"],
                            "description": "Transformation operation"
                        }
                    },
                    "required": ["data", "operation"]
                }),
            },
            ToolSchema {
                name: "simulate_upload".to_string(),
                description: "Simulate file upload with progress".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "filename": {
                            "type": "string",
                            "description": "Filename to simulate uploading"
                        },
                        "size_bytes": {
                            "type": "number",
                            "description": "File size in bytes"
                        }
                    },
                    "required": ["filename", "size_bytes"]
                }),
            },
            ToolSchema {
                name: "health_check".to_string(),
                description: "Comprehensive system health check".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            ToolSchema {
                name: "long_task".to_string(),
                description: "Execute long-running task with progress updates".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "duration_seconds": {
                            "type": "number",
                            "description": "Duration in seconds (1-60)",
                            "minimum": 1,
                            "maximum": 60,
                            "default": 10
                        }
                    }
                }),
            },
        ]
    }

    fn load_transport_configs() -> Vec<TransportConfig> {
        vec![
            TransportConfig {
                name: "stdio".to_string(),
                port: None,
                protocol: "jsonrpc".to_string(),
                supports_browser: false,
            },
            TransportConfig {
                name: "sse".to_string(),
                port: Some(8025),
                protocol: "http".to_string(),
                supports_browser: true,
            },
            TransportConfig {
                name: "websocket".to_string(),
                port: Some(9001),
                protocol: "ws".to_string(),
                supports_browser: true,
            },
            TransportConfig {
                name: "http".to_string(),
                port: Some(8080),
                protocol: "http".to_string(),
                supports_browser: true,
            },
            TransportConfig {
                name: "http_stream".to_string(),
                port: Some(8026),
                protocol: "http".to_string(),
                supports_browser: true,
            },
            TransportConfig {
                name: "grpc".to_string(),
                port: Some(50051),
                protocol: "grpc".to_string(),
                supports_browser: false,
            },
        ]
    }

    fn generate_typescript(&self) -> String {
        let mut code = String::new();
        
        code.push_str(&format!(
            "// MCP Client SDK for TypeScript\n\
             // Auto-generated from mcp-boilerplate-rust v{}\n\
             // Do not edit manually\n\n",
            self.project_version
        ));

        code.push_str(
            "export interface McpClientConfig {\n\
             \tbaseUrl?: string;\n\
             \tport?: number;\n\
             \ttransport: 'sse' | 'websocket' | 'http' | 'http-stream' | 'grpc';\n\
             \ttimeout?: number;\n\
             }\n\n"
        );

        code.push_str(
            "export interface McpResponse<T = any> {\n\
             \tsuccess: boolean;\n\
             \tdata?: T;\n\
             \terror?: string;\n\
             }\n\n"
        );

        for tool in &self.tools {
            let type_name = Self::to_pascal_case(&tool.name);
            code.push_str(&format!("export interface {}Request {{\n", type_name));
            
            if let Some(props) = tool.input_schema.get("properties").and_then(|p| p.as_object()) {
                for (key, value) in props {
                    let ts_type = Self::json_type_to_ts(value);
                    let required = tool.input_schema.get("required")
                        .and_then(|r| r.as_array())
                        .map(|arr| arr.iter().any(|v| v.as_str() == Some(key)))
                        .unwrap_or(false);
                    let optional = if required { "" } else { "?" };
                    
                    if let Some(desc) = value.get("description").and_then(|d| d.as_str()) {
                        code.push_str(&format!("\t/** {} */\n", desc));
                    }
                    code.push_str(&format!("\t{}{}: {};\n", key, optional, ts_type));
                }
            }
            
            code.push_str("}\n\n");
        }

        code.push_str(
            "export class McpClient {\n\
             \tprivate config: McpClientConfig;\n\
             \tprivate baseUrl: string;\n\n\
             \tconstructor(config: McpClientConfig) {\n\
             \t\tthis.config = config;\n\
             \t\tconst port = config.port || this.getDefaultPort(config.transport);\n\
             \t\tthis.baseUrl = config.baseUrl || `http://127.0.0.1:${port}`;\n\
             \t}\n\n\
             \tprivate getDefaultPort(transport: string): number {\n"
        );

        for transport in &self.transports {
            if let Some(port) = transport.port {
                code.push_str(&format!(
                    "\t\tif (transport === '{}') return {};\n",
                    transport.name, port
                ));
            }
        }

        code.push_str(
            "\t\treturn 8080;\n\
             \t}\n\n"
        );

        for tool in &self.tools {
            let method_name = &tool.name;
            let type_name = Self::to_pascal_case(&tool.name);
            let has_params = tool.input_schema.get("properties")
                .and_then(|p| p.as_object())
                .map(|obj| !obj.is_empty())
                .unwrap_or(false);

            code.push_str(&format!(
                "\t/**\n\
                 \t * {}\n\
                 \t */\n",
                tool.description
            ));

            if has_params {
                code.push_str(&format!(
                    "\tasync {}(params: {}Request): Promise<McpResponse> {{\n",
                    method_name, type_name
                ));
            } else {
                code.push_str(&format!(
                    "\tasync {}(): Promise<McpResponse> {{\n",
                    method_name
                ));
            }

            code.push_str(&format!(
                "\t\tconst args = {};\n\
                 \t\treturn this.callTool('{}', args);\n\
                 \t}}\n\n",
                if has_params { "params" } else { "{}" },
                method_name
            ));
        }

        code.push_str(
            "\tprivate async callTool(name: string, args: any): Promise<McpResponse> {\n\
             \t\tconst response = await fetch(`${this.baseUrl}/tools/call`, {\n\
             \t\t\tmethod: 'POST',\n\
             \t\t\theaders: { 'Content-Type': 'application/json' },\n\
             \t\t\tbody: JSON.stringify({\n\
             \t\t\t\tjsonrpc: '2.0',\n\
             \t\t\t\tid: Date.now(),\n\
             \t\t\t\tmethod: 'tools/call',\n\
             \t\t\t\tparams: { name, arguments: args }\n\
             \t\t\t})\n\
             \t\t});\n\n\
             \t\tif (!response.ok) {\n\
             \t\t\treturn { success: false, error: `HTTP ${response.status}` };\n\
             \t\t}\n\n\
             \t\tconst data = await response.json();\n\
             \t\tif (data.error) {\n\
             \t\t\treturn { success: false, error: data.error.message };\n\
             \t\t}\n\n\
             \t\treturn { success: true, data: data.result };\n\
             \t}\n\
             }\n"
        );

        code
    }

    fn generate_python(&self) -> String {
        let mut code = String::new();
        
        code.push_str(&format!(
            "\"\"\"MCP Client SDK for Python\n\
             Auto-generated from mcp-boilerplate-rust v{}\n\
             Do not edit manually\n\
             \"\"\"\n\n\
             from typing import Optional, Dict, Any, List\n\
             from dataclasses import dataclass\n\
             import requests\n\
             import json\n\n",
            self.project_version
        ));

        code.push_str(
            "@dataclass\n\
             class McpClientConfig:\n\
             \tbase_url: Optional[str] = None\n\
             \tport: Optional[int] = None\n\
             \ttransport: str = 'http'\n\
             \ttimeout: int = 30\n\n\n\
             @dataclass\n\
             class McpResponse:\n\
             \tsuccess: bool\n\
             \tdata: Optional[Dict[str, Any]] = None\n\
             \terror: Optional[str] = None\n\n\n"
        );

        code.push_str(
            "class McpClient:\n\
             \t\"\"\"MCP Client for Python\"\"\"\n\n\
             \tDEFAULT_PORTS = {\n"
        );

        for transport in &self.transports {
            if let Some(port) = transport.port {
                code.push_str(&format!("\t\t'{}': {},\n", transport.name, port));
            }
        }

        code.push_str(
            "\t}\n\n\
             \tdef __init__(self, config: McpClientConfig):\n\
             \t\tself.config = config\n\
             \t\tport = config.port or self.DEFAULT_PORTS.get(config.transport, 8080)\n\
             \t\tself.base_url = config.base_url or f'http://127.0.0.1:{port}'\n\
             \t\tself.session = requests.Session()\n\n"
        );

        for tool in &self.tools {
            let method_name = &tool.name;
            let has_params = tool.input_schema.get("properties")
                .and_then(|p| p.as_object())
                .map(|obj| !obj.is_empty())
                .unwrap_or(false);

            code.push_str(&format!(
                "\tdef {}(self{}",
                method_name,
                if has_params { ", **kwargs" } else { "" }
            ));

            code.push_str(") -> McpResponse:\n");
            code.push_str(&format!("\t\t\"\"\"{}\"\"\"\n", tool.description));

            if has_params {
                let line = format!("\t\treturn self._call_tool('{}', kwargs)\n\n", method_name);
                code.push_str(&line);
            } else {
                let line = format!("\t\treturn self._call_tool('{}', {{}})\n\n", method_name);
                code.push_str(&line);
            }
        }

        code.push_str(
            "\tdef _call_tool(self, name: str, args: Dict[str, Any]) -> McpResponse:\n\
             \t\t\"\"\"Internal method to call a tool\"\"\"\n\
             \t\tpayload = {\n\
             \t\t\t'jsonrpc': '2.0',\n\
             \t\t\t'id': 1,\n\
             \t\t\t'method': 'tools/call',\n\
             \t\t\t'params': {'name': name, 'arguments': args}\n\
             \t\t}\n\n\
             \t\ttry:\n\
             \t\t\tresponse = self.session.post(\n\
             \t\t\t\tf'{self.base_url}/tools/call',\n\
             \t\t\t\tjson=payload,\n\
             \t\t\t\ttimeout=self.config.timeout\n\
             \t\t\t)\n\
             \t\t\tresponse.raise_for_status()\n\
             \t\t\tdata = response.json()\n\n\
             \t\t\tif 'error' in data:\n\
             \t\t\t\treturn McpResponse(success=False, error=data['error'].get('message'))\n\n\
             \t\t\treturn McpResponse(success=True, data=data.get('result'))\n\n\
             \t\texcept requests.RequestException as e:\n\
             \t\t\treturn McpResponse(success=False, error=str(e))\n"
        );

        code
    }

    fn generate_go(&self) -> String {
        let mut code = String::new();
        
        code.push_str(&format!(
            "// MCP Client SDK for Go\n\
             // Auto-generated from mcp-boilerplate-rust v{}\n\
             // Do not edit manually\n\n\
             package mcpclient\n\n\
             import (\n\
             \t\"bytes\"\n\
             \t\"encoding/json\"\n\
             \t\"fmt\"\n\
             \t\"net/http\"\n\
             \t\"time\"\n\
             )\n\n",
            self.project_version
        ));

        code.push_str(
            "type Config struct {\n\
             \tBaseURL   string\n\
             \tPort      int\n\
             \tTransport string\n\
             \tTimeout   time.Duration\n\
             }\n\n\
             type Response struct {\n\
             \tSuccess bool                   `json:\"success\"`\n\
             \tData    map[string]interface{} `json:\"data,omitempty\"`\n\
             \tError   string                 `json:\"error,omitempty\"`\n\
             }\n\n\
             type Client struct {\n\
             \tconfig     Config\n\
             \tbaseURL    string\n\
             \thttpClient *http.Client\n\
             }\n\n"
        );

        code.push_str(
            "func NewClient(config Config) *Client {\n\
             \tif config.Timeout == 0 {\n\
             \t\tconfig.Timeout = 30 * time.Second\n\
             \t}\n\n\
             \tport := config.Port\n\
             \tif port == 0 {\n\
             \t\tport = getDefaultPort(config.Transport)\n\
             \t}\n\n\
             \tbaseURL := config.BaseURL\n\
             \tif baseURL == \"\" {\n\
             \t\tbaseURL = fmt.Sprintf(\"http://127.0.0.1:%d\", port)\n\
             \t}\n\n\
             \treturn &Client{\n\
             \t\tconfig:  config,\n\
             \t\tbaseURL: baseURL,\n\
             \t\thttpClient: &http.Client{\n\
             \t\t\tTimeout: config.Timeout,\n\
             \t\t},\n\
             \t}\n\
             }\n\n\
             func getDefaultPort(transport string) int {\n\
             \tswitch transport {\n"
        );

        for transport in &self.transports {
            if let Some(port) = transport.port {
                code.push_str(&format!(
                    "\tcase \"{}\":\n\
                     \t\treturn {}\n",
                    transport.name, port
                ));
            }
        }

        code.push_str(
            "\tdefault:\n\
             \t\treturn 8080\n\
             \t}\n\
             }\n\n"
        );

        for tool in &self.tools {
            let method_name = Self::to_pascal_case(&tool.name);
            
            code.push_str(&format!(
                "// {} - {}\n",
                method_name, tool.description
            ));
            
            code.push_str(&format!(
                "func (c *Client) {}(args map[string]interface{{}}) (*Response, error) {{\n\
                 \treturn c.callTool(\"{}\", args)\n\
                 }}\n\n",
                method_name, tool.name
            ));
        }

        code.push_str(
            "func (c *Client) callTool(name string, args map[string]interface{}) (*Response, error) {\n\
             \tpayload := map[string]interface{}{\n\
             \t\t\"jsonrpc\": \"2.0\",\n\
             \t\t\"id\":      1,\n\
             \t\t\"method\":  \"tools/call\",\n\
             \t\t\"params\": map[string]interface{}{\n\
             \t\t\t\"name\":      name,\n\
             \t\t\t\"arguments\": args,\n\
             \t\t},\n\
             \t}\n\n\
             \tjsonData, err := json.Marshal(payload)\n\
             \tif err != nil {\n\
             \t\treturn nil, fmt.Errorf(\"marshal error: %w\", err)\n\
             \t}\n\n\
             \tresp, err := c.httpClient.Post(\n\
             \t\tfmt.Sprintf(\"%s/tools/call\", c.baseURL),\n\
             \t\t\"application/json\",\n\
             \t\tbytes.NewBuffer(jsonData),\n\
             \t)\n\
             \tif err != nil {\n\
             \t\treturn nil, fmt.Errorf(\"request error: %w\", err)\n\
             \t}\n\
             \tdefer resp.Body.Close()\n\n\
             \tvar result map[string]interface{}\n\
             \tif err := json.NewDecoder(resp.Body).Decode(&result); err != nil {\n\
             \t\treturn nil, fmt.Errorf(\"decode error: %w\", err)\n\
             \t}\n\n\
             \tif errData, ok := result[\"error\"].(map[string]interface{}); ok {\n\
             \t\treturn &Response{\n\
             \t\t\tSuccess: false,\n\
             \t\t\tError:   errData[\"message\"].(string),\n\
             \t\t}, nil\n\
             \t}\n\n\
             \treturn &Response{\n\
             \t\tSuccess: true,\n\
             \t\tData:    result[\"result\"].(map[string]interface{}),\n\
             \t}, nil\n\
             }\n"
        );

        code
    }

    fn to_pascal_case(s: &str) -> String {
        s.split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect()
    }

    fn json_type_to_ts(value: &Value) -> String {
        if let Some(type_str) = value.get("type").and_then(|t| t.as_str()) {
            match type_str {
                "string" => "string".to_string(),
                "number" => "number".to_string(),
                "boolean" => "boolean".to_string(),
                "array" => {
                    if let Some(items) = value.get("items") {
                        format!("{}[]", Self::json_type_to_ts(items))
                    } else {
                        "any[]".to_string()
                    }
                }
                "object" => "Record<string, any>".to_string(),
                _ => "any".to_string(),
            }
        } else if value.get("enum").is_some() {
            "string".to_string()
        } else {
            "any".to_string()
        }
    }

    fn write_to_file(&self, content: &str, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    fn generate_all(&self) -> std::io::Result<()> {
        let output_dir = Path::new("output");
        
        let ts_code = self.generate_typescript();
        self.write_to_file(&ts_code, &output_dir.join("typescript/mcp-client.ts"))?;
        
        let py_code = self.generate_python();
        self.write_to_file(&py_code, &output_dir.join("python/mcp_client.py"))?;
        
        let go_code = self.generate_go();
        self.write_to_file(&go_code, &output_dir.join("go/mcpclient/client.go"))?;
        
        // Generate Rust SDK (Race Car Edition 🏎️)
        self.save_rust_sdk("output")?;
        
        println!("SDK generation complete!");
        println!("  - TypeScript: output/typescript/mcp-client.ts");
        println!("  - Python:     output/python/mcp_client.py");
        println!("  - Go:         output/go/mcpclient/client.go");
        println!("  - Rust:       output/rust/mcp_client.rs (Race Car Edition 🏎️)");
        
        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    println!("MCP Boilerplate Rust - SDK Generator");
    println!("=====================================\n");
    
    let generator = SdkGenerator::new();
    
    println!("Loaded {} tools", generator.tools.len());
    println!("Loaded {} transports", generator.transports.len());
    println!();
    
    generator.generate_all()?;
    
    Ok(())
}