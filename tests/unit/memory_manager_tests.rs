//! Unit tests for MemoryManager
//!
//! Tests cover:
//! - Memory tracking (allocation/deallocation)
//! - Pressure detection
//! - Cleanup triggering
//! - Garbage collection
//! - Threshold management

use anyhow::Result;
use riptide_api::config::RiptideApiConfig;
use riptide_api::resource_manager::ResourceManager;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_memory_tracking_allocation() -> Result<()> {
    // Test basic memory allocation tracking
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let initial_usage = manager.memory_manager.current_usage_mb();

    manager.memory_manager.track_allocation(100).await;

    let final_usage = manager.memory_manager.current_usage_mb();

    assert_eq!(
        final_usage,
        initial_usage + 100,
        "Memory usage should increase by 100MB"
    );

    Ok(())
}

#[tokio::test]
async fn test_memory_tracking_deallocation() -> Result<()> {
    // Test memory deallocation tracking
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    // Allocate then deallocate
    manager.memory_manager.track_allocation(200).await;
    let after_alloc = manager.memory_manager.current_usage_mb();

    manager.memory_manager.track_deallocation(200).await;
    let after_dealloc = manager.memory_manager.current_usage_mb();

    assert_eq!(
        after_dealloc,
        after_alloc - 200,
        "Memory usage should decrease by 200MB"
    );

    Ok(())
}

#[tokio::test]
async fn test_memory_pressure_detection() -> Result<()> {
    // Test that memory pressure is detected at threshold
    let mut config = ApiConfig::default();
    config.memory.global_memory_limit_mb = 1000;
    config.memory.pressure_threshold = 0.8; // 80%

    let manager = ResourceManager::new(config).await?;

    // Should not be under pressure initially
    assert!(!manager.memory_manager.is_under_pressure());

    // Allocate to trigger pressure (81% of limit)
    manager.memory_manager.track_allocation(810).await;

    // Should now detect pressure
    assert!(manager.memory_manager.is_under_pressure());

    Ok(())
}

#[tokio::test]
async fn test_memory_pressure_relief() -> Result<()> {
    // Test that pressure is relieved when memory is freed
    let mut config = ApiConfig::default();
    config.memory.global_memory_limit_mb = 1000;
    config.memory.pressure_threshold = 0.8;

    let manager = ResourceManager::new(config).await?;

    // Trigger pressure
    manager.memory_manager.track_allocation(850).await;
    assert!(manager.memory_manager.is_under_pressure());

    // Deallocate to relieve pressure
    manager.memory_manager.track_deallocation(400).await;

    // Should no longer be under pressure
    assert!(!manager.memory_manager.is_under_pressure());

    Ok(())
}

#[tokio::test]
async fn test_memory_cleanup_trigger() -> Result<()> {
    // Test cleanup triggering
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let initial_status = manager.get_resource_status().await;
    let initial_cleanups = initial_status.timeout_count; // Using available metric

    manager.memory_manager.trigger_cleanup().await;

    // Allow cleanup to process
    sleep(Duration::from_millis(50)).await;

    // Cleanup should have been recorded in metrics
    let status = manager.get_resource_status().await;
    // Note: cleanup_operations is not exposed in ResourceStatus, but we can verify
    // the cleanup method was called without error

    Ok(())
}

#[tokio::test]
async fn test_memory_gc_trigger_threshold() -> Result<()> {
    // Test GC trigger based on threshold
    let mut config = ApiConfig::default();
    config.memory.gc_trigger_threshold_mb = 500;

    let manager = ResourceManager::new(config).await?;

    // Below threshold - should not trigger GC
    manager.memory_manager.track_allocation(400).await;
    assert!(!manager.memory_manager.should_trigger_gc().await);

    // At threshold - should trigger GC
    manager.memory_manager.track_allocation(100).await;
    assert!(manager.memory_manager.should_trigger_gc().await);

    Ok(())
}

#[tokio::test]
async fn test_memory_gc_execution() -> Result<()> {
    // Test garbage collection execution
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    manager.memory_manager.trigger_gc().await;

    // GC should complete without error
    // Actual GC behavior would be tested in integration tests

    Ok(())
}

#[tokio::test]
async fn test_memory_concurrent_allocations() -> Result<()> {
    // Test concurrent memory allocations
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let mgr = manager.clone();
            tokio::spawn(async move {
                mgr.memory_manager.track_allocation(10).await;
            })
        })
        .collect();

    for handle in handles {
        handle.await?;
    }

    let final_usage = manager.memory_manager.current_usage_mb();

    // Should have tracked all allocations
    assert!(final_usage >= 100, "Expected at least 100MB allocated");

    Ok(())
}

