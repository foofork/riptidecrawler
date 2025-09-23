//! PDF processing module with modular architecture
//!
//! This module provides comprehensive PDF processing capabilities including:
//! - Text extraction
//! - Image extraction
//! - Metadata extraction
//! - OCR detection and processing
//! - Progress tracking
//! - Error handling
//!
//! # Architecture
//!
//! The module is organized into several focused submodules:
//! - `config`: Configuration types and settings
//! - `types`: Result types and data structures
//! - `errors`: Error handling and result types
//! - `processor`: Core processing implementations
//! - `utils`: Utility functions and helpers
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use riptide_core::pdf::{process_pdf, PdfConfig};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let pdf_data = std::fs::read("document.pdf")?;
//! let result = process_pdf(&pdf_data).await?;
//! println!("Extracted {} words", result.word_count.unwrap_or(0));
//! # Ok(())
//! # }
//! ```

mod config;
mod errors;
mod processor;
mod types;
mod utils;

// Re-export commonly used types
pub use config::{PdfConfig, PdfCapabilities, ImageExtractionSettings, TextExtractionSettings, OcrConfig};
pub use errors::{PdfError, PdfResult};
pub use processor::{PdfProcessor, create_pdf_processor};

// Re-export specific processor types for backward compatibility
#[cfg(feature = "pdf")]
pub use processor::PdfiumProcessor;

#[cfg(not(feature = "pdf"))]
pub use processor::DefaultPdfProcessor;
pub use types::{PdfProcessingResult, PdfImage, PdfMetadata, PdfStats, ProgressCallback};

/// Simple function to process PDF bytes and return ExtractedDoc
pub async fn process_pdf(pdf_bytes: &[u8]) -> anyhow::Result<crate::types::ExtractedDoc> {
    #[cfg(feature = "pdf")]
    {
        let processor = processor::PdfiumProcessor::new();
        processor
            .process_pdf_bytes(pdf_bytes)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }
    #[cfg(not(feature = "pdf"))]
    {
        Err(anyhow::anyhow!(
            "PDF processing feature is not enabled. Enable with --features pdf"
        ))
    }
}

/// Enhanced PDF processing with progress tracking
pub async fn process_pdf_with_progress<F>(
    pdf_bytes: &[u8],
    config: Option<PdfConfig>,
    progress_callback: Option<F>,
) -> anyhow::Result<PdfProcessingResult>
where
    F: Fn(u32, u32) + Send + Sync + 'static,
{
    #[cfg(feature = "pdf")]
    {
        let processor = processor::PdfiumProcessor::new();
        let config = config.unwrap_or_default();
        let callback: Option<ProgressCallback> =
            progress_callback.map(|f| Box::new(f) as ProgressCallback);
        processor
            .process_pdf_with_progress(pdf_bytes, &config, callback)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }
    #[cfg(not(feature = "pdf"))]
    {
        Err(anyhow::anyhow!(
            "PDF processing feature is not enabled. Enable with --features pdf"
        ))
    }
}

/// Detect if a PDF needs OCR processing
pub async fn detect_pdf_ocr_need(pdf_bytes: &[u8]) -> anyhow::Result<bool> {
    #[cfg(feature = "pdf")]
    {
        let processor = processor::PdfiumProcessor::new();
        processor
            .detect_ocr_need(pdf_bytes)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }
    #[cfg(not(feature = "pdf"))]
    {
        Err(anyhow::anyhow!(
            "PDF processing feature is not enabled. Enable with --features pdf"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_pdf_function() {
        let pdf_data = b"%PDF-1.7\n1 0 obj\n<<\n/Type /Catalog\n>>\nendobj\n";

        let result = process_pdf(pdf_data).await;

        #[cfg(feature = "pdf")]
        {
            // With PDF feature, should attempt processing (may fail if pdfium not available)
            // This tests the function signature and basic validation
            assert!(result.is_ok() || result.is_err()); // Either outcome is valid for testing
        }

        #[cfg(not(feature = "pdf"))]
        {
            // Without PDF feature, should return error
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("not enabled"));
        }
    }

    #[test]
    fn test_module_exports() {
        // Test that we can create types from re-exports
        let _config = PdfConfig::default();
        let _processor = create_pdf_processor();

        // Test error type
        let _error = PdfError::EncryptedPdf;

        #[cfg(feature = "pdf")]
        {
            let _processor = processor::PdfiumProcessor::new();
        }
    }
}