use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{debug, warn};
use url::Url;

/// Maximum URL length to prevent abuse
const MAX_URL_LENGTH: usize = 2048;

/// Maximum header size to prevent large header attacks
const MAX_HEADER_SIZE: usize = 8192;

/// Allowed content types for extraction
const ALLOWED_CONTENT_TYPES: &[&str] = &[
    "text/html",
    "application/xhtml+xml",
    "text/xml",
    "application/xml",
    "text/plain",
    "application/pdf",
    "application/json",
    "text/markdown",
    "text/x-markdown",
];

/// Security validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Maximum URL length
    pub max_url_length: usize,
    /// Maximum header size
    pub max_header_size: usize,
    /// Allowed content types
    pub allowed_content_types: HashSet<String>,
    /// Blocked URL patterns
    pub blocked_patterns: HashSet<String>,
    /// Allowed domains (if specified, only these domains are allowed)
    pub allowed_domains: Option<HashSet<String>>,
    /// Block private/local IP addresses
    pub block_private_ips: bool,
    /// Maximum content size in bytes (20MB)
    pub max_content_size: usize,
    /// Enable strict SSL verification
    pub strict_ssl: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        let mut allowed_types = HashSet::new();
        for content_type in ALLOWED_CONTENT_TYPES {
            allowed_types.insert(content_type.to_string());
        }

        let mut blocked_patterns = HashSet::new();
        // Block common dangerous patterns
        blocked_patterns.insert("localhost".to_string());
        blocked_patterns.insert("127.0.0.1".to_string());
        blocked_patterns.insert("0.0.0.0".to_string());
        blocked_patterns.insert("::1".to_string());
        blocked_patterns.insert("169.254.".to_string()); // Link-local
        blocked_patterns.insert("10.".to_string()); // Private Class A
        blocked_patterns.insert("172.16.".to_string()); // Private Class B start
        blocked_patterns.insert("192.168.".to_string()); // Private Class C

        Self {
            max_url_length: MAX_URL_LENGTH,
            max_header_size: MAX_HEADER_SIZE,
            allowed_content_types: allowed_types,
            blocked_patterns,
            allowed_domains: None,
            block_private_ips: true,
            max_content_size: 20 * 1024 * 1024, // 20MB
            strict_ssl: true,
        }
    }
}

/// URL and input validator with security checks
pub struct InputValidator {
    config: ValidationConfig,
}

impl InputValidator {
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    pub fn new_default() -> Self {
        Self {
            config: ValidationConfig::default(),
        }
    }

    /// Validate URL for security and format
    pub fn validate_url(&self, url_str: &str) -> Result<Url> {
        // Check URL length
        if url_str.len() > self.config.max_url_length {
            warn!(
                url_length = url_str.len(),
                max_length = self.config.max_url_length,
                "URL too long"
            );
            return Err(anyhow!(
                "URL length {} exceeds maximum {}",
                url_str.len(),
                self.config.max_url_length
            ));
        }

        // Parse URL
        let url = Url::parse(url_str).map_err(|e| anyhow!("Invalid URL format: {}", e))?;

        // Validate scheme
        match url.scheme() {
            "http" | "https" => {}
            scheme => {
                warn!(scheme = scheme, "Unsupported URL scheme");
                return Err(anyhow!("Unsupported URL scheme: {}", scheme));
            }
        }

        // Check for blocked patterns
        for pattern in &self.config.blocked_patterns {
            if url_str.contains(pattern) {
                warn!(
                    url = url_str,
                    pattern = pattern,
                    "URL contains blocked pattern"
                );
                return Err(anyhow!("URL contains blocked pattern: {}", pattern));
            }
        }

        // Check allowed domains if specified
        if let Some(ref allowed_domains) = self.config.allowed_domains {
            if let Some(host) = url.host_str() {
                if !allowed_domains.contains(host) {
                    warn!(host = host, "Host not in allowed domains list");
                    return Err(anyhow!("Host {} not in allowed domains", host));
                }
            }
        }

        // Check for private IP addresses if blocking is enabled
        if self.config.block_private_ips {
            if let Some(host) = url.host_str() {
                if self.is_private_or_local_address(host) {
                    warn!(host = host, "Private or local IP address blocked");
                    return Err(anyhow!(
                        "Private or local IP addresses are not allowed: {}",
                        host
                    ));
                }
            }
        }

        debug!(url = url_str, "URL validation passed");
        Ok(url)
    }

    /// Validate content type against allowlist
    pub fn validate_content_type(&self, content_type: &str) -> Result<()> {
        // Extract base content type (ignore charset and other parameters)
        let base_type = content_type
            .split(';')
            .next()
            .unwrap_or(content_type)
            .trim()
            .to_lowercase();

        if !self.config.allowed_content_types.contains(&base_type) {
            warn!(content_type = content_type, "Content type not allowed");
            return Err(anyhow!("Content type not allowed: {}", content_type));
        }

        debug!(
            content_type = content_type,
            "Content type validation passed"
        );
        Ok(())
    }

