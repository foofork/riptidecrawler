//! ExtractionFacade - URL-based content extraction with business logic
//!
//! This module extends the existing extraction facade with URL-based extraction
//! capabilities, moving business logic from handlers into the facade layer.
//!
//! Sprint 1.2 Part A: Extract business logic from extract handler

use crate::config::RiptideConfig;
use crate::error::RiptideError;
use crate::workflows::backpressure::BackpressureManager;
use riptide_extraction::ContentExtractor;
use riptide_types::ExtractedContent;
use std::collections::HashMap;
use std::sync::Arc;

/// Result type alias for extraction operations
pub type Result<T> = std::result::Result<T, RiptideError>;

/// Options for URL-based extraction
#[derive(Debug, Clone, Default)]
pub struct UrlExtractionOptions {
    /// Extraction strategy to use
    pub strategy: String,
    /// Minimum quality threshold (0.0-1.0)
    pub quality_threshold: f64,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
    /// Extract as markdown
    pub as_markdown: bool,
    /// Include metadata
    pub include_metadata: bool,
}

/// Extracted document with metadata
#[derive(Debug, Clone)]
pub struct ExtractedDoc {
    /// Source URL
    pub url: String,
    /// Document title
    pub title: Option<String>,
    /// Extracted text content
    pub content: String,
    /// Markdown representation
    pub markdown: Option<String>,
    /// Document metadata
    pub metadata: HashMap<String, String>,
    /// Strategy used for extraction
    pub strategy_used: String,
    /// Quality/confidence score (0.0-1.0)
    pub confidence: f64,
    /// Whether quality gates passed
    pub quality_passed: bool,
}

/// Facade for URL-based content extraction with HTTP fetching and quality gates
pub struct UrlExtractionFacade {
    http_client: Arc<reqwest::Client>,
    extractor: Arc<dyn ContentExtractor>,
    gate_hi_threshold: f64,
    gate_lo_threshold: f64,
    timeout: std::time::Duration,
    backpressure: BackpressureManager,
}

impl UrlExtractionFacade {
    /// Create a new URL extraction facade
    pub async fn new(
        http_client: Arc<reqwest::Client>,
        extractor: Arc<dyn ContentExtractor>,
        config: RiptideConfig,
    ) -> Result<Self> {
        // Extract gate thresholds from config or use defaults
        let gate_hi_threshold = 0.7; // Default high threshold
        let gate_lo_threshold = 0.3; // Default low threshold
        let timeout = config.timeout;

        // Create backpressure manager with reasonable concurrency limit
        let backpressure = BackpressureManager::new(50); // Max 50 concurrent extractions

        Ok(Self {
            http_client,
            extractor,
            gate_hi_threshold,
            gate_lo_threshold,
            timeout,
            backpressure,
        })
    }

    /// Create with custom thresholds
    pub fn with_thresholds(
        http_client: Arc<reqwest::Client>,
        extractor: Arc<dyn ContentExtractor>,
        gate_hi_threshold: f64,
        gate_lo_threshold: f64,
        timeout: std::time::Duration,
    ) -> Self {
        Self {
            http_client,
            extractor,
            gate_hi_threshold,
            gate_lo_threshold,
            timeout,
            backpressure: BackpressureManager::new(50),
        }
    }

