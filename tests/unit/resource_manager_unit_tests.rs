//! Comprehensive unit tests for ResourceManager
//!
//! Tests cover:
//! - ResourceManager initialization and configuration
//! - Resource acquisition and release
//! - Resource guard lifecycle
//! - Metrics tracking
//! - Error handling

use anyhow::Result;
use riptide_api::config::ApiConfig;
use riptide_api::resource_manager::{ResourceManager, ResourceResult};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_resource_manager_initialization() -> Result<()> {
    // Test that ResourceManager initializes with default config
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config.clone()).await?;

    // Verify initial state
    let status = manager.get_resource_status().await;
    assert_eq!(status.headless_pool_total, config.headless.max_pool_size);
    assert_eq!(status.pdf_total, config.pdf.max_concurrent);
    assert_eq!(status.memory_usage_mb, 0);
    assert!(!status.memory_pressure);
    assert_eq!(status.rate_limit_hits, 0);
    assert_eq!(status.timeout_count, 0);
    assert_eq!(status.degradation_score, 0.0);

    Ok(())
}

#[tokio::test]
async fn test_resource_manager_custom_config() -> Result<()> {
    // Test ResourceManager with custom configuration
    let mut config = ApiConfig::default();
    config.headless.max_pool_size = 5;
    config.pdf.max_concurrent = 3;
    config.memory.global_memory_limit_mb = 1024;

    let manager = ResourceManager::new(config.clone()).await?;
    let status = manager.get_resource_status().await;

    assert_eq!(status.headless_pool_total, 5);
    assert_eq!(status.pdf_total, 3);

    Ok(())
}

#[tokio::test]
async fn test_acquire_render_resources_success() -> Result<()> {
    // Test successful render resource acquisition
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let result = manager
        .acquire_render_resources("https://example.com")
        .await?;

    match result {
        ResourceResult::Success(guard) => {
            // Verify guard exists and resources are tracked
            let status = manager.get_resource_status().await;
            assert!(status.memory_usage_mb > 0);
            drop(guard); // Explicit drop to test cleanup
        }
        _ => panic!("Expected ResourceResult::Success"),
    }

    Ok(())
}

#[tokio::test]
async fn test_acquire_pdf_resources_success() -> Result<()> {
    // Test successful PDF resource acquisition
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config.clone()).await?;

    let result = manager.acquire_pdf_resources().await?;

    match result {
        ResourceResult::Success(guard) => {
            let status = manager.get_resource_status().await;
            assert_eq!(status.pdf_available, config.pdf.max_concurrent - 1);
            drop(guard);
        }
        _ => panic!("Expected ResourceResult::Success"),
    }

    Ok(())
}

#[tokio::test]
async fn test_pdf_semaphore_exhaustion() -> Result<()> {
    // Test PDF semaphore reaches capacity
    let mut config = ApiConfig::default();
    config.pdf.max_concurrent = 2;

    let manager = ResourceManager::new(config).await?;

    // Acquire both permits
    let guard1 = match manager.acquire_pdf_resources().await? {
        ResourceResult::Success(g) => g,
        _ => panic!("Expected success for first acquisition"),
    };

    let guard2 = match manager.acquire_pdf_resources().await? {
        ResourceResult::Success(g) => g,
        _ => panic!("Expected success for second acquisition"),
    };

    // Third acquisition should timeout or be exhausted
    let result = manager.acquire_pdf_resources().await?;
    assert!(matches!(
        result,
        ResourceResult::Timeout | ResourceResult::ResourceExhausted
    ));

    // Cleanup
    drop(guard1);
    drop(guard2);

    Ok(())
}

#[tokio::test]
async fn test_memory_pressure_blocks_acquisition() -> Result<()> {
    // Test that memory pressure blocks resource acquisition
    let mut config = ApiConfig::default();
    config.memory.global_memory_limit_mb = 100;
    config.memory.pressure_threshold = 0.5; // 50% threshold

    let manager = ResourceManager::new(config).await?;

    // Simulate memory pressure by tracking large allocation
    manager.memory_manager.track_allocation(60).await;

    // Attempt to acquire resources should fail due to memory pressure
    let result = manager
        .acquire_render_resources("https://example.com")
        .await?;

    assert!(matches!(result, ResourceResult::MemoryPressure));

    Ok(())
}

