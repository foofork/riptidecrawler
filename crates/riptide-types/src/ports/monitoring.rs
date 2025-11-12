//! Monitoring and observability port for hexagonal architecture
//!
//! Provides backend-agnostic traits for system monitoring, health checks,
//! and performance tracking.

use crate::error::Result as RiptideResult;
use async_trait::async_trait;

/// Performance tracking port trait
///
/// Defines the interface for performance monitoring and profiling.
#[async_trait]
pub trait PerformanceTracker: Send + Sync {
    /// Start monitoring
    ///
    /// # Returns
    /// * `Ok(())` - Monitoring started successfully
    /// * `Err(_)` - Failed to start monitoring
    async fn start_monitoring(&self) -> RiptideResult<()>;

    /// Stop monitoring
    async fn stop_monitoring(&self) -> RiptideResult<()>;

    /// Check if monitoring is active
    ///
    /// # Returns
    /// `true` if performance monitoring is running
    async fn is_monitoring(&self) -> bool;

    /// Get current memory usage in bytes
    async fn memory_usage(&self) -> usize;

    /// Get current CPU usage percentage
    async fn cpu_usage(&self) -> f64;
}

/// Transport metrics port trait
///
/// Tracks transport-level metrics (HTTP, WebSocket, SSE protocols).
pub trait TransportMetrics: Send + Sync {
    /// Record HTTP request
    ///
    /// # Arguments
    /// * `method` - HTTP method
    /// * `path` - Request path
    /// * `status` - Status code
    /// * `duration_secs` - Request duration in seconds
    fn record_http_request(&self, method: &str, path: &str, status: u16, duration_secs: f64);

    /// Record HTTP error
    fn record_http_error(&self);

    /// Record WASM error
    fn record_wasm_error(&self);

    /// Record Redis error
    fn record_redis_error(&self);

    /// Get prometheus registry
    fn registry(&self) -> &prometheus::Registry;
}

/// Combined metrics port trait
///
/// Merges business and transport metrics for unified export.
pub trait CombinedMetricsCollector: Send + Sync {
    /// Get the merged prometheus registry
    ///
    /// # Returns
    /// Registry containing all business and transport metrics
    fn registry(&self) -> &prometheus::Registry;

    /// Get metrics snapshot as text
    ///
    /// # Returns
    /// Prometheus-format metrics for scraping
    fn snapshot(&self) -> RiptideResult<String>;
}

/// PDF processing metrics port trait
pub trait PdfMetrics: Send + Sync {
    /// Record PDF processing success
    ///
    /// # Arguments
    /// * `pages` - Number of pages processed
    /// * `duration_secs` - Processing duration in seconds
    fn record_success(&self, pages: usize, duration_secs: f64);

    /// Record PDF processing failure
    fn record_failure(&self);

    /// Get total PDFs processed
    fn total_processed(&self) -> u64;
}

/// Monitoring system port trait
///
/// Comprehensive monitoring with alerting and health scoring.
pub trait MonitoringSystem: Send + Sync {
    /// Calculate current health score
    ///
    /// # Returns
    /// Health score from 0.0 (critical) to 1.0 (healthy)
    fn health_score(&self) -> impl std::future::Future<Output = RiptideResult<f32>> + Send;

    /// Get current system status
    ///
    /// # Returns
    /// Human-readable status message
    fn status(&self) -> impl std::future::Future<Output = String> + Send;
}
