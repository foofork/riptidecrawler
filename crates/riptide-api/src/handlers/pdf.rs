//! PDF processing handlers with comprehensive progress tracking
//!
//! This module provides HTTP endpoints for PDF processing with real-time progress streaming.
//! It integrates with the RipTide PDF processing pipeline and provides both synchronous
//! and streaming endpoints for different use cases.
//!
//! **Architecture:**
//! - Handlers are thin wrappers around riptide_facade::PdfFacade
//! - All business logic (validation, decoding, processing) is in the facade
//! - Handlers only manage HTTP-specific concerns (resource acquisition, response building)

use axum::{
    extract::{Multipart, State},
    response::Response,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;

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
    pub stats: riptide_facade::facades::ProcessingStats,
}

/// Processing statistics (re-exported from facade)
#[cfg(test)]
pub type ProcessingStats = riptide_facade::facades::ProcessingStats;

/// Synchronous PDF processing endpoint
///
/// Processes a PDF file and returns the extracted content.
/// Supports both JSON and multipart/form-data requests.
pub async fn process_pdf(
    State(state): State<AppState>,
    Json(request): Json<PdfProcessRequest>,
) -> Result<Json<PdfProcessResponse>, ApiError> {
    // Extract PDF data from request
    let pdf_data = request
        .pdf_data
        .ok_or_else(|| ApiError::validation("PDF data is required"))?;

    // Acquire PDF processing resources (semaphore + memory tracking)
    let _pdf_guard = acquire_pdf_resources(&state).await?;

    // Use PdfFacade for all business logic
    let pdf_facade = riptide_facade::facades::PdfFacade::new();
    let options = riptide_facade::facades::PdfProcessOptions {
        extract_text: true,
        extract_metadata: true,
        extract_images: false,
        include_page_numbers: true,
        filename: request.filename,
        url: request.url,
        timeout: request.timeout,
    };

    match pdf_facade
        .process_pdf(riptide_facade::facades::PdfInput::Base64(pdf_data), options)
        .await
    {
        Ok(result) => {
            let response = PdfProcessResponse {
                success: true,
                document: Some(result.document),
                error: None,
                stats: result.stats,
            };
            Ok(Json(response))
        }
        Err(e) => {
            state.metrics.record_error(ErrorType::Http);
            tracing::error!(error = %e, "PDF extraction failed");
            Err(ApiError::from(e))
        }
    }
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

    // Use PdfFacade for streaming logic
    let pdf_facade = riptide_facade::facades::PdfFacade::new();
    let options = riptide_facade::facades::PdfProcessOptions {
        extract_text: true,
        extract_metadata: true,
        extract_images: false,
        include_page_numbers: true,
        filename: request.filename.clone(),
        url: request.url,
        timeout: request.timeout,
    };

    let (progress_receiver, file_size, filename) = pdf_facade
        .process_pdf_stream(riptide_facade::facades::PdfInput::Base64(pdf_data), options)
        .await
        .map_err(|e| {
            state.metrics.record_error(ErrorType::Http);
            ApiError::from(e)
        })?;

    // Create enhanced progress stream using facade
    let enhanced_stream = riptide_facade::facades::PdfFacade::create_enhanced_stream(
        progress_receiver,
        start_time,
        file_size,
    );

    // Build streaming response
    let response = StreamingResponseBuilder::new(StreamingResponseType::Ndjson)
        .header(
            "x-pdf-filename",
            filename.unwrap_or_else(|| "document.pdf".to_string()),
        )
        .header("x-pdf-size", file_size.to_string())
        .build(enhanced_stream);

    info!(file_size = file_size, "Started streaming PDF processing");

    Ok(response)
}

/// Helper function to acquire PDF processing resources
///
/// Handles all resource acquisition error cases and converts them to ApiError.
async fn acquire_pdf_resources(
    state: &AppState,
) -> Result<crate::resource_manager::PdfResourceGuard, ApiError> {
    match state.resource_manager.acquire_pdf_resources().await {
        Ok(ResourceResult::Success(guard)) => Ok(guard),
        Ok(ResourceResult::Timeout) => {
            state.metrics.record_error(ErrorType::Http);
            Err(ApiError::timeout(
                "Resource acquisition",
                "Timeout acquiring PDF processing resources",
            ))
        }
        Ok(ResourceResult::ResourceExhausted) => {
            state.metrics.record_error(ErrorType::Http);
            Err(ApiError::internal(
                "PDF processing resources exhausted, please try again later",
            ))
        }
        Ok(ResourceResult::MemoryPressure) => {
            state.metrics.record_error(ErrorType::Http);
            Err(ApiError::internal(
                "System under memory pressure, please try again later",
            ))
        }
        Ok(ResourceResult::RateLimited { retry_after }) => {
            state.metrics.record_error(ErrorType::Http);
            Err(ApiError::internal(format!(
                "Rate limited, retry after {}ms",
                retry_after.as_millis()
            )))
        }
        Ok(ResourceResult::Error(msg)) => {
            state.metrics.record_error(ErrorType::Http);
            Err(ApiError::internal(format!(
                "Failed to acquire PDF resources: {}",
                msg
            )))
        }
        Err(e) => {
            state.metrics.record_error(ErrorType::Http);
            Err(ApiError::internal(format!(
                "Failed to acquire PDF resources: {}",
                e
            )))
        }
    }
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
///
/// # File Validation
/// - Maximum file size: 50MB
/// - Content type must be application/pdf or valid PDF magic bytes
/// - File must have valid PDF structure
pub async fn upload_pdf(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<Json<PdfProcessResponse>, ApiError> {
    // Acquire PDF processing resources
    let _pdf_guard = acquire_pdf_resources(&state).await?;

    // Use PdfFacade for multipart processing
    let pdf_facade = riptide_facade::facades::PdfFacade::new();

    match pdf_facade.process_multipart(multipart).await {
        Ok(result) => {
            let response = PdfProcessResponse {
                success: true,
                document: Some(result.document),
                error: None,
                stats: result.stats,
            };
            Ok(Json(response))
        }
        Err(e) => {
            state.metrics.record_error(ErrorType::Http);
            tracing::error!(error = %e, "PDF upload processing failed");
            Err(ApiError::from(e))
        }
    }
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
        use riptide_facade::facades::EnhancedProgressUpdate;
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
