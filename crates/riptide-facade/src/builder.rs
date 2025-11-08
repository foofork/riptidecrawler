//! Builder pattern implementation for Riptide facade.

use crate::{config::RiptideConfig, error::RiptideResult, facades::ScraperFacade, RiptideError};
use std::time::Duration;

/// Builder for creating Riptide facade instances.
///
/// Provides a fluent API for configuring and building different types of facades.
#[derive(Debug, Clone)]
pub struct RiptideBuilder {
    config: RiptideConfig,
}

impl Default for RiptideBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RiptideBuilder {
    /// Create a new builder with default configuration.
    pub fn new() -> Self {
        Self {
            config: RiptideConfig::default(),
        }
    }

    /// Set the user agent string.
    ///
    /// # Example
    ///
    /// ```
    /// use riptide_facade::RiptideBuilder;
    ///
    /// let builder = RiptideBuilder::new()
    ///     .user_agent("MyBot/1.0");
    /// ```
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.config.user_agent = user_agent.into();
        self
    }

    /// Set the request timeout in seconds.
    ///
    /// # Example
    ///
    /// ```
    /// use riptide_facade::RiptideBuilder;
    ///
    /// let builder = RiptideBuilder::new()
    ///     .timeout_secs(60);
    /// ```
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.config.timeout = Duration::from_secs(secs);
        self
    }

    /// Set the request timeout duration.
    ///
    /// # Example
    ///
    /// ```
    /// use riptide_facade::RiptideBuilder;
    /// use std::time::Duration;
    ///
    /// let builder = RiptideBuilder::new()
    ///     .timeout(Duration::from_secs(60));
    /// ```
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set the maximum number of redirects to follow.
    ///
    /// # Example
    ///
    /// ```
    /// use riptide_facade::RiptideBuilder;
    ///
    /// let builder = RiptideBuilder::new()
    ///     .max_redirects(10);
    /// ```
    pub fn max_redirects(mut self, max_redirects: u32) -> Self {
        self.config.max_redirects = max_redirects;
        self
    }

    /// Set whether to verify SSL certificates.
    ///
    /// # Example
    ///
    /// ```
    /// use riptide_facade::RiptideBuilder;
    ///
    /// let builder = RiptideBuilder::new()
    ///     .verify_ssl(false);
    /// ```
    pub fn verify_ssl(mut self, verify: bool) -> Self {
        self.config.verify_ssl = verify;
        self
    }

    /// Add a custom header to all requests.
    ///
    /// # Example
    ///
    /// ```
    /// use riptide_facade::RiptideBuilder;
    ///
    /// let builder = RiptideBuilder::new()
    ///     .header("X-API-Key", "secret");
    /// ```
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.metadata.insert(key.into(), value.into());
        self
    }

    /// Set the maximum response body size in bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use riptide_facade::RiptideBuilder;
    ///
    /// let builder = RiptideBuilder::new()
    ///     .max_body_size(5 * 1024 * 1024); // 5 MB
    /// ```
    pub fn max_body_size(mut self, size: usize) -> Self {
        self.config.max_body_size = size;
        self
    }

    /// Set the complete configuration.
    ///
    /// # Example
    ///
    /// ```
    /// use riptide_facade::{RiptideBuilder, RiptideConfig};
    ///
    /// let config = RiptideConfig::default();
    /// let builder = RiptideBuilder::new()
    ///     .config(config);
    /// ```
    pub fn config(mut self, config: RiptideConfig) -> Self {
        self.config = config;
        self
    }

    /// Build a scraper facade instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid or if initialization fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use riptide_facade::RiptideBuilder;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let scraper = RiptideBuilder::new()
    ///     .user_agent("MyBot/1.0")
    ///     .build_scraper()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn build_scraper(self) -> RiptideResult<ScraperFacade> {
        // Validate configuration
        self.config.validate().map_err(RiptideError::config)?;

        // Build the scraper facade
        ScraperFacade::new(self.config).await
    }

    /// Build a browser facade instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid or if browser initialization fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use riptide_facade::RiptideBuilder;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let browser = RiptideBuilder::new()
    ///     .user_agent("MyBot/1.0")
    ///     .build_browser()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn build_browser(self) -> RiptideResult<crate::facades::BrowserFacade> {
        // Validate configuration
        self.config.validate().map_err(RiptideError::config)?;

        // Build the browser facade
        crate::facades::BrowserFacade::new(self.config).await
    }

    /// Build an extractor facade instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid or if initialization fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use riptide_facade::RiptideBuilder;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let extractor = RiptideBuilder::new()
    ///     .build_extractor()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn build_extractor(self) -> RiptideResult<crate::facades::ExtractionFacade> {
        // Validate configuration
        self.config.validate().map_err(RiptideError::config)?;

        // Build the extractor facade
        crate::facades::ExtractionFacade::new(self.config).await
    }

    /// Get a reference to the current configuration.
    pub fn get_config(&self) -> &RiptideConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let builder = RiptideBuilder::new();
        let config = builder.get_config();
        assert_eq!(config.user_agent, "RiptideFacade/0.1.0");
        assert_eq!(config.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_builder_user_agent() {
        let builder = RiptideBuilder::new().user_agent("TestBot/2.0");
        assert_eq!(builder.get_config().user_agent, "TestBot/2.0");
    }

    #[test]
    fn test_builder_timeout() {
        let builder = RiptideBuilder::new().timeout_secs(60);
        assert_eq!(builder.get_config().timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_builder_timeout_duration() {
        let duration = Duration::from_millis(500);
        let builder = RiptideBuilder::new().timeout(duration);
        assert_eq!(builder.get_config().timeout, duration);
    }

    #[test]
    fn test_builder_max_redirects() {
        let builder = RiptideBuilder::new().max_redirects(10);
        assert_eq!(builder.get_config().max_redirects, 10);
    }

    #[test]
    fn test_builder_verify_ssl() {
        let builder = RiptideBuilder::new().verify_ssl(false);
        assert!(!builder.get_config().verify_ssl);
    }

    #[test]
    fn test_builder_header() {
        let builder = RiptideBuilder::new().header("X-Custom", "value");
        let metadata = &builder.get_config().metadata;
        assert_eq!(metadata.len(), 1);
        assert_eq!(metadata.get("X-Custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_builder_chaining() {
        let builder = RiptideBuilder::new()
            .user_agent("ChainBot/1.0")
            .timeout_secs(45)
            .max_redirects(7)
            .verify_ssl(false)
            .header("X-Test", "test")
            .max_body_size(2048);

        let config = builder.get_config();
        assert_eq!(config.user_agent, "ChainBot/1.0");
        assert_eq!(config.timeout, Duration::from_secs(45));
        assert_eq!(config.max_redirects, 7);
        assert!(!config.verify_ssl);
        assert_eq!(config.metadata.len(), 1);
        assert_eq!(config.max_body_size, 2048);
    }
}
