use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Dynamic content handling configuration for crawling complex web applications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicConfig {
    /// Wait conditions before extracting content
    pub wait_for: Option<WaitCondition>,

    /// Scroll configuration for infinite scroll pages
    pub scroll: Option<ScrollConfig>,

    /// Actions to perform on the page
    pub actions: Vec<PageAction>,

    /// Whether to capture artifacts (screenshots, MHTML)
    pub capture_artifacts: bool,

    /// Timeout for dynamic operations (default: 30s)
    pub timeout: Duration,

    /// Viewport configuration
    pub viewport: Option<ViewportConfig>,
}

impl Default for DynamicConfig {
    fn default() -> Self {
        Self {
            wait_for: None,
            scroll: None,
            actions: Vec::new(),
            capture_artifacts: false,
            timeout: Duration::from_secs(30),
            viewport: None,
        }
    }
}

/// Wait conditions for dynamic content loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WaitCondition {
    /// Wait for a CSS selector to be present
    Selector { selector: String, timeout: Duration },

    /// Wait for custom JavaScript to return true
    Javascript { script: String, timeout: Duration },

    /// Wait for network activity to cease
    NetworkIdle {
        timeout: Duration,
        idle_time: Duration,
    },

    /// Wait for page load event
    DomContentLoaded,

    /// Wait for all resources to load
    Load,

    /// Wait for a specific timeout
    Timeout(Duration),

    /// Combine multiple wait conditions
    Multiple(Vec<WaitCondition>),
}

/// Scroll configuration for infinite scroll and lazy loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollConfig {
    /// Number of scroll steps to perform
    pub steps: u32,

    /// Pixels to scroll per step (default: viewport height)
    pub step_px: Option<u32>,

    /// Delay between scroll steps
    pub delay_ms: u64,

    /// Whether to scroll to bottom or use stepped scrolling
    pub mode: ScrollMode,

    /// JavaScript to execute after each scroll
    pub after_scroll_js: Option<String>,

    /// Stop scrolling when this condition is met
    pub stop_condition: Option<String>,
}

impl Default for ScrollConfig {
    fn default() -> Self {
        Self {
            steps: 3,
            step_px: None,
            delay_ms: 1000,
            mode: ScrollMode::Stepped,
            after_scroll_js: None,
            stop_condition: None,
        }
    }
}

/// Scroll behavior modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScrollMode {
    /// Scroll step by step with delays
    Stepped,

    /// Scroll to the very bottom of the page
    ToBottom,

    /// Smooth scroll with animation
    Smooth,

    /// Custom scroll with JavaScript
    Custom(String),
}

/// Actions to perform on the page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PageAction {
    /// Click on an element
    Click {
        selector: String,
        wait_after: Option<Duration>,
    },

    /// Type text into an input field
    Type {
        selector: String,
        text: String,
        clear_first: bool,
        wait_after: Option<Duration>,
    },

    /// Execute arbitrary JavaScript
    Evaluate {
        script: String,
        wait_after: Option<Duration>,
    },

    /// Take a screenshot
    Screenshot {
        filename: Option<String>,
        full_page: bool,
    },

    /// Wait for a condition
    Wait(WaitCondition),

    /// Navigate to a URL
    Navigate { url: String, wait_for_load: bool },

    /// Set cookies
    SetCookies { cookies: HashMap<String, String> },

    /// Hover over an element
    Hover {
        selector: String,
        wait_after: Option<Duration>,
    },
}

/// Viewport configuration for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportConfig {
    /// Viewport width in pixels
    pub width: u32,

    /// Viewport height in pixels
    pub height: u32,

    /// Device scale factor (1.0 = normal, 2.0 = retina)
    pub device_scale_factor: f32,

    /// Whether this is a mobile viewport
    pub is_mobile: bool,

    /// User agent string override
    pub user_agent: Option<String>,
}

impl Default for ViewportConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            device_scale_factor: 1.0,
            is_mobile: false,
            user_agent: None,
        }
    }
}

/// Artifacts captured during rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderArtifacts {
    /// Screenshot data (PNG format, base64 encoded)
    pub screenshot: Option<String>,

    /// MHTML capture of the complete page
    pub mhtml: Option<String>,

    /// Page metadata collected during rendering
    pub metadata: PageMetadata,

    /// Console logs captured during rendering
    pub console_logs: Vec<ConsoleMessage>,

    /// Network requests made during rendering
    pub network_activity: Vec<NetworkRequest>,
}

/// Page metadata extracted during rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMetadata {
    /// Page title from document.title
    pub title: Option<String>,

    /// Meta description
    pub description: Option<String>,

    /// Open Graph tags
    pub og_tags: HashMap<String, String>,

    /// Twitter Card tags
    pub twitter_tags: HashMap<String, String>,

    /// JSON-LD structured data
    pub json_ld: Vec<serde_json::Value>,

    /// Final URL after redirects
    pub final_url: String,

    /// Response headers
    pub headers: HashMap<String, String>,

    /// Page load timing information
    pub timing: Option<PageTiming>,
}

