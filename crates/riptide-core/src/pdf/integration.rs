//! Integration example for PDF processing in the RipTide pipeline
//!
//! This module demonstrates how to integrate PDF processing with the main extraction pipeline.

use super::*;
use crate::types::ExtractedDoc;
use std::collections::HashMap;

/// PDF pipeline integration handler
pub struct PdfPipelineIntegration {
    processor: super::processor::AnyPdfProcessor,
    config: PdfConfig,
}

impl PdfPipelineIntegration {
    /// Create a new PDF pipeline integration
    pub fn new() -> Self {
        Self {
            processor: create_pdf_processor(),
            config: PdfConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: PdfConfig) -> Self {
        Self {
            processor: create_pdf_processor(),
            config,
        }
    }

    /// Check if content should be processed as PDF
    pub fn should_process_as_pdf(
        &self,
        content_type: Option<&str>,
        url: Option<&str>,
        data: Option<&[u8]>,
    ) -> bool {
        utils::detect_pdf_content(content_type, url, data)
    }

    /// Process PDF bytes and convert to ExtractedDoc
    #[cfg(feature = "pdf")]
    pub async fn process_pdf_to_extracted_doc(
        &self,
        pdf_bytes: &[u8],
        url: Option<&str>,
    ) -> PdfResult<ExtractedDoc> {
        // For now, use the standard processing path since downcasting is complex
        // In a real implementation, this could be optimized to use PdfiumProcessor directly
        let result = self.processor.process_pdf(pdf_bytes, &self.config).await?;
        Ok(self.convert_pdf_result_to_extracted_doc(result, url))
    }

    /// Fallback for when PDF feature is not available
    #[cfg(not(feature = "pdf"))]
    pub async fn process_pdf_to_extracted_doc(
        &self,
        _pdf_bytes: &[u8],
        _url: Option<&str>,
    ) -> PdfResult<ExtractedDoc> {
        Err(PdfError::ProcessingError {
            message: "PDF processing feature is not enabled. Enable with --features pdf".to_string(),
        })
    }

    /// Convert PdfProcessingResult to ExtractedDoc
    fn convert_pdf_result_to_extracted_doc(
        &self,
        result: PdfProcessingResult,
        url: Option<&str>,
    ) -> ExtractedDoc {
        let mut metadata = HashMap::new();
        let metadata_obj = &result.metadata;

        // Extract basic metadata
        if let Some(ref title) = metadata_obj.title {
            metadata.insert("title".to_string(), title.clone());
        }
        if let Some(ref author) = metadata_obj.author {
            metadata.insert("author".to_string(), author.clone());
        }
        if let Some(ref subject) = metadata_obj.subject {
            metadata.insert("subject".to_string(), subject.clone());
        }
        if let Some(ref creator) = metadata_obj.creator {
            metadata.insert("creator".to_string(), creator.clone());
        }
        if let Some(ref producer) = metadata_obj.producer {
            metadata.insert("producer".to_string(), producer.clone());
        }

        // Add extraction statistics
        metadata.insert("pages".to_string(), metadata_obj.page_count.to_string());
        metadata.insert("images_extracted".to_string(), result.stats.images_extracted.to_string());
        metadata.insert("processing_time_ms".to_string(), result.stats.processing_time_ms.to_string());

        // Create media list for images
        let mut media = Vec::new();
        for (i, _) in result.images.iter().enumerate() {
            media.push(format!("pdf:image:{}", i));
        }

        // Calculate reading time
        let text_content = result.text.unwrap_or_default();
        let word_count = text_content.split_whitespace().count() as u32;
        let reading_time = Some(utils::estimate_reading_time(word_count));

        ExtractedDoc {
            url: url.unwrap_or("pdf://document").to_string(),
            title: metadata_obj.title.clone(),
            byline: metadata_obj.author.clone(),
            published_iso: metadata_obj.creation_date.clone(),
            markdown: text_content.clone(),
            text: text_content,
            links: Vec::new(), // PDFs typically don't have external links in our extraction
            media,
            language: None, // Could be detected from text content
            reading_time,
            quality_score: Some(if result.success { 85 } else { 30 }),
            word_count: Some(word_count),
            categories: vec!["document".to_string(), "pdf".to_string()],
            site_name: metadata_obj.producer.clone(),
            description: metadata_obj.subject.clone(),
        }
    }

    /// Get processor capabilities
    pub fn capabilities(&self) -> PdfCapabilities {
        self.processor.capabilities()
    }

    /// Check if processor is available
    pub fn is_available(&self) -> bool {
        self.processor.is_available()
    }
}

impl Default for PdfPipelineIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory function to create PDF integration with custom settings
pub fn create_pdf_integration_for_pipeline() -> PdfPipelineIntegration {
    let config = PdfConfig {
        max_size_bytes: 50 * 1024 * 1024, // 50MB limit for pipeline
        extract_text: true,
        extract_images: true,
        extract_metadata: true,
        enable_progress_tracking: false, // Disable for pipeline use
        timeout_seconds: 30,
        ..Default::default()
    };

    PdfPipelineIntegration::with_config(config)
}

/// Quick utility function to detect and process PDF content
pub async fn detect_and_process_pdf(
    content_type: Option<&str>,
    url: Option<&str>,
    data: &[u8],
) -> Option<PdfResult<ExtractedDoc>> {
    let integration = create_pdf_integration_for_pipeline();

    if integration.should_process_as_pdf(content_type, url, Some(data)) {
        Some(integration.process_pdf_to_extracted_doc(data, url).await)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_integration_creation() {
        let integration = PdfPipelineIntegration::new();
        assert_eq!(integration.config.max_size_bytes, 100 * 1024 * 1024);

        let custom_config = PdfConfig {
            max_size_bytes: 10 * 1024 * 1024,
            ..Default::default()
        };
        let custom_integration = PdfPipelineIntegration::with_config(custom_config);
        assert_eq!(custom_integration.config.max_size_bytes, 10 * 1024 * 1024);
    }

    #[test]
    fn test_pdf_detection() {
        let integration = PdfPipelineIntegration::new();

        // Test content-type detection
        assert!(integration.should_process_as_pdf(Some("application/pdf"), None, None));
        assert!(!integration.should_process_as_pdf(Some("text/html"), None, None));

        // Test magic bytes detection
        let pdf_data = b"%PDF-1.7\n...";
        assert!(integration.should_process_as_pdf(None, None, Some(pdf_data)));
        assert!(!integration.should_process_as_pdf(None, None, Some(b"not pdf")));

        // Test URL extension detection
        assert!(integration.should_process_as_pdf(None, Some("document.pdf"), None));
        assert!(!integration.should_process_as_pdf(None, Some("document.html"), None));
    }

    #[test]
    fn test_factory_function() {
        let integration = create_pdf_integration_for_pipeline();
        assert_eq!(integration.config.max_size_bytes, 50 * 1024 * 1024);
        assert!(integration.config.extract_text);
        assert!(integration.config.extract_images);
        assert!(integration.config.extract_metadata);
        assert!(!integration.config.enable_progress_tracking);
    }

    #[tokio::test]
    async fn test_detect_and_process_pdf() {
        let pdf_data = b"%PDF-1.7\n...";

        // Should detect PDF and return Some
        let result = detect_and_process_pdf(Some("application/pdf"), None, pdf_data).await;
        assert!(result.is_some());

        // Should not detect non-PDF and return None
        let result = detect_and_process_pdf(Some("text/html"), None, b"<html></html>").await;
        assert!(result.is_none());
    }
}