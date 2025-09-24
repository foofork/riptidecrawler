//! PDF processing module for RipTide
//!
//! This module provides PDF processing capabilities using pdfium-render when available.
//! Falls back to a minimal implementation when the "pdf" feature is disabled.

pub mod config;
pub mod errors;
pub mod integration;
pub mod processor;
pub mod types;
pub mod utils;

#[cfg(test)]
pub mod tests;

#[cfg(feature = "benchmarks")]
pub mod benchmarks;

// Re-export main types and functions
pub use config::{ImageFormat, OcrConfig, PdfCapabilities, PdfConfig};
pub use errors::{PdfError, PdfResult};
pub use integration::{PdfPipelineIntegration, create_pdf_integration_for_pipeline, detect_and_process_pdf};
pub use processor::{create_pdf_processor, PdfProcessor};
pub use types::{
    PdfImage, PdfMetadata, PdfProcessingResult, PdfStats, ProgressCallback, StructuredContent,
};
pub use utils::{detect_pdf_content, detect_pdf_by_extension, detect_pdf_by_magic_bytes};

#[cfg(feature = "pdf")]
pub use processor::PdfiumProcessor;

#[cfg(not(feature = "pdf"))]
pub use processor::DefaultPdfProcessor;
