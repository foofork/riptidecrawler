//! Composite Metrics Adapter (Priority 2: Metrics Consolidation)
//!
//! This adapter consolidates 5 metrics types into a single MetricsCollector implementation:
//! 1. BusinessMetrics (riptide-facade) - extraction, gate, cache, PDF
//! 2. TransportMetrics (riptide-api) - HTTP, WebSocket, streaming
//! 3. CombinedMetrics (riptide-api) - unified /metrics endpoint
//! 4. PdfMetricsCollector (riptide-pdf) - PDF-specific metrics
//! 5. PerformanceMetrics (riptide-performance) - system resources
//!
//! Design: FACADE_DETOX_PLAN.md Priority 2
//!
//! Routes metrics to appropriate internal collectors based on name prefixes:
//! - `business.*` → BusinessMetrics
//! - `transport.*` → TransportMetrics
//! - `pdf.*` → PdfMetricsCollector
//! - `perf.*` → PerformanceMetrics

use async_trait::async_trait;
use riptide_types::ports::metrics::{
    ExtractionResult, GateFeatures, MetricLabels, MetricsCollector, MetricsSnapshot,
    PdfProcessingResult, StreamingMetrics, SystemMetrics,
};
use std::sync::Arc;
use std::time::Duration;
use tracing::warn;

/// Composite adapter that routes metrics to appropriate collectors
pub struct CompositeMetricsAdapter {
    /// Business domain metrics (extraction, gate, cache)
    business: Option<Arc<dyn BusinessMetricsPort>>,
    /// Transport-level metrics (HTTP, WebSocket, streaming)
    transport: Option<Arc<dyn TransportMetricsPort>>,
    /// PDF-specific metrics collector
    pdf: Option<Arc<dyn PdfMetricsPort>>,
    /// Performance/system metrics
    performance: Option<Arc<dyn PerformanceMetricsPort>>,
}

impl CompositeMetricsAdapter {
    /// Create new composite adapter with all collectors
    pub fn new(
        business: Option<Arc<dyn BusinessMetricsPort>>,
        transport: Option<Arc<dyn TransportMetricsPort>>,
        pdf: Option<Arc<dyn PdfMetricsPort>>,
        performance: Option<Arc<dyn PerformanceMetricsPort>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            business,
            transport,
            pdf,
            performance,
        })
    }

    /// Route metric based on name prefix
    fn route_metric(&self, name: &str) -> MetricCategory {
        if name.starts_with("business.") {
            MetricCategory::Business
        } else if name.starts_with("transport.") {
            MetricCategory::Transport
        } else if name.starts_with("pdf.") {
            MetricCategory::Pdf
        } else if name.starts_with("perf.") {
            MetricCategory::Performance
        } else {
            warn!("Unknown metric category for: {}", name);
            MetricCategory::Unknown
        }
    }
}

#[async_trait]
impl MetricsCollector for CompositeMetricsAdapter {
    async fn record_counter(&self, name: &str, value: u64, labels: MetricLabels) {
        match self.route_metric(name) {
            MetricCategory::Business => {
                if let Some(ref collector) = self.business {
                    collector.record_counter(name, value, labels).await;
                }
            }
            MetricCategory::Transport => {
                if let Some(ref collector) = self.transport {
                    collector.record_counter(name, value, labels).await;
                }
            }
            MetricCategory::Pdf => {
                if let Some(ref collector) = self.pdf {
                    collector.record_counter(name, value, labels).await;
                }
            }
            MetricCategory::Performance => {
                if let Some(ref collector) = self.performance {
                    collector.record_counter(name, value, labels).await;
                }
            }
            MetricCategory::Unknown => {
                warn!("Cannot route counter metric: {}", name);
            }
        }
    }

    async fn record_gauge(&self, name: &str, value: f64, labels: MetricLabels) {
        match self.route_metric(name) {
            MetricCategory::Business => {
                if let Some(ref collector) = self.business {
                    collector.record_gauge(name, value, labels).await;
                }
            }
            MetricCategory::Transport => {
                if let Some(ref collector) = self.transport {
                    collector.record_gauge(name, value, labels).await;
                }
            }
            MetricCategory::Pdf => {
                if let Some(ref collector) = self.pdf {
                    collector.record_gauge(name, value, labels).await;
                }
            }
            MetricCategory::Performance => {
                if let Some(ref collector) = self.performance {
                    collector.record_gauge(name, value, labels).await;
                }
            }
            MetricCategory::Unknown => {
                warn!("Cannot route gauge metric: {}", name);
            }
        }
    }

