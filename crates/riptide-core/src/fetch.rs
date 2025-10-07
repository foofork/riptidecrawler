use crate::circuit::{self, CircuitBreaker, Config as CircuitConfig};
use crate::robots::{RobotsConfig, RobotsManager};
use crate::{telemetry_info, telemetry_span};
// Removed unused error imports
use anyhow::Result;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, instrument};

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub open_cooldown_ms: u64,
    pub half_open_max_in_flight: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            open_cooldown_ms: 30_000,
            half_open_max_in_flight: 3,
        }
    }
}

pub use crate::circuit::State as CircuitState;

// Circuit breaker implementation moved to circuit.rs module

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
#[derive(Debug)]
pub struct ReliableHttpClient {
    client: Client,
    retry_config: RetryConfig,
    circuit_breaker: Arc<CircuitBreaker>,
    robots_manager: Option<Arc<RobotsManager>>,
}

impl ReliableHttpClient {
    pub fn new(
        retry_config: RetryConfig,
        circuit_breaker_config: CircuitBreakerConfig,
    ) -> Result<Self> {
        let client = Client::builder()
            .user_agent("RipTide/1.0")
            .http2_prior_knowledge()
            .gzip(true)
            .brotli(true)
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(20)) // Increased to 20s for total timeout
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?;

        let cb_config = CircuitConfig {
            failure_threshold: circuit_breaker_config.failure_threshold,
            open_cooldown_ms: circuit_breaker_config.open_cooldown_ms,
            half_open_max_in_flight: circuit_breaker_config.half_open_max_in_flight,
        };

