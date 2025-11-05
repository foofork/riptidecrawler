//! Extractor trait for content extraction
//!
//! Provides async trait for extracting structured data from web content.

use async_trait::async_trait;
use crate::error::RiptideResult;
use crate::dto::Document;

/// Content type for extraction
#[derive(Debug, Clone)]
pub enum Content {
    /// Raw HTML content
    Html(String),
    /// URL to fetch and extract
    Url(String),
    /// Raw text content
    Text(String),
}

impl From<String> for Content {
    fn from(s: String) -> Self {
        if s.starts_with("http://") || s.starts_with("https://") {
            Content::Url(s)
        } else if s.contains("</") {
            Content::Html(s)
        } else {
            Content::Text(s)
        }
    }
}

impl From<url::Url> for Content {
    fn from(url: url::Url) -> Self {
        Content::Url(url.to_string())
    }
}

/// Options for extraction behavior
#[derive(Debug, Clone)]
pub struct ExtractOpts {
    /// Extraction strategy to use
    pub strategy: ExtractionStrategy,
    /// CSS selector for targeted extraction
    pub selector: Option<String>,
    /// Schema for structured data extraction
    pub schema: Option<String>,
}

impl Default for ExtractOpts {
    fn default() -> Self {
        Self {
            strategy: ExtractionStrategy::Auto,
            selector: None,
            schema: None,
        }
    }
}

/// Extraction strategy selection
#[derive(Debug, Clone)]
pub enum ExtractionStrategy {
    /// Automatic strategy selection
    Auto,
    /// CSS selector-based extraction
    Css,
    /// JSON-LD structured data
    JsonLd,
    /// LLM-powered extraction
    Llm,
}

/// Extractor trait for extracting structured data
///
/// This trait provides async content extraction capabilities. It can process
/// various content types and return structured documents.
///
/// # Examples
///
/// ```no_run
/// use riptide_facade::traits::{Extractor, Content, ExtractOpts};
///
/// # async fn example(extractor: impl Extractor) -> Result<(), Box<dyn std::error::Error>> {
/// let content = Content::Url("https://example.com".to_string());
/// let doc = extractor.extract(content, ExtractOpts::default()).await?;
/// println!("Title: {}", doc.title);
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait Extractor: Send + Sync {
    /// Extract structured data from content
    ///
    /// # Arguments
    ///
    /// * `content` - Content to extract from (HTML, URL, or text)
    /// * `opts` - Configuration options for extraction behavior
    ///
    /// # Returns
    ///
    /// Document - Extracted structured data
    async fn extract(
        &self,
        content: Content,
        opts: ExtractOpts,
    ) -> RiptideResult<Document>;
}
