//! Browser pool management handlers (ultra-thin, delegates to BrowserFacade)

use crate::errors::ApiError;
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    /// Stealth preset configuration (reserved for future browser facade integration)
    #[allow(dead_code)]
    pub stealth_preset: Option<String>,
    /// Initial URL to navigate to on session creation (reserved for future browser facade integration)
    #[allow(dead_code)]
    pub initial_url: Option<String>,
    /// Session timeout in seconds (reserved for future browser facade integration)
    #[allow(dead_code)]
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub session_id: String,
    pub pool_stats: serde_json::Value,
    pub created_at: String,
    pub expires_at: String,
}

pub async fn create_browser_session(
    State(_state): State<AppState>,
    Json(_request): Json<CreateSessionRequest>,
) -> Result<Json<SessionResponse>, ApiError> {
    // browser_facade was removed due to circular dependency
    // TODO: Restore browser session creation when facade is re-integrated
    Err(ApiError::invalid_request(
        "Browser facade temporarily unavailable due to refactoring. \
        Browser session creation will be restored in a future update.",
    ))

    /* Original implementation - disabled until browser_facade is restored
    let facade = &state.browser_facade;
    let session = facade.launch().await.map_err(ApiError::from)?;
    let pool_stats = facade.pool_status().await.map_err(ApiError::from)?;
    let session_id = uuid::Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now();
    let expires_at = created_at + chrono::Duration::seconds(300);
    facade.close(session).await.map_err(ApiError::from)?;

    Ok(Json(SessionResponse {
        session_id,
        pool_stats,
        created_at: created_at.to_rfc3339(),
        expires_at: expires_at.to_rfc3339(),
    }))
    */
}

pub async fn get_browser_pool_status(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // browser_facade was removed due to circular dependency
    // TODO: Restore pool status retrieval when facade is re-integrated
    Err(ApiError::invalid_request(
        "Browser facade temporarily unavailable due to refactoring. \
        Pool status will be restored in a future update.",
    ))

    /* Original implementation - disabled until browser_facade is restored
    let stats = state
        .browser_facade
        .pool_status()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(stats))
    */
}

pub async fn close_browser_session(
    State(_state): State<AppState>,
    axum::extract::Path(_session_id): axum::extract::Path<String>,
) -> Result<StatusCode, ApiError> {
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct BrowserActionRequest {
    pub session_id: String,
    pub action: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct BrowserActionResponse {
    pub success: bool,
    pub result: Option<serde_json::Value>,
}

pub async fn execute_browser_action(
    State(_state): State<AppState>,
    Json(_request): Json<BrowserActionRequest>,
) -> Result<Json<BrowserActionResponse>, ApiError> {
    // browser_facade was removed due to circular dependency
    // TODO: Restore browser action execution when facade is re-integrated
    Err(ApiError::invalid_request(
        "Browser facade temporarily unavailable due to refactoring. \
        Browser actions will be restored in a future update.",
    ))
}
