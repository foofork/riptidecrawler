//! API configuration for RipTide API server
//!
//! This module provides configuration structures for the RipTide API server,
//! including authentication, rate limiting, and request handling settings.
//!
//! # Features
//!
//! - **Authentication**: API key configuration and validation settings
//! - **Rate Limiting**: Request rate limits and concurrency controls
//! - **Request Settings**: Timeout, payload limits, and CORS configuration
//! - **Environment Loading**: Automatic configuration from environment variables
//!
//! # Example
//!
//! ```no_run
//! use riptide_config::ApiConfig;
//!
//! // Load configuration from environment
//! let config = ApiConfig::from_env();
//!
//! // Or create with custom settings
//! let custom = ApiConfig::default();
//! ```

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// API key validation module
pub mod validation {
    /// Minimum required length for API keys (32 characters for strong security)
    pub const MIN_API_KEY_LENGTH: usize = 32;

    /// Weak patterns that indicate insecure keys
    /// These are checked as exact matches or at string boundaries to avoid false positives
    const WEAK_PATTERNS: &[&str] = &[
        "test", "password", "admin", "demo", "example", "sample", "default", "changeme",
    ];

    /// Validates an API key against security requirements
    ///
    /// # Requirements
    /// - Minimum length: 32 characters
    /// - Must contain both alphabetic and numeric characters
    /// - Must not BE a weak pattern (exact match or start with weak pattern + underscore/hyphen)
    /// - Must not consist of simple repeated patterns
    ///
    /// # Arguments
    /// - `key`: The API key to validate
    ///
    /// # Returns
    /// - `Ok(())` if the key is valid
    /// - `Err(String)` with a descriptive error message if invalid
    ///
    /// # Examples
    /// ```
    /// use riptide_config::api_key_validation::validate_api_key;
    ///
    /// // Valid key
    /// assert!(validate_api_key("a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6").is_ok());
    ///
    /// // Too short
    /// assert!(validate_api_key("short").is_err());
    ///
    /// // IS a weak pattern (starts with pattern)
    /// assert!(validate_api_key("test1234567890123456789012345678").is_err());
    ///
    /// // Strong key that contains "123" as substring is OK
    /// assert!(validate_api_key("AbCdEf123456789GhIjKl987654321MnOpQr").is_ok());
    /// ```
    pub fn validate_api_key(key: &str) -> Result<(), String> {
        // Check minimum length
        if key.len() < MIN_API_KEY_LENGTH {
            return Err(format!(
                "API key too short: {} characters (minimum {})",
                key.len(),
                MIN_API_KEY_LENGTH
            ));
        }

        // Ensure both letters and numbers are present
        let has_alpha = key.chars().any(|c| c.is_alphabetic());
        let has_numeric = key.chars().any(|c| c.is_numeric());

        if !has_alpha || !has_numeric {
            return Err("API key must contain both letters and numbers".to_string());
        }

        // Check for weak patterns - reject if key contains weak pattern as a word/substring
        // But allow patterns like "123" when mixed with other random characters
        let key_lower = key.to_lowercase();
        let key_trimmed = key_lower.trim();

        for pattern in WEAK_PATTERNS {
            // Reject if the entire key (after trim) is exactly the weak pattern
            if key_trimmed == *pattern {
                return Err(format!("API key is a weak pattern: '{}'", pattern));
            }

            // Reject if key starts with weak pattern (with or without separator)
            if key_trimmed.starts_with(pattern) {
                // Check if it's actually starting with the pattern (not just coincidental chars)
                let after_pattern = &key_trimmed[pattern.len()..];
                // It's a weak pattern if nothing follows, or if followed by separator or digit
                if after_pattern.is_empty()
                    || after_pattern.starts_with('_')
                    || after_pattern.starts_with('-')
                    || after_pattern.chars().next().unwrap().is_numeric()
                {
                    return Err(format!("API key starts with weak pattern: '{}'", pattern));
                }
            }

            // Reject if weak pattern appears in the middle (preceded by digits/separator)
            if key_trimmed.contains(pattern) {
                // Check if it's surrounded by word boundaries (digits, separators, etc.)
                // This catches cases like "123test456" but not "contest" or "latest"
                let parts: Vec<&str> = key_trimmed.split(pattern).collect();
                if parts.len() > 1 {
                    // Pattern appears in the string
                    for i in 0..parts.len() - 1 {
                        let before = parts[i];
                        let after = parts[i + 1];

                        // Check if this looks like an intentional weak pattern insertion
                        let before_is_boundary = before.is_empty()
                            || before.ends_with(|c: char| c.is_numeric() || c == '_' || c == '-');
                        let after_is_boundary = after.is_empty()
                            || after.starts_with(|c: char| c.is_numeric() || c == '_' || c == '-');

                        if before_is_boundary && after_is_boundary {
                            return Err(format!("API key contains weak pattern: '{}'", pattern));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_valid_api_keys() {
            // Strong keys with good entropy
            assert!(validate_api_key("a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6").is_ok());
            assert!(validate_api_key("AbCdEf123456789GhIjKl987654321MnOpQr").is_ok());
            assert!(validate_api_key("api_prod_1234567890abcdefghijklmnopqrstuvwxyz").is_ok());
            assert!(validate_api_key("prod_key_51234567890aBcDeFgHiJkLmNoPqRsTuVwXyZ").is_ok());
        }

        #[test]
        fn test_short_keys_rejected() {
            assert!(validate_api_key("short").is_err());
            assert!(validate_api_key("12345678901234567890123456789").is_err()); // 29 chars
            assert!(validate_api_key("1234567890123456789012345678901").is_err());
            // 31 chars
        }

        #[test]
        fn test_weak_patterns_rejected() {
            assert!(validate_api_key("test1234567890123456789012345678").is_err());
            assert!(validate_api_key("1234567890123456789012345678test").is_err());
            assert!(validate_api_key("123456789test0123456789012345678").is_err());
            assert!(validate_api_key("password123456789012345678901234").is_err());
            assert!(validate_api_key("admin1234567890123456789012345678").is_err());
            assert!(validate_api_key("demo12345678901234567890123456789").is_err());
            assert!(validate_api_key("example123456789012345678901234567").is_err());
            assert!(validate_api_key("sample1234567890123456789012345678").is_err());
            assert!(validate_api_key("default123456789012345678901234567").is_err());
            assert!(validate_api_key("changeme12345678901234567890123456").is_err());
        }

        #[test]
        fn test_weak_patterns_case_insensitive() {
            assert!(validate_api_key("TEST1234567890123456789012345678").is_err());
            assert!(validate_api_key("TeSt1234567890123456789012345678").is_err());
            assert!(validate_api_key("PASSWORD12345678901234567890123").is_err());
        }

        #[test]
        fn test_requires_alphanumeric() {
            // Only letters
            assert!(validate_api_key("abcdefghijklmnopqrstuvwxyzabcdefgh").is_err());
            // Only numbers
            assert!(validate_api_key("12345678901234567890123456789012").is_err());
        }

        #[test]
        fn test_special_characters_allowed() {
            // Special characters are allowed as long as alphanumeric requirements are met
            assert!(validate_api_key("a1b2-c3d4_e5f6.g7h8/i9j0k1l2m3n4o5p6").is_ok());
            assert!(validate_api_key("prod_key_1234567890abcdefghijklmnopqrstuvwxyz").is_ok());
        }
    }
}

/// Authentication configuration for the API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    /// Whether authentication is required (default: true)
    /// Set to false only in development environments
    pub require_auth: bool,

    /// Valid API keys (comma-separated in environment)
    /// Environment variable: API_KEYS
    pub api_keys: Vec<String>,

    /// Whether to use constant-time comparison for API keys (default: true)
    /// Should always be true in production for security
    pub constant_time_comparison: bool,

    /// Paths that don't require authentication
    pub public_paths: Vec<String>,
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        Self {
            require_auth: true,
            api_keys: Vec::new(),
            constant_time_comparison: true,
            public_paths: vec![
                "/health".to_string(),
                "/healthz".to_string(),
                "/metrics".to_string(),
                "/api/v1/health".to_string(),
                "/api/v1/metrics".to_string(),
                "/api/health/detailed".to_string(),
            ],
        }
    }
}

impl AuthenticationConfig {
    /// Load authentication configuration from environment variables
    ///
    /// Environment variables:
    /// - `API_KEYS`: Comma-separated list of valid API keys
    /// - `REQUIRE_AUTH`: Set to "false" to disable authentication (dev only)
    ///
    /// # Validation
    /// This method validates all API keys against security requirements:
    /// - Minimum 32 characters
    /// - Must contain both letters and numbers
    /// - Must not contain weak patterns
    ///
    /// # Panics
    /// Panics if any API key fails validation when `require_auth` is true.
    /// This is intentional to prevent weak keys in production.
    pub fn from_env() -> Self {
        let api_keys: Vec<String> = std::env::var("API_KEYS")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string())
            .collect();

        let require_auth = std::env::var("REQUIRE_AUTH")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true);

        // Validate API keys if authentication is required
        if require_auth && !api_keys.is_empty() {
            for key in &api_keys {
                if let Err(e) = validation::validate_api_key(key) {
                    panic!(
                        "Invalid API key detected during configuration load: {}. \
                         Please use strong API keys (minimum 32 characters, alphanumeric, no weak patterns). \
                         Generate a secure key with: openssl rand -base64 32",
                        e
                    );
                }
            }
        }

        Self {
            api_keys,
            require_auth,
            ..Default::default()
        }
    }

    /// Create configuration with custom API keys
    ///
    /// # Validation
    /// This method validates all API keys if authentication is required.
    ///
    /// # Panics
    /// Panics if any API key fails validation when `require_auth` is true.
    pub fn with_api_keys(mut self, keys: Vec<String>) -> Self {
        // Validate keys if authentication is required
        if self.require_auth {
            for key in &keys {
                if let Err(e) = validation::validate_api_key(key) {
                    panic!(
                        "Invalid API key in with_api_keys: {}. \
                         Please use strong API keys (minimum 32 characters, alphanumeric, no weak patterns).",
                        e
                    );
                }
            }
        }
        self.api_keys = keys;
        self
    }

    /// Set whether authentication is required
    pub fn with_require_auth(mut self, require: bool) -> Self {
        self.require_auth = require;
        self
    }

    /// Add a public path that doesn't require authentication
    pub fn add_public_path(mut self, path: String) -> Self {
        self.public_paths.push(path);
        self
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum number of concurrent requests (default: 100)
    /// Environment variable: MAX_CONCURRENT_REQUESTS
    pub max_concurrent_requests: usize,

    /// Requests per minute per client (default: 60)
    /// Environment variable: RATE_LIMIT_PER_MINUTE
    pub requests_per_minute: u32,

    /// Whether to enable rate limiting (default: true)
    /// Environment variable: ENABLE_RATE_LIMITING
    pub enabled: bool,

    /// Custom rate limits for specific paths or clients
    pub custom_limits: Vec<CustomRateLimit>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 100,
            requests_per_minute: 60,
            enabled: true,
            custom_limits: Vec::new(),
        }
    }
}

