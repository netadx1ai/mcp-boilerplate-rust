//! Tool registry implementation for managing MCP tools.

use crate::error::{ServerError, ServerResult};
use mcp_core::McpTool;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Tool registry for managing MCP tool instances
///
/// The registry provides thread-safe storage and lookup for MCP tools,
/// allowing dynamic registration and management of tool implementations.
pub struct ToolRegistry {
    /// Map of tool name to tool implementation
    tools: HashMap<String, Arc<dyn McpTool>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry
    ///
    /// # Returns
    ///
    /// A new tool registry instance
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool with the registry
    ///
    /// # Arguments
    ///
    /// * `tool` - The tool implementation to register
    ///
    /// # Returns
    ///
    /// Result indicating success or failure of registration
    pub fn register_tool(&mut self, tool: Arc<dyn McpTool>) -> ServerResult<()> {
        let name = tool.name().to_string();

        if self.tools.contains_key(&name) {
            warn!("Tool '{}' is already registered, replacing existing", name);
        }

        self.tools.insert(name.clone(), tool);
        info!("Registered tool: {}", name);

        Ok(())
    }

    /// Unregister a tool by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tool to unregister
    ///
    /// # Returns
    ///
    /// Result indicating success or failure of unregistration
    pub fn unregister_tool(&mut self, name: &str) -> ServerResult<()> {
        if self.tools.remove(name).is_some() {
            info!("Unregistered tool: {}", name);
            Ok(())
        } else {
            Err(ServerError::ToolNotFound(name.to_string()))
        }
    }

    /// Get a tool by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tool to retrieve
    ///
    /// # Returns
    ///
    /// An optional reference to the tool if found
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn McpTool>> {
        self.tools.get(name).cloned()
    }

    /// List all registered tools
    ///
    /// # Returns
    ///
    /// A vector of all registered tool instances
    pub fn list_tools(&self) -> Vec<Arc<dyn McpTool>> {
        self.tools.values().cloned().collect()
    }

    /// Get the names of all registered tools
    ///
    /// # Returns
    ///
    /// A vector of tool names
    pub fn tool_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// Get the number of registered tools
    ///
    /// # Returns
    ///
    /// The count of registered tools
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    /// Check if a tool is registered
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tool to check
    ///
    /// # Returns
    ///
    /// True if the tool is registered, false otherwise
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Clear all registered tools
    ///
    /// This removes all tools from the registry
    pub fn clear(&mut self) {
        let count = self.tools.len();
        self.tools.clear();
        info!("Cleared {} tools from registry", count);
    }

