//! ExtractionFacade - Comprehensive content extraction with multiple strategies
//!
//! Provides unified interface for extracting content from various sources:
//! - HTML extraction (clean, markdown, structured)
//! - PDF extraction (text, metadata, images)
//! - JSON/API extraction
//! - Schema-based extraction
//! - AI-powered extraction

use crate::config::RiptideConfig;
use crate::error::RiptideError;
use riptide_extraction::{css_extract, ContentExtractor, CssExtractorStrategy, UnifiedExtractor};

#[cfg(feature = "wasm-extractor")]
use riptide_extraction::StrategyWasmExtractor;

use riptide_pdf::{create_pdf_processor, AnyPdfProcessor, PdfConfig};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Result type alias for extraction operations
pub type Result<T> = std::result::Result<T, RiptideError>;

/// Extraction strategies available
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtractionStrategy {
    /// HTML extraction with CSS selectors
    HtmlCss,
    /// HTML extraction with regex patterns
    HtmlRegex,
    /// WASM-based extraction (high quality)
    Wasm,
    /// Fallback extraction (basic scraping)
    Fallback,
    /// PDF text extraction
    PdfText,
    /// Schema-based extraction
    Schema,
}

impl ExtractionStrategy {
    /// Get strategy name
    pub fn name(&self) -> &'static str {
        match self {
            Self::HtmlCss => "html_css",
            Self::HtmlRegex => "html_regex",
            Self::Wasm => "wasm",
            Self::Fallback => "fallback",
            Self::PdfText => "pdf_text",
            Self::Schema => "schema",
        }
    }
}

/// Options for HTML extraction
#[derive(Debug, Clone, Default)]
pub struct HtmlExtractionOptions {
    /// Extract as markdown
    pub as_markdown: bool,
    /// Clean extracted text
    pub clean: bool,
    /// Include metadata
    pub include_metadata: bool,
    /// Extract links
    pub extract_links: bool,
    /// Extract images
    pub extract_images: bool,
    /// Custom CSS selectors
    pub custom_selectors: Option<HashMap<String, String>>,
    /// Specific extraction strategy to use (None = auto/UnifiedExtractor)
    pub extraction_strategy: Option<ExtractionStrategy>,
}

/// Options for PDF extraction
#[derive(Debug, Clone, Default)]
pub struct PdfExtractionOptions {
    /// Extract text
    pub extract_text: bool,
    /// Extract metadata
    pub extract_metadata: bool,
    /// Extract images
    pub extract_images: bool,
    /// Include page numbers
    pub include_page_numbers: bool,
}

/// Extracted content with rich metadata
#[derive(Debug, Clone)]
pub struct ExtractedData {
    /// Extracted title
    pub title: Option<String>,
    /// Main text content
    pub text: String,
    /// Markdown representation
    pub markdown: Option<String>,
    /// Metadata key-value pairs
    pub metadata: HashMap<String, String>,
    /// Extracted links
    pub links: Vec<String>,
    /// Extracted images
    pub images: Vec<String>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Strategy used for extraction
    pub strategy_used: String,
    /// Source URL
    pub url: String,
    /// Raw HTML content (only included if requested)
    pub raw_html: Option<String>,
}

/// Schema definition for structured extraction
#[derive(Debug, Clone)]
pub struct Schema {
    /// Field definitions
    pub fields: HashMap<String, FieldSpec>,
}

/// Field specification for schema
#[derive(Debug, Clone)]
pub struct FieldSpec {
    /// CSS selector for field
    pub selector: String,
    /// Field is required
    pub required: bool,
    /// Field type
    pub field_type: FieldType,
}

/// Field type for schema extraction
#[derive(Debug, Clone)]
pub enum FieldType {
    Text,
    Number,
    Url,
    Date,
}

/// Registry of extraction strategies
struct ExtractionRegistry {
    strategies: HashMap<String, Box<dyn ContentExtractor>>,
}

