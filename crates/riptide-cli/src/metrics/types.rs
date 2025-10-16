//! Core metric type definitions for CLI operations
//!
//! This module defines the data structures for tracking CLI command performance,
//! execution statistics, and resource usage with minimal overhead.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// CLI command execution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMetrics {
    /// Command name (e.g., "extract", "crawl", "search")
    pub command_name: String,

    /// Execution start time
    #[serde(with = "chrono::serde::ts_seconds")]
    pub started_at: DateTime<Utc>,

    /// Execution duration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,

    /// Success/failure status
    pub success: bool,

    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Number of items processed
    pub items_processed: u64,

    /// Bytes transferred (download + upload)
    pub bytes_transferred: u64,

    /// Cache hits during execution
    pub cache_hits: u64,

    /// API calls made
    pub api_calls: u64,

    /// Peak memory usage in bytes
    pub peak_memory_bytes: u64,

    /// Additional command-specific metadata
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

impl CommandMetrics {
    /// Create new command metrics instance
    pub fn new(command_name: impl Into<String>) -> Self {
        Self {
            command_name: command_name.into(),
            started_at: Utc::now(),
            duration_ms: None,
            success: false,
            error: None,
            items_processed: 0,
            bytes_transferred: 0,
            cache_hits: 0,
            api_calls: 0,
            peak_memory_bytes: 0,
            metadata: HashMap::new(),
        }
    }

    /// Mark as completed with duration
    pub fn complete(&mut self, duration: Duration) {
        self.duration_ms = Some(duration.as_millis() as u64);
        self.success = true;
    }

    /// Mark as failed with error message
    pub fn fail(&mut self, duration: Duration, error: impl Into<String>) {
        self.duration_ms = Some(duration.as_millis() as u64);
        self.success = false;
        self.error = Some(error.into());
    }

    /// Add metadata key-value pair
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }
}

/// Aggregated metrics for a specific command over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandAggregates {
    /// Command name
    pub command: String,

    /// Total executions
    pub total_executions: u64,

    /// Successful executions
    pub successful_executions: u64,

    /// Failed executions
    pub failed_executions: u64,

    /// Average execution time (ms)
    pub avg_duration_ms: f64,

    /// P50 execution time (ms)
    pub p50_duration_ms: f64,

    /// P95 execution time (ms)
    pub p95_duration_ms: f64,

    /// P99 execution time (ms)
    pub p99_duration_ms: f64,

    /// Total items processed
    pub total_items_processed: u64,

    /// Total bytes transferred
    pub total_bytes_transferred: u64,

    /// Cache hit rate (0.0 - 1.0)
    pub cache_hit_rate: f64,

    /// Total API calls
    pub total_api_calls: u64,

    /// Average memory usage (bytes)
    pub avg_memory_bytes: f64,

    /// Last execution timestamp
    #[serde(with = "chrono::serde::ts_seconds")]
    pub last_executed: DateTime<Utc>,

    /// Error distribution (error type -> count)
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub error_distribution: HashMap<String, u64>,
}

impl CommandAggregates {
    /// Create new empty aggregates for a command
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            avg_duration_ms: 0.0,
            p50_duration_ms: 0.0,
            p95_duration_ms: 0.0,
            p99_duration_ms: 0.0,
            total_items_processed: 0,
            total_bytes_transferred: 0,
            cache_hit_rate: 0.0,
            total_api_calls: 0,
            avg_memory_bytes: 0.0,
            last_executed: Utc::now(),
            error_distribution: HashMap::new(),
        }
    }

    /// Success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_executions == 0 {
            return 0.0;
        }
        (self.successful_executions as f64 / self.total_executions as f64) * 100.0
    }

    /// Error rate as percentage
    pub fn error_rate(&self) -> f64 {
        if self.total_executions == 0 {
            return 0.0;
        }
        (self.failed_executions as f64 / self.total_executions as f64) * 100.0
    }
}

/// System-wide CLI metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliMetricsSummary {
    /// Total commands executed
    pub total_commands: u64,

    /// Commands by name with their aggregates
    pub command_stats: HashMap<String, CommandAggregates>,

    /// Overall success rate
    pub overall_success_rate: f64,

    /// Total execution time (ms)
    pub total_execution_time_ms: u64,

    /// Total bytes transferred
    pub total_bytes_transferred: u64,

    /// Total API calls
    pub total_api_calls: u64,

    /// Average command execution time (ms)
    pub avg_command_duration_ms: f64,

    /// CLI session start time
    #[serde(with = "chrono::serde::ts_seconds")]
    pub session_start: DateTime<Utc>,

    /// Last command execution time
    #[serde(with = "chrono::serde::ts_seconds")]
    pub last_command_time: DateTime<Utc>,
}

impl Default for CliMetricsSummary {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            total_commands: 0,
            command_stats: HashMap::new(),
            overall_success_rate: 0.0,
            total_execution_time_ms: 0,
            total_bytes_transferred: 0,
            total_api_calls: 0,
            avg_command_duration_ms: 0.0,
            session_start: now,
            last_command_time: now,
        }
    }
}

/// Metric data point for time-series tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    /// Timestamp of the measurement
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,

    /// Metric value
    pub value: f64,

    /// Optional labels/tags
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub labels: HashMap<String, String>,
}

