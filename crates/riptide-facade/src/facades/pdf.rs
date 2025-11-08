//! PdfFacade - Comprehensive PDF processing business logic
//!
//! This facade encapsulates all PDF processing business logic extracted from riptide-api handlers.
//! It provides a clean separation of concerns, making handlers thin and testable.
//!
//! **Responsibilities:**
//! - PDF data decoding (base64, raw bytes)
//! - File validation (size limits, magic bytes)
//! - Multipart form data extraction
//! - PDF processing orchestration
//! - Progress tracking and streaming
//! - Resource acquisition delegation
//!
//! **Original Handler Logic Moved Here:**
//! - `process_pdf`: base64 decoding, validation, synchronous processing
//! - `process_pdf_stream`: streaming with progress updates
//! - `upload_pdf`: multipart parsing, file upload handling

use crate::error::RiptideError;
use axum::extract::Multipart;
use base64::prelude::*;
use futures::stream::Stream;
use riptide_pdf::types::{ProgressReceiver, ProgressUpdate};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Result type alias for PDF operations
pub type Result<T> = std::result::Result<T, RiptideError>;

const MAX_PDF_SIZE: usize = 50 * 1024 * 1024; // 50MB
const PDF_MAGIC_BYTES: &[u8] = b"%PDF";

/// PDF input data variants
#[derive(Debug, Clone)]
pub enum PdfInput {
    /// Base64 encoded PDF data
    Base64(String),
    /// Raw PDF bytes
    Bytes(Vec<u8>),
}

/// Options for PDF processing
#[derive(Debug, Clone, Default)]
pub struct PdfProcessOptions {
    /// Extract text from PDF
    pub extract_text: bool,
    /// Extract metadata
    pub extract_metadata: bool,
    /// Extract images
    pub extract_images: bool,
    /// Include page numbers
    pub include_page_numbers: bool,
    /// Optional filename
    pub filename: Option<String>,
    /// Optional URL to associate
    pub url: Option<String>,
    /// Timeout override in seconds
    pub timeout: Option<u64>,
}

impl Default for PdfInput {
    fn default() -> Self {
        Self::Bytes(Vec::new())
    }
}

/// Result of PDF processing
#[derive(Debug, Clone)]
pub struct PdfProcessResult {
    /// Extracted document
    pub document: riptide_types::ExtractedDoc,
    /// Processing statistics
    pub stats: ProcessingStats,
}

/// Processing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStats {
    /// Total processing time in milliseconds
    pub processing_time_ms: u64,
    /// File size in bytes
    pub file_size: u64,
    /// Number of pages processed
    pub pages_processed: u32,
    /// Memory used in bytes
    pub memory_used: u64,
    /// Pages per second processing rate
    pub pages_per_second: f64,
    /// Progress callback overhead in microseconds
    pub progress_overhead_us: Option<u64>,
}

/// Metadata extracted from multipart upload
#[derive(Debug, Clone, Default)]
pub struct PdfMetadata {
    /// Original filename
    pub filename: Option<String>,
    /// Associated URL
    pub url: Option<String>,
    /// Content type
    pub content_type: Option<String>,
}

/// PDF processing facade
///
/// Encapsulates all PDF processing business logic, providing a clean API
/// for handlers to use. This facade handles:
/// - Data decoding and validation
/// - PDF processing orchestration
/// - Progress streaming
/// - Multipart form data extraction
pub struct PdfFacade {
    // Note: ResourceManager and metrics are handled by the caller (AppState)
    // This keeps the facade lightweight and composable
}

impl PdfFacade {
    /// Create a new PDF facade
    pub fn new() -> Self {
        Self {}
    }

