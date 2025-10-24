//! Main DOM crawler implementation that combines all HTML spider functionality

use crate::spider::traits::{
    DomSpider, DomCrawlerResult, FormData, MetaData, ContentAnalysis, DomSpiderConfig
};
use crate::spider::{
    link_extractor::HtmlLinkExtractor,
    form_parser::HtmlFormParser,
    meta_extractor::HtmlMetaExtractor,
};
use anyhow::{Context, Result};
use scraper::Html;
use url::Url;

/// Main HTML DOM crawler that implements the DomSpider trait
pub struct HtmlDomCrawler {
    link_extractor: HtmlLinkExtractor,
    form_parser: HtmlFormParser,
    meta_extractor: HtmlMetaExtractor,
    config: DomSpiderConfig,
}

impl HtmlDomCrawler {
    /// Create a new DOM crawler with configuration
    pub fn new(config: DomSpiderConfig) -> Self {
        let link_extractor = HtmlLinkExtractor::new(config.clone());
        let form_parser = HtmlFormParser::new(config.clone());
        let meta_extractor = HtmlMetaExtractor::new(config.clone());

        Self {
            link_extractor,
            form_parser,
            meta_extractor,
            config,
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(DomSpiderConfig::default())
    }

    /// Comprehensive DOM crawling that extracts everything
    pub async fn crawl_dom(&self, html: &str, base_url: &Url) -> Result<DomCrawlerResult> {
        // Extract all data in parallel for better performance
        let (links, forms, metadata, text_content) = tokio::try_join!(
            self.extract_links(html, base_url),
            self.extract_forms(html, base_url),
            self.extract_metadata(html),
            self.extract_text_content(html)
        )?;

        // Analyze content structure
        let analysis = self.analyze_content(html).await?;

        Ok(DomCrawlerResult {
            links,
            forms,
            metadata,
            text_content,
            analysis,
        })
    }

    /// Extract only navigation-related elements (for focused crawling)
    pub async fn extract_navigation_data(&self, html: &str, base_url: &Url) -> Result<DomCrawlerResult> {
        let nav_links = self.link_extractor.extract_navigation_links(html, base_url).await?;
        let search_forms = self.form_parser.extract_search_forms(html, base_url).await?;
        let basic_metadata = self.meta_extractor.extract_metadata(html).await?;

        let analysis = self.link_extractor.analyze_link_structure(html, base_url).await?;

        Ok(DomCrawlerResult {
            links: nav_links,
            forms: search_forms,
            metadata: basic_metadata,
            text_content: None, // Skip text extraction for navigation focus
            analysis,
        })
    }

    /// Extract only content-related elements (skip navigation)
    pub async fn extract_content_data(&self, html: &str, base_url: &Url) -> Result<DomCrawlerResult> {
        let content_links = self.link_extractor.extract_content_links(html, base_url).await?;
        let content_forms = self.form_parser.extract_forms(html, base_url).await?;
        let full_metadata = self.meta_extractor.extract_metadata(html).await?;
        let text_content = self.extract_text_content(html).await?;

        let analysis = self.analyze_content(html).await?;

        Ok(DomCrawlerResult {
            links: content_links,
            forms: content_forms,
            metadata: full_metadata,
            text_content,
            analysis,
        })
    }

    /// Check if page requires authentication for full access
    pub async fn requires_authentication(&self, html: &str, base_url: &Url) -> Result<bool> {
        self.form_parser.has_authenticated_forms(html, base_url).await
    }

    /// Extract structured data for SEO analysis
    pub async fn extract_seo_data(&self, html: &str) -> Result<serde_json::Value> {
        let seo_metadata = self.meta_extractor.extract_seo_metadata(html).await?;
        let social_metadata = self.meta_extractor.extract_social_metadata(html).await?;
        let accessibility_data = self.meta_extractor.extract_accessibility_metadata(html).await?;

        let mut seo_data = serde_json::Map::new();
        seo_data.insert("seo".to_string(), serde_json::to_value(seo_metadata)?);
        seo_data.insert("social".to_string(), serde_json::to_value(social_metadata)?);
        seo_data.insert("accessibility".to_string(), serde_json::to_value(accessibility_data)?);

        Ok(serde_json::Value::Object(seo_data))
    }

    /// Validate HTML quality for spider optimization
    pub async fn validate_html_quality(&self, html: &str) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        // Check metadata quality
        let meta_issues = self.meta_extractor.validate_metadata_quality(html).await?;
        issues.extend(meta_issues);

        // Check document structure
        let structure_issues = self.validate_document_structure(html)?;
        issues.extend(structure_issues);

        // Check link quality
        let link_issues = self.validate_link_quality(html)?;
        issues.extend(link_issues);

        Ok(issues)
    }

