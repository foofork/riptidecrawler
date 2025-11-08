//! Minimal render handlers delegating to processing strategies.

use super::models::{RenderRequest, RenderResponse, RenderStats};
use crate::errors::ApiError;
use crate::sessions::middleware::SessionContext;
use crate::state::AppState;
use axum::{extract::State, response::IntoResponse, Json};
use std::time::Instant;
use tracing::{info, warn};

/// Enhanced render endpoint with resource controls
pub async fn render(
    State(state): State<AppState>,
    session_ctx: SessionContext,
    Json(body): Json<RenderRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let start = Instant::now();
    let _guard = state
        .resource_manager
        .acquire_render_resources(&body.url)
        .await?;
    let timeout = body
        .timeout
        .map(std::time::Duration::from_secs)
        .unwrap_or_else(|| state.api_config.get_timeout("render"));
    tokio::time::timeout(
        timeout,
        process_render(state.clone(), session_ctx, body, start),
    )
    .await
    .map_err(|_| ApiError::timeout("Render", "Exceeded timeout"))?
}

async fn process_render(
    state: AppState,
    session_ctx: SessionContext,
    body: RenderRequest,
    start: Instant,
) -> Result<impl IntoResponse, ApiError> {
    let session_id = body
        .session_id
        .or_else(|| Some(session_ctx.session_id().to_string()));
    if body.url.is_empty() {
        return Err(ApiError::validation("URL cannot be empty"));
    }

    let (final_url, result, pdf) =
        super::strategies::process_by_mode(&state, &body, session_id.as_deref()).await?;
    let content = super::extraction::extract_content(
        &state.extraction_facade,
        &result,
        &body.output_format.unwrap_or_default(),
        &final_url,
    )
    .await?;

    let stats = RenderStats {
        total_time_ms: start.elapsed().as_millis() as u64,
        dynamic_time_ms: result.as_ref().map(|r| r.render_time_ms),
        pdf_time_ms: pdf.as_ref().map(|p| p.stats.processing_time_ms),
        extraction_time_ms: 0,
        actions_executed: result
            .as_ref()
            .map(|r| r.actions_executed.len() as u32)
            .unwrap_or(0),
        wait_conditions_met: result
            .as_ref()
            .map(|r| r.wait_conditions_met.len() as u32)
            .unwrap_or(0),
        network_requests: result
            .as_ref()
            .and_then(|r| r.artifacts.as_ref())
            .map(|a| a.network_activity.len() as u32)
            .unwrap_or(0),
        page_size_bytes: result.as_ref().map(|r| r.html.len() as u64).unwrap_or(0),
    };

    let success = result.as_ref().map(|r| r.success).unwrap_or(false)
        && pdf.as_ref().map(|p| p.success).unwrap_or(true);
    let response = RenderResponse {
        url: body.url,
        final_url,
        mode: format!("{:?}", body.mode.unwrap_or_default()),
        success,
        content,
        pdf_result: pdf,
        artifacts: result.and_then(|r| r.artifacts),
        stats,
        error: None,
        stealth_applied: vec![],
        session_info: None,
    };

    if let Err(e) = state
        .resource_manager
        .performance_monitor
        .record_render_operation(&response.url, start.elapsed(), success, 0, 0)
        .await
    {
        warn!("Metrics failed: {}", e);
    }
    info!(url=%response.url, success, ms=response.stats.total_time_ms, "Render complete");
    Ok(Json(response))
}
