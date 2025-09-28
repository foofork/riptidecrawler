//! RipTide Intelligence - LLM Abstraction Layer
//!
//! This crate provides a vendor-agnostic abstraction layer for Large Language Models (LLMs)
//! with built-in safety guarantees including timeouts, circuit breakers, and fallback chains.

use std::sync::Arc;
use thiserror::Error;

pub mod provider;
pub mod registry;
pub mod timeout;
pub mod circuit_breaker;
pub mod fallback;
pub mod failover;
pub mod metrics;
pub mod runtime_switch;
pub mod providers;
pub mod health;
pub mod plugin;

#[cfg(feature = "mock")]
pub mod mock_provider;

// Re-export core types
pub use provider::{
    LlmProvider, CompletionRequest, CompletionResponse, LlmCapabilities,
    Cost, ModelInfo, Usage, Message, Role
};
pub use registry::{LlmRegistry, ProviderConfig};
pub use timeout::{TimeoutWrapper, with_timeout, with_custom_timeout};
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState, with_circuit_breaker, with_custom_circuit_breaker};
pub use fallback::{FallbackChain, FallbackStrategy, create_fallback_chain, create_fallback_chain_with_strategy};
pub use failover::{FailoverManager, FailoverConfig, ProviderPriority, ProviderState, FailoverStatistics};
pub use metrics::{MetricsCollector, LlmOpsDashboard, TimeWindow, RequestMetrics, AggregatedMetrics};
pub use runtime_switch::{RuntimeSwitchManager, RuntimeSwitchConfig, SwitchState, GradualRolloutConfig};
pub use providers::{
    OpenAIProvider, AnthropicProvider, OllamaProvider, LocalAIProvider,
    AzureOpenAIProvider, BedrockProvider, VertexAIProvider,
    create_provider_from_config, register_builtin_providers
};

#[cfg(feature = "mock")]
pub use mock_provider::MockLlmProvider;

/// Main error type for the intelligence layer
#[derive(Error, Debug)]
pub enum IntelligenceError {
    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Timeout error: operation took longer than {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Circuit breaker open: {reason}")]
    CircuitOpen { reason: String },

    #[error("All providers failed in fallback chain")]
    AllProvidersFailed,

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Rate limit exceeded: {retry_after_ms}ms")]
    RateLimit { retry_after_ms: u64 },

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Network error: {0}")]
    Network(String),
}

pub type Result<T> = std::result::Result<T, IntelligenceError>;

/// Intelligence client that combines all safety features
pub struct IntelligenceClient {
    registry: Arc<LlmRegistry>,
    default_provider: String,
}

impl IntelligenceClient {
    /// Create a new intelligence client with the given registry
    pub fn new(registry: LlmRegistry, default_provider: impl Into<String>) -> Self {
        Self {
            registry: Arc::new(registry),
            default_provider: default_provider.into(),
        }
    }

    /// Get a provider by name
    pub fn provider(&self, name: &str) -> Option<Arc<dyn LlmProvider>> {
        self.registry.get_provider(name)
    }

    /// Get the default provider
    pub fn default_provider(&self) -> Option<Arc<dyn LlmProvider>> {
        self.registry.get_provider(&self.default_provider)
    }

    /// Complete a text using the default provider
    pub async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        match self.default_provider() {
            Some(provider) => provider.complete(request).await,
            None => Err(IntelligenceError::Configuration(
                format!("Default provider '{}' not found", self.default_provider)
            )),
        }
    }

    /// Complete a text using a specific provider
    pub async fn complete_with_provider(
        &self,
        provider_name: &str,
        request: CompletionRequest
    ) -> Result<CompletionResponse> {
        match self.provider(provider_name) {
            Some(provider) => provider.complete(request).await,
            None => Err(IntelligenceError::Configuration(
                format!("Provider '{}' not found", provider_name)
            )),
        }
    }

    /// Generate embeddings using the default provider
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        match self.default_provider() {
            Some(provider) => provider.embed(text).await,
            None => Err(IntelligenceError::Configuration(
                format!("Default provider '{}' not found", self.default_provider)
            )),
        }
    }

    /// Get capabilities of a provider
    pub fn capabilities(&self, provider_name: &str) -> Option<LlmCapabilities> {
        self.provider(provider_name).map(|p| p.capabilities())
    }

    /// Estimate cost for a request
    pub fn estimate_cost(&self, provider_name: &str, tokens: usize) -> Option<Cost> {
        self.provider(provider_name).map(|p| p.estimate_cost(tokens))
    }
}