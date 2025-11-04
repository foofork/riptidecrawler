//! Shared utilities for handlers to reduce code duplication

pub mod spider;

use crate::errors::ApiError;
use crate::state::AppState;

/// Helper for recording metrics in handlers
pub struct MetricsRecorder<'a> {
    state: &'a AppState,
}

impl<'a> MetricsRecorder<'a> {
    pub fn new(state: &'a AppState) -> Self {
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
        // Record spider crawl completion metrics
        self.state.metrics.record_spider_crawl_completion(
            pages_crawled,
            pages_failed,
            duration.as_secs_f64(),
        );
    }

    pub fn record_spider_crawl_failure(&self) {
        // Spider crawl failure - record as failed without completion
        self.state.metrics.spider_active_crawls.dec();
    }

    pub fn update_frontier_size(&self, size: usize) {
        // Update spider frontier size gauge
        self.state.metrics.update_spider_frontier_size(size);
    }

    pub fn record_http_request(
        &self,
        method: &str,
        path: &str,
        status: u16,
        duration: std::time::Duration,
    ) {
        // Record HTTP request metrics
        self.state
            .metrics
            .record_http_request(method, path, status, duration.as_secs_f64());
    }
}

/// Helper for emitting events in handlers (WIP - behind `events` feature)
#[cfg(feature = "events")]
pub struct EventEmitter<'a> {
    _state: &'a AppState,
}

#[cfg(feature = "events")]
impl<'a> EventEmitter<'a> {
    #[allow(dead_code)]
    pub fn new(state: &'a AppState) -> Self {
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
    _state: &'a AppState,
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
    pub fn new(state: &'a AppState, seed_url: url::Url) -> Self {
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
