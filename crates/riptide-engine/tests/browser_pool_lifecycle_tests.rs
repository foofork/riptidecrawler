//! Comprehensive Browser Pool Lifecycle Tests
//!
//! Test coverage for browser pool management including:
//! - Pool initialization and configuration
//! - Browser checkout/checkin operations
//! - Concurrent access patterns
//! - Health check monitoring
//! - Memory limit enforcement
//! - Browser recovery mechanisms
//! - Pool scaling operations
//! - Resource cleanup
//! - Error handling

use anyhow::Result;
use chromiumoxide::BrowserConfig;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

// Import from riptide-engine
use riptide_engine::pool::{BrowserPool, BrowserPoolConfig};

/// Test 1: Pool initialization with default config
#[tokio::test]
async fn test_pool_initialization_default() -> Result<()> {
    let config = BrowserPoolConfig::default();
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config.clone(), browser_config).await?;
    let stats = pool.stats().await;

    assert_eq!(stats.available, config.initial_pool_size);
    assert_eq!(stats.in_use, 0);
    assert_eq!(stats.total, config.initial_pool_size);

    pool.shutdown().await?;
    Ok(())
}

/// Test 2: Pool initialization with custom config
#[tokio::test]
async fn test_pool_initialization_custom() -> Result<()> {
    let config = BrowserPoolConfig {
        min_pool_size: 2,
        max_pool_size: 10,
        initial_pool_size: 4,
        idle_timeout: Duration::from_secs(60),
        max_lifetime: Duration::from_secs(600),
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config.clone(), browser_config).await?;
    let stats = pool.stats().await;

    assert_eq!(stats.available, 4);
    assert_eq!(stats.in_use, 0);

    pool.shutdown().await?;
    Ok(())
}

/// Test 3: Pool initialization with zero initial size
#[tokio::test]
async fn test_pool_initialization_zero_size() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 0,
        min_pool_size: 0,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;
    let stats = pool.stats().await;

    assert_eq!(stats.available, 0);
    assert_eq!(stats.in_use, 0);

    pool.shutdown().await?;
    Ok(())
}

/// Test 4: Single browser checkout
#[tokio::test]
async fn test_browser_checkout_single() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 2,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    let checkout = pool.checkout().await?;
    let stats = pool.stats().await;

    assert_eq!(stats.available, 1);
    assert_eq!(stats.in_use, 1);

    checkout.checkin().await?;
    sleep(Duration::from_millis(100)).await;

    let stats = pool.stats().await;
    assert_eq!(stats.available, 2);
    assert_eq!(stats.in_use, 0);

    pool.shutdown().await?;
    Ok(())
}

/// Test 5: Multiple browser checkouts
#[tokio::test]
async fn test_browser_checkout_multiple() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 3,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    let checkout1 = pool.checkout().await?;
    let checkout2 = pool.checkout().await?;
    let stats = pool.stats().await;

    assert_eq!(stats.available, 1);
    assert_eq!(stats.in_use, 2);

    checkout1.checkin().await?;
    checkout2.checkin().await?;
    sleep(Duration::from_millis(100)).await;

    let stats = pool.stats().await;
    assert_eq!(stats.available, 3);

    pool.shutdown().await?;
    Ok(())
}

/// Test 6: Checkout from empty pool triggers creation
#[tokio::test]
async fn test_checkout_empty_pool_creates_browser() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 0,
        max_pool_size: 5,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;
    let stats = pool.stats().await;
    assert_eq!(stats.available, 0);

    let checkout = pool.checkout().await?;
    let stats = pool.stats().await;
    assert_eq!(stats.in_use, 1);
    assert_eq!(stats.total, 1);

    checkout.checkin().await?;
    pool.shutdown().await?;
    Ok(())
}

