//! Core rendering handlers with resource management and timeout controls.

use crate::errors::ApiError;
use crate::resource_manager::{RenderResourceGuard, ResourceResult};
use crate::sessions::middleware::SessionContext;
use crate::state::AppState;
use axum::{extract::State, response::IntoResponse, Json};
use riptide_stealth::StealthController;
use std::time::Instant;
use tracing::{debug, error, info, warn};

use super::extraction::extract_content;
use super::models::{RenderRequest, RenderResponse, RenderStats, SessionRenderInfo};
use super::processors::{process_adaptive, process_dynamic, process_pdf, process_static};

/// Enhanced render endpoint with dynamic content handling and resource controls
pub async fn render(
    State(state): State<AppState>,
    session_ctx: SessionContext,
    Json(body): Json<RenderRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Apply comprehensive resource controls first
    let resource_guard = match state
        .resource_manager
        .acquire_render_resources(&body.url)
        .await
    {
        Ok(ResourceResult::Success(guard)) => guard,
        Ok(ResourceResult::Timeout) => {
            warn!(url = %body.url, "Render request timed out during resource acquisition");
            return Err(ApiError::timeout(
                "Resource acquisition",
                "Resource acquisition timed out",
            ));
        }
        Ok(ResourceResult::ResourceExhausted) => {
            warn!(url = %body.url, "Render request rejected - resources exhausted");
            return Err(ApiError::service_unavailable(
                "All rendering resources are currently in use",
            ));
        }
        Ok(ResourceResult::RateLimited { retry_after }) => {
            warn!(url = %body.url, retry_after_ms = retry_after.as_millis(), "Render request rate limited");
            return Err(ApiError::rate_limited(format!(
                "Rate limited. Retry after {}ms",
                retry_after.as_millis()
            )));
        }
        Ok(ResourceResult::MemoryPressure) => {
            warn!(url = %body.url, "Render request rejected due to memory pressure");
            return Err(ApiError::service_unavailable(
                "System under memory pressure",
            ));
        }
        Ok(ResourceResult::Error(e)) => {
            error!(url = %body.url, error = %e, "Resource acquisition failed");
            return Err(ApiError::internal(format!(
                "Resource acquisition failed: {}",
                e
            )));
        }
        Err(e) => {
            error!(url = %body.url, error = %e, "Unexpected error during resource acquisition");
            return Err(ApiError::internal("Failed to acquire rendering resources"));
        }
    };

    // Apply hard timeout wrapper around entire operation (3s requirement)
    // Allow per-request timeout override if specified in request
    let render_timeout = if let Some(timeout_secs) = body.timeout {
        std::time::Duration::from_secs(timeout_secs)
    } else {
        state.api_config.get_timeout("render")
    };

    let render_result = tokio::time::timeout(render_timeout, async {
        render_with_resources(state.clone(), session_ctx, body, resource_guard).await
    })
    .await;

    match render_result {
        Ok(result) => result,
        Err(_) => {
            // Timeout occurred - trigger cleanup
            state.resource_manager.cleanup_on_timeout("render").await;
            warn!(
                "Render operation timed out after {}ms",
                render_timeout.as_millis()
            );
            Err(ApiError::timeout(
                "Render operation",
                "Render operation exceeded maximum time limit",
            ))
        }
    }
}

