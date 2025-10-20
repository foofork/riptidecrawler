//! Built-in provider implementations
//!
//! This module contains concrete implementations of LLM providers for various services:
//! - OpenAI (GPT models)
//! - Anthropic (Claude models)
//! - Local models (Ollama, LocalAI)
//! - Azure OpenAI
//! - AWS Bedrock
//! - Google Vertex AI

pub mod anthropic;
pub mod aws_bedrock;
pub mod azure;
pub mod base;
pub mod google_vertex;
pub mod local;
pub mod openai;

// Re-export provider implementations
pub use anthropic::AnthropicProvider;
pub use aws_bedrock::BedrockProvider;
pub use azure::AzureOpenAIProvider;
pub use google_vertex::VertexAIProvider;
pub use local::{LocalAIProvider, OllamaProvider};
pub use openai::OpenAIProvider;

use std::sync::Arc;

use crate::{registry::ProviderConfig, IntelligenceError, LlmProvider, Result};

/// Factory function to create providers from configuration
pub fn create_provider_from_config(config: &ProviderConfig) -> Result<Arc<dyn LlmProvider>> {
    match config.provider_type.as_str() {
        "openai" => {
            let api_key = get_config_string(config, "api_key")?;
            let base_url = get_config_string_optional(config, "base_url");
            Ok(Arc::new(OpenAIProvider::new(api_key, base_url)?))
        }
        "anthropic" => {
            let api_key = get_config_string(config, "api_key")?;
            Ok(Arc::new(AnthropicProvider::new(api_key)?))
        }
        "ollama" => {
            let base_url = get_config_string_optional(config, "base_url")
                .unwrap_or_else(|| "http://localhost:11434".to_string());
            Ok(Arc::new(OllamaProvider::new(base_url)?))
        }
        "localai" => {
            let base_url = get_config_string(config, "base_url")?;
            Ok(Arc::new(LocalAIProvider::new(base_url)?))
        }
        "azure_openai" => {
            let api_key = get_config_string(config, "api_key")?;
            let endpoint = get_config_string(config, "endpoint")?;
            let api_version = get_config_string_optional(config, "api_version")
                .unwrap_or_else(|| "2023-12-01-preview".to_string());
            Ok(Arc::new(AzureOpenAIProvider::new(
                api_key,
                endpoint,
                api_version,
            )?))
        }
        "aws_bedrock" => {
            let region = get_config_string(config, "region")?;
            let access_key = get_config_string_optional(config, "access_key");
            let secret_key = get_config_string_optional(config, "secret_key");
            Ok(Arc::new(BedrockProvider::new(
                region, access_key, secret_key,
            )?))
        }
        "google_vertex" => {
            let project_id = get_config_string(config, "project_id")?;
            let location = get_config_string_optional(config, "location")
                .unwrap_or_else(|| "us-central1".to_string());
            Ok(Arc::new(VertexAIProvider::new(project_id, location)?))
        }
        _ => Err(IntelligenceError::Configuration(format!(
            "Unknown provider type: {}",
            config.provider_type
        ))),
    }
}

/// Helper to get string configuration value
fn get_config_string(config: &ProviderConfig, key: &str) -> Result<String> {
    config
        .config
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            IntelligenceError::Configuration(format!("Missing required configuration key: {}", key))
        })
}

/// Helper to get optional string configuration value
fn get_config_string_optional(config: &ProviderConfig, key: &str) -> Option<String> {
    config
        .config
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Helper to get boolean configuration value
#[allow(dead_code)]
fn get_config_bool(config: &ProviderConfig, key: &str, default: bool) -> bool {
    config
        .config
        .get(key)
        .and_then(|v| v.as_bool())
        .unwrap_or(default)
}

/// Helper to get number configuration value
#[allow(dead_code)]
fn get_config_f64(config: &ProviderConfig, key: &str, default: f64) -> f64 {
    config
        .config
        .get(key)
        .and_then(|v| v.as_f64())
        .unwrap_or(default)
}

/// Register all built-in provider factories
pub fn register_builtin_providers(registry: &crate::registry::LlmRegistry) -> Result<()> {
    // OpenAI
    registry.register_factory("openai", create_provider_from_config)?;

    // Anthropic
    registry.register_factory("anthropic", create_provider_from_config)?;

    // Local providers
    registry.register_factory("ollama", create_provider_from_config)?;

    registry.register_factory("localai", create_provider_from_config)?;

    // Cloud providers
    registry.register_factory("azure_openai", create_provider_from_config)?;

    registry.register_factory("aws_bedrock", create_provider_from_config)?;

    registry.register_factory("google_vertex", create_provider_from_config)?;

    Ok(())
}

// ============================================================================
// Provider Health Monitoring
// ============================================================================

use crate::circuit_breaker::{CircuitBreaker, CircuitBreakerStats, CircuitState};
use crate::health::{HealthLevel, ProviderMetrics as HealthProviderMetrics};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Health status for a provider
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Critical,
    Unavailable,
}