impl ExtractionRegistry {
    fn new() -> Self {
        Self {
            strategies: HashMap::new(),
        }
    }

    async fn register_default_strategies(&mut self) -> Result<()> {
        // Register WASM extractor (only when feature enabled)
        #[cfg(feature = "wasm-extractor")]
        {
            if let Ok(wasm) = StrategyWasmExtractor::new(None).await {
                self.strategies.insert("wasm".to_string(), Box::new(wasm));
            }
        }

        // Register CSS extractor
        let css = CssExtractorStrategy::new();
        self.strategies
            .insert("html_css".to_string(), Box::new(css));

        Ok(())
    }

    fn get_strategy(&self, name: &str) -> Option<&dyn ContentExtractor> {
        self.strategies.get(name).map(|b| b.as_ref())
    }
}

/// Main extraction facade
pub struct ExtractionFacade {
    #[allow(dead_code)]
    config: RiptideConfig,
    extractors: Arc<RwLock<ExtractionRegistry>>,
    pdf_processor: AnyPdfProcessor,
}

impl ExtractionFacade {
    /// Create a new extraction facade
    pub async fn new(config: RiptideConfig) -> Result<Self> {
        let mut registry = ExtractionRegistry::new();
        registry.register_default_strategies().await?;

        Ok(Self {
            config,
            extractors: Arc::new(RwLock::new(registry)),
            pdf_processor: create_pdf_processor(),
        })
    }

    /// Extract content from URL (fetch + extract)
    pub async fn extract_from_url(
        &self,
        url: &str,
        options: HtmlExtractionOptions,
    ) -> Result<ExtractedData> {
        // Fetch HTML content using riptide-fetch
        let fetcher = riptide_fetch::FetchEngine::new()
            .map_err(|e| RiptideError::fetch(url, e.to_string()))?;

        let html = fetcher
            .fetch_text(url)
            .await
            .map_err(|e| RiptideError::fetch(url, e.to_string()))?;

        // Extract content - store raw HTML if requested via options
        let mut extracted = self.extract_html(&html, url, options).await?;

        // Note: raw_html inclusion is controlled by the caller setting it on the result
        // The HTML is already captured here and can be passed through
        extracted.raw_html = Some(html);

        Ok(extracted)
    }

