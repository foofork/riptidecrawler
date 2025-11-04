//! Unit tests for WasmInstanceManager
//!
//! Tests cover:
//! - Single instance per worker requirement
//! - Instance lifecycle management
//! - Health monitoring
//! - Cleanup detection
//! - Concurrent access

use anyhow::Result;
use riptide_api::config::RiptideApiConfig;
use riptide_api::resource_manager::ResourceManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_wasm_single_instance_per_worker() -> Result<()> {
    // Test that only one WASM instance exists per worker
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let worker_id = "worker_test_1";

    // Acquire multiple times for same worker
    let _guard1 = manager.wasm_manager.test_acquire_instance(worker_id).await?;
    let _guard2 = manager.wasm_manager.test_acquire_instance(worker_id).await?;
    let _guard3 = manager.wasm_manager.test_acquire_instance(worker_id).await?;

    // Check instance health
    let health = manager.wasm_manager.get_instance_health().await;

    // Should only have one instance for this worker
    let worker_instances: Vec<_> = health
        .iter()
        .filter(|(id, _, _, _, _)| id == worker_id)
        .collect();

    assert_eq!(
        worker_instances.len(),
        1,
        "Expected exactly 1 instance per worker"
    );

    Ok(())
}

#[tokio::test]
async fn test_wasm_multiple_workers() -> Result<()> {
    // Test multiple workers can have their own instances
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let workers = vec!["worker_1", "worker_2", "worker_3"];

    for worker in &workers {
        let _guard = manager.wasm_manager.test_acquire_instance(worker).await?;
    }

    let health = manager.wasm_manager.get_instance_health().await;

    // Should have one instance per worker
    assert_eq!(health.len(), workers.len());

    // Verify each worker has exactly one instance
    for worker in &workers {
        let count = health.iter().filter(|(id, _, _, _, _)| id == worker).count();
        assert_eq!(count, 1);
    }

    Ok(())
}

#[tokio::test]
async fn test_wasm_instance_health_tracking() -> Result<()> {
    // Test that instance health is properly tracked
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let worker_id = "health_test_worker";

    // Acquire instance
    let _guard = manager.wasm_manager.test_acquire_instance(worker_id).await?;

    let health = manager.wasm_manager.get_instance_health().await;

    assert!(!health.is_empty());

    for (id, is_healthy, ops_count, memory_usage, age) in health {
        println!(
            "Worker: {}, Healthy: {}, Ops: {}, Memory: {}MB, Age: {:?}",
            id, is_healthy, ops_count, memory_usage, age
        );

        // Verify health metrics
        assert!(is_healthy, "Instance should be healthy");
        assert!(ops_count >= 1, "Should have at least one operation");
        assert!(age >= Duration::from_millis(0));
    }

    Ok(())
}

#[tokio::test]
async fn test_wasm_operations_counter() -> Result<()> {
    // Test that operations are counted per instance
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let worker_id = "ops_counter_worker";

    // Perform multiple operations
    for _ in 0..5 {
        let _guard = manager.wasm_manager.test_acquire_instance(worker_id).await?;
    }

    let health = manager.wasm_manager.get_instance_health().await;

    let worker_health = health
        .iter()
        .find(|(id, _, _, _, _)| id == worker_id)
        .expect("Worker should exist");

    let (_, _, ops_count, _, _) = worker_health;

    assert!(
        *ops_count >= 5,
        "Expected at least 5 operations, got {}",
        ops_count
    );

    Ok(())
}

#[tokio::test]
async fn test_wasm_cleanup_detection_idle() -> Result<()> {
    // Test cleanup detection for idle instances
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    // Create instance
    let _guard = manager
        .wasm_manager
        .test_acquire_instance("idle_worker")
        .await?;

    // Immediately check - should not need cleanup
    let needs_cleanup = manager.wasm_manager.needs_cleanup().await;
    assert!(!needs_cleanup, "Should not need cleanup immediately");

    Ok(())
}

