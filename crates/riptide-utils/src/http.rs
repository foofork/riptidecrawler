//! HTTP client factory with connection pooling and timeout configuration

use reqwest::{Client, ClientBuilder};
use std::time::Duration;
use tracing::{debug, info};

/// Configuration for HTTP client
#[derive(Debug, Clone)]
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
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 30000,
            connect_timeout_ms: 10000,
            pool_idle_timeout_secs: 90,
            pool_max_idle_per_host: 10,
            user_agent: format!("riptide-eventmesh/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

/// HTTP client factory
pub struct HttpClientFactory;

impl HttpClientFactory {
    /// Creates a new HTTP client with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - HTTP client configuration
    ///
    /// # Errors
    ///
    /// Returns error if client creation fails
    pub fn create(config: HttpConfig) -> Result<Client, reqwest::Error> {
        info!("Creating HTTP client with timeout: {}ms", config.timeout_ms);

        let client = ClientBuilder::new()
            .timeout(Duration::from_millis(config.timeout_ms))
            .connect_timeout(Duration::from_millis(config.connect_timeout_ms))
            .pool_idle_timeout(Duration::from_secs(config.pool_idle_timeout_secs))
            .pool_max_idle_per_host(config.pool_max_idle_per_host)
            .user_agent(config.user_agent)
            .use_rustls_tls()
            .build()?;

        debug!("HTTP client created successfully");

        Ok(client)
    }

    /// Creates a default HTTP client
    pub fn create_default() -> Result<Client, reqwest::Error> {
        Self::create(HttpConfig::default())
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
    }

    #[test]
    fn test_http_config_custom() {
        let config = HttpConfig {
            timeout_ms: 5000,
            connect_timeout_ms: 2000,
            pool_idle_timeout_secs: 60,
            pool_max_idle_per_host: 5,
            user_agent: "custom-agent".to_string(),
        };

        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.connect_timeout_ms, 2000);
        assert_eq!(config.pool_idle_timeout_secs, 60);
        assert_eq!(config.pool_max_idle_per_host, 5);
        assert_eq!(config.user_agent, "custom-agent");
    }

    #[test]
    fn test_create_http_client_default() {
        let result = HttpClientFactory::create_default();
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_http_client_custom() {
        let config = HttpConfig {
            timeout_ms: 15000,
            connect_timeout_ms: 5000,
            pool_idle_timeout_secs: 120,
            pool_max_idle_per_host: 20,
            user_agent: "test-agent".to_string(),
        };

        let result = HttpClientFactory::create(config);
        assert!(result.is_ok());
    }
}
