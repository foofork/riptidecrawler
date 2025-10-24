//! Dynamic Content Handling Tests
//!
//! Tests for dynamic configuration, wait conditions, scroll behavior, and page actions

use riptide_headless::dynamic::{
    DynamicCapabilities, DynamicConfig, DynamicError, PageAction, ScrollConfig, ScrollMode,
    ViewportConfig, WaitCondition,
};
use std::collections::HashMap;
use std::time::Duration;

#[test]
fn test_dynamic_config_default() {
    let config = DynamicConfig::default();

    assert!(config.wait_for.is_none());
    assert!(config.scroll.is_none());
    assert!(config.actions.is_empty());
    assert!(!config.capture_artifacts);
    assert_eq!(config.timeout, Duration::from_secs(30));
    assert!(config.viewport.is_none());
}

#[test]
fn test_dynamic_config_with_custom_values() {
    let config = DynamicConfig {
        wait_for: Some(WaitCondition::Timeout(Duration::from_secs(5))),
        scroll: Some(ScrollConfig::default()),
        actions: vec![PageAction::Click {
            selector: "button".to_string(),
            wait_after: None,
        }],
        capture_artifacts: true,
        timeout: Duration::from_secs(60),
        viewport: Some(ViewportConfig::default()),
    };

    assert!(config.wait_for.is_some());
    assert!(config.scroll.is_some());
    assert_eq!(config.actions.len(), 1);
    assert!(config.capture_artifacts);
    assert_eq!(config.timeout, Duration::from_secs(60));
    assert!(config.viewport.is_some());
}

#[test]
fn test_wait_condition_variants() {
    let selector_wait = WaitCondition::Selector {
        selector: ".dynamic-content".to_string(),
        timeout: Duration::from_secs(10),
    };

    let js_wait = WaitCondition::Javascript {
        script: "return document.readyState === 'complete'".to_string(),
        timeout: Duration::from_secs(5),
    };

    let network_idle = WaitCondition::NetworkIdle {
        timeout: Duration::from_secs(30),
        idle_time: Duration::from_millis(500),
    };

    let dom_loaded = WaitCondition::DomContentLoaded;
    let load = WaitCondition::Load;
    let timeout = WaitCondition::Timeout(Duration::from_secs(3));

    let multiple = WaitCondition::Multiple(vec![dom_loaded.clone(), timeout.clone()]);

    // Verify all variants can be created
    assert!(matches!(selector_wait, WaitCondition::Selector { .. }));
    assert!(matches!(js_wait, WaitCondition::Javascript { .. }));
    assert!(matches!(network_idle, WaitCondition::NetworkIdle { .. }));
    assert!(matches!(dom_loaded, WaitCondition::DomContentLoaded));
    assert!(matches!(load, WaitCondition::Load));
    assert!(matches!(timeout, WaitCondition::Timeout(_)));
    assert!(matches!(multiple, WaitCondition::Multiple(_)));
}

#[test]
fn test_scroll_config_default() {
    let config = ScrollConfig::default();

    assert_eq!(config.steps, 3);
    assert!(config.step_px.is_none());
    assert_eq!(config.delay_ms, 1000);
    assert!(matches!(config.mode, ScrollMode::Stepped));
    assert!(config.after_scroll_js.is_none());
    assert!(config.stop_condition.is_none());
}

#[test]
fn test_scroll_config_custom() {
    let config = ScrollConfig {
        steps: 10,
        step_px: Some(800),
        delay_ms: 500,
        mode: ScrollMode::ToBottom,
        after_scroll_js: Some("console.log('scrolled')".to_string()),
        stop_condition: Some("document.body.scrollHeight === window.scrollY".to_string()),
    };

    assert_eq!(config.steps, 10);
    assert_eq!(config.step_px, Some(800));
    assert_eq!(config.delay_ms, 500);
    assert!(matches!(config.mode, ScrollMode::ToBottom));
    assert!(config.after_scroll_js.is_some());
    assert!(config.stop_condition.is_some());
}

#[test]
fn test_scroll_mode_variants() {
    let stepped = ScrollMode::Stepped;
    let to_bottom = ScrollMode::ToBottom;
    let smooth = ScrollMode::Smooth;
    let custom = ScrollMode::Custom("window.scrollBy(0, 100)".to_string());

    assert!(matches!(stepped, ScrollMode::Stepped));
    assert!(matches!(to_bottom, ScrollMode::ToBottom));
    assert!(matches!(smooth, ScrollMode::Smooth));
    assert!(matches!(custom, ScrollMode::Custom(_)));
}

