//! PDF processing routes configuration
//!
//! This module defines the HTTP routes for PDF processing with progress tracking.

use crate::{handlers::pdf, state::AppState};
use axum::{
    routing::{get, post},
    Router,
};

/// Configure PDF processing routes
///
/// Provides endpoints for:
/// - Synchronous PDF processing (JSON)
/// - Multipart PDF file upload and processing
/// - Streaming PDF processing with real-time progress
/// - Health check for PDF processing capabilities
pub fn pdf_routes() -> Router<AppState> {
    Router::new()
        // Synchronous PDF processing endpoint (JSON body)
        .route("/process", post(pdf::process_pdf))
        // Multipart PDF file upload endpoint
        .route("/upload", post(pdf::process_pdf_upload))
        // Streaming PDF processing with NDJSON progress updates
        .route("/process-stream", post(pdf::process_pdf_stream))
        // Health check for PDF processing capabilities
        .route("/healthz", get(pdf::pdf_health_check))
}
