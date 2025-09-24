#[cfg(test)]
mod tests {
    use riptide_api::session::{Session, SessionManager, SessionStore, Cookie};
    use std::time::Duration;
    use uuid::Uuid;

    #[test]
    fn test_session_creation() {
        let session = Session::new("user123");

        assert!(Uuid::parse_str(&session.id).is_ok());
        assert_eq!(session.user_id, Some("user123".to_string()));
        assert!(session.created_at > 0);
        assert!(session.data.is_empty());
    }

    #[test]
    fn test_session_data_management() {
        let mut session = Session::new("user123");

        // Add data
        session.set("theme", "dark");
        session.set("language", "en");

        assert_eq!(session.get("theme"), Some(&"dark".to_string()));
        assert_eq!(session.get("language"), Some(&"en".to_string()));

        // Update data
        session.set("theme", "light");
        assert_eq!(session.get("theme"), Some(&"light".to_string()));

        // Remove data
        session.remove("language");
        assert_eq!(session.get("language"), None);
    }

    #[test]
    fn test_session_expiration() {
        let mut session = Session::with_ttl("user123", Duration::from_secs(3600));

        assert!(!session.is_expired());

        // Manually expire
        session.expire();
        assert!(session.is_expired());
    }

    #[test]
    fn test_cookie_creation() {
        let cookie = Cookie::new("session_id", "abc123")
            .domain("example.com")
            .path("/")
            .secure(true)
            .http_only(true)
            .same_site("Strict")
            .max_age(3600);

        assert_eq!(cookie.name, "session_id");
        assert_eq!(cookie.value, "abc123");
        assert_eq!(cookie.domain, Some("example.com".to_string()));
        assert_eq!(cookie.path, Some("/".to_string()));
        assert!(cookie.secure);
        assert!(cookie.http_only);
        assert_eq!(cookie.same_site, Some("Strict".to_string()));
        assert_eq!(cookie.max_age, Some(3600));
    }

    #[tokio::test]
    async fn test_session_manager() {
        let manager = SessionManager::new();

        // Create session
        let session_id = manager.create("user123").await;
        assert!(session_id.len() > 0);

        // Get session
        let session = manager.get(&session_id).await;
        assert!(session.is_some());
        assert_eq!(session.unwrap().user_id, Some("user123".to_string()));

        // Update session
        manager.update(&session_id, |session| {
            session.set("last_access", "2024-01-01");
        }).await;

        let updated = manager.get(&session_id).await.unwrap();
        assert_eq!(updated.get("last_access"), Some(&"2024-01-01".to_string()));

        // Delete session
        manager.delete(&session_id).await;
        assert!(manager.get(&session_id).await.is_none());
    }

    #[tokio::test]
    async fn test_session_store_memory() {
        let store = SessionStore::memory();

        let session = Session::new("user123");
        let id = session.id.clone();

        // Save
        store.save(session.clone()).await.unwrap();

        // Load
        let loaded = store.load(&id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().user_id, Some("user123".to_string()));

        // Delete
        store.delete(&id).await.unwrap();
        assert!(store.load(&id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_session_cleanup() {
        let manager = SessionManager::new();

        // Create sessions with different TTLs
        let session1 = manager.create_with_ttl("user1", Duration::from_millis(100)).await;
        let session2 = manager.create_with_ttl("user2", Duration::from_secs(3600)).await;

        // Both should exist initially
        assert!(manager.get(&session1).await.is_some());
        assert!(manager.get(&session2).await.is_some());

        // Wait for first to expire
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Cleanup expired
        let cleaned = manager.cleanup_expired().await;
        assert_eq!(cleaned, 1);

        // Check results
        assert!(manager.get(&session1).await.is_none());
        assert!(manager.get(&session2).await.is_some());
    }

    #[test]
    fn test_session_regenerate_id() {
        let mut session = Session::new("user123");
        let original_id = session.id.clone();

        session.regenerate_id();

        assert_ne!(session.id, original_id);
        assert_eq!(session.user_id, Some("user123".to_string()));
    }

    #[test]
    fn test_session_flash_messages() {
        let mut session = Session::new("user123");

        // Add flash message
        session.add_flash("success", "Operation completed!");
        session.add_flash("error", "Something went wrong");

        // Get flash messages (should remove them)
        let success_msgs = session.get_flash("success");
        assert_eq!(success_msgs.len(), 1);
        assert_eq!(success_msgs[0], "Operation completed!");

        // Second get should return empty
        let success_msgs2 = session.get_flash("success");
        assert_eq!(success_msgs2.len(), 0);

        // Error message should still be there
        let error_msgs = session.get_flash("error");
        assert_eq!(error_msgs.len(), 1);
    }

    #[tokio::test]
    async fn test_concurrent_session_access() {
        let manager = Arc::new(SessionManager::new());
        let session_id = manager.create("user123").await;

        let mut handles = vec![];

        // Spawn 10 tasks updating the same session
        for i in 0..10 {
            let manager_clone = manager.clone();
            let sid = session_id.clone();
            let handle = tokio::spawn(async move {
                manager_clone.update(&sid, |session| {
                    session.set(&format!("key_{}", i), &format!("value_{}", i));
                }).await;
            });
            handles.push(handle);
        }

        futures::future::join_all(handles).await;

        // Check all updates were applied
        let final_session = manager.get(&session_id).await.unwrap();
        for i in 0..10 {
            assert_eq!(
                final_session.get(&format!("key_{}", i)),
                Some(&format!("value_{}", i))
            );
        }
    }

    #[test]
    fn test_session_csrf_token() {
        let mut session = Session::new("user123");

        // Generate CSRF token
        let token = session.generate_csrf_token();
        assert!(token.len() >= 32);

        // Validate correct token
        assert!(session.validate_csrf_token(&token));

        // Validate incorrect token
        assert!(!session.validate_csrf_token("wrong_token"));
    }
}