//! None provider - parses URLs from query without external API

use super::{SearchBackend, SearchHit, SearchProvider};
use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;

/// Compile-time URL regex pattern for extracting HTTP(S) URLs from text
static URL_REGEX: Lazy<Option<Regex>> = Lazy::new(|| {
    // This regex is a compile-time constant and should always compile successfully,
    // but we handle the error to satisfy clippy's expect_used lint
    Regex::new(r"https?://[^\s,\n]+").ok()
});

/// None provider that extracts URLs from the query string.
///
/// This provider doesn't perform actual web searches but instead
/// parses URLs directly from the query string, useful for when
/// users paste URLs directly or when no search API is configured.
#[derive(Clone)]
pub struct NoneProvider {
    enable_url_parsing: bool,
}

impl NoneProvider {
    /// Create a new NoneProvider.
    ///
    /// # Parameters
    /// - `enable_url_parsing`: Whether to parse URLs from query string
    pub fn new(enable_url_parsing: bool) -> Self {
        Self { enable_url_parsing }
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

        URL_REGEX
            .as_ref()
            .map(|regex| {
                regex
                    .find_iter(query)
                    .map(|m| {
                        m.as_str()
                            .trim_end_matches(&[',', '.', ';', ')', ']', '}'] as &[_])
                    })
                    .filter(|url| {
                        // Validate URL is well-formed
                        url::Url::parse(url).is_ok()
                    })
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[async_trait::async_trait]
impl SearchProvider for NoneProvider {
    async fn search(
        &self,
        query: &str,
        limit: u32,
        _country: &str,
        _locale: &str,
    ) -> Result<Vec<SearchHit>> {
        if !self.enable_url_parsing {
            return Err(anyhow::anyhow!(
                "URL parsing is disabled for NoneProvider. Enable it or configure a search backend."
            ));
        }

        let urls = self.extract_urls(query);

        if urls.is_empty() {
            return Err(anyhow::anyhow!(
                "No URLs found in query. Either paste URLs directly or configure a search backend."
            ));
        }

        // Convert URLs to SearchHit results, respecting the limit
        let results: Vec<SearchHit> = urls
            .into_iter()
            .take(limit as usize)
            .enumerate()
            .map(|(index, url)| {
                // Safe rank calculation: index is bounded by limit (max u32), saturating add prevents overflow
                let rank = u32::try_from(index).unwrap_or(u32::MAX).saturating_add(1);
                SearchHit::new(url.clone(), rank)
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

impl std::fmt::Debug for NoneProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NoneProvider")
            .field("enable_url_parsing", &self.enable_url_parsing)
            .finish()
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
        let urls = provider
            .extract_urls("Check out https://example.com and also https://test.org for more info");
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
