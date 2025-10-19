use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// HTTP conditional request support for ETag and Last-Modified headers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalRequest {
    /// ETag from If-None-Match header
    pub if_none_match: Option<String>,
    /// DateTime from If-Modified-Since header
    pub if_modified_since: Option<DateTime<Utc>>,
    /// ETag from If-Match header
    pub if_match: Option<String>,
    /// DateTime from If-Unmodified-Since header
    pub if_unmodified_since: Option<DateTime<Utc>>,
}

impl ConditionalRequest {
    /// Check if request has conditional headers
    pub fn has_conditions(&self) -> bool {
        self.if_none_match.is_some()
            || self.if_modified_since.is_some()
            || self.if_match.is_some()
            || self.if_unmodified_since.is_some()
    }
}

/// HTTP conditional response information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalResponse {
    /// Generated ETag for the response
    pub etag: Option<String>,
    /// Last-Modified timestamp
    pub last_modified: Option<DateTime<Utc>>,
    /// Whether the response should be 304 Not Modified
    pub not_modified: bool,
    /// Cache-Control header value
    pub cache_control: Option<String>,
}

impl ConditionalResponse {
    pub fn new() -> Self {
        Self {
            etag: None,
            last_modified: None,
            not_modified: false,
            cache_control: None,
        }
    }

    /// Generate ETag from content
    pub fn with_etag_from_content(mut self, content: &[u8]) -> Self {
        self.etag = Some(generate_etag(content));
        self
    }

    /// Set ETag manually
    pub fn with_etag(mut self, etag: String) -> Self {
        self.etag = Some(etag);
        self
    }

    /// Set Last-Modified timestamp
    pub fn with_last_modified(mut self, timestamp: DateTime<Utc>) -> Self {
        self.last_modified = Some(timestamp);
        self
    }

    /// Set Cache-Control header
    pub fn with_cache_control(mut self, cache_control: String) -> Self {
        self.cache_control = Some(cache_control);
        self
    }

    /// Check if response matches conditional request (should return 304)
    pub fn check_conditions(&mut self, request: &ConditionalRequest) -> bool {
        let mut not_modified = false;

        // Check If-None-Match (ETag)
        if let (Some(client_etag), Some(response_etag)) = (&request.if_none_match, &self.etag) {
            if client_etag == "*" || client_etag == response_etag {
                not_modified = true;
            }
        }

        // Check If-Modified-Since
        if let (Some(if_modified), Some(last_modified)) =
            (&request.if_modified_since, &self.last_modified)
        {
            if *last_modified <= *if_modified {
                not_modified = true;
            }
        }

        // Check If-Match (for unsafe methods)
        if let (Some(client_etag), Some(response_etag)) = (&request.if_match, &self.etag) {
            if client_etag != "*" && client_etag != response_etag {
                not_modified = true;
            }
        }

        // Check If-Unmodified-Since
        if let (Some(if_unmodified), Some(last_modified)) =
            (&request.if_unmodified_since, &self.last_modified)
        {
            if *last_modified > *if_unmodified {
                not_modified = true;
            }
        }

        self.not_modified = not_modified;
        not_modified
    }
}

