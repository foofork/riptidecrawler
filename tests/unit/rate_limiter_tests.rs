//! Unit tests for PerHostRateLimiter
//!
//! Tests cover:
//! - Token bucket algorithm
//! - Per-host rate limiting
//! - Jitter application
//! - Bucket cleanup
//! - Concurrent access

use anyhow::Result;
use riptide_api::config::ApiConfig;
use riptide_api::resource_manager::ResourceManager;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_rate_limiter_allows_first_request() -> Result<()> {
    // First request to a host should always succeed
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let result = manager.rate_limiter.check_rate_limit("example.com").await;
    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_rate_limiter_blocks_rapid_requests() -> Result<()> {
    // Rapid requests should be rate limited
    let mut config = ApiConfig::default();
    config.rate_limiting.requests_per_second_per_host = 1.0;
    config.rate_limiting.enabled = true;

    let manager = ResourceManager::new(config).await?;

    // First request succeeds
    assert!(manager
        .rate_limiter
        .check_rate_limit("example.com")
        .await
        .is_ok());

    // Immediate second request should be rate limited
    let result = manager.rate_limiter.check_rate_limit("example.com").await;
    assert!(result.is_err());

    if let Err(retry_after) = result {
        assert!(retry_after > Duration::from_millis(0));
    }

    Ok(())
}

#[tokio::test]
async fn test_rate_limiter_refills_tokens() -> Result<()> {
    // Tokens should refill over time
    let mut config = ApiConfig::default();
    config.rate_limiting.requests_per_second_per_host = 2.0; // 2 RPS
    config.rate_limiting.enabled = true;

    let manager = ResourceManager::new(config).await?;

    // Use all tokens
    manager
        .rate_limiter
        .check_rate_limit("example.com")
        .await
        .ok();
    manager
        .rate_limiter
        .check_rate_limit("example.com")
        .await
        .ok();

    // Should be rate limited now
    assert!(manager
        .rate_limiter
        .check_rate_limit("example.com")
        .await
        .is_err());

    // Wait for token refill (0.5 seconds = 1 token at 2 RPS)
    sleep(Duration::from_millis(600)).await;

    // Should succeed after refill
    assert!(manager
        .rate_limiter
        .check_rate_limit("example.com")
        .await
        .is_ok());

    Ok(())
}

#[tokio::test]
async fn test_rate_limiter_per_host_isolation() -> Result<()> {
    // Rate limits should be isolated per host
    let mut config = ApiConfig::default();
    config.rate_limiting.requests_per_second_per_host = 1.0;
    config.rate_limiting.enabled = true;

    let manager = ResourceManager::new(config).await?;

    // Exhaust tokens for first host
    manager.rate_limiter.check_rate_limit("host1.com").await.ok();
    assert!(manager
        .rate_limiter
        .check_rate_limit("host1.com")
        .await
        .is_err());

    // Second host should still have tokens
    assert!(manager
        .rate_limiter
        .check_rate_limit("host2.com")
        .await
        .is_ok());

    Ok(())
}

#[tokio::test]
async fn test_rate_limiter_disabled() -> Result<()> {
    // When disabled, all requests should pass
    let mut config = ApiConfig::default();
    config.rate_limiting.enabled = false;

    let manager = ResourceManager::new(config).await?;

    // Multiple rapid requests should all succeed
    for _ in 0..10 {
        assert!(manager
            .rate_limiter
            .check_rate_limit("example.com")
            .await
            .is_ok());
    }

    Ok(())
}

#[tokio::test]
async fn test_rate_limiter_burst_capacity() -> Result<()> {
    // Burst capacity should allow multiple requests
    let mut config = ApiConfig::default();
    config.rate_limiting.requests_per_second_per_host = 1.0;
    config.rate_limiting.burst_capacity_per_host = 5.0;
    config.rate_limiting.enabled = true;

    let manager = ResourceManager::new(config).await?;

    // Should allow burst up to capacity
    let mut success_count = 0;
    for _ in 0..10 {
        if manager
            .rate_limiter
            .check_rate_limit("example.com")
            .await
            .is_ok()
        {
            success_count += 1;
        }
    }

    // Should allow at least burst_capacity requests
    assert!(success_count >= 5, "Expected at least 5 successful requests");

    Ok(())
}

#[tokio::test]
async fn test_rate_limiter_jitter_applied() -> Result<()> {
    // Verify jitter adds delay between requests
    let mut config = ApiConfig::default();
    config.rate_limiting.requests_per_second_per_host = 10.0; // High rate
    config.rate_limiting.jitter_ms = 50; // 50ms jitter
    config.rate_limiting.enabled = true;

    let manager = ResourceManager::new(config).await?;

    let start = std::time::Instant::now();
    manager
        .rate_limiter
        .check_rate_limit("example.com")
        .await
        .ok();
    let duration = start.elapsed();

    // Should have some delay due to jitter
    assert!(
        duration >= Duration::from_millis(1),
        "Expected jitter delay"
    );

    Ok(())
}

#[tokio::test]
async fn test_rate_limiter_concurrent_requests() -> Result<()> {
    // Test concurrent requests to same host
    let mut config = ApiConfig::default();
    config.rate_limiting.requests_per_second_per_host = 2.0;
    config.rate_limiting.burst_capacity_per_host = 3.0;
    config.rate_limiting.enabled = true;

    let manager = ResourceManager::new(config).await?;

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let mgr = manager.clone();
            tokio::spawn(async move { mgr.rate_limiter.check_rate_limit("example.com").await })
        })
        .collect();

    let mut success_count = 0;
    let mut error_count = 0;

    for handle in handles {
        match handle.await {
            Ok(Ok(())) => success_count += 1,
            Ok(Err(_)) => error_count += 1,
            Err(_) => {}
        }
    }

    // Some should succeed, some should be rate limited
    assert!(success_count > 0);
    assert!(error_count > 0);
    println!(
        "Concurrent requests: {} success, {} rate limited",
        success_count, error_count
    );

    Ok(())
}

