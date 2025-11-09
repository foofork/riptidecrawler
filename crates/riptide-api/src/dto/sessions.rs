//! Sessions API DTOs - Request/Response types for session and cookie management
//!
//! Extracted from handlers/sessions.rs (Phase 3 Sprint 3.1)
//! Contains all 7 DTOs + conversion traits

use crate::sessions::{Cookie, Session};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize, Debug)]
pub struct SetCookieRequest {
    pub domain: String,
    pub name: String,
    pub value: String,
    pub path: Option<String>,
    pub expires_in_seconds: Option<u64>,
    pub secure: Option<bool>,
    pub http_only: Option<bool>,
}

#[derive(Serialize, Debug)]
pub struct CreateSessionResponse {
    pub session_id: String,
    pub user_data_dir: String,
    pub created_at: String,
    pub expires_at: String,
}

impl From<&Session> for CreateSessionResponse {
    fn from(session: &Session) -> Self {
        Self {
            session_id: session.session_id.clone(),
            user_data_dir: session.user_data_dir.to_string_lossy().to_string(),
            created_at: format_timestamp(session.created_at),
            expires_at: format_timestamp(session.expires_at),
        }
    }
}

/// Session info response - for future session management API
#[allow(dead_code)]
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

impl From<&Session> for SessionInfoResponse {
    fn from(session: &Session) -> Self {
        Self {
            session_id: session.session_id.clone(),
            user_data_dir: session.user_data_dir.to_string_lossy().to_string(),
            created_at: format_timestamp(session.created_at),
            last_accessed: format_timestamp(session.last_accessed),
            expires_at: format_timestamp(session.expires_at),
            cookie_count: session.cookies.values().map(|v| v.len()).sum(),
            total_domains: session.cookies.len(),
        }
    }
}

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

impl From<Cookie> for CookieResponse {
    fn from(cookie: Cookie) -> Self {
        Self {
            name: cookie.name,
            value: cookie.value,
            domain: cookie.domain,
            path: cookie.path,
            expires: cookie.expires.map(|exp| format_timestamp(exp)),
            secure: cookie.secure,
            http_only: cookie.http_only,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct ListSessionsQuery {
    pub include_expired: Option<bool>,
    pub limit: Option<usize>,
}

#[derive(Deserialize, Debug)]
pub struct ExtendSessionRequest {
    pub additional_seconds: u64,
}

// Helper function to format SystemTime as Unix timestamp string
fn format_timestamp(time: SystemTime) -> String {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