#[tokio::test]
async fn test_wasm_concurrent_acquisitions() -> Result<()> {
    // Test concurrent acquisitions from multiple tasks
    let config = ApiConfig::default();
    let manager = Arc::new(ResourceManager::new(config).await?);

    let worker_id = "concurrent_worker";

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let mgr = Arc::clone(&manager);
            let wid = worker_id.to_string();
            tokio::spawn(async move { mgr.wasm_manager.test_acquire_instance(&wid).await })
        })
        .collect();

    let mut success_count = 0;
    for handle in handles {
        if handle.await.is_ok() {
            success_count += 1;
        }
    }

    assert_eq!(success_count, 10, "All acquisitions should succeed");

    // Verify only one instance was created
    let health = manager.wasm_manager.get_instance_health().await;
    let worker_instances: Vec<_> = health
        .iter()
        .filter(|(id, _, _, _, _)| id == worker_id)
        .collect();

    assert_eq!(worker_instances.len(), 1, "Should still have only 1 instance");

    Ok(())
}

#[tokio::test]
async fn test_wasm_instance_age_tracking() -> Result<()> {
    // Test that instance age is tracked correctly
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let worker_id = "age_tracking_worker";

    let _guard = manager.wasm_manager.test_acquire_instance(worker_id).await?;

    // Wait a bit
    sleep(Duration::from_millis(100)).await;

    let health = manager.wasm_manager.get_instance_health().await;

    let worker_health = health
        .iter()
        .find(|(id, _, _, _, _)| id == worker_id)
        .expect("Worker should exist");

    let (_, _, _, _, age) = worker_health;

    assert!(
        *age >= Duration::from_millis(100),
        "Instance age should be at least 100ms"
    );

    Ok(())
}

#[tokio::test]
async fn test_wasm_metrics_tracking() -> Result<()> {
    // Test that WASM instances are tracked in metrics
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let initial_status = manager.get_resource_status().await;

    // Create some instances
    let workers = vec!["metrics_worker_1", "metrics_worker_2", "metrics_worker_3"];

    for worker in workers {
        let _guard = manager.wasm_manager.test_acquire_instance(worker).await?;
    }

    // Check metrics through status
    let health = manager.wasm_manager.get_instance_health().await;

    assert!(
        health.len() > initial_status.memory_usage_mb / 100,
        "Should have tracked instance creation"
    );

    Ok(())
}

#[tokio::test]
async fn test_wasm_guard_lifecycle() -> Result<()> {
    // Test that WasmGuard lifecycle is managed properly
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let worker_id = "guard_lifecycle_worker";

    {
        let _guard = manager.wasm_manager.test_acquire_instance(worker_id).await?;
        // Guard is alive in this scope
    } // Guard dropped here

    // Instance should still exist (single instance per worker)
    let health = manager.wasm_manager.get_instance_health().await;

    let exists = health.iter().any(|(id, _, _, _, _)| id == worker_id);
    assert!(exists, "Instance should still exist after guard drop");

    Ok(())
}

#[tokio::test]
async fn test_wasm_empty_worker_id() -> Result<()> {
    // Test handling of empty worker ID
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let result = manager.wasm_manager.test_acquire_instance("").await;

    // Should handle empty ID gracefully
    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_wasm_special_characters_in_worker_id() -> Result<()> {
    // Test worker IDs with special characters
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let special_ids = vec![
        "worker-with-dashes",
        "worker_with_underscores",
        "worker.with.dots",
        "worker@with@at",
    ];

    for worker_id in special_ids {
        let result = manager.wasm_manager.test_acquire_instance(worker_id).await;
        assert!(result.is_ok(), "Should handle special chars in ID: {}", worker_id);
    }

    Ok(())
}

#[tokio::test]
async fn test_wasm_long_worker_id() -> Result<()> {
    // Test with very long worker ID
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let long_id = "a".repeat(1000);

    let result = manager.wasm_manager.test_acquire_instance(&long_id).await;
    assert!(result.is_ok(), "Should handle long worker IDs");

    Ok(())
}