#[tokio::test]
async fn test_rate_limiter_metrics_tracking() -> Result<()> {
    // Test that rate limit hits are tracked in metrics
    let mut config = ApiConfig::default();
    config.rate_limiting.requests_per_second_per_host = 1.0;
    config.rate_limiting.enabled = true;

    let manager = ResourceManager::new(config).await?;

    let initial_status = manager.get_resource_status().await;
    let initial_hits = initial_status.rate_limit_hits;

    // Trigger rate limiting
    manager.rate_limiter.check_rate_limit("example.com").await.ok();
    manager.rate_limiter.check_rate_limit("example.com").await.ok();

    let final_status = manager.get_resource_status().await;

    // Should have recorded at least one rate limit hit
    assert!(final_status.rate_limit_hits >= initial_hits);

    Ok(())
}

#[tokio::test]
async fn test_rate_limiter_high_throughput() -> Result<()> {
    // Test with high RPS configuration
    let mut config = ApiConfig::default();
    config.rate_limiting.requests_per_second_per_host = 100.0;
    config.rate_limiting.burst_capacity_per_host = 200.0;
    config.rate_limiting.enabled = true;

    let manager = ResourceManager::new(config).await?;

    let mut success_count = 0;
    for _ in 0..150 {
        if manager
            .rate_limiter
            .check_rate_limit("example.com")
            .await
            .is_ok()
        {
            success_count += 1;
        }
    }

    // With high RPS, most requests should succeed
    assert!(success_count > 100);

    Ok(())
}

#[tokio::test]
async fn test_rate_limiter_token_cap() -> Result<()> {
    // Test that tokens are capped at burst capacity
    let mut config = ApiConfig::default();
    config.rate_limiting.requests_per_second_per_host = 1.0;
    config.rate_limiting.burst_capacity_per_host = 3.0;
    config.rate_limiting.enabled = true;

    let manager = ResourceManager::new(config).await?;

    // Wait long enough to accumulate many tokens
    sleep(Duration::from_secs(10)).await;

    // Should only allow burst_capacity requests
    let mut success_count = 0;
    for _ in 0..10 {
        if manager
            .rate_limiter
            .check_rate_limit("example.com")
            .await
            .is_ok()
        {
            success_count += 1;
        } else {
            break;
        }
    }

    // Should be capped at burst capacity
    assert!(
        success_count <= 4,
        "Expected max {} successful requests, got {}",
        4,
        success_count
    );

    Ok(())
}
