//! Parameter types for page operations

use serde::{Deserialize, Serialize};

/// Screenshot parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotParams {
    /// Full page screenshot (default: false)
    pub full_page: bool,
    /// Image format (default: "png")
    pub format: ScreenshotFormat,
    /// JPEG quality 0-100 (default: 80)
    pub quality: Option<u8>,
    /// Capture viewport only
    pub viewport_only: bool,
}

impl Default for ScreenshotParams {
    fn default() -> Self {
        Self {
            full_page: false,
            format: ScreenshotFormat::Png,
            quality: Some(80),
            viewport_only: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScreenshotFormat {
    Png,
    Jpeg,
}

/// PDF generation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfParams {
    /// Print background graphics (default: true)
    pub print_background: bool,
    /// Page scale (default: 1.0)
    pub scale: f64,
    /// Landscape orientation (default: false)
    pub landscape: bool,
    /// Paper width in inches
    pub paper_width: Option<f64>,
    /// Paper height in inches
    pub paper_height: Option<f64>,
}

impl Default for PdfParams {
    fn default() -> Self {
        Self {
            print_background: true,
            scale: 1.0,
            landscape: false,
            paper_width: Some(8.5),
            paper_height: Some(11.0),
        }
    }
}

/// Navigation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigateParams {
    /// Wait for navigation timeout in ms (default: 30000)
    pub timeout_ms: u64,
    /// Wait until condition (default: Load)
    pub wait_until: WaitUntil,
    /// Referrer URL
    pub referer: Option<String>,
}

impl Default for NavigateParams {
    fn default() -> Self {
        Self {
            timeout_ms: 30000,
            wait_until: WaitUntil::Load,
            referer: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WaitUntil {
    /// Wait for load event
    Load,
    /// Wait for DOM content loaded
    DOMContentLoaded,
    /// Wait for network idle
    NetworkIdle,
}