    /// Extract content from URL (all business logic)
    ///
    /// This method encapsulates:
    /// 1. Backpressure control (limit concurrent extractions)
    /// 2. URL validation (domain-level checks)
    /// 3. HTTP fetching with error handling
    /// 4. Content extraction with strategies
    /// 5. Quality gate application
    /// 6. Return structured ExtractedDoc
    pub async fn extract_from_url(
        &self,
        url: &str,
        options: UrlExtractionOptions,
    ) -> Result<ExtractedDoc> {
        // 1. Acquire backpressure permit
        let _guard = self.backpressure.acquire().await?;

        tracing::debug!(
            url = %url,
            active = self.backpressure.active_operations(),
            load = %format!("{:.1}%", self.backpressure.current_load() * 100.0),
            "Acquired extraction permit"
        );

        // 2. Validate URL (domain-level, not just format)
        self.validate_url(url)?;

        tracing::info!(
            url = %url,
            strategy = %options.strategy,
            quality_threshold = options.quality_threshold,
            "Starting URL-based extraction"
        );

        // 3. Fetch HTML with HTTP client + error handling
        let html = self.fetch_html(url).await?;

        // 4. Apply extraction strategies
        let extracted = self.extract_with_strategies(&html, url, &options).await?;

        // 5. Apply quality gates
        let quality_passed =
            self.apply_quality_gates(extracted.extraction_confidence, options.quality_threshold);

        // 6. Return ExtractedDoc (guard automatically released)
        Ok(ExtractedDoc {
            url: url.to_string(),
            title: Some(extracted.title.clone()),
            content: extracted.content.clone(),
            markdown: if options.as_markdown {
                Some(self.html_to_markdown(&html))
            } else {
                None
            },
            metadata: if options.include_metadata {
                self.extract_metadata(&html)
            } else {
                HashMap::new()
            },
            strategy_used: extracted.strategy_used.clone(),
            confidence: extracted.extraction_confidence,
            quality_passed,
        })
    }

    /// Extract from raw HTML (for cases where HTML is already fetched)
    pub async fn extract_from_html(
        &self,
        url: &str,
        html: &str,
        options: UrlExtractionOptions,
    ) -> Result<ExtractedDoc> {
        // Acquire backpressure permit
        let _guard = self.backpressure.acquire().await?;

        // Validate URL
        self.validate_url(url)?;

        tracing::info!(
            url = %url,
            strategy = %options.strategy,
            html_length = html.len(),
            "Extracting from provided HTML"
        );

        // Apply extraction strategies
        let extracted = self.extract_with_strategies(html, url, &options).await?;

        // Apply quality gates
        let quality_passed =
            self.apply_quality_gates(extracted.extraction_confidence, options.quality_threshold);

        Ok(ExtractedDoc {
            url: url.to_string(),
            title: Some(extracted.title.clone()),
            content: extracted.content.clone(),
            markdown: if options.as_markdown {
                Some(self.html_to_markdown(html))
            } else {
                None
            },
            metadata: if options.include_metadata {
                self.extract_metadata(html)
            } else {
                HashMap::new()
            },
            strategy_used: extracted.strategy_used.clone(),
            confidence: extracted.extraction_confidence,
            quality_passed,
        })
    }

    // ============================================================================
    // Private Helper Methods
    // ============================================================================

    /// Validate URL format and domain-level checks
    fn validate_url(&self, url: &str) -> Result<()> {
        // Parse URL to validate format
        let parsed = url::Url::parse(url)
            .map_err(|e| RiptideError::extraction(format!("Invalid URL format: {}", e)))?;

        // Check for valid scheme (http/https only)
        if parsed.scheme() != "http" && parsed.scheme() != "https" {
            return Err(RiptideError::extraction(format!(
                "Invalid URL scheme: {}. Only http and https are supported",
                parsed.scheme()
            )));
        }

        // Check for host
        if parsed.host().is_none() {
            return Err(RiptideError::extraction(
                "URL must have a valid host".to_string(),
            ));
        }

        tracing::debug!(
            url = %url,
            host = %parsed.host().unwrap(),
            "URL validation passed"
        );

        Ok(())
    }

    /// Fetch HTML content from URL with error handling
    async fn fetch_html(&self, url: &str) -> Result<String> {
        tracing::debug!(url = %url, "Fetching HTML content");

        let response = tokio::time::timeout(self.timeout, self.http_client.get(url).send())
            .await
            .map_err(|_| {
                RiptideError::extraction(format!("Request timeout after {:?}", self.timeout))
            })?
            .map_err(|e| RiptideError::extraction(format!("HTTP request failed: {}", e)))?;

        // Check HTTP status
        let status = response.status();
        if !status.is_success() {
            tracing::warn!(
                url = %url,
                status = %status,
                "HTTP request returned non-success status"
            );
            return Err(RiptideError::extraction(format!(
                "Server returned status: {}",
                status
            )));
        }

        // Extract text body
        let html = response.text().await.map_err(|e| {
            RiptideError::extraction(format!("Failed to read response body: {}", e))
        })?;

        tracing::debug!(
            url = %url,
            html_length = html.len(),
            "HTML fetched successfully"
        );

        Ok(html)
    }

