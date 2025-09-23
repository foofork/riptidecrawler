use riptide_core::dynamic::{
    DynamicConfig, WaitCondition, ScrollConfig, ScrollMode, PageAction, ViewportConfig,
    DynamicRenderResult, RenderArtifacts, PageMetadata, ConsoleMessage, NetworkRequest,
    DynamicCapabilities, DynamicError, PageTiming
};
use std::time::Duration;
use std::collections::HashMap;

#[tokio::test]
async fn test_dynamic_config_creation() {
    let config = DynamicConfig {
        wait_for: Some(WaitCondition::Selector {
            selector: ".content".to_string(),
            timeout: Duration::from_secs(10),
        }),
        scroll: Some(ScrollConfig {
            steps: 5,
            step_px: Some(1000),
            delay_ms: 1500,
            mode: ScrollMode::Stepped,
            after_scroll_js: Some("window.triggerLazyLoad()".to_string()),
            stop_condition: Some("document.querySelector('.end-of-content')".to_string()),
        }),
        actions: vec![
            PageAction::Wait(WaitCondition::DomContentLoaded),
            PageAction::Click {
                selector: ".show-more".to_string(),
                wait_after: Some(Duration::from_millis(2000)),
            },
            PageAction::Type {
                selector: "input[name='search']".to_string(),
                text: "test query".to_string(),
                clear_first: true,
                wait_after: Some(Duration::from_millis(1000)),
            },
            PageAction::Evaluate {
                script: "document.querySelector('.content').scrollIntoView()".to_string(),
                wait_after: Some(Duration::from_millis(500)),
            },
        ],
        capture_artifacts: true,
        timeout: Duration::from_secs(30),
        viewport: Some(ViewportConfig {
            width: 1920,
            height: 1080,
            device_scale_factor: 2.0,
            is_mobile: false,
            user_agent: Some("Custom/1.0".to_string()),
        }),
    };

    assert!(config.capture_artifacts);
    assert_eq!(config.timeout, Duration::from_secs(30));
    assert_eq!(config.actions.len(), 4);

    if let Some(WaitCondition::Selector { selector, timeout }) = &config.wait_for {
        assert_eq!(selector, ".content");
        assert_eq!(*timeout, Duration::from_secs(10));
    } else {
        panic!("Expected Selector wait condition");
    }

    if let Some(scroll) = &config.scroll {
        assert_eq!(scroll.steps, 5);
        assert_eq!(scroll.step_px, Some(1000));
        assert!(matches!(scroll.mode, ScrollMode::Stepped));
    } else {
        panic!("Expected scroll configuration");
    }
}

#[tokio::test]
async fn test_wait_condition_variants() {
    let conditions = vec![
        WaitCondition::Selector {
            selector: ".loading".to_string(),
            timeout: Duration::from_secs(5),
        },
        WaitCondition::Javascript {
            script: "window.dataLoaded === true".to_string(),
            timeout: Duration::from_secs(10),
        },
        WaitCondition::NetworkIdle {
            timeout: Duration::from_secs(15),
            idle_time: Duration::from_secs(2),
        },
        WaitCondition::DomContentLoaded,
        WaitCondition::Load,
        WaitCondition::Timeout(Duration::from_secs(5)),
        WaitCondition::Multiple(vec![
            WaitCondition::DomContentLoaded,
            WaitCondition::Selector {
                selector: ".ready".to_string(),
                timeout: Duration::from_secs(3),
            },
        ]),
    ];

    // Test serialization/deserialization
    for condition in conditions {
        let json = serde_json::to_string(&condition).expect("Should serialize");
        let deserialized: WaitCondition = serde_json::from_str(&json).expect("Should deserialize");

        match (&condition, &deserialized) {
            (WaitCondition::Selector { selector: s1, .. }, WaitCondition::Selector { selector: s2, .. }) => {
                assert_eq!(s1, s2);
            }
            (WaitCondition::Javascript { script: s1, .. }, WaitCondition::Javascript { script: s2, .. }) => {
                assert_eq!(s1, s2);
            }
            (WaitCondition::DomContentLoaded, WaitCondition::DomContentLoaded) => {},
            (WaitCondition::Load, WaitCondition::Load) => {},
            _ => {
                // For other variants, ensure they match
                assert!(std::mem::discriminant(&condition) == std::mem::discriminant(&deserialized));
            }
        }
    }
}

