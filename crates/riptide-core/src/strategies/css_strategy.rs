//! CSS Selector-based extraction strategy
//!
//! This module provides a CSS selector-based strategy for extracting structured
//! content from HTML documents using scraper's CSS selector engine.

use anyhow::Result;
use async_trait::async_trait;
use scraper::{Html, Selector};
use std::collections::HashMap;

use crate::strategies::{traits::*, ExtractedContent, PerformanceMetrics};

/// CSS Selector strategy for targeted content extraction
#[derive(Debug, Clone)]
pub struct CssSelectorStrategy {
    selectors: HashMap<String, String>,
}

impl CssSelectorStrategy {
    /// Create a new CSS selector strategy with default selectors
    pub fn new() -> Self {
        let mut selectors = HashMap::new();

        // Common article selectors
        selectors.insert(
            "article".to_string(),
            "article, .article, .post-content, .entry-content, main".to_string(),
        );

        // Title selectors
        selectors.insert(
            "title".to_string(),
            "h1, .title, .headline, .post-title, .entry-title".to_string(),
        );

        // Author selectors
        selectors.insert(
            "author".to_string(),
            ".author, .byline, [rel=author], .writer, .post-author".to_string(),
        );

        // Date selectors
        selectors.insert(
            "date".to_string(),
            "time, .date, .published, .post-date, .entry-date".to_string(),
        );

        // Description/Summary selectors
        selectors.insert(
            "description".to_string(),
            ".description, .summary, .excerpt, .lead".to_string(),
        );

        Self { selectors }
    }

    /// Create strategy with custom selectors
    pub fn with_selectors(selectors: HashMap<String, String>) -> Self {
        Self { selectors }
    }

    /// Extract content using a specific selector
    fn extract_by_selector(&self, doc: &Html, content_type: &str) -> Option<String> {
        let selector_str = self.selectors.get(content_type)?;

        // Try each selector in the comma-separated list
        for sel_str in selector_str.split(',') {
            let sel_str = sel_str.trim();
            if let Ok(selector) = Selector::parse(sel_str) {
                if let Some(element) = doc.select(&selector).next() {
                    let text = element.text().collect::<Vec<_>>().join(" ");
                    let text = text.trim().to_string();
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
            }
        }

        None
    }

    /// Extract all matching elements for a selector
    fn extract_all_by_selector(&self, doc: &Html, content_type: &str) -> Vec<String> {
        let selector_str = match self.selectors.get(content_type) {
            Some(s) => s,
            None => return Vec::new(),
        };

        let mut results = Vec::new();

        for sel_str in selector_str.split(',') {
            let sel_str = sel_str.trim();
            if let Ok(selector) = Selector::parse(sel_str) {
                for element in doc.select(&selector) {
                    let text = element.text().collect::<Vec<_>>().join(" ");
                    let text = text.trim().to_string();
                    if !text.is_empty() {
                        results.push(text);
                    }
                }
            }
        }

        results
    }

    /// Calculate extraction confidence based on found elements
    fn calculate_confidence(&self, doc: &Html) -> f64 {
        let mut score = 0.0;
        let mut weight_sum = 0.0;

        // Check important selectors with weights
        let checks = vec![
            ("title", 0.3),
            ("article", 0.4),
            ("author", 0.1),
            ("date", 0.1),
            ("description", 0.1),
        ];

        for (content_type, weight) in checks {
            weight_sum += weight;
            if self.extract_by_selector(doc, content_type).is_some() {
                score += weight;
            }
        }

        if weight_sum > 0.0 {
            score / weight_sum
        } else {
            0.0
        }
    }
}

impl Default for CssSelectorStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ExtractionStrategy for CssSelectorStrategy {
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractionResult> {
        let start = std::time::Instant::now();

        // Parse HTML
        let doc = Html::parse_document(html);

        // Extract content using selectors
        let title = self
            .extract_by_selector(&doc, "title")
            .unwrap_or_else(|| "Untitled".to_string());

        let content = self
            .extract_by_selector(&doc, "article")
            .unwrap_or_else(|| {
                // Fallback: extract all text from body
                if let Ok(body_selector) = Selector::parse("body") {
                    doc.select(&body_selector)
                        .next()
                        .map(|el| el.text().collect::<Vec<_>>().join(" "))
                        .unwrap_or_default()
                } else {
                    String::new()
                }
            });

        let author = self.extract_by_selector(&doc, "author");
        let date = self.extract_by_selector(&doc, "date");
        let description = self.extract_by_selector(&doc, "description");

        // Generate summary
        let summary = description.or_else(|| {
            let words: Vec<&str> = content.split_whitespace().collect();
            if words.len() > 50 {
                Some(words[..50].join(" ") + "...")
            } else if !words.is_empty() {
                Some(words.join(" "))
            } else {
                None
            }
        });

        let confidence = self.calculate_confidence(&doc);

        let extracted = ExtractedContent {
            title,
            content: content.clone(),
            summary,
            url: url.to_string(),
            strategy_used: "css".to_string(),
            extraction_confidence: confidence,
        };

        let duration = start.elapsed();

        // Calculate quality metrics
        let quality = ExtractionQuality {
            content_length: extracted.content.len(),
            title_quality: if extracted.title.is_empty() { 0.0 } else { 0.9 },
            content_quality: calculate_content_quality(&extracted.content),
            structure_score: 0.85, // CSS provides good structure
            metadata_completeness: {
                let mut score = 0.0;
                if author.is_some() {
                    score += 0.33;
                }
                if date.is_some() {
                    score += 0.33;
                }
                if extracted.summary.is_some() {
                    score += 0.34;
                }
                score
            },
        };

        // Build metadata
        let mut metadata = HashMap::new();
        metadata.insert(
            "extraction_time_ms".to_string(),
            duration.as_millis().to_string(),
        );
        metadata.insert(
            "selector_count".to_string(),
            self.selectors.len().to_string(),
        );
        if let Some(author) = author {
            metadata.insert("author".to_string(), author);
        }
        if let Some(date) = date {
            metadata.insert("date".to_string(), date);
        }

        Ok(ExtractionResult {
            content: extracted,
            quality,
            performance: Some(PerformanceMetrics::new()),
            metadata,
        })
    }

