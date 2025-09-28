use crate::robots::RobotsConfig;
use crate::spider::{
    adaptive_stop::AdaptiveStopConfig,
    budget::BudgetConfig,
    frontier::FrontierConfig,
    session::SessionConfig,
    strategy::AdaptiveCriteria as StrategyAdaptiveCriteria,
    types::{SitemapConfig, StrategyConfig},
    query_aware::QueryAwareConfig,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

/// Main spider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiderConfig {
    /// Base URL for crawling
    pub base_url: Url,
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
    /// Session configuration
    pub session: SessionConfig,
    /// Budget configuration
    pub budget: BudgetConfig,
    /// Frontier configuration
    pub frontier: FrontierConfig,
    /// Strategy configuration
    pub strategy: StrategyConfig,
    /// Sitemap configuration
    pub sitemap: SitemapConfig,
    /// Adaptive stop configuration
    pub adaptive_stop: AdaptiveStopConfig,
    /// Robots.txt configuration
    pub robots: RobotsConfig,
    /// URL processing configuration
    pub url_processing: UrlProcessingConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
    /// Query-aware crawling configuration
    pub query_aware: QueryAwareConfig,
}

impl Default for SpiderConfig {
    fn default() -> Self {
        Self {
            base_url: Url::parse("https://example.com").expect("Valid default URL"),
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
            session: SessionConfig::default(),
            budget: BudgetConfig::default(),
            frontier: FrontierConfig::default(),
            strategy: StrategyConfig::default(),
            sitemap: SitemapConfig::default(),
            adaptive_stop: AdaptiveStopConfig::default(),
            robots: RobotsConfig::default(),
            url_processing: UrlProcessingConfig::default(),
            performance: PerformanceConfig::default(),
            query_aware: QueryAwareConfig::default(),
        }
    }
}

/// Configuration for URL processing
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
                r"\.(css|js|png|jpg|jpeg|gif|svg|ico|pdf|zip|tar|gz|exe|dmg)$".to_string()
            ],
            exclude_extensions: vec![
                "css".to_string(), "js".to_string(), "png".to_string(), "jpg".to_string(),
                "jpeg".to_string(), "gif".to_string(), "svg".to_string(), "ico".to_string(),
                "pdf".to_string(), "zip".to_string(), "tar".to_string(), "gz".to_string(),
                "exe".to_string(), "dmg".to_string()
            ],
        }
    }
}

/// Configuration for performance settings
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
            min_request_delay_micros: 100_000, // 100ms
            max_request_delay_micros: 5_000_000, // 5s
        }
    }
}

impl SpiderConfig {
    /// Create a new spider configuration
    pub fn new(base_url: Url) -> Self {
        Self {
            base_url,
            ..Default::default()
        }
    }

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

    /// Set session configuration
    pub fn with_session_config(mut self, session: SessionConfig) -> Self {
        self.session = session;
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

    /// Estimate memory usage based on configuration
    pub fn estimate_memory_usage(&self) -> usize {
        // Base memory usage
        let mut memory = 1024 * 1024; // 1MB base

        // Add memory for URL processing
        if self.url_processing.enable_deduplication {
            memory += self.url_processing.bloom_filter_capacity * 8; // Bloom filter bits
            memory += self.url_processing.max_exact_urls * 256; // URL storage estimate
        }

        // Add memory for concurrency
        memory += self.performance.max_concurrent_global * 1024; // Per-request overhead
        memory += self.performance.max_concurrent_per_host * 512; // Per-host tracking

        // Add frontier memory estimate
        if let Some(max_pages) = self.max_pages {
            memory += max_pages * 128; // Frontier entry estimate
        }

        memory
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
            self.frontier.memory_limit_mb = 50;
        } else if available_memory_mb < 4096 {
            // Medium memory
            self.url_processing.bloom_filter_capacity = 100_000;
            self.url_processing.max_exact_urls = 10_000;
            self.frontier.memory_limit_mb = 200;
        } else {
            // High memory
            self.url_processing.bloom_filter_capacity = 1_000_000;
            self.url_processing.max_exact_urls = 100_000;
            self.frontier.memory_limit_mb = 500;
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
            adaptive_stop: AdaptiveStopConfig {
                min_gain_threshold: 200.0,
                patience: 10,
                ..Default::default()
            },
            strategy: StrategyConfig {
                default_strategy: "BreadthFirst".to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// E-commerce site crawling configuration
    pub fn ecommerce_site() -> SpiderConfig {
        let scoring_config = crate::spider::types::ScoringConfig {
            content_keywords: vec![
                "price".to_string(),
                "product".to_string(),
                "buy".to_string(),
                "cart".to_string(),
            ],
            ..Default::default()
        };

        SpiderConfig {
            concurrency: 6,
            max_pages: Some(10000),
            max_depth: Some(8),
            delay: Duration::from_millis(500),
            enable_javascript: true,
            follow_redirects: true,
            strategy: StrategyConfig {
                default_strategy: "BestFirst".to_string(),
                scoring: scoring_config,
                ..Default::default()
            },
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
            strategy: StrategyConfig {
                default_strategy: "DepthFirst".to_string(),
                ..Default::default()
            },
            adaptive_stop: AdaptiveStopConfig {
                min_gain_threshold: 100.0,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Authenticated crawling configuration
    pub fn authenticated_crawling() -> SpiderConfig {
        SpiderConfig {
            concurrency: 2,
            max_pages: Some(1000),
            delay: Duration::from_millis(1000),
            session: SessionConfig {
                enable_authentication: true,
                enable_cookie_persistence: true,
                session_timeout: Duration::from_secs(3600),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

impl StrategyConfig {
    /// Convert to CrawlingStrategy
    pub fn to_crawling_strategy(&self) -> crate::spider::strategy::CrawlingStrategy {
        use crate::spider::strategy::*;

        match self.default_strategy.as_str() {
            "BreadthFirst" => breadth_first_strategy(),
            "DepthFirst" => depth_first_strategy(),
            "BestFirst" => best_first_strategy_with_config(self.scoring.clone()),
            "Adaptive" => {
                if self.enable_adaptive {
                    adaptive_strategy(
                        best_first_strategy_with_config(self.scoring.clone()),
                        breadth_first_strategy(),
                        Some(StrategyAdaptiveCriteria {
                            max_frontier_size: self.adaptive_criteria.max_frontier_size,
                            max_average_depth: self.adaptive_criteria.max_average_depth,
                            min_success_rate: self.adaptive_criteria.min_success_rate,
                            min_pages_for_switch: self.adaptive_criteria.min_pages_for_switch,
                            switch_cooldown_pages: self.adaptive_criteria.switch_cooldown_pages,
                        }),
                    )
                } else {
                    breadth_first_strategy()
                }
            },
            _ => breadth_first_strategy(), // Default fallback
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

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
        let base_url = Url::from_str("https://test.example.com").expect("Valid URL");
        let config = SpiderConfig::new(base_url.clone())
            .with_user_agent("Custom Agent".to_string())
            .with_timeout(Duration::from_secs(60))
            .with_concurrency(8)
            .with_max_depth(Some(5))
            .with_javascript(true);

        assert_eq!(config.base_url, base_url);
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
}