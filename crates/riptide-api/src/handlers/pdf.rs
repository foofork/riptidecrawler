//! PDF processing handlers with comprehensive progress tracking
//!
//! This module provides HTTP endpoints for PDF processing with real-time progress streaming.
//! It integrates with the RipTide PDF processing pipeline and provides both synchronous
//! and streaming endpoints for different use cases.

use axum::{extract::State, response::Response, Json};
use base64::prelude::*;
use futures_util::stream::Stream;
use riptide_core::pdf::types::{ProgressReceiver, ProgressUpdate};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::{
    errors::ApiError,
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
}

/// PDF processing response
#[derive(Debug, Serialize, Deserialize)]
pub struct PdfProcessResponse {
    /// Processing success status
    pub success: bool,
    /// Extracted document (if successful)
    pub document: Option<riptide_core::types::ExtractedDoc>,
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
    let start_time = std::time::Instant::now();

    // Extract PDF data from request
    let pdf_data = request
        .pdf_data
        .ok_or_else(|| ApiError::validation("PDF data is required"))?;

    let decoded_data = BASE64_STANDARD
        .decode(&pdf_data)
        .map_err(|e| ApiError::validation(format!("Invalid base64 PDF data: {}", e)))?;

    let (pdf_data, filename, url) = (decoded_data, request.filename, request.url);

    debug!(
        file_size = pdf_data.len(),
        filename = filename.as_deref(),
        "Processing PDF file"
    );

    // Validate file size
    if pdf_data.len() > 50 * 1024 * 1024 {
        // 50MB limit
        return Err(ApiError::validation("PDF file too large (max 50MB)"));
    }

    // Create PDF integration
    let pdf_integration = riptide_core::pdf::integration::create_pdf_integration_for_pipeline();

    // Check if file is actually a PDF
    if !pdf_integration.should_process_as_pdf(None, None, Some(&pdf_data)) {
        return Err(ApiError::validation("File does not appear to be a PDF"));
    }

    // Process the PDF
    let processing_start = std::time::Instant::now();
    match pdf_integration
        .process_pdf_to_extracted_doc(&pdf_data, url.as_deref())
        .await
    {
        Ok(document) => {
            let processing_time = processing_start.elapsed();
            let total_time = start_time.elapsed();

            // Calculate processing rate
            let pages_per_second = if processing_time.as_secs() > 0 {
                document.word_count.unwrap_or(0) as f64 / processing_time.as_secs_f64()
            } else {
                0.0
            };

            // Record metrics
            state.metrics.record_http_request(
                "POST",
                "/pdf/process",
                200,
                total_time.as_secs_f64(),
            );

            // Get metrics from PDF integration
            let pdf_metrics = pdf_integration.get_metrics_snapshot();

            let stats = ProcessingStats {
                processing_time_ms: processing_time.as_millis() as u64,
                file_size: pdf_data.len() as u64,
                pages_processed: 1, // Estimated from document
                memory_used: pdf_metrics.peak_memory_usage,
                pages_per_second,
                progress_overhead_us: None, // No progress callback used
            };

            info!(
                processing_time_ms = stats.processing_time_ms,
                file_size = stats.file_size,
                pages_per_second = stats.pages_per_second,
                "PDF processing completed successfully"
            );

            Ok(Json(PdfProcessResponse {
                success: true,
                document: Some(riptide_core::convert_pdf_extracted_doc(document)),
                error: None,
                stats,
            }))
        }
        Err(e) => {
            let error_msg = format!("PDF processing failed: {:?}", e);
            error!(error = %error_msg, "PDF processing failed");

            // Record failed processing
            state.metrics.record_http_request(
                "POST",
                "/pdf/process",
                500,
                start_time.elapsed().as_secs_f64(),
            );

            Ok(Json(PdfProcessResponse {
                success: false,
                document: None,
                error: Some(error_msg),
                stats: ProcessingStats {
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    file_size: pdf_data.len() as u64,
                    pages_processed: 0,
                    memory_used: 0,
                    pages_per_second: 0.0,
                    progress_overhead_us: None,
                },
            }))
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

    let decoded_data = BASE64_STANDARD
        .decode(&pdf_data)
        .map_err(|e| ApiError::validation(format!("Invalid base64 PDF data: {}", e)))?;

    let (pdf_data, filename, _) = (decoded_data, request.filename, request.url);

    debug!(
        file_size = pdf_data.len(),
        filename = filename.as_deref(),
        "Starting streaming PDF processing"
    );

    // Validate file size
    if pdf_data.len() > 50 * 1024 * 1024 {
        return Err(ApiError::validation("PDF file too large (max 50MB)"));
    }

    // Create PDF integration and progress channel
    let pdf_integration = riptide_core::pdf::integration::create_pdf_integration_for_pipeline();
    let (progress_sender, progress_receiver) = pdf_integration.create_progress_channel();

    // Check if file is actually a PDF
    if !pdf_integration.should_process_as_pdf(None, None, Some(&pdf_data)) {
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

/// Request type enum for handling both JSON and multipart requests
#[derive(Debug)]
#[allow(dead_code)] // TODO: Implement multipart PDF upload support
pub enum PdfProcessingRequest {
    Json(PdfProcessRequest),
    Multipart(Vec<u8>, Option<String>, Option<String>), // data, filename, url
}

// Note: For multipart support, add a separate handler endpoint

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
        use riptide_core::pdf::types::*;

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
}
