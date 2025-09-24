//! Regex pattern extraction strategy

use anyhow::Result;
use ::regex::Regex;
use std::collections::HashMap;
use crate::strategies::{ExtractedContent, extraction::*, RegexPattern};

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
}

impl ContentExtractor for RegexExtractor {
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractedContent> {
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
                    .or_insert_with(Vec::new)
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
            .map(|v| v.join("

"))
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

    fn confidence_score(&self, html: &str) -> f64 {
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

    fn strategy_name(&self) -> &'static str {
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
            name: "phone".to_string(),
            pattern: r"\b\d{3}-\d{3}-\d{4}\b|\b\(\d{3}\)\s?\d{3}-\d{4}\b".to_string(),
            field: "phones".to_string(),
            required: false,
        },
        RegexPattern {
            name: "url".to_string(),
            pattern: r"https?://[^\s<>]+".to_string(),
            field: "urls".to_string(),
            required: false,
        },
        RegexPattern {
            name: "date".to_string(),
            pattern: "date_pattern_placeholder".to_string(),
            field: "dates".to_string(),
            required: false,
        },
    ]
}

/// Direct extraction function
pub async fn extract(html: &str, url: &str, patterns: &[RegexPattern]) -> Result<ExtractedContent> {
    let extractor = RegexExtractor::new(patterns)?;
    extractor.extract(html, url).await
}