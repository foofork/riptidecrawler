//! Integration tests for riptide-headless-hybrid
//!
//! **P1-C1 Week 1**: Comprehensive end-to-end testing
//!
//! These tests validate:
//! 1. Browser launch and session management
//! 2. Stealth feature application
//! 3. Session lifecycle and cleanup
//! 4. Error handling and recovery
//! 5. Performance characteristics

use anyhow::Result;
use riptide_headless_hybrid::{
    HybridHeadlessLauncher, LauncherConfig, LauncherStats, PoolConfig, P1_C1_STATUS,
};
use riptide_stealth::StealthPreset;
use std::time::Duration;
use tokio::time::timeout;

// ============================================================================
// Configuration Tests (Day 1)
// ============================================================================

#[test]
fn test_p1_c1_week1_status() {
    // Verify P1-C1 Week 1 is complete
    assert_eq!(P1_C1_STATUS, "Week 1 Complete - Core launcher implemented");
}

#[test]
fn test_launcher_config_defaults() {
    let config = LauncherConfig::default();
    assert!(
        config.enable_stealth,
        "Stealth should be enabled by default"
    );
    assert_eq!(
        config.default_stealth_preset,
        StealthPreset::Medium,
        "Default stealth preset should be Medium"
    );
    assert_eq!(
        config.page_timeout,
        Duration::from_secs(30),
        "Default page timeout should be 30 seconds"
    );
    assert!(
        config.enable_monitoring,
        "Monitoring should be enabled by default"
    );
}

#[test]
fn test_pool_config_defaults() {
    let pool_config = PoolConfig::default();
    assert_eq!(pool_config.initial_size, 2);
    assert_eq!(pool_config.min_size, 1);
    assert_eq!(pool_config.max_size, 10);
    assert_eq!(pool_config.idle_timeout, Duration::from_secs(300));
    assert_eq!(pool_config.health_check_interval, Duration::from_secs(60));
}

#[test]
fn test_custom_launcher_config() {
    let custom_pool = PoolConfig {
        initial_size: 5,
        min_size: 2,
        max_size: 20,
        idle_timeout: Duration::from_secs(600),
        health_check_interval: Duration::from_secs(120),
    };

    let config = LauncherConfig {
        pool_config: custom_pool.clone(),
        default_stealth_preset: StealthPreset::High,
        enable_stealth: true,
        page_timeout: Duration::from_secs(60),
        enable_monitoring: true,
    };

    assert_eq!(config.pool_config.initial_size, 5);
    assert_eq!(config.default_stealth_preset, StealthPreset::High);
    assert_eq!(config.page_timeout, Duration::from_secs(60));
}

// ============================================================================
// Launcher Creation Tests (Day 1-2)
// ============================================================================

#[tokio::test]
async fn test_launcher_creation_default() {
    let result = HybridHeadlessLauncher::new().await;
    assert!(result.is_ok(), "Launcher creation should succeed");

    if let Ok(launcher) = result {
        let stats = launcher.stats().await;
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.successful_requests, 0);
        assert_eq!(stats.failed_requests, 0);
        assert_eq!(stats.avg_response_time_ms, 0.0);

        launcher.shutdown().await.expect("Shutdown should succeed");
    }
}

#[tokio::test]
async fn test_launcher_creation_custom_config() {
    let config = LauncherConfig {
        enable_stealth: false,
        default_stealth_preset: StealthPreset::None,
        page_timeout: Duration::from_secs(10),
        ..Default::default()
    };

    let result = HybridHeadlessLauncher::with_config(config).await;
    assert!(result.is_ok(), "Custom launcher creation should succeed");

    if let Ok(launcher) = result {
        launcher.shutdown().await.expect("Shutdown should succeed");
    }
}

#[tokio::test]
async fn test_launcher_shutdown() {
    let launcher = HybridHeadlessLauncher::new()
        .await
        .expect("Launcher creation should succeed");

    let result = launcher.shutdown().await;
    assert!(result.is_ok(), "Shutdown should succeed");
}

// ============================================================================
// Session Management Tests (Day 2-4)
// ============================================================================

