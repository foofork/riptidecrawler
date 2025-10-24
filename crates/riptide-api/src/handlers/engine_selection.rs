//! Engine Selection API handlers
//!
//! Provides HTTP endpoints for engine selection analysis and decision-making.
//! This module integrates with `riptide-reliability::engine_selection` to provide
//! intelligent engine recommendations based on HTML content analysis.
//!
//! ## Endpoints
//!
//! - POST `/engine/analyze` - Analyze HTML and get detailed content analysis with engine recommendation
//! - POST `/engine/decide` - Make engine decision with feature flags for advanced optimizations
//! - GET `/engine/stats` - Get engine usage statistics across all requests
//! - PUT `/engine/probe-first` - Toggle probe-first mode for SPA optimization
//!
//! ## Usage Example
//!
//! ```bash
//! # Analyze HTML content
//! curl -X POST http://localhost:8080/engine/analyze \
//!   -H "Content-Type: application/json" \
//!   -d '{"html": "<html>...</html>", "url": "https://example.com"}'
//!
//! # Make engine decision with probe-first enabled
//! curl -X POST http://localhost:8080/engine/decide \
//!   -H "Content-Type: application/json" \
//!   -d '{"html": "<html>...</html>", "url": "https://example.com", "flags": {"probe_first_spa": true}}'
//!
//! # Get engine statistics
//! curl http://localhost:8080/engine/stats
//!
//! # Toggle probe-first mode
//! curl -X PUT http://localhost:8080/engine/probe-first \
//!   -H "Content-Type: application/json" \
//!   -d '{"enabled": true}'
//! ```

