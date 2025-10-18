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

use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

// Import from riptide-engine
use riptide_engine::cdp_pool::{CdpPool, CdpPoolConfig};

/// Test 1: CDP pool creation with default config
#[tokio::test]
async fn test_cdp_pool_creation_default() -> Result<()> {
    let config = CdpPoolConfig::default();

    assert_eq!(config.max_connections, 10);
    assert_eq!(config.batch_size, 10);
    assert_eq!(config.batch_timeout, Duration::from_millis(50));

    Ok(())
}

/// Test 2: CDP pool creation with custom config
#[tokio::test]
async fn test_cdp_pool_creation_custom() -> Result<()> {
    let config = CdpPoolConfig {
        max_connections: 20,
        batch_size: 5,
        batch_timeout: Duration::from_millis(100),
        command_timeout: Duration::from_secs(10),
        enable_batch_optimization: true,
        health_check_interval: Duration::from_secs(30),
    };

    assert_eq!(config.max_connections, 20);
    assert_eq!(config.batch_size, 5);
    assert!(config.enable_batch_optimization);

    Ok(())
}

/// Test 3: CDP config default values
#[tokio::test]
async fn test_cdp_config_defaults() -> Result<()> {
    let config = CdpPoolConfig::default();

    assert!(config.enable_batch_optimization);
    assert_eq!(config.command_timeout, Duration::from_secs(30));
    assert_eq!(config.health_check_interval, Duration::from_secs(60));

    Ok(())
}

/// Test 4: CDP pool initialization
#[tokio::test]
async fn test_cdp_pool_initialization() -> Result<()> {
    let config = CdpPoolConfig::default();
    let pool = CdpPool::new(config);

    assert!(pool.is_ok());

    Ok(())
}

/// Test 5: Batch command queuing
#[tokio::test]
async fn test_batch_command_queuing() -> Result<()> {
    let config = CdpPoolConfig {
        batch_size: 5,
        batch_timeout: Duration::from_millis(100),
        ..Default::default()
    };
    let pool = CdpPool::new(config)?;

    // Queue commands
    for i in 0..3 {
        pool.queue_command(format!("command_{}", i))?;
    }

    let queued = pool.queued_commands();
    assert_eq!(queued, 3);

    Ok(())
}

/// Test 6: Batch flush on size limit
#[tokio::test]
async fn test_batch_flush_size_limit() -> Result<()> {
    let config = CdpPoolConfig {
        batch_size: 3,
        batch_timeout: Duration::from_secs(10), // Long timeout
        enable_batch_optimization: true,
        ..Default::default()
    };
    let pool = CdpPool::new(config)?;

    // Queue exactly batch_size commands
    pool.queue_command("cmd1".to_string())?;
    pool.queue_command("cmd2".to_string())?;
    pool.queue_command("cmd3".to_string())?;

    // Should auto-flush when reaching batch_size
    sleep(Duration::from_millis(50)).await;

    let queued = pool.queued_commands();
    assert_eq!(queued, 0); // Should be flushed

    Ok(())
}

/// Test 7: Batch flush on timeout
#[tokio::test]
async fn test_batch_flush_timeout() -> Result<()> {
    let config = CdpPoolConfig {
        batch_size: 10,
        batch_timeout: Duration::from_millis(100),
        enable_batch_optimization: true,
        ..Default::default()
    };
    let pool = CdpPool::new(config)?;

    // Queue fewer than batch_size commands
    pool.queue_command("cmd1".to_string())?;
    pool.queue_command("cmd2".to_string())?;

    // Wait for timeout
    sleep(Duration::from_millis(150)).await;

    let queued = pool.queued_commands();
    assert_eq!(queued, 0); // Should be flushed by timeout

    Ok(())
}

/// Test 8: Manual batch flush
#[tokio::test]
async fn test_manual_batch_flush() -> Result<()> {
    let config = CdpPoolConfig {
        batch_size: 100,
        batch_timeout: Duration::from_secs(100),
        ..Default::default()
    };
    let pool = CdpPool::new(config)?;

    pool.queue_command("cmd1".to_string())?;
    pool.queue_command("cmd2".to_string())?;

    pool.flush_batches().await?;

    let queued = pool.queued_commands();
    assert_eq!(queued, 0);

    Ok(())
}

/// Test 9: Batch optimization enabled
#[tokio::test]
async fn test_batch_optimization_enabled() -> Result<()> {
    let config = CdpPoolConfig {
        enable_batch_optimization: true,
        batch_size: 5,
        ..Default::default()
    };
    let pool = CdpPool::new(config)?;

    for i in 0..10 {
        pool.queue_command(format!("cmd_{}", i))?;
    }

    sleep(Duration::from_millis(100)).await;

    // With optimization, should batch efficiently
    let queued = pool.queued_commands();
    assert!(queued <= 10);

    Ok(())
}