    /// Extract content from HTML with options
    pub async fn extract_html(
        &self,
        html: &str,
        url: &str,
        options: HtmlExtractionOptions,
    ) -> Result<ExtractedData> {
        // Route to specific strategy if requested
        let result = if let Some(selectors) = options.custom_selectors {
            // Custom selector extraction
            css_extract(html, url, &selectors)
                .await
                .map_err(|e| RiptideError::extraction(e.to_string()))?
        } else if let Some(strategy) = options.extraction_strategy {
            // Use specific requested strategy
            match strategy {
                ExtractionStrategy::HtmlCss => {
                    // Use native CSS extractor directly
                    let css_extractor = CssExtractorStrategy::new();
                    css_extractor
                        .extract(html, url)
                        .await
                        .map_err(|e| RiptideError::extraction(e.to_string()))?
                }
                ExtractionStrategy::Wasm => {
                    // Use WASM extractor if available
                    #[cfg(feature = "wasm-extractor")]
                    {
                        let wasm_extractor = StrategyWasmExtractor::new(
                            std::env::var("WASM_EXTRACTOR_PATH").ok().as_deref()
                        )
                        .await
                        .map_err(|e| RiptideError::extraction(e.to_string()))?;

                        wasm_extractor
                            .extract(html, url)
                            .await
                            .map_err(|e| RiptideError::extraction(e.to_string()))?
                    }
                    #[cfg(not(feature = "wasm-extractor"))]
                    {
                        return Err(RiptideError::extraction(
                            "WASM extractor not available - compile with 'wasm-extractor' feature"
                        ));
                    }
                }
                ExtractionStrategy::Fallback => {
                    // Use UnifiedExtractor (native-first with WASM fallback)
                    let extractor = UnifiedExtractor::new(None)
                        .await
                        .map_err(|e| RiptideError::extraction(e.to_string()))?;

                    extractor
                        .extract(html, url)
                        .await
                        .map_err(|e| RiptideError::extraction(e.to_string()))?
                }
                _ => {
                    return Err(RiptideError::extraction(format!(
                        "Strategy {:?} not implemented for HTML extraction",
                        strategy
                    )));
                }
            }
        } else {
            // Default: Use UnifiedExtractor (auto mode - native-first with WASM fallback)
            let extractor = UnifiedExtractor::new(std::env::var("WASM_EXTRACTOR_PATH").ok().as_deref())
                .await
                .map_err(|e| RiptideError::extraction(e.to_string()))?;

            extractor
                .extract(html, url)
                .await
                .map_err(|e| RiptideError::extraction(e.to_string()))?
        };

        // Extract metadata, links, and images using simple extraction
        let mut metadata = HashMap::new();
        let mut links = Vec::new();
        let mut images = Vec::new();

        if options.include_metadata || options.extract_links || options.extract_images {
            use scraper::{Html, Selector};
            let document = Html::parse_document(html);

            if options.include_metadata {
                // Extract metadata from meta tags
                if let Ok(selector) = Selector::parse("meta[name='description']") {
                    if let Some(elem) = document.select(&selector).next() {
                        if let Some(content) = elem.value().attr("content") {
                            metadata.insert("description".to_string(), content.to_string());
                        }
                    }
                }
                if let Ok(selector) = Selector::parse("meta[name='author']") {
                    if let Some(elem) = document.select(&selector).next() {
                        if let Some(content) = elem.value().attr("content") {
                            metadata.insert("author".to_string(), content.to_string());
                        }
                    }
                }
            }

            if options.extract_links {
                if let Ok(selector) = Selector::parse("a[href]") {
                    for elem in document.select(&selector) {
                        if let Some(href) = elem.value().attr("href") {
                            links.push(href.to_string());
                        }
                    }
                }
            }

            if options.extract_images {
                if let Ok(selector) = Selector::parse("img[src]") {
                    for elem in document.select(&selector) {
                        if let Some(src) = elem.value().attr("src") {
                            images.push(src.to_string());
                        }
                    }
                }
            }
        }

        // Generate markdown if requested (from original HTML, not extracted text)
        let markdown = if options.as_markdown {
            Some(self.html_to_markdown(html))
        } else {
            None
        };

        Ok(ExtractedData {
            title: Some(result.title),
            text: result.content,
            markdown,
            metadata,
            links,
            images,
            confidence: result.extraction_confidence,
            strategy_used: result.strategy_used,
            url: url.to_string(),
            raw_html: None,
        })
    }

    /// Extract content from PDF with options
    pub async fn extract_pdf(
        &self,
        bytes: &[u8],
        options: PdfExtractionOptions,
    ) -> Result<ExtractedData> {
        let pdf_config = PdfConfig {
            extract_text: options.extract_text,
            extract_images: options.extract_images,
            extract_metadata: options.extract_metadata,
            ..Default::default()
        };

        let result = self
            .pdf_processor
            .process_pdf(bytes, &pdf_config)
            .await
            .map_err(|e| RiptideError::extraction(e.to_string()))?;

        let mut metadata = HashMap::new();
        if options.extract_metadata {
            if let Some(title) = &result.metadata.title {
                metadata.insert("title".to_string(), title.clone());
            }
            if let Some(author) = &result.metadata.author {
                metadata.insert("author".to_string(), author.clone());
            }
            if let Some(subject) = &result.metadata.subject {
                metadata.insert("subject".to_string(), subject.clone());
            }
        }

        let images = if options.extract_images {
            result
                .images
                .into_iter()
                .map(|img| format!("{:?}", img.format))
                .collect()
        } else {
            Vec::new()
        };

        Ok(ExtractedData {
            title: result.metadata.title,
            text: result.text.unwrap_or_default(),
            markdown: None,
            metadata,
            links: Vec::new(),
            images,
            confidence: 0.9, // PDF extraction is high confidence
            strategy_used: "pdf_text".to_string(),
            url: String::new(),
            raw_html: None,
        })
    }

