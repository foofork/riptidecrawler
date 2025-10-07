//! Circuit breaker implementation with multi-signal support and 1 repair retry maximum

use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

use crate::{
    CompletionRequest, CompletionResponse, Cost, IntelligenceError, LlmCapabilities, LlmProvider,
    Result,
};

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CircuitState {
    /// Circuit is closed - requests pass through normally
    Closed,
    /// Circuit is open - requests are rejected immediately
    Open,
    /// Circuit is half-open - limited requests are allowed to test recovery
    HalfOpen,
}

/// Configuration for circuit breaker behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit
    pub failure_threshold: u32,
    /// Time window for counting failures (seconds)
    pub failure_window_secs: u64,
    /// Minimum number of requests before circuit can open
    pub min_request_threshold: u32,
    /// Time to wait before transitioning from Open to HalfOpen
    pub recovery_timeout_secs: u64,
    /// Maximum number of repair attempts (hard limit of 1 per requirement)
    pub max_repair_attempts: u32,
    /// Success rate threshold for closing circuit in HalfOpen state
    pub success_rate_threshold: f32,
    /// Number of requests to allow in HalfOpen state
    pub half_open_max_requests: u32,
}

impl CircuitBreakerConfig {
    /// Create a new configuration with safe defaults
    pub fn new() -> Self {
        Self {
            failure_threshold: 5,
            failure_window_secs: 60,
            min_request_threshold: 10,
            recovery_timeout_secs: 30,
            max_repair_attempts: 1, // Hard requirement limit
            success_rate_threshold: 0.7,
            half_open_max_requests: 3,
        }
    }

    /// Create a strict configuration for testing
    pub fn strict() -> Self {
        Self {
            failure_threshold: 3,
            failure_window_secs: 30,
            min_request_threshold: 5,
            recovery_timeout_secs: 15,
            max_repair_attempts: 1,
            success_rate_threshold: 0.8,
            half_open_max_requests: 2,
        }
    }

    /// Create a lenient configuration for stable environments
    pub fn lenient() -> Self {
        Self {
            failure_threshold: 10,
            failure_window_secs: 120,
            min_request_threshold: 20,
            recovery_timeout_secs: 60,
            max_repair_attempts: 1,
            success_rate_threshold: 0.6,
            half_open_max_requests: 5,
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for monitoring circuit breaker behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerStats {
    pub state: CircuitState,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub circuit_opens: u64,
    pub repair_attempts: u32,
    #[serde(skip)]
    pub last_failure_time: Option<Instant>,
    #[serde(skip)]
    pub last_success_time: Option<Instant>,
    pub current_failure_count: u32,
}

impl CircuitBreakerStats {
    fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            circuit_opens: 0,
            repair_attempts: 0,
            last_failure_time: None,
            last_success_time: None,
            current_failure_count: 0,
        }
    }

    fn success_rate(&self) -> f32 {
        if self.total_requests == 0 {
            1.0
        } else {
            self.successful_requests as f32 / self.total_requests as f32
        }
    }
}

/// Internal state of the circuit breaker
#[derive(Debug)]
struct CircuitBreakerState {
    config: CircuitBreakerConfig,
    stats: CircuitBreakerStats,
    state_change_time: Instant,
    half_open_requests: u32,
    recent_failures: Vec<Instant>,
}

impl CircuitBreakerState {
    fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            stats: CircuitBreakerStats::new(),
            state_change_time: Instant::now(),
            half_open_requests: 0,
            recent_failures: Vec::new(),
        }
    }

    /// Check if circuit should transition to open state
    fn should_open(&mut self) -> bool {
        // Clean old failures outside the window
        let cutoff = Instant::now() - Duration::from_secs(self.config.failure_window_secs);
        self.recent_failures.retain(|&time| time > cutoff);

        // Check thresholds
        self.stats.total_requests >= self.config.min_request_threshold as u64
            && self.recent_failures.len() >= self.config.failure_threshold as usize
    }

    /// Check if circuit should transition to half-open state
    fn should_half_open(&self) -> bool {
        matches!(self.stats.state, CircuitState::Open)
            && self.state_change_time.elapsed().as_secs() >= self.config.recovery_timeout_secs
    }

    /// Check if circuit should close from half-open state
    fn should_close_from_half_open(&self) -> bool {
        matches!(self.stats.state, CircuitState::HalfOpen)
            && self.half_open_requests >= self.config.half_open_max_requests
            && self.stats.success_rate() >= self.config.success_rate_threshold
    }

    /// Record a successful request
    fn record_success(&mut self) {
        self.stats.total_requests += 1;
        self.stats.successful_requests += 1;
        self.stats.last_success_time = Some(Instant::now());

        if self.stats.state == CircuitState::HalfOpen {
            self.half_open_requests += 1;
            if self.should_close_from_half_open() {
                self.transition_to_closed();
            }
        }
    }

    /// Record a failed request
    fn record_failure(&mut self) {
        self.stats.total_requests += 1;
        self.stats.failed_requests += 1;
        self.stats.current_failure_count += 1;
        let now = Instant::now();
        self.stats.last_failure_time = Some(now);
        self.recent_failures.push(now);

        match self.stats.state {
            CircuitState::Closed => {
                if self.should_open() {
                    self.transition_to_open();
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open state opens the circuit again
                self.transition_to_open();
            }
            CircuitState::Open => {
                // Already open, nothing to do
            }
        }
    }

    /// Transition to open state
    fn transition_to_open(&mut self) {
        info!("Circuit breaker opening due to failures");
        self.stats.state = CircuitState::Open;
        self.stats.circuit_opens += 1;
        self.state_change_time = Instant::now();
        self.half_open_requests = 0;
    }

    /// Transition to half-open state
    fn transition_to_half_open(&mut self) {
        if self.stats.repair_attempts >= self.config.max_repair_attempts {
            warn!(
                "Maximum repair attempts ({}) reached, keeping circuit open",
                self.config.max_repair_attempts
            );
            return;
        }

        info!("Circuit breaker transitioning to half-open for repair attempt");
        self.stats.state = CircuitState::HalfOpen;
        self.stats.repair_attempts += 1;
        self.state_change_time = Instant::now();
        self.half_open_requests = 0;
    }

    /// Transition to closed state
    fn transition_to_closed(&mut self) {
        info!("Circuit breaker closing - service recovered");
        self.stats.state = CircuitState::Closed;
        self.state_change_time = Instant::now();
        self.half_open_requests = 0;
        self.stats.current_failure_count = 0;
        self.recent_failures.clear();
    }

    /// Check current state and perform transitions if needed
    fn update_state(&mut self) {
        match self.stats.state {
            CircuitState::Open => {
                if self.should_half_open() {
                    self.transition_to_half_open();
                }
            }
            CircuitState::HalfOpen => {
                // Transitions are handled in record_success/record_failure
            }
            CircuitState::Closed => {
                // Transitions are handled in record_failure
            }
        }
    }

    /// Check if a request should be allowed
    fn should_allow_request(&mut self) -> bool {
        self.update_state();

        match self.stats.state {
            CircuitState::Closed => true,
            CircuitState::Open => false,
            CircuitState::HalfOpen => self.half_open_requests < self.config.half_open_max_requests,
        }
    }
}

