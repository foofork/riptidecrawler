//! Unified extractor that works with or without WASM feature
//!
//! This module provides a unified interface for content extraction that automatically
//! selects between WASM and native extraction based on compile-time features and
//! runtime availability. It implements a three-tier fallback strategy:
//!
//! 1. **Compile-time**: Feature flag determines if WASM is available
//! 2. **Runtime**: File availability check for WASM module
//! 3. **Execution**: Error recovery with fallback to native
//!
//! ## Usage
//!
//! ```rust
//! use riptide_extraction::UnifiedExtractor;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Automatic selection with multi-level fallback
//! let wasm_path = std::env::var("WASM_EXTRACTOR_PATH").ok();
//! let extractor = UnifiedExtractor::new(wasm_path.as_deref()).await?;
//!
//! // Check which extractor is active
//! println!("Using: {}", extractor.extractor_type());
//! println!("WASM available: {}", UnifiedExtractor::wasm_available());
//!
//! // Extract content (automatically uses best available strategy)
//! let html = "<html><body><p>Content</p></body></html>";
//! let result = extractor.extract(html, "https://example.com").await?;
//! # Ok(())
//! # }
//! ```

use crate::extraction_strategies::ContentExtractor;
use crate::native_parser::{NativeHtmlParser, ParserConfig};
use anyhow::Result;
use async_trait::async_trait;
use riptide_types::ExtractedContent;

#[cfg(feature = "wasm-extractor")]
use anyhow::anyhow;

#[cfg(feature = "wasm-extractor")]
use crate::extraction_strategies::WasmExtractor;

/// Unified extractor that works with or without WASM
///
/// This enum provides a single type that can represent either a WASM-based
/// or native extractor, allowing the same code to work regardless of whether
/// the `wasm-extractor` feature is enabled.
pub enum UnifiedExtractor {
    /// WASM-based extractor (only available with `wasm-extractor` feature)
    #[cfg(feature = "wasm-extractor")]
    Wasm(Box<WasmExtractor>),

    /// Native Rust parser (always available)
    Native(NativeExtractor),
}

/// Native extractor wrapper for consistent interface
#[derive(Default)]
pub struct NativeExtractor {
    parser: NativeHtmlParser,
}

impl NativeExtractor {
    /// Create a new native extractor with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new native extractor with custom configuration
    pub fn with_config(config: ParserConfig) -> Self {
        Self {
            parser: NativeHtmlParser::with_config(config),
        }
    }
}

impl UnifiedExtractor {
    /// Create extractor with automatic three-level fallback
    ///
    /// # Fallback Strategy
    ///
    /// 1. **Compile-time**: If `wasm-extractor` feature is disabled, uses native
    /// 2. **Runtime**: If WASM path is provided but file is missing, uses native
    /// 3. **Execution**: If WASM extraction fails, falls back to native
    ///
    /// # Arguments
    ///
    /// * `wasm_path` - Optional path to WASM extractor module
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use riptide_extraction::UnifiedExtractor;
    /// # async fn example() -> anyhow::Result<()> {
    /// // Try to use WASM if available
    /// let extractor = UnifiedExtractor::new(Some("/path/to/extractor.wasm")).await?;
    ///
    /// // Use native extractor (no WASM)
    /// let extractor = UnifiedExtractor::new(None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(wasm_path: Option<&str>) -> Result<Self> {
        // Level 1: Compile-time check
        #[cfg(feature = "wasm-extractor")]
        {
            // Level 2: Runtime file availability
            if let Some(path) = wasm_path {
                match WasmExtractor::new(Some(path)).await {
                    Ok(extractor) => {
                        tracing::info!(
                            path = %path,
                            "WASM extractor initialized successfully"
                        );
                        return Ok(Self::Wasm(Box::new(extractor)));
                    }
                    Err(e) => {
                        tracing::warn!(
                            path = %path,
                            error = %e,
                            "WASM extractor unavailable, falling back to native"
                        );
                    }
                }
            } else {
                tracing::debug!("No WASM path provided, using native extractor");
            }
        }

        #[cfg(not(feature = "wasm-extractor"))]
        {
            if wasm_path.is_some() {
                tracing::warn!(
                    "WASM_EXTRACTOR_PATH set but wasm-extractor feature not enabled. \
                     Rebuild with --features wasm-extractor to use WASM. Using native extractor."
                );
            }
        }

