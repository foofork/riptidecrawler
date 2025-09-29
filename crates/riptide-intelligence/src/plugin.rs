//! Plugin architecture for dynamic LLM provider loading
//!
//! This module provides a plugin system that allows for:
//! - Dynamic loading of LLM providers at runtime
//! - Configuration-driven provider instantiation
//! - Hot-swapping of providers without restart
//! - Plugin lifecycle management

use std::sync::Arc;
use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use thiserror::Error;
use crate::LlmProvider;

/// Plugin metadata describing a provider plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub provider_type: String,
    pub supported_models: Vec<String>,
    pub capabilities: PluginCapabilities,
    pub requirements: PluginRequirements,
}

/// Capabilities supported by a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCapabilities {
    pub completion: bool,
    pub embeddings: bool,
    pub streaming: bool,
    pub function_calling: bool,
    pub vision: bool,
    pub fine_tuning: bool,
}

/// Requirements for a plugin to function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRequirements {
    pub min_memory_mb: u64,
    pub requires_gpu: bool,
    pub requires_network: bool,
    pub min_api_version: String,
    pub dependencies: Vec<String>,
}

/// Configuration for a plugin instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub plugin_id: String,
    pub instance_name: String,
    pub enabled: bool,
    pub priority: u32,
    pub config: HashMap<String, serde_json::Value>,
    pub resource_limits: ResourceLimits,
    pub health_check_interval: u64, // seconds
    pub failover_timeout: u64,      // milliseconds
}

/// Resource limits for plugin instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u64,
    pub max_cpu_percent: u32,
    pub max_requests_per_minute: u32,
    pub max_concurrent_requests: u32,
    pub timeout_ms: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 1024,
            max_cpu_percent: 50,
            max_requests_per_minute: 60,
            max_concurrent_requests: 10,
            timeout_ms: 30000,
        }
    }
}

/// State of a plugin instance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginState {
    Unloaded,
    Loading,
    Ready,
    Active,
    Degraded,
    Error,
    Unloading,
}

/// Plugin instance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStatus {
    pub instance_name: String,
    pub plugin_id: String,
    pub state: PluginState,
    pub last_health_check: chrono::DateTime<chrono::Utc>,
    pub health_status: HealthStatus,
    pub metrics: PluginMetrics,
    pub error_count: u32,
    pub uptime_seconds: u64,
}

/// Health status of a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub response_time_ms: u64,
    pub error_rate: f64,
    pub last_error: Option<String>,
    pub resource_usage: ResourceUsage,
}

/// Current resource usage of a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub memory_mb: u64,
    pub cpu_percent: f32,
    pub active_requests: u32,
    pub requests_per_minute: u32,
}

/// Plugin performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: f64,
    pub tokens_processed: u64,
    pub cost_incurred: f64,
    pub uptime_percentage: f64,
}

impl Default for PluginMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time_ms: 0.0,
            tokens_processed: 0,
            cost_incurred: 0.0,
            uptime_percentage: 100.0,
        }
    }
}

/// Errors specific to the plugin system
#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Plugin not found: {id}")]
    NotFound { id: String },

    #[error("Plugin already loaded: {id}")]
    AlreadyLoaded { id: String },

    #[error("Plugin loading failed: {id} - {reason}")]
    LoadingFailed { id: String, reason: String },

    #[error("Plugin validation failed: {id} - {reason}")]
    ValidationFailed { id: String, reason: String },

    #[error("Resource limit exceeded: {resource} - {limit}")]
    ResourceLimitExceeded { resource: String, limit: String },

    #[error("Plugin incompatible: {id} - {reason}")]
    Incompatible { id: String, reason: String },

    #[error("Plugin IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Plugin serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Trait for implementing plugin loaders
#[async_trait]
pub trait PluginLoader: Send + Sync {
    /// Load a plugin from the given path
    async fn load_plugin(&self, path: &std::path::Path) -> std::result::Result<Box<dyn Plugin>, PluginError>;

    /// Validate plugin compatibility
    async fn validate_plugin(&self, metadata: &PluginMetadata) -> std::result::Result<(), PluginError>;

    /// Get supported plugin formats
    fn supported_formats(&self) -> Vec<String>;
}

/// Core plugin trait that all provider plugins must implement
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Initialize the plugin with configuration
    async fn initialize(&mut self, config: &PluginConfig) -> std::result::Result<(), PluginError>;

    /// Create a provider instance
    async fn create_provider(&self, config: &PluginConfig) -> std::result::Result<Arc<dyn LlmProvider>, PluginError>;

    /// Perform health check
    async fn health_check(&self) -> std::result::Result<HealthStatus, PluginError>;

    /// Get current metrics
    fn metrics(&self) -> PluginMetrics;

    /// Shutdown the plugin gracefully
    async fn shutdown(&mut self) -> std::result::Result<(), PluginError>;

    /// Hot-reload configuration
    async fn reload_config(&mut self, config: &PluginConfig) -> std::result::Result<(), PluginError>;
}

/// Plugin registry for managing loaded plugins
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
    instances: HashMap<String, PluginStatus>,
    loader: Box<dyn PluginLoader>,
}

impl PluginRegistry {
    /// Create a new plugin registry with the given loader
    pub fn new(loader: Box<dyn PluginLoader>) -> Self {
        Self {
            plugins: HashMap::new(),
            instances: HashMap::new(),
            loader,
        }
    }