#[test]
fn test_page_action_click() {
    let action = PageAction::Click {
        selector: "button.submit".to_string(),
        wait_after: Some(Duration::from_millis(500)),
    };

    match action {
        PageAction::Click {
            selector,
            wait_after,
        } => {
            assert_eq!(selector, "button.submit");
            assert_eq!(wait_after, Some(Duration::from_millis(500)));
        }
        _ => panic!("Expected Click action"),
    }
}

#[test]
fn test_page_action_type() {
    let action = PageAction::Type {
        selector: "input#email".to_string(),
        text: "user@example.com".to_string(),
        clear_first: true,
        wait_after: Some(Duration::from_millis(100)),
    };

    match action {
        PageAction::Type {
            selector,
            text,
            clear_first,
            wait_after,
        } => {
            assert_eq!(selector, "input#email");
            assert_eq!(text, "user@example.com");
            assert!(clear_first);
            assert_eq!(wait_after, Some(Duration::from_millis(100)));
        }
        _ => panic!("Expected Type action"),
    }
}

#[test]
fn test_page_action_evaluate() {
    let action = PageAction::Evaluate {
        script: "document.title".to_string(),
        wait_after: None,
    };

    match action {
        PageAction::Evaluate { script, wait_after } => {
            assert_eq!(script, "document.title");
            assert!(wait_after.is_none());
        }
        _ => panic!("Expected Evaluate action"),
    }
}

#[test]
fn test_page_action_screenshot() {
    let action = PageAction::Screenshot {
        filename: Some("screenshot.png".to_string()),
        full_page: true,
    };

    match action {
        PageAction::Screenshot {
            filename,
            full_page,
        } => {
            assert_eq!(filename, Some("screenshot.png".to_string()));
            assert!(full_page);
        }
        _ => panic!("Expected Screenshot action"),
    }
}

#[test]
fn test_page_action_navigate() {
    let action = PageAction::Navigate {
        url: "https://example.com".to_string(),
        wait_for_load: true,
    };

    match action {
        PageAction::Navigate { url, wait_for_load } => {
            assert_eq!(url, "https://example.com");
            assert!(wait_for_load);
        }
        _ => panic!("Expected Navigate action"),
    }
}

#[test]
fn test_page_action_set_cookies() {
    let mut cookies = HashMap::new();
    cookies.insert("session_id".to_string(), "abc123".to_string());
    cookies.insert("user_pref".to_string(), "dark_mode".to_string());

    let action = PageAction::SetCookies {
        cookies: cookies.clone(),
    };

    match action {
        PageAction::SetCookies { cookies: c } => {
            assert_eq!(c.len(), 2);
            assert_eq!(c.get("session_id"), Some(&"abc123".to_string()));
            assert_eq!(c.get("user_pref"), Some(&"dark_mode".to_string()));
        }
        _ => panic!("Expected SetCookies action"),
    }
}

#[test]
fn test_page_action_hover() {
    let action = PageAction::Hover {
        selector: ".menu-item".to_string(),
        wait_after: Some(Duration::from_millis(200)),
    };

    match action {
        PageAction::Hover {
            selector,
            wait_after,
        } => {
            assert_eq!(selector, ".menu-item");
            assert_eq!(wait_after, Some(Duration::from_millis(200)));
        }
        _ => panic!("Expected Hover action"),
    }
}

#[test]
fn test_viewport_config_default() {
    let config = ViewportConfig::default();

    assert_eq!(config.width, 1920);
    assert_eq!(config.height, 1080);
    assert_eq!(config.device_scale_factor, 1.0);
    assert!(!config.is_mobile);
    assert!(config.user_agent.is_none());
}

#[test]
fn test_viewport_config_mobile() {
    let config = ViewportConfig {
        width: 375,
        height: 812,
        device_scale_factor: 2.0,
        is_mobile: true,
        user_agent: Some("Mozilla/5.0 (iPhone; CPU iPhone OS 14_0)".to_string()),
    };

    assert_eq!(config.width, 375);
    assert_eq!(config.height, 812);
    assert_eq!(config.device_scale_factor, 2.0);
    assert!(config.is_mobile);
    assert!(config.user_agent.is_some());
}

#[test]
fn test_dynamic_capabilities() {
    let capabilities = DynamicCapabilities {
        wait_conditions: vec![
            "selector".to_string(),
            "javascript".to_string(),
            "network_idle".to_string(),
        ],
        actions: vec![
            "click".to_string(),
            "type".to_string(),
            "scroll".to_string(),
        ],
        screenshots: true,
        mhtml_capture: true,
        max_timeout_seconds: 120,
        viewport_sizes: vec![(1920, 1080), (1366, 768), (375, 812)],
    };

    assert_eq!(capabilities.wait_conditions.len(), 3);
    assert_eq!(capabilities.actions.len(), 3);
    assert!(capabilities.screenshots);
    assert!(capabilities.mhtml_capture);
    assert_eq!(capabilities.max_timeout_seconds, 120);
    assert_eq!(capabilities.viewport_sizes.len(), 3);
}

