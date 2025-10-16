//! HTML metadata extraction functionality

use crate::spider::traits::{MetaData, DomSpiderConfig};
use anyhow::{Context, Result};
use scraper::{Html, Selector};
use std::collections::HashMap;
use url::Url;

/// HTML metadata extractor for extracting page metadata and structured data
pub struct HtmlMetaExtractor {
    config: DomSpiderConfig,
}

impl HtmlMetaExtractor {
    /// Create a new meta extractor with configuration
    pub fn new(config: DomSpiderConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(DomSpiderConfig::default())
    }

    /// Extract all metadata from HTML content
    pub async fn extract_metadata(&self, html: &str) -> Result<MetaData> {
        if !self.config.extract_metadata {
            return Ok(MetaData::default());
        }

        let document = Html::parse_document(html);
        let mut metadata = MetaData::default();

        // Extract title
        metadata.title = self.extract_title(&document);

        // Extract basic meta tags
        self.extract_basic_meta(&document, &mut metadata)?;

        // Extract Open Graph metadata
        metadata.og_data = self.extract_open_graph(&document)?;

        // Extract Twitter Card metadata
        metadata.twitter_data = self.extract_twitter_card(&document)?;

        // Extract canonical URL
        metadata.canonical = self.extract_canonical_url(&document)?;

        // Extract structured data
        metadata.structured_data = self.extract_structured_data(&document)?;

        // Extract custom meta tags
        metadata.custom_meta = self.extract_custom_meta(&document)?;

        Ok(metadata)
    }

    /// Extract page title
    fn extract_title(&self, document: &Html) -> Option<String> {
        let title_selector = Selector::parse("title").ok()?;
        document
            .select(&title_selector)
            .next()
            .map(|title_elem| title_elem.text().collect::<String>().trim().to_string())
            .filter(|title| !title.is_empty())
    }

    /// Extract basic meta tags (description, keywords, author, etc.)
    fn extract_basic_meta(&self, document: &Html, metadata: &mut MetaData) -> Result<()> {
        let meta_selector = Selector::parse("meta[name], meta[property]")
            .map_err(|e| anyhow::anyhow!("Invalid meta selector: {}", e))?;

        for meta_elem in document.select(&meta_selector) {
            let name = meta_elem
                .value()
                .attr("name")
                .or_else(|| meta_elem.value().attr("property"))
                .unwrap_or("");

            let content = meta_elem.value().attr("content").unwrap_or("");

            if content.is_empty() {
                continue;
            }

            match name.to_lowercase().as_str() {
                "description" => {
                    metadata.description = Some(content.to_string());
                }
                "keywords" => {
                    metadata.keywords = content
                        .split(',')
                        .map(|k| k.trim().to_string())
                        .filter(|k| !k.is_empty())
                        .collect();
                }
                "author" => {
                    metadata.author = Some(content.to_string());
                }
                "language" | "lang" => {
                    metadata.language = Some(content.to_string());
                }
                _ => {
                    // Store other meta tags if configured
                    if self.config.meta_tags_to_extract.contains(&name.to_string())
                        || self.config.meta_tags_to_extract.is_empty()
                    {
                        metadata.custom_meta.insert(name.to_string(), content.to_string());
                    }
                }
            }
        }

        Ok(())
    }

    /// Extract Open Graph metadata
    fn extract_open_graph(&self, document: &Html) -> Result<HashMap<String, String>> {
        let mut og_data = HashMap::new();

        let og_selector = Selector::parse("meta[property^=\"og:\"]")
            .map_err(|e| anyhow::anyhow!("Invalid OG selector: {}", e))?;

        for meta_elem in document.select(&og_selector) {
            if let (Some(property), Some(content)) = (
                meta_elem.value().attr("property"),
                meta_elem.value().attr("content"),
            ) {
                if !content.is_empty() {
                    // Remove "og:" prefix for cleaner keys
                    let key = property.strip_prefix("og:").unwrap_or(property);
                    og_data.insert(key.to_string(), content.to_string());
                }
            }
        }

        Ok(og_data)
    }

