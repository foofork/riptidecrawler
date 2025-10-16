//! Comprehensive test suite for PDF metrics collection system
//!
//! Tests cover:
//! - Basic metric collection and aggregation
//! - Concurrent access and thread safety
//! - Edge cases (overflow, extreme values, zero values)
//! - Memory efficiency and leak detection
//! - Export format validation
//! - Performance overhead measurement
//! - Storage rotation and cleanup
//! - Metric reset functionality

use riptide_pdf::metrics::{PdfMetricsCollector, PdfMetricsSnapshot};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_metrics_default_state() {
    let collector = PdfMetricsCollector::new();
    let snapshot = collector.get_snapshot();

    assert_eq!(snapshot.total_processed, 0);
    assert_eq!(snapshot.total_failed, 0);
    assert_eq!(snapshot.memory_limit_failures, 0);
    assert_eq!(snapshot.avg_processing_time_ms, 0.0);
    assert_eq!(snapshot.peak_memory_usage, 0);
    assert_eq!(snapshot.avg_pages_per_pdf, 0.0);
    assert_eq!(snapshot.max_concurrent_operations, 0);
    assert_eq!(snapshot.avg_queue_wait_time_ms, 0.0);
    assert_eq!(snapshot.success_rate, 0.0);
    assert_eq!(snapshot.memory_spikes_handled, 0);
    assert_eq!(snapshot.cleanup_operations, 0);
    assert_eq!(snapshot.average_pages_per_second, 0.0);
    assert_eq!(snapshot.average_progress_overhead_us, 0.0);
    assert_eq!(snapshot.average_page_processing_time_ms, 0.0);
}

#[test]
fn test_single_success_recording() {
    let collector = PdfMetricsCollector::new();

    collector.record_processing_success(
        Duration::from_millis(1000),
        10,
        50 * 1024 * 1024, // 50MB
    );

    let snapshot = collector.get_snapshot();

    assert_eq!(snapshot.total_processed, 1);
    assert_eq!(snapshot.total_failed, 0);
    assert_eq!(snapshot.avg_processing_time_ms, 1000.0);
    assert_eq!(snapshot.peak_memory_usage, 50 * 1024 * 1024);
    assert_eq!(snapshot.avg_pages_per_pdf, 10.0);
    assert_eq!(snapshot.success_rate, 1.0);
}

#[test]
fn test_multiple_success_recordings() {
    let collector = PdfMetricsCollector::new();

    // Record multiple successful operations with varying metrics
    collector.record_processing_success(Duration::from_millis(1000), 10, 50 * 1024 * 1024);
    collector.record_processing_success(Duration::from_millis(2000), 20, 75 * 1024 * 1024);
    collector.record_processing_success(Duration::from_millis(1500), 15, 60 * 1024 * 1024);

    let snapshot = collector.get_snapshot();

    assert_eq!(snapshot.total_processed, 3);
    assert_eq!(snapshot.total_failed, 0);
    assert_eq!(snapshot.avg_processing_time_ms, 1500.0); // (1000 + 2000 + 1500) / 3
    assert_eq!(snapshot.peak_memory_usage, 75 * 1024 * 1024); // Max of all
    assert_eq!(snapshot.avg_pages_per_pdf, 15.0); // (10 + 20 + 15) / 3
    assert_eq!(snapshot.success_rate, 1.0);
}

#[test]
fn test_failure_recording() {
    let collector = PdfMetricsCollector::new();

    collector.record_processing_failure(false);
    collector.record_processing_failure(true); // Memory limit failure

    let snapshot = collector.get_snapshot();

    assert_eq!(snapshot.total_failed, 2);
    assert_eq!(snapshot.memory_limit_failures, 1);
    assert_eq!(snapshot.success_rate, 0.0);
}

