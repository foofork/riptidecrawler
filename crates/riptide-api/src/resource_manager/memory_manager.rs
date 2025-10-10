//! Memory management with pressure detection and cleanup.
//!
//! Monitors system memory usage and provides:
//! - Memory allocation/deallocation tracking
//! - Pressure detection based on configurable thresholds
//! - Automatic cleanup triggers
//! - Garbage collection coordination

use crate::config::ApiConfig;
use crate::resource_manager::{errors::Result, metrics::ResourceMetrics};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    Arc,
};
use tracing::{info, warn};

/// Memory manager with pressure detection and cleanup
///
/// Tracks memory allocations and triggers cleanup or GC when thresholds are exceeded.
/// All operations are thread-safe and non-blocking.
pub struct MemoryManager {
    config: ApiConfig,
    current_usage: AtomicUsize,
    pressure_detected: AtomicBool,
    last_cleanup: AtomicU64,
    last_gc: AtomicU64,
    metrics: Arc<ResourceMetrics>,
}

impl MemoryManager {
    /// Create a new memory manager
    pub(crate) fn new(config: ApiConfig, metrics: Arc<ResourceMetrics>) -> Result<Self> {
        info!(
            limit_mb = config.memory.global_memory_limit_mb,
            pressure_threshold = config.memory.pressure_threshold,
            gc_threshold_mb = config.memory.gc_trigger_threshold_mb,
            "Initializing memory manager"
        );

        Ok(Self {
            config,
            current_usage: AtomicUsize::new(0),
            pressure_detected: AtomicBool::new(false),
            last_cleanup: AtomicU64::new(0),
            last_gc: AtomicU64::new(0),
            metrics,
        })
    }

    /// Track a memory allocation
    ///
    /// Updates internal counters and checks for memory pressure.
    /// If pressure is detected, it will be reflected in subsequent calls to `is_under_pressure()`.
    ///
    /// # Arguments
    /// * `size_mb` - Size of allocation in megabytes
    pub(crate) fn track_allocation(&self, size_mb: usize) {
        let current = self.current_usage.fetch_add(size_mb, Ordering::Relaxed);
        let new_usage = current + size_mb;

        self.metrics
            .memory_usage_mb
            .store(new_usage, Ordering::Relaxed);

        // Check for memory pressure
        if self.config.is_memory_pressure(new_usage) {
            if !self.pressure_detected.swap(true, Ordering::Relaxed) {
                warn!(
                    current_mb = new_usage,
                    limit_mb = self.config.memory.global_memory_limit_mb,
                    threshold = self.config.memory.pressure_threshold,
                    "Memory pressure detected"
                );
            }
        }
    }

    /// Track a memory deallocation
    ///
    /// Updates internal counters and may clear pressure status if usage drops below threshold.
    ///
    /// # Arguments
    /// * `size_mb` - Size of deallocation in megabytes
    pub(crate) fn track_deallocation(&self, size_mb: usize) {
        let current = self.current_usage.fetch_sub(size_mb, Ordering::Relaxed);
        let new_usage = current.saturating_sub(size_mb);

        self.metrics
            .memory_usage_mb
            .store(new_usage, Ordering::Relaxed);

        // Update pressure status
        if !self.config.is_memory_pressure(new_usage) {
            if self.pressure_detected.swap(false, Ordering::Relaxed) {
                info!(
                    current_mb = new_usage,
                    "Memory pressure cleared"
                );
            }
        }
    }

    /// Check if system is under memory pressure
    ///
    /// Returns `true` if current memory usage exceeds the configured threshold.
    pub fn is_under_pressure(&self) -> bool {
        self.pressure_detected.load(Ordering::Relaxed)
    }

    /// Get current memory usage in megabytes
    pub fn current_usage_mb(&self) -> usize {
        self.current_usage.load(Ordering::Relaxed)
    }

    /// Get memory usage as a percentage of limit
    pub fn usage_percentage(&self) -> f64 {
        let current = self.current_usage_mb();
        let limit = self.config.memory.global_memory_limit_mb;
        if limit == 0 {
            return 0.0;
        }
        (current as f64 / limit as f64) * 100.0
    }

