//! Telemetry backend port for hexagonal architecture
//!
//! This module provides backend-agnostic telemetry interfaces that enable:
//! - Distributed tracing abstraction
//! - Metrics recording abstraction
//! - Dependency inversion for telemetry systems
//! - Testing with mock implementations
//!
//! # Design Goals
//!
//! - **Backend Independence**: Business logic doesn't depend on specific telemetry providers
//! - **Testability**: Easy mocking and testing without real telemetry
//! - **Flexibility**: Support multiple telemetry backends (OpenTelemetry, custom, etc.)
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────┐
//! │ Domain Layer (Ports)                     │
//! │  └─ TelemetryBackend                     │
//! └──────────────────────────────────────────┘
//!              ↑ implements          ↑ uses
//!              │                     │
//! ┌────────────┴──────────┐   ┌────┴──────────────┐
//! │ Infrastructure         │   │ Application       │
//! │ - TelemetrySystem      │   │ - Business logic  │
//! │ - OpenTelemetry        │   │ - Handlers        │
//! └────────────────────────┘   └───────────────────┘
//! ```

use crate::error::Result as RiptideResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Backend abstraction for telemetry and observability
///
/// This trait enables backend-agnostic telemetry recording for spans,
/// metrics, and events. Implementations handle the backend-specific
/// formatting and delivery to telemetry systems.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` for use in async contexts.
#[async_trait]
pub trait TelemetryBackend: Send + Sync {
    /// Record a distributed tracing span
    ///
    /// # Arguments
    ///
    /// * `span` - Span data to record
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Span recorded successfully
    /// * `Err(_)` - Backend error
    async fn record_span(&self, span: Span) -> RiptideResult<()>;

    /// Record a metric value
    ///
    /// # Arguments
    ///
    /// * `metric` - Metric data to record
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Metric recorded successfully
    /// * `Err(_)` - Backend error
    async fn record_metric(&self, metric: Metric) -> RiptideResult<()>;

    /// Flush buffered telemetry data
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Flush completed successfully
    /// * `Err(_)` - Backend error during flush
    async fn flush(&self) -> RiptideResult<()>;

    /// Get current telemetry status
    ///
    /// # Returns
    ///
    /// Status information about the telemetry backend
    fn status(&self) -> TelemetryStatus;

    /// Sanitize sensitive data from telemetry
    ///
    /// # Arguments
    ///
    /// * `data` - Raw data that may contain sensitive information
    ///
    /// # Returns
    ///
    /// Sanitized data safe for telemetry
    fn sanitize_data(&self, data: &str) -> String;
}

/// Distributed tracing span data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    /// Unique span identifier
    pub span_id: String,

    /// Parent span identifier (if any)
    pub parent_span_id: Option<String>,

    /// Trace identifier
    pub trace_id: String,

    /// Span name/operation
    pub name: String,

    /// Span start time
    pub start_time: chrono::DateTime<chrono::Utc>,

    /// Span duration
    pub duration: Duration,

    /// Span attributes/tags
    pub attributes: HashMap<String, String>,

    /// Span status
    pub status: SpanStatus,
}

/// Span status indicator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SpanStatus {
    /// Span completed successfully
    Ok,

    /// Span completed with error
    Error {
        /// Error message
        message: String,
    },

    /// Span status unknown
    Unknown,
}

/// Metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name
    pub name: String,

    /// Metric value
    pub value: MetricValue,

    /// Metric type
    pub metric_type: MetricType,

    /// Metric labels/tags
    pub labels: HashMap<String, String>,

    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Metric value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MetricValue {
    /// Integer counter value
    Counter(u64),

    /// Floating point gauge value
    Gauge(f64),

    /// Histogram bucket values
    Histogram {
        /// Bucket boundaries
        buckets: Vec<f64>,
        /// Bucket counts
        counts: Vec<u64>,
    },
}

/// Metric type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetricType {
    /// Monotonically increasing counter
    Counter,

    /// Point-in-time value
    Gauge,

    /// Distribution of values
    Histogram,
}

/// Telemetry backend status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryStatus {
    /// Whether telemetry is healthy
    pub healthy: bool,

    /// Status message
    pub message: String,

    /// Number of spans recorded
    pub spans_recorded: u64,

    /// Number of metrics recorded
    pub metrics_recorded: u64,

    /// Last flush time
    pub last_flush: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for TelemetryStatus {
    fn default() -> Self {
        Self {
            healthy: true,
            message: "Telemetry system operational".to_string(),
            spans_recorded: 0,
            metrics_recorded: 0,
            last_flush: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_status_serialization() {
        let ok_status = SpanStatus::Ok;
        let json = serde_json::to_string(&ok_status).unwrap();
        assert!(json.contains("Ok"));

        let error_status = SpanStatus::Error {
            message: "test error".to_string(),
        };
        let json = serde_json::to_string(&error_status).unwrap();
        assert!(json.contains("test error"));
    }

    #[test]
    fn test_metric_types() {
        let counter = MetricValue::Counter(42);
        assert!(matches!(counter, MetricValue::Counter(42)));

        let gauge = MetricValue::Gauge(3.14);
        assert!(matches!(gauge, MetricValue::Gauge(_)));

        let histogram = MetricValue::Histogram {
            buckets: vec![0.0, 1.0, 5.0],
            counts: vec![10, 20, 5],
        };
        assert!(matches!(histogram, MetricValue::Histogram { .. }));
    }

    #[test]
    fn test_telemetry_status_default() {
        let status = TelemetryStatus::default();
        assert!(status.healthy);
        assert_eq!(status.spans_recorded, 0);
        assert_eq!(status.metrics_recorded, 0);
    }
}
