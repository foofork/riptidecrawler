//! Spider configuration types and presets
//!
//! This module provides configuration structures for the spider crawling system,
//! extracted from riptide-core for better separation of concerns. Note that this
//! contains only the configuration types - the actual spider implementation remains
//! in riptide-core.
//!
//! # Note on Dependencies
//!
//! Some spider configuration types depend on types from riptide-core (e.g.,
//! RobotsConfig, SessionConfig, BudgetConfig). These will need to be imported
//! when used. This is an intentional design to avoid circular dependencies
//! while still providing configuration management.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for URL processing during crawling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlProcessingConfig {
    /// Enable URL normalization
    pub enable_normalization: bool,
    /// Remove www prefix from domains
    pub remove_www: bool,
    /// Remove trailing slashes
    pub remove_trailing_slash: bool,
    /// Convert URLs to lowercase
    pub lowercase_urls: bool,
    /// Remove fragment identifiers
    pub remove_fragments: bool,
    /// Remove default ports
    pub remove_default_ports: bool,
    /// Maximum URL length
    pub max_url_length: usize,
    /// Enable URL deduplication
    pub enable_deduplication: bool,
    /// Bloom filter capacity for deduplication
    pub bloom_filter_capacity: usize,
    /// Bloom filter false positive rate
    pub bloom_filter_fpr: f64,
    /// Maximum exact URLs to track
    pub max_exact_urls: usize,
    /// URL exclude patterns
    pub exclude_patterns: Vec<String>,
    /// File extensions to exclude
    pub exclude_extensions: Vec<String>,
}

impl Default for UrlProcessingConfig {
    fn default() -> Self {
        Self {
            enable_normalization: true,
            remove_www: false,
            remove_trailing_slash: true,
            lowercase_urls: true,
            remove_fragments: true,
            remove_default_ports: true,
            max_url_length: 2048,
            enable_deduplication: true,
            bloom_filter_capacity: 100_000,
            bloom_filter_fpr: 0.01,
            max_exact_urls: 10_000,
            exclude_patterns: vec![
                r"\.(css|js|png|jpg|jpeg|gif|svg|ico|pdf|zip|tar|gz|exe|dmg)$".to_string(),
            ],
            exclude_extensions: vec![
                "css".to_string(),
                "js".to_string(),
                "png".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "gif".to_string(),
                "svg".to_string(),
                "ico".to_string(),
                "pdf".to_string(),
                "zip".to_string(),
                "tar".to_string(),
                "gz".to_string(),
                "exe".to_string(),
                "dmg".to_string(),
            ],
        }
    }
}

/// Configuration for performance settings during crawling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum concurrent requests globally
    pub max_concurrent_global: usize,
    /// Maximum concurrent requests per host
    pub max_concurrent_per_host: usize,
    /// Request timeout duration
    pub request_timeout: Duration,
    /// Interval for updating metrics
    pub metrics_interval: Duration,
    /// Memory pressure threshold (bytes)
    pub memory_pressure_threshold: usize,
    /// CPU usage threshold (0.0-1.0)
    pub cpu_usage_threshold: f64,
    /// Enable adaptive throttling
    pub enable_adaptive_throttling: bool,
    /// Minimum delay between requests (microseconds)
    pub min_request_delay_micros: u64,
    /// Maximum delay between requests (microseconds)
    pub max_request_delay_micros: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_global: 10,
            max_concurrent_per_host: 2,
            request_timeout: Duration::from_secs(30),
            metrics_interval: Duration::from_secs(10),
            memory_pressure_threshold: 512 * 1024 * 1024, // 512MB
            cpu_usage_threshold: 0.8,
            enable_adaptive_throttling: true,
            min_request_delay_micros: 100_000,   // 100ms
            max_request_delay_micros: 5_000_000, // 5s
        }
    }
}

