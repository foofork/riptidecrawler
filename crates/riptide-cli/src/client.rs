use anyhow::{Context, Result};
use reqwest::{Client, Response};
use serde_json::Value;
use std::time::Duration;

/// Simple HTTP client wrapper for EventMesh API interactions
///
/// This is a thin wrapper around reqwest::Client that handles:
/// - Base URL management
/// - Authentication via Bearer token
/// - Standard timeout configuration
/// - JSON and streaming requests
pub struct ApiClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl ApiClient {
    /// Creates a new API client with the specified base URL and optional API key
    ///
    /// # Arguments
    /// * `base_url` - The base URL of the EventMesh API (e.g., "http://localhost:8080")
    /// * `api_key` - Optional API key for Bearer token authentication
    ///
    /// # Example
    /// ```no_run
    /// use riptide_cli::client::ApiClient;
    ///
    /// let client = ApiClient::new(
    ///     "http://localhost:8080".to_string(),
    ///     Some("my-api-key".to_string())
    /// ).unwrap();
    /// ```
    pub fn new(base_url: String, api_key: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .context("Failed to create HTTP client")?;

        // Ensure base_url doesn't end with a slash for consistent URL building
        let base_url = base_url.trim_end_matches('/').to_string();

        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }

    /// Builds the full URL by combining base URL with the given path
    ///
    /// Handles both paths with and without leading slashes
    fn build_url(&self, path: &str) -> String {
        let path = path.trim_start_matches('/');
        format!("{}/{}", self.base_url, path)
    }

    /// Performs a GET request to the specified path
    ///
    /// # Arguments
    /// * `path` - The API path to request (e.g., "/api/pools" or "api/pools")
    ///
    /// # Example
    /// ```no_run
    /// # use riptide_cli::client::ApiClient;
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = ApiClient::new("http://localhost:8080".to_string(), None)?;
    /// let response = client.get("/api/pools").await?;
    /// let pools: serde_json::Value = response.json().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, path: &str) -> Result<Response> {
        let url = self.build_url(path);
        let mut req = self.client.get(&url);

        if let Some(key) = &self.api_key {
            req = req.bearer_auth(key);
        }

        req.send()
            .await
            .context(format!("Failed to send GET request to {}", url))
    }

    /// Performs a POST request with JSON body to the specified path
    ///
    /// # Arguments
    /// * `path` - The API path to request (e.g., "/api/pools")
    /// * `body` - The JSON body to send
    ///
    /// # Example
    /// ```no_run
    /// # use riptide_cli::client::ApiClient;
    /// # use serde_json::json;
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = ApiClient::new("http://localhost:8080".to_string(), None)?;
    /// let body = json!({
    ///     "name": "test-pool",
    ///     "max_size": 10
    /// });
    /// let response = client.post_json("/api/pools", body).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn post_json(&self, path: &str, body: Value) -> Result<Response> {
        let url = self.build_url(path);
        let mut req = self.client.post(&url).json(&body);

        if let Some(key) = &self.api_key {
            req = req.bearer_auth(key);
        }

        req.send()
            .await
            .context(format!("Failed to send POST request to {}", url))
    }

    /// Performs a POST request with typed request/response bodies
    ///
    /// This is a type-safe wrapper that handles serialization and deserialization.
    ///
    /// # Arguments
    /// * `path` - The API path to request (e.g., "/api/extract")
    /// * `request` - The request body to serialize
    ///
    /// # Type Parameters
    /// * `Req` - The request type (must be Serialize)
    /// * `Res` - The response type (must be DeserializeOwned)
    ///
    /// # Example
    /// ```no_run
    /// # use riptide_cli::client::ApiClient;
    /// # use serde::{Serialize, Deserialize};
    /// # async fn example() -> anyhow::Result<()> {
    /// #[derive(Serialize)]
    /// struct ExtractRequest {
    ///     url: String,
    /// }
    ///
    /// #[derive(Deserialize)]
    /// struct ExtractResponse {
    ///     content: String,
    /// }
    ///
    /// let client = ApiClient::new("http://localhost:8080".to_string(), None)?;
    /// let request = ExtractRequest {
    ///     url: "https://example.com".to_string()
    /// };
    ///
    /// let response: ExtractResponse = client.post("/extract", &request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn post<Req, Res>(&self, path: &str, request: &Req) -> Result<Res>
    where
        Req: serde::Serialize,
        Res: serde::de::DeserializeOwned,
    {
        let url = self.build_url(path);
        let mut req = self.client.post(&url).json(request);

        if let Some(key) = &self.api_key {
            req = req.bearer_auth(key);
        }

        let response = req
            .send()
            .await
            .context(format!("Failed to send POST request to {}", url))?;

        // Check for HTTP errors
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| String::from("Unknown error"));
            anyhow::bail!("API returned error {}: {}", status, error_text);
        }

        // Deserialize response
        response
            .json::<Res>()
            .await
            .context("Failed to deserialize response JSON")
    }

    /// Performs a POST request for streaming responses (NDJSON)
    ///
    /// This method is used for endpoints that return newline-delimited JSON streams,
    /// such as the extract streaming endpoint.
    ///
    /// # Arguments
    /// * `path` - The API path to request (e.g., "/api/extract/stream")
    /// * `body` - The JSON body to send
    ///
    /// # Example
    /// ```no_run
    /// # use riptide_cli::client::ApiClient;
    /// # use serde_json::json;
    /// # async fn example() -> anyhow::Result<()> {
    /// use futures_util::StreamExt;
    ///
    /// let client = ApiClient::new("http://localhost:8080".to_string(), None)?;
    /// let body = json!({"url": "https://example.com"});
    /// let response = client.post_stream("/api/extract/stream", body).await?;
    ///
    /// // Process the streaming response line by line
    /// let mut stream = response.bytes_stream();
    /// while let Some(chunk) = stream.next().await {
    ///     let chunk = chunk?;
    ///     // Process chunk...
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn post_stream(&self, path: &str, body: Value) -> Result<Response> {
        let url = self.build_url(path);
        let mut req = self.client.post(&url).json(&body);

        if let Some(key) = &self.api_key {
            req = req.bearer_auth(key);
        }

        req.send()
            .await
            .context(format!("Failed to send POST stream request to {}", url))
    }

    /// Returns the base URL of this client
    #[allow(dead_code)]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

