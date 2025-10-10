//! Security tests for session middleware
//!
//! Tests cover:
//! - Session expiration validation
//! - Session-based rate limiting
//! - Cookie security attributes
//! - Concurrent session handling
//! - Suspicious activity detection

#[cfg(test)]
mod security_tests {
    use axum::{
        body::Body,
        extract::Request,
        http::{header, StatusCode},
        Router,
    };
    use riptide_api::sessions::{
        middleware::{SecurityConfig, SessionLayer},
        Cookie, Session, SessionConfig, SessionManager,
    };
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::TempDir;
    use tower::ServiceExt;

    /// Helper to create test session manager
    async fn create_test_manager() -> (Arc<SessionManager>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = SessionConfig {
            base_data_dir: temp_dir.path().to_path_buf(),
            default_ttl: Duration::from_secs(3600),
            ..Default::default()
        };

        let manager = Arc::new(
            SessionManager::new(config)
                .await
                .expect("Failed to create manager"),
        );
        (manager, temp_dir)
    }

    /// Helper to create test app with session middleware
    async fn create_test_app(
        manager: Arc<SessionManager>,
        security_config: SecurityConfig,
    ) -> Router {
        use axum::routing::get;

        Router::new()
            .route("/test", get(|| async { "OK" }))
            .layer(SessionLayer::with_security_config(manager, security_config))
    }

    #[tokio::test]
    async fn test_session_expiration_validation() {
        // Test expiration check directly on Session object
        let config = SessionConfig {
            default_ttl: Duration::from_millis(50),
            ..Default::default()
        };

        let session = Session::new("test_expired".to_string(), &config);
        assert!(
            !session.is_expired(),
            "Session should not be expired initially"
        );

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert!(session.is_expired(), "Session should be expired after TTL");
    }

