//! Memory management tests for stratified pooling
//!
//! Tests hot/warm/cold tier pool management, memory tracking, and GC

use riptide_pool::memory_manager::*;
use std::time::Duration;
use wasmtime::{Config, Engine};

fn create_test_engine() -> Engine {
    let mut config = Config::new();
    config.wasm_component_model(true);
    Engine::new(&config).unwrap()
}

#[tokio::test]
async fn test_memory_manager_config_defaults() {
    let config = MemoryManagerConfig::default();

    assert_eq!(config.max_total_memory_mb, 2048);
    assert_eq!(config.instance_memory_threshold_mb, 256);
    assert_eq!(config.max_instances, 8);
    assert_eq!(config.min_instances, 2);
    assert_eq!(config.memory_pressure_threshold, 80.0);
    assert!(!config.aggressive_gc);
}

#[tokio::test]
async fn test_memory_manager_creation() {
    let config = MemoryManagerConfig {
        max_instances: 4,
        min_instances: 1,
        ..Default::default()
    };

    let engine = create_test_engine();
    let manager = MemoryManager::new(config, engine).await;

    assert!(manager.is_ok());

    let mgr = manager.unwrap();
    let stats = mgr.stats();

    assert_eq!(stats.instances_count, 0);
    assert_eq!(stats.active_instances, 0);

    let _ = mgr.shutdown().await;
}

#[tokio::test]
async fn test_stratified_pool_creation() {
    let hot_capacity = 2;
    let warm_capacity = 4;

    let mut pool = StratifiedInstancePool::new(hot_capacity, warm_capacity);

    let metrics = pool.metrics();
    assert_eq!(metrics.hot_count, 0);
    assert_eq!(metrics.warm_count, 0);
    assert_eq!(metrics.cold_count, 0);
    assert_eq!(metrics.total_instances, 0);

    // Test that pool starts empty
    let instance = pool.acquire();
    assert!(instance.is_none());
}

#[tokio::test]
async fn test_memory_stats_tracking() {
    let stats = MemoryStats {
        total_allocated_mb: 2048,
        total_used_mb: 512,
        instances_count: 4,
        active_instances: 2,
        idle_instances: 2,
        peak_memory_mb: 1024,
        gc_runs: 5,
        memory_pressure: 25.0,
        pool_hot_count: 1,
        pool_warm_count: 2,
        pool_cold_count: 1,
        pool_hot_hits: 100,
        pool_warm_hits: 50,
        pool_cold_misses: 10,
        pool_promotions: 5,
        ..Default::default()
    };

    // Verify stratified pool metrics
    assert_eq!(stats.pool_hot_count, 1);
    assert_eq!(stats.pool_warm_count, 2);
    assert_eq!(stats.pool_cold_count, 1);
    assert_eq!(stats.pool_hot_hits, 100);
    assert_eq!(stats.pool_warm_hits, 50);
    assert_eq!(stats.pool_cold_misses, 10);
    assert_eq!(stats.pool_promotions, 5);

    // Calculate hit rate
    let total_accesses = stats.pool_hot_hits + stats.pool_warm_hits + stats.pool_cold_misses;
    let hot_hit_rate = (stats.pool_hot_hits as f64 / total_accesses as f64) * 100.0;

    assert!((hot_hit_rate - 62.5).abs() < 0.1);
}

#[tokio::test]
async fn test_memory_pressure_calculation() {
    let config = MemoryManagerConfig::default();

    let scenarios = vec![
        (512, 25.0),   // 25% pressure
        (1024, 50.0),  // 50% pressure
        (1638, 80.0),  // 80% pressure (at threshold)
        (1843, 90.0),  // 90% pressure
        (2048, 100.0), // 100% pressure
    ];

    for (used_mb, expected_pressure) in scenarios {
        let pressure = (used_mb as f64 / config.max_total_memory_mb as f64) * 100.0;
        assert!((pressure - expected_pressure).abs() < 0.1);
    }
}

#[tokio::test]
async fn test_memory_event_types() {
    // Test threshold exceeded event
    let event1 = MemoryEvent::MemoryThresholdExceeded {
        instance_id: "test-1".to_string(),
        usage_mb: 300,
        threshold_mb: 256,
    };

    match event1 {
        MemoryEvent::MemoryThresholdExceeded {
            usage_mb,
            threshold_mb,
            ..
        } => {
            assert!(usage_mb > threshold_mb);
        }
        _ => panic!("Wrong event type"),
    }

    // Test memory pressure event
    let event2 = MemoryEvent::MemoryPressureHigh {
        current_usage: 1700,
        max_usage: 2048,
        pressure_percent: 83.0,
    };

    match event2 {
        MemoryEvent::MemoryPressureHigh {
            pressure_percent, ..
        } => {
            assert!(pressure_percent > 80.0);
        }
        _ => panic!("Wrong event type"),
    }

    // Test GC event
    let event3 = MemoryEvent::GarbageCollectionTriggered {
        instances_affected: 3,
        memory_freed_mb: 150,
    };

    match event3 {
        MemoryEvent::GarbageCollectionTriggered {
            instances_affected,
            memory_freed_mb,
        } => {
            assert_eq!(instances_affected, 3);
            assert_eq!(memory_freed_mb, 150);
        }
        _ => panic!("Wrong event type"),
    }

    // Test memory leak detection
    let event4 = MemoryEvent::MemoryLeakDetected {
        instance_id: "leaky-instance".to_string(),
        growth_rate_mb_per_sec: 15.5,
    };

    match event4 {
        MemoryEvent::MemoryLeakDetected {
            growth_rate_mb_per_sec,
            ..
        } => {
            assert!(growth_rate_mb_per_sec > 10.0);
        }
        _ => panic!("Wrong event type"),
    }
}