    /// Extract content using configured strategies
    async fn extract_with_strategies(
        &self,
        html: &str,
        url: &str,
        _options: &UrlExtractionOptions,
    ) -> Result<ExtractedContent> {
        tracing::debug!(
            url = %url,
            html_length = html.len(),
            "Applying extraction strategies"
        );

        // Use the provided extractor (could be WASM, CSS, or fallback)
        let result = self
            .extractor
            .extract(html, url)
            .await
            .map_err(|e| RiptideError::extraction(format!("Extraction failed: {}", e)))?;

        tracing::info!(
            url = %url,
            strategy = %result.strategy_used,
            confidence = result.extraction_confidence,
            content_length = result.content.len(),
            "Extraction completed"
        );

        Ok(result)
    }

    /// Apply quality gates to extraction result
    fn apply_quality_gates(&self, confidence: f64, user_threshold: f64) -> bool {
        // Use user-provided threshold if higher than system low threshold
        let effective_threshold = user_threshold.max(self.gate_lo_threshold);

        let passed = confidence >= effective_threshold;

        tracing::debug!(
            confidence = confidence,
            threshold = effective_threshold,
            hi_threshold = self.gate_hi_threshold,
            lo_threshold = self.gate_lo_threshold,
            passed = passed,
            "Quality gate evaluation"
        );

        if !passed {
            tracing::warn!(
                confidence = confidence,
                threshold = effective_threshold,
                "Quality gate failed - confidence below threshold"
            );
        }

        passed
    }

    /// Extract metadata from HTML
    fn extract_metadata(&self, html: &str) -> HashMap<String, String> {
        use scraper::{Html, Selector};

        let document = Html::parse_document(html);
        let mut metadata = HashMap::new();

        // Extract common metadata tags
        let meta_tags = vec![
            ("description", "meta[name='description']"),
            ("author", "meta[name='author']"),
            ("keywords", "meta[name='keywords']"),
            ("language", "meta[http-equiv='content-language']"),
            ("og:title", "meta[property='og:title']"),
            ("og:description", "meta[property='og:description']"),
            ("twitter:title", "meta[name='twitter:title']"),
        ];

        for (key, selector_str) in meta_tags {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(elem) = document.select(&selector).next() {
                    if let Some(content) = elem.value().attr("content") {
                        metadata.insert(key.to_string(), content.to_string());
                    }
                }
            }
        }

        tracing::debug!(
            metadata_count = metadata.len(),
            "Extracted metadata from HTML"
        );

