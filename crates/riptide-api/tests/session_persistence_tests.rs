//! Session persistence tests for stateful rendering
//!
//! Tests session state management, cookie persistence, and session expiration handling

#[cfg(feature = "browser")]
use riptide_api::rpc_client::RpcClient;
use riptide_api::sessions::manager::SessionManager;
use riptide_api::sessions::types::{Cookie, SessionConfig};
#[cfg(feature = "browser")]
use riptide_headless::dynamic::DynamicConfig;
use std::time::{Duration, SystemTime};

/// Test session persistence with RPC client
#[tokio::test]
#[cfg(feature = "browser")]
async fn test_session_persistence_integration() {
    let temp_dir =
        std::env::temp_dir().join(format!("riptide-session-test-{}", uuid::Uuid::new_v4()));

    let config = SessionConfig {
        base_data_dir: temp_dir.clone(),
        default_ttl: Duration::from_secs(3600),
        max_sessions: 100,
        cleanup_interval: Duration::from_secs(300),
        persist_cookies: true,
        encrypt_session_data: false,
    };

    let session_manager = SessionManager::new(config)
        .await
        .expect("Failed to create session manager");

    // Create a new session
    let session = session_manager
        .create_session()
        .await
        .expect("Failed to create session");
    let session_id = session.session_id.clone();

    // Get user data directory
    let user_data_dir = session_manager
        .get_user_data_dir(&session_id)
        .await
        .expect("Failed to get user data dir");

    // Verify directory exists
    assert!(user_data_dir.exists(), "User data directory should exist");

    // Create RPC client
    let rpc_client = RpcClient::new();

    // Create dynamic config
    let dynamic_config = DynamicConfig {
        actions: vec![],
        wait_for: None,
        scroll: None,
        capture_artifacts: false,
        timeout: Duration::from_secs(3),
        viewport: None,
    };

    // Note: This will fail if headless service is not running, but tests the API
    let result = rpc_client
        .render_dynamic_with_session(
            "http://example.com",
            &dynamic_config,
            None,
            Some(&session_id),
            Some(&user_data_dir.to_string_lossy()),
        )
        .await;

    // Clean up
    let _ = session_manager.remove_session(&session_id).await;
    let _ = std::fs::remove_dir_all(&temp_dir);

    // Result may be error if headless service not running, but API should be called correctly
    match result {
        Ok(_) => {
            // Success - headless service is running and returned data
        }
        Err(e) => {
            // Expected if headless service is not running
            assert!(
                e.to_string().contains("timed out")
                    || e.to_string().contains("connection")
                    || e.to_string().contains("refused"),
                "Error should be network-related: {}",
                e
            );
        }
    }
}

/// Test session cookie storage and retrieval
#[tokio::test]
async fn test_session_cookie_persistence() {
    let temp_dir =
        std::env::temp_dir().join(format!("riptide-cookie-test-{}", uuid::Uuid::new_v4()));

    let config = SessionConfig {
        base_data_dir: temp_dir.clone(),
        default_ttl: Duration::from_secs(3600),
        max_sessions: 100,
        cleanup_interval: Duration::from_secs(300),
        persist_cookies: true,
        encrypt_session_data: false,
    };

    let session_manager = SessionManager::new(config)
        .await
        .expect("Failed to create session manager");

    // Create session
    let session = session_manager
        .create_session()
        .await
        .expect("Failed to create session");
    let session_id = session.session_id.clone();

    // Add cookies
    let cookie1 = Cookie::new("auth_token".to_string(), "secret123".to_string())
        .with_domain("example.com".to_string())
        .with_path("/".to_string())
        .secure()
        .http_only();

    let cookie2 = Cookie::new("session_id".to_string(), "abc123".to_string())
        .with_domain("example.com".to_string())
        .with_expires(SystemTime::now() + Duration::from_secs(7200));

    session_manager
        .set_cookie(&session_id, "example.com", cookie1.clone())
        .await
        .expect("Failed to set cookie 1");

    session_manager
        .set_cookie(&session_id, "example.com", cookie2.clone())
        .await
        .expect("Failed to set cookie 2");

    // Retrieve cookies
    let cookies = session_manager
        .get_cookies_for_domain(&session_id, "example.com")
        .await
        .expect("Failed to get cookies");

    assert_eq!(cookies.len(), 2, "Should have 2 cookies");

    // Verify cookie attributes
    let auth_cookie = cookies
        .iter()
        .find(|c| c.name == "auth_token")
        .expect("Auth cookie not found");
    assert_eq!(auth_cookie.value, "secret123");
    assert!(auth_cookie.secure, "Auth cookie should be secure");
    assert!(auth_cookie.http_only, "Auth cookie should be HTTP only");

    // Verify cookies persisted to disk
    let cookies_file = temp_dir.join(&session_id).join("cookies.json");
    assert!(cookies_file.exists(), "Cookies file should exist");

    // Clean up
    let _ = session_manager.remove_session(&session_id).await;
    let _ = std::fs::remove_dir_all(&temp_dir);
}

