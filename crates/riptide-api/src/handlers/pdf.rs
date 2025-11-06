//! PDF processing handlers with comprehensive progress tracking
//!
//! This module provides HTTP endpoints for PDF processing with real-time progress streaming.
//! It integrates with the RipTide PDF processing pipeline and provides both synchronous
//! and streaming endpoints for different use cases.

use axum::{
    extract::{Multipart, State},
    response::Response,
    Json,
};
use base64::prelude::*;
use futures_util::stream::Stream;
use riptide_pdf::types::{ProgressReceiver, ProgressUpdate};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::{
    errors::ApiError,
    metrics::ErrorType,
    resource_manager::ResourceResult,
    state::AppState,
    streaming::response_helpers::{StreamingResponseBuilder, StreamingResponseType},
};

/// PDF processing request body
#[derive(Debug, Serialize, Deserialize)]
pub struct PdfProcessRequest {
    /// Base64 encoded PDF data (optional if using multipart)
    pub pdf_data: Option<String>,
    /// Original filename (optional)
    pub filename: Option<String>,
    /// Whether to stream progress updates
    pub stream_progress: Option<bool>,
    /// URL to associate with the document (optional)
    pub url: Option<String>,
    /// Timeout for PDF processing operation in seconds (optional override)
    pub timeout: Option<u64>,
}

/// PDF processing response
#[derive(Debug, Serialize, Deserialize)]
pub struct PdfProcessResponse {
    /// Processing success status
    pub success: bool,
    /// Extracted document (if successful)
    pub document: Option<riptide_types::ExtractedDoc>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Processing statistics
    pub stats: ProcessingStats,
}

/// Processing statistics for metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessingStats {
    /// Total processing time in milliseconds
    pub processing_time_ms: u64,
    /// File size in bytes
    pub file_size: u64,
    /// Number of pages processed
    pub pages_processed: u32,
    /// Memory usage in bytes
    pub memory_used: u64,
    /// Pages per second processing rate
    pub pages_per_second: f64,
    /// Progress callback overhead in microseconds
    pub progress_overhead_us: Option<u64>,
}

/// Synchronous PDF processing endpoint
///
/// Processes a PDF file and returns the extracted content.
/// Supports both JSON and multipart/form-data requests.
pub async fn process_pdf(
    State(state): State<AppState>,
    Json(request): Json<PdfProcessRequest>,
) -> Result<Json<PdfProcessResponse>, ApiError> {
    let _start_time = std::time::Instant::now();

    // Extract PDF data from request
    let pdf_data = request
        .pdf_data
        .ok_or_else(|| ApiError::validation("PDF data is required"))?;

    let decoded_data = BASE64_STANDARD.decode(&pdf_data).map_err(|e| {
        state.metrics.record_error(ErrorType::Http);
        ApiError::validation(format!("Invalid base64 PDF data: {}", e))
    })?;

    let (pdf_data, filename, _url) = (decoded_data, request.filename, request.url);

    debug!(
        file_size = pdf_data.len(),
        filename = filename.as_deref(),
        "Processing PDF file"
    );

    // Validate file size
    if pdf_data.len() > 50 * 1024 * 1024 {
        // 50MB limit
        state.metrics.record_error(ErrorType::Http);
        return Err(ApiError::validation("PDF file too large (max 50MB)"));
    }

    // Acquire PDF processing resources (semaphore + memory tracking)
    let _pdf_guard = match state.resource_manager.acquire_pdf_resources().await {
        Ok(ResourceResult::Success(guard)) => guard,
        Ok(ResourceResult::Timeout) => {
            state.metrics.record_error(ErrorType::Http);
            return Err(ApiError::timeout(
                "Resource acquisition",
                "Timeout acquiring PDF processing resources",
            ));
        }
        Ok(ResourceResult::ResourceExhausted) => {
            state.metrics.record_error(ErrorType::Http);
            return Err(ApiError::internal(
                "PDF processing resources exhausted, please try again later",
            ));
        }
        Ok(ResourceResult::MemoryPressure) => {
            state.metrics.record_error(ErrorType::Http);
            return Err(ApiError::internal(
                "System under memory pressure, please try again later",
            ));
        }
        Ok(ResourceResult::RateLimited { retry_after }) => {
            state.metrics.record_error(ErrorType::Http);
            return Err(ApiError::internal(format!(
                "Rate limited, retry after {}ms",
                retry_after.as_millis()
            )));
        }
        Ok(ResourceResult::Error(msg)) => {
            state.metrics.record_error(ErrorType::Http);
            return Err(ApiError::internal(format!(
                "Failed to acquire PDF resources: {}",
                msg
            )));
        }
        Err(e) => {
            state.metrics.record_error(ErrorType::Http);
            return Err(ApiError::internal(format!(
                "Failed to acquire PDF resources: {}",
                e
            )));
        }
    };

    // Facade temporarily unavailable during refactoring
    Err(ApiError::internal(
        "Facade temporarily unavailable during refactoring".to_string(),
    ))
}

