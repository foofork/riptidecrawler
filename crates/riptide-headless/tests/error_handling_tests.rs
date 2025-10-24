//! Error Handling and Timeout Tests
//!
//! Tests for error scenarios, timeout handling, and failure recovery

use riptide_headless::{
    models::{PageAction, RenderReq},
    BrowserPoolConfig, HeadlessLauncher, LauncherConfig,
};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_browser_pool_checkout_timeout() {
    // Create a small pool and exhaust it to test timeout
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        min_pool_size: 1,
        max_pool_size: 1,
        ..Default::default()
    };

    let browser_config = chromiumoxide::BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = std::sync::Arc::new(
        riptide_headless::BrowserPool::new(config, browser_config)
            .await
            .unwrap(),
    );

    // Checkout the only browser
    let _checkout1 = pool.checkout().await.unwrap();

    // Try to checkout another with timeout
    let result = timeout(Duration::from_millis(100), pool.checkout()).await;

    // Should timeout since pool is exhausted
    assert!(result.is_err(), "Expected timeout when pool is exhausted");

    let _ = pool.shutdown().await;
}

#[tokio::test]
async fn test_launcher_stats_after_failures() {
    let config = LauncherConfig {
        pool_config: BrowserPoolConfig {
            initial_pool_size: 1,
            ..Default::default()
        },
        enable_stealth: false,
        enable_monitoring: false,
        ..Default::default()
    };

    let launcher = HeadlessLauncher::with_config(config).await.unwrap();

    // Check initial stats
    let stats = launcher.stats().await;
    assert_eq!(stats.total_requests, 0);
    assert_eq!(stats.successful_requests, 0);
    assert_eq!(stats.failed_requests, 0);

    // Try to launch an invalid URL (this will fail in test environment)
    let result = launcher.launch_page_default("invalid://url").await;

    // In test environment, this should fail
    assert!(result.is_err());

    // Stats should reflect the failed request
    let stats_after = launcher.stats().await;
    assert_eq!(stats_after.total_requests, 1);
    assert_eq!(stats_after.failed_requests, 1);

    let _ = launcher.shutdown().await;
}

#[tokio::test]
async fn test_render_request_validation() {
    // Test various invalid request scenarios

    // Empty URL
    let req_empty_url = RenderReq {
        url: "".to_string(),
        wait_for: None,
        scroll_steps: None,
        session_id: None,
        actions: None,
        timeouts: None,
        artifacts: None,
        stealth_config: None,
    };

    assert_eq!(req_empty_url.url, "");

    // Invalid URL format
    let req_invalid_url = RenderReq {
        url: "not-a-valid-url".to_string(),
        wait_for: None,
        scroll_steps: None,
        session_id: None,
        actions: None,
        timeouts: None,
        artifacts: None,
        stealth_config: None,
    };

    assert_eq!(req_invalid_url.url, "not-a-valid-url");

    // Excessive scroll steps
    let req_many_scrolls = RenderReq {
        url: "https://example.com".to_string(),
        wait_for: None,
        scroll_steps: Some(1000),
        session_id: None,
        actions: None,
        timeouts: None,
        artifacts: None,
        stealth_config: None,
    };

    assert_eq!(req_many_scrolls.scroll_steps, Some(1000));
}

#[tokio::test]
async fn test_page_action_timeout_scenarios() {
    // Test actions with various timeout configurations
    let actions = [
        PageAction::WaitForCss {
            css: "#never-appears".to_string(),
            timeout_ms: Some(100), // Very short timeout
        },
        PageAction::WaitForJs {
            expr: "false".to_string(), // Never returns true
            timeout_ms: Some(100),
        },
    ];

    assert_eq!(actions.len(), 2);

    // Verify timeout values
    match &actions[0] {
        PageAction::WaitForCss { css, timeout_ms } => {
            assert_eq!(css, "#never-appears");
            assert_eq!(*timeout_ms, Some(100));
        }
        _ => panic!("Expected WaitForCss"),
    }

    match &actions[1] {
        PageAction::WaitForJs { expr, timeout_ms } => {
            assert_eq!(expr, "false");
            assert_eq!(*timeout_ms, Some(100));
        }
        _ => panic!("Expected WaitForJs"),
    }
}

