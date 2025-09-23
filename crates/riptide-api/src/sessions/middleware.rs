use super::manager::SessionManager;
use super::types::{Session, SessionError};
use axum::{
    extract::{FromRequestParts, Request, State},
    http::{header, request::Parts, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tracing::{debug, warn};
use uuid::Uuid;

/// Session middleware for Axum that handles session management and cookies
pub struct SessionMiddleware {
    manager: Arc<SessionManager>,
}

impl SessionMiddleware {
    /// Create a new session middleware with the given manager
    pub fn new(manager: Arc<SessionManager>) -> Self {
        Self { manager }
    }

    /// Middleware function for session handling
    pub async fn session_middleware(
        State(session_manager): State<Arc<SessionManager>>,
        mut request: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        // Extract session ID from cookie or create new one
        let session_id = extract_session_id_from_request(&request)
            .unwrap_or_else(|| Session::generate_session_id());

        debug!(
            session_id = %session_id,
            path = %request.uri().path(),
            method = %request.method(),
            "Processing request with session"
        );

        // Get or create session
        let session = match session_manager.get_or_create_session(&session_id).await {
            Ok(session) => session,
            Err(e) => {
                warn!(
                    session_id = %session_id,
                    error = %e,
                    "Failed to get or create session"
                );
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        // Add session to request extensions
        request.extensions_mut().insert(SessionContext {
            session: session.clone(),
            manager: session_manager.clone(),
        });

        // Process the request
        let mut response = next.run(request).await;

        // Set session cookie in response
        let cookie_value = format!("riptide_session_id={}; Path=/; HttpOnly; Max-Age=86400", session_id);
        if let Ok(header_value) = HeaderValue::from_str(&cookie_value) {
            response.headers_mut().insert(header::SET_COOKIE, header_value);
        }

        Ok(response)
    }
}

/// Session context that gets added to request extensions
#[derive(Clone)]
pub struct SessionContext {
    pub session: Session,
    pub manager: Arc<SessionManager>,
}

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
}

/// Extractor for session context from request
#[axum::async_trait]
impl<S> FromRequestParts<S> for SessionContext
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<SessionContext>()
            .cloned()
            .ok_or((
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
            SessionError::InvalidSessionId { .. } => (StatusCode::BAD_REQUEST, "Invalid session ID"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal session error"),
        };

        (status, message).into_response()
    }
}

/// Session-aware request header for browser automation
#[derive(Debug, Clone)]
pub struct SessionHeaders {
    pub session_id: String,
    pub user_agent: Option<String>,
    pub accept_language: Option<String>,
    pub cookies: Vec<String>,
}

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

/// Helper function to create session middleware layer
pub fn create_session_layer(
    manager: Arc<SessionManager>,
) -> axum::middleware::FromFnLayer<
    impl Fn(
        axum::extract::State<Arc<SessionManager>>,
        Request,
        Next,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Response, StatusCode>>
                + Send
                + 'static,
        >,
    > + Clone,
    Arc<SessionManager>,
> {
    axum::middleware::from_fn_with_state(manager, SessionMiddleware::session_middleware)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sessions::{SessionConfig, SessionManager};
    use axum::{
        body::Body,
        http::{HeaderMap, Method, Request},
    };
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_session_context_creation() {
        let config = SessionConfig::default();
        let manager = Arc::new(SessionManager::new(config).await.unwrap());
        let session = manager.create_session().await.unwrap();

        let context = SessionContext {
            session: session.clone(),
            manager: manager.clone(),
        };

        assert_eq!(context.session_id(), &session.session_id);
        assert_eq!(context.session().session_id, session.session_id);
    }

    #[tokio::test]
    async fn test_session_headers_creation() {
        let config = SessionConfig::default();
        let manager = Arc::new(SessionManager::new(config).await.unwrap());
        let session = manager.create_session().await.unwrap();

        let context = SessionContext {
            session: session.clone(),
            manager: manager.clone(),
        };

        // Set a test cookie
        let cookie = super::super::types::Cookie::new("test_cookie".to_string(), "test_value".to_string());
        context.set_cookie("example.com", cookie).await.unwrap();

        // Create session headers
        let headers = SessionHeaders::from_session_context(&context, "example.com")
            .await
            .unwrap();

        assert_eq!(headers.session_id, session.session_id);
        assert_eq!(headers.cookies.len(), 1);
        assert_eq!(headers.cookies[0], "test_cookie=test_value");

        // Convert to header map
        let header_map = headers.to_header_map();
        assert!(header_map.contains_key("X-Session-ID"));
        assert!(header_map.contains_key("Cookie"));
        assert_eq!(header_map["Cookie"], "test_cookie=test_value");
    }

    #[test]
    fn test_extract_session_id_from_request() {
        // Test with valid session cookie
        let mut request = Request::new(Body::empty());
        request.headers_mut().insert(
            header::COOKIE,
            HeaderValue::from_static("riptide_session_id=test_session_123; other_cookie=value"),
        );

        let session_id = extract_session_id_from_request(&request);
        assert_eq!(session_id, Some("test_session_123".to_string()));

        // Test with no cookies
        let request = Request::new(Body::empty());
        let session_id = extract_session_id_from_request(&request);
        assert_eq!(session_id, None);

        // Test with wrong cookie name
        let mut request = Request::new(Body::empty());
        request.headers_mut().insert(
            header::COOKIE,
            HeaderValue::from_static("other_session_id=test_session_123"),
        );

        let session_id = extract_session_id_from_request(&request);
        assert_eq!(session_id, None);
    }
}