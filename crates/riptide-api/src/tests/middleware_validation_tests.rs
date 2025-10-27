//! Integration tests for request validation middleware
//!
//! Tests validation of:
//! - Malformed JSON payloads (400 Bad Request)
//! - Unsupported HTTP methods (405 Method Not Allowed)
//! - Invalid Content-Type headers (415 Unsupported Media Type)

#[cfg(test)]
mod tests {
    use crate::middleware::request_validation::{
        get_allowed_methods, request_validation_middleware, should_validate_body,
        validate_http_method,
    };
    use axum::{
        body::Body,
        extract::Request,
        http::{header, Method, StatusCode},
        Router,
    };
    use serde_json::json;
    use tower::ServiceExt; // for `oneshot`

    #[test]
    fn test_should_validate_body() {
        assert!(should_validate_body(&Method::POST));
        assert!(should_validate_body(&Method::PUT));
        assert!(should_validate_body(&Method::PATCH));
        assert!(!should_validate_body(&Method::GET));
        assert!(!should_validate_body(&Method::DELETE));
        assert!(!should_validate_body(&Method::HEAD));
    }

    #[test]
    fn test_get_allowed_methods_health_endpoints() {
        let methods = get_allowed_methods("/healthz");
        assert!(methods.contains("GET"));
        assert!(methods.contains("HEAD"));
        assert!(!methods.contains("POST"));
        assert_eq!(methods.len(), 2);

        let methods = get_allowed_methods("/health/detailed");
        assert!(methods.contains("GET"));
        assert!(!methods.contains("POST"));
    }

    #[test]
    fn test_get_allowed_methods_metrics() {
        let methods = get_allowed_methods("/metrics");
        assert!(methods.contains("GET"));
        assert!(methods.contains("HEAD"));
        assert!(!methods.contains("POST"));

        let methods = get_allowed_methods("/api/v1/metrics");
        assert!(methods.contains("GET"));
        assert!(!methods.contains("POST"));
    }

    #[test]
    fn test_get_allowed_methods_search() {
        let methods = get_allowed_methods("/search");
        assert!(methods.contains("GET"));
        assert!(methods.contains("HEAD"));
        assert!(!methods.contains("POST"));

        let methods = get_allowed_methods("/api/v1/search");
        assert!(methods.contains("GET"));
        assert!(!methods.contains("POST"));
    }

    #[test]
    fn test_get_allowed_methods_post_only_endpoints() {
        // Crawl endpoints
        let methods = get_allowed_methods("/crawl");
        assert!(methods.contains("POST"));
        assert!(!methods.contains("GET"));
        assert_eq!(methods.len(), 1);

        let methods = get_allowed_methods("/api/v1/crawl");
        assert!(methods.contains("POST"));
        assert!(!methods.contains("GET"));

        // Extract endpoints
        let methods = get_allowed_methods("/extract");
        assert!(methods.contains("POST"));
        assert!(!methods.contains("GET"));

        let methods = get_allowed_methods("/api/v1/extract");
        assert!(methods.contains("POST"));
        assert!(!methods.contains("GET"));

        // DeepSearch
        let methods = get_allowed_methods("/deepsearch");
        assert!(methods.contains("POST"));
        assert!(!methods.contains("GET"));

        // Render
        let methods = get_allowed_methods("/render");
        assert!(methods.contains("POST"));
        assert!(!methods.contains("GET"));
    }

    #[test]
    fn test_get_allowed_methods_websocket() {
        let methods = get_allowed_methods("/crawl/ws");
        assert!(methods.contains("GET"));
        assert!(!methods.contains("POST"));
        assert_eq!(methods.len(), 1);
    }

    #[test]
    fn test_get_allowed_methods_restful_apis() {
        let methods = get_allowed_methods("/api/v1/browser/session");
        assert!(methods.contains("GET"));
        assert!(methods.contains("POST"));
        assert!(methods.contains("PUT"));
        assert!(methods.contains("PATCH"));
        assert!(methods.contains("DELETE"));

        let methods = get_allowed_methods("/api/v1/llm/providers");
        assert!(methods.contains("GET"));
        assert!(methods.contains("POST"));

        let methods = get_allowed_methods("/admin/tenants");
        assert!(methods.contains("GET"));
        assert!(methods.contains("POST"));
        assert!(methods.contains("PUT"));
        assert!(methods.contains("DELETE"));
    }

