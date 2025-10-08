use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::state::AppState;

/// Authentication configuration
#[derive(Clone)]
pub struct AuthConfig {
    /// Valid API keys for authentication
    valid_api_keys: Arc<RwLock<HashSet<String>>>,
    /// Whether to require authentication (can be disabled for development)
    require_auth: bool,
    /// Paths that don't require authentication
    public_paths: Arc<Vec<String>>,
}

impl AuthConfig {
    /// Create new authentication configuration
    pub fn new() -> Self {
        let valid_keys = std::env::var("API_KEYS")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string())
            .collect();

        let require_auth = std::env::var("REQUIRE_AUTH")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true); // Authentication required by default

        let public_paths = vec![
            "/health".to_string(),
            "/metrics".to_string(),
            "/api/v1/health".to_string(),
            "/api/v1/metrics".to_string(),
        ];

        Self {
            valid_api_keys: Arc::new(RwLock::new(valid_keys)),
            require_auth,
            public_paths: Arc::new(public_paths),
        }
    }

    /// Create configuration with custom API keys
    /// Reserved for future API key authentication features
    #[allow(dead_code)]
    pub fn with_api_keys(api_keys: Vec<String>) -> Self {
        Self {
            valid_api_keys: Arc::new(RwLock::new(api_keys.into_iter().collect())),
            require_auth: true,
            public_paths: Arc::new(vec![
                "/health".to_string(),
                "/metrics".to_string(),
                "/api/v1/health".to_string(),
                "/api/v1/metrics".to_string(),
            ]),
        }
    }

    /// Add a valid API key
    /// Reserved for future API key authentication features
    #[allow(dead_code)]
    pub async fn add_api_key(&self, key: String) {
        let mut keys = self.valid_api_keys.write().await;
        keys.insert(key);
    }

    /// Remove an API key
    /// Reserved for future API key authentication features
    #[allow(dead_code)]
    pub async fn remove_api_key(&self, key: &str) {
        let mut keys = self.valid_api_keys.write().await;
        keys.remove(key);
    }

    /// Check if a key is valid
    pub async fn is_valid_key(&self, key: &str) -> bool {
        let keys = self.valid_api_keys.read().await;
        keys.contains(key)
    }

    /// Check if authentication is required
    pub fn requires_auth(&self) -> bool {
        self.require_auth
    }

    /// Check if path is public (doesn't require auth)
    pub fn is_public_path(&self, path: &str) -> bool {
        self.public_paths.iter().any(|p| path.starts_with(p))
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Authentication middleware that validates API keys
///
/// This middleware checks for API keys in the X-API-Key header and validates
/// them against the configured set of valid keys.
///
/// ## Configuration
/// - `API_KEYS`: Comma-separated list of valid API keys (env var)
/// - `REQUIRE_AUTH`: Whether to require authentication (default: true)
///
/// ## Public Paths
/// The following paths don't require authentication:
/// - /health
/// - /metrics
/// - /api/v1/health
/// - /api/v1/metrics
pub async fn auth_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let path = request.uri().path();

    debug!(path = %path, "Authentication check");

    // Check if this is a public path
    if state.auth_config.is_public_path(path) {
        debug!(path = %path, "Public path, skipping authentication");
        return Ok(next.run(request).await);
    }

    // Check if authentication is required
    if !state.auth_config.requires_auth() {
        debug!("Authentication disabled, allowing request");
        return Ok(next.run(request).await);
    }

    // Extract API key from header
    let api_key = match extract_api_key(&request) {
        Some(key) => key,
        None => {
            warn!(path = %path, "Missing API key");
            return Err(unauthorized_response("Missing API key"));
        }
    };

    // Validate API key
    if !state.auth_config.is_valid_key(&api_key).await {
        warn!(path = %path, "Invalid API key");
        return Err(unauthorized_response("Invalid API key"));
    }

    debug!(path = %path, "Authentication successful");

    // Proceed with the request
    Ok(next.run(request).await)
}

/// Extract API key from request headers
fn extract_api_key(request: &Request) -> Option<String> {
    // Try X-API-Key header
    if let Some(api_key) = request
        .headers()
        .get("X-API-Key")
        .and_then(|h| h.to_str().ok())
    {
        return Some(api_key.to_string());
    }

    // Try Authorization header with "Bearer" scheme
    if let Some(auth_header) = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
    {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            return Some(token.to_string());
        }
    }

    None
}

/// Create unauthorized response
fn unauthorized_response(message: &str) -> Response {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("Content-Type", "application/json")
        .header("WWW-Authenticate", "Bearer")
        .body(Body::from(
            serde_json::json!({
                "error": "Unauthorized",
                "message": message,
            })
            .to_string(),
        ))
        .unwrap()
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    #[test]
    fn test_extract_api_key_from_header() {
        // Test X-API-Key header
        let request = Request::builder()
            .header("X-API-Key", "test-key-123")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_api_key(&request), Some("test-key-123".to_string()));

        // Test Authorization Bearer header
        let request = Request::builder()
            .header("Authorization", "Bearer token-456")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_api_key(&request), Some("token-456".to_string()));

        // Test missing headers
        let request = Request::builder().body(Body::empty()).unwrap();
        assert_eq!(extract_api_key(&request), None);
    }

    #[tokio::test]
    async fn test_auth_config() {
        let config = AuthConfig::with_api_keys(vec!["key1".to_string(), "key2".to_string()]);

        assert!(config.is_valid_key("key1").await);
        assert!(config.is_valid_key("key2").await);
        assert!(!config.is_valid_key("key3").await);

        // Test add/remove
        config.add_api_key("key3".to_string()).await;
        assert!(config.is_valid_key("key3").await);

        config.remove_api_key("key3").await;
        assert!(!config.is_valid_key("key3").await);
    }

    #[test]
    fn test_public_paths() {
        let config = AuthConfig::new();

        assert!(config.is_public_path("/health"));
        assert!(config.is_public_path("/metrics"));
        assert!(config.is_public_path("/api/v1/health"));
        assert!(!config.is_public_path("/api/v1/extract"));
    }
}