        Ok(Self {
            client,
            retry_config,
            circuit_breaker: CircuitBreaker::new(cb_config, Arc::new(circuit::RealClock)),
            robots_manager: None,
        })
    }

    /// Create a new client with robots.txt compliance enabled
    pub fn new_with_robots(
        retry_config: RetryConfig,
        circuit_breaker_config: CircuitBreakerConfig,
        robots_config: RobotsConfig,
    ) -> Result<Self> {
        let client = Client::builder()
            .user_agent(&robots_config.user_agent)
            .http2_prior_knowledge()
            .gzip(true)
            .brotli(true)
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?;

        let cb_config = CircuitConfig {
            failure_threshold: circuit_breaker_config.failure_threshold,
            open_cooldown_ms: circuit_breaker_config.open_cooldown_ms,
            half_open_max_in_flight: circuit_breaker_config.half_open_max_in_flight,
        };

        Ok(Self {
            client,
            retry_config,
            circuit_breaker: CircuitBreaker::new(cb_config, Arc::new(circuit::RealClock)),
            robots_manager: Some(Arc::new(RobotsManager::new(robots_config)?)),
        })
    }

    /// Enable robots.txt compliance for existing client
    pub fn with_robots_manager(mut self, robots_config: RobotsConfig) -> Self {
        match RobotsManager::new(robots_config) {
            Ok(manager) => {
                self.robots_manager = Some(Arc::new(manager));
            }
            Err(e) => {
                tracing::warn!("Failed to create robots manager: {}", e);
                // Continue without robots manager
            }
        }
        self
    }

    /// Perform HTTP GET with retry logic, circuit breaker protection, and robots.txt compliance
    #[instrument(skip(self), fields(url = %url))]
    pub async fn get_with_retry(&self, url: &str) -> Result<Response> {
        let _span = telemetry_span!(
            "http_fetch_with_retry",
            url = %url,
            max_attempts = %self.retry_config.max_attempts
        );

        info!("Starting HTTP GET request with retry");
        // Check robots.txt compliance first if manager is available
        if let Some(robots_manager) = &self.robots_manager {
            let _robots_span = telemetry_span!("robots_check", url = %url);
            if !robots_manager.can_crawl_with_wait(url).await? {
                error!("URL blocked by robots.txt: {}", url);
                return Err(anyhow::anyhow!("URL blocked by robots.txt: {}", url));
            }
            info!("Robots.txt check passed");
        }

        let mut last_error = None;

        for attempt in 0..self.retry_config.max_attempts {
            // Use circuit breaker for the request
            match circuit::guarded_call(&self.circuit_breaker, || async {
                self.client
                    .get(url)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!(e))
            })
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
                Err(err) => {
                    let err_str = err.to_string();
                    if err_str.contains("circuit open") {
                        return Err(anyhow::anyhow!("Circuit breaker is open for {}", url));
                    }
                    // For other circuit breaker errors, treat as generic failure
                    // Treat other circuit breaker errors as non-retryable for now
                    return Err(err);
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
        self.circuit_breaker.state()
    }

    pub fn get_circuit_breaker_failure_count(&self) -> u32 {
        self.circuit_breaker.failure_count()
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

/// FetchEngine provides a high-level interface for fetching content with full integration
#[derive(Debug)]
pub struct FetchEngine {
    client: ReliableHttpClient,
}

impl FetchEngine {
    /// Create a new fetch engine with default configuration
    pub fn new() -> Result<Self> {
        let client =
            ReliableHttpClient::new(RetryConfig::default(), CircuitBreakerConfig::default())?;

        Ok(Self { client })
    }

    /// Create a fetch engine with custom configuration
    pub fn with_config(
        retry_config: RetryConfig,
        circuit_breaker_config: CircuitBreakerConfig,
    ) -> Result<Self> {
        let client = ReliableHttpClient::new(retry_config, circuit_breaker_config)?;
        Ok(Self { client })
    }

    /// Create a fetch engine with robots.txt compliance
    pub fn with_robots(
        retry_config: RetryConfig,
        circuit_breaker_config: CircuitBreakerConfig,
        robots_config: RobotsConfig,
    ) -> Result<Self> {
        let client = ReliableHttpClient::new_with_robots(
            retry_config,
            circuit_breaker_config,
            robots_config,
        )?;
        Ok(Self { client })
    }

    /// Fetch content from a URL with full retry and circuit breaker protection
    pub async fn fetch(&self, url: &str) -> Result<Response> {
        self.client.get_with_retry(url).await
    }

    /// Fetch content and return as text
    pub async fn fetch_text(&self, url: &str) -> Result<String> {
        let response = self.client.get_with_retry(url).await?;
        let text = response.text().await.map_err(|e| anyhow::anyhow!(e))?;
        Ok(text)
    }

    /// Fetch content and return as bytes
    pub async fn fetch_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let response = self.client.get_with_retry(url).await?;
        let bytes = response.bytes().await.map_err(|e| anyhow::anyhow!(e))?;
        Ok(bytes.to_vec())
    }

    /// Get circuit breaker status
    pub async fn get_circuit_breaker_status(&self) -> CircuitState {
        self.client.get_circuit_breaker_state().await
    }

    /// Check if robots.txt compliance is enabled
    pub fn is_robots_enabled(&self) -> bool {
        self.client.is_robots_enabled()
    }
}

/// Rate limiting configuration for per-host rate limiting
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_capacity: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10,
            burst_capacity: 20,
        }
    }
}

/// Per-host rate limiter using token bucket algorithm
#[derive(Debug)]
pub struct RateLimiter {
    config: RateLimitConfig,
    tokens: Arc<tokio::sync::Mutex<f64>>,
    last_refill: Arc<tokio::sync::Mutex<std::time::Instant>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config: config.clone(),
            tokens: Arc::new(tokio::sync::Mutex::new(config.burst_capacity as f64)),
            last_refill: Arc::new(tokio::sync::Mutex::new(std::time::Instant::now())),
        }
    }

    pub async fn check_limit(&self) -> Result<()> {
        let mut tokens = self.tokens.lock().await;
        let mut last_refill = self.last_refill.lock().await;

        let now = std::time::Instant::now();
        let elapsed = now.duration_since(*last_refill).as_secs_f64();

        // Refill tokens based on elapsed time
        *tokens = (*tokens + elapsed * self.config.requests_per_second as f64)
            .min(self.config.burst_capacity as f64);
        *last_refill = now;

        if *tokens >= 1.0 {
            *tokens -= 1.0;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Rate limit exceeded"))
        }
    }
}