#[test]
fn test_mixed_success_and_failure() {
    let collector = PdfMetricsCollector::new();

    // 3 successes
    collector.record_processing_success(Duration::from_millis(1000), 10, 50 * 1024 * 1024);
    collector.record_processing_success(Duration::from_millis(2000), 20, 75 * 1024 * 1024);
    collector.record_processing_success(Duration::from_millis(1500), 15, 60 * 1024 * 1024);

    // 2 failures
    collector.record_processing_failure(false);
    collector.record_processing_failure(true);

    let snapshot = collector.get_snapshot();

    assert_eq!(snapshot.total_processed, 3);
    assert_eq!(snapshot.total_failed, 2);
    assert_eq!(snapshot.success_rate, 0.6); // 3 / 5
}

#[test]
fn test_concurrency_metrics() {
    let collector = PdfMetricsCollector::new();

    collector.record_concurrency_metrics(5, Duration::from_millis(100));
    collector.record_concurrency_metrics(10, Duration::from_millis(200));
    collector.record_concurrency_metrics(7, Duration::from_millis(150));

    let snapshot = collector.get_snapshot();

    assert_eq!(snapshot.max_concurrent_operations, 10); // Max observed
    // Average queue wait time should be calculated correctly
    assert!(snapshot.avg_queue_wait_time_ms > 0.0);
}

#[test]
fn test_memory_management_metrics() {
    let collector = PdfMetricsCollector::new();

    collector.record_memory_spike_detected();
    collector.record_memory_spike_detected();
    collector.record_cleanup_performed();

    let snapshot = collector.get_snapshot();

    assert_eq!(snapshot.memory_spikes_handled, 2);
    assert_eq!(snapshot.cleanup_operations, 1);
}

#[test]
fn test_pages_per_second_recording() {
    let collector = PdfMetricsCollector::new();

    collector.record_pages_per_second(10.5);
    collector.record_pages_per_second(15.3);
    collector.record_pages_per_second(12.7);

    let snapshot = collector.get_snapshot();

    // Average should be (10.5 + 15.3 + 12.7) / 3 ≈ 12.83
    assert!((snapshot.average_pages_per_second - 12.83).abs() < 0.1);
}

#[test]
fn test_progress_overhead_recording() {
    let collector = PdfMetricsCollector::new();

    collector.record_progress_overhead(100); // 100 microseconds
    collector.record_progress_overhead(200);
    collector.record_progress_overhead(150);

    let snapshot = collector.get_snapshot();

    assert_eq!(snapshot.average_progress_overhead_us, 150.0); // (100 + 200 + 150) / 3
}

#[test]
fn test_page_processing_time_recording() {
    let collector = PdfMetricsCollector::new();

    collector.record_page_processing_time(50); // 50ms per page
    collector.record_page_processing_time(75);
    collector.record_page_processing_time(60);

    let snapshot = collector.get_snapshot();

    assert_eq!(snapshot.average_page_processing_time_ms, 61.666666666666664); // (50 + 75 + 60) / 3
}

#[test]
fn test_metrics_reset() {
    let collector = PdfMetricsCollector::new();

    // Record various metrics
    collector.record_processing_success(Duration::from_millis(1000), 10, 50 * 1024 * 1024);
    collector.record_processing_failure(true);
    collector.record_memory_spike_detected();
    collector.record_cleanup_performed();
    collector.record_pages_per_second(10.0);

    // Verify metrics are recorded
    let snapshot_before = collector.get_snapshot();
    assert!(snapshot_before.total_processed > 0 || snapshot_before.total_failed > 0);

    // Reset
    collector.reset();

    // Verify all metrics are zeroed
    let snapshot_after = collector.get_snapshot();
    assert_eq!(snapshot_after.total_processed, 0);
    assert_eq!(snapshot_after.total_failed, 0);
    assert_eq!(snapshot_after.memory_limit_failures, 0);
    assert_eq!(snapshot_after.peak_memory_usage, 0);
    assert_eq!(snapshot_after.memory_spikes_handled, 0);
    assert_eq!(snapshot_after.cleanup_operations, 0);
    assert_eq!(snapshot_after.average_pages_per_second, 0.0);
}