    /// Extract Twitter Card metadata
    fn extract_twitter_card(&self, document: &Html) -> Result<HashMap<String, String>> {
        let mut twitter_data = HashMap::new();

        let twitter_selector = Selector::parse("meta[name^=\"twitter:\"]")
            .map_err(|e| anyhow::anyhow!("Invalid Twitter selector: {}", e))?;

        for meta_elem in document.select(&twitter_selector) {
            if let (Some(name), Some(content)) = (
                meta_elem.value().attr("name"),
                meta_elem.value().attr("content"),
            ) {
                if !content.is_empty() {
                    // Remove "twitter:" prefix for cleaner keys
                    let key = name.strip_prefix("twitter:").unwrap_or(name);
                    twitter_data.insert(key.to_string(), content.to_string());
                }
            }
        }

        Ok(twitter_data)
    }

    /// Extract canonical URL
    fn extract_canonical_url(&self, document: &Html) -> Result<Option<Url>> {
        let canonical_selector = Selector::parse("link[rel=\"canonical\"]")
            .map_err(|e| anyhow::anyhow!("Invalid canonical selector: {}", e))?;

        if let Some(canonical_elem) = document.select(&canonical_selector).next() {
            if let Some(href) = canonical_elem.value().attr("href") {
                if let Ok(url) = Url::parse(href) {
                    return Ok(Some(url));
                }
            }
        }

        Ok(None)
    }

    /// Extract structured data (JSON-LD, Microdata, RDFa)
    fn extract_structured_data(&self, document: &Html) -> Result<Vec<String>> {
        let mut structured_data = Vec::new();

        // Extract JSON-LD
        let jsonld_selector = Selector::parse("script[type=\"application/ld+json\"]")
            .map_err(|e| anyhow::anyhow!("Invalid JSON-LD selector: {}", e))?;

        for script_elem in document.select(&jsonld_selector) {
            let json_content = script_elem.text().collect::<String>();
            if !json_content.trim().is_empty() {
                structured_data.push(json_content.trim().to_string());
            }
        }

        // Extract Microdata (itemscope elements)
        let microdata_selector = Selector::parse("[itemscope]")
            .map_err(|e| anyhow::anyhow!("Invalid microdata selector: {}", e))?;

        for item_elem in document.select(&microdata_selector) {
            if let Some(itemtype) = item_elem.value().attr("itemtype") {
                structured_data.push(format!("microdata:itemtype:{}", itemtype));
            }
        }

        // Extract RDFa (typeof attributes)
        let rdfa_selector = Selector::parse("[typeof]")
            .map_err(|e| anyhow::anyhow!("Invalid RDFa selector: {}", e))?;

        for rdfa_elem in document.select(&rdfa_selector) {
            if let Some(typeof_val) = rdfa_elem.value().attr("typeof") {
                structured_data.push(format!("rdfa:typeof:{}", typeof_val));
            }
        }

        Ok(structured_data)
    }

    /// Extract custom meta tags not covered by standard extraction
    fn extract_custom_meta(&self, document: &Html) -> Result<HashMap<String, String>> {
        let mut custom_meta = HashMap::new();

        // Extract viewport and other important meta tags
        let viewport_selector = Selector::parse("meta[name=\"viewport\"]")
            .map_err(|e| anyhow::anyhow!("Invalid viewport selector: {}", e))?;

        if let Some(viewport_elem) = document.select(&viewport_selector).next() {
            if let Some(content) = viewport_elem.value().attr("content") {
                custom_meta.insert("viewport".to_string(), content.to_string());
            }
        }

        // Extract robots meta
        let robots_selector = Selector::parse("meta[name=\"robots\"]")
            .map_err(|e| anyhow::anyhow!("Invalid robots selector: {}", e))?;

        if let Some(robots_elem) = document.select(&robots_selector).next() {
            if let Some(content) = robots_elem.value().attr("content") {
                custom_meta.insert("robots".to_string(), content.to_string());
            }
        }

        // Extract refresh meta
        let refresh_selector = Selector::parse("meta[http-equiv=\"refresh\"]")
            .map_err(|e| anyhow::anyhow!("Invalid refresh selector: {}", e))?;

        if let Some(refresh_elem) = document.select(&refresh_selector).next() {
            if let Some(content) = refresh_elem.value().attr("content") {
                custom_meta.insert("refresh".to_string(), content.to_string());
            }
        }

        // Extract generator meta
        let generator_selector = Selector::parse("meta[name=\"generator\"]")
            .map_err(|e| anyhow::anyhow!("Invalid generator selector: {}", e))?;

        if let Some(generator_elem) = document.select(&generator_selector).next() {
            if let Some(content) = generator_elem.value().attr("content") {
                custom_meta.insert("generator".to_string(), content.to_string());
            }
        }

        Ok(custom_meta)
    }

