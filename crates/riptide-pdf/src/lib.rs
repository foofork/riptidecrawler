//! # Riptide PDF Processing
//!
//! PDF processing capabilities for the RipTide web scraping framework.
//! This crate provides PDF text extraction, image extraction, and metadata
//! processing using pdfium-render when available, with fallback implementations.
//!
//! ## Features
//!
//! - **PDF Text Extraction**: Extract text content with layout preservation
//! - **Image Extraction**: Extract embedded images from PDF documents
//! - **Metadata Processing**: Extract document metadata and structure
//! - **Memory Optimization**: Efficient memory usage and monitoring
//! - **Performance Metrics**: Comprehensive metrics collection
//! - **Pipeline Integration**: Seamless integration with RipTide pipelines
//!
//! ## Usage
//!
//! ```rust
//! use riptide_pdf::{PdfProcessor, create_pdf_processor, PdfConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let processor = create_pdf_processor();
//!     let config = PdfConfig::default();
//!
//!     let pdf_bytes = std::fs::read("document.pdf")?;
//!     let result = processor.process_pdf(&pdf_bytes, &config, None).await?;
//!
//!     println!("Extracted {} characters", result.text.len());
//!     Ok(())
//! }
//! ```
//!
//! ## Feature Flags
//!
//! - `pdf` (default): Enable pdfium-render support for full PDF processing
//! - `benchmarks`: Enable performance benchmarking tools

// Core modules
pub mod config;
pub mod errors;
pub mod integration;
pub mod memory_benchmark;
pub mod metrics;
pub mod processor;
pub mod types;
pub mod utils;

// PDF extraction module
#[cfg(feature = "pdf")]
pub mod pdf_extraction;

// Conditional modules
#[cfg(test)]
pub mod tests;

#[cfg(feature = "benchmarks")]
pub mod benchmarks;

// Re-export main types and functions for convenient access
pub use config::{ImageFormat, MemorySettings, OcrConfig, PdfCapabilities, PdfConfig};
pub use errors::{PdfError, PdfResult};
pub use integration::{
    create_pdf_integration_for_pipeline, detect_and_process_pdf, PdfPipelineIntegration,
};
pub use memory_benchmark::{MemoryBenchmarkResults, PdfMemoryBenchmark};
pub use metrics::{PdfMetricsCollector, PdfMetricsSnapshot, PdfOperationTimer};
pub use processor::{create_pdf_processor, AnyPdfProcessor, PdfProcessor};
pub use types::{
    ExtractedDoc, PdfImage, PdfMetadata, PdfProcessingResult, PdfStats, ProgressCallback,
    StructuredContent,
};
pub use utils::{detect_pdf_by_extension, detect_pdf_by_magic_bytes, detect_pdf_content};

// Re-export PDF extraction types
#[cfg(feature = "pdf")]
pub use pdf_extraction::{
    ExtractedTable, PageContent, PdfContent, PdfDocMetadata, PdfExtractor, TablePosition,
};

// Conditionally export processor implementations
#[cfg(feature = "pdf")]
pub use processor::PdfiumProcessor;

#[cfg(not(feature = "pdf"))]
pub use processor::DefaultPdfProcessor;

/// Version information for the riptide-pdf crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Create a PDF processor with default configuration
///
/// This is a convenience function that creates the appropriate PDF processor
/// based on available features. When the "pdf" feature is enabled, it returns
/// a PdfiumProcessor. Otherwise, it returns a DefaultPdfProcessor.
///
/// # Examples
///
/// ```rust
/// use riptide_pdf::create_pdf_processor;
///
/// let processor = create_pdf_processor();
/// ```
pub fn create_default_pdf_processor() -> AnyPdfProcessor {
    create_pdf_processor()
}

/// Create a PDF configuration with sensible defaults
///
/// This provides a quick way to get a working PDF configuration for most use cases.
///
/// # Examples
///
/// ```rust
/// use riptide_pdf::create_default_config;
///
/// let config = create_default_config();
/// ```
pub fn create_default_config() -> PdfConfig {
    PdfConfig::default()
}

/// Check if the current build supports full PDF processing
///
/// Returns true if the "pdf" feature is enabled and pdfium-render is available.
///
/// # Examples
///
/// ```rust
/// use riptide_pdf::supports_pdf_processing;
///
/// if supports_pdf_processing() {
///     println!("Full PDF processing is available");
/// } else {
///     println!("Using fallback PDF processor");
/// }
/// ```
pub fn supports_pdf_processing() -> bool {
    cfg!(feature = "pdf")
}

/// Get information about the PDF processing capabilities
///
/// Returns a structure describing what PDF operations are supported
/// in the current build configuration.
///
/// # Examples
///
/// ```rust
/// use riptide_pdf::get_pdf_capabilities;
///
/// let capabilities = get_pdf_capabilities();
/// println!("Text extraction: {}", capabilities.can_extract_text);
/// println!("Image extraction: {}", capabilities.can_extract_images);
/// ```
pub fn get_pdf_capabilities() -> PdfCapabilities {
    PdfCapabilities::current()
}
