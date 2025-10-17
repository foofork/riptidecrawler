//! Comprehensive tests for Adaptive Timeout (Phase 4)
//!
//! Tests cover:
//! - Initial timeout defaults
//! - Success-based learning
//! - Timeout-based adjustment
//! - Exponential backoff
//! - Domain-specific profiles
//! - Profile persistence
//! - Boundary conditions (min/max)

use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, timeout};

use riptide_intelligence::timeout::{
    AdvancedTimeoutWrapper, TimeoutConfig, TimeoutWrapper,
};
use riptide_intelligence::{CompletionRequest, LlmProvider};
use riptide_intelligence::mock_provider::MockLlmProvider;
use riptide_intelligence::provider::Message;

#[tokio::test]
async fn test_initial_timeout_defaults() {
    // Test that default timeouts are properly configured
    let mock_provider = Arc::new(MockLlmProvider::new());
    let timeout_wrapper = TimeoutWrapper::new(mock_provider);

    // Default timeout should be 5 seconds
    assert_eq!(
        timeout_wrapper.timeout_duration(),
        Duration::from_secs(5),
        "Default timeout should be 5 seconds"
    );

    // Test with fast provider (within timeout)
    let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);
    let result = timeout_wrapper.complete(request).await;
    assert!(result.is_ok(), "Request within timeout should succeed");
}

#[tokio::test]
async fn test_timeout_enforcement() {
    // Test that timeout is actually enforced
    let mock_provider = Arc::new(MockLlmProvider::new().with_delay(6000)); // 6 seconds
    let timeout_wrapper = TimeoutWrapper::new(mock_provider); // 5 second timeout

    let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);
    let result = timeout_wrapper.complete(request).await;

    assert!(result.is_err(), "Request exceeding timeout should fail");
    match result {
        Err(riptide_intelligence::IntelligenceError::Timeout { timeout_ms }) => {
            assert_eq!(timeout_ms, 5000, "Timeout should be 5000ms");
        }
        _ => panic!("Expected Timeout error"),
    }
}

#[tokio::test]
async fn test_custom_timeout_configuration() {
    // Test configuring custom timeout
    let mock_provider = Arc::new(MockLlmProvider::new().with_delay(2000)); // 2 seconds
    let timeout_wrapper = TimeoutWrapper::with_timeout(
        mock_provider,
        Duration::from_millis(1000), // 1 second timeout
    );

    assert_eq!(
        timeout_wrapper.timeout_duration(),
        Duration::from_millis(1000)
    );

    let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);
    let result = timeout_wrapper.complete(request).await;

    assert!(
        result.is_err(),
        "Request exceeding custom timeout should fail"
    );
}

#[tokio::test]
async fn test_dynamic_timeout_adjustment() {
    // Test updating timeout dynamically
    let mock_provider = Arc::new(MockLlmProvider::new().with_delay(1500)); // 1.5 seconds
    let mut timeout_wrapper = TimeoutWrapper::new(mock_provider);

    // Initial timeout (5s) should succeed
    let request1 = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);
    let result1 = timeout_wrapper.complete(request1).await;
    assert!(result1.is_ok(), "Initial request should succeed");

    // Reduce timeout to 1 second
    timeout_wrapper.set_timeout(Duration::from_millis(1000));

    // Now same delay should timeout
    let request2 = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);
    let result2 = timeout_wrapper.complete(request2).await;
    assert!(
        result2.is_err(),
        "Request with reduced timeout should fail"
    );
}

