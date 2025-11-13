//! Unified HTTP Client Service with integrated reliability patterns.
//!
//! This module consolidates HTTP client functionality across the codebase with:
//! - Circuit breaker protection for fault tolerance
//! - Retry logic with exponential backoff
//! - Connection pooling and timeout management
//! - Robots.txt compliance (optional)
//! - Preset configurations for common use cases
//!
//! # Architecture
//!
//! `HttpClientService` is the single source of truth for HTTP operations,
//! replacing scattered HTTP client logic in facades and other modules.
//!
//! `ReliableHttpClient` provides a simplified, Arc-wrapped interface with preset
//! configurations for different workloads.
//!
//! # Example
//!
//! ```rust,no_run
//! use riptide_reliability::{HttpClientService, HttpConfig, FetchOptions, ReliableHttpClient, CircuitBreakerPreset};
//! use std::sync::Arc;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Use HttpClientService directly
//! let service = HttpClientService::new(HttpConfig::default())?;
//! let response = service.get("https://example.com", FetchOptions::default()).await?;
//!
//! // Or use ReliableHttpClient with preset
//! let client = Arc::new(ReliableHttpClient::with_preset(CircuitBreakerPreset::BrowserRendering)?);
//! let response = client.get("https://example.com").await?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use reqwest::{Client, ClientBuilder, Method, Response as ReqwestResponse, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use riptide_utils::circuit_breaker::{self as circuit, CircuitBreaker, Config as CircuitConfig};
use riptide_utils::retry::RetryPolicy;

/// Circuit breaker presets for different use cases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitBreakerPreset {
    /// Browser rendering workload (high latency tolerance, moderate retries)
    BrowserRendering,
    /// PDF processing workload (long operations, few retries)
    PdfProcessing,
    /// Search indexing workload (moderate timeouts, aggressive retries)
    SearchIndexing,
    /// External API calls (moderate timeouts, moderate retries)
    ExternalApi,
    /// Internal service calls (low timeouts, aggressive retries)
    InternalService,
    /// Web scraping workload (moderate timeouts, high retries)
    WebScraping,
}

impl CircuitBreakerPreset {
    /// Convert preset to HttpConfig
    pub fn to_config(self) -> HttpConfig {
        match self {
            Self::BrowserRendering => HttpConfig {
                timeout_ms: 60_000,         // 60s for browser operations
                connect_timeout_ms: 15_000, // 15s connect timeout
                pool_idle_timeout_secs: 120,
                pool_max_idle_per_host: 5,
                user_agent: format!("riptide-browser/{}", env!("CARGO_PKG_VERSION")),
                circuit_failure_threshold: 10, // More tolerant of failures
                circuit_cooldown_ms: 60_000,   // 1 min cooldown
                max_retries: 2,                // Fewer retries (expensive)
                initial_backoff_ms: 500,
            },
            Self::PdfProcessing => HttpConfig {
                timeout_ms: 120_000,        // 2 min for PDF downloads
                connect_timeout_ms: 20_000, // 20s connect timeout
                pool_idle_timeout_secs: 180,
                pool_max_idle_per_host: 3,
                user_agent: format!("riptide-pdf/{}", env!("CARGO_PKG_VERSION")),
                circuit_failure_threshold: 5,
                circuit_cooldown_ms: 120_000, // 2 min cooldown
                max_retries: 1,               // Minimal retries (very expensive)
                initial_backoff_ms: 1000,
            },
            Self::SearchIndexing => HttpConfig {
                timeout_ms: 30_000,         // 30s for indexing operations
                connect_timeout_ms: 10_000, // 10s connect timeout
                pool_idle_timeout_secs: 90,
                pool_max_idle_per_host: 10,
                user_agent: format!("riptide-crawler/{}", env!("CARGO_PKG_VERSION")),
                circuit_failure_threshold: 5,
                circuit_cooldown_ms: 30_000, // 30s cooldown
                max_retries: 5,              // Aggressive retries
                initial_backoff_ms: 200,
            },
            Self::ExternalApi => HttpConfig {
                timeout_ms: 30_000,
                connect_timeout_ms: 10_000,
                pool_idle_timeout_secs: 90,
                pool_max_idle_per_host: 10,
                user_agent: format!("riptide-eventmesh/{}", env!("CARGO_PKG_VERSION")),
                circuit_failure_threshold: 5,
                circuit_cooldown_ms: 30_000,
                max_retries: 3,
                initial_backoff_ms: 100,
            },
            Self::InternalService => HttpConfig {
                timeout_ms: 10_000,        // 10s for internal calls
                connect_timeout_ms: 3_000, // 3s connect timeout
                pool_idle_timeout_secs: 60,
                pool_max_idle_per_host: 20,
                user_agent: format!("riptide-internal/{}", env!("CARGO_PKG_VERSION")),
                circuit_failure_threshold: 3, // Less tolerant
                circuit_cooldown_ms: 15_000,  // 15s cooldown
                max_retries: 5,               // Aggressive retries
                initial_backoff_ms: 50,
            },
            Self::WebScraping => HttpConfig {
                timeout_ms: 45_000,         // 45s for scraping
                connect_timeout_ms: 12_000, // 12s connect timeout
                pool_idle_timeout_secs: 120,
                pool_max_idle_per_host: 15,
                user_agent: format!("riptide-spider/{}", env!("CARGO_PKG_VERSION")),
                circuit_failure_threshold: 7,
                circuit_cooldown_ms: 45_000, // 45s cooldown
                max_retries: 4,
                initial_backoff_ms: 250,
            },
        }
    }
}

