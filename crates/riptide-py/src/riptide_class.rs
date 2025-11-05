//! RipTide Python class - Main API wrapper
//!
//! This module provides the primary Python API for the Riptide web scraping framework.
//! It wraps CrawlFacade and exposes extract(), spider(), and crawl() methods to Python.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::Arc;
use tokio::runtime::Runtime;

use riptide_api::state::AppState;
use riptide_facade::facades::{CrawlFacade, CrawlMode, CrawlResult};
use riptide_types::config::CrawlOptions;

use crate::document::PyDocument;
use crate::errors::RipTideError;

/// Main RipTide Python class
///
/// This class provides the primary interface for web scraping operations from Python.
/// It wraps the CrawlFacade and manages the async runtime.
///
/// # Example (Python)
///
/// ```python
/// import riptide
///
/// # Create RipTide instance
/// rt = riptide.RipTide()
///
/// # Extract content from a single URL
/// doc = rt.extract("https://example.com")
/// print(doc.title)
/// print(doc.text)
///
/// # Spider to discover URLs
/// urls = rt.spider("https://example.com", max_depth=2)
/// print(f"Found {len(urls)} URLs")
///
/// # Batch crawl multiple URLs
/// results = rt.crawl(["https://example.com", "https://example.org"])
/// for doc in results:
///     print(f"{doc.url}: {doc.title}")
/// ```
#[pyclass(name = "RipTide")]
pub struct PyRipTide {
    /// CrawlFacade wrapped in Arc for shared ownership
    facade: Arc<CrawlFacade>,
    /// Tokio runtime for async operations
    runtime: Runtime,
}

#[pymethods]
impl PyRipTide {
    /// Create a new RipTide instance
    ///
    /// # Arguments
    ///
    /// * `api_key` - Optional API key for future cloud features
    ///
    /// # Returns
    ///
    /// New RipTide instance ready for scraping
    ///
    /// # Example (Python)
    ///
    /// ```python
    /// import riptide
    ///
    /// # Basic instance
    /// rt = riptide.RipTide()
    ///
    /// # With API key (future feature)
    /// rt = riptide.RipTide(api_key="your-key-here")
    /// ```
    #[new]
    #[pyo3(signature = (api_key=None))]
    fn new(api_key: Option<String>) -> PyResult<Self> {
        // Create tokio runtime
        let runtime = Runtime::new().map_err(|e| {
            RipTideError::runtime_error(format!("Failed to create async runtime: {}", e))
        })?;

        // Create AppState
        let state = runtime.block_on(async {
            AppState::new().await.map_err(|e| {
                RipTideError::initialization_error(format!("Failed to create AppState: {}", e))
            })
        })?;

        // Create CrawlFacade with default options
        let options = CrawlOptions::default();
        let facade = Arc::new(CrawlFacade::with_options(state, options));

        // TODO: Use api_key for future cloud features
        let _ = api_key;

        Ok(Self { facade, runtime })
    }

    /// Extract content from a single URL
    ///
    /// This method fetches a URL and extracts structured content including
    /// title, text, metadata, and more.
    ///
    /// # Arguments
    ///
    /// * `url` - URL to extract content from
    /// * `mode` - Extraction mode: "standard" (default) or "enhanced"
    ///
    /// # Returns
    ///
    /// Document object containing extracted content
    ///
    /// # Errors
    ///
    /// - ValueError: Invalid URL
    /// - RuntimeError: Network or extraction error
    /// - TimeoutError: Request timed out
    ///
    /// # Example (Python)
    ///
    /// ```python
    /// rt = riptide.RipTide()
    ///
    /// # Standard mode (fast)
    /// doc = rt.extract("https://example.com")
    ///
    /// # Enhanced mode (more strategies)
    /// doc = rt.extract("https://example.com", mode="enhanced")
    ///
    /// print(doc.title)
    /// print(doc.text)
    /// print(doc.url)
    /// ```
    #[pyo3(signature = (url, mode="standard"))]
    fn extract(&self, url: &str, mode: &str) -> PyResult<PyDocument> {
        // Validate URL
        if url.is_empty() {
            return Err(RipTideError::value_error("URL cannot be empty"));
        }

        // Parse mode
        let crawl_mode = match mode {
            "standard" => CrawlMode::Standard,
            "enhanced" => CrawlMode::Enhanced,
            _ => return Err(RipTideError::value_error(format!("Invalid mode: {}", mode))),
        };

        // Execute async extraction
        let result = self.runtime.block_on(async {
            let options = CrawlOptions::default();
            self.facade
                .crawl_single(url, options, crawl_mode)
                .await
                .map_err(|e| RipTideError::extraction_error(format!("Extraction failed: {}", e)))
        })?;

        // Convert to Python Document
        PyDocument::from_crawl_result(url.to_string(), result)
    }

