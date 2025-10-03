use anyhow::{anyhow, Result};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use tracing::{debug, warn};

// Re-export all security modules
pub mod api_keys;
pub mod audit;
pub mod budget;
pub mod middleware;
pub mod pii;
pub mod types;

// Re-export commonly used types and structs
pub use api_keys::{ApiKey, ApiKeyManager};
pub use audit::{AuditConfig, AuditLogEntry, AuditLogger};
pub use budget::{BudgetHealthStatus, BudgetManager};
pub use middleware::{RequestSecurityContext, SecurityHealthStatus, SecurityMiddleware};
pub use pii::{PiiRedactionMiddleware, PiiRedactor};
pub use types::*;

/// Security middleware configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable CORS protection
    pub enable_cors: bool,
    /// Allowed origins for CORS
    pub cors_allowed_origins: Vec<String>,
    /// Enable XSS protection headers
    pub enable_xss_protection: bool,
    /// Enable content type sniffing protection
    pub enable_content_type_protection: bool,
    /// Enable frame options protection
    pub enable_frame_protection: bool,
    /// Enable strict transport security
    pub enable_hsts: bool,
    /// Maximum request size in bytes
    pub max_request_size: usize,
    /// Rate limiting configuration
    pub rate_limit: Option<RateLimitConfig>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_cors: true,
            cors_allowed_origins: vec!["*".to_string()],
            enable_xss_protection: true,
            enable_content_type_protection: true,
            enable_frame_protection: true,
            enable_hsts: true,
            max_request_size: 20 * 1024 * 1024, // 20MB
            rate_limit: Some(RateLimitConfig::default()),
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per window
    pub requests_per_window: u32,
    /// Window duration in seconds
    pub window_seconds: u64,
    /// Maximum burst size
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_window: 100,
            window_seconds: 60,
            burst_size: 10,
        }
    }
}

/// Legacy security middleware for HTTP responses and requests
pub struct LegacySecurityMiddleware {
    config: SecurityConfig,
}

