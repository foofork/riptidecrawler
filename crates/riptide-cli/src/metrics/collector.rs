//! Lightweight metrics collection with minimal overhead (< 5ms)
//!
//! This module provides thread-safe, low-latency metrics collection
//! designed specifically for CLI command tracking.

use super::types::{CommandMetrics, Counter, MetricPoint};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Thread-safe metrics collector optimized for CLI operations
pub struct MetricsCollector {
    /// Active command metrics (currently running commands)
    active_commands: Arc<RwLock<HashMap<String, CommandTracker>>>,

    /// Global counters for various metrics
    counters: Arc<RwLock<HashMap<String, Counter>>>,

    /// Time-series data points
    time_series: Arc<RwLock<HashMap<String, Vec<MetricPoint>>>>,

    /// Collector start time for uptime tracking
    start_time: Instant,
}

/// Tracks a command execution in progress
#[derive(Debug)]
struct CommandTracker {
    /// Command metrics being built
    metrics: CommandMetrics,

    /// Start instant for duration calculation
    start: Instant,

    /// Peak memory usage during execution
    peak_memory: u64,

    /// Memory sampling instant
    last_memory_check: Instant,
}

impl CommandTracker {
    fn new(command_name: impl Into<String>) -> Self {
        Self {
            metrics: CommandMetrics::new(command_name),
            start: Instant::now(),
            peak_memory: 0,
            last_memory_check: Instant::now(),
        }
    }

    /// Update peak memory if needed (throttled to avoid overhead)
    fn update_memory(&mut self) {
        // Only check memory every 100ms to minimize overhead
        if self.last_memory_check.elapsed() < Duration::from_millis(100) {
            return;
        }

        if let Some(current) = get_current_memory_usage() {
            if current > self.peak_memory {
                self.peak_memory = current;
                self.metrics.peak_memory_bytes = current;
            }
        }

        self.last_memory_check = Instant::now();
    }

    /// Finalize metrics with duration
    fn finalize(&mut self, success: bool, error: Option<String>) -> CommandMetrics {
        let duration = self.start.elapsed();

        if success {
            self.metrics.complete(duration);
        } else if let Some(err) = error {
            self.metrics.fail(duration, err);
        }

        self.metrics.clone()
    }
}

impl MetricsCollector {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self {
            active_commands: Arc::new(RwLock::new(HashMap::new())),
            counters: Arc::new(RwLock::new(HashMap::new())),
            time_series: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    /// Start tracking a command execution
    ///
    /// Returns a tracking ID to be used when completing the command
    pub fn start_command(&self, command_name: impl Into<String>) -> Result<String> {
        let command_name = command_name.into();
        let tracking_id = format!("{}-{}", command_name, Instant::now().elapsed().as_nanos());

        let tracker = CommandTracker::new(command_name.clone());

        {
            let mut active = self
                .active_commands
                .write()
                .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;
            active.insert(tracking_id.clone(), tracker);
        }

        // Increment command counter
        self.increment_counter(&format!("command.{}.started", command_name))?;

        Ok(tracking_id)
    }

    /// Record progress during command execution (items processed, bytes, etc.)
    pub fn record_progress(
        &self,
        tracking_id: &str,
        items_processed: u64,
        bytes_transferred: u64,
        cache_hits: u64,
        api_calls: u64,
    ) -> Result<()> {
        let mut active = self
            .active_commands
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;

        if let Some(tracker) = active.get_mut(tracking_id) {
            tracker.metrics.items_processed = items_processed;
            tracker.metrics.bytes_transferred = bytes_transferred;
            tracker.metrics.cache_hits = cache_hits;
            tracker.metrics.api_calls = api_calls;
            tracker.update_memory();
        }

        Ok(())
    }

