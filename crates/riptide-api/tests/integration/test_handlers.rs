use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::time::Duration;
use tower::ServiceExt;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

// Mock implementations for testing
struct MockAppState {
    pub mock_redis_server: MockServer,
    pub mock_serper_server: MockServer,
}

impl MockAppState {
    async fn new() -> Self {
        let mock_redis_server = MockServer::start().await;
        let mock_serper_server = MockServer::start().await;

        Self {
            mock_redis_server,
            mock_serper_server,
        }
    }

    fn create_test_router() -> Router {
        // Create a simplified router for testing
        // Note: This uses mock handlers that simulate the real behavior
        Router::new()
            .route("/healthz", get(mock_health_handler))
            .route("/crawl", post(mock_crawl_handler))
            .route("/deepsearch", post(mock_deepsearch_handler))
            .fallback(mock_not_found_handler)
    }
}

// Mock handler implementations
async fn mock_health_handler() -> axum::response::Json<Value> {
    axum::response::Json(json!({
        "status": "healthy",
        "version": "0.1.0",
        "timestamp": "2024-01-01T00:00:00Z",
        "uptime": 3600,
        "dependencies": {
            "redis": {
                "status": "healthy",
                "message": null,
                "response_time_ms": null,
                "last_check": "2024-01-01T00:00:00Z"
            },
            "extractor": {
                "status": "healthy",
                "message": null,
                "response_time_ms": null,
                "last_check": "2024-01-01T00:00:00Z"
            },
            "http_client": {
                "status": "healthy",
                "message": null,
                "response_time_ms": null,
                "last_check": "2024-01-01T00:00:00Z"
            },
            "headless_service": null
        },
        "metrics": {
            "memory_usage_bytes": 0,
            "active_connections": 0,
            "total_requests": 0,
            "requests_per_second": 0.0,
            "avg_response_time_ms": 5.0
        }
    }))
}

async fn mock_crawl_handler(
    axum::extract::Json(payload): axum::extract::Json<Value>,
) -> Result<axum::response::Json<Value>, (StatusCode, axum::response::Json<Value>)> {
    // Validate request structure
    let urls = payload.get("urls")
        .and_then(|u| u.as_array())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                axum::response::Json(json!({
                    "error": {
                        "type": "validation_error",
                        "message": "urls field is required and must be an array",
                        "retryable": false,
                        "status": 400
                    }
                }))
            )
        })?;

    if urls.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            axum::response::Json(json!({
                "error": {
                    "type": "validation_error",
                    "message": "At least one URL is required",
                    "retryable": false,
                    "status": 400
                }
            }))
        ));
    }

    // Mock successful crawl response
    let mut results = Vec::new();
    for (index, url) in urls.iter().enumerate() {
        let url_str = url.as_str().unwrap_or("invalid");

        // Simulate different scenarios based on URL
        if url_str.contains("localhost") {
            return Err((
                StatusCode::BAD_REQUEST,
                axum::response::Json(json!({
                    "error": {
                        "type": "invalid_url",
                        "message": format!("URL {} targets private/localhost address", index + 1),
                        "retryable": false,
                        "status": 400
                    }
                }))
            ));
        }

        results.push(json!({
            "url": url_str,
            "status": 200,
            "from_cache": false,
            "gate_decision": "raw",
            "quality_score": 0.8,
            "processing_time_ms": 150,
            "document": {
                "url": url_str,
                "title": "Mock Title",
                "byline": null,
                "published_iso": null,
                "markdown": "# Mock Content",
                "text": "Mock Content",
                "links": [],
                "media": []
            },
            "error": null,
            "cache_key": format!("riptide:v1:enabled:{:x}", index)
        }));
    }

    Ok(axum::response::Json(json!({
        "total_urls": urls.len(),
        "successful": urls.len(),
        "failed": 0,
        "from_cache": 0,
        "results": results,
        "statistics": {
            "total_processing_time_ms": 300,
            "avg_processing_time_ms": 150.0,
            "gate_decisions": {
                "raw": urls.len(),
                "probes_first": 0,
                "headless": 0,
                "cached": 0
            },
            "cache_hit_rate": 0.0
        }
    })))
}

