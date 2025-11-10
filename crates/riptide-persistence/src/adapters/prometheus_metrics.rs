//! Prometheus metrics adapter implementation
//!
//! This module provides a production-ready metrics collector using Prometheus.
//! It implements both MetricsCollector and BusinessMetrics traits.

use prometheus::{CounterVec, GaugeVec, HistogramOpts, HistogramVec, Opts, Registry};
use riptide_types::ports::metrics::{BusinessMetrics, MetricsCollector};
use std::sync::Arc;
use std::time::Duration;

/// Prometheus-based metrics collector
pub struct PrometheusMetrics {
    registry: Arc<Registry>,
    // Low-level metric types
    counters: Arc<parking_lot::RwLock<std::collections::HashMap<String, CounterVec>>>,
    histograms: Arc<parking_lot::RwLock<std::collections::HashMap<String, HistogramVec>>>,
    gauges: Arc<parking_lot::RwLock<std::collections::HashMap<String, GaugeVec>>>,
    // Business metrics
    extraction_duration: HistogramVec,
    extraction_total: CounterVec,
    cache_hits: CounterVec,
    cache_misses: CounterVec,
    events_published: CounterVec,
    http_request_duration: HistogramVec,
    pipeline_stage_duration: HistogramVec,
    errors_total: CounterVec,
}

impl PrometheusMetrics {
    /// Creates a new Prometheus metrics collector
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let registry = Registry::new();

        // Initialize business metrics
        let extraction_duration = HistogramVec::new(
            HistogramOpts::new(
                "riptide_extraction_duration_seconds",
                "Duration of data extraction operations",
            ),
            &["success"],
        )?;
        registry.register(Box::new(extraction_duration.clone()))?;

        let extraction_total = CounterVec::new(
            Opts::new("riptide_extractions_total", "Total number of extractions"),
            &["success"],
        )?;
        registry.register(Box::new(extraction_total.clone()))?;

        let cache_hits = CounterVec::new(
            Opts::new("riptide_cache_hits_total", "Total cache hits"),
            &["type"],
        )?;
        registry.register(Box::new(cache_hits.clone()))?;

        let cache_misses = CounterVec::new(
            Opts::new("riptide_cache_misses_total", "Total cache misses"),
            &["type"],
        )?;
        registry.register(Box::new(cache_misses.clone()))?;

        let events_published = CounterVec::new(
            Opts::new("riptide_events_published_total", "Total events published"),
            &["type"],
        )?;
        registry.register(Box::new(events_published.clone()))?;

        let http_request_duration = HistogramVec::new(
            HistogramOpts::new(
                "riptide_http_request_duration_seconds",
                "HTTP request duration",
            ),
            &["method", "status"],
        )?;
        registry.register(Box::new(http_request_duration.clone()))?;

        let pipeline_stage_duration = HistogramVec::new(
            HistogramOpts::new(
                "riptide_pipeline_stage_duration_seconds",
                "Pipeline stage duration",
            ),
            &["stage", "success"],
        )?;
        registry.register(Box::new(pipeline_stage_duration.clone()))?;

        let errors_total = CounterVec::new(
            Opts::new("riptide_errors_total", "Total errors by type"),
            &["type", "context"],
        )?;
        registry.register(Box::new(errors_total.clone()))?;

        Ok(Self {
            registry: Arc::new(registry),
            counters: Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new())),
            histograms: Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new())),
            gauges: Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new())),
            extraction_duration,
            extraction_total,
            cache_hits,
            cache_misses,
            events_published,
            http_request_duration,
            pipeline_stage_duration,
            errors_total,
        })
    }

    /// Gets or creates a counter metric
    fn get_or_create_counter(&self, name: &str, help: &str, labels: &[&str]) -> CounterVec {
        let mut counters = self.counters.write();
        counters
            .entry(name.to_string())
            .or_insert_with(|| {
                let counter = CounterVec::new(Opts::new(name, help), labels)
                    .expect("Failed to create counter");
                self.registry.register(Box::new(counter.clone())).ok(); // Ignore if already registered
                counter
            })
            .clone()
    }

    /// Gets or creates a histogram metric
    fn get_or_create_histogram(&self, name: &str, help: &str, labels: &[&str]) -> HistogramVec {
        let mut histograms = self.histograms.write();
        histograms
            .entry(name.to_string())
            .or_insert_with(|| {
                let histogram = HistogramVec::new(HistogramOpts::new(name, help), labels)
                    .expect("Failed to create histogram");
                self.registry.register(Box::new(histogram.clone())).ok(); // Ignore if already registered
                histogram
            })
            .clone()
    }

    /// Gets or creates a gauge metric
    fn get_or_create_gauge(&self, name: &str, help: &str, labels: &[&str]) -> GaugeVec {
        let mut gauges = self.gauges.write();
        gauges
            .entry(name.to_string())
            .or_insert_with(|| {
                let gauge =
                    GaugeVec::new(Opts::new(name, help), labels).expect("Failed to create gauge");
                self.registry.register(Box::new(gauge.clone())).ok(); // Ignore if already registered
                gauge
            })
            .clone()
    }

    /// Returns the Prometheus registry for HTTP endpoint exposure
    pub fn registry(&self) -> Arc<Registry> {
        self.registry.clone()
    }
}

