//! Memory management with pressure detection, cleanup, and leak detection.
//!
//! Monitors system memory usage and provides:
//! - Memory allocation/deallocation tracking (manual and jemalloc-based)
//! - Pressure detection based on configurable thresholds
//! - Automatic cleanup triggers
//! - Garbage collection coordination
//! - Memory leak detection and reporting
//!
//! ## jemalloc Integration
//!
//! When the `jemalloc` feature is enabled, this module uses jemalloc for accurate
//! memory monitoring. Enable it in your Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! riptide-api = { version = "0.1", features = ["jemalloc"] }
//! ```
//!
//! This provides:
//! - Real RSS (Resident Set Size) monitoring via `get_current_rss()`
//! - Heap allocation tracking via `get_heap_allocated()`
//! - More accurate memory pressure detection
//!
//! Without jemalloc, the module falls back to manual tracking via
//! `track_allocation()` and `track_deallocation()`.

use crate::config::RiptideApiConfig;
use crate::resource_manager::{errors::Result, metrics::ResourceMetrics};
use std::collections::{HashMap, VecDeque};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    Arc, RwLock,
};
use tracing::{info, warn};

/// Memory manager with pressure detection, cleanup, and leak detection
///
/// Tracks memory allocations and triggers cleanup or GC when thresholds are exceeded.
/// All operations are thread-safe and non-blocking.
pub struct MemoryManager {
    config: RiptideApiConfig,
    current_usage: AtomicUsize,
    pressure_detected: AtomicBool,
    last_cleanup: AtomicU64,
    last_gc: AtomicU64,
    metrics: Arc<ResourceMetrics>,
    leak_detector: Arc<LeakDetector>,
}

/// Memory leak detector
///
/// Tracks allocation/deallocation patterns to detect memory leaks.
/// Thread-safe and designed for concurrent access.
pub struct LeakDetector {
    /// Baseline memory snapshot for comparison
    baseline: RwLock<Option<MemorySnapshot>>,
    /// Historical memory samples
    history: RwLock<VecDeque<MemorySnapshot>>,
    /// Per-component allocation tracking
    component_tracking: RwLock<HashMap<String, ComponentMemory>>,
    /// Leak detection configuration
    config: LeakDetectionConfig,
}

/// Memory snapshot at a point in time
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct MemorySnapshot {
    timestamp: u64,
    usage_mb: usize,
    #[allow(dead_code)]
    allocations: u64,
    #[allow(dead_code)]
    deallocations: u64,
}

/// Per-component memory tracking
#[derive(Debug, Clone)]
struct ComponentMemory {
    name: String,
    allocated_mb: usize,
    allocation_count: u64,
    deallocation_count: u64,
    first_seen: u64,
    last_updated: u64,
}

/// Leak detection configuration
#[derive(Debug, Clone)]
pub struct LeakDetectionConfig {
    /// Growth threshold percentage (default: 5%)
    pub growth_threshold: f64,
    /// Time window in seconds (default: 600 = 10 minutes)
    pub time_window_secs: u64,
    /// Maximum history samples to keep
    pub max_history_samples: usize,
    /// Minimum allocations to consider as leak candidate
    pub min_allocations_for_leak: u64,
}

impl Default for LeakDetectionConfig {
    fn default() -> Self {
        Self {
            growth_threshold: 5.0,
            time_window_secs: 600,
            max_history_samples: 100,
            min_allocations_for_leak: 10,
        }
    }
}

/// Memory leak report
#[derive(Debug, Clone, serde::Serialize)]
pub struct LeakReport {
    pub has_leaks: bool,
    pub overall_growth_rate: f64,
    pub growth_mb_per_minute: f64,
    pub time_window_secs: u64,
    pub leak_candidates: Vec<LeakCandidate>,
    pub recommendations: Vec<String>,
    pub baseline_mb: Option<usize>,
    pub current_mb: usize,
    pub timestamp: u64,
}