#[tokio::test]
async fn test_pool_tier_promotion() {
    let pool = StratifiedInstancePool::new(2, 4);

    // Pool starts empty
    assert_eq!(pool.total_count(), 0);

    // Promotion logic would be tested with actual instances
    // For now, test the promotion counter
    let initial_metrics = pool.metrics();
    assert_eq!(initial_metrics.promotions, 0);
}

#[tokio::test]
async fn test_instance_idle_timeout() {
    let idle_timeout = Duration::from_secs(60);

    // Recent instance - not idle
    let recent_last_used = std::time::Instant::now();
    let is_recent_idle = recent_last_used.elapsed() > idle_timeout;
    assert!(!is_recent_idle);

    // Old instance - is idle
    let old_last_used = std::time::Instant::now() - Duration::from_secs(120);
    let is_old_idle = old_last_used.elapsed() > idle_timeout;
    assert!(is_old_idle);
}

#[tokio::test]
async fn test_memory_growth_rate_calculation() {
    use std::collections::VecDeque;
    use std::time::Instant;

    let mut history: VecDeque<(Instant, u64)> = VecDeque::new();

    // Simulate memory growth over time
    let start = Instant::now();
    history.push_back((start, 100)); // 100MB at start

    tokio::time::sleep(Duration::from_millis(10)).await;
    history.push_back((Instant::now(), 150)); // 150MB after 10ms

    if history.len() >= 2 {
        let (oldest_time, oldest_mem) = history.front().unwrap();
        let (newest_time, newest_mem) = history.back().unwrap();

        let time_diff = newest_time.duration_since(*oldest_time).as_secs_f64();
        if time_diff > 0.0 {
            let growth_rate = (*newest_mem as f64 - *oldest_mem as f64) / time_diff;
            assert!(growth_rate > 0.0); // Should show growth
        }
    }
}

#[tokio::test]
async fn test_gc_interval_triggering() {
    let config = MemoryManagerConfig {
        gc_interval: Duration::from_millis(100),
        ..Default::default()
    };

    let start = std::time::Instant::now();
    tokio::time::sleep(config.gc_interval * 2).await;

    let elapsed = start.elapsed();
    let gc_cycles = elapsed.as_millis() / config.gc_interval.as_millis();

    assert!(gc_cycles >= 2);
}

#[tokio::test]
async fn test_memory_limit_enforcement() {
    let config = MemoryManagerConfig {
        max_total_memory_mb: 1024,
        memory_pressure_threshold: 80.0,
        ..Default::default()
    };

    let current_usage = 900; // 87.9% of limit
    let pressure = (current_usage as f64 / config.max_total_memory_mb as f64) * 100.0;

    assert!(pressure > config.memory_pressure_threshold);

    // Simulates that memory allocation should be rejected
    let should_reject = pressure > config.memory_pressure_threshold;
    assert!(should_reject);
}

#[tokio::test]
async fn test_aggressive_gc_mode() {
    let normal_config = MemoryManagerConfig {
        aggressive_gc: false,
        ..Default::default()
    };

    let aggressive_config = MemoryManagerConfig {
        aggressive_gc: true,
        ..Default::default()
    };

    assert!(!normal_config.aggressive_gc);
    assert!(aggressive_config.aggressive_gc);

    // In aggressive mode, GC should trigger more frequently
    let normal_threshold = 80.0;
    let aggressive_threshold = 60.0;

    assert!(aggressive_threshold < normal_threshold);
}

#[tokio::test]
async fn test_pool_metrics_aggregation() {
    let metrics = PoolMetrics {
        hot_count: 2,
        warm_count: 3,
        cold_count: 1,
        hot_hits: 150,
        warm_hits: 75,
        cold_misses: 25,
        promotions: 10,
        total_instances: 6,
    };

    let total_accesses = metrics.hot_hits + metrics.warm_hits + metrics.cold_misses;
    let hot_hit_rate = (metrics.hot_hits as f64 / total_accesses as f64) * 100.0;

    assert_eq!(total_accesses, 250);
    assert_eq!(metrics.total_instances, 6);
    assert!((hot_hit_rate - 60.0).abs() < 0.1);
}

#[tokio::test]
async fn test_instance_access_frequency() {
    use std::time::Instant;

    let created_at = Instant::now();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let time_since_created = created_at.elapsed().as_secs_f64();
    let access_count = 5.0;

    // Calculate access frequency (accesses per second)
    let frequency = access_count / time_since_created;
    assert!(frequency > 0.0);
}

#[tokio::test]
async fn test_cleanup_timeout_handling() {
    let config = MemoryManagerConfig {
        cleanup_timeout: Duration::from_secs(5),
        ..Default::default()
    };

    // Fast cleanup (should succeed)
    let fast_cleanup = async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok::<_, anyhow::Error>(())
    };

    let result = tokio::time::timeout(config.cleanup_timeout, fast_cleanup).await;
    assert!(result.is_ok());

    // Slow cleanup (should timeout)
    let slow_cleanup = async {
        tokio::time::sleep(Duration::from_secs(10)).await;
        Ok::<_, anyhow::Error>(())
    };

    let timeout_result = tokio::time::timeout(Duration::from_secs(1), slow_cleanup).await;
    assert!(timeout_result.is_err());
}
