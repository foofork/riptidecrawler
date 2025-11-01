//! LRU Eviction Tracking Tests
//!
//! Comprehensive tests for cache eviction tracking functionality

use riptide_persistence::metrics::{CacheMetrics, EvictionReason};
use std::time::Duration;
use tokio::time::sleep;

#[cfg(feature = "metrics")]
#[tokio::test]
async fn test_eviction_tracking_basic() {
    let metrics = CacheMetrics::new().expect("Failed to create metrics");

    // Record an eviction
    metrics
        .record_eviction(EvictionReason::LruCapacity, 1024, Some(60))
        .await;

    // Get eviction stats
    let stats = metrics.get_eviction_stats().await;

    assert_eq!(stats.total_evictions, 1);
    assert_eq!(stats.total_evicted_bytes, 1024);
    assert_eq!(stats.avg_time_since_access_seconds, 60);
    assert_eq!(
        *stats
            .evictions_by_reason
            .get(&EvictionReason::LruCapacity)
            .unwrap_or(&0),
        1
    );
}

#[cfg(feature = "metrics")]
#[tokio::test]
async fn test_multiple_evictions() {
    let metrics = CacheMetrics::new().expect("Failed to create metrics");

    // Record multiple evictions with different reasons
    metrics
        .record_eviction(EvictionReason::LruCapacity, 2048, Some(120))
        .await;
    metrics
        .record_eviction(EvictionReason::TtlExpired, 1024, Some(300))
        .await;
    metrics
        .record_eviction(EvictionReason::MemoryPressure, 4096, Some(60))
        .await;
    metrics
        .record_eviction(EvictionReason::Manual, 512, None)
        .await;

    let stats = metrics.get_eviction_stats().await;

    assert_eq!(stats.total_evictions, 4);
    assert_eq!(stats.total_evicted_bytes, 2048 + 1024 + 4096 + 512);
    assert_eq!(
        *stats
            .evictions_by_reason
            .get(&EvictionReason::LruCapacity)
            .unwrap(),
        1
    );
    assert_eq!(
        *stats
            .evictions_by_reason
            .get(&EvictionReason::TtlExpired)
            .unwrap(),
        1
    );
    assert_eq!(
        *stats
            .evictions_by_reason
            .get(&EvictionReason::MemoryPressure)
            .unwrap(),
        1
    );
    assert_eq!(
        *stats
            .evictions_by_reason
            .get(&EvictionReason::Manual)
            .unwrap(),
        1
    );
}

#[cfg(feature = "metrics")]
#[tokio::test]
async fn test_eviction_rate_calculation() {
    let metrics = CacheMetrics::new().expect("Failed to create metrics");

    // Wait a second to ensure time difference for rate calculation
    sleep(Duration::from_secs(1)).await;

    // Record several evictions
    for i in 0..10 {
        metrics
            .record_eviction(EvictionReason::LruCapacity, 1024, Some(i * 10))
            .await;
    }

    let stats = metrics.get_eviction_stats().await;

    assert_eq!(stats.total_evictions, 10);
    // Eviction rate should be > 0 since time has passed
    assert!(stats.eviction_rate > 0.0);
}

#[cfg(feature = "metrics")]
#[tokio::test]
async fn test_recent_evictions_tracking() {
    let metrics = CacheMetrics::new().expect("Failed to create metrics");

    // Record 15 evictions (more than the 10 we keep)
    for i in 0..15 {
        metrics
            .record_eviction(EvictionReason::LruCapacity, 1024 * (i + 1), Some(i as u64))
            .await;
    }

    let stats = metrics.get_eviction_stats().await;

    // Should keep only the last 10
    assert_eq!(stats.recent_evictions.len(), 10);

    // Most recent should be last
    assert_eq!(stats.recent_evictions[0].entry_size, 1024 * 15);
}

#[cfg(feature = "metrics")]
#[tokio::test]
async fn test_eviction_stats_summary_integration() {
    let metrics = CacheMetrics::new().expect("Failed to create metrics");

    // Record some operations
    metrics.record_hit(Duration::from_micros(100)).await;
    metrics.record_miss().await;

    // Record evictions
    metrics
        .record_eviction(EvictionReason::LruCapacity, 2048, Some(60))
        .await;
    metrics
        .record_eviction(EvictionReason::TtlExpired, 1024, Some(120))
        .await;

    let summary = metrics.get_current_stats().await;

    assert_eq!(summary.eviction_count, 2);
    assert!(summary.eviction_rate >= 0.0);
    assert_eq!(summary.total_operations, 2); // 1 hit + 1 miss
}

#[cfg(feature = "metrics")]
#[tokio::test]
async fn test_avg_time_since_access() {
    let metrics = CacheMetrics::new().expect("Failed to create metrics");

    // Record evictions with known access times
    metrics
        .record_eviction(EvictionReason::LruCapacity, 1024, Some(100))
        .await;
    metrics
        .record_eviction(EvictionReason::LruCapacity, 1024, Some(200))
        .await;
    metrics
        .record_eviction(EvictionReason::LruCapacity, 1024, Some(300))
        .await;

    let stats = metrics.get_eviction_stats().await;

    // Average should be (100 + 200 + 300) / 3 = 200
    assert_eq!(stats.avg_time_since_access_seconds, 200);
}

