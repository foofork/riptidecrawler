//! Core parser implementation for native HTML parsing

use anyhow::Result as AnyhowResult;
use riptide_types::extractors::HtmlParser as HtmlParserTrait;
use riptide_types::{ExtractedDoc, ParserMetadata};
use scraper::Html;
use tracing::{debug, warn};

use crate::native_parser::{
    error::{NativeParserError, Result},
    extractors::*,
    fallbacks::FallbackStrategy,
    quality::QualityAssessor,
};

/// Parser configuration
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Enable markdown generation
    pub enable_markdown: bool,
    /// Enable link extraction
    pub extract_links: bool,
    /// Enable media extraction
    pub extract_media: bool,
    /// Enable language detection
    pub detect_language: bool,
    /// Enable category extraction
    pub extract_categories: bool,
    /// Maximum content length (bytes)
    pub max_content_length: usize,
    /// Parse timeout (milliseconds)
    pub parse_timeout_ms: u64,
    /// Minimum quality score threshold
    pub min_quality_score: u32,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            enable_markdown: true,
            extract_links: true,
            extract_media: true,
            detect_language: true,
            extract_categories: true,
            max_content_length: 10_000_000, // 10MB
            parse_timeout_ms: 5000,         // 5 seconds
            min_quality_score: 15,          // Lowered from 30 to allow more content through
        }
    }
}

/// Native HTML parser for headless-rendered content
pub struct NativeHtmlParser {
    config: ParserConfig,
}

impl NativeHtmlParser {
    /// Create new parser with default config
    pub fn new() -> Self {
        Self::with_config(ParserConfig::default())
    }

    /// Create parser with custom config
    pub fn with_config(config: ParserConfig) -> Self {
        Self { config }
    }

    /// Parse headless-rendered HTML and extract document
    ///
    /// # Arguments
    /// * `html` - Fully rendered HTML from headless service
    /// * `url` - Original URL (for link resolution)
    ///
    /// # Returns
    /// * `Ok(ExtractedDoc)` - Successfully extracted document
    /// * `Err(NativeParserError)` - Parsing or extraction failed
    pub fn parse_headless_html(&self, html: &str, url: &str) -> Result<ExtractedDoc> {
        // 1. Validate input
        self.validate_html(html)?;

        debug!(
            url = %url,
            html_size = html.len(),
            "Starting native HTML parsing"
        );

        // 2. Parse HTML document (native Rust - won't crash!)
        let document = Html::parse_document(html);

        // 3. Extract all components
        let title = TitleExtractor::extract(&document);
        let byline = MetadataExtractor::extract_byline(&document);
        let published = MetadataExtractor::extract_published_date(&document);
        let description = MetadataExtractor::extract_description(&document);
        let site_name = MetadataExtractor::extract_site_name(&document);

        // 4. Extract content (text + markdown)
        let (text, markdown) = ContentExtractor::extract(&document, url)?;

        // 5. Extract links (conditional)
        let links = if self.config.extract_links {
            LinkExtractor::extract(&document, url)
        } else {
            Vec::new()
        };

        // 6. Extract media (conditional)
        let media = if self.config.extract_media {
            MediaExtractor::extract(&document, url)
        } else {
            Vec::new()
        };

        // 7. Detect language (conditional)
        let language = if self.config.detect_language {
            LanguageDetector::detect(&document, html)
        } else {
            None
        };

        // 8. Extract categories (conditional)
        let categories = if self.config.extract_categories {
            CategoryExtractor::extract(&document)
        } else {
            Vec::new()
        };

        // 9. Calculate quality metrics
        let word_count = text.split_whitespace().count();
        let reading_time = (word_count / 200).max(1); // 200 wpm
        let quality_score = QualityAssessor::calculate(&text, &markdown, &title);

        debug!(
            url = %url,
            word_count = word_count,
            quality_score = quality_score,
            "Extraction completed"
        );

        // 10. Build result
        let doc = ExtractedDoc {
            url: url.to_string(),
            title,
            byline,
            published_iso: published,
            markdown,
            text,
            links,
            media,
            language,
            reading_time: Some(reading_time as u32),
            quality_score: Some(quality_score as u8),
            word_count: Some(word_count as u32),
            parser_metadata: Some(ParserMetadata {
                parser_used: "native".to_string(),
                confidence_score: quality_score as f64 / 100.0,
                fallback_occurred: false,
                parse_time_ms: 0,
                extraction_path: None,
                primary_error: None,
            }),
            categories,
            site_name,
            description,
            html: None, // We don't store the original HTML
        };

        // 11. Validate minimum quality
        if quality_score < self.config.min_quality_score as usize {
            return Err(NativeParserError::LowQuality {
                score: quality_score as f32,
                threshold: self.config.min_quality_score as f32,
            });
        }

        Ok(doc)
    }

    /// Extract document with quality-based fallbacks
    ///
    /// Tries multiple extraction strategies if initial attempt
    /// produces low-quality results.
    pub fn extract_with_fallbacks(&self, html: &str, url: &str) -> Result<ExtractedDoc> {
        // Primary extraction
        match self.parse_headless_html(html, url) {
            Ok(doc) => {
                // Check quality
                let quality = doc.quality_score.unwrap_or(0);
                if quality >= 60 {
                    return Ok(doc);
                }
                // Low quality - try fallback
                warn!(
                    url = %url,
                    quality = quality,
                    "Primary extraction low quality, trying fallback"
                );
            }
            Err(e) => {
                warn!(
                    url = %url,
                    error = %e,
                    "Primary extraction failed, trying fallback"
                );
            }
        }

        // Fallback strategy 1: Full content extraction
        let doc = FallbackStrategy::full_content_extraction(html, url)?;
        if doc.quality_score.unwrap_or(0) >= 40 {
            return Ok(doc);
        }

        // Fallback strategy 2: Simple text extraction
        FallbackStrategy::simple_text_extraction(html, url)
    }

    /// Validate HTML before parsing
    fn validate_html(&self, html: &str) -> Result<()> {
        // Check size
        if html.len() > self.config.max_content_length {
            return Err(NativeParserError::OversizedHtml {
                size: html.len(),
                max: self.config.max_content_length,
            });
        }

        // Check encoding (must be valid UTF-8)
        if !html.is_ascii() && html.contains('\u{FFFD}') {
            return Err(NativeParserError::EncodingError(
                "Invalid UTF-8 encoding detected".to_string(),
            ));
        }

        // Check basic HTML structure
        if !html.contains("<html") && !html.contains("<HTML") {
            return Err(NativeParserError::InvalidStructure(
                "Missing <html> tag".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for NativeHtmlParser {
    fn default() -> Self {
        Self::new()
    }
}

// Implement HtmlParser trait for dependency injection in riptide-reliability
impl HtmlParserTrait for NativeHtmlParser {
    fn parse_html(&self, html: &str, url: &str) -> AnyhowResult<ExtractedDoc> {
        self.parse_headless_html(html, url)
            .map_err(|e| anyhow::anyhow!("Native HTML parsing failed: {}", e))
    }

    fn parse_with_fallbacks(&self, html: &str, url: &str) -> AnyhowResult<ExtractedDoc> {
        self.extract_with_fallbacks(html, url)
            .map_err(|e| anyhow::anyhow!("Native HTML parsing with fallbacks failed: {}", e))
    }
}
