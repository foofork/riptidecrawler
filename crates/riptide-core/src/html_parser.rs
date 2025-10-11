//! Enhanced HTML extraction with metadata, links, and media support
//!
//! This module provides production-quality HTML extraction capabilities including:
//! - Metadata extraction (Open Graph, meta tags)
//! - Main content detection with article heuristics
//! - Link extraction with URL resolution
//! - Media extraction (images, videos) with alt text
//! - Quality score calculation

use anyhow::{Context, Result};
use scraper::{Html, Selector};
use url::Url;

/// Extracted metadata from HTML document
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub og_title: Option<String>,
    pub og_description: Option<String>,
    pub og_image: Option<String>,
    pub author: Option<String>,
    pub published_date: Option<String>,
    pub keywords: Vec<String>,
    pub lang: Option<String>,
}

/// Extracted link information
#[derive(Debug, Clone)]
pub struct Link {
    pub url: String,
    pub text: String,
    pub title: Option<String>,
    pub rel: Option<String>,
}

/// Extracted media information
#[derive(Debug, Clone)]
pub struct Media {
    pub url: String,
    pub media_type: MediaType,
    pub alt_text: Option<String>,
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MediaType {
    Image,
    Video,
    Audio,
}

/// Complete extracted content with all metadata
#[derive(Debug, Clone)]
pub struct ExtractedHtmlContent {
    pub metadata: Metadata,
    pub main_content: String,
    pub markdown_content: String,
    pub links: Vec<Link>,
    pub media: Vec<Media>,
    pub quality_score: f64,
    pub is_article: bool,
}

/// Enhanced HTML extractor with production-quality parsing
pub struct EnhancedHtmlExtractor {
    base_url: Option<Url>,
}

impl EnhancedHtmlExtractor {
    /// Create a new extractor with an optional base URL for link resolution
    pub fn new(base_url: Option<&str>) -> Result<Self> {
        let base_url = match base_url {
            Some(url) => Some(Url::parse(url).context("Invalid base URL")?),
            None => None,
        };
        Ok(Self { base_url })
    }

    /// Extract complete content from HTML
    pub fn extract(&self, html: &str, url: &str) -> Result<ExtractedHtmlContent> {
        let document = Html::parse_document(html);

        // Update base URL if provided
        let base_url = Url::parse(url).ok().or(self.base_url.clone());

        // Extract all components
        let metadata = self.extract_metadata(&document)?;
        let is_article = self.detect_article(&document);
        let main_content = self.find_main_content(&document, is_article)?;
        let markdown_content = self.html_to_markdown(&main_content);
        let links = self.extract_links(&document, &base_url)?;
        let media = self.extract_media(&document, &base_url)?;
        let quality_score =
            self.calculate_quality_score(&metadata, &main_content, &links, &media, is_article);

        Ok(ExtractedHtmlContent {
            metadata,
            main_content,
            markdown_content,
            links,
            media,
            quality_score,
            is_article,
        })
    }

    /// Extract metadata from HTML document
    fn extract_metadata(&self, doc: &Html) -> Result<Metadata> {
        let mut metadata = Metadata::default();

        // Extract title
        if let Ok(selector) = Selector::parse("title") {
            if let Some(element) = doc.select(&selector).next() {
                metadata.title = Some(element.text().collect::<String>().trim().to_string());
            }
        }

        // Extract meta tags
        if let Ok(selector) = Selector::parse("meta") {
            for element in doc.select(&selector) {
                // Get meta attributes
                let name = element.value().attr("name");
                let property = element.value().attr("property");
                let content = element.value().attr("content");

                if let Some(content) = content {
                    let content = content.to_string();

                    // Standard meta tags
                    if let Some(name) = name {
                        match name {
                            "description" => metadata.description = Some(content.clone()),
                            "author" => metadata.author = Some(content.clone()),
                            "keywords" => {
                                metadata.keywords =
                                    content.split(',').map(|s| s.trim().to_string()).collect();
                            }
                            "date" | "pubdate" => metadata.published_date = Some(content.clone()),
                            _ => {}
                        }
                    }

                    // Open Graph tags
                    if let Some(property) = property {
                        match property {
                            "og:title" => metadata.og_title = Some(content.clone()),
                            "og:description" => metadata.og_description = Some(content.clone()),
                            "og:image" => metadata.og_image = Some(content.clone()),
                            _ => {}
                        }
                    }
                }
            }
        }

        // Extract language
        if let Ok(selector) = Selector::parse("html") {
            if let Some(element) = doc.select(&selector).next() {
                metadata.lang = element.value().attr("lang").map(|s| s.to_string());
            }
        }

        Ok(metadata)
    }