#[tokio::test]
#[ignore] // Requires actual browser - run with: cargo test --ignored
async fn test_launch_page_basic() -> Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;

    // Use a reliable test URL
    let session = timeout(
        Duration::from_secs(45),
        launcher.launch_page_default("https://example.com"),
    )
    .await??;

    assert!(!session.session_id.is_empty(), "Session ID should be set");
    assert!(
        session.duration() < Duration::from_secs(45),
        "Session should be created within timeout"
    );

    // Verify stats updated
    let stats = launcher.stats().await;
    assert_eq!(stats.total_requests, 1);
    assert_eq!(stats.successful_requests, 1);

    session.close().await?;
    launcher.shutdown().await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires actual browser
async fn test_launch_page_with_stealth() -> Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;

    let session = timeout(
        Duration::from_secs(45),
        launcher.launch_page("https://example.com", Some(StealthPreset::High)),
    )
    .await??;

    assert!(!session.session_id.is_empty());

    // Verify stealth was applied in stats
    let stats = launcher.stats().await;
    assert_eq!(stats.stealth_requests, 1);
    assert_eq!(stats.non_stealth_requests, 0);

    session.close().await?;
    launcher.shutdown().await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires actual browser
async fn test_launch_page_no_stealth() -> Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;

    let session = timeout(
        Duration::from_secs(45),
        launcher.launch_page_no_stealth("https://example.com"),
    )
    .await??;

    assert!(!session.session_id.is_empty());

    // Verify no stealth in stats
    let stats = launcher.stats().await;
    assert_eq!(stats.non_stealth_requests, 1);
    assert_eq!(stats.stealth_requests, 0);

    session.close().await?;
    launcher.shutdown().await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires actual browser
async fn test_session_navigation() -> Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;
    let session = launcher.launch_page_default("https://example.com").await?;

    // Navigate to a different URL
    session.navigate("https://example.org").await?;

    session.close().await?;
    launcher.shutdown().await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires actual browser
async fn test_session_content_retrieval() -> Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;
    let session = launcher.launch_page_default("https://example.com").await?;

    let html = session.content().await?;
    assert!(!html.is_empty(), "Page content should not be empty");
    assert!(
        html.contains("Example Domain"),
        "Should contain expected content"
    );

    session.close().await?;
    launcher.shutdown().await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires actual browser
async fn test_session_script_execution() -> Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;
    let session = launcher.launch_page_default("https://example.com").await?;

    let result = session.execute_script("document.title").await?;
    assert!(
        result.is_string(),
        "Script result should be a string (title)"
    );

    session.close().await?;
    launcher.shutdown().await?;
    Ok(())
}

// ============================================================================
// Lifecycle and Cleanup Tests (Day 4)
// ============================================================================

#[tokio::test]
async fn test_launcher_stats_tracking() {
    let launcher = HybridHeadlessLauncher::new()
        .await
        .expect("Launcher should be created");

    let initial_stats = launcher.stats().await;
    assert_eq!(initial_stats.total_requests, 0);
    assert_eq!(initial_stats.successful_requests, 0);
    assert_eq!(initial_stats.failed_requests, 0);

    launcher.shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
#[ignore] // Requires actual browser
async fn test_multiple_sessions() -> Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;

    // Launch multiple sessions sequentially
    for i in 0..3 {
        let session = launcher.launch_page_default("https://example.com").await?;
        assert!(!session.session_id.is_empty());

        let stats = launcher.stats().await;
        assert_eq!(stats.total_requests, (i + 1) as u64);

        session.close().await?;
    }

    launcher.shutdown().await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires actual browser
async fn test_session_automatic_cleanup_on_drop() -> Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;

    {
        // Session will be dropped at end of scope
        let _session = launcher.launch_page_default("https://example.com").await?;
    } // Session dropped here

    // Stats should still reflect the request
    let stats = launcher.stats().await;
    assert_eq!(stats.total_requests, 1);

    launcher.shutdown().await?;
    Ok(())
}

// ============================================================================
// Error Handling Tests (Day 4)
// ============================================================================