#[tokio::test]
async fn test_page_actions() {
    let actions = vec![
        PageAction::Click {
            selector: "button.submit".to_string(),
            wait_after: Some(Duration::from_millis(1000)),
        },
        PageAction::Type {
            selector: "input[type='email']".to_string(),
            text: "test@example.com".to_string(),
            clear_first: true,
            wait_after: Some(Duration::from_millis(500)),
        },
        PageAction::Evaluate {
            script: "window.scrollTo(0, document.body.scrollHeight)".to_string(),
            wait_after: Some(Duration::from_millis(2000)),
        },
        PageAction::Screenshot {
            filename: Some("test-screenshot.png".to_string()),
            full_page: true,
        },
        PageAction::Navigate {
            url: "https://example.com/page2".to_string(),
            wait_for_load: true,
        },
        PageAction::SetCookies {
            cookies: {
                let mut cookies = HashMap::new();
                cookies.insert("session_id".to_string(), "12345".to_string());
                cookies.insert("user_pref".to_string(), "dark_mode".to_string());
                cookies
            },
        },
        PageAction::Hover {
            selector: ".tooltip-trigger".to_string(),
            wait_after: Some(Duration::from_millis(300)),
        },
    ];

    // Test each action type
    for action in &actions {
        let json = serde_json::to_string(action).expect("Should serialize");
        let deserialized: PageAction = serde_json::from_str(&json).expect("Should deserialize");

        match (action, &deserialized) {
            (PageAction::Click { selector: s1, .. }, PageAction::Click { selector: s2, .. }) => {
                assert_eq!(s1, s2);
            }
            (PageAction::Type { selector: s1, text: t1, .. }, PageAction::Type { selector: s2, text: t2, .. }) => {
                assert_eq!(s1, s2);
                assert_eq!(t1, t2);
            }
            (PageAction::Screenshot { filename: f1, .. }, PageAction::Screenshot { filename: f2, .. }) => {
                assert_eq!(f1, f2);
            }
            _ => {
                assert!(std::mem::discriminant(action) == std::mem::discriminant(&deserialized));
            }
        }
    }
}

#[tokio::test]
async fn test_render_artifacts() {
    let artifacts = RenderArtifacts {
        screenshot: Some("iVBORw0KGgoAAAANS...".to_string()), // Base64 encoded image
        mhtml: Some("MIME-Version: 1.0...".to_string()),
        metadata: PageMetadata {
            title: Some("Test Page".to_string()),
            description: Some("A test page for dynamic rendering".to_string()),
            og_tags: {
                let mut tags = HashMap::new();
                tags.insert("og:title".to_string(), "Test Page".to_string());
                tags.insert("og:description".to_string(), "Test description".to_string());
                tags
            },
            twitter_tags: {
                let mut tags = HashMap::new();
                tags.insert("twitter:card".to_string(), "summary".to_string());
                tags
            },
            json_ld: vec![
                serde_json::json!({
                    "@context": "https://schema.org",
                    "@type": "Article",
                    "headline": "Test Article"
                })
            ],
            final_url: "https://example.com/final".to_string(),
            headers: {
                let mut headers = HashMap::new();
                headers.insert("content-type".to_string(), "text/html".to_string());
                headers.insert("x-custom".to_string(), "test-value".to_string());
                headers
            },
            timing: Some(PageTiming {
                ttfb_ms: 150,
                dom_content_loaded_ms: 800,
                load_ms: 1200,
                first_contentful_paint_ms: Some(600),
                largest_contentful_paint_ms: Some(1000),
            }),
        },
        console_logs: vec![
            ConsoleMessage {
                level: "info".to_string(),
                text: "Page loaded successfully".to_string(),
                timestamp: "2024-01-15T10:30:00Z".to_string(),
                source: Some("https://example.com/script.js:42".to_string()),
            },
            ConsoleMessage {
                level: "warn".to_string(),
                text: "Deprecated API usage".to_string(),
                timestamp: "2024-01-15T10:30:05Z".to_string(),
                source: None,
            },
        ],
        network_activity: vec![
            NetworkRequest {
                url: "https://api.example.com/data".to_string(),
                method: "GET".to_string(),
                status: 200,
                content_type: Some("application/json".to_string()),
                size: 1024,
                duration_ms: 250,
            },
            NetworkRequest {
                url: "https://cdn.example.com/image.jpg".to_string(),
                method: "GET".to_string(),
                status: 200,
                content_type: Some("image/jpeg".to_string()),
                size: 65536,
                duration_ms: 500,
            },
        ],
    };

    assert!(artifacts.screenshot.is_some());
    assert!(artifacts.mhtml.is_some());
    assert_eq!(artifacts.metadata.title, Some("Test Page".to_string()));
    assert_eq!(artifacts.console_logs.len(), 2);
    assert_eq!(artifacts.network_activity.len(), 2);

    // Test timing information
    if let Some(timing) = &artifacts.metadata.timing {
        assert_eq!(timing.ttfb_ms, 150);
        assert_eq!(timing.dom_content_loaded_ms, 800);
        assert_eq!(timing.first_contentful_paint_ms, Some(600));
    }

    // Test serialization
    let json = serde_json::to_string(&artifacts).expect("Should serialize");
    let deserialized: RenderArtifacts = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(artifacts.metadata.title, deserialized.metadata.title);
    assert_eq!(artifacts.console_logs.len(), deserialized.console_logs.len());
    assert_eq!(artifacts.network_activity.len(), deserialized.network_activity.len());
}

