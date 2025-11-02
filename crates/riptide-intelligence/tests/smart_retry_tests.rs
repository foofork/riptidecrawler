//! Integration tests for smart retry logic

use riptide_intelligence::smart_retry::{RetryConfig, SmartRetry, SmartRetryStrategy};
use riptide_intelligence::{CircuitBreaker, CircuitBreakerConfig, IntelligenceError};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[tokio::test]
async fn test_exponential_backoff_real_timing() {
    let config = RetryConfig {
        max_attempts: 4,
        initial_delay_ms: 50,
        max_delay_ms: 10_000,
        jitter: 0.0,
        backoff_multiplier: 2.0,
    };
    let retry = SmartRetry::with_config(SmartRetryStrategy::Exponential, config);

    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();
    let start = Instant::now();

    let result = retry
        .execute(|| {
            let counter = counter_clone.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                if count < 3 {
                    Err(IntelligenceError::Network("test".to_string()))
                } else {
                    Ok(())
                }
            }
        })
        .await;

    let elapsed = start.elapsed();

    assert!(result.is_ok());
    assert_eq!(counter.load(Ordering::SeqCst), 4);

    // Total expected delay: 50ms + 100ms + 200ms = 350ms
    // Allow some tolerance for timing variations
    assert!(elapsed.as_millis() >= 300 && elapsed.as_millis() <= 500);
}

#[tokio::test]
async fn test_linear_backoff_real_timing() {
    let config = RetryConfig {
        max_attempts: 4,
        initial_delay_ms: 50,
        max_delay_ms: 10_000,
        jitter: 0.0,
        backoff_multiplier: 2.0,
    };
    let retry = SmartRetry::with_config(SmartRetryStrategy::Linear, config);

    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();
    let start = Instant::now();

    let result = retry
        .execute(|| {
            let counter = counter_clone.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                if count < 3 {
                    Err(IntelligenceError::Network("test".to_string()))
                } else {
                    Ok(())
                }
            }
        })
        .await;

    let elapsed = start.elapsed();

    assert!(result.is_ok());
    // Total expected delay: 50ms + 100ms + 150ms = 300ms
    assert!(elapsed.as_millis() >= 250 && elapsed.as_millis() <= 450);
}

#[tokio::test]
async fn test_fibonacci_sequence_backoff() {
    let config = RetryConfig {
        max_attempts: 6,
        initial_delay_ms: 10,
        max_delay_ms: 10_000,
        jitter: 0.0,
        backoff_multiplier: 2.0,
    };
    let retry = SmartRetry::with_config(SmartRetryStrategy::Fibonacci, config);

    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();
    let start = Instant::now();

    let result = retry
        .execute(|| {
            let counter = counter_clone.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                if count < 5 {
                    Err(IntelligenceError::Network("test".to_string()))
                } else {
                    Ok(())
                }
            }
        })
        .await;

    let elapsed = start.elapsed();

    assert!(result.is_ok());
    // Fibonacci: 1, 1, 2, 3, 5
    // Total expected delay: 10ms + 10ms + 20ms + 30ms + 50ms = 120ms
    assert!(elapsed.as_millis() >= 100 && elapsed.as_millis() <= 200);
}

#[tokio::test]
async fn test_adaptive_strategy_learning() {
    let config = RetryConfig {
        max_attempts: 10,
        initial_delay_ms: 10,
        max_delay_ms: 1_000,
        jitter: 0.0,
        backoff_multiplier: 2.0,
    };
    let retry = SmartRetry::with_config(SmartRetryStrategy::Adaptive, config);

    // First attempt - should succeed quickly
    let result = retry
        .execute(|| async { Ok::<_, IntelligenceError>(42) })
        .await;

    assert!(result.is_ok());

    // Second attempt with failures - adaptive should adjust
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result = retry
        .execute(|| {
            let counter = counter_clone.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(IntelligenceError::Network("test".to_string()))
                } else {
                    Ok(100)
                }
            }
        })
        .await;

    assert_eq!(result.unwrap(), 100);

    // Check that stats were updated
    let stats = retry.stats();
    assert!(stats.total_attempts > 0);
    // Note: success_rate() is private, just verify attempts were tracked
    assert!(stats.successful_retries > 0);
}