#[cfg(feature = "metrics")]
#[tokio::test]
async fn test_eviction_with_no_access_time() {
    let metrics = CacheMetrics::new().expect("Failed to create metrics");

    // Record evictions without access time (e.g., manual deletions)
    metrics
        .record_eviction(EvictionReason::Manual, 1024, None)
        .await;
    metrics
        .record_eviction(EvictionReason::Manual, 2048, None)
        .await;

    let stats = metrics.get_eviction_stats().await;

    assert_eq!(stats.total_evictions, 2);
    assert_eq!(stats.total_evicted_bytes, 3072);
    // Average should be 0 when no access times recorded
    assert_eq!(stats.avg_time_since_access_seconds, 0);
}

#[cfg(feature = "metrics")]
#[tokio::test]
async fn test_eviction_counter_prometheus_integration() {
    use prometheus::Registry;

    let metrics = CacheMetrics::new().expect("Failed to create metrics");
    let registry = Registry::new();

    metrics
        .register(&registry)
        .expect("Failed to register metrics");

    // Record evictions
    metrics
        .record_eviction(EvictionReason::LruCapacity, 1024, Some(60))
        .await;
    metrics
        .record_eviction(EvictionReason::LruCapacity, 2048, Some(120))
        .await;

    // Verify counter is incremented
    let metric_families = registry.gather();
    let eviction_metric = metric_families
        .iter()
        .find(|mf| mf.get_name() == "riptide_cache_evictions_total");

    assert!(eviction_metric.is_some());
}

#[cfg(feature = "metrics")]
#[tokio::test]
async fn test_stats_reset_clears_evictions() {
    let metrics = CacheMetrics::new().expect("Failed to create metrics");

    // Record some evictions
    metrics
        .record_eviction(EvictionReason::LruCapacity, 1024, Some(60))
        .await;
    metrics
        .record_eviction(EvictionReason::TtlExpired, 2048, Some(120))
        .await;

    let stats_before = metrics.get_eviction_stats().await;
    assert_eq!(stats_before.total_evictions, 2);

    // Reset stats
    metrics.reset_stats().await;

    let stats_after = metrics.get_eviction_stats().await;
    assert_eq!(stats_after.total_evictions, 0);
    assert_eq!(stats_after.total_evicted_bytes, 0);
    assert!(stats_after.evictions_by_reason.is_empty());
    assert!(stats_after.recent_evictions.is_empty());
}

#[cfg(feature = "metrics")]
#[tokio::test]
async fn test_memory_pressure_evictions() {
    let metrics = CacheMetrics::new().expect("Failed to create metrics");

    // Simulate memory pressure scenario
    for i in 0..20 {
        metrics
            .record_eviction(
                EvictionReason::MemoryPressure,
                4096 * (i + 1),
                Some(i as u64 * 5),
            )
            .await;
    }

    let stats = metrics.get_eviction_stats().await;

    assert_eq!(stats.total_evictions, 20);
    assert_eq!(
        *stats
            .evictions_by_reason
            .get(&EvictionReason::MemoryPressure)
            .unwrap(),
        20
    );

    // Total bytes should be sum of arithmetic sequence: 4096 * (1 + 2 + ... + 20) = 4096 * 210
    assert_eq!(stats.total_evicted_bytes, 4096 * 210);
}

#[cfg(feature = "metrics")]
#[tokio::test]
async fn test_mixed_eviction_reasons() {
    let metrics = CacheMetrics::new().expect("Failed to create metrics");

    // Create a realistic mix of eviction reasons
    for _ in 0..5 {
        metrics
            .record_eviction(EvictionReason::LruCapacity, 1024, Some(300))
            .await;
    }
    for _ in 0..3 {
        metrics
            .record_eviction(EvictionReason::TtlExpired, 2048, Some(600))
            .await;
    }
    for _ in 0..2 {
        metrics
            .record_eviction(EvictionReason::MemoryPressure, 4096, Some(150))
            .await;
    }
    metrics
        .record_eviction(EvictionReason::Manual, 512, None)
        .await;

    let stats = metrics.get_eviction_stats().await;

    assert_eq!(stats.total_evictions, 11);
    assert_eq!(
        *stats
            .evictions_by_reason
            .get(&EvictionReason::LruCapacity)
            .unwrap(),
        5
    );
    assert_eq!(
        *stats
            .evictions_by_reason
            .get(&EvictionReason::TtlExpired)
            .unwrap(),
        3
    );
    assert_eq!(
        *stats
            .evictions_by_reason
            .get(&EvictionReason::MemoryPressure)
            .unwrap(),
        2
    );
    assert_eq!(
        *stats
            .evictions_by_reason
            .get(&EvictionReason::Manual)
            .unwrap(),
        1
    );
}

#[cfg(feature = "metrics")]
#[tokio::test]
async fn test_eviction_entry_fields() {
    let metrics = CacheMetrics::new().expect("Failed to create metrics");

    metrics
        .record_eviction(EvictionReason::LruCapacity, 8192, Some(240))
        .await;

    let stats = metrics.get_eviction_stats().await;

    assert_eq!(stats.recent_evictions.len(), 1);
    let entry = &stats.recent_evictions[0];

    assert_eq!(entry.reason, EvictionReason::LruCapacity);
    assert_eq!(entry.entry_size, 8192);
    assert_eq!(entry.time_since_access, Some(240));
    // evicted_at should be recent (within last second)
    let now = chrono::Utc::now();
    let age = now.signed_duration_since(entry.evicted_at);
    assert!(age.num_seconds() < 2);
}