#[test]
fn test_dynamic_error_timeout() {
    let error = DynamicError::Timeout {
        condition: "selector: .loading".to_string(),
        waited_ms: 30000,
    };

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Timeout"));
    assert!(error_msg.contains(".loading"));
    assert!(error_msg.contains("30000ms"));
}

#[test]
fn test_dynamic_error_element_not_found() {
    let error = DynamicError::ElementNotFound {
        selector: "#missing-element".to_string(),
    };

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Element not found"));
    assert!(error_msg.contains("#missing-element"));
}

#[test]
fn test_dynamic_error_javascript_error() {
    let error = DynamicError::JavascriptError {
        script: "invalid.syntax()".to_string(),
        error: "ReferenceError: invalid is not defined".to_string(),
    };

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("JavaScript error"));
    assert!(error_msg.contains("invalid.syntax()"));
    assert!(error_msg.contains("ReferenceError"));
}

#[test]
fn test_dynamic_error_navigation_error() {
    let error = DynamicError::NavigationError {
        url: "https://invalid.example".to_string(),
        error: "DNS resolution failed".to_string(),
    };

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Navigation"));
    assert!(error_msg.contains("invalid.example"));
    assert!(error_msg.contains("DNS resolution"));
}

#[test]
fn test_dynamic_error_renderer_error() {
    let error = DynamicError::RendererError {
        error: "Browser crashed".to_string(),
    };

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Renderer error"));
    assert!(error_msg.contains("Browser crashed"));
}

#[test]
fn test_dynamic_error_config_error() {
    let error = DynamicError::ConfigError {
        message: "Invalid timeout value".to_string(),
    };

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Configuration error"));
    assert!(error_msg.contains("Invalid timeout value"));
}

#[test]
fn test_dynamic_error_network_error() {
    let error = DynamicError::NetworkError {
        error: "Connection refused".to_string(),
    };

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Network error"));
    assert!(error_msg.contains("Connection refused"));
}

#[test]
fn test_dynamic_error_resource_limit() {
    let error = DynamicError::ResourceLimit {
        limit: "memory".to_string(),
        value: 2048,
    };

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Resource limit exceeded"));
    assert!(error_msg.contains("memory"));
    assert!(error_msg.contains("2048"));
}

#[test]
fn test_wait_condition_serialization() {
    use serde_json;

    let condition = WaitCondition::Selector {
        selector: ".content".to_string(),
        timeout: Duration::from_secs(10),
    };

    let json = serde_json::to_string(&condition).unwrap();
    let deserialized: WaitCondition = serde_json::from_str(&json).unwrap();

    match deserialized {
        WaitCondition::Selector { selector, timeout } => {
            assert_eq!(selector, ".content");
            assert_eq!(timeout, Duration::from_secs(10));
        }
        _ => panic!("Unexpected deserialized type"),
    }
}

#[test]
fn test_complex_dynamic_config() {
    let config = DynamicConfig {
        wait_for: Some(WaitCondition::Multiple(vec![
            WaitCondition::DomContentLoaded,
            WaitCondition::NetworkIdle {
                timeout: Duration::from_secs(10),
                idle_time: Duration::from_millis(500),
            },
        ])),
        scroll: Some(ScrollConfig {
            steps: 5,
            step_px: Some(600),
            delay_ms: 300,
            mode: ScrollMode::Smooth,
            after_scroll_js: Some("loadMoreContent()".to_string()),
            stop_condition: Some("noMoreContent()".to_string()),
        }),
        actions: vec![
            PageAction::Wait(WaitCondition::Selector {
                selector: ".page-loaded".to_string(),
                timeout: Duration::from_secs(5),
            }),
            PageAction::Click {
                selector: "button.load-more".to_string(),
                wait_after: Some(Duration::from_millis(500)),
            },
            PageAction::Screenshot {
                filename: Some("after-click.png".to_string()),
                full_page: true,
            },
        ],
        capture_artifacts: true,
        timeout: Duration::from_secs(90),
        viewport: Some(ViewportConfig {
            width: 1366,
            height: 768,
            device_scale_factor: 1.0,
            is_mobile: false,
            user_agent: None,
        }),
    };

    assert!(config.wait_for.is_some());
    assert!(config.scroll.is_some());
    assert_eq!(config.actions.len(), 3);
    assert!(config.capture_artifacts);
    assert_eq!(config.timeout, Duration::from_secs(90));
    assert!(config.viewport.is_some());
}