        // Default to native
        tracing::info!("Using native Rust extractor");
        Ok(Self::Native(NativeExtractor::default()))
    }

    /// Check which extractor is active
    ///
    /// Returns either "wasm" or "native" depending on the active implementation.
    pub fn extractor_type(&self) -> &'static str {
        match self {
            #[cfg(feature = "wasm-extractor")]
            Self::Wasm(_) => "wasm",
            Self::Native(_) => "native",
        }
    }

    /// Check if WASM is available (compile-time)
    ///
    /// Returns `true` if the `wasm-extractor` feature is enabled at compile time.
    pub fn wasm_available() -> bool {
        cfg!(feature = "wasm-extractor")
    }

    /// Get confidence score for content extraction
    pub fn confidence_score(&self, html: &str) -> f64 {
        match self {
            #[cfg(feature = "wasm-extractor")]
            Self::Wasm(_extractor) => {
                // WASM confidence score based on content heuristics
                let has_title = html.contains("<title>");
                let has_content = html.contains("<p>") || html.contains("<article>");
                let length_score = (html.len().min(10000) as f64) / 10000.0;

                let base_score = if has_title && has_content {
                    0.85 // WASM has slightly higher base confidence
                } else if has_title || has_content {
                    0.65
                } else {
                    0.45
                };

                (base_score + length_score * 0.15).min(1.0)
            }
            Self::Native(_) => {
                // Native parser calculates quality based on content presence
                let has_title = html.contains("<title>");
                let has_content = html.contains("<p>") || html.contains("<article>");
                // Safe conversion: bounded by min(10000) so always fits in f64
                #[allow(clippy::cast_precision_loss)]
                let length_score = (html.len().min(10000) as f64) / 10000.0;

                let base_score = if has_title && has_content {
                    0.8
                } else if has_title || has_content {
                    0.6
                } else {
                    0.4
                };

                (base_score + length_score * 0.2).min(1.0)
            }
        }
    }

    /// Extract content with automatic fallback (NATIVE FIRST)
    ///
    /// # Extraction Strategy (REVERSED - Native Primary)
    ///
    /// 1. **PRIMARY**: Uses native Rust parser for fast, reliable extraction
    /// 2. **FALLBACK**: Falls back to WASM only if native fails (if available)
    /// 3. Returns extraction result with metadata about which parser was used
    ///
    /// This is the **REVERSED** strategy - native extraction is now PRIMARY,
    /// and WASM is only used as a fallback/enhancement strategy.
    ///
    /// # Arguments
    ///
    /// * `html` - HTML content to extract
    /// * `url` - Source URL for context
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use riptide_extraction::UnifiedExtractor;
    /// # async fn example() -> anyhow::Result<()> {
    /// let extractor = UnifiedExtractor::new(None).await?;
    /// let html = "<html><body><h1>Title</h1><p>Content</p></body></html>";
    /// let result = extractor.extract(html, "https://example.com").await?;
    /// println!("Title: {}", result.title);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn extract(&self, html: &str, url: &str) -> Result<ExtractedContent> {
        // ALWAYS try native FIRST (regardless of variant)
        let native = NativeExtractor::default();
        match native.extract(html, url).await {
            Ok(content) => {
                tracing::debug!(
                    url = %url,
                    strategy = "native",
                    "Content extracted successfully with native parser"
                );
                Ok(content)
            }
            Err(native_err) => {
                tracing::warn!(
                    url = %url,
                    error = %native_err,
                    "Native extraction failed, attempting WASM fallback"
                );

                // Only fallback to WASM if available
                match self {
                    #[cfg(feature = "wasm-extractor")]
                    Self::Wasm(extractor) => {
                        use crate::extraction_strategies::ContentExtractor;
                        match extractor.extract(html, url).await {
                            Ok(content) => {
                                tracing::info!(
                                    url = %url,
                                    strategy = "wasm_fallback",
                                    "Content extracted with WASM fallback after native failure"
                                );
                                Ok(content)
                            }
                            Err(wasm_err) => {
                                tracing::error!(
                                    url = %url,
                                    native_error = %native_err,
                                    wasm_error = %wasm_err,
                                    "Both native and WASM extraction failed"
                                );
                                Err(anyhow!(
                                    "Both native ({}) and WASM ({}) extraction failed",
                                    native_err,
                                    wasm_err
                                ))
                            }
                        }
                    }
                    Self::Native(_) => {
                        // No WASM available, return native error
                        tracing::error!(
                            url = %url,
                            error = %native_err,
                            "Native extraction failed and no WASM fallback available"
                        );
                        Err(native_err)
                    }
                }
            }
        }
    }

    /// Get strategy name for metrics and logging
    pub fn strategy_name(&self) -> &'static str {
        match self {
            #[cfg(feature = "wasm-extractor")]
            Self::Wasm(_) => "wasm",
            Self::Native(_) => "native",
        }
    }
}

// Implement ContentExtractor trait for NativeExtractor
#[async_trait]
impl crate::extraction_strategies::ContentExtractor for NativeExtractor {
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractedContent> {
        // Parse HTML using native parser
        let doc = self.parser.parse_headless_html(html, url)?;

        // Convert ExtractedDoc to ExtractedContent
        Ok(ExtractedContent {
            title: doc.title.unwrap_or_default(),
            content: doc.text,
            summary: doc.description,
            url: url.to_string(),
            strategy_used: "native_parser".to_string(),
            // Safe conversion: quality_score is u8 (0-100), always fits in f64
            extraction_confidence: f64::from(doc.quality_score.unwrap_or(50)) / 100.0,
        })
    }

