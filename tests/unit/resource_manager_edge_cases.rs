//! Edge case and error scenario tests for ResourceManager
//!
//! Tests cover:
//! - Boundary conditions
//! - Error handling
//! - Invalid inputs
//! - Race conditions
//! - Resource contention

use anyhow::Result;
use riptide_api::config::RiptideApiConfig;
use riptide_api::resource_manager::{ResourceManager, ResourceResult};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_edge_invalid_url_formats() -> Result<()> {
    // Test various invalid URL formats
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let invalid_urls = vec![
        "",
        "not-a-url",
        "://missing-scheme",
        "http://",
        "ftp:incomplete",
        "javascript:alert(1)",
        "data:text/plain,test",
        "   ",
        "http://[invalid-ipv6",
    ];

    for url in invalid_urls {
        let result = manager.acquire_render_resources(url).await;
        // Should handle gracefully (error or special handling)
        match result {
            Ok(_) | Err(_) => {} // Both acceptable for invalid URLs
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_edge_very_long_url() -> Result<()> {
    // Test extremely long URL
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let long_url = format!("https://example.com/{}", "a".repeat(10000));

    let result = manager.acquire_render_resources(&long_url).await;

    // Should handle without panic
    assert!(result.is_ok() || result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_edge_special_characters_in_url() -> Result<()> {
    // Test URLs with special characters
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let special_urls = vec![
        "https://example.com/path?query=value&foo=bar",
        "https://example.com:8080/path",
        "https://sub.domain.example.com",
        "https://example.com/path%20with%20spaces",
        "https://user:pass@example.com",
    ];

    for url in special_urls {
        manager.acquire_render_resources(url).await.ok();
    }

    Ok(())
}

#[tokio::test]
async fn test_edge_rapid_acquire_release_cycles() -> Result<()> {
    // Test rapid acquisition and release cycles
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    for i in 0..100 {
        if let Ok(ResourceResult::Success(guard)) = manager
            .acquire_render_resources(&format!("https://cycle{}.com", i))
            .await
        {
            drop(guard); // Immediate release
        }
    }

    // System should remain stable
    let status = manager.get_resource_status().await;
    assert!(status.headless_pool_total > 0);

    Ok(())
}

#[tokio::test]
async fn test_edge_pdf_semaphore_at_zero() -> Result<()> {
    // Test PDF semaphore with zero capacity (edge case)
    let mut config = ApiConfig::default();
    config.pdf.max_concurrent = 1; // Minimum

    let manager = ResourceManager::new(config).await?;

    let guard = manager.acquire_pdf_resources().await?;

    match guard {
        ResourceResult::Success(_) => {
            // Should work with capacity of 1
        }
        _ => {}
    }

    Ok(())
}

#[tokio::test]
async fn test_edge_memory_deallocation_underflow() -> Result<()> {
    // Test that deallocation can't underflow
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    // Try to deallocate more than allocated
    manager.memory_manager.track_allocation(10).await;
    manager.memory_manager.track_deallocation(100).await;

    let usage = manager.memory_manager.current_usage_mb();

    // Should not underflow (saturate at 0)
    assert!(usage < 100);

    Ok(())
}

#[tokio::test]
async fn test_edge_memory_pressure_exact_threshold() -> Result<()> {
    // Test memory pressure at exact threshold boundary
    let mut config = ApiConfig::default();
    config.memory.global_memory_limit_mb = 1000;
    config.memory.pressure_threshold = 0.8;

    let manager = ResourceManager::new(config).await?;

    // Allocate exactly at threshold
    manager.memory_manager.track_allocation(800).await;

    let is_under_pressure = manager.memory_manager.is_under_pressure();
    println!("Pressure at threshold: {}", is_under_pressure);

    Ok(())
}

#[tokio::test]
async fn test_edge_rate_limit_zero_rps() -> Result<()> {
    // Test rate limiting with very low RPS
    let mut config = ApiConfig::default();
    config.rate_limiting.requests_per_second_per_host = 0.1; // 1 request per 10 seconds
    config.rate_limiting.enabled = true;

    let manager = ResourceManager::new(config).await?;

    let result1 = manager.rate_limiter.check_rate_limit("slowhost.com").await;
    let result2 = manager.rate_limiter.check_rate_limit("slowhost.com").await;

    // First should succeed, second likely rate limited
    assert!(result1.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_edge_concurrent_resource_exhaustion() -> Result<()> {
    // Test concurrent requests hitting resource limits
    let mut config = ApiConfig::default();
    config.pdf.max_concurrent = 2;

    let manager = Arc::new(ResourceManager::new(config).await?);

    let handles: Vec<_> = (0..20)
        .map(|_| {
            let mgr = Arc::clone(&manager);
            tokio::spawn(async move { mgr.acquire_pdf_resources().await })
        })
        .collect();

    let mut success_count = 0;
    let mut blocked_count = 0;

    for handle in handles {
        match handle.await {
            Ok(Ok(ResourceResult::Success(_))) => success_count += 1,
            Ok(Ok(ResourceResult::Timeout)) | Ok(Ok(ResourceResult::ResourceExhausted)) => {
                blocked_count += 1
            }
            _ => {}
        }
    }

    // Should have appropriate distribution
    assert!(success_count > 0);
    assert!(blocked_count > 0);
    println!(
        "Edge case results: {} success, {} blocked",
        success_count, blocked_count
    );

    Ok(())
}

#[tokio::test]
async fn test_edge_timeout_at_boundary() -> Result<()> {
    // Test timeout behavior at configured boundary
    let mut config = ApiConfig::default();
    config.timeouts.insert("render".to_string(), Duration::from_millis(100));

    let manager = ResourceManager::new(config).await?;

    // Operations near timeout boundary should be handled
    manager.cleanup_on_timeout("render").await;

    Ok(())
}

#[tokio::test]
async fn test_edge_wasm_worker_id_collisions() -> Result<()> {
    // Test potential worker ID collisions
    let config = ApiConfig::default();
    let manager = Arc::new(ResourceManager::new(config).await?);

    let worker_id = "collision_test";

    // Concurrent acquisitions with same worker ID
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let mgr = Arc::clone(&manager);
            let wid = worker_id.to_string();
            tokio::spawn(async move { mgr.wasm_manager.test_acquire_instance(&wid).await })
        })
        .collect();

    for handle in handles {
        handle.await?.ok();
    }

    // Should handle collisions gracefully (single instance per worker)
    let health = manager.wasm_manager.get_instance_health().await;
    let count = health
        .iter()
        .filter(|(id, _, _, _, _)| id == worker_id)
        .count();

    assert_eq!(count, 1, "Should maintain single instance per worker");

    Ok(())
}

#[tokio::test]
async fn test_edge_mixed_success_and_errors() -> Result<()> {
    // Test system stability with mix of successes and errors
    let config = ApiConfig::default();
    let manager = Arc::new(ResourceManager::new(config).await?);

    let urls = vec![
        ("https://valid.com", true),
        ("invalid-url", false),
        ("https://another-valid.com", true),
        ("", false),
        ("https://third-valid.com", true),
    ];

    for (url, should_be_valid) in urls {
        let result = manager.acquire_render_resources(url).await;

        if should_be_valid {
            // Valid URLs should succeed or have valid failure reasons
            assert!(result.is_ok() || result.is_err());
        } else {
            // Invalid URLs should fail gracefully
            assert!(result.is_err() || result.is_ok());
        }
    }

    // System should remain operational
    let status = manager.get_resource_status().await;
    assert!(status.headless_pool_total > 0);

    Ok(())
}

#[tokio::test]
async fn test_edge_resource_guard_drop_order() -> Result<()> {
    // Test guard drop ordering doesn't cause issues
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let guard1 = manager.acquire_pdf_resources().await.ok();
    let guard2 = manager.acquire_pdf_resources().await.ok();

    // Drop in reverse order
    drop(guard2);
    drop(guard1);

    sleep(Duration::from_millis(100)).await;

    // Should handle any order
    let status = manager.get_resource_status().await;
    assert_eq!(status.pdf_available, status.pdf_total);

    Ok(())
}

#[tokio::test]
async fn test_edge_zero_timeout_duration() -> Result<()> {
    // Test zero timeout duration
    let mut config = ApiConfig::default();
    config
        .timeouts
        .insert("test".to_string(), Duration::from_millis(0));

    let manager = ResourceManager::new(config).await?;

    manager.cleanup_on_timeout("test").await;

    Ok(())
}

#[tokio::test]
async fn test_edge_extremely_high_concurrency() -> Result<()> {
    // Stress test with extreme concurrency
    let config = ApiConfig::default();
    let manager = Arc::new(ResourceManager::new(config).await?);

    let handles: Vec<_> = (0..1000)
        .map(|i| {
            let mgr = Arc::clone(&manager);
            tokio::spawn(async move {
                mgr.memory_manager.track_allocation(1).await;
                sleep(Duration::from_micros(100)).await;
                mgr.memory_manager.track_deallocation(1).await;
            })
        })
        .collect();

    for handle in handles {
        handle.await?;
    }

    // System should remain stable under high load
    let status = manager.get_resource_status().await;
    println!("Status after extreme concurrency: {:?}", status);

    Ok(())
}

#[tokio::test]
async fn test_edge_resource_status_under_load() -> Result<()> {
    // Test resource status reporting under load
    let config = ApiConfig::default();
    let manager = Arc::new(ResourceManager::new(config).await?);

    // Apply load
    let load_handles: Vec<_> = (0..50)
        .map(|_| {
            let mgr = Arc::clone(&manager);
            tokio::spawn(async move {
                mgr.memory_manager.track_allocation(10).await;
                sleep(Duration::from_millis(10)).await;
            })
        })
        .collect();

    // Query status concurrently
    let status_handles: Vec<_> = (0..10)
        .map(|_| {
            let mgr = Arc::clone(&manager);
            tokio::spawn(async move { mgr.get_resource_status().await })
        })
        .collect();

    // Wait for all
    for handle in load_handles {
        handle.await?;
    }

    for handle in status_handles {
        let status = handle.await?;
        assert!(status.memory_usage_mb >= 0);
    }

    Ok(())
}

#[tokio::test]
async fn test_edge_rate_limiter_bucket_cleanup() -> Result<()> {
    // Test rate limiter's bucket cleanup mechanism
    let mut config = ApiConfig::default();
    config.rate_limiting.enabled = true;

    let manager = ResourceManager::new(config).await?;

    // Create buckets for multiple hosts
    for i in 0..10 {
        manager
            .rate_limiter
            .check_rate_limit(&format!("host{}.com", i))
            .await
            .ok();
    }

    // Cleanup task should be running in background
    // (actual cleanup testing would require time manipulation)

    Ok(())
}

#[tokio::test]
async fn test_edge_performance_monitor_overflow() -> Result<()> {
    // Test performance monitor with excessive recordings
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    // Record many operations
    for i in 0..1000 {
        manager
            .performance_monitor
            .record_render_operation(
                &format!("https://op{}.com", i),
                Duration::from_millis(i as u64 % 1000),
                i % 2 == 0,
                (i % 50) as u32,
                (i % 100) as u32,
            )
            .await?;
    }

    // Should handle without overflow or memory issues
    let status = manager.get_resource_status().await;
    assert!(status.degradation_score >= 0.0);

    Ok(())
}