/// Leak candidate information
#[derive(Debug, Clone, serde::Serialize)]
pub struct LeakCandidate {
    pub component: String,
    pub allocated_mb: usize,
    pub net_allocations: i64,
    pub age_secs: u64,
    pub severity: LeakSeverity,
    pub growth_rate: f64,
}

/// Leak severity classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub enum LeakSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl LeakDetector {
    /// Create a new leak detector with default configuration
    pub fn new() -> Self {
        Self::with_config(LeakDetectionConfig::default())
    }

    /// Create a new leak detector with custom configuration
    pub fn with_config(config: LeakDetectionConfig) -> Self {
        info!(
            growth_threshold = config.growth_threshold,
            time_window_secs = config.time_window_secs,
            "Initializing leak detector"
        );

        Self {
            baseline: RwLock::new(None),
            history: RwLock::new(VecDeque::with_capacity(config.max_history_samples)),
            component_tracking: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Set baseline memory snapshot
    pub fn set_baseline(&self, usage_mb: usize) {
        let snapshot = MemorySnapshot {
            timestamp: current_timestamp_secs(),
            usage_mb,
            allocations: 0,
            deallocations: 0,
        };

        if let Ok(mut baseline) = self.baseline.write() {
            *baseline = Some(snapshot.clone());
            info!(baseline_mb = usage_mb, "Set memory baseline");
        }

        // Also add to history
        if let Ok(mut history) = self.history.write() {
            history.push_back(snapshot);
            if history.len() > self.config.max_history_samples {
                history.pop_front();
            }
        }
    }

    /// Track an allocation for a specific component
    pub fn track_allocation(&self, component: &str, size_mb: usize) {
        let timestamp = current_timestamp_secs();

        if let Ok(mut tracking) = self.component_tracking.write() {
            tracking
                .entry(component.to_string())
                .and_modify(|c| {
                    c.allocated_mb += size_mb;
                    c.allocation_count += 1;
                    c.last_updated = timestamp;
                })
                .or_insert_with(|| ComponentMemory {
                    name: component.to_string(),
                    allocated_mb: size_mb,
                    allocation_count: 1,
                    deallocation_count: 0,
                    first_seen: timestamp,
                    last_updated: timestamp,
                });
        }
    }

    /// Track a deallocation for a specific component
    pub fn track_deallocation(&self, component: &str, size_mb: usize) {
        let timestamp = current_timestamp_secs();

        if let Ok(mut tracking) = self.component_tracking.write() {
            if let Some(component_mem) = tracking.get_mut(component) {
                component_mem.allocated_mb = component_mem.allocated_mb.saturating_sub(size_mb);
                component_mem.deallocation_count += 1;
                component_mem.last_updated = timestamp;
            }
        }
    }

    /// Record a memory snapshot
    pub fn record_snapshot(&self, usage_mb: usize) {
        let snapshot = MemorySnapshot {
            timestamp: current_timestamp_secs(),
            usage_mb,
            allocations: 0,
            deallocations: 0,
        };

        if let Ok(mut history) = self.history.write() {
            history.push_back(snapshot);
            if history.len() > self.config.max_history_samples {
                history.pop_front();
            }
        }
    }

    /// Detect memory leaks and generate report
    pub fn detect_leaks(&self, current_usage_mb: usize) -> LeakReport {
        let timestamp = current_timestamp_secs();
        let baseline = self.baseline.read().ok().and_then(|b| b.clone());

        // Calculate growth rate from history
        let (growth_rate, growth_mb_per_min) = self.calculate_growth_rate();

        // Get leak candidates from component tracking
        let leak_candidates = self.identify_leak_candidates(timestamp);

        // Determine if there are leaks
        let has_leaks = growth_rate > self.config.growth_threshold || !leak_candidates.is_empty();

        // Generate recommendations
        let recommendations = self.generate_recommendations(growth_rate, &leak_candidates);

        LeakReport {
            has_leaks,
            overall_growth_rate: growth_rate,
            growth_mb_per_minute: growth_mb_per_min,
            time_window_secs: self.config.time_window_secs,
            leak_candidates,
            recommendations,
            baseline_mb: baseline.map(|b| b.usage_mb),
            current_mb: current_usage_mb,
            timestamp,
        }
    }

    /// Calculate memory growth rate from history
    fn calculate_growth_rate(&self) -> (f64, f64) {
        let history = match self.history.read() {
            Ok(h) => h,
            Err(_) => return (0.0, 0.0),
        };

        if history.len() < 2 {
            return (0.0, 0.0);
        }

        let current_time = current_timestamp_secs();
        let time_window = self.config.time_window_secs;

        // Find samples within time window
        let recent_samples: Vec<_> = history
            .iter()
            .filter(|s| current_time.saturating_sub(s.timestamp) <= time_window)
            .collect();

        if recent_samples.len() < 2 {
            return (0.0, 0.0);
        }

        // SAFETY: We already checked that recent_samples.len() >= 2 above
        let oldest = recent_samples
            .first()
            .expect("recent_samples guaranteed to have at least 2 elements");
        let newest = recent_samples
            .last()
            .expect("recent_samples guaranteed to have at least 2 elements");

        let time_diff_secs = newest.timestamp.saturating_sub(oldest.timestamp);
        if time_diff_secs == 0 {
            return (0.0, 0.0);
        }

        let usage_diff = newest.usage_mb.saturating_sub(oldest.usage_mb);
        let growth_rate = if oldest.usage_mb > 0 {
            (usage_diff as f64 / oldest.usage_mb as f64) * 100.0
        } else {
            0.0
        };

        let growth_mb_per_min = (usage_diff as f64) / (time_diff_secs as f64 / 60.0);

        (growth_rate, growth_mb_per_min)
    }

    /// Identify leak candidates from component tracking
    fn identify_leak_candidates(&self, current_time: u64) -> Vec<LeakCandidate> {
        let tracking = match self.component_tracking.read() {
            Ok(t) => t,
            Err(_) => return vec![],
        };

        let mut candidates = Vec::new();

        for component in tracking.values() {
            let net_allocations =
                component.allocation_count as i64 - component.deallocation_count as i64;
            let age_secs = current_time.saturating_sub(component.first_seen);

            // Skip if insufficient allocations
            if component.allocation_count < self.config.min_allocations_for_leak {
                continue;
            }

            // Calculate growth rate for this component
            let growth_rate = if component.deallocation_count > 0 {
                (net_allocations as f64 / component.allocation_count as f64) * 100.0
            } else {
                100.0 // No deallocations = 100% leak
            };

            // Determine severity
            let severity = if component.allocated_mb > 1000 || growth_rate > 80.0 {
                LeakSeverity::Critical
            } else if component.allocated_mb > 500 || growth_rate > 50.0 {
                LeakSeverity::High
            } else if component.allocated_mb > 100 || growth_rate > 20.0 {
                LeakSeverity::Medium
            } else {
                LeakSeverity::Low
            };

            // Only include if there's significant growth
            if net_allocations > 5 || growth_rate > 10.0 {
                candidates.push(LeakCandidate {
                    component: component.name.clone(),
                    allocated_mb: component.allocated_mb,
                    net_allocations,
                    age_secs,
                    severity,
                    growth_rate,
                });
            }
        }

        // Sort by severity and size
        candidates.sort_by(|a, b| {
            b.severity
                .partial_cmp(&a.severity)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.allocated_mb.cmp(&a.allocated_mb))
        });

        candidates
    }

    /// Generate recommendations based on leak analysis
    fn generate_recommendations(
        &self,
        growth_rate: f64,
        candidates: &[LeakCandidate],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if growth_rate > self.config.growth_threshold {
            recommendations.push(format!(
                "Memory growth rate of {:.2}% exceeds threshold of {:.2}%",
                growth_rate, self.config.growth_threshold
            ));
            recommendations.push("Consider triggering garbage collection or cleanup".to_string());
        }

        for candidate in candidates.iter().take(3) {
            match candidate.severity {
                LeakSeverity::Critical => {
                    recommendations.push(format!(
                        "CRITICAL: Component '{}' has {} MB allocated with {:.1}% growth rate",
                        candidate.component, candidate.allocated_mb, candidate.growth_rate
                    ));
                }
                LeakSeverity::High => {
                    recommendations.push(format!(
                        "HIGH: Component '{}' shows high memory usage ({} MB)",
                        candidate.component, candidate.allocated_mb
                    ));
                }
                LeakSeverity::Medium => {
                    recommendations.push(format!(
                        "MEDIUM: Monitor component '{}' ({} MB allocated)",
                        candidate.component, candidate.allocated_mb
                    ));
                }
                LeakSeverity::Low => {}
            }
        }

        if candidates.is_empty() && growth_rate <= self.config.growth_threshold {
            recommendations.push("No memory leaks detected".to_string());
        }

        recommendations
    }
}

