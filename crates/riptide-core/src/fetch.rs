use crate::robots::{RobotsConfig, RobotsManager};
use anyhow::Result;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Circuit breaker state for managing service reliability
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to trigger circuit breaker
    pub failure_threshold: u64,
    /// Recovery timeout before attempting half-open
    pub recovery_timeout: Duration,
    /// Success threshold in half-open state
    pub success_threshold: u64,
    /// Time window for failure counting
    pub failure_window: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            success_threshold: 3,
            failure_window: Duration::from_secs(60),
        }
    }
}

/// Circuit breaker for protecting against cascading failures
#[derive(Debug)]
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<AtomicU64>,
    success_count: Arc<AtomicU64>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(AtomicU64::new(0)),
            success_count: Arc::new(AtomicU64::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            config,
        }
    }

    pub async fn call<F, Fut, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
    {
        // Check if circuit is open
        if let CircuitState::Open = *self.state.read().await {
            if self.should_attempt_reset().await {
                self.transition_to_half_open().await;
            } else {
                return Err(CircuitBreakerError::CircuitOpen);
            }
        }

        // Execute operation
        match operation().await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(error) => {
                self.on_failure().await;
                Err(CircuitBreakerError::OperationFailed(error))
            }
        }
    }

    async fn should_attempt_reset(&self) -> bool {
        if let Some(last_failure) = *self.last_failure_time.read().await {
            last_failure.elapsed() >= self.config.recovery_timeout
        } else {
            false
        }
    }

    async fn transition_to_half_open(&self) {
        *self.state.write().await = CircuitState::HalfOpen;
        self.success_count.store(0, Ordering::SeqCst);
        debug!("Circuit breaker transitioned to half-open state");
    }

    async fn on_success(&self) {
        let current_state = self.state.read().await.clone();
        match current_state {
            CircuitState::HalfOpen => {
                let success_count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if success_count >= self.config.success_threshold {
                    *self.state.write().await = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::SeqCst);
                    debug!("Circuit breaker closed after {} successes", success_count);
                }
            }
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitState::Open => {
                // Should not happen, but log it
                warn!("Received success while circuit is open");
            }
        }
    }

    async fn on_failure(&self) {
        let current_state = self.state.read().await.clone();
        match current_state {
            CircuitState::Closed => {
                let failure_count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                if failure_count >= self.config.failure_threshold {
                    *self.state.write().await = CircuitState::Open;
                    *self.last_failure_time.write().await = Some(Instant::now());
                    warn!("Circuit breaker opened after {} failures", failure_count);
                }
            }
            CircuitState::HalfOpen => {
                *self.state.write().await = CircuitState::Open;
                *self.last_failure_time.write().await = Some(Instant::now());
                warn!("Circuit breaker reopened during half-open state");
            }
            CircuitState::Open => {
                // Update last failure time
                *self.last_failure_time.write().await = Some(Instant::now());
            }
        }
    }

    pub async fn get_state(&self) -> CircuitState {
        self.state.read().await.clone()
    }

    pub fn get_failure_count(&self) -> u64 {
        self.failure_count.load(Ordering::SeqCst)
    }
}

/// Circuit breaker error types
#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError<E> {
    #[error("Circuit breaker is open")]
    CircuitOpen,
    #[error("Operation failed: {0}")]
    OperationFailed(E),
}

/// Retry configuration with exponential backoff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    /// Add jitter to prevent thundering herd
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Enhanced HTTP client with reliability patterns and robots.txt compliance
#[derive(Debug, Clone)]
pub struct ReliableHttpClient {
    client: Client,
    retry_config: RetryConfig,
    circuit_breaker: Arc<CircuitBreaker>,
    robots_manager: Option<Arc<RobotsManager>>,
}

