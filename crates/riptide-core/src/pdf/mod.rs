//! PDF processing module for RipTide
//!
//! This module provides PDF processing capabilities using pdfium-render when available.
//! Falls back to a minimal implementation when the "pdf" feature is disabled.

pub mod config;
pub mod errors;
pub mod processor;
pub mod types;
pub mod utils;

// Re-export main types and functions
pub use config::{ImageFormat, OcrConfig, PdfCapabilities, PdfConfig};
pub use errors::{PdfError, PdfResult};
pub use processor::{create_pdf_processor, PdfProcessor};
pub use types::{
    PdfImage, PdfMetadata, PdfProcessingResult, PdfStats, ProgressCallback, StructuredContent,
};

#[cfg(feature = "pdf")]
pub use processor::PdfiumProcessor;

#[cfg(not(feature = "pdf"))]
pub use processor::DefaultPdfProcessor;