/// Per-host metrics for tracking request performance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostMetrics {
    pub request_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub total_duration_ms: u64,
}

/// Aggregated metrics response for all hosts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchMetricsResponse {
    pub hosts: std::collections::HashMap<String, HostMetricsResponse>,
    pub total_requests: u64,
    pub total_success: u64,
    pub total_failures: u64,
}

/// Per-host metrics response with calculated averages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMetricsResponse {
    pub request_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub avg_duration_ms: f64,
    pub circuit_state: String,
}

/// Enhanced FetchEngine with per-host circuit breakers, rate limiting, and metrics
#[derive(Debug)]
pub struct PerHostFetchEngine {
    /// Per-host HTTP clients with individual circuit breakers
    clients: Arc<std::sync::RwLock<std::collections::HashMap<String, Arc<ReliableHttpClient>>>>,

    /// Per-host rate limiters
    rate_limiters: Arc<std::sync::RwLock<std::collections::HashMap<String, Arc<RateLimiter>>>>,

    /// Per-host metrics tracking
    metrics: Arc<std::sync::RwLock<std::collections::HashMap<String, HostMetrics>>>,

    /// Default retry configuration for new clients
    retry_config: RetryConfig,

    /// Default circuit breaker configuration for new clients
    circuit_config: CircuitBreakerConfig,

    /// Default rate limit configuration for new rate limiters
    rate_limit_config: RateLimitConfig,
}

impl PerHostFetchEngine {
    /// Create a new per-host fetch engine with configuration
    pub fn new(
        retry_config: RetryConfig,
        circuit_config: CircuitBreakerConfig,
        rate_limit_config: RateLimitConfig,
    ) -> Result<Self> {
        Ok(Self {
            clients: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
            rate_limiters: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
            metrics: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
            retry_config,
            circuit_config,
            rate_limit_config,
        })
    }

    /// Fetch content from a URL with per-host circuit breakers and rate limiting
    #[instrument(skip(self), fields(url = %url))]
    pub async fn fetch(&self, url: &str) -> Result<Response> {
        let host = Self::extract_host(url)?;

        // 1. Check per-host rate limit
        self.check_rate_limit(&host).await?;

        // 2. Log request start
        self.log_request_start(url);

        // 3. Get or create per-host client
        let client = self.get_or_create_client(&host).await?;

        // 4. Make request with circuit breaker
        let start = std::time::Instant::now();
        let result = client.get_with_retry(url).await;

        // 5. Track metrics
        let duration = start.elapsed();
        self.record_metrics(&host, duration, result.is_ok());

        // 6. Log response
        self.log_response(url, &result, duration);

        result
    }

    /// Get or create a per-host HTTP client
    async fn get_or_create_client(&self, host: &str) -> Result<Arc<ReliableHttpClient>> {
        // Check if client already exists (read lock)
        {
            let clients = self.clients.read().unwrap();
            if let Some(client) = clients.get(host) {
                return Ok(client.clone());
            }
        }

        // Create new client (write lock)
        let mut clients = self.clients.write().unwrap();

        // Double-check in case another thread created it
        if let Some(client) = clients.get(host) {
            return Ok(client.clone());
        }

        // Create new client with per-host circuit breaker
        let client = Arc::new(ReliableHttpClient::new(
            self.retry_config.clone(),
            self.circuit_config.clone(),
        )?);

        clients.insert(host.to_string(), client.clone());
        info!(host = %host, "Created new per-host HTTP client");

        Ok(client)
    }

    /// Get or create a per-host rate limiter
    fn get_or_create_rate_limiter(&self, host: &str) -> Arc<RateLimiter> {
        // Check if rate limiter already exists (read lock)
        {
            let limiters = self.rate_limiters.read().unwrap();
            if let Some(limiter) = limiters.get(host) {
                return limiter.clone();
            }
        }

        // Create new rate limiter (write lock)
        let mut limiters = self.rate_limiters.write().unwrap();

        // Double-check in case another thread created it
        if let Some(limiter) = limiters.get(host) {
            return limiter.clone();
        }

        let limiter = Arc::new(RateLimiter::new(self.rate_limit_config.clone()));
        limiters.insert(host.to_string(), limiter.clone());
        debug!(host = %host, "Created new per-host rate limiter");

        limiter
    }

