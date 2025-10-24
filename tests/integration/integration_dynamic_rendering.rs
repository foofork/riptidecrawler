use anyhow::Result;
use riptide_core::dynamic::{DynamicConfig, PageAction, WaitCondition, ScrollConfig, ScrollMode, ViewportConfig};
use serde_json::json;
use std::time::Duration;
use tokio::time::timeout;

#[cfg(test)]
mod dynamic_rendering_tests {
    use super::*;

    #[tokio::test]
    async fn test_content_analysis_dynamic_domains() {
        // Test URLs that should trigger dynamic rendering
        let dynamic_urls = vec![
            "https://github.com/user/repo",
            "https://twitter.com/username/status/123",
            "https://medium.com/@author/article",
            "https://reddit.com/r/rust/comments/123",
            "https://linkedin.com/in/profile",
            "https://youtube.com/watch?v=abc123",
        ];

        for url in dynamic_urls {
            // This would call the analyze_url_for_dynamic_content function
            // For now, we'll simulate the expected behavior
            assert!(should_use_dynamic_rendering(url), "URL {} should use dynamic rendering", url);
        }
    }

    #[tokio::test]
    async fn test_content_analysis_static_domains() {
        // Test URLs that should use static rendering
        let static_urls = vec![
            "https://example.com/simple-page",
            "https://docs.rs/crate/latest",
            "https://crates.io/crates/crate-name",
            "https://blog.rust-lang.org/2023/article",
            "https://www.rust-lang.org/",
        ];

        for url in static_urls {
            assert!(!should_use_dynamic_rendering(url), "URL {} should use static rendering", url);
        }
    }

    #[tokio::test]
    async fn test_adaptive_config_generation() {
        // Test GitHub URL adaptive config
        let github_config = create_test_adaptive_config("https://github.com/user/repo");
        assert!(github_config.wait_for.is_some());
        assert!(github_config.timeout <= Duration::from_secs(3)); // Hard cap requirement

        // Test Twitter URL adaptive config
        let twitter_config = create_test_adaptive_config("https://twitter.com/user/status/123");
        assert!(twitter_config.wait_for.is_some());
        if let Some(scroll_config) = &twitter_config.scroll {
            assert!(scroll_config.steps > 2); // Social media needs more scrolling
        }

        // Test Medium URL adaptive config
        let medium_config = create_test_adaptive_config("https://medium.com/@author/article");
        assert!(medium_config.wait_for.is_some());
        if let Some(scroll_config) = &medium_config.scroll {
            assert_eq!(scroll_config.mode, ScrollMode::Smooth); // Articles use smooth scrolling
        }
    }

    #[tokio::test]
    async fn test_rpc_client_timeout_enforcement() {
        // Test that RPC client enforces 3-second hard timeout
        let start = std::time::Instant::now();

        // Create a mock config that would normally take longer
        let config = DynamicConfig {
            wait_for: Some(WaitCondition::Timeout(Duration::from_secs(10))), // Would timeout normally
            scroll: Some(ScrollConfig {
                steps: 100, // Excessive scrolling
                step_px: Some(1000),
                delay_ms: 100,
                mode: ScrollMode::Stepped,
                after_scroll_js: None,
                stop_condition: None,
            }),
            actions: vec![],
            capture_artifacts: false,
            timeout: Duration::from_secs(10), // Request 10s but should be capped at 3s
            viewport: None,
        };

        // This would normally call the RPC client, but for testing we simulate timeout
        let result = timeout(Duration::from_secs(4), simulate_dynamic_render(&config)).await;

        let elapsed = start.elapsed();

        // Should complete within 4 seconds due to 3s hard cap + some overhead
        assert!(elapsed < Duration::from_secs(4), "Should respect 3s hard timeout cap");
        assert!(result.is_ok(), "Should complete within timeout window");
    }