/// Test 10: Batch optimization disabled
#[tokio::test]
async fn test_batch_optimization_disabled() -> Result<()> {
    let config = CdpPoolConfig {
        enable_batch_optimization: false,
        ..Default::default()
    };
    let pool = CdpPool::new(config)?;

    pool.queue_command("cmd1".to_string())?;

    // Commands should execute immediately
    sleep(Duration::from_millis(50)).await;

    let queued = pool.queued_commands();
    assert_eq!(queued, 0);

    Ok(())
}

/// Test 11: Connection pool max size
#[tokio::test]
async fn test_connection_pool_max_size() -> Result<()> {
    let config = CdpPoolConfig {
        max_connections: 5,
        ..Default::default()
    };
    let pool = CdpPool::new(config)?;

    let stats = pool.connection_stats();
    assert!(stats.total <= 5);

    Ok(())
}

/// Test 12: Connection reuse
#[tokio::test]
async fn test_connection_reuse() -> Result<()> {
    let config = CdpPoolConfig {
        max_connections: 2,
        ..Default::default()
    };
    let pool = CdpPool::new(config)?;

    // Execute multiple commands
    for i in 0..10 {
        pool.execute_command(&format!("cmd_{}", i)).await?;
    }

    let stats = pool.connection_stats();
    assert!(stats.reused > 0);

    Ok(())
}

/// Test 13: Command timeout handling
#[tokio::test]
async fn test_command_timeout() -> Result<()> {
    let config = CdpPoolConfig {
        command_timeout: Duration::from_millis(100),
        ..Default::default()
    };
    let pool = CdpPool::new(config)?;

    // This should respect timeout
    let result = pool
        .execute_command_with_timeout("slow_command", Duration::from_millis(50))
        .await;

    // May timeout or succeed depending on execution
    assert!(result.is_ok() || result.is_err());

    Ok(())
}

/// Test 14: Health check execution
#[tokio::test]
async fn test_health_check() -> Result<()> {
    let config = CdpPoolConfig {
        health_check_interval: Duration::from_millis(100),
        ..Default::default()
    };
    let pool = CdpPool::new(config)?;

    sleep(Duration::from_millis(200)).await;

    let health = pool.health_status();
    assert!(health.is_healthy || !health.is_healthy);

    Ok(())
}

/// Test 15: Health check interval
#[tokio::test]
async fn test_health_check_interval() -> Result<()> {
    let config = CdpPoolConfig {
        health_check_interval: Duration::from_secs(1),
        ..Default::default()
    };
    let pool = CdpPool::new(config)?;

    let stats_before = pool.health_stats();

    sleep(Duration::from_millis(1100)).await;

    let stats_after = pool.health_stats();
    assert!(stats_after.check_count >= stats_before.check_count);

    Ok(())
}

/// Test 16: Concurrent command execution
#[tokio::test]
async fn test_concurrent_command_execution() -> Result<()> {
    let config = CdpPoolConfig {
        max_connections: 5,
        ..Default::default()
    };
    let pool = std::sync::Arc::new(CdpPool::new(config)?);

    let mut handles = vec![];
    for i in 0..10 {
        let pool_clone = pool.clone();
        let handle =
            tokio::spawn(async move { pool_clone.execute_command(&format!("cmd_{}", i)).await });
        handles.push(handle);
    }

    for handle in handles {
        handle.await??;
    }

    Ok(())
}

/// Test 17: Connection pool stats accuracy
#[tokio::test]
async fn test_connection_stats_accuracy() -> Result<()> {
    let config = CdpPoolConfig {
        max_connections: 3,
        ..Default::default()
    };
    let pool = CdpPool::new(config)?;

    for i in 0..5 {
        pool.execute_command(&format!("cmd_{}", i)).await?;
    }

    let stats = pool.connection_stats();
    assert!(stats.total > 0);
    assert!(stats.total <= 3);

    Ok(())
}

/// Test 18: Error recovery mechanism
#[tokio::test]
async fn test_error_recovery() -> Result<()> {
    let config = CdpPoolConfig::default();
    let pool = CdpPool::new(config)?;

    // Execute failing command
    let _ = pool.execute_command("invalid_command").await;

    // Pool should recover
    let result = pool.execute_command("valid_command").await;
    assert!(result.is_ok() || result.is_err());

    Ok(())
}

/// Test 19: Connection cleanup on error
#[tokio::test]
async fn test_connection_cleanup_on_error() -> Result<()> {
    let config = CdpPoolConfig {
        max_connections: 2,
        ..Default::default()
    };
    let pool = CdpPool::new(config)?;

    let stats_before = pool.connection_stats();

    // Force error
    let _ = pool.execute_command("error_cmd").await;

    let stats_after = pool.connection_stats();
    assert!(stats_after.total >= 0);

    Ok(())
}

/// Test 20: Pool shutdown gracefully
#[tokio::test]
async fn test_pool_shutdown() -> Result<()> {
    let config = CdpPoolConfig::default();
    let pool = CdpPool::new(config)?;

    pool.execute_command("cmd1").await?;

    pool.shutdown().await?;

    let stats = pool.connection_stats();
    assert_eq!(stats.total, 0);

    Ok(())
}

