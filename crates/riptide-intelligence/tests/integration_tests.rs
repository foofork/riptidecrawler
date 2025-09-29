//! Integration tests for the RipTide Intelligence layer

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use riptide_intelligence::{
    IntelligenceClient, LlmRegistry, ProviderConfig, MockLlmProvider,
    TimeoutWrapper, CircuitBreaker, CircuitBreakerConfig, FallbackChain,
    CompletionRequest, Message, IntelligenceError, LlmProvider,
    create_fallback_chain, with_timeout, with_circuit_breaker,
};

/// Test basic provider registration and usage
#[tokio::test]
async fn test_basic_provider_registration() {
    let registry = LlmRegistry::new();

    // Register mock provider factory
    registry.register_factory("mock", |_config| {
        Ok(Arc::new(MockLlmProvider::new()) as Arc<dyn LlmProvider>)
    }).unwrap();

    // Load provider from config
    let config = ProviderConfig::new("test-provider", "mock");
    registry.load_provider(config).unwrap();

    // Create client
    let client = IntelligenceClient::new(registry, "test-provider");

    // Test completion
    let request = CompletionRequest::new(
        "mock-gpt-3.5",
        vec![Message::user("Hello, world!")],
    );

    let response = client.complete(request).await.unwrap();
    assert!(response.content.contains("Mock response"));
}

/// Test timeout functionality
#[tokio::test]
async fn test_timeout_functionality() {
    let slow_provider = Arc::new(MockLlmProvider::new().with_delay(6000)); // 6 seconds
    let timeout_provider = with_timeout(slow_provider);

    let request = CompletionRequest::new(
        "mock-gpt-3.5",
        vec![Message::user("This should timeout")],
    );

    let result = timeout_provider.complete(request).await;
    assert!(matches!(result, Err(IntelligenceError::Timeout { .. })));
}

/// Test circuit breaker functionality
#[tokio::test]
async fn test_circuit_breaker_functionality() {
    let failing_provider = Arc::new(MockLlmProvider::new().fail_after(0)); // Always fail
    let config = CircuitBreakerConfig::strict();
    let circuit_provider = with_circuit_breaker(failing_provider);

    let request = CompletionRequest::new(
        "mock-gpt-3.5",
        vec![Message::user("This will fail")],
    );

    // Make enough requests to trigger circuit opening
    for i in 0..10 {
        let result = circuit_provider.complete(request.clone()).await;
        println!("Request {}: {:?}", i + 1, result.is_err());

        // After a few failures, circuit should open and reject requests immediately
        if i > 5 {
            if let Err(IntelligenceError::CircuitOpen { .. }) = result {
                // Circuit is open, test passed
                return;
            }
        }
    }

    // Verify circuit is open
    assert!(matches!(
        circuit_provider.complete(request).await,
        Err(IntelligenceError::CircuitOpen { .. })
    ));
}

/// Test fallback chain functionality
#[tokio::test]
async fn test_fallback_chain_functionality() {
    let failing_provider = Arc::new(MockLlmProvider::with_name("failing").fail_after(0));
    let working_provider = Arc::new(MockLlmProvider::with_name("working"));

    let chain = create_fallback_chain(vec![failing_provider, working_provider]);

    let request = CompletionRequest::new(
        "mock-gpt-3.5",
        vec![Message::user("Test fallback")],
    );

    let response = chain.complete(request).await.unwrap();
    assert!(response.content.contains("Mock response"));

    let stats = chain.stats();
    assert_eq!(stats.fallback_triggers, 1);
    assert_eq!(stats.successful_requests, 1);
}

