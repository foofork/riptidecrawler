/// Integration tests module
///
/// This module contains integration tests that test the interaction between
/// multiple components of the RipTide API system.

pub mod test_handlers;
pub mod test_edge_cases;
pub mod spider_pages_mode_tests;

#[cfg(test)]
mod test_helpers {
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
        Router,
    };
    use serde_json::Value;
    use tower::ServiceExt;

    /// Helper to create a test request
    pub async fn create_test_request(
        method: Method,
        uri: &str,
        body: Option<Value>,
    ) -> Request<Body> {
        let mut request_builder = Request::builder().method(method).uri(uri);

        let body = if let Some(json_body) = body {
            request_builder = request_builder.header("content-type", "application/json");
            Body::from(serde_json::to_string(&json_body).unwrap())
        } else {
            Body::empty()
        };

        request_builder.body(body).unwrap()
    }

    /// Helper to execute a request and return the response body as JSON
    pub async fn execute_request_json(
        app: Router,
        request: Request<Body>,
    ) -> (StatusCode, Value) {
        let response = app.oneshot(request).await.unwrap();
        let status = response.status();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        (status, json)
    }

    /// Helper to check if a response contains expected error structure
    pub fn assert_error_response(json: &Value, expected_type: &str, expected_status: u16) {
        assert!(json.get("error").is_some(), "Response should contain error field");

        let error = &json["error"];
        assert_eq!(error["type"], expected_type);
        assert_eq!(error["status"], expected_status);
        assert!(error.get("message").is_some());
        assert!(error.get("retryable").is_some());
    }

    /// Helper to check if a health response is valid
    pub fn assert_valid_health_response(json: &Value) {
        assert!(json.get("status").is_some());
        assert!(json.get("version").is_some());
        assert!(json.get("timestamp").is_some());
        assert!(json.get("uptime").is_some());
        assert!(json.get("dependencies").is_some());

        let deps = &json["dependencies"];
        assert!(deps.get("redis").is_some());
        assert!(deps.get("extractor").is_some());
        assert!(deps.get("http_client").is_some());
    }

    /// Helper to check if a crawl response is valid
    pub fn assert_valid_crawl_response(json: &Value, expected_url_count: usize) {
        assert!(json.get("total_urls").is_some());
        assert!(json.get("successful").is_some());
        assert!(json.get("failed").is_some());
        assert!(json.get("from_cache").is_some());
        assert!(json.get("results").is_some());
        assert!(json.get("statistics").is_some());

        assert_eq!(json["total_urls"], expected_url_count);

        let results = json["results"].as_array().unwrap();
        assert_eq!(results.len(), expected_url_count);

        // Check each result has required fields
        for result in results {
            assert!(result.get("url").is_some());
            assert!(result.get("status").is_some());
            assert!(result.get("from_cache").is_some());
            assert!(result.get("gate_decision").is_some());
            assert!(result.get("quality_score").is_some());
            assert!(result.get("processing_time_ms").is_some());
            assert!(result.get("cache_key").is_some());
        }

        // Check statistics structure
        let stats = &json["statistics"];
        assert!(stats.get("total_processing_time_ms").is_some());
        assert!(stats.get("avg_processing_time_ms").is_some());
        assert!(stats.get("gate_decisions").is_some());
        assert!(stats.get("cache_hit_rate").is_some());
    }
}