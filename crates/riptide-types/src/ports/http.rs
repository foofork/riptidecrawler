//! HTTP client port definition
//!
//! This module defines the abstract HTTP client interface that adapters must implement.
//! It provides a technology-agnostic way to perform HTTP operations.

use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;

use crate::error::Result;

/// HTTP request structure
#[derive(Debug, Clone)]
pub struct HttpRequest {
    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    pub method: String,
    /// Target URL
    pub url: String,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Optional request body
    pub body: Option<Vec<u8>>,
    /// Optional timeout duration
    pub timeout: Option<Duration>,
}

impl HttpRequest {
    /// Creates a new HTTP request
    pub fn new(method: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            url: url.into(),
            headers: HashMap::new(),
            body: None,
            timeout: None,
        }
    }

    /// Adds a header to the request
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Sets the request body
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    /// Sets the request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// HTTP response structure
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// HTTP status code
    pub status: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Creates a new HTTP response
    pub fn new(status: u16, headers: HashMap<String, String>, body: Vec<u8>) -> Self {
        Self {
            status,
            headers,
            body,
        }
    }

    /// Checks if the response was successful (2xx status code)
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Gets the response body as a UTF-8 string
    pub fn text(&self) -> Result<String> {
        String::from_utf8(self.body.clone())
            .map_err(|e| crate::error::RiptideError::Parse(format!("Invalid UTF-8: {}", e)))
    }

    /// Gets a header value by name (case-insensitive)
    pub fn header(&self, name: &str) -> Option<&String> {
        let name_lower = name.to_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == name_lower)
            .map(|(_, v)| v)
    }
}

/// HTTP client port interface
///
/// This trait defines the contract for HTTP client adapters.
/// Implementations should handle connection pooling, retries, and error handling.
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// Performs a GET request
    async fn get(&self, url: &str) -> Result<HttpResponse>;

    /// Performs a POST request with a body
    async fn post(&self, url: &str, body: &[u8]) -> Result<HttpResponse>;

    /// Performs a custom HTTP request
    async fn request(&self, req: HttpRequest) -> Result<HttpResponse>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_request_builder() {
        let req = HttpRequest::new("GET", "https://example.com")
            .with_header("User-Agent", "Riptide/1.0")
            .with_timeout(Duration::from_secs(30));

        assert_eq!(req.method, "GET");
        assert_eq!(req.url, "https://example.com");
        assert_eq!(req.headers.get("User-Agent").unwrap(), "Riptide/1.0");
        assert_eq!(req.timeout, Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_http_response_success() {
        let resp = HttpResponse::new(200, HashMap::new(), vec![]);
        assert!(resp.is_success());

        let resp = HttpResponse::new(404, HashMap::new(), vec![]);
        assert!(!resp.is_success());
    }

    #[test]
    fn test_http_response_text() {
        let body = b"Hello, World!".to_vec();
        let resp = HttpResponse::new(200, HashMap::new(), body);
        assert_eq!(resp.text().unwrap(), "Hello, World!");
    }

    #[test]
    fn test_http_response_header() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        let resp = HttpResponse::new(200, headers, vec![]);

        assert_eq!(resp.header("content-type").unwrap(), "application/json");
        assert_eq!(resp.header("Content-Type").unwrap(), "application/json");
        assert!(resp.header("Authorization").is_none());
    }
}
