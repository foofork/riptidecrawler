//! Metadata extraction from HTML documents

use scraper::{Html, Selector};

pub struct MetadataExtractor;

impl MetadataExtractor {
    /// Extract author/byline information
    pub fn extract_byline(document: &Html) -> Option<String> {
        // Try meta tags first
        if let Some(author) = Self::extract_meta_author(document) {
            return Some(author);
        }

        // Try schema.org author
        if let Some(author) = Self::extract_schema_author(document) {
            return Some(author);
        }

        // Try common byline selectors
        Self::extract_byline_from_selectors(document)
    }

    /// Extract published date
    pub fn extract_published_date(document: &Html) -> Option<String> {
        // Try meta tags
        if let Some(date) = Self::extract_meta_date(document) {
            return Some(date);
        }

        // Try schema.org datePublished
        if let Some(date) = Self::extract_schema_date(document) {
            return Some(date);
        }

        // Try time element
        Self::extract_time_element(document)
    }

    /// Extract description
    pub fn extract_description(document: &Html) -> Option<String> {
        // Priority 1: Open Graph description
        if let Some(desc) = Self::extract_og_description(document) {
            return Some(desc);
        }

        // Priority 2: Twitter description
        if let Some(desc) = Self::extract_twitter_description(document) {
            return Some(desc);
        }

        // Priority 3: Meta description
        Self::extract_meta_description(document)
    }

    /// Extract site name
    pub fn extract_site_name(document: &Html) -> Option<String> {
        // Open Graph site_name
        let selector = Selector::parse("meta[property='og:site_name']").ok()?;
        document
            .select(&selector)
            .next()?
            .value()
            .attr("content")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn extract_meta_author(document: &Html) -> Option<String> {
        let selectors = [
            "meta[name='author']",
            "meta[property='article:author']",
            "meta[name='twitter:creator']",
        ];

        for selector_str in &selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    if let Some(content) = element.value().attr("content") {
                        let author = content.trim().to_string();
                        if !author.is_empty() {
                            return Some(author);
                        }
                    }
                }
            }
        }
        None
    }

    fn extract_schema_author(document: &Html) -> Option<String> {
        let selector = Selector::parse("[itemprop='author']").ok()?;
        let author: String = document.select(&selector).next()?.text().collect();
        let cleaned = author.trim().to_string();
        if cleaned.is_empty() {
            None
        } else {
            Some(cleaned)
        }
    }

    fn extract_byline_from_selectors(document: &Html) -> Option<String> {
        let selectors = [
            ".author",
            ".byline",
            "[rel='author']",
            ".author-name",
            ".post-author",
        ];

        for selector_str in &selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    let author: String = element.text().collect();
                    let cleaned = author.trim().to_string();
                    if !cleaned.is_empty() {
                        return Some(cleaned);
                    }
                }
            }
        }
        None
    }

    fn extract_meta_date(document: &Html) -> Option<String> {
        let selectors = [
            "meta[property='article:published_time']",
            "meta[name='date']",
            "meta[name='publication_date']",
            "meta[property='og:published_time']",
        ];

        for selector_str in &selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    if let Some(content) = element.value().attr("content") {
                        let date = content.trim().to_string();
                        if !date.is_empty() {
                            return Some(date);
                        }
                    }
                }
            }
        }
        None
    }

    fn extract_schema_date(document: &Html) -> Option<String> {
        let selector = Selector::parse("[itemprop='datePublished']").ok()?;
        let element = document.select(&selector).next()?;

        // Try datetime attribute first
        if let Some(datetime) = element.value().attr("datetime") {
            return Some(datetime.trim().to_string());
        }

        // Otherwise get text content
        let date: String = element.text().collect();
        let cleaned = date.trim().to_string();
        if cleaned.is_empty() {
            None
        } else {
            Some(cleaned)
        }
    }

    fn extract_time_element(document: &Html) -> Option<String> {
        let selector = Selector::parse("time[datetime]").ok()?;
        document
            .select(&selector)
            .next()?
            .value()
            .attr("datetime")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn extract_og_description(document: &Html) -> Option<String> {
        let selector = Selector::parse("meta[property='og:description']").ok()?;
        document
            .select(&selector)
            .next()?
            .value()
            .attr("content")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn extract_twitter_description(document: &Html) -> Option<String> {
        let selector = Selector::parse("meta[name='twitter:description']").ok()?;
        document
            .select(&selector)
            .next()?
            .value()
            .attr("content")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn extract_meta_description(document: &Html) -> Option<String> {
        let selector = Selector::parse("meta[name='description']").ok()?;
        document
            .select(&selector)
            .next()?
            .value()
            .attr("content")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }
}
