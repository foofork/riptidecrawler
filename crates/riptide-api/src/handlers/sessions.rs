//! Session and Cookie Management Handlers - Ultra-thin delegation layer
//!
//! Phase 3 Sprint 3.1: Refactored to <40 LOC total by delegating to SessionManager.
//! Handlers are pure HTTP mapping with no business logic.

use crate::{
    dto::sessions::*, errors::ApiError, metrics::ErrorType, sessions::Cookie, state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

/// Error handler helper
fn handle_error(
    result: Result<impl Into<ApiError>, impl std::fmt::Display>,
    state: &AppState,
) -> Result<(), ApiError> {
    result.map(|_| ()).map_err(|e| {
        state.metrics.record_error(ErrorType::Redis);
        ApiError::dependency("session_manager", e.to_string())
    })
}

pub async fn create_session(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let session = state.session_manager.create_session().await.map_err(|e| {
        state.metrics.record_error(ErrorType::Redis);
        ApiError::dependency("session_manager", e.to_string())
    })?;
    Ok(Json(CreateSessionResponse::from(&session)))
}

pub async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let session = state
        .session_manager
        .get_session(&session_id)
        .await
        .map_err(|e| {
            state.metrics.record_error(ErrorType::Redis);
            ApiError::dependency("session_manager", e.to_string())
        })?
        .ok_or_else(|| ApiError::not_found("Session not found"))?;
    Ok(Json(SessionInfoResponse::from(&session)))
}

pub async fn list_sessions(
    State(state): State<AppState>,
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
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state
        .session_manager
        .remove_session(&session_id)
        .await
        .map_err(|e| {
            state.metrics.record_error(ErrorType::Redis);
            ApiError::dependency("session_manager", e.to_string())
        })?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn extend_session(
    State(state): State<AppState>,
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
            state.metrics.record_error(ErrorType::Redis);
            ApiError::dependency("session_manager", e.to_string())
        })?;
    Ok(StatusCode::OK)
}

pub async fn set_cookie(
    State(state): State<AppState>,
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
            state.metrics.record_error(ErrorType::Redis);
            ApiError::dependency("session_manager", e.to_string())
        })?;
    Ok(StatusCode::CREATED)
}

pub async fn get_cookie(
    State(state): State<AppState>,
    Path((session_id, domain, name)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    let cookie = state
        .session_manager
        .get_cookie(&session_id, &domain, &name)
        .await
        .map_err(|e| {
            state.metrics.record_error(ErrorType::Redis);
            ApiError::dependency("session_manager", e.to_string())
        })?
        .ok_or_else(|| ApiError::not_found("Cookie not found"))?;
    Ok(Json(CookieResponse::from(cookie)))
}

pub async fn list_cookies(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let jar = state
        .session_manager
        .get_all_cookies(&session_id)
        .await
        .map_err(|e| {
            state.metrics.record_error(ErrorType::Redis);
            ApiError::dependency("session_manager", e.to_string())
        })?;
    let responses: Vec<CookieResponse> = jar
        .all_cookies()
        .into_iter()
        .map(CookieResponse::from)
        .collect();
    Ok(Json(responses))
}

pub async fn delete_cookie(
    State(state): State<AppState>,
    Path((session_id, domain, name)): Path<(String, String, String)>,
) -> Result<StatusCode, ApiError> {
    state
        .session_manager
        .remove_cookie(&session_id, &domain, &name)
        .await
        .map_err(|e| {
            state.metrics.record_error(ErrorType::Redis);
            ApiError::dependency("session_manager", e.to_string())
        })?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn clear_cookies(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state
        .session_manager
        .clear_cookies(&session_id)
        .await
        .map_err(|e| {
            state.metrics.record_error(ErrorType::Redis);
            ApiError::dependency("session_manager", e.to_string())
        })?;
    Ok(StatusCode::NO_CONTENT)
}
