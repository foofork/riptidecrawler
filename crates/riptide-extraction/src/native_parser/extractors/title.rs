//! Title extraction from HTML documents

use scraper::{Html, Selector};

pub struct TitleExtractor;

impl TitleExtractor {
    /// Extract title using multiple strategies with priority fallback
    pub fn extract(document: &Html) -> Option<String> {
        // Priority 1: Open Graph title
        if let Some(title) = Self::extract_og_title(document) {
            return Some(title);
        }

        // Priority 2: Twitter title
        if let Some(title) = Self::extract_twitter_title(document) {
            return Some(title);
        }

        // Priority 3: <title> tag
        if let Some(title) = Self::extract_html_title(document) {
            return Some(title);
        }

        // Priority 4: <h1> tag
        Self::extract_h1_title(document)
    }

    fn extract_og_title(document: &Html) -> Option<String> {
        let selector = Selector::parse("meta[property='og:title']").ok()?;
        document
            .select(&selector)
            .next()?
            .value()
            .attr("content")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn extract_twitter_title(document: &Html) -> Option<String> {
        let selector = Selector::parse("meta[name='twitter:title']").ok()?;
        document
            .select(&selector)
            .next()?
            .value()
            .attr("content")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn extract_html_title(document: &Html) -> Option<String> {
        let selector = Selector::parse("title").ok()?;
        let title = document
            .select(&selector)
            .next()?
            .text()
            .collect::<String>();

        let cleaned = title.trim().to_string();
        if cleaned.is_empty() {
            None
        } else {
            Some(cleaned)
        }
    }

    fn extract_h1_title(document: &Html) -> Option<String> {
        let selector = Selector::parse("h1").ok()?;
        let h1 = document
            .select(&selector)
            .next()?
            .text()
            .collect::<String>();

        let cleaned = h1.trim().to_string();
        if cleaned.is_empty() || cleaned.len() > 200 {
            None
        } else {
            Some(cleaned)
        }
    }
}