    /// Complete a command execution successfully
    pub fn complete_command(&self, tracking_id: &str) -> Result<CommandMetrics> {
        let mut active = self
            .active_commands
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;

        let tracker = active
            .remove(tracking_id)
            .ok_or_else(|| anyhow::anyhow!("Tracking ID not found: {}", tracking_id))?;

        let mut tracker = tracker;
        let metrics = tracker.finalize(true, None);

        // Increment success counter
        self.increment_counter(&format!("command.{}.success", metrics.command_name))?;

        // Record duration in time series
        if let Some(duration_ms) = metrics.duration_ms {
            self.record_metric(
                &format!("command.{}.duration_ms", metrics.command_name),
                duration_ms as f64,
            )?;
        }

        Ok(metrics)
    }

    /// Complete a command execution with failure
    pub fn fail_command(
        &self,
        tracking_id: &str,
        error: impl Into<String>,
    ) -> Result<CommandMetrics> {
        let mut active = self
            .active_commands
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;

        let tracker = active
            .remove(tracking_id)
            .ok_or_else(|| anyhow::anyhow!("Tracking ID not found: {}", tracking_id))?;

        let mut tracker = tracker;
        let error_msg = error.into();
        let metrics = tracker.finalize(false, Some(error_msg.clone()));

        // Increment failure counter
        self.increment_counter(&format!("command.{}.failed", metrics.command_name))?;

        // Record error type
        self.increment_counter(&format!("error.{}", extract_error_type(&error_msg)))?;

        Ok(metrics)
    }

    /// Increment a named counter
    pub fn increment_counter(&self, name: &str) -> Result<()> {
        let mut counters = self
            .counters
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;

        counters
            .entry(name.to_string())
            .or_insert_with(|| Counter::new(name))
            .inc();

        Ok(())
    }

    /// Increment counter by specific amount
    pub fn increment_counter_by(&self, name: &str, amount: u64) -> Result<()> {
        let mut counters = self
            .counters
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;

        counters
            .entry(name.to_string())
            .or_insert_with(|| Counter::new(name))
            .inc_by(amount);

        Ok(())
    }

    /// Get current value of a counter
    pub fn get_counter(&self, name: &str) -> Result<u64> {
        let counters = self
            .counters
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock: {}", e))?;

        Ok(counters.get(name).map(|c| c.value).unwrap_or(0))
    }

    /// Record a metric data point
    pub fn record_metric(&self, metric_name: &str, value: f64) -> Result<()> {
        let mut series = self
            .time_series
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;

        series
            .entry(metric_name.to_string())
            .or_insert_with(Vec::new)
            .push(MetricPoint::new(value));

        // Keep only last 1000 points per metric to avoid unbounded growth
        if let Some(points) = series.get_mut(metric_name) {
            if points.len() > 1000 {
                points.drain(0..(points.len() - 1000));
            }
        }

        Ok(())
    }

    /// Get all counter values
    pub fn get_all_counters(&self) -> Result<HashMap<String, u64>> {
        let counters = self
            .counters
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock: {}", e))?;

        Ok(counters.iter().map(|(k, v)| (k.clone(), v.value)).collect())
    }

    /// Get metric time series
    pub fn get_metric_series(&self, metric_name: &str) -> Result<Vec<MetricPoint>> {
        let series = self
            .time_series
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock: {}", e))?;

        Ok(series.get(metric_name).cloned().unwrap_or_default())
    }

    /// Get number of active commands
    pub fn active_command_count(&self) -> Result<usize> {
        let active = self
            .active_commands
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock: {}", e))?;

        Ok(active.len())
    }