impl Default for PrometheusMetrics {
    fn default() -> Self {
        Self::new().expect("Failed to create default PrometheusMetrics")
    }
}

impl MetricsCollector for PrometheusMetrics {
    fn record_counter(&self, name: &str, value: u64, tags: &[(&str, &str)]) {
        let label_names: Vec<&str> = tags.iter().map(|(k, _)| *k).collect();
        let label_values: Vec<&str> = tags.iter().map(|(_, v)| *v).collect();

        let counter = self.get_or_create_counter(name, &format!("{} counter", name), &label_names);

        counter
            .with_label_values(&label_values)
            .inc_by(value as f64);
    }

    fn record_histogram(&self, name: &str, value: f64, tags: &[(&str, &str)]) {
        let label_names: Vec<&str> = tags.iter().map(|(k, _)| *k).collect();
        let label_values: Vec<&str> = tags.iter().map(|(_, v)| *v).collect();

        let histogram =
            self.get_or_create_histogram(name, &format!("{} histogram", name), &label_names);

        histogram.with_label_values(&label_values).observe(value);
    }

    fn record_gauge(&self, name: &str, value: f64, tags: &[(&str, &str)]) {
        let label_names: Vec<&str> = tags.iter().map(|(k, _)| *k).collect();
        let label_values: Vec<&str> = tags.iter().map(|(_, v)| *v).collect();

        let gauge = self.get_or_create_gauge(name, &format!("{} gauge", name), &label_names);

        gauge.with_label_values(&label_values).set(value);
    }
}

impl BusinessMetrics for PrometheusMetrics {
    fn record_extraction(&self, duration: Duration, success: bool) {
        let success_str = if success { "true" } else { "false" };
        self.extraction_duration
            .with_label_values(&[success_str])
            .observe(duration.as_secs_f64());
        self.extraction_total
            .with_label_values(&[success_str])
            .inc();
    }

    fn record_cache_hit(&self, key_type: &str) {
        self.cache_hits.with_label_values(&[key_type]).inc();
    }

    fn record_cache_miss(&self, key_type: &str) {
        self.cache_misses.with_label_values(&[key_type]).inc();
    }

    fn record_event_published(&self, event_type: &str) {
        self.events_published.with_label_values(&[event_type]).inc();
    }

    fn record_http_request(&self, method: &str, status: u16, duration: Duration) {
        let status_str = status.to_string();
        self.http_request_duration
            .with_label_values(&[method, &status_str])
            .observe(duration.as_secs_f64());
    }

    fn record_pipeline_stage(&self, stage: &str, duration: Duration, success: bool) {
        let success_str = if success { "true" } else { "false" };
        self.pipeline_stage_duration
            .with_label_values(&[stage, success_str])
            .observe(duration.as_secs_f64());
    }

    fn record_error(&self, error_type: &str, context: &str) {
        self.errors_total
            .with_label_values(&[error_type, context])
            .inc();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prometheus_creation() {
        let metrics = PrometheusMetrics::new();
        assert!(metrics.is_ok());
    }

    #[test]
    fn test_record_counter() {
        let metrics = PrometheusMetrics::new().unwrap();
        metrics.record_counter("test_counter", 5, &[("label", "value")]);

        // Verify the metric was recorded (would need prometheus text export to verify value)
        assert!(!metrics.registry.gather().is_empty());
    }

    #[test]
    fn test_record_histogram() {
        let metrics = PrometheusMetrics::new().unwrap();
        metrics.record_histogram("test_histogram", 0.5, &[("label", "value")]);

        assert!(!metrics.registry.gather().is_empty());
    }

    #[test]
    fn test_record_gauge() {
        let metrics = PrometheusMetrics::new().unwrap();
        metrics.record_gauge("test_gauge", 42.0, &[("label", "value")]);

        assert!(!metrics.registry.gather().is_empty());
    }

    #[test]
    fn test_business_metrics() {
        let metrics = PrometheusMetrics::new().unwrap();

        metrics.record_extraction(Duration::from_millis(100), true);
        metrics.record_cache_hit("page");
        metrics.record_cache_miss("extraction");
        metrics.record_event_published("test_event");
        metrics.record_http_request("GET", 200, Duration::from_millis(50));
        metrics.record_pipeline_stage("parse", Duration::from_millis(20), true);
        metrics.record_error("validation", "test");

        let gathered = metrics.registry.gather();
        assert!(gathered.len() >= 7); // At least 7 business metrics
    }

    #[test]
    fn test_multiple_recordings() {
        let metrics = PrometheusMetrics::new().unwrap();

        for i in 0..10 {
            metrics.record_counter("test_multi", i, &[("iteration", &i.to_string())]);
        }

        assert!(!metrics.registry.gather().is_empty());
    }
}
