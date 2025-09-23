use riptide_core::stealth::StealthConfig;
use serde::{Deserialize, Serialize};

/// Enhanced render request with Phase 3 PR-1 features
#[derive(Deserialize, Clone, Debug)]
pub struct RenderReq {
    /// Target URL to render
    pub url: String,

    /// Legacy: CSS selector to wait for (backward compatibility)
    pub wait_for: Option<String>,

    /// Legacy: Number of scroll steps (backward compatibility)
    pub scroll_steps: Option<u32>,

    // NEW Phase 3 PR-1 fields:
    /// Session ID for persistent browser sessions
    pub session_id: Option<String>,

    /// Interactive page actions to execute
    pub actions: Option<Vec<PageAction>>,

    /// Timeout configurations
    pub timeouts: Option<Timeouts>,

    /// Artifacts to capture
    pub artifacts: Option<Artifacts>,

    /// Stealth configuration for anti-detection
    pub stealth_config: Option<StealthConfig>,
}

/// Timeout configurations for rendering
#[derive(Deserialize, Clone, Debug)]
pub struct Timeouts {
    /// Navigation timeout in milliseconds
    pub nav_ms: Option<u64>,

    /// Idle time after DOMContentLoaded
    pub idle_after_dcl_ms: Option<u64>,

    /// Hard cap timeout for entire render
    pub hard_cap_ms: Option<u64>,
}

/// Page actions that can be executed
#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PageAction {
    /// Wait for CSS selector to appear
    WaitForCss {
        css: String,
        timeout_ms: Option<u64>,
    },

    /// Wait for JavaScript expression to return true
    WaitForJs {
        expr: String,
        timeout_ms: Option<u64>,
    },

    /// Scroll the page
    Scroll {
        steps: u32,
        step_px: u32,
        delay_ms: u64,
    },

    /// Execute JavaScript code
    Js { code: String },

    /// Click an element
    Click { css: String },

    /// Type text into an element
    Type {
        css: String,
        text: String,
        delay_ms: Option<u64>,
    },
}

/// Artifacts configuration
#[derive(Deserialize, Clone, Debug, Default)]
pub struct Artifacts {
    /// Capture screenshot
    pub screenshot: bool,

    /// Capture MHTML archive
    pub mhtml: bool,
}

/// Enhanced render response
#[derive(Serialize)]
pub struct RenderResp {
    /// Final URL after redirects
    pub final_url: String,

    /// Rendered HTML content
    pub html: String,

    /// Legacy: Screenshot base64 (for backward compatibility)
    pub screenshot_b64: Option<String>,

    // NEW Phase 3 PR-1 fields:
    /// Session ID for reuse
    pub session_id: Option<String>,

    /// Captured artifacts
    pub artifacts: ArtifactsOut,
}

/// Output artifacts
#[derive(Serialize, Default)]
pub struct ArtifactsOut {
    /// Screenshot as base64
    pub screenshot_b64: Option<String>,

    /// MHTML archive as base64
    pub mhtml_b64: Option<String>,
}

#[derive(Serialize)]
pub struct RenderErrorResp {
    pub error: String,
    pub request_id: Option<String>,
    pub duration_ms: u64,
}
