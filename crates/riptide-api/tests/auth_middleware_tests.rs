//! Authentication Middleware Integration Tests
//!
//! Tests for authentication middleware integration with:
//! - Route integration and middleware ordering
//! - Public vs protected endpoint handling
//! - Rate limiting middleware interaction
//! - Error response format validation
//! - Request extension population
//! - Middleware chain behavior

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware,
    routing::{get, post},
    Extension, Router,
};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

use riptide_api::{
    middleware::{auth_middleware, rate_limit_middleware, AuthConfig},
    state::AppState,
};

/// Helper to extract JSON from response body
async fn body_to_json(body: Body) -> Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap_or(serde_json::json!({}))
}

// ============================================================================
// ROUTE INTEGRATION TESTS
// ============================================================================

#[tokio::test]
async fn test_middleware_integration_with_protected_routes() {
    // Setup: Create app with auth middleware on specific routes
    let auth_config = AuthConfig::with_api_keys(vec!["test-key".to_string()]);
    let mut state = AppState::new_test_minimal().await;
    state.auth_config = auth_config;

    let protected_routes = Router::new()
        .route("/protected", get(|| async { "protected data" }))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let app = Router::new()
        .route("/public", get(|| async { "public data" }))
        .merge(protected_routes)
        .with_state(state);

    // Test: Public route accessible without auth
    let request = Request::builder()
        .uri("/public")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Public route should be accessible"
    );

    // Test: Protected route requires auth
    let request = Request::builder()
        .uri("/protected")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Protected route should require auth"
    );

    // Test: Protected route with valid auth
    let request = Request::builder()
        .uri("/protected")
        .header("X-API-Key", "test-key")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Protected route should accept valid auth"
    );
}

#[tokio::test]
async fn test_public_endpoints_bypass_authentication() {
    // Setup: Create app state with auth required
    let auth_config = AuthConfig::with_api_keys(vec!["required-key".to_string()]);
    let mut state = AppState::new_test_minimal().await;
    state.auth_config = auth_config;

    // Create app with auth middleware
    let app = Router::new()
        .route("/health", get(|| async { "healthy" }))
        .route("/metrics", get(|| async { "metrics" }))
        .route("/api/v1/health", get(|| async { "healthy" }))
        .route("/api/v1/data", get(|| async { "protected" }))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state);

    // Test all public paths
    let public_paths = vec!["/health", "/metrics", "/api/v1/health"];

    for path in public_paths {
        let request = Request::builder().uri(path).body(Body::empty()).unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Public path {} should bypass auth",
            path
        );
    }

    // Test protected path still requires auth
    let request = Request::builder()
        .uri("/api/v1/data")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Protected path should still require auth"
    );
}

#[tokio::test]
async fn test_middleware_ordering_auth_before_handler() {
    // Setup: Create handler that would panic if auth not checked
    async fn sensitive_handler(Extension(auth): Extension<bool>) -> &'static str {
        if !auth {
            panic!("Should never reach here without auth!");
        }
        "authenticated"
    }

    let auth_config = AuthConfig::with_api_keys(vec!["test-key".to_string()]);
    let mut state = AppState::new_test_minimal().await;
    state.auth_config = auth_config;

    let app = Router::new()
        .route("/sensitive", get(sensitive_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state);

    // Test: Request without auth should be rejected before handler
    let request = Request::builder()
        .uri("/sensitive")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should get 401, not panic
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Auth middleware should reject before handler"
    );
}

// ============================================================================
// RATE LIMITING INTEGRATION TESTS
// ============================================================================

#[tokio::test]
async fn test_rate_limit_and_auth_middleware_integration() {
    // Setup: Create app with both auth and rate limiting
    let auth_config = AuthConfig::with_api_keys(vec!["test-key".to_string()]);
    let mut state = AppState::new_test_minimal().await;
    state.auth_config = auth_config;

    let app = Router::new()
        .route("/api/test", get(|| async { "success" }))
        // Order: rate limit first, then auth
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ))
        .with_state(state);

    // Test: Request with valid auth should pass both middlewares
    let request = Request::builder()
        .uri("/api/test")
        .header("X-API-Key", "test-key")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Valid auth should pass both middlewares"
    );

    // Test: Request without auth should fail at auth layer
    let request = Request::builder()
        .uri("/api/test")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Missing auth should fail at auth middleware"
    );
}

