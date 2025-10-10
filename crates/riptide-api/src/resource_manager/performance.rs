//! Performance monitoring and degradation detection.
//!
//! Tracks system performance metrics including:
//! - Render operation timing
//! - Timeout detection
//! - Performance degradation scoring
//! - Success/failure rates

use crate::resource_manager::{errors::Result, metrics::ResourceMetrics};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, warn};

/// Performance monitor for resource efficiency
///
/// Collects and analyzes performance metrics to detect degradation
/// and provide insights into system health.
pub struct PerformanceMonitor {
    render_times: Mutex<Vec<Duration>>,
    timeout_count: AtomicU64,
    degradation_score: AtomicU64, // Stored as u64 bits for atomic ops
    metrics: Arc<ResourceMetrics>,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub(crate) fn new(metrics: Arc<ResourceMetrics>) -> Result<Self> {
        debug!("Initializing performance monitor");

        Ok(Self {
            render_times: Mutex::new(Vec::new()),
            timeout_count: AtomicU64::new(0),
            degradation_score: AtomicU64::new(0),
            metrics,
        })
    }

    /// Record a timeout event
    ///
    /// Updates internal counters and may trigger degradation score updates.
    pub(crate) async fn record_timeout(&self) {
        let count = self.timeout_count.fetch_add(1, Ordering::Relaxed);
        self.metrics.timeouts_count.fetch_add(1, Ordering::Relaxed);

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
        let render_ops = self.metrics.render_operations.load(Ordering::Relaxed);
        let failed_renders = self.metrics.failed_renders.load(Ordering::Relaxed);

        // Calculate score based on failure rates
        let score = if render_ops == 0 {
            0.0
        } else {
            let timeout_rate = timeout_count as f64 / render_ops as f64;
            let failure_rate = failed_renders as f64 / render_ops as f64;

            // Weighted combination of metrics
            let score = (timeout_rate * 0.6) + (failure_rate * 0.4);
            score.min(1.0) // Cap at 1.0
        };

        self.degradation_score
            .store(score.to_bits(), Ordering::Relaxed);
    }

    /// Record render operation metrics
    ///
    /// Tracks timing, success/failure, and other metrics for analysis.
    ///
    /// # Arguments
    /// * `url` - The URL that was rendered
    /// * `duration` - Time taken for the operation
    /// * `success` - Whether the operation succeeded
    /// * `actions_executed` - Number of actions executed
    /// * `network_requests` - Number of network requests made
    pub async fn record_render_operation(
        &self,
        url: &str,
        duration: Duration,
        success: bool,
        actions_executed: u32,
        network_requests: u32,
    ) -> Result<()> {
        // Record render timing
        {
            let mut render_times = self.render_times.lock().await;
            render_times.push(duration);

            // Keep only recent measurements (last 100)
            if render_times.len() > 100 {
                render_times.remove(0);
            }
        }

        // Update metrics
        self.metrics
            .render_operations
            .fetch_add(1, Ordering::Relaxed);

        if success {
            self.metrics
                .successful_renders
                .fetch_add(1, Ordering::Relaxed);
        } else {
            self.metrics.failed_renders.fetch_add(1, Ordering::Relaxed);
        }

        // Update degradation score
        self.update_degradation_score().await;

        debug!(
            url = %url,
            duration_ms = duration.as_millis(),
            success = success,
            actions_executed = actions_executed,
            network_requests = network_requests,
            "Recorded render operation metrics"
        );

        Ok(())
    }

    /// Get performance statistics
    #[allow(dead_code)] // Reserved for future monitoring API
    pub async fn get_stats(&self) -> PerformanceStats {
        let render_times = self.render_times.lock().await;

        let avg_render_time = if !render_times.is_empty() {
            let sum: Duration = render_times.iter().sum();
            sum / render_times.len() as u32
        } else {
            Duration::ZERO
        };

        let p95_render_time = if !render_times.is_empty() {
            let mut sorted = render_times.clone();
            sorted.sort();
            let idx = (sorted.len() as f64 * 0.95) as usize;
            sorted.get(idx).copied().unwrap_or(Duration::ZERO)
        } else {
            Duration::ZERO
        };

        let render_ops = self.metrics.render_operations.load(Ordering::Relaxed);
        let successful = self.metrics.successful_renders.load(Ordering::Relaxed);
        let failed = self.metrics.failed_renders.load(Ordering::Relaxed);

        PerformanceStats {
            total_timeouts: self.timeout_count.load(Ordering::Relaxed),
            degradation_score: self.get_degradation_score().await,
            avg_render_time,
            p95_render_time,
            total_renders: render_ops,
            successful_renders: successful,
            failed_renders: failed,
            success_rate: if render_ops > 0 {
                successful as f64 / render_ops as f64
            } else {
                0.0
            },
        }
    }

