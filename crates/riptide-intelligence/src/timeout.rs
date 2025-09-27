//! Timeout wrapper for LLM providers with 5-second hard timeout

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use crate::{
    LlmProvider, CompletionRequest, CompletionResponse, LlmCapabilities,
    Cost, IntelligenceError, Result
};

/// Wrapper that adds timeout functionality to any LLM provider
pub struct TimeoutWrapper {
    inner: Arc<dyn LlmProvider>,
    timeout_duration: Duration,
}

impl TimeoutWrapper {
    /// Create a new timeout wrapper with the default 5-second timeout
    pub fn new(provider: Arc<dyn LlmProvider>) -> Self {
        Self {
            inner: provider,
            timeout_duration: Duration::from_secs(5),
        }
    }

    /// Create a new timeout wrapper with a custom timeout
    pub fn with_timeout(provider: Arc<dyn LlmProvider>, timeout_duration: Duration) -> Self {
        Self {
            inner: provider,
            timeout_duration,
        }
    }

    /// Get the configured timeout duration
    pub fn timeout_duration(&self) -> Duration {
        self.timeout_duration
    }

    /// Update the timeout duration
    pub fn set_timeout(&mut self, duration: Duration) {
        self.timeout_duration = duration;
    }

    /// Get the wrapped provider
    pub fn inner(&self) -> &Arc<dyn LlmProvider> {
        &self.inner
    }
}

#[async_trait]
impl LlmProvider for TimeoutWrapper {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        match timeout(self.timeout_duration, self.inner.complete(request)).await {
            Ok(result) => result,
            Err(_) => Err(IntelligenceError::Timeout {
                timeout_ms: self.timeout_duration.as_millis() as u64,
            }),
        }
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        match timeout(self.timeout_duration, self.inner.embed(text)).await {
            Ok(result) => result,
            Err(_) => Err(IntelligenceError::Timeout {
                timeout_ms: self.timeout_duration.as_millis() as u64,
            }),
        }
    }

    fn capabilities(&self) -> LlmCapabilities {
        self.inner.capabilities()
    }

    fn estimate_cost(&self, tokens: usize) -> Cost {
        self.inner.estimate_cost(tokens)
    }

    async fn health_check(&self) -> Result<()> {
        match timeout(self.timeout_duration, self.inner.health_check()).await {
            Ok(result) => result,
            Err(_) => Err(IntelligenceError::Timeout {
                timeout_ms: self.timeout_duration.as_millis() as u64,
            }),
        }
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn is_available(&self) -> bool {
        match timeout(self.timeout_duration, self.inner.is_available()).await {
            Ok(available) => available,
            Err(_) => false, // Timeout means provider is not responsive
        }
    }
}

/// Helper function to wrap a provider with default 5-second timeout
pub fn with_timeout(provider: Arc<dyn LlmProvider>) -> TimeoutWrapper {
    TimeoutWrapper::new(provider)
}

/// Helper function to wrap a provider with custom timeout
pub fn with_custom_timeout(
    provider: Arc<dyn LlmProvider>,
    timeout_duration: Duration,
) -> TimeoutWrapper {
    TimeoutWrapper::with_timeout(provider, timeout_duration)
}

/// Batch timeout configuration for multiple operations
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    pub completion_timeout: Duration,
    pub embedding_timeout: Duration,
    pub health_check_timeout: Duration,
}

impl TimeoutConfig {
    /// Create a new timeout configuration with reasonable defaults
    pub fn new() -> Self {
        Self {
            completion_timeout: Duration::from_secs(5),
            embedding_timeout: Duration::from_secs(3),
            health_check_timeout: Duration::from_secs(2),
        }
    }

    /// Create a strict timeout configuration with shorter timeouts
    pub fn strict() -> Self {
        Self {
            completion_timeout: Duration::from_secs(3),
            embedding_timeout: Duration::from_secs(2),
            health_check_timeout: Duration::from_secs(1),
        }
    }

