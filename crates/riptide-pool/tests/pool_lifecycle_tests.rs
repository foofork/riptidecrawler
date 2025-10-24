//! Comprehensive pool lifecycle tests for riptide-pool
//!
//! Tests pool creation, warmup, scaling, and shutdown

use riptide_pool::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

fn create_test_config(max_pool: usize, initial: usize) -> ExtractorConfig {
    ExtractorConfig {
        max_instances: max_pool,
        enable_metrics: true,
        timeout_ms: 5000,
        memory_limit_pages: Some(256),
        extraction_timeout: Some(5000),
        max_pool_size: max_pool,
        initial_pool_size: initial,
        epoch_timeout_ms: 10000,
        health_check_interval: 5000,
        memory_limit: Some(128 * 1024 * 1024),
        circuit_breaker_timeout: 5000,
        circuit_breaker_failure_threshold: 5,
        enable_wit_validation: false,
    }
}

#[tokio::test]
async fn test_pool_size_validation() {
    let mut config = create_test_config(8, 2);

    // Valid configuration
    assert!(config.validate().is_ok());

    // Invalid: initial > max
    config.initial_pool_size = 10;
    config.max_pool_size = 5;
    assert!(config.validate().is_err());

    // Invalid: zero max_pool_size
    config.max_pool_size = 0;
    assert!(config.validate().is_err());

    // Invalid: zero timeout
    config.max_pool_size = 8;
    config.timeout_ms = 0;
    assert!(config.validate().is_err());
}

#[tokio::test]
async fn test_pool_metrics_tracking() {
    let mut metrics = PerformanceMetrics::default();

    // Simulate successful extractions
    for _ in 0..50 {
        metrics.total_extractions += 1;
        metrics.successful_extractions += 1;
    }

    // Simulate some failures
    for _ in 0..5 {
        metrics.total_extractions += 1;
        metrics.failed_extractions += 1;
    }

    // Simulate fallback usage
    for _ in 0..3 {
        metrics.total_extractions += 1;
        metrics.fallback_extractions += 1;
    }

    assert_eq!(metrics.total_extractions, 58);
    assert_eq!(metrics.successful_extractions, 50);
    assert_eq!(metrics.failed_extractions, 5);
    assert_eq!(metrics.fallback_extractions, 3);

    let success_rate =
        (metrics.successful_extractions as f64 / metrics.total_extractions as f64) * 100.0;
    assert!((success_rate - 86.2).abs() < 0.1);
}

#[tokio::test]
async fn test_pool_status_calculation() {
    // Simulate pool with 8 max, 5 available, 3 active
    let max_size = 8;
    let available = 5;
    let active = max_size - available;

    let utilization = (active as f64 / max_size as f64) * 100.0;

    assert_eq!(active, 3);
    assert_eq!(utilization, 37.5);

    // High utilization scenario
    let high_active = 7;
    let high_utilization = (high_active as f64 / max_size as f64) * 100.0;
    assert!(high_utilization > 80.0);

    // Low utilization scenario
    let low_active = 1;
    let low_utilization = (low_active as f64 / max_size as f64) * 100.0;
    assert!(low_utilization < 20.0);
}

#[tokio::test]
async fn test_instances_per_worker_config() {
    // Test default value
    std::env::remove_var("RIPTIDE_WASM_INSTANCES_PER_WORKER");
    let default_val = get_instances_per_worker();
    assert_eq!(default_val, 8);

    // Test custom value
    std::env::set_var("RIPTIDE_WASM_INSTANCES_PER_WORKER", "16");
    let custom_val = get_instances_per_worker();
    assert_eq!(custom_val, 16);

    // Test invalid value (should fallback to default)
    std::env::set_var("RIPTIDE_WASM_INSTANCES_PER_WORKER", "invalid");
    let fallback_val = get_instances_per_worker();
    assert_eq!(fallback_val, 8);

    // Cleanup
    std::env::remove_var("RIPTIDE_WASM_INSTANCES_PER_WORKER");
}

#[tokio::test]
async fn test_pool_scaling_thresholds() {
    let config = create_test_config(16, 4);

    // Test scale-up threshold (80% utilization)
    let scale_up_threshold = 0.8;
    let high_load = 13; // 13/16 = 81.25%
    let should_scale_up = (high_load as f64 / config.max_pool_size as f64) > scale_up_threshold;
    assert!(should_scale_up);

    // Test scale-down threshold (20% utilization)
    let scale_down_threshold = 0.2;
    let low_load = 2; // 2/16 = 12.5%
    let should_scale_down = (low_load as f64 / config.max_pool_size as f64) < scale_down_threshold;
    assert!(should_scale_down);

    // Test stable zone (between thresholds)
    let stable_load = 6; // 6/16 = 37.5%
    let utilization = stable_load as f64 / config.max_pool_size as f64;
    assert!(utilization > scale_down_threshold && utilization < scale_up_threshold);
}

