//! Configuration system for LLM providers
//!
//! This module provides environment-driven configuration loading with:
//! - Environment variable parsing
//! - Configuration file loading (YAML, JSON, TOML)
//! - Runtime configuration updates
//! - Configuration validation
//! - Provider auto-discovery

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, warn, debug};

use crate::{
    registry::ProviderConfig,
    plugin::PluginConfig,
    failover::FailoverConfig,
    health::HealthCheckConfig,
};

/// Complete LLM intelligence system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceConfig {
    pub providers: Vec<ProviderConfig>,
    pub plugins: Vec<PluginConfig>,
    pub failover: FailoverConfig,
    pub health_check: HealthCheckConfig,
    pub metrics: MetricsConfig,
    pub runtime: RuntimeConfig,
    pub tenant_isolation: TenantIsolationConfig,
    pub cost_tracking: CostTrackingConfig,
}

/// Metrics collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub retention_days: u32,
    pub export_interval_seconds: u64,
    pub high_cardinality_metrics: bool,
    pub prometheus_endpoint: Option<String>,
    pub dashboard_enabled: bool,
    pub real_time_streaming: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_days: 30,
            export_interval_seconds: 60,
            high_cardinality_metrics: false,
            prometheus_endpoint: None,
            dashboard_enabled: true,
            real_time_streaming: true,
        }
    }
}

/// Runtime configuration for hot-reloading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub hot_reload_enabled: bool,
    pub config_watch_interval: u64, // seconds
    pub graceful_shutdown_timeout: u64, // seconds
    pub provider_switch_timeout: u64, // milliseconds
    pub max_concurrent_requests: u32,
    pub request_queue_size: u32,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            hot_reload_enabled: true,
            config_watch_interval: 5,
            graceful_shutdown_timeout: 30,
            provider_switch_timeout: 1000,
            max_concurrent_requests: 100,
            request_queue_size: 1000,
        }
    }
}

/// Tenant isolation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantIsolationConfig {
    pub enabled: bool,
    pub strict_isolation: bool,
    pub per_tenant_limits: HashMap<String, TenantLimits>,
    pub default_limits: TenantLimits,
    pub tenant_provider_mapping: HashMap<String, Vec<String>>,
}

/// Resource limits per tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantLimits {
    pub max_requests_per_minute: u32,
    pub max_tokens_per_minute: u32,
    pub max_cost_per_hour: f64,
    pub max_concurrent_requests: u32,
    pub allowed_models: Option<Vec<String>>,
    pub priority: u32,
}

impl Default for TenantLimits {
    fn default() -> Self {
        Self {
            max_requests_per_minute: 60,
            max_tokens_per_minute: 100_000,
            max_cost_per_hour: 10.0,
            max_concurrent_requests: 10,
            allowed_models: None,
            priority: 100,
        }
    }
}

impl Default for TenantIsolationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strict_isolation: false,
            per_tenant_limits: HashMap::new(),
            default_limits: TenantLimits::default(),
            tenant_provider_mapping: HashMap::new(),
        }
    }
}

/// Cost tracking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostTrackingConfig {
    pub enabled: bool,
    pub detailed_tracking: bool,
    pub cost_alerts_enabled: bool,
    pub per_tenant_budgets: HashMap<String, f64>,
    pub billing_period_days: u32,
    pub currency: String,
    pub cost_optimization_enabled: bool,
    pub auto_switch_cheaper_provider: bool,
}

impl Default for CostTrackingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            detailed_tracking: true,
            cost_alerts_enabled: true,
            per_tenant_budgets: HashMap::new(),
            billing_period_days: 30,
            currency: "USD".to_string(),
            cost_optimization_enabled: false,
            auto_switch_cheaper_provider: false,
        }
    }
}

