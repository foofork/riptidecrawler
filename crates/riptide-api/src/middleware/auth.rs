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

/// Constant-time string comparison to prevent timing attacks.
///
/// This function compares two strings in constant time, ensuring that
/// the comparison time does not leak information about the strings' contents.
/// This is critical for API key validation to prevent timing-based attacks
/// where an attacker could determine the correct key character by character
/// by measuring response times.
///
/// # Security
///
/// - Always compares all bytes regardless of early mismatches
/// - Uses bitwise OR to accumulate differences
/// - Returns only after comparing the entire length
///
/// # Arguments
///
/// * `a` - First string to compare (e.g., provided API key)
/// * `b` - Second string to compare (e.g., valid API key)
///
/// # Returns
///
/// `true` if the strings are equal, `false` otherwise
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (byte_a, byte_b) in a.bytes().zip(b.bytes()) {
        result |= byte_a ^ byte_b;
    }

    result == 0
}

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
            "/healthz".to_string(), // Kubernetes-style health check
            "/metrics".to_string(),
            "/api/v1/health".to_string(),
            "/api/v1/metrics".to_string(),
            "/api/health/detailed".to_string(),
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
                "/healthz".to_string(), // Kubernetes-style health check
                "/metrics".to_string(),
                "/api/v1/health".to_string(),
                "/api/v1/metrics".to_string(),
                "/api/health/detailed".to_string(),
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

    /// Check if a key is valid using constant-time comparison.
    ///
    /// This method performs a constant-time comparison against all valid API keys
    /// to prevent timing attacks. The comparison time does not leak information
    /// about which characters in the key are correct.
    ///
    /// # Security
    ///
    /// - Uses constant-time comparison for each key
    /// - Always checks all keys to prevent early exit timing leaks
    /// - Does not short-circuit on first match
    ///
    /// # Arguments
    ///
    /// * `key` - The API key to validate
    ///
    /// # Returns
    ///
    /// `true` if the key matches any valid key, `false` otherwise
    pub async fn is_valid_key(&self, key: &str) -> bool {
        let keys = self.valid_api_keys.read().await;

        // Use constant-time comparison to prevent timing attacks
        // We check all keys even after finding a match to maintain constant timing
        let mut is_valid = false;
        for valid_key in keys.iter() {
            if constant_time_compare(key, valid_key) {
                is_valid = true;
                // Continue checking to maintain constant time
            }
        }

        is_valid
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
/// - /health - Health check endpoint
/// - /healthz - Kubernetes-style health check
/// - /metrics - Prometheus metrics endpoint
/// - /api/v1/health - Versioned health endpoint
/// - /api/v1/metrics - Versioned metrics endpoint
/// - /api/health/detailed - Detailed health diagnostics
///
/// ## Security Features
/// - **Constant-time comparison**: API key validation uses constant-time comparison
///   to prevent timing attacks that could leak information about valid keys
/// - **Secure headers**: Failed authentication includes WWW-Authenticate header
/// - **Audit logging**: All authentication attempts are logged for security monitoring
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

    // Validate API key using constant-time comparison
    if !state.auth_config.is_valid_key(&api_key).await {
        warn!(
            path = %path,
            key_prefix = &api_key.chars().take(4).collect::<String>(),
            "Invalid API key - authentication failed"
        );
        return Err(unauthorized_response("Invalid API key"));
    }

    debug!(
        path = %path,
        key_prefix = &api_key.chars().take(4).collect::<String>(),
        "Authentication successful"
    );

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
        .unwrap_or_else(|_| {
            Response::new(Body::from(
                r#"{"error":"Unauthorized","message":"Failed to build response"}"#,
            ))
        })
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    #[test]
    fn test_constant_time_compare() {
        // Test equal strings
        assert!(constant_time_compare("secret123", "secret123"));
        assert!(constant_time_compare("", ""));

        // Test unequal strings of same length
        assert!(!constant_time_compare("secret123", "secret124"));
        assert!(!constant_time_compare("aaaa", "aaab"));

        // Test unequal strings of different lengths
        assert!(!constant_time_compare("secret", "secret123"));
        assert!(!constant_time_compare("secret123", "secret"));

        // Test with special characters
        assert!(constant_time_compare(
            "key-with-dash_123",
            "key-with-dash_123"
        ));
        assert!(!constant_time_compare(
            "key-with-dash_123",
            "key-with-dash_124"
        ));
    }

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

        // Test valid keys with constant-time comparison
        assert!(config.is_valid_key("key1").await);
        assert!(config.is_valid_key("key2").await);
        assert!(!config.is_valid_key("key3").await);

        // Test invalid keys (should not match)
        assert!(!config.is_valid_key("key").await); // Shorter
        assert!(!config.is_valid_key("key11").await); // One char different
        assert!(!config.is_valid_key("KEY1").await); // Case sensitive

        // Test add/remove
        config.add_api_key("key3".to_string()).await;
        assert!(config.is_valid_key("key3").await);

        config.remove_api_key("key3").await;
        assert!(!config.is_valid_key("key3").await);
    }

    #[test]
    fn test_public_paths() {
        let config = AuthConfig::new();

        // Test all public paths
        assert!(config.is_public_path("/health"));
        assert!(config.is_public_path("/healthz"));
        assert!(config.is_public_path("/metrics"));
        assert!(config.is_public_path("/api/v1/health"));
        assert!(config.is_public_path("/api/v1/metrics"));
        assert!(config.is_public_path("/api/health/detailed"));

        // Test protected paths
        assert!(!config.is_public_path("/api/v1/extract"));
        assert!(!config.is_public_path("/crawl"));
        assert!(!config.is_public_path("/api/v1/crawl"));
    }

    #[test]
    fn test_auth_config_from_env() {
        // Note: This test relies on environment variables not being set
        // In production, API_KEYS and REQUIRE_AUTH would be configured
        let config = AuthConfig::new();

        // Default should require auth
        assert!(config.requires_auth());

        // Should have standard public paths
        assert!(config.is_public_path("/health"));
        assert!(config.is_public_path("/healthz"));
    }
}
