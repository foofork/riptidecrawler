//! Content extraction from HTML documents

use scraper::{Html, Selector};

use crate::native_parser::error::{NativeParserError, Result};

pub struct ContentExtractor;

impl ContentExtractor {
    /// Extract both text and markdown representation
    pub fn extract(document: &Html, _url: &str) -> Result<(String, Option<String>)> {
        // Try article-specific selectors first
        if let Some(content) = Self::extract_article_content(document) {
            let markdown = Self::convert_to_markdown(&content, document);
            return Ok((content, Some(markdown)));
        }

        // Fallback to main content
        if let Some(content) = Self::extract_main_content(document) {
            let markdown = Self::convert_to_markdown(&content, document);
            return Ok((content, Some(markdown)));
        }

        // Last resort: body text
        if let Some(content) = Self::extract_body_content(document) {
            let markdown = Self::convert_to_markdown(&content, document);
            return Ok((content, Some(markdown)));
        }

        Err(NativeParserError::NoContentFound)
    }

    fn extract_article_content(document: &Html) -> Option<String> {
        let selectors = [
            "article",
            "[role='article']",
            ".article-content",
            ".post-content",
            ".entry-content",
            "main article",
        ];

        let mut all_content = Vec::new();
        let mut seen_text = std::collections::HashSet::new();

        for selector_str in &selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                // CRITICAL FIX: Iterate ALL matching elements, not just first
                for element in document.select(&selector) {
                    let text = Self::extract_clean_text(element.html().as_str());
                    // Deduplicate to avoid nested content repetition
                    if !text.is_empty() && !seen_text.contains(&text) {
                        seen_text.insert(text.clone());
                        all_content.push(text);
                    }
                }
            }
        }

        let combined = all_content.join("\n\n");
        if combined.len() > 100 {
            Some(combined)
        } else {
            None
        }
    }

    fn extract_main_content(document: &Html) -> Option<String> {
        let selectors = ["main", "[role='main']", ".main-content", "#content"];

        let mut all_content = Vec::new();
        let mut seen_text = std::collections::HashSet::new();

        for selector_str in &selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                // CRITICAL FIX: Iterate ALL matching elements
                for element in document.select(&selector) {
                    let text = Self::extract_clean_text(element.html().as_str());
                    if !text.is_empty() && !seen_text.contains(&text) {
                        seen_text.insert(text.clone());
                        all_content.push(text);
                    }
                }
            }
        }

        let combined = all_content.join("\n\n");
        if combined.len() > 100 {
            Some(combined)
        } else {
            None
        }
    }

    fn extract_body_content(document: &Html) -> Option<String> {
        let selector = Selector::parse("body").ok()?;
        let body = document.select(&selector).next()?;
        let text = Self::extract_clean_text(body.html().as_str());
        if text.is_empty() {
            None
        } else {
            Some(text)
        }
    }

    fn extract_clean_text(html: &str) -> String {
        let doc = Html::parse_fragment(html);

        // First, remove nav and footer content to prevent quality degradation
        let exclude_selectors = ["nav", "footer", "script", "style", "noscript"];
        let mut excluded_text = std::collections::HashSet::new();

        for selector_str in &exclude_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in doc.select(&selector) {
                    let text: String = element.text().collect();
                    excluded_text.insert(text);
                }
            }
        }

        let mut text = String::new();

        // CRITICAL FIX: Expanded content selectors to capture more content
        let content_selectors = [
            "p", "h1", "h2", "h3", "h4", "h5", "h6",
            "li", "blockquote", "div", "section", "aside",
            "code", "pre", "span"
        ];

        for selector_str in &content_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in doc.select(&selector) {
                    let element_text: String = element.text().collect();
                    let cleaned = element_text.trim();
                    // Skip if empty or from excluded sections
                    if !cleaned.is_empty() && !excluded_text.iter().any(|ex| ex.contains(cleaned)) {
                        text.push_str(cleaned);
                        text.push('\n');
                    }
                }
            }
        }

        text.trim().to_string()
    }

    fn convert_to_markdown(_text: &str, document: &Html) -> String {
        let mut markdown = String::new();

        // Extract title as heading
        let h1_selector = Selector::parse("h1").ok();
        if let Some(selector) = h1_selector {
            if let Some(h1) = document.select(&selector).next() {
                let title: String = h1.text().collect();
                markdown.push_str(&format!("# {}\n\n", title.trim()));
            }
        }

        // Extract paragraphs
        let p_selector = Selector::parse("p").ok();
        if let Some(selector) = p_selector {
            for p in document.select(&selector) {
                let para: String = p.text().collect();
                let cleaned = para.trim();
                if !cleaned.is_empty() {
                    markdown.push_str(cleaned);
                    markdown.push_str("\n\n");
                }
            }
        }

        // Extract headings
        for level in 2..=6 {
            let selector_str = format!("h{}", level);
            let heading_selector = Selector::parse(&selector_str).ok();
            if let Some(selector) = heading_selector {
                for heading in document.select(&selector) {
                    let heading_text: String = heading.text().collect();
                    let cleaned = heading_text.trim();
                    if !cleaned.is_empty() {
                        markdown.push_str(&format!("{} {}\n\n", "#".repeat(level), cleaned));
                    }
                }
            }
        }

        if markdown.is_empty() {
            // Return the original text if no markdown was generated
            String::new()
        } else {
            markdown
        }
    }
}