impl LegacySecurityMiddleware {
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }

    pub fn new_default() -> Self {
        Self {
            config: SecurityConfig::default(),
        }
    }

    /// Apply security headers to HTTP response
    pub fn apply_security_headers(&self, headers: &mut HeaderMap) -> Result<()> {
        // XSS Protection
        if self.config.enable_xss_protection {
            headers.insert(
                HeaderName::from_str("X-XSS-Protection")?,
                HeaderValue::from_str("1; mode=block")?,
            );

            headers.insert(
                HeaderName::from_str("X-Content-Type-Options")?,
                HeaderValue::from_str("nosniff")?,
            );
        }

        // Content Type Protection
        if self.config.enable_content_type_protection {
            headers.insert(
                HeaderName::from_str("X-Content-Type-Options")?,
                HeaderValue::from_str("nosniff")?,
            );
        }

        // Frame Protection
        if self.config.enable_frame_protection {
            headers.insert(
                HeaderName::from_str("X-Frame-Options")?,
                HeaderValue::from_str("DENY")?,
            );

            headers.insert(
                HeaderName::from_str("Content-Security-Policy")?,
                HeaderValue::from_str("frame-ancestors 'none'")?,
            );
        }

        // Strict Transport Security
        if self.config.enable_hsts {
            headers.insert(
                HeaderName::from_str("Strict-Transport-Security")?,
                HeaderValue::from_str("max-age=31536000; includeSubDomains")?,
            );
        }

        // CORS Headers
        if self.config.enable_cors {
            self.apply_cors_headers(headers)?;
        }

        // Additional security headers
        headers.insert(
            HeaderName::from_str("Referrer-Policy")?,
            HeaderValue::from_str("strict-origin-when-cross-origin")?,
        );

        headers.insert(
            HeaderName::from_str("Permissions-Policy")?,
            HeaderValue::from_str("geolocation=(), microphone=(), camera=()")?,
        );

        debug!("Applied security headers");
        Ok(())
    }

    /// Apply CORS headers
    fn apply_cors_headers(&self, headers: &mut HeaderMap) -> Result<()> {
        let allowed_origins = if self.config.cors_allowed_origins.contains(&"*".to_string()) {
            "*"
        } else {
            // For multiple specific origins, this would need more complex logic
            self.config
                .cors_allowed_origins
                .first()
                .map(|s| s.as_str())
                .unwrap_or("*")
        };

        headers.insert(
            HeaderName::from_str("Access-Control-Allow-Origin")?,
            HeaderValue::from_str(allowed_origins)?,
        );

        headers.insert(
            HeaderName::from_str("Access-Control-Allow-Methods")?,
            HeaderValue::from_str("GET, POST, OPTIONS")?,
        );

        headers.insert(
            HeaderName::from_str("Access-Control-Allow-Headers")?,
            HeaderValue::from_str("Content-Type, Authorization, X-Requested-With")?,
        );

        headers.insert(
            HeaderName::from_str("Access-Control-Max-Age")?,
            HeaderValue::from_str("86400")?,
        );

        Ok(())
    }

    /// Validate request size
    pub fn validate_request_size(&self, size: usize) -> Result<()> {
        if size > self.config.max_request_size {
            warn!(
                request_size = size,
                max_size = self.config.max_request_size,
                "Request size exceeds limit"
            );
            return Err(anyhow!(
                "Request size {} exceeds maximum allowed {}",
                size,
                self.config.max_request_size
            ));
        }

        debug!(request_size = size, "Request size validation passed");
        Ok(())
    }

    /// Sanitize and validate request headers
    pub fn sanitize_headers(&self, headers: &HeaderMap) -> Result<HeaderMap> {
        let mut sanitized = HeaderMap::new();

        for (name, value) in headers {
            // Skip potentially dangerous headers
            let name_str = name.as_str().to_lowercase();
            match name_str.as_str() {
                "host" | "connection" | "upgrade" => {
                    debug!(header = name_str, "Skipping potentially dangerous header");
                    continue;
                }
                _ => {}
            }

            // Validate header value
            if let Ok(value_str) = value.to_str() {
                // Check for suspicious patterns
                if self.contains_suspicious_patterns(value_str) {
                    warn!(
                        header = name_str,
                        value = value_str,
                        "Suspicious header value detected"
                    );
                    continue;
                }

                sanitized.insert(name.clone(), value.clone());
            }
        }

        debug!(
            original_count = headers.len(),
            sanitized_count = sanitized.len(),
            "Headers sanitized"
        );
        Ok(sanitized)
    }

    /// Check for suspicious patterns in header values
    fn contains_suspicious_patterns(&self, value: &str) -> bool {
        let suspicious_patterns = [
            "<script",
            "javascript:",
            "data:",
            "vbscript:",
            "onload=",
            "onerror=",
            "eval(",
            "alert(",
            "confirm(",
            "prompt(",
            "document.cookie",
            "window.location",
        ];

        let value_lower = value.to_lowercase();
        for pattern in &suspicious_patterns {
            if value_lower.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// Create secure HTTP client headers
    pub fn create_secure_client_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        // Set secure user agent
        headers.insert(
            HeaderName::from_str("User-Agent")?,
            HeaderValue::from_str("RipTide/1.0 (+https://github.com/your-org/riptide)")?,
        );

        // Disable potentially dangerous features
        headers.insert(HeaderName::from_str("DNT")?, HeaderValue::from_str("1")?);

        // Set acceptable content types
        headers.insert(
            HeaderName::from_str("Accept")?,
            HeaderValue::from_str("text/html,application/xhtml+xml,application/xml;q=0.9,text/plain;q=0.8,application/json;q=0.7")?,
        );

        // Set acceptable encodings
        headers.insert(
            HeaderName::from_str("Accept-Encoding")?,
            HeaderValue::from_str("gzip, br")?,
        );

        debug!("Created secure client headers");
        Ok(headers)
    }

    /// Get security configuration
    pub fn get_config(&self) -> &SecurityConfig {
        &self.config
    }
}

/// HTTP method validation
pub fn validate_http_method(method: &str) -> Result<()> {
    match method.to_uppercase().as_str() {
        "GET" | "POST" | "HEAD" | "OPTIONS" => Ok(()),
        method => {
            warn!(method = method, "Unsupported HTTP method");
            Err(anyhow!("Unsupported HTTP method: {}", method))
        }
    }
}

/// Sanitize file path to prevent directory traversal
pub fn sanitize_file_path(path: &str) -> Result<String> {
    // Remove dangerous patterns
    let sanitized = path
        .replace("../", "")
        .replace("..", "")
        .replace("./", "")
        .replace("~", "")
        .chars()
        .filter(|c| c.is_alphanumeric() || "._-/".contains(*c))
        .collect::<String>();

    if sanitized.is_empty() {
        return Err(anyhow!("Invalid file path after sanitization"));
    }

    if sanitized != path {
        warn!(
            original = path,
            sanitized = sanitized,
            "File path sanitized"
        );
    }

    Ok(sanitized)
}