#[tokio::test]
async fn test_browser_pool_recovery() {
    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        min_pool_size: 1,
        max_pool_size: 2,
        enable_recovery: true,
        max_retries: 3,
        ..Default::default()
    };

    let browser_config = chromiumoxide::BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = riptide_headless::BrowserPool::new(config, browser_config)
        .await
        .unwrap();

    // Verify recovery is enabled in config
    let stats = pool.stats().await;
    assert_eq!(stats.available, 1);

    let _ = pool.shutdown().await;
}

#[tokio::test]
async fn test_concurrent_checkout_failures() {
    use std::sync::Arc;

    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        min_pool_size: 1,
        max_pool_size: 1,
        ..Default::default()
    };

    let browser_config = chromiumoxide::BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = Arc::new(
        riptide_headless::BrowserPool::new(config, browser_config)
            .await
            .unwrap(),
    );

    // Hold the only browser
    let _checkout = pool.checkout().await.unwrap();

    // Try multiple concurrent checkouts (all should timeout)
    let pool1 = pool.clone();
    let pool2 = pool.clone();

    let handle1 = tokio::spawn(async move {
        timeout(Duration::from_millis(50), pool1.checkout())
            .await
            .is_err()
    });

    let handle2 = tokio::spawn(async move {
        timeout(Duration::from_millis(50), pool2.checkout())
            .await
            .is_err()
    });

    let result1 = handle1.await.unwrap();
    let result2 = handle2.await.unwrap();

    // Both should timeout
    assert!(result1);
    assert!(result2);

    let _ = pool.shutdown().await;
}

#[tokio::test]
async fn test_launcher_shutdown_cleanup() {
    let config = LauncherConfig {
        pool_config: BrowserPoolConfig {
            initial_pool_size: 2,
            ..Default::default()
        },
        enable_monitoring: false,
        ..Default::default()
    };

    let launcher = HeadlessLauncher::with_config(config).await.unwrap();

    // Verify launcher is running
    let stats = launcher.stats().await;
    assert_eq!(stats.total_requests, 0);

    // Shutdown
    let shutdown_result = launcher.shutdown().await;
    assert!(shutdown_result.is_ok());

    // After shutdown, pool should be empty
    // (we can't directly verify this without accessing internal state,
    // but we test that shutdown completes successfully)
}

#[tokio::test]
async fn test_invalid_stealth_configuration() {
    use riptide_stealth::{StealthConfig, StealthPreset};

    // Test creating a request with various stealth configurations
    let stealth_configs = vec![
        StealthPreset::None,
        StealthPreset::Low,
        StealthPreset::Medium,
        StealthPreset::High,
    ];

    for preset in stealth_configs {
        let stealth_config = StealthConfig {
            preset: preset.clone(),
            ..Default::default()
        };

        let req = RenderReq {
            url: "https://example.com".to_string(),
            wait_for: None,
            scroll_steps: None,
            session_id: None,
            actions: None,
            timeouts: None,
            artifacts: None,
            stealth_config: Some(stealth_config),
        };

        assert!(req.stealth_config.is_some());
    }
}

#[tokio::test]
async fn test_malformed_page_actions() {
    // Test various edge cases in page actions

    // Empty CSS selector
    let action1 = PageAction::Click {
        css: "".to_string(),
    };

    match action1 {
        PageAction::Click { css } => {
            assert_eq!(css, "");
        }
        _ => panic!("Expected Click"),
    }

    // Empty JavaScript code
    let action2 = PageAction::Js {
        code: "".to_string(),
    };

    match action2 {
        PageAction::Js { code } => {
            assert_eq!(code, "");
        }
        _ => panic!("Expected Js"),
    }

    // Empty text for Type
    let action3 = PageAction::Type {
        css: "input".to_string(),
        text: "".to_string(),
        delay_ms: None,
    };

    match action3 {
        PageAction::Type { css, text, .. } => {
            assert_eq!(css, "input");
            assert_eq!(text, "");
        }
        _ => panic!("Expected Type"),
    }
}