/// Configuration loading errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Environment variable error: {variable} - {reason}")]
    Environment { variable: String, reason: String },

    #[error("File error: {path} - {reason}")]
    File { path: String, reason: String },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Validation error: {field} - {reason}")]
    Validation { field: String, reason: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Configuration loader with environment and file support
#[derive(Clone)]
pub struct ConfigLoader {
    env_prefix: String,
    config_paths: Vec<PathBuf>,
}

impl ConfigLoader {
    /// Create a new configuration loader
    pub fn new() -> Self {
        Self {
            env_prefix: "RIPTIDE_".to_string(),
            config_paths: vec![
                PathBuf::from("config/intelligence.yaml"),
                PathBuf::from("config/intelligence.json"),
                PathBuf::from("config/intelligence.toml"),
                PathBuf::from("intelligence.yaml"),
                PathBuf::from("intelligence.json"),
                PathBuf::from("intelligence.toml"),
            ],
        }
    }

    /// Set environment variable prefix
    pub fn with_env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.env_prefix = prefix.into();
        self
    }

    /// Add configuration file path
    pub fn with_config_path(mut self, path: PathBuf) -> Self {
        self.config_paths.insert(0, path);
        self
    }

    /// Load complete configuration from environment and files
    pub fn load(&self) -> Result<IntelligenceConfig, ConfigError> {
        info!("Loading LLM intelligence configuration");

        // Start with default configuration
        let mut config = IntelligenceConfig::default();

        // Try to load from config files
        if let Some(file_config) = self.load_from_files()? {
            config = file_config;
            info!("Loaded base configuration from file");
        }

        // Override with environment variables
        self.override_from_environment(&mut config)?;

        // Validate configuration
        self.validate_config(&config)?;

        info!("Configuration loaded successfully with {} providers", config.providers.len());
        Ok(config)
    }

    /// Load configuration from files
    fn load_from_files(&self) -> Result<Option<IntelligenceConfig>, ConfigError> {
        for path in &self.config_paths {
            if path.exists() {
                info!("Loading configuration from: {}", path.display());

                let content = std::fs::read_to_string(path)?;

                let config = match path.extension().and_then(|ext| ext.to_str()) {
                    Some("yaml") | Some("yml") => {
                        serde_yaml::from_str::<IntelligenceConfig>(&content)?
                    },
                    Some("json") => {
                        serde_json::from_str::<IntelligenceConfig>(&content)?
                    },
                    Some("toml") => {
                        toml::from_str::<IntelligenceConfig>(&content)?
                    },
                    _ => {
                        // Try JSON as default
                        serde_json::from_str::<IntelligenceConfig>(&content)?
                    }
                };

                return Ok(Some(config));
            }
        }

        debug!("No configuration file found, using defaults");
        Ok(None)
    }

    /// Override configuration with environment variables
    fn override_from_environment(&self, config: &mut IntelligenceConfig) -> Result<(), ConfigError> {
        info!("Loading environment overrides with prefix: {}", self.env_prefix);

        // Load provider configurations from environment
        let providers = self.load_providers_from_env()?;
        if !providers.is_empty() {
            config.providers = providers;
            info!("Loaded {} providers from environment", config.providers.len());
        }

        // Override metrics configuration
        if let Ok(enabled) = env::var(format!("{}METRICS_ENABLED", self.env_prefix)) {
            config.metrics.enabled = enabled.parse().unwrap_or(true);
        }

        if let Ok(retention) = env::var(format!("{}METRICS_RETENTION_DAYS", self.env_prefix)) {
            config.metrics.retention_days = retention.parse().unwrap_or(30);
        }

        if let Ok(endpoint) = env::var(format!("{}PROMETHEUS_ENDPOINT", self.env_prefix)) {
            config.metrics.prometheus_endpoint = Some(endpoint);
        }

        // Override failover configuration
        if let Ok(max_retries) = env::var(format!("{}FAILOVER_MAX_RETRIES", self.env_prefix)) {
            config.failover.max_retries = max_retries.parse().unwrap_or(3);
        }

        // Override tenant isolation
        if let Ok(enabled) = env::var(format!("{}TENANT_ISOLATION_ENABLED", self.env_prefix)) {
            config.tenant_isolation.enabled = enabled.parse().unwrap_or(true);
        }

        // Override cost tracking
        if let Ok(enabled) = env::var(format!("{}COST_TRACKING_ENABLED", self.env_prefix)) {
            config.cost_tracking.enabled = enabled.parse().unwrap_or(true);
        }

        if let Ok(currency) = env::var(format!("{}COST_CURRENCY", self.env_prefix)) {
            config.cost_tracking.currency = currency;
        }

        Ok(())
    }

    /// Load provider configurations from environment variables
    fn load_providers_from_env(&self) -> Result<Vec<ProviderConfig>, ConfigError> {
        let mut providers = Vec::new();

        // Look for RIPTIDE_PROVIDERS environment variable with JSON array
        if let Ok(providers_json) = env::var(format!("{}PROVIDERS", self.env_prefix)) {
            let parsed_providers: Vec<ProviderConfig> = serde_json::from_str(&providers_json)
                .map_err(|e| ConfigError::Environment {
                    variable: format!("{}PROVIDERS", self.env_prefix),
                    reason: format!("Invalid JSON: {}", e),
                })?;
            providers.extend(parsed_providers);
        }

        // Look for individual provider configurations
        // RIPTIDE_PROVIDER_OPENAI_ENABLED=true
        // RIPTIDE_PROVIDER_OPENAI_API_KEY=sk-...
        // RIPTIDE_PROVIDER_OPENAI_MODEL=gpt-4
        // RIPTIDE_PROVIDER_OPENAI_PRIORITY=1

        let provider_types = ["openai", "anthropic", "azure", "bedrock", "vertex", "ollama"];

        for provider_type in &provider_types {
            let enabled_var = format!("{}PROVIDER_{}_ENABLED", self.env_prefix, provider_type.to_uppercase());

            if let Ok(enabled) = env::var(&enabled_var) {
                if enabled.parse::<bool>().unwrap_or(false) {
                    let provider_config = self.build_provider_config_from_env(provider_type)?;
                    providers.push(provider_config);
                }
            }
        }

        Ok(providers)
    }

    /// Build provider configuration from environment variables
    fn build_provider_config_from_env(&self, provider_type: &str) -> Result<ProviderConfig, ConfigError> {
        let prefix = format!("{}PROVIDER_{}", self.env_prefix, provider_type.to_uppercase());

        let name = env::var(format!("{}_NAME", prefix))
            .unwrap_or_else(|_| provider_type.to_string());

        let mut config = ProviderConfig::new(name, provider_type);

        // Load configuration values
        let config_vars = [
            ("API_KEY", "api_key"),
            ("BASE_URL", "base_url"),
            ("MODEL", "default_model"),
            ("TIMEOUT", "timeout_ms"),
            ("MAX_TOKENS", "max_tokens"),
            ("TEMPERATURE", "temperature"),
            ("REGION", "region"),
            ("PROJECT_ID", "project_id"),
        ];

        for (env_key, config_key) in &config_vars {
            if let Ok(value) = env::var(format!("{}_{}", prefix, env_key)) {
                // Try to parse as different types
                if let Ok(num) = value.parse::<i64>() {
                    config.config.insert(config_key.to_string(), serde_json::Value::Number(num.into()));
                } else if let Ok(float) = value.parse::<f64>() {
                    config.config.insert(config_key.to_string(), serde_json::Value::Number(
                        serde_json::Number::from_f64(float).unwrap_or_else(|| 0.into())
                    ));
                } else if let Ok(bool_val) = value.parse::<bool>() {
                    config.config.insert(config_key.to_string(), serde_json::Value::Bool(bool_val));
                } else {
                    config.config.insert(config_key.to_string(), serde_json::Value::String(value));
                }
            }
        }

        // Set fallback order if specified
        if let Ok(priority) = env::var(format!("{}_PRIORITY", prefix)) {
            config.fallback_order = priority.parse().ok();
        }

        Ok(config)
    }

    /// Validate the loaded configuration
    fn validate_config(&self, config: &IntelligenceConfig) -> Result<(), ConfigError> {
        // Validate providers
        if config.providers.is_empty() {
            warn!("No providers configured, system may not function correctly");
        }

        for provider in &config.providers {
            if provider.name.is_empty() {
                return Err(ConfigError::Validation {
                    field: "provider.name".to_string(),
                    reason: "Provider name cannot be empty".to_string(),
                });
            }

            if provider.provider_type.is_empty() {
                return Err(ConfigError::Validation {
                    field: "provider.provider_type".to_string(),
                    reason: "Provider type cannot be empty".to_string(),
                });
            }
        }

        // Validate metrics config
        if config.metrics.retention_days == 0 {
            return Err(ConfigError::Validation {
                field: "metrics.retention_days".to_string(),
                reason: "Retention days must be greater than 0".to_string(),
            });
        }

        // Validate runtime config
        if config.runtime.max_concurrent_requests == 0 {
            return Err(ConfigError::Validation {
                field: "runtime.max_concurrent_requests".to_string(),
                reason: "Max concurrent requests must be greater than 0".to_string(),
            });
        }

        Ok(())
    }

    /// Load configuration specific for a tenant
    pub fn load_tenant_config(&self, tenant_id: &str) -> Result<TenantLimits, ConfigError> {
        let config = self.load()?;

        Ok(config.tenant_isolation.per_tenant_limits
            .get(tenant_id)
            .cloned()
            .unwrap_or(config.tenant_isolation.default_limits))
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::derivable_impls)]
impl Default for IntelligenceConfig {
    fn default() -> Self {
        Self {
            providers: Vec::new(),
            plugins: Vec::new(),
            failover: FailoverConfig::default(),
            health_check: HealthCheckConfig::default(),
            metrics: MetricsConfig::default(),
            runtime: RuntimeConfig::default(),
            tenant_isolation: TenantIsolationConfig::default(),
            cost_tracking: CostTrackingConfig::default(),
        }
    }
}

