//! Test data factories for creating test objects

use serde::{Deserialize, Serialize};

/// Builder for creating test extraction requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionRequestBuilder {
    pub url: String,
    pub format: String,
    pub include_metadata: bool,
    pub include_links: bool,
}

impl Default for ExtractionRequestBuilder {
    fn default() -> Self {
        Self {
            url: "https://example.com".to_string(),
            format: "markdown".to_string(),
            include_metadata: true,
            include_links: true,
        }
    }
}

impl ExtractionRequestBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the URL
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }

    /// Set the output format
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = format.into();
        self
    }

    /// Set metadata inclusion
    pub fn include_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    /// Set link inclusion
    pub fn include_links(mut self, include: bool) -> Self {
        self.include_links = include;
        self
    }

    /// Build as JSON
    pub fn build_json(&self) -> serde_json::Value {
        serde_json::json!({
            "url": self.url,
            "format": self.format,
            "include_metadata": self.include_metadata,
            "include_links": self.include_links,
        })
    }
}

/// Builder for creating test configuration
#[derive(Debug, Clone)]
pub struct TestConfigBuilder {
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub enable_caching: bool,
}

impl Default for TestConfigBuilder {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            max_retries: 3,
            enable_caching: true,
        }
    }
}

impl TestConfigBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set timeout
    pub fn timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Set max retries
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Set caching
    pub fn caching(mut self, enabled: bool) -> Self {
        self.enable_caching = enabled;
        self
    }
}

/// Factory for creating test data
pub struct Factory;

impl Factory {
    /// Create a test URL
    pub fn url() -> String {
        "https://test.example.com/page".to_string()
    }

    /// Create a test URL with path
    pub fn url_with_path(path: &str) -> String {
        format!("https://test.example.com{}", path)
    }

    /// Create test HTML content
    pub fn html_content() -> String {
        r#"
<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body><h1>Test Content</h1><p>Test paragraph</p></body>
</html>
        "#
        .to_string()
    }

    /// Create test JSON content
    pub fn json_content() -> serde_json::Value {
        serde_json::json!({
            "test": true,
            "data": {
                "key": "value"
            }
        })
    }

    /// Create a sequence of test URLs
    pub fn urls(count: usize) -> Vec<String> {
        (0..count)
            .map(|i| format!("https://test.example.com/page/{}", i))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_request_builder() {
        let request = ExtractionRequestBuilder::new()
            .url("https://test.com")
            .format("json")
            .include_metadata(false)
            .build_json();

        assert_eq!(request["url"], "https://test.com");
        assert_eq!(request["format"], "json");
        assert_eq!(request["include_metadata"], false);
    }

    #[test]
    fn test_test_config_builder() {
        let config = TestConfigBuilder::new()
            .timeout(60)
            .max_retries(5)
            .caching(false);

        assert_eq!(config.timeout_seconds, 60);
        assert_eq!(config.max_retries, 5);
        assert!(!config.enable_caching);
    }

    #[test]
    fn test_factory_methods() {
        assert!(Factory::url().starts_with("https://"));
        assert!(Factory::url_with_path("/test").ends_with("/test"));
        assert!(Factory::html_content().contains("<!DOCTYPE html>"));
        assert!(Factory::json_content()["test"].as_bool().unwrap());
        assert_eq!(Factory::urls(5).len(), 5);
    }
}
