//! Regex pattern extraction strategy
//!
//! This module provides functionality to extract content from HTML using regular expressions.
//! It supports configurable patterns with different fields and requirements, making it suitable
//! for extracting structured data like emails, phone numbers, dates, and custom patterns.

use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use crate::{ExtractedContent, RegexPattern};

/// Regex-based content extractor
pub struct RegexExtractor {
    patterns: Vec<CompiledPattern>,
}

#[derive(Debug)]
struct CompiledPattern {
    #[allow(dead_code)]
    name: String,
    regex: Regex,
    field: String,
    required: bool,
}

impl RegexExtractor {
    /// Create a new regex extractor with the given patterns
    pub fn new(patterns: &[RegexPattern]) -> Result<Self> {
        let mut compiled_patterns = Vec::new();

        for pattern in patterns {
            let regex = Regex::new(&pattern.pattern)?;
            compiled_patterns.push(CompiledPattern {
                name: pattern.name.clone(),
                regex,
                field: pattern.field.clone(),
                required: pattern.required,
            });
        }

        Ok(Self {
            patterns: compiled_patterns,
        })
    }

    /// Extract content using the configured patterns
    pub async fn extract(&self, html: &str, url: &str) -> Result<ExtractedContent> {
        let mut extracted_data: HashMap<String, Vec<String>> = HashMap::new();
        let text = strip_html_tags(html);

        // Apply all regex patterns
        for pattern in &self.patterns {
            let matches: Vec<String> = pattern.regex
                .find_iter(&text)
                .map(|m| m.as_str().to_string())
                .collect();

            if !matches.is_empty() {
                extracted_data
                    .entry(pattern.field.clone())
                    .or_default()
                    .extend(matches);
            }
        }

        // Build content structure
        let title = extracted_data
            .get("title")
            .and_then(|v| v.first())
            .cloned()
            .unwrap_or_else(|| extract_title_fallback(html));

        let content = extracted_data
            .get("content")
            .map(|v| v.join("\n\n"))
            .unwrap_or_else(|| {
                // Fallback: use cleaned text
                text.split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .chars()
                    .take(10000) // Limit content length
                    .collect()
            });

        let summary = extracted_data
            .get("description")
            .or_else(|| extracted_data.get("summary"))
            .and_then(|v| v.first())
            .cloned();

        Ok(ExtractedContent {
            title,
            content,
            summary,
            url: url.to_string(),
            strategy_used: "regex".to_string(),
            extraction_confidence: self.confidence_score(html),
        })
    }

    /// Calculate confidence score based on pattern matches
    pub fn confidence_score(&self, html: &str) -> f64 {
        let text = strip_html_tags(html);
        let mut matched_required = 0;
        let mut total_required = 0;
        let mut matched_optional = 0;
        let mut total_optional = 0;

        for pattern in &self.patterns {
            if pattern.required {
                total_required += 1;
                if pattern.regex.is_match(&text) {
                    matched_required += 1;
                }
            } else {
                total_optional += 1;
                if pattern.regex.is_match(&text) {
                    matched_optional += 1;
                }
            }
        }

        let required_score = if total_required > 0 {
            matched_required as f64 / total_required as f64
        } else {
            1.0
        };

        let optional_score = if total_optional > 0 {
            matched_optional as f64 / total_optional as f64
        } else {
            0.5
        };

        (required_score * 0.8 + optional_score * 0.2).min(0.95_f64)
    }

    /// Get strategy name
    pub fn strategy_name(&self) -> &'static str {
        "regex"
    }
}

/// Strip HTML tags and return clean text
fn strip_html_tags(html: &str) -> String {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);

    // Remove script and style content
    let script_selector = Selector::parse("script, style").unwrap();
    let mut clean_html = html.to_string();

    for element in document.select(&script_selector) {
        if let Some(inner_html) = element.html().get(..element.html().len()) {
            clean_html = clean_html.replace(inner_html, "");
        }
    }

    // Parse cleaned HTML and extract text
    let clean_document = Html::parse_document(&clean_html);
    clean_document
        .root_element()
        .text()
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Extract title using fallback methods
fn extract_title_fallback(html: &str) -> String {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);

    // Try title tag first
    if let Ok(selector) = Selector::parse("title") {
        if let Some(element) = document.select(&selector).next() {
            let title = element.text().collect::<String>().trim().to_string();
            if !title.is_empty() {
                return title;
            }
        }
    }

    // Try h1
    if let Ok(selector) = Selector::parse("h1") {
        if let Some(element) = document.select(&selector).next() {
            let title = element.text().collect::<String>().trim().to_string();
            if !title.is_empty() {
                return title;
            }
        }
    }

    "Untitled".to_string()
}

