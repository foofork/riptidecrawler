//! Provider registry for dynamic loading and configuration

use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use dashmap::DashMap;
use tracing::{info, warn, error};

use crate::{LlmProvider, IntelligenceError, Result};

/// Configuration for a single provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub provider_type: String,
    pub enabled: bool,
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub fallback_order: Option<usize>,
}

impl ProviderConfig {
    pub fn new(name: impl Into<String>, provider_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            provider_type: provider_type.into(),
            enabled: true,
            config: HashMap::new(),
            fallback_order: None,
        }
    }

    pub fn with_config(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.config.insert(key.into(), value);
        self
    }

    pub fn with_fallback_order(mut self, order: usize) -> Self {
        self.fallback_order = Some(order);
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Factory function type for creating providers
pub type ProviderFactory = Box<dyn Fn(&ProviderConfig) -> Result<Arc<dyn LlmProvider>> + Send + Sync>;

/// Registry for managing LLM providers
pub struct LlmRegistry {
    providers: DashMap<String, Arc<dyn LlmProvider>>,
    factories: DashMap<String, ProviderFactory>,
    configs: DashMap<String, ProviderConfig>,
}

impl LlmRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            providers: DashMap::new(),
            factories: DashMap::new(),
            configs: DashMap::new(),
        }
    }

    /// Register a provider factory for a given type
    pub fn register_factory<F>(&self, provider_type: impl Into<String>, factory: F) -> Result<()>
    where
        F: Fn(&ProviderConfig) -> Result<Arc<dyn LlmProvider>> + Send + Sync + 'static,
    {
        let provider_type = provider_type.into();
        info!("Registering provider factory for type: {}", provider_type);

        self.factories.insert(provider_type.clone(), Box::new(factory));
        Ok(())
    }

    /// Register a pre-created provider instance
    pub fn register_provider(&self, name: impl Into<String>, provider: Arc<dyn LlmProvider>) -> Result<()> {
        let name = name.into();
        info!("Registering provider instance: {}", name);

        self.providers.insert(name.clone(), provider);
        Ok(())
    }

    /// Configure and load a provider from its configuration
    pub fn load_provider(&self, config: ProviderConfig) -> Result<()> {
        if !config.enabled {
            info!("Provider {} is disabled, skipping", config.name);
            return Ok(());
        }

        info!("Loading provider: {} (type: {})", config.name, config.provider_type);

        // Check if we have a factory for this provider type
        let factory = self.factories.get(&config.provider_type)
            .ok_or_else(|| IntelligenceError::Configuration(
                format!("No factory registered for provider type: {}", config.provider_type)
            ))?;

        // Create the provider using the factory
        let provider = factory(&config)?;

        // Store the config and provider
        self.configs.insert(config.name.clone(), config.clone());
        self.providers.insert(config.name.clone(), provider);

        info!("Successfully loaded provider: {}", config.name);
        Ok(())
    }

    /// Load multiple providers from configurations
    pub fn load_providers(&self, configs: Vec<ProviderConfig>) -> Result<()> {
        let mut errors = Vec::new();

        for config in configs {
            if let Err(e) = self.load_provider(config.clone()) {
                error!("Failed to load provider {}: {}", config.name, e);
                errors.push((config.name, e));
            }
        }

        if !errors.is_empty() {
            warn!("Some providers failed to load: {:?}", errors);
            // Continue with successfully loaded providers
        }

        Ok(())
    }

    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn LlmProvider>> {
        self.providers.get(name).map(|entry| entry.value().clone())
    }

    /// Get all provider names
    pub fn list_providers(&self) -> Vec<String> {
        self.providers.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Get providers ordered by fallback priority
    pub fn get_fallback_providers(&self) -> Vec<(String, Arc<dyn LlmProvider>)> {
        let mut providers_with_order: Vec<_> = self.configs
            .iter()
            .filter_map(|entry| {
                let config = entry.value();
                if config.enabled {
                    self.providers.get(&config.name).map(|provider| {
                        (config.fallback_order.unwrap_or(999), config.name.clone(), provider.value().clone())
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort by fallback order
        providers_with_order.sort_by_key(|(order, _, _)| *order);

        providers_with_order
            .into_iter()
            .map(|(_, name, provider)| (name, provider))
            .collect()
    }

    /// Remove a provider
    pub fn remove_provider(&self, name: &str) -> Option<Arc<dyn LlmProvider>> {
        info!("Removing provider: {}", name);
        self.configs.remove(name);
        self.providers.remove(name).map(|(_, provider)| provider)
    }

    /// Check if a provider exists
    pub fn has_provider(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }

    /// Get provider configuration
    pub fn get_config(&self, name: &str) -> Option<ProviderConfig> {
        self.configs.get(name).map(|entry| entry.value().clone())
    }

    /// Update provider configuration and reload
    pub fn update_config(&self, config: ProviderConfig) -> Result<()> {
        // Remove existing provider if it exists
        if self.has_provider(&config.name) {
            self.remove_provider(&config.name);
        }

        // Load with new configuration
        self.load_provider(config)
    }

    /// Get registry statistics
    pub fn stats(&self) -> RegistryStats {
        let total_providers = self.providers.len();
        let enabled_providers = self.configs
            .iter()
            .filter(|entry| entry.value().enabled)
            .count();

        let provider_types = self.configs
            .iter()
            .map(|entry| entry.value().provider_type.clone())
            .collect::<std::collections::HashSet<_>>()
            .len();

        RegistryStats {
            total_providers,
            enabled_providers,
            provider_types,
            registered_factories: self.factories.len(),
        }
    }

    /// Perform health checks on all providers
    pub async fn health_check(&self) -> HashMap<String, bool> {
        let mut results = HashMap::new();

        for entry in self.providers.iter() {
            let name = entry.key().clone();
            let provider = entry.value();
            let is_healthy = provider.is_available().await;
            results.insert(name, is_healthy);
        }

        results
    }
}

impl Default for LlmRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_providers: usize,
    pub enabled_providers: usize,
    pub provider_types: usize,
    pub registered_factories: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_provider::MockLlmProvider;

    #[test]
    fn test_registry_creation() {
        let registry = LlmRegistry::new();
        assert_eq!(registry.list_providers().len(), 0);
    }

    #[test]
    fn test_provider_registration() {
        let registry = LlmRegistry::new();
        let provider = Arc::new(MockLlmProvider::new());

        registry.register_provider("test", provider).unwrap();
        assert!(registry.has_provider("test"));
        assert_eq!(registry.list_providers().len(), 1);
    }

    #[test]
    fn test_factory_registration() {
        let registry = LlmRegistry::new();

        registry.register_factory("mock", |_config| {
            Ok(Arc::new(MockLlmProvider::new()) as Arc<dyn LlmProvider>)
        }).unwrap();

        let config = ProviderConfig::new("test", "mock");
        registry.load_provider(config).unwrap();

        assert!(registry.has_provider("test"));
    }
}