/// Circuit breaker wrapper for LLM providers
pub struct CircuitBreaker {
    inner: Arc<dyn LlmProvider>,
    state: Arc<RwLock<CircuitBreakerState>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with default configuration
    pub fn new(provider: Arc<dyn LlmProvider>) -> Self {
        Self::with_config(provider, CircuitBreakerConfig::default())
    }

    /// Create a new circuit breaker with custom configuration
    pub fn with_config(provider: Arc<dyn LlmProvider>, config: CircuitBreakerConfig) -> Self {
        Self {
            inner: provider,
            state: Arc::new(RwLock::new(CircuitBreakerState::new(config))),
        }
    }

    /// Get current circuit state
    pub fn state(&self) -> CircuitState {
        self.state.read().stats.state.clone()
    }

    /// Get circuit breaker statistics
    pub fn stats(&self) -> CircuitBreakerStats {
        self.state.read().stats.clone()
    }

    /// Manually reset the circuit breaker
    pub fn reset(&self) {
        let mut state = self.state.write();
        state.transition_to_closed();
        info!("Circuit breaker manually reset");
    }

    /// Check if provider is healthy and update circuit state accordingly
    pub async fn health_check(&self) -> Result<()> {
        match self.inner.health_check().await {
            Ok(_) => {
                let state = self.state.read();
                // Don't record as success to avoid skewing stats, but note the health
                if matches!(state.stats.state, CircuitState::Open) {
                    // Optionally transition to half-open if health check passes
                    // This is a policy decision - some implementations might not do this
                }
                Ok(())
            }
            Err(e) => {
                // Don't record as failure for health checks to avoid skewing stats
                Err(e)
            }
        }
    }

    /// Force open the circuit (for testing or emergency situations)
    pub fn force_open(&self) {
        let mut state = self.state.write();
        state.transition_to_open();
        warn!("Circuit breaker forced open");
    }

    /// Get the wrapped provider
    pub fn inner(&self) -> &Arc<dyn LlmProvider> {
        &self.inner
    }

    /// Execute a request with circuit breaker protection
    async fn execute_with_circuit<F, T>(&self, operation: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        // Check if request should be allowed
        {
            let mut state = self.state.write();
            if !state.should_allow_request() {
                return Err(IntelligenceError::CircuitOpen {
                    reason: format!(
                        "Circuit breaker is {} (failures: {}, repair attempts: {}/{})",
                        match state.stats.state {
                            CircuitState::Open => "OPEN",
                            CircuitState::HalfOpen => "HALF_OPEN (max requests reached)",
                            CircuitState::Closed => "CLOSED", // Shouldn't happen
                        },
                        state.stats.failed_requests,
                        state.stats.repair_attempts,
                        state.config.max_repair_attempts
                    ),
                });
            }
        }

        // Execute the operation
        match operation.await {
            Ok(result) => {
                let mut state = self.state.write();
                state.record_success();
                Ok(result)
            }
            Err(error) => {
                let mut state = self.state.write();
                state.record_failure();
                error!(
                    "Request failed, circuit breaker stats: state={:?}, failures={}, repair_attempts={}/{}",
                    state.stats.state,
                    state.stats.failed_requests,
                    state.stats.repair_attempts,
                    state.config.max_repair_attempts
                );
                Err(error)
            }
        }
    }
}