    /// Extract content with a specific strategy
    pub async fn extract_with_strategy(
        &self,
        content: &str,
        url: &str,
        strategy: ExtractionStrategy,
    ) -> Result<ExtractedData> {
        let strategy_name = strategy.name();
        let start_time = std::time::Instant::now();

        tracing::info!(
            strategy = %strategy_name,
            url = %url,
            content_size = content.len(),
            "Starting extraction with strategy"
        );

        let result = match strategy {
            ExtractionStrategy::HtmlCss => {
                let extractors = self.extractors.read().await;
                if let Some(extractor) = extractors.get_strategy("html_css") {
                    extractor
                        .extract(content, url)
                        .await
                        .map_err(|e| RiptideError::extraction(e.to_string()))?
                } else {
                    return Err(RiptideError::extraction("CSS extractor not available"));
                }
            }
            ExtractionStrategy::Wasm => {
                let extractors = self.extractors.read().await;
                if let Some(extractor) = extractors.get_strategy("wasm") {
                    extractor
                        .extract(content, url)
                        .await
                        .map_err(|e| RiptideError::extraction(e.to_string()))?
                } else {
                    return Err(RiptideError::extraction("WASM extractor not available"));
                }
            }
            ExtractionStrategy::Fallback => {
                // Use UnifiedExtractor with native-first strategy
                let extractor = UnifiedExtractor::new(None)
                    .await
                    .map_err(|e| RiptideError::extraction(e.to_string()))?;

                extractor.extract(content, url)
                    .await
                    .map_err(|e| RiptideError::extraction(e.to_string()))?
            }
            _ => {
                return Err(RiptideError::extraction(format!(
                    "Strategy {:?} not implemented",
                    strategy
                )))
            }
        };

        let duration = start_time.elapsed();

        tracing::info!(
            strategy = %strategy_name,
            confidence = result.extraction_confidence,
            duration_ms = duration.as_millis(),
            content_length = result.content.len(),
            "Strategy execution complete"
        );

        Ok(ExtractedData {
            title: Some(result.title),
            text: result.content,
            markdown: None,
            metadata: HashMap::new(),
            links: Vec::new(),
            images: Vec::new(),
            confidence: result.extraction_confidence,
            strategy_used: result.strategy_used,
            url: url.to_string(),
            raw_html: None,
        })
    }

    /// Extract content with fallback strategy chain
    pub async fn extract_with_fallback(
        &self,
        content: &str,
        url: &str,
        strategies: &[ExtractionStrategy],
    ) -> Result<ExtractedData> {
        let mut last_error = None;
        let mut best_result: Option<ExtractedData> = None;
        let mut best_confidence = 0.0;

        tracing::info!(
            url = %url,
            strategies = ?strategies.iter().map(|s| s.name()).collect::<Vec<_>>(),
            "Starting fallback chain extraction"
        );

        for (index, strategy) in strategies.iter().enumerate() {
            tracing::debug!(
                strategy = %strategy.name(),
                attempt = index + 1,
                total_strategies = strategies.len(),
                "Trying extraction strategy"
            );

            match self
                .extract_with_strategy(content, url, strategy.clone())
                .await
            {
                Ok(result) => {
                    // Keep track of best result
                    if result.confidence > best_confidence {
                        best_confidence = result.confidence;
                        best_result = Some(result.clone());
                    }

                    // If confidence is high enough, return immediately
                    if result.confidence >= 0.85 {
                        tracing::info!(
                            strategy = %strategy.name(),
                            confidence = result.confidence,
                            attempt = index + 1,
                            "High confidence result achieved, returning early"
                        );
                        return Ok(result);
                    }

                    tracing::debug!(
                        strategy = %strategy.name(),
                        confidence = result.confidence,
                        "Strategy succeeded but confidence below threshold, continuing chain"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        strategy = %strategy.name(),
                        error = %e,
                        attempt = index + 1,
                        "Strategy failed, trying next"
                    );
                    last_error = Some(e);
                }
            }
        }

        // Return best result if found
        if let Some(result) = best_result {
            tracing::info!(
                best_strategy = %result.strategy_used,
                best_confidence = best_confidence,
                strategies_tried = strategies.len(),
                "Returning best result from fallback chain"
            );
            return Ok(result);
        }

        // All strategies failed
        tracing::error!(
            strategies_tried = strategies.len(),
            "All extraction strategies failed"
        );
        Err(last_error
            .unwrap_or_else(|| RiptideError::extraction("All extraction strategies failed")))
    }

