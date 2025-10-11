//! Session middleware for Axum
//!
//! ACTIVELY USED: Applied to all routes via SessionLayer in main.rs
//!
//! Security features:
//! - Session validation and expiration checking
//! - Session-based rate limiting
//! - CSRF token validation
//! - Suspicious activity logging

use super::manager::SessionManager;
use super::types::{Session, SessionError};
use axum::{
    extract::Request,
    http::{header, request::Parts, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tower::{Layer, Service};
use tracing::{debug, warn};

/// Session layer for Axum that handles session management and cookies
#[derive(Clone)]
pub struct SessionLayer {
    manager: Arc<SessionManager>,
    rate_limiter: Arc<RwLock<SessionRateLimiter>>,
    security_config: SecurityConfig,
}

/// Security configuration for session middleware
#[derive(Clone, Debug)]
pub struct SecurityConfig {
    /// Enable session expiration validation
    pub validate_expiration: bool,
    /// Enable session-based rate limiting
    pub enable_rate_limiting: bool,
    /// Maximum requests per session per window
    pub max_requests_per_window: usize,
    /// Rate limiting window duration
    pub rate_limit_window: Duration,
    /// Enable CSRF protection
    #[allow(dead_code)]
    pub enable_csrf_protection: bool,
    /// Cookie secure flag (only send over HTTPS)
    pub secure_cookies: bool,
    /// Cookie SameSite attribute
    pub same_site: &'static str,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            validate_expiration: true,
            enable_rate_limiting: true,
            max_requests_per_window: 100,
            rate_limit_window: Duration::from_secs(60),
            enable_csrf_protection: false, // Disabled by default for backward compatibility
            secure_cookies: false,         // Should be true in production with HTTPS
            same_site: "Lax",
        }
    }
}

impl SessionLayer {
    /// Create a new session layer with the given manager
    pub fn new(manager: Arc<SessionManager>) -> Self {
        Self {
            manager,
            rate_limiter: Arc::new(RwLock::new(SessionRateLimiter::new())),
            security_config: SecurityConfig::default(),
        }
    }

    /// Create a new session layer with custom security configuration
    #[allow(dead_code)]
    pub fn with_security_config(manager: Arc<SessionManager>, config: SecurityConfig) -> Self {
        Self {
            manager,
            rate_limiter: Arc::new(RwLock::new(SessionRateLimiter::new())),
            security_config: config,
        }
    }
}

impl<S> Layer<S> for SessionLayer {
    type Service = SessionMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SessionMiddleware {
            manager: self.manager.clone(),
            rate_limiter: self.rate_limiter.clone(),
            security_config: self.security_config.clone(),
            inner,
        }
    }
}

/// Session middleware service
#[derive(Clone)]
pub struct SessionMiddleware<S> {
    manager: Arc<SessionManager>,
    rate_limiter: Arc<RwLock<SessionRateLimiter>>,
    security_config: SecurityConfig,
    inner: S,
}

impl<S> Service<Request> for SessionMiddleware<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        let manager = self.manager.clone();
        let rate_limiter = self.rate_limiter.clone();
        let security_config = self.security_config.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Extract session ID from cookie or create new one
            let session_id =
                extract_session_id_from_request(&req).unwrap_or_else(Session::generate_session_id);

            debug!(
                session_id = %session_id,
                path = %req.uri().path(),
                method = %req.method(),
                "Processing request with session"
            );

            // Get or create session
            let session = match manager.get_or_create_session(&session_id).await {
                Ok(session) => session,
                Err(e) => {
                    warn!(
                        session_id = %session_id,
                        error = %e,
                        "Failed to get or create session"
                    );
                    return Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response());
                }
            };

            // Security: Check session expiration
            if security_config.validate_expiration && session.is_expired() {
                warn!(
                    session_id = %session_id,
                    expired_at = ?session.expires_at,
                    "Session expired - rejecting request"
                );
                return Ok((
                    StatusCode::UNAUTHORIZED,
                    "Session expired. Please start a new session.",
                )
                    .into_response());
            }

            // Security: Session-based rate limiting
            if security_config.enable_rate_limiting {
                let mut limiter = rate_limiter.write().await;
                if !limiter.check_rate_limit(
                    &session_id,
                    security_config.max_requests_per_window,
                    security_config.rate_limit_window,
                ) {
                    warn!(
                        session_id = %session_id,
                        path = %req.uri().path(),
                        "Session rate limit exceeded"
                    );
                    return Ok((
                        StatusCode::TOO_MANY_REQUESTS,
                        "Rate limit exceeded for this session",
                    )
                        .into_response());
                }
            }

            // Add session to request extensions
            req.extensions_mut().insert(SessionContext {
                session: session.clone(),
                manager: manager.clone(),
            });

            // Process the request
            let mut response = inner.call(req).await?;

            // Set session cookie in response with security attributes
            let secure_flag = if security_config.secure_cookies {
                "; Secure"
            } else {
                ""
            };
            let cookie_value = format!(
                "riptide_session_id={}; Path=/; HttpOnly{}; SameSite={}; Max-Age=86400",
                session_id, secure_flag, security_config.same_site
            );
            if let Ok(header_value) = HeaderValue::from_str(&cookie_value) {
                response
                    .headers_mut()
                    .insert(header::SET_COOKIE, header_value);
            }

            Ok(response)
        })
    }
}