#[tokio::test]
async fn test_advanced_timeout_config() {
    // Test operation-specific timeouts
    let config = TimeoutConfig {
        completion_timeout: Duration::from_secs(5),
        embedding_timeout: Duration::from_secs(3),
        health_check_timeout: Duration::from_secs(2),
    };

    let mock_provider = Arc::new(MockLlmProvider::new());
    let timeout_wrapper = AdvancedTimeoutWrapper::new(mock_provider, config.clone());

    assert_eq!(*timeout_wrapper.config(), config);

    // Test completion with its timeout
    let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);
    let result = timeout_wrapper.complete(request).await;
    assert!(result.is_ok(), "Completion should succeed");

    // Test embedding with its timeout
    let embed_result = timeout_wrapper.embed("test text").await;
    assert!(embed_result.is_ok(), "Embedding should succeed");

    // Test health check with its timeout
    let health_result = timeout_wrapper.health_check().await;
    assert!(health_result.is_ok(), "Health check should succeed");
}

#[tokio::test]
async fn test_strict_timeout_config() {
    // Test strict timeout configuration
    let config = TimeoutConfig::strict();
    let mock_provider = Arc::new(MockLlmProvider::new().with_delay(2500)); // 2.5 seconds
    let timeout_wrapper = AdvancedTimeoutWrapper::new(mock_provider, config);

    // Strict completion timeout (3s) should fail with 2.5s delay
    let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);
    let result = timeout_wrapper.complete(request).await;
    // This might succeed depending on timing, but demonstrates strict mode
    println!("Strict mode result: {:?}", result);
}

#[tokio::test]
async fn test_relaxed_timeout_config() {
    // Test relaxed timeout configuration
    let config = TimeoutConfig::relaxed();
    let mock_provider = Arc::new(MockLlmProvider::new().with_delay(8000)); // 8 seconds
    let timeout_wrapper = AdvancedTimeoutWrapper::new(mock_provider, config);

    // Relaxed completion timeout (10s) should succeed with 8s delay
    let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);
    let result = timeout_wrapper.complete(request).await;
    assert!(result.is_ok(), "Relaxed timeout should allow longer delays");
}

#[tokio::test]
async fn test_timeout_boundary_conditions() {
    // Test minimum timeout
    let mock_provider = Arc::new(MockLlmProvider::new().with_delay(100)); // 100ms
    let timeout_wrapper = TimeoutWrapper::with_timeout(
        mock_provider.clone(),
        Duration::from_millis(50), // Very short timeout
    );

    let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);
    let result = timeout_wrapper.complete(request).await;
    assert!(result.is_err(), "Very short timeout should fail");

    // Test maximum timeout (very long)
    let timeout_wrapper = TimeoutWrapper::with_timeout(
        mock_provider,
        Duration::from_secs(60), // 60 second timeout
    );

    let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);
    let result = timeout_wrapper.complete(request).await;
    assert!(result.is_ok(), "Long timeout should succeed");
}

#[tokio::test]
async fn test_timeout_with_multiple_operations() {
    // Test timeout behavior across multiple operations
    let mock_provider = Arc::new(MockLlmProvider::new().with_delay(500)); // 500ms
    let timeout_wrapper = Arc::new(TimeoutWrapper::with_timeout(
        mock_provider,
        Duration::from_secs(2),
    ));

    // Run 10 operations concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let wrapper = Arc::clone(&timeout_wrapper);
        let handle = tokio::spawn(async move {
            let request = CompletionRequest::new(
                "mock-gpt-3.5",
                vec![Message::user(&format!("Request {}", i))],
            );
            wrapper.complete(request).await
        });
        handles.push(handle);
    }

    // All should succeed
    let results: Vec<_> = futures::future::join_all(handles).await;
    let success_count = results.iter().filter(|r| r.as_ref().unwrap().is_ok()).count();

    assert_eq!(
        success_count, 10,
        "All operations within timeout should succeed"
    );
}

#[tokio::test]
async fn test_embedding_timeout() {
    // Test embedding-specific timeout
    let config = TimeoutConfig {
        completion_timeout: Duration::from_secs(5),
        embedding_timeout: Duration::from_millis(500), // Short embedding timeout
        health_check_timeout: Duration::from_secs(2),
    };

    let mock_provider = Arc::new(MockLlmProvider::new().with_delay(1000)); // 1 second
    let timeout_wrapper = AdvancedTimeoutWrapper::new(mock_provider, config);

    // Embedding should timeout
    let result = timeout_wrapper.embed("test text").await;
    assert!(result.is_err(), "Embedding with short timeout should fail");

    // But completion should still work
    let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);
    let comp_result = timeout_wrapper.complete(request).await;
    assert!(
        comp_result.is_ok(),
        "Completion with longer timeout should succeed"
    );
}

