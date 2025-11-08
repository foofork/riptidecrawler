//! Feature ports for specialized functionality
//!
//! This module provides backend-agnostic interfaces for feature capabilities:
//! - Browser automation (headless Chrome, Firefox, etc.)
//! - PDF processing (text extraction, image extraction, rendering)
//! - Search engine operations (indexing, searching)
//!
//! # Design Goals
//!
//! - **Abstraction**: Hide implementation details of feature backends
//! - **Testability**: Enable mock implementations for testing
//! - **Flexibility**: Support multiple backend implementations
//! - **Performance**: Async-first design for concurrent operations
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_types::ports::{BrowserDriver, PdfProcessor, SearchEngine};
//!
//! async fn example(
//!     browser: &dyn BrowserDriver,
//!     pdf: &dyn PdfProcessor,
//!     search: &dyn SearchEngine,
//! ) -> Result<()> {
//!     // Browser automation
//!     let session = browser.navigate("https://example.com").await?;
//!     let screenshot = browser.screenshot(&session).await?;
//!     browser.close(session).await?;
//!
//!     // PDF processing
//!     let text = pdf.extract_text(&pdf_bytes).await?;
//!
//!     // Search indexing
//!     search.index(document).await?;
//!
//!     Ok(())
//! }
//! ```

use crate::error::Result as RiptideResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// ============================================================================
// Browser Driver Port
// ============================================================================

/// Browser session handle
///
/// Represents an active browser session with a specific page loaded.
/// Sessions should be explicitly closed to free resources.
#[derive(Debug, Clone)]
pub struct BrowserSession {
    /// Unique session identifier
    pub id: String,
    /// Current page URL
    pub url: String,
    /// Session metadata (browser type, viewport, etc.)
    pub metadata: std::collections::HashMap<String, String>,
}