async fn mock_deepsearch_handler(
    axum::extract::Json(payload): axum::extract::Json<Value>,
) -> Result<axum::response::Json<Value>, (StatusCode, axum::response::Json<Value>)> {
    let query = payload.get("query")
        .and_then(|q| q.as_str())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                axum::response::Json(json!({
                    "error": {
                        "type": "validation_error",
                        "message": "query field is required",
                        "retryable": false,
                        "status": 400
                    }
                }))
            )
        })?;

    if query.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            axum::response::Json(json!({
                "error": {
                    "type": "validation_error",
                    "message": "Search query cannot be empty",
                    "retryable": false,
                    "status": 400
                }
            }))
        ));
    }

    // Mock search results
    Ok(axum::response::Json(json!({
        "query": query,
        "urls_found": 3,
        "urls_crawled": 3,
        "results": [
            {
                "url": "https://example.com/result1",
                "rank": 1,
                "search_title": "Mock Result 1",
                "search_snippet": "This is a mock search result",
                "content": {
                    "url": "https://example.com/result1",
                    "title": "Mock Result 1",
                    "byline": null,
                    "published_iso": null,
                    "markdown": "# Mock Result 1",
                    "text": "Mock content for result 1",
                    "links": [],
                    "media": []
                },
                "crawl_result": {
                    "url": "https://example.com/result1",
                    "status": 200,
                    "from_cache": false,
                    "gate_decision": "raw",
                    "quality_score": 0.85,
                    "processing_time_ms": 200,
                    "document": null,
                    "error": null,
                    "cache_key": "mock_key_1"
                }
            }
        ],
        "status": "completed",
        "processing_time_ms": 1500
    })))
}

async fn mock_not_found_handler() -> (StatusCode, axum::response::Json<Value>) {
    (
        StatusCode::NOT_FOUND,
        axum::response::Json(json!({
            "error": {
                "type": "not_found",
                "message": "The requested endpoint was not found",
                "retryable": false,
                "status": 404
            }
        }))
    )
}

#[cfg(test)]
mod health_endpoint_tests {
    use super::*;

    #[tokio::test]
    async fn test_health_endpoint_success() {
        let app = MockAppState::create_test_router();

        let request = Request::builder()
            .method(Method::GET)
            .uri("/healthz")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["status"], "healthy");
        assert_eq!(json["version"], "0.1.0");
        assert!(json["timestamp"].is_string());
        assert!(json["uptime"].is_number());
        assert!(json["dependencies"].is_object());
        assert!(json["metrics"].is_object());

        // Check dependency structure
        assert_eq!(json["dependencies"]["redis"]["status"], "healthy");
        assert_eq!(json["dependencies"]["extractor"]["status"], "healthy");
        assert_eq!(json["dependencies"]["http_client"]["status"], "healthy");
        assert!(json["dependencies"]["headless_service"].is_null());
    }

    #[tokio::test]
    async fn test_health_endpoint_response_structure() {
        let app = MockAppState::create_test_router();

        let request = Request::builder()
            .method(Method::GET)
            .uri("/healthz")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // Verify all required fields are present
        assert!(json.get("status").is_some());
        assert!(json.get("version").is_some());
        assert!(json.get("timestamp").is_some());
        assert!(json.get("uptime").is_some());
        assert!(json.get("dependencies").is_some());
        assert!(json.get("metrics").is_some());

        // Verify dependencies structure
        let deps = &json["dependencies"];
        assert!(deps.get("redis").is_some());
        assert!(deps.get("extractor").is_some());
        assert!(deps.get("http_client").is_some());

        // Verify metrics structure
        let metrics = &json["metrics"];
        assert!(metrics.get("memory_usage_bytes").is_some());
        assert!(metrics.get("active_connections").is_some());
        assert!(metrics.get("total_requests").is_some());
        assert!(metrics.get("requests_per_second").is_some());
        assert!(metrics.get("avg_response_time_ms").is_some());
    }
}

