//! Builder pattern for configuring Riptide facade.

use crate::config::*;
use crate::error::{Result, RiptideError};
use crate::runtime::RiptideRuntime;
use crate::Riptide;
use std::sync::Arc;

/// Builder for configuring and creating a Riptide instance.
///
/// # Example
///
/// ```no_run
/// use riptide_facade::Riptide;
/// use std::time::Duration;
///
/// # async fn example() -> anyhow::Result<()> {
/// let riptide = Riptide::builder()
///     .with_default_config()
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub struct RiptideBuilder {
    config: RiptideConfig,
}

impl RiptideBuilder {
    /// Create a new builder with empty configuration.
    pub fn new() -> Self {
        Self {
            config: RiptideConfig::default(),
        }
    }

    /// Use default configuration for all features.
    pub fn with_default_config(mut self) -> Self {
        self.config = RiptideConfig::default();
        self
    }

    // ========================================================================
    // Fetch Configuration
    // ========================================================================

    /// Configure fetch settings.
    #[cfg(feature = "scraper")]
    pub fn with_fetch<F>(mut self, configurator: F) -> Self
    where
        F: FnOnce(FetchConfigBuilder) -> FetchConfigBuilder,
    {
        let builder = FetchConfigBuilder::new(self.config.fetch);
        self.config.fetch = configurator(builder).build();
        self
    }

    // ========================================================================
    // Spider Configuration
    // ========================================================================

    /// Configure spider settings.
    #[cfg(feature = "spider")]
    pub fn with_spider<F>(mut self, configurator: F) -> Self
    where
        F: FnOnce(SpiderConfigBuilder) -> SpiderConfigBuilder,
    {
        let builder = SpiderConfigBuilder::new(self.config.spider);
        self.config.spider = configurator(builder).build();
        self
    }

    // ========================================================================
    // Browser Configuration
    // ========================================================================

    /// Configure browser settings.
    #[cfg(feature = "browser")]
    pub fn with_browser<F>(mut self, configurator: F) -> Self
    where
        F: FnOnce(BrowserConfigBuilder) -> BrowserConfigBuilder,
    {
        let builder = BrowserConfigBuilder::new(self.config.browser);
        self.config.browser = configurator(builder).build();
        self
    }

    // ========================================================================
    // Intelligence Configuration
    // ========================================================================

    /// Configure intelligence settings.
    #[cfg(feature = "intelligence")]
    pub fn with_intelligence<F>(mut self, configurator: F) -> Self
    where
        F: FnOnce(IntelligenceConfigBuilder) -> IntelligenceConfigBuilder,
    {
        let builder = IntelligenceConfigBuilder::new(self.config.intelligence);
        self.config.intelligence = configurator(builder).build();
        self
    }

    // ========================================================================
    // Security Configuration
    // ========================================================================

    /// Configure security settings.
    #[cfg(feature = "security")]
    pub fn with_security<F>(mut self, configurator: F) -> Self
    where
        F: FnOnce(SecurityConfigBuilder) -> SecurityConfigBuilder,
    {
        let builder = SecurityConfigBuilder::new(self.config.security);
        self.config.security = configurator(builder).build();
        self
    }

    // ========================================================================
    // Monitoring Configuration
    // ========================================================================

    /// Configure monitoring settings.
    #[cfg(feature = "monitoring")]
    pub fn with_monitoring<F>(mut self, configurator: F) -> Self
    where
        F: FnOnce(MonitoringConfigBuilder) -> MonitoringConfigBuilder,
    {
        let builder = MonitoringConfigBuilder::new(self.config.monitoring);
        self.config.monitoring = configurator(builder).build();
        self
    }

    // ========================================================================
    // Cache Configuration
    // ========================================================================

    /// Configure cache settings.
    #[cfg(feature = "cache")]
    pub fn with_cache<F>(mut self, configurator: F) -> Self
    where
        F: FnOnce(CacheConfigBuilder) -> CacheConfigBuilder,
    {
        let builder = CacheConfigBuilder::new(self.config.cache);
        self.config.cache = configurator(builder).build();
        self
    }

    // ========================================================================
    // Build
    // ========================================================================

    /// Build the Riptide instance.
    ///
    /// Validates configuration and initializes the runtime.
    pub fn build(self) -> Result<Riptide> {
        // Validate configuration
        self.validate_config()?;

        // Create runtime
        let runtime = RiptideRuntime::new(self.config.clone())?;

        Ok(Riptide {
            config: self.config,
            runtime: Arc::new(runtime),
        })
    }

    fn validate_config(&self) -> Result<()> {
        // Add configuration validation logic
        Ok(())
    }
}

impl Default for RiptideBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Feature-Specific Config Builders
// ============================================================================

#[cfg(feature = "scraper")]
pub struct FetchConfigBuilder(FetchConfig);

#[cfg(feature = "scraper")]
impl FetchConfigBuilder {
    pub fn new(config: FetchConfig) -> Self {
        Self(config)
    }