impl From<HealthLevel> for HealthStatus {
    fn from(level: HealthLevel) -> Self {
        match level {
            HealthLevel::Healthy => HealthStatus::Healthy,
            HealthLevel::Degraded => HealthStatus::Degraded,
            HealthLevel::Critical => HealthStatus::Critical,
            HealthLevel::Unavailable => HealthStatus::Unavailable,
        }
    }
}

/// Complete provider health information with circuit breaker integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub provider_name: String,
    pub status: HealthStatus,
    pub last_check: SystemTime,
    pub consecutive_failures: u32,
    pub circuit_breaker_state: CircuitState,
    pub error_rate: f64,
    pub avg_response_time_ms: f64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub uptime_percentage: f64,
    pub last_error_message: Option<String>,
}

impl ProviderHealth {
    /// Create a new provider health status
    fn new(provider_name: String) -> Self {
        Self {
            provider_name,
            status: HealthStatus::Unavailable,
            last_check: SystemTime::now(),
            consecutive_failures: 0,
            circuit_breaker_state: CircuitState::Closed,
            error_rate: 0.0,
            avg_response_time_ms: 0.0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            uptime_percentage: 100.0,
            last_error_message: None,
        }
    }

    /// Update health status from circuit breaker stats
    fn update_from_circuit_breaker(&mut self, stats: &CircuitBreakerStats) {
        self.circuit_breaker_state = stats.state.clone();
        self.total_requests = stats.total_requests;
        self.successful_requests = stats.successful_requests;
        self.failed_requests = stats.failed_requests;
        self.consecutive_failures = stats.current_failure_count;
        self.last_check = SystemTime::now();

        // Calculate error rate
        self.error_rate = if stats.total_requests > 0 {
            (stats.failed_requests as f64 / stats.total_requests as f64) * 100.0
        } else {
            0.0
        };

        // Calculate uptime percentage
        self.uptime_percentage = if stats.total_requests > 0 {
            (stats.successful_requests as f64 / stats.total_requests as f64) * 100.0
        } else {
            100.0
        };

        // Determine health status based on circuit breaker state and error rate
        self.status = match stats.state {
            CircuitState::Closed => {
                if self.error_rate < 5.0 {
                    HealthStatus::Healthy
                } else if self.error_rate < 20.0 {
                    HealthStatus::Degraded
                } else {
                    HealthStatus::Critical
                }
            }
            CircuitState::HalfOpen => HealthStatus::Degraded,
            CircuitState::Open => HealthStatus::Unavailable,
        };
    }

    /// Update health status from health monitor metrics
    fn update_from_health_metrics(&mut self, metrics: &HealthProviderMetrics) {
        self.avg_response_time_ms = metrics.avg_response_time.as_millis() as f64;
        self.total_requests = metrics.total_requests;
        self.successful_requests = metrics.successful_requests;
        self.failed_requests = metrics.failed_requests;
        self.error_rate = metrics.error_rate;
        self.uptime_percentage = metrics.uptime_percentage;
        self.last_check = SystemTime::now();
    }

    /// Check if provider is available for requests
    pub fn is_available(&self) -> bool {
        matches!(self.status, HealthStatus::Healthy | HealthStatus::Degraded)
    }

    /// Get health score (0-100)
    pub fn health_score(&self) -> f64 {
        match self.status {
            HealthStatus::Healthy => 100.0 - self.error_rate,
            HealthStatus::Degraded => 50.0 - (self.error_rate / 2.0),
            HealthStatus::Critical => 25.0 - (self.error_rate / 4.0),
            HealthStatus::Unavailable => 0.0,
        }
    }
}