    fn name(&self) -> &str {
        "css_selector"
    }

    fn capabilities(&self) -> StrategyCapabilities {
        StrategyCapabilities {
            strategy_type: "css_extraction".to_string(),
            supported_content_types: vec![
                "text/html".to_string(),
                "application/xhtml+xml".to_string(),
            ],
            performance_tier: PerformanceTier::Balanced,
            resource_requirements: ResourceRequirements {
                memory_tier: ResourceTier::Medium,
                cpu_tier: ResourceTier::Medium,
                requires_network: false,
                external_dependencies: vec!["scraper".to_string()],
            },
            features: vec![
                "css_selectors".to_string(),
                "dom_parsing".to_string(),
                "structured_extraction".to_string(),
                "customizable_selectors".to_string(),
            ],
        }
    }

    fn confidence_score(&self, html: &str) -> f64 {
        let doc = Html::parse_document(html);
        self.calculate_confidence(&doc)
    }
}

/// Calculate content quality score
fn calculate_content_quality(content: &str) -> f64 {
    if content.is_empty() {
        return 0.0;
    }

    let mut score = 0.5; // Base score

    // Length bonus (optimal around 2000 chars)
    let ideal_length = 2000.0;
    let length_ratio = (content.len() as f64 / ideal_length).min(1.0);
    score += length_ratio * 0.2;

    // Word count
    let words = content.split_whitespace().count();
    if words > 50 {
        score += 0.1;
    }
    if words > 200 {
        score += 0.1;
    }

    // Sentence structure
    let sentences =
        content.matches('.').count() + content.matches('!').count() + content.matches('?').count();
    if sentences > 5 {
        score += 0.1;
    }

    score.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_css_selector_basic() {
        let strategy = CssSelectorStrategy::new();
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head><title>Test Article</title></head>
            <body>
                <h1>Main Title</h1>
                <article>
                    <p>This is the main content of the article.</p>
                    <p>It has multiple paragraphs.</p>
                </article>
            </body>
            </html>
        "#;

        let result = strategy.extract(html, "https://example.com").await.unwrap();

        assert_eq!(result.content.strategy_used, "css");
        assert!(result.content.title.contains("Main Title"));
        assert!(result.content.content.contains("main content"));
        assert!(result.content.extraction_confidence > 0.5);
    }

    #[tokio::test]
    async fn test_css_selector_with_metadata() {
        let strategy = CssSelectorStrategy::new();
        let html = r#"
            <article>
                <h1>Article Title</h1>
                <div class="author">John Doe</div>
                <time>2025-01-01</time>
                <p>Content here.</p>
            </article>
        "#;

        let result = strategy.extract(html, "https://example.com").await.unwrap();

        assert!(result.metadata.contains_key("author"));
        assert!(result.metadata.contains_key("date"));
    }

    #[test]
    fn test_confidence_score() {
        let strategy = CssSelectorStrategy::new();

        let good_html = r#"
            <article><h1>Title</h1><p>Content</p></article>
        "#;
        let score = strategy.confidence_score(good_html);
        assert!(score > 0.5);

        let poor_html = "<div>Just some text</div>";
        let score = strategy.confidence_score(poor_html);
        assert!(score < 0.5);
    }
}
