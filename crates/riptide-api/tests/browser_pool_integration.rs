//! Browser pool integration tests
//!
//! Comprehensive test suite for riptide-headless browser pool integration with riptide-api

use axum::http::StatusCode;
use serde_json::json;

mod test_helpers;
use test_helpers::*;

#[tokio::test]
async fn test_create_browser_session_success() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/v1/browser/session")
                .header("content-type", "application/json")
                .body(
                    json!({
                        "stealth_preset": "medium",
                        "initial_url": "https://example.com",
                        "timeout_secs": 300
                    })
                    .to_string(),
                )
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let session_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(session_response["session_id"].is_string());
    assert!(session_response["pool_stats"].is_object());
    assert!(session_response["created_at"].is_string());
    assert!(session_response["expires_at"].is_string());
}

#[tokio::test]
async fn test_create_browser_session_with_no_stealth() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/v1/browser/session")
                .header("content-type", "application/json")
                .body(
                    json!({
                        "stealth_preset": "none",
                        "initial_url": "about:blank"
                    })
                    .to_string(),
                )
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_browser_session_minimal() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/v1/browser/session")
                .header("content-type", "application/json")
                .body(json!({}).to_string())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should succeed with default values
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_execute_navigate_action() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/v1/browser/action")
                .header("content-type", "application/json")
                .body(
                    json!({
                        "action_type": "navigate",
                        "session_id": "test-session-123",
                        "url": "https://example.com",
                        "wait_for_load": true
                    })
                    .to_string(),
                )
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let action_result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(action_result["success"], true);
    assert!(action_result["result"].is_object());
    assert!(action_result["duration_ms"].is_number());
    assert!(action_result["messages"].is_array());
}

#[tokio::test]
async fn test_execute_screenshot_action() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/v1/browser/action")
                .header("content-type", "application/json")
                .body(
                    json!({
                        "action_type": "screenshot",
                        "session_id": "test-session-123",
                        "full_page": true
                    })
                    .to_string(),
                )
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let action_result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(action_result["success"], true);
    assert!(action_result["result"]["screenshot_base64"].is_string());
}

#[tokio::test]
async fn test_execute_script_action() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/v1/browser/action")
                .header("content-type", "application/json")
                .body(
                    json!({
                        "action_type": "execute_script",
                        "session_id": "test-session-123",
                        "script": "document.title",
                        "timeout_ms": 5000
                    })
                    .to_string(),
                )
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let action_result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(action_result["success"], true);
}

#[tokio::test]
async fn test_execute_get_content_action() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/v1/browser/action")
                .header("content-type", "application/json")
                .body(
                    json!({
                        "action_type": "get_content",
                        "session_id": "test-session-123"
                    })
                    .to_string(),
                )
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let action_result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(action_result["success"], true);
    assert!(action_result["result"]["html"].is_string());
}

#[tokio::test]
async fn test_execute_wait_for_element_action() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/v1/browser/action")
                .header("content-type", "application/json")
                .body(
                    json!({
                        "action_type": "wait_for_element",
                        "session_id": "test-session-123",
                        "selector": "#main-content",
                        "timeout_ms": 5000
                    })
                    .to_string(),
                )
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_execute_click_action() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/v1/browser/action")
                .header("content-type", "application/json")
                .body(
                    json!({
                        "action_type": "click",
                        "session_id": "test-session-123",
                        "selector": "button.submit"
                    })
                    .to_string(),
                )
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_execute_type_text_action() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/v1/browser/action")
                .header("content-type", "application/json")
                .body(
                    json!({
                        "action_type": "type_text",
                        "session_id": "test-session-123",
                        "selector": "input[name='username']",
                        "text": "testuser"
                    })
                    .to_string(),
                )
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_execute_render_pdf_action() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/v1/browser/action")
                .header("content-type", "application/json")
                .body(
                    json!({
                        "action_type": "render_pdf",
                        "session_id": "test-session-123",
                        "landscape": true,
                        "print_background": true
                    })
                    .to_string(),
                )
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_browser_pool_status() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/v1/browser/pool/status")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let pool_status: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(pool_status["stats"].is_object());
    assert!(pool_status["stats"]["available"].is_number());
    assert!(pool_status["stats"]["in_use"].is_number());
    assert!(pool_status["stats"]["total_capacity"].is_number());
    assert!(pool_status["stats"]["utilization_percent"].is_number());

    assert!(pool_status["launcher_stats"].is_object());
    assert!(pool_status["launcher_stats"]["total_requests"].is_number());
    assert!(pool_status["launcher_stats"]["successful_requests"].is_number());

    assert!(pool_status["health"].is_string());
}

#[tokio::test]
async fn test_close_browser_session() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri("/api/v1/browser/session/test-session-123")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_browser_pool_auto_scaling() {
    // Test that pool automatically scales based on demand
    let app = create_test_app().await;

    // Get initial status
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/v1/browser/pool/status")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let pool_status: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify pool has capacity
    assert!(pool_status["stats"]["total_capacity"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn test_browser_session_lifecycle() {
    // Test full lifecycle: create -> use -> close
    let app = create_test_app().await;

    // 1. Create session
    let create_response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/v1/browser/session")
                .header("content-type", "application/json")
                .body(json!({}).to_string())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(create_response.into_body())
        .await
        .unwrap();
    let session_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let session_id = session_response["session_id"].as_str().unwrap();

    // 2. Use session (navigate)
    let action_response = create_test_app()
        .await
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/v1/browser/action")
                .header("content-type", "application/json")
                .body(
                    json!({
                        "action_type": "navigate",
                        "session_id": session_id,
                        "url": "https://example.com"
                    })
                    .to_string(),
                )
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(action_response.status(), StatusCode::OK);

    // 3. Close session
    let close_response = create_test_app()
        .await
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(&format!("/api/v1/browser/session/{}", session_id))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(close_response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_invalid_action_type_handling() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/v1/browser/action")
                .header("content-type", "application/json")
                .body(
                    json!({
                        "action_type": "invalid_action",
                        "session_id": "test-session"
                    })
                    .to_string(),
                )
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return error for invalid action type
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
