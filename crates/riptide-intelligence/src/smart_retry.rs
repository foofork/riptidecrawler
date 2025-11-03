//! Smart retry logic for riptide-intelligence operations
//!
//! Provides intelligent retry strategies for LLM calls, extraction, and API requests
//! with integration to existing circuit breaker and LLM pool.

use async_trait::async_trait;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, info, warn};

use crate::{CircuitBreaker, CircuitState, IntelligenceError, Result};

/// Smart retry strategy selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SmartRetryStrategy {
    /// Exponential backoff (2^n * initial_delay)
    Exponential,
    /// Linear backoff (n * initial_delay)
    Linear,
    /// Fibonacci sequence backoff
    Fibonacci,
    /// Adaptive strategy based on error patterns
    Adaptive,
}

/// Configuration for retry behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay before first retry (milliseconds)
    pub initial_delay_ms: u64,
    /// Maximum delay between retries (milliseconds)
    pub max_delay_ms: u64,
    /// Jitter percentage (0.0 to 1.0, typically 0.25 for 25%)
    pub jitter: f32,
    /// Backoff multiplier for exponential strategy
    pub backoff_multiplier: f32,
}

impl RetryConfig {
    /// Create a new configuration with safe defaults
    pub fn new() -> Self {
        Self {
            max_attempts: 5,
            initial_delay_ms: 100,
            max_delay_ms: 30_000,
            jitter: 0.25,
            backoff_multiplier: 2.0,
        }
    }