    /// Create a relaxed timeout configuration with longer timeouts
    pub fn relaxed() -> Self {
        Self {
            completion_timeout: Duration::from_secs(10),
            embedding_timeout: Duration::from_secs(5),
            health_check_timeout: Duration::from_secs(3),
        }
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Advanced timeout wrapper with operation-specific timeouts
pub struct AdvancedTimeoutWrapper {
    inner: Arc<dyn LlmProvider>,
    config: TimeoutConfig,
}

impl AdvancedTimeoutWrapper {
    /// Create a new advanced timeout wrapper
    pub fn new(provider: Arc<dyn LlmProvider>, config: TimeoutConfig) -> Self {
        Self {
            inner: provider,
            config,
        }
    }

    /// Create with default timeouts
    pub fn with_defaults(provider: Arc<dyn LlmProvider>) -> Self {
        Self::new(provider, TimeoutConfig::default())
    }

    /// Update the timeout configuration
    pub fn set_config(&mut self, config: TimeoutConfig) {
        self.config = config;
    }

    /// Get the current timeout configuration
    pub fn config(&self) -> &TimeoutConfig {
        &self.config
    }
}

#[async_trait]
impl LlmProvider for AdvancedTimeoutWrapper {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        match timeout(self.config.completion_timeout, self.inner.complete(request)).await {
            Ok(result) => result,
            Err(_) => Err(IntelligenceError::Timeout {
                timeout_ms: self.config.completion_timeout.as_millis() as u64,
            }),
        }
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        match timeout(self.config.embedding_timeout, self.inner.embed(text)).await {
            Ok(result) => result,
            Err(_) => Err(IntelligenceError::Timeout {
                timeout_ms: self.config.embedding_timeout.as_millis() as u64,
            }),
        }
    }

    fn capabilities(&self) -> LlmCapabilities {
        self.inner.capabilities()
    }

    fn estimate_cost(&self, tokens: usize) -> Cost {
        self.inner.estimate_cost(tokens)
    }

    async fn health_check(&self) -> Result<()> {
        match timeout(self.config.health_check_timeout, self.inner.health_check()).await {
            Ok(result) => result,
            Err(_) => Err(IntelligenceError::Timeout {
                timeout_ms: self.config.health_check_timeout.as_millis() as u64,
            }),
        }
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn is_available(&self) -> bool {
        match timeout(self.config.health_check_timeout, self.inner.is_available()).await {
            Ok(available) => available,
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_provider::MockLlmProvider;
    use crate::provider::Message;

    #[tokio::test]
    async fn test_timeout_wrapper_success() {
        let mock_provider = Arc::new(MockLlmProvider::new());
        let timeout_provider = TimeoutWrapper::new(mock_provider);

        let request = CompletionRequest::new(
            "mock-gpt-3.5",
            vec![Message::user("Hello")],
        );

        let result = timeout_provider.complete(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_timeout_wrapper_timeout() {
        let mock_provider = Arc::new(MockLlmProvider::new().with_delay(6000)); // 6 seconds
        let timeout_provider = TimeoutWrapper::new(mock_provider);

        let request = CompletionRequest::new(
            "mock-gpt-3.5",
            vec![Message::user("Hello")],
        );

        let result = timeout_provider.complete(request).await;
        assert!(matches!(result, Err(IntelligenceError::Timeout { .. })));
    }

    #[tokio::test]
    async fn test_custom_timeout() {
        let mock_provider = Arc::new(MockLlmProvider::new().with_delay(2000)); // 2 seconds
        let timeout_provider = TimeoutWrapper::with_timeout(
            mock_provider,
            Duration::from_millis(1000), // 1 second timeout
        );

        let request = CompletionRequest::new(
            "mock-gpt-3.5",
            vec![Message::user("Hello")],
        );

        let result = timeout_provider.complete(request).await;
        assert!(matches!(result, Err(IntelligenceError::Timeout { .. })));
    }

    #[tokio::test]
    async fn test_advanced_timeout_wrapper() {
        let mock_provider = Arc::new(MockLlmProvider::new());
        let config = TimeoutConfig::strict();
        let timeout_provider = AdvancedTimeoutWrapper::new(mock_provider, config);

        let request = CompletionRequest::new(
            "mock-gpt-3.5",
            vec![Message::user("Hello")],
        );

        let result = timeout_provider.complete(request).await;
        assert!(result.is_ok());

        let embedding_result = timeout_provider.embed("test").await;
        assert!(embedding_result.is_ok());
    }

    #[tokio::test]
    async fn test_health_check_timeout() {
        let mock_provider = Arc::new(MockLlmProvider::new().with_delay(3000));
        let timeout_provider = TimeoutWrapper::with_timeout(
            mock_provider,
            Duration::from_millis(1000),
        );

        let result = timeout_provider.health_check().await;
        assert!(matches!(result, Err(IntelligenceError::Timeout { .. })));
    }

    #[tokio::test]
    async fn test_is_available_timeout() {
        let mock_provider = Arc::new(MockLlmProvider::new().with_delay(3000));
        let timeout_provider = TimeoutWrapper::with_timeout(
            mock_provider,
            Duration::from_millis(1000),
        );

        let available = timeout_provider.is_available().await;
        assert!(!available); // Should be false due to timeout
    }
}