    /// Get collector uptime
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Reset all metrics (useful for testing)
    #[cfg(test)]
    pub fn reset(&self) -> Result<()> {
        let mut active = self
            .active_commands
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;
        let mut counters = self
            .counters
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;
        let mut series = self
            .time_series
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;

        active.clear();
        counters.clear();
        series.clear();

        Ok(())
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current process memory usage (lightweight implementation)
fn get_current_memory_usage() -> Option<u64> {
    // Use lightweight memory reading without heavy system calls
    #[cfg(target_os = "linux")]
    {
        // Read from /proc/self/statm for minimal overhead
        use std::fs;
        if let Ok(contents) = fs::read_to_string("/proc/self/statm") {
            // Second field is RSS in pages
            if let Some(rss_pages) = contents.split_whitespace().nth(1) {
                if let Ok(pages) = rss_pages.parse::<u64>() {
                    // Multiply by page size (typically 4096)
                    return Some(pages * 4096);
                }
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Fallback: use sysinfo but with minimal overhead
        use sysinfo::{Pid, ProcessesToUpdate, System};
        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::All, false); // Don't refresh CPU
        if let Some(process) = sys.process(Pid::from(std::process::id() as usize)) {
            return Some(process.memory() * 1024);
        }
    }

    None
}

/// Extract error type from error message (e.g., "NetworkError", "TimeoutError")
#[allow(dead_code)]
fn extract_error_type(error_msg: &str) -> String {
    // Try to extract error type from common patterns
    if error_msg.contains("timeout") || error_msg.contains("timed out") {
        return "timeout".to_string();
    }
    if error_msg.contains("network") || error_msg.contains("connection") {
        return "network".to_string();
    }
    if error_msg.contains("permission") || error_msg.contains("denied") {
        return "permission".to_string();
    }
    if error_msg.contains("not found") || error_msg.contains("404") {
        return "not_found".to_string();
    }
    if error_msg.contains("parse") || error_msg.contains("invalid") {
        return "parse".to_string();
    }

    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_collector_creation() {
        let collector = MetricsCollector::new();
        assert_eq!(collector.active_command_count().unwrap(), 0);
    }

    #[test]
    fn test_command_tracking() {
        let collector = MetricsCollector::new();

        let tracking_id = collector.start_command("extract").unwrap();
        assert_eq!(collector.active_command_count().unwrap(), 1);

        thread::sleep(Duration::from_millis(10));

        collector
            .record_progress(&tracking_id, 10, 1024, 5, 2)
            .unwrap();

        let metrics = collector.complete_command(&tracking_id).unwrap();

        assert_eq!(metrics.command_name, "extract");
        assert!(metrics.success);
        assert_eq!(metrics.items_processed, 10);
        assert_eq!(metrics.bytes_transferred, 1024);
        assert!(metrics.duration_ms.unwrap() >= 10);
    }

    #[test]
    fn test_command_failure() {
        let collector = MetricsCollector::new();

        let tracking_id = collector.start_command("crawl").unwrap();

        let metrics = collector
            .fail_command(&tracking_id, "Network timeout")
            .unwrap();

        assert!(!metrics.success);
        assert_eq!(metrics.error, Some("Network timeout".to_string()));
    }

    #[test]
    fn test_counter_operations() {
        let collector = MetricsCollector::new();

        collector.increment_counter("test_counter").unwrap();
        assert_eq!(collector.get_counter("test_counter").unwrap(), 1);

        collector.increment_counter_by("test_counter", 5).unwrap();
        assert_eq!(collector.get_counter("test_counter").unwrap(), 6);
    }

    #[test]
    fn test_metric_recording() {
        let collector = MetricsCollector::new();

        collector.record_metric("test_metric", 42.5).unwrap();
        collector.record_metric("test_metric", 38.2).unwrap();

        let series = collector.get_metric_series("test_metric").unwrap();
        assert_eq!(series.len(), 2);
        assert_eq!(series[0].value, 42.5);
        assert_eq!(series[1].value, 38.2);
    }

    #[test]
    fn test_error_type_extraction() {
        assert_eq!(extract_error_type("Connection timeout"), "timeout");
        assert_eq!(extract_error_type("Network error occurred"), "network");
        assert_eq!(extract_error_type("Permission denied"), "permission");
        assert_eq!(extract_error_type("Resource not found"), "not_found");
        assert_eq!(extract_error_type("Parse error"), "parse");
        assert_eq!(extract_error_type("Something weird"), "unknown");
    }
}
