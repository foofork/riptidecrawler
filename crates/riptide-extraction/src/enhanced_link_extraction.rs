//! Enhanced link extraction with context, classification, and attributes
//!
//! This module provides production-quality link extraction with:
//! - Link URL and anchor text extraction
//! - Surrounding context (text before/after the link)
//! - Link type classification (internal, external, download, etc.)
//! - HTML attributes extraction (rel, target, data-*, etc.)
//! - Position tracking in document

use anyhow::Result;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

/// Enhanced link information with context and classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedLink {
    /// The absolute or relative URL
    pub url: String,
    /// The anchor text (text inside <a> tag)
    pub anchor_text: String,
    /// Surrounding context (text before and after the link)
    pub surrounding_context: String,
    /// Classified link type
    pub link_type: LinkType,
    /// HTML attributes (rel, target, data-*, etc.)
    pub attributes: HashMap<String, String>,
    /// Position in the document (character offset)
    pub position: usize,
}

/// Link type classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkType {
    /// Internal link (same domain)
    Internal,
    /// External link (different domain)
    External,
    /// Download link (file extensions like .pdf, .zip, etc.)
    Download,
    /// Anchor link (starts with #)
    Anchor,
    /// Email link (mailto:)
    Email,
    /// Phone link (tel:)
    Phone,
    /// Unknown or other type
    Other,
}

/// Configuration for enhanced link extraction
#[derive(Debug, Clone)]
pub struct LinkExtractionConfig {
    /// Number of characters to extract before the link for context
    pub context_chars_before: usize,
    /// Number of characters to extract after the link for context
    pub context_chars_after: usize,
    /// Whether to extract internal links
    pub extract_internal: bool,
    /// Whether to extract external links
    pub extract_external: bool,
    /// Whether to extract download links
    pub extract_downloads: bool,
    /// Whether to extract anchor links
    pub extract_anchors: bool,
    /// Whether to extract email/phone links
    pub extract_special: bool,
    /// File extensions considered as downloads
    pub download_extensions: Vec<String>,
    /// Maximum number of links to extract
    pub max_links: Option<usize>,
}

impl Default for LinkExtractionConfig {
    fn default() -> Self {
        Self {
            context_chars_before: 75,
            context_chars_after: 75,
            extract_internal: true,
            extract_external: true,
            extract_downloads: true,
            extract_anchors: true,
            extract_special: true,
            download_extensions: vec![
                "pdf".to_string(),
                "zip".to_string(),
                "tar".to_string(),
                "gz".to_string(),
                "doc".to_string(),
                "docx".to_string(),
                "xls".to_string(),
                "xlsx".to_string(),
                "ppt".to_string(),
                "pptx".to_string(),
                "exe".to_string(),
                "dmg".to_string(),
                "pkg".to_string(),
                "deb".to_string(),
                "rpm".to_string(),
            ],
            max_links: None,
        }
    }
}

/// Enhanced link extractor with context awareness
pub struct EnhancedLinkExtractor {
    config: LinkExtractionConfig,
}