    /// Extract language information from various sources
    pub async fn extract_language_info(&self, html: &str) -> Result<Option<String>> {
        let document = Html::parse_document(html);

        // Check html lang attribute
        let html_selector = Selector::parse("html")
            .map_err(|e| anyhow::anyhow!("Invalid html selector: {}", e))?;

        if let Some(html_elem) = document.select(&html_selector).next() {
            if let Some(lang) = html_elem.value().attr("lang") {
                return Ok(Some(lang.to_string()));
            }
        }

        // Check meta language tag
        let lang_selector = Selector::parse("meta[name=\"language\"], meta[http-equiv=\"content-language\"]")
            .map_err(|e| anyhow::anyhow!("Invalid language selector: {}", e))?;

        if let Some(lang_elem) = document.select(&lang_selector).next() {
            if let Some(content) = lang_elem.value().attr("content") {
                return Ok(Some(content.to_string()));
            }
        }

        Ok(None)
    }

    /// Extract SEO-relevant metadata
    pub async fn extract_seo_metadata(&self, html: &str) -> Result<HashMap<String, String>> {
        let document = Html::parse_document(html);
        let mut seo_data = HashMap::new();

        // Extract title
        if let Some(title) = self.extract_title(&document) {
            seo_data.insert("title".to_string(), title);
        }

        // Extract meta description
        let desc_selector = Selector::parse("meta[name=\"description\"]")
            .map_err(|e| anyhow::anyhow!("Invalid description selector: {}", e))?;

        if let Some(desc_elem) = document.select(&desc_selector).next() {
            if let Some(content) = desc_elem.value().attr("content") {
                seo_data.insert("description".to_string(), content.to_string());
            }
        }

        // Extract canonical URL
        if let Some(canonical) = self.extract_canonical_url(&document)? {
            seo_data.insert("canonical".to_string(), canonical.to_string());
        }

        // Extract robots directive
        let robots_selector = Selector::parse("meta[name=\"robots\"]")
            .map_err(|e| anyhow::anyhow!("Invalid robots selector: {}", e))?;

        if let Some(robots_elem) = document.select(&robots_selector).next() {
            if let Some(content) = robots_elem.value().attr("content") {
                seo_data.insert("robots".to_string(), content.to_string());
            }
        }

        // Extract Open Graph title and description
        let og_title_selector = Selector::parse("meta[property=\"og:title\"]")
            .map_err(|e| anyhow::anyhow!("Invalid OG title selector: {}", e))?;

        if let Some(og_title_elem) = document.select(&og_title_selector).next() {
            if let Some(content) = og_title_elem.value().attr("content") {
                seo_data.insert("og_title".to_string(), content.to_string());
            }
        }

        let og_desc_selector = Selector::parse("meta[property=\"og:description\"]")
            .map_err(|e| anyhow::anyhow!("Invalid OG description selector: {}", e))?;

        if let Some(og_desc_elem) = document.select(&og_desc_selector).next() {
            if let Some(content) = og_desc_elem.value().attr("content") {
                seo_data.insert("og_description".to_string(), content.to_string());
            }
        }

        Ok(seo_data)
    }

