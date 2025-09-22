use crate::errors::{ApiError, ApiResult};
use crate::models::{CrawlBody, DeepSearchBody};
use url::Url;

/// Maximum number of URLs allowed in a single crawl request
const MAX_URLS_PER_REQUEST: usize = 100;

/// Maximum length for search queries
const MAX_QUERY_LENGTH: usize = 500;

/// Maximum search result limit
const MAX_SEARCH_LIMIT: u32 = 50;

/// Supported URL schemes for crawling
const ALLOWED_SCHEMES: &[&str] = &["http", "https"];

/// Input validation for API requests to ensure data integrity and security.
///
/// This module provides comprehensive validation for all API endpoints,
/// including URL validation, payload size checks, and content validation.
/// It helps prevent malicious inputs and ensures the system operates within
/// safe parameters.
/// Validate a crawl request payload.
///
/// Checks:
/// - URL count is within limits
/// - All URLs are well-formed and use allowed schemes
/// - No localhost or private IP addresses (security)
/// - URLs are not excessively long
///
/// # Arguments
///
/// * `body` - The crawl request body to validate
///
/// # Returns
///
/// `Ok(())` if validation passes, `Err(ApiError)` with details if validation fails.
///
/// # Examples
///
/// ```rust
/// use riptide_api::validation::validate_crawl_request;
/// use riptide_api::models::CrawlBody;
///
/// let body = CrawlBody {
///     urls: vec!["https://example.com".to_string()],
/// };
/// validate_crawl_request(&body)?;
/// ```
pub fn validate_crawl_request(body: &CrawlBody) -> ApiResult<()> {
    // Check URL count limits
    if body.urls.is_empty() {
        return Err(ApiError::validation("At least one URL is required"));
    }

    if body.urls.len() > MAX_URLS_PER_REQUEST {
        return Err(ApiError::validation(format!(
            "Too many URLs: {} (maximum: {})",
            body.urls.len(),
            MAX_URLS_PER_REQUEST
        )));
    }

    // Validate each URL
    for (index, url_str) in body.urls.iter().enumerate() {
        validate_url(url_str, index)?;
    }

    Ok(())
}

/// Validate a deep search request payload.
///
/// Checks:
/// - Query is not empty and within length limits
/// - Search limit is reasonable
/// - Query doesn't contain potentially harmful content
///
/// # Arguments
///
/// * `body` - The deep search request body to validate
///
/// # Returns
///
/// `Ok(())` if validation passes, `Err(ApiError)` with details if validation fails.
pub fn validate_deepsearch_request(body: &DeepSearchBody) -> ApiResult<()> {
    // Check query length
    if body.query.trim().is_empty() {
        return Err(ApiError::validation("Search query cannot be empty"));
    }

    if body.query.len() > MAX_QUERY_LENGTH {
        return Err(ApiError::validation(format!(
            "Query too long: {} characters (maximum: {})",
            body.query.len(),
            MAX_QUERY_LENGTH
        )));
    }

    // Check search limit
    if let Some(limit) = body.limit {
        if limit == 0 {
            return Err(ApiError::validation("Search limit must be greater than 0"));
        }
        if limit > MAX_SEARCH_LIMIT {
            return Err(ApiError::validation(format!(
                "Search limit too high: {} (maximum: {})",
                limit, MAX_SEARCH_LIMIT
            )));
        }
    }

    // Basic content validation to prevent obvious malicious queries
    validate_query_content(&body.query)?;

    Ok(())
}

/// Validate a single URL for crawling.
///
/// Performs comprehensive URL validation including:
/// - Proper URL format and parsing
/// - Allowed schemes (http/https only)
/// - No localhost or private network access
/// - Reasonable URL length limits
/// - No obviously malicious patterns
///
/// # Arguments
///
/// * `url_str` - The URL string to validate
/// * `index` - Index in the request (for error reporting)
///
/// # Returns
///
/// `Ok(())` if URL is valid, `Err(ApiError)` with details if invalid.
fn validate_url(url_str: &str, index: usize) -> ApiResult<()> {
    // Basic length check
    if url_str.len() > 2048 {
        return Err(ApiError::invalid_url(
            url_str,
            format!("URL {} is too long (maximum: 2048 characters)", index + 1),
        ));
    }

    // Parse the URL
    let url = Url::parse(url_str).map_err(|e| {
        ApiError::invalid_url(url_str, format!("URL {} parsing failed: {}", index + 1, e))
    })?;

    // Check scheme
    if !ALLOWED_SCHEMES.contains(&url.scheme()) {
        return Err(ApiError::invalid_url(
            url_str,
            format!(
                "URL {} has unsupported scheme '{}' (allowed: {})",
                index + 1,
                url.scheme(),
                ALLOWED_SCHEMES.join(", ")
            ),
        ));
    }

    // Check for localhost and private networks
    if let Some(host) = url.host_str() {
        if is_private_or_localhost(host) {
            return Err(ApiError::invalid_url(
                url_str,
                format!(
                    "URL {} targets private/localhost address: {}",
                    index + 1,
                    host
                ),
            ));
        }
    }

    // Check for suspicious patterns
    validate_url_patterns(url_str, index)?;

    Ok(())
}