    /// Process PDF synchronously
    ///
    /// Decodes, validates, and processes a PDF file, returning the extracted content.
    ///
    /// # Arguments
    /// * `pdf_data` - PDF input (base64 or raw bytes)
    /// * `options` - Processing options
    ///
    /// # Returns
    /// PDF processing result with document and statistics
    ///
    /// # Errors
    /// - Invalid base64 encoding
    /// - File too large (>50MB)
    /// - Invalid PDF format
    /// - Processing failure
    pub async fn process_pdf(
        &self,
        pdf_data: PdfInput,
        options: PdfProcessOptions,
    ) -> Result<PdfProcessResult> {
        let start_time = std::time::Instant::now();

        // 1. Decode PDF data
        let bytes = self.decode_pdf_data(pdf_data)?;

        debug!(
            file_size = bytes.len(),
            filename = options.filename.as_deref(),
            "Processing PDF file"
        );

        // 2. Validate PDF
        self.validate_pdf(&bytes)?;

        // 3. Process PDF using riptide-facade's ExtractionFacade
        let pdf_options = crate::facades::PdfExtractionOptions {
            extract_text: options.extract_text,
            extract_metadata: options.extract_metadata,
            extract_images: options.extract_images,
            include_page_numbers: options.include_page_numbers,
        };

        let facade_config = crate::config::RiptideConfig::default();
        let extraction_facade = crate::facades::ExtractionFacade::new(facade_config).await?;

        let extracted = extraction_facade.extract_pdf(&bytes, pdf_options).await?;

        debug!(
            text_length = extracted.text.len(),
            confidence = extracted.confidence,
            "PDF extraction completed successfully"
        );

        // 4. Convert to ExtractedDoc
        let doc = riptide_types::ExtractedDoc {
            url: extracted.url.clone(),
            title: extracted.title.clone(),
            text: extracted.text.clone(),
            quality_score: Some((extracted.confidence * 100.0) as u8),
            links: vec![],
            byline: extracted.metadata.get("author").cloned(),
            published_iso: extracted.metadata.get("publish_date").cloned(),
            markdown: extracted.markdown.clone(),
            media: vec![],
            language: extracted.metadata.get("language").cloned(),
            reading_time: None,
            word_count: Some(extracted.text.split_whitespace().count() as u32),
            categories: vec![],
            site_name: None,
            description: None,
            html: None,
            parser_metadata: None,
        };

        // 5. Build statistics
        let stats = ProcessingStats {
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            file_size: bytes.len() as u64,
            pages_processed: 0, // TODO: Get from PDF processor
            memory_used: 0,     // TODO: Track memory usage
            pages_per_second: 0.0,
            progress_overhead_us: None,
        };

        Ok(PdfProcessResult {
            document: doc,
            stats,
        })
    }

    /// Process PDF with streaming progress updates
    ///
    /// Processes a PDF and returns a stream of progress updates in real-time.
    ///
    /// # Arguments
    /// * `pdf_data` - PDF input (base64 or raw bytes)
    /// * `options` - Processing options
    ///
    /// # Returns
    /// Tuple of (progress stream, file size, filename)
    ///
    /// # Errors
    /// - Invalid base64 encoding
    /// - File too large (>50MB)
    /// - Invalid PDF format
    pub async fn process_pdf_stream(
        &self,
        pdf_data: PdfInput,
        options: PdfProcessOptions,
    ) -> Result<(
        ProgressReceiver,
        u64,            // file_size
        Option<String>, // filename
    )> {
        // 1. Decode PDF data
        let bytes = self.decode_pdf_data(pdf_data)?;

        debug!(
            file_size = bytes.len(),
            filename = options.filename.as_deref(),
            "Starting streaming PDF processing"
        );

        // 2. Validate PDF
        self.validate_pdf(&bytes)?;

        // 3. Create progress channel for streaming
        let pdf_integration = riptide_pdf::integration::create_pdf_integration_for_pipeline();
        let (progress_sender, progress_receiver) = pdf_integration.create_progress_channel();

        // 4. Check if file is actually a PDF
        if !pdf_integration.should_process_as_pdf(None, None, Some(&bytes)) {
            return Err(RiptideError::Validation(
                "File does not appear to be a PDF".to_string(),
            ));
        }

        // 5. Spawn PDF processing task
        let pdf_data_clone = bytes.clone();
        tokio::spawn(async move {
            let _ = pdf_integration
                .process_pdf_bytes_with_progress(&pdf_data_clone, progress_sender)
                .await;
        });

        let file_size = bytes.len() as u64;
        let filename = options.filename;

        Ok((progress_receiver, file_size, filename))
    }

    /// Process multipart PDF upload
    ///
    /// Extracts PDF data from multipart form data and processes it.
    ///
    /// # Arguments
    /// * `multipart` - Multipart form data stream
    ///
    /// # Returns
    /// PDF processing result with document and statistics
    ///
    /// # Errors
    /// - No file provided
    /// - Invalid multipart data
    /// - File too large
    /// - Invalid PDF format
    /// - Processing failure
    pub async fn process_multipart(&self, mut multipart: Multipart) -> Result<PdfProcessResult> {
        // 1. Extract PDF data and metadata from multipart
        let (pdf_bytes, metadata) = self.extract_multipart_data(&mut multipart).await?;

        debug!(
            file_size = pdf_bytes.len(),
            filename = metadata.filename.as_deref(),
            url = metadata.url.as_deref(),
            "Processing uploaded PDF file"
        );

        // 2. Process using standard process_pdf
        let options = PdfProcessOptions {
            extract_text: true,
            extract_metadata: true,
            extract_images: false,
            include_page_numbers: true,
            filename: metadata.filename,
            url: metadata.url,
            timeout: None,
        };

        self.process_pdf(PdfInput::Bytes(pdf_bytes), options).await
    }

