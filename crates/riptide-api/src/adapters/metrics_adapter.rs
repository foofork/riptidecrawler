//! Metrics Collector Adapter for hexagonal architecture
//!
//! Consolidates multiple metrics collectors into a single unified interface.

use riptide_types::ports::monitoring::{TransportMetrics, CombinedMetricsCollector};
use riptide_types::error::Result as RiptideResult;
use std::sync::Arc;

/// Unified metrics collector that combines business and transport metrics
pub struct MetricsCollectorAdapter {
    business_metrics: Arc<riptide_facade::metrics::BusinessMetrics>,
    transport_metrics: Arc<crate::metrics_transport::TransportMetrics>,
    combined_metrics: Arc<crate::metrics_integration::CombinedMetrics>,
}

impl MetricsCollectorAdapter {
    /// Create a new MetricsCollectorAdapter with all metrics collectors
    pub fn new(
        business_metrics: Arc<riptide_facade::metrics::BusinessMetrics>,
        transport_metrics: Arc<crate::metrics_transport::TransportMetrics>,
        combined_metrics: Arc<crate::metrics_integration::CombinedMetrics>,
    ) -> Self {
        Self {
            business_metrics,
            transport_metrics,
            combined_metrics,
        }
    }

    /// Get business metrics reference
    pub fn business_metrics(&self) -> &Arc<riptide_facade::metrics::BusinessMetrics> {
        &self.business_metrics
    }

    /// Get transport metrics reference
    pub fn transport_metrics(&self) -> &Arc<crate::metrics_transport::TransportMetrics> {
        &self.transport_metrics
    }
}

impl TransportMetrics for MetricsCollectorAdapter {
    fn record_http_request(&self, method: &str, path: &str, status: u16, duration_secs: f64) {
        self.transport_metrics
            .record_http_request(method, path, status, duration_secs);
    }

    fn record_http_error(&self) {
        self.transport_metrics.record_http_error();
    }

    fn record_wasm_error(&self) {
        self.transport_metrics.record_wasm_error();
    }

    fn record_redis_error(&self) {
        self.transport_metrics.record_redis_error();
    }

    fn registry(&self) -> &prometheus::Registry {
        self.transport_metrics.registry()
    }
}

impl CombinedMetricsCollector for MetricsCollectorAdapter {
    fn registry(&self) -> &prometheus::Registry {
        self.combined_metrics.registry()
    }

    fn snapshot(&self) -> RiptideResult<String> {
        let encoder = prometheus::TextEncoder::new();
        let metric_families = self.registry().gather();

        encoder
            .encode_to_string(&metric_families)
            .map_err(|e| riptide_types::error::RiptideError::Internal {
                message: format!("Failed to encode metrics: {}", e),
            })
    }
}
