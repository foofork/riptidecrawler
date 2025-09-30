//! API endpoint tests

use riptide_api::*;
use axum::http::StatusCode;
use axum::body::Body;
use tower::ServiceExt;

#[cfg(test)]
mod api_endpoint_tests {
    use super::*;

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = create_app();

        let response = app
            .oneshot(Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_crawl_endpoint() {
        let app = create_app();

        let body = r#"{"urls": ["https://example.com"], "max_pages": 10}"#;
        let response = app
            .oneshot(Request::builder()
                .method("POST")
                .uri("/api/v1/crawl")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap())
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn test_extract_endpoint() {
        let app = create_app();

        let body = r#"{"url": "https://example.com", "mode": "standard"}"#;
        let response = app
            .oneshot(Request::builder()
                .method("POST")
                .uri("/api/v1/extract")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap())
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_search_endpoint() {
        let app = create_app();

        let response = app
            .oneshot(Request::builder()
                .uri("/api/v1/search?q=test&limit=10")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_status_endpoint() {
        let app = create_app();

        let response = app
            .oneshot(Request::builder()
                .uri("/api/v1/status/job123")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let app = create_app();

        let response = app
            .oneshot(Request::builder()
                .uri("/api/v1/metrics")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_invalid_endpoint() {
        let app = create_app();

        let response = app
            .oneshot(Request::builder()
                .uri("/api/v1/invalid")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_cors_headers() {
        let app = create_app();

        let response = app
            .oneshot(Request::builder()
                .method("OPTIONS")
                .uri("/api/v1/crawl")
                .header("Origin", "https://example.com")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert!(response.headers().contains_key("access-control-allow-origin"));
        assert!(response.headers().contains_key("access-control-allow-methods"));
    }
}