#[tokio::test]
async fn test_dynamic_render_result() {
    let result = DynamicRenderResult {
        success: true,
        html: "<html><body><h1>Dynamic Content</h1></body></html>".to_string(),
        artifacts: Some(RenderArtifacts {
            screenshot: None,
            mhtml: None,
            metadata: PageMetadata {
                title: Some("Dynamic Page".to_string()),
                description: None,
                og_tags: HashMap::new(),
                twitter_tags: HashMap::new(),
                json_ld: Vec::new(),
                final_url: "https://example.com".to_string(),
                headers: HashMap::new(),
                timing: None,
            },
            console_logs: Vec::new(),
            network_activity: Vec::new(),
        }),
        error: None,
        render_time_ms: 2500,
        actions_executed: vec![
            "wait_for_dom_content_loaded".to_string(),
            "click_.show-more".to_string(),
            "scroll_to_bottom".to_string(),
        ],
        wait_conditions_met: vec![
            "dom_content_loaded".to_string(),
            "selector_.content".to_string(),
        ],
    };

    assert!(result.success);
    assert_eq!(result.render_time_ms, 2500);
    assert_eq!(result.actions_executed.len(), 3);
    assert_eq!(result.wait_conditions_met.len(), 2);
    assert!(result.error.is_none());
    assert!(result.artifacts.is_some());

    // Test serialization
    let json = serde_json::to_string(&result).expect("Should serialize");
    let deserialized: DynamicRenderResult = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(result.success, deserialized.success);
    assert_eq!(result.render_time_ms, deserialized.render_time_ms);
    assert_eq!(result.actions_executed, deserialized.actions_executed);
}

#[tokio::test]
async fn test_dynamic_error_types() {
    let errors = vec![
        DynamicError::Timeout {
            condition: "selector .loading".to_string(),
            waited_ms: 10000,
        },
        DynamicError::ElementNotFound {
            selector: "#missing-element".to_string(),
        },
        DynamicError::JavascriptError {
            script: "window.nonExistentFunction()".to_string(),
            error: "TypeError: window.nonExistentFunction is not a function".to_string(),
        },
        DynamicError::NavigationError {
            url: "https://invalid-domain.example".to_string(),
            error: "DNS resolution failed".to_string(),
        },
        DynamicError::RendererError {
            error: "Browser crashed".to_string(),
        },
        DynamicError::ConfigError {
            message: "Invalid timeout value".to_string(),
        },
        DynamicError::NetworkError {
            error: "Connection timeout".to_string(),
        },
        DynamicError::ResourceLimit {
            limit: "memory".to_string(),
            value: 2048,
        },
    ];

    for error in errors {
        // Test display formatting
        let error_string = error.to_string();
        assert!(!error_string.is_empty());

        // Test that it implements Error trait
        let _: &dyn std::error::Error = &error;

        // Test serialization
        let json = serde_json::to_string(&error).expect("Should serialize");
        let deserialized: DynamicError = serde_json::from_str(&json).expect("Should deserialize");

        assert!(std::mem::discriminant(&error) == std::mem::discriminant(&deserialized));
    }
}

