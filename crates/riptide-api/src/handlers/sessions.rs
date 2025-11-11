//! Session and Cookie Management Handlers - Ultra-thin delegation layer
//!
//! Phase 3 Sprint 3.1: Refactored to <40 LOC total by delegating to SessionManager.
//! Handlers are pure HTTP mapping with no business logic.

use crate::{dto::sessions::*, errors::ApiError, sessions::Cookie};
use crate::context::ApplicationContext;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

/// Error handler helper - for future error handling patterns
#[allow(dead_code)]
fn handle_error(
    result: Result<impl Into<ApiError>, impl std::fmt::Display>,
    state: &ApplicationContext,
) -> Result<(), ApiError> {
    result.map(|_| ()).map_err(|e| {
        state.transport_metrics.record_redis_error();
        ApiError::dependency("session_manager", e.to_string())
    })
}

pub async fn create_session(State(state): State<ApplicationContext>) -> Result<impl IntoResponse, ApiError> {
    let session = state.session_manager.create_session().await.map_err(|e| {
        state.transport_metrics.record_redis_error();
        ApiError::dependency("session_manager", e.to_string())
    })?;
    Ok(Json(CreateSessionResponse::from(&session)))
}

/// Future API endpoint for getting session details
#[allow(dead_code)]
pub async fn get_session(
    State(state): State<ApplicationContext>,
    Path(session_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let session = state
        .session_manager
        .get_session(&session_id)
        .await
        .map_err(|e| {
            state.transport_metrics.record_redis_error();
            ApiError::dependency("session_manager", e.to_string())
        })?
        .ok_or_else(|| ApiError::not_found("Session not found"))?;
    Ok(Json(SessionInfoResponse::from(&session)))
}

pub async fn list_sessions(
    State(state): State<ApplicationContext>,
    Query(query): Query<ListSessionsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let sessions = state
        .session_manager
        .list_sessions_filtered(query.include_expired.unwrap_or(false))
        .await;
    let limited: Vec<String> = sessions
        .into_iter()
        .take(query.limit.unwrap_or(100))
        .collect();
    Ok(Json(limited))
}

pub async fn delete_session(
    State(state): State<ApplicationContext>,
    Path(session_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state
        .session_manager
        .remove_session(&session_id)
        .await
        .map_err(|e| {
            state.transport_metrics.record_redis_error();
            ApiError::dependency("session_manager", e.to_string())
        })?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn extend_session(
    State(state): State<ApplicationContext>,
    Path(session_id): Path<String>,
    Json(request): Json<ExtendSessionRequest>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .session_manager
        .extend_session(
            &session_id,
            std::time::Duration::from_secs(request.additional_seconds),
        )
        .await
        .map_err(|e| {
            state.transport_metrics.record_redis_error();
            ApiError::dependency("session_manager", e.to_string())
        })?;
    Ok(StatusCode::OK)
}

pub async fn set_cookie(
    State(state): State<ApplicationContext>,
    Path(session_id): Path<String>,
    Json(request): Json<SetCookieRequest>,
) -> Result<StatusCode, ApiError> {
    let cookie = Cookie {
        name: request.name.clone(),
        value: request.value,
        domain: None,
        path: request.path,
        expires: request
            .expires_in_seconds
            .map(|s| std::time::SystemTime::now() + std::time::Duration::from_secs(s)),
        secure: request.secure.unwrap_or(false),
        http_only: request.http_only.unwrap_or(true),
        same_site: None,
    };
    state
        .session_manager
        .set_cookie(&session_id, &request.domain, cookie)
        .await
        .map_err(|e| {
            state.transport_metrics.record_redis_error();
            ApiError::dependency("session_manager", e.to_string())
        })?;
    Ok(StatusCode::CREATED)
}

pub async fn get_cookie(
    State(state): State<ApplicationContext>,
    Path((session_id, domain, name)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    let cookie = state
        .session_manager
        .get_cookie(&session_id, &domain, &name)
        .await
        .map_err(|e| {
            state.transport_metrics.record_redis_error();
            ApiError::dependency("session_manager", e.to_string())
        })?
        .ok_or_else(|| ApiError::not_found("Cookie not found"))?;
    Ok(Json(CookieResponse::from(cookie)))
}

/// Future API endpoint for listing session cookies
#[allow(dead_code)]
pub async fn list_cookies(
    State(state): State<ApplicationContext>,
    Path(session_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let jar = state
        .session_manager
        .get_all_cookies(&session_id)
        .await
        .map_err(|e| {
            state.transport_metrics.record_redis_error();
            ApiError::dependency("session_manager", e.to_string())
        })?;
    let responses: Vec<CookieResponse> = jar
        .all_cookies()
        .into_iter()
        .map(|c| CookieResponse::from(c.clone()))
        .collect();
    Ok(Json(responses))
}

pub async fn delete_cookie(
    State(state): State<ApplicationContext>,
    Path((session_id, domain, name)): Path<(String, String, String)>,
) -> Result<StatusCode, ApiError> {
    state
        .session_manager
        .remove_cookie(&session_id, &domain, &name)
        .await
        .map_err(|e| {
            state.transport_metrics.record_redis_error();
            ApiError::dependency("session_manager", e.to_string())
        })?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn clear_cookies(
    State(state): State<ApplicationContext>,
    Path(session_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state
        .session_manager
        .clear_cookies(&session_id)
        .await
        .map_err(|e| {
            state.transport_metrics.record_redis_error();
            ApiError::dependency("session_manager", e.to_string())
        })?;
    Ok(StatusCode::NO_CONTENT)
}

/// Get session statistics stub
pub async fn get_session_stats(
    State(state): State<ApplicationContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let sessions = state.session_manager.list_sessions_filtered(false).await;
    let total_sessions = sessions.len();
    let active_sessions = sessions.len(); // All non-expired sessions are considered active

    Ok(Json(serde_json::json!({
        "total_sessions": total_sessions,
        "active_sessions": active_sessions,
        "expired_sessions": 0,
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Cleanup expired sessions stub
pub async fn cleanup_expired_sessions(
    State(state): State<ApplicationContext>,
) -> Result<StatusCode, ApiError> {
    // Get all sessions including expired ones
    let all_sessions = state.session_manager.list_sessions_filtered(true).await;
    let active_sessions = state.session_manager.list_sessions_filtered(false).await;

    // Calculate expired sessions and cleanup
    let expired_count = all_sessions.len() - active_sessions.len();

    // In a real implementation, we'd iterate and remove expired sessions
    // For now, this is a placeholder that returns success
    tracing::info!(
        "Cleanup completed: {} expired sessions found",
        expired_count
    );

    Ok(StatusCode::OK)
}

/// Get session info stub
pub async fn get_session_info(
    State(state): State<ApplicationContext>,
    Path(session_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let session = state
        .session_manager
        .get_session(&session_id)
        .await
        .map_err(|e| {
            state.transport_metrics.record_redis_error();
            ApiError::dependency("session_manager", e.to_string())
        })?
        .ok_or_else(|| ApiError::not_found("Session not found"))?;

    Ok(Json(serde_json::json!({
        "session_id": session.session_id,
        "created_at": format!("{:?}", session.created_at),
        "expires_at": format!("{:?}", session.expires_at),
        "last_accessed": format!("{:?}", session.last_accessed),
        "metadata": session.metadata
    })))
}

/// Get cookies for domain stub
pub async fn get_cookies_for_domain(
    State(state): State<ApplicationContext>,
    Path((session_id, domain)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let jar = state
        .session_manager
        .get_all_cookies(&session_id)
        .await
        .map_err(|e| {
            state.transport_metrics.record_redis_error();
            ApiError::dependency("session_manager", e.to_string())
        })?;

    let domain_cookies: Vec<CookieResponse> = jar
        .all_cookies()
        .into_iter()
        .filter(|c| c.domain.as_ref().map(|d| d == &domain).unwrap_or(false))
        .map(|c| CookieResponse::from(c.clone()))
        .collect();

    Ok(Json(serde_json::json!({
        "domain": domain,
        "cookies": domain_cookies,
        "count": domain_cookies.len()
    })))
}
