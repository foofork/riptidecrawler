//! None provider - parses URLs from query without external API

use super::{SearchBackend, SearchHit, SearchProvider};
use anyhow::Result;
use regex::Regex;

/// None provider that extracts URLs from the query string.
///
/// This provider doesn't perform actual web searches but instead
/// parses URLs directly from the query string, useful for when
/// users paste URLs directly or when no search API is configured.
#[derive(Debug, Clone)]
pub struct NoneProvider {
    enable_url_parsing: bool,
    url_regex: Regex,
}

impl NoneProvider {
    /// Create a new NoneProvider.
    ///
    /// # Parameters
    /// - `enable_url_parsing`: Whether to parse URLs from query string
    pub fn new(enable_url_parsing: bool) -> Self {
        let url_regex = Regex::new(r"https?://[^\s,\n]+").expect("Failed to compile URL regex");
        Self {
            enable_url_parsing,
            url_regex,
        }
    }

    /// Extract URLs from a query string.
    ///
    /// Supports multiple formats:
    /// - Space-separated: "https://a.com https://b.com"
    /// - Comma-separated: "https://a.com,https://b.com"
    /// - Newline-separated: "https://a.com\nhttps://b.com"
    /// - Mixed with text: "Check https://example.com and https://test.org"
    fn extract_urls(&self, query: &str) -> Vec<String> {
        if !self.enable_url_parsing {
            return vec![];
        }

        self.url_regex
            .find_iter(query)
            .map(|m| m.as_str().trim_end_matches(&[',', '.', ';', ')', ']', '}'] as &[_]))
            .filter(|url| {
                // Validate URL is well-formed
                url::Url::parse(url).is_ok()
            })
            .map(|s| s.to_string())
            .collect()
    }
}

#[async_trait::async_trait]
impl SearchProvider for NoneProvider {
    async fn search(
        &self,
        query: &str,
        _limit: u32,
        _country: &str,
        _locale: &str,
    ) -> Result<Vec<SearchHit>> {
        let urls = self.extract_urls(query);

        if urls.is_empty() {
            return Err(anyhow::anyhow!(
                "No URLs found in query. Either paste URLs directly or configure a search backend."
            ));
        }

        // Convert URLs to SearchHit results
        let results: Vec<SearchHit> = urls
            .into_iter()
            .enumerate()
            .map(|(index, url)| {
                SearchHit::new(url.clone(), (index + 1) as u32)
                    .with_metadata("source".to_string(), "query".to_string())
            })
            .collect();

        Ok(results)
    }

    fn backend_type(&self) -> SearchBackend {
        SearchBackend::None
    }

    async fn health_check(&self) -> Result<()> {
        // None provider is always healthy as it doesn't rely on external services
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_extraction_space_separated() {
        let provider = NoneProvider::new(true);
        let urls = provider.extract_urls("https://example.com https://test.org");
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "https://example.com");
        assert_eq!(urls[1], "https://test.org");
    }

    #[test]
    fn test_url_extraction_comma_separated() {
        let provider = NoneProvider::new(true);
        let urls = provider.extract_urls("https://example.com,https://test.org");
        assert_eq!(urls.len(), 2);
    }

    #[test]
    fn test_url_extraction_mixed_text() {
        let provider = NoneProvider::new(true);
        let urls = provider.extract_urls("Check out https://example.com and also https://test.org for more info");
        assert_eq!(urls.len(), 2);
    }

    #[test]
    fn test_url_extraction_with_ports() {
        let provider = NoneProvider::new(true);
        let urls = provider.extract_urls("http://localhost:8080 https://example.com:443");
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "http://localhost:8080");
        assert_eq!(urls[1], "https://example.com:443");
    }

    #[test]
    fn test_url_extraction_disabled() {
        let provider = NoneProvider::new(false);
        let urls = provider.extract_urls("https://example.com");
        assert_eq!(urls.len(), 0);
    }

    #[test]
    fn test_invalid_urls_filtered() {
        let provider = NoneProvider::new(true);
        let urls = provider.extract_urls("https://example.com not-a-url https:// http://valid.com");
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "https://example.com");
        assert_eq!(urls[1], "http://valid.com");
    }
}