/// Page load timing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageTiming {
    /// Time to first byte (TTFB)
    pub ttfb_ms: u64,

    /// DOM content loaded event
    pub dom_content_loaded_ms: u64,

    /// Load event
    pub load_ms: u64,

    /// First contentful paint
    pub first_contentful_paint_ms: Option<u64>,

    /// Largest contentful paint
    pub largest_contentful_paint_ms: Option<u64>,
}

/// Console message captured during rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleMessage {
    /// Log level (log, warn, error, debug, info)
    pub level: String,

    /// Message text
    pub text: String,

    /// Timestamp
    pub timestamp: String,

    /// Source location if available
    pub source: Option<String>,
}

/// Network request information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    /// Request URL
    pub url: String,

    /// HTTP method
    pub method: String,

    /// Response status code
    pub status: u16,

    /// Content type
    pub content_type: Option<String>,

    /// Response size in bytes
    pub size: u64,

    /// Request duration in milliseconds
    pub duration_ms: u64,
}

/// Result of dynamic content rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicRenderResult {
    /// Whether rendering was successful
    pub success: bool,

    /// Final HTML content after dynamic loading
    pub html: String,

    /// Artifacts captured during rendering
    pub artifacts: Option<RenderArtifacts>,

    /// Error message if rendering failed
    pub error: Option<String>,

    /// Total rendering time
    pub render_time_ms: u64,

    /// Actions that were successfully executed
    pub actions_executed: Vec<String>,

    /// Wait conditions that were satisfied
    pub wait_conditions_met: Vec<String>,
}

/// Dynamic content handler trait for different rendering backends
pub trait DynamicHandler {
    /// Render a page with dynamic content handling
    async fn render_dynamic(
        &self,
        url: &str,
        config: &DynamicConfig,
    ) -> Result<DynamicRenderResult, DynamicError>;

    /// Check if the handler is available and working
    async fn health_check(&self) -> Result<(), DynamicError>;

    /// Get handler capabilities
    fn capabilities(&self) -> DynamicCapabilities;
}

/// Capabilities of a dynamic content handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicCapabilities {
    /// Supported wait conditions
    pub wait_conditions: Vec<String>,

    /// Supported actions
    pub actions: Vec<String>,

    /// Whether screenshots are supported
    pub screenshots: bool,

    /// Whether MHTML capture is supported
    pub mhtml_capture: bool,

    /// Maximum timeout supported
    pub max_timeout_seconds: u64,

    /// Supported viewport sizes
    pub viewport_sizes: Vec<(u32, u32)>,
}

/// Errors that can occur during dynamic content handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DynamicError {
    /// Timeout waiting for condition
    Timeout { condition: String, waited_ms: u64 },

    /// Element not found for action
    ElementNotFound { selector: String },

    /// JavaScript execution error
    JavascriptError { script: String, error: String },

    /// Navigation failed
    NavigationError { url: String, error: String },

    /// Browser/renderer error
    RendererError { error: String },

    /// Configuration error
    ConfigError { message: String },

    /// Network error during rendering
    NetworkError { error: String },

    /// Resource limit exceeded
    ResourceLimit { limit: String, value: u64 },
}

impl std::fmt::Display for DynamicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DynamicError::Timeout {
                condition,
                waited_ms,
            } => {
                write!(
                    f,
                    "Timeout waiting for condition '{}' after {}ms",
                    condition, waited_ms
                )
            }
            DynamicError::ElementNotFound { selector } => {
                write!(f, "Element not found for selector '{}'", selector)
            }
            DynamicError::JavascriptError { script, error } => {
                write!(f, "JavaScript error in '{}': {}", script, error)
            }
            DynamicError::NavigationError { url, error } => {
                write!(f, "Navigation to '{}' failed: {}", url, error)
            }
            DynamicError::RendererError { error } => {
                write!(f, "Renderer error: {}", error)
            }
            DynamicError::ConfigError { message } => {
                write!(f, "Configuration error: {}", message)
            }
            DynamicError::NetworkError { error } => {
                write!(f, "Network error: {}", error)
            }
            DynamicError::ResourceLimit { limit, value } => {
                write!(f, "Resource limit exceeded: {} = {}", limit, value)
            }
        }
    }
}

impl std::error::Error for DynamicError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_config_default() {
        let config = DynamicConfig::default();
        assert!(!config.capture_artifacts);
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert!(config.actions.is_empty());
    }

    #[test]
    fn test_wait_condition_serialization() {
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
    fn test_scroll_config_default() {
        let config = ScrollConfig::default();
        assert_eq!(config.steps, 3);
        assert_eq!(config.delay_ms, 1000);
        assert!(matches!(config.mode, ScrollMode::Stepped));
    }

    #[test]
    fn test_viewport_config_default() {
        let config = ViewportConfig::default();
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert_eq!(config.device_scale_factor, 1.0);
        assert!(!config.is_mobile);
    }
}