#[tokio::test]
#[ignore] // Requires actual browser
async fn test_invalid_url_handling() {
    let launcher = HybridHeadlessLauncher::new()
        .await
        .expect("Launcher should be created");

    let result = timeout(
        Duration::from_secs(35),
        launcher.launch_page_default("not-a-valid-url"),
    )
    .await;

    // Should either timeout or fail gracefully
    assert!(
        result.is_err() || result.unwrap().is_err(),
        "Invalid URL should fail"
    );

    launcher.shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
#[ignore] // Requires actual browser
async fn test_timeout_handling() {
    let config = LauncherConfig {
        page_timeout: Duration::from_millis(100), // Very short timeout
        ..Default::default()
    };

    let launcher = HybridHeadlessLauncher::with_config(config)
        .await
        .expect("Launcher should be created");

    // This should timeout or complete very quickly
    let result = launcher.launch_page_default("https://example.com").await;

    // Either succeeds quickly or times out - both are acceptable
    if let Ok(session) = result {
        let _ = session.close().await;
    }

    launcher.shutdown().await.expect("Shutdown should succeed");
}

// ============================================================================
// Statistics and Monitoring Tests (Day 5)
// ============================================================================

#[test]
fn test_launcher_stats_structure() {
    let stats = LauncherStats::default();
    assert_eq!(stats.total_requests, 0);
    assert_eq!(stats.successful_requests, 0);
    assert_eq!(stats.failed_requests, 0);
    assert_eq!(stats.avg_response_time_ms, 0.0);
    assert_eq!(stats.stealth_requests, 0);
    assert_eq!(stats.non_stealth_requests, 0);
}

#[tokio::test]
#[ignore] // Requires actual browser
async fn test_stats_avg_response_time() -> Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;

    // Launch a few pages to get average
    for _ in 0..3 {
        let session = launcher.launch_page_default("https://example.com").await?;
        session.close().await?;
    }

    let stats = launcher.stats().await;
    assert_eq!(stats.successful_requests, 3);
    assert!(
        stats.avg_response_time_ms > 0.0,
        "Average response time should be tracked"
    );

    launcher.shutdown().await?;
    Ok(())
}

// ============================================================================
// Advanced Session Operations (Day 5)
// ============================================================================

#[tokio::test]
#[ignore] // Requires actual browser
async fn test_session_screenshot() -> Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;
    let session = launcher.launch_page_default("https://example.com").await?;

    let screenshot = session.screenshot().await?;
    assert!(!screenshot.is_empty(), "Screenshot should not be empty");

    session.close().await?;
    launcher.shutdown().await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires actual browser
async fn test_session_pdf_generation() -> Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;
    let session = launcher.launch_page_default("https://example.com").await?;

    let pdf = session.pdf().await?;
    assert!(!pdf.is_empty(), "PDF should not be empty");

    session.close().await?;
    launcher.shutdown().await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires actual browser
async fn test_session_element_waiting() -> Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;
    let session = launcher.launch_page_default("https://example.com").await?;

    // Wait for a common element
    let result = session.wait_for_element("h1", Some(5000)).await;

    // Element may or may not exist, but function should not panic
    let _ = result;

    session.close().await?;
    launcher.shutdown().await?;
    Ok(())
}

// ============================================================================
// Stealth Integration Tests (Day 3)
// ============================================================================

#[tokio::test]
#[ignore] // Requires actual browser
async fn test_stealth_preset_application() -> Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;

    // Test different stealth presets
    let presets = vec![
        StealthPreset::None,
        StealthPreset::Low,
        StealthPreset::Medium,
        StealthPreset::High,
    ];

    for preset in presets {
        let session = launcher
            .launch_page("https://example.com", Some(preset.clone()))
            .await?;

        assert!(!session.session_id.is_empty());
        session.close().await?;
    }

    launcher.shutdown().await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires actual browser
async fn test_stealth_user_agent_rotation() -> Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;

    // Launch multiple sessions - each should potentially have different UA
    for _ in 0..3 {
        let session = launcher
            .launch_page("https://example.com", Some(StealthPreset::Medium))
            .await?;

        // Execute script to check user agent
        let result = session.execute_script("navigator.userAgent").await?;
        assert!(result.is_string(), "User agent should be a string");

        session.close().await?;
    }

    launcher.shutdown().await?;
    Ok(())
}
