//! Performance monitoring and degradation detection for business operations.
//!
//! This module tracks performance metrics at the business logic layer:
//! - Operation timing (render, extraction, PDF processing)
//! - Success/failure rates
//! - Degradation scoring
//! - Timeout detection
//!
//! Moved from riptide-api/src/resource_manager/performance.rs (Sprint 4.4)

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, warn};

use riptide_types::error::riptide_error::RiptideError;

/// Result type for performance operations
pub type Result<T> = std::result::Result<T, RiptideError>;

/// Performance monitor for business operations
///
/// Collects and analyzes performance metrics to detect degradation
/// and provide insights into system health.
///
/// # Example
///
/// ```rust,ignore
/// use riptide_facade::metrics::PerformanceMonitor;
///
/// let monitor = PerformanceMonitor::new();
///
/// // Record operation
/// monitor.record_operation(
///     "extraction",
///     Duration::from_millis(500),
///     true,  // success
///     100,   // items processed
/// ).await?;
///
/// // Check degradation
/// if monitor.is_degraded().await {
///     // Take action
/// }
/// ```
pub struct PerformanceMonitor {
    /// Operation timings (recent 100 measurements)
    operation_times: Mutex<Vec<OperationTiming>>,
    /// Total timeout count
    timeout_count: AtomicU64,
    /// Degradation score (stored as u64 bits for atomic ops)
    degradation_score: AtomicU64,
    /// Total operations
    total_operations: AtomicU64,
    /// Successful operations
    successful_operations: AtomicU64,
    /// Failed operations
    failed_operations: AtomicU64,
}

/// Timing information for an operation
#[derive(Debug, Clone)]
struct OperationTiming {
    /// Operation type (e.g., "render", "extraction", "pdf")
    #[allow(dead_code)] // Used in future sprints for detailed timing analysis
    operation_type: String,
    /// Duration of the operation
    duration: Duration,
    /// Timestamp of the operation
    #[allow(dead_code)] // Used in future sprints for time-series analysis
    timestamp: std::time::Instant,
    /// Whether the operation succeeded
    #[allow(dead_code)] // Used in future sprints for success rate calculations
    success: bool,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        debug!("Initializing performance monitor");

