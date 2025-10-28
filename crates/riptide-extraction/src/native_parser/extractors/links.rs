//! Link extraction from HTML documents

use scraper::{Html, Selector};
use std::collections::HashSet;
use url::Url;

pub struct LinkExtractor;

impl LinkExtractor {
    /// Extract and deduplicate links from document
    pub fn extract(document: &Html, base_url: &str) -> Vec<String> {
        let mut links = HashSet::new();
        let base = Url::parse(base_url).ok();

        if let Ok(selector) = Selector::parse("a[href]") {
            for element in document.select(&selector) {
                if let Some(href) = element.value().attr("href") {
                    // Resolve relative URLs
                    let resolved = if let Some(ref base) = base {
                        base.join(href).ok().map(|u| u.to_string())
                    } else {
                        Some(href.to_string())
                    };

                    if let Some(url) = resolved {
                        // Filter out non-http(s) URLs and fragments
                        if Self::is_valid_link(&url) {
                            links.insert(url);
                        }
                    }
                }
            }
        }

        links.into_iter().collect()
    }

    fn is_valid_link(url: &str) -> bool {
        // Must start with http:// or https://
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return false;
        }

        // Exclude common non-content URLs
        let excluded_patterns = [
            "#",
            "javascript:",
            "mailto:",
            "tel:",
            ".pdf",
            ".zip",
            ".exe",
        ];

        for pattern in &excluded_patterns {
            if url.contains(pattern) {
                return false;
            }
        }

        true
    }
}
