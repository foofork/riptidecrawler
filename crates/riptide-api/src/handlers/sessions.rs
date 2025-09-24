use crate::errors::{ApiError, ApiResult};
use crate::sessions::{Cookie, SessionContext, SessionManager, SessionStats};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Request body for setting a cookie
#[derive(Deserialize, Debug)]
pub struct SetCookieRequest {
    /// Cookie domain
    pub domain: String,
    /// Cookie name
    pub name: String,
    /// Cookie value
    pub value: String,
    /// Cookie path (optional)
    pub path: Option<String>,
    /// Cookie expiry in seconds from now (optional)
    pub expires_in_seconds: Option<u64>,
    /// Secure flag
    pub secure: Option<bool>,
    /// HttpOnly flag
    pub http_only: Option<bool>,
}

/// Response for session creation
#[derive(Serialize, Debug)]
pub struct CreateSessionResponse {
    pub session_id: String,
    pub user_data_dir: String,
    pub created_at: String,
    pub expires_at: String,
}

/// Response for getting session info
#[derive(Serialize, Debug)]
pub struct SessionInfoResponse {
    pub session_id: String,
    pub user_data_dir: String,
    pub created_at: String,
    pub last_accessed: String,
    pub expires_at: String,
    pub cookie_count: usize,
    pub total_domains: usize,
}

/// Response for listing cookies
#[derive(Serialize, Debug)]
pub struct CookieResponse {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub expires: Option<String>,
    pub secure: bool,
    pub http_only: bool,
}

/// Query parameters for listing sessions
#[derive(Deserialize, Debug)]
pub struct ListSessionsQuery {
    /// Include expired sessions
    pub include_expired: Option<bool>,
    /// Limit the number of results
    pub limit: Option<usize>,
}

/// Create a new session
pub async fn create_session(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let session = state
        .session_manager
        .create_session()
        .await
        .map_err(|e| ApiError::dependency("session_manager", e.to_string()))?;

    let response = CreateSessionResponse {
        session_id: session.session_id.clone(),
        user_data_dir: session.user_data_dir.to_string_lossy().to_string(),
        created_at: session
            .created_at
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string(),
        expires_at: session
            .expires_at
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string(),
    };

    info!(
        session_id = %session.session_id,
        "Created new session via API"
    );

    Ok(Json(response))
}