/// Main spider configuration
///
/// Note: This is a simplified version that contains only the configuration
/// that can be managed independently. The full SpiderConfig in riptide-core
/// includes additional dependencies on spider-specific types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiderConfig {
    /// User agent string
    pub user_agent: String,
    /// Request timeout
    pub timeout: Duration,
    /// Delay between requests
    pub delay: Duration,
    /// Maximum number of concurrent requests
    pub concurrency: usize,
    /// Maximum depth to crawl
    pub max_depth: Option<usize>,
    /// Maximum number of pages to crawl
    pub max_pages: Option<usize>,
    /// Respect robots.txt
    pub respect_robots: bool,
    /// Follow redirects
    pub follow_redirects: bool,
    /// Maximum number of redirects to follow
    pub max_redirects: usize,
    /// Enable JavaScript rendering
    pub enable_javascript: bool,
    /// URL processing configuration
    pub url_processing: UrlProcessingConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
}

impl Default for SpiderConfig {
    fn default() -> Self {
        Self {
            user_agent: "RipTide Spider/1.0".to_string(),
            timeout: Duration::from_secs(30),
            delay: Duration::from_millis(500),
            concurrency: 4,
            max_depth: Some(10),
            max_pages: Some(1000),
            respect_robots: true,
            follow_redirects: true,
            max_redirects: 5,
            enable_javascript: false,
            url_processing: UrlProcessingConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl SpiderConfig {
    /// Set user agent
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = user_agent;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set delay between requests
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Set concurrency level
    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency;
        self
    }

    /// Set maximum depth
    pub fn with_max_depth(mut self, max_depth: Option<usize>) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Set maximum pages
    pub fn with_max_pages(mut self, max_pages: Option<usize>) -> Self {
        self.max_pages = max_pages;
        self
    }

    /// Enable or disable robots.txt respect
    pub fn with_respect_robots(mut self, respect_robots: bool) -> Self {
        self.respect_robots = respect_robots;
        self
    }

    /// Enable or disable JavaScript rendering
    pub fn with_javascript(mut self, enable_javascript: bool) -> Self {
        self.enable_javascript = enable_javascript;
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.concurrency == 0 {
            return Err("Concurrency must be greater than 0".to_string());
        }

        if let Some(max_depth) = self.max_depth {
            if max_depth == 0 {
                return Err("Max depth must be greater than 0".to_string());
            }
        }

        if let Some(max_pages) = self.max_pages {
            if max_pages == 0 {
                return Err("Max pages must be greater than 0".to_string());
            }
        }

        if self.timeout.is_zero() {
            return Err("Timeout must be greater than 0".to_string());
        }

        if self.max_redirects > 20 {
            return Err("Max redirects should not exceed 20".to_string());
        }

        Ok(())
    }

    /// Optimize configuration for available resources
    pub fn optimize_for_resources(&mut self, available_memory_mb: usize, available_cores: usize) {
        // Adjust concurrency based on cores
        self.concurrency = (available_cores * 2).min(16);
        self.performance.max_concurrent_global = self.concurrency;
        self.performance.max_concurrent_per_host = (self.concurrency / 4).max(1);

        // Adjust memory-based settings
        if available_memory_mb < 1024 {
            // Low memory: optimize for efficiency
            self.url_processing.bloom_filter_capacity = 10_000;
            self.url_processing.max_exact_urls = 1_000;
        } else if available_memory_mb < 4096 {
            // Medium memory
            self.url_processing.bloom_filter_capacity = 100_000;
            self.url_processing.max_exact_urls = 10_000;
        } else {
            // High memory
            self.url_processing.bloom_filter_capacity = 1_000_000;
            self.url_processing.max_exact_urls = 100_000;
        }
    }
}

/// Preset spider configurations for common use cases
pub struct SpiderPresets;

impl SpiderPresets {
    /// Development and testing configuration
    pub fn development() -> SpiderConfig {
        SpiderConfig {
            concurrency: 2,
            max_pages: Some(50),
            max_depth: Some(3),
            delay: Duration::from_millis(100),
            respect_robots: false,
            performance: PerformanceConfig {
                max_concurrent_global: 2,
                max_concurrent_per_host: 1,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// High-performance crawling configuration
    pub fn high_performance() -> SpiderConfig {
        SpiderConfig {
            concurrency: 16,
            max_pages: Some(10000),
            max_depth: Some(10),
            delay: Duration::from_millis(50),
            timeout: Duration::from_secs(20),
            performance: PerformanceConfig {
                max_concurrent_global: 16,
                max_concurrent_per_host: 4,
                ..Default::default()
            },
            url_processing: UrlProcessingConfig {
                enable_deduplication: true,
                bloom_filter_capacity: 1_000_000,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// News site crawling configuration
    pub fn news_site() -> SpiderConfig {
        SpiderConfig {
            concurrency: 8,
            max_pages: Some(5000),
            max_depth: Some(5),
            delay: Duration::from_millis(200),
            enable_javascript: true,
            ..Default::default()
        }
    }

    /// E-commerce site crawling configuration
    pub fn ecommerce_site() -> SpiderConfig {
        SpiderConfig {
            concurrency: 6,
            max_pages: Some(10000),
            max_depth: Some(8),
            delay: Duration::from_millis(500),
            enable_javascript: true,
            follow_redirects: true,
            ..Default::default()
        }
    }

    /// Documentation site crawling configuration
    pub fn documentation_site() -> SpiderConfig {
        SpiderConfig {
            concurrency: 4,
            max_pages: Some(2000),
            max_depth: Some(15),
            delay: Duration::from_millis(100),
            ..Default::default()
        }
    }

    /// Authenticated crawling configuration
    pub fn authenticated_crawling() -> SpiderConfig {
        SpiderConfig {
            concurrency: 2,
            max_pages: Some(1000),
            delay: Duration::from_millis(1000),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SpiderConfig::default();
        assert_eq!(config.user_agent, "RipTide Spider/1.0");
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.concurrency, 4);
        assert!(config.respect_robots);
    }

    #[test]
    fn test_config_builder() {
        let config = SpiderConfig::default()
            .with_user_agent("Custom Agent".to_string())
            .with_timeout(Duration::from_secs(60))
            .with_concurrency(8)
            .with_max_depth(Some(5))
            .with_javascript(true);

        assert_eq!(config.user_agent, "Custom Agent");
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.concurrency, 8);
        assert_eq!(config.max_depth, Some(5));
        assert!(config.enable_javascript);
    }

    #[test]
    fn test_config_validation() {
        let mut config = SpiderConfig::default();

        // Valid config should pass
        assert!(config.validate().is_ok());

        // Zero concurrency should fail
        config.concurrency = 0;
        assert!(config.validate().is_err());

        // Reset and test zero max depth
        config = SpiderConfig::default();
        config.max_depth = Some(0);
        assert!(config.validate().is_err());

        // Reset and test zero max pages
        config = SpiderConfig::default();
        config.max_pages = Some(0);
        assert!(config.validate().is_err());

        // Reset and test zero timeout
        config = SpiderConfig::default();
        config.timeout = Duration::from_secs(0);
        assert!(config.validate().is_err());

        // Reset and test excessive redirects
        config = SpiderConfig::default();
        config.max_redirects = 25;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_spider_presets() {
        let dev = SpiderPresets::development();
        assert_eq!(dev.concurrency, 2);
        assert_eq!(dev.max_pages, Some(50));

        let hp = SpiderPresets::high_performance();
        assert_eq!(hp.concurrency, 16);
        assert_eq!(hp.max_pages, Some(10000));

        let news = SpiderPresets::news_site();
        assert!(news.enable_javascript);

        let ecom = SpiderPresets::ecommerce_site();
        assert_eq!(ecom.max_depth, Some(8));

        let docs = SpiderPresets::documentation_site();
        assert_eq!(docs.max_depth, Some(15));

        let auth = SpiderPresets::authenticated_crawling();
        assert_eq!(auth.delay, Duration::from_millis(1000));
    }
}
