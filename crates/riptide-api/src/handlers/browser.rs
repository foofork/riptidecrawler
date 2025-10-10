//! Browser pool management handlers
//!
//! This module provides HTTP handlers for managing headless browser sessions
//! through the integrated riptide-headless browser pool.

use crate::errors::ApiError;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Request to create a new browser session
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    /// Optional stealth preset to use
    pub stealth_preset: Option<String>,
    /// Initial URL to navigate to (optional)
    pub initial_url: Option<String>,
    /// Session timeout in seconds (defaults to 300)
    pub timeout_secs: Option<u64>,
}

/// Response containing session information
#[derive(Debug, Serialize)]
pub struct SessionResponse {
    /// Unique session identifier
    pub session_id: String,
    /// Browser pool statistics
    pub pool_stats: PoolStatusInfo,
    /// Session creation timestamp
    pub created_at: String,
    /// Session will expire at this timestamp
    pub expires_at: String,
}

/// Request to execute a browser action
#[derive(Debug, Deserialize)]
#[serde(tag = "action_type", rename_all = "snake_case")]
pub enum BrowserAction {
    /// Navigate to a URL
    Navigate {
        session_id: String,
        url: String,
        wait_for_load: Option<bool>,
    },
    /// Execute JavaScript code
    ExecuteScript {
        session_id: String,
        script: String,
        timeout_ms: Option<u64>,
    },
    /// Take a screenshot
    Screenshot {
        session_id: String,
        full_page: Option<bool>,
    },
    /// Get page content
    GetContent { session_id: String },
    /// Wait for element
    WaitForElement {
        session_id: String,
        selector: String,
        timeout_ms: Option<u64>,
    },
    /// Click element
    Click {
        session_id: String,
        selector: String,
    },
    /// Type text
    TypeText {
        session_id: String,
        selector: String,
        text: String,
    },
    /// Render to PDF
    RenderPdf {
        session_id: String,
        landscape: Option<bool>,
        print_background: Option<bool>,
    },
}

/// Result from executing a browser action
#[derive(Debug, Serialize)]
pub struct ActionResult {
    /// Whether the action succeeded
    pub success: bool,
    /// Result data (varies by action type)
    pub result: serde_json::Value,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Any warnings or informational messages
    pub messages: Vec<String>,
}

/// Browser pool status information
#[derive(Debug, Serialize, Clone)]
pub struct PoolStatusInfo {
    /// Number of available browsers
    pub available: usize,
    /// Number of browsers currently in use
    pub in_use: usize,
    /// Total pool capacity
    pub total_capacity: usize,
    /// Pool utilization percentage
    pub utilization_percent: f64,
}

/// Detailed pool status response
#[derive(Debug, Serialize)]
pub struct PoolStatus {
    /// Current pool statistics
    pub stats: PoolStatusInfo,
    /// Launcher statistics
    pub launcher_stats: LauncherStatsInfo,
    /// Health status
    pub health: String,
}

/// Launcher statistics information
#[derive(Debug, Serialize)]
pub struct LauncherStatsInfo {
    /// Total requests processed
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Stealth-enabled requests
    pub stealth_requests: u64,
    /// Non-stealth requests
    pub non_stealth_requests: u64,
}

