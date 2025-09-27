//! CSS selector to JSON extraction strategy
//!
//! This module provides functionality to extract content from HTML using CSS selectors
//! and map the results to structured JSON data. It supports flexible selector configuration
//! and provides default selectors for common content types.

use anyhow::Result;
use scraper::{Html, Selector};
use std::collections::HashMap;
use crate::ExtractedContent;

/// CSS-based content extractor
pub struct CssJsonExtractor {
    selectors: HashMap<String, String>,
}

impl CssJsonExtractor {
    /// Create a new CSS extractor with the given selectors
    pub fn new(selectors: HashMap<String, String>) -> Self {
        Self { selectors }
    }

    /// Extract content using the configured selectors
    pub async fn extract(&self, html: &str, url: &str) -> Result<ExtractedContent> {
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
            .map(|v| v.join("\n\n"))
            .or_else(|| extracted_data.get("body").map(|v| v.join("\n\n")))
            .unwrap_or_else(|| {
                // Fallback: combine all non-title fields
                extracted_data
                    .iter()
                    .filter(|(k, _)| *k != "title")
                    .flat_map(|(_, v)| v.iter())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("\n\n")
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

    /// Calculate confidence score based on selector matches
    pub fn confidence_score(&self, html: &str) -> f64 {
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

    /// Get strategy name
    pub fn strategy_name(&self) -> &'static str {
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

/// Common selectors for different content types
pub fn news_article_selectors() -> HashMap<String, String> {
    let mut selectors = HashMap::new();

    selectors.insert("title".to_string(), "h1, .headline, .article-title, [property='og:title']".to_string());
    selectors.insert("subtitle".to_string(), ".subtitle, .sub-headline, .article-subtitle".to_string());
    selectors.insert("author".to_string(), ".author, .byline, [rel='author'], .writer".to_string());
    selectors.insert("date".to_string(), "time, .date, .publish-date, [property='article:published_time']".to_string());
    selectors.insert("content".to_string(), ".article-body, .content, .post-content, article p".to_string());
    selectors.insert("summary".to_string(), ".summary, .excerpt, .lead, .article-summary".to_string());
    selectors.insert("category".to_string(), ".category, .section, .topic, [property='article:section']".to_string());
    selectors.insert("tags".to_string(), ".tag, .keyword, [property='article:tag']".to_string());

    selectors
}

/// Blog post selectors
pub fn blog_post_selectors() -> HashMap<String, String> {
    let mut selectors = HashMap::new();

    selectors.insert("title".to_string(), "h1, .post-title, .entry-title, [property='og:title']".to_string());
    selectors.insert("author".to_string(), ".author, .post-author, .entry-author".to_string());
    selectors.insert("date".to_string(), ".post-date, .entry-date, time".to_string());
    selectors.insert("content".to_string(), ".post-content, .entry-content, .blog-content".to_string());
    selectors.insert("excerpt".to_string(), ".excerpt, .post-excerpt, .summary".to_string());
    selectors.insert("category".to_string(), ".category, .post-category".to_string());
    selectors.insert("tags".to_string(), ".tag, .post-tag, .label".to_string());

    selectors
}

/// E-commerce product selectors
pub fn product_selectors() -> HashMap<String, String> {
    let mut selectors = HashMap::new();

    selectors.insert("title".to_string(), "h1, .product-title, .product-name".to_string());
    selectors.insert("price".to_string(), ".price, .product-price, .cost, .amount".to_string());
    selectors.insert("description".to_string(), ".description, .product-description, .details".to_string());
    selectors.insert("brand".to_string(), ".brand, .manufacturer, .product-brand".to_string());
    selectors.insert("model".to_string(), ".model, .product-model, .sku".to_string());
    selectors.insert("rating".to_string(), ".rating, .stars, .review-score".to_string());
    selectors.insert("availability".to_string(), ".availability, .stock, .in-stock".to_string());
    selectors.insert("images".to_string(), ".product-image img, .gallery img".to_string());

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

/// Extract using news article selectors
pub async fn extract_news_article(html: &str, url: &str) -> Result<ExtractedContent> {
    extract(html, url, &news_article_selectors()).await
}

/// Extract using blog post selectors
pub async fn extract_blog_post(html: &str, url: &str) -> Result<ExtractedContent> {
    extract(html, url, &blog_post_selectors()).await
}

/// Extract using product selectors
pub async fn extract_product(html: &str, url: &str) -> Result<ExtractedContent> {
    extract(html, url, &product_selectors()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_extraction() {
        let html = r#"
            <html>
                <head>
                    <title>Test Article</title>
                    <meta name="description" content="This is a test article">
                </head>
                <body>
                    <article>
                        <h1>Main Title</h1>
                        <p>This is the main content of the article.</p>
                        <p>Second paragraph with more content.</p>
                    </article>
                </body>
            </html>
        "#;

        let result = extract_default(html, "https://example.com").await.unwrap();

        assert_eq!(result.title, "Test Article");
        assert!(result.content.contains("This is the main content"));
        assert_eq!(result.summary, Some("This is a test article".to_string()));
        assert_eq!(result.strategy_used, "css_json");
        assert!(result.extraction_confidence > 0.0);
    }

    #[tokio::test]
    async fn test_custom_selectors() {
        let html = r#"
            <html>
                <body>
                    <div class="custom-title">Custom Title</div>
                    <div class="custom-content">Custom content here</div>
                </body>
            </html>
        "#;

        let mut selectors = HashMap::new();
        selectors.insert("title".to_string(), ".custom-title".to_string());
        selectors.insert("content".to_string(), ".custom-content".to_string());

        let result = extract(html, "https://example.com", &selectors).await.unwrap();

        assert_eq!(result.title, "Custom Title");
        assert_eq!(result.content, "Custom content here");
    }

    #[tokio::test]
    async fn test_confidence_score() {
        let extractor = CssJsonExtractor::new(default_selectors());

        let html_with_matches = r#"
            <html>
                <head><title>Test</title></head>
                <body><article>Content</article></body>
            </html>
        "#;

        let html_without_matches = r#"
            <html>
                <body><div>Some content</div></body>
            </html>
        "#;

        let score_with = extractor.confidence_score(html_with_matches);
        let score_without = extractor.confidence_score(html_without_matches);

        assert!(score_with > score_without);
    }
}