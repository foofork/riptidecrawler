//! Monitoring backend port for hexagonal architecture
//!
//! This module provides backend-agnostic monitoring interfaces that enable:
//! - Metrics collection abstraction
//! - Health monitoring abstraction
//! - Dependency inversion for monitoring systems
//! - Testing with mock implementations
//!
//! # Design Goals
//!
//! - **Backend Independence**: Business logic doesn't depend on specific monitoring providers
//! - **Testability**: Easy mocking and testing without real monitoring
//! - **Flexibility**: Support multiple monitoring backends (Prometheus, custom, etc.)
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────┐
//! │ Domain Layer (Ports)                     │
//! │  └─ MonitoringBackend                    │
//! └──────────────────────────────────────────┘
//!              ↑ implements          ↑ uses
//!              │                     │
//! ┌────────────┴──────────┐   ┌────┴──────────────┐
//! │ Infrastructure         │   │ Application       │
//! │ - MonitoringSystem     │   │ - Business logic  │
//! │ - MetricsCollector     │   │ - Handlers        │
//! └────────────────────────┘   └───────────────────┘
//! ```

use crate::error::Result as RiptideResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Backend abstraction for system monitoring and metrics
///
/// This trait enables backend-agnostic monitoring by abstracting over
/// different monitoring systems. Implementations handle the backend-specific
/// metric storage and query interfaces.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` for use in async contexts.
#[async_trait]
pub trait MonitoringBackend: Send + Sync {
    /// Report a metric value
    ///
    /// # Arguments
    ///
    /// * `name` - Metric name
    /// * `value` - Metric value
    /// * `tags` - Metric tags/labels
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Metric reported successfully
    /// * `Err(_)` - Backend error
    async fn report_metric(&self, name: &str, value: f64, tags: MetricTags) -> RiptideResult<()>;

    /// Query metrics from the monitoring backend
    ///
    /// # Arguments
    ///
    /// * `query` - Query specification
    ///
    /// # Returns
    ///
    /// * `Ok(metrics)` - Query results
    /// * `Err(_)` - Backend error
    async fn query_metrics(&self, query: MetricQuery) -> RiptideResult<Vec<MetricPoint>>;

    /// Get overall system health score
    ///
    /// # Returns
    ///
    /// * `Ok(score)` - Health score (0.0 = unhealthy, 1.0 = perfect health)
    /// * `Err(_)` - Backend error
    async fn health_score(&self) -> RiptideResult<f32>;

    /// Get system status message
    ///
    /// # Returns
    ///
    /// Human-readable status message
    fn status(&self) -> String;

    /// Record a performance metric
    ///
    /// # Arguments
    ///
    /// * `operation` - Operation name
    /// * `duration_ms` - Operation duration in milliseconds
    /// * `success` - Whether operation succeeded
    async fn record_performance(
        &self,
        operation: &str,
        duration_ms: u64,
        success: bool,
    ) -> RiptideResult<()>;

    /// Get aggregated metrics summary
    ///
    /// # Arguments
    ///
    /// * `metric_names` - List of metric names to summarize
    ///
    /// # Returns
    ///
    /// * `Ok(summary)` - Metrics summary
    /// * `Err(_)` - Backend error
    async fn metrics_summary(&self, metric_names: Vec<String>)
        -> RiptideResult<MetricsSummary>;
}

/// Metric tags/labels
pub type MetricTags = HashMap<String, String>;

/// Metric query specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricQuery {
    /// Metric name pattern
    pub metric_name: String,

    /// Tag filters
    pub tags: MetricTags,

    /// Time range start
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,

    /// Time range end
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,

    /// Aggregation function
    pub aggregation: AggregationFunction,

    /// Maximum number of results
    pub limit: Option<usize>,
}

/// Aggregation function for metric queries
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AggregationFunction {
    /// Average value
    Average,

    /// Sum of values
    Sum,

    /// Minimum value
    Min,

    /// Maximum value
    Max,

    /// Count of data points
    Count,

    /// 50th percentile (median)
    P50,

    /// 95th percentile
    P95,

    /// 99th percentile
    P99,
}

/// Single metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    /// Metric name
    pub name: String,

    /// Metric value
    pub value: f64,

    /// Metric tags
    pub tags: MetricTags,

    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Aggregated metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    /// Metric summaries by name
    pub metrics: HashMap<String, MetricStats>,

    /// Time range covered
    pub time_range: TimeRange,

    /// Total data points
    pub total_points: usize,
}

/// Statistical summary of a metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricStats {
    /// Metric name
    pub name: String,

    /// Average value
    pub average: f64,

    /// Minimum value
    pub min: f64,

    /// Maximum value
    pub max: f64,

    /// Standard deviation
    pub stddev: f64,

    /// Data point count
    pub count: usize,
}

/// Time range specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// Range start
    pub start: chrono::DateTime<chrono::Utc>,

    /// Range end
    pub end: chrono::DateTime<chrono::Utc>,
}

impl Default for MetricQuery {
    fn default() -> Self {
        Self {
            metric_name: "*".to_string(),
            tags: HashMap::new(),
            start_time: None,
            end_time: None,
            aggregation: AggregationFunction::Average,
            limit: Some(1000),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregation_functions() {
        assert_eq!(
            AggregationFunction::Average,
            AggregationFunction::Average
        );
        assert_ne!(AggregationFunction::Sum, AggregationFunction::Average);
    }

    #[test]
    fn test_metric_query_default() {
        let query = MetricQuery::default();
        assert_eq!(query.metric_name, "*");
        assert_eq!(query.aggregation, AggregationFunction::Average);
        assert_eq!(query.limit, Some(1000));
    }

    #[test]
    fn test_metric_point_serialization() {
        let mut tags = HashMap::new();
        tags.insert("env".to_string(), "test".to_string());

        let point = MetricPoint {
            name: "test_metric".to_string(),
            value: 42.0,
            tags,
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&point).unwrap();
        assert!(json.contains("test_metric"));
        assert!(json.contains("42"));
    }
}