/// Test complete safety stack (timeout + circuit breaker + fallback)
#[tokio::test]
async fn test_complete_safety_stack() {
    // Create providers with different failure modes
    let slow_provider = Arc::new(MockLlmProvider::with_name("slow").with_delay(3000));
    let failing_provider = Arc::new(MockLlmProvider::with_name("failing").fail_after(0));
    let working_provider = Arc::new(MockLlmProvider::with_name("working"));

    // Wrap providers with safety features
    let timeout_slow = TimeoutWrapper::with_timeout(slow_provider, Duration::from_millis(1000));
    let circuit_failing = CircuitBreaker::with_config(failing_provider, CircuitBreakerConfig::strict());
    let timeout_working = TimeoutWrapper::new(working_provider);

    // Create fallback chain
    let mut chain = FallbackChain::new();
    chain.add_provider_simple(Arc::new(timeout_slow));
    chain.add_provider_simple(Arc::new(circuit_failing));
    chain.add_provider_simple(Arc::new(timeout_working));

    let request = CompletionRequest::new(
        "mock-gpt-3.5",
        vec![Message::user("Test complete safety stack")],
    );

    // Should eventually succeed with the working provider
    let response = chain.complete(request).await.unwrap();
    assert!(response.content.contains("Mock response"));

    let stats = chain.stats();
    assert!(stats.fallback_triggers > 0);
    assert_eq!(stats.successful_requests, 1);
}

/// Test registry with multiple providers
#[tokio::test]
async fn test_registry_multiple_providers() {
    let registry = LlmRegistry::new();

    // Register multiple providers
    registry.register_provider(
        "provider1",
        Arc::new(MockLlmProvider::with_name("provider1")),
    ).unwrap();

    registry.register_provider(
        "provider2",
        Arc::new(MockLlmProvider::with_name("provider2")),
    ).unwrap();

    // Test client with different providers
    let client = IntelligenceClient::new(registry, "provider1");

    let request = CompletionRequest::new(
        "mock-gpt-3.5",
        vec![Message::user("Hello")],
    );

    // Test default provider
    let response1 = client.complete(request.clone()).await.unwrap();
    assert!(response1.content.contains("Mock response"));

    // Test specific provider
    let response2 = client.complete_with_provider("provider2", request).await.unwrap();
    assert!(response2.content.contains("Mock response"));

    // Test capabilities
    let caps1 = client.capabilities("provider1").unwrap();
    assert_eq!(caps1.provider_name, "provider1");

    let caps2 = client.capabilities("provider2").unwrap();
    assert_eq!(caps2.provider_name, "provider2");
}

/// Test embeddings functionality
#[tokio::test]
async fn test_embeddings_functionality() {
    let provider = Arc::new(MockLlmProvider::new());
    let embedding = provider.embed("test text").await.unwrap();

    assert_eq!(embedding.len(), 768);
    assert!(embedding.iter().all(|&x| (-1.0..=1.0).contains(&x)));
}

/// Test cost estimation
#[tokio::test]
async fn test_cost_estimation() {
    let provider = Arc::new(MockLlmProvider::new());
    let cost = provider.estimate_cost(1000);

    assert!(cost.total_cost > 0.0);
    assert_eq!(cost.currency, "USD");
    assert_eq!(cost.total_cost, cost.prompt_cost + cost.completion_cost);
}

/// Test provider availability checking
#[tokio::test]
async fn test_provider_availability() {
    let working_provider = Arc::new(MockLlmProvider::new());
    let failing_provider = Arc::new(MockLlmProvider::new().always_fail());

    assert!(working_provider.is_available().await);
    assert!(!failing_provider.is_available().await);

    // Test with timeout
    let slow_provider = Arc::new(MockLlmProvider::new().with_delay(3000));
    let timeout_provider = TimeoutWrapper::with_timeout(slow_provider, Duration::from_millis(500));

    assert!(!timeout_provider.is_available().await);
}