impl RateLimitConfig {
    /// Load rate limiting configuration from environment variables
    pub fn from_env() -> Self {
        let max_concurrent_requests = std::env::var("MAX_CONCURRENT_REQUESTS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);

        let requests_per_minute = std::env::var("RATE_LIMIT_PER_MINUTE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60);

        let enabled = std::env::var("ENABLE_RATE_LIMITING")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true);

        Self {
            max_concurrent_requests,
            requests_per_minute,
            enabled,
            custom_limits: Vec::new(),
        }
    }

    /// Set maximum concurrent requests
    pub fn with_max_concurrent_requests(mut self, max: usize) -> Self {
        self.max_concurrent_requests = max;
        self
    }

    /// Set requests per minute limit
    pub fn with_requests_per_minute(mut self, rpm: u32) -> Self {
        self.requests_per_minute = rpm;
        self
    }

    /// Enable or disable rate limiting
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Custom rate limit for specific path or client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRateLimit {
    /// Path pattern (e.g., "/api/v1/crawl")
    pub path_pattern: Option<String>,

    /// Client ID or API key
    pub client_id: Option<String>,

    /// Custom rate limit (requests per minute)
    pub requests_per_minute: u32,

    /// Custom max concurrent requests
    pub max_concurrent_requests: Option<usize>,
}

/// Request handling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestConfig {
    /// Request timeout duration (default: 30 seconds)
    /// Environment variable: REQUEST_TIMEOUT_SECS
    pub timeout: Duration,

    /// Maximum request payload size in bytes (default: 50MB)
    /// Environment variable: MAX_PAYLOAD_SIZE
    pub max_payload_size: usize,

    /// Whether to enable CORS (default: true)
    /// Environment variable: ENABLE_CORS
    pub enable_cors: bool,

    /// Whether to enable compression (default: true)
    /// Environment variable: ENABLE_COMPRESSION
    pub enable_compression: bool,
}

impl Default for RequestConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_payload_size: 50 * 1024 * 1024, // 50MB
            enable_cors: true,
            enable_compression: true,
        }
    }
}

