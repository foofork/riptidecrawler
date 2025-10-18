//! Configuration types for Riptide facade.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for Riptide facade components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiptideConfig {
    /// User agent string for HTTP requests
    pub user_agent: String,

    /// Request timeout duration
    pub timeout: Duration,

    /// Maximum number of redirects to follow
    pub max_redirects: u32,

    /// Whether to verify SSL certificates
    pub verify_ssl: bool,

    /// Additional headers to include in requests
    pub headers: Vec<(String, String)>,

    /// Maximum response body size in bytes
    pub max_body_size: usize,
}

impl Default for RiptideConfig {
    fn default() -> Self {
        Self {
            user_agent: "RiptideFacade/0.1.0".to_string(),
            timeout: Duration::from_secs(30),
            max_redirects: 5,
            verify_ssl: true,
            headers: Vec::new(),
            max_body_size: 10 * 1024 * 1024, // 10 MB
        }
    }
}

impl RiptideConfig {
    /// Create a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the user agent string.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Set the request timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the maximum number of redirects.
    pub fn with_max_redirects(mut self, max_redirects: u32) -> Self {
        self.max_redirects = max_redirects;
        self
    }

    /// Set whether to verify SSL certificates.
    pub fn with_verify_ssl(mut self, verify_ssl: bool) -> Self {
        self.verify_ssl = verify_ssl;
        self
    }

    /// Add a custom header.
    pub fn add_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    /// Set the maximum response body size.
    pub fn with_max_body_size(mut self, max_body_size: usize) -> Self {
        self.max_body_size = max_body_size;
        self
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.user_agent.is_empty() {
            return Err("User agent cannot be empty".to_string());
        }
        if self.timeout.as_secs() == 0 {
            return Err("Timeout must be greater than zero".to_string());
        }
        if self.max_body_size == 0 {
            return Err("Max body size must be greater than zero".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RiptideConfig::default();
        assert_eq!(config.user_agent, "RiptideFacade/0.1.0");
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_redirects, 5);
        assert!(config.verify_ssl);
        assert!(config.headers.is_empty());
    }

    #[test]
    fn test_config_builder() {
        let config = RiptideConfig::new()
            .with_user_agent("TestBot/1.0")
            .with_timeout(Duration::from_secs(60))
            .with_max_redirects(10)
            .with_verify_ssl(false)
            .add_header("X-Custom", "value");

        assert_eq!(config.user_agent, "TestBot/1.0");
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.max_redirects, 10);
        assert!(!config.verify_ssl);
        assert_eq!(config.headers.len(), 1);
    }

    #[test]
    fn test_config_validation() {
        let config = RiptideConfig::default();
        assert!(config.validate().is_ok());

        let invalid_config = RiptideConfig {
            user_agent: String::new(),
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }
}