    /// Trigger memory cleanup
    ///
    /// Records the cleanup timestamp and updates metrics.
    /// Actual cleanup implementation should be done by caller.
    pub async fn trigger_cleanup(&self) {
        info!(
            current_mb = self.current_usage_mb(),
            "Triggering memory cleanup"
        );

        let timestamp = current_timestamp_secs();
        self.last_cleanup.store(timestamp, Ordering::Relaxed);
        self.metrics
            .cleanup_operations
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Trigger garbage collection
    ///
    /// Records the GC timestamp and updates metrics.
    /// Actual GC should be implemented by runtime/caller.
    pub async fn trigger_gc(&self) {
        info!(
            current_mb = self.current_usage_mb(),
            "Triggering garbage collection"
        );

        let timestamp = current_timestamp_secs();
        self.last_gc.store(timestamp, Ordering::Relaxed);
        self.metrics.gc_triggers.fetch_add(1, Ordering::Relaxed);
    }

    /// Check if garbage collection should be triggered
    ///
    /// Returns `true` if current usage exceeds the GC trigger threshold.
    pub fn should_trigger_gc(&self) -> bool {
        self.current_usage.load(Ordering::Relaxed) >= self.config.memory.gc_trigger_threshold_mb
    }

    /// Get time since last cleanup (in seconds)
    pub fn time_since_last_cleanup(&self) -> Option<u64> {
        let last = self.last_cleanup.load(Ordering::Relaxed);
        if last == 0 {
            return None;
        }
        Some(current_timestamp_secs().saturating_sub(last))
    }

    /// Get time since last GC (in seconds)
    pub fn time_since_last_gc(&self) -> Option<u64> {
        let last = self.last_gc.load(Ordering::Relaxed);
        if last == 0 {
            return None;
        }
        Some(current_timestamp_secs().saturating_sub(last))
    }

    /// Get memory statistics
    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            current_usage_mb: self.current_usage_mb(),
            usage_percentage: self.usage_percentage(),
            is_under_pressure: self.is_under_pressure(),
            last_cleanup_secs_ago: self.time_since_last_cleanup(),
            last_gc_secs_ago: self.time_since_last_gc(),
            cleanup_count: self.metrics.cleanup_operations.load(Ordering::Relaxed),
            gc_count: self.metrics.gc_triggers.load(Ordering::Relaxed),
        }
    }
}

/// Memory manager statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct MemoryStats {
    pub current_usage_mb: usize,
    pub usage_percentage: f64,
    pub is_under_pressure: bool,
    pub last_cleanup_secs_ago: Option<u64>,
    pub last_gc_secs_ago: Option<u64>,
    pub cleanup_count: u64,
    pub gc_count: u64,
}

/// Get current timestamp in seconds since UNIX epoch
fn current_timestamp_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_else(|e| {
            warn!(error = %e, "Failed to get system time, using 0");
            0
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> ApiConfig {
        let mut config = ApiConfig::default();
        config.memory.global_memory_limit_mb = 1000;
        config.memory.pressure_threshold = 0.8; // 80%
        config.memory.gc_trigger_threshold_mb = 900;
        config
    }

    #[tokio::test]
    async fn test_memory_tracking() {
        let config = test_config();
        let metrics = Arc::new(ResourceMetrics::new());
        let manager = MemoryManager::new(config, metrics).unwrap();

        assert_eq!(manager.current_usage_mb(), 0);
        assert!(!manager.is_under_pressure());

        manager.track_allocation(500);
        assert_eq!(manager.current_usage_mb(), 500);
        assert!(!manager.is_under_pressure());

        manager.track_deallocation(200);
        assert_eq!(manager.current_usage_mb(), 300);
    }

    #[tokio::test]
    async fn test_memory_pressure_detection() {
        let config = test_config();
        let metrics = Arc::new(ResourceMetrics::new());
        let manager = MemoryManager::new(config, metrics).unwrap();

        // Below threshold - no pressure
        manager.track_allocation(700);
        assert!(!manager.is_under_pressure());

        // Above threshold (80% of 1000 = 800) - pressure detected
        manager.track_allocation(200);
        assert!(manager.is_under_pressure());

        // Back below threshold - pressure cleared
        manager.track_deallocation(300);
        assert!(!manager.is_under_pressure());
    }

    #[tokio::test]
    async fn test_gc_trigger_threshold() {
        let config = test_config();
        let metrics = Arc::new(ResourceMetrics::new());
        let manager = MemoryManager::new(config, metrics.clone()).unwrap();

        manager.track_allocation(850);
        assert!(!manager.should_trigger_gc());

        manager.track_allocation(100);
        assert!(manager.should_trigger_gc());

        manager.trigger_gc().await;
        assert_eq!(metrics.gc_triggers.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_cleanup_tracking() {
        let config = test_config();
        let metrics = Arc::new(ResourceMetrics::new());
        let manager = MemoryManager::new(config, metrics.clone()).unwrap();

        manager.trigger_cleanup().await;
        assert_eq!(metrics.cleanup_operations.load(Ordering::Relaxed), 1);
        assert!(manager.time_since_last_cleanup().is_some());
    }

    #[test]
    fn test_usage_percentage() {
        let config = test_config();
        let metrics = Arc::new(ResourceMetrics::new());
        let manager = MemoryManager::new(config, metrics).unwrap();

        manager.track_allocation(500);
        assert_eq!(manager.usage_percentage(), 50.0);

        manager.track_allocation(250);
        assert_eq!(manager.usage_percentage(), 75.0);
    }
}
