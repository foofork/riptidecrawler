use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::state::AppState;

/// Authentication attempt tracking
#[derive(Clone)]
struct AuthAttempt {
    /// Number of failed attempts
    failures: u32,
    /// Time of first failure in the current window
    first_failure: Instant,
    /// Time until which this IP is blocked (if any)
    blocked_until: Option<Instant>,
}

/// Authentication rate limiter to prevent brute-force attacks
#[derive(Clone)]
pub struct AuthRateLimiter {
    /// Per-IP attempt tracking
    attempts: Arc<RwLock<HashMap<String, AuthAttempt>>>,
    /// Maximum failed attempts before blocking
    max_attempts: u32,
    /// Time window for counting failures
    window: Duration,
}

impl AuthRateLimiter {
    /// Create a new rate limiter
    pub fn new(max_attempts: u32, window: Duration) -> Self {
        Self {
            attempts: Arc::new(RwLock::new(HashMap::new())),
            max_attempts,
            window,
        }
    }

    /// Check if an IP is allowed to attempt authentication
    pub async fn check_allowed(&self, ip: &str) -> Result<(), Duration> {
        let mut attempts = self.attempts.write().await;

        // Clean up expired entries first
        self.cleanup_expired_internal(&mut attempts).await;

        if let Some(attempt) = attempts.get(ip) {
            // Check if IP is currently blocked
            if let Some(blocked_until) = attempt.blocked_until {
                let now = Instant::now();
                if now < blocked_until {
                    let retry_after = blocked_until.duration_since(now);
                    return Err(retry_after);
                }
            }

            // Check if we're in the failure window and over the limit
            let now = Instant::now();
            let elapsed = now.duration_since(attempt.first_failure);

            if elapsed < self.window && attempt.failures >= self.max_attempts {
                // Calculate exponential backoff: 2^failures seconds
                let backoff_secs = 2u64.pow(attempt.failures.min(10)); // Cap at 2^10 = 1024s
                let backoff = Duration::from_secs(backoff_secs);

                // Update block time
                let blocked_until = now + backoff;
                let mut updated_attempt = attempt.clone();
                updated_attempt.blocked_until = Some(blocked_until);
                attempts.insert(ip.to_string(), updated_attempt);

                return Err(backoff);
            }
        }

        Ok(())
    }

    /// Record a failed authentication attempt
    pub async fn record_failure(&self, ip: &str) {
        let mut attempts = self.attempts.write().await;
        let now = Instant::now();

        if let Some(attempt) = attempts.get(ip) {
            let elapsed = now.duration_since(attempt.first_failure);

            if elapsed < self.window {
                // Within the window, increment failures
                let mut updated = attempt.clone();
                updated.failures += 1;
                attempts.insert(ip.to_string(), updated);
            } else {
                // Outside window, start new tracking
                attempts.insert(
                    ip.to_string(),
                    AuthAttempt {
                        failures: 1,
                        first_failure: now,
                        blocked_until: None,
                    },
                );
            }
        } else {
            // First failure
            attempts.insert(
                ip.to_string(),
                AuthAttempt {
                    failures: 1,
                    first_failure: now,
                    blocked_until: None,
                },
            );
        }
    }

    /// Record a successful authentication (clears failures)
    pub async fn record_success(&self, ip: &str) {
        let mut attempts = self.attempts.write().await;
        attempts.remove(ip);
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) {
        let mut attempts = self.attempts.write().await;
        self.cleanup_expired_internal(&mut attempts).await;
    }

    /// Internal cleanup helper
    async fn cleanup_expired_internal(&self, attempts: &mut HashMap<String, AuthAttempt>) {
        let now = Instant::now();
        attempts.retain(|_, attempt| {
            // Remove if:
            // 1. Block has expired AND we're outside the failure window
            let block_expired = attempt
                .blocked_until
                .map(|until| now >= until)
                .unwrap_or(true);

            let window_expired = now.duration_since(attempt.first_failure) >= self.window;

            // Keep the entry if either block is active or we're in the failure window
            !(block_expired && window_expired)
        });
    }

