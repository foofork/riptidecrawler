//! Basic web scraper facade implementation.

use crate::{config::RiptideConfig, error::RiptideResult, RiptideError};
use riptide_fetch::FetchEngine;
use std::sync::Arc;
use url::Url;

/// A simplified facade for web scraping operations.
///
/// Provides high-level methods for common scraping tasks while hiding
/// the complexity of the underlying fetch and extraction layers.
#[derive(Clone)]
pub struct ScraperFacade {
    config: Arc<RiptideConfig>,
    client: Arc<FetchEngine>,
}

impl ScraperFacade {
    /// Create a new scraper facade with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the fetch client cannot be initialized.
    pub async fn new(config: RiptideConfig) -> RiptideResult<Self> {
        let client = FetchEngine::new()
            .map_err(|e| RiptideError::config(format!("Failed to create fetch engine: {}", e)))?;

        Ok(Self {
            config: Arc::new(config),
            client: Arc::new(client),
        })
    }

    /// Fetch HTML content from a URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch
    ///
    /// # Returns
    ///
    /// Returns the HTML content as a string.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The URL is invalid
    /// - The request fails
    /// - The response is not valid UTF-8
    ///
    /// # Example
    ///
    /// ```no_run
    /// use riptide_facade::ScraperFacade;
    /// use riptide_facade::RiptideConfig;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = RiptideConfig::default();
    /// let scraper = ScraperFacade::new(config).await?;
    /// let html = scraper.fetch_html("https://example.com").await?;
    /// println!("Fetched {} bytes", html.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_html(&self, url: impl AsRef<str>) -> RiptideResult<String> {
        let url_str = url.as_ref();

        // Validate URL
        let _ = Url::parse(url_str)?;

        // Fetch as text with timeout enforcement
        self.client
            .fetch_text(url_str)
            .await
            .map_err(|e| RiptideError::extraction(format!("Failed to fetch HTML: {}", e)))
    }

    /// Fetch raw bytes from a URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch
    ///
    /// # Returns
    ///
    /// Returns the response body as bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL is invalid or the request fails.
    pub async fn fetch_bytes(&self, url: impl AsRef<str>) -> RiptideResult<Vec<u8>> {
        let url_str = url.as_ref();

        // Validate URL
        let _ = Url::parse(url_str)?;

        // Fetch as bytes with timeout enforcement
        self.client
            .fetch_bytes(url_str)
            .await
            .map_err(|e| RiptideError::extraction(format!("Failed to fetch bytes: {}", e)))
    }

    /// Get the current configuration.
    pub fn config(&self) -> &RiptideConfig {
        &self.config
    }

    /// Get a reference to the fetch engine.
    pub fn client(&self) -> &FetchEngine {
        &self.client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scraper_creation() {
        let config = RiptideConfig::default();
        let scraper = ScraperFacade::new(config).await;
        assert!(scraper.is_ok());
    }

    #[tokio::test]
    async fn test_scraper_config_access() {
        let config = RiptideConfig::default().with_user_agent("TestBot/1.0");
        let scraper = ScraperFacade::new(config).await.unwrap();
        assert_eq!(scraper.config().user_agent, "TestBot/1.0");
    }

    #[test]
    fn test_invalid_url() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let config = RiptideConfig::default();
            let scraper = ScraperFacade::new(config).await.unwrap();
            let result = scraper.fetch_html("not a valid url").await;
            assert!(result.is_err());
        });
    }
}
