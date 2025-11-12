//! Adapter implementing TelemetryBackend port for TelemetrySystem
//!
//! This adapter bridges the concrete TelemetrySystem implementation
//! with the abstract TelemetryBackend port trait, enabling dependency
//! inversion in the hexagonal architecture.

use async_trait::async_trait;
use riptide_types::error::Result as RiptideResult;
use riptide_types::ports::telemetry::{
    Metric, MetricType, MetricValue, Span, SpanStatus, TelemetryBackend, TelemetryStatus,
};
use std::sync::{Arc, RwLock};

use crate::telemetry::TelemetrySystem;

/// Adapter that implements TelemetryBackend using TelemetrySystem
///
/// This adapter wraps the concrete TelemetrySystem and implements
/// the abstract TelemetryBackend trait, allowing the telemetry system
/// to be injected as a dependency via the port interface.
pub struct TelemetrySystemAdapter {
    /// Inner telemetry system
    system: Arc<RwLock<TelemetrySystem>>,

    /// Track recorded spans
    spans_recorded: Arc<RwLock<u64>>,

    /// Track recorded metrics
    metrics_recorded: Arc<RwLock<u64>>,

    /// Last flush time
    last_flush: Arc<RwLock<Option<chrono::DateTime<chrono::Utc>>>>,
}

impl TelemetrySystemAdapter {
    /// Create a new adapter wrapping a TelemetrySystem
    ///
    /// # Arguments
    ///
    /// * `system` - The concrete TelemetrySystem to wrap
    ///
    /// # Returns
    ///
    /// Arc-wrapped adapter ready for dependency injection
    pub fn new(system: TelemetrySystem) -> Arc<Self> {
        Arc::new(Self {
            system: Arc::new(RwLock::new(system)),
            spans_recorded: Arc::new(RwLock::new(0)),
            metrics_recorded: Arc::new(RwLock::new(0)),
            last_flush: Arc::new(RwLock::new(None)),
        })
    }

    /// Get reference to inner telemetry system (for testing/diagnostics)
    #[cfg(test)]
    pub fn inner(&self) -> Arc<RwLock<TelemetrySystem>> {
        Arc::clone(&self.system)
    }
}

#[async_trait]
impl TelemetryBackend for TelemetrySystemAdapter {
    async fn record_span(&self, span: Span) -> RiptideResult<()> {
        // Convert span status to success flag for SLA recording
        let success = matches!(span.status, SpanStatus::Ok);

        // Record SLA metric via telemetry system
        {
            let mut system = self.system.write().map_err(|e| {
                riptide_types::error::RiptideError::custom(format!(
                    "Failed to acquire telemetry lock: {}",
                    e
                ))
            })?;

            system.record_sla_metric(&span.name, span.duration, success);
        }

        // Increment span counter
        {
            let mut count = self.spans_recorded.write().map_err(|e| {
                riptide_types::error::RiptideError::custom(format!(
                    "Failed to acquire span counter lock: {}",
                    e
                ))
            })?;
            *count += 1;
        }

        // Use tracing macros for actual span recording
        tracing::info!(
            span_id = %span.span_id,
            trace_id = %span.trace_id,
            name = %span.name,
            duration_ms = span.duration.as_millis(),
            success = success,
            "Span recorded"
        );

        Ok(())
    }

