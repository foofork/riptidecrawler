use anyhow::{Context, Result};
use reqwest::{Client, ClientBuilder, Method, Response, StatusCode};
use serde::Serialize;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Maximum number of retry attempts for failed requests
const MAX_RETRIES: u32 = 3;

/// Initial backoff duration in milliseconds
const INITIAL_BACKOFF_MS: u64 = 100;

/// Maximum backoff duration in milliseconds
const MAX_BACKOFF_MS: u64 = 5000;

pub struct RipTideClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
    /// Whether the API server is available (cached status)
    is_available: Option<bool>,
}

#[allow(dead_code)]
impl RipTideClient {
    pub fn new(base_url: String, api_key: Option<String>) -> Result<Self> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(300))
            .connect_timeout(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(90))
            .http2_prior_knowledge()
            .build()
            .context("Failed to create HTTP client")?;

        info!(
            "Created RipTide client for {} with auth: {}",
            base_url,
            api_key.is_some()
        );

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            is_available: None,
        })
    }

    /// Check if the API server is available and responding
    pub async fn check_health(&mut self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);

        debug!("Checking API health at {}", url);

        match tokio::time::timeout(Duration::from_secs(5), self.client.get(&url).send()).await {
            Ok(Ok(response)) => {
                let is_healthy = response.status().is_success();
                self.is_available = Some(is_healthy);

                if is_healthy {
                    info!("API server is healthy at {}", self.base_url);
                } else {
                    warn!(
                        "API server unhealthy at {}: status {}",
                        self.base_url,
                        response.status()
                    );
                }

                Ok(is_healthy)
            }
            Ok(Err(e)) => {
                warn!("API health check failed: {}", e);
                self.is_available = Some(false);
                Ok(false)
            }
            Err(_) => {
                warn!("API health check timed out");
                self.is_available = Some(false);
                Ok(false)
            }
        }
    }

    /// Get cached availability status (doesn't perform health check)
    #[allow(dead_code)]
    pub fn is_available(&self) -> Option<bool> {
        self.is_available
    }

    /// Perform request with exponential backoff retry logic
    async fn request_with_retry<T: Serialize>(
        &self,
        method: Method,
        path: &str,
        body: Option<&T>,
    ) -> Result<Response> {
        let url = format!("{}{}", self.base_url, path);
        let mut retry_count = 0;
        let mut backoff_ms = INITIAL_BACKOFF_MS;

        loop {
            let mut request = self.client.request(method.clone(), &url);

            // Add authentication header (Bearer token)
            if let Some(api_key) = &self.api_key {
                request = request.header("Authorization", format!("Bearer {}", api_key));
            }

            // Add body if provided
            if let Some(body) = body {
                request = request.json(body);
            }

            debug!(
                "Sending {} request to {} (attempt {})",
                method,
                url,
                retry_count + 1
            );

            match request.send().await {
                Ok(response) => {
                    let status = response.status();

                    // Success cases
                    if status.is_success() {
                        debug!("Request successful: {} {}", method, url);
                        return Ok(response);
                    }

                    // Retry on specific error codes
                    if Self::should_retry(status) && retry_count < MAX_RETRIES {
                        warn!(
                            "Request failed with status {} (attempt {}/{}), retrying in {}ms",
                            status,
                            retry_count + 1,
                            MAX_RETRIES,
                            backoff_ms
                        );

                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;

                        // Exponential backoff with cap
                        backoff_ms = (backoff_ms * 2).min(MAX_BACKOFF_MS);
                        retry_count += 1;
                        continue;
                    }

                    // Non-retryable error or max retries exceeded
                    let error_body = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unable to read error response".to_string());

                    anyhow::bail!("API request failed with status {}: {}", status, error_body);
                }
                Err(e) => {
                    // Network errors are retryable
                    if retry_count < MAX_RETRIES {
                        warn!(
                            "Request failed with error: {} (attempt {}/{}), retrying in {}ms",
                            e,
                            retry_count + 1,
                            MAX_RETRIES,
                            backoff_ms
                        );

                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;

                        // Exponential backoff with cap
                        backoff_ms = (backoff_ms * 2).min(MAX_BACKOFF_MS);
                        retry_count += 1;
                        continue;
                    }

                    return Err(e).context(format!(
                        "Failed to send request to {} after {} retries",
                        url, MAX_RETRIES
                    ));
                }
            }
        }
    }

    /// Determine if a status code should trigger a retry
    fn should_retry(status: StatusCode) -> bool {
        matches!(
            status,
            StatusCode::REQUEST_TIMEOUT
                | StatusCode::TOO_MANY_REQUESTS
                | StatusCode::INTERNAL_SERVER_ERROR
                | StatusCode::BAD_GATEWAY
                | StatusCode::SERVICE_UNAVAILABLE
                | StatusCode::GATEWAY_TIMEOUT
        )
    }

    pub async fn get(&self, path: &str) -> Result<Response> {
        self.request_with_retry(Method::GET, path, None::<&()>)
            .await
    }

    pub async fn post<T: Serialize>(&self, path: &str, body: &T) -> Result<Response> {
        self.request_with_retry(Method::POST, path, Some(body))
            .await
    }

    pub async fn put<T: Serialize>(&self, path: &str, body: &T) -> Result<Response> {
        self.request_with_retry(Method::PUT, path, Some(body)).await
    }

    pub async fn delete(&self, path: &str) -> Result<Response> {
        self.request_with_retry(Method::DELETE, path, None::<&()>)
            .await
    }

    // Utility method for raw requests with optional body
    #[allow(dead_code)]
    async fn request_raw(
        &self,
        method: Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<Response> {
        if let Some(body_val) = body {
            self.request_with_retry(method, path, Some(&body_val)).await
        } else {
            self.request_with_retry(method, path, None::<&()>).await
        }
    }

    #[allow(dead_code)]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}