    /// Validate HTTP headers for size and security
    pub fn validate_headers(&self, headers: &[(String, String)]) -> Result<()> {
        let total_size: usize = headers
            .iter()
            .map(|(name, value)| name.len() + value.len() + 2) // +2 for ": "
            .sum();

        if total_size > self.config.max_header_size {
            warn!(
                header_size = total_size,
                max_size = self.config.max_header_size,
                "Headers too large"
            );
            return Err(anyhow!(
                "Headers size {} exceeds maximum {}",
                total_size,
                self.config.max_header_size
            ));
        }

        // Check for suspicious headers
        for (name, value) in headers {
            let name_lower = name.to_lowercase();

            // Block certain headers that could be used for attacks
            match name_lower.as_str() {
                "host" | "authorization" | "cookie" => {
                    if value.len() > 512 {
                        warn!(
                            header = name,
                            value_length = value.len(),
                            "Suspicious header value length"
                        );
                        return Err(anyhow!("Header {} has suspicious value length", name));
                    }
                }
                _ => {}
            }

            // Check for control characters in header values
            if value.chars().any(|c| c.is_control() && c != '\t') {
                warn!(header = name, "Header contains control characters");
                return Err(anyhow!(
                    "Header {} contains invalid control characters",
                    name
                ));
            }
        }

        debug!(
            header_count = headers.len(),
            total_size = total_size,
            "Header validation passed"
        );
        Ok(())
    }

    /// Validate content size
    pub fn validate_content_size(&self, size: usize) -> Result<()> {
        if size > self.config.max_content_size {
            warn!(
                content_size = size,
                max_size = self.config.max_content_size,
                "Content too large"
            );
            return Err(anyhow!(
                "Content size {} exceeds maximum {}",
                size,
                self.config.max_content_size
            ));
        }

        debug!(content_size = size, "Content size validation passed");
        Ok(())
    }

    /// Check if address is private or local
    fn is_private_or_local_address(&self, host: &str) -> bool {
        // Check for localhost variants
        if host == "localhost" || host == "127.0.0.1" || host == "::1" {
            return true;
        }

        // Check for private IP ranges
        if let Ok(ip) = host.parse::<std::net::IpAddr>() {
            match ip {
                std::net::IpAddr::V4(ipv4) => {
                    let octets = ipv4.octets();
                    // Private IPv4 ranges
                    match octets[0] {
                        10 => true,                                        // 10.0.0.0/8
                        172 if octets[1] >= 16 && octets[1] <= 31 => true, // 172.16.0.0/12
                        192 if octets[1] == 168 => true,                   // 192.168.0.0/16
                        127 => true,                                       // 127.0.0.0/8 (loopback)
                        169 if octets[1] == 254 => true, // 169.254.0.0/16 (link-local)
                        _ => false,
                    }
                }
                std::net::IpAddr::V6(ipv6) => {
                    // Check for loopback and link-local IPv6
                    ipv6.is_loopback() || (ipv6.segments()[0] & 0xffc0) == 0xfe80
                    // Link-local
                }
            }
        } else {
            false
        }
    }

    /// Get validation configuration
    pub fn get_config(&self) -> &ValidationConfig {
        &self.config
    }
}

/// Validation result with details
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn success() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            valid: false,
            errors: vec![message],
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.valid = false;
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_validation() {
        let validator = InputValidator::new_default();

        // Valid URLs
        assert!(validator.validate_url("https://example.com").is_ok());
        assert!(validator.validate_url("http://test.org/path").is_ok());

        // Invalid URLs
        assert!(validator.validate_url("ftp://example.com").is_err());
        assert!(validator.validate_url("javascript:alert(1)").is_err());
        assert!(validator.validate_url("https://localhost").is_err());
        assert!(validator.validate_url("https://127.0.0.1").is_err());
        assert!(validator.validate_url("https://192.168.1.1").is_err());
    }

    #[test]
    fn test_content_type_validation() {
        let validator = InputValidator::new_default();

        // Valid content types
        assert!(validator.validate_content_type("text/html").is_ok());
        assert!(validator
            .validate_content_type("text/html; charset=utf-8")
            .is_ok());
        assert!(validator.validate_content_type("application/json").is_ok());

        // Invalid content types
        assert!(validator
            .validate_content_type("application/javascript")
            .is_err());
        assert!(validator.validate_content_type("text/css").is_err());
        assert!(validator.validate_content_type("image/png").is_err());
    }

    #[test]
    fn test_header_validation() {
        let validator = InputValidator::new_default();

        // Valid headers
        let headers = vec![
            ("Content-Type".to_string(), "text/html".to_string()),
            ("User-Agent".to_string(), "Mozilla/5.0".to_string()),
        ];
        assert!(validator.validate_headers(&headers).is_ok());

        // Headers too large
        let large_value = "x".repeat(9000);
        let large_headers = vec![("Large-Header".to_string(), large_value)];
        assert!(validator.validate_headers(&large_headers).is_err());
    }

    #[test]
    fn test_content_size_validation() {
        let validator = InputValidator::new_default();

        assert!(validator.validate_content_size(1024).is_ok());
        assert!(validator.validate_content_size(10 * 1024 * 1024).is_ok());
        assert!(validator.validate_content_size(25 * 1024 * 1024).is_err());
    }

    #[test]
    fn test_private_ip_detection() {
        let validator = InputValidator::new_default();

        assert!(validator.is_private_or_local_address("localhost"));
        assert!(validator.is_private_or_local_address("127.0.0.1"));
        assert!(validator.is_private_or_local_address("10.0.0.1"));
        assert!(validator.is_private_or_local_address("192.168.1.1"));
        assert!(validator.is_private_or_local_address("172.16.0.1"));
        assert!(!validator.is_private_or_local_address("8.8.8.8"));
        assert!(!validator.is_private_or_local_address("google.com"));
    }
}