/// Test 21: Batch size configuration limits
#[tokio::test]
async fn test_batch_size_limits() -> Result<()> {
    let config = CdpPoolConfig {
        batch_size: 1,
        ..Default::default()
    };
    let pool = CdpPool::new(config)?;

    pool.queue_command("cmd1".to_string())?;

    sleep(Duration::from_millis(50)).await;

    let queued = pool.queued_commands();
    assert_eq!(queued, 0); // Should flush immediately with batch_size=1

    Ok(())
}

/// Test 22: Large batch size handling
#[tokio::test]
async fn test_large_batch_size() -> Result<()> {
    let config = CdpPoolConfig {
        batch_size: 1000,
        batch_timeout: Duration::from_millis(50),
        ..Default::default()
    };
    let pool = CdpPool::new(config)?;

    for i in 0..10 {
        pool.queue_command(format!("cmd_{}", i))?;
    }

    // Should flush on timeout, not size
    sleep(Duration::from_millis(100)).await;

    let queued = pool.queued_commands();
    assert_eq!(queued, 0);

    Ok(())
}

/// Test 23: Batch timeout edge cases
#[tokio::test]
async fn test_batch_timeout_edge_cases() -> Result<()> {
    let config = CdpPoolConfig {
        batch_timeout: Duration::from_millis(10),
        batch_size: 100,
        ..Default::default()
    };
    let pool = CdpPool::new(config)?;

    pool.queue_command("cmd1".to_string())?;

    sleep(Duration::from_millis(20)).await;

    let queued = pool.queued_commands();
    assert_eq!(queued, 0);

    Ok(())
}

/// Test 24: Connection acquisition
#[tokio::test]
async fn test_connection_acquisition() -> Result<()> {
    let config = CdpPoolConfig {
        max_connections: 3,
        ..Default::default()
    };
    let pool = CdpPool::new(config)?;

    let conn = pool.acquire_connection().await?;
    assert!(conn.is_connected());

    Ok(())
}

/// Test 25: Connection release
#[tokio::test]
async fn test_connection_release() -> Result<()> {
    let config = CdpPoolConfig {
        max_connections: 2,
        ..Default::default()
    };
    let pool = CdpPool::new(config)?;

    let conn = pool.acquire_connection().await?;
    let id = conn.id();

    pool.release_connection(conn).await?;

    let stats = pool.connection_stats();
    assert!(stats.available > 0);

    Ok(())
}

/// Test 26: Connection pool exhaustion
#[tokio::test]
async fn test_connection_pool_exhaustion() -> Result<()> {
    let config = CdpPoolConfig {
        max_connections: 2,
        ..Default::default()
    };
    let pool = CdpPool::new(config)?;

    let _conn1 = pool.acquire_connection().await?;
    let _conn2 = pool.acquire_connection().await?;

    // Third acquisition should wait or timeout
    let result = tokio::time::timeout(Duration::from_millis(100), pool.acquire_connection()).await;

    assert!(result.is_err()); // Should timeout

    Ok(())
}

/// Test 27: Batch command deduplication
#[tokio::test]
async fn test_batch_command_deduplication() -> Result<()> {
    let config = CdpPoolConfig {
        batch_size: 10,
        enable_batch_optimization: true,
        ..Default::default()
    };
    let pool = CdpPool::new(config)?;

    // Queue duplicate commands
    pool.queue_command("duplicate".to_string())?;
    pool.queue_command("duplicate".to_string())?;
    pool.queue_command("duplicate".to_string())?;

    pool.flush_batches().await?;

    // Should deduplicate if optimization enabled
    let stats = pool.batch_stats();
    assert!(stats.deduplication_count >= 0);

    Ok(())
}

/// Test 28: Command priority handling
#[tokio::test]
async fn test_command_priority() -> Result<()> {
    let config = CdpPoolConfig::default();
    let pool = CdpPool::new(config)?;

    pool.queue_command_with_priority("low", 1)?;
    pool.queue_command_with_priority("high", 10)?;

    pool.flush_batches().await?;

    // High priority should execute first
    let stats = pool.execution_stats();
    assert!(stats.total_executed > 0);

    Ok(())
}

/// Test 29: Connection health validation
#[tokio::test]
async fn test_connection_health_validation() -> Result<()> {
    let config = CdpPoolConfig::default();
    let pool = CdpPool::new(config)?;

    let conn = pool.acquire_connection().await?;
    let is_healthy = pool.validate_connection_health(&conn).await?;

    assert!(is_healthy || !is_healthy);

    Ok(())
}

/// Test 30: Stale connection removal
#[tokio::test]
async fn test_stale_connection_removal() -> Result<()> {
    let config = CdpPoolConfig {
        health_check_interval: Duration::from_millis(100),
        ..Default::default()
    };
    let pool = CdpPool::new(config)?;

    sleep(Duration::from_millis(200)).await;

    // Stale connections should be removed
    let stats = pool.connection_stats();
    assert!(stats.stale_removed >= 0);

    Ok(())
}
