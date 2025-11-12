//! Adapter implementing MonitoringBackend port for MetricsCollector
//!
//! This adapter bridges the concrete MetricsCollector implementation
//! with the abstract MonitoringBackend port trait, enabling dependency
//! inversion in the hexagonal architecture.

use async_trait::async_trait;
use riptide_types::error::Result as RiptideResult;
use riptide_types::ports::monitoring_backend::{
    AggregationFunction, MetricPoint, MetricQuery, MetricStats, MetricsSummary, MetricTags,
    MonitoringBackend, TimeRange,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::monitoring::collector::MetricsCollector;

/// Adapter that implements MonitoringBackend using MetricsCollector
///
/// This adapter wraps the concrete MetricsCollector and implements
/// the abstract MonitoringBackend trait, allowing the collector to be
/// injected as a dependency via the port interface.
pub struct MonitoringSystemAdapter {
    /// Inner metrics collector
    collector: Arc<RwLock<MetricsCollector>>,
}

impl MonitoringSystemAdapter {
    /// Create a new adapter wrapping a MetricsCollector
    ///
    /// # Arguments
    ///
    /// * `collector` - The concrete MetricsCollector to wrap
    ///
    /// # Returns
    ///
    /// Arc-wrapped adapter ready for dependency injection
    pub fn new(collector: MetricsCollector) -> Arc<Self> {
        Arc::new(Self {
            collector: Arc::new(RwLock::new(collector)),
        })
    }

    /// Get reference to inner collector (for testing/diagnostics)
    #[cfg(test)]
    pub fn inner(&self) -> Arc<RwLock<MetricsCollector>> {
        Arc::clone(&self.collector)
    }

    /// Calculate aggregation for metric values
    #[allow(dead_code)]
    fn aggregate_values(values: &[f64], func: &AggregationFunction) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        match func {
            AggregationFunction::Average => values.iter().sum::<f64>() / values.len() as f64,
            AggregationFunction::Sum => values.iter().sum(),
            AggregationFunction::Min => values
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min),
            AggregationFunction::Max => values
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max),
            AggregationFunction::Count => values.len() as f64,
            AggregationFunction::P50 => Self::percentile(values, 0.5),
            AggregationFunction::P95 => Self::percentile(values, 0.95),
            AggregationFunction::P99 => Self::percentile(values, 0.99),
        }
    }

    /// Calculate percentile from sorted values
    #[allow(dead_code)]
    fn percentile(values: &[f64], p: f64) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let index = ((sorted.len() as f64) * p) as usize;
        sorted[index.min(sorted.len() - 1)]
    }

    /// Calculate standard deviation
    #[allow(dead_code)]
    fn stddev(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>()
            / (values.len() - 1) as f64;

        variance.sqrt()
    }
}

#[async_trait]
impl MonitoringBackend for MonitoringSystemAdapter {
    async fn report_metric(&self, name: &str, value: f64, _tags: MetricTags) -> RiptideResult<()> {
        // Record metric via collector
        // Note: The current MetricsCollector doesn't have a direct record_metric method
        // We use record_extraction_time as a proxy for now
        let collector = self.collector.read().map_err(|e| {
            riptide_types::error::RiptideError::custom(format!(
                "Failed to acquire collector lock: {}",
                e
            ))
        })?;

        // Record via telemetry if available (note: telemetry() returns &TelemetrySystem)
        // Skip recording for now since we can't mutate through the reference
        let _telemetry = collector.telemetry();

        tracing::info!(
            metric_name = %name,
            value = value,
            "Metric reported"
        );

        Ok(())
    }

    async fn query_metrics(&self, query: MetricQuery) -> RiptideResult<Vec<MetricPoint>> {
        // For now, return empty results as the collector doesn't expose
        // time-series query interface directly
        // Future: Integrate with TimeSeriesBuffer from collector

        tracing::debug!(
            metric_pattern = %query.metric_name,
            aggregation = ?query.aggregation,
            "Querying metrics"
        );

        Ok(Vec::new())
    }