    /// Check per-host rate limit
    async fn check_rate_limit(&self, host: &str) -> Result<()> {
        let limiter = self.get_or_create_rate_limiter(host);
        limiter.check_limit().await
    }

    /// Extract hostname from URL
    fn extract_host(url: &str) -> Result<String> {
        let parsed = url::Url::parse(url).map_err(|e| anyhow::anyhow!("Invalid URL: {}", e))?;

        let host = parsed
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("No host in URL"))?
            .to_string();

        Ok(host)
    }

    /// Extract hostname from URL (exposed for testing)
    #[cfg(test)]
    pub fn extract_host_for_test(url: &str) -> Result<String> {
        Self::extract_host(url)
    }

    /// Log request start
    fn log_request_start(&self, url: &str) {
        info!(url = %url, "FetchEngine: Starting request");
    }

    /// Log response with details
    fn log_response(&self, url: &str, result: &Result<Response>, duration: Duration) {
        match result {
            Ok(resp) => {
                info!(
                    url = %url,
                    status = resp.status().as_u16(),
                    duration_ms = duration.as_millis(),
                    "FetchEngine: Request completed"
                );
            }
            Err(e) => {
                error!(
                    url = %url,
                    error = %e,
                    duration_ms = duration.as_millis(),
                    "FetchEngine: Request failed"
                );
            }
        }
    }

    /// Record metrics for a host
    fn record_metrics(&self, host: &str, duration: Duration, success: bool) {
        let mut metrics_map = self.metrics.write().unwrap();
        let host_metrics = metrics_map.entry(host.to_string()).or_default();

        host_metrics.request_count += 1;
        host_metrics.total_duration_ms += duration.as_millis() as u64;

        if success {
            host_metrics.success_count += 1;
        } else {
            host_metrics.failure_count += 1;
        }
    }

    /// Get metrics for a specific host
    pub fn get_host_metrics(&self, host: &str) -> Option<HostMetrics> {
        let metrics = self.metrics.read().unwrap();
        metrics.get(host).cloned()
    }

    /// Get metrics for all hosts
    pub async fn get_all_metrics(&self) -> FetchMetricsResponse {
        // Clone data needed from locked sections to avoid holding locks across await
        let (metrics_snapshot, clients_snapshot) = {
            let metrics = self.metrics.read().unwrap();
            let clients = self.clients.read().unwrap();
            (metrics.clone(), clients.clone())
        }; // Locks dropped here

        let mut hosts = std::collections::HashMap::new();
        let mut total_requests = 0;
        let mut total_success = 0;
        let mut total_failures = 0;

        for (host, host_metrics) in metrics_snapshot.iter() {
            total_requests += host_metrics.request_count;
            total_success += host_metrics.success_count;
            total_failures += host_metrics.failure_count;

            let avg_duration_ms = if host_metrics.request_count > 0 {
                host_metrics.total_duration_ms as f64 / host_metrics.request_count as f64
            } else {
                0.0
            };

            let circuit_state = if let Some(client) = clients_snapshot.get(host) {
                format!("{:?}", client.get_circuit_breaker_state().await)
            } else {
                "Unknown".to_string()
            };

            hosts.insert(
                host.clone(),
                HostMetricsResponse {
                    request_count: host_metrics.request_count,
                    success_count: host_metrics.success_count,
                    failure_count: host_metrics.failure_count,
                    avg_duration_ms,
                    circuit_state,
                },
            );
        }

        FetchMetricsResponse {
            hosts,
            total_requests,
            total_success,
            total_failures,
        }
    }
}

/// Check if an error is retryable
#[allow(dead_code)]
fn is_retryable_error(error: &reqwest::Error) -> bool {
    error.is_timeout() || error.is_connect() || error.is_request() // Connection-level issues
}

