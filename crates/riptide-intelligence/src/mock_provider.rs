//! Mock LLM provider for testing

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;
use tokio::time::sleep;

use crate::{
    CompletionRequest, CompletionResponse, Cost, IntelligenceError, LlmCapabilities, LlmProvider,
    ModelInfo, Result, Usage,
};

/// Mock LLM provider for testing purposes
pub struct MockLlmProvider {
    name: String,
    request_count: AtomicU32,
    fail_after: Option<u32>,
    delay_ms: Option<u64>,
    should_fail: bool,
    is_healthy: AtomicBool,
}

impl MockLlmProvider {
    /// Create a new mock provider
    pub fn new() -> Self {
        Self {
            name: "mock".to_string(),
            request_count: AtomicU32::new(0),
            fail_after: None,
            delay_ms: None,
            should_fail: false,
            is_healthy: AtomicBool::new(true),
        }
    }

    /// Create a mock provider with a custom name
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            request_count: AtomicU32::new(0),
            fail_after: None,
            delay_ms: None,
            should_fail: false,
            is_healthy: AtomicBool::new(true),
        }
    }

    /// Configure the provider to fail after N requests
    pub fn fail_after(mut self, count: u32) -> Self {
        self.fail_after = Some(count);
        self
    }

    /// Configure the provider to add a delay to all requests
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = Some(delay_ms);
        self
    }

    /// Configure the provider to always fail
    pub fn always_fail(mut self) -> Self {
        self.should_fail = true;
        self
    }

    /// Get the number of requests made to this provider
    pub fn request_count(&self) -> u32 {
        self.request_count.load(Ordering::SeqCst)
    }

    /// Reset the request counter
    pub fn reset_counter(&self) {
        self.request_count.store(0, Ordering::SeqCst);
    }

    /// Set the health status of this provider (for testing health monitoring)
    pub fn set_healthy(&self, healthy: bool) {
        self.is_healthy.store(healthy, Ordering::SeqCst);
    }

    /// Check if the provider is healthy
    pub fn is_healthy(&self) -> bool {
        self.is_healthy.load(Ordering::SeqCst)
    }

    /// Generate a mock response based on the request
    fn generate_mock_response(&self, request: &CompletionRequest) -> CompletionResponse {
        // Simple mock response generation
        let content = if request.messages.is_empty() {
            "Hello! This is a mock response.".to_string()
        } else {
            let last_message = &request.messages[request.messages.len() - 1];
            format!(
                "Mock response to: {}",
                last_message.content.chars().take(50).collect::<String>()
            )
        };

        let usage = Usage {
            prompt_tokens: request
                .messages
                .iter()
                .map(|m| m.content.len() as u32 / 4) // Rough token estimation
                .sum(),
            completion_tokens: content.len() as u32 / 4,
            total_tokens: 0,
        };

        let mut response = CompletionResponse::new(
            request.id,
            content,
            request.model.clone(),
            Usage {
                total_tokens: usage.prompt_tokens + usage.completion_tokens,
                ..usage
            },
        );

        // Copy metadata from request
        for (key, value) in &request.metadata {
            response.metadata.insert(key.clone(), value.clone());
        }

        response
    }
}