/// Content Security Policy builder
pub struct CSPBuilder {
    directives: HashMap<String, Vec<String>>,
}

impl CSPBuilder {
    pub fn new() -> Self {
        Self {
            directives: HashMap::new(),
        }
    }

    pub fn default_policy() -> Self {
        Self::new()
            .default_src(&["'self'"])
            .script_src(&["'self'", "'unsafe-inline'"])
            .style_src(&["'self'", "'unsafe-inline'"])
            .img_src(&["'self'", "data:", "https:"])
            .font_src(&["'self'", "https:"])
            .connect_src(&["'self'"])
            .frame_ancestors(&["'none'"])
    }

    pub fn default_src(mut self, sources: &[&str]) -> Self {
        self.add_directive("default-src", sources);
        self
    }

    pub fn script_src(mut self, sources: &[&str]) -> Self {
        self.add_directive("script-src", sources);
        self
    }

    pub fn style_src(mut self, sources: &[&str]) -> Self {
        self.add_directive("style-src", sources);
        self
    }

    pub fn img_src(mut self, sources: &[&str]) -> Self {
        self.add_directive("img-src", sources);
        self
    }

    pub fn font_src(mut self, sources: &[&str]) -> Self {
        self.add_directive("font-src", sources);
        self
    }

    pub fn connect_src(mut self, sources: &[&str]) -> Self {
        self.add_directive("connect-src", sources);
        self
    }

    pub fn frame_ancestors(mut self, sources: &[&str]) -> Self {
        self.add_directive("frame-ancestors", sources);
        self
    }

    fn add_directive(&mut self, directive: &str, sources: &[&str]) {
        self.directives.insert(
            directive.to_string(),
            sources.iter().map(|s| s.to_string()).collect(),
        );
    }

    pub fn build(self) -> String {
        let mut policy_parts = Vec::new();

        for (directive, sources) in self.directives {
            let sources_str = sources.join(" ");
            policy_parts.push(format!("{} {}", directive, sources_str));
        }

        policy_parts.join("; ")
    }
}

impl Default for CSPBuilder {
    fn default() -> Self {
        Self::default_policy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_middleware_creation() {
        let _middleware = SecurityMiddleware::with_defaults().expect("Failed to create middleware");
    }

    #[test]
    fn test_security_headers() {
        let middleware = SecurityMiddleware::with_defaults().expect("Failed to create middleware");
        let mut headers = HeaderMap::new();

        assert!(middleware.apply_security_headers(&mut headers).is_ok());
        assert!(!headers.is_empty());
        assert!(headers.contains_key("X-XSS-Protection"));
        assert!(headers.contains_key("X-Content-Type-Options"));
        assert!(headers.contains_key("X-Frame-Options"));
    }

    #[test]
    fn test_request_size_validation() {
        let middleware = SecurityMiddleware::with_defaults().expect("Failed to create middleware");

        assert!(middleware.validate_request_size(1024).is_ok());
        assert!(middleware.validate_request_size(25 * 1024 * 1024).is_err());
    }

    #[test]
    fn test_suspicious_pattern_detection() {
        let middleware = LegacySecurityMiddleware::new_default();

        assert!(middleware.contains_suspicious_patterns("<script>alert(1)</script>"));
        assert!(middleware.contains_suspicious_patterns("javascript:void(0)"));
        assert!(!middleware.contains_suspicious_patterns("normal text content"));
    }

    #[test]
    fn test_file_path_sanitization() {
        assert_eq!(
            sanitize_file_path("normal_file.txt").expect("Should sanitize normal file path"),
            "normal_file.txt"
        );
        assert_eq!(
            sanitize_file_path("../../../etc/passwd").expect("Should sanitize directory traversal"),
            "etc/passwd"
        );
        assert_eq!(
            sanitize_file_path("file with spaces.txt").expect("Should sanitize file with spaces"),
            "filewithspaces.txt"
        );
    }

    #[test]
    fn test_http_method_validation() {
        assert!(validate_http_method("GET").is_ok());
        assert!(validate_http_method("POST").is_ok());
        assert!(validate_http_method("DELETE").is_err());
        assert!(validate_http_method("TRACE").is_err());
    }

    #[test]
    fn test_csp_builder() {
        let csp = CSPBuilder::default_policy().build();
        assert!(csp.contains("default-src 'self'"));
        assert!(csp.contains("frame-ancestors 'none'"));
    }
}