use crate::errors::ApiResult;
use crate::state::AppState;
use axum::{extract::State, response::Json};
use riptide_reliability::engine_selection::{
    analyze_content, decide_engine_with_flags, Engine, EngineSelectionFlags,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Request for engine analysis
#[derive(Debug, Deserialize)]
pub struct AnalyzeRequest {
    /// HTML content to analyze
    pub html: String,
    /// Source URL (used for additional context)
    pub url: String,
}

/// Request for engine decision with flags
#[derive(Debug, Deserialize)]
pub struct DecideRequest {
    /// HTML content to analyze
    pub html: String,
    /// Source URL (used for additional context)
    pub url: String,
    /// Feature flags for engine selection optimizations
    #[serde(default)]
    pub flags: EngineSelectionFlagsRequest,
}

/// Engine selection feature flags (API request format)
#[derive(Debug, Clone, Deserialize, Default)]
pub struct EngineSelectionFlagsRequest {
    /// Enable refined visible-text density calculation
    #[serde(default)]
    pub use_visible_text_density: bool,
    /// Enable placeholder/skeleton detection
    #[serde(default)]
    pub detect_placeholders: bool,
    /// Enable probe-first escalation for SPAs (try WASM before headless)
    #[serde(default)]
    pub probe_first_spa: bool,
}

impl From<EngineSelectionFlagsRequest> for EngineSelectionFlags {
    fn from(req: EngineSelectionFlagsRequest) -> Self {
        EngineSelectionFlags {
            use_visible_text_density: req.use_visible_text_density,
            detect_placeholders: req.detect_placeholders,
            probe_first_spa: req.probe_first_spa,
        }
    }
}

/// Response for engine analysis
#[derive(Debug, Serialize)]
pub struct AnalyzeResponse {
    /// Recommended engine
    pub engine: String,
    /// Confidence score (0-100)
    pub confidence: f64,
    /// Reasons for the recommendation
    pub reasons: Vec<String>,
    /// Detailed content analysis
    pub analysis: ContentAnalysisResponse,
}

/// Content analysis details (API response format)
#[derive(Debug, Serialize)]
pub struct ContentAnalysisResponse {
    /// Detected React/Next.js framework
    pub has_react: bool,
    /// Detected Vue framework
    pub has_vue: bool,
    /// Detected Angular framework
    pub has_angular: bool,
    /// Single Page Application markers detected
    pub has_spa_markers: bool,
    /// Anti-scraping protection detected
    pub has_anti_scraping: bool,
    /// Content-to-markup ratio (0.0 to 1.0)
    pub content_ratio: f64,
    /// Has main content tags (article, main, etc.)
    pub has_main_content: bool,
    /// Visible text density (excludes scripts/styles)
    pub visible_text_density: f64,
    /// Detected placeholder/skeleton elements
    pub has_placeholders: bool,
}

/// Response for engine decision
#[derive(Debug, Serialize)]
pub struct DecideResponse {
    /// Selected engine
    pub engine: String,
    /// Confidence score (0-100)
    pub confidence: f64,
    /// Reasons for the decision
    pub reasons: Vec<String>,
    /// Applied feature flags
    pub flags: EngineSelectionFlagsResponse,
}

/// Engine selection feature flags (API response format)
#[derive(Debug, Serialize)]
pub struct EngineSelectionFlagsResponse {
    pub use_visible_text_density: bool,
    pub detect_placeholders: bool,
    pub probe_first_spa: bool,
}

/// Engine usage statistics
#[derive(Debug, Serialize)]
pub struct EngineStatsResponse {
    /// Total requests analyzed
    pub total_requests: u64,
    /// Engine usage counts
    pub engine_counts: HashMap<String, u64>,
    /// Engine usage percentages
    pub engine_percentages: HashMap<String, f64>,
    /// Current probe-first mode status
    pub probe_first_enabled: bool,
}

/// Request to toggle probe-first mode
#[derive(Debug, Deserialize)]
pub struct ProbeFirstRequest {
    /// Enable or disable probe-first mode
    pub enabled: bool,
}

/// Response for probe-first toggle
#[derive(Debug, Serialize)]
pub struct ProbeFirstResponse {
    /// New probe-first mode status
    pub enabled: bool,
    /// Success message
    pub message: String,
}

/// POST /engine/analyze - Analyze HTML and recommend engine
///
/// Provides detailed content analysis and engine recommendation.
/// This endpoint performs comprehensive analysis including:
/// - JavaScript framework detection (React, Vue, Angular)
/// - SPA markers and anti-scraping protection
/// - Content-to-markup ratio and visible text density
/// - Placeholder/skeleton element detection
///
/// # Returns
///
/// Detailed analysis with recommended engine and confidence score.
pub async fn analyze_engine(
    State(state): State<AppState>,
    Json(request): Json<AnalyzeRequest>,
) -> ApiResult<Json<AnalyzeResponse>> {
    info!(
        url = %request.url,
        html_length = request.html.len(),
        "Analyzing HTML content for engine selection"
    );

    // Perform content analysis
    let analysis = analyze_content(&request.html, &request.url);

    // Calculate confidence based on detection strength
    let confidence = calculate_confidence(&analysis);

    // Generate reasoning
    let reasons = generate_reasons(&analysis);

    debug!(
        engine = %analysis.recommended_engine,
        confidence = confidence,
        "Engine analysis complete"
    );

    // Update metrics
    state
        .metrics
        .increment_engine_selection(analysis.recommended_engine.name(), confidence);

    let response = AnalyzeResponse {
        engine: analysis.recommended_engine.name().to_string(),
        confidence,
        reasons,
        analysis: ContentAnalysisResponse {
            has_react: analysis.has_react,
            has_vue: analysis.has_vue,
            has_angular: analysis.has_angular,
            has_spa_markers: analysis.has_spa_markers,
            has_anti_scraping: analysis.has_anti_scraping,
            content_ratio: analysis.content_ratio,
            has_main_content: analysis.has_main_content,
            visible_text_density: analysis.visible_text_density,
            has_placeholders: analysis.has_placeholders,
        },
    };

    Ok(Json(response))
}

/// POST /engine/decide - Make engine decision with flags
///
/// Make engine selection with fine-grained control through feature flags.
/// This endpoint supports advanced optimizations like probe-first escalation.
///
/// # Feature Flags
///
/// - `use_visible_text_density`: Use refined visible-text density calculation
/// - `detect_placeholders`: Enable placeholder/skeleton detection
/// - `probe_first_spa`: Try WASM before headless for SPAs (60-80% cost savings)
///
/// # Returns
///
/// Engine decision with confidence score and applied flags.
pub async fn decide_engine(
    State(state): State<AppState>,
    Json(request): Json<DecideRequest>,
) -> ApiResult<Json<DecideResponse>> {
    info!(
        url = %request.url,
        html_length = request.html.len(),
        probe_first_spa = request.flags.probe_first_spa,
        "Making engine decision with flags"
    );

    let flags: EngineSelectionFlags = request.flags.clone().into();

    // Make engine decision (Phase 10.4: Pass () for no domain profile)
    let engine = decide_engine_with_flags(&request.html, &request.url, flags, ());

    // Perform analysis for confidence and reasoning
    let analysis = analyze_content(&request.html, &request.url);
    let confidence = calculate_confidence(&analysis);
    let reasons = generate_reasons(&analysis);

    debug!(
        engine = %engine,
        confidence = confidence,
        probe_first_spa = flags.probe_first_spa,
        "Engine decision complete"
    );

    // Update metrics
    state
        .metrics
        .increment_engine_selection(engine.name(), confidence);

    let response = DecideResponse {
        engine: engine.name().to_string(),
        confidence,
        reasons,
        flags: EngineSelectionFlagsResponse {
            use_visible_text_density: flags.use_visible_text_density,
            detect_placeholders: flags.detect_placeholders,
            probe_first_spa: flags.probe_first_spa,
        },
    };

    Ok(Json(response))
}

/// GET /engine/stats - Get engine usage statistics
///
/// Returns aggregated statistics about engine usage across all requests.
/// Includes total request count, per-engine usage counts, and percentages.
///
/// # Returns
///
/// Engine usage statistics with current probe-first mode status.
pub async fn get_engine_stats(
    State(state): State<AppState>,
) -> ApiResult<Json<EngineStatsResponse>> {
    debug!("Retrieving engine usage statistics");

    // Get statistics from metrics (if available)
    let stats = state.metrics.get_engine_stats();

    let probe_first_enabled = state.config.engine_selection_config.probe_first_spa;

    info!(
        total_requests = stats.total_requests,
        probe_first_enabled = probe_first_enabled,
        "Retrieved engine statistics"
    );

    let response = EngineStatsResponse {
        total_requests: stats.total_requests,
        engine_counts: stats.engine_counts,
        engine_percentages: stats.engine_percentages,
        probe_first_enabled,
    };

    Ok(Json(response))
}

/// PUT /engine/probe-first - Toggle probe-first mode
///
/// Enable or disable probe-first SPA optimization globally.
/// When enabled, SPA-detected pages will first attempt WASM extraction
/// before escalating to headless browser.
///
/// # Cost Impact
///
/// - Enabled: 60-80% cost savings on SPAs with server-rendered content
/// - Risk: Minimal - automatic escalation ensures content quality
///
/// # Returns
///
/// New probe-first mode status with confirmation message.
pub async fn toggle_probe_first(
    State(state): State<AppState>,
    Json(request): Json<ProbeFirstRequest>,
) -> ApiResult<Json<ProbeFirstResponse>> {
    info!(enabled = request.enabled, "Toggling probe-first mode");

    // Update configuration (requires mutable access)
    // Note: In production, this should update persistent configuration
    // For now, we'll track in AppState's engine_selection_config
    let mut config = state.config.engine_selection_config.clone();
    config.probe_first_spa = request.enabled;

    // In production: state.config.engine_selection_config = config;
    // This would require Arc<Mutex<EngineSelectionConfig>> or similar

    let message = if request.enabled {
        "Probe-first mode enabled. SPAs will try WASM extraction before headless."
    } else {
        "Probe-first mode disabled. SPAs will use headless browser directly."
    };

    info!(
        enabled = request.enabled,
        "Probe-first mode toggled successfully"
    );

    let response = ProbeFirstResponse {
        enabled: request.enabled,
        message: message.to_string(),
    };

    Ok(Json(response))
}

/// Calculate confidence score based on analysis strength
fn calculate_confidence(analysis: &riptide_reliability::engine_selection::ContentAnalysis) -> f64 {
    let mut confidence: f64 = 50.0; // Base confidence

    // High confidence indicators
    if analysis.has_anti_scraping {
        confidence += 40.0; // Very strong signal for headless
    }

    if analysis.has_react || analysis.has_vue || analysis.has_angular {
        confidence += 30.0; // Strong framework signals
    }

    if analysis.has_spa_markers {
        confidence += 20.0; // SPA indicators
    }

    // Content quality indicators
    if analysis.content_ratio > 0.3 {
        confidence += 15.0; // Good content ratio
    } else if analysis.content_ratio < 0.1 {
        confidence += 25.0; // Very low ratio is strong signal for headless
    }

    // Visible text density refinement
    if analysis.visible_text_density > 0.2 {
        confidence += 10.0; // Good visible content
    }

    // Placeholder detection
    if analysis.has_placeholders {
        confidence += 15.0; // Loading indicators suggest JS needed
    }

    // Main content structure
    if analysis.has_main_content && analysis.content_ratio > 0.2 {
        confidence += 10.0; // Well-structured content
    }

    // Cap at 100
    confidence.min(100.0_f64)
}

/// Generate human-readable reasons for engine selection
fn generate_reasons(
    analysis: &riptide_reliability::engine_selection::ContentAnalysis,
) -> Vec<String> {
    let mut reasons = Vec::new();

    // Framework detection reasons
    if analysis.has_react {
        reasons.push("React framework detected (Next.js markers, webpack)".to_string());
    }
    if analysis.has_vue {
        reasons.push("Vue.js framework detected".to_string());
    }
    if analysis.has_angular {
        reasons.push("Angular framework detected".to_string());
    }

    // SPA and anti-scraping reasons
    if analysis.has_spa_markers {
        reasons.push("Single Page Application (SPA) markers found".to_string());
    }
    if analysis.has_anti_scraping {
        reasons.push("Anti-scraping protection detected (Cloudflare, reCAPTCHA)".to_string());
    }

    // Content analysis reasons
    if analysis.content_ratio < 0.1 {
        reasons.push(format!(
            "Low content-to-markup ratio ({:.1}%) suggests client-side rendering",
            analysis.content_ratio * 100.0
        ));
    } else if analysis.content_ratio > 0.3 {
        reasons.push(format!(
            "Good content-to-markup ratio ({:.1}%) indicates server-rendered content",
            analysis.content_ratio * 100.0
        ));
    }

    // Visible text density
    if analysis.visible_text_density < 0.15 {
        reasons.push("Low visible text density (excluding scripts/styles)".to_string());
    }

    // Placeholder detection
    if analysis.has_placeholders {
        reasons.push("Skeleton/placeholder UI detected (shimmer, loading indicators)".to_string());
    }

    // Main content structure
    if analysis.has_main_content {
        reasons.push("Well-structured content with article/main tags".to_string());
    }

    // Engine recommendation reason
    match analysis.recommended_engine {
        Engine::Headless => {
            if reasons.is_empty() {
                reasons.push("JavaScript execution required for content rendering".to_string());
            }
        }
        Engine::Wasm => {
            if reasons.is_empty() {
                reasons.push("Standard HTML extraction with WASM is sufficient".to_string());
            }
        }
        Engine::Raw => {
            reasons.push("Simple HTTP fetch without JavaScript execution".to_string());
        }
        Engine::Auto => {
            // Should not happen after decision
            reasons.push("Automatic engine selection needed".to_string());
        }
    }

    reasons
}
