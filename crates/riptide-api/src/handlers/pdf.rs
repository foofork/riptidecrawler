//! Ultra-thin PDF processing handlers
//!
//! Handlers delegate business logic to riptide-facade::PdfFacade.
//! Responsible only for HTTP mapping, resource management, and metrics.

use crate::{
    dto::pdf::*, errors::ApiError, metrics::ErrorType, resource_manager::ResourceResult,
    state::AppState,
};
use axum::{
    extract::{Multipart, State},
    response::{sse::Event, Response, Sse},
    Json,
};
use futures_util::stream::Stream;
use std::convert::Infallible;

/// PDF processing health check endpoint
pub async fn pdf_health_check() -> axum::response::Json<serde_json::Value> {
    use riptide_pdf::integration::create_pdf_integration_for_pipeline;

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

/// Process PDF synchronously - 9 LOC handler
pub async fn process_pdf(
    State(state): State<AppState>,
    Json(req): Json<PdfProcessRequest>,
) -> Result<Json<PdfProcessResponse>, ApiError> {
    let pdf_data = req
        .pdf_data
        .ok_or_else(|| ApiError::validation("PDF data is required"))?;
    let _guard = acquire_pdf_resources(&state).await?;
    let facade = riptide_facade::facades::PdfFacade::new();
    match facade
        .process_pdf(
            riptide_facade::facades::PdfInput::Base64(pdf_data),
            req.to_facade_options(),
        )
        .await
    {
        Ok(result) => Ok(Json(PdfProcessResponse {
            success: true,
            document: Some(result.document),
            error: None,
            stats: result.stats,
        })),
        Err(e) => {
            state.metrics.record_error(ErrorType::Http);
            tracing::error!(error = %e, "PDF extraction failed");
            Err(ApiError::from(e))
        }
    }
}

/// Process PDF with streaming - 15 LOC handler (refactored Phase 4.3)
pub async fn process_pdf_stream(
    State(state): State<AppState>,
    Json(req): Json<PdfProcessRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let pdf_data = req
        .pdf_data
        .ok_or_else(|| ApiError::validation("PDF data is required"))?;
    let _guard = acquire_pdf_resources(&state).await?;
    let facade = riptide_facade::facades::PdfFacade::new();
    let (progress_stream, _file_size, _filename) = facade
        .process_pdf_stream(
            riptide_facade::facades::PdfInput::Base64(pdf_data),
            req.to_facade_options(),
        )
        .await
        .map_err(|e| {
            state.metrics.record_error(ErrorType::Http);
            tracing::error!(error = %e, "PDF streaming failed");
            ApiError::from(e)
        })?;

    // Convert progress stream to SSE events (thin wrapper, no business logic)
    let sse_stream = progress_stream.map(|item| {
        Ok(Event::default()
            .json_data(item)
            .unwrap_or_else(|_| Event::default().data("error")))
    });

    Ok(Sse::new(sse_stream))
}

/// Upload and process PDF - 9 LOC handler
pub async fn process_pdf_upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<PdfProcessResponse>, ApiError> {
    let mut pdf_data = None;
    let mut filename = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::validation(&format!("Invalid multipart data: {}", e)))?
    {
        match field.name() {
            Some("pdf") => {
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| ApiError::validation(&format!("Failed to read PDF: {}", e)))?;
                pdf_data = Some(base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    &bytes,
                ));
            }
            Some("filename") => {
                filename = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| ApiError::validation(&format!("Invalid filename: {}", e)))?,
                )
            }
            _ => {}
        }
    }
    let pdf_data = pdf_data.ok_or_else(|| ApiError::validation("No PDF file provided"))?;
    let req = PdfProcessRequest {
        pdf_data: Some(pdf_data),
        filename,
        stream_progress: None,
        url: None,
        timeout: None,
    };
    process_pdf(State(state), Json(req)).await
}

