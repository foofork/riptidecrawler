//! Unit tests for PerformanceMonitor
//!
//! Tests cover:
//! - Timeout recording
//! - Degradation score calculation
//! - Render operation metrics
//! - Performance tracking

use anyhow::Result;
use riptide_api::config::RiptideApiConfig;
use riptide_api::resource_manager::ResourceManager;
use std::time::Duration;

#[tokio::test]
async fn test_performance_timeout_recording() -> Result<()> {
    // Test timeout recording
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let initial_status = manager.get_resource_status().await;
    let initial_timeouts = initial_status.timeout_count;

    manager.performance_monitor.record_timeout().await;

    let final_status = manager.get_resource_status().await;

    assert_eq!(
        final_status.timeout_count,
        initial_timeouts + 1,
        "Timeout count should increment"
    );

    Ok(())
}

#[tokio::test]
async fn test_performance_multiple_timeouts() -> Result<()> {
    // Test multiple timeout recordings
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let initial_status = manager.get_resource_status().await;
    let initial_timeouts = initial_status.timeout_count;

    for _ in 0..5 {
        manager.performance_monitor.record_timeout().await;
    }

    let final_status = manager.get_resource_status().await;

    assert_eq!(
        final_status.timeout_count,
        initial_timeouts + 5,
        "Should record all timeouts"
    );

    Ok(())
}

#[tokio::test]
async fn test_performance_degradation_score_initial() -> Result<()> {
    // Test initial degradation score
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let score = manager.performance_monitor.get_degradation_score().await;

    assert_eq!(score, 0.0, "Initial degradation score should be 0.0");

    Ok(())
}

#[tokio::test]
async fn test_performance_degradation_score_range() -> Result<()> {
    // Test degradation score stays in valid range
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    // Record some events
    for _ in 0..10 {
        manager.performance_monitor.record_timeout().await;
    }

    let score = manager.performance_monitor.get_degradation_score().await;

    assert!(
        score >= 0.0 && score <= 1.0,
        "Degradation score should be between 0.0 and 1.0, got {}",
        score
    );

    Ok(())
}

#[tokio::test]
async fn test_performance_render_operation_success() -> Result<()> {
    // Test recording successful render operation
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let url = "https://example.com";
    let duration = Duration::from_millis(500);

    manager
        .performance_monitor
        .record_render_operation(url, duration, true, 5, 10)
        .await?;

    // Operation should be recorded without error
    Ok(())
}

#[tokio::test]
async fn test_performance_render_operation_failure() -> Result<()> {
    // Test recording failed render operation
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let url = "https://example.com";
    let duration = Duration::from_millis(1000);

    manager
        .performance_monitor
        .record_render_operation(url, duration, false, 0, 0)
        .await?;

    // Operation should be recorded without error
    Ok(())
}

#[tokio::test]
async fn test_performance_multiple_render_operations() -> Result<()> {
    // Test recording multiple render operations
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let operations = vec![
        ("https://site1.com", Duration::from_millis(300), true, 3, 5),
        ("https://site2.com", Duration::from_millis(500), true, 7, 12),
        ("https://site3.com", Duration::from_millis(200), false, 1, 2),
        ("https://site4.com", Duration::from_millis(800), true, 10, 20),
    ];

    for (url, duration, success, actions, requests) in operations {
        manager
            .performance_monitor
            .record_render_operation(url, duration, success, actions, requests)
            .await?;
    }

    // All operations should be recorded
    Ok(())
}

#[tokio::test]
async fn test_performance_concurrent_timeout_recording() -> Result<()> {
    // Test concurrent timeout recordings
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let initial_status = manager.get_resource_status().await;
    let initial_timeouts = initial_status.timeout_count;

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let mgr = manager.clone();
            tokio::spawn(async move {
                mgr.performance_monitor.record_timeout().await;
            })
        })
        .collect();

    for handle in handles {
        handle.await?;
    }

    let final_status = manager.get_resource_status().await;

    assert_eq!(
        final_status.timeout_count,
        initial_timeouts + 10,
        "Should record all concurrent timeouts"
    );

    Ok(())
}

#[tokio::test]
async fn test_performance_render_timing_tracking() -> Result<()> {
    // Test render timing collection
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let timings = vec![
        Duration::from_millis(100),
        Duration::from_millis(200),
        Duration::from_millis(150),
        Duration::from_millis(300),
        Duration::from_millis(250),
    ];

    for (i, duration) in timings.iter().enumerate() {
        manager
            .performance_monitor
            .record_render_operation(
                &format!("https://site{}.com", i),
                *duration,
                true,
                5,
                10,
            )
            .await?;
    }

    // Timing data should be collected internally
    // (specific timing analysis would require exposed methods)
    Ok(())
}

#[tokio::test]
async fn test_performance_zero_duration_render() -> Result<()> {
    // Test render with zero duration
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    manager
        .performance_monitor
        .record_render_operation("https://instant.com", Duration::from_millis(0), true, 0, 0)
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_performance_very_long_render() -> Result<()> {
    // Test render with very long duration
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let long_duration = Duration::from_secs(60);

    manager
        .performance_monitor
        .record_render_operation("https://slow.com", long_duration, false, 100, 500)
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_performance_high_action_count() -> Result<()> {
    // Test render with high action count
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    manager
        .performance_monitor
        .record_render_operation(
            "https://complex.com",
            Duration::from_secs(5),
            true,
            1000, // Many actions
            5000, // Many network requests
        )
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_performance_metrics_integration() -> Result<()> {
    // Test performance metrics integration with resource status
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    // Record various events
    manager.performance_monitor.record_timeout().await;
    manager
        .performance_monitor
        .record_render_operation("https://test.com", Duration::from_millis(500), true, 5, 10)
        .await?;

    let status = manager.get_resource_status().await;

    // Verify metrics are accessible
    assert!(status.timeout_count > 0);
    assert!(status.degradation_score >= 0.0);

    Ok(())
}

#[tokio::test]
async fn test_performance_cleanup_integration() -> Result<()> {
    // Test performance monitor integration with cleanup
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    // Trigger cleanup which should record timeout
    manager.cleanup_on_timeout("render").await;

    let status = manager.get_resource_status().await;

    assert!(
        status.timeout_count > 0,
        "Cleanup should have recorded timeout"
    );

    Ok(())
}

#[tokio::test]
async fn test_performance_concurrent_render_recording() -> Result<()> {
    // Test concurrent render operation recording
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let handles: Vec<_> = (0..20)
        .map(|i| {
            let mgr = manager.clone();
            tokio::spawn(async move {
                mgr.performance_monitor
                    .record_render_operation(
                        &format!("https://concurrent{}.com", i),
                        Duration::from_millis(100),
                        true,
                        5,
                        10,
                    )
                    .await
            })
        })
        .collect();

    for handle in handles {
        handle.await??;
    }

    // All operations should be recorded
    Ok(())
}

#[tokio::test]
async fn test_performance_mixed_success_failure() -> Result<()> {
    // Test recording mix of successful and failed operations
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    // Record successes
    for i in 0..10 {
        manager
            .performance_monitor
            .record_render_operation(
                &format!("https://success{}.com", i),
                Duration::from_millis(300),
                true,
                5,
                10,
            )
            .await?;
    }

    // Record failures
    for i in 0..5 {
        manager
            .performance_monitor
            .record_render_operation(
                &format!("https://failure{}.com", i),
                Duration::from_millis(1000),
                false,
                0,
                0,
            )
            .await?;
    }

    // All operations should be recorded
    Ok(())
}
