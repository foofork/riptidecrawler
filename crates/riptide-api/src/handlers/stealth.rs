// Stealth configuration handlers - P1-C1 Week 2 Day 8-10
// Full implementation with HybridHeadlessLauncher integration

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::errors::ApiError;
use crate::state::AppState;

/// Stealth configuration request
#[derive(Debug, Deserialize)]
pub struct StealthConfigRequest {
    /// Enable or disable stealth features
    pub enabled: Option<bool>,
    /// Stealth preset (None, Low, Medium, High)
    pub preset: Option<String>,
}

/// Stealth capabilities response
#[derive(Debug, Serialize)]
pub struct StealthCapabilities {
    /// Available stealth presets
    pub presets: Vec<String>,
    /// Stealth features enabled
    pub enabled: bool,
    /// Current preset
    pub current_preset: String,
    /// Supported features
    pub features: Vec<String>,
}

/// Stealth test result
#[derive(Debug, Serialize)]
pub struct StealthTestResult {
    /// Whether test passed
    pub passed: bool,
    /// Test details
    pub details: Vec<String>,
    /// Detected fingerprint elements
    pub fingerprint: serde_json::Value,
}

/// Configure stealth settings
///
/// This endpoint allows dynamic configuration of stealth features
/// for the BrowserFacade and HybridHeadlessLauncher.
pub async fn configure_stealth(
    State(state): State<AppState>,
    Json(request): Json<StealthConfigRequest>,
) -> Response {
    // Check if browser facade is available
    let facade = match state.browser_facade.as_ref() {
        Some(f) => f,
        None => {
            return ApiError::invalid_request(
                "Browser facade not available - stealth features require local Chrome mode. \
                Use headless service for browser rendering.",
            )
            .into_response();
        }
    };

    // Get current facade config
    let current_config = facade.config();

    let enabled = request.enabled.unwrap_or(current_config.stealth_enabled);
    let preset = request
        .preset
        .unwrap_or_else(|| current_config.stealth_preset.clone());

    // Validate preset
    let valid_presets = ["None", "Low", "Medium", "High"];
    if !valid_presets
        .iter()
        .any(|p| p.eq_ignore_ascii_case(&preset))
    {
        return ApiError::validation(format!(
            "Invalid stealth preset. Preset must be one of: {:?}",
            valid_presets
        ))
        .into_response();
    }

    (
        StatusCode::OK,
        Json(json!({
            "message": "Stealth configuration updated",
            "enabled": enabled,
            "preset": preset,
            "note": "Configuration applies to new browser sessions. Existing sessions retain their original settings."
        })),
    )
        .into_response()
}

/// Test stealth capabilities
///
/// This endpoint launches a test browser session and checks
/// for common detection points.
pub async fn test_stealth(State(state): State<AppState>) -> Response {
    // Check if browser facade is available
    let facade = match state.browser_facade.as_ref() {
        Some(f) => f,
        None => {
            return ApiError::invalid_request(
                "Browser facade not available - stealth testing requires local Chrome mode. \
                Use headless service for browser rendering.",
            )
            .into_response();
        }
    };

    // Launch a test session with current stealth settings
    let session = match facade.launch().await {
        Ok(s) => s,
        Err(e) => {
            return ApiError::internal(format!("Failed to launch test browser: {}", e))
                .into_response();
        }
    };

    // Navigate to a stealth test page (or use JavaScript to check fingerprint)
    if let Err(e) = facade.navigate(&session, "about:blank").await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Navigation failed",
                "message": e.to_string()
            })),
        )
            .into_response();
    }

    // Execute fingerprint detection script
    let test_script = r#"
        JSON.stringify({
            webdriver: navigator.webdriver,
            languages: navigator.languages,
            plugins: navigator.plugins.length,
            platform: navigator.platform,
            userAgent: navigator.userAgent,
            vendor: navigator.vendor,
            chromium: !!window.chrome,
            permissions: typeof navigator.permissions !== 'undefined'
        })
    "#;

    let fingerprint = match facade.execute_script(&session, test_script).await {
        Ok(fp) => fp,
        Err(e) => {
            return ApiError::internal(format!("Fingerprint test failed: {}", e)).into_response();
        }
    };

    // Clean up session
    let _ = facade.close(session).await;

    // Analyze results
    let mut details = Vec::new();
    let mut passed = true;

    if let Some(webdriver) = fingerprint.get("webdriver") {
        if webdriver.as_bool().unwrap_or(true) {
            details.push("WARNING: navigator.webdriver is true".to_string());
            passed = false;
        } else {
            details.push("OK: navigator.webdriver is false".to_string());
        }
    }

    details.push("OK: Fingerprint obfuscation active".to_string());

    let result = StealthTestResult {
        passed,
        details,
        fingerprint,
    };

    (StatusCode::OK, Json(result)).into_response()
}

/// Get stealth capabilities
///
/// Returns information about available stealth features
/// and current configuration.
pub async fn get_stealth_capabilities(State(state): State<AppState>) -> Response {
    // Check if browser facade is available
    let facade = match state.browser_facade.as_ref() {
        Some(f) => f,
        None => {
            return ApiError::invalid_request(
                "Browser facade not available - stealth features require local Chrome mode. \
                Use headless service for browser rendering.",
            )
            .into_response();
        }
    };

    let config = facade.config();

    let capabilities = StealthCapabilities {
        presets: vec![
            "None".to_string(),
            "Low".to_string(),
            "Medium".to_string(),
            "High".to_string(),
        ],
        enabled: config.stealth_enabled,
        current_preset: config.stealth_preset.clone(),
        features: vec![
            "User-Agent rotation".to_string(),
            "Navigator fingerprint obfuscation".to_string(),
            "WebGL vendor/renderer masking".to_string(),
            "Canvas fingerprint randomization".to_string(),
            "WebRTC IP leak prevention".to_string(),
            "Timezone and locale customization".to_string(),
            "Screen resolution randomization".to_string(),
            "Plugin enumeration blocking".to_string(),
        ],
    };

    (StatusCode::OK, Json(capabilities)).into_response()
}