// ============================================================================
// ERROR RESPONSE FORMAT TESTS
// ============================================================================

#[tokio::test]
async fn test_error_response_format_consistency() {
    // Setup: Create app with auth
    let auth_config = AuthConfig::with_api_keys(vec!["valid-key".to_string()]);
    let mut state = AppState::new_test_minimal().await;
    state.auth_config = auth_config;

    let app = Router::new()
        .route("/test", get(|| async { "success" }))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state);

    // Test: Missing API key error format
    let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let body = body_to_json(response.into_body()).await;

    assert_eq!(body["error"], "Unauthorized");
    assert_eq!(body["message"], "Missing API key");
    assert!(body.get("error").is_some());
    assert!(body.get("message").is_some());

    // Test: Invalid API key error format
    let request = Request::builder()
        .uri("/test")
        .header("X-API-Key", "invalid-key")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body = body_to_json(response.into_body()).await;

    assert_eq!(body["error"], "Unauthorized");
    assert_eq!(body["message"], "Invalid API key");
    assert!(body.get("error").is_some());
    assert!(body.get("message").is_some());
}

#[tokio::test]
async fn test_error_response_headers() {
    // Setup: Create app with auth
    let auth_config = AuthConfig::with_api_keys(vec!["test-key".to_string()]);
    let mut state = AppState::new_test_minimal().await;
    state.auth_config = auth_config;

    let app = Router::new()
        .route("/test", get(|| async { "success" }))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state);

    // Test: Error response has correct headers
    let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Check Content-Type
    let content_type = response.headers().get("Content-Type");
    assert!(content_type.is_some(), "Content-Type header should be set");
    assert!(
        content_type
            .unwrap()
            .to_str()
            .unwrap()
            .contains("application/json"),
        "Content-Type should be application/json"
    );

    // Check WWW-Authenticate
    let www_auth = response.headers().get("WWW-Authenticate");
    assert!(www_auth.is_some(), "WWW-Authenticate header should be set");
    assert_eq!(
        www_auth.unwrap().to_str().unwrap(),
        "Bearer",
        "WWW-Authenticate should be Bearer"
    );
}

// ============================================================================
// MULTIPLE ROUTE PATTERNS TESTS
// ============================================================================

#[tokio::test]
async fn test_authentication_across_nested_routes() {
    // Setup: Create nested route structure
    let auth_config = AuthConfig::with_api_keys(vec!["api-key".to_string()]);
    let mut state = AppState::new_test_minimal().await;
    state.auth_config = auth_config;

    let api_v1 = Router::new()
        .route("/users", get(|| async { "users" }))
        .route("/posts", get(|| async { "posts" }));

    let api_v2 = Router::new()
        .route("/users", get(|| async { "users v2" }))
        .route("/posts", get(|| async { "posts v2" }));

    let app = Router::new()
        .nest("/api/v1", api_v1)
        .nest("/api/v2", api_v2)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state);

    // Test all routes require auth
    let test_paths = vec![
        "/api/v1/users",
        "/api/v1/posts",
        "/api/v2/users",
        "/api/v2/posts",
    ];

    for path in test_paths {
        // Without auth
        let request = Request::builder().uri(path).body(Body::empty()).unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "{} should require auth",
            path
        );

        // With auth
        let request = Request::builder()
            .uri(path)
            .header("X-API-Key", "api-key")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "{} should accept valid auth",
            path
        );
    }
}

#[tokio::test]
async fn test_authentication_with_different_route_methods() {
    // Setup: Create routes with different HTTP methods
    let auth_config = AuthConfig::with_api_keys(vec!["method-key".to_string()]);
    let mut state = AppState::new_test_minimal().await;
    state.auth_config = auth_config;

    let app = Router::new()
        .route("/resource", get(|| async { "GET resource" }))
        .route("/resource", post(|| async { "POST resource" }))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state);

    // Test GET without auth
    let request = Request::builder()
        .method("GET")
        .uri("/resource")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "GET should require auth"
    );

    // Test POST without auth
    let request = Request::builder()
        .method("POST")
        .uri("/resource")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "POST should require auth"
    );

    // Test GET with auth
    let request = Request::builder()
        .method("GET")
        .uri("/resource")
        .header("X-API-Key", "method-key")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "GET with auth should succeed"
    );

    // Test POST with auth
    let request = Request::builder()
        .method("POST")
        .uri("/resource")
        .header("X-API-Key", "method-key")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "POST with auth should succeed"
    );
}