#[tokio::test]
async fn test_dynamic_capabilities() {
    let capabilities = DynamicCapabilities {
        wait_conditions: vec![
            "selector".to_string(),
            "javascript".to_string(),
            "network_idle".to_string(),
            "dom_content_loaded".to_string(),
            "load".to_string(),
        ],
        actions: vec![
            "click".to_string(),
            "type".to_string(),
            "evaluate".to_string(),
            "screenshot".to_string(),
            "navigate".to_string(),
            "hover".to_string(),
        ],
        screenshots: true,
        mhtml_capture: true,
        max_timeout_seconds: 300,
        viewport_sizes: vec![
            (1920, 1080),
            (1366, 768),
            (390, 844), // iPhone 12
            (412, 915), // Pixel 5
        ],
    };

    assert!(capabilities.screenshots);
    assert!(capabilities.mhtml_capture);
    assert_eq!(capabilities.max_timeout_seconds, 300);
    assert_eq!(capabilities.wait_conditions.len(), 5);
    assert_eq!(capabilities.actions.len(), 6);
    assert_eq!(capabilities.viewport_sizes.len(), 4);

    // Test serialization
    let json = serde_json::to_string(&capabilities).expect("Should serialize");
    let deserialized: DynamicCapabilities = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(capabilities.screenshots, deserialized.screenshots);
    assert_eq!(capabilities.wait_conditions, deserialized.wait_conditions);
    assert_eq!(capabilities.viewport_sizes, deserialized.viewport_sizes);
}

#[tokio::test]
async fn test_scroll_config_modes() {
    let modes = vec![
        ScrollMode::Stepped,
        ScrollMode::ToBottom,
        ScrollMode::Smooth,
        ScrollMode::Custom("window.customScroll()".to_string()),
    ];

    for mode in modes {
        let config = ScrollConfig {
            steps: 3,
            step_px: Some(800),
            delay_ms: 1000,
            mode: mode.clone(),
            after_scroll_js: None,
            stop_condition: None,
        };

        let json = serde_json::to_string(&config).expect("Should serialize");
        let deserialized: ScrollConfig = serde_json::from_str(&json).expect("Should deserialize");

        assert!(std::mem::discriminant(&mode) == std::mem::discriminant(&deserialized.mode));
    }
}

#[tokio::test]
async fn test_viewport_config_variants() {
    let configs = vec![
        ViewportConfig {
            width: 1920,
            height: 1080,
            device_scale_factor: 1.0,
            is_mobile: false,
            user_agent: None,
        },
        ViewportConfig {
            width: 390,
            height: 844,
            device_scale_factor: 3.0,
            is_mobile: true,
            user_agent: Some("Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X)".to_string()),
        },
        ViewportConfig {
            width: 1366,
            height: 768,
            device_scale_factor: 1.0,
            is_mobile: false,
            user_agent: Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string()),
        },
    ];

    for config in configs {
        assert!(config.width > 0);
        assert!(config.height > 0);
        assert!(config.device_scale_factor > 0.0);

        let json = serde_json::to_string(&config).expect("Should serialize");
        let deserialized: ViewportConfig = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(config.width, deserialized.width);
        assert_eq!(config.height, deserialized.height);
        assert_eq!(config.is_mobile, deserialized.is_mobile);
        assert_eq!(config.user_agent, deserialized.user_agent);
    }
}

#[tokio::test]
async fn test_complex_wait_conditions() {
    let complex_condition = WaitCondition::Multiple(vec![
        WaitCondition::DomContentLoaded,
        WaitCondition::Selector {
            selector: ".content-loaded".to_string(),
            timeout: Duration::from_secs(5),
        },
        WaitCondition::Javascript {
            script: "window.apiDataLoaded && window.imagesLoaded".to_string(),
            timeout: Duration::from_secs(10),
        },
        WaitCondition::NetworkIdle {
            timeout: Duration::from_secs(15),
            idle_time: Duration::from_secs(2),
        },
    ]);

    if let WaitCondition::Multiple(conditions) = &complex_condition {
        assert_eq!(conditions.len(), 4);

        // Verify each condition type
        assert!(matches!(conditions[0], WaitCondition::DomContentLoaded));
        assert!(matches!(conditions[1], WaitCondition::Selector { .. }));
        assert!(matches!(conditions[2], WaitCondition::Javascript { .. }));
        assert!(matches!(conditions[3], WaitCondition::NetworkIdle { .. }));
    } else {
        panic!("Expected Multiple wait condition");
    }

    // Test serialization of complex nested structure
    let json = serde_json::to_string(&complex_condition).expect("Should serialize");
    let deserialized: WaitCondition = serde_json::from_str(&json).expect("Should deserialize");

    if let WaitCondition::Multiple(deserialized_conditions) = deserialized {
        assert_eq!(deserialized_conditions.len(), 4);
    } else {
        panic!("Deserialized condition should be Multiple");
    }
}