impl EnhancedLinkExtractor {
    /// Create a new enhanced link extractor with default configuration
    pub fn new() -> Self {
        Self {
            config: LinkExtractionConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: LinkExtractionConfig) -> Self {
        Self { config }
    }

    /// Extract all links from HTML with enhanced context and classification
    pub fn extract_links(&self, html: &str, base_url: Option<&str>) -> Result<Vec<ExtractedLink>> {
        let document = Html::parse_document(html);
        let base_url = base_url.and_then(|u| Url::parse(u).ok());

        let mut links = Vec::new();
        let link_selector = Selector::parse("a[href]")
            .map_err(|e| anyhow::anyhow!("Failed to parse link selector: {:?}", e))?;

        // Get the full text content for context extraction
        let full_text = self.extract_full_text(&document);

        for element in document.select(&link_selector) {
            if let Some(href) = element.value().attr("href") {
                // Skip empty hrefs
                if href.trim().is_empty() {
                    continue;
                }

                // Classify the link type
                let link_type = self.classify_link(href, &base_url);

                // Check if we should extract this link type
                if !self.should_extract_link_type(&link_type) {
                    continue;
                }

                // Extract anchor text
                let anchor_text = self.extract_anchor_text(&element);

                // Resolve URL to absolute
                let url = self.resolve_url(href, &base_url);

                // Extract surrounding context
                let (surrounding_context, position) =
                    self.extract_surrounding_context(&element, &full_text, &document);

                // Extract HTML attributes
                let attributes = self.extract_attributes(&element);

                links.push(ExtractedLink {
                    url,
                    anchor_text,
                    surrounding_context,
                    link_type,
                    attributes,
                    position,
                });

                // Check max links limit
                if let Some(max) = self.config.max_links {
                    if links.len() >= max {
                        break;
                    }
                }
            }
        }

        Ok(links)
    }

    /// Classify a link based on its URL
    fn classify_link(&self, href: &str, base_url: &Option<Url>) -> LinkType {
        let href_lower = href.to_lowercase();

        // Check for special protocols
        if href_lower.starts_with("mailto:") {
            return LinkType::Email;
        }
        if href_lower.starts_with("tel:") {
            return LinkType::Phone;
        }
        if href_lower.starts_with('#') {
            return LinkType::Anchor;
        }

        // Check for download links by extension
        if self.is_download_link(&href_lower) {
            return LinkType::Download;
        }

        // Check if internal or external
        if let Some(base) = base_url {
            match Url::parse(href) {
                Ok(url) => {
                    if url.host_str() == base.host_str() {
                        LinkType::Internal
                    } else {
                        LinkType::External
                    }
                }
                Err(_) => {
                    // Relative URL is internal
                    LinkType::Internal
                }
            }
        } else {
            // No base URL, classify based on absolute/relative
            if href.starts_with("http://") || href.starts_with("https://") {
                LinkType::External
            } else {
                LinkType::Internal
            }
        }
    }

    /// Check if a URL points to a downloadable file
    fn is_download_link(&self, href: &str) -> bool {
        // Extract file extension from URL
        if let Some(path) = href.split('?').next() {
            if let Some(extension) = path.rsplit('.').next() {
                return self
                    .config
                    .download_extensions
                    .iter()
                    .any(|ext| ext.eq_ignore_ascii_case(extension));
            }
        }
        false
    }

    /// Check if we should extract this link type based on configuration
    fn should_extract_link_type(&self, link_type: &LinkType) -> bool {
        match link_type {
            LinkType::Internal => self.config.extract_internal,
            LinkType::External => self.config.extract_external,
            LinkType::Download => self.config.extract_downloads,
            LinkType::Anchor => self.config.extract_anchors,
            LinkType::Email | LinkType::Phone => self.config.extract_special,
            LinkType::Other => true,
        }
    }

    /// Extract anchor text from link element
    fn extract_anchor_text(&self, element: &ElementRef) -> String {
        element
            .text()
            .collect::<Vec<_>>()
            .join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Resolve relative URLs to absolute
    fn resolve_url(&self, href: &str, base_url: &Option<Url>) -> String {
        if let Some(base) = base_url {
            base.join(href)
                .map(|u| u.to_string())
                .unwrap_or_else(|_| href.to_string())
        } else {
            href.to_string()
        }
    }

    /// Extract surrounding context for a link
    fn extract_surrounding_context(
        &self,
        element: &ElementRef,
        full_text: &str,
        _document: &Html,
    ) -> (String, usize) {
        let anchor_text = self.extract_anchor_text(element);

        // Try to find the anchor text in the full text
        if let Some(position) = full_text.find(&anchor_text) {
            let start = position.saturating_sub(self.config.context_chars_before);
            let end = (position + anchor_text.len() + self.config.context_chars_after)
                .min(full_text.len());

            let context = &full_text[start..end];
            let cleaned_context = self.clean_context(context);

            (cleaned_context, position)
        } else {
            // Fallback: try to get parent element text
            if let Some(parent) = element.parent() {
                if let Some(parent_elem) = ElementRef::wrap(parent) {
                    let parent_text = parent_elem
                        .text()
                        .collect::<Vec<_>>()
                        .join(" ")
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ");

                    let cleaned = self.clean_context(&parent_text);
                    return (cleaned, 0);
                }
            }
            (String::new(), 0)
        }
    }

    /// Clean context text by removing extra whitespace and normalizing
    fn clean_context(&self, text: &str) -> String {
        text.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }

    /// Extract full text content from document for context matching
    fn extract_full_text(&self, document: &Html) -> String {
        let body_selector = Selector::parse("body").unwrap();
        if let Some(body) = document.select(&body_selector).next() {
            body.text()
                .collect::<Vec<_>>()
                .join(" ")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            String::new()
        }
    }

    /// Extract HTML attributes from link element
    fn extract_attributes(&self, element: &ElementRef) -> HashMap<String, String> {
        let mut attributes = HashMap::new();

        // Extract common attributes
        let common_attrs = ["rel", "target", "title", "class", "id", "download"];

        for attr in &common_attrs {
            if let Some(value) = element.value().attr(attr) {
                attributes.insert(attr.to_string(), value.to_string());
            }
        }

        // Extract data-* attributes
        for (name, value) in element.value().attrs() {
            if name.starts_with("data-") {
                attributes.insert(name.to_string(), value.to_string());
            }
        }

        // Extract aria-* attributes
        for (name, value) in element.value().attrs() {
            if name.starts_with("aria-") {
                attributes.insert(name.to_string(), value.to_string());
            }
        }

        attributes
    }

    /// Extract links and group them by type
    pub fn extract_links_by_type(
        &self,
        html: &str,
        base_url: Option<&str>,
    ) -> Result<HashMap<LinkType, Vec<ExtractedLink>>> {
        let links = self.extract_links(html, base_url)?;
        let mut grouped = HashMap::new();

        for link in links {
            grouped
                .entry(link.link_type.clone())
                .or_insert_with(Vec::new)
                .push(link);
        }

        Ok(grouped)
    }

    /// Extract only internal links
    pub fn extract_internal_links(
        &self,
        html: &str,
        base_url: Option<&str>,
    ) -> Result<Vec<ExtractedLink>> {
        let links = self.extract_links(html, base_url)?;
        Ok(links
            .into_iter()
            .filter(|link| link.link_type == LinkType::Internal)
            .collect())
    }

    /// Extract only external links
    pub fn extract_external_links(
        &self,
        html: &str,
        base_url: Option<&str>,
    ) -> Result<Vec<ExtractedLink>> {
        let links = self.extract_links(html, base_url)?;
        Ok(links
            .into_iter()
            .filter(|link| link.link_type == LinkType::External)
            .collect())
    }
}

impl Default for EnhancedLinkExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_basic_links() {
        let html = r#"
            <html>
            <body>
                <p>Visit <a href="https://example.com">our website</a> for more information.</p>
            </body>
            </html>
        "#;

        let extractor = EnhancedLinkExtractor::new();
        let links = extractor.extract_links(html, None).unwrap();

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].anchor_text, "our website");
        assert_eq!(links[0].url, "https://example.com");
        assert!(links[0].surrounding_context.contains("Visit"));
        assert!(links[0]
            .surrounding_context
            .contains("for more information"));
    }

    #[test]
    fn test_link_type_classification() {
        let html = r##"
            <html>
            <body>
                <a href="https://other.com/page">External</a>
                <a href="/internal">Internal</a>
                <a href="#section">Anchor</a>
                <a href="mailto:test@example.com">Email</a>
                <a href="tel:+1234567890">Phone</a>
                <a href="/file.pdf">Download</a>
            </body>
            </html>
        "##;

        let extractor = EnhancedLinkExtractor::new();
        let links = extractor
            .extract_links(html, Some("https://example.com"))
            .unwrap();

        assert_eq!(links.len(), 6);

        // Find each link type
        let external = links.iter().find(|l| l.anchor_text == "External").unwrap();
        assert_eq!(external.link_type, LinkType::External);

        let internal = links.iter().find(|l| l.anchor_text == "Internal").unwrap();
        assert_eq!(internal.link_type, LinkType::Internal);

        let anchor = links.iter().find(|l| l.anchor_text == "Anchor").unwrap();
        assert_eq!(anchor.link_type, LinkType::Anchor);

        let email = links.iter().find(|l| l.anchor_text == "Email").unwrap();
        assert_eq!(email.link_type, LinkType::Email);

        let phone = links.iter().find(|l| l.anchor_text == "Phone").unwrap();
        assert_eq!(phone.link_type, LinkType::Phone);

        let download = links.iter().find(|l| l.anchor_text == "Download").unwrap();
        assert_eq!(download.link_type, LinkType::Download);
    }

    #[test]
    fn test_extract_attributes() {
        let html = r#"
            <html>
            <body>
                <a href="https://example.com"
                   rel="nofollow"
                   target="_blank"
                   title="Example Site"
                   data-tracking="abc123"
                   class="external-link">Link</a>
            </body>
            </html>
        "#;

        let extractor = EnhancedLinkExtractor::new();
        let links = extractor.extract_links(html, None).unwrap();

        assert_eq!(links.len(), 1);
        let link = &links[0];

        assert_eq!(link.attributes.get("rel"), Some(&"nofollow".to_string()));
        assert_eq!(link.attributes.get("target"), Some(&"_blank".to_string()));
        assert_eq!(
            link.attributes.get("title"),
            Some(&"Example Site".to_string())
        );
        assert_eq!(
            link.attributes.get("data-tracking"),
            Some(&"abc123".to_string())
        );
        assert_eq!(
            link.attributes.get("class"),
            Some(&"external-link".to_string())
        );
    }

    #[test]
    fn test_surrounding_context_extraction() {
        let html = r#"
            <html>
            <body>
                <p>This is some text before the link. Click <a href="/page">here</a> to continue reading more content after the link.</p>
            </body>
            </html>
        "#;

        let extractor = EnhancedLinkExtractor::new();
        let links = extractor.extract_links(html, None).unwrap();

        assert_eq!(links.len(), 1);
        let context = &links[0].surrounding_context;

        assert!(context.contains("before the link"));
        assert!(context.contains("here"));
        assert!(context.contains("after the link"));
    }

    #[test]
    fn test_download_link_detection() {
        let html = r#"
            <html>
            <body>
                <a href="/document.pdf">PDF</a>
                <a href="/archive.zip">ZIP</a>
                <a href="/document.PDF">PDF Uppercase</a>
                <a href="/page.html">HTML</a>
            </body>
            </html>
        "#;

        let extractor = EnhancedLinkExtractor::new();
        let links = extractor.extract_links(html, None).unwrap();

        let pdf_links: Vec<_> = links
            .iter()
            .filter(|l| l.link_type == LinkType::Download)
            .collect();

        assert_eq!(pdf_links.len(), 3); // Both PDF and ZIP
    }

    #[test]
    fn test_internal_external_classification() {
        let html = r#"
            <html>
            <body>
                <a href="https://example.com/page1">Same domain</a>
                <a href="https://other.com/page">Different domain</a>
                <a href="/relative">Relative</a>
                <a href="page.html">Relative no slash</a>
            </body>
            </html>
        "#;

        let extractor = EnhancedLinkExtractor::new();
        let links = extractor
            .extract_links(html, Some("https://example.com"))
            .unwrap();

        let internal: Vec<_> = links
            .iter()
            .filter(|l| l.link_type == LinkType::Internal)
            .collect();
        let external: Vec<_> = links
            .iter()
            .filter(|l| l.link_type == LinkType::External)
            .collect();

        assert_eq!(internal.len(), 3); // Same domain + two relative
        assert_eq!(external.len(), 1); // Different domain
    }

    #[test]
    fn test_max_links_config() {
        let html = r#"
            <html>
            <body>
                <a href="/link1">Link 1</a>
                <a href="/link2">Link 2</a>
                <a href="/link3">Link 3</a>
                <a href="/link4">Link 4</a>
                <a href="/link5">Link 5</a>
            </body>
            </html>
        "#;

        let config = LinkExtractionConfig {
            max_links: Some(3),
            ..Default::default()
        };

        let extractor = EnhancedLinkExtractor::with_config(config);
        let links = extractor.extract_links(html, None).unwrap();

        assert_eq!(links.len(), 3);
    }

    #[test]
    fn test_extract_links_by_type() {
        let html = r#"
            <html>
            <body>
                <a href="https://other.com/page">External</a>
                <a href="/internal">Internal 1</a>
                <a href="/other">Internal 2</a>
                <a href="mailto:test@example.com">Email</a>
            </body>
            </html>
        "#;

        let extractor = EnhancedLinkExtractor::new();
        let grouped = extractor
            .extract_links_by_type(html, Some("https://example.com"))
            .unwrap();

        assert_eq!(grouped.get(&LinkType::External).unwrap().len(), 1);
        assert_eq!(grouped.get(&LinkType::Internal).unwrap().len(), 2);
        assert_eq!(grouped.get(&LinkType::Email).unwrap().len(), 1);
    }

    #[test]
    fn test_context_length_config() {
        let html = r#"
            <html>
            <body>
                <p>A B C D E F G H I J K L M N O P Q R S T U V W X Y Z <a href="/link">LINK</a> 1 2 3 4 5 6 7 8 9 0</p>
            </body>
            </html>
        "#;

        let config = LinkExtractionConfig {
            context_chars_before: 10,
            context_chars_after: 10,
            ..Default::default()
        };

        let extractor = EnhancedLinkExtractor::with_config(config);
        let links = extractor.extract_links(html, None).unwrap();

        assert_eq!(links.len(), 1);
        // Context should be limited
        assert!(links[0].surrounding_context.len() < 100);
    }
}