/// Get session information
pub async fn get_session_info(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let session = state
        .session_manager
        .get_session(&session_id)
        .await
        .map_err(|e| ApiError::dependency("session_manager", e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Session not found"))?;

    let cookies = state
        .session_manager
        .get_all_cookies(&session_id)
        .await
        .map_err(|e| ApiError::dependency("session_manager", e.to_string()))?;

    let response = SessionInfoResponse {
        session_id: session.session_id.clone(),
        user_data_dir: session.user_data_dir.to_string_lossy().to_string(),
        created_at: session
            .created_at
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string(),
        last_accessed: session
            .last_accessed
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string(),
        expires_at: session
            .expires_at
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string(),
        cookie_count: cookies.all_cookies().len(),
        total_domains: cookies.cookies.len(),
    };

    debug!(
        session_id = %session_id,
        cookie_count = response.cookie_count,
        "Retrieved session information"
    );

    Ok(Json(response))
}

/// Delete a session
pub async fn delete_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .session_manager
        .remove_session(&session_id)
        .await
        .map_err(|e| ApiError::dependency("session_manager", e.to_string()))?;

    info!(
        session_id = %session_id,
        "Deleted session via API"
    );

    Ok(StatusCode::NO_CONTENT)
}

/// Set a cookie for a session
pub async fn set_cookie(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(body): Json<SetCookieRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let mut cookie = Cookie::new(body.name.clone(), body.value.clone());

    if let Some(path) = body.path {
        cookie = cookie.with_path(path);
    }

    if let Some(expires_in) = body.expires_in_seconds {
        let expires = std::time::SystemTime::now() + std::time::Duration::from_secs(expires_in);
        cookie = cookie.with_expires(expires);
    }

    if body.secure.unwrap_or(false) {
        cookie = cookie.secure();
    }

    if body.http_only.unwrap_or(false) {
        cookie = cookie.http_only();
    }

    state
        .session_manager
        .set_cookie(&session_id, &body.domain, cookie)
        .await
        .map_err(|e| ApiError::dependency("session_manager", e.to_string()))?;

    debug!(
        session_id = %session_id,
        domain = %body.domain,
        cookie_name = %body.name,
        "Set cookie for session"
    );

    Ok(StatusCode::CREATED)
}

/// Get a specific cookie from a session
pub async fn get_cookie(
    State(state): State<AppState>,
    Path((session_id, domain, name)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    let cookie = state
        .session_manager
        .get_cookie(&session_id, &domain, &name)
        .await
        .map_err(|e| ApiError::dependency("session_manager", e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Cookie not found"))?;

    let response = CookieResponse {
        name: cookie.name,
        value: cookie.value,
        domain: cookie.domain,
        path: cookie.path,
        expires: cookie.expires.map(|exp| {
            exp.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string()
        }),
        secure: cookie.secure,
        http_only: cookie.http_only,
    };

    Ok(Json(response))
}

/// Get all cookies for a domain from a session
pub async fn get_cookies_for_domain(
    State(state): State<AppState>,
    Path((session_id, domain)): Path<(String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    let cookies = state
        .session_manager
        .get_cookies_for_domain(&session_id, &domain)
        .await
        .map_err(|e| ApiError::dependency("session_manager", e.to_string()))?;

    let response: Vec<CookieResponse> = cookies
        .into_iter()
        .map(|cookie| CookieResponse {
            name: cookie.name,
            value: cookie.value,
            domain: cookie.domain,
            path: cookie.path,
            expires: cookie.expires.map(|exp| {
                exp.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    .to_string()
            }),
            secure: cookie.secure,
            http_only: cookie.http_only,
        })
        .collect();

    debug!(
        session_id = %session_id,
        domain = %domain,
        cookie_count = response.len(),
        "Retrieved cookies for domain"
    );

    Ok(Json(response))
}

/// Delete a specific cookie from a session
pub async fn delete_cookie(
    State(state): State<AppState>,
    Path((session_id, domain, name)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    let removed = state
        .session_manager
        .remove_cookie(&session_id, &domain, &name)
        .await
        .map_err(|e| ApiError::dependency("session_manager", e.to_string()))?;

    if removed.is_some() {
        debug!(
            session_id = %session_id,
            domain = %domain,
            cookie_name = %name,
            "Deleted cookie from session"
        );
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::not_found("Cookie not found"))
    }
}

/// Clear all cookies from a session
pub async fn clear_cookies(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .session_manager
        .clear_cookies(&session_id)
        .await
        .map_err(|e| ApiError::dependency("session_manager", e.to_string()))?;

    info!(
        session_id = %session_id,
        "Cleared all cookies from session"
    );

    Ok(StatusCode::NO_CONTENT)
}

/// List all active sessions
pub async fn list_sessions(
    State(state): State<AppState>,
    Query(query): Query<ListSessionsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let session_ids = state.session_manager.list_sessions().await;

    let limit = query.limit.unwrap_or(100);
    let limited_sessions: Vec<String> = session_ids.into_iter().take(limit).collect();

    debug!(
        session_count = limited_sessions.len(),
        limit = limit,
        "Listed active sessions"
    );

    Ok(Json(limited_sessions))
}

/// Get session statistics
pub async fn get_session_stats(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let stats = state
        .session_manager
        .get_stats()
        .await
        .map_err(|e| ApiError::dependency("session_manager", e.to_string()))?;

    debug!(
        total_sessions = stats.total_sessions,
        expired_cleaned = stats.expired_sessions_cleaned,
        "Retrieved session statistics"
    );

    Ok(Json(stats))
}

/// Cleanup expired sessions manually
pub async fn cleanup_expired_sessions(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let removed_count = state
        .session_manager
        .cleanup_expired()
        .await
        .map_err(|e| ApiError::dependency("session_manager", e.to_string()))?;

    info!(
        removed_count = removed_count,
        "Manual cleanup of expired sessions completed"
    );

    let response = serde_json::json!({
        "removed_count": removed_count,
        "message": format!("Cleaned up {} expired sessions", removed_count)
    });

    Ok(Json(response))
}

/// Extend session expiry time
#[derive(Deserialize, Debug)]
pub struct ExtendSessionRequest {
    /// Additional time in seconds
    pub additional_seconds: u64,
}

pub async fn extend_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(body): Json<ExtendSessionRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let additional_time = std::time::Duration::from_secs(body.additional_seconds);

    state
        .session_manager
        .extend_session(&session_id, additional_time)
        .await
        .map_err(|e| ApiError::dependency("session_manager", e.to_string()))?;

    info!(
        session_id = %session_id,
        additional_seconds = body.additional_seconds,
        "Extended session expiry time"
    );

    Ok(StatusCode::OK)
}
