//! Spider facade for web crawling operations.
//!
//! Provides a high-level interface for the riptide-spider crawling engine
//! with preset configurations and simplified API.

use anyhow::Result;
use riptide_spider::{config::SpiderPresets, CrawlState, PerformanceMetrics, Spider, SpiderConfig};
use std::sync::Arc;
use tokio::sync::Mutex;
use url::Url;

/// Preset spider configurations for common use cases.
#[derive(Debug, Clone, Copy)]
pub enum SpiderPreset {
    /// Development and testing configuration (low concurrency, limited pages)
    Development,
    /// High-performance crawling configuration (high concurrency, large limits)
    HighPerformance,
    /// News site crawling configuration (breadth-first, adaptive stopping)
    NewsSite,
    /// E-commerce site crawling configuration (best-first, product-focused)
    ECommerce,
    /// Documentation site crawling configuration (depth-first, hierarchical)
    Documentation,
    /// Authenticated crawling configuration (session management enabled)
    Authenticated,
}

/// High-level facade for web crawling operations.
///
/// Wraps the riptide-spider engine with a simplified interface and preset configurations.
#[derive(Clone)]
pub struct SpiderFacade {
    spider: Arc<Mutex<Spider>>,
}

impl SpiderFacade {
    /// Create a new spider from a preset configuration.
    ///
    /// # Arguments
    ///
    /// * `preset` - The preset configuration to use
    /// * `base_url` - The base URL for crawling
    ///
    /// # Returns
    ///
    /// Returns a configured `SpiderFacade` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The base URL is invalid
    /// - Spider initialization fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use riptide_facade::facades::spider::{SpiderFacade, SpiderPreset};
    /// use url::Url;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let base_url = Url::parse("https://example.com")?;
    /// let spider = SpiderFacade::from_preset(SpiderPreset::Development, base_url).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_preset(preset: SpiderPreset, base_url: Url) -> Result<Self> {
        let mut config = match preset {
            SpiderPreset::Development => SpiderPresets::development(),
            SpiderPreset::HighPerformance => SpiderPresets::high_performance(),
            SpiderPreset::NewsSite => SpiderPresets::news_site(),
            SpiderPreset::ECommerce => SpiderPresets::ecommerce_site(),
            SpiderPreset::Documentation => SpiderPresets::documentation_site(),
            SpiderPreset::Authenticated => SpiderPresets::authenticated_crawling(),
        };

        config.base_url = base_url;
        let spider = Spider::new(config).await?;

        Ok(Self {
            spider: Arc::new(Mutex::new(spider)),
        })
    }

    /// Create a new spider from a custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The spider configuration to use
    ///
    /// # Returns
    ///
    /// Returns a configured `SpiderFacade` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if spider initialization fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use riptide_facade::facades::spider::SpiderFacade;
    /// use riptide_spider::SpiderConfig;
    /// use url::Url;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = SpiderConfig::new(Url::parse("https://example.com")?);
    /// let spider = SpiderFacade::from_config(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_config(config: SpiderConfig) -> Result<Self> {
        let spider = Spider::new(config).await?;
        Ok(Self {
            spider: Arc::new(Mutex::new(spider)),
        })
    }

    /// Start crawling from seed URLs.
    ///
    /// # Arguments
    ///
    /// * `seeds` - Initial URLs to crawl from
    ///
    /// # Returns
    ///
    /// Returns a summary of the crawl operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the crawl operation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use riptide_facade::facades::spider::{SpiderFacade, SpiderPreset};
    /// use url::Url;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let base_url = Url::parse("https://example.com")?;
    /// let spider = SpiderFacade::from_preset(SpiderPreset::Development, base_url.clone()).await?;
    ///
    /// let seeds = vec![base_url];
    /// let summary = spider.crawl(seeds).await?;
    ///
    /// println!("Crawled {} pages in {:.2}s",
    ///          summary.pages_crawled,
    ///          summary.duration_secs);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn crawl(&self, seeds: Vec<Url>) -> Result<CrawlSummary> {
        let spider = self.spider.lock().await;
        let result = spider.crawl(seeds).await?;
        Ok(CrawlSummary::from(result))
    }

    /// Get the current crawl state.
    ///
    /// # Returns
    ///
    /// Returns the current state of the crawl operation.
    ///
    /// # Errors
    ///
    /// This method should not fail under normal circumstances.
    pub async fn get_state(&self) -> Result<CrawlState> {
        let spider = self.spider.lock().await;
        Ok(spider.get_crawl_state().await)
    }

    /// Get performance metrics for the crawl.
    ///
    /// # Returns
    ///
    /// Returns current performance metrics.
    ///
    /// # Errors
    ///
    /// This method should not fail under normal circumstances.
    pub async fn get_metrics(&self) -> Result<PerformanceMetrics> {
        let spider = self.spider.lock().await;
        Ok(spider.get_performance_metrics().await)
    }

    /// Stop the current crawl operation.
    ///
    /// # Errors
    ///
    /// This method should not fail under normal circumstances.
    pub async fn stop(&self) -> Result<()> {
        let spider = self.spider.lock().await;
        spider.stop().await;
        Ok(())
    }

    /// Reset the spider state and clear all caches.
    ///
    /// # Errors
    ///
    /// Returns an error if the reset operation fails.
    pub async fn reset(&self) -> Result<()> {
        let spider = self.spider.lock().await;
        spider.reset().await
    }
}

