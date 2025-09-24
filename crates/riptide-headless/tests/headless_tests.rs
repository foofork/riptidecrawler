use riptide_headless::{BrowserPool, BrowserConfig, SessionManager, LaunchOptions};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_browser_config_defaults() {
    let config = BrowserConfig::default();

    assert!(config.headless);
    assert_eq!(config.timeout, Duration::from_secs(30));
    assert!(!config.devtools);
    assert!(config.user_agent.contains("Mozilla"));
}

#[tokio::test]
async fn test_browser_config_builder() {
    let config = BrowserConfig::builder()
        .headless(false)
        .timeout(Duration::from_secs(60))
        .devtools(true)
        .viewport(1920, 1080)
        .user_agent("Custom User Agent")
        .build();

    assert!(!config.headless);
    assert_eq!(config.timeout, Duration::from_secs(60));
    assert!(config.devtools);
    assert_eq!(config.viewport_width, 1920);
    assert_eq!(config.viewport_height, 1080);
    assert_eq!(config.user_agent, "Custom User Agent");
}

#[tokio::test]
async fn test_browser_pool_creation() {
    let pool = BrowserPool::new(3, BrowserConfig::default());

    assert_eq!(pool.size(), 3);
    assert_eq!(pool.available(), 3);
    assert_eq!(pool.in_use(), 0);
}

#[tokio::test]
async fn test_browser_pool_checkout() {
    let pool = Arc::new(BrowserPool::new(2, BrowserConfig::default()));

    // Checkout first browser
    let browser1 = pool.checkout().await;
    assert!(browser1.is_ok());
    assert_eq!(pool.available(), 1);
    assert_eq!(pool.in_use(), 1);

    // Checkout second browser
    let browser2 = pool.checkout().await;
    assert!(browser2.is_ok());
    assert_eq!(pool.available(), 0);
    assert_eq!(pool.in_use(), 2);

    // Try to checkout third browser - should wait or timeout
    let pool_clone = pool.clone();
    let checkout_task = tokio::spawn(async move {
        tokio::time::timeout(
            Duration::from_millis(100),
            pool_clone.checkout()
        ).await
    });

    let result = checkout_task.await.unwrap();
    assert!(result.is_err()); // Should timeout
}

#[tokio::test]
async fn test_browser_pool_checkin() {
    let pool = Arc::new(BrowserPool::new(1, BrowserConfig::default()));

    let browser = pool.checkout().await.unwrap();
    assert_eq!(pool.available(), 0);

    pool.checkin(browser).await;
    assert_eq!(pool.available(), 1);
    assert_eq!(pool.in_use(), 0);
}

#[tokio::test]
async fn test_session_manager_creation() {
    let session_manager = SessionManager::new();

    let session_id = session_manager.create_session("user123").await;
    assert!(session_id.len() > 0);

    let session = session_manager.get_session(&session_id).await;
    assert!(session.is_some());
    assert_eq!(session.unwrap().user_id, "user123");
}

#[tokio::test]
async fn test_session_cookies() {
    let session_manager = SessionManager::new();
    let session_id = session_manager.create_session("user123").await;

    // Add cookies
    session_manager.add_cookie(
        &session_id,
        "auth_token",
        "secret123",
        "example.com",
    ).await;

    session_manager.add_cookie(
        &session_id,
        "session_id",
        "sess_abc",
        "example.com",
    ).await;

    // Get cookies
    let cookies = session_manager.get_cookies(&session_id, "example.com").await;
    assert_eq!(cookies.len(), 2);

    let auth_cookie = cookies.iter().find(|c| c.name == "auth_token");
    assert!(auth_cookie.is_some());
    assert_eq!(auth_cookie.unwrap().value, "secret123");
}

#[tokio::test]
async fn test_session_expiration() {
    let session_manager = SessionManager::new();

    let session_id = session_manager.create_session_with_ttl(
        "user123",
        Duration::from_millis(100)
    ).await;

    // Session should exist initially
    assert!(session_manager.get_session(&session_id).await.is_some());

    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Session should be expired
    assert!(session_manager.get_session(&session_id).await.is_none());
}

#[tokio::test]
async fn test_launch_options() {
    let options = LaunchOptions::builder()
        .headless(true)
        .args(vec!["--no-sandbox", "--disable-dev-shm-usage"])
        .env("DISPLAY", ":99")
        .user_data_dir("/tmp/chrome-profile")
        .build();

    assert!(options.headless);
    assert_eq!(options.args.len(), 2);
    assert!(options.env.contains_key("DISPLAY"));
    assert_eq!(options.user_data_dir, Some("/tmp/chrome-profile".to_string()));
}

#[tokio::test]
async fn test_browser_pool_with_timeout() {
    let mut config = BrowserConfig::default();
    config.timeout = Duration::from_secs(5);

    let pool = BrowserPool::new(1, config);
    let browser = pool.checkout().await.unwrap();

    // Simulate browser operation
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Browser should still be valid
    assert!(!browser.is_closed());

    pool.checkin(browser).await;
}

#[tokio::test]
async fn test_browser_pool_cleanup() {
    let pool = Arc::new(BrowserPool::new(3, BrowserConfig::default()));

    // Checkout all browsers
    let mut browsers = Vec::new();
    for _ in 0..3 {
        browsers.push(pool.checkout().await.unwrap());
    }

    assert_eq!(pool.in_use(), 3);

    // Return browsers
    for browser in browsers {
        pool.checkin(browser).await;
    }

    // Cleanup pool
    pool.cleanup().await;

    // Pool should be empty after cleanup
    assert_eq!(pool.size(), 0);
}

#[tokio::test]
async fn test_browser_health_check() {
    let pool = BrowserPool::new(2, BrowserConfig::default());

    let browser = pool.checkout().await.unwrap();

    // Health check should pass for new browser
    let is_healthy = browser.health_check().await;
    assert!(is_healthy);

    pool.checkin(browser).await;
}

#[tokio::test]
async fn test_session_manager_cleanup() {
    let session_manager = SessionManager::new();

    // Create multiple sessions
    let mut session_ids = Vec::new();
    for i in 0..5 {
        let id = session_manager.create_session(&format!("user{}", i)).await;
        session_ids.push(id);
    }

    assert_eq!(session_manager.active_sessions().await, 5);

    // Clean up expired sessions (none should be expired yet)
    let removed = session_manager.cleanup_expired().await;
    assert_eq!(removed, 0);

    // Manually remove a session
    session_manager.remove_session(&session_ids[0]).await;
    assert_eq!(session_manager.active_sessions().await, 4);
}