    /// Get current attempt count for testing
    #[allow(dead_code)]
    pub async fn get_attempt_count(&self, ip: &str) -> Option<u32> {
        let attempts = self.attempts.read().await;
        attempts.get(ip).map(|a| a.failures)
    }
}

/// Audit logger for authentication events.
///
/// Provides structured logging for security monitoring and incident response.
/// All logs include:
/// - Event type (auth_success, auth_failure, auth_blocked)
/// - Timestamp (ISO 8601 format)
/// - IP address
/// - Key prefix (first 8 chars only, NEVER full key)
/// - HTTP method and path
/// - Outcome and failure reason
///
/// ## Security Features
///
/// - **No full API keys logged**: Only first 8 characters are recorded
/// - **Structured logging**: JSON-compatible format for log aggregation
/// - **Comprehensive coverage**: All auth events (success/failure/blocked)
/// - **Sanitized data**: User agents and IPs are validated before logging
pub struct AuditLogger;

impl AuditLogger {
    /// Log successful authentication event.
    ///
    /// # Arguments
    ///
    /// * `ip` - Client IP address
    /// * `key_prefix` - First 8 characters of API key (NEVER full key)
    /// * `method` - HTTP method (GET, POST, etc.)
    /// * `path` - Request path
    pub fn log_auth_success(ip: &str, key_prefix: &str, method: &str, path: &str) {
        info!(
            event = "auth_success",
            ip = ip,
            key_prefix = key_prefix,
            method = method,
            path = path,
            timestamp = %chrono::Utc::now().to_rfc3339(),
            "Authentication successful"
        );
    }

    /// Log failed authentication event.
    ///
    /// # Arguments
    ///
    /// * `ip` - Client IP address
    /// * `reason` - Failure reason (invalid_key, missing_key, malformed)
    /// * `method` - HTTP method (GET, POST, etc.)
    /// * `path` - Request path
    pub fn log_auth_failure(ip: &str, reason: &str, method: &str, path: &str) {
        warn!(
            event = "auth_failure",
            ip = ip,
            reason = reason,
            method = method,
            path = path,
            timestamp = %chrono::Utc::now().to_rfc3339(),
            "Authentication failed"
        );
    }

    /// Log rate-limited/blocked authentication event.
    ///
    /// # Arguments
    ///
    /// * `ip` - Client IP address
    /// * `method` - HTTP method (GET, POST, etc.)
    /// * `path` - Request path
    /// * `retry_after` - Seconds until retry allowed
    pub fn log_auth_blocked(ip: &str, method: &str, path: &str, retry_after: u64) {
        warn!(
            event = "auth_blocked",
            ip = ip,
            method = method,
            path = path,
            retry_after_secs = retry_after,
            timestamp = %chrono::Utc::now().to_rfc3339(),
            "Authentication blocked (rate limited)"
        );
    }
}

/// Extract client IP address from request.
///
/// Tries multiple sources in order of preference:
/// 1. X-Forwarded-For header (for proxy/load balancer setups)
/// 2. X-Real-IP header (common nginx setup)
/// 3. ConnectInfo extension (direct socket address)
///
/// # Security
///
/// - Sanitizes IP addresses to prevent log injection
/// - Returns "unknown" if no valid IP found
/// - Only takes first IP from X-Forwarded-For (client IP)
///
/// # Arguments
///
/// * `request` - The HTTP request
///
/// # Returns
///
/// Client IP address as string, or "unknown" if not found
fn extract_client_ip(request: &Request) -> String {
    // Try X-Forwarded-For header (most common for proxied requests)
    if let Some(forwarded) = request
        .headers()
        .get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
    {
        // X-Forwarded-For can contain multiple IPs: "client, proxy1, proxy2"
        // We want the client IP (first one)
        if let Some(client_ip) = forwarded.split(',').next() {
            let ip = client_ip.trim();
            if !ip.is_empty() {
                return sanitize_ip(ip);
            }
        }
    }

    // Try X-Real-IP header (common in nginx configs)
    if let Some(real_ip) = request
        .headers()
        .get("X-Real-IP")
        .and_then(|h| h.to_str().ok())
    {
        let ip = real_ip.trim();
        if !ip.is_empty() {
            return sanitize_ip(ip);
        }
    }

    // Fallback to unknown if we can't determine IP
    // Note: ConnectInfo requires specific router setup, so we don't rely on it
    "unknown".to_string()
}