#[tokio::test]
async fn test_timeout_handling() {
    let config = RetryConfig {
        max_attempts: 3,
        initial_delay_ms: 10,
        max_delay_ms: 100,
        jitter: 0.0,
        backoff_multiplier: 2.0,
    };
    let retry = SmartRetry::with_config(SmartRetryStrategy::Exponential, config);

    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result: Result<(), IntelligenceError> = retry
        .execute(|| {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err(IntelligenceError::Timeout { timeout_ms: 5000 })
            }
        })
        .await;

    assert!(result.is_err());
    assert_eq!(counter.load(Ordering::SeqCst), 3);

    // Verify timeout errors use exponential strategy
    let strategy = retry.classify_error(&IntelligenceError::Timeout { timeout_ms: 5000 });
    assert_eq!(strategy, Some(SmartRetryStrategy::Exponential));
}

#[tokio::test]
async fn test_success_rate_tracking() {
    let config = RetryConfig {
        max_attempts: 5,
        initial_delay_ms: 1,
        max_delay_ms: 100,
        jitter: 0.0,
        backoff_multiplier: 2.0,
    };
    let retry = SmartRetry::with_config(SmartRetryStrategy::Exponential, config);

    // Successful operations
    for _ in 0..3 {
        let _ = retry
            .execute(|| async { Ok::<_, IntelligenceError>(()) })
            .await;
    }

    // Failed operation
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();
    let _: Result<(), IntelligenceError> = retry
        .execute(|| {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err(IntelligenceError::Network("test".to_string()))
            }
        })
        .await;

    let stats = retry.stats();
    assert_eq!(stats.total_attempts, 8); // 3 successful + 5 failed attempts
    assert_eq!(stats.successful_retries, 0); // No retries succeeded (first 3 didn't need retry)
    assert_eq!(stats.failed_retries, 1);
}