/// Default regex patterns for common content extraction
pub fn default_patterns() -> Vec<RegexPattern> {
    vec![
        RegexPattern {
            name: "email".to_string(),
            pattern: r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b".to_string(),
            field: "emails".to_string(),
            required: false,
        },
        RegexPattern {
            name: "phone_us".to_string(),
            pattern: r"\b\d{3}-\d{3}-\d{4}\b|\b\(\d{3}\)\s?\d{3}-\d{4}\b".to_string(),
            field: "phones".to_string(),
            required: false,
        },
        RegexPattern {
            name: "url".to_string(),
            pattern: r"https?://[^\s<>']+".to_string(),
            field: "urls".to_string(),
            required: false,
        },
        RegexPattern {
            name: "date_iso".to_string(),
            pattern: r"\b\d{4}-\d{2}-\d{2}\b".to_string(),
            field: "dates".to_string(),
            required: false,
        },
        RegexPattern {
            name: "date_us".to_string(),
            pattern: r"\b\d{1,2}/\d{1,2}/\d{4}\b".to_string(),
            field: "dates".to_string(),
            required: false,
        },
        RegexPattern {
            name: "price".to_string(),
            pattern: r"\$\d+(?:,\d{3})*(?:\.\d{2})?".to_string(),
            field: "prices".to_string(),
            required: false,
        },
        RegexPattern {
            name: "credit_card".to_string(),
            pattern: r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b".to_string(),
            field: "credit_cards".to_string(),
            required: false,
        },
        RegexPattern {
            name: "social_security".to_string(),
            pattern: r"\b\d{3}-\d{2}-\d{4}\b".to_string(),
            field: "ssns".to_string(),
            required: false,
        },
    ]
}

/// News article patterns
pub fn news_patterns() -> Vec<RegexPattern> {
    vec![
        RegexPattern {
            name: "byline".to_string(),
            pattern: r"(?i)By\s+([A-Z][a-z]+(?:\s+[A-Z][a-z]+)*)".to_string(),
            field: "authors".to_string(),
            required: false,
        },
        RegexPattern {
            name: "dateline".to_string(),
            pattern: r"([A-Z][A-Z\s]+)\s*[-–—]\s*".to_string(),
            field: "locations".to_string(),
            required: false,
        },
        RegexPattern {
            name: "quote".to_string(),
            pattern: r#""([^"]+)""#.to_string(),
            field: "quotes".to_string(),
            required: false,
        },
    ]
}

/// Contact information patterns
pub fn contact_patterns() -> Vec<RegexPattern> {
    vec![
        RegexPattern {
            name: "email".to_string(),
            pattern: r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b".to_string(),
            field: "emails".to_string(),
            required: false,
        },
        RegexPattern {
            name: "phone_international".to_string(),
            pattern: r"\+\d{1,3}[\s.-]?\d{1,4}[\s.-]?\d{1,4}[\s.-]?\d{1,4}".to_string(),
            field: "phones".to_string(),
            required: false,
        },
        RegexPattern {
            name: "address".to_string(),
            pattern: r"\d+\s+[A-Za-z\s]+(?:Street|St|Avenue|Ave|Road|Rd|Boulevard|Blvd|Lane|Ln|Drive|Dr)".to_string(),
            field: "addresses".to_string(),
            required: false,
        },
        RegexPattern {
            name: "zip_code".to_string(),
            pattern: r"\b\d{5}(?:-\d{4})?\b".to_string(),
            field: "zip_codes".to_string(),
            required: false,
        },
    ]
}

/// Financial patterns
pub fn financial_patterns() -> Vec<RegexPattern> {
    vec![
        RegexPattern {
            name: "currency_usd".to_string(),
            pattern: r"\$\d+(?:,\d{3})*(?:\.\d{2})?".to_string(),
            field: "prices_usd".to_string(),
            required: false,
        },
        RegexPattern {
            name: "currency_eur".to_string(),
            pattern: r"€\d+(?:,\d{3})*(?:\.\d{2})?".to_string(),
            field: "prices_eur".to_string(),
            required: false,
        },
        RegexPattern {
            name: "percentage".to_string(),
            pattern: r"\d+(?:\.\d+)?%".to_string(),
            field: "percentages".to_string(),
            required: false,
        },
        RegexPattern {
            name: "stock_symbol".to_string(),
            pattern: r"\b[A-Z]{1,5}\b".to_string(),
            field: "stock_symbols".to_string(),
            required: false,
        },
    ]
}