    /// Create a fast retry configuration for low-latency operations
    pub fn fast() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 50,
            max_delay_ms: 5_000,
            jitter: 0.1,
            backoff_multiplier: 1.5,
        }
    }

    /// Create an aggressive retry configuration for critical operations
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 10,
            initial_delay_ms: 100,
            max_delay_ms: 60_000,
            jitter: 0.25,
            backoff_multiplier: 2.0,
        }
    }

    /// Validate configuration values
    pub fn validate(&self) -> std::result::Result<(), String> {
        if self.max_attempts == 0 {
            return Err("max_attempts must be greater than 0".to_string());
        }
        if self.initial_delay_ms == 0 {
            return Err("initial_delay_ms must be greater than 0".to_string());
        }
        if self.max_delay_ms < self.initial_delay_ms {
            return Err("max_delay_ms must be >= initial_delay_ms".to_string());
        }
        if !(0.0..=1.0).contains(&self.jitter) {
            return Err("jitter must be between 0.0 and 1.0".to_string());
        }
        if self.backoff_multiplier <= 0.0 {
            return Err("backoff_multiplier must be positive".to_string());
        }
        Ok(())
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Retry statistics for tracking success/failure patterns
#[derive(Debug, Clone, Default)]
pub struct RetryStats {
    pub total_attempts: u32,
    pub successful_retries: u32,
    pub failed_retries: u32,
    pub strategy_switches: u32,
    pub last_success_time: Option<Instant>,
    pub last_failure_time: Option<Instant>,
}

impl RetryStats {
    fn new() -> Self {
        Self::default()
    }

    pub fn success_rate(&self) -> f32 {
        if self.total_attempts == 0 {
            0.0
        } else {
            self.successful_retries as f32 / self.total_attempts as f32
        }
    }
}

/// Smart retry with automatic strategy selection and switching
pub struct SmartRetry {
    config: RetryConfig,
    default_strategy: SmartRetryStrategy,
    stats: Arc<parking_lot::RwLock<RetryStats>>,
}

impl SmartRetry {
    /// Create a new SmartRetry with default configuration
    pub fn new(strategy: SmartRetryStrategy) -> Self {
        Self::with_config(strategy, RetryConfig::default())
    }

    /// Create a new SmartRetry with custom configuration
    pub fn with_config(strategy: SmartRetryStrategy, config: RetryConfig) -> Self {
        Self {
            config,
            default_strategy: strategy,
            stats: Arc::new(parking_lot::RwLock::new(RetryStats::new())),
        }
    }

    /// Get retry statistics
    pub fn stats(&self) -> RetryStats {
        self.stats.read().clone()
    }

    /// Classify error type and select appropriate retry strategy
    pub fn classify_error(&self, error: &IntelligenceError) -> Option<SmartRetryStrategy> {
        match error {
            // Circuit breaker open - do NOT retry
            IntelligenceError::CircuitOpen { .. } => None,

            // Client errors (400s) - do NOT retry
            IntelligenceError::InvalidRequest(_) => None,

            // Rate limit - use adaptive strategy with delay hints
            IntelligenceError::RateLimit { .. } => Some(SmartRetryStrategy::Adaptive),

            // Server errors (500s) - use fibonacci
            IntelligenceError::Provider(msg) if msg.contains("500") || msg.contains("503") => {
                Some(SmartRetryStrategy::Fibonacci)
            }

            // Transient errors (network, timeout) - use exponential
            IntelligenceError::Network(_) | IntelligenceError::Timeout { .. } => {
                Some(SmartRetryStrategy::Exponential)
            }

            // Default to configured strategy for unknown errors
            _ => Some(self.default_strategy),
        }
    }

    /// Calculate delay for given attempt using specified strategy
    pub fn calculate_delay(
        &self,
        attempt: u32,
        strategy: SmartRetryStrategy,
        error: Option<&IntelligenceError>,
    ) -> Duration {
        let base_delay_ms = match strategy {
            SmartRetryStrategy::Exponential => {
                // 2^n * initial_delay
                let exp = self.config.backoff_multiplier.powi(attempt as i32);
                (self.config.initial_delay_ms as f32 * exp) as u64
            }
            SmartRetryStrategy::Linear => {
                // n * initial_delay
                self.config.initial_delay_ms * (attempt as u64 + 1)
            }
            SmartRetryStrategy::Fibonacci => {
                // fibonacci(n) * initial_delay
                let fib = Self::fibonacci(attempt);
                self.config.initial_delay_ms * fib
            }
            SmartRetryStrategy::Adaptive => {
                // Use rate limit hint if available, otherwise exponential
                if let Some(IntelligenceError::RateLimit { retry_after_ms }) = error {
                    *retry_after_ms
                } else {
                    // Adaptive defaults to exponential with history
                    let base = self.config.backoff_multiplier.powi(attempt as i32);
                    let stats = self.stats.read();
                    let success_rate = stats.success_rate();

                    // Adjust based on historical success rate
                    let adjustment = if success_rate > 0.7 {
                        0.8 // Faster retries if high success rate
                    } else if success_rate > 0.4 {
                        1.0 // Normal retries
                    } else {
                        1.5 // Slower retries if low success rate
                    };

                    (self.config.initial_delay_ms as f32 * base * adjustment) as u64
                }
            }
        };

        // Apply max delay cap
        let capped_delay = base_delay_ms.min(self.config.max_delay_ms);

        // Apply jitter (0-25% random variance)
        let jitter_range = (capped_delay as f32 * self.config.jitter) as u64;
        let jitter = if jitter_range > 0 {
            rand::thread_rng().gen_range(0..=jitter_range)
        } else {
            0
        };

        Duration::from_millis(capped_delay + jitter)
    }

    /// Calculate fibonacci number iteratively
    fn fibonacci(n: u32) -> u64 {
        match n {
            0 => 1,
            1 => 1,
            _ => {
                let mut a = 1u64;
                let mut b = 1u64;
                for _ in 2..=n {
                    let temp = a.saturating_add(b);
                    a = b;
                    b = temp;
                }
                b
            }
        }
    }

    /// Execute an async operation with smart retry logic
    pub async fn execute<F, T, Fut>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempt = 0u32;
        let mut current_strategy = self.default_strategy;
        let mut last_error: Option<IntelligenceError> = None;

        loop {
            // Execute the operation
            match operation().await {
                Ok(result) => {
                    // Success - update stats and return
                    let mut stats = self.stats.write();
                    stats.total_attempts = stats
                        .total_attempts
                        .saturating_add(attempt.saturating_add(1));
                    if attempt > 0 {
                        stats.successful_retries = stats.successful_retries.saturating_add(1);
                    }
                    stats.last_success_time = Some(Instant::now());

                    debug!(
                        "Operation succeeded after {} attempt(s) using {:?} strategy",
                        attempt.saturating_add(1),
                        current_strategy
                    );

                    return Ok(result);
                }
                Err(error) => {
                    attempt = attempt.saturating_add(1);
                    last_error = Some(error.clone());

                    // Check if we should retry this error
                    let should_retry = self.classify_error(&error);

                    if should_retry.is_none() {
                        // Non-retryable error (circuit open, client error)
                        debug!("Non-retryable error encountered: {:?}", error);
                        return Err(error);
                    }

                    // Check if we've exhausted retry attempts
                    if attempt >= self.config.max_attempts {
                        warn!(
                            "Max retry attempts ({}) exhausted, last error: {:?}",
                            self.config.max_attempts, error
                        );

                        let mut stats = self.stats.write();
                        stats.total_attempts = stats.total_attempts.saturating_add(attempt);
                        stats.failed_retries = stats.failed_retries.saturating_add(1);
                        stats.last_failure_time = Some(Instant::now());

                        return Err(error);
                    }

                    // Switch strategy if needed (adaptive behavior)
                    if let Some(new_strategy) = should_retry {
                        if new_strategy != current_strategy {
                            info!(
                                "Switching retry strategy from {:?} to {:?} based on error type",
                                current_strategy, new_strategy
                            );
                            current_strategy = new_strategy;

                            let mut stats = self.stats.write();
                            stats.strategy_switches = stats.strategy_switches.saturating_add(1);
                        }
                    }

                    // Calculate and apply delay
                    let delay = self.calculate_delay(
                        attempt.saturating_sub(1),
                        current_strategy,
                        Some(&error),
                    );

                    debug!(
                        "Retry attempt {}/{} after {:?} using {:?} strategy",
                        attempt, self.config.max_attempts, delay, current_strategy
                    );

                    sleep(delay).await;
                }
            }
        }
    }

    /// Execute with automatic strategy switching if current strategy fails
    pub async fn execute_with_fallback<F, T, Fut>(
        &self,
        mut operation: F,
        fallback_strategies: Vec<SmartRetryStrategy>,
    ) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // Try primary strategy first
        let result = self.execute(&mut operation).await;

        if result.is_ok() {
            return result;
        }

        // Try each fallback strategy
        for (idx, strategy) in fallback_strategies.iter().enumerate() {
            info!(
                "Trying fallback strategy {}/{}: {:?}",
                idx.saturating_add(1),
                fallback_strategies.len(),
                strategy
            );

            let fallback_retry = SmartRetry::with_config(*strategy, self.config.clone());
            let result = fallback_retry.execute(&mut operation).await;

            if result.is_ok() {
                let mut stats = self.stats.write();
                stats.strategy_switches = stats.strategy_switches.saturating_add(1);
                return result;
            }
        }

        // All strategies failed
        result
    }

    /// Execute with circuit breaker integration
    pub async fn execute_with_circuit_breaker<F, T, Fut>(
        &self,
        circuit_breaker: &CircuitBreaker,
        mut operation: F,
    ) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // Check circuit breaker state first
        match circuit_breaker.state() {
            CircuitState::Open => {
                return Err(IntelligenceError::CircuitOpen {
                    reason: "Circuit breaker is open, skipping retry".to_string(),
                });
            }
            _ => {
                // Circuit is closed or half-open, proceed with retry logic
                self.execute(operation).await
            }
        }
    }
}

