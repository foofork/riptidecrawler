//! Comprehensive unit tests for riptide-intelligence crate

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

use riptide_intelligence::*;
use riptide_intelligence::mock_provider::*;

/// Test module for LlmProvider trait implementations
mod provider_tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_basic_completion() {
        let provider = MockLlmProvider::new();
        let request = CompletionRequest::new(
            "mock-gpt-3.5",
            vec![Message::user("Hello, world!")]
        );

        let response = provider.complete(request).await.unwrap();
        assert!(response.content.contains("Mock response"));
        assert_eq!(response.usage.total_tokens, response.usage.prompt_tokens + response.usage.completion_tokens);
        assert_eq!(provider.request_count(), 1);
    }

    #[tokio::test]
    async fn test_mock_provider_embeddings() {
        let provider = MockLlmProvider::new();
        let text = "This is a test sentence for embedding generation.";
        
        let embedding = provider.embed(text).await.unwrap();
        assert_eq!(embedding.len(), 768); // Standard embedding size
        
        // Test that same text produces same embedding
        let embedding2 = provider.embed(text).await.unwrap();
        assert_eq!(embedding, embedding2);
        
        // Test that different text produces different embedding
        let embedding3 = provider.embed("Different text").await.unwrap();
        assert_ne!(embedding, embedding3);
    }

    #[tokio::test]
    async fn test_mock_provider_capabilities() {
        let provider = MockLlmProvider::new();
        let capabilities = provider.capabilities();
        
        assert_eq!(capabilities.provider_name, "mock");
        assert_eq!(capabilities.models.len(), 2);
        assert!(capabilities.supports_embeddings);
        assert!(capabilities.supports_functions);
        assert_eq!(capabilities.max_context_length, 8192);
        
        // Check model details
        let gpt35_model = capabilities.models.iter()
            .find(|m| m.id == "mock-gpt-3.5")
            .unwrap();
        assert_eq!(gpt35_model.max_tokens, 4096);
        assert_eq!(gpt35_model.cost_per_1k_prompt_tokens, 0.001);
    }

    #[tokio::test]
    async fn test_mock_provider_health_check() {
        let provider = MockLlmProvider::new();
        assert!(provider.health_check().await.is_ok());
        assert!(provider.is_available().await);
        
        let failing_provider = MockLlmProvider::new().always_fail();
        assert!(failing_provider.health_check().await.is_err());
        assert!(!failing_provider.is_available().await);
    }

    #[tokio::test]
    async fn test_mock_provider_cost_estimation() {
        let provider = MockLlmProvider::new();
        let cost = provider.estimate_cost(1000);
        
        assert_eq!(cost.prompt_cost, 0.001);
        assert_eq!(cost.completion_cost, 0.002);
        assert_eq!(cost.total_cost, 0.003);
        assert_eq!(cost.currency, "USD");
    }

    #[tokio::test]
    async fn test_mock_provider_failure_modes() {
        // Test always fail mode
        let always_fail = MockLlmProvider::new().always_fail();
        let request = CompletionRequest::new("mock", vec![Message::user("test")]);
        assert!(always_fail.complete(request).await.is_err());
        
        // Test fail after N requests
        let fail_after = MockLlmProvider::new().fail_after(2);
        let request = CompletionRequest::new("mock", vec![Message::user("test")]);
        
        // First two should succeed
        assert!(fail_after.complete(request.clone()).await.is_ok());
        assert!(fail_after.complete(request.clone()).await.is_ok());
        
        // Third should fail
        assert!(fail_after.complete(request).await.is_err());
        assert_eq!(fail_after.request_count(), 3);
    }

    #[tokio::test]
    async fn test_mock_provider_delay() {
        let start = std::time::Instant::now();
        let delayed_provider = MockLlmProvider::new().with_delay(100);
        let request = CompletionRequest::new("mock", vec![Message::user("test")]);
        
        let _response = delayed_provider.complete(request).await.unwrap();
        let elapsed = start.elapsed();
        
        assert!(elapsed >= Duration::from_millis(100));
    }
}

/// Test module for TimeoutWrapper functionality
mod timeout_tests {
    use super::*;
    use riptide_intelligence::timeout::*;

