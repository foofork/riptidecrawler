//! Integration tests for ResourceManager
//!
//! Tests cover:
//! - Complete resource lifecycle (acquire -> use -> release)
//! - Cross-component interactions
//! - Real-world usage scenarios
//! - System-level behavior

use anyhow::Result;
use riptide_api::config::RiptideApiConfig;
use riptide_api::resource_manager::{ResourceManager, ResourceResult};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, timeout};

#[tokio::test]
async fn test_full_render_resource_lifecycle() -> Result<()> {
    // Test complete lifecycle: acquire -> use -> release
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let initial_status = manager.get_resource_status().await;

    // Acquire resources
    let guard = match manager
        .acquire_render_resources("https://example.com")
        .await?
    {
        ResourceResult::Success(g) => g,
        other => panic!("Expected success, got {:?}", other),
    };

    // Verify resources are in use
    let active_status = manager.get_resource_status().await;
    assert!(active_status.memory_usage_mb > initial_status.memory_usage_mb);

    // Simulate usage
    sleep(Duration::from_millis(100)).await;

    // Release resources
    drop(guard);
    sleep(Duration::from_millis(100)).await; // Allow async cleanup

    // Verify resources are released
    let final_status = manager.get_resource_status().await;
    assert!(final_status.memory_usage_mb <= active_status.memory_usage_mb);

    Ok(())
}

#[tokio::test]
async fn test_full_pdf_resource_lifecycle() -> Result<()> {
    // Test complete PDF resource lifecycle
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let initial_status = manager.get_resource_status().await;
    let initial_available = initial_status.pdf_available;

    // Acquire PDF resources
    let guard = match manager.acquire_pdf_resources().await? {
        ResourceResult::Success(g) => g,
        other => panic!("Expected success, got {:?}", other),
    };

    // Verify permit is acquired
    let active_status = manager.get_resource_status().await;
    assert_eq!(active_status.pdf_available, initial_available - 1);

    // Simulate PDF processing
    sleep(Duration::from_millis(50)).await;

    // Release resources
    drop(guard);
    sleep(Duration::from_millis(100)).await;

    // Verify permit is released
    let final_status = manager.get_resource_status().await;
    assert_eq!(final_status.pdf_available, initial_available);

    Ok(())
}

#[tokio::test]
async fn test_concurrent_resource_management() -> Result<()> {
    // Test concurrent resource acquisition and release
    let mut config = ApiConfig::default();
    config.headless.max_pool_size = 5;
    config.pdf.max_concurrent = 5;

    let manager = Arc::new(ResourceManager::new(config).await?);

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let mgr = Arc::clone(&manager);
            tokio::spawn(async move {
                if i % 2 == 0 {
                    // Render resources
                    match mgr
                        .acquire_render_resources(&format!("https://site{}.com", i))
                        .await
                    {
                        Ok(ResourceResult::Success(guard)) => {
                            sleep(Duration::from_millis(50)).await;
                            drop(guard);
                            true
                        }
                        _ => false,
                    }
                } else {
                    // PDF resources
                    match mgr.acquire_pdf_resources().await {
                        Ok(ResourceResult::Success(guard)) => {
                            sleep(Duration::from_millis(50)).await;
                            drop(guard);
                            true
                        }
                        _ => false,
                    }
                }
            })
        })
        .collect();

    let mut success_count = 0;
    for handle in handles {
        if handle.await? {
            success_count += 1;
        }
    }

    assert!(success_count > 0, "Some operations should succeed");

    Ok(())
}

#[tokio::test]
async fn test_memory_pressure_workflow() -> Result<()> {
    // Test complete workflow under memory pressure
    let mut config = ApiConfig::default();
    config.memory.global_memory_limit_mb = 200;
    config.memory.pressure_threshold = 0.5;

    let manager = ResourceManager::new(config).await?;

    // Fill memory to trigger pressure
    manager.memory_manager.track_allocation(120).await;

    // Attempt operations under pressure
    let result = manager
        .acquire_render_resources("https://example.com")
        .await?;

    assert!(
        matches!(result, ResourceResult::MemoryPressure),
        "Should block due to memory pressure"
    );

    // Relieve pressure
    manager.memory_manager.track_deallocation(100).await;

    // Operations should now succeed
    let result = manager
        .acquire_render_resources("https://example.com")
        .await?;

    assert!(
        matches!(result, ResourceResult::Success(_)),
        "Should succeed after pressure relief"
    );

    Ok(())
}

#[tokio::test]
async fn test_rate_limiting_workflow() -> Result<()> {
    // Test rate limiting across multiple requests
    let mut config = ApiConfig::default();
    config.rate_limiting.requests_per_second_per_host = 2.0;
    config.rate_limiting.burst_capacity_per_host = 3.0;
    config.rate_limiting.enabled = true;

    let manager = ResourceManager::new(config).await?;

    let host = "ratelimit-test.com";
    let url = format!("https://{}", host);

    let mut results = Vec::new();

    // Make burst of requests
    for _ in 0..10 {
        let result = manager.acquire_render_resources(&url).await?;
        results.push(result);
    }

    // Should have mix of success and rate limited
    let rate_limited_count = results
        .iter()
        .filter(|r| matches!(r, ResourceResult::RateLimited { .. }))
        .count();

    assert!(rate_limited_count > 0, "Some requests should be rate limited");

    Ok(())
}

#[tokio::test]
async fn test_timeout_and_cleanup_workflow() -> Result<()> {
    // Test timeout scenario with cleanup
    let mut config = ApiConfig::default();
    config.performance.auto_cleanup_on_timeout = true;

    let manager = ResourceManager::new(config).await?;

    let initial_status = manager.get_resource_status().await;

    // Simulate timeout scenario
    manager.cleanup_on_timeout("render").await;

    let final_status = manager.get_resource_status().await;

    // Verify cleanup was triggered
    assert!(final_status.timeout_count > initial_status.timeout_count);

    Ok(())
}