#[tokio::test]
async fn test_concurrent_retries() {
    let config = RetryConfig {
        max_attempts: 3,
        initial_delay_ms: 10,
        max_delay_ms: 100,
        jitter: 0.1,
        backoff_multiplier: 2.0,
    };
    let retry = Arc::new(SmartRetry::with_config(
        SmartRetryStrategy::Exponential,
        config,
    ));

    // Spawn multiple concurrent retry operations
    let mut handles = vec![];
    for i in 0..5 {
        let retry_clone = retry.clone();
        let handle = tokio::spawn(async move {
            let counter = Arc::new(AtomicU32::new(0));
            let counter_clone = counter.clone();

            retry_clone
                .execute(|| {
                    let counter = counter_clone.clone();
                    async move {
                        let count = counter.fetch_add(1, Ordering::SeqCst);
                        if count < 2 {
                            Err(IntelligenceError::Network(format!("worker-{}", i)))
                        } else {
                            Ok(i)
                        }
                    }
                })
                .await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    // Check aggregate stats
    let stats = retry.stats();
    assert_eq!(stats.total_attempts, 15); // 5 workers * 3 attempts each
    assert_eq!(stats.successful_retries, 5); // All succeeded on retry
}

#[tokio::test]
async fn test_configuration_validation() {
    // Valid configuration
    let config = RetryConfig::default();
    assert!(config.validate().is_ok());

    // Invalid max_attempts
    let mut config = RetryConfig::default();
    config.max_attempts = 0;
    assert!(config.validate().is_err());

    // Invalid jitter
    let mut config = RetryConfig::default();
    config.jitter = 1.5;
    assert!(config.validate().is_err());

    // Invalid delay range
    let mut config = RetryConfig::default();
    config.max_delay_ms = 50;
    config.initial_delay_ms = 100;
    assert!(config.validate().is_err());
}

#[tokio::test]
async fn test_rate_limit_adaptive_delay() {
    let config = RetryConfig {
        max_attempts: 3,
        initial_delay_ms: 10,
        max_delay_ms: 10_000,
        jitter: 0.0,
        backoff_multiplier: 2.0,
    };
    let retry = SmartRetry::with_config(SmartRetryStrategy::Adaptive, config);

    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();
    let start = Instant::now();

    let result = retry
        .execute(|| {
            let counter = counter_clone.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                if count == 0 {
                    // First attempt returns rate limit with hint
                    Err(IntelligenceError::RateLimit {
                        retry_after_ms: 200,
                    })
                } else {
                    Ok(())
                }
            }
        })
        .await;

    let elapsed = start.elapsed();

    assert!(result.is_ok());
    // Should use the rate limit hint (200ms)
    assert!(elapsed.as_millis() >= 180 && elapsed.as_millis() <= 300);
}

#[cfg(feature = "mock")]
#[tokio::test]
async fn test_circuit_breaker_integration() {
    use riptide_intelligence::mock_provider::MockLlmProvider;
    use std::sync::Arc;

    let mock_provider = Arc::new(MockLlmProvider::new().fail_after(0));
    let cb_config = CircuitBreakerConfig::strict();
    let circuit_breaker = CircuitBreaker::with_config(mock_provider, cb_config);

    let config = RetryConfig {
        max_attempts: 3,
        initial_delay_ms: 1,
        max_delay_ms: 100,
        jitter: 0.0,
        backoff_multiplier: 2.0,
    };
    let retry = SmartRetry::with_config(SmartRetryStrategy::Exponential, config);

    // Force circuit open
    circuit_breaker.force_open();

    let result = retry
        .execute_with_circuit_breaker(&circuit_breaker, || async {
            Ok::<_, IntelligenceError>(())
        })
        .await;

    // Should immediately fail due to circuit breaker
    assert!(matches!(result, Err(IntelligenceError::CircuitOpen { .. })));
}

#[tokio::test]
async fn test_strategy_switching_on_error_type() {
    let config = RetryConfig {
        max_attempts: 5,
        initial_delay_ms: 1,
        max_delay_ms: 100,
        jitter: 0.0,
        backoff_multiplier: 2.0,
    };
    let retry = SmartRetry::with_config(SmartRetryStrategy::Exponential, config);

    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result = retry
        .execute(|| {
            let counter = counter_clone.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                match count {
                    0 => Err(IntelligenceError::Network("network".to_string())), // Exponential
                    1 => Err(IntelligenceError::Provider("500 error".to_string())), // Fibonacci
                    2 => Err(IntelligenceError::RateLimit { retry_after_ms: 1 }), // Adaptive
                    _ => Ok(42),
                }
            }
        })
        .await;

    assert_eq!(result.unwrap(), 42);

    // Should have switched strategies multiple times
    let stats = retry.stats();
    assert_eq!(stats.strategy_switches, 2); // Network->Server, Server->RateLimit
}

#[tokio::test]
async fn test_fallback_strategies() {
    let config = RetryConfig {
        max_attempts: 2,
        initial_delay_ms: 1,
        max_delay_ms: 100,
        jitter: 0.0,
        backoff_multiplier: 2.0,
    };
    let retry = SmartRetry::with_config(SmartRetryStrategy::Exponential, config);

    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let fallback_strategies = vec![
        SmartRetryStrategy::Linear,
        SmartRetryStrategy::Fibonacci,
        SmartRetryStrategy::Adaptive,
    ];

    let result = retry
        .execute_with_fallback(
            || {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    // Succeed on 7th attempt (2 initial + 2*3 fallback retries)
                    if count < 6 {
                        Err(IntelligenceError::Network("test".to_string()))
                    } else {
                        Ok(100)
                    }
                }
            },
            fallback_strategies,
        )
        .await;

    assert_eq!(result.unwrap(), 100);
}

#[tokio::test]
async fn test_jitter_creates_variance() {
    let config = RetryConfig {
        max_attempts: 5,
        initial_delay_ms: 100,
        max_delay_ms: 10_000,
        jitter: 0.25,
        backoff_multiplier: 2.0,
    };
    let retry = SmartRetry::with_config(SmartRetryStrategy::Exponential, config);

    let mut timings = Vec::new();

    for _ in 0..5 {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        let start = Instant::now();

        let _ = retry
            .execute(|| {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err(IntelligenceError::Network("test".to_string()))
                    } else {
                        Ok(())
                    }
                }
            })
            .await;

        timings.push(start.elapsed().as_millis());
    }

    // Check that we got variance in timings due to jitter
    let unique_timings: std::collections::HashSet<_> = timings.iter().collect();
    assert!(
        unique_timings.len() > 1,
        "Expected jitter to create timing variance"
    );
}