// ============================================================================
// Legacy RipTideClient - Maintained for backward compatibility during refactor
// ============================================================================
// TODO: Remove this once all commands are migrated to ApiClient

use reqwest::{ClientBuilder, Method, StatusCode};
use serde::Serialize;

/// Maximum number of retry attempts for failed requests
const MAX_RETRIES: u32 = 3;

/// Initial backoff duration in milliseconds
const INITIAL_BACKOFF_MS: u64 = 100;

/// Maximum backoff duration in milliseconds
const MAX_BACKOFF_MS: u64 = 5000;

/// Legacy client with retry logic and health checks
///
/// This client is maintained for backward compatibility during the CLI refactor.
/// New code should use `ApiClient` instead.
#[deprecated(note = "Use ApiClient instead for new code")]
pub struct RipTideClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
    is_available: Option<bool>,
}

#[allow(dead_code, deprecated)]
impl RipTideClient {
    pub fn new(base_url: String, api_key: Option<String>) -> Result<Self> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(300))
            .connect_timeout(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(90))
            .http2_prior_knowledge()
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            is_available: None,
        })
    }

    pub async fn check_health(&mut self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);

        match tokio::time::timeout(Duration::from_secs(5), self.client.get(&url).send()).await {
            Ok(Ok(response)) => {
                let is_healthy = response.status().is_success();
                self.is_available = Some(is_healthy);
                Ok(is_healthy)
            }
            Ok(Err(_)) => {
                self.is_available = Some(false);
                Ok(false)
            }
            Err(_) => {
                self.is_available = Some(false);
                Ok(false)
            }
        }
    }

    pub fn is_available(&self) -> Option<bool> {
        self.is_available
    }

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

            if let Some(api_key) = &self.api_key {
                request = request.header("Authorization", format!("Bearer {}", api_key));
            }

            if let Some(body) = body {
                request = request.json(body);
            }

            match request.send().await {
                Ok(response) => {
                    let status = response.status();

                    if status.is_success() {
                        return Ok(response);
                    }

                    if Self::should_retry(status) && retry_count < MAX_RETRIES {
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                        backoff_ms = (backoff_ms * 2).min(MAX_BACKOFF_MS);
                        retry_count += 1;
                        continue;
                    }

                    let error_body = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unable to read error response".to_string());

                    anyhow::bail!("API request failed with status {}: {}", status, error_body);
                }
                Err(e) => {
                    if retry_count < MAX_RETRIES {
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
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

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_client_without_auth() {
        let client = ApiClient::new("http://localhost:8080".to_string(), None);
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.base_url, "http://localhost:8080");
        assert!(client.api_key.is_none());
    }

    #[test]
    fn test_new_client_with_auth() {
        let client = ApiClient::new(
            "http://localhost:8080".to_string(),
            Some("test-key".to_string()),
        );
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.api_key, Some("test-key".to_string()));
    }

    #[test]
    fn test_build_url_with_leading_slash() {
        let client = ApiClient::new("http://localhost:8080".to_string(), None).unwrap();
        assert_eq!(
            client.build_url("/api/pools"),
            "http://localhost:8080/api/pools"
        );
    }

    #[test]
    fn test_build_url_without_leading_slash() {
        let client = ApiClient::new("http://localhost:8080".to_string(), None).unwrap();
        assert_eq!(
            client.build_url("api/pools"),
            "http://localhost:8080/api/pools"
        );
    }

    #[test]
    fn test_build_url_with_trailing_slash_in_base() {
        let client = ApiClient::new("http://localhost:8080/".to_string(), None).unwrap();
        assert_eq!(
            client.build_url("/api/pools"),
            "http://localhost:8080/api/pools"
        );
    }

    #[test]
    fn test_build_url_complex_path() {
        let client = ApiClient::new("http://localhost:8080".to_string(), None).unwrap();
        assert_eq!(
            client.build_url("/api/extract/stream"),
            "http://localhost:8080/api/extract/stream"
        );
    }

    #[test]
    fn test_base_url_accessor() {
        let client = ApiClient::new("http://localhost:8080".to_string(), None).unwrap();
        assert_eq!(client.base_url(), "http://localhost:8080");
    }
}
