//! Transport registry for managing multiple transport implementations
//!
//! This module provides a registry for registering and retrieving transport
//! implementations at runtime. Supports dynamic transport selection based on
//! configuration.
//!
//! Note: Some methods are defined for public API but may not be used internally.
#![allow(dead_code)]

use super::r#trait::{Transport, TransportConfig, TransportError, TransportFactory};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Registry for managing transport implementations
pub struct TransportRegistry {
    /// Map of transport type name to factory
    factories: Arc<RwLock<HashMap<String, Arc<dyn TransportFactory>>>>,
}

impl TransportRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            factories: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a transport factory
    ///
    /// # Arguments
    /// * `transport_type` - Transport type name (e.g., "stdio", "sse")
    /// * `factory` - Factory implementation for creating transport instances
    ///
    /// # Returns
    /// * `Result<()>` - Success or error if type already registered
    pub fn register(
        &self,
        transport_type: impl Into<String>,
        factory: Arc<dyn TransportFactory>,
    ) -> Result<(), TransportError> {
        let transport_type = transport_type.into();
        let mut factories = self
            .factories
            .write()
            .map_err(|e| TransportError::Other(format!("Failed to acquire write lock: {e}")))?;

        if factories.contains_key(&transport_type) {
            return Err(TransportError::ConfigError(format!(
                "Transport type '{transport_type}' already registered"
            )));
        }

        factories.insert(transport_type, factory);
        Ok(())
    }

    /// Create a transport instance from configuration
    ///
    /// # Arguments
    /// * `config` - Transport configuration
    ///
    /// # Returns
    /// * `Result<Box<dyn Transport>>` - Transport instance or error
    pub fn create(&self, config: TransportConfig) -> Result<Box<dyn Transport>, TransportError> {
        let factories = self
            .factories
            .read()
            .map_err(|e| TransportError::Other(format!("Failed to acquire read lock: {e}")))?;

        let factory = factories.get(&config.transport_type).ok_or_else(|| {
            TransportError::ConfigError(format!(
                "Unknown transport type: '{}'. Available: {}",
                config.transport_type,
                self.list_available().join(", ")
            ))
        })?;

        factory.create(config)
    }

    /// List all registered transport types
    pub fn list_available(&self) -> Vec<String> {
        self.factories
            .read()
            .map(|factories| {
                let mut types: Vec<String> = factories.keys().cloned().collect();
                types.sort();
                types
            })
            .unwrap_or_default()
    }

    /// Check if a transport type is registered
    pub fn is_registered(&self, transport_type: &str) -> bool {
        self.factories
            .read()
            .map(|factories| factories.contains_key(transport_type))
            .unwrap_or(false)
    }

    /// Get the default global registry
    pub fn global() -> &'static Self {
        static INSTANCE: std::sync::OnceLock<TransportRegistry> = std::sync::OnceLock::new();
        INSTANCE.get_or_init(|| {
            
            TransportRegistry::new()
        })
    }
}

impl Default for TransportRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::r#trait::TransportCapabilities;
    use async_trait::async_trait;

    // Mock transport for testing
    struct MockTransport {
        transport_type: String,
    }

    #[async_trait]
    impl Transport for MockTransport {
        fn transport_type(&self) -> &str {
            &self.transport_type
        }

        fn capabilities(&self) -> TransportCapabilities {
            TransportCapabilities {
                bidirectional: true,
                server_push: false,
                multi_connection: false,
                streaming: false,
                browser_compatible: false,
            }
        }

        async fn send(
            &self,
            _message: crate::transport::r#trait::TransportMessage,
        ) -> Result<(), TransportError> {
            Ok(())
        }

        async fn receive(
            &self,
        ) -> Result<crate::transport::r#trait::TransportMessage, TransportError> {
            Err(TransportError::NotInitialized)
        }

        async fn initialize(&mut self) -> Result<(), TransportError> {
            Ok(())
        }

        async fn shutdown(&mut self) -> Result<(), TransportError> {
            Ok(())
        }

        fn is_ready(&self) -> bool {
            true
        }
    }

    struct MockFactory {
        transport_type: String,
    }

    impl TransportFactory for MockFactory {
        fn create(&self, _config: TransportConfig) -> Result<Box<dyn Transport>, TransportError> {
            Ok(Box::new(MockTransport {
                transport_type: self.transport_type.clone(),
            }))
        }

        fn supported_types(&self) -> Vec<String> {
            vec![self.transport_type.clone()]
        }
    }

    #[test]
    fn test_registry_register_and_create() {
        let registry = TransportRegistry::new();
        let factory = Arc::new(MockFactory {
            transport_type: "mock".to_string(),
        });

        registry.register("mock", factory).unwrap();
        assert!(registry.is_registered("mock"));

        let config = TransportConfig {
            transport_type: "mock".to_string(),
            ..Default::default()
        };

        let transport = registry.create(config).unwrap();
        assert_eq!(transport.transport_type(), "mock");
    }

    #[test]
    fn test_registry_duplicate_registration() {
        let registry = TransportRegistry::new();
        let factory = Arc::new(MockFactory {
            transport_type: "mock".to_string(),
        });

        registry.register("mock", factory.clone()).unwrap();
        let result = registry.register("mock", factory);
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_unknown_transport() {
        let registry = TransportRegistry::new();
        let config = TransportConfig {
            transport_type: "unknown".to_string(),
            ..Default::default()
        };

        let result = registry.create(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_list_available() {
        let registry = TransportRegistry::new();

        let factory1 = Arc::new(MockFactory {
            transport_type: "mock1".to_string(),
        });
        let factory2 = Arc::new(MockFactory {
            transport_type: "mock2".to_string(),
        });

        registry.register("mock1", factory1).unwrap();
        registry.register("mock2", factory2).unwrap();

        let available = registry.list_available();
        assert_eq!(available.len(), 2);
        assert!(available.contains(&"mock1".to_string()));
        assert!(available.contains(&"mock2".to_string()));
    }
}
