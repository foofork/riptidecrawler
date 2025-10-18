//! Configuration types for the Riptide facade.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Main configuration for Riptide facade.
///
/// Unifies configuration from all underlying crates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiptideConfig {
    /// Fetch configuration
    #[cfg(feature = "scraper")]
    pub fetch: FetchConfig,

    /// Spider configuration
    #[cfg(feature = "spider")]
    pub spider: SpiderConfig,

    /// Browser configuration
    #[cfg(feature = "browser")]
    pub browser: BrowserConfig,

    /// Intelligence configuration
    #[cfg(feature = "intelligence")]
    pub intelligence: IntelligenceConfig,

    /// Security configuration
    #[cfg(feature = "security")]
    pub security: SecurityConfig,

    /// Monitoring configuration
    #[cfg(feature = "monitoring")]
    pub monitoring: MonitoringConfig,

    /// Cache configuration
    #[cfg(feature = "cache")]
    pub cache: CacheConfig,
}

impl Default for RiptideConfig {
    fn default() -> Self {
        Self {
            #[cfg(feature = "scraper")]
            fetch: FetchConfig::default(),

            #[cfg(feature = "spider")]
            spider: SpiderConfig::default(),

            #[cfg(feature = "browser")]
            browser: BrowserConfig::default(),

            #[cfg(feature = "intelligence")]
            intelligence: IntelligenceConfig::default(),

            #[cfg(feature = "security")]
            security: SecurityConfig::default(),

            #[cfg(feature = "monitoring")]
            monitoring: MonitoringConfig::default(),

            #[cfg(feature = "cache")]
            cache: CacheConfig::default(),
        }
    }
}

// ============================================================================
// Fetch Configuration
// ============================================================================

/// Configuration for HTTP fetching operations.
#[cfg(feature = "scraper")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchConfig {
    /// Maximum number of retries
    pub max_retries: u32,

    /// Request timeout in seconds
    pub timeout_secs: u64,

    /// User agent string
    pub user_agent: String,

    /// Follow redirects
    pub follow_redirects: bool,

    /// Maximum redirects to follow
    pub max_redirects: u32,

    /// Enable gzip compression
    pub enable_gzip: bool,
}

#[cfg(feature = "scraper")]
impl Default for FetchConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            timeout_secs: 30,
            user_agent: "RiptideBot/1.0".to_string(),
            follow_redirects: true,
            max_redirects: 10,
            enable_gzip: true,
        }
    }
}

// ============================================================================
// Spider Configuration
// ============================================================================

/// Configuration for web crawling operations.
#[cfg(feature = "spider")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiderConfig {
    /// Maximum crawl depth
    pub max_depth: u32,

    /// Maximum pages to crawl
    pub max_pages: u32,

    /// Delay between requests in milliseconds
    pub crawl_delay_ms: u64,

    /// Respect robots.txt
    pub respect_robots_txt: bool,

    /// Enable query-aware crawling
    pub query_aware: bool,
}

#[cfg(feature = "spider")]
impl Default for SpiderConfig {
    fn default() -> Self {
        Self {
            max_depth: 5,
            max_pages: 1000,
            crawl_delay_ms: 200,
            respect_robots_txt: true,
            query_aware: false,
        }
    }
}

// ============================================================================
// Browser Configuration
// ============================================================================

/// Configuration for browser automation.
#[cfg(feature = "browser")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    /// Run browser in headless mode
    pub headless: bool,

    /// Browser pool size
    pub pool_size: usize,

    /// Enable stealth mode
    pub enable_stealth: bool,

    /// Browser launch timeout in seconds
    pub launch_timeout_secs: u64,

    /// Page load timeout in seconds
    pub page_load_timeout_secs: u64,
}

#[cfg(feature = "browser")]
impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            headless: true,
            pool_size: 5,
            enable_stealth: false,
            launch_timeout_secs: 30,
            page_load_timeout_secs: 30,
        }
    }
}

// ============================================================================
// Intelligence Configuration
// ============================================================================

/// Configuration for LLM operations.
#[cfg(feature = "intelligence")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceConfig {
    /// Default LLM provider
    pub default_provider: String,

    /// Enable fallback to alternative providers
    pub enable_fallback: bool,

    /// LLM request timeout in seconds
    pub timeout_secs: u64,

    /// Maximum tokens per request
    pub max_tokens: usize,
}

#[cfg(feature = "intelligence")]
impl Default for IntelligenceConfig {
    fn default() -> Self {
        Self {
            default_provider: "openai".to_string(),
            enable_fallback: true,
            timeout_secs: 60,
            max_tokens: 4096,
        }
    }
}

// ============================================================================
// Security Configuration
// ============================================================================

/// Configuration for security features.
#[cfg(feature = "security")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable rate limiting
    pub enable_rate_limiting: bool,

    /// Requests per minute
    pub rate_limit_rpm: u32,

    /// Enable PII redaction
    pub enable_pii_redaction: bool,

    /// Require API key authentication
    pub api_key_required: bool,
}

#[cfg(feature = "security")]
impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_rate_limiting: true,
            rate_limit_rpm: 100,
            enable_pii_redaction: false,
            api_key_required: false,
        }
    }
}

// ============================================================================
// Monitoring Configuration
// ============================================================================

/// Configuration for monitoring and telemetry.
#[cfg(feature = "monitoring")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable telemetry collection
    pub enable_telemetry: bool,

    /// Enable metrics collection
    pub enable_metrics: bool,

    /// OpenTelemetry endpoint
    pub otlp_endpoint: Option<String>,

    /// Metrics export interval in seconds
    pub metrics_interval_secs: u64,
}

#[cfg(feature = "monitoring")]
impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_telemetry: false,
            enable_metrics: true,
            otlp_endpoint: None,
            metrics_interval_secs: 60,
        }
    }
}

// ============================================================================
// Cache Configuration
// ============================================================================

/// Configuration for caching.
#[cfg(feature = "cache")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable memory cache
    pub enable_memory_cache: bool,

    /// Memory cache size in MB
    pub memory_cache_size_mb: usize,

    /// Enable Redis cache
    pub enable_redis: bool,

    /// Redis URL
    pub redis_url: Option<String>,

    /// Default TTL in seconds
    pub default_ttl_secs: u64,
}

#[cfg(feature = "cache")]
impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enable_memory_cache: true,
            memory_cache_size_mb: 100,
            enable_redis: false,
            redis_url: None,
            default_ttl_secs: 3600,
        }
    }
}