impl BrowserSession {
    /// Create new browser session
    pub fn new(id: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            url: url.into(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Add metadata to session
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Result of JavaScript execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptResult {
    /// Script return value (JSON-serializable)
    pub value: serde_json::Value,
    /// Script execution success
    pub success: bool,
    /// Error message if execution failed
    pub error: Option<String>,
}

/// Browser automation port
///
/// Provides headless browser capabilities for web scraping and testing.
/// Implementations typically wrap Selenium, Playwright, or puppeteer.
#[async_trait]
pub trait BrowserDriver: Send + Sync {
    /// Navigate to URL and create browser session
    ///
    /// # Arguments
    ///
    /// * `url` - URL to navigate to
    ///
    /// # Returns
    ///
    /// * `Ok(session)` - Browser session created and page loaded
    /// * `Err(_)` - Navigation failed or browser error
    async fn navigate(&self, url: &str) -> RiptideResult<BrowserSession>;

    /// Execute JavaScript in browser context
    ///
    /// # Arguments
    ///
    /// * `session` - Active browser session
    /// * `script` - JavaScript code to execute
    ///
    /// # Returns
    ///
    /// * `Ok(result)` - Script executed with result
    /// * `Err(_)` - Execution failed or browser error
    async fn execute_script(
        &self,
        session: &BrowserSession,
        script: &str,
    ) -> RiptideResult<ScriptResult>;

    /// Capture screenshot of current page
    ///
    /// # Arguments
    ///
    /// * `session` - Active browser session
    ///
    /// # Returns
    ///
    /// * `Ok(bytes)` - PNG image data
    /// * `Err(_)` - Screenshot failed or browser error
    async fn screenshot(&self, session: &BrowserSession) -> RiptideResult<Vec<u8>>;

    /// Close browser session and free resources
    ///
    /// # Arguments
    ///
    /// * `session` - Browser session to close
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Session closed successfully
    /// * `Err(_)` - Close failed (session may still be terminated)
    async fn close(&self, session: BrowserSession) -> RiptideResult<()>;

    /// Get page HTML content
    ///
    /// # Arguments
    ///
    /// * `session` - Active browser session
    ///
    /// # Returns
    ///
    /// * `Ok(html)` - Page HTML source
    /// * `Err(_)` - Failed to retrieve HTML
    async fn get_html(&self, session: &BrowserSession) -> RiptideResult<String> {
        let script = "document.documentElement.outerHTML";
        let result = self.execute_script(session, script).await?;
        if result.success {
            Ok(result.value.as_str().unwrap_or("").to_string())
        } else {
            Err(crate::error::RiptideError::BrowserOperation(
                result.error.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }

    /// Wait for element to appear on page
    ///
    /// # Arguments
    ///
    /// * `session` - Active browser session
    /// * `selector` - CSS selector to wait for
    /// * `timeout` - Maximum wait duration
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Element appeared within timeout
    /// * `Err(_)` - Timeout or browser error
    async fn wait_for_element(
        &self,
        session: &BrowserSession,
        selector: &str,
        timeout: std::time::Duration,
    ) -> RiptideResult<()> {
        let _ = (session, selector, timeout);
        // Default implementation - backends should override
        Ok(())
    }
}

// ============================================================================
// PDF Processor Port
// ============================================================================

/// PDF processing port
///
/// Provides PDF manipulation capabilities including text extraction,
/// image extraction, and page rendering.
#[async_trait]
pub trait PdfProcessor: Send + Sync {
    /// Extract text content from PDF
    ///
    /// # Arguments
    ///
    /// * `pdf_data` - PDF file binary data
    ///
    /// # Returns
    ///
    /// * `Ok(text)` - Extracted text content
    /// * `Err(_)` - Extraction failed or invalid PDF
    async fn extract_text(&self, pdf_data: &[u8]) -> RiptideResult<String>;

    /// Extract images from PDF
    ///
    /// # Arguments
    ///
    /// * `pdf_data` - PDF file binary data
    ///
    /// # Returns
    ///
    /// * `Ok(images)` - Vector of image data (PNG format)
    /// * `Err(_)` - Extraction failed or invalid PDF
    async fn extract_images(&self, pdf_data: &[u8]) -> RiptideResult<Vec<Vec<u8>>>;

    /// Render PDF page to image
    ///
    /// # Arguments
    ///
    /// * `pdf_data` - PDF file binary data
    /// * `page` - Page number (0-indexed)
    ///
    /// # Returns
    ///
    /// * `Ok(image)` - Rendered page as PNG image data
    /// * `Err(_)` - Rendering failed or invalid page number
    async fn render_page(&self, pdf_data: &[u8], page: usize) -> RiptideResult<Vec<u8>>;

    /// Get PDF metadata
    ///
    /// # Arguments
    ///
    /// * `pdf_data` - PDF file binary data
    ///
    /// # Returns
    ///
    /// * `Ok(metadata)` - PDF metadata (page count, title, author, etc.)
    /// * `Err(_)` - Failed to read metadata
    async fn get_metadata(&self, pdf_data: &[u8]) -> RiptideResult<PdfMetadata> {
        let _ = pdf_data;
        // Default implementation - backends should override
        Ok(PdfMetadata::default())
    }
}

/// PDF document metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PdfMetadata {
    /// Number of pages
    pub page_count: usize,
    /// Document title
    pub title: Option<String>,
    /// Document author
    pub author: Option<String>,
    /// Creation date
    pub created_at: Option<String>,
    /// PDF version
    pub pdf_version: Option<String>,
}

// ============================================================================
// Search Engine Port
// ============================================================================

/// Search document to be indexed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocument {
    /// Document unique identifier
    pub id: String,
    /// Document title
    pub title: String,
    /// Document content/body
    pub content: String,
    /// Additional searchable fields
    pub fields: std::collections::HashMap<String, serde_json::Value>,
    /// Document metadata (non-searchable)
    pub metadata: std::collections::HashMap<String, String>,
}

impl SearchDocument {
    /// Create new search document
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            fields: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Add searchable field
    pub fn with_field(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.fields.insert(key.into(), value);
        self
    }

    /// Add metadata field
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Search query parameters
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    /// Query string
    pub query: String,
    /// Fields to search in
    pub fields: Vec<String>,
    /// Result limit
    pub limit: Option<usize>,
    /// Result offset
    pub offset: Option<usize>,
    /// Filters to apply
    pub filters: std::collections::HashMap<String, serde_json::Value>,
}

impl SearchQuery {
    /// Create new search query
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            ..Default::default()
        }
    }

    /// Add field to search
    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.fields.push(field.into());
        self
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Add filter
    pub fn with_filter(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.filters.insert(key.into(), value);
        self
    }
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Document ID
    pub id: String,
    /// Relevance score
    pub score: f64,
    /// Document title
    pub title: String,
    /// Document content snippet
    pub snippet: String,
    /// Matched fields
    pub fields: std::collections::HashMap<String, serde_json::Value>,
}

/// Search engine port
///
/// Provides full-text search capabilities with indexing and querying.
/// Implementations typically wrap Elasticsearch, MeiliSearch, or similar.
#[async_trait]
pub trait SearchEngine: Send + Sync {
    /// Index document for searching
    ///
    /// # Arguments
    ///
    /// * `document` - Document to index
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Document indexed successfully
    /// * `Err(_)` - Indexing failed
    async fn index(&self, document: SearchDocument) -> RiptideResult<()>;

    /// Search indexed documents
    ///
    /// # Arguments
    ///
    /// * `query` - Search query parameters
    ///
    /// # Returns
    ///
    /// * `Ok(results)` - Matching documents with relevance scores
    /// * `Err(_)` - Search failed
    async fn search(&self, query: SearchQuery) -> RiptideResult<Vec<SearchResult>>;

    /// Delete document from index
    ///
    /// # Arguments
    ///
    /// * `id` - Document ID to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Document deleted (or didn't exist)
    /// * `Err(_)` - Deletion failed
    async fn delete(&self, id: &str) -> RiptideResult<()>;

    /// Index multiple documents in batch
    ///
    /// # Arguments
    ///
    /// * `documents` - Documents to index
    ///
    /// # Returns
    ///
    /// * `Ok(())` - All documents indexed successfully
    /// * `Err(_)` - Batch indexing failed (partial success possible)
    async fn index_batch(&self, documents: Vec<SearchDocument>) -> RiptideResult<()> {
        // Default implementation - sequential indexing
        for doc in documents {
            self.index(doc).await?;
        }
        Ok(())
    }

    /// Clear all indexed documents
    ///
    /// **Warning**: Destructive operation. Use with caution.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Index cleared successfully
    /// * `Err(_)` - Clear failed
    async fn clear(&self) -> RiptideResult<()> {
        // Default implementation - not supported
        Err(crate::error::RiptideError::Custom(
            "Clear operation not supported".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_session_creation() {
        let session = BrowserSession::new("session-123", "https://example.com")
            .with_metadata("browser", "chrome");

        assert_eq!(session.id, "session-123");
        assert_eq!(session.url, "https://example.com");
        assert_eq!(session.metadata.get("browser"), Some(&"chrome".to_string()));
    }

    #[test]
    fn test_search_document_builder() {
        let doc = SearchDocument::new("doc-1", "Test Title", "Test content")
            .with_field("category", serde_json::json!("test"))
            .with_metadata("author", "test-user");

        assert_eq!(doc.id, "doc-1");
        assert_eq!(doc.fields.len(), 1);
        assert_eq!(doc.metadata.len(), 1);
    }

    #[test]
    fn test_search_query_builder() {
        let query = SearchQuery::new("test query")
            .with_field("title")
            .with_field("content")
            .with_limit(10)
            .with_filter("status", serde_json::json!("published"));

        assert_eq!(query.query, "test query");
        assert_eq!(query.fields.len(), 2);
        assert_eq!(query.limit, Some(10));
        assert_eq!(query.filters.len(), 1);
    }
}