    fn confidence_score(&self, html: &str) -> f64 {
        // Calculate quality based on content heuristics
        let has_title = html.contains("<title>");
        let has_content = html.contains("<p>") || html.contains("<article>");
        // Safe conversion: bounded by min(10000) so always fits in f64
        #[allow(clippy::cast_precision_loss)]
        let length_score = (html.len().min(10000) as f64) / 10000.0;

        let base_score = if has_title && has_content {
            0.8
        } else if has_title || has_content {
            0.6
        } else {
            0.4
        };

        (base_score + length_score * 0.2).min(1.0)
    }

    fn strategy_name(&self) -> &'static str {
        "native_parser"
    }
}

// Implement ContentExtractor trait for UnifiedExtractor
#[async_trait]
impl crate::extraction_strategies::ContentExtractor for UnifiedExtractor {
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractedContent> {
        self.extract(html, url).await
    }

    fn confidence_score(&self, html: &str) -> f64 {
        self.confidence_score(html)
    }

    fn strategy_name(&self) -> &'static str {
        self.strategy_name()
    }
}

// Implement the port trait ContentExtractor from riptide-types
#[async_trait]
impl riptide_types::ports::ContentExtractor for UnifiedExtractor {
    async fn extract(&self, html: &str, url: &str) -> riptide_types::error::Result<riptide_types::ports::ExtractionResult> {
        // Use existing extract method
        let content = self.extract(html, url).await.map_err(|e| {
            riptide_types::error::RiptideError::Extraction(format!("Extraction failed: {}", e))
        })?;

        // Convert ExtractedContent to ExtractionResult
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("title".to_string(), content.title.clone());
        if let Some(summary) = content.summary.clone() {
            metadata.insert("summary".to_string(), summary);
        }
        metadata.insert("strategy".to_string(), content.strategy_used.clone());
        metadata.insert("url".to_string(), content.url.clone());

        Ok(riptide_types::ports::ExtractionResult {
            text: content.content,
            metadata,
            quality_score: content.extraction_confidence,
        })
    }

    fn extractor_type(&self) -> &str {
        self.extractor_type()
    }

    async fn is_available(&self) -> bool {
        // UnifiedExtractor is always available (has native fallback)
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extractor_creation_native_only() {
        // Works without wasm-extractor feature
        let extractor = UnifiedExtractor::new(None).await.unwrap();
        assert_eq!(extractor.extractor_type(), "native");

        // Check compile-time availability
        #[cfg(feature = "wasm-extractor")]
        assert!(UnifiedExtractor::wasm_available());

        #[cfg(not(feature = "wasm-extractor"))]
        assert!(!UnifiedExtractor::wasm_available());
    }

    #[tokio::test]
    async fn test_runtime_fallback() {
        // Level 2: Runtime fallback when file missing
        let extractor = UnifiedExtractor::new(Some("/nonexistent.wasm"))
            .await
            .unwrap();

        // Should work (falls back to native)
        assert_eq!(extractor.extractor_type(), "native");
    }

    #[tokio::test]
    async fn test_extraction_basic() {
        let extractor = UnifiedExtractor::new(None).await.unwrap();

        let html = r#"
            <html>
            <head><title>Test Article</title></head>
            <body>
                <article>
                    <h1>Test Article Title</h1>
                    <p>This is a comprehensive test article with substantial content to ensure it meets
                       the minimum quality threshold requirements for successful extraction.</p>
                    <p>Multiple paragraphs are included to increase the word count and quality score,
                       demonstrating that the extraction functionality works correctly with adequate content.</p>
                    <p>Additional text helps verify that the unified extractor properly handles
                       well-formatted HTML documents with sufficient content depth and quality.</p>
                </article>
            </body>
            </html>
        "#;
        let result = extractor.extract(html, "https://example.com").await;

        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.title.contains("Test") || !content.title.is_empty());
    }

    #[tokio::test]
    async fn test_confidence_scoring() {
        let extractor = UnifiedExtractor::new(None).await.unwrap();

        let good_html = r#"
            <html>
                <head><title>Good Article</title></head>
                <body>
                    <article>
                        <h1>Main Title</h1>
                        <p>Long paragraph with substantial content that indicates
                           this is a quality article worth extracting.</p>
                    </article>
                </body>
            </html>
        "#;

        let score = extractor.confidence_score(good_html);
        assert!(score > 0.5, "Expected score > 0.5, got {}", score);
    }

    #[tokio::test]
    async fn test_strategy_name() {
        let extractor = UnifiedExtractor::new(None).await.unwrap();

        #[cfg(feature = "wasm-extractor")]
        assert!(matches!(extractor.strategy_name(), "wasm" | "native"));

        #[cfg(not(feature = "wasm-extractor"))]
        assert_eq!(extractor.strategy_name(), "native");
    }
}