/// Social media patterns
pub fn social_media_patterns() -> Vec<RegexPattern> {
    vec![
        RegexPattern {
            name: "twitter_handle".to_string(),
            pattern: r"@[A-Za-z0-9_]+".to_string(),
            field: "twitter_handles".to_string(),
            required: false,
        },
        RegexPattern {
            name: "hashtag".to_string(),
            pattern: r"#[A-Za-z0-9_]+".to_string(),
            field: "hashtags".to_string(),
            required: false,
        },
        RegexPattern {
            name: "instagram_url".to_string(),
            pattern: r"https?://(?:www\.)?instagram\.com/[A-Za-z0-9_.]+".to_string(),
            field: "instagram_urls".to_string(),
            required: false,
        },
        RegexPattern {
            name: "facebook_url".to_string(),
            pattern: r"https?://(?:www\.)?facebook\.com/[A-Za-z0-9.]+".to_string(),
            field: "facebook_urls".to_string(),
            required: false,
        },
    ]
}

/// Direct extraction function
pub async fn extract(html: &str, url: &str, patterns: &[RegexPattern]) -> Result<ExtractedContent> {
    let extractor = RegexExtractor::new(patterns)?;
    extractor.extract(html, url).await
}

/// Extract using default patterns
pub async fn extract_default(html: &str, url: &str) -> Result<ExtractedContent> {
    extract(html, url, &default_patterns()).await
}

/// Extract using news patterns
pub async fn extract_news(html: &str, url: &str) -> Result<ExtractedContent> {
    extract(html, url, &news_patterns()).await
}

/// Extract using contact patterns
pub async fn extract_contacts(html: &str, url: &str) -> Result<ExtractedContent> {
    extract(html, url, &contact_patterns()).await
}

/// Extract using financial patterns
pub async fn extract_financial(html: &str, url: &str) -> Result<ExtractedContent> {
    extract(html, url, &financial_patterns()).await
}

/// Extract using social media patterns
pub async fn extract_social_media(html: &str, url: &str) -> Result<ExtractedContent> {
    extract(html, url, &social_media_patterns()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_email_extraction() {
        let html = r#"
            <html>
                <body>
                    <p>Contact us at john.doe@example.com or support@company.org</p>
                </body>
            </html>
        "#;

        let patterns = vec![RegexPattern {
            name: "email".to_string(),
            pattern: r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b".to_string(),
            field: "emails".to_string(),
            required: false,
        }];

        let result = extract(html, "https://example.com", &patterns).await.unwrap();

        assert!(result.content.contains("john.doe@example.com"));
        assert!(result.content.contains("support@company.org"));
    }

    #[tokio::test]
    async fn test_phone_extraction() {
        let html = r#"
            <html>
                <body>
                    <p>Call us at (555) 123-4567 or 555-987-6543</p>
                </body>
            </html>
        "#;

        let patterns = vec![RegexPattern {
            name: "phone".to_string(),
            pattern: r"\b\d{3}-\d{3}-\d{4}\b|\b\(\d{3}\)\s?\d{3}-\d{4}\b".to_string(),
            field: "phones".to_string(),
            required: false,
        }];

        let result = extract(html, "https://example.com", &patterns).await.unwrap();

        assert!(result.content.contains("(555) 123-4567"));
        assert!(result.content.contains("555-987-6543"));
    }

    #[tokio::test]
    async fn test_confidence_score() {
        let patterns = default_patterns();
        let extractor = RegexExtractor::new(&patterns).unwrap();

        let html_with_matches = r#"
            <html>
                <body>
                    <p>Contact: john@example.com, Phone: 555-123-4567</p>
                    <p>Website: https://example.com</p>
                </body>
            </html>
        "#;

        let html_without_matches = r#"
            <html>
                <body><p>Some content without patterns</p></body>
            </html>
        "#;

        let score_with = extractor.confidence_score(html_with_matches);
        let score_without = extractor.confidence_score(html_without_matches);

        assert!(score_with > score_without);
    }

    #[tokio::test]
    async fn test_strip_html_tags() {
        let html = r#"
            <html>
                <head><script>alert('test');</script></head>
                <body>
                    <p>This is <strong>bold</strong> text.</p>
                    <style>.test { color: red; }</style>
                </body>
            </html>
        "#;

        let text = strip_html_tags(html);

        assert!(!text.contains("<script>"));
        assert!(!text.contains("<style>"));
        assert!(text.contains("This is bold text."));
    }
}