// ============================================================================
// MIDDLEWARE STATE TESTS
// ============================================================================

#[tokio::test]
async fn test_auth_config_disable_authentication() {
    // Setup: Create auth config with auth disabled
    let mut auth_config = AuthConfig::new();
    // Simulate REQUIRE_AUTH=false via environment
    std::env::set_var("REQUIRE_AUTH", "false");
    let auth_config = AuthConfig::new();
    std::env::remove_var("REQUIRE_AUTH");

    let mut state = AppState::new_test_minimal().await;
    state.auth_config = auth_config;

    let app = Router::new()
        .route("/test", get(|| async { "success" }))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state);

    // Test: Should allow requests without API key when auth disabled
    let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Should allow access when auth disabled"
    );
}

#[tokio::test]
async fn test_dynamic_api_key_management() {
    use tokio::time::{sleep, Duration};

    // Setup: Create auth config
    let auth_config = AuthConfig::with_api_keys(vec!["initial-key".to_string()]);
    let mut state = AppState::new_test_minimal().await;
    let config_ref = auth_config.clone();
    state.auth_config = auth_config;

    let app = Router::new()
        .route("/test", get(|| async { "success" }))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state);

    // Test: Initial key works
    let request = Request::builder()
        .uri("/test")
        .header("X-API-Key", "initial-key")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Add new key dynamically
    config_ref.add_api_key("new-key".to_string()).await;

    // Small delay to ensure async update
    sleep(Duration::from_millis(10)).await;

    // Test: New key should work
    let request = Request::builder()
        .uri("/test")
        .header("X-API-Key", "new-key")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Dynamically added key should work"
    );

    // Remove initial key
    config_ref.remove_api_key("initial-key").await;
    sleep(Duration::from_millis(10)).await;

    // Test: Removed key should not work
    let request = Request::builder()
        .uri("/test")
        .header("X-API-Key", "initial-key")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Removed key should not work"
    );
}

// ============================================================================
// COMPREHENSIVE MIDDLEWARE CHAIN TESTS
// ============================================================================

#[tokio::test]
async fn test_middleware_chain_execution_order() {
    use std::sync::{Arc, Mutex};

    // Setup: Track middleware execution order
    let execution_order = Arc::new(Mutex::new(Vec::new()));

    // Custom middleware to track execution
    let order_tracker = execution_order.clone();
    let tracking_middleware = move |req: Request<Body>, next: middleware::Next| {
        let tracker = order_tracker.clone();
        async move {
            tracker.lock().unwrap().push("before_handler");
            let response = next.run(req).await;
            tracker.lock().unwrap().push("after_handler");
            response
        }
    };

    let auth_config = AuthConfig::with_api_keys(vec!["test-key".to_string()]);
    let mut state = AppState::new_test_minimal().await;
    state.auth_config = auth_config;

    let app = Router::new()
        .route("/test", get(|| async { "success" }))
        .layer(middleware::from_fn(tracking_middleware))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state);

    // Execute request
    let request = Request::builder()
        .uri("/test")
        .header("X-API-Key", "test-key")
        .body(Body::empty())
        .unwrap();

    let _ = app.oneshot(request).await.unwrap();

    // Verify execution order
    let order = execution_order.lock().unwrap();
    assert_eq!(order[0], "before_handler");
    assert_eq!(order[1], "after_handler");
}

#[tokio::test]
async fn test_error_propagation_through_middleware_chain() {
    // Setup: Create middleware chain with auth that fails
    let auth_config = AuthConfig::with_api_keys(vec!["valid-key".to_string()]);
    let mut state = AppState::new_test_minimal().await;
    state.auth_config = auth_config;

    let app = Router::new()
        .route("/test", get(|| async { "success" }))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state);

    // Test: Auth failure should prevent handler execution
    let request = Request::builder()
        .uri("/test")
        .header("X-API-Key", "invalid-key")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should get auth error, not handler response
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = body_to_json(response.into_body()).await;
    assert_eq!(body["error"], "Unauthorized");
}