    /// Detect if the page is an article
    fn detect_article(&self, doc: &Html) -> bool {
        // Check for article-specific tags
        let article_selectors = [
            "article",
            "[role='article']",
            ".article",
            ".post",
            ".entry",
            ".content-main",
            "main article",
        ];

        for selector_str in &article_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if doc.select(&selector).next().is_some() {
                    return true;
                }
            }
        }

        // Check for article schema.org markup
        if let Ok(selector) = Selector::parse("[itemtype*='Article']") {
            if doc.select(&selector).next().is_some() {
                return true;
            }
        }

        false
    }

    /// Find and extract main content
    fn find_main_content(&self, doc: &Html, is_article: bool) -> Result<String> {
        // Try article-specific selectors first
        if is_article {
            let article_selectors = [
                "article",
                "main article",
                "[role='main'] article",
                ".article-content",
                ".post-content",
                ".entry-content",
            ];

            for selector_str in &article_selectors {
                if let Ok(selector) = Selector::parse(selector_str) {
                    if let Some(element) = doc.select(&selector).next() {
                        return Ok(element.text().collect::<Vec<_>>().join(" "));
                    }
                }
            }
        }

        // Try generic content selectors
        let content_selectors = [
            "main",
            "[role='main']",
            "#content",
            ".content",
            "#main",
            ".main",
            "body",
        ];

        for selector_str in &content_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = doc.select(&selector).next() {
                    // Remove script and style elements
                    let html = element.html();
                    let cleaned = self.remove_unwanted_elements(&html);
                    let temp_doc = Html::parse_fragment(&cleaned);
                    let text = temp_doc.root_element().text().collect::<Vec<_>>().join(" ");

                    if !text.trim().is_empty() {
                        return Ok(text);
                    }
                }
            }
        }

        // Fallback: use body text
        if let Ok(selector) = Selector::parse("body") {
            if let Some(element) = doc.select(&selector).next() {
                return Ok(element.text().collect::<Vec<_>>().join(" "));
            }
        }

        Ok(String::new())
    }

    /// Remove script, style, and other unwanted elements
    fn remove_unwanted_elements(&self, html: &str) -> String {
        let mut content = html.to_string();

        // Remove script tags
        while let Some(start) = content.find("<script") {
            if let Some(end) = content[start..].find("</script>") {
                content.replace_range(start..start + end + 9, "");
            } else {
                break;
            }
        }

        // Remove style tags
        while let Some(start) = content.find("<style") {
            if let Some(end) = content[start..].find("</style>") {
                content.replace_range(start..start + end + 8, "");
            } else {
                break;
            }
        }

        // Remove common non-content elements
        let unwanted = ["<nav", "<header", "<footer", "<aside", "<form"];
        for tag in &unwanted {
            while let Some(start) = content.find(tag) {
                if let Some(tag_end) = content[start..].find('>') {
                    let tag_name = content[start + 1..start + tag_end]
                        .split_whitespace()
                        .next()
                        .unwrap_or("");
                    let closing = format!("</{}>", tag_name);
                    if let Some(end) = content[start..].find(&closing) {
                        content.replace_range(start..start + end + closing.len(), "");
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        content
    }

    /// Convert HTML to clean Markdown
    fn html_to_markdown(&self, html: &str) -> String {
        let doc = Html::parse_fragment(html);
        let text = doc.root_element().text().collect::<Vec<_>>().join(" ");

        // Clean up whitespace
        text.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }

    /// Extract all links from document
    fn extract_links(&self, doc: &Html, base_url: &Option<Url>) -> Result<Vec<Link>> {
        let mut links = Vec::new();

        if let Ok(selector) = Selector::parse("a[href]") {
            for element in doc.select(&selector) {
                if let Some(href) = element.value().attr("href") {
                    // Resolve relative URLs
                    let url = if let Some(base) = base_url {
                        base.join(href)
                            .unwrap_or_else(|_| Url::parse(href).unwrap_or_else(|_| base.clone()))
                            .to_string()
                    } else {
                        href.to_string()
                    };

                    let text = element.text().collect::<String>().trim().to_string();
                    let title = element.value().attr("title").map(|s| s.to_string());
                    let rel = element.value().attr("rel").map(|s| s.to_string());

                    links.push(Link {
                        url,
                        text,
                        title,
                        rel,
                    });
                }
            }
        }

        Ok(links)
    }

    /// Extract all media (images, videos) from document
    fn extract_media(&self, doc: &Html, base_url: &Option<Url>) -> Result<Vec<Media>> {
        let mut media = Vec::new();

        // Extract images
        if let Ok(selector) = Selector::parse("img[src]") {
            for element in doc.select(&selector) {
                if let Some(src) = element.value().attr("src") {
                    // Resolve relative URLs
                    let url = if let Some(base) = base_url {
                        base.join(src)
                            .unwrap_or_else(|_| Url::parse(src).unwrap_or_else(|_| base.clone()))
                            .to_string()
                    } else {
                        src.to_string()
                    };

                    let alt_text = element.value().attr("alt").map(|s| s.to_string());
                    let title = element.value().attr("title").map(|s| s.to_string());
                    let width = element.value().attr("width").and_then(|s| s.parse().ok());
                    let height = element.value().attr("height").and_then(|s| s.parse().ok());

                    media.push(Media {
                        url,
                        media_type: MediaType::Image,
                        alt_text,
                        title,
                        width,
                        height,
                    });
                }
            }
        }

        // Extract videos
        if let Ok(selector) = Selector::parse("video[src], video source[src]") {
            for element in doc.select(&selector) {
                if let Some(src) = element.value().attr("src") {
                    let url = if let Some(base) = base_url {
                        base.join(src)
                            .unwrap_or_else(|_| Url::parse(src).unwrap_or_else(|_| base.clone()))
                            .to_string()
                    } else {
                        src.to_string()
                    };

                    let title = element.value().attr("title").map(|s| s.to_string());

                    media.push(Media {
                        url,
                        media_type: MediaType::Video,
                        alt_text: None,
                        title,
                        width: None,
                        height: None,
                    });
                }
            }
        }

        Ok(media)
    }

    /// Calculate overall quality score for extracted content
    fn calculate_quality_score(
        &self,
        metadata: &Metadata,
        content: &str,
        links: &[Link],
        media: &[Media],
        is_article: bool,
    ) -> f64 {
        let mut score = 0.0;
        let mut max_score = 0.0;

        // Metadata completeness (40% of total)
        max_score += 40.0;
        if metadata.title.is_some() {
            score += 10.0;
        }
        if metadata.description.is_some() || metadata.og_description.is_some() {
            score += 10.0;
        }
        if metadata.author.is_some() {
            score += 5.0;
        }
        if !metadata.keywords.is_empty() {
            score += 5.0;
        }
        if metadata.og_title.is_some() || metadata.og_image.is_some() {
            score += 10.0;
        }

        // Content quality (40% of total)
        max_score += 40.0;
        let content_len = content.len();
        if content_len > 100 {
            score += 5.0;
        }
        if content_len > 500 {
            score += 10.0;
        }
        if content_len > 1000 {
            score += 10.0;
        }

        // Word count
        let word_count = content.split_whitespace().count();
        if word_count > 50 {
            score += 5.0;
        }
        if word_count > 200 {
            score += 5.0;
        }

        // Sentence structure
        let sentences = content.matches('.').count()
            + content.matches('!').count()
            + content.matches('?').count();
        if sentences > 3 {
            score += 5.0;
        }

        // Structure and richness (20% of total)
        max_score += 20.0;
        if is_article {
            score += 10.0; // Articles are typically higher quality
        }
        if !links.is_empty() {
            score += 5.0;
        }
        if !media.is_empty() {
            score += 5.0;
        }

        // Normalize to 0-1 range
        if max_score > 0.0 {
            score / max_score
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_basic_metadata() {
        let html = r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <title>Test Page</title>
                <meta name="description" content="Test description">
                <meta name="author" content="Test Author">
                <meta property="og:title" content="OG Title">
            </head>
            <body></body>
            </html>
        "#;

        let extractor = EnhancedHtmlExtractor::new(None).unwrap();
        let result = extractor.extract(html, "https://example.com").unwrap();

        assert_eq!(result.metadata.title, Some("Test Page".to_string()));
        assert_eq!(
            result.metadata.description,
            Some("Test description".to_string())
        );
        assert_eq!(result.metadata.author, Some("Test Author".to_string()));
        assert_eq!(result.metadata.og_title, Some("OG Title".to_string()));
        assert_eq!(result.metadata.lang, Some("en".to_string()));
    }

    #[test]
    fn test_detect_article() {
        let html_with_article = r#"
            <html><body><article>Content here</article></body></html>
        "#;

        let html_without_article = r#"
            <html><body><div>Content here</div></body></html>
        "#;

        let extractor = EnhancedHtmlExtractor::new(None).unwrap();

        let doc1 = Html::parse_document(html_with_article);
        assert!(extractor.detect_article(&doc1));

        let doc2 = Html::parse_document(html_without_article);
        assert!(!extractor.detect_article(&doc2));
    }

    #[test]
    fn test_extract_links() {
        let html = r#"
            <html><body>
                <a href="https://example.com">Example</a>
                <a href="/relative" title="Relative Link">Relative</a>
            </body></html>
        "#;

        let extractor = EnhancedHtmlExtractor::new(Some("https://example.com")).unwrap();
        let result = extractor.extract(html, "https://example.com").unwrap();

        assert_eq!(result.links.len(), 2);
        assert_eq!(result.links[0].url, "https://example.com/");
        assert_eq!(result.links[0].text, "Example");
        assert!(result.links[1].url.contains("relative"));
    }

    #[test]
    fn test_extract_images() {
        let html = r#"
            <html><body>
                <img src="https://example.com/image.jpg" alt="Test Image" width="800" height="600">
                <img src="/relative.png" alt="Relative Image">
            </body></html>
        "#;

        let extractor = EnhancedHtmlExtractor::new(Some("https://example.com")).unwrap();
        let result = extractor.extract(html, "https://example.com").unwrap();

        assert_eq!(result.media.len(), 2);
        assert_eq!(result.media[0].media_type, MediaType::Image);
        assert_eq!(result.media[0].alt_text, Some("Test Image".to_string()));
        assert_eq!(result.media[0].width, Some(800));
        assert_eq!(result.media[0].height, Some(600));
    }

    #[test]
    fn test_quality_score_calculation() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>High Quality Article</title>
                <meta name="description" content="A detailed description">
                <meta name="author" content="John Doe">
                <meta property="og:title" content="OG Title">
            </head>
            <body>
                <article>
                    <h1>Main Article</h1>
                    <p>This is a long paragraph with many words. It has multiple sentences.
                    The content is meaningful and well-structured. It provides value to readers.</p>
                    <img src="image.jpg" alt="Image">
                    <a href="https://example.com">Link</a>
                </article>
            </body>
            </html>
        "#;

        let extractor = EnhancedHtmlExtractor::new(None).unwrap();
        let result = extractor.extract(html, "https://example.com").unwrap();

        assert!(
            result.quality_score > 0.6,
            "Quality score should be high for complete content, got: {}",
            result.quality_score
        );
        assert!(result.is_article);
    }
}