/// Auto-discovery service for providers
pub struct ProviderDiscovery {
    #[allow(dead_code)]
    loader: ConfigLoader,
}

impl ProviderDiscovery {
    pub fn new(loader: ConfigLoader) -> Self {
        Self { loader }
    }

    /// Auto-discover available providers based on environment
    pub fn discover(&self) -> Result<Vec<ProviderConfig>, ConfigError> {
        info!("Starting provider auto-discovery");

        let mut discovered = Vec::new();

        // Check for OpenAI
        if env::var("OPENAI_API_KEY").is_ok() || env::var("RIPTIDE_PROVIDER_OPENAI_API_KEY").is_ok() {
            info!("Discovered OpenAI provider");
            discovered.push(self.create_openai_config()?);
        }

        // Check for Anthropic
        if env::var("ANTHROPIC_API_KEY").is_ok() || env::var("RIPTIDE_PROVIDER_ANTHROPIC_API_KEY").is_ok() {
            info!("Discovered Anthropic provider");
            discovered.push(self.create_anthropic_config()?);
        }

        // Check for Azure
        if env::var("AZURE_OPENAI_KEY").is_ok() || env::var("RIPTIDE_PROVIDER_AZURE_API_KEY").is_ok() {
            info!("Discovered Azure OpenAI provider");
            discovered.push(self.create_azure_config()?);
        }

        // Check for local providers (Ollama)
        if self.check_ollama_availability() {
            info!("Discovered Ollama provider");
            discovered.push(self.create_ollama_config()?);
        }

        info!("Auto-discovered {} providers", discovered.len());
        Ok(discovered)
    }

