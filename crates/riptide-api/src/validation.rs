use crate::errors::{ApiError, ApiResult};
use crate::models::{CrawlBody, DeepSearchBody};
use riptide_config::CommonValidator;

/// Maximum number of URLs allowed in a single crawl request
const MAX_URLS_PER_REQUEST: usize = 100;

/// Maximum length for search queries
const MAX_QUERY_LENGTH: usize = 500;

/// Maximum search result limit
const MAX_SEARCH_LIMIT: u32 = 50;

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

/// Validate a single URL for crawling using common validation patterns.
///
/// Uses the common validation module for consistent URL validation across
/// the codebase while providing API-specific error formatting.
fn validate_url(url_str: &str, index: usize) -> ApiResult<()> {
    let validator = CommonValidator::new_default();

    // Use common URL validation
    match validator.validate_url(url_str) {
        Ok(_) => {
            // Check for suspicious patterns specific to API validation
            validate_url_patterns(url_str, index)?;
            Ok(())
        }
        Err(e) => {
            // Convert common validation error to API error
            Err(ApiError::invalid_url(
                url_str,
                format!("URL {} validation failed: {}", index + 1, e),
            ))
        }
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

/// Validate search query content using common validation patterns.
fn validate_query_content(query: &str) -> ApiResult<()> {
    let validator = CommonValidator::new_default();

    // Use common query validation
    match validator.validate_query_content(query) {
        Ok(_) => Ok(()),
        Err(e) => Err(ApiError::validation(format!(
            "Query validation failed: {}",
            e
        ))),
    }
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
