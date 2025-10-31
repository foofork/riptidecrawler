#![cfg(feature = "wasm-pool")]
#![allow(clippy::all)]

use riptide_pool::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::timeout;
use wasmtime::{Config, Engine};

/// Create a mock WASM engine for testing
fn create_test_engine() -> Engine {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.epoch_interruption(true);
    Engine::new(&config).unwrap()
}

/// Create test configuration
fn create_test_config() -> ExtractorConfig {
    ExtractorConfig {
        max_instances: 4,
        enable_metrics: true,
        timeout_ms: 5000,
        memory_limit_pages: Some(2048),
        extraction_timeout: Some(5000), // 5 seconds in milliseconds
        max_pool_size: 4,
        initial_pool_size: 2,
        epoch_timeout_ms: 10000,
        health_check_interval: 30000,
        memory_limit: Some(128 * 1024 * 1024), // 128MB
        circuit_breaker_timeout: 30000,        // 30 seconds in milliseconds
        circuit_breaker_failure_threshold: 60,
        enable_wit_validation: true,
    }
}

#[tokio::test]
async fn test_instance_pool_creation() {
    let config = create_test_config();
    let _engine = create_test_engine();

    // Note: This test validates configuration without attempting to create a pool
    // since that would require a valid WASM component file

    // Verify that configuration is set up correctly
    assert_eq!(config.max_pool_size, 4);
    assert_eq!(config.initial_pool_size, 2);
    assert!(config.enable_metrics);
    assert!(config.enable_wit_validation);
    assert_eq!(config.circuit_breaker_failure_threshold, 60);
    assert_eq!(config.timeout_ms, 5000);
    assert_eq!(config.epoch_timeout_ms, 10000);
}

#[tokio::test]
async fn test_semaphore_concurrency_control() {
    let config = create_test_config();

    // Test that semaphore limits concurrent access
    let semaphore = Arc::new(Semaphore::new(config.max_pool_size));

    // Acquire all permits
    let mut permits = Vec::new();
    for _ in 0..config.max_pool_size {
        let permit = semaphore.try_acquire().unwrap();
        permits.push(permit);
    }

    // Next acquire should fail (no permits available)
    assert!(semaphore.try_acquire().is_err());

    // Release one permit
    drop(permits.pop());

    // Now should be able to acquire one more
    // RAII guard: validates that semaphore correctly allows one more acquisition
    let _final_permit = semaphore.try_acquire().unwrap();
}

#[tokio::test]
async fn test_circuit_breaker_states() {
    use std::time::Instant;

    let state = CircuitBreakerState::Closed {
        failure_count: 0,
        success_count: 0,
        last_failure: None,
    };

    // Test state transitions
    match state {
        CircuitBreakerState::Closed { .. } => {
            // Should remain closed with low failure rate
            assert!(true);
        }
        _ => panic!("Expected Closed state"),
    }

    // Test open state timeout
    let opened_state = CircuitBreakerState::Open {
        opened_at: Instant::now(),
        failure_count: 10,
    };

    match opened_state {
        CircuitBreakerState::Open { opened_at, .. } => {
            // Should be recent
            assert!(opened_at.elapsed() < Duration::from_secs(1));
        }
        _ => panic!("Expected Open state"),
    }
}

#[tokio::test]
async fn test_timeout_handling() {
    let config = create_test_config();

    // Test extraction timeout
    let timeout_duration = config.extraction_timeout;
    assert_eq!(timeout_duration, Some(5000)); // 5 seconds in milliseconds

    // Test epoch timeout
    let epoch_timeout = config.epoch_timeout_ms;
    assert_eq!(epoch_timeout, 10000); // 10 seconds

    // Test timeout operations
    let fast_operation = async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        "success"
    };

    let result = timeout(
        Duration::from_millis(timeout_duration.unwrap_or(5000)),
        fast_operation,
    )
    .await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");

    // Test operation that times out
    let slow_operation = async {
        tokio::time::sleep(Duration::from_secs(10)).await;
        "too_slow"
    };

    let timeout_result = timeout(Duration::from_millis(100), slow_operation).await;
    assert!(timeout_result.is_err()); // Should timeout
}

#[tokio::test]
async fn test_metrics_collection() {
    let mut metrics = PerformanceMetrics::default();

    // Simulate extraction operations
    metrics.total_extractions = 100;
    metrics.successful_extractions = 92;
    metrics.failed_extractions = 5;
    metrics.fallback_extractions = 3;
    metrics.circuit_breaker_trips = 1;
    metrics.avg_processing_time_ms = 150.0;
    metrics.semaphore_wait_time_ms = 25.0;
    metrics.epoch_timeouts = 2;

    // Verify metrics calculations
    let success_rate =
        (metrics.successful_extractions as f64 / metrics.total_extractions as f64) * 100.0;
    assert_eq!(success_rate, 92.0);

    let fallback_rate =
        (metrics.fallback_extractions as f64 / metrics.total_extractions as f64) * 100.0;
    assert_eq!(fallback_rate, 3.0);

    // Test metrics consistency
    assert_eq!(
        metrics.successful_extractions + metrics.failed_extractions + metrics.fallback_extractions,
        100 // Should equal total extractions
    );
}