impl MetricPoint {
    /// Create new metric point with current timestamp
    pub fn new(value: f64) -> Self {
        Self {
            timestamp: Utc::now(),
            value,
            labels: HashMap::new(),
        }
    }

    /// Create metric point with labels
    pub fn with_labels(value: f64, labels: HashMap<String, String>) -> Self {
        Self {
            timestamp: Utc::now(),
            value,
            labels,
        }
    }
}

/// Counter metric for tracking incrementing values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counter {
    /// Counter name
    pub name: String,

    /// Current value
    pub value: u64,

    /// Last increment timestamp
    #[serde(with = "chrono::serde::ts_seconds")]
    pub last_updated: DateTime<Utc>,
}

impl Counter {
    /// Create new counter
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: 0,
            last_updated: Utc::now(),
        }
    }

    /// Increment counter by 1
    pub fn inc(&mut self) {
        self.value += 1;
        self.last_updated = Utc::now();
    }

    /// Increment counter by amount
    pub fn inc_by(&mut self, amount: u64) {
        self.value += amount;
        self.last_updated = Utc::now();
    }

    /// Reset counter to 0
    pub fn reset(&mut self) {
        self.value = 0;
        self.last_updated = Utc::now();
    }
}

/// Timer metric for measuring durations
#[derive(Debug, Clone)]
pub struct Timer {
    /// Timer name
    pub name: String,

    /// Start instant
    start: Instant,

    /// Recorded durations (ms)
    durations: Vec<u64>,
}

impl Timer {
    /// Create and start new timer
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            durations: Vec::new(),
        }
    }

    /// Record current elapsed time and restart
    pub fn record(&mut self) -> Duration {
        let duration = self.start.elapsed();
        self.durations.push(duration.as_millis() as u64);
        self.start = Instant::now();
        duration
    }

    /// Get average duration in milliseconds
    pub fn avg_ms(&self) -> f64 {
        if self.durations.is_empty() {
            return 0.0;
        }
        let sum: u64 = self.durations.iter().sum();
        sum as f64 / self.durations.len() as f64
    }

    /// Get percentile duration in milliseconds
    pub fn percentile(&self, p: f64) -> f64 {
        if self.durations.is_empty() {
            return 0.0;
        }
        let mut sorted = self.durations.clone();
        sorted.sort_unstable();
        let idx = ((p / 100.0) * sorted.len() as f64).floor() as usize;
        sorted.get(idx).copied().unwrap_or(0) as f64
    }

    /// Get all recorded durations
    pub fn durations(&self) -> &[u64] {
        &self.durations
    }
}

/// Storage configuration for metrics persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsStorageConfig {
    /// Maximum number of command metrics to retain
    pub max_command_history: usize,

    /// Maximum age of metrics in days
    pub retention_days: u32,

    /// Enable automatic cleanup of old metrics
    pub auto_cleanup: bool,

    /// Storage file path
    pub storage_path: String,

    /// Rotation threshold (number of entries)
    pub rotation_threshold: usize,
}

impl Default for MetricsStorageConfig {
    fn default() -> Self {
        Self {
            max_command_history: 1000,
            retention_days: 30,
            auto_cleanup: true,
            storage_path: dirs::cache_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("riptide")
                .join("metrics.json")
                .to_string_lossy()
                .to_string(),
            rotation_threshold: 500,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_command_metrics_creation() {
        let metrics = CommandMetrics::new("extract");
        assert_eq!(metrics.command_name, "extract");
        assert!(!metrics.success);
        assert_eq!(metrics.items_processed, 0);
    }

    #[test]
    fn test_command_metrics_completion() {
        let mut metrics = CommandMetrics::new("crawl");
        metrics.complete(Duration::from_millis(250));
        assert!(metrics.success);
        assert_eq!(metrics.duration_ms, Some(250));
        assert!(metrics.error.is_none());
    }

    #[test]
    fn test_command_metrics_failure() {
        let mut metrics = CommandMetrics::new("search");
        metrics.fail(Duration::from_millis(100), "network timeout");
        assert!(!metrics.success);
        assert_eq!(metrics.duration_ms, Some(100));
        assert_eq!(metrics.error, Some("network timeout".to_string()));
    }

    #[test]
    fn test_counter() {
        let mut counter = Counter::new("api_calls");
        assert_eq!(counter.value, 0);

        counter.inc();
        assert_eq!(counter.value, 1);

        counter.inc_by(5);
        assert_eq!(counter.value, 6);

        counter.reset();
        assert_eq!(counter.value, 0);
    }

    #[test]
    fn test_timer() {
        let mut timer = Timer::new("extraction");
        thread::sleep(Duration::from_millis(10));
        timer.record();

        thread::sleep(Duration::from_millis(20));
        timer.record();

        assert_eq!(timer.durations().len(), 2);
        assert!(timer.avg_ms() > 0.0);
    }

    #[test]
    fn test_command_aggregates() {
        let agg = CommandAggregates::new("extract");
        assert_eq!(agg.command, "extract");
        assert_eq!(agg.success_rate(), 0.0);
        assert_eq!(agg.error_rate(), 0.0);
    }
}