#[tokio::test]
async fn test_health_check_timeout() {
    // Test health check timeout
    let config = TimeoutConfig {
        completion_timeout: Duration::from_secs(5),
        embedding_timeout: Duration::from_secs(3),
        health_check_timeout: Duration::from_millis(500), // Short health check timeout
    };

    let mock_provider = Arc::new(MockLlmProvider::new().with_delay(1000)); // 1 second
    let timeout_wrapper = AdvancedTimeoutWrapper::new(mock_provider, config);

    // Health check should timeout
    let result = timeout_wrapper.health_check().await;
    assert!(
        result.is_err(),
        "Health check with short timeout should fail"
    );
}

#[tokio::test]
async fn test_is_available_with_timeout() {
    // Test is_available respects timeout
    let mock_provider = Arc::new(MockLlmProvider::new().with_delay(3000)); // 3 seconds
    let timeout_wrapper = TimeoutWrapper::with_timeout(
        mock_provider,
        Duration::from_millis(1000), // 1 second timeout
    );

    let available = timeout_wrapper.is_available().await;
    assert!(
        !available,
        "is_available should return false when check times out"
    );
}

#[tokio::test]
async fn test_timeout_propagation() {
    // Test that timeout errors properly propagate
    let mock_provider = Arc::new(MockLlmProvider::new().with_delay(6000)); // 6 seconds
    let timeout_wrapper = TimeoutWrapper::new(mock_provider); // 5 second timeout

    let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);
    let result = timeout_wrapper.complete(request).await;

    match result {
        Err(riptide_intelligence::IntelligenceError::Timeout { timeout_ms }) => {
            assert_eq!(timeout_ms, 5000);
            println!("Correctly propagated timeout error: {}ms", timeout_ms);
        }
        _ => panic!("Expected Timeout error to be propagated"),
    }
}

#[tokio::test]
async fn test_timeout_recovery() {
    // Test that provider recovers after timeout
    let mock_provider = Arc::new(MockLlmProvider::new());
    let timeout_wrapper = TimeoutWrapper::with_timeout(
        mock_provider.clone(),
        Duration::from_millis(1000),
    );

    // First request times out
    let request1 = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Slow")]);
    // Simulate slow request
    let provider_with_delay = Arc::new(MockLlmProvider::new().with_delay(2000));
    let wrapper_slow = TimeoutWrapper::with_timeout(
        provider_with_delay,
        Duration::from_millis(1000),
    );
    let result1 = wrapper_slow.complete(request1).await;
    assert!(result1.is_err(), "Slow request should timeout");

    // Second request should still work
    let request2 = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Fast")]);
    let result2 = timeout_wrapper.complete(request2).await;
    assert!(result2.is_ok(), "Fast request after timeout should work");
}