    async fn record_histogram(&self, name: &str, value: f64, labels: MetricLabels) {
        match self.route_metric(name) {
            MetricCategory::Business => {
                if let Some(ref collector) = self.business {
                    collector.record_histogram(name, value, labels).await;
                }
            }
            MetricCategory::Transport => {
                if let Some(ref collector) = self.transport {
                    collector.record_histogram(name, value, labels).await;
                }
            }
            MetricCategory::Pdf => {
                if let Some(ref collector) = self.pdf {
                    collector.record_histogram(name, value, labels).await;
                }
            }
            MetricCategory::Performance => {
                if let Some(ref collector) = self.performance {
                    collector.record_histogram(name, value, labels).await;
                }
            }
            MetricCategory::Unknown => {
                warn!("Cannot route histogram metric: {}", name);
            }
        }
    }

    async fn record_gate_decision(
        &self,
        decision_type: &str,
        score: f32,
        features: Option<GateFeatures>,
    ) {
        if let Some(ref business) = self.business {
            business
                .record_gate_decision(decision_type, score, features)
                .await;
        }
    }

    async fn record_extraction_result(&self, result: ExtractionResult) {
        if let Some(ref business) = self.business {
            business.record_extraction_result(result).await;
        }
    }

    async fn record_extraction_fallback(&self, from_mode: &str, to_mode: &str, reason: &str) {
        if let Some(ref business) = self.business {
            business
                .record_extraction_fallback(from_mode, to_mode, reason)
                .await;
        }
    }

    async fn record_pdf_processing(&self, result: PdfProcessingResult) {
        if let Some(ref pdf) = self.pdf {
            pdf.record_pdf_processing(result).await;
        }
    }

    async fn record_cache_access(&self, key_type: &str, hit: bool) {
        if let Some(ref business) = self.business {
            business.record_cache_access(key_type, hit).await;
        }
    }

    async fn record_http_request(&self, method: &str, path: &str, status: u16, duration: Duration) {
        if let Some(ref transport) = self.transport {
            transport
                .record_http_request(method, path, status, duration)
                .await;
        }
    }

    async fn record_streaming_operation(&self, operation: &str, metrics: StreamingMetrics) {
        if let Some(ref transport) = self.transport {
            transport
                .record_streaming_operation(operation, metrics)
                .await;
        }
    }

    async fn record_system_metrics(&self, metrics: SystemMetrics) {
        if let Some(ref performance) = self.performance {
            performance.record_system_metrics(metrics).await;
        }
    }

    async fn record_memory_event(&self, event_type: &str, memory_mb: f64) {
        if let Some(ref performance) = self.performance {
            performance.record_memory_event(event_type, memory_mb).await;
        }
    }

    async fn snapshot(&self) -> MetricsSnapshot {
        let mut combined = MetricsSnapshot::new();

        // Aggregate snapshots from all collectors
        if let Some(ref business) = self.business {
            let snapshot = business.snapshot().await;
            combined.merge(snapshot);
        }

        if let Some(ref transport) = self.transport {
            let snapshot = transport.snapshot().await;
            combined.merge(snapshot);
        }

        if let Some(ref pdf) = self.pdf {
            let snapshot = pdf.snapshot().await;
            combined.merge(snapshot);
        }

        if let Some(ref performance) = self.performance {
            let snapshot = performance.snapshot().await;
            combined.merge(snapshot);
        }

        combined
    }

    async fn export_prometheus(&self) -> anyhow::Result<String> {
        let mut all_metrics = String::new();

        // Export from all collectors
        if let Some(ref business) = self.business {
            match business.export_prometheus().await {
                Ok(metrics) => all_metrics.push_str(&metrics),
                Err(e) => warn!("Failed to export business metrics: {}", e),
            }
        }

        if let Some(ref transport) = self.transport {
            match transport.export_prometheus().await {
                Ok(metrics) => all_metrics.push_str(&metrics),
                Err(e) => warn!("Failed to export transport metrics: {}", e),
            }
        }

        if let Some(ref pdf) = self.pdf {
            match pdf.export_prometheus().await {
                Ok(metrics) => all_metrics.push_str(&metrics),
                Err(e) => warn!("Failed to export PDF metrics: {}", e),
            }
        }

        if let Some(ref performance) = self.performance {
            match performance.export_prometheus().await {
                Ok(metrics) => all_metrics.push_str(&metrics),
                Err(e) => warn!("Failed to export performance metrics: {}", e),
            }
        }

        Ok(all_metrics)
    }
}

/// Metric category for routing
#[derive(Debug, Clone, Copy)]
enum MetricCategory {
    Business,
    Transport,
    Pdf,
    Performance,
    Unknown,
}

// ===== Port Traits for Internal Collectors =====
//
// These traits define the interface for existing metrics collectors.
// They allow the adapter to work with existing implementations without tight coupling.

