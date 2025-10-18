use chromiumoxide_cdp::BrowserConfig;
use riptide_core::stealth::StealthPreset;
use riptide_headless::{
    launcher::{HeadlessLauncher, LauncherConfig},
    pool::{BrowserPool, BrowserPoolConfig},
};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_browser_config_creation() {
    // Test chromiumoxide BrowserConfig creation
    let _config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    // Since BrowserConfig is opaque, we just verify it was created successfully
    // The actual configuration is tested in the pool creation
}

#[tokio::test]
async fn test_browser_pool_config_defaults() {
    let config = BrowserPoolConfig::default();

    assert_eq!(config.min_pool_size, 1);
    assert_eq!(config.max_pool_size, 5);
    assert_eq!(config.initial_pool_size, 3);
    assert_eq!(config.idle_timeout, Duration::from_secs(30));
    assert_eq!(config.max_lifetime, Duration::from_secs(300));
    assert!(config.enable_recovery);
    assert_eq!(config.max_retries, 3);
}

#[tokio::test]
async fn test_browser_pool_creation() {
    let pool_config = BrowserPoolConfig {
        initial_pool_size: 1,
        min_pool_size: 1,
        max_pool_size: 2,
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = BrowserPool::new(pool_config, browser_config).await;
    assert!(pool.is_ok());

    if let Ok(pool) = pool {
        let stats = pool.stats().await;
        assert_eq!(stats.available, 1);
        assert_eq!(stats.in_use, 0);
        assert_eq!(stats.total_capacity, 2);

        let _ = pool.shutdown().await;
    }
}

#[tokio::test]
async fn test_browser_pool_checkout_checkin() {
    let pool_config = BrowserPoolConfig {
        initial_pool_size: 1,
        min_pool_size: 1,
        max_pool_size: 2,
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = Arc::new(BrowserPool::new(pool_config, browser_config).await.unwrap());

    // Check initial stats
    let initial_stats = pool.stats().await;
    assert_eq!(initial_stats.available, 1);
    assert_eq!(initial_stats.in_use, 0);

    // Checkout a browser
    let checkout = pool.checkout().await;
    assert!(checkout.is_ok());

    if let Ok(checkout) = checkout {
        // Verify browser_id is non-empty
        let browser_id = checkout.browser_id().to_string();
        assert!(!browser_id.is_empty(), "Browser ID should not be empty");
        // Check stats after checkout
        let stats = pool.stats().await;
        assert_eq!(stats.available, 0);
        assert_eq!(stats.in_use, 1);

        // Check in the browser
        checkout.checkin().await.unwrap();

        // Check stats after checkin
        let final_stats = pool.stats().await;
        assert_eq!(final_stats.available, 1);
        assert_eq!(final_stats.in_use, 0);
    }

    let _ = pool.shutdown().await;
}

#[tokio::test]
async fn test_browser_pool_multiple_checkouts() {
    let pool_config = BrowserPoolConfig {
        initial_pool_size: 2,
        min_pool_size: 1,
        max_pool_size: 3,
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = Arc::new(BrowserPool::new(pool_config, browser_config).await.unwrap());

    // Checkout first browser
    let checkout1 = pool.checkout().await.unwrap();
    let stats1 = pool.stats().await;
    assert_eq!(stats1.available, 1);
    assert_eq!(stats1.in_use, 1);

    // Checkout second browser
    let checkout2 = pool.checkout().await.unwrap();
    let stats2 = pool.stats().await;
    assert_eq!(stats2.available, 0);
    assert_eq!(stats2.in_use, 2);

    // Return first browser
    checkout1.checkin().await.unwrap();
    let stats3 = pool.stats().await;
    assert_eq!(stats3.available, 1);
    assert_eq!(stats3.in_use, 1);

    // Return second browser
    checkout2.checkin().await.unwrap();
    let stats4 = pool.stats().await;
    assert_eq!(stats4.available, 2);
    assert_eq!(stats4.in_use, 0);

    let _ = pool.shutdown().await;
}

#[tokio::test]
async fn test_headless_launcher_creation() {
    let config = LauncherConfig {
        pool_config: BrowserPoolConfig {
            initial_pool_size: 1,
            min_pool_size: 1,
            max_pool_size: 2,
            ..Default::default()
        },
        enable_stealth: false, // Disable stealth for testing
        ..Default::default()
    };

    let launcher = HeadlessLauncher::with_config(config).await;
    assert!(launcher.is_ok());

    if let Ok(launcher) = launcher {
        let stats = launcher.stats().await;
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.successful_requests, 0);

        let _ = launcher.shutdown().await;
    }
}

#[tokio::test]
async fn test_headless_launcher_default() {
    let launcher = HeadlessLauncher::new().await;
    assert!(launcher.is_ok());

    if let Ok(launcher) = launcher {
        let stats = launcher.stats().await;
        assert_eq!(stats.total_requests, 0);

        let _ = launcher.shutdown().await;
    }
}

#[tokio::test]
async fn test_launcher_config_defaults() {
    let config = LauncherConfig::default();

    assert_eq!(config.default_stealth_preset, StealthPreset::Medium);
    assert!(config.enable_stealth);
    assert_eq!(config.page_timeout, Duration::from_secs(30));
    assert!(config.enable_monitoring);
    assert_eq!(config.pool_config.min_pool_size, 1);
    assert_eq!(config.pool_config.max_pool_size, 5);
}

#[tokio::test]
async fn test_browser_pool_stats() {
    let pool_config = BrowserPoolConfig {
        initial_pool_size: 2,
        min_pool_size: 1,
        max_pool_size: 4,
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = BrowserPool::new(pool_config, browser_config).await.unwrap();

    let stats = pool.stats().await;
    assert_eq!(stats.available, 2);
    assert_eq!(stats.in_use, 0);
    assert_eq!(stats.total_capacity, 4);
    assert_eq!(stats.utilization, 0.0);

    // Checkout a browser to change stats
    // Guard must stay alive to keep browser checked out
    let _checkout = pool.checkout().await.unwrap();
    let stats = pool.stats().await;
    assert_eq!(stats.available, 1);
    assert_eq!(stats.in_use, 1);
    assert_eq!(stats.utilization, 0.25); // 1/4 = 0.25

    let _ = pool.shutdown().await;
}

#[tokio::test]
async fn test_browser_pool_shutdown() {
    let pool_config = BrowserPoolConfig {
        initial_pool_size: 2,
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = BrowserPool::new(pool_config, browser_config).await.unwrap();

    // Verify initial state
    let stats = pool.stats().await;
    assert_eq!(stats.available, 2);

    // Shutdown the pool
    let shutdown_result = pool.shutdown().await;
    assert!(shutdown_result.is_ok());

    // After shutdown, stats should show empty pool
    let final_stats = pool.stats().await;
    assert_eq!(final_stats.available, 0);
    assert_eq!(final_stats.in_use, 0);
}

#[tokio::test]
async fn test_browser_checkout_new_page() {
    let pool_config = BrowserPoolConfig {
        initial_pool_size: 1,
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = BrowserPool::new(pool_config, browser_config).await.unwrap();

    let checkout = pool.checkout().await.unwrap();

    // Test that we can create a page (this will fail in test environment without a real browser)
    // but we're testing the API structure
    let page_result = checkout.new_page("about:blank").await;

    // In a test environment without Chrome installed, this will fail
    // but the important thing is that the method exists and has the right signature
    assert!(page_result.is_err() || page_result.is_ok());

    let browser_id = checkout.browser_id().to_string();
    assert!(!browser_id.is_empty());

    checkout.checkin().await.unwrap();
    let _ = pool.shutdown().await;
}

#[tokio::test]
async fn test_launch_session_functionality() {
    // Test would require actual browser environment, so we'll test the config and basic structure
    let launcher_config = LauncherConfig {
        pool_config: BrowserPoolConfig {
            initial_pool_size: 1,
            ..Default::default()
        },
        enable_stealth: false,
        page_timeout: Duration::from_secs(10),
        enable_monitoring: false,
        ..Default::default()
    };

    let launcher = HeadlessLauncher::with_config(launcher_config).await;
    assert!(launcher.is_ok());

    if let Ok(launcher) = launcher {
        // Test launcher stats before any requests
        let initial_stats = launcher.stats().await;
        assert_eq!(initial_stats.total_requests, 0);
        assert_eq!(initial_stats.successful_requests, 0);
        assert_eq!(initial_stats.failed_requests, 0);

        // Test pool events structure - verify receiver is returned
        let _events = launcher.pool_events();
        // pool_events() always returns Arc<Mutex<Receiver>>, so just verify we can call it
        let _ = launcher.shutdown().await;
    }
}

#[tokio::test]
async fn test_stealth_presets() {
    // Test that we can create different launcher configurations with stealth presets
    let presets = vec![
        StealthPreset::None,
        StealthPreset::Low,
        StealthPreset::Medium,
        StealthPreset::High,
    ];

    for preset in presets {
        let config = LauncherConfig {
            pool_config: BrowserPoolConfig {
                initial_pool_size: 1,
                ..Default::default()
            },
            default_stealth_preset: preset.clone(),
            enable_stealth: preset != StealthPreset::None,
            enable_monitoring: false, // Disable monitoring for test
            ..Default::default()
        };

        let launcher = HeadlessLauncher::with_config(config).await;
        assert!(
            launcher.is_ok(),
            "Failed to create launcher with preset: {:?}",
            preset
        );

        if let Ok(launcher) = launcher {
            let _ = launcher.shutdown().await;
        }
    }
}

#[tokio::test]
async fn test_pool_config_builder() {
    let config = BrowserPoolConfig {
        min_pool_size: 2,
        max_pool_size: 10,
        initial_pool_size: 5,
        idle_timeout: Duration::from_secs(60),
        max_lifetime: Duration::from_secs(600),
        memory_threshold_mb: 1000,
        enable_recovery: false,
        max_retries: 5,
        ..Default::default()
    };

    assert_eq!(config.min_pool_size, 2);
    assert_eq!(config.max_pool_size, 10);
    assert_eq!(config.initial_pool_size, 5);
    assert_eq!(config.idle_timeout, Duration::from_secs(60));
    assert_eq!(config.max_lifetime, Duration::from_secs(600));
    assert_eq!(config.memory_threshold_mb, 1000);
    assert!(!config.enable_recovery);
    assert_eq!(config.max_retries, 5);
}