    /// Decode PDF data from input variant
    fn decode_pdf_data(&self, input: PdfInput) -> Result<Vec<u8>> {
        match input {
            PdfInput::Bytes(bytes) => Ok(bytes),
            PdfInput::Base64(encoded) => BASE64_STANDARD
                .decode(&encoded)
                .map_err(|e| RiptideError::Validation(format!("Invalid base64 PDF data: {}", e))),
        }
    }

    /// Validate PDF file
    fn validate_pdf(&self, bytes: &[u8]) -> Result<()> {
        // Validate file size
        if bytes.len() > MAX_PDF_SIZE {
            return Err(RiptideError::Validation(
                "PDF file too large (max 50MB)".to_string(),
            ));
        }

        // Validate PDF magic bytes
        if bytes.len() < 5 || &bytes[0..4] != PDF_MAGIC_BYTES {
            return Err(RiptideError::Validation(
                "File does not appear to be a valid PDF".to_string(),
            ));
        }

        Ok(())
    }

    /// Extract PDF data and metadata from multipart form data
    async fn extract_multipart_data(
        &self,
        multipart: &mut Multipart,
    ) -> Result<(Vec<u8>, PdfMetadata)> {
        let mut pdf_data: Option<Vec<u8>> = None;
        let mut metadata = PdfMetadata::default();

        // Process multipart fields
        while let Some(field) = multipart.next_field().await.map_err(|e| {
            RiptideError::Validation(format!("Failed to read multipart field: {}", e))
        })? {
            let field_name = field.name().unwrap_or("").to_string();

            match field_name.as_str() {
                "file" => {
                    let content_type = field.content_type().unwrap_or("").to_string();
                    let field_filename = field.file_name().map(|s| s.to_string());

                    // Read file data
                    let data = field.bytes().await.map_err(|e| {
                        RiptideError::Validation(format!("Failed to read file data: {}", e))
                    })?;

                    // Validate file size
                    if data.len() > MAX_PDF_SIZE {
                        return Err(RiptideError::Validation(
                            "PDF file too large (max 50MB)".to_string(),
                        ));
                    }

                    if data.is_empty() {
                        return Err(RiptideError::Validation(
                            "Uploaded file is empty".to_string(),
                        ));
                    }

                    // Validate PDF magic bytes
                    if data.len() < 5 || &data[0..4] != PDF_MAGIC_BYTES {
                        return Err(RiptideError::Validation(
                            "File does not appear to be a valid PDF".to_string(),
                        ));
                    }

                    // Validate content type if provided
                    if !content_type.is_empty()
                        && !content_type.contains("application/pdf")
                        && !content_type.contains("application/octet-stream")
                    {
                        warn!(
                            content_type = %content_type,
                            "Unexpected content type for PDF upload, accepting based on magic bytes"
                        );
                    }

                    pdf_data = Some(data.to_vec());
                    metadata.content_type = Some(content_type);
                    if metadata.filename.is_none() {
                        metadata.filename = field_filename;
                    }

                    debug!(
                        file_size = data.len(),
                        filename = metadata.filename.as_deref(),
                        content_type = metadata.content_type.as_deref(),
                        "Received PDF file upload"
                    );
                }
                "filename" => {
                    let value = field.text().await.map_err(|e| {
                        RiptideError::Validation(format!("Failed to read filename field: {}", e))
                    })?;
                    if !value.is_empty() {
                        metadata.filename = Some(value);
                    }
                }
                "url" => {
                    let value = field.text().await.map_err(|e| {
                        RiptideError::Validation(format!("Failed to read url field: {}", e))
                    })?;
                    if !value.is_empty() {
                        metadata.url = Some(value);
                    }
                }
                _ => {
                    debug!(field_name = %field_name, "Ignoring unknown multipart field");
                }
            }
        }

        // Ensure we received a file
        let pdf_data = pdf_data.ok_or_else(|| {
            RiptideError::Validation("No PDF file provided in 'file' field".to_string())
        })?;

        Ok((pdf_data, metadata))
    }