#[async_trait]
impl LlmProvider for CircuitBreaker {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        self.execute_with_circuit(self.inner.complete(request))
            .await
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.execute_with_circuit(self.inner.embed(text)).await
    }

    fn capabilities(&self) -> LlmCapabilities {
        self.inner.capabilities()
    }

    fn estimate_cost(&self, tokens: usize) -> Cost {
        self.inner.estimate_cost(tokens)
    }

    async fn health_check(&self) -> Result<()> {
        self.health_check().await
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn is_available(&self) -> bool {
        match self.state() {
            CircuitState::Closed => self.inner.is_available().await,
            CircuitState::Open => false,
            CircuitState::HalfOpen => {
                // In half-open state, check if we can still accept requests
                let state = self.state.read();
                state.half_open_requests < state.config.half_open_max_requests
            }
        }
    }
}

/// Helper function to wrap a provider with default circuit breaker
pub fn with_circuit_breaker(provider: Arc<dyn LlmProvider>) -> CircuitBreaker {
    CircuitBreaker::new(provider)
}

/// Helper function to wrap a provider with custom circuit breaker config
pub fn with_custom_circuit_breaker(
    provider: Arc<dyn LlmProvider>,
    config: CircuitBreakerConfig,
) -> CircuitBreaker {
    CircuitBreaker::with_config(provider, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_provider::MockLlmProvider;
    use crate::provider::Message;

    #[tokio::test]
    async fn test_circuit_breaker_closed_state() {
        let mock_provider = Arc::new(MockLlmProvider::new());
        let circuit_breaker = CircuitBreaker::new(mock_provider);

        assert_eq!(circuit_breaker.state(), CircuitState::Closed);

        let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);

        let result = circuit_breaker.complete(request).await;
        assert!(result.is_ok());
        assert_eq!(circuit_breaker.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_on_failures() {
        let mock_provider = Arc::new(MockLlmProvider::new().fail_after(0)); // Always fail
        let config = CircuitBreakerConfig::strict();
        let circuit_breaker = CircuitBreaker::with_config(mock_provider, config);

        let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);

        // Make enough requests to trigger circuit opening
        for _ in 0..10 {
            let _ = circuit_breaker.complete(request.clone()).await;
        }

        assert_eq!(circuit_breaker.state(), CircuitState::Open);
    }

    #[tokio::test]
    async fn test_circuit_breaker_rejects_when_open() {
        let mock_provider = Arc::new(MockLlmProvider::new().fail_after(0));
        let config = CircuitBreakerConfig::strict();
        let circuit_breaker = CircuitBreaker::with_config(mock_provider, config);

        // Force circuit open
        circuit_breaker.force_open();

        let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);

        let result = circuit_breaker.complete(request).await;
        assert!(matches!(result, Err(IntelligenceError::CircuitOpen { .. })));
    }

    #[tokio::test]
    async fn test_max_repair_attempts() {
        let mock_provider = Arc::new(MockLlmProvider::new().fail_after(0));
        let config = CircuitBreakerConfig::strict();
        let circuit_breaker = CircuitBreaker::with_config(mock_provider, config);

        // Trigger circuit opening
        let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);

        for _ in 0..10 {
            let _ = circuit_breaker.complete(request.clone()).await;
        }

        assert_eq!(circuit_breaker.state(), CircuitState::Open);

        // Wait for recovery timeout and trigger half-open
        tokio::time::sleep(Duration::from_secs(16)).await;

        // Should allow one repair attempt
        let result = circuit_breaker.complete(request.clone()).await;
        // This will fail and circuit should open again
        assert!(result.is_err());

        // Verify repair attempts were recorded
        let stats = circuit_breaker.stats();
        assert_eq!(stats.repair_attempts, 1);

        // Wait again and try - should not transition to half-open due to max repair attempts
        tokio::time::sleep(Duration::from_secs(16)).await;

        // Should still be open and reject requests
        let result = circuit_breaker.complete(request).await;
        assert!(matches!(result, Err(IntelligenceError::CircuitOpen { .. })));
    }

    #[tokio::test]
    async fn test_circuit_breaker_reset() {
        let mock_provider = Arc::new(MockLlmProvider::new());
        let circuit_breaker = CircuitBreaker::new(mock_provider);

        circuit_breaker.force_open();
        assert_eq!(circuit_breaker.state(), CircuitState::Open);

        circuit_breaker.reset();
        assert_eq!(circuit_breaker.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_stats() {
        let mock_provider = Arc::new(MockLlmProvider::new());
        let circuit_breaker = CircuitBreaker::new(mock_provider);

        let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);

        // Make a successful request
        circuit_breaker.complete(request).await.unwrap();

        let stats = circuit_breaker.stats();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.successful_requests, 1);
        assert_eq!(stats.failed_requests, 0);
    }
}
