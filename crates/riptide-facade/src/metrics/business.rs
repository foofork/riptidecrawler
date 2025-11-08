//! Business-level metrics for facade operations
//!
//! This module provides domain-level metrics distinct from transport-level metrics.
//! It focuses on business operations like extractions, sessions, and pipeline stages.

use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Business metrics collector for facade-level operations
///
/// This struct provides domain-level metrics that measure business outcomes
/// rather than infrastructure concerns. It's designed to be injected into
/// facades for tracking business operations.
///
/// # Design Principles
///
/// - **Domain-Focused**: Metrics represent business concepts (extractions, sessions, profiles)
/// - **Technology-Agnostic**: No dependency on specific metrics backend (Prometheus, StatsD, etc.)
/// - **Facade-Level**: Scoped to application layer concerns, not infrastructure
/// - **Optional**: Facades can operate without metrics (graceful degradation)
///
/// # Example
///
/// ```rust,ignore
/// use riptide_facade::metrics::BusinessMetrics;
/// use std::time::Duration;
///
/// let metrics = BusinessMetrics::new();
///
/// // Record extraction
/// metrics.record_extraction_completed(Duration::from_millis(150), true);
///
/// // Record session
/// metrics.record_session_created();
/// metrics.record_session_active(5);
///
/// // Record cache hit
/// metrics.record_cache_hit();
/// ```
#[derive(Clone)]
pub struct BusinessMetrics {
    inner: Arc<BusinessMetricsInner>,
}

struct BusinessMetricsInner {
    // Domain-level counters
    profiles_created: Mutex<u64>,
    sessions_created: Mutex<u64>,
    extractions_completed: Mutex<u64>,
    extractions_failed: Mutex<u64>,

    // Timing metrics
    extraction_durations: Mutex<Vec<Duration>>,

    // Active resource tracking
    active_sessions: Mutex<i64>,

    // Cache metrics
    cache_hits: Mutex<u64>,
    cache_misses: Mutex<u64>,

    // Business SLOs
    extraction_success_count: Mutex<u64>,
    extraction_failure_count: Mutex<u64>,

    // Pipeline metrics
    pipeline_stages_completed: Mutex<u64>,
    pipeline_stages_failed: Mutex<u64>,

    // Browser metrics
    browser_actions: Mutex<u64>,
    screenshots_taken: Mutex<u64>,
}