    /// Extract structured data using schema
    pub async fn extract_schema(
        &self,
        html: &str,
        _url: &str,
        schema: &Schema,
    ) -> Result<serde_json::Value> {
        use scraper::{Html, Selector};

        let document = Html::parse_document(html);
        let mut result = serde_json::Map::new();

        for (field_name, field_spec) in &schema.fields {
            let selector = Selector::parse(&field_spec.selector)
                .map_err(|e| RiptideError::extraction(format!("Invalid selector: {}", e)))?;

            let value = if let Some(element) = document.select(&selector).next() {
                let text = element
                    .text()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim()
                    .to_string();

                match field_spec.field_type {
                    FieldType::Text => serde_json::Value::String(text),
                    FieldType::Number => text
                        .parse::<f64>()
                        .map(serde_json::Value::from)
                        .unwrap_or(serde_json::Value::Null),
                    FieldType::Url => serde_json::Value::String(text),
                    FieldType::Date => serde_json::Value::String(text),
                }
            } else if field_spec.required {
                return Err(RiptideError::extraction(format!(
                    "Required field '{}' not found",
                    field_name
                )));
            } else {
                serde_json::Value::Null
            };

            result.insert(field_name.clone(), value);
        }

        Ok(serde_json::Value::Object(result))
    }

    /// Calculate confidence score for extraction
    pub fn calculate_confidence(&self, extracted: &ExtractedData) -> f64 {
        let mut score = extracted.confidence;

        // Adjust based on content quality
        if extracted.text.len() > 500 {
            score += 0.05;
        }
        if extracted.title.is_some() {
            score += 0.05;
        }
        if !extracted.metadata.is_empty() {
            score += 0.05;
        }

        score.min(1.0)
    }

