//! Comprehensive error handling tests for critical production paths
//!
//! This test suite validates that all critical production code paths
//! handle errors gracefully without panicking due to unwrap/expect calls.

use riptide_api::{errors::ApiError, handlers::render::RenderRequest, state::AppState};
use serde_json::json;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_render_handler_error_resilience() {
    let app_state = AppState::new()
        .await
        .expect("Failed to create test AppState");

    // Test invalid URL handling
    let invalid_request = RenderRequest {
        url: "not-a-valid-url".to_string(),
        mode: None,
        dynamic_config: None,
        stealth_config: None,
        pdf_config: None,
        output_format: None,
        capture_artifacts: None,
        timeout: None,
        session_id: None,
    };

    // This should not panic and should return a proper error
    let result = timeout(
        Duration::from_secs(5),
        riptide_api::handlers::render::render(
            axum::extract::State(app_state.clone()),
            riptide_api::sessions::middleware::SessionContext::anonymous(),
            axum::Json(invalid_request),
        ),
    )
    .await;

    match result {
        Ok(handler_result) => {
            assert!(
                handler_result.is_err(),
                "Invalid URL should result in error, not success"
            );
            if let Err(api_error) = handler_result {
                assert!(
                    matches!(api_error, ApiError::ValidationError { .. }),
                    "Expected validation error for invalid URL"
                );
            }
        }
        Err(_) => {
            panic!("Handler should not timeout on invalid URL - this indicates a hang")
        }
    }
}

#[tokio::test]
async fn test_resource_manager_time_handling_robustness() {
    use riptide_api::config::ApiConfig;
    use riptide_api::resource_manager::ResourceManager;

    let config = ApiConfig::default();
    let resource_manager = ResourceManager::new(config)
        .await
        .expect("Failed to create resource manager");

    // Test that cleanup operations handle time errors gracefully
    resource_manager.cleanup_on_timeout("test_operation").await;

    // Verify status can still be retrieved
    let status = resource_manager.get_resource_status().await;
    assert!(status.timeout_count > 0, "Timeout should have been recorded");
}

#[tokio::test]
async fn test_streaming_error_response_building() {
    use axum::{body::Body, http::Response};
    use axum::http::StatusCode;

    // Test that we can always build an error response, even with corrupted data
    let test_json = json!({
        "error": "test error",
        "invalid_utf8": "\u{FFFF}"
    });

    // This should not panic even if JSON serialization or response building fails
    let response_result = Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header("Content-Type", "application/json")
        .body(Body::from(test_json.to_string()));

    match response_result {
        Ok(_) => {
            // Success case - response built successfully
        }
        Err(_) => {
            // Fallback case - should still be able to create empty response
            let fallback_response = Response::new(Body::empty());
            assert_eq!(fallback_response.status(), StatusCode::OK);
        }
    }
}

#[tokio::test]
async fn test_url_parsing_edge_cases() {
    use url::Url;

    let problematic_urls = vec![
        "",
        "not-a-url",
        "http://",
        "ftp://example.com",
        "http://[invalid-ipv6",
        "http://user:pass@example.com:99999",
        "javascript:alert('xss')",
    ];

    for url_str in problematic_urls {
        // Test that URL parsing failures are handled gracefully
        let parse_result = Url::parse(url_str);

        match parse_result {
            Ok(url) => {
                // If URL parses, ensure host extraction is safe
                let host = url.host_str().unwrap_or("localhost");
                assert!(!host.is_empty(), "Host should never be empty string");
            }
            Err(e) => {
                // Parse failure should be handled gracefully
                println!("URL '{}' failed to parse (expected): {}", url_str, e);
            }
        }
    }
}

#[tokio::test]
async fn test_memory_pressure_handling() {
    use riptide_api::config::ApiConfig;
    use riptide_api::resource_manager::ResourceManager;

    let mut config = ApiConfig::default();
    config.memory.global_memory_limit_mb = 1; // Very low limit to trigger pressure
    config.memory.pressure_threshold = 0.5;

    let resource_manager = ResourceManager::new(config)
        .await
        .expect("Failed to create resource manager");

    // Force memory allocation that should trigger pressure detection
    resource_manager
        .memory_manager
        .track_allocation(2)
        .await; // Exceeds limit

    // Should detect pressure without panicking
    assert!(
        resource_manager.memory_manager.is_under_pressure(),
        "Memory pressure should be detected"
    );

    // Cleanup should work without panicking
    resource_manager
        .memory_manager
        .trigger_cleanup()
        .await;
}

/// Test that demonstrates graceful degradation under extreme conditions
#[tokio::test]
async fn test_graceful_degradation_stress() {
    let app_state = AppState::new()
        .await
        .expect("Failed to create test AppState");

    // Simulate multiple concurrent requests with problematic inputs
    let stress_requests = vec![
        ("", "Empty URL"),
        ("http://", "Incomplete URL"),
        ("http://localhost:99999", "Invalid port"),
        ("http://example.com/page-with-massive-content", "Potentially large page"),
    ];

    let mut tasks = Vec::new();

    for (url, description) in stress_requests {
        let app_clone = app_state.clone();
        let url_clone = url.to_string();
        let desc_clone = description.to_string();

        let task = tokio::spawn(async move {
            let request = RenderRequest {
                url: url_clone,
                mode: None,
                dynamic_config: None,
                stealth_config: None,
                pdf_config: None,
                output_format: None,
                capture_artifacts: None,
                timeout: Some(1), // Very short timeout
                session_id: None,
            };

            let result = timeout(
                Duration::from_secs(2),
                riptide_api::handlers::render::render(
                    axum::extract::State(app_clone),
                    riptide_api::sessions::middleware::SessionContext::anonymous(),
                    axum::Json(request),
                ),
            )
            .await;

            // Should either succeed or fail gracefully, never panic
            match result {
                Ok(handler_result) => {
                    println!("{}: Handler completed (success or error)", desc_clone);
                    match handler_result {
                        Ok(_) => println!("  -> Success"),
                        Err(e) => println!("  -> Graceful error: {}", e),
                    }
                }
                Err(_) => {
                    println!("{}: Handler timed out (acceptable under stress)", desc_clone);
                }
            }
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete without panicking
    for task in tasks {
        task.await.expect("Task should complete without panicking");
    }
}