//! Circuit breaker implementation for search providers.
//!
//! This module provides circuit breaker functionality to protect against
//! cascading failures when search providers become unavailable or unreliable.
//!
//! ## Relationship to Canonical Circuit Breaker
//!
//! This is a **specialized wrapper** for the `SearchProvider` trait. The canonical,
//! production-ready circuit breaker lives in `riptide-types::reliability::circuit`.
//!
//! **Why this wrapper exists:**
//! - Provides SearchProvider-specific integration
//! - Percentage-based failure thresholds (more intuitive for search APIs)
//! - Transparent wrapper pattern for any SearchProvider implementation
//! - Health check passthrough (independent of circuit state)
//!
//! **Future work:** Could be refactored to use the canonical circuit breaker internally
//! while maintaining the same public API. See `/docs/architecture/CIRCUIT_BREAKER_CONSOLIDATION_SUMMARY.md`

use super::{SearchBackend, SearchHit, SearchProvider};
use anyhow::Result;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Circuit breaker states following the standard pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, all requests fail fast
    Open,
    /// Circuit is half-open, allowing test requests
    HalfOpen,
}

/// Configuration for the circuit breaker behavior.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold percentage (0-100) to trigger circuit opening
    pub failure_threshold_percentage: u32,
    /// Minimum number of requests before circuit can open
    pub minimum_request_threshold: u32,
    /// Time to wait before attempting to close an open circuit
    pub recovery_timeout: Duration,
    /// Maximum number of test requests in half-open state
    pub half_open_max_requests: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold_percentage: 50,
            minimum_request_threshold: 5,
            recovery_timeout: Duration::from_secs(60),
            half_open_max_requests: 3,
        }
    }
}

/// Metrics tracked by the circuit breaker for decision making.
#[derive(Debug)]
struct CircuitMetrics {
    total_requests: AtomicU32,
    failed_requests: AtomicU32,
    last_failure_time: AtomicU64,
    half_open_requests: AtomicU32,
    half_open_failures: AtomicU32,
}

impl CircuitMetrics {
    fn new() -> Self {
        Self {
            total_requests: AtomicU32::new(0),
            failed_requests: AtomicU32::new(0),
            last_failure_time: AtomicU64::new(0),
            half_open_requests: AtomicU32::new(0),
            half_open_failures: AtomicU32::new(0),
        }
    }

    fn record_success(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    fn record_failure(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.failed_requests.fetch_add(1, Ordering::Relaxed);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.last_failure_time.store(now, Ordering::Relaxed);
    }

    fn record_half_open_success(&self) {
        self.half_open_requests.fetch_add(1, Ordering::Relaxed);
    }

    fn record_half_open_failure(&self) {
        self.half_open_requests.fetch_add(1, Ordering::Relaxed);
        self.half_open_failures.fetch_add(1, Ordering::Relaxed);
    }

    fn failure_rate_percentage(&self) -> u32 {
        let total = self.total_requests.load(Ordering::Relaxed);
        let failed = self.failed_requests.load(Ordering::Relaxed);

        if total == 0 {
            return 0;
        }

        (failed * 100) / total
    }

    fn should_trip(&self, config: &CircuitBreakerConfig) -> bool {
        let total = self.total_requests.load(Ordering::Relaxed);

        total >= config.minimum_request_threshold
            && self.failure_rate_percentage() >= config.failure_threshold_percentage
    }

    fn can_attempt_reset(&self, config: &CircuitBreakerConfig) -> bool {
        let last_failure = self.last_failure_time.load(Ordering::Relaxed);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        (now - last_failure) >= config.recovery_timeout.as_secs()
    }

    fn reset(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.failed_requests.store(0, Ordering::Relaxed);
        self.half_open_requests.store(0, Ordering::Relaxed);
        self.half_open_failures.store(0, Ordering::Relaxed);
    }
}

/// Circuit breaker wrapper that adds reliability to search providers.
///
/// This wrapper monitors search provider health and can fail fast when
/// providers become unreliable, preventing cascade failures and reducing
/// response times during provider outages.
///
/// ## Behavior
///
/// - **Closed**: Normal operation, requests pass through
/// - **Open**: All requests fail immediately with circuit breaker error
/// - **Half-Open**: Limited test requests to check provider recovery
///
/// ## Configuration
///
/// The circuit breaker can be tuned for different failure scenarios:
/// - API rate limiting (higher threshold, longer recovery)
/// - Network issues (lower threshold, shorter recovery)
/// - Service outages (balanced settings)
pub struct CircuitBreakerWrapper {
    provider: Box<dyn SearchProvider>,
    config: CircuitBreakerConfig,
    state: Arc<std::sync::RwLock<CircuitState>>,
    metrics: Arc<CircuitMetrics>,
}

impl CircuitBreakerWrapper {
    /// Create a new circuit breaker wrapper with default configuration.
    pub fn new(provider: Box<dyn SearchProvider>) -> Self {
        Self::with_config(provider, CircuitBreakerConfig::default())
    }