impl RequestConfig {
    /// Load request configuration from environment variables
    pub fn from_env() -> Self {
        let timeout = std::env::var("REQUEST_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(30));

        let max_payload_size = std::env::var("MAX_PAYLOAD_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(50 * 1024 * 1024);

        let enable_cors = std::env::var("ENABLE_CORS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true);

        let enable_compression = std::env::var("ENABLE_COMPRESSION")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true);

        Self {
            timeout,
            max_payload_size,
            enable_cors,
            enable_compression,
        }
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set maximum payload size
    pub fn with_max_payload_size(mut self, size: usize) -> Self {
        self.max_payload_size = size;
        self
    }
}

/// Complete API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Authentication settings
    pub auth: AuthenticationConfig,

    /// Rate limiting settings
    pub rate_limit: RateLimitConfig,

    /// Request handling settings
    pub request: RequestConfig,

    /// Server bind address (default: 0.0.0.0:8080)
    /// Environment variable: BIND_ADDRESS
    pub bind_address: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            auth: AuthenticationConfig::default(),
            rate_limit: RateLimitConfig::default(),
            request: RequestConfig::default(),
            bind_address: "0.0.0.0:8080".to_string(),
        }
    }
}

impl ApiConfig {
    /// Load complete API configuration from environment variables
    ///
    /// This is the recommended way to configure the API in production.
    /// All configuration can be controlled via environment variables.
    pub fn from_env() -> Self {
        let bind_address =
            std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8080".to_string());

        Self {
            auth: AuthenticationConfig::from_env(),
            rate_limit: RateLimitConfig::from_env(),
            request: RequestConfig::from_env(),
            bind_address,
        }
    }

