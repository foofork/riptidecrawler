//! Common validation utilities shared across RipTide modules.
//!
//! This module consolidates duplicate validation logic found in multiple
//! places throughout the codebase, providing a unified approach to:
//! - URL validation with security checks
//! - Content-type validation
//! - Size limit validation
//! - Parameter validation

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{debug, warn};
use url::Url;

/// Maximum URL length to prevent abuse
pub const MAX_URL_LENGTH: usize = 2048;

/// Maximum header size to prevent large header attacks
pub const MAX_HEADER_SIZE: usize = 8192;

/// Default content size limit (20MB)
pub const DEFAULT_MAX_CONTENT_SIZE: usize = 20 * 1024 * 1024;

/// Allowed content types for extraction
pub const ALLOWED_CONTENT_TYPES: &[&str] = &[
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

/// Common validation configuration
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
    /// Maximum content size in bytes
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
            max_content_size: DEFAULT_MAX_CONTENT_SIZE,
            strict_ssl: true,
        }
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

/// Common validator with security checks
pub struct CommonValidator {
    config: ValidationConfig,
}

impl CommonValidator {
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
    pub fn is_private_or_local_address(&self, host: &str) -> bool {
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

    /// Validate query content for basic security
    pub fn validate_query_content(&self, query: &str) -> Result<()> {
        // Check for control characters
        if query
            .chars()
            .any(|c| c.is_control() && c != '\t' && c != '\n')
        {
            return Err(anyhow!("Query contains invalid control characters"));
        }

        // Check for obvious SQL injection patterns
        let sql_patterns = &["union select", "drop table", "insert into", "delete from"];
        let query_lower = query.to_lowercase();
        for pattern in sql_patterns {
            if query_lower.contains(pattern) {
                return Err(anyhow!("Query contains suspicious SQL patterns"));
            }
        }

        // Check for script injection patterns
        let script_patterns = &["<script", "javascript:", "data:text/html"];
        for pattern in script_patterns {
            if query_lower.contains(pattern) {
                return Err(anyhow!("Query contains suspicious script patterns"));
            }
        }

        Ok(())
    }
}

/// Specialized validators for different content types
pub struct ContentTypeValidator;

impl ContentTypeValidator {
    pub fn is_html(content_type: &str) -> bool {
        let base_type = content_type.split(';').next().unwrap_or(content_type).trim().to_lowercase();
        matches!(base_type.as_str(), "text/html" | "application/xhtml+xml")
    }

    pub fn is_json(content_type: &str) -> bool {
        let base_type = content_type.split(';').next().unwrap_or(content_type).trim().to_lowercase();
        base_type == "application/json" || base_type.ends_with("+json")
    }

    pub fn is_xml(content_type: &str) -> bool {
        let base_type = content_type.split(';').next().unwrap_or(content_type).trim().to_lowercase();
        matches!(base_type.as_str(), "text/xml" | "application/xml") || base_type.ends_with("+xml")
    }

    pub fn is_pdf(content_type: &str) -> bool {
        let base_type = content_type.split(';').next().unwrap_or(content_type).trim().to_lowercase();
        base_type == "application/pdf"
    }
}

/// URL validation utilities
pub struct UrlValidator;

impl UrlValidator {
    pub fn validate_scheme(url: &Url) -> Result<()> {
        match url.scheme() {
            "http" | "https" => Ok(()),
            scheme => Err(anyhow!("Unsupported URL scheme: {}", scheme)),
        }
    }

    pub fn has_suspicious_patterns(url_str: &str) -> bool {
        // Check for excessive URL encoding (potential obfuscation)
        let percent_count = url_str.matches('%').count();
        if percent_count > 20 {
            return true;
        }

        // Check for suspicious file extensions
        let suspicious_extensions = &[".exe", ".bat", ".cmd", ".scr", ".vbs", ".js"];
        for ext in suspicious_extensions {
            if url_str.to_lowercase().contains(ext) {
                return true;
            }
        }

        false
    }
}

/// Size validation utilities
pub struct SizeValidator;

impl SizeValidator {
    pub fn validate_url_length(url: &str, max_length: usize) -> Result<()> {
        if url.len() > max_length {
            return Err(anyhow!(
                "URL length {} exceeds maximum {}",
                url.len(),
                max_length
            ));
        }
        Ok(())
    }

    pub fn validate_content_length(size: usize, max_size: usize) -> Result<()> {
        if size > max_size {
            return Err(anyhow!(
                "Content size {} exceeds maximum {}",
                size,
                max_size
            ));
        }
        Ok(())
    }

    pub fn validate_query_length(query: &str, max_length: usize) -> Result<()> {
        if query.len() > max_length {
            return Err(anyhow!(
                "Query length {} exceeds maximum {}",
                query.len(),
                max_length
            ));
        }
        Ok(())
    }
}

/// Parameter validation utilities
pub struct ParameterValidator;

impl ParameterValidator {
    pub fn validate_positive_integer(value: i64, name: &str) -> Result<()> {
        if value <= 0 {
            return Err(anyhow!("{} must be greater than 0", name));
        }
        Ok(())
    }

    pub fn validate_range(value: f64, min: f64, max: f64, name: &str) -> Result<()> {
        if value < min || value > max {
            return Err(anyhow!("{} must be between {} and {}", name, min, max));
        }
        Ok(())
    }

    pub fn validate_non_empty_string(value: &str, name: &str) -> Result<()> {
        if value.trim().is_empty() {
            return Err(anyhow!("{} cannot be empty", name));
        }
        Ok(())
    }

    pub fn validate_url_list(urls: &[String], max_count: usize) -> Result<()> {
        if urls.is_empty() {
            return Err(anyhow!("At least one URL is required"));
        }

        if urls.len() > max_count {
            return Err(anyhow!(
                "Too many URLs: {} (maximum: {})",
                urls.len(),
                max_count
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_validator() {
        let validator = CommonValidator::new_default();

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
    fn test_content_type_validator() {
        assert!(ContentTypeValidator::is_html("text/html"));
        assert!(ContentTypeValidator::is_html("text/html; charset=utf-8"));
        assert!(ContentTypeValidator::is_json("application/json"));
        assert!(ContentTypeValidator::is_xml("text/xml"));
        assert!(ContentTypeValidator::is_pdf("application/pdf"));

        assert!(!ContentTypeValidator::is_html("application/json"));
        assert!(!ContentTypeValidator::is_json("text/html"));
    }

    #[test]
    fn test_size_validator() {
        assert!(SizeValidator::validate_url_length("https://example.com", 2048).is_ok());
        assert!(SizeValidator::validate_url_length(&"x".repeat(3000), 2048).is_err());

        assert!(SizeValidator::validate_content_length(1024, 2048).is_ok());
        assert!(SizeValidator::validate_content_length(3000, 2048).is_err());
    }

    #[test]
    fn test_parameter_validator() {
        assert!(ParameterValidator::validate_positive_integer(5, "count").is_ok());
        assert!(ParameterValidator::validate_positive_integer(0, "count").is_err());
        assert!(ParameterValidator::validate_positive_integer(-1, "count").is_err());

        assert!(ParameterValidator::validate_range(0.5, 0.0, 1.0, "ratio").is_ok());
        assert!(ParameterValidator::validate_range(1.5, 0.0, 1.0, "ratio").is_err());

        assert!(ParameterValidator::validate_non_empty_string("test", "name").is_ok());
        assert!(ParameterValidator::validate_non_empty_string("", "name").is_err());
        assert!(ParameterValidator::validate_non_empty_string("   ", "name").is_err());
    }
}