    /// Create a new circuit breaker wrapper with custom configuration.
    pub fn with_config(provider: Box<dyn SearchProvider>, config: CircuitBreakerConfig) -> Self {
        Self {
            provider,
            config,
            state: Arc::new(std::sync::RwLock::new(CircuitState::Closed)),
            metrics: Arc::new(CircuitMetrics::new()),
        }
    }

    /// Get the current circuit state for monitoring and debugging.
    pub fn current_state(&self) -> CircuitState {
        // If the lock is poisoned, we default to Open state for safety
        *self.state.read().unwrap_or_else(|e| {
            tracing::warn!("Circuit breaker state lock poisoned, defaulting to Open");
            e.into_inner()
        })
    }

    /// Get current failure rate percentage for monitoring.
    pub fn failure_rate(&self) -> u32 {
        self.metrics.failure_rate_percentage()
    }

    /// Manually reset the circuit breaker (for testing or manual recovery).
    pub fn reset(&self) {
        self.metrics.reset();
        // Handle poisoned lock by replacing it
        if let Ok(mut state) = self.state.write() {
            *state = CircuitState::Closed;
        } else {
            tracing::error!("Circuit breaker state lock poisoned during reset");
        }
    }

    /// Check if a request can proceed based on current circuit state.
    fn can_proceed(&self) -> Result<bool> {
        let current_state = self.current_state();

        match current_state {
            CircuitState::Closed => Ok(true),
            CircuitState::Open => {
                // Check if we can transition to half-open
                if self.metrics.can_attempt_reset(&self.config) {
                    if let Ok(mut state) = self.state.write() {
                        *state = CircuitState::HalfOpen;
                        Ok(true)
                    } else {
                        tracing::error!(
                            "Circuit breaker state lock poisoned, cannot transition to HalfOpen"
                        );
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests in half-open state
                let half_open_requests = self.metrics.half_open_requests.load(Ordering::Relaxed);
                Ok(half_open_requests < self.config.half_open_max_requests)
            }
        }
    }

    /// Handle the result of a request and update circuit state accordingly.
    fn handle_result<T>(&self, result: &Result<T>) {
        let current_state = self.current_state();

        match result {
            Ok(_) => {
                match current_state {
                    CircuitState::Closed => {
                        self.metrics.record_success();
                    }
                    CircuitState::HalfOpen => {
                        self.metrics.record_half_open_success();

                        // If we've had enough successful half-open requests, close the circuit
                        let half_open_requests =
                            self.metrics.half_open_requests.load(Ordering::Relaxed);
                        let half_open_failures =
                            self.metrics.half_open_failures.load(Ordering::Relaxed);

                        if half_open_requests >= self.config.half_open_max_requests
                            && half_open_failures == 0
                        {
                            if let Ok(mut state) = self.state.write() {
                                *state = CircuitState::Closed;
                                self.metrics.reset();
                            }
                        }
                    }
                    CircuitState::Open => {
                        // This shouldn't happen, but if it does, we can close the circuit
                        if let Ok(mut state) = self.state.write() {
                            *state = CircuitState::Closed;
                            self.metrics.reset();
                        }
                    }
                }
            }
            Err(_) => {
                match current_state {
                    CircuitState::Closed => {
                        self.metrics.record_failure();

                        // Check if we should open the circuit
                        if self.metrics.should_trip(&self.config) {
                            if let Ok(mut state) = self.state.write() {
                                *state = CircuitState::Open;
                            }
                        }
                    }
                    CircuitState::HalfOpen => {
                        self.metrics.record_half_open_failure();

                        // Any failure in half-open immediately opens the circuit
                        if let Ok(mut state) = self.state.write() {
                            *state = CircuitState::Open;
                        }
                    }
                    CircuitState::Open => {
                        // Already open, just update failure time
                        self.metrics.record_failure();
                    }
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl SearchProvider for CircuitBreakerWrapper {
    async fn search(
        &self,
        query: &str,
        limit: u32,
        country: &str,
        locale: &str,
    ) -> Result<Vec<SearchHit>> {
        // Check if the circuit allows this request
        if !self.can_proceed()? {
            return Err(anyhow::anyhow!(
                "Search provider circuit breaker is OPEN. Provider is currently unavailable. Failure rate: {}%",
                self.failure_rate()
            ));
        }

        // Execute the search request
        let result = self.provider.search(query, limit, country, locale).await;

        // Update circuit state based on result
        self.handle_result(&result);

        result
    }

    fn backend_type(&self) -> SearchBackend {
        self.provider.backend_type()
    }

    async fn health_check(&self) -> Result<()> {
        // Always allow health checks, but don't count them in circuit breaker metrics
        // This allows external monitoring to determine provider health independently
        self.provider.health_check().await
    }
}

impl std::fmt::Debug for CircuitBreakerWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CircuitBreakerWrapper")
            .field("backend_type", &self.provider.backend_type())
            .field("state", &self.current_state())
            .field("failure_rate", &self.failure_rate())
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::none_provider::NoneProvider;

    #[tokio::test]
    async fn test_circuit_breaker_closed_state() {
        let provider = Box::new(NoneProvider::new(true));
        let circuit = CircuitBreakerWrapper::new(provider);

        assert_eq!(circuit.current_state(), CircuitState::Closed);
        assert!(circuit.can_proceed().unwrap());
    }

    #[tokio::test]
    async fn test_circuit_breaker_success_flow() {
        let provider = Box::new(NoneProvider::new(true));
        let circuit = CircuitBreakerWrapper::new(provider);

        // Successful request should not affect circuit state
        let result = circuit.search("https://example.com", 1, "us", "en").await;
        assert!(result.is_ok());
        assert_eq!(circuit.current_state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_failure_threshold() {
        let provider = Box::new(NoneProvider::new(true));
        let config = CircuitBreakerConfig {
            failure_threshold_percentage: 50,
            minimum_request_threshold: 2,
            recovery_timeout: Duration::from_secs(300), // Long timeout to prevent auto-recovery during test
            half_open_max_requests: 1,
        };
        let circuit = CircuitBreakerWrapper::with_config(provider, config);

        // Generate intentional failures to trip the circuit breaker
        // These searches contain invalid input to trigger the failure threshold
        let _fail1_result = circuit.search("no urls here", 1, "us", "en").await;
        let _fail2_result = circuit.search("still no urls", 1, "us", "en").await;

        // Circuit should now be open due to 100% failure rate
        assert_eq!(circuit.current_state(), CircuitState::Open);

        // Next request should fail fast (recovery timeout hasn't elapsed)
        let result = circuit.search("https://example.com", 1, "us", "en").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("circuit breaker is OPEN"));
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery() {
        let provider = Box::new(NoneProvider::new(true));
        let config = CircuitBreakerConfig {
            failure_threshold_percentage: 50,
            minimum_request_threshold: 2,
            recovery_timeout: Duration::from_millis(50),
            half_open_max_requests: 1,
        };
        let circuit = CircuitBreakerWrapper::with_config(provider, config);

        // Trip the circuit breaker with intentional failures
        // These invalid queries are expected to fail and open the circuit
        let _fail1_result = circuit.search("no urls", 1, "us", "en").await;
        let _fail2_result = circuit.search("no urls", 1, "us", "en").await;
        assert_eq!(circuit.current_state(), CircuitState::Open);

        // Wait for recovery timeout
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Next request should transition to half-open and succeed
        let result = circuit.search("https://example.com", 1, "us", "en").await;
        assert!(result.is_ok());

        // Circuit should close after successful half-open request
        assert_eq!(circuit.current_state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_metrics() {
        let metrics = CircuitMetrics::new();

        // Initial state
        assert_eq!(metrics.failure_rate_percentage(), 0);

        // Record some requests
        metrics.record_success();
        metrics.record_success();
        metrics.record_failure();

        // Should be 33% failure rate (1 failure out of 3 total)
        assert_eq!(metrics.failure_rate_percentage(), 33);

        // Test should_trip logic
        let config = CircuitBreakerConfig {
            minimum_request_threshold: 3,
            failure_threshold_percentage: 30,
            ..Default::default()
        };

        assert!(metrics.should_trip(&config));
    }
}
