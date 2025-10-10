//! Resource metrics collection and reporting.
//!
//! Provides comprehensive metrics for monitoring resource utilization,
//! performance, and system health.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Comprehensive resource metrics collection
///
/// All metrics use atomic types for thread-safe concurrent updates.
/// Metrics are designed to be read frequently and updated occasionally.
#[derive(Debug, Default)]
pub struct ResourceMetrics {
    // Browser Pool Metrics
    /// Maximum browser pool capacity (set at initialization)
    pub headless_pool_size: AtomicUsize,
    /// Currently active browser operations
    pub headless_active: AtomicUsize,

    // PDF Processing Metrics
    /// Currently active PDF processing operations
    pub pdf_active: AtomicUsize,

    // WASM Instance Metrics
    /// Total number of WASM instances
    pub wasm_instances: AtomicUsize,

    // Memory Metrics
    /// Current memory usage in megabytes
    pub memory_usage_mb: AtomicUsize,

    // Rate Limiting Metrics
    /// Total rate limit violations
    pub rate_limit_hits: AtomicU64,

    // Performance Metrics
    /// Total operation timeouts
    pub timeouts_count: AtomicU64,
    /// Total cleanup operations performed
    pub cleanup_operations: AtomicU64,
    /// Total garbage collection triggers
    pub gc_triggers: AtomicU64,

    // Render Operation Metrics
    /// Total render operations attempted
    pub render_operations: AtomicU64,
    /// Successful render operations
    pub successful_renders: AtomicU64,
    /// Failed render operations
    pub failed_renders: AtomicU64,
}

impl ResourceMetrics {
    /// Create a new metrics instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a snapshot of current metrics
    #[allow(dead_code)] // Reserved for future monitoring API
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            headless_pool_size: self.headless_pool_size.load(Ordering::Relaxed),
            headless_active: self.headless_active.load(Ordering::Relaxed),
            pdf_active: self.pdf_active.load(Ordering::Relaxed),
            wasm_instances: self.wasm_instances.load(Ordering::Relaxed),
            memory_usage_mb: self.memory_usage_mb.load(Ordering::Relaxed),
            rate_limit_hits: self.rate_limit_hits.load(Ordering::Relaxed),
            timeouts_count: self.timeouts_count.load(Ordering::Relaxed),
            cleanup_operations: self.cleanup_operations.load(Ordering::Relaxed),
            gc_triggers: self.gc_triggers.load(Ordering::Relaxed),
            render_operations: self.render_operations.load(Ordering::Relaxed),
            successful_renders: self.successful_renders.load(Ordering::Relaxed),
            failed_renders: self.failed_renders.load(Ordering::Relaxed),
        }
    }

    /// Reset all metrics to zero (useful for testing)
    #[cfg(test)]
    pub fn reset(&self) {
        self.headless_pool_size.store(0, Ordering::Relaxed);
        self.headless_active.store(0, Ordering::Relaxed);
        self.pdf_active.store(0, Ordering::Relaxed);
        self.wasm_instances.store(0, Ordering::Relaxed);
        self.memory_usage_mb.store(0, Ordering::Relaxed);
        self.rate_limit_hits.store(0, Ordering::Relaxed);
        self.timeouts_count.store(0, Ordering::Relaxed);
        self.cleanup_operations.store(0, Ordering::Relaxed);
        self.gc_triggers.store(0, Ordering::Relaxed);
        self.render_operations.store(0, Ordering::Relaxed);
        self.successful_renders.store(0, Ordering::Relaxed);
        self.failed_renders.store(0, Ordering::Relaxed);
    }

    /// Calculate render success rate (0.0-1.0)
    #[allow(dead_code)] // Reserved for future monitoring API
    pub fn render_success_rate(&self) -> f64 {
        let total = self.render_operations.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let successful = self.successful_renders.load(Ordering::Relaxed);
        successful as f64 / total as f64
    }

    /// Calculate browser pool utilization (0.0-1.0)
    #[allow(dead_code)] // Reserved for future monitoring API
    pub fn browser_pool_utilization(&self) -> f64 {
        let capacity = self.headless_pool_size.load(Ordering::Relaxed);
        if capacity == 0 {
            return 0.0;
        }
        let active = self.headless_active.load(Ordering::Relaxed);
        active as f64 / capacity as f64
    }
}

/// Point-in-time snapshot of resource metrics
///
/// This struct holds a consistent view of all metrics at a specific moment,
/// useful for reporting and analysis without worrying about concurrent updates.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    pub headless_pool_size: usize,
    pub headless_active: usize,
    pub pdf_active: usize,
    pub wasm_instances: usize,
    pub memory_usage_mb: usize,
    pub rate_limit_hits: u64,
    pub timeouts_count: u64,
    pub cleanup_operations: u64,
    pub gc_triggers: u64,
    pub render_operations: u64,
    pub successful_renders: u64,
    pub failed_renders: u64,
}

impl MetricsSnapshot {
    /// Calculate render success rate
    pub fn render_success_rate(&self) -> f64 {
        if self.render_operations == 0 {
            return 0.0;
        }
        self.successful_renders as f64 / self.render_operations as f64
    }

    /// Calculate browser pool utilization
    pub fn browser_pool_utilization(&self) -> f64 {
        if self.headless_pool_size == 0 {
            return 0.0;
        }
        self.headless_active as f64 / self.headless_pool_size as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_snapshot() {
        let metrics = ResourceMetrics::new();

        metrics.headless_pool_size.store(10, Ordering::Relaxed);
        metrics.headless_active.store(5, Ordering::Relaxed);
        metrics.render_operations.store(100, Ordering::Relaxed);
        metrics.successful_renders.store(95, Ordering::Relaxed);

        let snapshot = metrics.snapshot();

        assert_eq!(snapshot.headless_pool_size, 10);
        assert_eq!(snapshot.headless_active, 5);
        assert_eq!(snapshot.browser_pool_utilization(), 0.5);
        assert_eq!(snapshot.render_success_rate(), 0.95);
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = ResourceMetrics::new();

        metrics.render_operations.store(100, Ordering::Relaxed);
        metrics.successful_renders.store(95, Ordering::Relaxed);

        metrics.reset();

        assert_eq!(metrics.render_operations.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.successful_renders.load(Ordering::Relaxed), 0);
    }
}
