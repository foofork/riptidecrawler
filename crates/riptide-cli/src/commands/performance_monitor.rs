/// Performance Monitoring and Metrics Collection
///
/// This module provides comprehensive performance tracking for extraction operations
/// including timing, memory usage, and engine selection metrics.
///
/// **Note**: This is infrastructure code for Phase 5+ monitoring features.
/// Currently unused but designed for future API integration.
use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Performance metrics for a single extraction operation
/// Infrastructure: Used by performance monitoring system
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionMetrics {
    pub operation_id: String,
    pub url: Option<String>,
    pub engine_used: String,
    pub total_duration_ms: u64,
    pub fetch_duration_ms: Option<u64>,
    pub extraction_duration_ms: Option<u64>,
    pub wasm_init_duration_ms: Option<u64>,
    pub browser_launch_duration_ms: Option<u64>,
    pub content_size_bytes: usize,
    pub confidence_score: Option<f64>,
    pub success: bool,
    pub error_message: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Stage-based timing tracker
/// Infrastructure: Timing utility for performance monitoring
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StageTimer {
    stages: HashMap<String, Duration>,
    current_stage: Option<(String, Instant)>,
}

impl StageTimer {
    pub fn new() -> Self {
        Self {
            stages: HashMap::new(),
            current_stage: None,
        }
    }

    /// Start timing a stage
    pub fn start_stage(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.current_stage = Some((name, Instant::now()));
    }

    /// End the current stage and record duration
    pub fn end_stage(&mut self) -> Option<Duration> {
        if let Some((name, start)) = self.current_stage.take() {
            let duration = start.elapsed();
            self.stages.insert(name, duration);
            Some(duration)
        } else {
            None
        }
    }

    /// Get duration for a specific stage
    pub fn get_stage(&self, name: &str) -> Option<Duration> {
        self.stages.get(name).copied()
    }

    /// Get all stages
    pub fn stages(&self) -> &HashMap<String, Duration> {
        &self.stages
    }
}

impl Default for StageTimer {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance monitor for tracking extraction operations
/// Infrastructure: Core monitoring system for Phase 5+
#[allow(dead_code)]
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<Vec<ExtractionMetrics>>>,
    max_history: usize,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(max_history: usize) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
            max_history,
        }
    }

    /// Record extraction metrics
    pub async fn record(&self, metrics: ExtractionMetrics) -> Result<()> {
        let mut history = self.metrics.write().await;

        // Add new metrics
        history.push(metrics);

        // Trim history if needed
        if history.len() > self.max_history {
            let excess = history.len() - self.max_history;
            history.drain(0..excess);
        }

        Ok(())
    }

    /// Get aggregate statistics
    pub async fn get_stats(&self) -> PerformanceStats {
        let history = self.metrics.read().await;

        if history.is_empty() {
            return PerformanceStats::default();
        }

        let total_operations = history.len();
        let successful_operations = history.iter().filter(|m| m.success).count();

        let avg_duration: f64 = history
            .iter()
            .map(|m| m.total_duration_ms as f64)
            .sum::<f64>()
            / total_operations as f64;

        let avg_content_size: f64 = history
            .iter()
            .map(|m| m.content_size_bytes as f64)
            .sum::<f64>()
            / total_operations as f64;

        let mut engine_usage = HashMap::new();
        for metrics in history.iter() {
            *engine_usage.entry(metrics.engine_used.clone()).or_insert(0) += 1;
        }

        PerformanceStats {
            total_operations,
            successful_operations,
            failed_operations: total_operations - successful_operations,
            success_rate: successful_operations as f64 / total_operations as f64,
            avg_duration_ms: avg_duration,
            avg_content_size_bytes: avg_content_size,
            engine_usage,
        }
    }

    /// Get recent metrics
    pub async fn get_recent(&self, count: usize) -> Vec<ExtractionMetrics> {
        let history = self.metrics.read().await;
        let start = history.len().saturating_sub(count);
        history[start..].to_vec()
    }

    /// Clear all metrics
    pub async fn clear(&self) {
        let mut history = self.metrics.write().await;
        history.clear();
    }

    /// Export metrics as JSON
    pub async fn export_json(&self) -> Result<String> {
        let history = self.metrics.read().await;
        serde_json::to_string_pretty(&*history)
            .map_err(|e| anyhow::anyhow!("Failed to serialize metrics: {}", e))
    }
}

/// Aggregate performance statistics
/// Infrastructure: Stats aggregation for monitoring
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub success_rate: f64,
    pub avg_duration_ms: f64,
    pub avg_content_size_bytes: f64,
    pub engine_usage: HashMap<String, usize>,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            success_rate: 0.0,
            avg_duration_ms: 0.0,
            avg_content_size_bytes: 0.0,
            engine_usage: HashMap::new(),
        }
    }
}

/// Global performance monitor instance (using Arc for shared ownership)
/// Infrastructure: Global singleton for monitoring system
#[allow(dead_code)]
static GLOBAL_MONITOR: Lazy<Arc<PerformanceMonitor>> =
    Lazy::new(|| Arc::new(PerformanceMonitor::new(1000)));

/// Get the global performance monitor as Arc (for shared ownership)
/// Infrastructure: Accessor for global monitoring instance
#[allow(dead_code)]
pub fn global_monitor() -> &'static PerformanceMonitor {
    &GLOBAL_MONITOR
}

impl PerformanceMonitor {
    /// Get the global performance monitor instance (Arc for shared ownership)
    pub fn get_global() -> Arc<Self> {
        Arc::clone(&GLOBAL_MONITOR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_timer() {
        let mut timer = StageTimer::new();

        timer.start_stage("fetch");
        std::thread::sleep(Duration::from_millis(10));
        timer.end_stage();

        assert!(timer.get_stage("fetch").is_some());
        assert!(timer.get_stage("fetch").unwrap() >= Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new(10);

        let metrics = ExtractionMetrics {
            operation_id: "test-1".to_string(),
            url: Some("https://example.com".to_string()),
            engine_used: "wasm".to_string(),
            total_duration_ms: 100,
            fetch_duration_ms: Some(50),
            extraction_duration_ms: Some(50),
            wasm_init_duration_ms: None,
            browser_launch_duration_ms: None,
            content_size_bytes: 1000,
            confidence_score: Some(0.95),
            success: true,
            error_message: None,
            timestamp: chrono::Utc::now(),
        };

        monitor.record(metrics).await.unwrap();

        let stats = monitor.get_stats().await;
        assert_eq!(stats.total_operations, 1);
        assert_eq!(stats.successful_operations, 1);
        assert_eq!(stats.success_rate, 1.0);
    }
}