impl Default for BusinessMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl BusinessMetrics {
    /// Create a new business metrics collector
    ///
    /// This creates an in-memory metrics collector suitable for testing
    /// and development. For production use, consider wrapping this with
    /// a metrics adapter that exports to Prometheus, StatsD, etc.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(BusinessMetricsInner {
                profiles_created: Mutex::new(0),
                sessions_created: Mutex::new(0),
                extractions_completed: Mutex::new(0),
                extractions_failed: Mutex::new(0),
                extraction_durations: Mutex::new(Vec::with_capacity(1000)),
                active_sessions: Mutex::new(0),
                cache_hits: Mutex::new(0),
                cache_misses: Mutex::new(0),
                extraction_success_count: Mutex::new(0),
                extraction_failure_count: Mutex::new(0),
                pipeline_stages_completed: Mutex::new(0),
                pipeline_stages_failed: Mutex::new(0),
                browser_actions: Mutex::new(0),
                screenshots_taken: Mutex::new(0),
            }),
        }
    }

    /// Record completion of an extraction operation
    ///
    /// # Arguments
    ///
    /// * `duration` - Time taken for the extraction
    /// * `success` - Whether the extraction succeeded
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::time::{Duration, Instant};
    ///
    /// let start = Instant::now();
    /// // ... perform extraction ...
    /// let duration = start.elapsed();
    /// metrics.record_extraction_completed(duration, true);
    /// ```
    pub fn record_extraction_completed(&self, duration: Duration, success: bool) {
        // Increment extraction counter
        if success {
            *self.inner.extractions_completed.lock().unwrap() += 1;
            *self.inner.extraction_success_count.lock().unwrap() += 1;
        } else {
            *self.inner.extractions_failed.lock().unwrap() += 1;
            *self.inner.extraction_failure_count.lock().unwrap() += 1;
        }

        // Record duration for percentile calculations
        let mut durations = self.inner.extraction_durations.lock().unwrap();
        durations.push(duration);

        // Keep only recent 1000 samples to avoid unbounded growth
        if durations.len() > 1000 {
            durations.drain(0..durations.len() - 1000);
        }
    }

    /// Record creation of a user profile
    ///
    /// Tracks when new user profiles are created in the system.
    pub fn record_profile_created(&self) {
        *self.inner.profiles_created.lock().unwrap() += 1;
    }

    /// Record creation of a new session
    ///
    /// Tracks when new browser/extraction sessions are initiated.
    pub fn record_session_created(&self) {
        *self.inner.sessions_created.lock().unwrap() += 1;
        *self.inner.active_sessions.lock().unwrap() += 1;
    }

    /// Record session termination
    ///
    /// Tracks when sessions are closed or terminated.
    pub fn record_session_closed(&self) {
        let mut active = self.inner.active_sessions.lock().unwrap();
        *active = (*active - 1).max(0);
    }

    /// Update active session count
    ///
    /// # Arguments
    ///
    /// * `count` - Current number of active sessions
    pub fn record_session_active(&self, count: i64) {
        *self.inner.active_sessions.lock().unwrap() = count;
    }

    /// Record a cache hit
    ///
    /// Tracks successful cache retrievals for business operations.
    pub fn record_cache_hit(&self) {
        *self.inner.cache_hits.lock().unwrap() += 1;
    }

    /// Record a cache miss
    ///
    /// Tracks cache misses requiring fresh data retrieval.
    pub fn record_cache_miss(&self) {
        *self.inner.cache_misses.lock().unwrap() += 1;
    }

    /// Record completion of a pipeline stage
    ///
    /// # Arguments
    ///
    /// * `_stage_name` - Name of the pipeline stage (for future use)
    /// * `success` - Whether the stage completed successfully
    pub fn record_pipeline_stage(&self, _stage_name: &str, success: bool) {
        if success {
            *self.inner.pipeline_stages_completed.lock().unwrap() += 1;
        } else {
            *self.inner.pipeline_stages_failed.lock().unwrap() += 1;
        }
    }

    /// Record a browser action
    ///
    /// Tracks browser operations like navigation, clicking, form filling, etc.
    pub fn record_browser_action(&self) {
        *self.inner.browser_actions.lock().unwrap() += 1;
    }

    /// Record a screenshot capture
    ///
    /// Tracks when screenshots are taken for monitoring or debugging.
    pub fn record_screenshot_taken(&self) {
        *self.inner.screenshots_taken.lock().unwrap() += 1;
    }

    /// Get current metrics snapshot
    ///
    /// Returns a snapshot of all current metric values for monitoring
    /// or reporting purposes.
    #[must_use]
    pub fn snapshot(&self) -> MetricsSnapshot {
        let durations = self.inner.extraction_durations.lock().unwrap();
        let p50 = Self::calculate_percentile(&durations, 50.0);
        let p95 = Self::calculate_percentile(&durations, 95.0);
        let p99 = Self::calculate_percentile(&durations, 99.0);
        let avg = Self::calculate_average(&durations);

        MetricsSnapshot {
            profiles_created: *self.inner.profiles_created.lock().unwrap(),
            sessions_created: *self.inner.sessions_created.lock().unwrap(),
            active_sessions: *self.inner.active_sessions.lock().unwrap(),
            extractions_completed: *self.inner.extractions_completed.lock().unwrap(),
            extractions_failed: *self.inner.extractions_failed.lock().unwrap(),
            extraction_success_count: *self.inner.extraction_success_count.lock().unwrap(),
            extraction_failure_count: *self.inner.extraction_failure_count.lock().unwrap(),
            cache_hits: *self.inner.cache_hits.lock().unwrap(),
            cache_misses: *self.inner.cache_misses.lock().unwrap(),
            pipeline_stages_completed: *self.inner.pipeline_stages_completed.lock().unwrap(),
            pipeline_stages_failed: *self.inner.pipeline_stages_failed.lock().unwrap(),
            browser_actions: *self.inner.browser_actions.lock().unwrap(),
            screenshots_taken: *self.inner.screenshots_taken.lock().unwrap(),
            extraction_p50_ms: p50,
            extraction_p95_ms: p95,
            extraction_p99_ms: p99,
            extraction_avg_ms: avg,
        }
    }

    /// Calculate percentile from duration samples
    fn calculate_percentile(durations: &[Duration], percentile: f64) -> f64 {
        if durations.is_empty() {
            return 0.0;
        }

        let mut sorted: Vec<f64> = durations.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let index = ((percentile / 100.0) * (sorted.len() - 1) as f64) as usize;
        sorted[index.min(sorted.len() - 1)]
    }

    /// Calculate average duration
    fn calculate_average(durations: &[Duration]) -> f64 {
        if durations.is_empty() {
            return 0.0;
        }

        let sum: f64 = durations.iter().map(|d| d.as_secs_f64() * 1000.0).sum();
        sum / durations.len() as f64
    }

    /// Reset all metrics (useful for testing)
    #[cfg(test)]
    pub fn reset(&self) {
        *self.inner.profiles_created.lock().unwrap() = 0;
        *self.inner.sessions_created.lock().unwrap() = 0;
        *self.inner.extractions_completed.lock().unwrap() = 0;
        *self.inner.extractions_failed.lock().unwrap() = 0;
        self.inner.extraction_durations.lock().unwrap().clear();
        *self.inner.active_sessions.lock().unwrap() = 0;
        *self.inner.cache_hits.lock().unwrap() = 0;
        *self.inner.cache_misses.lock().unwrap() = 0;
        *self.inner.extraction_success_count.lock().unwrap() = 0;
        *self.inner.extraction_failure_count.lock().unwrap() = 0;
        *self.inner.pipeline_stages_completed.lock().unwrap() = 0;
        *self.inner.pipeline_stages_failed.lock().unwrap() = 0;
        *self.inner.browser_actions.lock().unwrap() = 0;
        *self.inner.screenshots_taken.lock().unwrap() = 0;
    }
}

