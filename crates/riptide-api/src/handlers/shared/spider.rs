//! Spider-specific shared utilities

use crate::errors::ApiError;
use url::Url;

/// Parse and validate seed URLs from string inputs
pub fn parse_seed_urls(urls: &[String]) -> Result<Vec<Url>, ApiError> {
    urls.iter()
        .map(|url_str| {
            Url::parse(url_str).map_err(|e| ApiError::ValidationError {
                message: format!("Invalid URL '{}': {}", url_str, e),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_seed_urls_valid() {
        let urls = vec![
            "https://example.com".to_string(),
            "https://test.com/path".to_string(),
        ];

        let result = parse_seed_urls(&urls);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_parse_seed_urls_invalid() {
        let urls = vec!["not-a-url".to_string()];

        let result = parse_seed_urls(&urls);
        assert!(result.is_err());
    }
}