    #[tokio::test]
    async fn test_session_rate_limiting() {
        let (manager, _temp_dir) = create_test_manager().await;

        // Create session
        let session = manager
            .create_session()
            .await
            .expect("Failed to create session");
        let session_id = session.session_id.clone();

        // Create app with strict rate limiting (5 requests per 10 seconds)
        let security_config = SecurityConfig {
            validate_expiration: false,
            enable_rate_limiting: true,
            max_requests_per_window: 5,
            rate_limit_window: Duration::from_secs(10),
            ..Default::default()
        };

        let app = create_test_app(manager.clone(), security_config).await;

        // Make 5 requests - all should succeed
        for i in 0..5 {
            let request = Request::builder()
                .uri("/test")
                .header(header::COOKIE, format!("riptide_session_id={}", session_id))
                .body(Body::empty())
                .unwrap();

            let app_clone = app.clone();
            let response = app_clone.oneshot(request).await.unwrap();

            assert_eq!(
                response.status(),
                StatusCode::OK,
                "Request {} should succeed",
                i + 1
            );
        }

        // 6th request should be rate limited
        let request = Request::builder()
            .uri("/test")
            .header(header::COOKIE, format!("riptide_session_id={}", session_id))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn test_secure_cookie_attributes() {
        let (manager, _temp_dir) = create_test_manager().await;

        // Create app with secure cookies enabled
        let security_config = SecurityConfig {
            secure_cookies: true,
            same_site: "Strict",
            ..Default::default()
        };

        let app = create_test_app(manager.clone(), security_config).await;

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Check Set-Cookie header has security attributes
        if let Some(cookie_header) = response.headers().get(header::SET_COOKIE) {
            let cookie_str = cookie_header.to_str().unwrap();
            assert!(
                cookie_str.contains("HttpOnly"),
                "Cookie should have HttpOnly"
            );
            assert!(
                cookie_str.contains("Secure"),
                "Cookie should have Secure flag"
            );
            assert!(
                cookie_str.contains("SameSite=Strict"),
                "Cookie should have SameSite=Strict"
            );
        } else {
            panic!("No Set-Cookie header found");
        }
    }

    #[tokio::test]
    async fn test_concurrent_session_access() {
        let (manager, _temp_dir) = create_test_manager().await;

        // Create session
        let session = manager
            .create_session()
            .await
            .expect("Failed to create session");
        let session_id = session.session_id.clone();

        let security_config = SecurityConfig {
            enable_rate_limiting: false,
            ..Default::default()
        };

        // Spawn 20 concurrent requests with same session
        let mut handles = vec![];
        for _ in 0..20 {
            let app = create_test_app(manager.clone(), security_config.clone()).await;
            let sid = session_id.clone();
            let handle = tokio::spawn(async move {
                let request = Request::builder()
                    .uri("/test")
                    .header(header::COOKIE, format!("riptide_session_id={}", sid))
                    .body(Body::empty())
                    .unwrap();

                app.oneshot(request).await
            });
            handles.push(handle);
        }

        // All requests should succeed
        for handle in handles {
            let response = handle.await.expect("Task failed").expect("Request failed");
            assert_eq!(response.status(), StatusCode::OK);
        }
    }

    #[tokio::test]
    async fn test_rate_limit_window_expiry() {
        let (manager, _temp_dir) = create_test_manager().await;

        // Create session
        let session = manager
            .create_session()
            .await
            .expect("Failed to create session");
        let session_id = session.session_id.clone();

        // Create app with 2 requests per 1 second window
        let security_config = SecurityConfig {
            enable_rate_limiting: true,
            max_requests_per_window: 2,
            rate_limit_window: Duration::from_secs(1),
            ..Default::default()
        };

        let app = create_test_app(manager.clone(), security_config).await;

        // Make 2 requests - should succeed
        for _ in 0..2 {
            let request = Request::builder()
                .uri("/test")
                .header(header::COOKIE, format!("riptide_session_id={}", session_id))
                .body(Body::empty())
                .unwrap();

            let app_clone = app.clone();
            let response = app_clone.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }

        // 3rd request should fail
        let request = Request::builder()
            .uri("/test")
            .header(header::COOKIE, format!("riptide_session_id={}", session_id))
            .body(Body::empty())
            .unwrap();

        let app_clone = app.clone();
        let response = app_clone.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

        // Wait for window to expire
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // Should be able to make requests again
        let request = Request::builder()
            .uri("/test")
            .header(header::COOKIE, format!("riptide_session_id={}", session_id))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_session_cookie_security() {
        let (manager, _temp_dir) = create_test_manager().await;

        // Test cookie with security attributes
        let cookie = Cookie::new("secure_cookie".to_string(), "value".to_string())
            .with_domain("example.com".to_string())
            .secure()
            .http_only();

        let session = manager
            .create_session()
            .await
            .expect("Failed to create session");

        manager
            .set_cookie(&session.session_id, "example.com", cookie.clone())
            .await
            .expect("Failed to set cookie");

        // Retrieve and verify
        let retrieved = manager
            .get_cookie(&session.session_id, "example.com", "secure_cookie")
            .await
            .expect("Failed to get cookie")
            .expect("Cookie not found");

        assert!(retrieved.secure, "Cookie should be marked secure");
        assert!(retrieved.http_only, "Cookie should be HttpOnly");
    }

    #[tokio::test]
    async fn test_session_without_cookie() {
        let (manager, _temp_dir) = create_test_manager().await;

        let security_config = SecurityConfig::default();
        let app = create_test_app(manager.clone(), security_config).await;

        // Request without session cookie - should create new session
        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Should have Set-Cookie header
        assert!(
            response.headers().get(header::SET_COOKIE).is_some(),
            "Should set session cookie"
        );
    }

    #[tokio::test]
    async fn test_different_sessions_independent_rate_limits() {
        let (manager, _temp_dir) = create_test_manager().await;

        // Create two different sessions
        let session1 = manager
            .create_session()
            .await
            .expect("Failed to create session1");
        let session2 = manager
            .create_session()
            .await
            .expect("Failed to create session2");

        // Strict rate limit
        let security_config = SecurityConfig {
            enable_rate_limiting: true,
            max_requests_per_window: 2,
            rate_limit_window: Duration::from_secs(10),
            ..Default::default()
        };

        let app = create_test_app(manager.clone(), security_config).await;

        // Exhaust session1 rate limit
        for _ in 0..2 {
            let request = Request::builder()
                .uri("/test")
                .header(
                    header::COOKIE,
                    format!("riptide_session_id={}", session1.session_id),
                )
                .body(Body::empty())
                .unwrap();

            let app_clone = app.clone();
            let response = app_clone.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }

        // Session1 should be rate limited
        let request = Request::builder()
            .uri("/test")
            .header(
                header::COOKIE,
                format!("riptide_session_id={}", session1.session_id),
            )
            .body(Body::empty())
            .unwrap();

        let app_clone = app.clone();
        let response = app_clone.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

        // Session2 should still work (independent rate limit)
        let request = Request::builder()
            .uri("/test")
            .header(
                header::COOKIE,
                format!("riptide_session_id={}", session2.session_id),
            )
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_session_context_methods() {
        use riptide_api::sessions::middleware::SessionContext;

        let (manager, _temp_dir) = create_test_manager().await;

        let session = manager
            .create_session()
            .await
            .expect("Failed to create session");

        let ctx = SessionContext {
            session: session.clone(),
            manager: manager.clone(),
        };

        // Test session context methods
        assert_eq!(ctx.session_id(), &session.session_id);
        assert_eq!(ctx.session().session_id, session.session_id);
        assert!(ctx
            .manager()
            .list_sessions()
            .await
            .contains(&session.session_id));
        assert_eq!(*ctx.user_data_dir(), session.user_data_dir);
        assert!(!ctx.is_expired());

        // Test cookie operations through context
        let cookie = Cookie::new("test_cookie".to_string(), "value".to_string());
        ctx.set_cookie("example.com", cookie.clone())
            .await
            .expect("Failed to set cookie");

        let retrieved = ctx
            .get_cookie("example.com", "test_cookie")
            .await
            .expect("Failed to get cookie")
            .expect("Cookie not found");

        assert_eq!(retrieved.name, "test_cookie");
        assert_eq!(retrieved.value, "value");

        // Test extend session
        ctx.extend_session(Duration::from_secs(7200))
            .await
            .expect("Failed to extend session");
    }
}