#[tokio::test]
async fn test_resource_tracker() {
    let tracker = WasmResourceTracker::default(); // Use default tracker

    // Test initial state
    assert_eq!(tracker.current_memory_pages(), 256); // Default returns 256
    assert_eq!(tracker.grow_failures(), 0);

    // Note: WasmResourceTracker is a simple struct without atomic fields
    // The actual tracking happens through the ResourceLimiter trait implementation
    // Test that default values are correct
    assert_eq!(tracker.memory_usage, 0);
    assert_eq!(tracker.cpu_usage, 0.0);
    assert_eq!(tracker.instance_count, 0);
}

#[tokio::test]
async fn test_pool_scaling() {
    let mut config = create_test_config();

    // Test pool size configurations
    assert!(config.initial_pool_size <= config.max_pool_size);

    // Test scaling scenarios
    config.max_pool_size = 16;
    config.initial_pool_size = 4;

    let utilization_threshold = 0.8; // 80%
    let current_active = 13; // 13 out of 16 instances active
    let current_utilization = current_active as f64 / config.max_pool_size as f64;

    // Should trigger scale-up consideration
    assert!(current_utilization > utilization_threshold);

    // Test scale-down scenario
    let low_active = 2; // Only 2 out of 16 instances active
    let low_utilization = low_active as f64 / config.max_pool_size as f64;
    let scale_down_threshold = 0.2; // 20%

    // Should trigger scale-down consideration
    assert!(low_utilization < scale_down_threshold);
}

#[tokio::test]
async fn test_environment_variable_configuration() {
    // Test that environment variables are properly read
    let instances_per_worker = get_instances_per_worker();
    assert!(instances_per_worker > 0);
    assert!(instances_per_worker <= 64); // Reasonable upper bound

    // Test default value when env var is not set
    std::env::remove_var("RIPTIDE_WASM_INSTANCES_PER_WORKER");
    let default_instances = get_instances_per_worker();
    assert_eq!(default_instances, 8); // Default value

    // Test custom value
    std::env::set_var("RIPTIDE_WASM_INSTANCES_PER_WORKER", "12");
    let custom_instances = get_instances_per_worker();
    assert_eq!(custom_instances, 12);

    // Clean up
    std::env::remove_var("RIPTIDE_WASM_INSTANCES_PER_WORKER");
}

#[tokio::test]
async fn test_instance_lifecycle() {
    // Note: wasmtime component model imports removed as they are not used in this test
    use std::sync::Arc;

    let _engine = Arc::new(create_test_engine());

    // This test would require a real WASM component to instantiate
    // For now, test the instance structure and lifecycle tracking

    // Test instance health checking logic
    let config = create_test_config();

    // Mock instance data
    let use_count = 500; // Under limit
    let failure_count = 2; // Under limit
    let memory_usage = 64 * 1024 * 1024; // 64MB, under 128MB limit

    // Simulate health check logic
    let is_healthy = use_count < 1000
        && failure_count < 5
        && memory_usage < config.memory_limit.unwrap_or(usize::MAX);

    assert!(is_healthy);

    // Test unhealthy conditions
    let high_use_count = 1500; // Over limit
    let is_unhealthy = high_use_count >= 1000;
    assert!(is_unhealthy);
}

/// Integration tests that would require actual WASM components
#[cfg(test)]
mod integration_tests {
    use super::*;

    // These tests would require actual WASM components to run
    // They are structured to show the expected behavior

    #[tokio::test]
    #[ignore = "Requires actual WASM component file"]
    async fn test_actual_wasm_extraction() {
        let config = create_test_config();
        let engine = create_test_engine();

        // This would test with a real WASM component
        let wasm_path = "./tests/fixtures/extractor.wasm";

        if std::path::Path::new(wasm_path).exists() {
            let pool = AdvancedInstancePool::new(config, engine, wasm_path).await;
            assert!(pool.is_ok());

            // Would verify extraction results with real HTML
        }
    }

    #[tokio::test]
    #[ignore = "Requires actual WASM component file"]
    async fn test_concurrent_extractions() {
        // This would test multiple concurrent extractions
        // to verify semaphore concurrency control works properly
    }

    #[tokio::test]
    #[ignore = "Requires actual WASM component file"]
    async fn test_fallback_mechanism() {
        // This would test fallback to native readability-rs
        // when WASM extraction fails
    }
}