    /// Get registry statistics
    ///
    /// # Returns
    ///
    /// A map containing registry statistics
    pub fn stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        stats.insert("tool_count".to_string(), self.tool_count().into());
        stats.insert("tool_names".to_string(), self.tool_names().into());
        stats
    }

    /// Validate all registered tools
    ///
    /// This performs basic validation on all registered tools to ensure
    /// they meet minimum requirements.
    ///
    /// # Returns
    ///
    /// Result indicating validation success or the first error encountered
    pub fn validate_tools(&self) -> ServerResult<()> {
        for (name, tool) in &self.tools {
            // Basic validation
            if name.is_empty() {
                return Err(ServerError::ToolRegistration(
                    "Tool name cannot be empty".to_string(),
                ));
            }

            if tool.name() != name {
                return Err(ServerError::ToolRegistration(format!(
                    "Tool name mismatch: registry key '{}' != tool.name() '{}'",
                    name,
                    tool.name()
                )));
            }

            if tool.description().is_empty() {
                warn!("Tool '{}' has empty description", name);
            }
        }

        debug!("Validated {} tools successfully", self.tool_count());
        Ok(())
    }

    /// Find tools by capability
    ///
    /// # Arguments
    ///
    /// * `capability` - The capability to search for
    ///
    /// # Returns
    ///
    /// A vector of tools that support the specified capability
    pub fn find_tools_by_capability(&self, capability: &str) -> Vec<Arc<dyn McpTool>> {
        self.tools
            .values()
            .filter(|tool| tool.supports_capability(capability))
            .cloned()
            .collect()
    }

    /// Register multiple tools at once
    ///
    /// # Arguments
    ///
    /// * `tools` - A vector of tools to register
    ///
    /// # Returns
    ///
    /// Result indicating success or the first error encountered
    pub fn register_tools(&mut self, tools: Vec<Arc<dyn McpTool>>) -> ServerResult<()> {
        for tool in tools {
            self.register_tool(tool)?;
        }
        Ok(())
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ToolRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolRegistry")
            .field("tool_count", &self.tool_count())
            .field("tool_names", &self.tool_names())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use mcp_core::{McpError, McpRequest, McpResponse};

    struct MockTool {
        name: String,
        description: String,
    }

    #[async_trait]
    impl McpTool for MockTool {
        async fn call(&self, _request: McpRequest) -> Result<McpResponse, McpError> {
            Ok(McpResponse::simple_success("mock response"))
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.tool_count(), 0);
        assert!(registry.tool_names().is_empty());
    }

    #[test]
    fn test_tool_registration() {
        let mut registry = ToolRegistry::new();
        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
        });

        assert!(registry.register_tool(tool.clone()).is_ok());
        assert_eq!(registry.tool_count(), 1);
        assert!(registry.has_tool("test_tool"));

        let retrieved = registry.get_tool("test_tool");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "test_tool");
    }

    #[test]
    fn test_tool_unregistration() {
        let mut registry = ToolRegistry::new();
        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
        });

        registry.register_tool(tool).unwrap();
        assert_eq!(registry.tool_count(), 1);

        assert!(registry.unregister_tool("test_tool").is_ok());
        assert_eq!(registry.tool_count(), 0);
        assert!(!registry.has_tool("test_tool"));

        // Try to unregister non-existent tool
        assert!(registry.unregister_tool("non_existent").is_err());
    }

    #[test]
    fn test_multiple_tool_registration() {
        let mut registry = ToolRegistry::new();
        let tools = vec![
            Arc::new(MockTool {
                name: "tool1".to_string(),
                description: "First tool".to_string(),
            }) as Arc<dyn McpTool>,
            Arc::new(MockTool {
                name: "tool2".to_string(),
                description: "Second tool".to_string(),
            }) as Arc<dyn McpTool>,
        ];

        assert!(registry.register_tools(tools).is_ok());
        assert_eq!(registry.tool_count(), 2);
        assert!(registry.has_tool("tool1"));
        assert!(registry.has_tool("tool2"));
    }

    #[test]
    fn test_tool_replacement() {
        let mut registry = ToolRegistry::new();
        let tool1 = Arc::new(MockTool {
            name: "test_tool".to_string(),
            description: "First version".to_string(),
        });
        let tool2 = Arc::new(MockTool {
            name: "test_tool".to_string(),
            description: "Second version".to_string(),
        });

        registry.register_tool(tool1).unwrap();
        registry.register_tool(tool2).unwrap(); // Should replace the first one

        assert_eq!(registry.tool_count(), 1);
        let retrieved = registry.get_tool("test_tool").unwrap();
        assert_eq!(retrieved.description(), "Second version");
    }

    #[test]
    fn test_registry_clear() {
        let mut registry = ToolRegistry::new();
        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
        });

        registry.register_tool(tool).unwrap();
        assert_eq!(registry.tool_count(), 1);

        registry.clear();
        assert_eq!(registry.tool_count(), 0);
        assert!(!registry.has_tool("test_tool"));
    }

    #[test]
    fn test_registry_stats() {
        let mut registry = ToolRegistry::new();
        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
        });

        registry.register_tool(tool).unwrap();
        let stats = registry.stats();

        assert_eq!(stats["tool_count"], serde_json::Value::from(1));
        assert_eq!(stats["tool_names"], serde_json::json!(["test_tool"]));
    }

    #[test]
    fn test_tool_validation() {
        let mut registry = ToolRegistry::new();
        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
        });

        registry.register_tool(tool).unwrap();
        assert!(registry.validate_tools().is_ok());
    }
}
