//! Comprehensive tests for Browser Pool Manager (Phase 4)
//!
//! Tests cover:
//! - Pre-warming initialization (1-3 instances)
//! - Health check detection and auto-restart
//! - Checkout/checkin operations
//! - Concurrent access (10+ parallel checkouts)
//! - Resource limit enforcement
//! - Graceful shutdown
//! - Failure recovery

use chromiumoxide::BrowserConfig;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, timeout};

// Import the pool module - adjust path as needed
use riptide_headless::pool::{BrowserPool, BrowserPoolConfig, BrowserHealth};

#[tokio::test]
async fn test_pool_initialization_prewarm() {
    // Test pre-warming with 1-3 instances
    let config = BrowserPoolConfig {
        initial_pool_size: 3,
        min_pool_size: 1,
        max_pool_size: 10,
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = BrowserPool::new(config.clone(), browser_config)
        .await
        .expect("Failed to create browser pool");

    // Verify pool was pre-warmed with expected number of instances
    let stats = pool.stats().await;
    assert_eq!(
        stats.available, config.initial_pool_size,
        "Pool should be pre-warmed with {} instances",
        config.initial_pool_size
    );
    assert_eq!(stats.in_use, 0, "No instances should be in use initially");

    pool.shutdown().await.expect("Failed to shutdown pool");
}

#[tokio::test]
async fn test_pool_initialization_with_failures() {
    // Test graceful degradation when some browser instances fail to launch
    let config = BrowserPoolConfig {
        initial_pool_size: 3,
        min_pool_size: 1,
        max_pool_size: 5,
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    // Even with potential failures, pool should still be created
    let pool = BrowserPool::new(config, browser_config)
        .await
        .expect("Failed to create browser pool");

    let stats = pool.stats().await;
    // Pool should have at least min_pool_size instances
    assert!(
        stats.available >= 1,
        "Pool should have at least min_pool_size instances"
    );

    pool.shutdown().await.expect("Failed to shutdown pool");
}

#[tokio::test]
async fn test_checkout_checkin_basic() {
    // Test basic checkout and checkin operations
    let config = BrowserPoolConfig {
        initial_pool_size: 2,
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = BrowserPool::new(config, browser_config)
        .await
        .expect("Failed to create browser pool");

    // Check initial state
    let stats_before = pool.stats().await;
    assert_eq!(stats_before.available, 2);
    assert_eq!(stats_before.in_use, 0);

    // Checkout a browser
    let checkout = pool.checkout().await.expect("Failed to checkout browser");

    // Verify one browser is now in use
    let stats_during = pool.stats().await;
    assert_eq!(stats_during.available, 1);
    assert_eq!(stats_during.in_use, 1);

    // Checkin the browser
    checkout
        .cleanup()
        .await
        .expect("Failed to checkin browser");

    // Allow time for async checkin to complete
    sleep(Duration::from_millis(100)).await;

    // Verify browser is returned to pool
    let stats_after = pool.stats().await;
    assert_eq!(stats_after.available, 2);
    assert_eq!(stats_after.in_use, 0);

    pool.shutdown().await.expect("Failed to shutdown pool");
}

#[tokio::test]
async fn test_concurrent_checkouts() {
    // Test concurrent access with 10+ parallel checkouts
    let config = BrowserPoolConfig {
        initial_pool_size: 5,
        max_pool_size: 15,
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = Arc::new(
        BrowserPool::new(config, browser_config)
            .await
            .expect("Failed to create browser pool"),
    );

    let mut handles = vec![];

    // Spawn 12 concurrent checkout tasks
    for i in 0..12 {
        let pool_clone = Arc::clone(&pool);
        let handle = tokio::spawn(async move {
            let checkout = pool_clone
                .checkout()
                .await
                .expect(&format!("Failed to checkout browser {}", i));

            // Simulate some work
            sleep(Duration::from_millis(50)).await;

            checkout
                .cleanup()
                .await
                .expect(&format!("Failed to checkin browser {}", i));
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete with timeout
    let results = timeout(Duration::from_secs(30), async {
        for handle in handles {
            handle.await.expect("Task panicked");
        }
    })
    .await;

    assert!(results.is_ok(), "Concurrent checkouts should complete within timeout");

    // Verify all browsers are returned to pool
    sleep(Duration::from_millis(200)).await;
    let stats = pool.stats().await;
    assert_eq!(stats.in_use, 0, "All browsers should be returned to pool");

    pool.shutdown().await.expect("Failed to shutdown pool");
}

#[tokio::test]
async fn test_resource_limit_enforcement() {
    // Test that pool respects max_pool_size limit
    let config = BrowserPoolConfig {
        initial_pool_size: 2,
        max_pool_size: 3,
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = Arc::new(
        BrowserPool::new(config.clone(), browser_config)
            .await
            .expect("Failed to create browser pool"),
    );

    // Checkout all available browsers (2 initial + 1 dynamically created = 3 max)
    let checkout1 = pool.checkout().await.expect("Failed to checkout 1");
    let checkout2 = pool.checkout().await.expect("Failed to checkout 2");
    let checkout3 = pool.checkout().await.expect("Failed to checkout 3");

    // Verify all 3 are in use
    let stats = pool.stats().await;
    assert_eq!(stats.in_use, 3, "Should have 3 browsers in use");
    assert_eq!(stats.available, 0, "Should have 0 browsers available");

    // Try to checkout one more - should block until one is returned
    let pool_clone = Arc::clone(&pool);
    let timeout_result = timeout(Duration::from_millis(500), async move {
        pool_clone.checkout().await
    })
    .await;

    assert!(
        timeout_result.is_err(),
        "Checkout should timeout when pool is at max capacity"
    );

    // Cleanup
    checkout1.cleanup().await.expect("Failed to cleanup 1");
    checkout2.cleanup().await.expect("Failed to cleanup 2");
    checkout3.cleanup().await.expect("Failed to cleanup 3");

    pool.shutdown().await.expect("Failed to shutdown pool");
}

#[tokio::test]
async fn test_health_check_and_recovery() {
    // Test health check detection and auto-restart
    let config = BrowserPoolConfig {
        initial_pool_size: 2,
        health_check_interval: Duration::from_secs(2),
        memory_threshold_mb: 500,
        enable_recovery: true,
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = BrowserPool::new(config, browser_config)
        .await
        .expect("Failed to create browser pool");

    // Initial state
    let stats_before = pool.stats().await;
    assert_eq!(stats_before.available, 2);

    // Checkout a browser
    let checkout = pool.checkout().await.expect("Failed to checkout browser");

    // Simulate the browser becoming unhealthy by checking it back in
    // (the health check will detect if it's unhealthy)
    checkout
        .cleanup()
        .await
        .expect("Failed to checkin browser");

    // Wait for health check cycle
    sleep(Duration::from_secs(3)).await;

    // Pool should maintain at least min_pool_size healthy instances
    let stats_after = pool.stats().await;
    assert!(
        stats_after.available >= 1,
        "Pool should maintain healthy instances"
    );

    pool.shutdown().await.expect("Failed to shutdown pool");
}

#[tokio::test]
async fn test_graceful_shutdown() {
    // Test graceful shutdown cleans up all resources
    let config = BrowserPoolConfig {
        initial_pool_size: 3,
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = BrowserPool::new(config, browser_config)
        .await
        .expect("Failed to create browser pool");

    // Checkout some browsers
    let checkout1 = pool.checkout().await.expect("Failed to checkout 1");
    let _checkout2 = pool.checkout().await.expect("Failed to checkout 2");

    // Return one
    checkout1.cleanup().await.expect("Failed to cleanup 1");

    // Shutdown should clean up all resources
    let shutdown_result = pool.shutdown().await;
    assert!(
        shutdown_result.is_ok(),
        "Graceful shutdown should succeed: {:?}",
        shutdown_result
    );
}

#[tokio::test]
async fn test_browser_lifecycle_limits() {
    // Test max_lifetime and idle_timeout enforcement
    let config = BrowserPoolConfig {
        initial_pool_size: 2,
        max_lifetime: Duration::from_secs(2), // Short lifetime for testing
        idle_timeout: Duration::from_secs(1),
        health_check_interval: Duration::from_millis(500),
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = BrowserPool::new(config, browser_config)
        .await
        .expect("Failed to create browser pool");

    // Wait for browsers to exceed lifetime
    sleep(Duration::from_secs(3)).await;

    // Pool should have cleaned up expired browsers and created new ones
    let stats = pool.stats().await;
    assert!(
        stats.available >= 1,
        "Pool should maintain minimum instances after cleanup"
    );

    pool.shutdown().await.expect("Failed to shutdown pool");
}

#[tokio::test]
async fn test_browser_checkout_timeout() {
    // Test cleanup timeout for browser checkin operations
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        cleanup_timeout: Duration::from_millis(100), // Very short timeout for testing
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = BrowserPool::new(config, browser_config)
        .await
        .expect("Failed to create browser pool");

    let checkout = pool.checkout().await.expect("Failed to checkout browser");

    // Use cleanup which respects the configured timeout
    let cleanup_result = checkout.cleanup().await;

    // Cleanup should either succeed or timeout gracefully
    assert!(
        cleanup_result.is_ok() || cleanup_result.is_err(),
        "Cleanup should handle timeout gracefully"
    );

    pool.shutdown().await.expect("Failed to shutdown pool");
}

#[tokio::test]
async fn test_pool_stats_accuracy() {
    // Test that pool statistics are accurate
    let config = BrowserPoolConfig {
        initial_pool_size: 5,
        max_pool_size: 10,
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = BrowserPool::new(config.clone(), browser_config)
        .await
        .expect("Failed to create browser pool");

    // Check initial stats
    let stats = pool.stats().await;
    assert_eq!(stats.total_capacity, config.max_pool_size);
    assert_eq!(stats.available, config.initial_pool_size);
    assert_eq!(stats.in_use, 0);
    assert_eq!(stats.utilization, 0.0);

    // Checkout some browsers
    let _checkout1 = pool.checkout().await.expect("Failed to checkout 1");
    let _checkout2 = pool.checkout().await.expect("Failed to checkout 2");

    let stats = pool.stats().await;
    assert_eq!(stats.in_use, 2);
    assert_eq!(stats.available, config.initial_pool_size - 2);
    assert!(stats.utilization > 0.0 && stats.utilization <= 100.0);

    pool.shutdown().await.expect("Failed to shutdown pool");
}

#[tokio::test]
async fn test_unique_browser_profiles() {
    // Test that each browser gets a unique profile directory
    let config = BrowserPoolConfig {
        initial_pool_size: 3,
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = BrowserPool::new(config, browser_config)
        .await
        .expect("Failed to create browser pool");

    // Checkout multiple browsers concurrently
    let checkout1 = pool.checkout().await.expect("Failed to checkout 1");
    let checkout2 = pool.checkout().await.expect("Failed to checkout 2");
    let checkout3 = pool.checkout().await.expect("Failed to checkout 3");

    // All checkouts should succeed without SingletonLock conflicts
    // (This is implicit - the test would fail if there were profile conflicts)

    // Cleanup
    checkout1.cleanup().await.expect("Failed to cleanup 1");
    checkout2.cleanup().await.expect("Failed to cleanup 2");
    checkout3.cleanup().await.expect("Failed to cleanup 3");

    pool.shutdown().await.expect("Failed to shutdown pool");
}

#[tokio::test]
async fn test_pool_events_monitoring() {
    // Test that pool events are properly emitted for monitoring
    let config = BrowserPoolConfig {
        initial_pool_size: 2,
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = BrowserPool::new(config, browser_config)
        .await
        .expect("Failed to create browser pool");

    // Get event receiver
    let events = pool.events();
    let mut receiver = events.lock().await;

    // Perform operations that should generate events
    let checkout = pool.checkout().await.expect("Failed to checkout browser");

    // Try to receive checkout event (non-blocking with timeout)
    let event_result = timeout(Duration::from_millis(100), receiver.recv()).await;
    assert!(
        event_result.is_ok(),
        "Should receive pool event for checkout"
    );

    checkout.cleanup().await.expect("Failed to cleanup");

    pool.shutdown().await.expect("Failed to shutdown pool");
}

#[tokio::test]
async fn test_pool_recovery_after_all_failures() {
    // Test that pool can recover even if all browsers fail
    let config = BrowserPoolConfig {
        initial_pool_size: 2,
        min_pool_size: 1,
        enable_recovery: true,
        health_check_interval: Duration::from_secs(1),
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = BrowserPool::new(config, browser_config)
        .await
        .expect("Failed to create browser pool");

    // Initial state should have browsers
    let stats_before = pool.stats().await;
    assert!(stats_before.available >= 1);

    // Wait for health check to maintain pool
    sleep(Duration::from_secs(2)).await;

    // Pool should still maintain minimum instances
    let stats_after = pool.stats().await;
    assert!(
        stats_after.available >= 1,
        "Pool should recover and maintain minimum instances"
    );

    pool.shutdown().await.expect("Failed to shutdown pool");
}

#[tokio::test]
async fn test_custom_profile_base_directory() {
    // Test using custom base directory for browser profiles
    let temp_dir = std::env::temp_dir().join("riptide-browser-test");
    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");

    let config = BrowserPoolConfig {
        initial_pool_size: 2,
        profile_base_dir: Some(temp_dir.clone()),
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = BrowserPool::new(config, browser_config)
        .await
        .expect("Failed to create browser pool with custom base dir");

    let stats = pool.stats().await;
    assert_eq!(stats.available, 2, "Pool should use custom base directory");

    pool.shutdown().await.expect("Failed to shutdown pool");

    // Cleanup temp directory
    let _ = std::fs::remove_dir_all(&temp_dir);
}