    /// Load a plugin from file
    pub async fn load_plugin(&mut self, path: PathBuf) -> std::result::Result<String, PluginError> {
        let mut plugin = self.loader.load_plugin(&path).await?;
        let metadata = plugin.metadata().clone();

        // Validate plugin
        self.loader.validate_plugin(&metadata).await?;

        // Check if already loaded
        if self.plugins.contains_key(&metadata.id) {
            return Err(PluginError::AlreadyLoaded { id: metadata.id });
        }

        // Initialize plugin
        let default_config = PluginConfig {
            plugin_id: metadata.id.clone(),
            instance_name: format!("{}-default", metadata.id),
            enabled: true,
            priority: 100,
            config: HashMap::new(),
            resource_limits: ResourceLimits::default(),
            health_check_interval: 60,
            failover_timeout: 5000,
        };

        plugin.initialize(&default_config).await?;

        let plugin_id = metadata.id.clone();
        self.plugins.insert(plugin_id.clone(), plugin);

        Ok(plugin_id)
    }

    /// Create a provider instance from a plugin
    pub async fn create_instance(
        &mut self,
        plugin_id: &str,
        config: PluginConfig,
    ) -> std::result::Result<Arc<dyn LlmProvider>, PluginError> {
        let plugin = self.plugins.get(plugin_id)
            .ok_or_else(|| PluginError::NotFound { id: plugin_id.to_string() })?;

        let provider = plugin.create_provider(&config).await?;

        // Track instance status
        let status = PluginStatus {
            instance_name: config.instance_name.clone(),
            plugin_id: plugin_id.to_string(),
            state: PluginState::Active,
            last_health_check: chrono::Utc::now(),
            health_status: HealthStatus {
                is_healthy: true,
                response_time_ms: 0,
                error_rate: 0.0,
                last_error: None,
                resource_usage: ResourceUsage {
                    memory_mb: 0,
                    cpu_percent: 0.0,
                    active_requests: 0,
                    requests_per_minute: 0,
                },
            },
            metrics: PluginMetrics::default(),
            error_count: 0,
            uptime_seconds: 0,
        };

        self.instances.insert(config.instance_name.clone(), status);

        Ok(provider)
    }

    /// Get all loaded plugins
    pub fn list_plugins(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    /// Get plugin metadata
    pub fn get_metadata(&self, plugin_id: &str) -> Option<&PluginMetadata> {
        self.plugins.get(plugin_id).map(|p| p.metadata())
    }

    /// Get instance status
    pub fn get_instance_status(&self, instance_name: &str) -> Option<&PluginStatus> {
        self.instances.get(instance_name)
    }

    /// Update instance status
    pub fn update_instance_status(&mut self, instance_name: &str, status: PluginStatus) {
        self.instances.insert(instance_name.to_string(), status);
    }

    /// Perform health checks on all instances
    pub async fn health_check_all(&mut self) -> HashMap<String, bool> {
        let mut results = HashMap::new();

        for (plugin_id, plugin) in &self.plugins {
            match plugin.health_check().await {
                Ok(health) => {
                    results.insert(plugin_id.clone(), health.is_healthy);
                }
                Err(_) => {
                    results.insert(plugin_id.clone(), false);
                }
            }
        }

        results
    }

    /// Unload a plugin
    pub async fn unload_plugin(&mut self, plugin_id: &str) -> std::result::Result<(), PluginError> {
        if let Some(mut plugin) = self.plugins.remove(plugin_id) {
            plugin.shutdown().await?;

            // Remove associated instances
            self.instances.retain(|_, status| status.plugin_id != plugin_id);
        }

        Ok(())
    }

    /// Get registry statistics
    pub fn stats(&self) -> PluginRegistryStats {
        let total_plugins = self.plugins.len();
        let total_instances = self.instances.len();
        let healthy_instances = self.instances.values()
            .filter(|status| status.health_status.is_healthy)
            .count();

        PluginRegistryStats {
            total_plugins,
            total_instances,
            healthy_instances,
            plugin_types: self.plugins.values()
                .map(|p| p.metadata().provider_type.clone())
                .collect::<std::collections::HashSet<_>>()
                .len(),
        }
    }
}

/// Statistics about the plugin registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRegistryStats {
    pub total_plugins: usize,
    pub total_instances: usize,
    pub healthy_instances: usize,
    pub plugin_types: usize,
}

/// Default filesystem-based plugin loader
pub struct FileSystemPluginLoader {
    #[allow(dead_code)]
    plugin_dir: PathBuf,
}

impl FileSystemPluginLoader {
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self { plugin_dir }
    }
}

#[async_trait]
impl PluginLoader for FileSystemPluginLoader {
    async fn load_plugin(&self, _path: &std::path::Path) -> std::result::Result<Box<dyn Plugin>, PluginError> {
        // For now, this is a placeholder implementation
        // In a real implementation, this would:
        // 1. Load the plugin binary/library
        // 2. Initialize the plugin interface
        // 3. Return the plugin instance
        Err(PluginError::LoadingFailed {
            id: "unknown".to_string(),
            reason: "FileSystem plugin loading not yet implemented".to_string(),
        })
    }

    async fn validate_plugin(&self, metadata: &PluginMetadata) -> std::result::Result<(), PluginError> {
        // Basic validation
        if metadata.name.is_empty() {
            return Err(PluginError::ValidationFailed {
                id: metadata.id.clone(),
                reason: "Plugin name cannot be empty".to_string(),
            });
        }

        if metadata.provider_type.is_empty() {
            return Err(PluginError::ValidationFailed {
                id: metadata.id.clone(),
                reason: "Provider type cannot be empty".to_string(),
            });
        }

        Ok(())
    }

    fn supported_formats(&self) -> Vec<String> {
        vec!["dll".to_string(), "so".to_string(), "dylib".to_string()]
    }
}