/// Summary of a completed crawl operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CrawlSummary {
    /// Total number of pages successfully crawled
    pub pages_crawled: u64,
    /// Total number of pages that failed to crawl
    pub pages_failed: u64,
    /// Duration of the crawl in seconds
    pub duration_secs: f64,
    /// Total bytes downloaded during the crawl
    pub bytes_downloaded: u64,
    /// Number of errors encountered
    pub errors_count: usize,
    /// Reason for stopping the crawl
    pub stop_reason: String,
    /// List of domains that were crawled
    pub domains: Vec<String>,
}

impl From<riptide_spider::SpiderResult> for CrawlSummary {
    fn from(result: riptide_spider::SpiderResult) -> Self {
        Self {
            pages_crawled: result.pages_crawled,
            pages_failed: result.pages_failed,
            duration_secs: result.duration.as_secs_f64(),
            bytes_downloaded: 0, // SpiderResult doesn't track this in current implementation
            errors_count: result.pages_failed as usize,
            stop_reason: result.stop_reason,
            domains: result.domains,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spider_facade_creation_from_preset() {
        let base_url = Url::parse("https://example.com").unwrap();
        let spider = SpiderFacade::from_preset(SpiderPreset::Development, base_url).await;
        assert!(spider.is_ok());
    }

    #[tokio::test]
    async fn test_spider_facade_creation_from_config() {
        let config = SpiderConfig::new(Url::parse("https://example.com").unwrap());
        let spider = SpiderFacade::from_config(config).await;
        assert!(spider.is_ok());
    }

    #[tokio::test]
    async fn test_spider_facade_state() {
        let base_url = Url::parse("https://example.com").unwrap();
        let spider = SpiderFacade::from_preset(SpiderPreset::Development, base_url)
            .await
            .unwrap();

        let state = spider.get_state().await.unwrap();
        assert!(!state.active); // Should not be active initially
        assert_eq!(state.pages_crawled, 0);
    }

    #[tokio::test]
    async fn test_spider_facade_metrics() {
        let base_url = Url::parse("https://example.com").unwrap();
        let spider = SpiderFacade::from_preset(SpiderPreset::Development, base_url)
            .await
            .unwrap();

        let metrics = spider.get_metrics().await.unwrap();
        assert_eq!(metrics.pages_per_second, 0.0); // Should be zero initially
    }

    #[tokio::test]
    async fn test_spider_facade_reset() {
        let base_url = Url::parse("https://example.com").unwrap();
        let spider = SpiderFacade::from_preset(SpiderPreset::Development, base_url)
            .await
            .unwrap();

        let result = spider.reset().await;
        assert!(result.is_ok());
    }
}
