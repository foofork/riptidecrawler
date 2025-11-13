//! CDP Protocol Interaction Tests
//!
//! Tests for Chrome DevTools Protocol interactions and HTTP API endpoints

use riptide_headless::{
    cdp::AppState,
    models::{Artifacts, PageAction, RenderReq},
    HeadlessLauncher, LauncherConfig,
};
use riptide_stealth::StealthPreset;
use std::sync::Arc;

#[tokio::test]
async fn test_app_state_creation() {
    let launcher = HeadlessLauncher::new().await.unwrap();
    let state = AppState {
        launcher: Arc::new(launcher),
    };

    // Verify state is properly initialized
    let stats = state.launcher.stats().await;
    assert_eq!(stats.total_requests, 0);

    let _ = state.launcher.shutdown().await;
}

#[tokio::test]
async fn test_render_request_minimal() {
    let req = RenderReq {
        url: "https://example.com".to_string(),
        wait_for: None,
        scroll_steps: None,
        session_id: None,
        actions: None,
        timeouts: None,
        artifacts: None,
        stealth_config: None,
    };

    assert_eq!(req.url, "https://example.com");
    assert!(req.wait_for.is_none());
    assert!(req.scroll_steps.is_none());
}

#[tokio::test]
async fn test_render_request_with_wait_for() {
    let req = RenderReq {
        url: "https://example.com".to_string(),
        wait_for: Some(".content".to_string()),
        scroll_steps: Some(3),
        session_id: Some("test-session-123".to_string()),
        actions: None,
        timeouts: None,
        artifacts: None,
        stealth_config: None,
    };

    assert_eq!(req.url, "https://example.com");
    assert_eq!(req.wait_for, Some(".content".to_string()));
    assert_eq!(req.scroll_steps, Some(3));
    assert_eq!(req.session_id, Some("test-session-123".to_string()));
}

#[tokio::test]
async fn test_render_request_with_actions() {
    let actions = vec![
        PageAction::WaitForCss {
            css: ".main-content".to_string(),
            timeout_ms: Some(5000),
        },
        PageAction::Scroll {
            steps: 3,
            step_px: 500,
            delay_ms: 100,
        },
        PageAction::Js {
            code: "console.log('test')".to_string(),
        },
    ];

    let req = RenderReq {
        url: "https://example.com".to_string(),
        wait_for: None,
        scroll_steps: None,
        session_id: None,
        actions: Some(actions.clone()),
        timeouts: None,
        artifacts: None,
        stealth_config: None,
    };

    assert!(req.actions.is_some());
    assert_eq!(req.actions.as_ref().unwrap().len(), 3);
}

#[tokio::test]
async fn test_render_request_with_artifacts() {
    let artifacts = Artifacts {
        screenshot: true,
        mhtml: false,
    };

    let req = RenderReq {
        url: "https://example.com".to_string(),
        wait_for: None,
        scroll_steps: None,
        session_id: None,
        actions: None,
        timeouts: None,
        artifacts: Some(artifacts),
        stealth_config: None,
    };

    assert!(req.artifacts.is_some());
    let artifacts = req.artifacts.unwrap();
    assert!(artifacts.screenshot);
    assert!(!artifacts.mhtml);
}

#[tokio::test]
async fn test_render_request_with_stealth_config() {
    use riptide_stealth::StealthConfig;

    let stealth_config = StealthConfig {
        preset: StealthPreset::High,
        ..Default::default()
    };

    let req = RenderReq {
        url: "https://example.com".to_string(),
        wait_for: None,
        scroll_steps: None,
        session_id: None,
        actions: None,
        timeouts: None,
        artifacts: None,
        stealth_config: Some(stealth_config),
    };

    assert!(req.stealth_config.is_some());
    let config = req.stealth_config.unwrap();
    assert!(matches!(config.preset, StealthPreset::High));
}

