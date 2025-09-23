//! Modular performance monitoring and metrics collection system
//!
//! This module provides real-time monitoring of extractor performance,
//! resource usage, and system health with alerting capabilities.

pub mod alerts;
pub mod collector;
pub mod error;
pub mod health;
pub mod metrics;
pub mod reports;
pub mod time_series;

// Re-export main types for backward compatibility
pub use alerts::{Alert, AlertCondition, AlertManager, AlertRule, AlertSeverity};
pub use collector::MetricsCollector;
pub use error::{LockManager, MonitoringError, Result};
pub use health::HealthCalculator;
pub use metrics::{MetricDataPoint, MonitoringConfig, PerformanceMetrics};
pub use reports::{
    MetricSummary, PerformanceReport, ReportGenerator, TrendAnalysis, TrendDirection,
};
pub use time_series::TimeSeriesBuffer;