    /// Validate document structure
    fn validate_document_structure(&self, html: &str) -> Result<Vec<String>> {
        let document = Html::parse_document(html);
        let mut issues = Vec::new();

        // Check for basic HTML structure
        if !html.contains("<html") {
            issues.push("Missing HTML tag".to_string());
        }

        if !html.contains("<head") {
            issues.push("Missing HEAD tag".to_string());
        }

        if !html.contains("<body") {
            issues.push("Missing BODY tag".to_string());
        }

        // Check heading hierarchy
        let h1_selector = scraper::Selector::parse("h1")
            .map_err(|_| anyhow::anyhow!("Invalid h1 selector"))?;
        let h1_count = document.root_element().select(&h1_selector).count();
        if h1_count == 0 {
            issues.push("Missing H1 heading".to_string());
        } else if h1_count > 1 {
            issues.push("Multiple H1 headings found".to_string());
        }

        Ok(issues)
    }

    /// Validate link quality
    fn validate_link_quality(&self, html: &str) -> Result<Vec<String>> {
        let document = Html::parse_document(html);
        let mut issues = Vec::new();

        // Check for empty href attributes
        let empty_links_selector = scraper::Selector::parse("a[href=\"\"], a[href=\"#\"]")
            .map_err(|_| anyhow::anyhow!("Invalid empty links selector"))?;
        let empty_links = document
            .select(&empty_links_selector)
            .count();

        if empty_links > 0 {
            issues.push(format!("Found {} empty or fragment-only links", empty_links));
        }

        // Check for missing alt text on linked images
        let img_alt_selector = scraper::Selector::parse("a img:not([alt])")
            .map_err(|_| anyhow::anyhow!("Invalid image alt selector"))?;
        let linked_images_without_alt = document
            .select(&img_alt_selector)
            .count();

        if linked_images_without_alt > 0 {
            issues.push(format!(
                "Found {} linked images without alt text",
                linked_images_without_alt
            ));
        }

        Ok(issues)
    }