/// Snapshot of business metrics at a point in time
#[derive(Debug, Clone, PartialEq)]
pub struct MetricsSnapshot {
    /// Total profiles created
    pub profiles_created: u64,
    /// Total sessions created
    pub sessions_created: u64,
    /// Currently active sessions
    pub active_sessions: i64,
    /// Successful extractions
    pub extractions_completed: u64,
    /// Failed extractions
    pub extractions_failed: u64,
    /// Extraction success count (for SLO)
    pub extraction_success_count: u64,
    /// Extraction failure count (for SLO)
    pub extraction_failure_count: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Pipeline stages completed
    pub pipeline_stages_completed: u64,
    /// Pipeline stages failed
    pub pipeline_stages_failed: u64,
    /// Browser actions performed
    pub browser_actions: u64,
    /// Screenshots taken
    pub screenshots_taken: u64,
    /// 50th percentile extraction time (ms)
    pub extraction_p50_ms: f64,
    /// 95th percentile extraction time (ms)
    pub extraction_p95_ms: f64,
    /// 99th percentile extraction time (ms)
    pub extraction_p99_ms: f64,
    /// Average extraction time (ms)
    pub extraction_avg_ms: f64,
}

impl MetricsSnapshot {
    /// Calculate cache hit ratio
    #[must_use]
    pub fn cache_hit_ratio(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            return 0.0;
        }
        (self.cache_hits as f64 / total as f64) * 100.0
    }

    /// Calculate extraction success rate
    #[must_use]
    pub fn extraction_success_rate(&self) -> f64 {
        let total = self.extraction_success_count + self.extraction_failure_count;
        if total == 0 {
            return 0.0;
        }
        (self.extraction_success_count as f64 / total as f64) * 100.0
    }

    /// Calculate pipeline success rate
    #[must_use]
    pub fn pipeline_success_rate(&self) -> f64 {
        let total = self.pipeline_stages_completed + self.pipeline_stages_failed;
        if total == 0 {
            return 0.0;
        }
        (self.pipeline_stages_completed as f64 / total as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_metrics() {
        let metrics = BusinessMetrics::new();

        // Record successful extraction
        metrics.record_extraction_completed(Duration::from_millis(100), true);
        metrics.record_extraction_completed(Duration::from_millis(150), true);
        metrics.record_extraction_completed(Duration::from_millis(200), false);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.extractions_completed, 2);
        assert_eq!(snapshot.extractions_failed, 1);
        assert_eq!(snapshot.extraction_success_count, 2);
        assert_eq!(snapshot.extraction_failure_count, 1);
        assert!(snapshot.extraction_avg_ms > 0.0);
    }

    #[test]
    fn test_session_metrics() {
        let metrics = BusinessMetrics::new();

        metrics.record_session_created();
        metrics.record_session_created();
        metrics.record_session_created();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.sessions_created, 3);
        assert_eq!(snapshot.active_sessions, 3);

        metrics.record_session_closed();
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.active_sessions, 2);
    }

    #[test]
    fn test_cache_metrics() {
        let metrics = BusinessMetrics::new();

        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.cache_hits, 3);
        assert_eq!(snapshot.cache_misses, 1);
        assert_eq!(snapshot.cache_hit_ratio(), 75.0);
    }

    #[test]
    fn test_profile_metrics() {
        let metrics = BusinessMetrics::new();

        metrics.record_profile_created();
        metrics.record_profile_created();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.profiles_created, 2);
    }

    #[test]
    fn test_pipeline_metrics() {
        let metrics = BusinessMetrics::new();

        metrics.record_pipeline_stage("fetch", true);
        metrics.record_pipeline_stage("extract", true);
        metrics.record_pipeline_stage("transform", false);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.pipeline_stages_completed, 2);
        assert_eq!(snapshot.pipeline_stages_failed, 1);
        assert!((snapshot.pipeline_success_rate() - 66.67).abs() < 0.1);
    }

    #[test]
    fn test_browser_metrics() {
        let metrics = BusinessMetrics::new();

        metrics.record_browser_action();
        metrics.record_browser_action();
        metrics.record_screenshot_taken();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.browser_actions, 2);
        assert_eq!(snapshot.screenshots_taken, 1);
    }

    #[test]
    fn test_percentile_calculations() {
        let metrics = BusinessMetrics::new();

        // Record extractions with known durations
        metrics.record_extraction_completed(Duration::from_millis(100), true);
        metrics.record_extraction_completed(Duration::from_millis(200), true);
        metrics.record_extraction_completed(Duration::from_millis(300), true);
        metrics.record_extraction_completed(Duration::from_millis(400), true);
        metrics.record_extraction_completed(Duration::from_millis(500), true);

        let snapshot = metrics.snapshot();
        assert!(snapshot.extraction_p50_ms >= 200.0 && snapshot.extraction_p50_ms <= 300.0);
        assert!(snapshot.extraction_p95_ms >= 400.0);
        assert_eq!(snapshot.extraction_avg_ms, 300.0);
    }

    #[test]
    fn test_success_rate_calculation() {
        let metrics = BusinessMetrics::new();

        metrics.record_extraction_completed(Duration::from_millis(100), true);
        metrics.record_extraction_completed(Duration::from_millis(100), true);
        metrics.record_extraction_completed(Duration::from_millis(100), true);
        metrics.record_extraction_completed(Duration::from_millis(100), false);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.extraction_success_rate(), 75.0);
    }

    #[test]
    fn test_metrics_snapshot_clone() {
        let metrics = BusinessMetrics::new();
        metrics.record_extraction_completed(Duration::from_millis(100), true);

        let snapshot1 = metrics.snapshot();
        let snapshot2 = snapshot1.clone();

        assert_eq!(snapshot1, snapshot2);
    }

    #[test]
    fn test_duration_buffer_limit() {
        let metrics = BusinessMetrics::new();

        // Record more than 1000 extractions
        for i in 0..1500 {
            metrics.record_extraction_completed(Duration::from_millis(i % 500), true);
        }

        // Buffer should be limited to 1000
        let durations = metrics.inner.extraction_durations.lock().unwrap();
        assert!(durations.len() <= 1000);
    }

    #[test]
    fn test_active_sessions_never_negative() {
        let metrics = BusinessMetrics::new();

        // Try to close more sessions than created
        metrics.record_session_closed();
        metrics.record_session_closed();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.active_sessions, 0);
    }
}