    #[test]
    fn test_get_allowed_methods_default() {
        let methods = get_allowed_methods("/some/unknown/path");
        assert!(methods.contains("GET"));
        assert!(methods.contains("POST"));
        assert!(methods.contains("PUT"));
        assert!(methods.contains("PATCH"));
        assert!(methods.contains("DELETE"));
        assert!(methods.contains("HEAD"));
    }

    #[test]
    fn test_validate_http_method_success() {
        // Health endpoints allow GET
        let result = validate_http_method(&Method::GET, "/healthz");
        assert!(result.is_ok());

        let result = validate_http_method(&Method::HEAD, "/healthz");
        assert!(result.is_ok());

        // Crawl allows POST
        let result = validate_http_method(&Method::POST, "/crawl");
        assert!(result.is_ok());

        // Search allows GET
        let result = validate_http_method(&Method::GET, "/search");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_http_method_failure() {
        // Health endpoints don't allow POST
        let result = validate_http_method(&Method::POST, "/healthz");
        assert!(result.is_err());

        // Crawl doesn't allow GET
        let result = validate_http_method(&Method::GET, "/crawl");
        assert!(result.is_err());

        // Extract doesn't allow GET
        let result = validate_http_method(&Method::GET, "/api/v1/extract");
        assert!(result.is_err());

        // Search doesn't allow POST
        let result = validate_http_method(&Method::POST, "/search");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_method_not_allowed_response_format() {
        // Test that 405 responses have the correct format
        let result = validate_http_method(&Method::POST, "/healthz");
        assert!(result.is_err());

        if let Err(response) = result {
            assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);

            // Check Allow header is present
            let headers = response.headers();
            assert!(headers.contains_key(header::ALLOW));

            let allow_header = headers.get(header::ALLOW).unwrap().to_str().unwrap();
            assert!(allow_header.contains("GET"));
        }
    }

    // Note: JSON rejection handling is tested in integration tests
    // where actual malformed JSON is sent to endpoints

    #[tokio::test]
    async fn test_middleware_allows_valid_get_request() {
        use axum::{routing::get, Json};

        async fn handler() -> Json<serde_json::Value> {
            Json(json!({"status": "ok"}))
        }

        let app = Router::new()
            .route("/healthz", get(handler))
            .layer(axum::middleware::from_fn(request_validation_middleware));

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
    async fn test_middleware_rejects_invalid_method() {
        use axum::{routing::get, Json};

        async fn handler() -> Json<serde_json::Value> {
            Json(json!({"status": "ok"}))
        }

        let app = Router::new()
            .route("/healthz", get(handler))
            .layer(axum::middleware::from_fn(request_validation_middleware));

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

        // Check Allow header
        let allow_header = response.headers().get(header::ALLOW).unwrap();
        let allowed = allow_header.to_str().unwrap();
        assert!(allowed.contains("GET"));
    }

    #[tokio::test]
    async fn test_middleware_validates_content_type() {
        use axum::{routing::post, Json};

        async fn handler(Json(_payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
            Json(json!({"status": "ok"}))
        }

        let app = Router::new()
            .route("/crawl", post(handler))
            .layer(axum::middleware::from_fn(request_validation_middleware));

        // Test with wrong Content-Type
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/crawl")
                    .header(header::CONTENT_TYPE, "text/plain")
                    .body(Body::from("not json"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn test_middleware_allows_multipart_form_data() {
        use axum::{routing::post, Json};

        async fn handler() -> Json<serde_json::Value> {
            Json(json!({"status": "ok"}))
        }

        let app = Router::new()
            .route("/pdf/upload", post(handler))
            .layer(axum::middleware::from_fn(request_validation_middleware));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/pdf/upload")
                    .header(
                        header::CONTENT_TYPE,
                        "multipart/form-data; boundary=----WebKitFormBoundary",
                    )
                    .body(Body::from("------WebKitFormBoundary\r\n"))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should pass validation middleware and reach handler
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_middleware_allows_valid_json_post() {
        use axum::{routing::post, Json};

        async fn handler(Json(_payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
            Json(json!({"status": "ok"}))
        }

        let app = Router::new()
            .route("/crawl", post(handler))
            .layer(axum::middleware::from_fn(request_validation_middleware));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/crawl")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"url": "https://example.com"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_requests_bypass_content_validation() {
        use axum::{routing::get, Json};

        async fn handler() -> Json<serde_json::Value> {
            Json(json!({"status": "ok"}))
        }

        let app = Router::new()
            .route("/search", get(handler))
            .layer(axum::middleware::from_fn(request_validation_middleware));

        // GET requests don't need Content-Type validation
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/search?q=test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