/// Test session expiration handling
#[tokio::test]
async fn test_session_expiration() {
    let temp_dir =
        std::env::temp_dir().join(format!("riptide-expiry-test-{}", uuid::Uuid::new_v4()));

    let config = SessionConfig {
        base_data_dir: temp_dir.clone(),
        default_ttl: Duration::from_millis(100), // Very short TTL for testing
        max_sessions: 100,
        cleanup_interval: Duration::from_secs(1),
        persist_cookies: true,
        encrypt_session_data: false,
    };

    let session_manager = SessionManager::new(config)
        .await
        .expect("Failed to create session manager");

    // Create session
    let session = session_manager
        .create_session()
        .await
        .expect("Failed to create session");
    let session_id = session.session_id.clone();

    // Verify session exists
    let retrieved = session_manager
        .get_session(&session_id)
        .await
        .expect("Failed to get session");
    assert!(retrieved.is_some(), "Session should exist");

    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Cleanup expired sessions
    let cleaned = session_manager
        .cleanup_expired()
        .await
        .expect("Failed to cleanup");

    assert!(cleaned >= 1, "At least one session should be cleaned");

    // Verify session is gone
    let retrieved = session_manager
        .get_session(&session_id)
        .await
        .expect("Failed to get session");
    assert!(retrieved.is_none(), "Expired session should be removed");

    // Clean up
    let _ = std::fs::remove_dir_all(&temp_dir);
}

/// Test session state restoration
#[tokio::test]
async fn test_session_state_restoration() {
    let temp_dir =
        std::env::temp_dir().join(format!("riptide-restore-test-{}", uuid::Uuid::new_v4()));

    let config = SessionConfig {
        base_data_dir: temp_dir.clone(),
        default_ttl: Duration::from_secs(3600),
        max_sessions: 100,
        cleanup_interval: Duration::from_secs(300),
        persist_cookies: true,
        encrypt_session_data: false,
    };

    // Create first session manager and session
    let session_manager = SessionManager::new(config.clone())
        .await
        .expect("Failed to create session manager");
    let session = session_manager
        .create_session()
        .await
        .expect("Failed to create session");
    let session_id = session.session_id.clone();

    // Add cookie
    let cookie =
        Cookie::new("test".to_string(), "value".to_string()).with_domain("example.com".to_string());

    session_manager
        .set_cookie(&session_id, "example.com", cookie)
        .await
        .expect("Failed to set cookie");

    // Drop the session manager to simulate restart
    drop(session_manager);

    // Create new session manager - should load existing sessions
    let session_manager = SessionManager::new(config)
        .await
        .expect("Failed to create session manager");

    // Verify session was restored
    let restored_session = session_manager
        .get_session(&session_id)
        .await
        .expect("Failed to get session")
        .expect("Session should exist after restoration");

    assert_eq!(restored_session.session_id, session_id);

    // Verify cookies were restored
    let cookies = session_manager
        .get_cookies_for_domain(&session_id, "example.com")
        .await
        .expect("Failed to get cookies");

    assert_eq!(cookies.len(), 1, "Cookie should be restored");
    assert_eq!(cookies[0].name, "test");
    assert_eq!(cookies[0].value, "value");

    // Clean up
    let _ = session_manager.remove_session(&session_id).await;
    let _ = std::fs::remove_dir_all(&temp_dir);
}

/// Test session extension
#[tokio::test]
async fn test_session_extension() {
    let temp_dir =
        std::env::temp_dir().join(format!("riptide-extend-test-{}", uuid::Uuid::new_v4()));

    let config = SessionConfig {
        base_data_dir: temp_dir.clone(),
        default_ttl: Duration::from_secs(60),
        max_sessions: 100,
        cleanup_interval: Duration::from_secs(300),
        persist_cookies: true,
        encrypt_session_data: false,
    };

    let session_manager = SessionManager::new(config)
        .await
        .expect("Failed to create session manager");

    // Create session
    let session = session_manager
        .create_session()
        .await
        .expect("Failed to create session");
    let session_id = session.session_id.clone();
    let initial_expiry = session.expires_at;

    // Extend session
    session_manager
        .extend_session(&session_id, Duration::from_secs(3600))
        .await
        .expect("Failed to extend session");

    // Verify expiry was extended
    let extended_session = session_manager
        .get_session(&session_id)
        .await
        .expect("Failed to get session")
        .expect("Session should exist");

    assert!(
        extended_session.expires_at > initial_expiry,
        "Session expiry should be extended"
    );

    // Clean up
    let _ = session_manager.remove_session(&session_id).await;
    let _ = std::fs::remove_dir_all(&temp_dir);
}

/// Test user data directory creation
#[tokio::test]
async fn test_user_data_directory() {
    let temp_dir = std::env::temp_dir().join(format!("riptide-udd-test-{}", uuid::Uuid::new_v4()));

    let config = SessionConfig {
        base_data_dir: temp_dir.clone(),
        default_ttl: Duration::from_secs(3600),
        max_sessions: 100,
        cleanup_interval: Duration::from_secs(300),
        persist_cookies: true,
        encrypt_session_data: false,
    };

    let session_manager = SessionManager::new(config)
        .await
        .expect("Failed to create session manager");

    // Create session
    let session = session_manager
        .create_session()
        .await
        .expect("Failed to create session");
    let session_id = session.session_id.clone();

    // Get user data directory
    let user_data_dir = session_manager
        .get_user_data_dir(&session_id)
        .await
        .expect("Failed to get user data dir");

    // Verify directory exists and is in the right location
    assert!(user_data_dir.exists(), "User data directory should exist");
    assert!(
        user_data_dir.starts_with(&temp_dir),
        "User data dir should be in base dir"
    );
    assert!(
        user_data_dir.ends_with(&session_id),
        "User data dir should end with session ID"
    );

    // Verify session.json exists
    let session_file = user_data_dir.join("session.json");
    assert!(session_file.exists(), "Session file should exist");

    // Clean up
    let _ = session_manager.remove_session(&session_id).await;
    let _ = std::fs::remove_dir_all(&temp_dir);
}