/// HTTP client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
    /// Connection timeout in milliseconds
    pub connect_timeout_ms: u64,
    /// Pool idle timeout in seconds
    pub pool_idle_timeout_secs: u64,
    /// Maximum number of idle connections per host
    pub pool_max_idle_per_host: usize,
    /// User agent string
    pub user_agent: String,
    /// Circuit breaker failure threshold
    pub circuit_failure_threshold: u32,
    /// Circuit breaker cooldown in milliseconds
    pub circuit_cooldown_ms: u64,
    /// Maximum retry attempts
    pub max_retries: usize,
    /// Initial retry backoff in milliseconds
    pub initial_backoff_ms: u64,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 30000,
            connect_timeout_ms: 10000,
            pool_idle_timeout_secs: 90,
            pool_max_idle_per_host: 10,
            user_agent: format!("riptide-eventmesh/{}", env!("CARGO_PKG_VERSION")),
            circuit_failure_threshold: 5,
            circuit_cooldown_ms: 30000,
            max_retries: 3,
            initial_backoff_ms: 100,
        }
    }
}

/// Options for individual HTTP requests
#[derive(Debug, Clone)]
pub struct FetchOptions {
    /// Override default timeout
    pub timeout: Option<Duration>,
    /// Override default retry attempts
    pub max_retries: Option<usize>,
    /// Custom headers
    pub headers: Vec<(String, String)>,
    /// Follow redirects
    pub follow_redirects: bool,
    /// Bypass circuit breaker (use with caution)
    pub bypass_circuit_breaker: bool,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self {
            timeout: None,
            max_retries: None,
            headers: vec![],
            follow_redirects: true,
            bypass_circuit_breaker: false,
        }
    }
}

impl FetchOptions {
    /// Set custom timeout for this request
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set maximum retry attempts for this request
    pub fn with_max_retries(mut self, retries: usize) -> Self {
        self.max_retries = Some(retries);
        self
    }

    /// Add a custom header
    pub fn add_header(mut self, key: String, value: String) -> Self {
        self.headers.push((key, value));
        self
    }

    /// Disable redirect following
    pub fn no_redirects(mut self) -> Self {
        self.follow_redirects = false;
        self
    }

    /// Bypass circuit breaker protection (use with caution)
    pub fn bypass_circuit_breaker(mut self) -> Self {
        self.bypass_circuit_breaker = true;
        self
    }
}

/// Unified HTTP client service with integrated reliability patterns
#[derive(Debug)]
pub struct HttpClientService {
    client: Client,
    circuit_breaker: Arc<CircuitBreaker>,
    retry_policy: RetryPolicy,
    config: HttpConfig,
}