/// Trait for retry-aware operations
#[async_trait]
pub trait Retryable {
    /// Execute operation with retry logic
    async fn with_retry<F, T, Fut>(&self, operation: F) -> Result<T>
    where
        F: FnMut() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send;
}

#[async_trait]
impl Retryable for SmartRetry {
    async fn with_retry<F, T, Fut>(&self, operation: F) -> Result<T>
    where
        F: FnMut() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send,
    {
        self.execute(operation).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.initial_delay_ms, 100);
        assert_eq!(config.max_delay_ms, 30_000);
        assert_eq!(config.jitter, 0.25);
        assert_eq!(config.backoff_multiplier, 2.0);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_retry_config_validation() {
        let mut config = RetryConfig::default();

        // Invalid max_attempts
        config.max_attempts = 0;
        assert!(config.validate().is_err());
        config.max_attempts = 5;

        // Invalid initial_delay_ms
        config.initial_delay_ms = 0;
        assert!(config.validate().is_err());
        config.initial_delay_ms = 100;

        // Invalid max_delay_ms
        config.max_delay_ms = 50;
        assert!(config.validate().is_err());
        config.max_delay_ms = 30_000;

        // Invalid jitter
        config.jitter = 1.5;
        assert!(config.validate().is_err());
        config.jitter = 0.25;

        // Invalid backoff_multiplier
        config.backoff_multiplier = -1.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_fibonacci_calculation() {
        assert_eq!(SmartRetry::fibonacci(0), 1);
        assert_eq!(SmartRetry::fibonacci(1), 1);
        assert_eq!(SmartRetry::fibonacci(2), 2);
        assert_eq!(SmartRetry::fibonacci(3), 3);
        assert_eq!(SmartRetry::fibonacci(4), 5);
        assert_eq!(SmartRetry::fibonacci(5), 8);
        assert_eq!(SmartRetry::fibonacci(6), 13);
    }

    #[test]
    fn test_error_classification() {
        let retry = SmartRetry::new(SmartRetryStrategy::Exponential);

        // Circuit breaker - no retry
        let error = IntelligenceError::CircuitOpen {
            reason: "test".to_string(),
        };
        assert_eq!(retry.classify_error(&error), None);

        // Invalid request - no retry
        let error = IntelligenceError::InvalidRequest("bad request".to_string());
        assert_eq!(retry.classify_error(&error), None);

        // Rate limit - adaptive
        let error = IntelligenceError::RateLimit {
            retry_after_ms: 1000,
        };
        assert_eq!(
            retry.classify_error(&error),
            Some(SmartRetryStrategy::Adaptive)
        );

        // Server error - fibonacci
        let error = IntelligenceError::Provider("500 Internal Server Error".to_string());
        assert_eq!(
            retry.classify_error(&error),
            Some(SmartRetryStrategy::Fibonacci)
        );

        // Network error - exponential
        let error = IntelligenceError::Network("connection failed".to_string());
        assert_eq!(
            retry.classify_error(&error),
            Some(SmartRetryStrategy::Exponential)
        );

        // Timeout - exponential
        let error = IntelligenceError::Timeout { timeout_ms: 5000 };
        assert_eq!(
            retry.classify_error(&error),
            Some(SmartRetryStrategy::Exponential)
        );
    }

    #[test]
    fn test_exponential_backoff_timing() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay_ms: 100,
            max_delay_ms: 10_000,
            jitter: 0.0, // No jitter for predictable testing
            backoff_multiplier: 2.0,
        };
        let retry = SmartRetry::with_config(SmartRetryStrategy::Exponential, config);