#[async_trait]
pub trait BusinessMetricsPort: Send + Sync {
    async fn record_counter(&self, name: &str, value: u64, labels: MetricLabels);
    async fn record_gauge(&self, name: &str, value: f64, labels: MetricLabels);
    async fn record_histogram(&self, name: &str, value: f64, labels: MetricLabels);
    async fn record_gate_decision(
        &self,
        decision_type: &str,
        score: f32,
        features: Option<GateFeatures>,
    );
    async fn record_extraction_result(&self, result: ExtractionResult);
    async fn record_extraction_fallback(&self, from_mode: &str, to_mode: &str, reason: &str);
    async fn record_cache_access(&self, key_type: &str, hit: bool);
    async fn snapshot(&self) -> MetricsSnapshot;
    async fn export_prometheus(&self) -> anyhow::Result<String>;
}

#[async_trait]
pub trait TransportMetricsPort: Send + Sync {
    async fn record_counter(&self, name: &str, value: u64, labels: MetricLabels);
    async fn record_gauge(&self, name: &str, value: f64, labels: MetricLabels);
    async fn record_histogram(&self, name: &str, value: f64, labels: MetricLabels);
    async fn record_http_request(&self, method: &str, path: &str, status: u16, duration: Duration);
    async fn record_streaming_operation(&self, operation: &str, metrics: StreamingMetrics);
    async fn snapshot(&self) -> MetricsSnapshot;
    async fn export_prometheus(&self) -> anyhow::Result<String>;
}

#[async_trait]
pub trait PdfMetricsPort: Send + Sync {
    async fn record_counter(&self, name: &str, value: u64, labels: MetricLabels);
    async fn record_gauge(&self, name: &str, value: f64, labels: MetricLabels);
    async fn record_histogram(&self, name: &str, value: f64, labels: MetricLabels);
    async fn record_pdf_processing(&self, result: PdfProcessingResult);
    async fn snapshot(&self) -> MetricsSnapshot;
    async fn export_prometheus(&self) -> anyhow::Result<String>;
}

#[async_trait]
pub trait PerformanceMetricsPort: Send + Sync {
    async fn record_counter(&self, name: &str, value: u64, labels: MetricLabels);
    async fn record_gauge(&self, name: &str, value: f64, labels: MetricLabels);
    async fn record_histogram(&self, name: &str, value: f64, labels: MetricLabels);
    async fn record_system_metrics(&self, metrics: SystemMetrics);
    async fn record_memory_event(&self, event_type: &str, memory_mb: f64);
    async fn snapshot(&self) -> MetricsSnapshot;
    async fn export_prometheus(&self) -> anyhow::Result<String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockBusinessMetrics;

    #[async_trait]
    impl BusinessMetricsPort for MockBusinessMetrics {
        async fn record_counter(&self, _name: &str, _value: u64, _labels: MetricLabels) {}
        async fn record_gauge(&self, _name: &str, _value: f64, _labels: MetricLabels) {}
        async fn record_histogram(&self, _name: &str, _value: f64, _labels: MetricLabels) {}
        async fn record_gate_decision(
            &self,
            _decision_type: &str,
            _score: f32,
            _features: Option<GateFeatures>,
        ) {
        }
        async fn record_extraction_result(&self, _result: ExtractionResult) {}
        async fn record_extraction_fallback(&self, _from: &str, _to: &str, _reason: &str) {}
        async fn record_cache_access(&self, _key_type: &str, _hit: bool) {}
        async fn snapshot(&self) -> MetricsSnapshot {
            MetricsSnapshot::new()
        }
        async fn export_prometheus(&self) -> anyhow::Result<String> {
            Ok("# Business metrics\n".to_string())
        }
    }

    #[tokio::test]
    async fn test_composite_adapter_routes_metrics() {
        let business = Arc::new(MockBusinessMetrics);
        let adapter = CompositeMetricsAdapter::new(Some(business), None, None, None);

        // Test routing business metrics
        adapter
            .record_counter("business.test_counter", 1, HashMap::new())
            .await;
        adapter
            .record_gauge("business.test_gauge", 42.0, HashMap::new())
            .await;

        // Should not crash on unknown metrics
        adapter
            .record_counter("unknown.test", 1, HashMap::new())
            .await;
    }

    #[tokio::test]
    async fn test_snapshot_aggregation() {
        let business = Arc::new(MockBusinessMetrics);
        let adapter = CompositeMetricsAdapter::new(Some(business), None, None, None);

        let snapshot = adapter.snapshot().await;
        assert_eq!(snapshot.counters.len(), 0); // MockBusinessMetrics returns empty snapshot
    }

    #[tokio::test]
    async fn test_export_prometheus() {
        let business = Arc::new(MockBusinessMetrics);
        let adapter = CompositeMetricsAdapter::new(Some(business), None, None, None);

        let export = adapter.export_prometheus().await.unwrap();
        assert!(export.contains("Business metrics"));
    }
}
