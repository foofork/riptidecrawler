//! Trek extraction strategy - Default WASM-based extraction

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use crate::strategies::{ExtractedContent, extraction::*};
use crate::component::CmExtractor;
use crate::types::ExtractedDoc;

pub struct TrekExtractor {
    wasm_extractor: Option<CmExtractor>,
}

impl TrekExtractor {
    pub async fn new(wasm_path: Option<&str>) -> Result<Self> {
        let wasm_extractor = if let Some(path) = wasm_path {
            Some(CmExtractor::new(path).await?)
        } else {
            None
        };

        Ok(Self { wasm_extractor })
    }
}

impl ContentExtractor for TrekExtractor {
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractedContent> {
        if let Some(ref extractor) = self.wasm_extractor {
            let extracted = extractor.extract(html, url, "default")?;

            Ok(ExtractedContent {
                title: extracted.title.unwrap_or_else(|| "Untitled".to_string()),
                content: extracted.text,
                summary: None,  // ExtractedDoc doesn't have description field
                url: url.to_string(),
                strategy_used: "trek".to_string(),
                extraction_confidence: self.confidence_score(html),
            })
        } else {
            // Fallback extraction without WASM
            fallback_extract(html, url).await
        }
    }

    fn confidence_score(&self, html: &str) -> f64 {
        let mut score = 0.8_f64; // Base score for Trek

        // Adjust based on content indicators
        if html.contains("<article") {
            score += 0.1;
        }
        if html.contains("class=\"content\"") || html.contains("class=\"post\"") {
            score += 0.05;
        }
        if html.len() > 5000 {
            score += 0.05;
        }

        score.min(1.0_f64)
    }

    fn strategy_name(&self) -> &'static str {
        "trek"
    }
}

/// Fallback extraction when WASM is not available
pub async fn fallback_extract(html: &str, url: &str) -> Result<ExtractedContent> {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);

    // Try multiple selectors for title
    let title_selectors = [
        "title",
        "h1",
        "[property='og:title']",
        ".title",
        ".headline",
    ];

    let mut title = "Untitled".to_string();
    for selector_str in &title_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                if *selector_str == "[property='og:title']" {
                    if let Some(content) = element.value().attr("content") {
                        title = content.to_string();
                        break;
                    }
                } else {
                    title = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                    if !title.is_empty() {
                        break;
                    }
                }
            }
        }
    }

    // Extract main content
    let content_selectors = [
        "article",
        ".content",
        ".post-content",
        ".entry-content",
        "main",
        ".main",
        "#content",
        ".post",
    ];

    let mut content = String::new();
    for selector_str in &content_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                content = element.text().collect::<Vec<_>>().join(" ");
                if content.len() > 100 {
                    break;
                }
            }
        }
    }

    // If no good content found, try body
    if content.len() < 100 {
        if let Ok(selector) = Selector::parse("body") {
            if let Some(element) = document.select(&selector).next() {
                content = element.text().collect::<Vec<_>>().join(" ");
            }
        }
    }

    // Clean up content
    content = content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();

    Ok(ExtractedContent {
        title,
        content,
        summary: None,
        url: url.to_string(),
        strategy_used: "trek_fallback".to_string(),
        extraction_confidence: 0.6, // Lower confidence for fallback
    })
}

/// Direct extraction function for compatibility
pub async fn extract(html: &str, url: &str) -> Result<ExtractedContent> {
    let extractor = TrekExtractor::new(None).await?;
    extractor.extract(html, url).await
}