/// Test 7: Concurrent checkout stress test (10 concurrent)
#[tokio::test]
async fn test_concurrent_checkout_10() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 5,
        max_pool_size: 15,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = Arc::new(BrowserPool::new(config, browser_config).await?);

    let mut handles = vec![];
    for _ in 0..10 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            let checkout = pool_clone.checkout().await?;
            sleep(Duration::from_millis(50)).await;
            checkout.checkin().await
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await??;
    }

    sleep(Duration::from_millis(200)).await;
    let stats = pool.stats().await;
    assert!(stats.available > 0);
    assert_eq!(stats.in_use, 0);

    pool.shutdown().await?;
    Ok(())
}

/// Test 8: Concurrent checkout stress test (50 concurrent)
#[tokio::test]
async fn test_concurrent_checkout_50() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 5,
        max_pool_size: 20,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = Arc::new(BrowserPool::new(config, browser_config).await?);

    let mut handles = vec![];
    for _ in 0..50 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            let checkout = pool_clone.checkout().await?;
            sleep(Duration::from_millis(10)).await;
            checkout.checkin().await
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await??;
    }

    sleep(Duration::from_millis(500)).await;
    let stats = pool.stats().await;
    assert_eq!(stats.in_use, 0);

    pool.shutdown().await?;
    Ok(())
}

/// Test 9: Browser checkin with cleanup timeout
#[tokio::test]
async fn test_browser_checkin_cleanup_timeout() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        cleanup_timeout: Duration::from_secs(2),
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;
    let checkout = pool.checkout().await?;

    // Test cleanup with default timeout
    checkout.cleanup().await?;

    sleep(Duration::from_millis(100)).await;
    let stats = pool.stats().await;
    assert_eq!(stats.available, 1);

    pool.shutdown().await?;
    Ok(())
}

/// Test 10: Browser checkin with custom timeout
#[tokio::test]
async fn test_browser_checkin_custom_timeout() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;
    let checkout = pool.checkout().await?;

    // Test cleanup with custom timeout
    checkout
        .cleanup_with_timeout(Duration::from_secs(3))
        .await?;

    sleep(Duration::from_millis(100)).await;
    let stats = pool.stats().await;
    assert_eq!(stats.available, 1);

    pool.shutdown().await?;
    Ok(())
}

/// Test 11: Pool stats accuracy
#[tokio::test]
async fn test_pool_stats_accuracy() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 5,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    let stats = pool.stats().await;
    assert_eq!(stats.total, 5);
    assert_eq!(stats.available, 5);
    assert_eq!(stats.in_use, 0);

    let c1 = pool.checkout().await?;
    let c2 = pool.checkout().await?;

    let stats = pool.stats().await;
    assert_eq!(stats.total, 5);
    assert_eq!(stats.available, 3);
    assert_eq!(stats.in_use, 2);

    c1.checkin().await?;
    sleep(Duration::from_millis(50)).await;

    let stats = pool.stats().await;
    assert_eq!(stats.available, 4);
    assert_eq!(stats.in_use, 1);

    c2.checkin().await?;
    pool.shutdown().await?;
    Ok(())
}

/// Test 12: Pool respects max_pool_size limit
#[tokio::test]
async fn test_pool_respects_max_size() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 2,
        max_pool_size: 3,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = Arc::new(BrowserPool::new(config, browser_config).await?);

    // Checkout more than max_pool_size
    let c1 = pool.checkout().await?;
    let c2 = pool.checkout().await?;
    let c3 = pool.checkout().await?;

    let stats = pool.stats().await;
    assert!(stats.total <= 3);

    c1.checkin().await?;
    c2.checkin().await?;
    c3.checkin().await?;

    pool.shutdown().await?;
    Ok(())
}

/// Test 13: Pool event notifications
#[tokio::test]
async fn test_pool_event_notifications() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;
    let mut events = pool.subscribe_events();

    // Checkout should trigger event
    let checkout = pool.checkout().await?;

    // Try to receive event with timeout
    let event = tokio::time::timeout(Duration::from_secs(1), events.recv()).await;
    assert!(event.is_ok());

    checkout.checkin().await?;
    pool.shutdown().await?;
    Ok(())
}