/// Test configuration-driven provider loading
#[tokio::test]
async fn test_config_driven_loading() {
    let registry = LlmRegistry::new();

    // Register factory
    registry.register_factory("mock", |config| {
        let mut provider = MockLlmProvider::with_name(&config.name);

        // Configure based on config values
        if let Some(delay) = config.config.get("delay") {
            if let Some(delay_ms) = delay.as_u64() {
                provider = provider.with_delay(delay_ms);
            }
        }

        Ok(Arc::new(provider) as Arc<dyn riptide_intelligence::LlmProvider>)
    }).unwrap();

    // Load providers with different configurations
    let configs = vec![
        ProviderConfig::new("fast-provider", "mock"),
        ProviderConfig::new("slow-provider", "mock")
            .with_config("delay", serde_json::Value::Number(serde_json::Number::from(2000u64))),
    ];

    registry.load_providers(configs).unwrap();

    // Test both providers
    let client = IntelligenceClient::new(registry, "fast-provider");

    let request = CompletionRequest::new(
        "mock-gpt-3.5",
        vec![Message::user("Hello")],
    );

    // Fast provider should respond quickly
    let start = std::time::Instant::now();
    let _response = client.complete(request.clone()).await.unwrap();
    let fast_duration = start.elapsed();

    // Slow provider should take longer
    let start = std::time::Instant::now();
    let _response = client.complete_with_provider("slow-provider", request).await.unwrap();
    let slow_duration = start.elapsed();

    assert!(slow_duration > fast_duration);
    assert!(slow_duration >= Duration::from_millis(2000));
}

/// Test error handling and propagation
#[tokio::test]
async fn test_error_handling() {
    let registry = LlmRegistry::new();
    let client = IntelligenceClient::new(registry, "nonexistent");

    let request = CompletionRequest::new(
        "mock-gpt-3.5",
        vec![Message::user("Hello")],
    );

    // Should get configuration error for nonexistent provider
    let result = client.complete(request).await;
    assert!(matches!(result, Err(IntelligenceError::Configuration(_))));

    // Test invalid request handling
    let provider = Arc::new(MockLlmProvider::new());
    let empty_request = CompletionRequest::new("mock-gpt-3.5", vec![]);
    let result = provider.complete(empty_request).await;
    // Mock provider should handle empty requests gracefully
    assert!(result.is_ok());
}

/// Test concurrent requests and thread safety
#[tokio::test]
async fn test_concurrent_requests() {
    let provider = Arc::new(MockLlmProvider::new());
    let provider_clone = provider.clone();

    let mut handles = Vec::new();

    // Spawn multiple concurrent requests
    for i in 0..10 {
        let provider = provider_clone.clone();
        let handle = tokio::spawn(async move {
            let request = CompletionRequest::new(
                "mock-gpt-3.5",
                vec![Message::user(format!("Request {}", i))],
            );
            provider.complete(request).await
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // All requests should succeed
    for result in results {
        let response = result.unwrap().unwrap();
        assert!(response.content.contains("Mock response"));
    }

    // Provider should have seen all requests
    assert_eq!(provider.request_count(), 10);
}

/// Test circuit breaker repair attempts limit
#[tokio::test]
async fn test_circuit_breaker_repair_limit() {
    let failing_provider = Arc::new(MockLlmProvider::new().always_fail());
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        min_request_threshold: 2,
        recovery_timeout_secs: 1,
        max_repair_attempts: 1,
        ..CircuitBreakerConfig::strict()
    };
    let circuit_provider = CircuitBreaker::with_config(failing_provider, config);

    let request = CompletionRequest::new(
        "mock-gpt-3.5",
        vec![Message::user("This will fail")],
    );

    // Trigger circuit opening
    for _ in 0..5 {
        let _ = circuit_provider.complete(request.clone()).await;
    }

    // Wait for recovery timeout
    sleep(Duration::from_secs(2)).await;

    // Should allow one repair attempt
    let result = circuit_provider.complete(request.clone()).await;
    assert!(result.is_err());

    // Verify repair attempts
    let stats = circuit_provider.stats();
    assert_eq!(stats.repair_attempts, 1);

    // Wait again - should not allow more repair attempts
    sleep(Duration::from_secs(2)).await;

    let result = circuit_provider.complete(request).await;
    assert!(matches!(result, Err(IntelligenceError::CircuitOpen { .. })));
}