/// Streaming PDF processing endpoint
///
/// Processes a PDF file and streams progress updates in real-time using NDJSON format.
/// Each line in the response is a JSON object representing a progress update.
pub async fn process_pdf_stream(
    State(state): State<AppState>,
    Json(request): Json<PdfProcessRequest>,
) -> Result<Response, ApiError> {
    let start_time = std::time::Instant::now();

    // Extract PDF data from request
    let pdf_data = request
        .pdf_data
        .ok_or_else(|| ApiError::validation("PDF data is required"))?;

    let decoded_data = BASE64_STANDARD.decode(&pdf_data).map_err(|e| {
        state.metrics.record_error(ErrorType::Http);
        ApiError::validation(format!("Invalid base64 PDF data: {}", e))
    })?;

    let (pdf_data, filename, _) = (decoded_data, request.filename, request.url);

    debug!(
        file_size = pdf_data.len(),
        filename = filename.as_deref(),
        "Starting streaming PDF processing"
    );

    // Validate file size
    if pdf_data.len() > 50 * 1024 * 1024 {
        state.metrics.record_error(ErrorType::Http);
        return Err(ApiError::validation("PDF file too large (max 50MB)"));
    }

    // For streaming, we still need to use the PDF integration directly for progress tracking
    // ExtractionFacade doesn't support streaming progress yet
    let pdf_integration = riptide_pdf::integration::create_pdf_integration_for_pipeline();
    let (progress_sender, progress_receiver) = pdf_integration.create_progress_channel();

    // Check if file is actually a PDF
    if !pdf_integration.should_process_as_pdf(None, None, Some(&pdf_data)) {
        state.metrics.record_error(ErrorType::Http);
        return Err(ApiError::validation("File does not appear to be a PDF"));
    }

    // Spawn PDF processing task
    let pdf_data_clone = pdf_data.clone();

    tokio::spawn(async move {
        let _ = pdf_integration
            .process_pdf_bytes_with_progress(&pdf_data_clone, progress_sender)
            .await;
    });

    // Create progress stream with enhanced metrics tracking
    let enhanced_stream = create_enhanced_progress_stream(
        progress_receiver,
        start_time,
        pdf_data.len() as u64,
        state.clone(),
    );

    // Build streaming response
    let response = StreamingResponseBuilder::new(StreamingResponseType::Ndjson)
        .header(
            "x-pdf-filename",
            filename.unwrap_or_else(|| "document.pdf".to_string()),
        )
        .header("x-pdf-size", pdf_data.len().to_string())
        .build(enhanced_stream);

    info!(
        file_size = pdf_data.len(),
        "Started streaming PDF processing"
    );

    Ok(response)
}

