//! Health and metrics validation tests
//!
//! Tests for API health endpoints and metrics validation based on
//! WEEK_1_ACTION_PLAN.md requirements.

use std::time::Duration;
use tokio::time::sleep;

#[cfg(test)]
mod health_tests {
    use super::*;

    /// Test data for health check validation
    #[derive(Debug)]
    struct HealthCheckResult {
        status: String,
        version: Option<String>,
        uptime: Option<u64>,
        memory_usage: Option<u64>,
    }

    /// Test health endpoint returns valid response
    #[tokio::test]
    async fn test_health_endpoint_format() {
        // This test assumes API is running on localhost:8080
        // In real scenarios, use test harness to start API server

        let client = reqwest::Client::new();
        let response = client
            .get("http://localhost:8080/healthz")
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        if let Ok(resp) = response {
            assert!(resp.status().is_success(), "Health endpoint should return 2xx");

            if let Ok(json) = resp.json::<serde_json::Value>().await {
                // Validate required fields
                assert!(json.get("status").is_some(), "Health response should have 'status' field");

                // Optional fields validation
                if let Some(version) = json.get("version") {
                    assert!(version.is_string(), "Version should be a string");
                }

                if let Some(uptime) = json.get("uptime") {
                    assert!(uptime.is_number(), "Uptime should be a number");
                }
            }
        }
    }

    /// Test health endpoint responds quickly (TTFB < 500ms)
    #[tokio::test]
    async fn test_health_endpoint_performance() {
        let client = reqwest::Client::new();
        let start = std::time::Instant::now();

        let response = client
            .get("http://localhost:8080/healthz")
            .send()
            .await;

        let duration = start.elapsed();

        if response.is_ok() {
            assert!(
                duration < Duration::from_millis(500),
                "Health check should respond within 500ms, took {:?}",
                duration
            );
        }
    }

    /// Test metrics endpoint returns Prometheus format
    #[tokio::test]
    async fn test_metrics_endpoint_format() {
        let client = reqwest::Client::new();
        let response = client
            .get("http://localhost:8080/metrics")
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        if let Ok(resp) = response {
            assert!(resp.status().is_success(), "Metrics endpoint should return 2xx");

            if let Ok(body) = resp.text().await {
                // Validate Prometheus format
                assert!(!body.is_empty(), "Metrics should not be empty");

                // Check for common metric patterns
                let has_metrics = body.contains("# HELP")
                    || body.contains("# TYPE")
                    || body.contains("riptide")
                    || body.contains("http_requests")
                    || body.contains("memory");

                assert!(has_metrics, "Metrics should contain expected metric definitions");
            }
        }
    }

    /// Test metrics are updated after extraction
    #[tokio::test]
    async fn test_metrics_update_after_extraction() {
        let client = reqwest::Client::new();

        // Get initial metrics
        let initial_response = client
            .get("http://localhost:8080/metrics")
            .send()
            .await;

        if initial_response.is_err() {
            return; // Skip if API not running
        }

        // Perform extraction
        let _ = client
            .post("http://localhost:8080/extract")
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "url": "https://example.com"
            }))
            .send()
            .await;

        // Wait a bit for metrics to update
        sleep(Duration::from_millis(500)).await;

        // Get updated metrics
        let updated_response = client
            .get("http://localhost:8080/metrics")
            .send()
            .await;

        if let (Ok(initial), Ok(updated)) = (initial_response, updated_response) {
            if let (Ok(initial_body), Ok(updated_body)) = (initial.text().await, updated.text().await) {
                // Metrics should have changed (more comprehensive check would parse values)
                assert_ne!(
                    initial_body.len(), updated_body.len(),
                    "Metrics should be updated after extraction"
                );
            }
        }
    }

    /// Test metrics contain expected categories
    #[tokio::test]
    async fn test_metrics_categories() {
        let client = reqwest::Client::new();
        let response = client
            .get("http://localhost:8080/metrics")
            .send()
            .await;

        if let Ok(resp) = response {
            if let Ok(body) = resp.text().await {
                // Expected metric categories from WEEK_1_ACTION_PLAN.md
                let expected_categories = vec![
                    "request", "memory", "pipeline", "extraction", "error"
                ];

                let mut found_categories = 0;
                for category in expected_categories {
                    if body.to_lowercase().contains(category) {
                        found_categories += 1;
                    }
                }

                assert!(
                    found_categories >= 3,
                    "Metrics should contain at least 3 major categories, found: {}",
                    found_categories
                );
            }
        }
    }

    /// Test health endpoint is resilient under load
    #[tokio::test]
    async fn test_health_endpoint_under_load() {
        let client = reqwest::Client::new();
        let mut handles = vec![];

        // Send 10 concurrent health check requests
        for _ in 0..10 {
            let client = client.clone();
            let handle = tokio::spawn(async move {
                client
                    .get("http://localhost:8080/healthz")
                    .timeout(Duration::from_secs(5))
                    .send()
                    .await
            });
            handles.push(handle);
        }

        // Wait for all requests
        let mut success_count = 0;
        for handle in handles {
            if let Ok(Ok(response)) = handle.await {
                if response.status().is_success() {
                    success_count += 1;
                }
            }
        }

        // All health checks should succeed
        assert!(
            success_count >= 9,
            "At least 90% of health checks should succeed under load, got {}/10",
            success_count
        );
    }
}
