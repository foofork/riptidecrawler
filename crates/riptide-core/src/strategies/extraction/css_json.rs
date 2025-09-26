//! CSS selector to JSON extraction strategy

use anyhow::Result;
use scraper::{Html, Selector};
use std::collections::HashMap;
use crate::strategies::{ExtractedContent, extraction::*};

pub struct CssJsonExtractor {
    selectors: HashMap<String, String>,
}

impl CssJsonExtractor {
    pub fn new(selectors: HashMap<String, String>) -> Self {
        Self { selectors }
    }
}

impl ContentExtractor for CssJsonExtractor {
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractedContent> {
        let document = Html::parse_document(html);
        let mut extracted_data = HashMap::new();

        // Extract data using configured selectors
        for (field, selector_str) in &self.selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                let values: Vec<String> = document
                    .select(&selector)
                    .map(|element| {
                        // Check for content attribute (meta tags)
                        if let Some(content) = element.value().attr("content") {
                            content.to_string()
                        } else {
                            element.text().collect::<Vec<_>>().join(" ").trim().to_string()
                        }
                    })
                    .filter(|text| !text.is_empty())
                    .collect();

                if !values.is_empty() {
                    extracted_data.insert(field.clone(), values);
                }
            }
        }

        // Build content from extracted data
        let title = extracted_data
            .get("title")
            .and_then(|v| v.first())
            .cloned()
            .unwrap_or_else(|| "Untitled".to_string());

        let content = extracted_data
            .get("content")
            .map(|v| v.join("

"))
            .or_else(|| extracted_data.get("body").map(|v| v.join("

")))
            .unwrap_or_else(|| {
                // Fallback: combine all non-title fields
                extracted_data
                    .iter()
                    .filter(|(k, _)| *k != "title")
                    .flat_map(|(_, v)| v.iter())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("

")
            });

        let summary = extracted_data
            .get("description")
            .and_then(|v| v.first())
            .cloned();

        Ok(ExtractedContent {
            title,
            content,
            summary,
            url: url.to_string(),
            strategy_used: "css_json".to_string(),
            extraction_confidence: self.confidence_score(html),
        })
    }

    fn confidence_score(&self, html: &str) -> f64 {
        let document = Html::parse_document(html);
        let mut found_selectors = 0;
        let total_selectors = self.selectors.len();

        for selector_str in self.selectors.values() {
            if let Ok(selector) = Selector::parse(selector_str) {
                if document.select(&selector).next().is_some() {
                    found_selectors += 1;
                }
            }
        }

        if total_selectors == 0 {
            0.5
        } else {
            (found_selectors as f64 / total_selectors as f64) * 0.9
        }
    }

    fn strategy_name(&self) -> &'static str {
        "css_json"
    }
}

/// Default CSS selectors for common content types
pub fn default_selectors() -> HashMap<String, String> {
    let mut selectors = HashMap::new();

    selectors.insert("title".to_string(), "title, h1, [property='og:title']".to_string());
    selectors.insert("description".to_string(), "[name='description'], [property='og:description']".to_string());
    selectors.insert("content".to_string(), "article, .content, .post-content, .entry-content, main".to_string());
    selectors.insert("author".to_string(), "[name='author'], .author, .byline, [rel='author']".to_string());
    selectors.insert("date".to_string(), "time, .date, .published, [property='article:published_time']".to_string());
    selectors.insert("tags".to_string(), ".tag, .category, .label, [property='article:tag']".to_string());

    selectors
}

/// Direct extraction function with custom selectors
pub async fn extract(html: &str, url: &str, selectors: &HashMap<String, String>) -> Result<ExtractedContent> {
    let extractor = CssJsonExtractor::new(selectors.clone());
    extractor.extract(html, url).await
}

/// Direct extraction function with default selectors
pub async fn extract_default(html: &str, url: &str) -> Result<ExtractedContent> {
    extract(html, url, &default_selectors()).await
}