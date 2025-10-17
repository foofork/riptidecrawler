//! Enhanced content extraction that preserves HTML structure
//!
//! This module provides improved extraction that captures the full content
//! including headings, paragraphs, lists, links, and other structured elements.

use anyhow::Result;
use scraper::{Element, ElementRef, Html, Selector};
use std::collections::HashSet;

/// Extract structured content from HTML preserving formatting
pub struct StructuredExtractor;

impl StructuredExtractor {
    /// Extract content preserving structure (headings, paragraphs, lists, etc.)
    pub fn extract_structured_content(html: &str, base_url: Option<&str>) -> Result<String> {
        let doc = Html::parse_document(html);
        let mut content = Vec::new();

        // Try to find main content area
        let content_selectors = [
            "article",
            "main",
            "[role='main']",
            ".content",
            "#content",
            ".post",
            ".entry",
            ".story",
            ".article-body",
            "body",
        ];

        let mut found_main = false;
        for selector_str in &content_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = doc.select(&selector).next() {
                    content.push(Self::extract_element_content(element, 0, base_url));
                    found_main = true;
                    break;
                }
            }
        }

        // If no main content found, extract from body
        if !found_main {
            if let Ok(selector) = Selector::parse("body") {
                if let Some(element) = doc.select(&selector).next() {
                    content.push(Self::extract_element_content(element, 0, base_url));
                }
            }
        }

        Ok(content.join("\n\n"))
    }

    /// Recursively extract content from an element preserving structure
    fn extract_element_content(
        element: ElementRef,
        _depth: usize,
        base_url: Option<&str>,
    ) -> String {
        let mut content = Vec::new();
        let mut skip_tags = HashSet::new();
        skip_tags.insert("script");
        skip_tags.insert("style");
        skip_tags.insert("noscript");
        skip_tags.insert("nav");
        skip_tags.insert("footer");
        skip_tags.insert("header");
        skip_tags.insert("aside");

        for child in element.children() {
            if let Some(elem) = child.value().as_element() {
                let tag_name = elem.name();

                // Skip unwanted elements
                if skip_tags.contains(tag_name) {
                    continue;
                }

                if let Some(child_elem) = ElementRef::wrap(child) {
                    match tag_name {
                        // Headings
                        "h1" => {
                            let text = Self::get_text_content_with_base(child_elem, base_url);
                            if !text.is_empty() {
                                content.push(format!("# {}", text));
                            }
                        }
                        "h2" => {
                            let text = Self::get_text_content_with_base(child_elem, base_url);
                            if !text.is_empty() {
                                content.push(format!("## {}", text));
                            }
                        }
                        "h3" => {
                            let text = Self::get_text_content_with_base(child_elem, base_url);
                            if !text.is_empty() {
                                content.push(format!("### {}", text));
                            }
                        }
                        "h4" | "h5" | "h6" => {
                            let text = Self::get_text_content_with_base(child_elem, base_url);
                            if !text.is_empty() {
                                content.push(format!("#### {}", text));
                            }
                        }

                        // Paragraphs - preserve inline formatting
                        "p" => {
                            let text = Self::get_text_content_with_base(child_elem, base_url);
                            if !text.is_empty() {
                                content.push(text);
                            }
                        }

                        // Lists
                        "ul" | "ol" => {
                            let list_content = Self::extract_list_items_with_base(
                                child_elem,
                                tag_name == "ol",
                                base_url,
                            );
                            if !list_content.is_empty() {
                                content.push(list_content);
                            }
                        }

                        // Links
                        "a" => {
                            if let Some(href) = elem.attr("href") {
                                let text = Self::get_text_content(child_elem);
                                if !text.is_empty() {
                                    // Resolve relative URLs if base_url provided
                                    let url = if let Some(base) = base_url {
                                        if href.starts_with("http://")
                                            || href.starts_with("https://")
                                        {
                                            href.to_string()
                                        } else if href.starts_with('/') {
                                            format!("{}{}", base.trim_end_matches('/'), href)
                                        } else {
                                            format!("{}/{}", base.trim_end_matches('/'), href)
                                        }
                                    } else {
                                        href.to_string()
                                    };
                                    content.push(format!("[{}]({})", text, url));
                                }
                            }
                        }

                        // Blockquotes
                        "blockquote" => {
                            let text = Self::get_text_content_with_base(child_elem, base_url);
                            if !text.is_empty() {
                                content.push(format!("> {}", text.replace('\n', "\n> ")));
                            }
                        }

                        // Code blocks
                        "pre" => {
                            let text = Self::get_text_content_with_base(child_elem, base_url);
                            if !text.is_empty() {
                                content.push(format!("```\n{}\n```", text));
                            }
                        }

                        // Inline code
                        "code" => {
                            let text = Self::get_text_content_with_base(child_elem, base_url);
                            if !text.is_empty() {
                                content.push(format!("`{}`", text));
                            }
                        }

                        // Images
                        "img" => {
                            if let Some(src) = elem.attr("src") {
                                let alt = elem.attr("alt").unwrap_or("");
                                content.push(format!("![{}]({})", alt, src));
                            }
                        }

                        // Line breaks
                        "br" => {
                            content.push("\n".to_string());
                        }

                        // Horizontal rules
                        "hr" => {
                            content.push("---".to_string());
                        }

                        // Tables
                        "table" => {
                            let table_content = Self::extract_table(child_elem);
                            if !table_content.is_empty() {
                                content.push(table_content);
                            }
                        }

                        // Articles and sections - recurse
                        "article" | "section" | "div" | "main" => {
                            let nested =
                                Self::extract_element_content(child_elem, _depth + 1, base_url);
                            if !nested.trim().is_empty() {
                                content.push(nested);
                            }
                        }

                        // Default: get text content for other elements
                        _ => {
                            let text = Self::get_text_content_with_base(child_elem, base_url);
                            if !text.trim().is_empty() {
                                content.push(text);
                            }
                        }
                    }
                }
            } else if let Some(text) = child.value().as_text() {
                // Direct text nodes
                let text = text.trim();
                if !text.is_empty() {
                    content.push(text.to_string());
                }
            }
        }

        content.join("\n")
    }

    /// Extract list items
    #[allow(dead_code)]
    fn extract_list_items(element: ElementRef, ordered: bool) -> String {
        Self::extract_list_items_with_base(element, ordered, None)
    }

    /// Extract list items with base URL for link resolution
    fn extract_list_items_with_base(
        element: ElementRef,
        ordered: bool,
        base_url: Option<&str>,
    ) -> String {
        let mut items = Vec::new();

        if let Ok(selector) = Selector::parse("li") {
            for (i, li) in element.select(&selector).enumerate() {
                let text = Self::get_text_content_with_base(li, base_url);
                if !text.is_empty() {
                    if ordered {
                        items.push(format!("{}. {}", i + 1, text));
                    } else {
                        items.push(format!("- {}", text));
                    }
                }
            }
        }

        items.join("\n")
    }

    /// Extract table content as markdown
    fn extract_table(element: ElementRef) -> String {
        let mut rows = Vec::new();

        // Extract headers
        if let Ok(thead_selector) = Selector::parse("thead tr") {
            if let Some(header_row) = element.select(&thead_selector).next() {
                if let Ok(th_selector) = Selector::parse("th") {
                    let headers: Vec<String> = header_row
                        .select(&th_selector)
                        .map(|th| Self::get_text_content(th))
                        .collect();

                    if !headers.is_empty() {
                        rows.push(format!("| {} |", headers.join(" | ")));
                        rows.push(format!(
                            "| {} |",
                            headers
                                .iter()
                                .map(|_| "---")
                                .collect::<Vec<_>>()
                                .join(" | ")
                        ));
                    }
                }
            }
        }

        // Extract body rows
        if let Ok(tbody_selector) = Selector::parse("tbody tr, tr") {
            for row in element.select(&tbody_selector) {
                if let Ok(td_selector) = Selector::parse("td") {
                    let cells: Vec<String> = row
                        .select(&td_selector)
                        .map(|td| Self::get_text_content(td))
                        .collect();

                    if !cells.is_empty() {
                        rows.push(format!("| {} |", cells.join(" | ")));
                    }
                }
            }
        }

        rows.join("\n")
    }

    /// Get text content from an element with inline markdown formatting preserved
    fn get_text_content(element: ElementRef) -> String {
        Self::get_text_content_with_base(element, None)
    }

    /// Get text content with base URL for link resolution
    fn get_text_content_with_base(element: ElementRef, base_url: Option<&str>) -> String {
        Self::extract_inline_content_with_base(element, base_url)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }

    /// Extract content with inline markdown formatting (bold, italic, code, links)
    #[allow(dead_code)]
    fn extract_inline_content(element: ElementRef) -> String {
        Self::extract_inline_content_with_base(element, None)
    }

    /// Extract content with inline markdown formatting and base URL resolution
    fn extract_inline_content_with_base(element: ElementRef, base_url: Option<&str>) -> String {
        let mut result = Vec::new();

        for child in element.children() {
            if let Some(elem) = child.value().as_element() {
                let tag_name = elem.name();

                if let Some(child_elem) = ElementRef::wrap(child) {
                    match tag_name {
                        // Bold
                        "strong" | "b" => {
                            let text = Self::extract_inline_content_with_base(child_elem, base_url);
                            if !text.is_empty() {
                                result.push(format!("**{}**", text));
                            }
                        }
                        // Italic
                        "em" | "i" => {
                            let text = Self::extract_inline_content_with_base(child_elem, base_url);
                            if !text.is_empty() {
                                result.push(format!("*{}*", text));
                            }
                        }
                        // Inline code
                        "code" => {
                            let text: String = child_elem.text().collect();
                            if !text.is_empty() {
                                result.push(format!("`{}`", text.trim()));
                            }
                        }
                        // Links - resolve relative URLs
                        "a" => {
                            if let Some(href) = elem.attr("href") {
                                let text =
                                    Self::extract_inline_content_with_base(child_elem, base_url);
                                if !text.is_empty() {
                                    // Resolve relative URLs if base_url provided
                                    let url = if let Some(base) = base_url {
                                        if href.starts_with("http://")
                                            || href.starts_with("https://")
                                        {
                                            href.to_string()
                                        } else if href.starts_with('/') {
                                            format!("{}{}", base.trim_end_matches('/'), href)
                                        } else {
                                            format!("{}/{}", base.trim_end_matches('/'), href)
                                        }
                                    } else {
                                        href.to_string()
                                    };
                                    result.push(format!("[{}]({})", text, url));
                                }
                            } else {
                                result.push(Self::extract_inline_content_with_base(
                                    child_elem, base_url,
                                ));
                            }
                        }
                        // Nested inline elements - recurse
                        "span" | "sub" | "sup" | "mark" | "small" => {
                            result
                                .push(Self::extract_inline_content_with_base(child_elem, base_url));
                        }
                        // Line break
                        "br" => {
                            result.push("\n".to_string());
                        }
                        // Default - continue extraction
                        _ => {
                            result
                                .push(Self::extract_inline_content_with_base(child_elem, base_url));
                        }
                    }
                }
            } else if let Some(text) = child.value().as_text() {
                result.push(text.to_string());
            }
        }

        result.join("")
    }

    /// Extract content for specific sites with custom selectors
    pub fn extract_site_specific(html: &str, url: &str) -> Option<String> {
        // Special handling for known sites
        if url.contains("news.ycombinator.com") {
            return Self::extract_hackernews(html);
        } else if url.contains("github.com") {
            return Self::extract_github(html);
        } else if url.contains("wikipedia.org") {
            return Self::extract_wikipedia(html);
        } else if url.contains("bbc.com") || url.contains("bbc.co.uk") {
            return Self::extract_bbc(html);
        }

        None
    }

    /// Extract Hacker News stories
    fn extract_hackernews(html: &str) -> Option<String> {
        let doc = Html::parse_document(html);
        let mut stories = Vec::new();

        // Extract story items
        if let Ok(selector) = Selector::parse(".athing") {
            for story in doc.select(&selector) {
                // Get title
                if let Ok(title_selector) = Selector::parse(".titleline > a") {
                    if let Some(title_elem) = story.select(&title_selector).next() {
                        let title = Self::get_text_content(title_elem);
                        let href = title_elem.value().attr("href").unwrap_or("");

                        // Get metadata from next sibling
                        if let Some(next) = story.next_sibling_element() {
                            let points = if let Ok(sel) = Selector::parse(".score") {
                                next.select(&sel)
                                    .next()
                                    .map(Self::get_text_content)
                                    .unwrap_or_default()
                            } else {
                                String::new()
                            };

                            let comments = if let Ok(sel) = Selector::parse(".subline a:last-child")
                            {
                                next.select(&sel)
                                    .next()
                                    .map(Self::get_text_content)
                                    .unwrap_or_default()
                            } else {
                                String::new()
                            };

                            stories.push(format!(
                                "- **[{}]({})** - {} | {}",
                                title, href, points, comments
                            ));
                        } else {
                            stories.push(format!("- **[{}]({})**", title, href));
                        }
                    }
                }
            }
        }

        if !stories.is_empty() {
            Some(format!("# Hacker News Stories\n\n{}", stories.join("\n")))
        } else {
            None
        }
    }

    /// Extract GitHub repository content
    fn extract_github(html: &str) -> Option<String> {
        let doc = Html::parse_document(html);
        let mut content = Vec::new();

        // Repository name and description
        if let Ok(selector) = Selector::parse("h1.d-flex") {
            if let Some(elem) = doc.select(&selector).next() {
                content.push(format!("# {}", Self::get_text_content(elem)));
            }
        }

        // README content
        if let Ok(selector) = Selector::parse(".markdown-body") {
            if let Some(elem) = doc.select(&selector).next() {
                content.push(Self::extract_element_content(
                    elem,
                    0,
                    Some("https://github.com"),
                ));
            }
        }

        if !content.is_empty() {
            Some(content.join("\n\n"))
        } else {
            None
        }
    }

    /// Extract Wikipedia article
    fn extract_wikipedia(html: &str) -> Option<String> {
        let doc = Html::parse_document(html);
        let mut content = Vec::new();

        // Article title
        if let Ok(selector) = Selector::parse("#firstHeading") {
            if let Some(elem) = doc.select(&selector).next() {
                content.push(format!("# {}", Self::get_text_content(elem)));
            }
        }

        // Article content
        if let Ok(selector) = Selector::parse("#mw-content-text .mw-parser-output") {
            if let Some(elem) = doc.select(&selector).next() {
                content.push(Self::extract_element_content(
                    elem,
                    0,
                    Some("https://en.wikipedia.org"),
                ));
            }
        }

        if !content.is_empty() {
            Some(content.join("\n\n"))
        } else {
            None
        }
    }

    /// Extract BBC News article
    fn extract_bbc(html: &str) -> Option<String> {
        let doc = Html::parse_document(html);
        let mut content = Vec::new();

        // Article title
        if let Ok(selector) = Selector::parse("h1") {
            if let Some(elem) = doc.select(&selector).next() {
                content.push(format!("# {}", Self::get_text_content(elem)));
            }
        }

        // Article content
        let article_selectors = [
            "[data-component='text-block']",
            ".ssrcss-1q0x1qg-Paragraph",
            ".article__body-content",
            "article",
        ];

        for selector_str in &article_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for elem in doc.select(&selector) {
                    let text = Self::get_text_content(elem);
                    if !text.is_empty() {
                        content.push(text);
                    }
                }
                if content.len() > 1 {
                    break;
                }
            }
        }

        if !content.is_empty() {
            Some(content.join("\n\n"))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structured_extraction() {
        let html = r#"
            <html>
            <body>
                <article>
                    <h1>Main Title</h1>
                    <p>First paragraph with some content.</p>
                    <h2>Section Header</h2>
                    <p>Another paragraph.</p>
                    <ul>
                        <li>Item 1</li>
                        <li>Item 2</li>
                    </ul>
                    <p>Final paragraph with <a href="/link">a link</a>.</p>
                </article>
            </body>
            </html>
        "#;

        let content =
            StructuredExtractor::extract_structured_content(html, Some("https://example.com"))
                .unwrap();

        assert!(content.contains("# Main Title"));
        assert!(content.contains("## Section Header"));
        assert!(content.contains("First paragraph"));
        assert!(content.contains("- Item 1"));
        assert!(content.contains("[a link](https://example.com/link)"));
    }

    #[test]
    fn test_inline_markdown_formatting() {
        let html = r#"
            <html>
            <body>
                <article>
                    <h1>Test Article</h1>
                    <p>This is a paragraph with <strong>bold text</strong> and <em>italic text</em>.</p>
                    <p>Here is some <code>inline code</code> and a <a href="https://example.com">link</a>.</p>
                    <ul>
                        <li>First item with <b>bold</b></li>
                        <li>Second item with <i>italic</i></li>
                    </ul>
                    <pre>code block
multiline</pre>
                </article>
            </body>
            </html>
        "#;

        let result =
            StructuredExtractor::extract_structured_content(html, Some("https://test.com"))
                .expect("Extraction should succeed");

        // Verify markdown formatting is preserved
        assert!(
            result.contains("# Test Article"),
            "Should have H1 markdown header"
        );
        assert!(
            result.contains("**bold text**"),
            "Should have bold markdown"
        );
        assert!(
            result.contains("*italic text*"),
            "Should have italic markdown"
        );
        assert!(
            result.contains("`inline code`"),
            "Should have inline code markdown"
        );
        assert!(
            result.contains("[link](https://example.com)"),
            "Should have markdown link"
        );
        assert!(
            result.contains("**bold**"),
            "Should have bold in list items"
        );
        assert!(
            result.contains("*italic*"),
            "Should have italic in list items"
        );
        assert!(result.contains("```"), "Should have code block markers");
    }

    #[test]
    fn test_nested_inline_formatting() {
        let html = r#"
            <article>
                <p>Text with <strong><em>bold and italic</em></strong> together.</p>
                <p>Link with <a href="/test"><strong>bold link text</strong></a>.</p>
            </article>
        "#;

        let result =
            StructuredExtractor::extract_structured_content(html, Some("https://test.com"))
                .expect("Extraction should succeed");

        // Check for nested formatting
        assert!(
            result.contains("***"),
            "Should handle nested bold+italic with triple asterisks"
        );
        assert!(
            result.contains("[**bold link text**]"),
            "Should have bold within link"
        );
    }
}