    /// Get recent render times for analysis
    #[allow(dead_code)] // Reserved for future monitoring API
    pub async fn get_recent_render_times(&self, limit: usize) -> Vec<Duration> {
        let render_times = self.render_times.lock().await;
        let start = render_times.len().saturating_sub(limit);
        render_times[start..].to_vec()
    }

    /// Check if performance is degraded
    ///
    /// Returns `true` if degradation score exceeds 0.6 (critical threshold)
    #[allow(dead_code)] // Reserved for future monitoring API
    pub async fn is_degraded(&self) -> bool {
        self.get_degradation_score().await > 0.6
    }

    /// Reset all performance metrics (useful for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub async fn reset(&self) {
        self.render_times.lock().await.clear();
        self.timeout_count.store(0, Ordering::Relaxed);
        self.degradation_score.store(0, Ordering::Relaxed);
    }
}

/// Performance statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct PerformanceStats {
    pub total_timeouts: u64,
    pub degradation_score: f64,
    pub avg_render_time: Duration,
    pub p95_render_time: Duration,
    pub total_renders: u64,
    pub successful_renders: u64,
    pub failed_renders: u64,
    pub success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_monitor_creation() {
        let metrics = Arc::new(ResourceMetrics::new());
        let monitor = PerformanceMonitor::new(metrics).unwrap();

        assert_eq!(monitor.get_degradation_score().await, 0.0);
    }

    #[tokio::test]
    async fn test_timeout_recording() {
        let metrics = Arc::new(ResourceMetrics::new());
        let monitor = PerformanceMonitor::new(metrics.clone()).unwrap();

        monitor.record_timeout().await;
        assert_eq!(monitor.timeout_count.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.timeouts_count.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_render_operation_recording() {
        let metrics = Arc::new(ResourceMetrics::new());
        let monitor = PerformanceMonitor::new(metrics.clone()).unwrap();

        monitor
            .record_render_operation(
                "https://example.com",
                Duration::from_millis(500),
                true,
                5,
                10,
            )
            .await
            .unwrap();

        let stats = monitor.get_stats().await;
        assert_eq!(stats.total_renders, 1);
        assert_eq!(stats.successful_renders, 1);
        assert_eq!(stats.success_rate, 1.0);
        assert_eq!(stats.avg_render_time, Duration::from_millis(500));
    }

    #[tokio::test]
    async fn test_degradation_score_calculation() {
        let metrics = Arc::new(ResourceMetrics::new());
        let monitor = PerformanceMonitor::new(metrics.clone()).unwrap();

        // Record successful operations
        for _ in 0..10 {
            monitor
                .record_render_operation(
                    "https://example.com",
                    Duration::from_millis(100),
                    true,
                    1,
                    1,
                )
                .await
                .unwrap();
        }

        // Should have low degradation
        assert!(monitor.get_degradation_score().await < 0.1);

        // Record failures
        for _ in 0..5 {
            monitor
                .record_render_operation(
                    "https://example.com",
                    Duration::from_millis(100),
                    false,
                    1,
                    1,
                )
                .await
                .unwrap();
        }

        // Should have higher degradation
        assert!(monitor.get_degradation_score().await > 0.1);
    }

    #[tokio::test]
    async fn test_performance_statistics() {
        let metrics = Arc::new(ResourceMetrics::new());
        let monitor = PerformanceMonitor::new(metrics).unwrap();

        // Record various operations
        monitor
            .record_render_operation(
                "https://example.com",
                Duration::from_millis(100),
                true,
                1,
                1,
            )
            .await
            .unwrap();
        monitor
            .record_render_operation(
                "https://example.com",
                Duration::from_millis(200),
                true,
                1,
                1,
            )
            .await
            .unwrap();
        monitor
            .record_render_operation(
                "https://example.com",
                Duration::from_millis(150),
                false,
                1,
                1,
            )
            .await
            .unwrap();

        let stats = monitor.get_stats().await;
        assert_eq!(stats.total_renders, 3);
        assert_eq!(stats.successful_renders, 2);
        assert_eq!(stats.failed_renders, 1);
        assert!((stats.success_rate - 0.666).abs() < 0.01);
        assert!(stats.avg_render_time.as_millis() > 100);
    }

    #[tokio::test]
    async fn test_is_degraded() {
        let metrics = Arc::new(ResourceMetrics::new());
        let monitor = PerformanceMonitor::new(metrics).unwrap();

        // Fresh monitor should not be degraded
        assert!(!monitor.is_degraded().await);

        // Record many failures to trigger degradation
        for _ in 0..10 {
            monitor
                .record_render_operation(
                    "https://example.com",
                    Duration::from_millis(100),
                    false,
                    1,
                    1,
                )
                .await
                .unwrap();
            monitor.record_timeout().await;
        }

        // Should be degraded now
        assert!(monitor.is_degraded().await);
    }
}