    /// Extract social media metadata
    pub async fn extract_social_metadata(&self, html: &str) -> Result<HashMap<String, HashMap<String, String>>> {
        let mut social_data = HashMap::new();

        // Extract Open Graph data
        let og_data = self.extract_open_graph(&Html::parse_document(html))?;
        if !og_data.is_empty() {
            social_data.insert("open_graph".to_string(), og_data);
        }

        // Extract Twitter Card data
        let twitter_data = self.extract_twitter_card(&Html::parse_document(html))?;
        if !twitter_data.is_empty() {
            social_data.insert("twitter".to_string(), twitter_data);
        }

        // Extract Facebook-specific metadata
        let fb_data = self.extract_facebook_metadata(&Html::parse_document(html))?;
        if !fb_data.is_empty() {
            social_data.insert("facebook".to_string(), fb_data);
        }

        Ok(social_data)
    }

    /// Extract Facebook-specific metadata
    fn extract_facebook_metadata(&self, document: &Html) -> Result<HashMap<String, String>> {
        let mut fb_data = HashMap::new();

        let fb_selector = Selector::parse("meta[property^=\"fb:\"]")
            .map_err(|e| anyhow::anyhow!("Invalid FB selector: {}", e))?;

        for meta_elem in document.select(&fb_selector) {
            if let (Some(property), Some(content)) = (
                meta_elem.value().attr("property"),
                meta_elem.value().attr("content"),
            ) {
                if !content.is_empty() {
                    let key = property.strip_prefix("fb:").unwrap_or(property);
                    fb_data.insert(key.to_string(), content.to_string());
                }
            }
        }

        Ok(fb_data)
    }

    /// Extract accessibility metadata
    pub async fn extract_accessibility_metadata(&self, html: &str) -> Result<HashMap<String, String>> {
        let document = Html::parse_document(html);
        let mut accessibility_data = HashMap::new();

        // Check for alt text on images
        let img_selector = Selector::parse("img")
            .map_err(|e| anyhow::anyhow!("Invalid img selector: {}", e))?;

        let total_images = document.select(&img_selector).count();
        let images_with_alt = document
            .select(&img_selector)
            .filter(|img| img.value().attr("alt").is_some())
            .count();

        if total_images > 0 {
            let alt_coverage = (images_with_alt as f64 / total_images as f64) * 100.0;
            accessibility_data.insert(
                "alt_text_coverage".to_string(),
                format!("{:.1}%", alt_coverage),
            );
        }

        // Check for ARIA landmarks
        let landmark_selector = Selector::parse("[role]")
            .map_err(|e| anyhow::anyhow!("Invalid role selector: {}", e))?;

        let landmark_count = document.select(&landmark_selector).count();
        accessibility_data.insert("aria_landmarks".to_string(), landmark_count.to_string());

        // Check for heading structure
        let heading_selector = Selector::parse("h1, h2, h3, h4, h5, h6")
            .map_err(|e| anyhow::anyhow!("Invalid heading selector: {}", e))?;

        let heading_count = document.select(&heading_selector).count();
        accessibility_data.insert("heading_count".to_string(), heading_count.to_string());

        Ok(accessibility_data)
    }

    /// Check if page has metadata quality issues
    pub async fn validate_metadata_quality(&self, html: &str) -> Result<Vec<String>> {
        let metadata = self.extract_metadata(html).await?;
        let mut issues = Vec::new();

        // Check for missing title
        if metadata.title.is_none() || metadata.title.as_ref().unwrap().is_empty() {
            issues.push("Missing or empty page title".to_string());
        } else if let Some(title) = &metadata.title {
            if title.len() < 30 {
                issues.push("Page title is too short (< 30 characters)".to_string());
            } else if title.len() > 60 {
                issues.push("Page title is too long (> 60 characters)".to_string());
            }
        }

        // Check for missing description
        if metadata.description.is_none() || metadata.description.as_ref().unwrap().is_empty() {
            issues.push("Missing or empty meta description".to_string());
        } else if let Some(desc) = &metadata.description {
            if desc.len() < 120 {
                issues.push("Meta description is too short (< 120 characters)".to_string());
            } else if desc.len() > 160 {
                issues.push("Meta description is too long (> 160 characters)".to_string());
            }
        }

        // Check for missing Open Graph data
        if metadata.og_data.is_empty() {
            issues.push("Missing Open Graph metadata".to_string());
        }

        // Check for missing canonical URL
        if metadata.canonical.is_none() {
            issues.push("Missing canonical URL".to_string());
        }

        Ok(issues)
    }
}