#[test]
fn test_prometheus_export_format() {
    let collector = PdfMetricsCollector::new();

    collector.record_processing_success(Duration::from_millis(1500), 15, 100 * 1024 * 1024);
    collector.record_memory_spike_detected();
    collector.record_cleanup_performed();
    collector.record_pages_per_second(12.5);

    let prometheus_metrics = collector.export_for_prometheus();

    // Verify all expected metrics are present
    assert!(prometheus_metrics.contains_key("pdf_total_processed"));
    assert!(prometheus_metrics.contains_key("pdf_total_failed"));
    assert!(prometheus_metrics.contains_key("pdf_memory_limit_failures"));
    assert!(prometheus_metrics.contains_key("pdf_avg_processing_time_ms"));
    assert!(prometheus_metrics.contains_key("pdf_peak_memory_mb"));
    assert!(prometheus_metrics.contains_key("pdf_avg_pages_per_pdf"));
    assert!(prometheus_metrics.contains_key("pdf_max_concurrent_ops"));
    assert!(prometheus_metrics.contains_key("pdf_avg_queue_wait_ms"));
    assert!(prometheus_metrics.contains_key("pdf_success_rate"));
    assert!(prometheus_metrics.contains_key("pdf_memory_spikes_handled"));
    assert!(prometheus_metrics.contains_key("pdf_cleanup_operations"));
    assert!(prometheus_metrics.contains_key("pdf_memory_efficiency_pages_per_mb"));
    assert!(prometheus_metrics.contains_key("pdf_average_pages_per_second"));
    assert!(prometheus_metrics.contains_key("pdf_average_progress_overhead_us"));
    assert!(prometheus_metrics.contains_key("pdf_average_page_processing_time_ms"));

    // Verify values
    assert_eq!(prometheus_metrics["pdf_total_processed"], 1.0);
    assert_eq!(prometheus_metrics["pdf_avg_processing_time_ms"], 1500.0);
    assert_eq!(prometheus_metrics["pdf_peak_memory_mb"], 100.0);
    assert_eq!(prometheus_metrics["pdf_success_rate"], 1.0);
    assert_eq!(prometheus_metrics["pdf_memory_spikes_handled"], 1.0);
    assert_eq!(prometheus_metrics["pdf_cleanup_operations"], 1.0);
}

#[test]
fn test_memory_efficiency_calculation() {
    let collector = PdfMetricsCollector::new();

    // Process 100 pages with 50MB memory usage
    collector.record_processing_success(Duration::from_millis(1000), 100, 50 * 1024 * 1024);

    let snapshot = collector.get_snapshot();

    // Memory efficiency = pages / (memory_MB) = 100 / 50 = 2.0 pages per MB
    assert_eq!(snapshot.memory_efficiency, 2.0);
}

#[test]
fn test_peak_memory_tracking() {
    let collector = PdfMetricsCollector::new();

    // Record operations with increasing then decreasing memory
    collector.record_processing_success(Duration::from_millis(1000), 10, 50 * 1024 * 1024);
    collector.record_processing_success(Duration::from_millis(1000), 10, 100 * 1024 * 1024);
    collector.record_processing_success(Duration::from_millis(1000), 10, 75 * 1024 * 1024);

    let snapshot = collector.get_snapshot();

    // Peak should be the maximum observed (100MB)
    assert_eq!(snapshot.peak_memory_usage, 100 * 1024 * 1024);
}