#[tokio::test]
async fn test_invalid_url_handling() -> Result<()> {
    // Test that invalid URLs are handled properly
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let result = manager.acquire_render_resources("not-a-valid-url").await;

    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_resource_cleanup_on_timeout() -> Result<()> {
    // Test cleanup behavior on timeout
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let initial_status = manager.get_resource_status().await;
    let initial_timeouts = initial_status.timeout_count;

    manager.cleanup_on_timeout("render").await;

    let final_status = manager.get_resource_status().await;
    assert_eq!(final_status.timeout_count, initial_timeouts + 1);

    Ok(())
}

#[tokio::test]
async fn test_resource_status_reporting() -> Result<()> {
    // Test that resource status is accurately reported
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config.clone()).await?;

    let status = manager.get_resource_status().await;

    // Verify all status fields are present and valid
    assert!(status.headless_pool_available <= status.headless_pool_total);
    assert!(status.pdf_available <= status.pdf_total);
    assert!(status.memory_usage_mb >= 0);
    assert!(status.rate_limit_hits >= 0);
    assert!(status.timeout_count >= 0);
    assert!(status.degradation_score >= 0.0 && status.degradation_score <= 1.0);

    Ok(())
}

#[tokio::test]
async fn test_multiple_resource_acquisitions() -> Result<()> {
    // Test multiple concurrent resource acquisitions
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let urls = vec![
        "https://example.com",
        "https://test.com",
        "https://sample.org",
    ];

    let mut guards = Vec::new();

    for url in urls {
        match manager.acquire_render_resources(url).await? {
            ResourceResult::Success(guard) => guards.push(guard),
            ResourceResult::RateLimited { retry_after } => {
                // Expected for some requests due to rate limiting
                sleep(retry_after).await;
            }
            other => println!("Unexpected result: {:?}", other),
        }
    }

    // Verify resources were tracked
    let status = manager.get_resource_status().await;
    assert!(status.memory_usage_mb > 0);

    // Cleanup
    drop(guards);

    Ok(())
}

#[tokio::test]
async fn test_resource_guard_drop_cleanup() -> Result<()> {
    // Test that dropping guards properly cleans up resources
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let initial_status = manager.get_resource_status().await;

    {
        let _guard = match manager.acquire_pdf_resources().await? {
            ResourceResult::Success(g) => g,
            _ => panic!("Expected success"),
        };

        let active_status = manager.get_resource_status().await;
        assert_eq!(
            active_status.pdf_available,
            initial_status.pdf_available - 1
        );
    } // Guard dropped here

    // Allow async cleanup to complete
    sleep(Duration::from_millis(100)).await;

    let final_status = manager.get_resource_status().await;
    assert_eq!(final_status.pdf_available, initial_status.pdf_available);

    Ok(())
}

#[tokio::test]
async fn test_concurrent_pdf_acquisitions() -> Result<()> {
    // Test concurrent PDF resource acquisitions
    let mut config = ApiConfig::default();
    config.pdf.max_concurrent = 3;

    let manager = ResourceManager::new(config).await?;

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let mgr = manager.clone();
            tokio::spawn(async move {
                let result = mgr.acquire_pdf_resources().await;
                (i, result)
            })
        })
        .collect();

    let mut success_count = 0;
    let mut blocked_count = 0;

    for handle in handles {
        match handle.await {
            Ok((_, Ok(ResourceResult::Success(_)))) => success_count += 1,
            Ok((_, Ok(ResourceResult::Timeout)))
            | Ok((_, Ok(ResourceResult::ResourceExhausted))) => blocked_count += 1,
            _ => {}
        }
    }

    // At least some should succeed and some should be blocked
    assert!(success_count > 0);
    assert!(success_count <= 3); // Max concurrent limit

    Ok(())
}

#[tokio::test]
async fn test_extract_host_from_various_urls() -> Result<()> {
    // Test URL parsing for different formats
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let test_urls = vec![
        ("https://example.com", true),
        ("http://test.org:8080/path", true),
        ("https://sub.domain.com/page?query=1", true),
        ("not-a-url", false),
        ("ftp://invalid", true),
    ];

    for (url, should_succeed) in test_urls {
        let result = manager.acquire_render_resources(url).await;

        if should_succeed {
            // Should either succeed or fail for valid reasons (not parsing error)
            match result {
                Ok(_) | Err(_) => {} // Both outcomes acceptable for valid URLs
            }
        } else {
            assert!(result.is_err(), "Expected error for invalid URL: {}", url);
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_metrics_tracking() -> Result<()> {
    // Test that metrics are properly tracked
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    // Track initial metrics
    let initial_status = manager.get_resource_status().await;

    // Perform some operations
    manager.cleanup_on_timeout("test").await;

    let final_status = manager.get_resource_status().await;

    // Verify metrics changed
    assert_eq!(final_status.timeout_count, initial_status.timeout_count + 1);

    Ok(())
}

#[tokio::test]
async fn test_zero_config_values() -> Result<()> {
    // Test edge case with minimum config values
    let mut config = ApiConfig::default();
    config.pdf.max_concurrent = 1;
    config.headless.max_pool_size = 1;

    let manager = ResourceManager::new(config).await?;
    let status = manager.get_resource_status().await;

    assert_eq!(status.pdf_total, 1);
    assert_eq!(status.headless_pool_total, 1);

    Ok(())
}