    /// Convert HTML to markdown (simple implementation)
    fn html_to_markdown(&self, html: &str) -> String {
        use scraper::{Html, Selector};

        let document = Html::parse_document(html);
        let mut markdown = String::new();

        // Extract headers
        for level in 1..=6 {
            if let Ok(selector) = Selector::parse(&format!("h{}", level)) {
                for element in document.select(&selector) {
                    let text = element.text().collect::<Vec<_>>().join(" ");
                    markdown.push_str(&format!("{} {}\n\n", "#".repeat(level), text));
                }
            }
        }

        // Extract paragraphs
        if let Ok(selector) = Selector::parse("p") {
            for element in document.select(&selector) {
                let text = element.text().collect::<Vec<_>>().join(" ");
                markdown.push_str(&format!("{}\n\n", text));
            }
        }

        markdown.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> RiptideConfig {
        RiptideConfig::default()
    }

    async fn create_test_facade() -> Result<ExtractionFacade> {
        let config = create_test_config();
        ExtractionFacade::new(config).await
    }

    #[tokio::test]
    async fn test_html_extraction_clean() {
        let facade = create_test_facade().await.unwrap();
        let html = r#"
            <html>
                <head><title>Test Page</title></head>
                <body>
                    <article>
                        <h1>Main Title</h1>
                        <p>This is the main content.</p>
                    </article>
                </body>
            </html>
        "#;

        let options = HtmlExtractionOptions {
            clean: true,
            include_metadata: false,
            ..Default::default()
        };

        let result = facade
            .extract_html(html, "https://example.com", options)
            .await;
        if let Err(e) = &result {
            eprintln!("ERROR: {:?}", e);
        }
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        let data = result.unwrap();
        assert!(data.confidence > 0.0);
        assert!(!data.text.is_empty());
    }

    #[tokio::test]
    async fn test_html_extraction_markdown() {
        let facade = create_test_facade().await.unwrap();
        let html = r#"
            <html>
                <body>
                    <h1>Header 1</h1>
                    <p>Paragraph text</p>
                    <h2>Header 2</h2>
                </body>
            </html>
        "#;

        let options = HtmlExtractionOptions {
            as_markdown: true,
            ..Default::default()
        };

        let result = facade
            .extract_html(html, "https://example.com", options)
            .await;
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(data.markdown.is_some());
        let md = data.markdown.unwrap();
        assert!(md.contains("# Header 1"));
    }

    #[tokio::test]
    async fn test_html_extraction_with_links() {
        let facade = create_test_facade().await.unwrap();
        let html = r#"
            <html>
                <body>
                    <article>
                        <p>This page contains some links:</p>
                        <a href="https://example.com/page1">Link 1</a>
                        <a href="https://example.com/page2">Link 2</a>
                    </article>
                </body>
            </html>
        "#;

        let options = HtmlExtractionOptions {
            extract_links: true,
            ..Default::default()
        };

        let result = facade
            .extract_html(html, "https://example.com", options)
            .await;
        if let Err(e) = &result {
            eprintln!("ERROR: {:?}", e);
        }
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        let data = result.unwrap();
        assert!(!data.links.is_empty());
    }

    #[tokio::test]
    async fn test_strategy_fallback() {
        let facade = create_test_facade().await.unwrap();
        let html = r#"
            <html>
                <body>
                    <article>
                        <h1>Title</h1>
                        <p>Content here</p>
                    </article>
                </body>
            </html>
        "#;

        let strategies = vec![
            ExtractionStrategy::Wasm,
            ExtractionStrategy::HtmlCss,
            ExtractionStrategy::Fallback,
        ];

        let result = facade
            .extract_with_fallback(html, "https://example.com", &strategies)
            .await;
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(data.confidence > 0.0);
    }

    #[tokio::test]
    async fn test_schema_extraction() {
        let facade = create_test_facade().await.unwrap();
        let html = r#"
            <html>
                <body>
                    <h1 class="title">Product Name</h1>
                    <span class="price">29.99</span>
                    <p class="description">Product description here</p>
                </body>
            </html>
        "#;

        let mut schema = Schema {
            fields: HashMap::new(),
        };
        schema.fields.insert(
            "title".to_string(),
            FieldSpec {
                selector: ".title".to_string(),
                required: true,
                field_type: FieldType::Text,
            },
        );
        schema.fields.insert(
            "price".to_string(),
            FieldSpec {
                selector: ".price".to_string(),
                required: true,
                field_type: FieldType::Number,
            },
        );

        let result = facade
            .extract_schema(html, "https://example.com", &schema)
            .await;
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(data.is_object());
        assert!(data.get("title").is_some());
    }

    #[tokio::test]
    async fn test_confidence_scoring() {
        let facade = create_test_facade().await.unwrap();

        let mut data = ExtractedData {
            title: Some("Test".to_string()),
            text: "Short".to_string(),
            markdown: None,
            metadata: HashMap::new(),
            links: Vec::new(),
            images: Vec::new(),
            confidence: 0.8,
            strategy_used: "test".to_string(),
            url: "https://example.com".to_string(),
            raw_html: None,
        };

        let score1 = facade.calculate_confidence(&data);
        assert!(score1 >= 0.8);

        // Add more content
        data.text = "a".repeat(1000);
        data.metadata
            .insert("author".to_string(), "Test".to_string());
        let score2 = facade.calculate_confidence(&data);
        assert!(score2 > score1);
    }
}