        // Attempt 0: 2^0 * 100 = 100ms
        let delay = retry.calculate_delay(0, SmartRetryStrategy::Exponential, None);
        assert_eq!(delay.as_millis(), 100);

        // Attempt 1: 2^1 * 100 = 200ms
        let delay = retry.calculate_delay(1, SmartRetryStrategy::Exponential, None);
        assert_eq!(delay.as_millis(), 200);

        // Attempt 2: 2^2 * 100 = 400ms
        let delay = retry.calculate_delay(2, SmartRetryStrategy::Exponential, None);
        assert_eq!(delay.as_millis(), 400);

        // Attempt 3: 2^3 * 100 = 800ms
        let delay = retry.calculate_delay(3, SmartRetryStrategy::Exponential, None);
        assert_eq!(delay.as_millis(), 800);
    }

    #[test]
    fn test_linear_backoff_timing() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay_ms: 100,
            max_delay_ms: 10_000,
            jitter: 0.0,
            backoff_multiplier: 2.0,
        };
        let retry = SmartRetry::with_config(SmartRetryStrategy::Linear, config);

        // Attempt 0: (0+1) * 100 = 100ms
        let delay = retry.calculate_delay(0, SmartRetryStrategy::Linear, None);
        assert_eq!(delay.as_millis(), 100);

        // Attempt 1: (1+1) * 100 = 200ms
        let delay = retry.calculate_delay(1, SmartRetryStrategy::Linear, None);
        assert_eq!(delay.as_millis(), 200);

        // Attempt 2: (2+1) * 100 = 300ms
        let delay = retry.calculate_delay(2, SmartRetryStrategy::Linear, None);
        assert_eq!(delay.as_millis(), 300);
    }

    #[test]
    fn test_fibonacci_backoff_timing() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay_ms: 100,
            max_delay_ms: 10_000,
            jitter: 0.0,
            backoff_multiplier: 2.0,
        };
        let retry = SmartRetry::with_config(SmartRetryStrategy::Fibonacci, config);

        // Attempt 0: fib(0) * 100 = 1 * 100 = 100ms
        let delay = retry.calculate_delay(0, SmartRetryStrategy::Fibonacci, None);
        assert_eq!(delay.as_millis(), 100);

        // Attempt 1: fib(1) * 100 = 1 * 100 = 100ms
        let delay = retry.calculate_delay(1, SmartRetryStrategy::Fibonacci, None);
        assert_eq!(delay.as_millis(), 100);

        // Attempt 2: fib(2) * 100 = 2 * 100 = 200ms
        let delay = retry.calculate_delay(2, SmartRetryStrategy::Fibonacci, None);
        assert_eq!(delay.as_millis(), 200);

        // Attempt 3: fib(3) * 100 = 3 * 100 = 300ms
        let delay = retry.calculate_delay(3, SmartRetryStrategy::Fibonacci, None);
        assert_eq!(delay.as_millis(), 300);

        // Attempt 4: fib(4) * 100 = 5 * 100 = 500ms
        let delay = retry.calculate_delay(4, SmartRetryStrategy::Fibonacci, None);
        assert_eq!(delay.as_millis(), 500);
    }

    #[test]
    fn test_max_delay_enforcement() {
        let config = RetryConfig {
            max_attempts: 10,
            initial_delay_ms: 100,
            max_delay_ms: 1_000,
            jitter: 0.0,
            backoff_multiplier: 2.0,
        };
        let retry = SmartRetry::with_config(SmartRetryStrategy::Exponential, config);

        // High attempt number should be capped at max_delay_ms
        let delay = retry.calculate_delay(10, SmartRetryStrategy::Exponential, None);
        assert_eq!(delay.as_millis(), 1_000);
    }

    #[test]
    fn test_jitter_variance() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay_ms: 100,
            max_delay_ms: 10_000,
            jitter: 0.25, // 25% jitter
            backoff_multiplier: 2.0,
        };
        let retry = SmartRetry::with_config(SmartRetryStrategy::Exponential, config);

        // Calculate multiple delays and ensure they vary within jitter range
        let mut delays = Vec::new();
        for _ in 0..10 {
            let delay = retry.calculate_delay(2, SmartRetryStrategy::Exponential, None);
            delays.push(delay.as_millis());
        }

        // Base delay for attempt 2: 400ms
        // Jitter range: 0-100ms (25% of 400)
        // Expected range: 400-500ms
        for &delay in &delays {
            assert!(delay >= 400 && delay <= 500);
        }

        // Ensure we got some variance (not all the same)
        let unique_delays: std::collections::HashSet<_> = delays.into_iter().collect();
        assert!(
            unique_delays.len() > 1,
            "Expected jitter to create variance"
        );
    }

    #[test]
    fn test_adaptive_strategy_with_rate_limit() {
        let config = RetryConfig {
            jitter: 0.0, // No jitter for predictable test
            ..RetryConfig::default()
        };
        let retry = SmartRetry::with_config(SmartRetryStrategy::Adaptive, config);

        // With rate limit hint
        let error = IntelligenceError::RateLimit {
            retry_after_ms: 5000,
        };
        let delay = retry.calculate_delay(0, SmartRetryStrategy::Adaptive, Some(&error));
        assert_eq!(delay.as_millis(), 5000);
    }

    #[tokio::test]
    async fn test_max_attempts_enforcement() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 1, // Very short for testing
            max_delay_ms: 10,
            jitter: 0.0,
            backoff_multiplier: 2.0,
        };
        let retry = SmartRetry::with_config(SmartRetryStrategy::Exponential, config);

        let call_count = Arc::new(AtomicU32::new(0));
        let counter_clone = call_count.clone();
        let result: Result<()> = retry
            .execute(|| {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err(IntelligenceError::Network("test error".to_string()))
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 3); // Initial attempt + 2 retries
    }

    #[tokio::test]
    async fn test_successful_retry() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay_ms: 1,
            max_delay_ms: 10,
            jitter: 0.0,
            backoff_multiplier: 2.0,
        };
        let retry = SmartRetry::with_config(SmartRetryStrategy::Exponential, config);

        let call_count = Arc::new(AtomicU32::new(0));
        let counter_clone = call_count.clone();
        let result = retry
            .execute(|| {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err(IntelligenceError::Network("test error".to_string()))
                    } else {
                        Ok(42)
                    }
                }
            })
            .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count.load(Ordering::SeqCst), 3);

        // Check stats
        let stats = retry.stats();
        assert_eq!(stats.successful_retries, 1);
        assert_eq!(stats.total_attempts, 3);
    }

    #[tokio::test]
    async fn test_non_retryable_error() {
        let config = RetryConfig::default();
        let retry = SmartRetry::with_config(SmartRetryStrategy::Exponential, config);

        let call_count = Arc::new(AtomicU32::new(0));
        let counter_clone = call_count.clone();
        let result: Result<()> = retry
            .execute(|| {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err(IntelligenceError::CircuitOpen {
                        reason: "test".to_string(),
                    })
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 1); // Should not retry
    }

    #[tokio::test]
    async fn test_strategy_switching() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay_ms: 1,
            max_delay_ms: 10,
            jitter: 0.0,
            backoff_multiplier: 2.0,
        };
        let retry = SmartRetry::with_config(SmartRetryStrategy::Exponential, config);

        let call_count = Arc::new(AtomicU32::new(0));
        let counter_clone = call_count.clone();
        let result = retry
            .execute(|| {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    match count {
                        0 => Err(IntelligenceError::Network("network".to_string())),
                        1 => Err(IntelligenceError::RateLimit { retry_after_ms: 1 }),
                        _ => Ok(42),
                    }
                }
            })
            .await;

        assert_eq!(result.unwrap(), 42);

        // Should have switched strategy from Exponential to Adaptive
        let stats = retry.stats();
        assert_eq!(stats.strategy_switches, 1);
    }
}