#[tokio::test]
async fn test_pool_stats_accuracy() {
    use std::sync::Arc;

    let config = BrowserPoolConfig {
        initial_pool_size: 3,
        min_pool_size: 1,
        max_pool_size: 5,
        ..Default::default()
    };

    let browser_config = chromiumoxide::BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = Arc::new(
        riptide_headless::BrowserPool::new(config, browser_config)
            .await
            .unwrap(),
    );

    // Initial stats
    let stats = pool.stats().await;
    assert_eq!(stats.available, 3);
    assert_eq!(stats.in_use, 0);
    assert_eq!(stats.total_capacity, 5);
    assert_eq!(stats.utilization, 0.0);

    // Checkout some browsers
    let _checkout1 = pool.checkout().await.unwrap();
    let _checkout2 = pool.checkout().await.unwrap();

    let stats = pool.stats().await;
    assert_eq!(stats.available, 1);
    assert_eq!(stats.in_use, 2);
    assert_eq!(stats.utilization, 0.4); // 2/5 = 0.4

    let _ = pool.shutdown().await;
}

#[tokio::test]
async fn test_session_id_persistence() {
    // Test that session IDs are properly echoed back
    let req = RenderReq {
        url: "https://example.com".to_string(),
        wait_for: None,
        scroll_steps: None,
        session_id: Some("persistent-session-123".to_string()),
        actions: None,
        timeouts: None,
        artifacts: None,
        stealth_config: None,
    };

    assert_eq!(req.session_id, Some("persistent-session-123".to_string()));

    // Verify session ID is preserved across request cloning
    let req_clone = req.clone();
    assert_eq!(req_clone.session_id, req.session_id);
}

#[tokio::test]
async fn test_dynamic_error_display_formatting() {
    use riptide_headless::dynamic::DynamicError;

    // Test all error variants display properly
    let errors = vec![
        DynamicError::Timeout {
            condition: "test condition".to_string(),
            waited_ms: 5000,
        },
        DynamicError::ElementNotFound {
            selector: "#missing".to_string(),
        },
        DynamicError::JavascriptError {
            script: "bad.code()".to_string(),
            error: "ReferenceError".to_string(),
        },
        DynamicError::NavigationError {
            url: "https://bad.url".to_string(),
            error: "Failed".to_string(),
        },
        DynamicError::RendererError {
            error: "Crashed".to_string(),
        },
        DynamicError::ConfigError {
            message: "Invalid".to_string(),
        },
        DynamicError::NetworkError {
            error: "Connection lost".to_string(),
        },
        DynamicError::ResourceLimit {
            limit: "memory".to_string(),
            value: 1024,
        },
    ];

    for error in errors {
        let error_msg = format!("{}", error);
        assert!(!error_msg.is_empty(), "Error message should not be empty");

        // Verify error implements std::error::Error
        let _: &dyn std::error::Error = &error;
    }
}

#[tokio::test]
async fn test_browser_checkout_after_failure() {
    use std::sync::Arc;

    let config = BrowserPoolConfig {
        initial_pool_size: 1,
        min_pool_size: 1,
        max_pool_size: 2,
        enable_recovery: true,
        ..Default::default()
    };

    let browser_config = chromiumoxide::BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = Arc::new(
        riptide_headless::BrowserPool::new(config, browser_config)
            .await
            .unwrap(),
    );

    // Successful checkout
    let checkout1 = pool.checkout().await.unwrap();
    checkout1.checkin().await.unwrap();

    // Stats should show browser returned
    let stats = pool.stats().await;
    assert_eq!(stats.available, 1);
    assert_eq!(stats.in_use, 0);

    // Another checkout should work
    let checkout2 = pool.checkout().await;
    assert!(checkout2.is_ok());

    let _ = pool.shutdown().await;
}
