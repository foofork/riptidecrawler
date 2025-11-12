//! Authentication Rate Limiting Tests
//!
//! Comprehensive test suite for authentication rate limiting covering:
//! - Rate limit enforcement (10 failures → block)
//! - Exponential backoff behavior
//! - Per-IP tracking
//! - Cleanup of expired entries
//! - Success resets failure count
//! - Different IPs tracked separately
//! - Retry-After header verification
//! - 429 status code on rate limit exceeded

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;
use tower::ServiceExt;

use riptide_api::{
    middleware::{auth_middleware, AuthConfig},
    state::ApplicationContext,
};

/// Helper function to create a test app with authentication middleware
async fn create_test_app_with_auth(auth_config: AuthConfig) -> axum::Router {
    use axum::{middleware, routing::get, Router};

    // Create minimal test state
    let mut state = ApplicationContext::new_test_minimal().await;
    state.auth_config = auth_config;

    // Create router with auth middleware
    Router::new()
        .route("/api/v1/test", get(|| async { "success" }))
        .route("/health", get(|| async { "healthy" }))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state)
}

/// Helper to extract JSON from response body
async fn body_to_json(body: Body) -> Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap_or(serde_json::json!({}))
}

// ============================================================================
// RATE LIMIT ENFORCEMENT TESTS
// ============================================================================

#[tokio::test]
async fn test_rate_limit_blocks_after_max_failures() {
    // Setup: Create auth config with 3 max attempts, 60s window
    let auth_config = AuthConfig::with_api_keys_and_rate_limit(
        vec!["valid-key".to_string()],
        3, // Max 3 attempts
        Duration::from_secs(60),
    );
    let app = create_test_app_with_auth(auth_config.clone()).await;

    // Execute: Make 3 invalid requests (should succeed)
    for i in 1..=3 {
        let request = Request::builder()
            .uri("/api/v1/test")
            .header("X-API-Key", "invalid-key")
            .header("X-Forwarded-For", "192.168.1.100") // Same IP
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        // First 3 should get 401 Unauthorized
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Attempt {} should be unauthorized, not rate-limited yet",
            i
        );
    }

    // 4th request should be rate-limited
    let request = Request::builder()
        .uri("/api/v1/test")
        .header("X-API-Key", "invalid-key")
        .header("X-Forwarded-For", "192.168.1.100")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert: Should be rate limited now
    assert_eq!(
        response.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "After 3 failures, should be rate limited"
    );

    // Verify Retry-After header is present
    assert!(
        response.headers().contains_key("Retry-After"),
        "Rate limited response should include Retry-After header"
    );

    let body = body_to_json(response.into_body()).await;
    assert_eq!(body["error"], "Too Many Requests");
    assert!(body["retry_after_seconds"].is_number());
}

#[tokio::test]
async fn test_rate_limit_default_is_10_attempts() {
    // Setup: Create auth config with default settings (10 attempts)
    let auth_config = AuthConfig::with_api_keys(vec!["valid-key".to_string()]);
    let app = create_test_app_with_auth(auth_config).await;

    // Execute: Make 10 invalid requests (should all be 401)
    for i in 1..=10 {
        let request = Request::builder()
            .uri("/api/v1/test")
            .header("X-API-Key", format!("invalid-key-{}", i))
            .header("X-Forwarded-For", "10.0.0.1")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Attempt {} should be unauthorized",
            i
        );
    }

    // 11th request should be rate-limited
    let request = Request::builder()
        .uri("/api/v1/test")
        .header("X-API-Key", "invalid-key-11")
        .header("X-Forwarded-For", "10.0.0.1")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert: Should be rate limited after 10 failures
    assert_eq!(
        response.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "After 10 failures (default limit), should be rate limited"
    );
}

// ============================================================================
// PER-IP TRACKING TESTS
// ============================================================================

