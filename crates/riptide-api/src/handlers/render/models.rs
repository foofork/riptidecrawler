use serde::{Deserialize, Serialize};
use riptide_core::types::{ExtractedDoc, OutputFormat, RenderMode};
use std::collections::HashMap;

/// Request body for enhanced render endpoint
#[derive(Deserialize, Debug, Clone)]
pub struct RenderRequest {
    /// URL to render
    pub url: String,

    /// Rendering mode (static, dynamic, adaptive, pdf)
    pub mode: Option<RenderMode>,

    /// Dynamic content configuration
    pub dynamic_config: Option<riptide_core::dynamic::DynamicConfig>,

    /// Stealth configuration for anti-detection
    pub stealth_config: Option<riptide_core::stealth::StealthConfig>,

    /// PDF processing configuration (for PDF URLs)
    pub pdf_config: Option<riptide_core::pdf::PdfConfig>,

    /// Output format preference
    pub output_format: Option<OutputFormat>,

    /// Whether to capture artifacts (screenshots, MHTML)
    pub capture_artifacts: Option<bool>,

    /// Timeout for rendering operation (seconds)
    pub timeout: Option<u64>,

    /// Session ID to use for persistent browser state (optional)
    pub session_id: Option<String>,
}

/// Response for render endpoint
#[derive(Serialize, Debug)]
pub struct RenderResponse {
    /// Original URL
    pub url: String,

    /// Final URL after redirects
    pub final_url: String,

    /// Rendering mode used
    pub mode: String,

    /// Whether rendering was successful
    pub success: bool,

    /// Extracted content
    pub content: Option<ExtractedDoc>,

    /// PDF processing result (if applicable)
    pub pdf_result: Option<riptide_core::pdf::PdfProcessingResult>,

    /// Dynamic rendering artifacts
    pub artifacts: Option<riptide_core::dynamic::RenderArtifacts>,

    /// Processing statistics
    pub stats: RenderStats,

    /// Error information if rendering failed
    pub error: Option<crate::models::ErrorInfo>,

    /// Stealth measures applied
    pub stealth_applied: Vec<String>,

    /// Session information used for rendering
    pub session_info: Option<SessionRenderInfo>,
}

/// Rendering statistics
#[derive(Serialize, Debug)]
pub struct RenderStats {
    /// Total processing time
    pub total_time_ms: u64,

    /// Time spent on dynamic rendering
    pub dynamic_time_ms: Option<u64>,

    /// Time spent on PDF processing
    pub pdf_time_ms: Option<u64>,

    /// Time spent on content extraction
    pub extraction_time_ms: u64,

    /// Number of actions executed
    pub actions_executed: u32,

    /// Number of wait conditions satisfied
    pub wait_conditions_met: u32,

    /// Network requests made during rendering
    pub network_requests: u32,

    /// Final page size in bytes
    pub page_size_bytes: u64,
}

/// Session information for render response
#[derive(Serialize, Debug)]
pub struct SessionRenderInfo {
    /// Session ID used
    pub session_id: String,

    /// User data directory
    pub user_data_dir: String,

    /// Number of cookies available for the domain
    pub cookies_for_domain: usize,

    /// Whether session state was preserved
    pub state_preserved: bool,
}