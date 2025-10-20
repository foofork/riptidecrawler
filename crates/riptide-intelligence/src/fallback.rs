//! Fallback chain implementation for deterministic provider switching

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::{
    CompletionRequest, CompletionResponse, Cost, IntelligenceError, LlmCapabilities, LlmProvider,
    Result,
};

/// Strategy for selecting the next provider in a fallback chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FallbackStrategy {
    /// Try providers in sequence until one succeeds
    Sequential,
    /// Try the provider with the lowest estimated cost first
    LowestCost,
    /// Try the fastest provider (based on historical data) first
    FastestFirst,
    /// Round-robin through providers
    RoundRobin,
    /// Try providers based on their current availability/health
    HealthBased,
}

/// Configuration for a provider in the fallback chain
#[derive(Clone)]
pub struct FallbackProviderConfig {
    pub provider: Arc<dyn LlmProvider>,
    pub priority: u32,
    pub max_retries: u32,
    pub enabled: bool,
}

impl FallbackProviderConfig {
    pub fn new(provider: Arc<dyn LlmProvider>) -> Self {
        Self {
            provider,
            priority: 0,
            max_retries: 1,
            enabled: true,
        }
    }

    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Statistics for fallback chain operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub provider_usage: std::collections::HashMap<String, u64>,
    pub fallback_triggers: u64,
    pub average_providers_tried: f32,
}

impl FallbackStats {
    fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            provider_usage: std::collections::HashMap::new(),
            fallback_triggers: 0,
            average_providers_tried: 0.0,
        }
    }

    fn record_request(&mut self, provider_name: &str, providers_tried: u32, success: bool) {
        self.total_requests += 1;
        if success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }

        *self
            .provider_usage
            .entry(provider_name.to_string())
            .or_insert(0) += 1;

        if providers_tried > 1 {
            self.fallback_triggers += 1;
        }

        // Update average providers tried
        let total_providers_tried = self.average_providers_tried * (self.total_requests - 1) as f32
            + providers_tried as f32;
        self.average_providers_tried = total_providers_tried / self.total_requests as f32;
    }
}

/// Fallback chain that tries multiple providers in sequence
pub struct FallbackChain {
    providers: Vec<FallbackProviderConfig>,
    strategy: FallbackStrategy,
    stats: parking_lot::RwLock<FallbackStats>,
    round_robin_index: parking_lot::RwLock<usize>,
}

