//! # Riptide Python Bindings
//!
//! Python bindings for the Riptide web scraping framework using PyO3.
//!
//! ## Overview
//!
//! This library provides Python bindings for the Riptide web scraping framework,
//! allowing you to extract content, spider websites, and crawl multiple URLs
//! from Python with the full power of Rust's performance and safety.
//!
//! ## Features
//!
//! - **extract()** - Extract content from a single URL
//! - **spider()** - Discover URLs by crawling a website
//! - **crawl()** - Batch process multiple URLs in parallel
//! - **Document** - Rich document type with metadata
//! - **Async runtime** - Tokio async runtime managed automatically
//!
//! ## Python Example
//!
//! ```python
//! import riptide
//!
//! # Create RipTide instance
//! rt = riptide.RipTide()
//!
//! # Extract content
//! doc = rt.extract("https://example.com")
//! print(doc.title, doc.text)
//!
//! # Spider for URLs
//! urls = rt.spider("https://example.com", max_depth=2)
//!
//! # Batch crawl
//! docs = rt.crawl(urls)
//! ```

use pyo3::prelude::*;

// Modules
mod document;
mod errors;
mod riptide_class;
use pyo3::types::{PyDict, PyList};
use std::sync::Arc;
use tokio::runtime::Runtime;

/// PyO3 Spike: Test basic async operation
///
/// Tests if tokio runtime can be created and used within PyO3.
#[pyfunction]
fn test_async_basic() -> PyResult<String> {
    // Create tokio runtime
    let rt = Runtime::new()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {}", e)))?;

    // Test basic async operation
    rt.block_on(async {
        Ok("Async runtime works!".to_string())
    })
}

/// PyO3 Spike: Test async with multiple tasks
///
/// Tests if tokio runtime can handle concurrent tasks without deadlocks.
#[pyfunction]
fn test_async_concurrent() -> PyResult<Vec<String>> {
    let rt = Runtime::new()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {}", e)))?;

    rt.block_on(async {
        let mut tasks = Vec::new();

        // Spawn multiple concurrent tasks
        for i in 0..5 {
            tasks.push(tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                format!("Task {} completed", i)
            }));
        }

        // Wait for all tasks
        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(result) => results.push(result),
                Err(e) => return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Task failed: {}", e))),
            }
        }

        Ok(results)
    })
}

/// PyO3 Spike: Test async with timeout
///
/// Tests if tokio timeout mechanisms work correctly in PyO3.
#[pyfunction]
fn test_async_timeout(timeout_ms: u64) -> PyResult<String> {
    let rt = Runtime::new()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {}", e)))?;

    rt.block_on(async move {
        let result = tokio::time::timeout(
            tokio::time::Duration::from_millis(timeout_ms),
            async {
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                "Operation completed"
            }
        ).await;

        match result {
            Ok(msg) => Ok(msg.to_string()),
            Err(_) => Err(PyErr::new::<pyo3::exceptions::PyTimeoutError, _>("Operation timed out")),
        }
    })
}

/// PyO3 Spike: Test async with error handling
///
/// Tests if Rust Result types can be properly converted to Python exceptions.
#[pyfunction]
fn test_async_error_handling(should_fail: bool) -> PyResult<String> {
    let rt = Runtime::new()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {}", e)))?;

    rt.block_on(async move {
        if should_fail {
            Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Intentional error for testing"))
        } else {
            Ok("Success!".to_string())
        }
    })
}

/// Spike test for CrawlFacade wrapping
///
/// This is a minimal test to verify we can wrap the CrawlFacade
/// in a Python class with async support.
#[pyclass]
struct RipTideSpike {
    /// Tokio runtime for async operations
    runtime: Runtime,
    /// Test state
    initialized: bool,
}

#[pymethods]
impl RipTideSpike {
    /// Create a new RipTideSpike instance
    #[new]
    fn new() -> PyResult<Self> {
        let runtime = Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {}", e)))?;

        Ok(Self {
            runtime,
            initialized: true,
        })
    }

    /// Test method that uses async
    fn test_async_method(&self) -> PyResult<String> {
        if !self.initialized {
            return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Runtime not initialized"));
        }

        self.runtime.block_on(async {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            Ok("Async method works!".to_string())
        })
    }

    /// Test method that simulates crawl operation
    fn test_crawl_simulation(&self, url: String) -> PyResult<PyObject> {
        if !self.initialized {
            return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Runtime not initialized"));
        }

        Python::with_gil(|py| {
            let result = self.runtime.block_on(async move {
                // Simulate async crawl
                tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;

                Ok::<_, PyErr>(format!("Crawled: {}", url))
            })?;

            // Create Python dict result
            let dict = PyDict::new(py);
            dict.set_item("url", url)?;
            dict.set_item("status", "success")?;
            dict.set_item("content", result)?;

            Ok(dict.into())
        })
    }

    /// Test method that returns a list of URLs
    fn test_spider_simulation(&self, url: String, count: usize) -> PyResult<Vec<String>> {
        if !self.initialized {
            return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Runtime not initialized"));
        }

        self.runtime.block_on(async move {
            // Simulate async spider
            let mut urls = Vec::new();
            for i in 0..count {
                tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
                urls.push(format!("{}/page{}", url, i));
            }
            Ok(urls)
        })
    }

    /// Check if runtime is healthy
    fn is_healthy(&self) -> bool {
        self.initialized
    }
}

/// Python module for Riptide
#[pymodule]
fn riptide(_py: Python, m: &PyModule) -> PyResult<()> {
    // Production classes
    m.add_class::<riptide_class::PyRipTide>()?;
    m.add_class::<document::PyDocument>()?;

    // Spike test functions (kept for testing)
    m.add_function(wrap_pyfunction!(test_async_basic, m)?)?;
    m.add_function(wrap_pyfunction!(test_async_concurrent, m)?)?;
    m.add_function(wrap_pyfunction!(test_async_timeout, m)?)?;
    m.add_function(wrap_pyfunction!(test_async_error_handling, m)?)?;

    // Spike test class (kept for testing)
    m.add_class::<RipTideSpike>()?;

    // Module metadata
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__doc__", "Riptide Python Bindings - High-performance web scraping for Python")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        // Test that we can create a tokio runtime
        let rt = Runtime::new();
        assert!(rt.is_ok(), "Failed to create tokio runtime");
    }

    #[test]
    fn test_async_execution() {
        let rt = Runtime::new().unwrap();
        let result = rt.block_on(async {
            "success"
        });
        assert_eq!(result, "success");
    }

    #[test]
    fn test_concurrent_tasks() {
        let rt = Runtime::new().unwrap();
        let result = rt.block_on(async {
            let mut tasks = Vec::new();
            for i in 0..10 {
                tasks.push(tokio::spawn(async move { i * 2 }));
            }

            let mut results = Vec::new();
            for task in tasks {
                results.push(task.await.unwrap());
            }
            results
        });

        assert_eq!(result.len(), 10);
        assert_eq!(result[0], 0);
        assert_eq!(result[9], 18);
    }

    #[test]
    fn test_spike_instance_creation() {
        let spike = RipTideSpike::new();
        assert!(spike.is_ok(), "Failed to create RipTideSpike instance");

        let spike = spike.unwrap();
        assert!(spike.is_healthy(), "RipTideSpike should be healthy");
    }
}