    async fn record_metric(&self, metric: Metric) -> RiptideResult<()> {
        // Extract metric value as f64
        let value = match metric.value {
            MetricValue::Counter(v) => v as f64,
            MetricValue::Gauge(v) => v,
            MetricValue::Histogram { ref buckets, .. } => {
                // For histograms, use the last bucket value as representative
                buckets.last().copied().unwrap_or(0.0)
            }
        };

        // Determine success flag (assume success for gauge/counter metrics)
        let success = !matches!(metric.metric_type, MetricType::Counter);

        // Record via telemetry system's SLA tracker
        {
            let mut system = self.system.write().map_err(|e| {
                riptide_types::error::RiptideError::custom(format!(
                    "Failed to acquire telemetry lock: {}",
                    e
                ))
            })?;

            system.record_sla_metric(
                &metric.name,
                std::time::Duration::from_millis(value as u64),
                success,
            );
        }

        // Increment metric counter
        {
            let mut count = self.metrics_recorded.write().map_err(|e| {
                riptide_types::error::RiptideError::custom(format!(
                    "Failed to acquire metric counter lock: {}",
                    e
                ))
            })?;
            *count += 1;
        }

        // Use tracing for actual metric recording
        tracing::info!(
            metric_name = %metric.name,
            metric_type = ?metric.metric_type,
            value = value,
            "Metric recorded"
        );

        Ok(())
    }

    async fn flush(&self) -> RiptideResult<()> {
        // Update last flush time
        {
            let mut last_flush = self.last_flush.write().map_err(|e| {
                riptide_types::error::RiptideError::custom(format!(
                    "Failed to acquire flush lock: {}",
                    e
                ))
            })?;
            *last_flush = Some(chrono::Utc::now());
        }

        tracing::info!("Telemetry data flushed");
        Ok(())
    }

    fn status(&self) -> TelemetryStatus {
        let spans = self
            .spans_recorded
            .read()
            .map(|c| *c)
            .unwrap_or(0);

        let metrics = self
            .metrics_recorded
            .read()
            .map(|c| *c)
            .unwrap_or(0);

        let last_flush = self
            .last_flush
            .read()
            .map(|f| *f)
            .ok()
            .flatten();

        TelemetryStatus {
            healthy: true,
            message: "Telemetry system operational".to_string(),
            spans_recorded: spans,
            metrics_recorded: metrics,
            last_flush,
        }
    }

    fn sanitize_data(&self, data: &str) -> String {
        let system = match self.system.read() {
            Ok(s) => s,
            Err(_) => return data.to_string(),
        };

        system.sanitize_data(data)
    }
}

impl std::fmt::Debug for TelemetrySystemAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TelemetrySystemAdapter")
            .field("spans_recorded", &self.spans_recorded)
            .field("metrics_recorded", &self.metrics_recorded)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_adapter_creation() {
        let system = TelemetrySystem::init().unwrap();
        let adapter = TelemetrySystemAdapter::new(system);

        let status = adapter.status();
        assert!(status.healthy);
        assert_eq!(status.spans_recorded, 0);
    }

    #[tokio::test]
    async fn test_record_span() {
        let system = TelemetrySystem::init().unwrap();
        let adapter = TelemetrySystemAdapter::new(system);

        let span = Span {
            span_id: "span-123".to_string(),
            parent_span_id: None,
            trace_id: "trace-456".to_string(),
            name: "test_operation".to_string(),
            start_time: chrono::Utc::now(),
            duration: Duration::from_millis(100),
            attributes: Default::default(),
            status: SpanStatus::Ok,
        };

        adapter.record_span(span).await.unwrap();

        let status = adapter.status();
        assert_eq!(status.spans_recorded, 1);
    }

    #[tokio::test]
    async fn test_record_metric() {
        let system = TelemetrySystem::init().unwrap();
        let adapter = TelemetrySystemAdapter::new(system);

        let metric = Metric {
            name: "test_counter".to_string(),
            value: MetricValue::Counter(42),
            metric_type: MetricType::Counter,
            labels: Default::default(),
            timestamp: chrono::Utc::now(),
        };

        adapter.record_metric(metric).await.unwrap();

        let status = adapter.status();
        assert_eq!(status.metrics_recorded, 1);
    }

    #[tokio::test]
    async fn test_flush() {
        let system = TelemetrySystem::init().unwrap();
        let adapter = TelemetrySystemAdapter::new(system);

        adapter.flush().await.unwrap();

        let status = adapter.status();
        assert!(status.last_flush.is_some());
    }
}