    pub fn max_retries(mut self, retries: u32) -> Self {
        self.0.max_retries = retries;
        self
    }

    pub fn timeout(mut self, secs: u64) -> Self {
        self.0.timeout_secs = secs;
        self
    }

    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.0.user_agent = ua.into();
        self
    }

    pub fn follow_redirects(mut self, follow: bool) -> Self {
        self.0.follow_redirects = follow;
        self
    }

    pub fn build(self) -> FetchConfig {
        self.0
    }
}

#[cfg(feature = "spider")]
pub struct SpiderConfigBuilder(SpiderConfig);

#[cfg(feature = "spider")]
impl SpiderConfigBuilder {
    pub fn new(config: SpiderConfig) -> Self {
        Self(config)
    }

    pub fn max_depth(mut self, depth: u32) -> Self {
        self.0.max_depth = depth;
        self
    }

    pub fn max_pages(mut self, pages: u32) -> Self {
        self.0.max_pages = pages;
        self
    }

    pub fn crawl_delay_ms(mut self, delay: u64) -> Self {
        self.0.crawl_delay_ms = delay;
        self
    }

    pub fn respect_robots_txt(mut self, respect: bool) -> Self {
        self.0.respect_robots_txt = respect;
        self
    }

    pub fn build(self) -> SpiderConfig {
        self.0
    }
}

#[cfg(feature = "browser")]
pub struct BrowserConfigBuilder(BrowserConfig);

#[cfg(feature = "browser")]
impl BrowserConfigBuilder {
    pub fn new(config: BrowserConfig) -> Self {
        Self(config)
    }

    pub fn headless(mut self, headless: bool) -> Self {
        self.0.headless = headless;
        self
    }

    pub fn pool_size(mut self, size: usize) -> Self {
        self.0.pool_size = size;
        self
    }

    pub fn enable_stealth(mut self) -> Self {
        self.0.enable_stealth = true;
        self
    }

    pub fn build(self) -> BrowserConfig {
        self.0
    }
}

#[cfg(feature = "intelligence")]
pub struct IntelligenceConfigBuilder(IntelligenceConfig);

#[cfg(feature = "intelligence")]
impl IntelligenceConfigBuilder {
    pub fn new(config: IntelligenceConfig) -> Self {
        Self(config)
    }

    pub fn default_provider(mut self, provider: impl Into<String>) -> Self {
        self.0.default_provider = provider.into();
        self
    }

    pub fn enable_fallback(mut self) -> Self {
        self.0.enable_fallback = true;
        self
    }

    pub fn timeout(mut self, secs: u64) -> Self {
        self.0.timeout_secs = secs;
        self
    }

    pub fn build(self) -> IntelligenceConfig {
        self.0
    }
}

#[cfg(feature = "security")]
pub struct SecurityConfigBuilder(SecurityConfig);

#[cfg(feature = "security")]
impl SecurityConfigBuilder {
    pub fn new(config: SecurityConfig) -> Self {
        Self(config)
    }

    pub fn enable_rate_limiting(mut self) -> Self {
        self.0.enable_rate_limiting = true;
        self
    }

    pub fn rate_limit_rpm(mut self, rpm: u32) -> Self {
        self.0.rate_limit_rpm = rpm;
        self
    }

    pub fn enable_pii_redaction(mut self) -> Self {
        self.0.enable_pii_redaction = true;
        self
    }

    pub fn api_key_required(mut self, required: bool) -> Self {
        self.0.api_key_required = required;
        self
    }

    pub fn build(self) -> SecurityConfig {
        self.0
    }
}

#[cfg(feature = "monitoring")]
pub struct MonitoringConfigBuilder(MonitoringConfig);

#[cfg(feature = "monitoring")]
impl MonitoringConfigBuilder {
    pub fn new(config: MonitoringConfig) -> Self {
        Self(config)
    }

    pub fn enable_telemetry(mut self) -> Self {
        self.0.enable_telemetry = true;
        self
    }

    pub fn enable_metrics(mut self) -> Self {
        self.0.enable_metrics = true;
        self
    }

    pub fn otlp_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.0.otlp_endpoint = Some(endpoint.into());
        self
    }

    pub fn build(self) -> MonitoringConfig {
        self.0
    }
}

#[cfg(feature = "cache")]
pub struct CacheConfigBuilder(CacheConfig);

#[cfg(feature = "cache")]
impl CacheConfigBuilder {
    pub fn new(config: CacheConfig) -> Self {
        Self(config)
    }

    pub fn enable_memory_cache(mut self) -> Self {
        self.0.enable_memory_cache = true;
        self
    }

    pub fn memory_cache_size_mb(mut self, size: usize) -> Self {
        self.0.memory_cache_size_mb = size;
        self
    }

    pub fn enable_redis(mut self) -> Self {
        self.0.enable_redis = true;
        self
    }

    pub fn redis_url(mut self, url: impl Into<String>) -> Self {
        self.0.redis_url = Some(url.into());
        self
    }

    pub fn build(self) -> CacheConfig {
        self.0
    }
}