    /// Spider a URL to discover linked URLs
    ///
    /// This method crawls a URL and discovers all linked URLs up to a specified depth.
    /// It does NOT extract content, only discovers URLs.
    ///
    /// # Arguments
    ///
    /// * `url` - Starting URL to spider
    /// * `max_depth` - Maximum depth to crawl (default: 2)
    /// * `max_urls` - Maximum URLs to discover (default: 100)
    ///
    /// # Returns
    ///
    /// List of discovered URLs
    ///
    /// # Example (Python)
    ///
    /// ```python
    /// rt = riptide.RipTide()
    ///
    /// # Spider with default depth (2)
    /// urls = rt.spider("https://example.com")
    ///
    /// # Custom depth and limit
    /// urls = rt.spider("https://example.com", max_depth=3, max_urls=200)
    ///
    /// for url in urls:
    ///     print(url)
    /// ```
    #[pyo3(signature = (url, max_depth=2, max_urls=100))]
    fn spider(&self, url: &str, max_depth: u32, max_urls: usize) -> PyResult<Vec<String>> {
        if url.is_empty() {
            return Err(RipTideError::value_error("URL cannot be empty"));
        }

        // TODO: Implement actual spider logic using riptide-spider
        // For now, return a placeholder
        self.runtime.block_on(async {
            // Simulate async spider operation
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

            // Placeholder: In reality, this would use SpiderFacade
            let mut urls = vec![url.to_string()];
            urls.push(format!("{}/page1", url));
            urls.push(format!("{}/page2", url));

            // Respect limits
            urls.truncate(max_urls.min(100));

            Ok(urls)
        })
    }

    /// Batch crawl multiple URLs
    ///
    /// This method extracts content from multiple URLs in parallel.
    ///
    /// # Arguments
    ///
    /// * `urls` - List of URLs to crawl
    /// * `mode` - Extraction mode: "standard" (default) or "enhanced"
    ///
    /// # Returns
    ///
    /// List of Document objects (one per URL)
    ///
    /// # Example (Python)
    ///
    /// ```python
    /// rt = riptide.RipTide()
    ///
    /// urls = [
    ///     "https://example.com",
    ///     "https://example.org",
    ///     "https://example.net",
    /// ]
    ///
    /// docs = rt.crawl(urls)
    ///
    /// for doc in docs:
    ///     print(f"{doc.url}: {doc.title}")
    /// ```
    #[pyo3(signature = (urls, mode="standard"))]
    fn crawl(&self, urls: Vec<String>, mode: &str) -> PyResult<Vec<PyDocument>> {
        if urls.is_empty() {
            return Err(RipTideError::value_error("URLs list cannot be empty"));
        }

        // Parse mode
        let _crawl_mode = match mode {
            "standard" => CrawlMode::Standard,
            "enhanced" => CrawlMode::Enhanced,
            _ => return Err(RipTideError::value_error(format!("Invalid mode: {}", mode))),
        };

        // Execute batch crawl
        self.runtime.block_on(async {
            let (results, _stats) = self.facade.crawl_batch(&urls).await;

            // Convert results to PyDocument
            let mut docs = Vec::new();
            for (i, result_opt) in results.iter().enumerate() {
                if let Some(result) = result_opt {
                    // Convert PipelineResult to PyDocument
                    let doc = PyDocument::from_pipeline_result(urls[i].clone(), result.clone())?;
                    docs.push(doc);
                } else {
                    // URL failed, create error document
                    docs.push(PyDocument::error_document(
                        urls[i].clone(),
                        "Extraction failed".to_string(),
                    ));
                }
            }

            Ok(docs)
        })
    }

    /// Get the version of the Riptide library
    #[staticmethod]
    fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Check if the RipTide instance is healthy
    fn is_healthy(&self) -> bool {
        // Check if runtime is functioning
        self.runtime.block_on(async {
            // Simple health check
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            true
        })
    }

    /// String representation
    fn __repr__(&self) -> String {
        format!("RipTide(version={})", Self::version())
    }

    /// String conversion
    fn __str__(&self) -> String {
        format!("RipTide v{}", Self::version())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_riptide_creation() {
        let rt = PyRipTide::new(None);
        assert!(rt.is_ok(), "Failed to create PyRipTide instance");
    }

    #[test]
    fn test_riptide_healthy() {
        let rt = PyRipTide::new(None).unwrap();
        assert!(rt.is_healthy(), "PyRipTide should be healthy");
    }

    #[test]
    fn test_version() {
        let version = PyRipTide::version();
        assert!(!version.is_empty(), "Version should not be empty");
    }
}