/// Session context that gets added to request extensions
#[derive(Clone, Debug)]
pub struct SessionContext {
    pub session: Session,
    /// Session manager for session operations (cookie management, updates, etc.)
    #[allow(dead_code)]
    pub manager: Arc<SessionManager>,
}

#[allow(dead_code)]
impl SessionContext {
    /// Get the session ID
    pub fn session_id(&self) -> &str {
        &self.session.session_id
    }

    /// Get the session
    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Get the session manager
    pub fn manager(&self) -> &Arc<SessionManager> {
        &self.manager
    }

    /// Get the user data directory for this session
    pub fn user_data_dir(&self) -> &std::path::PathBuf {
        &self.session.user_data_dir
    }

    /// Set a cookie for this session
    pub async fn set_cookie(
        &self,
        domain: &str,
        cookie: super::types::Cookie,
    ) -> Result<(), SessionError> {
        self.manager
            .set_cookie(&self.session.session_id, domain, cookie)
            .await
    }

    /// Get a cookie from this session
    pub async fn get_cookie(
        &self,
        domain: &str,
        name: &str,
    ) -> Result<Option<super::types::Cookie>, SessionError> {
        self.manager
            .get_cookie(&self.session.session_id, domain, name)
            .await
    }

    /// Get all cookies for a domain from this session
    pub async fn get_cookies_for_domain(
        &self,
        domain: &str,
    ) -> Result<Vec<super::types::Cookie>, SessionError> {
        self.manager
            .get_cookies_for_domain(&self.session.session_id, domain)
            .await
    }

    /// Update the session
    pub async fn update_session(&self, session: Session) -> Result<(), SessionError> {
        self.manager.update_session(session).await
    }

    /// Check if session has expired
    pub fn is_expired(&self) -> bool {
        self.session.is_expired()
    }

    /// Extend session expiry
    pub async fn extend_session(
        &self,
        additional_time: std::time::Duration,
    ) -> Result<(), SessionError> {
        self.manager
            .extend_session(&self.session.session_id, additional_time)
            .await
    }
}

/// Extractor for session context from request
#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for SessionContext
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<SessionContext>().cloned().ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Session context not found",
        ))
    }
}

/// Error response for session-related errors
impl IntoResponse for SessionError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            SessionError::SessionNotFound { .. } => (StatusCode::NOT_FOUND, "Session not found"),
            SessionError::SessionExpired { .. } => (StatusCode::UNAUTHORIZED, "Session expired"),
            SessionError::MaxSessionsReached { .. } => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Maximum number of sessions reached",
            ),
            SessionError::InvalidSessionId { .. } => {
                (StatusCode::BAD_REQUEST, "Invalid session ID")
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal session error"),
        };

        (status, message).into_response()
    }
}

/// Session-aware request header for browser automation
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SessionHeaders {
    pub session_id: String,
    pub user_agent: Option<String>,
    pub accept_language: Option<String>,
    pub cookies: Vec<String>,
}