/// Internal render function with resource management
async fn render_with_resources(
    state: AppState,
    session_ctx: SessionContext,
    body: RenderRequest,
    _resource_guard: RenderResourceGuard,
) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();

    // Determine session to use (from request or middleware)
    let session_id = body
        .session_id
        .clone()
        .or_else(|| Some(session_ctx.session_id().to_string()));

    info!(
        url = %body.url,
        mode = ?body.mode,
        has_dynamic_config = body.dynamic_config.is_some(),
        has_stealth_config = body.stealth_config.is_some(),
        session_id = ?session_id,
        "Received enhanced render request"
    );

    // Validate URL
    if body.url.is_empty() {
        return Err(ApiError::validation("URL cannot be empty"));
    }

    let url = body.url.clone();
    let mode = body.mode.clone().unwrap_or_default();
    let output_format = body.output_format.clone().unwrap_or_default();
    let capture_artifacts = body.capture_artifacts.unwrap_or(false);

    debug!(
        url = %url,
        mode = ?mode,
        output_format = ?output_format,
        capture_artifacts = capture_artifacts,
        "Processing render request"
    );

    // Initialize stealth tracking
    let mut stealth_applied = Vec::new();

    // Get session information for rendering context
    let session_info = if let Some(ref sid) = session_id {
        match state.session_manager.get_session(sid).await {
            Ok(Some(session)) => {
                // Extract domain from URL for cookie context
                let domain = url::Url::parse(&url)
                    .map(|parsed_url| {
                        parsed_url
                            .host_str()
                            .map(|host| host.to_string())
                            .unwrap_or_else(|| {
                                warn!(url = %url, "URL has no host, using 'localhost'");
                                "localhost".to_string()
                            })
                    })
                    .unwrap_or_else(|e| {
                        warn!(url = %url, error = %e, "Failed to parse URL for domain extraction");
                        "unknown".to_string()
                    });

                let cookies_for_domain = state
                    .session_manager
                    .get_cookies_for_domain(sid, &domain)
                    .await
                    .map(|cookies| cookies.len())
                    .unwrap_or_else(|e| {
                        warn!(session_id = %sid, domain = %domain, error = %e, "Failed to get session cookies for domain");
                        0
                    });

                Some(SessionRenderInfo {
                    session_id: sid.clone(),
                    user_data_dir: session.user_data_dir.to_string_lossy().to_string(),
                    cookies_for_domain,
                    state_preserved: true,
                })
            }
            Ok(None) => {
                warn!(session_id = %sid, "Session not found, proceeding without session state");
                None
            }
            Err(e) => {
                warn!(session_id = %sid, error = %e, "Failed to get session, proceeding without session state");
                None
            }
        }
    } else {
        None
    };

    // Determine processing path based on content type and mode
    let (final_url, render_result, pdf_result) = match &mode {
        riptide_types::RenderMode::Pdf => {
            // Handle PDF processing (no stealth support)
            process_pdf(&state, &url, body.pdf_config.as_ref()).await?
        }
        riptide_types::RenderMode::Dynamic => {
            // Force dynamic rendering with stealth
            let mut stealth_controller = body.stealth_config.as_ref().map(|config| {
                stealth_applied.push("user_agent_rotation".to_string());
                stealth_applied.push("header_randomization".to_string());
                stealth_applied.push("timing_jitter".to_string());
                StealthController::new(config.clone())
            });
            let dynamic_config = body.dynamic_config.unwrap_or_default();
            process_dynamic(
                &state,
                &url,
                &dynamic_config,
                stealth_controller.as_mut(),
                session_id.as_deref(),
            )
            .await?
        }
        riptide_types::RenderMode::Static => {
            // Force static processing with stealth
            let mut stealth_controller = body.stealth_config.as_ref().map(|config| {
                stealth_applied.push("user_agent_rotation".to_string());
                stealth_applied.push("header_randomization".to_string());
                stealth_applied.push("timing_jitter".to_string());
                StealthController::new(config.clone())
            });
            process_static(
                &state,
                &url,
                stealth_controller.as_mut(),
                session_id.as_deref(),
            )
            .await?
        }
        riptide_types::RenderMode::Adaptive => {
            // Adaptive processing with stealth
            let mut stealth_controller = body.stealth_config.as_ref().map(|config| {
                stealth_applied.push("user_agent_rotation".to_string());
                stealth_applied.push("header_randomization".to_string());
                stealth_applied.push("timing_jitter".to_string());
                StealthController::new(config.clone())
            });
            process_adaptive(
                &state,
                &url,
                &body,
                stealth_controller.as_mut(),
                session_id.as_deref(),
            )
            .await?
        }
        riptide_types::RenderMode::Html => {
            // HTML output mode with stealth
            let mut stealth_controller = body.stealth_config.as_ref().map(|config| {
                stealth_applied.push("user_agent_rotation".to_string());
                stealth_applied.push("header_randomization".to_string());
                stealth_applied.push("timing_jitter".to_string());
                StealthController::new(config.clone())
            });
            process_static(
                &state,
                &url,
                stealth_controller.as_mut(),
                session_id.as_deref(),
            )
            .await?
        }
        riptide_types::RenderMode::Markdown => {
            // Markdown output mode with stealth
            let mut stealth_controller = body.stealth_config.as_ref().map(|config| {
                stealth_applied.push("user_agent_rotation".to_string());
                stealth_applied.push("header_randomization".to_string());
                stealth_applied.push("timing_jitter".to_string());
                StealthController::new(config.clone())
            });
            process_static(
                &state,
                &url,
                stealth_controller.as_mut(),
                session_id.as_deref(),
            )
            .await?
        }
    };

    // Extract content from the rendered result
    let extraction_start = Instant::now();
    let content =
        extract_content(&state.extractor, &render_result, &output_format, &final_url).await?;
    let extraction_time_ms = extraction_start.elapsed().as_millis() as u64;

    // Build statistics
    let stats = RenderStats {
        total_time_ms: start_time.elapsed().as_millis() as u64,
        dynamic_time_ms: render_result.as_ref().map(|r| r.render_time_ms),
        pdf_time_ms: pdf_result.as_ref().map(|r| r.stats.processing_time_ms),
        extraction_time_ms,
        actions_executed: render_result
            .as_ref()
            .map(|r| r.actions_executed.len() as u32)
            .unwrap_or(0),
        wait_conditions_met: render_result
            .as_ref()
            .map(|r| r.wait_conditions_met.len() as u32)
            .unwrap_or(0),
        network_requests: render_result
            .as_ref()
            .and_then(|r| r.artifacts.as_ref())
            .map(|a| a.network_activity.len() as u32)
            .unwrap_or(0),
        page_size_bytes: render_result
            .as_ref()
            .map(|r| r.html.len() as u64)
            .unwrap_or(0),
    };

    let response = RenderResponse {
        url: body.url,
        final_url,
        mode: format!("{:?}", mode),
        success: render_result.as_ref().map(|r| r.success).unwrap_or(false)
            && pdf_result.as_ref().map(|r| r.success).unwrap_or(true),
        content,
        pdf_result,
        artifacts: render_result.and_then(|r| r.artifacts),
        stats,
        error: None,
        stealth_applied,
        session_info,
    };

    // Record metrics for monitoring and performance tracking
    if let Err(e) = state
        .resource_manager
        .performance_monitor
        .record_render_operation(
            &url,
            start_time.elapsed(),
            response.success,
            response.stats.actions_executed,
            response.stats.network_requests,
        )
        .await
    {
        warn!(error = %e, "Failed to record render operation metrics");
    }

    info!(
        url = %url,
        success = response.success,
        total_time_ms = response.stats.total_time_ms,
        mode = %response.mode,
        actions_executed = response.stats.actions_executed,
        network_requests = response.stats.network_requests,
        "Render request completed with resource controls and metrics recorded"
    );

    Ok(Json(response))
}