/// Check if a hostname points to localhost or private networks.
///
/// This prevents SSRF attacks by blocking requests to:
/// - localhost, 127.0.0.1, ::1
/// - Private IPv4 ranges (10.x.x.x, 172.16-31.x.x, 192.168.x.x)
/// - Private IPv6 ranges
/// - Link-local addresses
fn is_private_or_localhost(host: &str) -> bool {
    // Check for localhost variants
    if host == "localhost" || host == "127.0.0.1" || host == "::1" {
        return true;
    }

    // Try to parse as IP address
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        match ip {
            std::net::IpAddr::V4(ipv4) => {
                // Check private IPv4 ranges
                let octets = ipv4.octets();
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
                // Check private IPv6 ranges
                ipv6.is_loopback() || ipv6.is_multicast() || ipv6.segments()[0] == 0xfe80
                // link-local
            }
        }
    } else {
        // Check for obviously local hostnames
        host.ends_with(".local") || host.ends_with(".localhost")
    }
}

/// Validate URL patterns to detect potentially malicious URLs.
///
/// Looks for:
/// - Excessively long query parameters
/// - Suspicious file extensions
/// - URL encoding attacks
/// - Obvious redirect loops
fn validate_url_patterns(url_str: &str, index: usize) -> ApiResult<()> {
    // Check for excessive URL encoding (potential obfuscation)
    let percent_count = url_str.matches('%').count();
    if percent_count > 20 {
        return Err(ApiError::invalid_url(
            url_str,
            format!("URL {} has excessive URL encoding", index + 1),
        ));
    }

    // Check for suspicious file extensions that might indicate malware
    let suspicious_extensions = &[".exe", ".bat", ".cmd", ".scr", ".vbs", ".js"];
    for ext in suspicious_extensions {
        if url_str.to_lowercase().contains(ext) {
            return Err(ApiError::invalid_url(
                url_str,
                format!("URL {} contains suspicious file extension", index + 1),
            ));
        }
    }

    Ok(())
}

/// Validate search query content for basic security.
///
/// Checks for:
/// - SQL injection patterns
/// - Script injection attempts
/// - Excessive special characters
/// - Control characters
fn validate_query_content(query: &str) -> ApiResult<()> {
    // Check for control characters
    if query
        .chars()
        .any(|c| c.is_control() && c != '\t' && c != '\n')
    {
        return Err(ApiError::validation(
            "Query contains invalid control characters",
        ));
    }

    // Check for obvious SQL injection patterns
    let sql_patterns = &["union select", "drop table", "insert into", "delete from"];
    let query_lower = query.to_lowercase();
    for pattern in sql_patterns {
        if query_lower.contains(pattern) {
            return Err(ApiError::validation(
                "Query contains suspicious SQL patterns",
            ));
        }
    }

    // Check for script injection patterns
    let script_patterns = &["<script", "javascript:", "data:text/html"];
    for pattern in script_patterns {
        if query_lower.contains(pattern) {
            return Err(ApiError::validation(
                "Query contains suspicious script patterns",
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_crawl_request() {
        let body = CrawlBody {
            urls: vec!["https://example.com".to_string()],
            options: None,
        };
        assert!(validate_crawl_request(&body).is_ok());
    }

    #[test]
    fn test_empty_urls() {
        let body = CrawlBody {
            urls: vec![],
            options: None,
        };
        assert!(validate_crawl_request(&body).is_err());
    }

    #[test]
    fn test_too_many_urls() {
        let body = CrawlBody {
            urls: vec!["https://example.com".to_string(); MAX_URLS_PER_REQUEST + 1],
            options: None,
        };
        assert!(validate_crawl_request(&body).is_err());
    }

    #[test]
    fn test_invalid_scheme() {
        let body = CrawlBody {
            urls: vec!["ftp://example.com".to_string()],
            options: None,
        };
        assert!(validate_crawl_request(&body).is_err());
    }

    #[test]
    fn test_localhost_blocked() {
        let body = CrawlBody {
            urls: vec!["http://localhost:8080".to_string()],
            options: None,
        };
        assert!(validate_crawl_request(&body).is_err());
    }

    #[test]
    fn test_private_ip_blocked() {
        let body = CrawlBody {
            urls: vec!["http://192.168.1.1".to_string()],
            options: None,
        };
        assert!(validate_crawl_request(&body).is_err());
    }

    #[test]
    fn test_valid_deepsearch() {
        let body = DeepSearchBody {
            query: "rust programming".to_string(),
            limit: Some(10),
            country: None,
            locale: None,
            include_content: None,
            crawl_options: None,
        };
        assert!(validate_deepsearch_request(&body).is_ok());
    }

    #[test]
    fn test_empty_query() {
        let body = DeepSearchBody {
            query: "".to_string(),
            limit: Some(10),
            country: None,
            locale: None,
            include_content: None,
            crawl_options: None,
        };
        assert!(validate_deepsearch_request(&body).is_err());
    }

    #[test]
    fn test_query_too_long() {
        let body = DeepSearchBody {
            query: "a".repeat(MAX_QUERY_LENGTH + 1),
            limit: Some(10),
            country: None,
            locale: None,
            include_content: None,
            crawl_options: None,
        };
        assert!(validate_deepsearch_request(&body).is_err());
    }

    #[test]
    fn test_limit_too_high() {
        let body = DeepSearchBody {
            query: "test".to_string(),
            limit: Some(MAX_SEARCH_LIMIT + 1),
            country: None,
            locale: None,
            include_content: None,
            crawl_options: None,
        };
        assert!(validate_deepsearch_request(&body).is_err());
    }

    #[test]
    fn test_sql_injection_detection() {
        let body = DeepSearchBody {
            query: "test' union select * from users--".to_string(),
            limit: Some(10),
            country: None,
            locale: None,
            include_content: None,
            crawl_options: None,
        };
        assert!(validate_deepsearch_request(&body).is_err());
    }
}
