//! Metrics collection for PDF processing performance monitoring

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::{Serialize, Deserialize};

/// Global metrics collector for PDF processing
#[derive(Debug, Clone)]
pub struct PdfMetricsCollector {
    metrics: Arc<PdfMetricsStorage>,
}

/// Thread-safe storage for PDF processing metrics
#[derive(Debug)]
struct PdfMetricsStorage {
    // Processing counters
    total_processed: AtomicU64,
    total_failed: AtomicU64,
    total_memory_limit_errors: AtomicU64,

    // Performance metrics
    total_processing_time_ms: AtomicU64,
    peak_memory_usage: AtomicU64,
    total_pages_processed: AtomicU64,

    // Concurrency metrics
    max_concurrent_operations: AtomicU64,
    total_queue_wait_time_ms: AtomicU64,

    // Memory spike tracking
    memory_spikes_detected: AtomicU64,
    cleanup_operations_performed: AtomicU64,
}

/// Snapshot of current PDF processing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfMetricsSnapshot {
    /// Total number of PDFs successfully processed
    pub total_processed: u64,

    /// Total number of failed processing attempts
    pub total_failed: u64,

    /// Number of failures due to memory limits
    pub memory_limit_failures: u64,

    /// Average processing time per PDF (milliseconds)
    pub avg_processing_time_ms: f64,

    /// Peak memory usage across all operations (bytes)
    pub peak_memory_usage: u64,

    /// Average pages processed per PDF
    pub avg_pages_per_pdf: f64,

    /// Maximum concurrent operations observed
    pub max_concurrent_operations: u64,

    /// Average queue wait time (milliseconds)
    pub avg_queue_wait_time_ms: f64,

    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,

    /// Number of memory spikes detected and handled
    pub memory_spikes_handled: u64,

    /// Total cleanup operations performed
    pub cleanup_operations: u64,

    /// Memory efficiency ratio (useful output per memory used)
    pub memory_efficiency: f64,

    /// Timestamp when snapshot was taken
    pub timestamp: u64,
}

impl Default for PdfMetricsStorage {
    fn default() -> Self {
        Self {
            total_processed: AtomicU64::new(0),
            total_failed: AtomicU64::new(0),
            total_memory_limit_errors: AtomicU64::new(0),
            total_processing_time_ms: AtomicU64::new(0),
            peak_memory_usage: AtomicU64::new(0),
            total_pages_processed: AtomicU64::new(0),
            max_concurrent_operations: AtomicU64::new(0),
            total_queue_wait_time_ms: AtomicU64::new(0),
            memory_spikes_detected: AtomicU64::new(0),
            cleanup_operations_performed: AtomicU64::new(0),
        }
    }
}