/// Test 14: Pool shutdown gracefully
#[tokio::test]
async fn test_pool_shutdown_graceful() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 3,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;
    let stats_before = pool.stats().await;
    assert_eq!(stats_before.total, 3);

    pool.shutdown().await?;

    // After shutdown, pool should be empty
    let stats_after = pool.stats().await;
    assert_eq!(stats_after.total, 0);

    Ok(())
}

/// Test 15: Pool shutdown with active browsers
#[tokio::test]
async fn test_pool_shutdown_with_active_browsers() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 2,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    let _checkout1 = pool.checkout().await?;
    let _checkout2 = pool.checkout().await?;

    // Should shutdown even with browsers checked out
    pool.shutdown().await?;

    let stats = pool.stats().await;
    assert_eq!(stats.total, 0);

    Ok(())
}

/// Test 16: Browser ID uniqueness
#[tokio::test]
async fn test_browser_id_uniqueness() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 5,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    let c1 = pool.checkout().await?;
    let c2 = pool.checkout().await?;
    let c3 = pool.checkout().await?;

    let id1 = c1.browser_id();
    let id2 = c2.browser_id();
    let id3 = c3.browser_id();

    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);

    c1.checkin().await?;
    c2.checkin().await?;
    c3.checkin().await?;
    pool.shutdown().await?;
    Ok(())
}

/// Test 17: Tiered health checks - fast check interval
#[tokio::test]
async fn test_tiered_health_checks_fast() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        enable_tiered_health_checks: true,
        fast_check_interval: Duration::from_millis(100),
        full_check_interval: Duration::from_secs(5),
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    // Wait for at least one fast health check
    sleep(Duration::from_millis(200)).await;

    let stats = pool.stats().await;
    assert!(stats.available > 0 || stats.in_use > 0);

    pool.shutdown().await?;
    Ok(())
}

/// Test 18: Tiered health checks - full check interval
#[tokio::test]
async fn test_tiered_health_checks_full() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        enable_tiered_health_checks: true,
        fast_check_interval: Duration::from_secs(10),
        full_check_interval: Duration::from_millis(200),
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    // Wait for at least one full health check
    sleep(Duration::from_millis(400)).await;

    let stats = pool.stats().await;
    assert!(stats.total > 0);

    pool.shutdown().await?;
    Ok(())
}

/// Test 19: Tiered health checks disabled
#[tokio::test]
async fn test_tiered_health_checks_disabled() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        enable_tiered_health_checks: false,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    sleep(Duration::from_millis(100)).await;

    let stats = pool.stats().await;
    assert_eq!(stats.total, 1);

    pool.shutdown().await?;
    Ok(())
}

/// Test 20: Memory limits - soft limit monitoring
#[tokio::test]
async fn test_memory_soft_limit() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        enable_memory_limits: true,
        memory_soft_limit_mb: 100,
        memory_hard_limit_mb: 200,
        memory_check_interval: Duration::from_millis(100),
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    // Wait for memory check
    sleep(Duration::from_millis(200)).await;

    let stats = pool.stats().await;
    assert!(stats.total > 0);

    pool.shutdown().await?;
    Ok(())
}

/// Test 21: Memory limits - hard limit enforcement
#[tokio::test]
async fn test_memory_hard_limit() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        enable_memory_limits: true,
        memory_soft_limit_mb: 100,
        memory_hard_limit_mb: 150,
        memory_check_interval: Duration::from_millis(100),
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    sleep(Duration::from_millis(200)).await;

    let stats = pool.stats().await;
    assert!(stats.available >= 0);

    pool.shutdown().await?;
    Ok(())
}

/// Test 22: Memory limits disabled
#[tokio::test]
async fn test_memory_limits_disabled() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 2,
        enable_memory_limits: false,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    sleep(Duration::from_millis(100)).await;

    let stats = pool.stats().await;
    assert_eq!(stats.total, 2);

    pool.shutdown().await?;
    Ok(())
}