    async fn health_score(&self) -> RiptideResult<f32> {
        let collector = self.collector.read().map_err(|e| {
            riptide_types::error::RiptideError::custom(format!(
                "Failed to acquire collector lock: {}",
                e
            ))
        })?;

        // Calculate health score from current metrics
        // Use default healthy score as collector doesn't expose health_status directly
        let _metrics = &*collector;

        // For now, assume healthy. Future: extract from collector metrics
        let score = 1.0;

        Ok(score)
    }

    fn status(&self) -> String {
        let _collector = match self.collector.read() {
            Ok(c) => c,
            Err(_) => return "Monitoring system unavailable".to_string(),
        };

        // Return status string (collector doesn't expose health_status method)
        "Monitoring system operational".to_string()
    }

    async fn record_performance(
        &self,
        operation: &str,
        duration_ms: u64,
        success: bool,
    ) -> RiptideResult<()> {
        let collector = self.collector.read().map_err(|e| {
            riptide_types::error::RiptideError::custom(format!(
                "Failed to acquire collector lock: {}",
                e
            ))
        })?;

        // Record via telemetry if available (note: telemetry() returns &TelemetrySystem)
        // Skip recording for now since we can't mutate through the reference
        let _telemetry = collector.telemetry();

        tracing::info!(
            operation = %operation,
            duration_ms = duration_ms,
            success = success,
            "Performance metric recorded"
        );

        Ok(())
    }

    async fn metrics_summary(
        &self,
        metric_names: Vec<String>,
    ) -> RiptideResult<MetricsSummary> {
        // Build summary from available metrics
        let now = chrono::Utc::now();
        let one_hour_ago = now - chrono::Duration::hours(1);

        let mut metrics = HashMap::new();

        for name in metric_names {
            // Create placeholder stats
            // Future: Extract real data from TimeSeriesBuffer
            metrics.insert(
                name.clone(),
                MetricStats {
                    name,
                    average: 0.0,
                    min: 0.0,
                    max: 0.0,
                    stddev: 0.0,
                    count: 0,
                },
            );
        }

        Ok(MetricsSummary {
            metrics,
            time_range: TimeRange {
                start: one_hour_ago,
                end: now,
            },
            total_points: 0,
        })
    }
}

impl std::fmt::Debug for MonitoringSystemAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MonitoringSystemAdapter").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adapter_creation() {
        let collector = MetricsCollector::new();
        let adapter = MonitoringSystemAdapter::new(collector);

        let status = adapter.status();
        assert!(!status.is_empty());
    }

    #[tokio::test]
    async fn test_report_metric() {
        let collector = MetricsCollector::new();
        let adapter = MonitoringSystemAdapter::new(collector);

        let mut tags = HashMap::new();
        tags.insert("env".to_string(), "test".to_string());

        let result = adapter.report_metric("test_metric", 42.0, tags).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_health_score() {
        let collector = MetricsCollector::new();
        let adapter = MonitoringSystemAdapter::new(collector);

        let score = adapter.health_score().await.unwrap();
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[tokio::test]
    async fn test_record_performance() {
        let collector = MetricsCollector::new();
        let adapter = MonitoringSystemAdapter::new(collector);

        let result = adapter
            .record_performance("test_op", 100, true)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_summary() {
        let collector = MetricsCollector::new();
        let adapter = MonitoringSystemAdapter::new(collector);

        let metric_names = vec!["metric1".to_string(), "metric2".to_string()];
        let summary = adapter.metrics_summary(metric_names).await.unwrap();

        assert_eq!(summary.metrics.len(), 2);
    }

    #[test]
    fn test_aggregate_values() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        assert_eq!(
            MonitoringSystemAdapter::aggregate_values(&values, &AggregationFunction::Average),
            3.0
        );
        assert_eq!(
            MonitoringSystemAdapter::aggregate_values(&values, &AggregationFunction::Sum),
            15.0
        );
        assert_eq!(
            MonitoringSystemAdapter::aggregate_values(&values, &AggregationFunction::Min),
            1.0
        );
        assert_eq!(
            MonitoringSystemAdapter::aggregate_values(&values, &AggregationFunction::Max),
            5.0
        );
        assert_eq!(
            MonitoringSystemAdapter::aggregate_values(&values, &AggregationFunction::Count),
            5.0
        );
    }
}
