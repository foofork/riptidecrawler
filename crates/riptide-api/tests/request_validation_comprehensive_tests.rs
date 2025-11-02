//! Comprehensive tests for request validation middleware
//!
//! Tests all validation rules:
//! - Content-Type validation
//! - Payload size limits
//! - Required header validation
//! - URL parameter sanitization
//! - Method allowlist enforcement

use axum::{
    body::Body,
    extract::Request,
    http::{header, Method, StatusCode, Uri},
    middleware,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use tower::ServiceExt;

// Import from the actual middleware module
use riptide_api::middleware::request_validation::request_validation_middleware;

/// Helper to extract JSON body from response
async fn extract_json_body(response: Response) -> Value {
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body_bytes).unwrap()
}

// ============================================================================
// Content-Type Validation Tests
// ============================================================================

#[tokio::test]
async fn test_valid_json_content_type_passes() {
    async fn handler(Json(_payload): Json<Value>) -> Json<Value> {
        Json(json!({"status": "ok"}))
    }

    let app = Router::new()
        .route("/api/test", post(handler))
        .layer(middleware::from_fn(request_validation_middleware));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/test")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"test": "data"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_invalid_content_type_rejected() {
    async fn handler() -> Json<Value> {
        Json(json!({"status": "ok"}))
    }

    let app = Router::new()
        .route("/api/test", post(handler))
        .layer(middleware::from_fn(request_validation_middleware));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/test")
                .header(header::CONTENT_TYPE, "text/plain")
                .body(Body::from("plain text"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

    let body = extract_json_body(response).await;
    assert_eq!(body["error"]["type"], "unsupported_media_type");
    assert_eq!(body["error"]["status"], 415);
}

#[tokio::test]
async fn test_multipart_form_data_allowed() {
    async fn handler() -> Json<Value> {
        Json(json!({"status": "ok"}))
    }

    let app = Router::new()
        .route("/api/upload", post(handler))
        .layer(middleware::from_fn(request_validation_middleware));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/upload")
                .header(
                    header::CONTENT_TYPE,
                    "multipart/form-data; boundary=----WebKitFormBoundary",
                )
                .body(Body::from("------WebKitFormBoundary\r\n"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ============================================================================
// Payload Size Validation Tests
// ============================================================================

#[tokio::test]
async fn test_payload_within_size_limit_passes() {
    async fn handler(Json(_payload): Json<Value>) -> Json<Value> {
        Json(json!({"status": "ok"}))
    }

    let app = Router::new()
        .route("/api/test", post(handler))
        .layer(middleware::from_fn(request_validation_middleware));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/test")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::CONTENT_LENGTH, "1024") // 1KB - well within limit
                .body(Body::from(r#"{"test": "data"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_oversized_payload_rejected() {
    async fn handler() -> Json<Value> {
        Json(json!({"status": "ok"}))
    }

    let app = Router::new()
        .route("/api/test", post(handler))
        .layer(middleware::from_fn(request_validation_middleware));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/test")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::CONTENT_LENGTH, "11534336") // 11MB - exceeds 10MB limit
                .body(Body::from(r#"{"test": "data"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);

    let body = extract_json_body(response).await;
    assert_eq!(body["error"]["type"], "payload_too_large");
    assert_eq!(body["error"]["status"], 413);
    assert!(body["error"]["max_size_bytes"].is_number());
    assert!(body["error"]["received_bytes"].is_number());
}

// ============================================================================
// Header Validation Tests
// ============================================================================

#[tokio::test]
async fn test_valid_api_key_header_passes() {
    async fn handler() -> Json<Value> {
        Json(json!({"status": "ok"}))
    }

    let app = Router::new()
        .route("/api/test", get(handler))
        .layer(middleware::from_fn(request_validation_middleware));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/test")
                .header("X-API-Key", "valid-api-key-12345")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_empty_api_key_rejected() {
    async fn handler() -> Json<Value> {
        Json(json!({"status": "ok"}))
    }

    let app = Router::new()
        .route("/api/test", get(handler))
        .layer(middleware::from_fn(request_validation_middleware));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/test")
                .header("X-API-Key", "")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = extract_json_body(response).await;
    assert_eq!(body["error"]["type"], "invalid_header_value");
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("invalid length"));
}

#[tokio::test]
async fn test_api_key_with_whitespace_rejected() {
    async fn handler() -> Json<Value> {
        Json(json!({"status": "ok"}))
    }

    let app = Router::new()
        .route("/api/test", get(handler))
        .layer(middleware::from_fn(request_validation_middleware));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/test")
                .header("X-API-Key", "key with spaces")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = extract_json_body(response).await;
    assert_eq!(body["error"]["type"], "invalid_header_value");
}

#[tokio::test]
async fn test_negative_content_length_rejected() {
    async fn handler() -> Json<Value> {
        Json(json!({"status": "ok"}))
    }

    let app = Router::new()
        .route("/api/test", post(handler))
        .layer(middleware::from_fn(request_validation_middleware));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/test")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::CONTENT_LENGTH, "-100")
                .body(Body::from(r#"{"test": "data"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = extract_json_body(response).await;
    assert_eq!(body["error"]["type"], "invalid_header_value");
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("cannot be negative"));
}

// ============================================================================
// URL Parameter Sanitization Tests
// ============================================================================

#[tokio::test]
async fn test_sql_injection_attempt_rejected() {
    async fn handler() -> Json<Value> {
        Json(json!({"status": "ok"}))
    }

    let app = Router::new()
        .route("/api/search", get(handler))
        .layer(middleware::from_fn(request_validation_middleware));

    let malicious_queries = vec![
        "?q=test' UNION SELECT * FROM users--",
        "?id=1; DROP TABLE users;",
        "?search=test' OR '1'='1",
        "?param=value/*comment*/",
        "?filter=test'; exec xp_cmdshell 'dir'",
    ];

    for query in malicious_queries {
        let uri = format!("/api/search{}", query);
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(uri.clone())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Failed to reject SQL injection: {}",
            query
        );

        let body = extract_json_body(response).await;
        assert_eq!(body["error"]["type"], "invalid_parameter");
    }
}

#[tokio::test]
async fn test_xss_attempt_rejected() {
    async fn handler() -> Json<Value> {
        Json(json!({"status": "ok"}))
    }

    let app = Router::new()
        .route("/api/search", get(handler))
        .layer(middleware::from_fn(request_validation_middleware));

    let malicious_queries = vec![
        "?q=<script>alert('xss')</script>",
        "?url=javascript:alert(1)",
        "?param=<iframe src='evil.com'>",
        "?data=<img onerror='alert(1)'>",
        "?code=eval(malicious)",
    ];

    for query in malicious_queries {
        let uri = format!("/api/search{}", query);
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(uri.clone())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Failed to reject XSS: {}",
            query
        );

        let body = extract_json_body(response).await;
        assert_eq!(body["error"]["type"], "invalid_parameter");
    }
}

#[tokio::test]
async fn test_path_traversal_rejected() {
    async fn handler() -> Json<Value> {
        Json(json!({"status": "ok"}))
    }

    let app = Router::new()
        .route("/api/files", get(handler))
        .layer(middleware::from_fn(request_validation_middleware));

    let malicious_queries = vec![
        "?path=../../etc/passwd",
        "?file=..\\..\\windows\\system32",
        "?name=../../../secret",
    ];

    for query in malicious_queries {
        let uri = format!("/api/files{}", query);
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(uri.clone())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Failed to reject path traversal: {}",
            query
        );

        let body = extract_json_body(response).await;
        assert_eq!(body["error"]["type"], "invalid_parameter");
        assert!(body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("path traversal"));
    }
}

#[tokio::test]
async fn test_safe_parameters_allowed() {
    async fn handler() -> Json<Value> {
        Json(json!({"status": "ok"}))
    }

    let app = Router::new()
        .route("/api/search", get(handler))
        .layer(middleware::from_fn(request_validation_middleware));

    let safe_queries = vec![
        "?q=hello world",
        "?filter=category:tech",
        "?page=1&limit=10",
        "?sort=name&order=asc",
        "?tags=rust,axum,api",
    ];

    for query in safe_queries {
        let uri = format!("/api/search{}", query);
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(uri.clone())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Incorrectly rejected safe query: {}",
            query
        );
    }
}

// ============================================================================
// HTTP Method Validation Tests
// ============================================================================

#[tokio::test]
async fn test_get_allowed_on_health_endpoint() {
    async fn handler() -> Json<Value> {
        Json(json!({"status": "healthy"}))
    }

    let app = Router::new()
        .route("/healthz", get(handler))
        .layer(middleware::from_fn(request_validation_middleware));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_post_rejected_on_health_endpoint() {
    async fn handler() -> Json<Value> {
        Json(json!({"status": "healthy"}))
    }

    let app = Router::new()
        .route("/healthz", get(handler))
        .layer(middleware::from_fn(request_validation_middleware));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/healthz")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);

    // Check Allow header before consuming body
    let allow_header = response.headers().get(header::ALLOW).cloned();
    assert!(allow_header.is_some());

    let body = extract_json_body(response).await;
    assert_eq!(body["error"]["type"], "method_not_allowed");
    assert_eq!(body["error"]["status"], 405);
}

// ============================================================================
// Integration Tests - Multiple Validations
// ============================================================================

#[tokio::test]
async fn test_multiple_validation_errors_first_one_wins() {
    async fn handler() -> Json<Value> {
        Json(json!({"status": "ok"}))
    }

    let app = Router::new()
        .route("/healthz", get(handler))
        .layer(middleware::from_fn(request_validation_middleware));

    // POST on health endpoint (method error) + invalid content-type
    // Method validation happens first
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/healthz?q=../../etc/passwd")
                .header(header::CONTENT_TYPE, "text/plain")
                .body(Body::from("invalid"))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should fail on method validation (first check)
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn test_valid_request_passes_all_validations() {
    async fn handler(Json(_payload): Json<Value>) -> Json<Value> {
        Json(json!({"result": "success"}))
    }

    let app = Router::new()
        .route("/api/v1/crawl", post(handler))
        .layer(middleware::from_fn(request_validation_middleware));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/crawl?depth=2&limit=10")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::CONTENT_LENGTH, "50")
                .header(header::USER_AGENT, "RipTide-Client/1.0")
                .header("X-API-Key", "valid-key-123")
                .body(Body::from(r#"{"url":"https://example.com"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
