//! Fallback strategies for content extraction

use riptide_types::ExtractedDoc;
use scraper::{Html, Selector};

use super::error::Result;

pub struct FallbackStrategy;

impl FallbackStrategy {
    /// Fallback 1: Extract all visible text
    pub fn full_content_extraction(html: &str, url: &str) -> Result<ExtractedDoc> {
        let document = Html::parse_document(html);

        // Remove script and style tags
        let text = Self::extract_all_text(&document);

        Ok(ExtractedDoc {
            url: url.to_string(),
            title: Some("Fallback Extraction".to_string()),
            text,
            quality_score: Some(40),
            html: None,
            ..Default::default()
        })
    }

    /// Fallback 2: Simple text extraction
    pub fn simple_text_extraction(html: &str, url: &str) -> Result<ExtractedDoc> {
        // Strip HTML tags with regex
        let text = Self::strip_html_tags(html);

        Ok(ExtractedDoc {
            url: url.to_string(),
            title: Some("Simple Extraction".to_string()),
            text,
            quality_score: Some(20),
            html: None,
            ..Default::default()
        })
    }

    fn extract_all_text(document: &Html) -> String {
        let selector = Selector::parse("body").ok();
        if let Some(sel) = selector {
            if let Some(body) = document.select(&sel).next() {
                let text: String = body.text().collect();
                return text.trim().to_string();
            }
        }
        String::new()
    }

    fn strip_html_tags(html: &str) -> String {
        use regex::Regex;
        let tag_regex = Regex::new(r"<[^>]*>").unwrap();
        let script_regex = Regex::new(r"(?s)<script[^>]*>.*?</script>").unwrap();
        let style_regex = Regex::new(r"(?s)<style[^>]*>.*?</style>").unwrap();

        // Remove scripts and styles first
        let without_scripts = script_regex.replace_all(html, " ");
        let without_styles = style_regex.replace_all(&without_scripts, " ");

        // Remove all HTML tags
        let text = tag_regex.replace_all(&without_styles, " ");

        // Clean up whitespace
        let whitespace_regex = Regex::new(r"\s+").unwrap();
        whitespace_regex.replace_all(&text, " ").trim().to_string()
    }
}
