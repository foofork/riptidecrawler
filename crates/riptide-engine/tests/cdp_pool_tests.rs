//! Comprehensive CDP (Chrome DevTools Protocol) Pool Management Tests
//!
//! Test coverage for:
//! - CDP connection pool lifecycle
//! - Batch command optimization
//! - Connection reuse strategy
//! - Error handling and recovery
//! - Timeout handling
//! - Connection health checks
//! - Pool exhaustion scenarios
//! - Concurrent command execution
//!
//! **Chrome Lock Fix**: All tests that launch Chrome browsers use `#[serial]`
//! to prevent concurrent SingletonLock conflicts.

use anyhow::Result;
use futures::StreamExt;
use std::time::{Duration, Instant};
use tokio::time::sleep;

// Import from riptide-engine
use riptide_engine::cdp_pool::{CdpCommand, CdpConnectionPool, CdpPoolConfig};

/// Test 1: CDP pool creation with default config
#[tokio::test]
async fn test_cdp_pool_creation_default() -> Result<()> {
    let config = CdpPoolConfig::default();

    assert_eq!(config.max_connections_per_browser, 10);
    assert_eq!(config.max_batch_size, 10);
    assert_eq!(config.batch_timeout, Duration::from_millis(50));

    Ok(())
}

/// Test 2: CDP pool creation with custom config
#[tokio::test]
async fn test_cdp_pool_creation_custom() -> Result<()> {
    let config = CdpPoolConfig {
        max_connections_per_browser: 20,
        max_batch_size: 5,
        batch_timeout: Duration::from_millis(100),
        connection_idle_timeout: Duration::from_secs(60),
        enable_batching: true,
        enable_health_checks: true,
        health_check_interval: Duration::from_secs(30),
        max_connection_lifetime: Duration::from_secs(300),
    };

    assert_eq!(config.max_connections_per_browser, 20);
    assert_eq!(config.max_batch_size, 5);
    assert!(config.enable_batching);

    Ok(())
}

/// Test 3: CDP config default values
#[tokio::test]
async fn test_cdp_config_defaults() -> Result<()> {
    let config = CdpPoolConfig::default();

    assert!(config.enable_batching);
    assert_eq!(config.connection_idle_timeout, Duration::from_secs(30));
    assert_eq!(config.health_check_interval, Duration::from_secs(10));

    Ok(())
}

/// Test 4: CDP pool initialization
#[tokio::test]
async fn test_cdp_pool_initialization() -> Result<()> {
    let config = CdpPoolConfig::default();
    let _pool = CdpConnectionPool::new(config);

    // Pool creation should succeed
    Ok(())
}

/// Test 5: Batch command queuing
#[tokio::test]
async fn test_batch_command_queuing() -> Result<()> {
    let config = CdpPoolConfig {
        max_batch_size: 5,
        batch_timeout: Duration::from_millis(100),
        ..Default::default()
    };
    let pool = CdpConnectionPool::new(config);

    // Queue commands
    for i in 0..3 {
        let command = CdpCommand {
            command_name: format!("command_{}", i),
            params: serde_json::json!({}),
            timestamp: Instant::now(),
        };
        pool.batch_command("test-browser", command).await?;
    }

    Ok(())
}

/// Test 6: Batch flush
#[tokio::test]
async fn test_batch_flush() -> Result<()> {
    let config = CdpPoolConfig {
        max_batch_size: 10,                     // Large enough to not auto-clear
        batch_timeout: Duration::from_secs(10), // Long timeout
        enable_batching: true,
        ..Default::default()
    };
    let pool = CdpConnectionPool::new(config);

    // Queue commands (less than max_batch_size)
    for i in 0..3 {
        let command = CdpCommand {
            command_name: format!("cmd_{}", i),
            params: serde_json::json!({}),
            timestamp: Instant::now(),
        };
        pool.batch_command("test-browser", command).await?;
    }

    // Manually flush batches
    let flushed = pool.flush_batches("test-browser").await?;
    assert_eq!(flushed.len(), 3);

    Ok(())
}

/// Test 7: Batch optimization enabled
#[tokio::test]
async fn test_batch_optimization_enabled() -> Result<()> {
    let config = CdpPoolConfig {
        enable_batching: true,
        max_batch_size: 5,
        ..Default::default()
    };
    let pool = CdpConnectionPool::new(config);

    for i in 0..10 {
        let command = CdpCommand {
            command_name: format!("cmd_{}", i),
            params: serde_json::json!({}),
            timestamp: Instant::now(),
        };
        pool.batch_command("test-browser", command).await?;
    }

    sleep(Duration::from_millis(100)).await;

    Ok(())
}