        Self {
            operation_times: Mutex::new(Vec::new()),
            timeout_count: AtomicU64::new(0),
            degradation_score: AtomicU64::new(0),
            total_operations: AtomicU64::new(0),
            successful_operations: AtomicU64::new(0),
            failed_operations: AtomicU64::new(0),
        }
    }

    /// Record a timeout event
    ///
    /// Updates internal counters and may trigger degradation score updates.
    pub async fn record_timeout(&self) {
        let count = self.timeout_count.fetch_add(1, Ordering::Relaxed);

        warn!(timeout_count = count + 1, "Operation timeout recorded");

        // Update degradation score based on timeout frequency
        self.update_degradation_score().await;
    }

    /// Get current degradation score (0.0-1.0)
    ///
    /// Higher scores indicate worse performance:
    /// - 0.0-0.3: Healthy
    /// - 0.3-0.6: Degraded
    /// - 0.6-1.0: Critical
    pub async fn get_degradation_score(&self) -> f64 {
        f64::from_bits(self.degradation_score.load(Ordering::Relaxed))
    }

    /// Update degradation score based on current metrics
    async fn update_degradation_score(&self) {
        let timeout_count = self.timeout_count.load(Ordering::Relaxed);
        let total_ops = self.total_operations.load(Ordering::Relaxed);
        let failed_ops = self.failed_operations.load(Ordering::Relaxed);

        // Calculate score based on failure rates
        let score = if total_ops == 0 {
            0.0
        } else {
            let timeout_rate = timeout_count as f64 / total_ops as f64;
            let failure_rate = failed_ops as f64 / total_ops as f64;

            // Weighted combination of metrics
            let score = (timeout_rate * 0.6) + (failure_rate * 0.4);
            score.min(1.0) // Cap at 1.0
        };

        self.degradation_score
            .store(score.to_bits(), Ordering::Relaxed);
    }

    /// Record business operation metrics
    ///
    /// Tracks timing, success/failure, and other metrics for analysis.
    ///
    /// # Arguments
    /// * `operation_type` - Type of operation (e.g., "render", "extraction")
    /// * `duration` - Time taken for the operation
    /// * `success` - Whether the operation succeeded
    /// * `items_processed` - Number of items processed (for throughput tracking)
    pub async fn record_operation(
        &self,
        operation_type: &str,
        duration: Duration,
        success: bool,
        items_processed: u32,
    ) -> Result<()> {
        // Record operation timing
        {
            let mut operation_times = self.operation_times.lock().await;
            operation_times.push(OperationTiming {
                operation_type: operation_type.to_string(),
                duration,
                timestamp: std::time::Instant::now(),
                success,
            });

            // Keep only recent measurements (last 100)
            if operation_times.len() > 100 {
                operation_times.remove(0);
            }
        }

        // Update metrics
        self.total_operations.fetch_add(1, Ordering::Relaxed);

        if success {
            self.successful_operations.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_operations.fetch_add(1, Ordering::Relaxed);
        }

        // Update degradation score
        self.update_degradation_score().await;

        debug!(
            operation_type = %operation_type,
            duration_ms = duration.as_millis(),
            success = success,
            items_processed = items_processed,
            "Recorded operation metrics"
        );

        Ok(())
    }

    /// Get performance statistics
    pub async fn get_stats(&self) -> PerformanceStats {
        let operation_times = self.operation_times.lock().await;

        let avg_duration = if !operation_times.is_empty() {
            let sum: Duration = operation_times.iter().map(|t| t.duration).sum();
            let count = u32::try_from(operation_times.len()).unwrap_or(u32::MAX);
            sum / count
        } else {
            Duration::ZERO
        };

        let p95_duration = if !operation_times.is_empty() {
            let mut sorted: Vec<_> = operation_times.iter().map(|t| t.duration).collect();
            sorted.sort();
            let idx = (sorted.len() as f64 * 0.95) as usize;
            sorted.get(idx).copied().unwrap_or(Duration::ZERO)
        } else {
            Duration::ZERO
        };

        let total_ops = self.total_operations.load(Ordering::Relaxed);
        let successful = self.successful_operations.load(Ordering::Relaxed);
        let failed = self.failed_operations.load(Ordering::Relaxed);

        PerformanceStats {
            total_timeouts: self.timeout_count.load(Ordering::Relaxed),
            degradation_score: self.get_degradation_score().await,
            avg_duration,
            p95_duration,
            total_operations: total_ops,
            successful_operations: successful,
            failed_operations: failed,
            success_rate: if total_ops > 0 {
                successful as f64 / total_ops as f64
            } else {
                0.0
            },
        }
    }

    /// Get recent operation times for analysis
    ///
    /// # Arguments
    /// * `limit` - Maximum number of recent timings to return
    pub async fn get_recent_timings(&self, limit: usize) -> Vec<Duration> {
        let operation_times = self.operation_times.lock().await;
        let start = operation_times.len().saturating_sub(limit);
        operation_times[start..]
            .iter()
            .map(|t| t.duration)
            .collect()
    }

    /// Check if performance is degraded
    ///
    /// Returns `true` if degradation score exceeds 0.6 (critical threshold)
    pub async fn is_degraded(&self) -> bool {
        self.get_degradation_score().await > 0.6
    }

    /// Reset all performance metrics (useful for testing)
    #[cfg(test)]
    pub async fn reset(&self) {
        self.operation_times.lock().await.clear();
        self.timeout_count.store(0, Ordering::Relaxed);
        self.degradation_score.store(0_u64, Ordering::Relaxed);
        self.total_operations.store(0, Ordering::Relaxed);
        self.successful_operations.store(0, Ordering::Relaxed);
        self.failed_operations.store(0, Ordering::Relaxed);
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct PerformanceStats {
    /// Total operation timeouts
    pub total_timeouts: u64,
    /// Degradation score (0.0-1.0)
    pub degradation_score: f64,
    /// Average operation duration
    pub avg_duration: Duration,
    /// 95th percentile operation duration
    pub p95_duration: Duration,
    /// Total operations executed
    pub total_operations: u64,
    /// Successful operations
    pub successful_operations: u64,
    /// Failed operations
    pub failed_operations: u64,
    /// Success rate (0.0-1.0)
    pub success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        assert_eq!(monitor.get_degradation_score().await, 0.0);
    }

    #[tokio::test]
    async fn test_timeout_recording() {
        let monitor = PerformanceMonitor::new();

        monitor.record_timeout().await;
        assert_eq!(monitor.timeout_count.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_operation_recording() {
        let monitor = PerformanceMonitor::new();

        monitor
            .record_operation("test_op", Duration::from_millis(500), true, 10)
            .await
            .unwrap();

        let stats = monitor.get_stats().await;
        assert_eq!(stats.total_operations, 1);
        assert_eq!(stats.successful_operations, 1);
        assert_eq!(stats.success_rate, 1.0);
        assert_eq!(stats.avg_duration, Duration::from_millis(500));
    }

    #[tokio::test]
    async fn test_degradation_score_calculation() {
        let monitor = PerformanceMonitor::new();

        // Record successful operations
        for _ in 0..10 {
            monitor
                .record_operation("test_op", Duration::from_millis(100), true, 1)
                .await
                .unwrap();
        }

        // Should have low degradation
        assert!(monitor.get_degradation_score().await < 0.1);

        // Record failures
        for _ in 0..5 {
            monitor
                .record_operation("test_op", Duration::from_millis(100), false, 1)
                .await
                .unwrap();
        }

        // Should have higher degradation
        assert!(monitor.get_degradation_score().await > 0.1);
    }

    #[tokio::test]
    async fn test_is_degraded() {
        let monitor = PerformanceMonitor::new();

        // Fresh monitor should not be degraded
        assert!(!monitor.is_degraded().await);

        // Record many failures to trigger degradation
        for _ in 0..10 {
            monitor
                .record_operation("test_op", Duration::from_millis(100), false, 1)
                .await
                .unwrap();
            monitor.record_timeout().await;
        }

        // Should be degraded now
        assert!(monitor.is_degraded().await);
    }
}
