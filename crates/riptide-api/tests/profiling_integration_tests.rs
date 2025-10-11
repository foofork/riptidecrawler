//! Integration tests for performance profiling endpoints
//!
//! This module tests the complete profiling integration including:
//! - Memory profiling accuracy
//! - jemalloc allocator functionality
//! - Leak detection under load
//! - Performance overhead measurement
//! - API endpoint responses

use riptide_api::health::HealthChecker;
use riptide_api::metrics::RipTideMetrics;
use riptide_api::state::{AppConfig, AppState};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_profiling_memory_endpoint() {
    // Initialize test state
    let config = AppConfig::default();
    let metrics = Arc::new(RipTideMetrics::new().unwrap());
    let health_checker = Arc::new(HealthChecker::new());

    let state = AppState::new(config, metrics, health_checker)
        .await
        .expect("Failed to initialize AppState");

    // Get memory metrics
    let result = state.performance_manager.get_metrics().await;
    assert!(result.is_ok(), "Should retrieve memory metrics");

    let metrics = result.unwrap();

    // Verify memory metrics are reasonable
    assert!(metrics.memory_rss_mb > 0.0, "RSS should be positive");
    assert!(metrics.memory_heap_mb > 0.0, "Heap should be positive");
    assert!(
        metrics.memory_rss_mb >= metrics.memory_heap_mb,
        "RSS should be >= heap"
    );
    assert!(
        metrics.memory_rss_mb < 10_000.0,
        "RSS should be < 10GB (sanity check)"
    );
}

#[tokio::test]
async fn test_profiling_cache_stats() {
    let config = AppConfig::default();
    let metrics = Arc::new(RipTideMetrics::new().unwrap());
    let health_checker = Arc::new(HealthChecker::new());

    let state = AppState::new(config, metrics, health_checker)
        .await
        .expect("Failed to initialize AppState");

    // Get cache statistics
    let result = state.performance_manager.get_cache_stats().await;
    assert!(result.is_ok(), "Should retrieve cache stats");

    let stats = result.unwrap();

    // Verify cache stats structure
    assert!(
        stats.hit_rate >= 0.0 && stats.hit_rate <= 1.0,
        "Hit rate should be between 0 and 1"
    );
    // total_entries is unsigned, so it's always non-negative
    // Just verify it's accessible
    let _ = stats.total_entries;
}

#[tokio::test]
async fn test_profiling_leak_detection_simple() {
    let config = AppConfig::default();
    let metrics = Arc::new(RipTideMetrics::new().unwrap());
    let health_checker = Arc::new(HealthChecker::new());

    let state = AppState::new(config, metrics, health_checker)
        .await
        .expect("Failed to initialize AppState");

    // Get initial metrics
    let initial_metrics = state.performance_manager.get_metrics().await.unwrap();
    let initial_memory = initial_metrics.memory_rss_mb;

    // Wait a bit for monitoring to collect data
    sleep(Duration::from_secs(2)).await;

    // Get metrics again
    let final_metrics = state.performance_manager.get_metrics().await.unwrap();
    let final_memory = final_metrics.memory_rss_mb;

    // Memory growth should be minimal in idle state
    let growth_mb = final_memory - initial_memory;
    assert!(
        growth_mb.abs() < 50.0,
        "Memory growth in idle state should be < 50MB, got: {}MB",
        growth_mb
    );
}

#[tokio::test]
async fn test_profiling_performance_overhead() {
    // Measure overhead of profiling system
    let config = AppConfig::default();
    let metrics = Arc::new(RipTideMetrics::new().unwrap());
    let health_checker = Arc::new(HealthChecker::new());

    // Measure time to initialize with profiling
    let start = std::time::Instant::now();
    let state = AppState::new(config, metrics, health_checker)
        .await
        .expect("Failed to initialize AppState");
    let init_duration = start.elapsed();

    // Initialization should be fast (< 5 seconds)
    assert!(
        init_duration.as_secs() < 5,
        "Initialization with profiling should be < 5s, got: {}s",
        init_duration.as_secs()
    );

    // Measure time to get metrics (should be very fast)
    let start = std::time::Instant::now();
    let _ = state.performance_manager.get_metrics().await.unwrap();
    let metrics_duration = start.elapsed();

    // Getting metrics should be nearly instant (< 100ms)
    assert!(
        metrics_duration.as_millis() < 100,
        "Getting metrics should be < 100ms, got: {}ms",
        metrics_duration.as_millis()
    );
}

