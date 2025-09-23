use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Barrier};
use tokio::time::sleep;
use riptide_core::monitoring::{
    MetricsCollector, AlertManager, TimeSeriesBuffer, PerformanceMetrics, AlertSeverity
};

/// Integration tests for the monitoring system
/// These tests verify the complete monitoring pipeline works correctly
/// with all components integrated together.

#[tokio::test]
async fn test_full_monitoring_pipeline() {
    let collector = Arc::new(MetricsCollector::new());
    let mut alert_manager = AlertManager::new();

    // Simulate realistic workload
    let extraction_durations = vec![
        Duration::from_millis(50),
        Duration::from_millis(150),
        Duration::from_millis(300),
        Duration::from_millis(75),
        Duration::from_millis(200),
    ];

    // Record metrics over time
    for (i, duration) in extraction_durations.iter().enumerate() {
        let success = i % 4 != 3; // 75% success rate
        let quality_score = if success { Some(80 + (i % 20) as u8) } else { None };
        let word_count = if success { Some(400 + (i * 50) as u32) } else { None };
        let was_cached = i % 3 == 0; // 33% cache hit rate

        collector.record_extraction(*duration, success, quality_score, word_count, was_cached).await;

        if !success {
            collector.record_error("timeout", true).await;
        }

        // Small delay to simulate realistic timing
        sleep(Duration::from_millis(10)).await;
    }

    // Update pool stats
    collector.update_pool_stats(10, 7, 3).await;

    // Allow metrics to be processed
    sleep(Duration::from_millis(100)).await;

    // Verify metrics are collected correctly
    let metrics = collector.get_current_metrics().await;
    assert_eq!(metrics.total_extractions, 5);
    assert_eq!(metrics.successful_extractions, 4);
    assert_eq!(metrics.failed_extractions, 1);
    assert!(metrics.avg_content_quality_score > 0.0);
    assert!(metrics.cache_hit_ratio > 0.0 && metrics.cache_hit_ratio <= 1.0);
    assert_eq!(metrics.pool_size, 10);
    assert_eq!(metrics.active_instances, 7);
    assert_eq!(metrics.idle_instances, 3);

    // Test performance report generation
    let report = collector.get_performance_report(Duration::from_secs(60)).await;
    assert!(report.avg_extraction_time > 0.0);
    assert!(!report.recommendations.is_empty());

    // Test alert checking
    let alerts = alert_manager.check_alerts(&metrics).await;
    // Should not trigger alerts with normal metrics
    assert!(alerts.is_empty() || alerts.iter().all(|a| matches!(a.severity, AlertSeverity::Info | AlertSeverity::Warning)));
}

#[tokio::test]
async fn test_monitoring_under_stress() {
    let collector = Arc::new(MetricsCollector::new());
    let mut alert_manager = AlertManager::new();

    // Simulate high error rate scenario
    for i in 0..100 {
        let success = i % 2 == 0; // 50% success rate (high error rate)
        let duration = Duration::from_millis(200 + (i % 100) as u64);

        collector.record_extraction(duration, success, Some(75), Some(300), false).await;

        if !success {
            collector.record_error("network_timeout", true).await;
        }
    }

    // Allow metrics to stabilize
    sleep(Duration::from_millis(200)).await;

    let metrics = collector.get_current_metrics().await;

    // Verify high error rate is detected
    assert!(metrics.error_rate > 40.0, "Error rate should be high: {}", metrics.error_rate);
    assert!(metrics.health_score < 80.0, "Health score should be degraded: {}", metrics.health_score);

    // Check that alerts are triggered
    let alerts = alert_manager.check_alerts(&metrics).await;
    assert!(!alerts.is_empty(), "Should trigger alerts under stress");

    // Verify performance report identifies issues
    let report = collector.get_performance_report(Duration::from_secs(60)).await;
    assert!(report.recommendations.iter().any(|r| r.contains("error rate")));
}

#[tokio::test]
async fn test_monitoring_system_recovery() {
    let collector = Arc::new(MetricsCollector::new());
    let mut alert_manager = AlertManager::new();

    // First phase: Create problematic conditions
    for i in 0..20 {
        collector.record_extraction(Duration::from_millis(5000), false, None, None, false).await; // Slow failures
        collector.record_error("severe_error", false).await;
    }

    let stressed_metrics = collector.get_current_metrics().await;
    let stressed_alerts = alert_manager.check_alerts(&stressed_metrics).await;
    assert!(!stressed_alerts.is_empty(), "Should have alerts during stress");

    // Second phase: System recovery
    for i in 0..50 {
        collector.record_extraction(Duration::from_millis(50), true, Some(90), Some(800), true).await; // Fast successes
    }

    // Allow recovery metrics to be processed
    sleep(Duration::from_millis(200)).await;

    let recovered_metrics = collector.get_current_metrics().await;

    // Verify recovery
    assert!(recovered_metrics.health_score > stressed_metrics.health_score,
           "Health score should improve: {} -> {}", stressed_metrics.health_score, recovered_metrics.health_score);
    assert!(recovered_metrics.error_rate < stressed_metrics.error_rate,
           "Error rate should decrease: {} -> {}", stressed_metrics.error_rate, recovered_metrics.error_rate);
}

