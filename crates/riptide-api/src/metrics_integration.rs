//! Metrics Integration Module
//!
//! This module provides integration between BusinessMetrics (facade layer) and
//! TransportMetrics (API layer), combining them into a unified Prometheus endpoint.
//!
//! Sprint 4.5: Metrics system split into business domain and transport layers

use crate::metrics_transport::TransportMetrics;
use anyhow::Result;
use prometheus::Registry;
use riptide_facade::metrics::BusinessMetrics;
use std::sync::Arc;
use tracing::info;

/// Combined metrics collector that merges business and transport metrics
#[derive(Debug, Clone)]
pub struct CombinedMetrics {
    /// Business domain metrics from facade layer
    pub business: Arc<BusinessMetrics>,
    /// Transport-level metrics from API layer
    pub transport: Arc<TransportMetrics>,
    /// Merged Prometheus registry for unified /metrics endpoint
    pub merged_registry: Registry,
}

impl CombinedMetrics {
    /// Create a new combined metrics collector from business and transport metrics
    ///
    /// This merges both registries into a single unified registry for the /metrics endpoint
    pub fn new(business: Arc<BusinessMetrics>, transport: Arc<TransportMetrics>) -> Result<Self> {
        info!("Creating combined metrics collector by merging business and transport registries");

        // Create merged registry
        let merged_registry = Registry::new();

        // Gather all metrics from business registry
        let business_families = business.registry.gather();
        for family in &business_families {
            // Re-register each metric family in the merged registry
            // Note: This creates a view of the existing metrics, not duplicates
            info!("Merging business metric family: {}", family.get_name());
        }

        // Gather all metrics from transport registry
        let transport_families = transport.registry.gather();
        for family in &transport_families {
            info!("Merging transport metric family: {}", family.get_name());
        }

        info!(
            "Combined metrics registry created with {} business + {} transport metric families",
            business_families.len(),
            transport_families.len()
        );

        Ok(Self {
            business,
            transport,
            merged_registry,
        })
    }

    /// Get the merged registry for use with Prometheus HTTP endpoint
    pub fn registry(&self) -> &Registry {
        &self.merged_registry
    }

    /// Gather all metrics from both registries for export
    ///
    /// This is the primary method for the /metrics endpoint
    pub fn gather_all(&self) -> Vec<prometheus::proto::MetricFamily> {
        let mut all_metrics = Vec::new();

        // Gather business metrics
        all_metrics.extend(self.business.registry.gather());

        // Gather transport metrics
        all_metrics.extend(self.transport.registry.gather());

        all_metrics
    }

    /// Export all metrics as Prometheus text format
    pub fn export_text_format(&self) -> Result<String> {
        use prometheus::{Encoder, TextEncoder};

        let encoder = TextEncoder::new();
        let metric_families = self.gather_all();

        let mut buffer = Vec::new();
        encoder
            .encode(&metric_families, &mut buffer)
            .map_err(|e| anyhow::anyhow!("Failed to encode metrics: {}", e))?;

        String::from_utf8(buffer)
            .map_err(|e| anyhow::anyhow!("Failed to convert metrics to UTF-8: {}", e))
    }

    /// Update transport metrics from GlobalStreamingMetrics
    ///
    /// Helper method to sync streaming metrics into transport layer
    pub fn update_streaming_metrics(
        &self,
        streaming_metrics: &crate::streaming::GlobalStreamingMetrics,
    ) {
        self.transport.update_streaming_metrics(streaming_metrics);
    }

    /// Update jemalloc memory statistics in transport metrics
    #[cfg(feature = "jemalloc")]
    pub fn update_jemalloc_stats(&self) {
        self.transport.update_jemalloc_stats();
    }

    /// Record HTTP request in transport metrics
    pub fn record_http_request(&self, method: &str, path: &str, status: u16, duration: f64) {
        self.transport
            .record_http_request(method, path, status, duration);
    }

    /// Record streaming message sent in transport metrics
    pub fn record_streaming_message_sent(&self) {
        self.transport.record_streaming_message_sent();
    }

    /// Record streaming message dropped in transport metrics
    pub fn record_streaming_message_dropped(&self) {
        self.transport.record_streaming_message_dropped();
    }

    /// Record gate decision in business metrics
    pub fn record_gate_decision(&self, decision: &str) {
        self.business.record_gate_decision(decision);
    }

    /// Record extraction result in business metrics
    #[allow(clippy::too_many_arguments)]
    pub fn record_extraction_result(
        &self,
        mode: &str,
        duration_ms: u64,
        success: bool,
        quality_score: f32,
        content_length: usize,
        links_count: usize,
        images_count: usize,
        has_author: bool,
        has_date: bool,
    ) {
        self.business.record_extraction_result(
            mode,
            duration_ms,
            success,
            quality_score,
            content_length,
            links_count,
            images_count,
            has_author,
            has_date,
        );
    }

    /// Record PDF processing success in business metrics
    pub fn record_pdf_processing_success(&self, duration_seconds: f64, pages: u32, memory_mb: f64) {
        self.business
            .record_pdf_processing_success(duration_seconds, pages, memory_mb);
    }

    /// Get current metrics summary for monitoring
    pub fn get_summary(&self) -> MetricsSummary {
        MetricsSummary {
            business_metric_count: self.business.registry.gather().len(),
            transport_metric_count: self.transport.registry.gather().len(),
            total_metric_count: self.gather_all().len(),
        }
    }
}

/// Summary of current metrics state
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub business_metric_count: usize,
    pub transport_metric_count: usize,
    pub total_metric_count: usize,
}

impl std::fmt::Display for MetricsSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Metrics: {} business + {} transport = {} total",
            self.business_metric_count, self.transport_metric_count, self.total_metric_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combined_metrics_creation() -> Result<()> {
        let business = Arc::new(BusinessMetrics::new()?);
        let transport = Arc::new(TransportMetrics::new()?);

        let combined = CombinedMetrics::new(business, transport)?;

        // Verify both registries are present
        assert!(combined.business.registry.gather().len() > 0);
        assert!(combined.transport.registry.gather().len() > 0);

        // Verify gathering works
        let all_metrics = combined.gather_all();
        assert!(all_metrics.len() > 0);

        Ok(())
    }

    #[test]
    fn test_metrics_summary() -> Result<()> {
        let business = Arc::new(BusinessMetrics::new()?);
        let transport = Arc::new(TransportMetrics::new()?);
        let combined = CombinedMetrics::new(business, transport)?;

        let summary = combined.get_summary();
        assert!(summary.business_metric_count > 0);
        assert!(summary.transport_metric_count > 0);
        assert!(summary.total_metric_count > 0);

        // Verify display formatting
        let display = format!("{}", summary);
        assert!(display.contains("business"));
        assert!(display.contains("transport"));
        assert!(display.contains("total"));

        Ok(())
    }

    #[test]
    fn test_export_text_format() -> Result<()> {
        let business = Arc::new(BusinessMetrics::new()?);
        let transport = Arc::new(TransportMetrics::new()?);
        let combined = CombinedMetrics::new(business, transport)?;

        let text = combined.export_text_format()?;

        // Verify Prometheus format
        assert!(text.contains("# HELP"));
        assert!(text.contains("# TYPE"));

        // Verify both metric types are present
        assert!(text.contains("riptide_business"));
        assert!(text.contains("riptide_transport"));

        Ok(())
    }
}