#[tokio::test]
async fn test_resource_exhaustion_recovery() -> Result<()> {
    // Test recovery from resource exhaustion
    let mut config = ApiConfig::default();
    config.pdf.max_concurrent = 2;

    let manager = Arc::new(ResourceManager::new(config).await?);

    // Exhaust resources
    let guard1 = match manager.acquire_pdf_resources().await? {
        ResourceResult::Success(g) => g,
        _ => panic!("Expected success"),
    };

    let guard2 = match manager.acquire_pdf_resources().await? {
        ResourceResult::Success(g) => g,
        _ => panic!("Expected success"),
    };

    // Should be exhausted
    let result = manager.acquire_pdf_resources().await?;
    assert!(matches!(
        result,
        ResourceResult::Timeout | ResourceResult::ResourceExhausted
    ));

    // Release one resource
    drop(guard1);
    sleep(Duration::from_millis(100)).await;

    // Should be able to acquire again
    let result = manager.acquire_pdf_resources().await?;
    assert!(matches!(result, ResourceResult::Success(_)));

    drop(guard2);

    Ok(())
}

#[tokio::test]
async fn test_multi_host_rate_limiting() -> Result<()> {
    // Test rate limiting with multiple hosts
    let mut config = ApiConfig::default();
    config.rate_limiting.requests_per_second_per_host = 1.0;
    config.rate_limiting.enabled = true;

    let manager = ResourceManager::new(config).await?;

    let hosts = vec!["host1.com", "host2.com", "host3.com"];

    for host in hosts {
        let url = format!("https://{}", host);

        // First request should succeed for each host
        let result = manager.acquire_render_resources(&url).await?;
        assert!(matches!(result, ResourceResult::Success(_) | ResourceResult::RateLimited { .. }));
    }

    Ok(())
}

#[tokio::test]
async fn test_wasm_instance_per_worker_integration() -> Result<()> {
    // Test WASM instance management in real workflow
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    // Acquire resources which should also acquire WASM instance
    let _guard = match manager
        .acquire_render_resources("https://example.com")
        .await?
    {
        ResourceResult::Success(g) => g,
        _ => panic!("Expected success"),
    };

    // Check WASM instance health
    let health = manager.wasm_manager.get_instance_health().await;

    assert!(!health.is_empty(), "Should have at least one WASM instance");

    Ok(())
}

#[tokio::test]
async fn test_performance_monitoring_integration() -> Result<()> {
    // Test performance monitoring through complete workflow
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    // Perform operations
    let _guard = match manager
        .acquire_render_resources("https://example.com")
        .await?
    {
        ResourceResult::Success(g) => g,
        _ => return Ok(()),
    };

    // Record performance metrics
    manager
        .performance_monitor
        .record_render_operation(
            "https://example.com",
            Duration::from_millis(500),
            true,
            10,
            25,
        )
        .await?;

    // Verify metrics in status
    let status = manager.get_resource_status().await;
    assert!(status.degradation_score >= 0.0);

    Ok(())
}

#[tokio::test]
async fn test_stress_concurrent_mixed_operations() -> Result<()> {
    // Stress test with mixed concurrent operations
    let config = ApiConfig::default();
    let manager = Arc::new(ResourceManager::new(config).await?);

    let operations = 50;
    let handles: Vec<_> = (0..operations)
        .map(|i| {
            let mgr = Arc::clone(&manager);
            tokio::spawn(async move {
                match i % 4 {
                    0 => {
                        // Render acquisition
                        mgr.acquire_render_resources(&format!("https://site{}.com", i))
                            .await
                            .ok();
                    }
                    1 => {
                        // PDF acquisition
                        mgr.acquire_pdf_resources().await.ok();
                    }
                    2 => {
                        // Memory tracking
                        mgr.memory_manager.track_allocation(10).await;
                        sleep(Duration::from_millis(10)).await;
                        mgr.memory_manager.track_deallocation(10).await;
                    }
                    _ => {
                        // Timeout recording
                        mgr.performance_monitor.record_timeout().await;
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        handle.await?;
    }

    // System should remain stable
    let status = manager.get_resource_status().await;
    println!("Final status after stress test: {:?}", status);

    Ok(())
}

#[tokio::test]
async fn test_resource_status_consistency() -> Result<()> {
    // Test that resource status remains consistent
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    // Perform various operations
    let _guard1 = manager.acquire_pdf_resources().await.ok();
    let _guard2 = manager
        .acquire_render_resources("https://example.com")
        .await
        .ok();

    manager.memory_manager.track_allocation(50).await;
    manager.performance_monitor.record_timeout().await;

    // Check status multiple times - should be consistent
    let status1 = manager.get_resource_status().await;
    sleep(Duration::from_millis(10)).await;
    let status2 = manager.get_resource_status().await;

    // Counts should match (no unexpected changes)
    assert_eq!(status1.timeout_count, status2.timeout_count);

    Ok(())
}

#[tokio::test]
async fn test_long_running_operation() -> Result<()> {
    // Test long-running operation with resources
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let guard = match manager
        .acquire_render_resources("https://longrunn ing.com")
        .await?
    {
        ResourceResult::Success(g) => g,
        _ => return Ok(()),
    };

    // Simulate long operation
    timeout(Duration::from_secs(2), async {
        sleep(Duration::from_millis(500)).await;
        // Operation continues...
        sleep(Duration::from_millis(500)).await;
    })
    .await?;

    drop(guard);

    Ok(())
}