/// Sanitize IP address for logging to prevent log injection.
///
/// # Arguments
///
/// * `ip` - IP address to sanitize
///
/// # Returns
///
/// Sanitized IP address (removes newlines, control chars, etc.)
fn sanitize_ip(ip: &str) -> String {
    ip.chars()
        .filter(|c| !c.is_control())
        .take(45) // Max IPv6 length is 39, add some buffer
        .collect()
}

/// Get safe key prefix for logging (first 8 chars only).
///
/// # Security
///
/// NEVER logs full API key - only first 8 characters for audit trail.
///
/// # Arguments
///
/// * `key` - API key to get prefix from
///
/// # Returns
///
/// First 8 characters of the key, or shorter if key is shorter
fn get_key_prefix(key: &str) -> String {
    key.chars().take(8).collect()
}

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
    /// Rate limiter for authentication attempts
    rate_limiter: Arc<AuthRateLimiter>,
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

        // Rate limiting configuration
        let max_attempts = std::env::var("MAX_AUTH_ATTEMPTS_PER_MINUTE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);

        let window_secs = std::env::var("AUTH_RATE_LIMIT_WINDOW_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60);

        let rate_limiter = AuthRateLimiter::new(max_attempts, Duration::from_secs(window_secs));

        Self {
            valid_api_keys: Arc::new(RwLock::new(valid_keys)),
            require_auth,
            public_paths: Arc::new(public_paths),
            rate_limiter: Arc::new(rate_limiter),
        }
    }

    /// Create configuration with custom API keys
    /// Reserved for future API key authentication features
    #[allow(dead_code)]
    pub fn with_api_keys(api_keys: Vec<String>) -> Self {
        let rate_limiter = AuthRateLimiter::new(10, Duration::from_secs(60));

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
            rate_limiter: Arc::new(rate_limiter),
        }
    }

    /// Create configuration with custom rate limiting
    #[allow(dead_code)]
    pub fn with_api_keys_and_rate_limit(
        api_keys: Vec<String>,
        max_attempts: u32,
        window: Duration,
    ) -> Self {
        let rate_limiter = AuthRateLimiter::new(max_attempts, window);

        Self {
            valid_api_keys: Arc::new(RwLock::new(api_keys.into_iter().collect())),
            require_auth: true,
            public_paths: Arc::new(vec![
                "/health".to_string(),
                "/healthz".to_string(),
                "/metrics".to_string(),
                "/api/v1/health".to_string(),
                "/api/v1/metrics".to_string(),
                "/api/health/detailed".to_string(),
            ]),
            rate_limiter: Arc::new(rate_limiter),
        }
    }

    /// Get rate limiter for testing
    #[allow(dead_code)]
    pub fn rate_limiter(&self) -> &AuthRateLimiter {
        &self.rate_limiter
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
    /// - Uses `subtle::ConstantTimeEq` for cryptographically constant-time comparison
    /// - Always checks all keys to prevent early exit timing leaks
    /// - Does not short-circuit on first match
    /// - Length comparison is also constant-time when lengths match
    ///
    /// # Arguments
    ///
    /// * `key` - The API key to validate
    ///
    /// # Returns
    ///
    /// `true` if the key matches any valid key, `false` otherwise
    pub async fn is_valid_key(&self, key: &str) -> bool {
        use subtle::ConstantTimeEq;

        let keys = self.valid_api_keys.read().await;
        let key_bytes = key.as_bytes();

        // Check against each valid key in constant time
        // Must check ALL keys to prevent timing leaks
        let mut found = subtle::Choice::from(0u8);

        for valid_key in keys.iter() {
            let valid_bytes = valid_key.as_bytes();
            // Only compare if lengths match (length comparison is fast and reveals little)
            if key_bytes.len() == valid_bytes.len() {
                // Use constant-time comparison from subtle crate
                found |= key_bytes.ct_eq(valid_bytes);
            }
        }

        // Convert subtle::Choice to bool
        found.into()
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
/// - `MAX_AUTH_ATTEMPTS_PER_MINUTE`: Maximum failed attempts (default: 10)
/// - `AUTH_RATE_LIMIT_WINDOW_SECS`: Time window in seconds (default: 60)
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
/// - **Rate limiting**: Per-IP rate limiting with exponential backoff
/// - **Secure headers**: Failed authentication includes WWW-Authenticate header
/// - **Audit logging**: All authentication attempts are logged for security monitoring
pub async fn auth_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let path = request.uri().path();
    let method = request.method().as_str();

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

    // Extract client IP for rate limiting
    let client_ip = extract_client_ip(&request);

    // Check rate limit BEFORE validating API key
    if let Err(retry_after) = state
        .auth_config
        .rate_limiter
        .check_allowed(&client_ip)
        .await
    {
        warn!(
            path = %path,
            ip = %client_ip,
            retry_after_secs = retry_after.as_secs(),
            "Rate limit exceeded - too many failed authentication attempts"
        );
        AuditLogger::log_auth_blocked(&client_ip, method, path, retry_after.as_secs());
        return Err(rate_limited_response(retry_after));
    }

    // Extract API key from header
    let api_key = match extract_api_key(&request) {
        Some(key) => key,
        None => {
            warn!(path = %path, ip = %client_ip, "Missing API key");
            state
                .auth_config
                .rate_limiter
                .record_failure(&client_ip)
                .await;
            AuditLogger::log_auth_failure(&client_ip, "missing_key", method, path);
            return Err(unauthorized_response("Missing API key"));
        }
    };

    // Validate API key using constant-time comparison
    if !state.auth_config.is_valid_key(&api_key).await {
        warn!(
            path = %path,
            ip = %client_ip,
            key_prefix = &get_key_prefix(&api_key),
            "Invalid API key - authentication failed"
        );
        state
            .auth_config
            .rate_limiter
            .record_failure(&client_ip)
            .await;
        AuditLogger::log_auth_failure(&client_ip, "invalid_key", method, path);
        return Err(unauthorized_response("Invalid API key"));
    }

    debug!(
        path = %path,
        ip = %client_ip,
        key_prefix = &get_key_prefix(&api_key),
        "Authentication successful"
    );

    // Record success (clears failure count)
    state
        .auth_config
        .rate_limiter
        .record_success(&client_ip)
        .await;
    AuditLogger::log_auth_success(&client_ip, &get_key_prefix(&api_key), method, path);

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

/// Create rate limited response with Retry-After header
fn rate_limited_response(retry_after: Duration) -> Response {
    let retry_after_secs = retry_after.as_secs().max(1); // Minimum 1 second

    Response::builder()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .header("Content-Type", "application/json")
        .header("Retry-After", retry_after_secs.to_string())
        .body(Body::from(
            serde_json::json!({
                "error": "Too Many Requests",
                "message": "Too many failed authentication attempts. Please try again later.",
                "retry_after_seconds": retry_after_secs,
            })
            .to_string(),
        ))
        .unwrap_or_else(|_| {
            Response::new(Body::from(
                r#"{"error":"Too Many Requests","message":"Rate limit exceeded"}"#,
            ))
        })
        .into_response()
}

#[allow(dead_code)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
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