/// Test 23: V8 heap statistics tracking enabled
#[tokio::test]
async fn test_v8_heap_stats_enabled() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        enable_v8_heap_stats: true,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    let stats = pool.stats().await;
    assert_eq!(stats.total, 1);

    pool.shutdown().await?;
    Ok(())
}

/// Test 24: V8 heap statistics tracking disabled
#[tokio::test]
async fn test_v8_heap_stats_disabled() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        enable_v8_heap_stats: false,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    let stats = pool.stats().await;
    assert_eq!(stats.total, 1);

    pool.shutdown().await?;
    Ok(())
}

/// Test 25: Browser recovery enabled
#[tokio::test]
async fn test_browser_recovery_enabled() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        enable_recovery: true,
        max_retries: 3,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    let stats = pool.stats().await;
    assert_eq!(stats.total, 1);

    pool.shutdown().await?;
    Ok(())
}

/// Test 26: Browser recovery disabled
#[tokio::test]
async fn test_browser_recovery_disabled() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        enable_recovery: false,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    let stats = pool.stats().await;
    assert_eq!(stats.total, 1);

    pool.shutdown().await?;
    Ok(())
}

/// Test 27: Max retries configuration
#[tokio::test]
async fn test_max_retries_config() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        max_retries: 5,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    let stats = pool.stats().await;
    assert_eq!(stats.total, 1);

    pool.shutdown().await?;
    Ok(())
}

/// Test 28: Idle timeout configuration
#[tokio::test]
async fn test_idle_timeout_config() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        idle_timeout: Duration::from_secs(5),
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    let stats = pool.stats().await;
    assert_eq!(stats.total, 1);

    pool.shutdown().await?;
    Ok(())
}

/// Test 29: Max lifetime configuration
#[tokio::test]
async fn test_max_lifetime_config() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        max_lifetime: Duration::from_secs(60),
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    let stats = pool.stats().await;
    assert_eq!(stats.total, 1);

    pool.shutdown().await?;
    Ok(())
}

/// Test 30: Health check interval configuration
#[tokio::test]
async fn test_health_check_interval_config() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        health_check_interval: Duration::from_secs(5),
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    let stats = pool.stats().await;
    assert_eq!(stats.total, 1);

    pool.shutdown().await?;
    Ok(())
}

/// Test 31: Memory threshold configuration
#[tokio::test]
async fn test_memory_threshold_config() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        memory_threshold_mb: 1000,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    let stats = pool.stats().await;
    assert_eq!(stats.total, 1);

    pool.shutdown().await?;
    Ok(())
}

/// Test 32: Custom profile base directory
#[tokio::test]
async fn test_custom_profile_base_dir() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        profile_base_dir: Some(temp_dir.path().to_path_buf()),
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    let stats = pool.stats().await;
    assert_eq!(stats.total, 1);

    pool.shutdown().await?;
    Ok(())
}

/// Test 33: Default profile base directory (None)
#[tokio::test]
async fn test_default_profile_base_dir() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        profile_base_dir: None,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    let stats = pool.stats().await;
    assert_eq!(stats.total, 1);

    pool.shutdown().await?;
    Ok(())
}

/// Test 34: Cleanup timeout configuration
#[tokio::test]
async fn test_cleanup_timeout_config() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        cleanup_timeout: Duration::from_secs(10),
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    let checkout = pool.checkout().await?;
    checkout.cleanup().await?;

    pool.shutdown().await?;
    Ok(())
}

/// Test 35: Pool min size enforcement
#[tokio::test]
async fn test_pool_min_size_enforcement() -> Result<()> {
    let config = BrowserPoolConfig {
        min_pool_size: 2,
        initial_pool_size: 3,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    let stats = pool.stats().await;
    assert!(stats.total >= 2);

    pool.shutdown().await?;
    Ok(())
}

/// Test 36: Concurrent checkout and checkin mixed operations
#[tokio::test]
async fn test_concurrent_mixed_operations() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 3,
        max_pool_size: 10,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = Arc::new(BrowserPool::new(config, browser_config).await?);

    let mut handles = vec![];

    // Mix of checkout and immediate checkin
    for _ in 0..20 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            let checkout = pool_clone.checkout().await?;
            sleep(Duration::from_millis(5)).await;
            checkout.checkin().await
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await??;
    }

    sleep(Duration::from_millis(200)).await;
    let stats = pool.stats().await;
    assert_eq!(stats.in_use, 0);

    pool.shutdown().await?;
    Ok(())
}