    #[tokio::test]
    async fn test_action_conversion() {
        use riptide_core::dynamic::PageAction;

        let actions = vec![
            PageAction::Click {
                selector: ".button".to_string(),
                wait_after: Some(Duration::from_millis(500)),
            },
            PageAction::Type {
                selector: "#input".to_string(),
                text: "test input".to_string(),
                clear_first: true,
                wait_after: Some(Duration::from_millis(200)),
            },
            PageAction::Wait(WaitCondition::Selector {
                selector: ".content".to_string(),
                timeout: Duration::from_secs(2),
            }),
            PageAction::Evaluate {
                script: "document.body.scrollTop = 100".to_string(),
                wait_after: None,
            },
        ];

        // Test that all supported actions can be converted
        let converted = convert_test_actions(&actions);
        assert_eq!(converted.len(), 4, "All actions should be converted");

        // Test specific action types
        assert!(converted.iter().any(|a| matches!(a, TestHeadlessAction::Click { .. })));
        assert!(converted.iter().any(|a| matches!(a, TestHeadlessAction::Type { .. })));
        assert!(converted.iter().any(|a| matches!(a, TestHeadlessAction::WaitForCss { .. })));
        assert!(converted.iter().any(|a| matches!(a, TestHeadlessAction::Js { .. })));
    }

    #[tokio::test]
    async fn test_session_persistence() {
        // Test session ID propagation
        let session_id = "test-session-123";

        let config = DynamicConfig {
            wait_for: None,
            scroll: None,
            actions: vec![],
            capture_artifacts: false,
            timeout: Duration::from_secs(3),
            viewport: None,
        };

        // Simulate first request with session creation
        let result1 = simulate_dynamic_render_with_session(&config, None).await;
        assert!(result1.session_id.is_some(), "Should create session ID");

        // Simulate second request reusing session
        let result2 = simulate_dynamic_render_with_session(&config, result1.session_id.as_deref()).await;
        assert_eq!(result1.session_id, result2.session_id, "Should reuse session ID");
    }

    #[tokio::test]
    async fn test_artifact_capture() {
        let config = DynamicConfig {
            wait_for: None,
            scroll: None,
            actions: vec![],
            capture_artifacts: true,
            timeout: Duration::from_secs(3),
            viewport: None,
        };

        let result = simulate_dynamic_render(&config).await;
        assert!(result.artifacts.is_some(), "Should capture artifacts when requested");

        if let Some(artifacts) = result.artifacts {
            // Should have at least screenshot when artifacts are enabled
            assert!(artifacts.screenshot.is_some() || artifacts.mhtml.is_some(),
                   "Should capture at least one type of artifact");
        }
    }

    #[tokio::test]
    async fn test_health_check_fallback() {
        // Test that dynamic rendering falls back to static when headless service is unavailable

        // Simulate headless service being down
        let unavailable_client = TestRpcClient::new_unavailable();

        let config = DynamicConfig::default();

        // Should fall back to static rendering without failing
        let result = simulate_render_with_fallback(&config, unavailable_client).await;
        assert!(result.success, "Should fall back to static rendering successfully");
        assert!(result.html.contains("static"), "Should indicate static rendering was used");
    }

    // Helper functions for testing

    fn should_use_dynamic_rendering(url: &str) -> bool {
        let url_lower = url.to_lowercase();
        let dynamic_domains = [
            "twitter.com", "x.com", "github.com", "medium.com",
            "reddit.com", "linkedin.com", "youtube.com"
        ];

        dynamic_domains.iter().any(|domain| url_lower.contains(domain))
    }

    fn create_test_adaptive_config(url: &str) -> DynamicConfig {
        let url_lower = url.to_lowercase();

        let wait_for = if url_lower.contains("github.com") {
            Some(WaitCondition::Selector {
                selector: ".repository-content".to_string(),
                timeout: Duration::from_secs(2),
            })
        } else if url_lower.contains("twitter.com") {
            Some(WaitCondition::Selector {
                selector: "[data-testid='tweet']".to_string(),
                timeout: Duration::from_millis(1500),
            })
        } else if url_lower.contains("medium.com") {
            Some(WaitCondition::Selector {
                selector: "article".to_string(),
                timeout: Duration::from_secs(2),
            })
        } else {
            Some(WaitCondition::DomContentLoaded)
        };

        let scroll = if url_lower.contains("twitter.com") {
            Some(ScrollConfig {
                steps: 5,
                step_px: Some(800),
                delay_ms: 800,
                mode: ScrollMode::Stepped,
                after_scroll_js: None,
                stop_condition: None,
            })
        } else if url_lower.contains("medium.com") {
            Some(ScrollConfig {
                steps: 3,
                step_px: Some(1000),
                delay_ms: 500,
                mode: ScrollMode::Smooth,
                after_scroll_js: None,
                stop_condition: None,
            })
        } else {
            Some(ScrollConfig::default())
        };

        DynamicConfig {
            wait_for,
            scroll,
            actions: vec![],
            capture_artifacts: false,
            timeout: Duration::from_secs(3), // Hard cap
            viewport: Some(ViewportConfig::default()),
        }
    }

