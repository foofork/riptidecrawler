//! Shared utilities for handlers to reduce code duplication
#![allow(dead_code)]
pub mod spider;

use crate::context::ApplicationContext;
#[cfg(feature = "spider")]
use crate::errors::ApiError;

/// Helper for recording metrics in handlers
pub struct MetricsRecorder<'a> {
    state: &'a ApplicationContext,
}

impl<'a> MetricsRecorder<'a> {
    pub fn new(state: &'a ApplicationContext) -> Self {
        Self { state }
    }

    #[allow(dead_code)] // Reserved for future metrics collection
    pub fn record_request(&self, _endpoint: &str) {
        // Metrics recording implementation
        // This can be expanded as metrics infrastructure is activated
    }

    #[allow(dead_code)] // Reserved for future metrics collection
    pub fn record_success(&self, _endpoint: &str, _duration_ms: u64) {
        // Success metrics
    }

    #[allow(dead_code)] // Reserved for future metrics collection
    pub fn record_error(&self, _endpoint: &str, _error_type: &str) {
        // Error metrics
    }

    pub fn record_spider_crawl(
        &self,
        pages_crawled: u64,
        pages_failed: u64,
        duration: std::time::Duration,
    ) {
        // Phase D: Spider metrics now tracked via business_metrics
        // record_spider_crawl(&self, _url: &str, _max_depth: u32)
        self.state.business_metrics.record_spider_crawl("", 0);

        // Track pages failed via record_spider_error(&self, _error_type: &str)
        for _ in 0..pages_failed {
            self.state
                .business_metrics
                .record_spider_error("crawl_failure");
        }

        // Track individual pages crawled via record_spider_page(&self, _url: &str, _depth: u32)
        for _ in 0..pages_crawled {
            self.state.business_metrics.record_spider_page("", 0);
        }

        // Duration tracking not supported by current BusinessMetrics interface
        let _ = duration; // Acknowledge unused parameter
    }

    pub fn record_spider_crawl_failure(&self) {
        // Phase D: Spider failure tracked via business_metrics
        self.state
            .business_metrics
            .record_spider_error("general_failure");
    }

    pub fn update_frontier_size(&self, size: usize) {
        // Phase D: Spider frontier size now tracked via business_metrics
        // Note: record_spider_page expects (url: &str, depth: u32)
        // Using placeholder url and treating size as a pseudo-depth for now
        // TODO: Consider adding a dedicated update_frontier_size method to BusinessMetrics
        self.state
            .business_metrics
            .record_spider_page("", size as u32);
    }

    pub fn record_http_request(
        &self,
        method: &str,
        path: &str,
        status: u16,
        duration: std::time::Duration,
    ) {
        // Phase D: HTTP request metrics now via AppState helper (delegates to transport_metrics)
        self.state
            .record_http_request(method, path, status, duration.as_secs_f64());
    }
}

/// Helper for emitting events in handlers (WIP - behind `events` feature)
#[cfg(feature = "events")]
pub struct EventEmitter<'a> {
    _state: &'a ApplicationContext,
}

#[cfg(feature = "events")]
impl<'a> EventEmitter<'a> {
    #[allow(dead_code)]
    pub fn new(state: &'a ApplicationContext) -> Self {
        Self { _state: state }
    }

    #[allow(dead_code)]
    pub fn emit_event(&self, _event_type: &str, _data: serde_json::Value) {
        // Event emission implementation
    }
}

/// Helper for transforming crawl results (WIP - behind `events` feature)
#[cfg(feature = "events")]
pub struct ResultTransformer;

#[cfg(feature = "events")]
impl ResultTransformer {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "events")]
impl Default for ResultTransformer {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper for building spider configurations
pub struct SpiderConfigBuilder<'a> {
    _state: &'a ApplicationContext,
    seed_url: url::Url,
    #[allow(dead_code)] // Reserved for future spider configuration
    max_depth: Option<usize>,
    #[allow(dead_code)] // Reserved for future spider configuration
    max_pages: Option<usize>,
    #[allow(dead_code)] // Reserved for future spider configuration
    strategy: Option<String>,
    #[allow(dead_code)] // Reserved for future spider configuration
    concurrency: Option<usize>,
}

impl<'a> SpiderConfigBuilder<'a> {
    pub fn new(state: &'a ApplicationContext, seed_url: url::Url) -> Self {
        Self {
            _state: state,
            seed_url,
            max_depth: None,
            max_pages: None,
            strategy: None,
            concurrency: None,
        }
    }

    #[allow(dead_code)] // Reserved for future spider configuration
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = Some(max_depth);
        self
    }

    #[allow(dead_code)] // Reserved for future spider configuration
    pub fn with_max_pages(mut self, max_pages: usize) -> Self {
        self.max_pages = Some(max_pages);
        self
    }

    #[allow(dead_code)] // Reserved for future spider configuration
    pub fn with_strategy(mut self, strategy: &str) -> Self {
        self.strategy = Some(strategy.to_string());
        self
    }

    #[allow(clippy::wrong_self_convention)] // Builder pattern - applies options to existing builder
    pub fn from_crawl_options(mut self, options: &riptide_types::config::CrawlOptions) -> Self {
        // Apply CrawlOptions to the spider configuration builder

        // Map spider-specific depth option
        if let Some(depth) = options.spider_max_depth {
            self.max_depth = Some(depth);
        }

        // Map spider strategy option
        if let Some(ref strategy) = options.spider_strategy {
            self.strategy = Some(strategy.clone());
        }

        // Map concurrency option
        self.concurrency = Some(options.concurrency);

        self
    }

    #[allow(dead_code)] // Reserved for future spider configuration
    #[cfg(feature = "spider")]
    pub fn build(self) -> Result<riptide_spider::SpiderConfig, ApiError> {
        // Start with base configuration
        let mut config = riptide_spider::SpiderConfig::new(self.seed_url);

        // Apply max_depth if set
        if let Some(depth) = self.max_depth {
            config = config.with_max_depth(Some(depth));
        }

        // Apply max_pages if set
        if let Some(pages) = self.max_pages {
            config = config.with_max_pages(Some(pages));
        }

        // Apply concurrency if set
        if let Some(concurrency) = self.concurrency {
            config = config.with_concurrency(concurrency);
        }

        // Note: Strategy application would require additional SpiderConfig API
        // The strategy field is stored but not yet applied to the config
        // This can be extended when SpiderConfig provides a strategy setter

        Ok(config)
    }
}