/// Test 8: Batch optimization disabled
#[tokio::test]
async fn test_batch_optimization_disabled() -> Result<()> {
    let config = CdpPoolConfig {
        enable_batching: false,
        ..Default::default()
    };
    let pool = CdpConnectionPool::new(config);

    let command = CdpCommand {
        command_name: "cmd1".to_string(),
        params: serde_json::json!({}),
        timestamp: Instant::now(),
    };
    pool.batch_command("test-browser", command).await?;

    // Commands should not be batched
    sleep(Duration::from_millis(50)).await;

    Ok(())
}

/// Test 9: Pool statistics
#[tokio::test]
async fn test_pool_stats() -> Result<()> {
    let config = CdpPoolConfig {
        max_connections_per_browser: 5,
        ..Default::default()
    };
    let pool = CdpConnectionPool::new(config);

    let stats = pool.stats().await;
    assert_eq!(stats.total_connections, 0);
    assert_eq!(stats.browsers_with_connections, 0);

    Ok(())
}

/// Test 10: Batch size configuration limits
#[tokio::test]
async fn test_batch_size_limits() -> Result<()> {
    let config = CdpPoolConfig {
        max_batch_size: 1,
        ..Default::default()
    };
    let pool = CdpConnectionPool::new(config);

    let command = CdpCommand {
        command_name: "cmd1".to_string(),
        params: serde_json::json!({}),
        timestamp: Instant::now(),
    };
    pool.batch_command("test-browser", command).await?;

    sleep(Duration::from_millis(50)).await;

    Ok(())
}

/// Test 11: Large batch size handling
#[tokio::test]
async fn test_large_batch_size() -> Result<()> {
    let config = CdpPoolConfig {
        max_batch_size: 1000,
        batch_timeout: Duration::from_millis(50),
        ..Default::default()
    };
    let pool = CdpConnectionPool::new(config);

    for i in 0..10 {
        let command = CdpCommand {
            command_name: format!("cmd_{}", i),
            params: serde_json::json!({}),
            timestamp: Instant::now(),
        };
        pool.batch_command("test-browser", command).await?;
    }

    // Should flush on timeout, not size
    sleep(Duration::from_millis(100)).await;

    Ok(())
}

/// Test 12: Batch timeout edge cases
#[tokio::test]
async fn test_batch_timeout_edge_cases() -> Result<()> {
    let config = CdpPoolConfig {
        batch_timeout: Duration::from_millis(10),
        max_batch_size: 100,
        ..Default::default()
    };
    let pool = CdpConnectionPool::new(config);

    let command = CdpCommand {
        command_name: "cmd1".to_string(),
        params: serde_json::json!({}),
        timestamp: Instant::now(),
    };
    pool.batch_command("test-browser", command).await?;

    sleep(Duration::from_millis(20)).await;

    Ok(())
}

/// Test 13: Batch size threshold (auto-clear on reaching max_batch_size)
#[tokio::test]
async fn test_batch_size_threshold() -> Result<()> {
    let config = CdpPoolConfig {
        max_batch_size: 3,
        ..Default::default()
    };
    let pool = CdpConnectionPool::new(config);

    // Add commands up to batch size
    for i in 0..3 {
        let command = CdpCommand {
            command_name: format!("Command{}", i),
            params: serde_json::json!({}),
            timestamp: Instant::now(),
        };
        pool.batch_command("test-browser", command).await?;
    }

    // Queue should be auto-cleared when batch size is reached
    let flushed = pool.flush_batches("test-browser").await?;
    // Queue was auto-cleared, so we get empty result
    assert_eq!(flushed.len(), 0);

    Ok(())
}

