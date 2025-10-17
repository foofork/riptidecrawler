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
/// - Synchronous PDF processing
/// - Streaming PDF processing with real-time progress
/// - Health check for PDF processing capabilities
pub fn pdf_routes() -> Router<AppState> {
    Router::new()
        // Synchronous PDF processing endpoint
        .route("/process", post(pdf::process_pdf))
        // Streaming PDF processing with NDJSON progress updates
        .route("/process-stream", post(pdf::process_pdf_stream))
        // Health check for PDF processing capabilities
        .route("/healthz", get(pdf_health_check))
}

/// PDF processing health check endpoint
async fn pdf_health_check() -> axum::response::Json<serde_json::Value> {
    use riptide_core::pdf::integration::create_pdf_integration_for_pipeline;

    let integration = create_pdf_integration_for_pipeline();
    let available = integration.is_available();
    let capabilities = integration.capabilities();

    axum::response::Json(serde_json::json!({
        "status": if available { "healthy" } else { "unavailable" },
        "pdf_processing_available": available,
        "capabilities": {
            "text_extraction": capabilities.text_extraction,
            "image_extraction": capabilities.image_extraction,
            "metadata_extraction": capabilities.metadata_extraction,
            "table_extraction": capabilities.table_extraction,
            "form_extraction": capabilities.form_extraction,
            "encrypted_pdfs": capabilities.encrypted_pdfs,
            "max_file_size_mb": capabilities.max_file_size / (1024 * 1024),
            "supported_versions": capabilities.supported_versions
        },
        "features": {
            "progress_streaming": true,
            "concurrent_processing": true,
            "memory_monitoring": true,
            "performance_metrics": true
        }
    }))
}