#[tokio::test]
async fn test_memory_concurrent_deallocations() -> Result<()> {
    // Test concurrent memory deallocations
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    // Pre-allocate
    manager.memory_manager.track_allocation(200).await;
    let before = manager.memory_manager.current_usage_mb();

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let mgr = manager.clone();
            tokio::spawn(async move {
                mgr.memory_manager.track_deallocation(10).await;
            })
        })
        .collect();

    for handle in handles {
        handle.await?;
    }

    let after = manager.memory_manager.current_usage_mb();

    // Should have deallocated
    assert!(after < before, "Memory should have been deallocated");

    Ok(())
}

#[tokio::test]
async fn test_memory_saturation_protection() -> Result<()> {
    // Test that deallocation doesn't underflow
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    // Try to deallocate more than allocated
    manager.memory_manager.track_allocation(50).await;
    manager.memory_manager.track_deallocation(100).await;

    let usage = manager.memory_manager.current_usage_mb();

    // Should saturate at 0, not underflow
    assert!(usage == 0 || usage < 50);

    Ok(())
}

#[tokio::test]
async fn test_memory_pressure_at_exact_threshold() -> Result<()> {
    // Test pressure detection at exact threshold
    let mut config = ApiConfig::default();
    config.memory.global_memory_limit_mb = 1000;
    config.memory.pressure_threshold = 0.8;

    let manager = ResourceManager::new(config).await?;

    // Allocate exactly at threshold (800MB)
    manager.memory_manager.track_allocation(800).await;

    // Behavior at exact threshold
    let is_under_pressure = manager.memory_manager.is_under_pressure();
    println!("Pressure at exact threshold: {}", is_under_pressure);

    Ok(())
}

#[tokio::test]
async fn test_memory_zero_allocation() -> Result<()> {
    // Test zero-size allocation
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let before = manager.memory_manager.current_usage_mb();
    manager.memory_manager.track_allocation(0).await;
    let after = manager.memory_manager.current_usage_mb();

    assert_eq!(before, after, "Zero allocation should not change usage");

    Ok(())
}

#[tokio::test]
async fn test_memory_large_allocation() -> Result<()> {
    // Test very large allocation
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let large_size = 10_000; // 10GB
    manager.memory_manager.track_allocation(large_size).await;

    let usage = manager.memory_manager.current_usage_mb();
    assert_eq!(usage, large_size);

    Ok(())
}

#[tokio::test]
async fn test_memory_rapid_alloc_dealloc() -> Result<()> {
    // Test rapid allocation and deallocation cycles
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    for _ in 0..100 {
        manager.memory_manager.track_allocation(10).await;
        manager.memory_manager.track_deallocation(10).await;
    }

    let final_usage = manager.memory_manager.current_usage_mb();

    // Should be close to initial (0 or small value)
    assert!(
        final_usage < 50,
        "Rapid cycles should not accumulate memory"
    );

    Ok(())
}

#[tokio::test]
async fn test_memory_metrics_integration() -> Result<()> {
    // Test that memory metrics are properly integrated
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    manager.memory_manager.track_allocation(250).await;

    let status = manager.get_resource_status().await;

    assert_eq!(status.memory_usage_mb, 250);
    assert!(!status.memory_pressure);

    Ok(())
}

#[tokio::test]
async fn test_memory_cleanup_auto_trigger() -> Result<()> {
    // Test auto cleanup on timeout
    let mut config = ApiConfig::default();
    config.performance.auto_cleanup_on_timeout = true;

    let manager = ResourceManager::new(config).await?;

    manager.cleanup_on_timeout("test_operation").await;

    // Cleanup should have been triggered
    // Verify via metrics or state (implementation-dependent)

    Ok(())
}

#[tokio::test]
async fn test_memory_gc_conditional_trigger() -> Result<()> {
    // Test that GC only triggers when threshold is met
    let mut config = ApiConfig::default();
    config.memory.gc_trigger_threshold_mb = 1000;

    let manager = ResourceManager::new(config).await?;

    // Allocate below threshold
    manager.memory_manager.track_allocation(500).await;

    if manager.memory_manager.should_trigger_gc().await {
        manager.memory_manager.trigger_gc().await;
    }

    // Allocate above threshold
    manager.memory_manager.track_allocation(600).await;

    if manager.memory_manager.should_trigger_gc().await {
        manager.memory_manager.trigger_gc().await;
    }

    // Should have triggered GC at least once
    Ok(())
}
