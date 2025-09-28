//! Stealth configuration and testing routes
//!
//! This module defines the HTTP routes for stealth configuration, testing, and capabilities.

use axum::{
    routing::{get, post},
    Router,
};
use crate::{handlers::stealth, state::AppState};

/// Configure stealth processing routes
///
/// Provides endpoints for:
/// - Stealth configuration management
/// - Stealth effectiveness testing
/// - Stealth capabilities inquiry
/// - Health check for stealth features
pub fn stealth_routes() -> Router<AppState> {
    Router::new()
        // Stealth configuration endpoint
        .route("/configure", post(stealth::configure_stealth))
        // Stealth testing endpoint
        .route("/test", post(stealth::test_stealth))
        // Stealth capabilities endpoint
        .route("/capabilities", get(stealth::get_stealth_capabilities))
        // Health check for stealth features
        .route("/health", get(stealth_health_check))
}

/// Stealth features health check endpoint
async fn stealth_health_check() -> axum::response::Json<serde_json::Value> {
    use riptide_stealth::{StealthController, StealthPreset};

    // Test basic stealth functionality
    let _controller = StealthController::from_preset(StealthPreset::Medium);

    axum::response::Json(serde_json::json!({
        "status": "healthy",
        "stealth_available": true,
        "features": {
            "user_agent_rotation": true,
            "header_randomization": true,
            "timing_jitter": true,
            "fingerprinting_countermeasures": true,
            "proxy_support": true,
            "javascript_evasion": true
        },
        "presets": ["None", "Low", "Medium", "High"],
        "rotation_strategies": ["Random", "Sequential", "Sticky", "DomainBased"],
        "version": riptide_stealth::VERSION,
        "crate_name": riptide_stealth::CRATE_NAME
    }))
}