impl Default for MockLlmProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmProvider for MockLlmProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let count = self.request_count.fetch_add(1, Ordering::SeqCst);

        // Add delay if configured
        if let Some(delay) = self.delay_ms {
            sleep(Duration::from_millis(delay)).await;
        }

        // Check if we should fail
        if self.should_fail {
            return Err(IntelligenceError::Provider(
                "Mock provider configured to fail".to_string(),
            ));
        }

        // Check if we should fail after N requests
        if let Some(fail_after) = self.fail_after {
            if count >= fail_after {
                return Err(IntelligenceError::Provider(format!(
                    "Mock provider failing after {} requests",
                    fail_after
                )));
            }
        }

        Ok(self.generate_mock_response(&request))
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let count = self.request_count.fetch_add(1, Ordering::SeqCst);

        // Add delay if configured
        if let Some(delay) = self.delay_ms {
            sleep(Duration::from_millis(delay)).await;
        }

        // Check if we should fail
        if self.should_fail {
            return Err(IntelligenceError::Provider(
                "Mock provider configured to fail".to_string(),
            ));
        }

        // Check if we should fail after N requests
        if let Some(fail_after) = self.fail_after {
            if count >= fail_after {
                return Err(IntelligenceError::Provider(format!(
                    "Mock provider failing after {} requests",
                    fail_after
                )));
            }
        }

        // Generate a mock embedding based on text hash
        let hash = text.chars().map(|c| c as u32).sum::<u32>();
        let embedding_size = 768; // Common embedding size
        let mut embedding = Vec::with_capacity(embedding_size);

        for i in 0..embedding_size {
            let value = ((hash + i as u32) as f32 / u32::MAX as f32) * 2.0 - 1.0;
            embedding.push(value);
        }

        Ok(embedding)
    }

    fn capabilities(&self) -> LlmCapabilities {
        LlmCapabilities {
            provider_name: self.name.clone(),
            models: vec![
                ModelInfo {
                    id: "mock-gpt-3.5".to_string(),
                    name: "Mock GPT-3.5".to_string(),
                    description: "Mock implementation of GPT-3.5 for testing".to_string(),
                    max_tokens: 4096,
                    supports_functions: false,
                    supports_streaming: false,
                    cost_per_1k_prompt_tokens: 0.001,
                    cost_per_1k_completion_tokens: 0.002,
                },
                ModelInfo {
                    id: "mock-gpt-4".to_string(),
                    name: "Mock GPT-4".to_string(),
                    description: "Mock implementation of GPT-4 for testing".to_string(),
                    max_tokens: 8192,
                    supports_functions: true,
                    supports_streaming: true,
                    cost_per_1k_prompt_tokens: 0.03,
                    cost_per_1k_completion_tokens: 0.06,
                },
            ],
            supports_embeddings: true,
            supports_streaming: false,
            supports_functions: true,
            max_context_length: 8192,
            rate_limits: {
                let mut limits = HashMap::new();
                limits.insert("mock-gpt-3.5".to_string(), 60);
                limits.insert("mock-gpt-4".to_string(), 20);
                limits
            },
        }
    }

    fn estimate_cost(&self, tokens: usize) -> Cost {
        // Use mock-gpt-3.5 pricing for estimation
        let prompt_cost = (tokens as f64 / 1000.0) * 0.001;
        let completion_cost = (tokens as f64 / 1000.0) * 0.002;
        Cost::new(prompt_cost, completion_cost, "USD")
    }

    async fn health_check(&self) -> Result<()> {
        // Add delay if configured (same as in complete)
        if let Some(delay) = self.delay_ms {
            sleep(Duration::from_millis(delay)).await;
        }

        // Check configured health status first
        if !self.is_healthy.load(Ordering::SeqCst) {
            return Err(IntelligenceError::Provider(
                "Mock provider is unhealthy".to_string(),
            ));
        }

        if self.should_fail {
            Err(IntelligenceError::Provider(
                "Mock provider is configured to fail".to_string(),
            ))
        } else {
            Ok(())
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Create multiple mock providers for testing fallback chains
pub fn create_mock_provider_chain(count: usize) -> Vec<MockLlmProvider> {
    (0..count)
        .map(|i| MockLlmProvider::with_name(format!("mock-{}", i)))
        .collect()
}

/// Create a mock provider that simulates various failure modes
pub fn create_failing_provider(failure_mode: FailureMode) -> MockLlmProvider {
    match failure_mode {
        FailureMode::AlwaysFail => MockLlmProvider::new().always_fail(),
        FailureMode::FailAfter(count) => MockLlmProvider::new().fail_after(count),
        FailureMode::Slow(delay_ms) => MockLlmProvider::new().with_delay(delay_ms),
        FailureMode::SlowThenFail {
            delay_ms,
            fail_after,
        } => MockLlmProvider::new()
            .with_delay(delay_ms)
            .fail_after(fail_after),
    }
}

/// Different failure modes for testing
pub enum FailureMode {
    AlwaysFail,
    FailAfter(u32),
    Slow(u64),
    SlowThenFail { delay_ms: u64, fail_after: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::Message;

    #[tokio::test]
    async fn test_mock_provider_basic() {
        let provider = MockLlmProvider::new();
        let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello, world!")]);

        let response = provider.complete(request).await.unwrap();
        assert!(response.content.contains("Mock response"));
        assert_eq!(provider.request_count(), 1);
    }

    #[tokio::test]
    async fn test_mock_provider_failure() {
        let provider = MockLlmProvider::new().always_fail();
        let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello, world!")]);

        let result = provider.complete(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_provider_fail_after() {
        let provider = MockLlmProvider::new().fail_after(2);
        let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello, world!")]);

        // First two requests should succeed
        provider.complete(request.clone()).await.unwrap();
        provider.complete(request.clone()).await.unwrap();

        // Third request should fail
        let result = provider.complete(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_provider_embeddings() {
        let provider = MockLlmProvider::new();
        let embedding = provider.embed("test text").await.unwrap();
        assert_eq!(embedding.len(), 768);
    }

    #[tokio::test]
    async fn test_mock_provider_capabilities() {
        let provider = MockLlmProvider::new();
        let capabilities = provider.capabilities();
        assert_eq!(capabilities.provider_name, "mock");
        assert_eq!(capabilities.models.len(), 2);
        assert!(capabilities.supports_embeddings);
    }
}