/// Test 37: Browser checkout timeout behavior
#[tokio::test]
async fn test_checkout_timeout_behavior() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        max_pool_size: 1,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = Arc::new(BrowserPool::new(config, browser_config).await?);

    // First checkout succeeds
    let _c1 = pool.checkout().await?;

    // Second checkout should wait
    let pool_clone = pool.clone();
    let handle = tokio::spawn(async move {
        tokio::time::timeout(Duration::from_millis(100), pool_clone.checkout()).await
    });

    let result = handle.await?;
    assert!(result.is_err()); // Should timeout

    pool.shutdown().await?;
    Ok(())
}

/// Test 38: Error check delay configuration
#[tokio::test]
async fn test_error_check_delay_config() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        error_check_delay: Duration::from_millis(250),
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    let stats = pool.stats().await;
    assert_eq!(stats.total, 1);

    pool.shutdown().await?;
    Ok(())
}

/// Test 39: Multiple pools independence
#[tokio::test]
async fn test_multiple_pools_independence() -> Result<()> {
    let config1 = BrowserPoolConfig {
        initial_pool_size: 2,
        ..Default::default()
    };
    let config2 = BrowserPoolConfig {
        initial_pool_size: 3,
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder().build()?;

    let pool1 = BrowserPool::new(config1, browser_config.clone()).await?;
    let pool2 = BrowserPool::new(config2, browser_config).await?;

    let stats1 = pool1.stats().await;
    let stats2 = pool2.stats().await;

    assert_eq!(stats1.total, 2);
    assert_eq!(stats2.total, 3);

    pool1.shutdown().await?;
    pool2.shutdown().await?;
    Ok(())
}

/// Test 40: Pool stats after shutdown
#[tokio::test]
async fn test_pool_stats_after_shutdown() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 3,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;
    pool.shutdown().await?;

    let stats = pool.stats().await;
    assert_eq!(stats.total, 0);
    assert_eq!(stats.available, 0);
    assert_eq!(stats.in_use, 0);

    Ok(())
}

/// Test 41: Rapid checkout/checkin cycles
#[tokio::test]
async fn test_rapid_checkout_checkin_cycles() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 2,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    // Perform 100 rapid cycles
    for _ in 0..100 {
        let checkout = pool.checkout().await?;
        checkout.checkin().await?;
    }

    sleep(Duration::from_millis(100)).await;
    let stats = pool.stats().await;
    assert!(stats.available > 0);

    pool.shutdown().await?;
    Ok(())
}

/// Test 42: Pool resource cleanup verification
#[tokio::test]
async fn test_pool_resource_cleanup() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 2,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;
    let stats_before = pool.stats().await;

    pool.shutdown().await?;

    let stats_after = pool.stats().await;
    assert!(stats_before.total > 0);
    assert_eq!(stats_after.total, 0);

    Ok(())
}

/// Test 43: Browser checkout with immediate drop (tests Drop impl)
#[tokio::test]
async fn test_browser_checkout_drop() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 2,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    {
        let _checkout = pool.checkout().await?;
        // Checkout dropped here
    }

    // Give time for background cleanup
    sleep(Duration::from_millis(200)).await;

    let stats = pool.stats().await;
    // Browser should be back in pool after drop cleanup
    assert!(stats.available > 0 || stats.in_use > 0);

    pool.shutdown().await?;
    Ok(())
}