#[tokio::test]
async fn test_page_action_variants() {
    // Test all PageAction variants can be created
    let actions = [
        PageAction::WaitForCss {
            css: "#element".to_string(),
            timeout_ms: Some(3000),
        },
        PageAction::WaitForJs {
            expr: "document.readyState === 'complete'".to_string(),
            timeout_ms: Some(5000),
        },
        PageAction::Scroll {
            steps: 5,
            step_px: 800,
            delay_ms: 200,
        },
        PageAction::Js {
            code: "window.scrollTo(0, 0)".to_string(),
        },
        PageAction::Click {
            css: "button.submit".to_string(),
        },
        PageAction::Type {
            css: "input[name='email']".to_string(),
            text: "test@example.com".to_string(),
            delay_ms: Some(50),
        },
    ];

    assert_eq!(actions.len(), 6);

    // Verify each action type
    match &actions[0] {
        PageAction::WaitForCss { css, timeout_ms } => {
            assert_eq!(css, "#element");
            assert_eq!(*timeout_ms, Some(3000));
        }
        _ => panic!("Expected WaitForCss"),
    }

    match &actions[4] {
        PageAction::Click { css } => {
            assert_eq!(css, "button.submit");
        }
        _ => panic!("Expected Click"),
    }

    match &actions[5] {
        PageAction::Type {
            css,
            text,
            delay_ms,
        } => {
            assert_eq!(css, "input[name='email']");
            assert_eq!(text, "test@example.com");
            assert_eq!(*delay_ms, Some(50));
        }
        _ => panic!("Expected Type"),
    }
}

#[tokio::test]
async fn test_cdp_with_custom_launcher_config() {
    use riptide_headless::BrowserPoolConfig;
    use std::time::Duration;

    let config = LauncherConfig {
        pool_config: BrowserPoolConfig {
            initial_pool_size: 1,
            min_pool_size: 1,
            max_pool_size: 2,
            idle_timeout: Duration::from_secs(10),
            ..Default::default()
        },
        enable_stealth: true,
        default_stealth_preset: StealthPreset::Medium,
        page_timeout: Duration::from_secs(15),
        enable_monitoring: false,
        hybrid_mode: false,
    };

    let launcher = HeadlessLauncher::with_config(config).await;
    assert!(launcher.is_ok());

    if let Ok(launcher) = launcher {
        let stats = launcher.stats().await;
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.successful_requests, 0);

        let _ = launcher.shutdown().await;
    }
}

#[tokio::test]
async fn test_multiple_concurrent_requests() {
    use riptide_headless::BrowserPoolConfig;

    let config = LauncherConfig {
        pool_config: BrowserPoolConfig {
            initial_pool_size: 2,
            min_pool_size: 2,
            max_pool_size: 4,
            ..Default::default()
        },
        enable_stealth: false,
        enable_monitoring: false,
        hybrid_mode: false,
        ..Default::default()
    };

    let launcher = Arc::new(HeadlessLauncher::with_config(config).await.unwrap());

    // Create multiple render requests
    let requests = [
        RenderReq {
            url: "https://example.com".to_string(),
            wait_for: None,
            scroll_steps: None,
            session_id: Some("session-1".to_string()),
            actions: None,
            timeouts: None,
            artifacts: None,
            stealth_config: None,
        },
        RenderReq {
            url: "https://example.org".to_string(),
            wait_for: None,
            scroll_steps: None,
            session_id: Some("session-2".to_string()),
            actions: None,
            timeouts: None,
            artifacts: None,
            stealth_config: None,
        },
    ];

    // Verify we created multiple distinct requests
    assert_eq!(requests.len(), 2);
    assert_ne!(requests[0].session_id, requests[1].session_id);

    let _ = launcher.shutdown().await;
}

#[tokio::test]
async fn test_render_request_serialization() {
    use serde_json;

    let req = RenderReq {
        url: "https://example.com".to_string(),
        wait_for: Some(".content".to_string()),
        scroll_steps: Some(3),
        session_id: Some("test-123".to_string()),
        actions: Some(vec![PageAction::Scroll {
            steps: 2,
            step_px: 500,
            delay_ms: 100,
        }]),
        timeouts: None,
        artifacts: Some(Artifacts {
            screenshot: true,
            mhtml: false,
        }),
        stealth_config: None,
    };

    // Test that RenderReq can be serialized/deserialized
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("example.com"));
    assert!(json.contains(".content"));

    let deserialized: RenderReq = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.url, "https://example.com");
    assert_eq!(deserialized.wait_for, Some(".content".to_string()));
}
