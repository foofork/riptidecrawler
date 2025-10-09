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
    pub fn new(state: &'a AppState) -> Self {
        Self { _state: state }
    }

    pub fn emit_event(&self, _event_type: &str, _data: serde_json::Value) {
        // Event emission implementation
    }
}

/// Helper for transforming crawl results (WIP - behind `events` feature)
#[cfg(feature = "events")]
pub struct ResultTransformer;

#[cfg(feature = "events")]
impl ResultTransformer {
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
    #[allow(dead_code)] // Reserved for future spider configuration
    seed_url: url::Url,
}

impl<'a> SpiderConfigBuilder<'a> {
    pub fn new(state: &'a AppState, seed_url: url::Url) -> Self {
        Self {
            _state: state,
            seed_url,
        }
    }

    #[allow(dead_code)] // Reserved for future spider configuration
    pub fn with_max_depth(self, _max_depth: usize) -> Self {
        self
    }

    #[allow(dead_code)] // Reserved for future spider configuration
    pub fn with_max_pages(self, _max_pages: usize) -> Self {
        self
    }

    #[allow(dead_code)] // Reserved for future spider configuration
    pub fn with_strategy(self, _strategy: &str) -> Self {
        self
    }

    #[allow(clippy::wrong_self_convention)] // Builder pattern - applies options to existing builder
    pub fn from_crawl_options(self, _options: &riptide_core::types::CrawlOptions) -> Self {
        // Apply options to the builder
        // TODO(P1): Apply CrawlOptions to spider config
        // PLAN: Map CrawlOptions fields to SpiderConfig
        // IMPLEMENTATION:
        //   1. Map depth limit from CrawlOptions
        //   2. Apply URL patterns and exclusion rules
        //   3. Set concurrency and rate limiting options
        //   4. Configure respect_robots_txt flag
        //   5. Apply custom headers and authentication
        // DEPENDENCIES: None - both types are available
        // EFFORT: Low (2-3 hours)
        // PRIORITY: Required for full spider functionality
        // BLOCKER: None
        self
    }

    #[allow(dead_code)] // Reserved for future spider configuration
    pub fn build(self) -> Result<riptide_core::spider::SpiderConfig, ApiError> {
        Ok(riptide_core::spider::SpiderConfig::new(self.seed_url))
    }
}
