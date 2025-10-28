//! Integration example for PDF processing in the RipTide pipeline
//!
//! This module demonstrates how to integrate PDF processing with the main extraction pipeline.

use super::metrics::PdfMetricsCollector;
use super::types::ExtractedDoc;
use super::*;
use riptide_types::ParserMetadata;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, warn};

/// PDF pipeline integration handler with comprehensive monitoring
pub struct PdfPipelineIntegration {
    processor: super::processor::AnyPdfProcessor,
    config: PdfConfig,
    metrics: Arc<PdfMetricsCollector>,
}

impl PdfPipelineIntegration {
    /// Create a new PDF pipeline integration with metrics
    pub fn new() -> Self {
        Self {
            processor: create_pdf_processor(),
            config: PdfConfig::default(),
            metrics: Arc::new(PdfMetricsCollector::new()),
        }
    }

    /// Create with custom configuration and metrics
    pub fn with_config(config: PdfConfig) -> Self {
        Self {
            processor: create_pdf_processor(),
            config,
            metrics: Arc::new(PdfMetricsCollector::new()),
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

    /// Process PDF bytes and convert to ExtractedDoc with comprehensive monitoring
    #[cfg(feature = "pdf")]
    pub async fn process_pdf_to_extracted_doc(
        &self,
        pdf_bytes: &[u8],
        url: Option<&str>,
    ) -> PdfResult<ExtractedDoc> {
        let start_time = Instant::now();

        info!(
            "Starting PDF processing: {} bytes, url: {:?}",
            pdf_bytes.len(),
            url
        );

        // Validate size against ROADMAP limits
        if pdf_bytes.len() as u64 > self.config.max_size_bytes {
            self.metrics.record_processing_failure(false);
            return Err(PdfError::FileTooLarge {
                size: pdf_bytes.len() as u64,
                max_size: self.config.max_size_bytes,
            });
        }

        // Process with error handling and metrics
        match self.processor.process_pdf(pdf_bytes, &self.config).await {
            Ok(result) => {
                let processing_time = start_time.elapsed();
                let pages = result.metadata.page_count;
                let memory_used = result.stats.memory_used;

                // Record successful operation with metrics
                self.metrics
                    .record_processing_success(processing_time, pages, memory_used);

                info!(
                    "PDF processing successful: {} pages, {:?} processing time, {} bytes memory",
                    pages, processing_time, memory_used
                );

                Ok(self.convert_pdf_result_to_extracted_doc(result, url))
            }
            Err(e) => {
                let processing_time = start_time.elapsed();
                let is_memory_limit = matches!(e, PdfError::MemoryLimit { .. });

                self.metrics.record_processing_failure(is_memory_limit);

                warn!("PDF processing failed after {:?}: {:?}", processing_time, e);
                Err(e)
            }
        }
    }

    /// Fallback for when PDF feature is not available
    #[cfg(not(feature = "pdf"))]
    pub async fn process_pdf_to_extracted_doc(
        &self,
        _pdf_bytes: &[u8],
        _url: Option<&str>,
    ) -> PdfResult<ExtractedDoc> {
        Err(PdfError::ProcessingError {
            message: "PDF processing feature is not enabled. Enable with --features pdf"
                .to_string(),
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
        metadata.insert(
            "images_extracted".to_string(),
            result.stats.images_extracted.to_string(),
        );
        metadata.insert(
            "processing_time_ms".to_string(),
            result.stats.processing_time_ms.to_string(),
        );

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
            markdown: Some(text_content.clone()),
            text: text_content,
            links: Vec::new(), // PDFs typically don't have external links in our extraction
            media,
            language: None, // Could be detected from text content
            reading_time,
            quality_score: Some(if result.success { 85 } else { 30 }),
            word_count: Some(word_count),
            parser_metadata: Some(ParserMetadata {
                parser_used: "pdf".to_string(),
                confidence_score: if result.success { 0.85 } else { 0.3 },
                fallback_occurred: false,
                parse_time_ms: 0,
                extraction_path: None,
                primary_error: None,
            }),
            categories: vec!["document".to_string(), "pdf".to_string()],
            site_name: metadata_obj.producer.clone(),
            description: metadata_obj.subject.clone(),
            html: None, // PDFs don't have HTML content
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

    /// Get comprehensive PDF processing metrics
    pub fn get_metrics_snapshot(&self) -> super::metrics::PdfMetricsSnapshot {
        self.metrics.get_snapshot()
    }

    /// Export metrics for monitoring systems (Prometheus format)
    pub fn export_metrics_for_monitoring(&self) -> std::collections::HashMap<String, f64> {
        self.metrics.export_for_prometheus()
    }

    /// Reset metrics (useful for testing and periodic resets)
    pub fn reset_metrics(&self) {
        self.metrics.reset();
    }

    /// Create async progress channel for streaming updates
    pub fn create_progress_channel(
        &self,
    ) -> (super::types::ProgressSender, super::types::ProgressReceiver) {
        super::types::create_progress_channel()
    }

    /// Process PDF with streaming progress updates
    #[cfg(feature = "pdf")]
    pub async fn process_pdf_bytes_with_progress(
        &self,
        pdf_bytes: &[u8],
        progress_sender: super::types::ProgressSender,
    ) -> PdfResult<ExtractedDoc> {
        use super::types::ProgressUpdate;

        // Send started event
        let _ = progress_sender.send(ProgressUpdate::Started {
            total_pages: 0, // Will be updated once document loads
            file_size: pdf_bytes.len() as u64,
            timestamp: chrono::Utc::now().to_rfc3339(),
        });

        // Create progress callback that sends updates
        let progress_callback = {
            let sender = progress_sender.clone();
            Some(Box::new(move |current_page: u32, total_pages: u32| {
                let percentage = if total_pages > 0 {
                    (current_page as f32 / total_pages as f32) * 100.0
                } else {
                    0.0
                };
                let progress = super::types::ProcessingProgress {
                    current_page,
                    total_pages,
                    percentage,
                    estimated_remaining_ms: None,
                    stage: super::types::ProcessingStage::ExtractingText(current_page),
                };
                let _ = sender.send(ProgressUpdate::Progress(progress));
            }) as super::types::ProgressCallback)
        };

        // Create a PdfConfig with progress enabled
        let mut config = self.config.clone();
        config.enable_progress_tracking = true;

        // Use the processor's process_pdf_with_progress method
        match self
            .processor
            .process_pdf_with_progress(pdf_bytes, &config, progress_callback)
            .await
        {
            Ok(pdf_result) => {
                // Convert PdfProcessingResult to ExtractedDoc
                let extracted_doc =
                    self.convert_pdf_result_to_extracted_doc(pdf_result.clone(), None);

                let _ = progress_sender.send(ProgressUpdate::Completed {
                    result: Box::new(pdf_result),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                });
                Ok(extracted_doc)
            }
            Err(e) => {
                let _ = progress_sender.send(ProgressUpdate::Failed {
                    error: format!("{:?}", e),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                });
                Err(e)
            }
        }
    }
}

impl Default for PdfPipelineIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory function to create PDF integration with ROADMAP-compliant settings
pub fn create_pdf_integration_for_pipeline() -> PdfPipelineIntegration {
    let config = PdfConfig {
        max_size_bytes: 50 * 1024 * 1024, // 50MB limit for pipeline
        extract_text: true,
        extract_images: true,
        extract_metadata: true,
        enable_progress_tracking: false, // Disable for pipeline use
        timeout_seconds: 30,
        memory_settings: super::config::MemorySettings {
            max_memory_spike_bytes: 200 * 1024 * 1024, // ROADMAP: No >200MB RSS spikes
            max_concurrent_operations: 2,              // ROADMAP: Max 2 concurrent PDF operations
            memory_check_interval: 3,                  // Check every 3 pages
            cleanup_interval: 10,                      // Cleanup every 10 pages
            memory_pressure_threshold: 0.8,            // 80% memory pressure threshold
            aggressive_cleanup: true,                  // Enable aggressive cleanup
        },
        ..Default::default()
    };

    PdfPipelineIntegration::with_config(config)
}

/// Quick utility function to detect and process PDF content with monitoring
pub async fn detect_and_process_pdf(
    content_type: Option<&str>,
    url: Option<&str>,
    data: &[u8],
) -> Option<PdfResult<ExtractedDoc>> {
    let integration = create_pdf_integration_for_pipeline();

    if integration.should_process_as_pdf(content_type, url, Some(data)) {
        debug!("Detected PDF content, processing: {} bytes", data.len());
        Some(integration.process_pdf_to_extracted_doc(data, url).await)
    } else {
        debug!("Content not detected as PDF, skipping PDF processing");
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