impl FallbackChain {
    /// Create a new fallback chain with sequential strategy
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            strategy: FallbackStrategy::Sequential,
            stats: parking_lot::RwLock::new(FallbackStats::new()),
            round_robin_index: parking_lot::RwLock::new(0),
        }
    }

    /// Create a fallback chain with a specific strategy
    pub fn with_strategy(strategy: FallbackStrategy) -> Self {
        Self {
            providers: Vec::new(),
            strategy,
            stats: parking_lot::RwLock::new(FallbackStats::new()),
            round_robin_index: parking_lot::RwLock::new(0),
        }
    }

    /// Add a provider to the fallback chain
    pub fn add_provider(&mut self, config: FallbackProviderConfig) -> &mut Self {
        info!(
            "Adding provider '{}' to fallback chain",
            config.provider.name()
        );
        self.providers.push(config);
        self
    }

    /// Add a provider with default configuration
    pub fn add_provider_simple(&mut self, provider: Arc<dyn LlmProvider>) -> &mut Self {
        self.add_provider(FallbackProviderConfig::new(provider));
        self
    }

    /// Get the current strategy
    pub fn strategy(&self) -> &FallbackStrategy {
        &self.strategy
    }

    /// Set the fallback strategy
    pub fn set_strategy(&mut self, strategy: FallbackStrategy) {
        self.strategy = strategy;
    }

    /// Get fallback statistics
    pub fn stats(&self) -> FallbackStats {
        self.stats.read().clone()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        *self.stats.write() = FallbackStats::new();
    }

    /// Get the number of enabled providers
    pub fn enabled_provider_count(&self) -> usize {
        self.providers.iter().filter(|p| p.enabled).count()
    }

    /// Get provider names in order
    pub fn provider_names(&self) -> Vec<String> {
        self.providers
            .iter()
            .filter(|p| p.enabled)
            .map(|p| p.provider.name().to_string())
            .collect()
    }

    /// Order providers based on the current strategy
    fn order_providers(&self, request: &CompletionRequest) -> Vec<&FallbackProviderConfig> {
        let mut enabled_providers: Vec<_> = self.providers.iter().filter(|p| p.enabled).collect();

        match self.strategy {
            FallbackStrategy::Sequential => {
                // Sort by priority (lower number = higher priority)
                enabled_providers.sort_by_key(|p| p.priority);
            }
            FallbackStrategy::LowestCost => {
                // Estimate cost for each provider and sort by lowest cost
                let token_estimate = self.estimate_tokens(request);
                enabled_providers.sort_by(|a, b| {
                    let cost_a = a.provider.estimate_cost(token_estimate).total_cost;
                    let cost_b = b.provider.estimate_cost(token_estimate).total_cost;
                    cost_a
                        .partial_cmp(&cost_b)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            FallbackStrategy::FastestFirst => {
                // For now, use priority as a proxy for speed
                // In a real implementation, this would use historical response times
                enabled_providers.sort_by_key(|p| p.priority);
            }
            FallbackStrategy::RoundRobin => {
                // Rotate the starting provider
                if !enabled_providers.is_empty() {
                    let mut index = self.round_robin_index.write();
                    *index = (*index + 1) % enabled_providers.len();
                    enabled_providers.rotate_left(*index);
                }
            }
            FallbackStrategy::HealthBased => {
                // For now, use priority. In a real implementation, this would
                // check provider health and sort by availability
                enabled_providers.sort_by_key(|p| p.priority);
            }
        }

        enabled_providers
    }

    /// Estimate token count for cost calculation
    fn estimate_tokens(&self, request: &CompletionRequest) -> usize {
        // Simple token estimation: roughly 4 characters per token
        let total_chars: usize = request.messages.iter().map(|m| m.content.len()).sum();
        total_chars / 4 + request.max_tokens.unwrap_or(100) as usize
    }

    /// Try a specific provider with retries
    async fn try_provider(
        &self,
        config: &FallbackProviderConfig,
        request: &CompletionRequest,
    ) -> Result<CompletionResponse> {
        let mut last_error = None;

        for attempt in 0..=config.max_retries {
            match config.provider.complete(request.clone()).await {
                Ok(response) => {
                    if attempt > 0 {
                        info!(
                            "Provider '{}' succeeded on retry attempt {}",
                            config.provider.name(),
                            attempt
                        );
                    }
                    return Ok(response);
                }
                Err(error) => {
                    warn!(
                        "Provider '{}' failed on attempt {}: {}",
                        config.provider.name(),
                        attempt + 1,
                        error
                    );
                    last_error = Some(error);

                    // Don't retry on certain error types
                    match &last_error {
                        Some(IntelligenceError::InvalidRequest(_)) => break,
                        Some(IntelligenceError::RateLimit { .. }) => break,
                        _ => {}
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| IntelligenceError::Provider("Unknown error".to_string())))
    }

    /// Check if all providers are available
    pub async fn check_all_providers(&self) -> std::collections::HashMap<String, bool> {
        let mut results = std::collections::HashMap::new();

        for config in &self.providers {
            if config.enabled {
                let is_available = config.provider.is_available().await;
                results.insert(config.provider.name().to_string(), is_available);
            }
        }

        results
    }

    /// Get the primary (first) provider
    pub fn primary_provider(&self) -> Option<&Arc<dyn LlmProvider>> {
        self.providers
            .iter()
            .filter(|p| p.enabled)
            .min_by_key(|p| p.priority)
            .map(|p| &p.provider)
    }

    /// Enable or disable a provider by name
    pub fn set_provider_enabled(&mut self, provider_name: &str, enabled: bool) {
        for config in &mut self.providers {
            if config.provider.name() == provider_name {
                config.enabled = enabled;
                info!(
                    "Provider '{}' {}",
                    provider_name,
                    if enabled { "enabled" } else { "disabled" }
                );
                return;
            }
        }
        warn!("Provider '{}' not found in fallback chain", provider_name);
    }
}

impl Default for FallbackChain {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmProvider for FallbackChain {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        if self.providers.is_empty() {
            return Err(IntelligenceError::Configuration(
                "No providers configured in fallback chain".to_string(),
            ));
        }

        let ordered_providers = self.order_providers(&request);
        if ordered_providers.is_empty() {
            return Err(IntelligenceError::Configuration(
                "No enabled providers in fallback chain".to_string(),
            ));
        }

        let mut providers_tried = 0;
        let mut last_error = None;

        for config in &ordered_providers {
            providers_tried += 1;
            info!(
                "Trying provider '{}' (attempt {} of {})",
                config.provider.name(),
                providers_tried,
                self.enabled_provider_count()
            );

            match self.try_provider(config, &request).await {
                Ok(response) => {
                    info!(
                        "Provider '{}' succeeded after {} providers tried",
                        config.provider.name(),
                        providers_tried
                    );

                    // Record successful request
                    self.stats.write().record_request(
                        config.provider.name(),
                        providers_tried,
                        true,
                    );

                    return Ok(response);
                }
                Err(error) => {
                    error!("Provider '{}' failed: {}", config.provider.name(), error);
                    last_error = Some(error);
                }
            }
        }

        // All providers failed
        if let Some(error) = last_error {
            error!(
                "All {} providers failed in fallback chain, last error: {}",
                providers_tried, error
            );
        } else {
            error!("All {} providers failed in fallback chain", providers_tried);
        }

        // Record failed request
        if let Some(last_config) = ordered_providers.last() {
            self.stats
                .write()
                .record_request(last_config.provider.name(), providers_tried, false);
        }

        Err(IntelligenceError::AllProvidersFailed)
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        if self.providers.is_empty() {
            return Err(IntelligenceError::Configuration(
                "No providers configured in fallback chain".to_string(),
            ));
        }

        let enabled_providers: Vec<_> = self.providers.iter().filter(|p| p.enabled).collect();

        if enabled_providers.is_empty() {
            return Err(IntelligenceError::Configuration(
                "No enabled providers in fallback chain".to_string(),
            ));
        }

        for config in enabled_providers {
            match config.provider.embed(text).await {
                Ok(embedding) => return Ok(embedding),
                Err(error) => {
                    warn!(
                        "Provider '{}' failed for embedding: {}",
                        config.provider.name(),
                        error
                    );
                }
            }
        }

        Err(IntelligenceError::AllProvidersFailed)
    }

    fn capabilities(&self) -> LlmCapabilities {
        // Return capabilities of the primary provider, or a merged view
        if let Some(primary) = self.primary_provider() {
            primary.capabilities()
        } else {
            // Return empty capabilities if no providers
            LlmCapabilities {
                provider_name: "fallback-chain".to_string(),
                models: Vec::new(),
                supports_embeddings: false,
                supports_streaming: false,
                supports_functions: false,
                max_context_length: 0,
                rate_limits: std::collections::HashMap::new(),
            }
        }
    }

    fn estimate_cost(&self, tokens: usize) -> Cost {
        // Use the primary provider for cost estimation
        if let Some(primary) = self.primary_provider() {
            primary.estimate_cost(tokens)
        } else {
            Cost::zero("USD")
        }
    }

    async fn health_check(&self) -> Result<()> {
        // Check if at least one provider is healthy
        for config in &self.providers {
            if config.enabled && config.provider.health_check().await.is_ok() {
                return Ok(());
            }
        }

        Err(IntelligenceError::Provider(
            "No healthy providers in fallback chain".to_string(),
        ))
    }

    fn name(&self) -> &str {
        "fallback-chain"
    }

    async fn is_available(&self) -> bool {
        // Return true if at least one provider is available
        for config in &self.providers {
            if config.enabled && config.provider.is_available().await {
                return true;
            }
        }
        false
    }
}

/// Helper function to create a simple fallback chain
pub fn create_fallback_chain(providers: Vec<Arc<dyn LlmProvider>>) -> FallbackChain {
    let mut chain = FallbackChain::new();
    for (index, provider) in providers.into_iter().enumerate() {
        let config = FallbackProviderConfig::new(provider).with_priority(index as u32);
        chain.add_provider(config);
    }
    chain
}

/// Helper function to create a fallback chain with custom strategy
pub fn create_fallback_chain_with_strategy(
    providers: Vec<Arc<dyn LlmProvider>>,
    strategy: FallbackStrategy,
) -> FallbackChain {
    let mut chain = FallbackChain::with_strategy(strategy);
    for (index, provider) in providers.into_iter().enumerate() {
        let config = FallbackProviderConfig::new(provider).with_priority(index as u32);
        chain.add_provider(config);
    }
    chain
}

#[cfg(all(test, feature = "mock"))]
mod tests {
    use super::*;
    use crate::mock_provider::{create_failing_provider, FailureMode, MockLlmProvider};
    use crate::provider::Message;

    #[tokio::test]
    async fn test_fallback_chain_success() {
        let provider1 = Arc::new(MockLlmProvider::with_name("provider1"));
        let provider2 = Arc::new(MockLlmProvider::with_name("provider2"));

        let chain = create_fallback_chain(vec![provider1, provider2]);

        let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);

        let result = chain.complete(request).await;
        assert!(result.is_ok());

        let stats = chain.stats();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.successful_requests, 1);
    }

    #[tokio::test]
    async fn test_fallback_chain_failover() {
        let provider1 = Arc::new(create_failing_provider(FailureMode::AlwaysFail));
        let provider2 = Arc::new(MockLlmProvider::with_name("provider2"));

        let chain = create_fallback_chain(vec![provider1, provider2]);

        let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);

        let result = chain.complete(request).await;
        assert!(result.is_ok());

        let stats = chain.stats();
        assert_eq!(stats.fallback_triggers, 1);
        assert_eq!(stats.average_providers_tried, 2.0);
    }

    #[tokio::test]
    async fn test_fallback_chain_all_fail() {
        let provider1 = Arc::new(create_failing_provider(FailureMode::AlwaysFail));
        let provider2 = Arc::new(create_failing_provider(FailureMode::AlwaysFail));

        let chain = create_fallback_chain(vec![provider1, provider2]);

        let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);

        let result = chain.complete(request).await;
        assert!(matches!(result, Err(IntelligenceError::AllProvidersFailed)));
    }

    #[tokio::test]
    async fn test_fallback_strategies() {
        let provider1 = Arc::new(MockLlmProvider::with_name("provider1"));
        let provider2 = Arc::new(MockLlmProvider::with_name("provider2"));

        // Test round-robin strategy
        let chain = create_fallback_chain_with_strategy(
            vec![provider1.clone(), provider2.clone()],
            FallbackStrategy::RoundRobin,
        );

        let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);

        // Make multiple requests to test round-robin
        for _ in 0..4 {
            let result = chain.complete(request.clone()).await;
            assert!(result.is_ok());
        }

        let stats = chain.stats();
        assert_eq!(stats.total_requests, 4);
    }

    #[tokio::test]
    async fn test_provider_enable_disable() {
        let provider1 = Arc::new(MockLlmProvider::with_name("provider1"));
        let provider2 = Arc::new(MockLlmProvider::with_name("provider2"));

        let mut chain = create_fallback_chain(vec![provider1, provider2]);

        assert_eq!(chain.enabled_provider_count(), 2);

        chain.set_provider_enabled("provider1", false);
        assert_eq!(chain.enabled_provider_count(), 1);

        let provider_names = chain.provider_names();
        assert_eq!(provider_names, vec!["provider2"]);
    }

    #[tokio::test]
    async fn test_empty_fallback_chain() {
        let chain = FallbackChain::new();

        let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);

        let result = chain.complete(request).await;
        assert!(matches!(result, Err(IntelligenceError::Configuration(_))));
    }

    #[tokio::test]
    async fn test_check_all_providers() {
        let provider1 = Arc::new(MockLlmProvider::with_name("provider1"));
        let provider2 = Arc::new(create_failing_provider(FailureMode::AlwaysFail));

        let chain = create_fallback_chain(vec![provider1, provider2]);

        let health_status = chain.check_all_providers().await;
        assert_eq!(health_status.len(), 2);
        assert_eq!(health_status.get("provider1"), Some(&true));
        assert_eq!(health_status.get("mock"), Some(&false)); // Failing provider
    }
}
