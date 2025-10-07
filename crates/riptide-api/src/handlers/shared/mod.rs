//! Shared utilities for handlers to reduce code duplication

pub mod spider;

use crate::state::AppState;
use crate::errors::ApiError;

/// Helper for recording metrics in handlers
pub struct MetricsRecorder<'a> {
    _state: &'a AppState,
}

impl<'a> MetricsRecorder<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { _state: state }
    }

    pub fn record_request(&self, _endpoint: &str) {
        // Metrics recording implementation
        // This can be expanded as metrics infrastructure is activated
    }

    pub fn record_success(&self, _endpoint: &str, _duration_ms: u64) {
        // Success metrics
    }

    pub fn record_error(&self, _endpoint: &str, _error_type: &str) {
        // Error metrics
    }

    pub fn record_spider_crawl(&self, _pages_crawled: u64, _pages_failed: u64, _duration: std::time::Duration) {
        // Spider crawl metrics
    }

    pub fn record_spider_crawl_failure(&self) {
        // Spider crawl failure metrics
    }

    pub fn update_frontier_size(&self, _size: usize) {
        // Frontier size metrics
    }

    pub fn record_http_request(&self, _method: &str, _path: &str, _status: u16, _duration: std::time::Duration) {
        // HTTP request metrics
    }
}

/// Helper for emitting events in handlers
pub struct EventEmitter<'a> {
    _state: &'a AppState,
}

impl<'a> EventEmitter<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { _state: state }
    }

    pub fn emit_event(&self, _event_type: &str, _data: serde_json::Value) {
        // Event emission implementation
    }
}

/// Helper for transforming crawl results
pub struct ResultTransformer;

impl ResultTransformer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ResultTransformer {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper for building spider configurations
pub struct SpiderConfigBuilder<'a> {
    _state: &'a AppState,
    seed_url: url::Url,
}

impl<'a> SpiderConfigBuilder<'a> {
    pub fn new(state: &'a AppState, seed_url: url::Url) -> Self {
        Self { _state: state, seed_url }
    }

    pub fn with_max_depth(self, _max_depth: usize) -> Self {
        self
    }

    pub fn with_max_pages(self, _max_pages: usize) -> Self {
        self
    }

    pub fn with_strategy(self, _strategy: &str) -> Self {
        self
    }

    pub fn from_crawl_options(self, _options: &riptide_core::types::CrawlOptions) -> Self {
        // Apply options to the builder
        // TODO: Apply CrawlOptions to spider config
        self
    }

    pub fn build(self) -> Result<riptide_core::spider::SpiderConfig, ApiError> {
        Ok(riptide_core::spider::SpiderConfig::new(self.seed_url))
    }
}