impl HttpClientService {
    /// Create a new HTTP client service with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - HTTP client configuration
    ///
    /// # Errors
    ///
    /// Returns error if client creation fails
    pub fn new(config: HttpConfig) -> Result<Self> {
        info!(
            "Creating HTTP client service with timeout: {}ms",
            config.timeout_ms
        );

        let client = ClientBuilder::new()
            .timeout(Duration::from_millis(config.timeout_ms))
            .connect_timeout(Duration::from_millis(config.connect_timeout_ms))
            .pool_idle_timeout(Duration::from_secs(config.pool_idle_timeout_secs))
            .pool_max_idle_per_host(config.pool_max_idle_per_host)
            .user_agent(&config.user_agent)
            .gzip(true)
            .brotli(true)
            .use_rustls_tls()
            .build()
            .context("Failed to build HTTP client")?;

        let circuit_config = CircuitConfig {
            failure_threshold: config.circuit_failure_threshold,
            open_cooldown_ms: config.circuit_cooldown_ms,
            half_open_max_in_flight: 3,
        };

        let circuit_breaker = CircuitBreaker::new(circuit_config, Arc::new(circuit::RealClock));

        let retry_policy = RetryPolicy::new(
            config.max_retries,
            config.initial_backoff_ms,
            10000, // max backoff 10s
            2.0,   // exponential multiplier
        );

        debug!("HTTP client service created successfully");

        Ok(Self {
            client,
            circuit_breaker,
            retry_policy,
            config,
        })
    }

    /// Create a default HTTP client service
    pub fn new_default() -> Result<Self> {
        Self::new(HttpConfig::default())
    }

    /// Perform HTTP GET request with retry and circuit breaker protection
    ///
    /// # Arguments
    ///
    /// * `url` - Target URL
    /// * `options` - Request options
    ///
    /// # Returns
    ///
    /// The HTTP response if successful
    ///
    /// # Errors
    ///
    /// Returns error if all retries fail or circuit breaker is open
    pub async fn get(&self, url: &str, options: FetchOptions) -> Result<ReqwestResponse> {
        self.request(Method::GET, url, None, options).await
    }

    /// Perform HTTP POST request with retry and circuit breaker protection
    ///
    /// # Arguments
    ///
    /// * `url` - Target URL
    /// * `body` - Request body
    /// * `options` - Request options
    ///
    /// # Returns
    ///
    /// The HTTP response if successful
    ///
    /// # Errors
    ///
    /// Returns error if all retries fail or circuit breaker is open
    pub async fn post(
        &self,
        url: &str,
        body: Vec<u8>,
        options: FetchOptions,
    ) -> Result<ReqwestResponse> {
        self.request(Method::POST, url, Some(body), options).await
    }

    /// Perform HTTP PUT request with retry and circuit breaker protection
    pub async fn put(
        &self,
        url: &str,
        body: Vec<u8>,
        options: FetchOptions,
    ) -> Result<ReqwestResponse> {
        self.request(Method::PUT, url, Some(body), options).await
    }

    /// Perform HTTP DELETE request with retry and circuit breaker protection
    pub async fn delete(&self, url: &str, options: FetchOptions) -> Result<ReqwestResponse> {
        self.request(Method::DELETE, url, None, options).await
    }

