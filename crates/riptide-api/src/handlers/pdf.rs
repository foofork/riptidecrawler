//! Ultra-thin PDF processing handlers
//!
//! Handlers delegate business logic to riptide-facade::PdfFacade.
//! Responsible only for HTTP mapping, resource management, and metrics.

use crate::{dto::pdf::*, errors::ApiError, resource_manager::ResourceResult, context::ApplicationContext};
use axum::{
    extract::{Multipart, State},
    response::{sse::Event, Sse},
    Json,
};
use futures_util::stream::Stream;
use futures_util::StreamExt;
use std::convert::Infallible;
use tokio_stream::wrappers::UnboundedReceiverStream;

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
    State(state): State<ApplicationContext>,
    Json(req): Json<PdfProcessRequest>,
) -> Result<Json<PdfProcessResponse>, ApiError> {
    let pdf_data = req
        .pdf_data
        .clone()
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
            state.transport_metrics.record_http_error();
            tracing::error!(error = %e, "PDF extraction failed");
            Err(ApiError::from(e))
        }
    }
}

/// Process PDF with streaming - 15 LOC handler (refactored Phase 4.3)
pub async fn process_pdf_stream(
    State(state): State<ApplicationContext>,
    Json(req): Json<PdfProcessRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let pdf_data = req
        .pdf_data
        .clone()
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
            state.transport_metrics.record_http_error();
            tracing::error!(error = %e, "PDF streaming failed");
            ApiError::from(e)
        })?;

    // Convert progress stream to SSE events (thin wrapper, no business logic)
    let sse_stream = UnboundedReceiverStream::new(progress_stream).map(|item| {
        Ok(Event::default()
            .json_data(item)
            .unwrap_or_else(|_| Event::default().data("error")))
    });

    Ok(Sse::new(sse_stream))
}

/// Upload and process PDF - 9 LOC handler
pub async fn process_pdf_upload(
    State(state): State<ApplicationContext>,
    mut multipart: Multipart,
) -> Result<Json<PdfProcessResponse>, ApiError> {
    let mut pdf_data = None;
    let mut filename = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::validation(format!("Invalid multipart data: {}", e)))?
    {
        match field.name() {
            Some("pdf") => {
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| ApiError::validation(format!("Failed to read PDF: {}", e)))?;
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
                        .map_err(|e| ApiError::validation(format!("Invalid filename: {}", e)))?,
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
    state: &ApplicationContext,
) -> Result<crate::resource_manager::PdfResourceGuard, ApiError> {
    // Use ResourceFacade for unified resource coordination (Sprint 4.4 - Phase 5 Complete)
    use riptide_facade::facades::ResourceResult as FacadeResult;

    // Acquire WASM slot through facade (handles rate limiting, memory pressure, pool coordination)
    let tenant_id = "pdf-processing"; // TODO: Extract from request context in Phase 6
    match state.resource_facade.acquire_wasm_slot(tenant_id).await {
        Ok(FacadeResult::Success(_slot)) => {
            // ResourceFacade has validated all preconditions
            // Now acquire PDF semaphore resources directly (legacy path until Phase 6)
            match state.resource_manager.acquire_pdf_resources().await {
                Ok(ResourceResult::Success(guard)) => Ok(guard),
                Ok(ResourceResult::RateLimited { retry_after }) => {
                    state.transport_metrics.record_http_error();
                    Err(ApiError::rate_limited(format!(
                        "Rate limit exceeded, retry after {:?}",
                        retry_after
                    )))
                }
                Ok(ResourceResult::Timeout) => {
                    state.transport_metrics.record_http_error();
                    Err(ApiError::timeout(
                        "Resource acquisition",
                        "Timeout waiting for PDF processing resources",
                    ))
                }
                Ok(ResourceResult::ResourceExhausted) => {
                    state.transport_metrics.record_http_error();
                    Err(ApiError::internal("PDF processing resources exhausted"))
                }
                Ok(ResourceResult::MemoryPressure) => {
                    state.transport_metrics.record_http_error();
                    Err(ApiError::internal("System under memory pressure"))
                }
                Ok(ResourceResult::Error(e)) => {
                    state.transport_metrics.record_http_error();
                    Err(ApiError::internal(format!("Resource error: {}", e)))
                }
                Err(e) => {
                    state.transport_metrics.record_http_error();
                    Err(ApiError::internal(format!(
                        "Resource acquisition failed: {}",
                        e
                    )))
                }
            }
        }
        Ok(FacadeResult::RateLimited { retry_after }) => {
            state.transport_metrics.record_http_error();
            Err(ApiError::rate_limited(format!(
                "Rate limit exceeded, retry after {:?}",
                retry_after
            )))
        }
        Ok(FacadeResult::MemoryPressure) => {
            state.transport_metrics.record_http_error();
            Err(ApiError::internal("System under memory pressure"))
        }
        Ok(FacadeResult::ResourceExhausted) => {
            state.transport_metrics.record_http_error();
            Err(ApiError::internal("WASM resources exhausted"))
        }
        Ok(FacadeResult::Timeout) => {
            state.transport_metrics.record_http_error();
            Err(ApiError::timeout(
                "Resource acquisition",
                "Timeout acquiring WASM slot",
            ))
        }
        Err(e) => {
            state.transport_metrics.record_http_error();
            Err(ApiError::internal(format!("Resource facade error: {}", e)))
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
                url: "http://test.com".to_string(),
                title: Some("Test".to_string()),
                text: "Test content".to_string(),
                quality_score: Some(85),
                links: vec![],
                byline: None,
                published_iso: None,
                markdown: Some("Test".to_string()),
                media: vec![],
                language: Some("en".to_string()),
                reading_time: Some(1),
                word_count: Some(10),
                categories: vec![],
                site_name: None,
                description: None,
                html: None,
                parser_metadata: None,
            }),
            error: None,
            stats: ProcessingStats {
                processing_time_ms: 100,
                file_size: 1024,
                pages_processed: 1,
                memory_used: 512,
                pages_per_second: 10.0,
                progress_overhead_us: Some(50),
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
                processing_time_ms: 50,
                file_size: 0,
                pages_processed: 0,
                memory_used: 0,
                pages_per_second: 0.0,
                progress_overhead_us: None,
            },
        };
        assert!(!response.success);
        assert!(response.document.is_none());
        assert!(response.error.is_some());
    }
}