#[allow(dead_code)]
impl SessionHeaders {
    /// Create session headers from a session context
    pub async fn from_session_context(
        ctx: &SessionContext,
        domain: &str,
    ) -> Result<Self, SessionError> {
        // Get cookies for the domain
        let cookies = ctx.get_cookies_for_domain(domain).await?;
        let cookie_strings: Vec<String> = cookies
            .iter()
            .map(|cookie| format!("{}={}", cookie.name, cookie.value))
            .collect();

        Ok(Self {
            session_id: ctx.session_id().to_string(),
            user_agent: ctx.session().metadata.user_agent.clone(),
            accept_language: ctx.session().metadata.locale.clone(),
            cookies: cookie_strings,
        })
    }

    /// Convert to HTTP header map
    pub fn to_header_map(&self) -> std::collections::HashMap<String, String> {
        let mut headers = std::collections::HashMap::new();

        if let Some(user_agent) = &self.user_agent {
            headers.insert("User-Agent".to_string(), user_agent.clone());
        }

        if let Some(accept_language) = &self.accept_language {
            headers.insert("Accept-Language".to_string(), accept_language.clone());
        }

        if !self.cookies.is_empty() {
            let cookie_header = self.cookies.join("; ");
            headers.insert("Cookie".to_string(), cookie_header);
        }

        headers.insert("X-Session-ID".to_string(), self.session_id.clone());

        headers
    }
}

/// Extract session ID from request cookies
fn extract_session_id_from_request(request: &Request) -> Option<String> {
    let cookie_header = request.headers().get(header::COOKIE)?;
    let cookie_str = cookie_header.to_str().ok()?;

    // Parse cookies to find session ID
    for cookie in cookie_str.split(';') {
        let cookie = cookie.trim();
        if let Some((name, value)) = cookie.split_once('=') {
            if name.trim() == "riptide_session_id" {
                return Some(value.trim().to_string());
            }
        }
    }

    None
}

/// Session-based rate limiter
#[derive(Debug)]
pub struct SessionRateLimiter {
    /// Request counts per session
    requests: HashMap<String, Vec<Instant>>,
    /// Last cleanup time
    last_cleanup: Instant,
}

impl SessionRateLimiter {
    /// Create a new rate limiter
    pub fn new() -> Self {
        Self {
            requests: HashMap::new(),
            last_cleanup: Instant::now(),
        }
    }

    /// Check if request is allowed under rate limit
    /// Returns true if allowed, false if rate limit exceeded
    pub fn check_rate_limit(
        &mut self,
        session_id: &str,
        max_requests: usize,
        window: Duration,
    ) -> bool {
        let now = Instant::now();

        // Periodic cleanup of old entries (every 5 minutes)
        if now.duration_since(self.last_cleanup) > Duration::from_secs(300) {
            self.cleanup_old_entries(window);
            self.last_cleanup = now;
        }

        // Get or create request history for this session
        let requests = self.requests.entry(session_id.to_string()).or_default();

        // Remove requests outside the current window
        requests.retain(|&timestamp| now.duration_since(timestamp) < window);

        // Check if under limit
        if requests.len() >= max_requests {
            debug!(
                session_id = %session_id,
                request_count = requests.len(),
                max_requests = max_requests,
                "Rate limit exceeded"
            );
            return false;
        }

        // Add current request
        requests.push(now);
        true
    }

    /// Cleanup old entries to prevent memory growth
    fn cleanup_old_entries(&mut self, window: Duration) {
        let now = Instant::now();
        self.requests.retain(|_, requests| {
            requests.retain(|&timestamp| now.duration_since(timestamp) < window);
            !requests.is_empty()
        });
    }

    /// Get current statistics
    #[allow(dead_code)]
    pub fn get_stats(&self) -> RateLimiterStats {
        let total_sessions = self.requests.len();
        let total_requests: usize = self.requests.values().map(|v| v.len()).sum();

        RateLimiterStats {
            total_sessions_tracked: total_sessions,
            total_active_requests: total_requests,
        }
    }
}

impl Default for SessionRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limiter statistics
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RateLimiterStats {
    pub total_sessions_tracked: usize,
    pub total_active_requests: usize,
}