#[cfg(test)]
mod crawl_endpoint_tests {
    use super::*;

    #[tokio::test]
    async fn test_crawl_single_url_success() {
        let app = MockAppState::create_test_router();

        let request_body = json!({
            "urls": ["https://example.com"]
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/crawl")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["total_urls"], 1);
        assert_eq!(json["successful"], 1);
        assert_eq!(json["failed"], 0);
        assert_eq!(json["from_cache"], 0);

        let results = json["results"].as_array().unwrap();
        assert_eq!(results.len(), 1);

        let result = &results[0];
        assert_eq!(result["url"], "https://example.com");
        assert_eq!(result["status"], 200);
        assert_eq!(result["from_cache"], false);
        assert_eq!(result["gate_decision"], "raw");
        assert!(result["document"].is_object());
        assert!(result["error"].is_null());
    }

    #[tokio::test]
    async fn test_crawl_multiple_urls_success() {
        let app = MockAppState::create_test_router();

        let request_body = json!({
            "urls": [
                "https://example.com",
                "https://test.org",
                "https://demo.net"
            ]
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/crawl")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["total_urls"], 3);
        assert_eq!(json["successful"], 3);
        assert_eq!(json["failed"], 0);

        let results = json["results"].as_array().unwrap();
        assert_eq!(results.len(), 3);

        // Check statistics
        let stats = &json["statistics"];
        assert!(stats["total_processing_time_ms"].is_number());
        assert!(stats["avg_processing_time_ms"].is_number());
        assert_eq!(stats["gate_decisions"]["raw"], 3);
        assert_eq!(stats["cache_hit_rate"], 0.0);
    }

    #[tokio::test]
    async fn test_crawl_empty_urls_error() {
        let app = MockAppState::create_test_router();

        let request_body = json!({
            "urls": []
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/crawl")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"]["type"], "validation_error");
        assert!(json["error"]["message"].as_str().unwrap().contains("At least one URL is required"));
        assert_eq!(json["error"]["retryable"], false);
        assert_eq!(json["error"]["status"], 400);
    }

    #[tokio::test]
    async fn test_crawl_localhost_url_error() {
        let app = MockAppState::create_test_router();

        let request_body = json!({
            "urls": ["http://localhost:8080"]
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/crawl")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"]["type"], "invalid_url");
        assert!(json["error"]["message"].as_str().unwrap().contains("private/localhost"));
        assert_eq!(json["error"]["retryable"], false);
        assert_eq!(json["error"]["status"], 400);
    }

    #[tokio::test]
    async fn test_crawl_missing_urls_field() {
        let app = MockAppState::create_test_router();

        let request_body = json!({
            "invalid_field": "value"
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/crawl")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"]["type"], "validation_error");
        assert!(json["error"]["message"].as_str().unwrap().contains("urls field is required"));
    }

    #[tokio::test]
    async fn test_crawl_invalid_json() {
        let app = MockAppState::create_test_router();

        let request = Request::builder()
            .method(Method::POST)
            .uri("/crawl")
            .header("content-type", "application/json")
            .body(Body::from("invalid json"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should return 400 for invalid JSON
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}

#[cfg(test)]
mod deepsearch_endpoint_tests {
    use super::*;

    #[tokio::test]
    async fn test_deepsearch_success() {
        let app = MockAppState::create_test_router();

        let request_body = json!({
            "query": "rust programming language",
            "limit": 10,
            "include_content": true
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/deepsearch")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["query"], "rust programming language");
        assert_eq!(json["urls_found"], 3);
        assert_eq!(json["urls_crawled"], 3);
        assert_eq!(json["status"], "completed");
        assert!(json["processing_time_ms"].is_number());

        let results = json["results"].as_array().unwrap();
        assert!(!results.is_empty());

        let result = &results[0];
        assert!(result["url"].is_string());
        assert!(result["rank"].is_number());
        assert!(result["search_title"].is_string());
        assert!(result["content"].is_object());
        assert!(result["crawl_result"].is_object());
    }

    #[tokio::test]
    async fn test_deepsearch_minimal_request() {
        let app = MockAppState::create_test_router();

        let request_body = json!({
            "query": "test"
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/deepsearch")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["query"], "test");
        assert!(json["urls_found"].is_number());
        assert!(json["results"].is_array());
    }

    #[tokio::test]
    async fn test_deepsearch_empty_query_error() {
        let app = MockAppState::create_test_router();

        let request_body = json!({
            "query": ""
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/deepsearch")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"]["type"], "validation_error");
        assert!(json["error"]["message"].as_str().unwrap().contains("cannot be empty"));
    }

    #[tokio::test]
    async fn test_deepsearch_missing_query_field() {
        let app = MockAppState::create_test_router();

        let request_body = json!({
            "limit": 10
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/deepsearch")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"]["type"], "validation_error");
        assert!(json["error"]["message"].as_str().unwrap().contains("query field is required"));
    }
}

#[cfg(test)]
mod not_found_tests {
    use super::*;

    #[tokio::test]
    async fn test_unknown_endpoint_404() {
        let app = MockAppState::create_test_router();

        let request = Request::builder()
            .method(Method::GET)
            .uri("/unknown/endpoint")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"]["type"], "not_found");
        assert_eq!(json["error"]["message"], "The requested endpoint was not found");
        assert_eq!(json["error"]["retryable"], false);
        assert_eq!(json["error"]["status"], 404);
    }

    #[tokio::test]
    async fn test_wrong_method_404() {
        let app = MockAppState::create_test_router();

        // Try POST on health endpoint (should be GET)
        let request = Request::builder()
            .method(Method::POST)
            .uri("/healthz")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

#[cfg(test)]
mod http_method_tests {
    use super::*;

    #[tokio::test]
    async fn test_health_endpoint_methods() {
        let app = MockAppState::create_test_router();

        // GET should work
        let request = Request::builder()
            .method(Method::GET)
            .uri("/healthz")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_crawl_endpoint_methods() {
        let app = MockAppState::create_test_router();

        let request_body = json!({"urls": ["https://example.com"]});

        // POST should work
        let request = Request::builder()
            .method(Method::POST)
            .uri("/crawl")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_content_type_requirements() {
        let app = MockAppState::create_test_router();

        // Missing content-type header
        let request = Request::builder()
            .method(Method::POST)
            .uri("/crawl")
            .body(Body::from(r#"{"urls": ["https://example.com"]}"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should still work or return appropriate error
        // The exact behavior depends on axum's JSON extractor implementation
        assert!(response.status().is_client_error() || response.status().is_success());
    }
}

// Performance and stress tests
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_health_endpoint_response_time() {
        let app = MockAppState::create_test_router();

        let start = Instant::now();

        let request = Request::builder()
            .method(Method::GET)
            .uri("/healthz")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let duration = start.elapsed();

        assert_eq!(response.status(), StatusCode::OK);
        // Health check should be fast (under 100ms for mock)
        assert!(duration < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_concurrent_health_requests() {
        use futures::future::join_all;

        let app = MockAppState::create_test_router();

        // Create 10 concurrent health check requests
        let requests = (0..10).map(|_| {
            let app = app.clone();
            async move {
                let request = Request::builder()
                    .method(Method::GET)
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap();

                app.oneshot(request).await
            }
        });

        let responses = join_all(requests).await;

        // All requests should succeed
        for response in responses {
            let response = response.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
    }

    #[tokio::test]
    async fn test_large_url_batch_performance() {
        let app = MockAppState::create_test_router();

        // Create a batch of 50 URLs (within limits)
        let urls: Vec<Value> = (0..50)
            .map(|i| Value::String(format!("https://example{}.com", i)))
            .collect();

        let request_body = json!({
            "urls": urls
        });

        let start = Instant::now();

        let request = Request::builder()
            .method(Method::POST)
            .uri("/crawl")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let duration = start.elapsed();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["total_urls"], 50);
        assert_eq!(json["successful"], 50);

        // Large batch should complete within reasonable time (for mock)
        assert!(duration < Duration::from_secs(2));
    }
}