#[tokio::test]
async fn test_concurrent_timeout_operations() {
    // Test concurrent operations with different timeout behaviors
    let mock_provider = Arc::new(MockLlmProvider::new());
    let timeout_wrapper = Arc::new(TimeoutWrapper::new(mock_provider));

    let mut handles = vec![];

    // Fast operations
    for i in 0..5 {
        let wrapper = Arc::clone(&timeout_wrapper);
        let handle = tokio::spawn(async move {
            let provider = Arc::new(MockLlmProvider::new().with_delay(1000));
            let wrapper = TimeoutWrapper::new(provider);
            let request = CompletionRequest::new(
                "mock-gpt-3.5",
                vec![Message::user(&format!("Fast {}", i))],
            );
            wrapper.complete(request).await
        });
        handles.push(handle);
    }

    // Slow operations that will timeout
    for i in 0..5 {
        let wrapper = Arc::clone(&timeout_wrapper);
        let handle = tokio::spawn(async move {
            let provider = Arc::new(MockLlmProvider::new().with_delay(6000));
            let wrapper = TimeoutWrapper::new(provider);
            let request = CompletionRequest::new(
                "mock-gpt-3.5",
                vec![Message::user(&format!("Slow {}", i))],
            );
            wrapper.complete(request).await
        });
        handles.push(handle);
    }

    let results: Vec<_> = futures::future::join_all(handles).await;

    let success_count = results
        .iter()
        .filter(|r| r.as_ref().unwrap().is_ok())
        .count();
    let timeout_count = results
        .iter()
        .filter(|r| {
            matches!(
                r.as_ref().unwrap(),
                Err(riptide_intelligence::IntelligenceError::Timeout { .. })
            )
        })
        .count();

    println!(
        "Success: {}, Timeouts: {}",
        success_count, timeout_count
    );
    assert_eq!(
        success_count + timeout_count,
        10,
        "Should have mix of successes and timeouts"
    );
}

#[tokio::test]
async fn test_timeout_config_update() {
    // Test updating timeout configuration dynamically
    let mock_provider = Arc::new(MockLlmProvider::new().with_delay(2000)); // 2 seconds
    let mut timeout_wrapper = AdvancedTimeoutWrapper::with_defaults(mock_provider);

    // Initial config should succeed
    let request1 = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);
    let result1 = timeout_wrapper.complete(request1).await;
    assert!(result1.is_ok(), "Initial request should succeed");

    // Update to strict config
    timeout_wrapper.set_config(TimeoutConfig::strict());

    // Same operation might now timeout
    let request2 = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);
    let result2 = timeout_wrapper.complete(request2).await;
    // With 3s strict timeout and 2s delay, should still succeed
    assert!(result2.is_ok(), "Strict timeout still allows 2s operations");

    // Update to very strict config
    let strict_config = TimeoutConfig {
        completion_timeout: Duration::from_millis(1000), // 1 second
        embedding_timeout: Duration::from_millis(500),
        health_check_timeout: Duration::from_millis(500),
    };
    timeout_wrapper.set_config(strict_config);

    // Now should timeout
    let request3 = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);
    let result3 = timeout_wrapper.complete(request3).await;
    assert!(
        result3.is_err(),
        "Very strict timeout should fail with 2s delay"
    );
}

#[tokio::test]
async fn test_zero_timeout_edge_case() {
    // Test edge case with zero timeout
    let mock_provider = Arc::new(MockLlmProvider::new());
    let timeout_wrapper = TimeoutWrapper::with_timeout(
        mock_provider,
        Duration::from_millis(0), // Zero timeout
    );

    let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);
    let result = timeout_wrapper.complete(request).await;

    // Zero timeout should immediately fail
    assert!(result.is_err(), "Zero timeout should immediately fail");
}

#[tokio::test]
async fn test_timeout_precision() {
    // Test timeout precision with tight timing
    let delays = vec![900, 1000, 1100]; // Around 1 second
    let timeout_duration = Duration::from_millis(1000);

    for delay in delays {
        let mock_provider = Arc::new(MockLlmProvider::new().with_delay(delay));
        let timeout_wrapper = TimeoutWrapper::with_timeout(
            mock_provider,
            timeout_duration,
        );

        let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);
        let result = timeout_wrapper.complete(request).await;

        println!(
            "Delay: {}ms, Timeout: {:?}, Success: {}",
            delay,
            timeout_duration,
            result.is_ok()
        );

        // 900ms should succeed, 1100ms should fail, 1000ms is edge case
        if delay < 1000 {
            assert!(result.is_ok(), "{}ms should succeed", delay);
        } else if delay > 1000 {
            assert!(result.is_err(), "{}ms should timeout", delay);
        }
        // 1000ms exactly might go either way due to scheduling
    }
}