        metadata
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
                    markdown.push_str(&format!("{} {}\n\n", "#".repeat(level), text.trim()));
                }
            }
        }

        // Extract paragraphs
        if let Ok(selector) = Selector::parse("p") {
            for element in document.select(&selector) {
                let text = element.text().collect::<Vec<_>>().join(" ");
                if !text.trim().is_empty() {
                    markdown.push_str(&format!("{}\n\n", text.trim()));
                }
            }
        }

        // Extract lists
        if let Ok(selector) = Selector::parse("ul li, ol li") {
            for element in document.select(&selector) {
                let text = element.text().collect::<Vec<_>>().join(" ");
                if !text.trim().is_empty() {
                    markdown.push_str(&format!("- {}\n", text.trim()));
                }
            }
        }

        markdown.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riptide_extraction::fallback_extract;

    // Mock extractor for testing
    struct MockExtractor;

    #[async_trait::async_trait]
    impl ContentExtractor for MockExtractor {
        async fn extract(&self, html: &str, url: &str) -> anyhow::Result<ExtractedContent> {
            // Use fallback_extract for testing
            fallback_extract(html, url).await
        }

        fn confidence_score(&self, _html: &str) -> f64 {
            0.8
        }

        fn strategy_name(&self) -> &'static str {
            "mock"
        }
    }

    fn create_test_facade() -> UrlExtractionFacade {
        let http_client = Arc::new(reqwest::Client::new());
        let extractor: Arc<dyn ContentExtractor> = Arc::new(MockExtractor);

        UrlExtractionFacade::with_thresholds(
            http_client,
            extractor,
            0.7,
            0.3,
            std::time::Duration::from_secs(30),
        )
    }

    #[test]
    fn test_url_validation_valid() {
        let facade = create_test_facade();

        assert!(facade.validate_url("https://example.com").is_ok());
        assert!(facade.validate_url("http://example.com/path").is_ok());
    }

    #[test]
    fn test_url_validation_invalid_scheme() {
        let facade = create_test_facade();

        assert!(facade.validate_url("ftp://example.com").is_err());
        assert!(facade.validate_url("file:///etc/passwd").is_err());
    }

    #[test]
    fn test_url_validation_invalid_format() {
        let facade = create_test_facade();

        assert!(facade.validate_url("not a url").is_err());
        assert!(facade.validate_url("://example.com").is_err());
    }

    #[test]
    fn test_quality_gates_pass() {
        let facade = create_test_facade();

        // High confidence should pass
        assert!(facade.apply_quality_gates(0.9, 0.7));
        assert!(facade.apply_quality_gates(0.8, 0.7));
    }

    #[test]
    fn test_quality_gates_fail() {
        let facade = create_test_facade();

        // Low confidence should fail
        assert!(!facade.apply_quality_gates(0.2, 0.7));
        assert!(!facade.apply_quality_gates(0.5, 0.7));
    }

    #[test]
    fn test_quality_gates_threshold() {
        let facade = create_test_facade();

        // Exactly at threshold
        assert!(facade.apply_quality_gates(0.7, 0.7));

        // Just below threshold
        assert!(!facade.apply_quality_gates(0.69, 0.7));
    }

    #[tokio::test]
    async fn test_extract_from_html() {
        let facade = create_test_facade();

        let html = r#"
            <html>
                <head>
                    <title>Test Page</title>
                    <meta name="description" content="Test description">
                    <meta name="author" content="Test Author">
                </head>
                <body>
                    <article>
                        <h1>Main Title</h1>
                        <p>This is the main content.</p>
                    </article>
                </body>
            </html>
        "#;

        let options = UrlExtractionOptions {
            strategy: "multi".to_string(),
            quality_threshold: 0.5,
            timeout_ms: 30000,
            as_markdown: true,
            include_metadata: true,
        };

        let result = facade
            .extract_from_html("https://example.com", html, options)
            .await;

        assert!(result.is_ok());
        let doc = result.unwrap();

        assert_eq!(doc.url, "https://example.com");
        assert!(doc.title.is_some());
        assert!(!doc.content.is_empty());
        assert!(doc.markdown.is_some());
        assert!(!doc.metadata.is_empty());
    }

    #[test]
    fn test_extract_metadata() {
        let facade = create_test_facade();

        let html = r#"
            <html>
                <head>
                    <meta name="description" content="Test description">
                    <meta name="author" content="John Doe">
                    <meta property="og:title" content="OG Title">
                </head>
            </html>
        "#;

        let metadata = facade.extract_metadata(html);

        assert_eq!(
            metadata.get("description"),
            Some(&"Test description".to_string())
        );
        assert_eq!(metadata.get("author"), Some(&"John Doe".to_string()));
        assert_eq!(metadata.get("og:title"), Some(&"OG Title".to_string()));
    }

    #[test]
    fn test_html_to_markdown() {
        let facade = create_test_facade();

        let html = r#"
            <html>
                <body>
                    <h1>Header 1</h1>
                    <p>Paragraph text</p>
                    <h2>Header 2</h2>
                    <ul>
                        <li>Item 1</li>
                        <li>Item 2</li>
                    </ul>
                </body>
            </html>
        "#;

        let markdown = facade.html_to_markdown(html);

        assert!(markdown.contains("# Header 1"));
        assert!(markdown.contains("## Header 2"));
        assert!(markdown.contains("Paragraph text"));
        assert!(markdown.contains("- Item 1"));
        assert!(markdown.contains("- Item 2"));
    }

    #[test]
    fn test_default_options() {
        let options = UrlExtractionOptions::default();

        assert_eq!(options.strategy, "");
        assert_eq!(options.quality_threshold, 0.0);
        assert_eq!(options.timeout_ms, 0);
        assert!(!options.as_markdown);
        assert!(!options.include_metadata);
    }
}
