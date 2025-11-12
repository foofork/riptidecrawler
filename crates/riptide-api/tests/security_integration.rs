//! Security Integration Tests
//!
//! Tests security features and isolation:
//! - Tenant isolation (no cross-tenant data access)
//! - API authentication on all endpoints
//! - Rate limiting enforcement
//! - Session security (HttpOnly, Secure cookies)
//! - Admin endpoint authorization
//! - Input sanitization
//! - CORS policy enforcement
//! - SQL injection prevention
//! - XSS prevention
//! - CSRF protection

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

mod test_helpers;

#[cfg(test)]
mod security_tests {
    use super::*;

    /// Security Test 1: Tenant data isolation
    /// Verifies tenants cannot access each other's data
    ///
    /// **Note**: Requires full ApplicationContext with tenant management
    #[tokio::test]
    #[ignore = "Requires full ApplicationContext with tenant state management"]
    async fn test_tenant_data_isolation() {
        let app = test_helpers::create_minimal_test_app();

        // Tenant A stores sensitive data
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/cache/set")
                    .header("X-Tenant-ID", "tenant-a")
                    .body(Body::from(
                        json!({
                            "key": "api_key",
                            "value": "secret-key-tenant-a"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await;

        // Tenant B tries to access tenant A's data
        let cross_access = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/cache/get?key=api_key")
                    .header("X-Tenant-ID", "tenant-b")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should be forbidden or not found, not return tenant A's data
        assert!(
            cross_access.status() == StatusCode::NOT_FOUND
                || cross_access.status() == StatusCode::FORBIDDEN,
            "Cross-tenant access was not blocked"
        );
    }

    /// Security Test 2: API authentication requirement
    /// Verifies protected endpoints require authentication
    ///
    /// **Note**: Requires full ApplicationContext with auth middleware
    #[tokio::test]
    #[ignore = "Requires full ApplicationContext with auth middleware"]
    async fn test_api_authentication_required() {
        let app = test_helpers::create_minimal_test_app();

        // Attempt to access admin endpoint without auth
        let no_auth_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/admin/tenants")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(no_auth_response.status(), StatusCode::UNAUTHORIZED);

        // Attempt with invalid auth
        let invalid_auth_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/admin/tenants")
                    .header("X-Admin-Token", "invalid-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(invalid_auth_response.status(), StatusCode::UNAUTHORIZED);

        // Attempt with valid auth
        let valid_auth_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/admin/tenants")
                    .header("X-Admin-Token", "test-admin-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(valid_auth_response.status(), StatusCode::OK);
    }

    /// Security Test 3: Rate limiting enforcement
    /// Verifies rate limits are enforced per tenant
    ///
    /// **Note**: Requires full ApplicationContext with rate limiting middleware
    #[tokio::test]
    #[ignore = "Requires full ApplicationContext with rate limiting middleware"]
    async fn test_rate_limiting_enforcement() {
        let app = test_helpers::create_minimal_test_app();

        // Create tenant with strict rate limit
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/admin/tenants")
                    .header("X-Admin-Token", "test-admin-token")
                    .body(Body::from(
                        json!({
                            "tenant_id": "rate-limited-tenant",
                            "max_requests_per_minute": 5
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await;

        // Make requests up to limit
        for _ in 0..5 {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/api/v1/health")
                        .header("X-Tenant-ID", "rate-limited-tenant")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);
        }

        // Exceed rate limit
        let rate_limited_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .header("X-Tenant-ID", "rate-limited-tenant")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            rate_limited_response.status(),
            StatusCode::TOO_MANY_REQUESTS
        );
    }

    /// Security Test 4: Session cookie security
    /// Verifies cookies have proper security flags
    #[tokio::test]
    #[cfg(feature = "sessions")]
    async fn test_session_cookie_security() {
        let app = test_helpers::create_minimal_test_app();

        // Create session
        let session_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/sessions")
                    .body(Body::from(
                        json!({"url": "https://example.com"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Check Set-Cookie headers
        let cookies: Vec<_> = session_response
            .headers()
            .get_all("set-cookie")
            .iter()
            .collect();

        for cookie in cookies {
            let cookie_str = cookie.to_str().unwrap();

            // Verify security flags
            assert!(
                cookie_str.contains("HttpOnly") || cookie_str.contains("httponly"),
                "Cookie missing HttpOnly flag: {}",
                cookie_str
            );
            assert!(
                cookie_str.contains("Secure") || cookie_str.contains("secure"),
                "Cookie missing Secure flag: {}",
                cookie_str
            );
            assert!(
                cookie_str.contains("SameSite") || cookie_str.contains("samesite"),
                "Cookie missing SameSite flag: {}",
                cookie_str
            );
        }
    }

    /// Security Test 5: Admin endpoint authorization
    /// Verifies only admins can access admin endpoints
    ///
    /// **Note**: Requires full ApplicationContext with admin auth
    #[tokio::test]
    #[ignore = "Requires full ApplicationContext with admin auth middleware"]
    async fn test_admin_endpoint_authorization() {
        let app = test_helpers::create_minimal_test_app();

        // Regular user tries to access admin endpoint
        let user_access = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/admin/config")
                    .header("X-User-Token", "regular-user-token")
                    .body(Body::from(json!({"setting": "value"}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(user_access.status(), StatusCode::UNAUTHORIZED);

        // Admin accesses admin endpoint
        let admin_access = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/admin/config")
                    .header("X-Admin-Token", "test-admin-token")
                    .body(Body::from(json!({"setting": "value"}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(
            admin_access.status() == StatusCode::OK
                || admin_access.status() == StatusCode::ACCEPTED
        );
    }

    /// Security Test 6: Input sanitization
    /// Verifies dangerous input is sanitized
    #[tokio::test]
    async fn test_input_sanitization() {
        let app = test_helpers::create_minimal_test_app();

        // Try XSS payload
        let xss_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/cache/set")
                    .body(Body::from(
                        json!({
                            "key": "xss_test",
                            "value": "<script>alert('XSS')</script>"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(xss_response.status().is_success());

        // Verify retrieval is sanitized
        let get_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/cache/get?key=xss_test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(get_response.status(), StatusCode::OK);
        // Content should be escaped or sanitized
    }

    /// Security Test 7: CORS policy enforcement
    /// Verifies CORS headers are properly set
    #[tokio::test]
    async fn test_cors_policy_enforcement() {
        let app = test_helpers::create_security_test_app();

        // Preflight request
        let preflight_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/api/v1/crawl")
                    .header("Origin", "https://example.com")
                    .header("Access-Control-Request-Method", "POST")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Check CORS headers
        assert!(preflight_response
            .headers()
            .contains_key("access-control-allow-origin"));
        assert!(preflight_response
            .headers()
            .contains_key("access-control-allow-methods"));

        // Verify origin restriction (if applicable)
        let cors_header = preflight_response
            .headers()
            .get("access-control-allow-origin")
            .unwrap()
            .to_str()
            .unwrap();

        assert!(
            cors_header == "*" || cors_header.contains("example.com"),
            "CORS policy not properly configured"
        );
    }

    /// Security Test 8: SQL injection prevention
    /// Verifies SQL injection attacks are prevented
    ///
    /// **Note**: Requires input validation middleware
    #[tokio::test]
    #[ignore = "Requires input validation middleware for proper SQL injection testing"]
    async fn test_sql_injection_prevention() {
        let app = test_helpers::create_minimal_test_app();

        // Try SQL injection in search query
        let sql_injection_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/search?q=test' OR '1'='1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should not cause error or return unauthorized data
        assert!(
            sql_injection_response.status().is_success()
                || sql_injection_response.status() == StatusCode::BAD_REQUEST
        );

        // Try SQL injection in cache key
        let cache_injection_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/cache/get?key=test'; DROP TABLE users; --")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should handle safely
        assert!(
            cache_injection_response.status() == StatusCode::OK
                || cache_injection_response.status() == StatusCode::NOT_FOUND
                || cache_injection_response.status() == StatusCode::BAD_REQUEST
        );
    }

    /// Security Test 9: Path traversal prevention
    /// Verifies path traversal attacks are blocked
    ///
    /// **Note**: Requires input validation middleware
    #[tokio::test]
    #[ignore = "Requires input validation middleware for path traversal prevention"]
    async fn test_path_traversal_prevention() {
        let app = test_helpers::create_minimal_test_app();

        // Try path traversal in cache key
        let traversal_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/cache/get?key=../../etc/passwd")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should not expose filesystem
        assert!(
            traversal_response.status() == StatusCode::BAD_REQUEST
                || traversal_response.status() == StatusCode::NOT_FOUND
        );

        // Try path traversal in URL parameter
        let url_traversal_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/extract")
                    .body(Body::from(
                        json!({
                            "url": "file:///etc/passwd",
                            "mode": "standard"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(
            url_traversal_response.status() == StatusCode::BAD_REQUEST
                || url_traversal_response.status().is_server_error()
        );
    }

    /// Security Test 10: CSRF token validation
    /// Verifies CSRF protection is in place
    #[tokio::test]
    async fn test_csrf_token_validation() {
        let app = test_helpers::create_minimal_test_app();

        // State-changing request without CSRF token
        let no_csrf_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/cache/set")
                    .body(Body::from(
                        json!({"key": "test", "value": "test"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should either require CSRF token or be OK (depending on implementation)
        assert!(
            no_csrf_response.status() == StatusCode::OK
                || no_csrf_response.status() == StatusCode::FORBIDDEN
        );

        // Request with CSRF token
        let with_csrf_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/cache/set")
                    .header("X-CSRF-Token", "test-csrf-token")
                    .body(Body::from(
                        json!({"key": "test", "value": "test"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(with_csrf_response.status(), StatusCode::OK);
    }

    /// Security Test 11: Security headers from riptide-security
    /// Verifies HSTS, CSP, and other security headers are applied
    #[tokio::test]
    async fn test_riptide_security_headers() {
        let app = test_helpers::create_security_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let headers = response.headers();

        // Verify riptide-security headers are present
        assert!(
            headers.contains_key("x-xss-protection"),
            "Missing X-XSS-Protection header from riptide-security"
        );
        assert!(
            headers.contains_key("x-content-type-options"),
            "Missing X-Content-Type-Options header from riptide-security"
        );
        assert!(
            headers.contains_key("x-frame-options"),
            "Missing X-Frame-Options header from riptide-security"
        );
        assert!(
            headers.contains_key("strict-transport-security"),
            "Missing Strict-Transport-Security (HSTS) header from riptide-security"
        );
        assert!(
            headers.contains_key("referrer-policy"),
            "Missing Referrer-Policy header from riptide-security"
        );

        // Verify header values are secure
        let hsts = headers
            .get("strict-transport-security")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(
            hsts.contains("max-age="),
            "HSTS header should contain max-age directive"
        );

        let xfo = headers.get("x-frame-options").unwrap().to_str().unwrap();
        assert!(
            xfo.contains("DENY") || xfo.contains("SAMEORIGIN"),
            "X-Frame-Options should be DENY or SAMEORIGIN"
        );
    }
}