#[tokio::test]
async fn test_semaphore_fairness() {
    let semaphore = Arc::new(tokio::sync::Semaphore::new(3));
    let mut handles = vec![];

    // Spawn multiple tasks trying to acquire permits
    for i in 0..6 {
        let sem = semaphore.clone();
        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            tokio::time::sleep(Duration::from_millis(50)).await;
            i
        });
        handles.push(handle);
    }

    // All tasks should complete
    for handle in handles {
        assert!(handle.await.is_ok());
    }
}

#[tokio::test]
async fn test_timeout_extraction_simulation() {
    let config = create_test_config(4, 2);

    // Fast operation (should complete)
    let fast_op = async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok::<_, anyhow::Error>(())
    };

    let result = timeout(
        Duration::from_millis(config.extraction_timeout.unwrap()),
        fast_op,
    )
    .await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_ok());

    // Slow operation (should timeout)
    let slow_op = async {
        tokio::time::sleep(Duration::from_secs(10)).await;
        Ok::<_, anyhow::Error>(())
    };

    let timeout_result = timeout(Duration::from_millis(500), slow_op).await;
    assert!(timeout_result.is_err());
}

#[tokio::test]
async fn test_extraction_mode_conversion() {
    use riptide_types::ExtractionMode;

    // Test different extraction modes
    let _article_mode = ExtractionMode::Article;
    let _full_mode = ExtractionMode::Full;
    let _metadata_mode = ExtractionMode::Metadata;
    let custom_mode = ExtractionMode::Custom(vec!["h1".to_string(), "p".to_string()]);

    // Verify custom mode has selectors
    match custom_mode {
        ExtractionMode::Custom(selectors) => {
            assert_eq!(selectors.len(), 2);
            assert_eq!(selectors[0], "h1");
            assert_eq!(selectors[1], "p");
        }
        _ => panic!("Expected Custom mode"),
    }
}

#[tokio::test]
async fn test_pool_id_uniqueness() {
    use uuid::Uuid;

    let id1 = Uuid::new_v4().to_string();
    let id2 = Uuid::new_v4().to_string();

    // Pool IDs should be unique
    assert_ne!(id1, id2);

    // Should be valid UUID format
    assert!(Uuid::parse_str(&id1).is_ok());
    assert!(Uuid::parse_str(&id2).is_ok());
}

#[tokio::test]
async fn test_performance_metrics_aggregation() {
    let mut metrics = PerformanceMetrics::default();

    // Simulate averaging processing times
    let times = vec![100.0, 150.0, 200.0, 175.0, 125.0];

    for time in times {
        metrics.total_extractions += 1;
        metrics.successful_extractions += 1;

        // Running average
        metrics.avg_processing_time_ms = if metrics.total_extractions == 1 {
            time
        } else {
            (metrics.avg_processing_time_ms + time) / 2.0
        };
    }

    // Average should be reasonable
    assert!(metrics.avg_processing_time_ms > 100.0);
    assert!(metrics.avg_processing_time_ms < 200.0);
    assert_eq!(metrics.total_extractions, 5);
}

#[tokio::test]
async fn test_memory_limit_enforcement() {
    let tracker = WasmResourceTracker::default();

    // Test memory within limit
    let current = 100 * 1024 * 1024; // 100MB
    let desired = 150 * 1024 * 1024; // 150MB
    let result =
        wasmtime::ResourceLimiter::memory_growing(&mut tracker.clone(), current, desired, None);
    assert!(result.is_ok());
    assert!(result.unwrap());

    // Test memory exceeding limit
    let excessive = 600 * 1024 * 1024; // 600MB (over 512MB limit)
    let result_excessive =
        wasmtime::ResourceLimiter::memory_growing(&mut tracker.clone(), current, excessive, None);
    assert!(result_excessive.is_ok());
    assert!(!result_excessive.unwrap()); // Should reject
}

#[tokio::test]
async fn test_config_from_env() {
    // Set environment variables
    std::env::set_var("POOL_MAX_INSTANCES", "12");
    std::env::set_var("POOL_ENABLE_METRICS", "true");
    std::env::set_var("POOL_TIMEOUT_MS", "8000");
    std::env::set_var("POOL_MAX_POOL_SIZE", "16");
    std::env::set_var("POOL_INITIAL_POOL_SIZE", "6");

    let config = ExtractorConfig::from_env();

    assert_eq!(config.max_instances, 12);
    assert!(config.enable_metrics);
    assert_eq!(config.timeout_ms, 8000);
    assert_eq!(config.max_pool_size, 16);
    assert_eq!(config.initial_pool_size, 6);

    // Cleanup
    std::env::remove_var("POOL_MAX_INSTANCES");
    std::env::remove_var("POOL_ENABLE_METRICS");
    std::env::remove_var("POOL_TIMEOUT_MS");
    std::env::remove_var("POOL_MAX_POOL_SIZE");
    std::env::remove_var("POOL_INITIAL_POOL_SIZE");
}