impl Default for ConditionalResponse {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate ETag from content using SHA-256
pub fn generate_etag(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let hash = hasher.finalize();
    format!("{:x}", hash)[..16].to_string() // Use first 16 chars for brevity
}

/// Generate weak ETag (W/) for dynamic content
pub fn generate_weak_etag(content: &[u8]) -> String {
    format!("W/\"{}\"", generate_etag(content))
}

/// Parse HTTP date string to DateTime<Utc>
pub fn parse_http_date(date_str: &str) -> Option<DateTime<Utc>> {
    // Try RFC 2822 format first
    if let Ok(date) = DateTime::parse_from_rfc2822(date_str) {
        return Some(date.with_timezone(&Utc));
    }

    // Try RFC 3339 format
    if let Ok(date) = DateTime::parse_from_rfc3339(date_str) {
        return Some(date.with_timezone(&Utc));
    }

    // Try common HTTP date formats
    let formats = [
        "%a, %d %b %Y %H:%M:%S GMT", // RFC 1123
        "%A, %d-%b-%y %H:%M:%S GMT", // RFC 1036
        "%a %b %d %H:%M:%S %Y",      // ANSI C asctime()
    ];

    for format in &formats {
        if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(date_str, format) {
            return Some(DateTime::from_naive_utc_and_offset(naive, Utc));
        }
    }

    None
}

/// Format DateTime as HTTP date string (RFC 1123)
pub fn format_http_date(date: DateTime<Utc>) -> String {
    date.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

/// Cache validation result
#[derive(Debug)]
pub enum CacheValidation {
    /// Cache is still valid (304 Not Modified)
    Valid,
    /// Cache is stale, need to refetch
    Stale,
    /// No cache validation possible
    Unknown,
}

/// Validate cached content against server response
pub fn validate_cache(
    cached_etag: Option<&str>,
    cached_last_modified: Option<DateTime<Utc>>,
    server_etag: Option<&str>,
    server_last_modified: Option<DateTime<Utc>>,
) -> CacheValidation {
    // Check ETag first (more reliable)
    if let (Some(cached), Some(server)) = (cached_etag, server_etag) {
        if cached == server {
            return CacheValidation::Valid;
        } else {
            return CacheValidation::Stale;
        }
    }

    // Fall back to Last-Modified comparison
    if let (Some(cached), Some(server)) = (cached_last_modified, server_last_modified) {
        if cached >= server {
            return CacheValidation::Valid;
        } else {
            return CacheValidation::Stale;
        }
    }

    CacheValidation::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_etag_generation() {
        let content = b"Hello, World!";
        let etag = generate_etag(content);
        assert!(!etag.is_empty());
        assert_eq!(etag.len(), 16);

        // Same content should generate same ETag
        let etag2 = generate_etag(content);
        assert_eq!(etag, etag2);

        // Different content should generate different ETag
        let etag3 = generate_etag(b"Different content");
        assert_ne!(etag, etag3);
    }

    #[test]
    fn test_weak_etag() {
        let content = b"Dynamic content";
        let weak_etag = generate_weak_etag(content);
        assert!(weak_etag.starts_with(r#"W/""#));
        assert!(weak_etag.ends_with("\""));
    }

    #[test]
    fn test_http_date_parsing() {
        // RFC 1123 format
        let date_str = "Wed, 21 Oct 2015 07:28:00 GMT";
        let parsed = parse_http_date(date_str);
        assert!(parsed.is_some());

        // Invalid format
        let invalid = parse_http_date("invalid date");
        assert!(invalid.is_none());
    }

    #[test]
    fn test_http_date_formatting() {
        let date = DateTime::parse_from_rfc3339("2015-10-21T07:28:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let formatted = format_http_date(date);
        assert_eq!(formatted, "Wed, 21 Oct 2015 07:28:00 GMT");
    }

    #[test]
    fn test_conditional_response_matching() {
        let mut response = ConditionalResponse::new()
            .with_etag("abc123".to_string())
            .with_last_modified(Utc::now());

        let request = ConditionalRequest {
            if_none_match: Some("abc123".to_string()),
            if_modified_since: None,
            if_match: None,
            if_unmodified_since: None,
        };

        let not_modified = response.check_conditions(&request);
        assert!(not_modified);
        assert!(response.not_modified);
    }

    #[test]
    fn test_cache_validation() {
        let cached_etag = Some("abc123");
        let server_etag = Some("abc123");

        let result = validate_cache(cached_etag, None, server_etag, None);
        assert!(matches!(result, CacheValidation::Valid));

        let different_etag = Some("xyz789");
        let result = validate_cache(cached_etag, None, different_etag, None);
        assert!(matches!(result, CacheValidation::Stale));
    }

    #[test]
    fn test_last_modified_validation() {
        let cached_time = Some(Utc::now());
        let older_time = Some(Utc::now() - chrono::Duration::hours(1));
        let newer_time = Some(Utc::now() + chrono::Duration::hours(1));

        let result = validate_cache(None, cached_time, None, older_time);
        assert!(matches!(result, CacheValidation::Valid));

        let result = validate_cache(None, cached_time, None, newer_time);
        assert!(matches!(result, CacheValidation::Stale));
    }
}