/// Legacy function for backward compatibility
pub fn http_client() -> Result<Client> {
    Client::builder()
        .user_agent("RipTide/1.0")
        .http2_prior_knowledge()
        .gzip(true)
        .brotli(true)
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(20)) // Updated to 20s
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))
}

/// Legacy function for backward compatibility with telemetry
#[instrument(skip(client), fields(url = %url))]
pub async fn get(client: &Client, url: &str) -> Result<Response> {
    let _span = telemetry_span!(
        "http_fetch",
        url = %url,
        method = "GET"
    );

    info!("Starting HTTP GET request");
    let start_time = std::time::Instant::now();

    match client.get(url).send().await {
        Ok(response) => {
            let duration = start_time.elapsed();
            let status = response.status();

            telemetry_info!(
                status_code = status.as_u16(),
                duration_ms = duration.as_millis(),
                "HTTP request completed"
            );

            if status.is_success() {
                Ok(response)
            } else {
                let error_msg = format!("HTTP error: {}", status);
                error!("{}", error_msg);
                Err(anyhow::anyhow!(error_msg))
            }
        }
        Err(e) => {
            let duration = start_time.elapsed();
            error!("HTTP request failed after {:?}: {}", duration, e);
            Err(anyhow::anyhow!("Request failed: {}", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    #[test]
    fn test_client_creation() {
        let _client = http_client();
        // Test passes if client creation doesn't panic
    }

    #[test]
    fn test_reliable_client_creation() {
        let _client =
            ReliableHttpClient::new(RetryConfig::default(), CircuitBreakerConfig::default());
        // Test passes if client creation doesn't panic
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
                .expect("Failed to create reliable HTTP client for test")
                .with_robots_manager(RobotsConfig::default());

        assert!(client.is_robots_enabled());
        assert!(client.get_robots_manager().is_some());
    }

    #[tokio::test]
    async fn test_circuit_breaker_transitions() {
        // Using TestClock for deterministic testing
        #[derive(Debug)]
        struct TestClock {
            now: std::sync::atomic::AtomicU64,
        }
        impl circuit::Clock for TestClock {
            fn now_ms(&self) -> u64 {
                self.now.load(std::sync::atomic::Ordering::Relaxed)
            }
        }

        let test_clock = Arc::new(TestClock {
            now: std::sync::atomic::AtomicU64::new(1000),
        });

        let circuit = CircuitBreaker::new(
            circuit::Config {
                failure_threshold: 2,
                open_cooldown_ms: 100,
                half_open_max_in_flight: 2,
            },
            test_clock.clone(),
        );

        // Initially closed
        assert_eq!(circuit.state(), circuit::State::Closed);

        // Trigger failures to open circuit
        circuit.on_failure();
        assert_eq!(circuit.state(), circuit::State::Closed);
        circuit.on_failure();
        assert_eq!(circuit.state(), circuit::State::Open);
        assert_eq!(circuit.failure_count(), 0); // Reset after tripping

        // Should reject while in open state
        assert!(circuit.try_acquire().is_err());

        // Advance time past cooldown
        test_clock
            .now
            .store(1100, std::sync::atomic::Ordering::Relaxed);

        // Next acquire should transition to half-open and return permit
        let permit = circuit
            .try_acquire()
            .expect("should get permit in test conditions");
        assert!(permit.is_some());
        assert_eq!(circuit.state(), circuit::State::HalfOpen);

        // Success should close the circuit
        circuit.on_success();
        assert_eq!(circuit.state(), circuit::State::Closed);

        // Can acquire again when closed
        let permit2 = circuit
            .try_acquire()
            .expect("should get permit when closed in test conditions");
        assert!(permit2.is_none()); // Closed state doesn't require permits
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
        )
        .expect("Failed to create reliable HTTP client for test");

        assert_eq!(client.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(client.calculate_delay(1), Duration::from_millis(200));
        assert_eq!(client.calculate_delay(2), Duration::from_millis(400));
    }

    // TODO: Implement test_retryable_error_detection
    // This test would require creating actual reqwest::Error instances to test the
    // is_retryable_error function, which checks for timeout, connect, and request errors.
    // Consider using mock HTTP servers or error injection for comprehensive testing.
}