async fn acquire_pdf_resources(
    state: &AppState,
) -> Result<riptide_resource::PdfResourceGuard, ApiError> {
    // Use ResourceFacade for unified resource coordination (Sprint 4.4)
    use riptide_facade::facades::ResourceResult as FacadeResult;

    // Acquire WASM slot through facade (uses tenant_id for rate limiting)
    let tenant_id = "pdf-processing"; // TODO: Extract from request context
    match state.resource_facade.acquire_wasm_slot(tenant_id).await {
        Ok(FacadeResult::Success(_slot)) => {
            // Now acquire PDF resources through resource manager
            // The facade has already handled rate limiting and memory pressure
            match state.resource_manager.acquire_pdf_resources().await {
                Ok(crate::resource_manager::ResourceResult::Success(guard)) => Ok(guard),
                Ok(crate::resource_manager::ResourceResult::Timeout) => {
                    state.metrics.record_error(ErrorType::Http);
                    Err(ApiError::timeout(
                        "Resource acquisition",
                        "Timeout waiting for PDF processing resources",
                    ))
                }
                Ok(crate::resource_manager::ResourceResult::ResourceExhausted) => {
                    state.metrics.record_error(ErrorType::Http);
                    Err(ApiError::internal("PDF processing resources exhausted"))
                }
                Ok(crate::resource_manager::ResourceResult::MemoryPressure) => {
                    state.metrics.record_error(ErrorType::Http);
                    Err(ApiError::internal("System under memory pressure"))
                }
                Err(e) => {
                    state.metrics.record_error(ErrorType::Http);
                    Err(ApiError::internal(&format!(
                        "Resource acquisition failed: {}",
                        e
                    )))
                }
            }
        }
        Ok(FacadeResult::RateLimited { retry_after }) => {
            state.metrics.record_error(ErrorType::Http);
            Err(ApiError::rate_limited(&format!(
                "Rate limit exceeded, retry after {:?}",
                retry_after
            )))
        }
        Ok(FacadeResult::MemoryPressure) => {
            state.metrics.record_error(ErrorType::Http);
            Err(ApiError::internal("System under memory pressure"))
        }
        Ok(FacadeResult::ResourceExhausted) => {
            state.metrics.record_error(ErrorType::Http);
            Err(ApiError::internal("WASM resources exhausted"))
        }
        Ok(FacadeResult::Timeout) => {
            state.metrics.record_error(ErrorType::Http);
            Err(ApiError::timeout(
                "Resource acquisition",
                "Timeout acquiring WASM slot",
            ))
        }
        Err(e) => {
            state.metrics.record_error(ErrorType::Http);
            Err(ApiError::internal(&format!("Resource facade error: {}", e)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::pdf::ProcessingStats;
    use riptide_types::ExtractedDoc;
    #[tokio::test]
    async fn test_pdf_response_structure() {
        let response = PdfProcessResponse {
            success: true,
            document: Some(ExtractedDoc {
                markdown: "Test".into(),
                metadata: None,
                stats: None,
            }),
            error: None,
            stats: ProcessingStats {
                total_pages: 1,
                pages_processed: 1,
                text_extracted: true,
                metadata_extracted: true,
                images_extracted: false,
                processing_time_ms: 100,
                engine_used: "test".into(),
                errors: vec![],
            },
        };
        assert!(response.success);
        assert!(response.document.is_some());
        assert!(response.error.is_none());
    }
    #[tokio::test]
    async fn test_pdf_error_response() {
        let response = PdfProcessResponse {
            success: false,
            document: None,
            error: Some("Test error".into()),
            stats: ProcessingStats {
                total_pages: 0,
                pages_processed: 0,
                text_extracted: false,
                metadata_extracted: false,
                images_extracted: false,
                processing_time_ms: 50,
                engine_used: "test".into(),
                errors: vec!["Test error".into()],
            },
        };
        assert!(!response.success);
        assert!(response.document.is_none());
        assert!(response.error.is_some());
    }
}