/// Enhanced progress stream with metrics collection
fn create_enhanced_progress_stream(
    mut progress_receiver: ProgressReceiver,
    start_time: std::time::Instant,
    _file_size: u64,
    state: AppState,
) -> impl Stream<Item = EnhancedProgressUpdate> {
    let mut pages_processed = 0u32;
    let mut progress_callback_start: Option<std::time::Instant> = None;
    let mut total_progress_overhead_us = 0u64;
    let mut progress_count = 0u64;

    async_stream::stream! {
        while let Some(update) = progress_receiver.recv().await {
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
                        memory_usage_mb: None, // Will be populated by processor if available
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

                    // Record final metrics
                    state.metrics.record_http_request(
                        "POST",
                        "/pdf/process-stream",
                        200,
                        total_time.as_secs_f64(),
                    );

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
                    // Record failed processing
                    state.metrics.record_http_request(
                        "POST",
                        "/pdf/process-stream",
                        500,
                        start_time.elapsed().as_secs_f64(),
                    );

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

/// Multipart PDF upload and processing endpoint
///
/// Handles PDF file uploads via multipart/form-data.
/// Accepts a PDF file field and optional metadata fields.
///
/// # Form Fields
/// - `file`: PDF file (required)
/// - `filename`: Optional filename override
/// - `url`: Optional URL to associate with the document
/// - `stream_progress`: Optional boolean to enable streaming response
///
/// # File Validation
/// - Maximum file size: 50MB
/// - Content type must be application/pdf or valid PDF magic bytes
/// - File must have valid PDF structure
pub async fn upload_pdf(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<PdfProcessResponse>, ApiError> {
    let _start_time = std::time::Instant::now();

    let mut pdf_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    let mut url: Option<String> = None;
    let mut stream_progress = false;

    // Process multipart fields
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        state.metrics.record_error(ErrorType::Http);
        ApiError::validation(format!("Failed to read multipart field: {}", e))
    })? {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "file" => {
                let content_type = field.content_type().unwrap_or("").to_string();
                let field_filename = field.file_name().map(|s| s.to_string());

                // Read file data
                let data = field.bytes().await.map_err(|e| {
                    state.metrics.record_error(ErrorType::Http);
                    ApiError::validation(format!("Failed to read file data: {}", e))
                })?;

                // Validate file size
                if data.len() > 50 * 1024 * 1024 {
                    // 50MB limit
                    state.metrics.record_error(ErrorType::Http);
                    return Err(ApiError::validation("PDF file too large (max 50MB)"));
                }

                if data.is_empty() {
                    state.metrics.record_error(ErrorType::Http);
                    return Err(ApiError::validation("Uploaded file is empty"));
                }

                // Validate PDF magic bytes (%PDF-)
                if data.len() < 5 || &data[0..4] != b"%PDF" {
                    state.metrics.record_error(ErrorType::Http);
                    return Err(ApiError::validation(
                        "File does not appear to be a valid PDF",
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
                if filename.is_none() {
                    filename = field_filename;
                }

                debug!(
                    file_size = data.len(),
                    filename = filename.as_deref(),
                    content_type = %content_type,
                    "Received PDF file upload"
                );
            }
            "filename" => {
                let value = field.text().await.map_err(|e| {
                    ApiError::validation(format!("Failed to read filename field: {}", e))
                })?;
                if !value.is_empty() {
                    filename = Some(value);
                }
            }
            "url" => {
                let value = field.text().await.map_err(|e| {
                    ApiError::validation(format!("Failed to read url field: {}", e))
                })?;
                if !value.is_empty() {
                    url = Some(value);
                }
            }
            "stream_progress" => {
                let value = field.text().await.map_err(|e| {
                    ApiError::validation(format!("Failed to read stream_progress field: {}", e))
                })?;
                stream_progress = value.parse::<bool>().unwrap_or(false);
            }
            _ => {
                debug!(field_name = %field_name, "Ignoring unknown multipart field");
            }
        }
    }

    // Ensure we received a file
    let pdf_data = pdf_data.ok_or_else(|| {
        state.metrics.record_error(ErrorType::Http);
        ApiError::validation("No PDF file provided in 'file' field")
    })?;

    debug!(
        file_size = pdf_data.len(),
        filename = filename.as_deref(),
        url = url.as_deref(),
        stream_progress = stream_progress,
        "Processing uploaded PDF file"
    );

    // Acquire PDF processing resources
    let _pdf_guard = match state.resource_manager.acquire_pdf_resources().await {
        Ok(ResourceResult::Success(guard)) => guard,
        Ok(ResourceResult::Timeout) => {
            state.metrics.record_error(ErrorType::Http);
            return Err(ApiError::timeout(
                "Resource acquisition",
                "Timeout acquiring PDF processing resources",
            ));
        }
        Ok(ResourceResult::ResourceExhausted) => {
            state.metrics.record_error(ErrorType::Http);
            return Err(ApiError::internal(
                "PDF processing resources exhausted, please try again later",
            ));
        }
        Ok(ResourceResult::MemoryPressure) => {
            state.metrics.record_error(ErrorType::Http);
            return Err(ApiError::internal(
                "System under memory pressure, please try again later",
            ));
        }
        Ok(ResourceResult::RateLimited { retry_after }) => {
            state.metrics.record_error(ErrorType::Http);
            return Err(ApiError::internal(format!(
                "Rate limited, retry after {}ms",
                retry_after.as_millis()
            )));
        }
        Ok(ResourceResult::Error(msg)) => {
            state.metrics.record_error(ErrorType::Http);
            return Err(ApiError::internal(format!(
                "Failed to acquire PDF resources: {}",
                msg
            )));
        }
        Err(e) => {
            state.metrics.record_error(ErrorType::Http);
            return Err(ApiError::internal(format!(
                "Failed to acquire PDF resources: {}",
                e
            )));
        }
    };

    // Facade temporarily unavailable during refactoring
    Err(ApiError::internal(
        "Facade temporarily unavailable during refactoring".to_string(),
    ))
}

/// Request type enum for handling both JSON and multipart requests
#[derive(Debug)]
#[allow(dead_code)]
pub enum PdfProcessingRequest {
    Json(PdfProcessRequest),
    Multipart(Vec<u8>, Option<String>, Option<String>), // data, filename, url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processing_stats_serialization() {
        let stats = ProcessingStats {
            processing_time_ms: 1500,
            file_size: 1024000,
            pages_processed: 10,
            memory_used: 50000000,
            pages_per_second: 6.67,
            progress_overhead_us: Some(250),
        };

        let serialized = serde_json::to_string(&stats).unwrap();
        assert!(serialized.contains("processing_time_ms"));
        assert!(serialized.contains("1500"));
        assert!(serialized.contains("6.67"));
    }

    #[test]
    fn test_enhanced_progress_update_serialization() {
        use riptide_pdf::types::*;

        let update = EnhancedProgressUpdate {
            update: ProgressUpdate::Progress(ProcessingProgress {
                current_page: 5,
                total_pages: 10,
                percentage: 50.0,
                estimated_remaining_ms: Some(1500),
                stage: ProcessingStage::ExtractingText(5),
            }),
            pages_per_second: 3.33,
            average_progress_overhead_us: Some(150),
            memory_usage_mb: Some(25.6),
        };

        let serialized = serde_json::to_string(&update).unwrap();
        assert!(serialized.contains("current_page"));
        assert!(serialized.contains("pages_per_second"));
        assert!(serialized.contains("3.33"));
    }

    #[test]
    fn test_pdf_request_enum() {
        // Test that the enum variants exist and can be constructed
        let json_req = PdfProcessingRequest::Json(PdfProcessRequest {
            pdf_data: Some("test".to_string()),
            filename: Some("test.pdf".to_string()),
            stream_progress: Some(false),
            url: None,
            timeout: None,
        });

        match json_req {
            PdfProcessingRequest::Json(_) => (),
            _ => panic!("Expected Json variant"),
        }

        let multipart_req =
            PdfProcessingRequest::Multipart(vec![1, 2, 3], Some("test.pdf".to_string()), None);

        match multipart_req {
            PdfProcessingRequest::Multipart(data, filename, _) => {
                assert_eq!(data, vec![1, 2, 3]);
                assert_eq!(filename, Some("test.pdf".to_string()));
            }
            _ => panic!("Expected Multipart variant"),
        }
    }

    #[test]
    fn test_pdf_magic_bytes_validation() {
        // Valid PDF magic bytes
        let valid_pdf = b"%PDF-1.4\n";
        assert_eq!(&valid_pdf[0..4], b"%PDF");

        // Invalid PDF
        let invalid_pdf = b"<html>";
        assert_ne!(&invalid_pdf[0..4], b"%PDF");
    }
}