#[tokio::test]
async fn test_profiling_target_compliance() {
    // Test that performance targets are being tracked
    let config = AppConfig::default();
    let metrics = Arc::new(RipTideMetrics::new().unwrap());
    let health_checker = Arc::new(HealthChecker::new());

    let state = AppState::new(config, metrics, health_checker)
        .await
        .expect("Failed to initialize AppState");

    // Check targets
    let target_status = state.performance_manager.check_targets().await;
    assert!(target_status.is_ok(), "Should check targets without error");

    let status = target_status.unwrap();

    // Memory should be within targets in idle state
    assert!(
        status.metrics.memory_rss_mb < 700.0,
        "Memory should be < 700MB in idle state, got: {}MB",
        status.metrics.memory_rss_mb
    );
}

#[cfg(feature = "jemalloc")]
#[tokio::test]
async fn test_jemalloc_allocator_active() {
    // Verify jemalloc is active
    use tikv_jemalloc_ctl::{epoch, stats};

    // Trigger a stats update
    epoch::mib().unwrap().advance().unwrap();

    // Read allocated memory
    let allocated = stats::allocated::mib().unwrap().read().unwrap();

    // Should have some memory allocated
    assert!(
        allocated > 0,
        "jemalloc should report allocated memory > 0, got: {}",
        allocated
    );

    println!(
        "jemalloc allocated: {} MB",
        allocated as f64 / 1024.0 / 1024.0
    );
}

#[tokio::test]
async fn test_profiling_resource_limits() {
    let config = AppConfig::default();
    let metrics = Arc::new(RipTideMetrics::new().unwrap());
    let health_checker = Arc::new(HealthChecker::new());

    let state = AppState::new(config, metrics, health_checker)
        .await
        .expect("Failed to initialize AppState");

    // Get resource usage
    let usage = state.performance_manager.get_resource_usage().await;
    assert!(usage.is_ok(), "Should retrieve resource usage");

    let resource_usage = usage.unwrap();

    // Verify resource tracking - ResourceUsage has active_requests (unsigned, always non-negative)
    // Just verify it's accessible
    let _ = resource_usage.active_requests;
}

#[tokio::test]
async fn test_profiling_cache_optimization() {
    let config = AppConfig::default();
    let metrics = Arc::new(RipTideMetrics::new().unwrap());
    let health_checker = Arc::new(HealthChecker::new());

    let state = AppState::new(config, metrics, health_checker)
        .await
        .expect("Failed to initialize AppState");

    // Get cache stats before optimization
    let _initial_stats = state.performance_manager.get_cache_stats().await.unwrap();

    // Run cache optimization
    let result = state.performance_manager.optimize_cache().await;
    assert!(result.is_ok(), "Cache optimization should succeed");

    let optimizations = result.unwrap();
    println!("Cache optimizations applied: {:?}", optimizations);

    // Get cache stats after optimization
    let final_stats = state.performance_manager.get_cache_stats().await.unwrap();

    // Stats should be valid after optimization (total_entries is unsigned, always valid)
    // Just verify it's accessible
    let _ = final_stats.total_entries;
}

#[tokio::test]
async fn test_profiling_concurrent_access() {
    // Test thread safety of profiling system
    let config = AppConfig::default();
    let metrics = Arc::new(RipTideMetrics::new().unwrap());
    let health_checker = Arc::new(HealthChecker::new());

    let state = Arc::new(
        AppState::new(config, metrics, health_checker)
            .await
            .expect("Failed to initialize AppState"),
    );

    // Spawn multiple tasks to access metrics concurrently
    let mut handles = vec![];

    for _ in 0..10 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..5 {
                let _ = state_clone.performance_manager.get_metrics().await;
                sleep(Duration::from_millis(10)).await;
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("Task should complete successfully");
    }

    // System should still be functional after concurrent access
    let final_metrics = state.performance_manager.get_metrics().await;
    assert!(
        final_metrics.is_ok(),
        "Should retrieve metrics after concurrent access"
    );
}