/// Provider manager with health monitoring capabilities
pub struct ProviderManager {
    providers: Arc<RwLock<HashMap<String, Arc<dyn LlmProvider>>>>,
    circuit_breakers: Arc<RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
    health_status: Arc<RwLock<HashMap<String, ProviderHealth>>>,
}

impl ProviderManager {
    /// Create a new provider manager
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            health_status: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a provider with circuit breaker protection
    pub fn add_provider(
        &self,
        name: String,
        provider: Arc<dyn LlmProvider>,
        circuit_breaker: Option<Arc<CircuitBreaker>>,
    ) {
        let mut providers = self.providers.write();
        providers.insert(name.clone(), provider);

        if let Some(cb) = circuit_breaker {
            let mut circuit_breakers = self.circuit_breakers.write();
            circuit_breakers.insert(name.clone(), cb);
        }

        let mut health_status = self.health_status.write();
        health_status.insert(name.clone(), ProviderHealth::new(name));
    }

    /// Remove a provider
    pub fn remove_provider(&self, name: &str) -> Option<Arc<dyn LlmProvider>> {
        let mut providers = self.providers.write();
        let provider = providers.remove(name);

        let mut circuit_breakers = self.circuit_breakers.write();
        circuit_breakers.remove(name);

        let mut health_status = self.health_status.write();
        health_status.remove(name);

        provider
    }

    /// Check health of a specific provider
    pub async fn check_provider_health(&self, provider: &str) -> Result<ProviderHealth> {
        // Get provider and circuit breaker
        let (provider_arc, circuit_breaker_opt) = {
            let providers = self.providers.read();
            let circuit_breakers = self.circuit_breakers.read();

            let provider_arc = providers.get(provider).cloned().ok_or_else(|| {
                IntelligenceError::Configuration(format!("Provider '{}' not found", provider))
            })?;

            let circuit_breaker_opt = circuit_breakers.get(provider).cloned();

            (provider_arc, circuit_breaker_opt)
        };

        let mut health = ProviderHealth::new(provider.to_string());

        // Check circuit breaker status if available
        if let Some(cb) = &circuit_breaker_opt {
            let stats = cb.stats();
            health.update_from_circuit_breaker(&stats);

            // Perform health check through circuit breaker
            match cb.health_check().await {
                Ok(_) => {
                    health.consecutive_failures = 0;
                }
                Err(e) => {
                    health.consecutive_failures += 1;
                    health.last_error_message = Some(e.to_string());
                }
            }
        } else {
            // Direct health check without circuit breaker
            match provider_arc.health_check().await {
                Ok(_) => {
                    health.status = HealthStatus::Healthy;
                    health.consecutive_failures = 0;
                }
                Err(e) => {
                    health.status = HealthStatus::Unavailable;
                    health.consecutive_failures += 1;
                    health.last_error_message = Some(e.to_string());
                }
            }
        }

        health.last_check = SystemTime::now();

        // Update stored health status
        {
            let mut health_status = self.health_status.write();
            health_status.insert(provider.to_string(), health.clone());
        }

        Ok(health)
    }

    /// Get health status for all providers
    pub async fn get_all_health_status(&self) -> Vec<ProviderHealth> {
        let provider_names: Vec<String> = {
            let providers = self.providers.read();
            providers.keys().cloned().collect()
        };

        let mut results = Vec::new();
        for name in provider_names {
            if let Ok(health) = self.check_provider_health(&name).await {
                results.push(health);
            }
        }

        results
    }

    /// Get cached health status without performing checks
    pub fn get_cached_health_status(&self) -> HashMap<String, ProviderHealth> {
        self.health_status.read().clone()
    }

