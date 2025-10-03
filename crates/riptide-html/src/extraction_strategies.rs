//! Content extraction strategies moved from riptide-core
//!
//! This module provides different content extraction strategies including
//! Trek-based extraction and fallback methods.

use crate::{wasm_extraction::CmExtractor, ExtractedContent};
use anyhow::Result;
use async_trait::async_trait;
use scraper::{Html, Selector};
use url::Url;

/// Common extraction trait for all strategies
#[async_trait]
pub trait ContentExtractor: Send + Sync {
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractedContent>;
    fn confidence_score(&self, html: &str) -> f64;
    fn strategy_name(&self) -> &'static str;
}

/// Trek extraction strategy - Default WASM-based extraction
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

#[async_trait]
impl ContentExtractor for TrekExtractor {
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractedContent> {
        if let Some(ref extractor) = self.wasm_extractor {
            let extracted = extractor.extract(html, url, "default")?;
            Ok(extracted.into())
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
                    title = element
                        .text()
                        .collect::<Vec<_>>()
                        .join(" ")
                        .trim()
                        .to_string();
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

/// CSS-based extraction strategy
pub struct CssExtractorStrategy {
    selector_map: std::collections::HashMap<String, String>,
}

impl Default for CssExtractorStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl CssExtractorStrategy {
    pub fn new() -> Self {
        let mut selector_map = std::collections::HashMap::new();

        // Default selectors
        selector_map.insert(
            "title".to_string(),
            "title, h1, .title, .headline".to_string(),
        );
        selector_map.insert(
            "content".to_string(),
            "article, .content, .post-content, .entry-content, main".to_string(),
        );
        selector_map.insert(
            "summary".to_string(),
            "meta[name='description'], .summary, .excerpt".to_string(),
        );

        Self { selector_map }
    }

    pub fn with_selectors(mut self, selectors: std::collections::HashMap<String, String>) -> Self {
        self.selector_map = selectors;
        self
    }
}

#[async_trait]
impl ContentExtractor for CssExtractorStrategy {
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractedContent> {
        let document = Html::parse_document(html);

        let mut title = "Untitled".to_string();
        let mut content = String::new();
        let mut summary = None;

        // Extract title
        if let Some(title_selectors) = self.selector_map.get("title") {
            for selector_str in title_selectors.split(", ") {
                if let Ok(selector) = Selector::parse(selector_str.trim()) {
                    if let Some(element) = document.select(&selector).next() {
                        if selector_str.contains("meta") {
                            if let Some(attr_content) = element.value().attr("content") {
                                title = attr_content.to_string();
                                break;
                            }
                        } else {
                            let extracted_title = element
                                .text()
                                .collect::<Vec<_>>()
                                .join(" ")
                                .trim()
                                .to_string();
                            if !extracted_title.is_empty() {
                                title = extracted_title;
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Extract content
        if let Some(content_selectors) = self.selector_map.get("content") {
            for selector_str in content_selectors.split(", ") {
                if let Ok(selector) = Selector::parse(selector_str.trim()) {
                    if let Some(element) = document.select(&selector).next() {
                        content = element.text().collect::<Vec<_>>().join(" ");
                        if content.len() > 100 {
                            break;
                        }
                    }
                }
            }
        }

        // Extract summary/description
        if let Some(summary_selectors) = self.selector_map.get("summary") {
            for selector_str in summary_selectors.split(", ") {
                if let Ok(selector) = Selector::parse(selector_str.trim()) {
                    if let Some(element) = document.select(&selector).next() {
                        if selector_str.contains("meta") {
                            if let Some(attr_content) = element.value().attr("content") {
                                summary = Some(attr_content.to_string());
                                break;
                            }
                        } else {
                            let extracted_summary = element
                                .text()
                                .collect::<Vec<_>>()
                                .join(" ")
                                .trim()
                                .to_string();
                            if !extracted_summary.is_empty() {
                                summary = Some(extracted_summary);
                                break;
                            }
                        }
                    }
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
            summary,
            url: url.to_string(),
            strategy_used: "css_extraction".to_string(),
            extraction_confidence: self.confidence_score(html),
        })
    }

    fn confidence_score(&self, html: &str) -> f64 {
        let mut score = 0.7_f64; // Base score for CSS extraction

        // Check for structured content indicators
        if html.contains("article") || html.contains("main") {
            score += 0.15;
        }
        if html.contains("class=\"content\"") || html.contains("class=\"post") {
            score += 0.1;
        }
        if html.contains("meta name=\"description\"") {
            score += 0.05;
        }

        score.min(1.0_f64)
    }

    fn strategy_name(&self) -> &'static str {
        "css_extraction"
    }
}

/// Basic link extraction using regex (simplified for core)
pub fn extract_links_basic(content: &str, base_url: &Url) -> Result<Vec<Url>> {
    let link_regex = regex::Regex::new(r#"href\s*=\s*["']([^"']+)["']"#)?;
    let mut links = Vec::new();

    for cap in link_regex.captures_iter(content) {
        if let Some(link_str) = cap.get(1) {
            if let Ok(url) = base_url.join(link_str.as_str()) {
                links.push(url);
            }
        }
    }

    Ok(links)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fallback_extract() {
        let html = r#"
        <html>
            <head><title>Test Title</title></head>
            <body>
                <article>
                    <h1>Article Title</h1>
                    <p>This is the main content of the article.</p>
                    <p>It contains multiple paragraphs.</p>
                </article>
            </body>
        </html>
        "#;

        let result = fallback_extract(html, "https://example.com").await.unwrap();
        assert_eq!(result.title, "Test Title");
        assert!(result.content.contains("This is the main content"));
        assert_eq!(result.strategy_used, "trek_fallback");
    }

    #[tokio::test]
    async fn test_css_extractor_strategy() {
        let html = r#"
        <html>
            <head>
                <title>CSS Test Title</title>
                <meta name="description" content="This is a test description">
            </head>
            <body>
                <main>
                    <h1>Main Heading</h1>
                    <div class="content">
                        <p>Main content goes here.</p>
                        <p>More content paragraphs.</p>
                    </div>
                </main>
            </body>
        </html>
        "#;

        let extractor = CssExtractorStrategy::new();
        let result = extractor
            .extract(html, "https://example.com")
            .await
            .unwrap();

        assert_eq!(result.title, "CSS Test Title");
        assert!(result.content.contains("Main content goes here"));
        assert_eq!(
            result.summary,
            Some("This is a test description".to_string())
        );
        assert_eq!(result.strategy_used, "css_extraction");
        assert!(result.extraction_confidence > 0.7);
    }

    #[tokio::test]
    async fn test_trek_extractor_fallback() {
        let html = r#"<html><head><title>Test</title></head><body><p>Content</p></body></html>"#;

        // Create extractor without WASM path (should use fallback)
        let extractor = TrekExtractor::new(None).await.unwrap();
        let result = extractor
            .extract(html, "https://example.com")
            .await
            .unwrap();

        assert_eq!(result.title, "Test");
        assert!(result.content.contains("Content"));
        assert_eq!(result.strategy_used, "trek_fallback");
    }

    #[test]
    fn test_extract_links_basic() {
        let html = r#"
        <a href="https://example.com/page1">Link 1</a>
        <a href="/relative/page2">Link 2</a>
        <a href="mailto:test@example.com">Email</a>
        "#;

        let base_url = Url::parse("https://example.com").unwrap();
        let links = extract_links_basic(html, &base_url).unwrap();

        assert_eq!(links.len(), 3);
        assert_eq!(links[0].as_str(), "https://example.com/page1");
        assert_eq!(links[1].as_str(), "https://example.com/relative/page2");
        assert_eq!(links[2].as_str(), "mailto:test@example.com");
    }
}