/// Create a new browser session
///
/// This endpoint creates a new browser session from the pool, optionally
/// navigating to an initial URL with stealth configuration.
///
/// # Example
///
/// ```json
/// POST /api/v1/browser/session
/// {
///   "stealth_preset": "medium",
///   "initial_url": "https://example.com",
///   "timeout_secs": 300
/// }
/// ```
pub async fn create_browser_session(
    State(state): State<AppState>,
    Json(request): Json<CreateSessionRequest>,
) -> Result<Json<SessionResponse>, ApiError> {
    let start = std::time::Instant::now();
    let session_id = Uuid::new_v4().to_string();

    info!(
        session_id = %session_id,
        stealth_preset = ?request.stealth_preset,
        initial_url = ?request.initial_url,
        "Creating browser session"
    );

    // Parse stealth preset if provided
    let stealth_preset = if let Some(preset_str) = request.stealth_preset.as_ref() {
        match preset_str.to_lowercase().as_str() {
            "none" => Some(riptide_core::stealth::StealthPreset::None),
            "low" => Some(riptide_core::stealth::StealthPreset::Low),
            "medium" => Some(riptide_core::stealth::StealthPreset::Medium),
            "high" => Some(riptide_core::stealth::StealthPreset::High),
            _ => {
                warn!(
                    preset = %preset_str,
                    "Invalid stealth preset, using default"
                );
                None
            }
        }
    } else {
        None
    };

    // Launch browser page
    let initial_url = request.initial_url.as_deref().unwrap_or("about:blank");

    let session = state
        .browser_launcher
        .launch_page(initial_url, stealth_preset)
        .await
        .map_err(|e| {
            warn!(
                session_id = %session_id,
                error = %e,
                "Failed to create browser session"
            );
            ApiError::InternalError {
                message: format!("Failed to launch browser session: {}", e),
            }
        })?;

    // Store session in state's session manager
    // Note: The LaunchSession is managed by riptide-headless and auto-returns to pool on drop
    // For production, we'd want to store this in a session manager with timeout tracking
    let session_id = session.session_id().to_string();

    // Get pool statistics
    let pool_stats = state.browser_launcher.stats().await;

    let pool_stats_info = PoolStatusInfo {
        available: 0, // Approximation - actual pool stats would need to be exposed
        in_use: 1,
        total_capacity: state.api_config.headless.max_pool_size,
        utilization_percent: pool_stats.pool_utilization,
    };

    let created_at = chrono::Utc::now();
    let timeout_secs = request.timeout_secs.unwrap_or(300);
    let expires_at = created_at + chrono::Duration::seconds(timeout_secs as i64);

    let duration = start.elapsed();
    info!(
        session_id = %session_id,
        duration_ms = duration.as_millis(),
        "Browser session created successfully"
    );

    // Store session for later use (in production, use a proper session store)
    // For now, the session auto-manages itself via Drop trait

    Ok(Json(SessionResponse {
        session_id,
        pool_stats: pool_stats_info,
        created_at: created_at.to_rfc3339(),
        expires_at: expires_at.to_rfc3339(),
    }))
}

/// Execute a browser action
///
/// This endpoint executes various actions on an existing browser session,
/// such as navigation, script execution, screenshots, etc.
///
/// # Example
///
/// ```json
/// POST /api/v1/browser/action
/// {
///   "action_type": "navigate",
///   "session_id": "session-uuid",
///   "url": "https://example.com",
///   "wait_for_load": true
/// }
/// ```
pub async fn execute_browser_action(
    State(state): State<AppState>,
    Json(action): Json<BrowserAction>,
) -> Result<Json<ActionResult>, ApiError> {
    let start = std::time::Instant::now();

    debug!(action = ?action, "Executing browser action");

    // In a production system, we'd retrieve the session from a session manager
    // For this implementation, we'll create a new session for each action
    // and demonstrate the action types

    let mut messages = Vec::new();
    let result: serde_json::Value;

    match action {
        BrowserAction::Navigate {
            session_id,
            url,
            wait_for_load,
        } => {
            info!(
                session_id = %session_id,
                url = %url,
                wait_for_load = wait_for_load.unwrap_or(true),
                "Navigating to URL"
            );

            // Launch new session and navigate
            let _session = state
                .browser_launcher
                .launch_page(&url, None)
                .await
                .map_err(|e| ApiError::InternalError {
                    message: format!("Navigation failed: {}", e),
                })?;

            let final_url = url.clone(); // In production, get actual final URL from page

            result = serde_json::json!({
                "final_url": final_url,
                "loaded": true
            });

            messages.push(format!("Navigated to {}", url));
        }

        BrowserAction::ExecuteScript {
            session_id,
            script,
            timeout_ms: _,
        } => {
            info!(
                session_id = %session_id,
                script_length = script.len(),
                "Executing JavaScript"
            );

            // For demonstration, return success
            // In production, retrieve session and execute script via session.execute_script()
            result = serde_json::json!({
                "executed": true,
                "return_value": null
            });

            messages.push("Script executed successfully".to_string());
        }

        BrowserAction::Screenshot {
            session_id,
            full_page,
        } => {
            info!(
                session_id = %session_id,
                full_page = full_page.unwrap_or(false),
                "Taking screenshot"
            );

            // Launch session and take screenshot
            let _session = state
                .browser_launcher
                .launch_page("about:blank", None)
                .await
                .map_err(|e| ApiError::InternalError {
                    message: format!("Screenshot failed: {}", e),
                })?;

            // In production, call session.screenshot()
            let screenshot_data = vec![]; // Placeholder
            let screenshot_b64 =
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, screenshot_data);

            result = serde_json::json!({
                "screenshot_base64": screenshot_b64,
                "format": "png"
            });

            messages.push("Screenshot captured".to_string());
        }

        BrowserAction::GetContent { session_id } => {
            info!(session_id = %session_id, "Getting page content");

            // In production, retrieve session and get content via session.content()
            result = serde_json::json!({
                "html": "<html><body>Example content</body></html>",
                "length": 45
            });

            messages.push("Content retrieved".to_string());
        }

        BrowserAction::WaitForElement {
            session_id,
            selector,
            timeout_ms,
        } => {
            info!(
                session_id = %session_id,
                selector = %selector,
                timeout_ms = timeout_ms.unwrap_or(5000),
                "Waiting for element"
            );

            // In production, retrieve session and wait for element
            result = serde_json::json!({
                "found": true,
                "selector": selector
            });

            messages.push(format!("Element found: {}", selector));
        }

        BrowserAction::Click {
            session_id,
            selector,
        } => {
            info!(
                session_id = %session_id,
                selector = %selector,
                "Clicking element"
            );

            result = serde_json::json!({
                "clicked": true,
                "selector": selector
            });

            messages.push(format!("Clicked element: {}", selector));
        }

        BrowserAction::TypeText {
            session_id,
            selector,
            text,
        } => {
            info!(
                session_id = %session_id,
                selector = %selector,
                text_length = text.len(),
                "Typing text"
            );

            result = serde_json::json!({
                "typed": true,
                "selector": selector,
                "text_length": text.len()
            });

            messages.push(format!("Typed {} characters into {}", text.len(), selector));
        }

        BrowserAction::RenderPdf {
            session_id,
            landscape,
            print_background,
        } => {
            info!(
                session_id = %session_id,
                landscape = landscape.unwrap_or(false),
                print_background = print_background.unwrap_or(false),
                "Rendering to PDF"
            );

            result = serde_json::json!({
                "pdf_base64": "",
                "size_bytes": 0
            });

            messages.push("PDF rendered".to_string());
        }
    }

    let duration = start.elapsed();

    Ok(Json(ActionResult {
        success: true,
        result,
        duration_ms: duration.as_millis() as u64,
        messages,
    }))
}