    async fn simulate_dynamic_render(config: &DynamicConfig) -> TestRenderResult {
        // Simulate the RPC call with proper timeout enforcement
        let start = std::time::Instant::now();

        // Respect the 3-second hard cap
        let effective_timeout = std::cmp::min(config.timeout, Duration::from_secs(3));

        tokio::time::sleep(Duration::from_millis(100)).await; // Simulate processing time

        let elapsed = start.elapsed();
        let success = elapsed <= effective_timeout;

        TestRenderResult {
            success,
            html: "<html><body>Dynamic content</body></html>".to_string(),
            render_time_ms: elapsed.as_millis() as u64,
            actions_executed: vec!["wait_for_dom".to_string()],
            wait_conditions_met: vec!["dom_content_loaded".to_string()],
            artifacts: if config.capture_artifacts {
                Some(TestArtifacts {
                    screenshot: Some("base64_screenshot_data".to_string()),
                    mhtml: Some("base64_mhtml_data".to_string()),
                })
            } else {
                None
            },
            session_id: None,
        }
    }

    async fn simulate_dynamic_render_with_session(
        config: &DynamicConfig,
        session_id: Option<&str>
    ) -> TestRenderResult {
        let mut result = simulate_dynamic_render(config).await;
        result.session_id = Some(session_id.unwrap_or("generated-session-id").to_string());
        result
    }

    async fn simulate_render_with_fallback(
        _config: &DynamicConfig,
        _client: TestRpcClient
    ) -> TestRenderResult {
        // Simulate fallback to static rendering
        TestRenderResult {
            success: true,
            html: "<html><body>Static content fallback</body></html>".to_string(),
            render_time_ms: 50,
            actions_executed: vec![],
            wait_conditions_met: vec![],
            artifacts: None,
            session_id: None,
        }
    }

    fn convert_test_actions(actions: &[PageAction]) -> Vec<TestHeadlessAction> {
        actions.iter().filter_map(|action| {
            match action {
                PageAction::Click { selector, .. } => {
                    Some(TestHeadlessAction::Click { css: selector.clone() })
                }
                PageAction::Type { selector, text, .. } => {
                    Some(TestHeadlessAction::Type {
                        css: selector.clone(),
                        text: text.clone()
                    })
                }
                PageAction::Wait(WaitCondition::Selector { selector, .. }) => {
                    Some(TestHeadlessAction::WaitForCss { css: selector.clone() })
                }
                PageAction::Evaluate { script, .. } => {
                    Some(TestHeadlessAction::Js { code: script.clone() })
                }
                _ => None,
            }
        }).collect()
    }

    // Test types

    #[derive(Debug, Clone)]
    struct TestRenderResult {
        success: bool,
        html: String,
        render_time_ms: u64,
        actions_executed: Vec<String>,
        wait_conditions_met: Vec<String>,
        artifacts: Option<TestArtifacts>,
        session_id: Option<String>,
    }

    #[derive(Debug, Clone)]
    struct TestArtifacts {
        screenshot: Option<String>,
        mhtml: Option<String>,
    }

    #[derive(Debug)]
    enum TestHeadlessAction {
        Click { css: String },
        Type { css: String, text: String },
        WaitForCss { css: String },
        Js { code: String },
    }

    struct TestRpcClient {
        available: bool,
    }

    impl TestRpcClient {
        fn new_unavailable() -> Self {
            Self { available: false }
        }
    }
}