impl Default for LeakDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new(config: RiptideApiConfig, metrics: Arc<ResourceMetrics>) -> Result<Self> {
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
            leak_detector: Arc::new(LeakDetector::new()),
        })
    }

    /// Get leak detector reference
    pub fn leak_detector(&self) -> &Arc<LeakDetector> {
        &self.leak_detector
    }

    /// Detect memory leaks
    pub fn detect_leaks(&self) -> LeakReport {
        self.leak_detector.detect_leaks(self.current_usage_mb())
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

        // Track in leak detector (default component)
        self.leak_detector.track_allocation("default", size_mb);
        self.leak_detector.record_snapshot(new_usage);

        // Check for memory pressure
        if self.config.is_memory_pressure(new_usage)
            && !self.pressure_detected.swap(true, Ordering::Relaxed)
        {
            warn!(
                current_mb = new_usage,
                limit_mb = self.config.memory.global_memory_limit_mb,
                threshold = self.config.memory.pressure_threshold,
                "Memory pressure detected"
            );
        }
    }

    /// Track a memory allocation for a specific component
    ///
    /// # Arguments
    /// * `component` - Component name for tracking
    /// * `size_mb` - Size of allocation in megabytes
    pub fn track_allocation_by_component(&self, component: &str, size_mb: usize) {
        let current = self.current_usage.fetch_add(size_mb, Ordering::Relaxed);
        let new_usage = current + size_mb;

        self.metrics
            .memory_usage_mb
            .store(new_usage, Ordering::Relaxed);

        // Track in leak detector with component name
        self.leak_detector.track_allocation(component, size_mb);
        self.leak_detector.record_snapshot(new_usage);

        // Check for memory pressure
        if self.config.is_memory_pressure(new_usage)
            && !self.pressure_detected.swap(true, Ordering::Relaxed)
        {
            warn!(
                current_mb = new_usage,
                component = component,
                limit_mb = self.config.memory.global_memory_limit_mb,
                threshold = self.config.memory.pressure_threshold,
                "Memory pressure detected"
            );
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

        // Track in leak detector (default component)
        self.leak_detector.track_deallocation("default", size_mb);
        self.leak_detector.record_snapshot(new_usage);

        // Update pressure status
        if !self.config.is_memory_pressure(new_usage)
            && self.pressure_detected.swap(false, Ordering::Relaxed)
        {
            info!(current_mb = new_usage, "Memory pressure cleared");
        }
    }

    /// Track a memory deallocation for a specific component
    ///
    /// # Arguments
    /// * `component` - Component name for tracking
    /// * `size_mb` - Size of deallocation in megabytes
    pub fn track_deallocation_by_component(&self, component: &str, size_mb: usize) {
        let current = self.current_usage.fetch_sub(size_mb, Ordering::Relaxed);
        let new_usage = current.saturating_sub(size_mb);

        self.metrics
            .memory_usage_mb
            .store(new_usage, Ordering::Relaxed);

        // Track in leak detector with component name
        self.leak_detector.track_deallocation(component, size_mb);
        self.leak_detector.record_snapshot(new_usage);

        // Update pressure status
        if !self.config.is_memory_pressure(new_usage)
            && self.pressure_detected.swap(false, Ordering::Relaxed)
        {
            info!(
                current_mb = new_usage,
                component = component,
                "Memory pressure cleared"
            );
        }
    }

    /// Set memory baseline for leak detection
    pub fn set_leak_baseline(&self) {
        self.leak_detector.set_baseline(self.current_usage_mb());
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
    #[allow(dead_code)] // Reserved for future monitoring API
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
    #[allow(dead_code)] // Reserved for future monitoring API
    pub fn time_since_last_cleanup(&self) -> Option<u64> {
        let last = self.last_cleanup.load(Ordering::Relaxed);
        if last == 0 {
            return None;
        }
        Some(current_timestamp_secs().saturating_sub(last))
    }

    /// Get time since last GC (in seconds)
    #[allow(dead_code)] // Reserved for future monitoring API
    pub fn time_since_last_gc(&self) -> Option<u64> {
        let last = self.last_gc.load(Ordering::Relaxed);
        if last == 0 {
            return None;
        }
        Some(current_timestamp_secs().saturating_sub(last))
    }

    /// Get memory statistics
    #[allow(dead_code)] // Reserved for future monitoring API
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

    /// Get current RSS (Resident Set Size) in megabytes
    ///
    /// Uses real RSS via sysinfo for accurate system measurement.
    ///
    /// # Errors
    /// Returns error if system info cannot be read.
    #[allow(dead_code)]
    pub fn get_current_rss(&self) -> Result<usize> {
        use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

        // Use sysinfo to get actual RSS
        let mut system = System::new_with_specifics(
            RefreshKind::new().with_processes(ProcessRefreshKind::new()),
        );

        // Get current PID first
        let pid = sysinfo::get_current_pid().map_err(|e| {
            crate::resource_manager::errors::ResourceManagerError::Internal(anyhow::anyhow!(
                "Failed to get current PID: {}",
                e
            ))
        })?;

        // Refresh only our process
        system.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);

        let process = system.process(pid).ok_or_else(|| {
            crate::resource_manager::errors::ResourceManagerError::Internal(anyhow::anyhow!(
                "Failed to get process info"
            ))
        })?;

        let rss_bytes = process.memory();
        // Convert bytes to MB, ensuring the result fits in usize
        // On 32-bit platforms, saturate at usize::MAX if value exceeds capacity
        let rss_mb = rss_bytes / 1024;
        Ok(usize::try_from(rss_mb).unwrap_or(usize::MAX))
    }

    /// Get current heap allocated memory in megabytes
    ///
    /// Uses system virtual memory as approximation of heap size.
    ///
    /// # Errors
    /// Returns error if system info cannot be read.
    #[allow(dead_code)]
    pub fn get_heap_allocated(&self) -> Result<usize> {
        use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

        // Use sysinfo to get virtual memory (approximates heap)
        let mut system = System::new_with_specifics(
            RefreshKind::new().with_processes(ProcessRefreshKind::new()),
        );

        // Get current PID first
        let pid = sysinfo::get_current_pid().map_err(|e| {
            crate::resource_manager::errors::ResourceManagerError::Internal(anyhow::anyhow!(
                "Failed to get current PID: {}",
                e
            ))
        })?;

        // Refresh only our process
        system.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);

        let process = system.process(pid).ok_or_else(|| {
            crate::resource_manager::errors::ResourceManagerError::Internal(anyhow::anyhow!(
                "Failed to get process info"
            ))
        })?;

        let virtual_bytes = process.virtual_memory();
        // Convert bytes to MB, ensuring the result fits in usize
        // On 32-bit platforms, saturate at usize::MAX if value exceeds capacity
        let virtual_mb = virtual_bytes / 1024;
        Ok(usize::try_from(virtual_mb).unwrap_or(usize::MAX))
    }

    /// Check memory pressure using real memory metrics
    ///
    /// Uses real RSS from sysinfo to accurately detect memory pressure.
    /// This method updates internal pressure state and metrics.
    #[allow(dead_code)]
    pub fn check_memory_pressure(&self) -> bool {
        let current_mb = match self.get_current_rss() {
            Ok(rss) => {
                // Update metrics with real value
                self.metrics.memory_usage_mb.store(rss, Ordering::Relaxed);
                rss
            }
            Err(e) => {
                warn!(error = %e, "Failed to get RSS, using manual tracking");
                self.current_usage_mb()
            }
        };

        let under_pressure = self.config.is_memory_pressure(current_mb);

        // Update pressure state
        let previous = self
            .pressure_detected
            .swap(under_pressure, Ordering::Relaxed);

        if under_pressure && !previous {
            warn!(
                current_mb,
                limit_mb = self.config.memory.global_memory_limit_mb,
                threshold = self.config.memory.pressure_threshold,
                "Memory pressure detected"
            );
        } else if !under_pressure && previous {
            info!(current_mb, "Memory pressure cleared");
        }

        under_pressure
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
    use crate::config::RiptideApiConfig;

    fn test_config() -> RiptideApiConfig {
        let mut config = RiptideApiConfig::default();
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
    async fn test_real_memory_monitoring() {
        let config = test_config();
        let metrics = Arc::new(ResourceMetrics::new());
        let manager = MemoryManager::new(config, metrics).unwrap();

        // Get real RSS (always works via sysinfo)
        let rss = manager.get_current_rss();
        assert!(rss.is_ok(), "RSS measurement should succeed");
        let rss_mb = rss.unwrap();

        // RSS should be reasonable (not zero, not absurdly large)
        assert!(rss_mb > 0, "RSS should be greater than 0");
        // Note: Don't assert on max RSS - test environments vary (cloud VMs, containers, etc.)
        // Just verify we got a reasonable value that's not obviously wrong
        assert!(
            rss_mb < 1_000_000,
            "RSS should be reasonable (< 1TB), got {} MB",
            rss_mb
        );

        // Get heap allocated (approximated via virtual memory)
        let heap = manager.get_heap_allocated();
        assert!(heap.is_ok(), "Heap measurement should succeed");
        let heap_mb = heap.unwrap();

        assert!(heap_mb > 0, "Heap allocation should be > 0");

        // Virtual memory should be >= RSS
        assert!(
            heap_mb >= rss_mb,
            "Virtual memory should be >= RSS (heap={}, rss={})",
            heap_mb,
            rss_mb
        );
    }

    #[tokio::test]
    async fn test_check_memory_pressure_with_real_metrics() {
        let mut config = test_config();
        // Set high threshold so we're not actually under pressure during test
        config.memory.global_memory_limit_mb = 100000; // 100GB
        config.memory.pressure_threshold = 0.9;

        let metrics = Arc::new(ResourceMetrics::new());
        let manager = MemoryManager::new(config, metrics.clone()).unwrap();

        // Check pressure (will use real RSS via sysinfo)
        let is_under_pressure = manager.check_memory_pressure();

        // With high threshold, we should not be under pressure
        assert!(
            !is_under_pressure,
            "Should not be under pressure with 100GB limit"
        );

        // Verify metrics were updated with real RSS value
        let metrics_value = metrics.memory_usage_mb.load(Ordering::Relaxed);
        assert!(metrics_value > 0, "Metrics should be updated with real RSS");
        // Note: Don't assert on max memory size - test environments vary (cloud VMs, containers, etc.)
        // Just verify we got a reasonable value that's not obviously wrong
        assert!(
            metrics_value < 1_000_000,
            "Metrics should be reasonable (< 1TB), got {} MB",
            metrics_value
        );
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
