//! Metrics collection port definition
//!
//! This module defines the abstract metrics collection interfaces.
//! It supports both low-level metrics (counters, histograms, gauges) and
//! high-level business metrics specific to Riptide operations.

use std::time::Duration;

/// Low-level metrics collector port
///
/// This trait defines the contract for metrics collection adapters.
/// Implementations should handle metric registration, aggregation, and export.
pub trait MetricsCollector: Send + Sync {
    /// Records a counter metric (monotonically increasing value)
    ///
    /// # Arguments
    /// * `name` - Metric name
    /// * `value` - Value to add to the counter
    /// * `tags` - Key-value pairs for metric dimensions
    fn record_counter(&self, name: &str, value: u64, tags: &[(&str, &str)]);

    /// Records a histogram metric (distribution of values)
    ///
    /// # Arguments
    /// * `name` - Metric name
    /// * `value` - Observed value
    /// * `tags` - Key-value pairs for metric dimensions
    fn record_histogram(&self, name: &str, value: f64, tags: &[(&str, &str)]);

    /// Records a gauge metric (point-in-time value)
    ///
    /// # Arguments
    /// * `name` - Metric name
    /// * `value` - Current value
    /// * `tags` - Key-value pairs for metric dimensions
    fn record_gauge(&self, name: &str, value: f64, tags: &[(&str, &str)]);
}

/// Business-level metrics port
///
/// This trait defines high-level metrics specific to Riptide's business operations.
/// It provides a semantic layer on top of the low-level metrics.
pub trait BusinessMetrics: Send + Sync {
    /// Records a data extraction operation
    ///
    /// # Arguments
    /// * `duration` - Time taken for extraction
    /// * `success` - Whether the extraction succeeded
    fn record_extraction(&self, duration: Duration, success: bool);

    /// Records a cache hit or miss
    ///
    /// # Arguments
    /// * `key_type` - Type of cache key (e.g., "page", "extraction")
    fn record_cache_hit(&self, key_type: &str);

    /// Records a cache miss
    ///
    /// # Arguments
    /// * `key_type` - Type of cache key
    fn record_cache_miss(&self, key_type: &str);

    /// Records an event publication
    ///
    /// # Arguments
    /// * `event_type` - Type of event published
    fn record_event_published(&self, event_type: &str);

    /// Records HTTP request metrics
    ///
    /// # Arguments
    /// * `method` - HTTP method
    /// * `status` - HTTP status code
    /// * `duration` - Request duration
    fn record_http_request(&self, method: &str, status: u16, duration: Duration);

    /// Records pipeline processing metrics
    ///
    /// # Arguments
    /// * `stage` - Pipeline stage name
    /// * `duration` - Stage duration
    /// * `success` - Whether the stage succeeded
    fn record_pipeline_stage(&self, stage: &str, duration: Duration, success: bool);

    /// Records error occurrences
    ///
    /// # Arguments
    /// * `error_type` - Type of error
    /// * `context` - Additional context
    fn record_error(&self, error_type: &str, context: &str);
}

/// Metrics registry for managing multiple collectors
pub trait MetricsRegistry: Send + Sync {
    /// Registers a metrics collector
    fn register_collector(&mut self, name: String, collector: Box<dyn MetricsCollector>);

    /// Gets a collector by name
    fn get_collector(&self, name: &str) -> Option<&dyn MetricsCollector>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct MockMetricsCollector {
        counters: Arc<Mutex<Vec<(String, u64)>>>,
        histograms: Arc<Mutex<Vec<(String, f64)>>>,
        gauges: Arc<Mutex<Vec<(String, f64)>>>,
    }

    impl MetricsCollector for MockMetricsCollector {
        fn record_counter(&self, name: &str, value: u64, _tags: &[(&str, &str)]) {
            self.counters
                .lock()
                .unwrap()
                .push((name.to_string(), value));
        }

        fn record_histogram(&self, name: &str, value: f64, _tags: &[(&str, &str)]) {
            self.histograms
                .lock()
                .unwrap()
                .push((name.to_string(), value));
        }

        fn record_gauge(&self, name: &str, value: f64, _tags: &[(&str, &str)]) {
            self.gauges.lock().unwrap().push((name.to_string(), value));
        }
    }

    impl BusinessMetrics for MockMetricsCollector {
        fn record_extraction(&self, duration: Duration, success: bool) {
            self.record_histogram("extraction_duration", duration.as_secs_f64(), &[]);
            self.record_counter(
                "extractions_total",
                1,
                &[("success", if success { "true" } else { "false" })],
            );
        }

        fn record_cache_hit(&self, key_type: &str) {
            self.record_counter("cache_hits_total", 1, &[("type", key_type)]);
        }

        fn record_cache_miss(&self, key_type: &str) {
            self.record_counter("cache_misses_total", 1, &[("type", key_type)]);
        }

        fn record_event_published(&self, event_type: &str) {
            self.record_counter("events_published_total", 1, &[("type", event_type)]);
        }

        fn record_http_request(&self, _method: &str, _status: u16, duration: Duration) {
            self.record_histogram("http_request_duration", duration.as_secs_f64(), &[]);
        }

        fn record_pipeline_stage(&self, _stage: &str, duration: Duration, _success: bool) {
            self.record_histogram("pipeline_stage_duration", duration.as_secs_f64(), &[]);
        }

        fn record_error(&self, error_type: &str, _context: &str) {
            self.record_counter("errors_total", 1, &[("type", error_type)]);
        }
    }

    #[test]
    fn test_metrics_collector() {
        let collector = MockMetricsCollector::default();

        collector.record_counter("test_counter", 42, &[]);
        collector.record_histogram("test_histogram", 3.14, &[]);
        collector.record_gauge("test_gauge", 99.9, &[]);

        assert_eq!(collector.counters.lock().unwrap().len(), 1);
        assert_eq!(collector.histograms.lock().unwrap().len(), 1);
        assert_eq!(collector.gauges.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_business_metrics() {
        let collector = MockMetricsCollector::default();

        collector.record_extraction(Duration::from_millis(100), true);
        collector.record_cache_hit("page");
        collector.record_event_published("extraction_complete");

        assert!(collector.counters.lock().unwrap().len() >= 2);
        assert_eq!(collector.histograms.lock().unwrap().len(), 1);
    }
}