    /// Get healthy providers only
    pub fn get_healthy_providers(&self) -> Vec<String> {
        let health_status = self.health_status.read();
        health_status
            .iter()
            .filter(|(_, health)| health.is_available())
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Get provider by name
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn LlmProvider>> {
        self.providers.read().get(name).cloned()
    }

    /// Get circuit breaker for a provider
    pub fn get_circuit_breaker(&self, name: &str) -> Option<Arc<CircuitBreaker>> {
        self.circuit_breakers.read().get(name).cloned()
    }

    /// Update health status from external health monitor
    pub fn update_health_from_monitor(&self, provider_name: &str, metrics: &HealthProviderMetrics) {
        let mut health_status = self.health_status.write();
        if let Some(health) = health_status.get_mut(provider_name) {
            health.update_from_health_metrics(metrics);
        }
    }

    /// Get providers sorted by health score
    pub fn get_providers_by_health_score(&self) -> Vec<(String, f64, HealthStatus)> {
        let health_status = self.health_status.read();
        let mut providers: Vec<_> = health_status
            .iter()
            .map(|(name, health)| (name.clone(), health.health_score(), health.status.clone()))
            .collect();

        // Sort by health score descending
        providers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        providers
    }

    /// Check if any providers are available
    pub fn has_healthy_providers(&self) -> bool {
        let health_status = self.health_status.read();
        health_status.values().any(|health| health.is_available())
    }

    /// Get overall system health percentage
    pub fn get_system_health_percentage(&self) -> f64 {
        let health_status = self.health_status.read();
        if health_status.is_empty() {
            return 100.0;
        }

        let total_score: f64 = health_status.values().map(|h| h.health_score()).sum();
        total_score / health_status.len() as f64
    }
}

impl Default for ProviderManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "mock"))]
mod provider_health_tests {
    use super::*;
    use crate::circuit_breaker::with_circuit_breaker;
    use crate::mock_provider::MockLlmProvider;

    #[tokio::test]
    async fn test_provider_manager_creation() {
        let manager = ProviderManager::new();
        assert_eq!(manager.get_all_health_status().await.len(), 0);
    }

    #[tokio::test]
    async fn test_add_provider_without_circuit_breaker() {
        let manager = ProviderManager::new();
        let provider = Arc::new(MockLlmProvider::new());

        manager.add_provider("test".to_string(), provider, None);
        assert!(manager.get_provider("test").is_some());
    }

    #[tokio::test]
    async fn test_add_provider_with_circuit_breaker() {
        let manager = ProviderManager::new();
        let provider = Arc::new(MockLlmProvider::new());
        let circuit_breaker = Arc::new(with_circuit_breaker(provider.clone()));

        manager.add_provider("test".to_string(), provider, Some(circuit_breaker));
        assert!(manager.get_provider("test").is_some());
        assert!(manager.get_circuit_breaker("test").is_some());
    }

    #[tokio::test]
    async fn test_check_provider_health() {
        let manager = ProviderManager::new();
        let provider = Arc::new(MockLlmProvider::new());

        manager.add_provider("test".to_string(), provider, None);

        let health = manager.check_provider_health("test").await.unwrap();
        assert_eq!(health.provider_name, "test");
        assert_eq!(health.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_get_healthy_providers() {
        let manager = ProviderManager::new();
        let provider1 = Arc::new(MockLlmProvider::new());
        let provider2 = Arc::new(MockLlmProvider::new().fail_after(0));

        manager.add_provider("healthy".to_string(), provider1, None);
        manager.add_provider("unhealthy".to_string(), provider2, None);

        // Check health to populate status
        let _ = manager.check_provider_health("healthy").await;
        let _ = manager.check_provider_health("unhealthy").await;

        let healthy = manager.get_healthy_providers();
        assert!(healthy.contains(&"healthy".to_string()));
    }

    #[tokio::test]
    async fn test_health_score_calculation() {
        let mut health = ProviderHealth::new("test".to_string());
        health.status = HealthStatus::Healthy;
        health.error_rate = 2.0;

        let score = health.health_score();
        assert!(score > 95.0 && score < 100.0);
    }

    #[tokio::test]
    async fn test_system_health_percentage() {
        let manager = ProviderManager::new();
        let provider1 = Arc::new(MockLlmProvider::new());
        let provider2 = Arc::new(MockLlmProvider::new());

        manager.add_provider("test1".to_string(), provider1, None);
        manager.add_provider("test2".to_string(), provider2, None);

        let _ = manager.check_provider_health("test1").await;
        let _ = manager.check_provider_health("test2").await;

        let system_health = manager.get_system_health_percentage();
        assert!((0.0..=100.0).contains(&system_health));
    }
}