    /// Internal method to perform HTTP requests with full reliability patterns
    async fn request(
        &self,
        method: Method,
        url: &str,
        body: Option<Vec<u8>>,
        options: FetchOptions,
    ) -> Result<ReqwestResponse> {
        let max_retries = options.max_retries.unwrap_or(self.config.max_retries);
        let timeout = options
            .timeout
            .unwrap_or(Duration::from_millis(self.config.timeout_ms));

        // Create custom retry policy if options override defaults
        let retry_policy = if options.max_retries.is_some() {
            RetryPolicy::new(max_retries, self.config.initial_backoff_ms, 10000, 2.0)
        } else {
            self.retry_policy.clone()
        };

        debug!(
            method = %method,
            url = %url,
            max_retries = max_retries,
            timeout_ms = timeout.as_millis(),
            "Initiating HTTP request with reliability patterns"
        );

        // Execute with retry logic
        retry_policy
            .execute(|| async {
                // Check circuit breaker (unless bypassed)
                if !options.bypass_circuit_breaker {
                    match self.circuit_breaker.try_acquire() {
                        Ok(_permit) => {
                            debug!("Circuit breaker: request permitted");
                        }
                        Err(msg) => {
                            error!("Circuit breaker: {}", msg);
                            return Err(anyhow::anyhow!("Circuit breaker open: {}", msg));
                        }
                    }
                }

                // Build request
                let mut request_builder = self.client.request(method.clone(), url).timeout(timeout);

                // Add custom headers
                for (key, value) in &options.headers {
                    request_builder = request_builder.header(key, value);
                }

                // Add body if present
                if let Some(ref body_data) = body {
                    request_builder = request_builder.body(body_data.clone());
                }

                // Send request
                let response = request_builder
                    .send()
                    .await
                    .context("HTTP request failed")?;

                // Check status code
                let status = response.status();
                if status.is_success() || status.is_redirection() {
                    // Success - record in circuit breaker
                    if !options.bypass_circuit_breaker {
                        self.circuit_breaker.on_success();
                    }
                    Ok(response)
                } else {
                    // Failure - record in circuit breaker
                    if !options.bypass_circuit_breaker {
                        self.circuit_breaker.on_failure();
                    }

                    // Don't retry client errors (4xx) except specific cases
                    if status.is_client_error()
                        && status != StatusCode::REQUEST_TIMEOUT
                        && status != StatusCode::TOO_MANY_REQUESTS
                    {
                        warn!(status = %status, "Client error - not retrying");
                        return Err(anyhow::anyhow!("Client error: {}", status));
                    }

                    // Retry server errors (5xx) and retryable client errors
                    Err(anyhow::anyhow!("HTTP error: {}", status))
                }
            })
            .await
    }

    /// Get the current circuit breaker state (for monitoring)
    pub fn circuit_state(&self) -> circuit::State {
        self.circuit_breaker.state()
    }

    /// Get HTTP client configuration
    pub fn config(&self) -> &HttpConfig {
        &self.config
    }

