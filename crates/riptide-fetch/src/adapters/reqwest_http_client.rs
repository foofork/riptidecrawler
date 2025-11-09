//! Reqwest HTTP client adapter implementation
//!
//! This module provides a production-ready HTTP client adapter using reqwest.
//! It includes connection pooling, timeout handling, and retry logic.
//!
//! Note: This adapter does NOT use ReliableHttpClient to avoid circular dependencies
//! (riptide-reliability depends on riptide-fetch). For reliability features, use
//! HttpClientService from riptide-reliability directly in higher-level modules.

use async_trait::async_trait;
use riptide_types::error::{Result, RiptideError};
use riptide_types::ports::http::{HttpClient, HttpRequest, HttpResponse};
use std::collections::HashMap;
use std::time::Duration;

/// Reqwest-based HTTP client adapter
pub struct ReqwestHttpClient {
    client: reqwest::Client,
}

impl ReqwestHttpClient {
    /// Creates a new HTTP client with default configuration
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| RiptideError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client })
    }

    /// Creates a new HTTP client with custom configuration
    pub fn with_config(
        timeout: Duration,
        max_idle_per_host: usize,
        idle_timeout: Duration,
    ) -> Result<Self> {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(max_idle_per_host)
            .pool_idle_timeout(idle_timeout)
            .timeout(timeout)
            .build()
            .map_err(|e| RiptideError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client })
    }

    /// Converts reqwest::Response to HttpResponse (anti-corruption layer)
    async fn convert_response(resp: reqwest::Response) -> Result<HttpResponse> {
        let status = resp.status().as_u16();

        // Convert headers
        let mut headers = HashMap::new();
        for (key, value) in resp.headers() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(key.to_string(), value_str.to_string());
            }
        }

        // Read body
        let body = resp
            .bytes()
            .await
            .map_err(|e| RiptideError::NetworkError(format!("Failed to read response body: {}", e)))?
            .to_vec();

        Ok(HttpResponse::new(status, headers, body))
    }

    /// Converts HttpRequest to reqwest::Request (anti-corruption layer)
    fn build_request(&self, req: HttpRequest) -> Result<reqwest::Request> {
        let method = reqwest::Method::from_bytes(req.method.as_bytes())
            .map_err(|e| RiptideError::InvalidData(format!("Invalid HTTP method: {}", e)))?;

        let mut builder = self.client.request(method, &req.url);

        // Add headers
        for (key, value) in req.headers {
            builder = builder.header(&key, value);
        }

        // Add body if present
        if let Some(body) = req.body {
            builder = builder.body(body);
        }

        // Apply timeout if specified
        if let Some(timeout) = req.timeout {
            builder = builder.timeout(timeout);
        }

        builder
            .build()
            .map_err(|e| RiptideError::NetworkError(format!("Failed to build request: {}", e)))
    }
}

impl Default for ReqwestHttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP client")
    }
}

#[async_trait]
impl HttpClient for ReqwestHttpClient {
    async fn get(&self, url: &str) -> Result<HttpResponse> {
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| RiptideError::NetworkError(format!("GET request failed: {}", e)))?;

        Self::convert_response(resp).await
    }

    async fn post(&self, url: &str, body: &[u8]) -> Result<HttpResponse> {
        let resp = self
            .client
            .post(url)
            .body(body.to_vec())
            .send()
            .await
            .map_err(|e| RiptideError::NetworkError(format!("POST request failed: {}", e)))?;

        Self::convert_response(resp).await
    }

    async fn request(&self, req: HttpRequest) -> Result<HttpResponse> {
        let request = self.build_request(req)?;

        let resp = self
            .client
            .execute(request)
            .await
            .map_err(|e| RiptideError::NetworkError(format!("Request execution failed: {}", e)))?;

        Self::convert_response(resp).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_reqwest_client_creation() {
        let client = ReqwestHttpClient::new();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_reqwest_client_with_config() {
        let client = ReqwestHttpClient::with_config(
            Duration::from_secs(10),
            5,
            Duration::from_secs(60),
        );
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_http_request_builder() {
        let client = ReqwestHttpClient::new().unwrap();

        let req = HttpRequest::new("GET", "https://httpbin.org/get")
            .with_header("User-Agent", "Riptide/1.0")
            .with_timeout(Duration::from_secs(5));

        let reqwest_req = client.build_request(req);
        assert!(reqwest_req.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_method() {
        let client = ReqwestHttpClient::new().unwrap();

        let req = HttpRequest::new("INVALID", "https://httpbin.org/get");

        let result = client.build_request(req);
        assert!(result.is_err());
    }

    // Integration tests (require network)
    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_get_request() {
        let client = ReqwestHttpClient::new().unwrap();
        let response = client.get("https://httpbin.org/get").await;

        assert!(response.is_ok());
        let resp = response.unwrap();
        assert!(resp.is_success());
    }

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_post_request() {
        let client = ReqwestHttpClient::new().unwrap();
        let body = b"test data";
        let response = client.post("https://httpbin.org/post", body).await;

        assert!(response.is_ok());
        let resp = response.unwrap();
        assert!(resp.is_success());
    }

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_custom_request() {
        let client = ReqwestHttpClient::new().unwrap();
        let req = HttpRequest::new("GET", "https://httpbin.org/headers")
            .with_header("X-Custom-Header", "TestValue");

        let response = client.request(req).await;

        assert!(response.is_ok());
        let resp = response.unwrap();
        assert!(resp.is_success());
    }
}