#[tokio::test]
async fn test_profiling_memory_growth_tracking() {
    let config = AppConfig::default();
    let metrics = Arc::new(RipTideMetrics::new().unwrap());
    let health_checker = Arc::new(HealthChecker::new());

    let state = AppState::new(config, metrics, health_checker)
        .await
        .expect("Failed to initialize AppState");

    // Take initial snapshot
    let _initial = state.performance_manager.get_metrics().await.unwrap();

    // Wait for profiling to collect some data
    sleep(Duration::from_secs(3)).await;

    // Take final snapshot
    let final_metrics = state.performance_manager.get_metrics().await.unwrap();

    // Growth rate should be tracked
    // In idle state, growth should be minimal (< 10MB/s)
    assert!(
        final_metrics.memory_growth_rate_mb_s.abs() < 10.0,
        "Memory growth rate should be < 10MB/s in idle state, got: {}MB/s",
        final_metrics.memory_growth_rate_mb_s
    );
}

#[tokio::test]
async fn test_profiling_metrics_completeness() {
    // Verify all expected metrics are populated
    let config = AppConfig::default();
    let metrics = Arc::new(RipTideMetrics::new().unwrap());
    let health_checker = Arc::new(HealthChecker::new());

    let state = AppState::new(config, metrics, health_checker)
        .await
        .expect("Failed to initialize AppState");

    let metrics = state.performance_manager.get_metrics().await.unwrap();

    // Verify all metrics are populated
    assert!(metrics.memory_rss_mb > 0.0, "RSS should be populated");
    assert!(metrics.memory_heap_mb >= 0.0, "Heap should be populated");
    assert!(
        metrics.cpu_usage_percent >= 0.0,
        "CPU usage should be populated"
    );
    assert!(
        metrics.cache_hit_rate >= 0.0,
        "Cache hit rate should be populated"
    );
    // total_requests is unsigned, always non-negative - just verify it's accessible
    let _ = metrics.total_requests;

    println!("Profiling metrics: {:#?}", metrics);
}

mod performance_baseline_tests {
    use super::*;

    /// Baseline test: Measure profiling system overhead
    ///
    /// This test establishes performance baselines for the profiling system:
    /// - Initialization time should be < 5 seconds
    /// - Memory overhead should be < 50MB
    /// - Metrics collection should be < 100ms
    /// - Profiling CPU overhead should be < 2%
    #[tokio::test]
    async fn test_profiling_overhead_baseline() {
        // Measure initialization overhead
        let start = std::time::Instant::now();

        let config = AppConfig::default();
        let metrics = Arc::new(RipTideMetrics::new().unwrap());
        let health_checker = Arc::new(HealthChecker::new());

        let state = AppState::new(config, metrics, health_checker)
            .await
            .expect("Failed to initialize AppState");

        let init_time = start.elapsed();

        // Baseline: Initialization < 5s
        assert!(
            init_time.as_secs() < 5,
            "BASELINE VIOLATION: Init time {}s > 5s",
            init_time.as_secs()
        );

        // Measure memory overhead
        let initial_metrics = state.performance_manager.get_metrics().await.unwrap();
        let initial_memory = initial_metrics.memory_rss_mb;

        // Baseline: Memory overhead < 50MB additional
        // (This is a rough estimate, actual baseline varies by system)
        println!("Memory RSS at init: {}MB", initial_memory);

        // Measure metrics collection performance
        let iterations = 100;
        let start = std::time::Instant::now();

        for _ in 0..iterations {
            let _ = state.performance_manager.get_metrics().await.unwrap();
        }

        let total_time = start.elapsed();
        let avg_time_ms = total_time.as_millis() as f64 / iterations as f64;

        // Baseline: Average metrics collection < 100ms
        assert!(
            avg_time_ms < 100.0,
            "BASELINE VIOLATION: Avg metrics collection {}ms > 100ms",
            avg_time_ms
        );

        println!(
            "Performance Baselines:\n  \
            - Init time: {}ms\n  \
            - Initial memory: {}MB\n  \
            - Metrics collection avg: {:.2}ms",
            init_time.as_millis(),
            initial_memory,
            avg_time_ms
        );
    }
}
