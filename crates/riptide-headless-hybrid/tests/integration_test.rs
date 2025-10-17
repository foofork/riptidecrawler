//! Integration tests for hybrid headless launcher

use anyhow::Result;
use riptide_headless_hybrid::{HybridHeadlessLauncher, LauncherConfig};
use riptide_stealth::StealthPreset;

#[tokio::test]
async fn test_hybrid_launcher_creation() -> Result<()> {
    let config = LauncherConfig {
        enable_stealth: false,
        ..Default::default()
    };

    let launcher = HybridHeadlessLauncher::with_config(config).await?;
    let stats = launcher.stats().await;

    assert_eq!(stats.total_requests, 0);
    assert_eq!(stats.successful_requests, 0);
    assert_eq!(stats.failed_requests, 0);

    launcher.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn test_stats_initialization() -> Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;
    let stats = launcher.stats().await;

    assert_eq!(stats.total_requests, 0);
    assert_eq!(stats.stealth_requests, 0);
    assert_eq!(stats.non_stealth_requests, 0);

    launcher.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn test_different_stealth_presets() -> Result<()> {
    let presets = vec![
        StealthPreset::None,
        StealthPreset::Low,
        StealthPreset::Medium,
        StealthPreset::High,
    ];

    for preset in presets {
        let config = LauncherConfig {
            default_stealth_preset: preset.clone(),
            enable_stealth: preset != StealthPreset::None,
            ..Default::default()
        };

        let launcher = HybridHeadlessLauncher::with_config(config).await?;
        let stats = launcher.stats().await;
        assert_eq!(stats.total_requests, 0);

        launcher.shutdown().await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_launcher_shutdown() -> Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;

    // Shutdown should succeed
    launcher.shutdown().await?;

    // Multiple shutdowns should be safe
    launcher.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_compatibility_with_existing_api() -> Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;

    // Test that the API matches expectations
    let _stats = launcher.stats().await;

    // Test different launch methods exist
    // Note: These would require actual browser launch which we skip in tests
    // let _session = launcher.launch_page_default("about:blank").await?;
    // let _session = launcher.launch_page_no_stealth("about:blank").await?;

    launcher.shutdown().await?;
    Ok(())
}

// Browser launch tests are commented out because they require Chrome/Chromium
// These would be run in a full integration test environment

/*
#[tokio::test]
async fn test_page_launch() -> Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;

    // Launch a page
    let session = launcher.launch_page_default("about:blank").await?;

    // Verify session
    assert!(!session.session_id().is_empty());
    assert!(session.duration().as_millis() >= 0);

    // Close session
    session.close().await?;

    launcher.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn test_stealth_applied() -> Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;

    // Launch with stealth
    let session = launcher
        .launch_page("about:blank", Some(StealthPreset::High))
        .await?;

    // Execute script to check webdriver is undefined
    let result = session
        .execute_script("return navigator.webdriver === undefined;")
        .await?;

    assert_eq!(result, serde_json::json!(true));

    session.close().await?;
    launcher.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn test_page_navigation() -> Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;

    let session = launcher.launch_page_default("about:blank").await?;

    // Navigate to another URL
    session.navigate("https://example.com").await?;

    // Get content
    let content = session.content().await?;
    assert!(content.contains("Example Domain") || content.len() > 0);

    session.close().await?;
    launcher.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn test_screenshot() -> Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;

    let session = launcher.launch_page_default("about:blank").await?;

    // Take screenshot
    let screenshot = session.screenshot().await?;
    assert!(!screenshot.is_empty());

    session.close().await?;
    launcher.shutdown().await?;
    Ok(())
}
*/