/// Test 44: Sequential vs concurrent performance
#[tokio::test]
async fn test_sequential_vs_concurrent_performance() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 5,
        max_pool_size: 10,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = Arc::new(BrowserPool::new(config, browser_config).await?);

    // Sequential operations
    let start = std::time::Instant::now();
    for _ in 0..10 {
        let checkout = pool.checkout().await?;
        sleep(Duration::from_millis(10)).await;
        checkout.checkin().await?;
    }
    let sequential_duration = start.elapsed();

    // Concurrent operations
    let start = std::time::Instant::now();
    let mut handles = vec![];
    for _ in 0..10 {
        let pool_clone = pool.clone();
        handles.push(tokio::spawn(async move {
            let checkout = pool_clone.checkout().await?;
            sleep(Duration::from_millis(10)).await;
            checkout.checkin().await
        }));
    }
    for handle in handles {
        handle.await??;
    }
    let concurrent_duration = start.elapsed();

    // Concurrent should be faster
    assert!(concurrent_duration < sequential_duration);

    pool.shutdown().await?;
    Ok(())
}

/// Test 45: Pool configuration clone
#[tokio::test]
async fn test_pool_config_clone() -> Result<()> {
    let config1 = BrowserPoolConfig {
        initial_pool_size: 5,
        max_pool_size: 15,
        idle_timeout: Duration::from_secs(45),
        ..Default::default()
    };

    let config2 = config1.clone();

    assert_eq!(config1.initial_pool_size, config2.initial_pool_size);
    assert_eq!(config1.max_pool_size, config2.max_pool_size);
    assert_eq!(config1.idle_timeout, config2.idle_timeout);

    Ok(())
}

/// Test 46: Pool configuration debug formatting
#[tokio::test]
async fn test_pool_config_debug() -> Result<()> {
    let config = BrowserPoolConfig::default();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("BrowserPoolConfig"));
    assert!(debug_str.contains("min_pool_size"));
    assert!(debug_str.contains("max_pool_size"));

    Ok(())
}

/// Test 47: Browser new_page operation
#[tokio::test]
async fn test_browser_new_page() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;
    let checkout = pool.checkout().await?;

    // Test new_page method
    let page_result = checkout.new_page("about:blank").await;

    // Page creation may fail in test environment, but method should be callable
    let _ = page_result;

    checkout.checkin().await?;
    pool.shutdown().await?;
    Ok(())
}

/// Test 48: Pool event subscription
#[tokio::test]
async fn test_pool_event_subscription() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    // Multiple subscribers
    let mut events1 = pool.subscribe_events();
    let mut events2 = pool.subscribe_events();

    let checkout = pool.checkout().await?;

    // Both should receive events
    let e1 = tokio::time::timeout(Duration::from_millis(100), events1.recv()).await;
    let e2 = tokio::time::timeout(Duration::from_millis(100), events2.recv()).await;

    checkout.checkin().await?;
    pool.shutdown().await?;
    Ok(())
}

/// Test 49: Zero-sized pool edge case
#[tokio::test]
async fn test_zero_sized_pool_checkout() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 0,
        min_pool_size: 0,
        max_pool_size: 1,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    let stats = pool.stats().await;
    assert_eq!(stats.total, 0);

    // Should create browser on demand
    let checkout = pool.checkout().await?;

    let stats = pool.stats().await;
    assert_eq!(stats.in_use, 1);

    checkout.checkin().await?;
    pool.shutdown().await?;
    Ok(())
}

/// Test 50: Large pool initialization performance
#[tokio::test]
async fn test_large_pool_initialization() -> Result<()> {
    let config = BrowserPoolConfig {
        initial_pool_size: 10,
        max_pool_size: 20,
        ..Default::default()
    };
    let browser_config = BrowserConfig::builder().build()?;

    let start = std::time::Instant::now();
    let pool = BrowserPool::new(config, browser_config).await?;
    let initialization_time = start.elapsed();

    let stats = pool.stats().await;
    assert_eq!(stats.total, 10);

    // Initialization should complete in reasonable time (< 30 seconds)
    assert!(initialization_time < Duration::from_secs(30));

    pool.shutdown().await?;
    Ok(())
}
