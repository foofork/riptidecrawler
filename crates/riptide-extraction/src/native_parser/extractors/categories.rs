//! Category/tag extraction from HTML documents

use scraper::{Html, Selector};

pub struct CategoryExtractor;

impl CategoryExtractor {
    /// Extract categories/tags from document
    pub fn extract(document: &Html) -> Vec<String> {
        let mut categories = Vec::new();

        // Extract from meta keywords
        categories.extend(Self::extract_meta_keywords(document));

        // Extract from article:tag meta tags
        categories.extend(Self::extract_article_tags(document));

        // Extract from common tag/category selectors
        categories.extend(Self::extract_from_selectors(document));

        // Deduplicate and clean
        categories.sort();
        categories.dedup();
        categories.into_iter().filter(|c| !c.is_empty()).collect()
    }

    fn extract_meta_keywords(document: &Html) -> Vec<String> {
        if let Ok(selector) = Selector::parse("meta[name='keywords']") {
            if let Some(element) = document.select(&selector).next() {
                if let Some(content) = element.value().attr("content") {
                    return content
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }
        }
        Vec::new()
    }

    fn extract_article_tags(document: &Html) -> Vec<String> {
        let mut tags = Vec::new();

        if let Ok(selector) = Selector::parse("meta[property='article:tag']") {
            for element in document.select(&selector) {
                if let Some(content) = element.value().attr("content") {
                    let tag = content.trim().to_string();
                    if !tag.is_empty() {
                        tags.push(tag);
                    }
                }
            }
        }

        tags
    }

    fn extract_from_selectors(document: &Html) -> Vec<String> {
        let selectors = [
            ".tag",
            ".category",
            "[rel='tag']",
            ".post-tag",
            ".article-tag",
        ];

        let mut tags = Vec::new();

        for selector_str in &selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    let tag: String = element.text().collect();
                    let cleaned = tag.trim().to_string();
                    if !cleaned.is_empty() {
                        tags.push(cleaned);
                    }
                }
            }
        }

        tags
    }
}