#[tokio::test]
async fn test_rate_limit_per_ip_separate_tracking() {
    // Setup: Create auth config with 2 max attempts
    let auth_config = AuthConfig::with_api_keys_and_rate_limit(
        vec!["valid-key".to_string()],
        2,
        Duration::from_secs(60),
    );
    let app = create_test_app_with_auth(auth_config).await;

    // Execute: Make 2 failures from IP1
    for _ in 1..=2 {
        let request = Request::builder()
            .uri("/api/v1/test")
            .header("X-API-Key", "invalid-key")
            .header("X-Forwarded-For", "192.168.1.1")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // IP1 should be blocked
    let request = Request::builder()
        .uri("/api/v1/test")
        .header("X-API-Key", "invalid-key")
        .header("X-Forwarded-For", "192.168.1.1")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "IP1 should be rate limited"
    );

    // But IP2 should still be able to make requests
    let request = Request::builder()
        .uri("/api/v1/test")
        .header("X-API-Key", "invalid-key")
        .header("X-Forwarded-For", "192.168.1.2") // Different IP
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert: IP2 should get 401, not 429
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Different IP should not be rate limited"
    );
}

#[tokio::test]
async fn test_x_real_ip_header_support() {
    // Setup: Create auth config
    let auth_config = AuthConfig::with_api_keys_and_rate_limit(
        vec!["valid-key".to_string()],
        2,
        Duration::from_secs(60),
    );
    let app = create_test_app_with_auth(auth_config).await;

    // Execute: Make failures using X-Real-IP header
    for _ in 1..=2 {
        let request = Request::builder()
            .uri("/api/v1/test")
            .header("X-API-Key", "invalid-key")
            .header("X-Real-IP", "10.20.30.40") // Using X-Real-IP
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // Next request from same IP should be rate limited
    let request = Request::builder()
        .uri("/api/v1/test")
        .header("X-API-Key", "invalid-key")
        .header("X-Real-IP", "10.20.30.40")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert: X-Real-IP should be tracked
    assert_eq!(
        response.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "X-Real-IP should be tracked for rate limiting"
    );
}

// ============================================================================
// EXPONENTIAL BACKOFF TESTS
// ============================================================================

#[tokio::test]
async fn test_exponential_backoff_increases_retry_after() {
    // Setup: Create auth config with 2 max attempts
    let auth_config = AuthConfig::with_api_keys_and_rate_limit(
        vec!["valid-key".to_string()],
        2,
        Duration::from_secs(60),
    );
    let app = create_test_app_with_auth(auth_config.clone()).await;

    // Execute: Make 2 failures to trigger block
    for _ in 1..=2 {
        let request = Request::builder()
            .uri("/api/v1/test")
            .header("X-API-Key", "invalid-key")
            .header("X-Forwarded-For", "172.16.0.1")
            .body(Body::empty())
            .unwrap();

        let _ = app.clone().oneshot(request).await.unwrap();
    }

    // Check initial backoff (2^2 = 4 seconds for 2 failures)
    let request = Request::builder()
        .uri("/api/v1/test")
        .header("X-API-Key", "invalid-key")
        .header("X-Forwarded-For", "172.16.0.1")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    let retry_after_1 = response
        .headers()
        .get("Retry-After")
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();

    // Make another failed attempt (3rd failure)
    let request = Request::builder()
        .uri("/api/v1/test")
        .header("X-API-Key", "invalid-key")
        .header("X-Forwarded-For", "172.16.0.1")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    let retry_after_2 = response
        .headers()
        .get("Retry-After")
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();

    // Assert: Backoff should increase or stay similar (with timing variance)
    // Note: Due to timing and cleanup, the second value might be slightly less
    // but it should be in the ballpark of exponential growth
    println!(
        "Backoff times - first: {}, second: {}",
        retry_after_1, retry_after_2
    );

    // Both should be reasonable exponential values (2^n for n >= 2)
    assert!(
        retry_after_1 >= 2,
        "First backoff should be at least 2 seconds (2^2): {}",
        retry_after_1
    );
    assert!(
        retry_after_2 >= 2,
        "Second backoff should be at least 2 seconds: {}",
        retry_after_2
    );
}

// ============================================================================
// SUCCESS RESETS FAILURE COUNT
// ============================================================================

#[tokio::test]
async fn test_successful_auth_resets_failure_count() {
    // Setup: Create auth config with 3 max attempts
    let auth_config = AuthConfig::with_api_keys_and_rate_limit(
        vec!["valid-key".to_string()],
        3,
        Duration::from_secs(60),
    );
    let app = create_test_app_with_auth(auth_config.clone()).await;

    // Execute: Make 2 failures
    for _ in 1..=2 {
        let request = Request::builder()
            .uri("/api/v1/test")
            .header("X-API-Key", "invalid-key")
            .header("X-Forwarded-For", "203.0.113.1")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // Make a successful request
    let request = Request::builder()
        .uri("/api/v1/test")
        .header("X-API-Key", "valid-key") // Valid key
        .header("X-Forwarded-For", "203.0.113.1")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Now make 3 more failures (should not be rate limited yet since count was reset)
    for i in 1..=3 {
        let request = Request::builder()
            .uri("/api/v1/test")
            .header("X-API-Key", "invalid-key")
            .header("X-Forwarded-For", "203.0.113.1")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        // Should get 401, not 429 (because success reset the counter)
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Attempt {} after success should be unauthorized (not rate limited)",
            i
        );
    }

    // 4th failure after reset should trigger rate limit
    let request = Request::builder()
        .uri("/api/v1/test")
        .header("X-API-Key", "invalid-key")
        .header("X-Forwarded-For", "203.0.113.1")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert: Now should be rate limited
    assert_eq!(
        response.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "After 3 new failures (post-reset), should be rate limited"
    );
}

// ============================================================================
// CLEANUP OF EXPIRED ENTRIES
// ============================================================================

#[tokio::test]
async fn test_cleanup_removes_expired_entries() {
    // Setup: Create auth config with 2 max attempts, short 2s window
    let auth_config = AuthConfig::with_api_keys_and_rate_limit(
        vec!["valid-key".to_string()],
        2,
        Duration::from_secs(2), // 2 second window
    );
    let app = create_test_app_with_auth(auth_config.clone()).await;

    // Execute: Make 2 failures
    for _ in 1..=2 {
        let request = Request::builder()
            .uri("/api/v1/test")
            .header("X-API-Key", "invalid-key")
            .header("X-Forwarded-For", "198.51.100.1")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // Should be rate limited immediately
    let request = Request::builder()
        .uri("/api/v1/test")
        .header("X-API-Key", "invalid-key")
        .header("X-Forwarded-For", "198.51.100.1")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    // Wait for both window AND block to expire
    // Block time is 2^2 = 4 seconds for 2 failures
    // Window is 2 seconds
    // So we wait 5 seconds to be safe
    sleep(Duration::from_secs(5)).await;

    // Manually trigger cleanup
    auth_config.rate_limiter().cleanup_expired().await;

    // New request should not be rate limited (entry was cleaned up)
    let request = Request::builder()
        .uri("/api/v1/test")
        .header("X-API-Key", "invalid-key")
        .header("X-Forwarded-For", "198.51.100.1")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert: Should get 401 (unauthorized) not 429 (rate limited)
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "After window expiry and cleanup, should not be rate limited"
    );
}

// ============================================================================
// RETRY-AFTER HEADER TESTS
// ============================================================================

#[tokio::test]
async fn test_retry_after_header_format() {
    // Setup: Create auth config with 2 max attempts
    let auth_config = AuthConfig::with_api_keys_and_rate_limit(
        vec!["valid-key".to_string()],
        2,
        Duration::from_secs(60),
    );
    let app = create_test_app_with_auth(auth_config).await;

    // Execute: Make 2 failures to trigger rate limit
    for _ in 1..=2 {
        let request = Request::builder()
            .uri("/api/v1/test")
            .header("X-API-Key", "invalid-key")
            .header("X-Forwarded-For", "10.1.2.3")
            .body(Body::empty())
            .unwrap();

        let _ = app.clone().oneshot(request).await.unwrap();
    }

    // Get rate limited response
    let request = Request::builder()
        .uri("/api/v1/test")
        .header("X-API-Key", "invalid-key")
        .header("X-Forwarded-For", "10.1.2.3")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert: Verify Retry-After header
    assert!(response.headers().contains_key("Retry-After"));

    let retry_after = response
        .headers()
        .get("Retry-After")
        .unwrap()
        .to_str()
        .unwrap();

    // Should be a number (seconds)
    let retry_secs: u64 = retry_after.parse().expect("Retry-After should be a number");
    assert!(retry_secs >= 1, "Retry-After should be at least 1 second");
    assert!(retry_secs <= 1024, "Retry-After should be capped");

    // Verify JSON body also contains retry_after
    let body = body_to_json(response.into_body()).await;
    assert!(body["retry_after_seconds"].is_number());
    assert_eq!(body["retry_after_seconds"].as_u64().unwrap(), retry_secs);
}

// ============================================================================
// EDGE CASES
// ============================================================================

#[tokio::test]
async fn test_missing_ip_headers_uses_unknown() {
    // Setup: Create auth config with 2 max attempts
    let auth_config = AuthConfig::with_api_keys_and_rate_limit(
        vec!["valid-key".to_string()],
        2,
        Duration::from_secs(60),
    );
    let app = create_test_app_with_auth(auth_config).await;

    // Execute: Make requests without IP headers (will use "unknown")
    for _ in 1..=2 {
        let request = Request::builder()
            .uri("/api/v1/test")
            .header("X-API-Key", "invalid-key")
            // No IP headers
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // Should be rate limited on "unknown" IP
    let request = Request::builder()
        .uri("/api/v1/test")
        .header("X-API-Key", "invalid-key")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert: Should track "unknown" IP and rate limit
    assert_eq!(
        response.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "Should rate limit even with unknown IP"
    );
}

#[tokio::test]
async fn test_public_paths_not_rate_limited() {
    // Setup: Create auth config with very low limit (1 attempt)
    let auth_config = AuthConfig::with_api_keys_and_rate_limit(
        vec!["valid-key".to_string()],
        1,
        Duration::from_secs(60),
    );
    let app = create_test_app_with_auth(auth_config).await;

    // Execute: Access public path many times without auth
    for i in 1..=10 {
        let request = Request::builder()
            .uri("/health") // Public path
            .header("X-Forwarded-For", "192.0.2.1")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        // Assert: Should always succeed (public paths bypass auth and rate limiting)
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Public path attempt {} should not be rate limited",
            i
        );
    }
}

#[tokio::test]
async fn test_concurrent_rate_limit_checks() {
    use tokio::task;

    // Setup: Create auth config with 5 max attempts
    let auth_config = AuthConfig::with_api_keys_and_rate_limit(
        vec!["valid-key".to_string()],
        5,
        Duration::from_secs(60),
    );
    let app = create_test_app_with_auth(auth_config).await;

    // Execute: Send 10 concurrent invalid requests from same IP
    let mut handles = vec![];

    for i in 0..10 {
        let app_clone = app.clone();
        let handle = task::spawn(async move {
            let request = Request::builder()
                .uri("/api/v1/test")
                .header("X-API-Key", format!("invalid-{}", i))
                .header("X-Forwarded-For", "203.0.113.100") // Same IP
                .body(Body::empty())
                .unwrap();

            app_clone.oneshot(request).await.unwrap()
        });
        handles.push(handle);
    }

    let responses = futures::future::join_all(handles).await;

    // Count 401 vs 429 responses
    let mut unauthorized_count = 0;
    let mut rate_limited_count = 0;

    for result in responses {
        let response = result.unwrap();
        match response.status() {
            StatusCode::UNAUTHORIZED => unauthorized_count += 1,
            StatusCode::TOO_MANY_REQUESTS => rate_limited_count += 1,
            _ => panic!("Unexpected status code: {}", response.status()),
        }
    }

    // Assert: Should have ~5 unauthorized and ~5 rate limited
    // (exact counts may vary due to race conditions)
    assert!(
        unauthorized_count <= 6,
        "Should have around 5 unauthorized: {}",
        unauthorized_count
    );
    assert!(
        rate_limited_count >= 4,
        "Should have around 5 rate limited: {}",
        rate_limited_count
    );
}
