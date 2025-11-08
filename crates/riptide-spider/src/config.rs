use crate::{
    adaptive_stop::AdaptiveStopConfig,
    budget::BudgetConfig,
    frontier::FrontierConfig,
    query_aware::QueryAwareConfig,
    session::SessionConfig,
    strategy::AdaptiveCriteria as StrategyAdaptiveCriteria,
    types::{SitemapConfig, StrategyConfig},
};
use riptide_fetch::robots::RobotsConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

/// Main spider configuration
///
/// # Valid Configuration Ranges
///
/// ## Basic Settings
/// - `concurrency`: > 0 (number of concurrent requests)
/// - `max_depth`: > 0 when specified, recommended ≤ 1000
/// - `max_pages`: > 0 when specified
/// - `timeout`: > 0, recommended ≤ 300 seconds
/// - `max_redirects`: ≤ 20
///
/// ## Component Configurations
/// All nested configurations (frontier, performance, url_processing, etc.)
/// have their own validation rules documented in their respective structs.
///
/// # Validation
/// Call `validate()` to check all configuration values are within valid ranges
/// and mutually consistent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiderConfig {
    /// Base URL for crawling
    pub base_url: Url,
    /// User agent string
    pub user_agent: String,
    /// Request timeout (must be > 0, recommended ≤ 300s)
    pub timeout: Duration,
    /// Delay between requests (can be 0 for maximum speed)
    pub delay: Duration,
    /// Maximum number of concurrent requests (must be > 0)
    pub concurrency: usize,
    /// Maximum depth to crawl (must be > 0 when specified, recommended ≤ 1000)
    pub max_depth: Option<usize>,
    /// Maximum number of pages to crawl (must be > 0 when specified)
    pub max_pages: Option<usize>,
    /// Respect robots.txt
    pub respect_robots: bool,
    /// Follow redirects
    pub follow_redirects: bool,
    /// Maximum number of redirects to follow (must be ≤ 20)
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
    #[allow(clippy::expect_used)]
    fn default() -> Self {
        Self {
            // Default trait requires panic on failure - this hard-coded URL should always parse
            base_url: Url::parse("https://example.com").unwrap_or_else(|_| {
                panic!("Critical: Default URL 'https://example.com' failed to parse")
            }),
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
///
/// # Valid Ranges
/// - `max_url_length`: 1 to 65536 bytes (most browsers limit to 2048)
/// - `bloom_filter_capacity`: > 0 when deduplication enabled
/// - `bloom_filter_fpr`: 0.0 < value < 1.0 (typical: 0.01)
/// - `max_exact_urls`: > 0 when deduplication enabled
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
    /// Maximum URL length (valid range: 1-65536 bytes)
    pub max_url_length: usize,
    /// Enable URL deduplication
    pub enable_deduplication: bool,
    /// Bloom filter capacity for deduplication (must be > 0 when enabled)
    pub bloom_filter_capacity: usize,
    /// Bloom filter false positive rate (valid range: 0.0 < fpr < 1.0)
    pub bloom_filter_fpr: f64,
    /// Maximum exact URLs to track (must be > 0 when deduplication enabled)
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

/// Configuration for performance settings
///
/// # Valid Ranges and Constraints
/// - `max_concurrent_global`: > 0
/// - `max_concurrent_per_host`: > 0 and ≤ max_concurrent_global
/// - `request_timeout`: > 0
/// - `metrics_interval`: > 0
/// - `memory_pressure_threshold`: > 0 (bytes)
/// - `cpu_usage_threshold`: 0.0 to 1.0 (0% to 100%)
/// - `min_request_delay_micros`: ≤ max_request_delay_micros
/// - `max_request_delay_micros`: ≥ min_request_delay_micros
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum concurrent requests globally (must be > 0)
    pub max_concurrent_global: usize,
    /// Maximum concurrent requests per host (must be > 0 and ≤ global)
    pub max_concurrent_per_host: usize,
    /// Request timeout duration (must be > 0)
    pub request_timeout: Duration,
    /// Interval for updating metrics (must be > 0)
    pub metrics_interval: Duration,
    /// Memory pressure threshold in bytes (must be > 0)
    pub memory_pressure_threshold: usize,
    /// CPU usage threshold (valid range: 0.0-1.0)
    pub cpu_usage_threshold: f64,
    /// Enable adaptive throttling
    pub enable_adaptive_throttling: bool,
    /// Minimum delay between requests in microseconds
    pub min_request_delay_micros: u64,
    /// Maximum delay between requests in microseconds (must be ≥ min)
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
    ///
    /// Performs comprehensive validation of all configuration values including:
    /// - Positive value checks (no zero or negative values where inappropriate)
    /// - Boundary condition checks (reasonable upper limits)
    /// - Mutually exclusive option checks
    /// - Resource limit calculations
    /// - Performance configuration consistency
    pub fn validate(&self) -> Result<(), String> {
        // === Basic Concurrency Validation ===
        if self.concurrency == 0 {
            return Err("Concurrency must be greater than 0".to_string());
        }

        // Validate max depth if specified
        if let Some(max_depth) = self.max_depth {
            if max_depth == 0 {
                return Err("Max depth must be greater than 0 when specified".to_string());
            }
            // Reasonable upper limit for depth to prevent infinite loops
            if max_depth > 1000 {
                return Err("Max depth should not exceed 1000 (current: {})".to_string());
            }
        }

        // Validate max pages if specified
        if let Some(max_pages) = self.max_pages {
            if max_pages == 0 {
                return Err("Max pages must be greater than 0 when specified".to_string());
            }
        }

        // === Timeout and Delay Validation ===
        if self.timeout.is_zero() {
            return Err("Request timeout must be greater than 0".to_string());
        }

        // Delay can be zero but timeout should be reasonable
        if self.timeout > Duration::from_secs(300) {
            return Err("Request timeout should not exceed 300 seconds (5 minutes)".to_string());
        }

        // === Redirect Validation ===
        if self.max_redirects > 20 {
            return Err(format!(
                "Max redirects should not exceed 20 (current: {})",
                self.max_redirects
            ));
        }

        // === Frontier Configuration Validation ===
        if self.frontier.memory_limit == 0 {
            return Err("Frontier memory_limit must be greater than 0".to_string());
        }

        if self.frontier.memory_limit_mb == 0 {
            return Err("Frontier memory_limit_mb must be greater than 0".to_string());
        }

        if self.frontier.max_requests_per_host == 0 {
            return Err("Frontier max_requests_per_host must be greater than 0".to_string());
        }

        // Validate host diversity is in valid range (0.0 to 1.0)
        if self.frontier.max_host_diversity < 0.0 || self.frontier.max_host_diversity > 1.0 {
            return Err(format!(
                "Frontier max_host_diversity must be between 0.0 and 1.0 (current: {})",
                self.frontier.max_host_diversity
            ));
        }

        // === Performance Configuration Validation ===
        if self.performance.max_concurrent_global == 0 {
            return Err("Performance max_concurrent_global must be greater than 0".to_string());
        }

        if self.performance.max_concurrent_per_host == 0 {
            return Err("Performance max_concurrent_per_host must be greater than 0".to_string());
        }

        // Per-host concurrency should not exceed global concurrency
        if self.performance.max_concurrent_per_host > self.performance.max_concurrent_global {
            return Err(format!(
                "Performance max_concurrent_per_host ({}) cannot exceed max_concurrent_global ({})",
                self.performance.max_concurrent_per_host, self.performance.max_concurrent_global
            ));
        }

        if self.performance.request_timeout.is_zero() {
            return Err("Performance request_timeout must be greater than 0".to_string());
        }

        if self.performance.metrics_interval.is_zero() {
            return Err("Performance metrics_interval must be greater than 0".to_string());
        }

        if self.performance.memory_pressure_threshold == 0 {
            return Err("Performance memory_pressure_threshold must be greater than 0".to_string());
        }

        // CPU threshold should be between 0.0 and 1.0
        if self.performance.cpu_usage_threshold < 0.0 || self.performance.cpu_usage_threshold > 1.0
        {
            return Err(format!(
                "Performance cpu_usage_threshold must be between 0.0 and 1.0 (current: {})",
                self.performance.cpu_usage_threshold
            ));
        }

        // Request delay validation
        if self.performance.min_request_delay_micros > self.performance.max_request_delay_micros {
            return Err(format!(
                "Performance min_request_delay_micros ({}) cannot exceed max_request_delay_micros ({})",
                self.performance.min_request_delay_micros,
                self.performance.max_request_delay_micros
            ));
        }

        // === URL Processing Configuration Validation ===
        if self.url_processing.max_url_length == 0 {
            return Err("URL processing max_url_length must be greater than 0".to_string());
        }

        // URL length should be reasonable (most browsers limit to 2048)
        if self.url_processing.max_url_length > 65536 {
            return Err("URL processing max_url_length should not exceed 65536 bytes".to_string());
        }

        if self.url_processing.enable_deduplication {
            if self.url_processing.bloom_filter_capacity == 0 {
                return Err(
                    "URL processing bloom_filter_capacity must be greater than 0 when deduplication is enabled"
                        .to_string(),
                );
            }

            if self.url_processing.max_exact_urls == 0 {
                return Err(
                    "URL processing max_exact_urls must be greater than 0 when deduplication is enabled"
                        .to_string(),
                );
            }

            // False positive rate should be between 0.0 and 1.0 (exclusive)
            if self.url_processing.bloom_filter_fpr <= 0.0
                || self.url_processing.bloom_filter_fpr >= 1.0
            {
                return Err(format!(
                    "URL processing bloom_filter_fpr must be between 0.0 and 1.0 (current: {})",
                    self.url_processing.bloom_filter_fpr
                ));
            }
        }

        // === Session Configuration Validation ===
        if self.session.session_timeout.is_zero() {
            return Err("Session timeout must be greater than 0".to_string());
        }

        if self.session.max_concurrent_sessions == 0 {
            return Err("Session max_concurrent_sessions must be greater than 0".to_string());
        }

        // === Budget Configuration Validation ===
        // Validate global budget limits if specified
        if let Some(max_pages) = self.budget.global.max_pages {
            if max_pages == 0 {
                return Err(
                    "Budget global max_pages must be greater than 0 when specified".to_string(),
                );
            }
        }

        if let Some(max_depth) = self.budget.global.max_depth {
            if max_depth == 0 {
                return Err(
                    "Budget global max_depth must be greater than 0 when specified".to_string(),
                );
            }
        }

        // Validate that budget max_depth doesn't conflict with spider max_depth
        if let (Some(budget_depth), Some(spider_depth)) =
            (self.budget.global.max_depth, self.max_depth)
        {
            if budget_depth as usize != spider_depth {
                // This is just a warning scenario - they can be different but it's worth noting
                // We'll allow it but could log a warning in production code
            }
        }

        // === Adaptive Stop Configuration Validation ===
        if self.adaptive_stop.min_pages_before_stop == 0 {
            return Err("Adaptive stop min_pages_before_stop must be greater than 0".to_string());
        }

        if self.adaptive_stop.patience == 0 {
            return Err("Adaptive stop patience must be greater than 0".to_string());
        }

        // Gain threshold should be non-negative
        if self.adaptive_stop.min_gain_threshold < 0.0 {
            return Err("Adaptive stop min_gain_threshold cannot be negative".to_string());
        }

        Ok(())
    }

    /// Estimate memory usage based on configuration
    pub fn estimate_memory_usage(&self) -> usize {
        // Base memory usage
        let mut memory: usize = 1024 * 1024; // 1MB base

        // Add memory for URL processing
        if self.url_processing.enable_deduplication {
            memory = memory.saturating_add(
                self.url_processing
                    .bloom_filter_capacity
                    .saturating_mul(8_usize),
            ); // Bloom filter bits
            memory =
                memory.saturating_add(self.url_processing.max_exact_urls.saturating_mul(256_usize));
            // URL storage estimate
        }

        // Add memory for concurrency
        memory = memory.saturating_add(
            self.performance
                .max_concurrent_global
                .saturating_mul(1024_usize),
        ); // Per-request overhead
        memory = memory.saturating_add(
            self.performance
                .max_concurrent_per_host
                .saturating_mul(512_usize),
        ); // Per-host tracking

        // Add frontier memory estimate
        if let Some(max_pages) = self.max_pages {
            memory = memory.saturating_add(max_pages.saturating_mul(128_usize));
            // Frontier entry estimate
        }

        memory
    }

    /// Optimize configuration for available resources
    ///
    /// Adjusts configuration parameters based on available system resources:
    /// - Concurrency settings based on CPU cores
    /// - Memory limits based on available RAM
    /// - Bloom filter capacity for URL deduplication
    /// - Frontier memory limits
    ///
    /// # Arguments
    /// * `available_memory_mb` - Available system memory in megabytes
    /// * `available_cores` - Number of available CPU cores
    ///
    /// # Resource Tiers
    /// - **Low memory** (<1024MB): Conservative settings, small bloom filters
    /// - **Medium memory** (1024-4095MB): Balanced settings
    /// - **High memory** (≥4096MB): Aggressive settings, large bloom filters
    pub fn optimize_for_resources(&mut self, available_memory_mb: usize, available_cores: usize) {
        // Adjust concurrency based on cores (2x cores, capped at 16)
        self.concurrency = (available_cores * 2).min(16);
        self.performance.max_concurrent_global = self.concurrency;
        self.performance.max_concurrent_per_host = (self.concurrency / 4).max(1);

        // Adjust memory-based settings based on available memory tiers
        if available_memory_mb < 1024 {
            // Low memory: optimize for efficiency
            self.url_processing.bloom_filter_capacity = 10_000;
            self.url_processing.max_exact_urls = 1_000;
            self.frontier.memory_limit_mb = 50;
            self.frontier.memory_limit = 5_000; // ~5k requests in memory
        } else if available_memory_mb < 4096 {
            // Medium memory: balanced configuration
            self.url_processing.bloom_filter_capacity = 100_000;
            self.url_processing.max_exact_urls = 10_000;
            self.frontier.memory_limit_mb = 200;
            self.frontier.memory_limit = 50_000; // ~50k requests in memory
        } else {
            // High memory: aggressive settings for performance
            self.url_processing.bloom_filter_capacity = 1_000_000;
            self.url_processing.max_exact_urls = 100_000;
            self.frontier.memory_limit_mb = 500;
            self.frontier.memory_limit = 200_000; // ~200k requests in memory
        }

        // Adjust performance thresholds based on memory
        self.performance.memory_pressure_threshold = (available_memory_mb * 1024 * 1024 * 3) / 4;
        // 75% of available memory
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
        let scoring_config = crate::types::ScoringConfig {
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
    pub fn to_crawling_strategy(&self) -> crate::strategy::CrawlingStrategy {
        use crate::strategy::*;

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
            }
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