impl PdfMetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(PdfMetricsStorage::default()),
        }
    }

    /// Record a successful PDF processing operation
    pub fn record_processing_success(&self, processing_time: Duration, pages: u32, memory_used: u64) {
        self.metrics.total_processed.fetch_add(1, Ordering::Relaxed);
        self.metrics.total_processing_time_ms.fetch_add(
            processing_time.as_millis() as u64,
            Ordering::Relaxed
        );
        self.metrics.total_pages_processed.fetch_add(pages as u64, Ordering::Relaxed);

        // Update peak memory usage
        let current_peak = self.metrics.peak_memory_usage.load(Ordering::Relaxed);
        if memory_used > current_peak {
            self.metrics.peak_memory_usage.store(memory_used, Ordering::Relaxed);
        }
    }

    /// Record a failed PDF processing operation
    pub fn record_processing_failure(&self, is_memory_limit: bool) {
        self.metrics.total_failed.fetch_add(1, Ordering::Relaxed);
        if is_memory_limit {
            self.metrics.total_memory_limit_errors.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record concurrency metrics
    pub fn record_concurrency_metrics(&self, concurrent_ops: u64, queue_wait_time: Duration) {
        // Update max concurrent operations
        let current_max = self.metrics.max_concurrent_operations.load(Ordering::Relaxed);
        if concurrent_ops > current_max {
            self.metrics.max_concurrent_operations.store(concurrent_ops, Ordering::Relaxed);
        }

        self.metrics.total_queue_wait_time_ms.fetch_add(
            queue_wait_time.as_millis() as u64,
            Ordering::Relaxed
        );
    }

    /// Record memory management events
    pub fn record_memory_spike_detected(&self) {
        self.metrics.memory_spikes_detected.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cleanup_performed(&self) {
        self.metrics.cleanup_operations_performed.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current metrics snapshot
    pub fn get_snapshot(&self) -> PdfMetricsSnapshot {
        let total_processed = self.metrics.total_processed.load(Ordering::Relaxed);
        let total_failed = self.metrics.total_failed.load(Ordering::Relaxed);
        let total_operations = total_processed + total_failed;

        let success_rate = if total_operations > 0 {
            total_processed as f64 / total_operations as f64
        } else {
            0.0
        };

        let avg_processing_time_ms = if total_processed > 0 {
            self.metrics.total_processing_time_ms.load(Ordering::Relaxed) as f64 / total_processed as f64
        } else {
            0.0
        };

        let total_pages = self.metrics.total_pages_processed.load(Ordering::Relaxed);
        let avg_pages_per_pdf = if total_processed > 0 {
            total_pages as f64 / total_processed as f64
        } else {
            0.0
        };

        let avg_queue_wait_time_ms = if total_operations > 0 {
            self.metrics.total_queue_wait_time_ms.load(Ordering::Relaxed) as f64 / total_operations as f64
        } else {
            0.0
        };

        let peak_memory = self.metrics.peak_memory_usage.load(Ordering::Relaxed);
        let memory_efficiency = if peak_memory > 0 && total_pages > 0 {
            total_pages as f64 / (peak_memory as f64 / (1024.0 * 1024.0)) // Pages per MB
        } else {
            0.0
        };

        PdfMetricsSnapshot {
            total_processed,
            total_failed,
            memory_limit_failures: self.metrics.total_memory_limit_errors.load(Ordering::Relaxed),
            avg_processing_time_ms,
            peak_memory_usage: peak_memory,
            avg_pages_per_pdf,
            max_concurrent_operations: self.metrics.max_concurrent_operations.load(Ordering::Relaxed),
            avg_queue_wait_time_ms,
            success_rate,
            memory_spikes_handled: self.metrics.memory_spikes_detected.load(Ordering::Relaxed),
            cleanup_operations: self.metrics.cleanup_operations_performed.load(Ordering::Relaxed),
            memory_efficiency,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Reset all metrics (for testing or periodic resets)
    pub fn reset(&self) {
        self.metrics.total_processed.store(0, Ordering::Relaxed);
        self.metrics.total_failed.store(0, Ordering::Relaxed);
        self.metrics.total_memory_limit_errors.store(0, Ordering::Relaxed);
        self.metrics.total_processing_time_ms.store(0, Ordering::Relaxed);
        self.metrics.peak_memory_usage.store(0, Ordering::Relaxed);
        self.metrics.total_pages_processed.store(0, Ordering::Relaxed);
        self.metrics.max_concurrent_operations.store(0, Ordering::Relaxed);
        self.metrics.total_queue_wait_time_ms.store(0, Ordering::Relaxed);
        self.metrics.memory_spikes_detected.store(0, Ordering::Relaxed);
        self.metrics.cleanup_operations_performed.store(0, Ordering::Relaxed);
    }

    /// Export metrics in various formats for monitoring systems
    pub fn export_for_prometheus(&self) -> HashMap<String, f64> {
        let snapshot = self.get_snapshot();
        let mut metrics = HashMap::new();

        metrics.insert("pdf_total_processed".to_string(), snapshot.total_processed as f64);
        metrics.insert("pdf_total_failed".to_string(), snapshot.total_failed as f64);
        metrics.insert("pdf_memory_limit_failures".to_string(), snapshot.memory_limit_failures as f64);
        metrics.insert("pdf_avg_processing_time_ms".to_string(), snapshot.avg_processing_time_ms);
        metrics.insert("pdf_peak_memory_mb".to_string(), snapshot.peak_memory_usage as f64 / (1024.0 * 1024.0));
        metrics.insert("pdf_avg_pages_per_pdf".to_string(), snapshot.avg_pages_per_pdf);
        metrics.insert("pdf_max_concurrent_ops".to_string(), snapshot.max_concurrent_operations as f64);
        metrics.insert("pdf_avg_queue_wait_ms".to_string(), snapshot.avg_queue_wait_time_ms);
        metrics.insert("pdf_success_rate".to_string(), snapshot.success_rate);
        metrics.insert("pdf_memory_spikes_handled".to_string(), snapshot.memory_spikes_handled as f64);
        metrics.insert("pdf_cleanup_operations".to_string(), snapshot.cleanup_operations as f64);
        metrics.insert("pdf_memory_efficiency_pages_per_mb".to_string(), snapshot.memory_efficiency);

        metrics
    }
}

impl Default for PdfMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance measurement helper for individual operations
#[derive(Debug)]
pub struct PdfOperationTimer {
    start_time: Instant,
    queue_start_time: Option<Instant>,
    metrics: Arc<PdfMetricsCollector>,
}

impl PdfOperationTimer {
    /// Start timing a new PDF operation
    pub fn start(metrics: Arc<PdfMetricsCollector>) -> Self {
        Self {
            start_time: Instant::now(),
            queue_start_time: Some(Instant::now()),
            metrics,
        }
    }

    /// Mark the end of queueing and start of actual processing
    pub fn start_processing(&mut self) {
        if let Some(queue_start) = self.queue_start_time.take() {
            let queue_time = queue_start.elapsed();
            self.metrics.record_concurrency_metrics(1, queue_time);
        }
        self.start_time = Instant::now();
    }

    /// Complete the operation successfully
    pub fn complete_success(self, pages: u32, memory_used: u64) {
        let processing_time = self.start_time.elapsed();
        self.metrics.record_processing_success(processing_time, pages, memory_used);
    }

    /// Complete the operation with failure
    pub fn complete_failure(self, is_memory_limit: bool) {
        self.metrics.record_processing_failure(is_memory_limit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_metrics_collection() {
        let collector = PdfMetricsCollector::new();

        // Record some successful operations
        collector.record_processing_success(Duration::from_millis(1000), 10, 50 * 1024 * 1024);
        collector.record_processing_success(Duration::from_millis(2000), 20, 75 * 1024 * 1024);

        // Record some failures
        collector.record_processing_failure(false);
        collector.record_processing_failure(true);

        let snapshot = collector.get_snapshot();

        assert_eq!(snapshot.total_processed, 2);
        assert_eq!(snapshot.total_failed, 2);
        assert_eq!(snapshot.memory_limit_failures, 1);
        assert_eq!(snapshot.avg_processing_time_ms, 1500.0);
        assert_eq!(snapshot.peak_memory_usage, 75 * 1024 * 1024);
        assert_eq!(snapshot.avg_pages_per_pdf, 15.0);
        assert_eq!(snapshot.success_rate, 0.5);
    }

    #[test]
    fn test_metrics_reset() {
        let collector = PdfMetricsCollector::new();

        collector.record_processing_success(Duration::from_millis(1000), 10, 50 * 1024 * 1024);
        collector.record_processing_failure(true);

        let snapshot_before = collector.get_snapshot();
        assert!(snapshot_before.total_processed > 0 || snapshot_before.total_failed > 0);

        collector.reset();

        let snapshot_after = collector.get_snapshot();
        assert_eq!(snapshot_after.total_processed, 0);
        assert_eq!(snapshot_after.total_failed, 0);
        assert_eq!(snapshot_after.peak_memory_usage, 0);
    }

    #[test]
    fn test_prometheus_export() {
        let collector = PdfMetricsCollector::new();

        collector.record_processing_success(Duration::from_millis(1500), 15, 100 * 1024 * 1024);
        collector.record_memory_spike_detected();
        collector.record_cleanup_performed();

        let prometheus_metrics = collector.export_for_prometheus();

        assert!(prometheus_metrics.contains_key("pdf_total_processed"));
        assert!(prometheus_metrics.contains_key("pdf_avg_processing_time_ms"));
        assert!(prometheus_metrics.contains_key("pdf_peak_memory_mb"));
        assert!(prometheus_metrics.contains_key("pdf_success_rate"));

        assert_eq!(prometheus_metrics["pdf_total_processed"], 1.0);
        assert_eq!(prometheus_metrics["pdf_avg_processing_time_ms"], 1500.0);
        assert_eq!(prometheus_metrics["pdf_peak_memory_mb"], 100.0);
        assert_eq!(prometheus_metrics["pdf_success_rate"], 1.0);
    }

    #[test]
    fn test_operation_timer() {
        let collector = Arc::new(PdfMetricsCollector::new());

        let mut timer = PdfOperationTimer::start(collector.clone());

        // Simulate queue wait
        std::thread::sleep(Duration::from_millis(10));
        timer.start_processing();

        // Simulate processing
        std::thread::sleep(Duration::from_millis(20));
        timer.complete_success(5, 25 * 1024 * 1024);

        let snapshot = collector.get_snapshot();
        assert_eq!(snapshot.total_processed, 1);
        assert!(snapshot.avg_processing_time_ms > 0.0);
    }
}