    /// Create configuration with custom authentication
    pub fn with_auth(mut self, auth: AuthenticationConfig) -> Self {
        self.auth = auth;
        self
    }

    /// Create configuration with custom rate limiting
    pub fn with_rate_limit(mut self, rate_limit: RateLimitConfig) -> Self {
        self.rate_limit = rate_limit;
        self
    }

    /// Create configuration with custom request settings
    pub fn with_request(mut self, request: RequestConfig) -> Self {
        self.request = request;
        self
    }

    /// Set bind address
    pub fn with_bind_address(mut self, address: String) -> Self {
        self.bind_address = address;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_auth_config() {
        let config = AuthenticationConfig::default();
        assert!(config.require_auth);
        assert!(config.constant_time_comparison);
        assert!(config.api_keys.is_empty());
        assert!(config.public_paths.contains(&"/health".to_string()));
        assert!(config.public_paths.contains(&"/healthz".to_string()));
    }

    #[test]
    fn test_auth_config_builder() {
        // Disable auth first, then set weak keys (validation is skipped when auth disabled)
        let config = AuthenticationConfig::default()
            .with_require_auth(false)
            .with_api_keys(vec!["key1".to_string(), "key2".to_string()])
            .add_public_path("/custom".to_string());

        assert!(!config.require_auth);
        assert_eq!(config.api_keys.len(), 2);
        assert!(config.public_paths.contains(&"/custom".to_string()));
    }

    #[test]
    fn test_default_rate_limit_config() {
        let config = RateLimitConfig::default();
        assert_eq!(config.max_concurrent_requests, 100);
        assert_eq!(config.requests_per_minute, 60);
        assert!(config.enabled);
    }

    #[test]
    fn test_rate_limit_config_builder() {
        let config = RateLimitConfig::default()
            .with_max_concurrent_requests(200)
            .with_requests_per_minute(120)
            .with_enabled(false);

        assert_eq!(config.max_concurrent_requests, 200);
        assert_eq!(config.requests_per_minute, 120);
        assert!(!config.enabled);
    }

    #[test]
    fn test_default_request_config() {
        let config = RequestConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_payload_size, 50 * 1024 * 1024);
        assert!(config.enable_cors);
        assert!(config.enable_compression);
    }

    #[test]
    fn test_request_config_builder() {
        let config = RequestConfig::default()
            .with_timeout(Duration::from_secs(60))
            .with_max_payload_size(100 * 1024 * 1024);

        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.max_payload_size, 100 * 1024 * 1024);
    }

    #[test]
    fn test_default_api_config() {
        let config = ApiConfig::default();
        assert_eq!(config.bind_address, "0.0.0.0:8080");
        assert!(config.auth.require_auth);
        assert!(config.rate_limit.enabled);
    }

    #[test]
    fn test_api_config_builder() {
        let auth = AuthenticationConfig::default().with_require_auth(false);

        let config = ApiConfig::default()
            .with_auth(auth)
            .with_bind_address("127.0.0.1:3000".to_string());

        assert!(!config.auth.require_auth);
        assert_eq!(config.bind_address, "127.0.0.1:3000");
    }
}