    fn create_openai_config(&self) -> Result<ProviderConfig, ConfigError> {
        let api_key = env::var("OPENAI_API_KEY")
            .or_else(|_| env::var("RIPTIDE_PROVIDER_OPENAI_API_KEY"))
            .map_err(|_| ConfigError::Environment {
                variable: "OPENAI_API_KEY".to_string(),
                reason: "API key not found".to_string(),
            })?;

        let mut config = ProviderConfig::new("openai", "openai")
            .with_config("api_key", serde_json::Value::String(api_key))
            .with_config("default_model", serde_json::Value::String("gpt-4".to_string()))
            .with_fallback_order(1);

        if let Ok(base_url) = env::var("OPENAI_BASE_URL") {
            config = config.with_config("base_url", serde_json::Value::String(base_url));
        }

        Ok(config)
    }

    fn create_anthropic_config(&self) -> Result<ProviderConfig, ConfigError> {
        let api_key = env::var("ANTHROPIC_API_KEY")
            .or_else(|_| env::var("RIPTIDE_PROVIDER_ANTHROPIC_API_KEY"))
            .map_err(|_| ConfigError::Environment {
                variable: "ANTHROPIC_API_KEY".to_string(),
                reason: "API key not found".to_string(),
            })?;

        let config = ProviderConfig::new("anthropic", "anthropic")
            .with_config("api_key", serde_json::Value::String(api_key))
            .with_config("default_model", serde_json::Value::String("claude-3-sonnet-20240229".to_string()))
            .with_fallback_order(2);

        Ok(config)
    }