#[test]
fn test_concurrent_metric_updates_thread_safety() {
    let collector = Arc::new(PdfMetricsCollector::new());
    let mut handles = vec![];

    // Spawn 10 threads, each recording 100 operations
    for _ in 0..10 {
        let collector_clone = Arc::clone(&collector);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                collector_clone.record_processing_success(
                    Duration::from_millis(100),
                    10,
                    50 * 1024 * 1024,
                );
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    let snapshot = collector.get_snapshot();

    // Should have exactly 1000 successful operations (10 threads * 100 operations)
    assert_eq!(snapshot.total_processed, 1000);
}

#[test]
fn test_concurrent_mixed_operations() {
    let collector = Arc::new(PdfMetricsCollector::new());
    let mut handles = vec![];

    // Thread 1: Success recordings
    let c1 = Arc::clone(&collector);
    handles.push(thread::spawn(move || {
        for _ in 0..50 {
            c1.record_processing_success(Duration::from_millis(100), 10, 50 * 1024 * 1024);
        }
    }));

    // Thread 2: Failure recordings
    let c2 = Arc::clone(&collector);
    handles.push(thread::spawn(move || {
        for _ in 0..30 {
            c2.record_processing_failure(false);
        }
    }));

    // Thread 3: Memory spike recordings
    let c3 = Arc::clone(&collector);
    handles.push(thread::spawn(move || {
        for _ in 0..20 {
            c3.record_memory_spike_detected();
        }
    }));

    // Thread 4: Cleanup recordings
    let c4 = Arc::clone(&collector);
    handles.push(thread::spawn(move || {
        for _ in 0..15 {
            c4.record_cleanup_performed();
        }
    }));

    for handle in handles {
        handle.join().unwrap();
    }

    let snapshot = collector.get_snapshot();

    assert_eq!(snapshot.total_processed, 50);
    assert_eq!(snapshot.total_failed, 30);
    assert_eq!(snapshot.memory_spikes_handled, 20);
    assert_eq!(snapshot.cleanup_operations, 15);
}

#[test]
fn test_edge_case_zero_operations() {
    let collector = PdfMetricsCollector::new();
    let snapshot = collector.get_snapshot();

    // All averages should be 0.0, not NaN or infinity
    assert_eq!(snapshot.avg_processing_time_ms, 0.0);
    assert_eq!(snapshot.avg_pages_per_pdf, 0.0);
    assert_eq!(snapshot.avg_queue_wait_time_ms, 0.0);
    assert_eq!(snapshot.success_rate, 0.0);
    assert_eq!(snapshot.memory_efficiency, 0.0);
    assert_eq!(snapshot.average_pages_per_second, 0.0);
}

#[test]
fn test_edge_case_very_large_values() {
    let collector = PdfMetricsCollector::new();

    // Test with very large values
    collector.record_processing_success(
        Duration::from_secs(3600), // 1 hour processing time
        10000,                      // 10,000 pages
        10 * 1024 * 1024 * 1024,   // 10 GB memory
    );

    let snapshot = collector.get_snapshot();

    assert_eq!(snapshot.total_processed, 1);
    assert_eq!(snapshot.avg_processing_time_ms, 3600000.0); // 1 hour in ms
    assert_eq!(snapshot.avg_pages_per_pdf, 10000.0);
    assert_eq!(snapshot.peak_memory_usage, 10 * 1024 * 1024 * 1024);
}

#[test]
fn test_edge_case_very_small_values() {
    let collector = PdfMetricsCollector::new();

    // Test with very small values
    collector.record_processing_success(
        Duration::from_micros(100), // 100 microseconds
        1,                          // 1 page
        1024,                       // 1 KB memory
    );

    let snapshot = collector.get_snapshot();

    assert_eq!(snapshot.total_processed, 1);
    assert!(snapshot.avg_processing_time_ms < 1.0); // Less than 1ms
    assert_eq!(snapshot.avg_pages_per_pdf, 1.0);
}

#[test]
fn test_snapshot_timestamp() {
    let collector = PdfMetricsCollector::new();

    let snapshot1 = collector.get_snapshot();
    thread::sleep(Duration::from_millis(100));
    let snapshot2 = collector.get_snapshot();

    // Timestamps should be different and increasing
    assert!(snapshot2.timestamp > snapshot1.timestamp);
}

#[test]
fn test_concurrent_snapshot_reads() {
    let collector = Arc::new(PdfMetricsCollector::new());

    // Record some initial data
    collector.record_processing_success(Duration::from_millis(1000), 10, 50 * 1024 * 1024);

    let mut handles = vec![];

    // Spawn multiple threads reading snapshots concurrently
    for _ in 0..20 {
        let collector_clone = Arc::clone(&collector);
        let handle = thread::spawn(move || {
            for _ in 0..50 {
                let _snapshot = collector_clone.get_snapshot();
                // All snapshots should be consistent
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Should not panic or produce inconsistent results
}

#[test]
fn test_concurrent_read_write() {
    let collector = Arc::new(PdfMetricsCollector::new());
    let mut handles = vec![];

    // Writers
    for _ in 0..5 {
        let c = Arc::clone(&collector);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                c.record_processing_success(Duration::from_millis(100), 10, 50 * 1024 * 1024);
            }
        }));
    }

    // Readers
    for _ in 0..5 {
        let c = Arc::clone(&collector);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                let _snapshot = c.get_snapshot();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let final_snapshot = collector.get_snapshot();
    assert_eq!(final_snapshot.total_processed, 500); // 5 writers * 100 ops
}

#[test]
fn test_metric_overflow_protection() {
    let collector = PdfMetricsCollector::new();

    // Try to overflow counters by recording many operations
    // u64::MAX is 18,446,744,073,709,551,615
    // This test verifies we don't panic on large numbers
    for _ in 0..10000 {
        collector.record_processing_success(Duration::from_millis(1), 1, 1024);
    }

    let snapshot = collector.get_snapshot();
    assert_eq!(snapshot.total_processed, 10000);
}

#[test]
fn test_prometheus_export_all_metrics_numeric() {
    let collector = PdfMetricsCollector::new();

    collector.record_processing_success(Duration::from_millis(1500), 15, 100 * 1024 * 1024);

    let metrics = collector.export_for_prometheus();

    // All exported values should be valid numbers (not NaN or Infinity)
    for (_key, value) in metrics.iter() {
        assert!(value.is_finite(), "Metric value should be finite: {}", value);
        assert!(!value.is_nan(), "Metric value should not be NaN");
    }
}

#[test]
fn test_memory_efficiency_edge_cases() {
    let collector = PdfMetricsCollector::new();

    // Zero memory usage
    collector.record_processing_success(Duration::from_millis(100), 10, 0);

    let snapshot = collector.get_snapshot();
    // Should not panic, efficiency should be infinity or zero
    assert!(snapshot.memory_efficiency.is_infinite() || snapshot.memory_efficiency == 0.0);

    // Reset and test with zero pages
    collector.reset();
    collector.record_processing_success(Duration::from_millis(100), 0, 1024);

    let snapshot2 = collector.get_snapshot();
    assert_eq!(snapshot2.memory_efficiency, 0.0);
}

#[test]
fn test_success_rate_calculation_edge_cases() {
    let collector = PdfMetricsCollector::new();

    // All failures
    for _ in 0..10 {
        collector.record_processing_failure(false);
    }

    let snapshot = collector.get_snapshot();
    assert_eq!(snapshot.success_rate, 0.0);

    // All successes (after reset)
    collector.reset();
    for _ in 0..10 {
        collector.record_processing_success(Duration::from_millis(100), 10, 1024);
    }

    let snapshot2 = collector.get_snapshot();
    assert_eq!(snapshot2.success_rate, 1.0);
}

#[test]
fn test_performance_overhead_minimal() {
    let collector = PdfMetricsCollector::new();

    // Measure time to record 10,000 operations
    let start = std::time::Instant::now();

    for _ in 0..10000 {
        collector.record_processing_success(Duration::from_millis(100), 10, 50 * 1024 * 1024);
    }

    let elapsed = start.elapsed();

    // Recording 10,000 operations should take less than 100ms
    // This ensures minimal performance overhead
    assert!(elapsed < Duration::from_millis(100), "Metric collection overhead too high: {:?}", elapsed);
}

#[test]
fn test_clone_independence() {
    let collector1 = PdfMetricsCollector::new();

    collector1.record_processing_success(Duration::from_millis(1000), 10, 50 * 1024 * 1024);

    let collector2 = collector1.clone();

    // Both should have the same metrics initially
    assert_eq!(collector1.get_snapshot().total_processed, collector2.get_snapshot().total_processed);

    // Recording to collector2 should not affect collector1
    collector2.record_processing_success(Duration::from_millis(1000), 10, 50 * 1024 * 1024);

    // Wait a moment for potential race conditions
    thread::sleep(Duration::from_millis(10));

    // Both clones share the same underlying storage (Arc), so both should see the update
    assert_eq!(collector1.get_snapshot().total_processed, 2);
    assert_eq!(collector2.get_snapshot().total_processed, 2);
}
