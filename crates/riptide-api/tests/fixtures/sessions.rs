//! Browser session test fixtures
//!
//! Provides sample session data for testing session management endpoints

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Test fixture for browser sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFixture {
    /// Unique session identifier
    pub session_id: String,
    /// URL the session is browsing
    pub url: String,
    /// Session status
    pub status: SessionStatus,
    /// Session creation timestamp
    pub created_at: String,
    /// Session expiry timestamp
    pub expires_at: String,
    /// Last activity timestamp
    pub last_activity: String,
    /// Session cookies
    pub cookies: Vec<CookieFixture>,
    /// Session metadata
    pub metadata: SessionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Active,
    Idle,
    Expired,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieFixture {
    /// Cookie domain
    pub domain: String,
    /// Cookie name
    pub name: String,
    /// Cookie value
    pub value: String,
    /// Cookie path
    pub path: String,
    /// Expiry timestamp
    pub expires: Option<String>,
    /// HTTP only flag
    pub http_only: bool,
    /// Secure flag
    pub secure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Browser type
    pub browser: String,
    /// Browser version
    pub browser_version: String,
    /// User agent string
    pub user_agent: String,
    /// Viewport size
    pub viewport: Option<Viewport>,
    /// JavaScript enabled
    pub javascript_enabled: bool,
    /// Request count
    pub request_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

/// Get default session fixtures for testing
pub fn get_default_session_fixtures() -> Vec<SessionFixture> {
    vec![
        // Active session with cookies
        SessionFixture {
            session_id: "test-session-123".to_string(),
            url: "https://example.com".to_string(),
            status: SessionStatus::Active,
            created_at: "2025-10-27T00:00:00Z".to_string(),
            expires_at: "2025-10-27T01:00:00Z".to_string(),
            last_activity: "2025-10-27T00:30:00Z".to_string(),
            cookies: vec![
                CookieFixture {
                    domain: "example.com".to_string(),
                    name: "session_token".to_string(),
                    value: "abc123def456".to_string(),
                    path: "/".to_string(),
                    expires: Some("2025-10-28T00:00:00Z".to_string()),
                    http_only: true,
                    secure: true,
                },
                CookieFixture {
                    domain: "example.com".to_string(),
                    name: "user_preferences".to_string(),
                    value: "theme=dark&lang=en".to_string(),
                    path: "/".to_string(),
                    expires: Some("2025-11-27T00:00:00Z".to_string()),
                    http_only: false,
                    secure: false,
                },
                CookieFixture {
                    domain: ".example.com".to_string(),
                    name: "analytics_id".to_string(),
                    value: "ga_xyz789".to_string(),
                    path: "/".to_string(),
                    expires: Some("2026-10-27T00:00:00Z".to_string()),
                    http_only: false,
                    secure: true,
                },
            ],
            metadata: SessionMetadata {
                browser: "chromium".to_string(),
                browser_version: "120.0.0".to_string(),
                user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
                viewport: Some(Viewport { width: 1920, height: 1080 }),
                javascript_enabled: true,
                request_count: 15,
            },
        },

        // Idle session
        SessionFixture {
            session_id: "test-session-456".to_string(),
            url: "https://test.example.com/dashboard".to_string(),
            status: SessionStatus::Idle,
            created_at: "2025-10-27T00:00:00Z".to_string(),
            expires_at: "2025-10-27T02:00:00Z".to_string(),
            last_activity: "2025-10-27T00:15:00Z".to_string(),
            cookies: vec![
                CookieFixture {
                    domain: "test.example.com".to_string(),
                    name: "auth_token".to_string(),
                    value: "xyz987uvw654".to_string(),
                    path: "/dashboard".to_string(),
                    expires: Some("2025-10-27T12:00:00Z".to_string()),
                    http_only: true,
                    secure: true,
                },
            ],
            metadata: SessionMetadata {
                browser: "chromium".to_string(),
                browser_version: "120.0.0".to_string(),
                user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36".to_string(),
                viewport: Some(Viewport { width: 1440, height: 900 }),
                javascript_enabled: true,
                request_count: 8,
            },
        },

        // Session with multiple domain cookies
        SessionFixture {
            session_id: "test-session-789".to_string(),
            url: "https://app.example.com/api/v1/data".to_string(),
            status: SessionStatus::Active,
            created_at: "2025-10-27T00:00:00Z".to_string(),
            expires_at: "2025-10-27T03:00:00Z".to_string(),
            last_activity: "2025-10-27T00:45:00Z".to_string(),
            cookies: vec![
                CookieFixture {
                    domain: "app.example.com".to_string(),
                    name: "api_key".to_string(),
                    value: "key_abcdef123456".to_string(),
                    path: "/api".to_string(),
                    expires: None,
                    http_only: true,
                    secure: true,
                },
                CookieFixture {
                    domain: ".example.com".to_string(),
                    name: "tracking_consent".to_string(),
                    value: "accepted".to_string(),
                    path: "/".to_string(),
                    expires: Some("2026-10-27T00:00:00Z".to_string()),
                    http_only: false,
                    secure: false,
                },
                CookieFixture {
                    domain: "cdn.example.com".to_string(),
                    name: "cache_id".to_string(),
                    value: "cache_123".to_string(),
                    path: "/".to_string(),
                    expires: Some("2025-10-28T00:00:00Z".to_string()),
                    http_only: false,
                    secure: true,
                },
            ],
            metadata: SessionMetadata {
                browser: "chromium".to_string(),
                browser_version: "120.0.0".to_string(),
                user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36".to_string(),
                viewport: Some(Viewport { width: 2560, height: 1440 }),
                javascript_enabled: true,
                request_count: 23,
            },
        },
    ]
}

/// Get cookies for a specific session and domain
pub fn get_session_cookies_by_domain(session_id: &str, domain: &str) -> Vec<CookieFixture> {
    let fixtures = get_default_session_fixtures();

    if let Some(session) = fixtures.iter().find(|s| s.session_id == session_id) {
        session.cookies
            .iter()
            .filter(|c| c.domain == domain || c.domain == format!(".{}", domain))
            .cloned()
            .collect()
    } else {
        vec![]
    }
}

/// Get a specific cookie by session, domain, and name
pub fn get_session_cookie(session_id: &str, domain: &str, name: &str) -> Option<CookieFixture> {
    let cookies = get_session_cookies_by_domain(session_id, domain);
    cookies.into_iter().find(|c| c.name == name)
}

/// Session statistics fixture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatsFixture {
    pub total_sessions: u32,
    pub active_sessions: u32,
    pub idle_sessions: u32,
    pub expired_sessions: u32,
    pub total_cookies: u32,
    pub average_duration_seconds: u32,
    pub peak_concurrent_sessions: u32,
}

/// Get session statistics fixture
pub fn get_session_stats_fixture() -> SessionStatsFixture {
    let fixtures = get_default_session_fixtures();

    SessionStatsFixture {
        total_sessions: fixtures.len() as u32,
        active_sessions: fixtures.iter().filter(|s| matches!(s.status, SessionStatus::Active)).count() as u32,
        idle_sessions: fixtures.iter().filter(|s| matches!(s.status, SessionStatus::Idle)).count() as u32,
        expired_sessions: 0,
        total_cookies: fixtures.iter().map(|s| s.cookies.len()).sum::<usize>() as u32,
        average_duration_seconds: 1800,
        peak_concurrent_sessions: 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_fixtures() {
        let fixtures = get_default_session_fixtures();
        assert_eq!(fixtures.len(), 3, "Should have 3 default session fixtures");

        // Verify test-session-123 exists
        let session = fixtures.iter().find(|s| s.session_id == "test-session-123");
        assert!(session.is_some(), "Should have test-session-123 fixture");
    }

    #[test]
    fn test_get_session_cookies_by_domain() {
        let cookies = get_session_cookies_by_domain("test-session-123", "example.com");
        assert!(cookies.len() >= 2, "Should find cookies for example.com domain");
    }

    #[test]
    fn test_get_session_cookie() {
        let cookie = get_session_cookie("test-session-123", "example.com", "session_token");
        assert!(cookie.is_some(), "Should find session_token cookie");

        let cookie = cookie.unwrap();
        assert_eq!(cookie.name, "session_token");
        assert_eq!(cookie.value, "abc123def456");
    }

    #[test]
    fn test_session_stats() {
        let stats = get_session_stats_fixture();
        assert_eq!(stats.total_sessions, 3);
        assert_eq!(stats.active_sessions, 2);
        assert_eq!(stats.idle_sessions, 1);
    }
}