#[tokio::test]
async fn test_alert_cooldown_mechanism() {
    let mut alert_manager = AlertManager::new();

    let high_error_metrics = PerformanceMetrics {
        error_rate: 15.0, // Above threshold
        ..Default::default()
    };

    // First alert should trigger
    let alerts1 = alert_manager.check_alerts(&high_error_metrics).await;
    assert!(!alerts1.is_empty(), "First alert should trigger");

    // Immediate second check should not trigger due to cooldown
    let alerts2 = alert_manager.check_alerts(&high_error_metrics).await;
    assert!(alerts2.is_empty(), "Second alert should be suppressed by cooldown");

    // Verify the alert contains proper information
    let alert = &alerts1[0];
    assert_eq!(alert.rule_name, "high_error_rate");
    assert_eq!(alert.current_value, 15.0);
    assert!(alert.message.contains("15.00"));
}

#[tokio::test]
async fn test_metrics_backward_compatibility() {
    let collector = MetricsCollector::new();

    // Test that all expected fields are present and accessible
    let metrics = collector.get_current_metrics().await;

    // Verify all timing metrics
    assert_eq!(metrics.avg_extraction_time_ms, 0.0);
    assert_eq!(metrics.p95_extraction_time_ms, 0.0);
    assert_eq!(metrics.p99_extraction_time_ms, 0.0);

    // Verify throughput metrics
    assert_eq!(metrics.requests_per_second, 0.0);
    assert_eq!(metrics.successful_extractions, 0);
    assert_eq!(metrics.failed_extractions, 0);
    assert_eq!(metrics.total_extractions, 0);

    // Verify resource metrics
    assert_eq!(metrics.memory_usage_bytes, 0);
    assert_eq!(metrics.cpu_usage_percent, 0.0);
    assert_eq!(metrics.pool_size, 0);
    assert_eq!(metrics.active_instances, 0);
    assert_eq!(metrics.idle_instances, 0);

    // Verify quality metrics
    assert_eq!(metrics.avg_content_quality_score, 0.0);
    assert_eq!(metrics.avg_extracted_word_count, 0.0);
    assert_eq!(metrics.cache_hit_ratio, 0.0);

    // Verify error metrics
    assert_eq!(metrics.error_rate, 0.0);
    assert_eq!(metrics.timeout_rate, 0.0);
    assert_eq!(metrics.circuit_breaker_trips, 0);

    // Verify system health
    assert_eq!(metrics.health_score, 100.0);
    assert_eq!(metrics.uptime_seconds, 0);
}

#[tokio::test]
async fn test_concurrent_metrics_collection() {
    let collector = Arc::new(MetricsCollector::new());
    let barrier = Arc::new(Barrier::new(5));
    let mut handles = vec![];

    // Spawn multiple tasks that record metrics concurrently
    for thread_id in 0..5 {
        let collector = Arc::clone(&collector);
        let barrier = Arc::clone(&barrier);

        let handle = tokio::spawn(async move {
            barrier.wait();

            for i in 0..20 {
                let duration = Duration::from_millis(50 + (thread_id * 10 + i) as u64);
                let success = (thread_id + i) % 3 != 0;
                let quality = if success { Some(80 + (i % 20) as u8) } else { None };
                let word_count = if success { Some(300 + (i * 20) as u32) } else { None };
                let cached = i % 2 == 0;

                collector.record_extraction(duration, success, quality, word_count, cached).await;

                if !success {
                    collector.record_error("concurrent_error", i % 4 == 0).await;
                }

                // Small delay to increase chance of interleaving
                if i % 5 == 0 {
                    sleep(Duration::from_millis(1)).await;
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("Task should complete successfully");
    }

    // Verify final metrics are consistent
    let metrics = collector.get_current_metrics().await;
    assert_eq!(metrics.total_extractions, 100, "Should have recorded exactly 100 extractions");

    // Verify the metrics are reasonable
    assert!(metrics.successful_extractions > 0);
    assert!(metrics.failed_extractions > 0);
    assert!(metrics.successful_extractions + metrics.failed_extractions == metrics.total_extractions);
    assert!(metrics.avg_content_quality_score >= 0.0);
    assert!(metrics.cache_hit_ratio >= 0.0 && metrics.cache_hit_ratio <= 1.0);
}

#[tokio::test]
async fn test_memory_leak_prevention() {
    let collector = Arc::new(MetricsCollector::new());

    // Record a large number of metrics to test memory management
    for i in 0..10000 {
        collector.record_extraction(
            Duration::from_millis(100),
            true,
            Some(80),
            Some(500),
            false
        ).await;

        // Periodically check that memory isn't growing unbounded
        if i % 1000 == 0 {
            let metrics = collector.get_current_metrics().await;
            // This is a basic check - in a real scenario you'd monitor actual memory usage
            assert!(metrics.total_extractions <= 10000);
        }
    }

    // Verify that the time series buffers respect their size limits
    let report = collector.get_performance_report(Duration::from_secs(3600)).await;

    // The buffers should not contain more than max_data_points entries
    // This is an indirect test since the TimeSeriesBuffer internals aren't exposed
    assert!(report.avg_extraction_time > 0.0);
    assert!(report.p95_extraction_time >= report.avg_extraction_time);
}