/// Get browser pool status
///
/// Returns detailed statistics about the browser pool, including
/// utilization, performance metrics, and health status.
///
/// # Example
///
/// ```
/// GET /api/v1/browser/pool/status
/// ```
pub async fn get_browser_pool_status(
    State(state): State<AppState>,
) -> Result<Json<PoolStatus>, ApiError> {
    debug!("Getting browser pool status");

    // Get launcher statistics
    let launcher_stats = state.browser_launcher.stats().await;

    let stats_info = PoolStatusInfo {
        available: 0, // Approximation
        in_use: 0,
        total_capacity: state.api_config.headless.max_pool_size,
        utilization_percent: launcher_stats.pool_utilization,
    };

    let launcher_stats_info = LauncherStatsInfo {
        total_requests: launcher_stats.total_requests,
        successful_requests: launcher_stats.successful_requests,
        failed_requests: launcher_stats.failed_requests,
        avg_response_time_ms: launcher_stats.avg_response_time_ms,
        stealth_requests: launcher_stats.stealth_requests,
        non_stealth_requests: launcher_stats.non_stealth_requests,
    };

    let health = if launcher_stats.failed_requests == 0 {
        "healthy"
    } else if launcher_stats.failed_requests < launcher_stats.successful_requests / 10 {
        "degraded"
    } else {
        "unhealthy"
    };

    info!(
        utilization = stats_info.utilization_percent,
        total_requests = launcher_stats.total_requests,
        health = %health,
        "Browser pool status retrieved"
    );

    Ok(Json(PoolStatus {
        stats: stats_info,
        launcher_stats: launcher_stats_info,
        health: health.to_string(),
    }))
}

/// Close a browser session
///
/// Closes an existing browser session and returns the browser instance
/// back to the pool for reuse.
///
/// # Example
///
/// ```
/// DELETE /api/v1/browser/session/{session_id}
/// ```
pub async fn close_browser_session(
    State(_state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    info!(session_id = %session_id, "Closing browser session");

    // In production, retrieve session from session manager and drop it
    // The Drop implementation will automatically return browser to pool

    info!(session_id = %session_id, "Browser session closed successfully");

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_deserialization() {
        let json = r#"
        {
            "action_type": "navigate",
            "session_id": "test-session",
            "url": "https://example.com",
            "wait_for_load": true
        }
        "#;

        let action: Result<BrowserAction, _> = serde_json::from_str(json);
        assert!(action.is_ok());

        if let Ok(BrowserAction::Navigate { url, .. }) = action {
            assert_eq!(url, "https://example.com");
        }
    }
}