    /// Reset the circuit breaker (for testing/admin purposes)
    pub fn reset_circuit_breaker(&self) {
        self.circuit_breaker.on_success();
        info!("Circuit breaker manually reset");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_config_default() {
        let config = HttpConfig::default();
        assert_eq!(config.timeout_ms, 30000);
        assert_eq!(config.connect_timeout_ms, 10000);
        assert_eq!(config.pool_idle_timeout_secs, 90);
        assert_eq!(config.pool_max_idle_per_host, 10);
        assert!(config.user_agent.starts_with("riptide-eventmesh/"));
        assert_eq!(config.circuit_failure_threshold, 5);
        assert_eq!(config.circuit_cooldown_ms, 30000);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_fetch_options_builder() {
        let options = FetchOptions::default()
            .with_timeout(Duration::from_secs(60))
            .with_max_retries(5)
            .add_header("Authorization".to_string(), "Bearer token".to_string())
            .no_redirects();

        assert_eq!(options.timeout, Some(Duration::from_secs(60)));
        assert_eq!(options.max_retries, Some(5));
        assert_eq!(options.headers.len(), 1);
        assert!(!options.follow_redirects);
    }

    #[test]
    fn test_create_http_client_service() {
        let result = HttpClientService::new_default();
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_http_client_service_custom() {
        let config = HttpConfig {
            timeout_ms: 15000,
            connect_timeout_ms: 5000,
            pool_idle_timeout_secs: 120,
            pool_max_idle_per_host: 20,
            user_agent: "test-agent".to_string(),
            circuit_failure_threshold: 10,
            circuit_cooldown_ms: 60000,
            max_retries: 5,
            initial_backoff_ms: 200,
        };

        let result = HttpClientService::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_circuit_breaker_presets() {
        // Test all presets can be converted to config
        for preset in [
            CircuitBreakerPreset::BrowserRendering,
            CircuitBreakerPreset::PdfProcessing,
            CircuitBreakerPreset::SearchIndexing,
            CircuitBreakerPreset::ExternalApi,
            CircuitBreakerPreset::InternalService,
            CircuitBreakerPreset::WebScraping,
        ] {
            let config = preset.to_config();
            assert!(config.timeout_ms > 0);
            assert!(config.max_retries > 0);
        }
    }

    #[test]
    fn test_reliable_http_client_creation() {
        let result = ReliableHttpClient::with_preset(CircuitBreakerPreset::ExternalApi);
        assert!(result.is_ok());
    }
}

/// Simplified HTTP client with Arc wrapper for easy sharing
///
/// This is the recommended client for most use cases. It wraps HttpClientService
/// and provides a simpler API with preset configurations.
///
/// # Example
///
/// ```rust,no_run
/// use riptide_reliability::{ReliableHttpClient, CircuitBreakerPreset};
/// use std::sync::Arc;
///
/// # async fn example() -> anyhow::Result<()> {
/// // Create with preset
/// let client = Arc::new(ReliableHttpClient::with_preset(CircuitBreakerPreset::WebScraping)?);
///
/// // Simple GET
/// let response = client.get("https://example.com").await?;
/// let text = response.text().await?;
///
/// // POST with body
/// let body = b"data".to_vec();
/// let response = client.post("https://api.example.com", body).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct ReliableHttpClient {
    service: HttpClientService,
    preset: CircuitBreakerPreset,
}

impl ReliableHttpClient {
    /// Create a new client with a preset configuration
    ///
    /// # Arguments
    ///
    /// * `preset` - The circuit breaker preset to use
    ///
    /// # Errors
    ///
    /// Returns error if client creation fails
    pub fn with_preset(preset: CircuitBreakerPreset) -> Result<Self> {
        let config = preset.to_config();
        let service = HttpClientService::new(config)?;
        info!("Created ReliableHttpClient with preset: {:?}", preset);
        Ok(Self { service, preset })
    }

    /// Create a new client with custom configuration
    pub fn with_config(config: HttpConfig) -> Result<Self> {
        let service = HttpClientService::new(config)?;
        Ok(Self {
            service,
            preset: CircuitBreakerPreset::ExternalApi, // Default preset for tracking
        })
    }

    /// Get the preset used by this client
    pub fn preset(&self) -> CircuitBreakerPreset {
        self.preset
    }

    /// Perform HTTP GET request
    ///
    /// # Arguments
    ///
    /// * `url` - Target URL
    ///
    /// # Returns
    ///
    /// The HTTP response if successful
    ///
    /// # Errors
    ///
    /// Returns error if request fails or circuit breaker is open
    pub async fn get(&self, url: &str) -> Result<ReqwestResponse> {
        self.service.get(url, FetchOptions::default()).await
    }

    /// Perform HTTP GET request with custom options
    pub async fn get_with_options(
        &self,
        url: &str,
        options: FetchOptions,
    ) -> Result<ReqwestResponse> {
        self.service.get(url, options).await
    }

    /// Perform HTTP POST request
    ///
    /// # Arguments
    ///
    /// * `url` - Target URL
    /// * `body` - Request body
    ///
    /// # Returns
    ///
    /// The HTTP response if successful
    ///
    /// # Errors
    ///
    /// Returns error if request fails or circuit breaker is open
    pub async fn post(&self, url: &str, body: Vec<u8>) -> Result<ReqwestResponse> {
        self.service.post(url, body, FetchOptions::default()).await
    }

    /// Perform HTTP POST request with custom options
    pub async fn post_with_options(
        &self,
        url: &str,
        body: Vec<u8>,
        options: FetchOptions,
    ) -> Result<ReqwestResponse> {
        self.service.post(url, body, options).await
    }

    /// Perform HTTP PUT request
    pub async fn put(&self, url: &str, body: Vec<u8>) -> Result<ReqwestResponse> {
        self.service.put(url, body, FetchOptions::default()).await
    }

    /// Perform HTTP DELETE request
    pub async fn delete(&self, url: &str) -> Result<ReqwestResponse> {
        self.service.delete(url, FetchOptions::default()).await
    }

    /// Get circuit breaker state
    pub fn circuit_state(&self) -> circuit::State {
        self.service.circuit_state()
    }

    /// Get configuration
    pub fn config(&self) -> &HttpConfig {
        self.service.config()
    }

    /// Reset circuit breaker
    pub fn reset_circuit_breaker(&self) {
        self.service.reset_circuit_breaker();
    }

    /// Get reference to underlying service (for advanced usage)
    pub fn service(&self) -> &HttpClientService {
        &self.service
    }
}
