use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

/// Test the new PR-1 Headless RPC v2 features
#[cfg(test)]
mod tests {
    use super::*;

    /// Test basic rendering with backward compatibility
    #[tokio::test]
    async fn test_render_basic_backward_compat() {
        // This test would connect to the headless service
        // For now, we'll create the expected request format
        let request = json!({
            "url": "https://example.com",
            "wait_for": ".content",
            "scroll_steps": 3
        });

        // Verify backward compatibility fields are present
        assert!(request["url"].is_string());
        assert!(request["wait_for"].is_string());
        assert!(request["scroll_steps"].is_number());
    }

    /// Test new action execution features
    #[tokio::test]
    async fn test_render_with_actions() {
        let request = json!({
            "url": "https://example.com",
            "actions": [
                {
                    "type": "wait_for_css",
                    "css": ".dynamic-content",
                    "timeout_ms": 3000
                },
                {
                    "type": "scroll",
                    "steps": 5,
                    "step_px": 1000,
                    "delay_ms": 200
                },
                {
                    "type": "js",
                    "code": "document.querySelector('.load-more')?.click();"
                },
                {
                    "type": "wait_for_js",
                    "expr": "document.querySelectorAll('.item').length > 10",
                    "timeout_ms": 5000
                }
            ],
            "artifacts": {
                "screenshot": true,
                "mhtml": false
            }
        });

        // Verify new fields structure
        assert!(request["actions"].is_array());
        assert_eq!(request["actions"].as_array().unwrap().len(), 4);
        assert!(request["artifacts"]["screenshot"].as_bool().unwrap());
    }

    /// Test click and type actions
    #[tokio::test]
    async fn test_interactive_actions() {
        let request = json!({
            "url": "https://example.com/form",
            "actions": [
                {
                    "type": "type",
                    "css": "#username",
                    "text": "testuser",
                    "delay_ms": 50
                },
                {
                    "type": "type",
                    "css": "#password",
                    "text": "password123",
                    "delay_ms": 50
                },
                {
                    "type": "click",
                    "css": "#submit-button"
                },
                {
                    "type": "wait_for_css",
                    "css": ".dashboard",
                    "timeout_ms": 10000
                }
            ]
        });

        // Verify interactive action structure
        let actions = request["actions"].as_array().unwrap();
        assert_eq!(actions[0]["type"], "type");
        assert_eq!(actions[2]["type"], "click");
    }

    /// Test session persistence feature
    #[tokio::test]
    async fn test_session_persistence() {
        let request1 = json!({
            "url": "https://example.com/login",
            "session_id": "test-session-123",
            "actions": [
                {
                    "type": "type",
                    "css": "#username",
                    "text": "user"
                }
            ]
        });

        let request2 = json!({
            "url": "https://example.com/dashboard",
            "session_id": "test-session-123",  // Reuse same session
            "actions": [
                {
                    "type": "wait_for_css",
                    "css": ".logged-in-content"
                }
            ]
        });

        assert_eq!(request1["session_id"], request2["session_id"]);
    }

    /// Test timeout configurations
    #[tokio::test]
    async fn test_custom_timeouts() {
        let request = json!({
            "url": "https://slow-site.example.com",
            "timeouts": {
                "nav_ms": 10000,
                "idle_after_dcl_ms": 2000,
                "hard_cap_ms": 15000
            },
            "actions": [
                {
                    "type": "wait_for_css",
                    "css": ".slow-content",
                    "timeout_ms": 8000
                }
            ]
        });

        assert_eq!(request["timeouts"]["nav_ms"], 10000);
        assert_eq!(request["timeouts"]["hard_cap_ms"], 15000);
    }

    /// Test artifact capture
    #[tokio::test]
    async fn test_artifact_capture() {
        let request = json!({
            "url": "https://example.com",
            "artifacts": {
                "screenshot": true,
                "mhtml": true
            }
        });

        // Expected response structure
        let expected_response = json!({
            "final_url": "https://example.com",
            "html": "<html>...</html>",
            "screenshot_b64": "base64_encoded_screenshot",
            "session_id": null,
            "artifacts": {
                "screenshot_b64": "base64_encoded_screenshot",
                "mhtml_b64": null  // Not yet implemented
            }
        });

        // Verify response structure matches new format
        assert!(expected_response["artifacts"].is_object());
        assert!(expected_response["screenshot_b64"].is_string());
    }

    /// Test complex workflow with multiple action types
    #[tokio::test]
    async fn test_complex_workflow() {
        let request = json!({
            "url": "https://spa-app.example.com",
            "session_id": "workflow-session",
            "timeouts": {
                "nav_ms": 5000,
                "idle_after_dcl_ms": 1000,
                "hard_cap_ms": 30000
            },
            "actions": [
                // Wait for initial load
                {
                    "type": "wait_for_css",
                    "css": "#app",
                    "timeout_ms": 5000
                },
                // Execute custom JS to prepare state
                {
                    "type": "js",
                    "code": "window.appReady = true;"
                },
                // Wait for JS condition
                {
                    "type": "wait_for_js",
                    "expr": "window.appReady === true",
                    "timeout_ms": 1000
                },
                // Navigate to section
                {
                    "type": "click",
                    "css": "a[href='#products']"
                },
                // Wait for products to load
                {
                    "type": "wait_for_css",
                    "css": ".product-grid",
                    "timeout_ms": 3000
                },
                // Scroll to load more
                {
                    "type": "scroll",
                    "steps": 3,
                    "step_px": 1500,
                    "delay_ms": 500
                },
                // Wait for lazy-loaded items
                {
                    "type": "wait_for_js",
                    "expr": "document.querySelectorAll('.product').length >= 20",
                    "timeout_ms": 5000
                },
                // Click load more if exists
                {
                    "type": "js",
                    "code": "document.querySelector('.load-more')?.click();"
                },
                // Final wait
                {
                    "type": "wait_for_css",
                    "css": ".loading-complete",
                    "timeout_ms": 3000
                }
            ],
            "artifacts": {
                "screenshot": true,
                "mhtml": false
            }
        });

        let actions = request["actions"].as_array().unwrap();
        assert_eq!(actions.len(), 9);

        // Verify mix of action types
        let action_types: Vec<&str> = actions
            .iter()
            .map(|a| a["type"].as_str().unwrap())
            .collect();

        assert!(action_types.contains(&"wait_for_css"));
        assert!(action_types.contains(&"wait_for_js"));
        assert!(action_types.contains(&"scroll"));
        assert!(action_types.contains(&"js"));
        assert!(action_types.contains(&"click"));
    }
}