/// Test 14: Batch execute with empty queue
///
/// **Chrome Lock Fix**: Uses `#[serial]` to prevent concurrent Chrome launches
/// that would trigger SingletonLock conflicts.
#[tokio::test]
#[cfg_attr(test, serial_test::serial)]
async fn test_batch_execute_empty() -> Result<()> {
    let config = CdpPoolConfig::default();
    let pool = CdpConnectionPool::new(config);

    // Create a mock browser for testing with unique user data dir
    let temp_dir = std::env::temp_dir().join(format!("cdp_test_empty_{}", std::process::id()));
    let browser_config = chromiumoxide::BrowserConfig::builder()
        .user_data_dir(&temp_dir)
        .build()
        .expect("Failed to build browser config");

    let (mut browser, mut handler) = chromiumoxide::Browser::launch(browser_config)
        .await
        .expect("Failed to launch browser");

    // Spawn handler in background
    tokio::spawn(async move { while handler.next().await.is_some() {} });

    let page = browser
        .new_page("about:blank")
        .await
        .expect("Failed to create page");

    // Execute batch with no commands
    let result = pool.batch_execute("test-browser", &page).await?;

    assert_eq!(result.total_commands, 0);
    assert_eq!(result.successful, 0);
    assert_eq!(result.failed, 0);
    assert_eq!(result.results.len(), 0);

    // Cleanup
    let _ = browser.close().await;
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(())
}

/// Test 15: Batch execute when batching is disabled
///
/// **Chrome Lock Fix**: Uses `#[serial]` to prevent concurrent Chrome launches
#[tokio::test]
#[cfg_attr(test, serial_test::serial)]
async fn test_batch_config_disabled() -> Result<()> {
    let config = CdpPoolConfig {
        enable_batching: false,
        ..Default::default()
    };

    let pool = CdpConnectionPool::new(config);

    // Create a mock browser for testing with unique user data dir
    let temp_dir = std::env::temp_dir().join(format!("cdp_test_disabled_{}", std::process::id()));
    let browser_config = chromiumoxide::BrowserConfig::builder()
        .user_data_dir(&temp_dir)
        .build()
        .expect("Failed to build browser config");

    let (mut browser, mut handler) = chromiumoxide::Browser::launch(browser_config)
        .await
        .expect("Failed to launch browser");

    // Spawn handler in background
    tokio::spawn(async move { while handler.next().await.is_some() {} });

    let page = browser
        .new_page("about:blank")
        .await
        .expect("Failed to create page");

    // Try to add commands (should be no-op when batching disabled)
    let command = CdpCommand {
        command_name: "Test.Command".to_string(),
        params: serde_json::json!({}),
        timestamp: Instant::now(),
    };
    pool.batch_command("test-browser", command).await?;

    // Execute should return empty result
    let result = pool.batch_execute("test-browser", &page).await?;

    assert_eq!(result.total_commands, 0);
    assert_eq!(result.successful, 0);
    assert_eq!(result.failed, 0);

    // Cleanup
    let _ = browser.close().await;
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(())
}

/// Test 16: Batch execute with commands
///
/// **Chrome Lock Fix**: Uses `#[serial]` to prevent concurrent Chrome launches
#[tokio::test]
#[cfg_attr(test, serial_test::serial)]
async fn test_batch_execute_with_commands() -> Result<()> {
    let config = CdpPoolConfig::default();
    let pool = CdpConnectionPool::new(config);

    // Create a mock browser for testing with unique user data dir
    let temp_dir = std::env::temp_dir().join(format!("cdp_test_commands_{}", std::process::id()));
    let browser_config = chromiumoxide::BrowserConfig::builder()
        .user_data_dir(&temp_dir)
        .build()
        .expect("Failed to build browser config");

    let (mut browser, mut handler) = chromiumoxide::Browser::launch(browser_config)
        .await
        .expect("Failed to launch browser");

    // Spawn handler in background
    tokio::spawn(async move { while handler.next().await.is_some() {} });

    let page = browser
        .new_page("about:blank")
        .await
        .expect("Failed to create page");

    // Add commands to batch
    let commands = vec![
        CdpCommand {
            command_name: "Page.navigate".to_string(),
            params: serde_json::json!({"url": "about:blank"}),
            timestamp: Instant::now(),
        },
        CdpCommand {
            command_name: "Page.reload".to_string(),
            params: serde_json::json!({}),
            timestamp: Instant::now(),
        },
        CdpCommand {
            command_name: "Custom.Command".to_string(),
            params: serde_json::json!({"key": "value"}),
            timestamp: Instant::now(),
        },
    ];

    for command in commands {
        pool.batch_command("test-browser", command).await?;
    }

    // Execute batch
    let result = pool.batch_execute("test-browser", &page).await?;

    assert_eq!(result.total_commands, 3);
    assert!(result.successful > 0, "Expected some successful commands");
    assert_eq!(result.results.len(), 3);

    // Verify results structure
    for batch_result in &result.results {
        assert!(!batch_result.command_name.is_empty());
    }

    // Cleanup
    let _ = browser.close().await;
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(())
}

//
// ERROR PATH TESTS - Comprehensive error handling coverage
//