    fn create_azure_config(&self) -> Result<ProviderConfig, ConfigError> {
        let api_key = env::var("AZURE_OPENAI_KEY")
            .or_else(|_| env::var("RIPTIDE_PROVIDER_AZURE_API_KEY"))
            .map_err(|_| ConfigError::Environment {
                variable: "AZURE_OPENAI_KEY".to_string(),
                reason: "API key not found".to_string(),
            })?;

        let endpoint = env::var("AZURE_OPENAI_ENDPOINT")
            .or_else(|_| env::var("RIPTIDE_PROVIDER_AZURE_BASE_URL"))
            .map_err(|_| ConfigError::Environment {
                variable: "AZURE_OPENAI_ENDPOINT".to_string(),
                reason: "Endpoint not found".to_string(),
            })?;

        let config = ProviderConfig::new("azure", "azure")
            .with_config("api_key", serde_json::Value::String(api_key))
            .with_config("base_url", serde_json::Value::String(endpoint))
            .with_config("default_model", serde_json::Value::String("gpt-4".to_string()))
            .with_fallback_order(3);

        Ok(config)
    }

    fn create_ollama_config(&self) -> Result<ProviderConfig, ConfigError> {
        let base_url = env::var("OLLAMA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());

        let config = ProviderConfig::new("ollama", "ollama")
            .with_config("base_url", serde_json::Value::String(base_url))
            .with_config("default_model", serde_json::Value::String("llama2".to_string()))
            .with_fallback_order(10); // Lower priority for local

        Ok(config)
    }

    fn check_ollama_availability(&self) -> bool {
        // This would make an HTTP request to check if Ollama is running
        // For now, just check if the environment variable is set or default port might be available
        env::var("OLLAMA_BASE_URL").is_ok() ||
        std::net::TcpStream::connect("127.0.0.1:11434").is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_loader_creation() {
        let loader = ConfigLoader::new();
        assert_eq!(loader.env_prefix, "RIPTIDE_");
    }

    #[test]
    fn test_config_loader_with_prefix() {
        let loader = ConfigLoader::new().with_env_prefix("TEST_");
        assert_eq!(loader.env_prefix, "TEST_");
    }

    #[test]
    fn test_provider_discovery() {
        // Set up test environment
        env::set_var("OPENAI_API_KEY", "sk-test");

        let loader = ConfigLoader::new();
        let discovery = ProviderDiscovery::new(loader);
        let providers = discovery.discover().unwrap();

        assert!(!providers.is_empty());

        // Clean up
        env::remove_var("OPENAI_API_KEY");
    }

    #[test]
    fn test_default_config() {
        let config = IntelligenceConfig::default();
        assert!(config.providers.is_empty());
        assert!(config.metrics.enabled);
        assert_eq!(config.metrics.retention_days, 30);
    }
}