    #[tokio::test]
    async fn test_timeout_wrapper_success() {
        let mock_provider = Arc::new(MockLlmProvider::new());
        let timeout_wrapper = TimeoutWrapper::new(mock_provider);
        
        let request = CompletionRequest::new("mock", vec![Message::user("test")]);
        let result = timeout_wrapper.complete(request).await;
        
        assert!(result.is_ok());
        assert_eq!(timeout_wrapper.timeout_duration(), Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_timeout_wrapper_timeout() {
        let mock_provider = Arc::new(MockLlmProvider::new().with_delay(2000));
        let timeout_wrapper = TimeoutWrapper::with_timeout(mock_provider, Duration::from_millis(100));
        
        let request = CompletionRequest::new("mock", vec![Message::user("test")]);
        let result = timeout_wrapper.complete(request).await;
        
        assert!(matches!(result, Err(IntelligenceError::Timeout { .. })));
    }

    #[tokio::test]
    async fn test_timeout_wrapper_embedding_timeout() {
        let mock_provider = Arc::new(MockLlmProvider::new().with_delay(2000));
        let timeout_wrapper = TimeoutWrapper::with_timeout(mock_provider, Duration::from_millis(100));
        
        let result = timeout_wrapper.embed("test").await;
        assert!(matches!(result, Err(IntelligenceError::Timeout { .. })));
    }

    #[tokio::test]
    async fn test_timeout_configuration() {
        let mock_provider = Arc::new(MockLlmProvider::new());
        let mut timeout_wrapper = TimeoutWrapper::new(mock_provider);
        
        // Test default timeout
        assert_eq!(timeout_wrapper.timeout_duration(), Duration::from_secs(5));
        
        // Test custom timeout
        timeout_wrapper.set_timeout(Duration::from_secs(10));
        assert_eq!(timeout_wrapper.timeout_duration(), Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_timeout_health_check() {
        let mock_provider = Arc::new(MockLlmProvider::new().with_delay(2000));
        let timeout_wrapper = TimeoutWrapper::with_timeout(mock_provider, Duration::from_millis(100));
        
        let result = timeout_wrapper.health_check().await;
        assert!(matches!(result, Err(IntelligenceError::Timeout { .. })));
        
        let is_available = timeout_wrapper.is_available().await;
        assert!(!is_available);
    }
}

/// Test module for CircuitBreaker functionality  
mod circuit_breaker_tests {
    use super::*;
    use riptide_intelligence::circuit_breaker::*;

    #[tokio::test]
    async fn test_circuit_breaker_consecutive_failures() {
        let mock_provider = Arc::new(MockLlmProvider::new().fail_after(0)); // Always fail
        let config = CircuitBreakerConfig {
            consecutive_failure_threshold: 3,
            ..Default::default()
        };
        let circuit_breaker = CircuitBreaker::with_config(mock_provider, config);
        
        // Circuit should start closed
        assert_eq!(circuit_breaker.state(), CircuitState::Closed);
        
        // Make failing requests
        for i in 0..3 {
            let request = CompletionRequest::new("mock", vec![Message::user("test")]);
            let result = circuit_breaker.complete(request).await;
            assert!(result.is_err());
            
            // Check if circuit opened after threshold
            if i >= 2 {
                assert_eq!(circuit_breaker.state(), CircuitState::Open);
            }
        }
        
        // Next request should fail due to open circuit
        let request = CompletionRequest::new("mock", vec![Message::user("test")]);
        let result = circuit_breaker.complete(request).await;
        assert!(matches!(result, Err(IntelligenceError::CircuitOpen { .. })));
    }

    #[tokio::test] 
    async fn test_circuit_breaker_stats() {
        let mock_provider = Arc::new(MockLlmProvider::new());
        let circuit_breaker = CircuitBreaker::new(mock_provider);
        
        let stats = circuit_breaker.stats();
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.consecutive_failures, 0);
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.error_rate, 0.0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_recovery() {
        let mock_provider = Arc::new(MockLlmProvider::new());
        let config = CircuitBreakerConfig {
            consecutive_failure_threshold: 2,
            open_duration: Duration::from_millis(100),
            ..Default::default()
        };
        let circuit_breaker = CircuitBreaker::with_config(mock_provider, config);
        
        // Force failures to open circuit
        mock_provider.always_fail();
        for _ in 0..2 {
            let request = CompletionRequest::new("mock", vec![Message::user("test")]);
            let _ = circuit_breaker.complete(request).await;
        }
        
        assert_eq!(circuit_breaker.state(), CircuitState::Open);
        
        // Wait for circuit to try half-open
        sleep(Duration::from_millis(150)).await;
        
        // This test would need more complex mock provider state management
        // to fully test half-open -> closed transition
    }

    #[test]
    fn test_circuit_breaker_config() {
        let config = CircuitBreakerConfig::default();
        assert_eq!(config.error_rate_threshold, 0.2);
        assert_eq!(config.p95_latency_threshold_ms, 4000);
        assert_eq!(config.consecutive_failure_threshold, 5);
        assert_eq!(config.minimum_requests, 10);
    }
}

/// Test module for LlmRegistry functionality
mod registry_tests {
    use super::*;
    use riptide_intelligence::registry::*;

    #[test]
    fn test_registry_creation() {
        let registry = LlmRegistry::new();
        assert_eq!(registry.list_providers().len(), 0);
        assert!(!registry.has_provider("test"));
    }

    #[test]
    fn test_provider_registration() {
        let registry = LlmRegistry::new();
        let provider = Arc::new(MockLlmProvider::new());
        
        registry.register_provider("test", provider).unwrap();
        assert!(registry.has_provider("test"));
        assert_eq!(registry.list_providers().len(), 1);
        
        let retrieved = registry.get_provider("test");
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_factory_registration_and_loading() {
        let registry = LlmRegistry::new();
        
        // Register factory
        registry.register_factory("mock", |config| {
            Ok(Arc::new(MockLlmProvider::with_name(config.name.clone())) as Arc<dyn LlmProvider>)
        }).unwrap();
        
        // Load provider using factory
        let config = ProviderConfig::new("test", "mock");
        registry.load_provider(config).unwrap();
        
        assert!(registry.has_provider("test"));
        assert_eq!(registry.list_providers().len(), 1);
    }

    #[test]
    fn test_provider_config() {
        let config = ProviderConfig::new("test", "mock")
            .with_config("key", serde_json::Value::String("value".to_string()))
            .with_fallback_order(1)
            .disabled();
        
        assert_eq!(config.name, "test");
        assert_eq!(config.provider_type, "mock");
        assert!(!config.enabled);
        assert_eq!(config.fallback_order, Some(1));
        assert!(config.config.contains_key("key"));
    }

    #[test]
    fn test_multiple_provider_loading() {
        let registry = LlmRegistry::new();
        
        registry.register_factory("mock", |config| {
            Ok(Arc::new(MockLlmProvider::with_name(config.name.clone())) as Arc<dyn LlmProvider>)
        }).unwrap();
        
        let configs = vec![
            ProviderConfig::new("provider1", "mock").with_fallback_order(1),
            ProviderConfig::new("provider2", "mock").with_fallback_order(2),
            ProviderConfig::new("provider3", "mock").disabled(),
        ];
        
        registry.load_providers(configs).unwrap();
        
        assert_eq!(registry.list_providers().len(), 2); // provider3 is disabled
        
        let fallback_providers = registry.get_fallback_providers();
        assert_eq!(fallback_providers.len(), 2);
        assert_eq!(fallback_providers[0].0, "provider1");
        assert_eq!(fallback_providers[1].0, "provider2");
    }

    #[test]
    fn test_registry_stats() {
        let registry = LlmRegistry::new();
        
        registry.register_factory("mock", |config| {
            Ok(Arc::new(MockLlmProvider::with_name(config.name.clone())) as Arc<dyn LlmProvider>)
        }).unwrap();
        
        let configs = vec![
            ProviderConfig::new("provider1", "mock"),
            ProviderConfig::new("provider2", "mock").disabled(),
        ];
        
        registry.load_providers(configs).unwrap();
        
        let stats = registry.stats();
        assert_eq!(stats.total_providers, 1); // Only enabled provider
        assert_eq!(stats.enabled_providers, 1);
        assert_eq!(stats.provider_types, 1);
        assert_eq!(stats.registered_factories, 1);
    }

    #[tokio::test]
    async fn test_registry_health_check() {
        let registry = LlmRegistry::new();
        
        let healthy_provider = Arc::new(MockLlmProvider::new());
        let unhealthy_provider = Arc::new(MockLlmProvider::new().always_fail());
        
        registry.register_provider("healthy", healthy_provider).unwrap();
        registry.register_provider("unhealthy", unhealthy_provider).unwrap();
        
        let health_results = registry.health_check().await;
        assert_eq!(health_results.len(), 2);
        assert_eq!(health_results["healthy"], true);
        assert_eq!(health_results["unhealthy"], false);
    }

    #[test]
    fn test_provider_removal() {
        let registry = LlmRegistry::new();
        let provider = Arc::new(MockLlmProvider::new());
        
        registry.register_provider("test", provider).unwrap();
        assert!(registry.has_provider("test"));
        
        let removed = registry.remove_provider("test");
        assert!(removed.is_some());
        assert!(!registry.has_provider("test"));
        assert_eq!(registry.list_providers().len(), 0);
    }
}

/// Test module for FallbackChain functionality
mod fallback_tests {
    use super::*;
    use riptide_intelligence::fallback::*;

    #[tokio::test]
    async fn test_fallback_chain_success_on_first() {
        let providers = vec![
            Arc::new(MockLlmProvider::with_name("provider1")) as Arc<dyn LlmProvider>,
            Arc::new(MockLlmProvider::with_name("provider2")) as Arc<dyn LlmProvider>,
        ];
        
        let fallback_chain = FallbackChain::new(providers, FallbackStrategy::FirstAvailable);
        
        let request = CompletionRequest::new("mock", vec![Message::user("test")]);
        let result = fallback_chain.complete(request).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fallback_chain_failure_recovery() {
        let providers = vec![
            Arc::new(MockLlmProvider::with_name("provider1").always_fail()) as Arc<dyn LlmProvider>,
            Arc::new(MockLlmProvider::with_name("provider2")) as Arc<dyn LlmProvider>,
            Arc::new(MockLlmProvider::with_name("provider3").always_fail()) as Arc<dyn LlmProvider>,
        ];
        
        let fallback_chain = FallbackChain::new(providers, FallbackStrategy::FirstAvailable);
        
        let request = CompletionRequest::new("mock", vec![Message::user("test")]);
        let result = fallback_chain.complete(request).await;
        
        // Should succeed using provider2
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fallback_chain_all_fail() {
        let providers = vec![
            Arc::new(MockLlmProvider::with_name("provider1").always_fail()) as Arc<dyn LlmProvider>,
            Arc::new(MockLlmProvider::with_name("provider2").always_fail()) as Arc<dyn LlmProvider>,
        ];
        
        let fallback_chain = FallbackChain::new(providers, FallbackStrategy::FirstAvailable);
        
        let request = CompletionRequest::new("mock", vec![Message::user("test")]);
        let result = fallback_chain.complete(request).await;
        
        assert!(matches!(result, Err(IntelligenceError::AllProvidersFailed)));
    }
}

/// Test module for IntelligenceClient integration
mod client_tests {
    use super::*;

    #[tokio::test]
    async fn test_intelligence_client_basic() {
        let registry = LlmRegistry::new();
        let provider = Arc::new(MockLlmProvider::new());
        
        registry.register_provider("default", provider).unwrap();
        
        let client = IntelligenceClient::new(registry, "default");
        
        let request = CompletionRequest::new("mock", vec![Message::user("test")]);
        let result = client.complete(request).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_intelligence_client_missing_provider() {
        let registry = LlmRegistry::new();
        let client = IntelligenceClient::new(registry, "nonexistent");
        
        let request = CompletionRequest::new("mock", vec![Message::user("test")]);
        let result = client.complete(request).await;
        
        assert!(matches!(result, Err(IntelligenceError::Configuration(_))));
    }

    #[tokio::test]
    async fn test_intelligence_client_specific_provider() {
        let registry = LlmRegistry::new();
        
        let provider1 = Arc::new(MockLlmProvider::with_name("provider1"));
        let provider2 = Arc::new(MockLlmProvider::with_name("provider2"));
        
        registry.register_provider("provider1", provider1).unwrap();
        registry.register_provider("provider2", provider2).unwrap();
        
        let client = IntelligenceClient::new(registry, "provider1");
        
        let request = CompletionRequest::new("mock", vec![Message::user("test")]);
        let result = client.complete_with_provider("provider2", request).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_intelligence_client_embeddings() {
        let registry = LlmRegistry::new();
        let provider = Arc::new(MockLlmProvider::new());
        
        registry.register_provider("default", provider).unwrap();
        
        let client = IntelligenceClient::new(registry, "default");
        
        let result = client.embed("test text").await;
        assert!(result.is_ok());
        
        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 768);
    }

    #[test]
    fn test_intelligence_client_capabilities() {
        let registry = LlmRegistry::new();
        let provider = Arc::new(MockLlmProvider::new());
        
        registry.register_provider("default", provider).unwrap();
        
        let client = IntelligenceClient::new(registry, "default");
        
        let capabilities = client.capabilities("default");
        assert!(capabilities.is_some());
        
        let caps = capabilities.unwrap();
        assert_eq!(caps.provider_name, "mock");
        assert!(caps.supports_embeddings);
    }

    #[test]
    fn test_intelligence_client_cost_estimation() {
        let registry = LlmRegistry::new();
        let provider = Arc::new(MockLlmProvider::new());
        
        registry.register_provider("default", provider).unwrap();
        
        let client = IntelligenceClient::new(registry, "default");
        
        let cost = client.estimate_cost("default", 1000);
        assert!(cost.is_some());
        
        let cost = cost.unwrap();
        assert_eq!(cost.total_cost, 0.003);
        assert_eq!(cost.currency, "USD");
    }
}

/// Edge case and error handling tests
mod edge_case_tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_message_handling() {
        let provider = MockLlmProvider::new();
        let request = CompletionRequest::new("mock", vec![]);
        
        let result = provider.complete(request).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(response.content.contains("Hello! This is a mock response."));
    }

    #[tokio::test]
    async fn test_large_message_handling() {
        let provider = MockLlmProvider::new();
        let large_content = "x".repeat(10000);
        let request = CompletionRequest::new("mock", vec![Message::user(large_content)]);
        
        let result = provider.complete(request).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(response.usage.prompt_tokens > 0);
        assert!(response.usage.completion_tokens > 0);
    }

    #[tokio::test]
    async fn test_unicode_content_handling() {
        let provider = MockLlmProvider::new();
        let unicode_content = "Hello 世界! 🌍 Testing émojis and spëcial characters";
        let request = CompletionRequest::new("mock", vec![Message::user(unicode_content)]);
        
        let result = provider.complete(request).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_message_formatting() {
        let timeout_error = IntelligenceError::Timeout { timeout_ms: 5000 };
        assert_eq!(timeout_error.to_string(), "Timeout error: operation took longer than 5000ms");
        
        let circuit_error = IntelligenceError::CircuitOpen { 
            reason: "Too many failures".to_string() 
        };
        assert_eq!(circuit_error.to_string(), "Circuit breaker open: Too many failures");
        
        let rate_limit_error = IntelligenceError::RateLimit { retry_after_ms: 60000 };
        assert_eq!(rate_limit_error.to_string(), "Rate limit exceeded: 60000ms");
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        let provider = Arc::new(MockLlmProvider::new());
        let timeout_wrapper = Arc::new(TimeoutWrapper::new(provider));
        
        let mut handles = vec![];
        
        // Spawn 10 concurrent requests
        for i in 0..10 {
            let provider_clone = timeout_wrapper.clone();
            let handle = tokio::spawn(async move {
                let request = CompletionRequest::new("mock", vec![Message::user(format!("Request {}", i))]);
                provider_clone.complete(request).await
            });
            handles.push(handle);
        }
        
        // Wait for all requests to complete
        let results = futures::future::join_all(handles).await;
        
        // All requests should succeed
        for result in results {
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
        }
    }

    #[tokio::test]
    async fn test_request_metadata_preservation() {
        let provider = MockLlmProvider::new();
        let mut request = CompletionRequest::new("mock", vec![Message::user("test")]);
        
        // Add metadata
        request = request.with_metadata("test_key", serde_json::Value::String("test_value".to_string()));
        request = request.with_metadata("number", serde_json::Value::Number(serde_json::Number::from(42)));
        
        let response = provider.complete(request).await.unwrap();
        
        // Check metadata is preserved
        assert_eq!(response.metadata.len(), 2);
        assert_eq!(response.metadata["test_key"], serde_json::Value::String("test_value".to_string()));
        assert_eq!(response.metadata["number"], serde_json::Value::Number(serde_json::Number::from(42)));
    }
}