    /// Create enhanced progress stream with metrics
    ///
    /// This wraps a progress receiver with additional metrics tracking.
    pub fn create_enhanced_stream(
        progress_receiver: ProgressReceiver,
        start_time: std::time::Instant,
        _file_size: u64,
    ) -> impl Stream<Item = EnhancedProgressUpdate> {
        let mut pages_processed = 0u32;
        let mut progress_callback_start: Option<std::time::Instant> = None;
        let mut total_progress_overhead_us = 0u64;
        let mut progress_count = 0u64;

        async_stream::stream! {
            let mut receiver = progress_receiver;
            while let Some(update) = receiver.recv().await {
                let callback_start = std::time::Instant::now();

                // Track progress callback overhead
                if let Some(start) = progress_callback_start {
                    total_progress_overhead_us += start.elapsed().as_micros() as u64;
                    progress_count += 1;
                }
                progress_callback_start = Some(callback_start);

                // Update metrics based on progress type
                match &update {
                    ProgressUpdate::Progress(progress) => {
                        pages_processed = progress.current_page;

                        // Calculate pages per second
                        let elapsed = start_time.elapsed();
                        let pages_per_second = if elapsed.as_secs() > 0 {
                            pages_processed as f64 / elapsed.as_secs_f64()
                        } else {
                            0.0
                        };

                        // Yield enhanced progress update
                        yield EnhancedProgressUpdate {
                            update: update.clone(),
                            pages_per_second,
                            average_progress_overhead_us: if progress_count > 0 {
                                Some(total_progress_overhead_us / progress_count)
                            } else {
                                None
                            },
                            memory_usage_mb: None,
                        };
                    }
                    ProgressUpdate::Completed { .. } => {
                        // Calculate final statistics
                        let total_time = start_time.elapsed();
                        let pages_per_second = if total_time.as_secs() > 0 {
                            pages_processed as f64 / total_time.as_secs_f64()
                        } else {
                            0.0
                        };

                        yield EnhancedProgressUpdate {
                            update: update.clone(),
                            pages_per_second,
                            average_progress_overhead_us: if progress_count > 0 {
                                Some(total_progress_overhead_us / progress_count)
                            } else {
                                None
                            },
                            memory_usage_mb: None,
                        };
                        break;
                    }
                    ProgressUpdate::Failed { .. } => {
                        yield EnhancedProgressUpdate {
                            update: update.clone(),
                            pages_per_second: 0.0,
                            average_progress_overhead_us: if progress_count > 0 {
                                Some(total_progress_overhead_us / progress_count)
                            } else {
                                None
                            },
                            memory_usage_mb: None,
                        };
                        break;
                    }
                    _ => {
                        yield EnhancedProgressUpdate {
                            update: update.clone(),
                            pages_per_second: 0.0,
                            average_progress_overhead_us: None,
                            memory_usage_mb: None,
                        };
                    }
                }
            }

            // Send final keep-alive to ensure stream completion
            yield EnhancedProgressUpdate {
                update: ProgressUpdate::KeepAlive {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
                pages_per_second: 0.0,
                average_progress_overhead_us: None,
                memory_usage_mb: None,
            };
        }
    }
}

impl Default for PdfFacade {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced progress update with performance metrics
#[derive(Debug, Clone, Serialize)]
pub struct EnhancedProgressUpdate {
    /// The original progress update
    #[serde(flatten)]
    pub update: ProgressUpdate,
    /// Current processing rate (pages per second)
    pub pages_per_second: f64,
    /// Average progress callback overhead in microseconds
    pub average_progress_overhead_us: Option<u64>,
    /// Current memory usage in MB (if available)
    pub memory_usage_mb: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_magic_bytes_validation() {
        let facade = PdfFacade::new();

        // Valid PDF
        let valid_pdf = b"%PDF-1.4\n";
        assert!(facade.validate_pdf(valid_pdf).is_ok());

        // Invalid PDF
        let invalid_pdf = b"<html>";
        assert!(facade.validate_pdf(invalid_pdf).is_err());

        // Too small
        let too_small = b"%PD";
        assert!(facade.validate_pdf(too_small).is_err());
    }

    #[test]
    fn test_pdf_size_validation() {
        let facade = PdfFacade::new();

        // Create a file that's too large (>50MB)
        let mut too_large = vec![0u8; MAX_PDF_SIZE + 1];
        too_large[0..4].copy_from_slice(PDF_MAGIC_BYTES);

        assert!(facade.validate_pdf(&too_large).is_err());

        // Valid size
        let valid_size = b"%PDF-1.4\nSome content";
        assert!(facade.validate_pdf(valid_size).is_ok());
    }

    #[tokio::test]
    async fn test_decode_pdf_data() {
        let facade = PdfFacade::new();

        // Test raw bytes
        let raw_bytes = vec![1, 2, 3, 4];
        let result = facade
            .decode_pdf_data(PdfInput::Bytes(raw_bytes.clone()))
            .unwrap();
        assert_eq!(result, raw_bytes);

        // Test base64
        let base64_str = BASE64_STANDARD.encode(&raw_bytes);
        let result = facade
            .decode_pdf_data(PdfInput::Base64(base64_str))
            .unwrap();
        assert_eq!(result, raw_bytes);

        // Test invalid base64
        let invalid_base64 = "not valid base64!!!";
        assert!(facade
            .decode_pdf_data(PdfInput::Base64(invalid_base64.to_string()))
            .is_err());
    }
}
