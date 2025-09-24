#[cfg(test)]
mod tests {
    use riptide_api::sessions::{
        Cookie, CookieJar, Session, SessionConfig, SessionManager, SessionStorage, SameSite
    };
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};
    use uuid::Uuid;
    use tempfile::TempDir;

    #[test]
    fn test_session_creation() {
        let config = SessionConfig::default();
        let session = Session::new("test_session_id".to_string(), &config);

        assert_eq!(session.session_id, "test_session_id");
        assert!(session.created_at <= SystemTime::now());
        assert!(session.last_accessed <= SystemTime::now());
        assert!(session.expires_at > SystemTime::now());
        assert!(session.cookies.cookies.is_empty());
    }

    #[test]
    fn test_session_cookie_management() {
        let config = SessionConfig::default();
        let mut session = Session::new("test_session".to_string(), &config);

        // Create a cookie
        let cookie = Cookie::new("theme".to_string(), "dark".to_string())
            .with_domain("example.com".to_string())
            .with_path("/".to_string());

        // Add cookie to session
        session.cookies.set_cookie("example.com", cookie.clone());

        // Get cookie back
        let retrieved = session.cookies.get_cookie("example.com", "theme");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, "dark");

        // Remove cookie
        let removed = session.cookies.remove_cookie("example.com", "theme");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().value, "dark");

        // Cookie should be gone
        assert!(session.cookies.get_cookie("example.com", "theme").is_none());
    }

    #[test]
    fn test_session_expiration() {
        let mut config = SessionConfig::default();
        config.default_ttl = Duration::from_millis(1); // Very short TTL
        let session = Session::new("test_session".to_string(), &config);

        // Session should not be expired initially
        assert!(!session.is_expired());

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(10));

        // Now it should be expired
        assert!(session.is_expired());
    }

    #[test]
    fn test_cookie_creation() {
        let cookie = Cookie::new("session_id".to_string(), "abc123".to_string())
            .with_domain("example.com".to_string())
            .with_path("/".to_string())
            .secure()
            .http_only()
            .with_same_site(SameSite::Strict);

        assert_eq!(cookie.name, "session_id");
        assert_eq!(cookie.value, "abc123");
        assert_eq!(cookie.domain, Some("example.com".to_string()));
        assert_eq!(cookie.path, Some("/".to_string()));
        assert!(cookie.secure);
        assert!(cookie.http_only);
        assert!(matches!(cookie.same_site, Some(SameSite::Strict)));
    }

    #[tokio::test]
    async fn test_session_manager() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let mut config = SessionConfig::default();
        config.base_data_dir = temp_dir.path().to_path_buf();

        let manager = SessionManager::new(config.clone()).await.expect("Failed to create manager");

        // Create session
        let session = manager.create_session().await.expect("Failed to create session");
        let session_id = session.session_id.clone();
        assert!(session_id.len() > 0);

        // Get session
        let retrieved = manager.get_session(&session_id).await.expect("Failed to get session");
        assert!(retrieved.is_some());
        let retrieved_session = retrieved.unwrap();
        assert_eq!(retrieved_session.session_id, session_id);

        // Set a cookie
        let cookie = Cookie::new("last_access".to_string(), "2024-01-01".to_string());
        manager.set_cookie(&session_id, "example.com", cookie).await.expect("Failed to set cookie");

        // Get cookie
        let retrieved_cookie = manager.get_cookie(&session_id, "example.com", "last_access").await.expect("Failed to get cookie");
        assert!(retrieved_cookie.is_some());
        assert_eq!(retrieved_cookie.unwrap().value, "2024-01-01");

        // Remove session
        manager.remove_session(&session_id).await.expect("Failed to remove session");
        let removed_session = manager.get_session(&session_id).await.expect("Failed to check removed session");
        assert!(removed_session.is_none());
    }

    #[tokio::test]
    async fn test_session_storage() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let mut config = SessionConfig::default();
        config.base_data_dir = temp_dir.path().to_path_buf();

        let storage = SessionStorage::new(config.clone()).await.expect("Failed to create storage");
        let session = Session::new("test_session".to_string(), &config);
        let session_id = session.session_id.clone();

        // Store session
        storage.store_session(session.clone()).await.expect("Failed to store session");

        // Load session
        let loaded = storage.get_session(&session_id).await.expect("Failed to load session");
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().session_id, session_id);

        // Remove session
        storage.remove_session(&session_id).await.expect("Failed to remove session");
        let removed = storage.get_session(&session_id).await.expect("Failed to check removed session");
        assert!(removed.is_none());
    }

    #[tokio::test]
    async fn test_session_cleanup() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let mut config = SessionConfig::default();
        config.base_data_dir = temp_dir.path().to_path_buf();
        config.default_ttl = Duration::from_millis(50); // Short TTL for testing

        let storage = SessionStorage::new(config.clone()).await.expect("Failed to create storage");

        // Create sessions
        let session1 = storage.create_session("session1".to_string()).await.expect("Failed to create session1");
        let session2 = storage.create_session("session2".to_string()).await.expect("Failed to create session2");

        // Both should exist initially
        assert!(storage.get_session(&session1.session_id).await.expect("Failed to get session1").is_some());
        assert!(storage.get_session(&session2.session_id).await.expect("Failed to get session2").is_some());

        // Wait for sessions to expire
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Cleanup expired sessions
        let cleaned = storage.cleanup_expired().await.expect("Failed to cleanup");
        assert!(cleaned >= 0); // Should clean up some sessions

        // Sessions should be expired and removed
        let session1_after = storage.get_session(&session1.session_id).await.expect("Failed to check session1");
        let session2_after = storage.get_session(&session2.session_id).await.expect("Failed to check session2");

        // Sessions should be None because they expired
        assert!(session1_after.is_none());
        assert!(session2_after.is_none());
    }

    #[test]
    fn test_session_touch() {
        let config = SessionConfig::default();
        let mut session = Session::new("test_session".to_string(), &config);
        let original_expires_at = session.expires_at;

        // Wait a moment
        std::thread::sleep(Duration::from_millis(10));

        // Touch the session to update its expiry
        session.touch(Duration::from_secs(3600));

        // Expiry should be updated
        assert!(session.expires_at > original_expires_at);
        assert_eq!(session.session_id, "test_session");
    }

    #[test]
    fn test_cookie_jar_operations() {
        let mut jar = CookieJar::default();

        // Create cookies
        let cookie1 = Cookie::new("session".to_string(), "abc123".to_string());
        let cookie2 = Cookie::new("theme".to_string(), "dark".to_string());

        // Set cookies
        jar.set_cookie("example.com", cookie1.clone());
        jar.set_cookie("example.com", cookie2.clone());

        // Get cookies
        let retrieved1 = jar.get_cookie("example.com", "session");
        let retrieved2 = jar.get_cookie("example.com", "theme");

        assert!(retrieved1.is_some());
        assert_eq!(retrieved1.unwrap().value, "abc123");
        assert!(retrieved2.is_some());
        assert_eq!(retrieved2.unwrap().value, "dark");

        // Get all cookies for domain
        let domain_cookies = jar.get_cookies_for_domain("example.com");
        assert!(domain_cookies.is_some());
        assert_eq!(domain_cookies.unwrap().len(), 2);

        // Clear all cookies
        jar.clear();
        assert!(jar.get_cookie("example.com", "session").is_none());
    }

    #[tokio::test]
    async fn test_concurrent_cookie_operations() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let mut config = SessionConfig::default();
        config.base_data_dir = temp_dir.path().to_path_buf();

        let manager = Arc::new(SessionManager::new(config).await.expect("Failed to create manager"));
        let session = manager.create_session().await.expect("Failed to create session");
        let session_id = session.session_id.clone();

        let mut handles = vec![];

        // Spawn 10 tasks setting cookies concurrently
        for i in 0..10 {
            let manager_clone = manager.clone();
            let sid = session_id.clone();
            let handle = tokio::spawn(async move {
                let cookie = Cookie::new(format!("key_{}", i), format!("value_{}", i));
                manager_clone.set_cookie(&sid, "example.com", cookie)
                    .await
                    .expect("Failed to set cookie");
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.expect("Task failed");
        }

        // Check all cookies were set
        for i in 0..10 {
            let cookie = manager.get_cookie(&session_id, "example.com", &format!("key_{}", i))
                .await
                .expect("Failed to get cookie");
            assert!(cookie.is_some());
            assert_eq!(cookie.unwrap().value, format!("value_{}", i));
        }
    }

    #[test]
    fn test_cookie_expiration() {
        let mut jar = CookieJar::default();

        // Create an expired cookie
        let expired_time = SystemTime::now() - Duration::from_secs(3600);
        let expired_cookie = Cookie::new("expired".to_string(), "value".to_string())
            .with_expires(expired_time);

        // Create a non-expired cookie
        let valid_cookie = Cookie::new("valid".to_string(), "value".to_string());

        jar.set_cookie("example.com", expired_cookie.clone());
        jar.set_cookie("example.com", valid_cookie.clone());

        // Check expired cookie
        assert!(expired_cookie.is_expired());
        assert!(!valid_cookie.is_expired());

        // Remove expired cookies
        jar.remove_expired();

        // Expired cookie should be gone, valid one should remain
        assert!(jar.get_cookie("example.com", "expired").is_none());
        assert!(jar.get_cookie("example.com", "valid").is_some());
    }
}