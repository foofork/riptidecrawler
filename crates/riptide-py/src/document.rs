//! Document class for Python
//!
//! Represents extracted content from a web page.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use riptide_api::pipeline::PipelineResult;
use riptide_facade::facades::CrawlResult;

use crate::errors::RipTideError;

/// Document class representing extracted web content
///
/// This class holds all extracted content including title, text,
/// metadata, and more.
///
/// # Attributes (Python)
///
/// - `url` (str): Source URL
/// - `title` (str): Page title
/// - `text` (str): Extracted text content
/// - `html` (str | None): Raw HTML (if available)
/// - `quality_score` (float): Content quality score (0.0-1.0)
/// - `word_count` (int): Number of words
/// - `from_cache` (bool): Whether content was cached
/// - `processing_time_ms` (int): Processing time in milliseconds
///
/// # Example (Python)
///
/// ```python
/// rt = riptide.RipTide()
/// doc = rt.extract("https://example.com")
///
/// print(doc.url)           # "https://example.com"
/// print(doc.title)         # "Example Domain"
/// print(doc.text)          # "Example text..."
/// print(doc.quality_score) # 0.95
/// ```
#[pyclass(name = "Document")]
#[derive(Clone)]
pub struct PyDocument {
    #[pyo3(get)]
    pub url: String,

    #[pyo3(get)]
    pub title: String,

    #[pyo3(get)]
    pub text: String,

    #[pyo3(get)]
    pub html: Option<String>,

    #[pyo3(get)]
    pub quality_score: f64,

    #[pyo3(get)]
    pub word_count: usize,

    #[pyo3(get)]
    pub from_cache: bool,

    #[pyo3(get)]
    pub processing_time_ms: u64,
}

#[pymethods]
impl PyDocument {
    /// Create a new Document
    #[new]
    fn new(
        url: String,
        title: String,
        text: String,
        html: Option<String>,
        quality_score: f64,
        word_count: usize,
        from_cache: bool,
        processing_time_ms: u64,
    ) -> Self {
        Self {
            url,
            title,
            text,
            html,
            quality_score,
            word_count,
            from_cache,
            processing_time_ms,
        }
    }

    /// Convert to dictionary
    ///
    /// # Returns (Python)
    ///
    /// Dictionary with all document fields
    fn to_dict(&self, py: Python) -> PyResult<PyObject> {
        let dict = PyDict::new(py);
        dict.set_item("url", &self.url)?;
        dict.set_item("title", &self.title)?;
        dict.set_item("text", &self.text)?;
        dict.set_item("html", &self.html)?;
        dict.set_item("quality_score", self.quality_score)?;
        dict.set_item("word_count", self.word_count)?;
        dict.set_item("from_cache", self.from_cache)?;
        dict.set_item("processing_time_ms", self.processing_time_ms)?;
        Ok(dict.into())
    }

    /// String representation
    fn __repr__(&self) -> String {
        format!(
            "Document(url='{}', title='{}', words={}, quality={:.2})",
            self.url, self.title, self.word_count, self.quality_score
        )
    }

    /// String conversion
    fn __str__(&self) -> String {
        format!("{}: {}", self.url, self.title)
    }

    /// Get text length
    fn __len__(&self) -> usize {
        self.text.len()
    }
}

impl PyDocument {
    /// Create from CrawlResult
    pub fn from_crawl_result(url: String, result: CrawlResult) -> PyResult<Self> {
        match result {
            CrawlResult::Standard(pipeline_result) => {
                Self::from_pipeline_result(url, pipeline_result)
            }
            CrawlResult::Enhanced(strategies_result) => {
                // Convert strategies result to PyDocument
                Ok(PyDocument {
                    url: url.clone(),
                    title: "Enhanced Result".to_string(), // TODO: Extract from strategies_result
                    text: format!("Processed content from {}", url),
                    html: None,
                    quality_score: strategies_result.quality_score as f64,
                    word_count: 0, // TODO: Calculate from processed_content
                    from_cache: strategies_result.from_cache,
                    processing_time_ms: strategies_result.processing_time_ms,
                })
            }
        }
    }

    /// Create from PipelineResult
    pub fn from_pipeline_result(url: String, result: PipelineResult) -> PyResult<Self> {
        let doc = &result.document;

        Ok(PyDocument {
            url: url.clone(),
            title: doc.title.clone().unwrap_or_else(|| "Untitled".to_string()),
            text: doc.text.clone(),
            html: doc.html.clone(),
            quality_score: doc.quality_score.unwrap_or(0.0) as f64,
            word_count: doc.word_count.unwrap_or(0) as usize,
            from_cache: result.from_cache,
            processing_time_ms: result.processing_time_ms,
        })
    }

    /// Create error document
    pub fn error_document(url: String, error: String) -> Self {
        PyDocument {
            url,
            title: "Error".to_string(),
            text: error,
            html: None,
            quality_score: 0.0,
            word_count: 0,
            from_cache: false,
            processing_time_ms: 0,
        }
    }
}