    /// Extract page performance hints for spider optimization
    pub async fn extract_performance_hints(&self, html: &str) -> Result<serde_json::Value> {
        let document = Html::parse_document(html);
        let mut hints = serde_json::Map::new();

        // Count external resources
        let script_selector = scraper::Selector::parse("script[src]")
            .map_err(|_| anyhow::anyhow!("Invalid script selector"))?;
        let external_scripts = document
            .select(&script_selector)
            .count();
        hints.insert("external_scripts".to_string(), serde_json::Value::Number(external_scripts.into()));

        let stylesheet_selector = scraper::Selector::parse("link[rel=\"stylesheet\"]")
            .map_err(|_| anyhow::anyhow!("Invalid stylesheet selector"))?;
        let external_stylesheets = document
            .select(&stylesheet_selector)
            .count();
        hints.insert("external_stylesheets".to_string(), serde_json::Value::Number(external_stylesheets.into()));

        let img_selector = scraper::Selector::parse("img")
            .map_err(|_| anyhow::anyhow!("Invalid img selector"))?;
        let images = document
            .select(&img_selector)
            .count();
        hints.insert("images".to_string(), serde_json::Value::Number(images.into()));

        // Check for performance-related meta tags
        let mut performance_meta = Vec::new();
        let meta_selector = scraper::Selector::parse("meta")
            .map_err(|_| anyhow::anyhow!("Invalid meta selector"))?;
        for meta in document.select(&meta_selector) {
            if let Some(name) = meta.value().attr("name") {
                if name.contains("preload") || name.contains("prefetch") || name.contains("dns-prefetch") {
                    performance_meta.push(name.to_string());
                }
            }
        }
        hints.insert("performance_meta".to_string(), serde_json::Value::Array(
            performance_meta.into_iter().map(serde_json::Value::String).collect()
        ));

        // Estimate page complexity
        let all_selector = scraper::Selector::parse("*")
            .map_err(|_| anyhow::anyhow!("Invalid wildcard selector"))?;
        let total_elements = document.select(&all_selector).count();
        hints.insert("total_elements".to_string(), serde_json::Value::Number(total_elements.into()));

        let complexity_score = if total_elements < 100 {
            "low"
        } else if total_elements < 500 {
            "medium"
        } else {
            "high"
        };
        hints.insert("complexity".to_string(), serde_json::Value::String(complexity_score.to_string()));

        Ok(serde_json::Value::Object(hints))
    }
}

#[async_trait::async_trait]
impl DomSpider for HtmlDomCrawler {
    /// Extract all links from HTML content
    async fn extract_links(&self, html: &str, base_url: &Url) -> Result<Vec<Url>> {
        self.link_extractor.extract_links(html, base_url).await
    }

    /// Extract forms from HTML content
    async fn extract_forms(&self, html: &str, base_url: &Url) -> Result<Vec<FormData>> {
        self.form_parser.extract_forms(html, base_url).await
    }

    /// Extract metadata from HTML content
    async fn extract_metadata(&self, html: &str) -> Result<MetaData> {
        self.meta_extractor.extract_metadata(html).await
    }

    /// Extract plain text content from HTML
    async fn extract_text_content(&self, html: &str) -> Result<Option<String>> {
        let document = Html::parse_document(html);

        // Remove script and style content
        let mut clean_html = html.to_string();

        // Simple removal of script and style tags (in production, use proper HTML cleaning)
        if let Ok(script_regex) = regex::Regex::new(r"<script[^>]*>.*?</script>") {
            clean_html = script_regex.replace_all(&clean_html, "").to_string();
        }

        if let Ok(style_regex) = regex::Regex::new(r"<style[^>]*>.*?</style>") {
            clean_html = style_regex.replace_all(&clean_html, "").to_string();
        }

        // Extract text using simple tag removal
        let text = self.simple_text_extraction(&clean_html);

        if text.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(text))
        }
    }

    /// Analyze HTML content for spider optimization hints
    async fn analyze_content(&self, html: &str) -> Result<ContentAnalysis> {
        // Use the link extractor's analysis as the base
        let base_url = Url::parse("http://example.com")
            .map_err(|e| anyhow::anyhow!("Invalid base URL: {}", e))?;
        self.link_extractor.analyze_link_structure(html, &base_url).await
    }
}

impl HtmlDomCrawler {
    /// Simple text extraction (fallback method)
    fn simple_text_extraction(&self, html: &str) -> String {
        let mut text = String::new();
        let mut in_tag = false;
        let mut last_char_space = false;

        for char in html.chars() {
            match char {
                '<' => in_tag = true,
                '>' => {
                    in_tag = false;
                    if !last_char_space {
                        text.push(' ');
                        last_char_space = true;
                    }
                }
                c if !in_tag => {
                    if c.is_whitespace() {
                        if !last_char_space {
                            text.push(' ');
                            last_char_space = true;
                        }
                    } else {
                        text.push(c);
                        last_char_space = false;
                    }
                }
                _ => {}
            }
        }

        // Clean up multiple spaces and trim
        regex::Regex::new(r"\s+")
            .unwrap()
            .replace_all(text.trim(), " ")
            .to_string()
    }
}