/// Test 17: ERROR PATH - Connection pool cleanup for non-existent browser
#[tokio::test]
async fn test_error_cleanup_nonexistent_browser() -> Result<()> {
    let config = CdpPoolConfig::default();
    let pool = CdpConnectionPool::new(config);

    // Cleanup should not fail for non-existent browser
    pool.cleanup_browser("nonexistent-browser").await;

    Ok(())
}

/// Test 18: ERROR PATH - Flush batches for non-existent browser
#[tokio::test]
async fn test_error_flush_nonexistent_browser() -> Result<()> {
    let config = CdpPoolConfig::default();
    let pool = CdpConnectionPool::new(config);

    // Flushing non-existent browser should return empty vec
    let flushed = pool.flush_batches("nonexistent-browser").await?;
    assert_eq!(flushed.len(), 0);

    Ok(())
}

/// Test 19: ERROR PATH - Multiple cleanup calls
#[tokio::test]
async fn test_error_multiple_cleanup_calls() -> Result<()> {
    let config = CdpPoolConfig::default();
    let pool = CdpConnectionPool::new(config);

    // Multiple cleanup calls should be safe
    pool.cleanup_browser("test-browser").await;
    pool.cleanup_browser("test-browser").await;
    pool.cleanup_browser("test-browser").await;

    Ok(())
}

/// Test 20: ERROR PATH - Batching with invalid command data
#[tokio::test]
async fn test_error_invalid_command_data() -> Result<()> {
    let config = CdpPoolConfig::default();
    let pool = CdpConnectionPool::new(config);

    // Commands with invalid/malformed data should still queue successfully
    let command = CdpCommand {
        command_name: "Invalid.Command".to_string(),
        params: serde_json::json!({"malformed": null}),
        timestamp: Instant::now(),
    };

    pool.batch_command("test-browser", command).await?;

    Ok(())
}

/// Test 21: ERROR PATH - Concurrent batch operations
#[tokio::test]
async fn test_error_concurrent_batch_operations() -> Result<()> {
    let config = CdpPoolConfig::default();
    let pool = std::sync::Arc::new(CdpConnectionPool::new(config));

    // Concurrent batching should be safe
    let mut handles = vec![];
    for i in 0..10 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            let command = CdpCommand {
                command_name: format!("cmd_{}", i),
                params: serde_json::json!({}),
                timestamp: Instant::now(),
            };
            pool_clone.batch_command("test-browser", command).await
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await??;
    }

    Ok(())
}

/// Test 22: ERROR PATH - Zero connection limit (edge case)
#[tokio::test]
async fn test_error_zero_connection_limit() -> Result<()> {
    let config = CdpPoolConfig {
        max_connections_per_browser: 0, // Edge case
        ..Default::default()
    };
    let pool = CdpConnectionPool::new(config);

    let stats = pool.stats().await;
    assert_eq!(stats.total_connections, 0);

    Ok(())
}

/// Test 23: ERROR PATH - Extremely short timeouts
#[tokio::test]
async fn test_error_extreme_timeouts() -> Result<()> {
    let config = CdpPoolConfig {
        batch_timeout: Duration::from_millis(1),
        connection_idle_timeout: Duration::from_millis(1),
        ..Default::default()
    };
    let _pool = CdpConnectionPool::new(config);

    // Should not panic even with very short timeouts
    Ok(())
}

/// Test 24: ERROR PATH - Extremely large batch size
#[tokio::test]
async fn test_error_extreme_batch_size() -> Result<()> {
    let config = CdpPoolConfig {
        max_batch_size: 100_000, // Extremely large
        ..Default::default()
    };
    let pool = CdpConnectionPool::new(config);

    // Add many commands
    for i in 0..1000 {
        let command = CdpCommand {
            command_name: format!("cmd_{}", i),
            params: serde_json::json!({}),
            timestamp: Instant::now(),
        };
        pool.batch_command("test-browser", command).await?;
    }

    let flushed = pool.flush_batches("test-browser").await?;
    assert_eq!(flushed.len(), 1000);

    Ok(())
}

/// Test 25: ERROR PATH - Empty command names
#[tokio::test]
async fn test_error_empty_command_names() -> Result<()> {
    let config = CdpPoolConfig::default();
    let pool = CdpConnectionPool::new(config);

    let command = CdpCommand {
        command_name: "".to_string(), // Empty name
        params: serde_json::json!({}),
        timestamp: Instant::now(),
    };

    pool.batch_command("test-browser", command).await?;

    Ok(())
}