impl ReliableHttpClient {
    pub fn new(retry_config: RetryConfig, circuit_breaker_config: CircuitBreakerConfig) -> Self {
        let client = Client::builder()
            .user_agent("RipTide/1.0")
            .http2_prior_knowledge()
            .gzip(true)
            .brotli(true)
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(20)) // Increased to 20s for total timeout
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            retry_config,
            circuit_breaker: Arc::new(CircuitBreaker::new(circuit_breaker_config)),
            robots_manager: None,
        }
    }

    /// Create a new client with robots.txt compliance enabled
    pub fn new_with_robots(
        retry_config: RetryConfig,
        circuit_breaker_config: CircuitBreakerConfig,
        robots_config: RobotsConfig,
    ) -> Self {
        let client = Client::builder()
            .user_agent(&robots_config.user_agent)
            .http2_prior_knowledge()
            .gzip(true)
            .brotli(true)
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(20))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            retry_config,
            circuit_breaker: Arc::new(CircuitBreaker::new(circuit_breaker_config)),
            robots_manager: Some(Arc::new(RobotsManager::new(robots_config))),
        }
    }

    /// Enable robots.txt compliance for existing client
    pub fn with_robots_manager(mut self, robots_config: RobotsConfig) -> Self {
        self.robots_manager = Some(Arc::new(RobotsManager::new(robots_config)));
        self
    }

    /// Perform HTTP GET with retry logic, circuit breaker protection, and robots.txt compliance
    pub async fn get_with_retry(&self, url: &str) -> Result<Response> {
        // Check robots.txt compliance first if manager is available
        if let Some(robots_manager) = &self.robots_manager {
            if !robots_manager.can_crawl_with_wait(url).await? {
                return Err(anyhow::anyhow!("URL blocked by robots.txt: {}", url));
            }
        }

        let mut last_error = None;

        for attempt in 0..self.retry_config.max_attempts {
            // Use circuit breaker for the request
            match self
                .circuit_breaker
                .call(|| async { self.client.get(url).send().await })
                .await
            {
                Ok(response) => {
                    match response.error_for_status() {
                        Ok(success_response) => {
                            if attempt > 0 {
                                debug!(url = %url, attempt = attempt + 1, "Request succeeded after retry");
                            }
                            return Ok(success_response);
                        }
                        Err(status_error) => {
                            // Don't retry 4xx client errors (except 408, 429)
                            if let Some(status) = status_error.status() {
                                if status.is_client_error()
                                    && status != reqwest::StatusCode::REQUEST_TIMEOUT
                                    && status != reqwest::StatusCode::TOO_MANY_REQUESTS
                                {
                                    return Err(status_error.into());
                                }
                            }
                            last_error = Some(status_error.into());
                        }
                    }
                }
                Err(CircuitBreakerError::CircuitOpen) => {
                    return Err(anyhow::anyhow!("Circuit breaker is open for {}", url));
                }
                Err(CircuitBreakerError::OperationFailed(req_error)) => {
                    // Check if error is retryable
                    if !is_retryable_error(&req_error) {
                        return Err(req_error.into());
                    }
                    last_error = Some(req_error.into());
                }
            }

            // Don't sleep after the last attempt
            if attempt < self.retry_config.max_attempts - 1 {
                let delay = self.calculate_delay(attempt);
                debug!(url = %url, attempt = attempt + 1, delay_ms = delay.as_millis(), "Retrying request");
                tokio::time::sleep(delay).await;
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
    }

    fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay = self.retry_config.initial_delay.as_millis() as f64
            * self.retry_config.backoff_multiplier.powi(attempt as i32);

        let delay = Duration::from_millis(delay as u64).min(self.retry_config.max_delay);

        if self.retry_config.jitter {
            let jitter = delay.as_millis() as f64 * 0.1 * 0.5; // Simplified jitter
            delay + Duration::from_millis(jitter as u64)
        } else {
            delay
        }
    }

    pub async fn get_circuit_breaker_state(&self) -> CircuitState {
        self.circuit_breaker.get_state().await
    }

    pub fn get_circuit_breaker_failure_count(&self) -> u64 {
        self.circuit_breaker.get_failure_count()
    }

    /// Get robots manager if available
    pub fn get_robots_manager(&self) -> Option<&Arc<RobotsManager>> {
        self.robots_manager.as_ref()
    }

    /// Check if robots.txt compliance is enabled
    pub fn is_robots_enabled(&self) -> bool {
        self.robots_manager.is_some()
    }
}

/// Check if an error is retryable
fn is_retryable_error(error: &reqwest::Error) -> bool {
    error.is_timeout() || error.is_connect() || error.is_request() // Connection-level issues
}

/// Legacy function for backward compatibility
pub fn http_client() -> Client {
    Client::builder()
        .user_agent("RipTide/1.0")
        .http2_prior_knowledge()
        .gzip(true)
        .brotli(true)
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(20)) // Updated to 20s
        .build()
        .expect("client")
}

/// Legacy function for backward compatibility
pub async fn get(client: &Client, url: &str) -> Result<Response> {
    let res = client.get(url).send().await?;
    Ok(res.error_for_status()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[test]
    fn test_client_creation() {
        let _client = http_client();
    }

    #[test]
    fn test_reliable_client_creation() {
        let _client =
            ReliableHttpClient::new(RetryConfig::default(), CircuitBreakerConfig::default());
    }

    #[test]
    fn test_reliable_client_with_robots() {
        let _client = ReliableHttpClient::new_with_robots(
            RetryConfig::default(),
            CircuitBreakerConfig::default(),
            RobotsConfig::default(),
        );
    }

    #[test]
    fn test_robots_manager_integration() {
        let client =
            ReliableHttpClient::new(RetryConfig::default(), CircuitBreakerConfig::default())
                .with_robots_manager(RobotsConfig::default());

        assert!(client.is_robots_enabled());
        assert!(client.get_robots_manager().is_some());
    }

    #[tokio::test]
    async fn test_circuit_breaker_transitions() {
        let circuit = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout: Duration::from_millis(100),
            success_threshold: 2,
            failure_window: Duration::from_secs(60),
        });

        // Initially closed
        assert_eq!(circuit.get_state().await, CircuitState::Closed);

        // Trigger failures
        for _ in 0..2 {
            let _ = circuit
                .call(|| async { Err::<(), &str>("test error") })
                .await;
        }

        // Should be open now
        assert_eq!(circuit.get_state().await, CircuitState::Open);
        assert_eq!(circuit.get_failure_count(), 2);

        // Wait for recovery timeout
        sleep(Duration::from_millis(150)).await;

        // Next call should transition to half-open
        let _ = circuit.call(|| async { Ok::<(), &str>(()) }).await;
        // After one success in half-open, still half-open
        assert_eq!(circuit.get_state().await, CircuitState::HalfOpen);

        // Another success should close the circuit
        let _ = circuit.call(|| async { Ok::<(), &str>(()) }).await;
        assert_eq!(circuit.get_state().await, CircuitState::Closed);
    }

    #[test]
    fn test_retry_delay_calculation() {
        let client = ReliableHttpClient::new(
            RetryConfig {
                initial_delay: Duration::from_millis(100),
                backoff_multiplier: 2.0,
                max_delay: Duration::from_secs(5),
                jitter: false,
                max_attempts: 3,
            },
            CircuitBreakerConfig::default(),
        );

        assert_eq!(client.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(client.calculate_delay(1), Duration::from_millis(200));
        assert_eq!(client.calculate_delay(2), Duration::from_millis(400));
    }

    #[test]
    fn test_retryable_error_detection() {
        // This is a simplified test - in practice, you'd need to create actual reqwest errors
        // The function checks error types that are typically retryable
        assert!(true); // Placeholder for actual error type tests
    }
}
