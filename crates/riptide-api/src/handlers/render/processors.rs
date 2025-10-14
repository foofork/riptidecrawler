//! Content processing strategies for different render modes.
//!
//! This module provides specialized processors for PDF, dynamic, static, and adaptive rendering.

use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;
use riptide_core::dynamic::{DynamicConfig, DynamicRenderResult};
use riptide_core::pdf::utils as pdf_utils;
use riptide_core::stealth::StealthController;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, warn};

use super::models::RenderRequest;

/// Process PDF content
pub async fn process_pdf(
    state: &AppState,
    url: &str,
    pdf_config: Option<&riptide_core::pdf::PdfConfig>,
) -> ApiResult<(
    String,
    Option<DynamicRenderResult>,
    Option<riptide_core::pdf::PdfProcessingResult>,
)> {
    debug!(url = %url, "Processing as PDF");

    // Fetch the PDF content
    let response =
        state.http_client.get(url).send().await.map_err(|e| {
            ApiError::dependency("http_client", format!("Failed to fetch PDF: {}", e))
        })?;

    if !response.status().is_success() {
        return Err(ApiError::dependency(
            "http_client",
            format!("HTTP error fetching PDF: {}", response.status()),
        ));
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|ct| ct.to_str().ok())
        .map(|s| s.to_string());

    let data = response.bytes().await.map_err(|e| {
        ApiError::dependency("http_client", format!("Failed to read PDF data: {}", e))
    })?;

    // Verify it's actually a PDF
    if !pdf_utils::is_pdf_content(content_type.as_deref(), &data) {
        return Err(ApiError::validation("Content is not a valid PDF"));
    }

    // Process the PDF
    let pdf_processor = riptide_core::pdf::create_pdf_processor();
    let config = pdf_config.cloned().unwrap_or_default();

    let pdf_result = pdf_processor
        .process_pdf(&data, &config)
        .await
        .map_err(|e| ApiError::dependency("pdf_processor", e.to_string()))?;

    Ok((url.to_string(), None, Some(pdf_result)))
}

/// Process content with dynamic rendering
pub async fn process_dynamic(
    state: &AppState,
    url: &str,
    dynamic_config: &DynamicConfig,
    stealth_controller: Option<&mut StealthController>,
    session_id: Option<&str>,
) -> ApiResult<(
    String,
    Option<DynamicRenderResult>,
    Option<riptide_core::pdf::PdfProcessingResult>,
)> {
    debug!(url = %url, "Processing with dynamic rendering");

    // Get stealth configuration for RPC call
    // The stealth config is passed directly to render_dynamic() below (line 170)
    // which includes it in the HeadlessRenderRequest for the headless service
    let stealth_config = stealth_controller
        .as_ref()
        .map(|controller| controller.config().clone());

    // Create RPC client with configured headless URL
    let rpc_client = if let Some(headless_url) = &state.config.headless_url {
        crate::rpc_client::RpcClient::with_url(headless_url.clone())
    } else {
        crate::rpc_client::RpcClient::new()
    };

    // Perform health check on headless service
    if let Err(e) = rpc_client.health_check().await {
        warn!(
            url = %url,
            error = %e,
            "Headless service health check failed, falling back to static rendering"
        );

        // Fall back to static rendering if headless service is unavailable
        return process_static(state, url, stealth_controller, session_id).await;
    }

    // Get session user data directory if available
    let user_data_dir = if let Some(sid) = session_id {
        state
            .session_manager
            .get_user_data_dir(sid)
            .await
            .ok()
            .map(|path| path.to_string_lossy().to_string())
    } else {
        None
    };

    debug!(
        url = %url,
        session_id = ?session_id,
        user_data_dir = ?user_data_dir,
        "Calling dynamic rendering with session context"
    );

    // TODO(P1): Pass session context to RPC client for browser state persistence
    // STATUS: Session manager provides user_data_dir but not passed to RPC
    // PLAN: Extend RPC client to support session-based rendering
    // IMPLEMENTATION:
    //   1. Add render_dynamic_with_session() method to RpcClient
    //   2. Pass session_id and user_data_dir to headless service
    //   3. Headless service launches browser with persistent profile
    //   4. Browser maintains cookies, localStorage, and auth state
    //   5. Enable multi-step workflows with session continuity
    // DEPENDENCIES: Headless service must support profile directories
    // EFFORT: Medium (6-8 hours)
    // PRIORITY: Important for authenticated scraping workflows
    // BLOCKER: None - infrastructure exists, just needs wiring

    // Get render timeout from configuration
    let render_timeout = Duration::from_secs(state.api_config.performance.render_timeout_secs);

    debug!(
        url = %url,
        timeout_secs = state.api_config.performance.render_timeout_secs,
        "Applying render timeout protection"
    );

    // Call dynamic rendering via RPC with timeout protection
    let render_result = timeout(
        render_timeout,
        rpc_client.render_dynamic(url, dynamic_config, stealth_config.as_ref()),
    )
    .await;

    match render_result {
        Ok(Ok(mut render_result)) => {
            debug!(
                url = %url,
                render_time_ms = render_result.render_time_ms,
                html_size = render_result.html.len(),
                actions_executed = render_result.actions_executed.len(),
                "Dynamic rendering completed successfully"
            );

            // Get final URL from response or use original URL
            let final_url = render_result
                .artifacts
                .as_ref()
                .and_then(|a| {
                    let final_url = &a.metadata.final_url;
                    if final_url.is_empty() {
                        None
                    } else {
                        Some(final_url.clone())
                    }
                })
                .unwrap_or_else(|| {
                    debug!(url = %url, "No final URL from render artifacts, using original URL");
                    url.to_string()
                });

            // Update render result with correct final URL
            if let Some(ref mut artifacts) = render_result.artifacts {
                artifacts.metadata.final_url = final_url.clone();
            }

            Ok((final_url, Some(render_result), None))
        }
        Ok(Err(e)) => {
            warn!(
                url = %url,
                error = %e,
                "Dynamic rendering failed, falling back to static rendering"
            );

            // Fall back to static rendering on error
            process_static(state, url, stealth_controller, session_id).await
        }
        Err(_) => {
            error!(
                url = %url,
                timeout_secs = state.api_config.performance.render_timeout_secs,
                "Render operation timed out"
            );

            Err(ApiError::timeout(
                "render",
                format!(
                    "Operation exceeded {}s timeout",
                    state.api_config.performance.render_timeout_secs
                ),
            ))
        }
    }
}

/// Process content statically
pub async fn process_static(
    state: &AppState,
    url: &str,
    stealth_controller: Option<&mut StealthController>,
    session_id: Option<&str>,
) -> ApiResult<(
    String,
    Option<DynamicRenderResult>,
    Option<riptide_core::pdf::PdfProcessingResult>,
)> {
    debug!(url = %url, "Processing with static rendering");

    // Apply stealth measures and session cookies if configured
    let mut request_builder = state.http_client.get(url);

    if let Some(stealth) = stealth_controller {
        let user_agent = stealth.next_user_agent();
        request_builder = request_builder.header("User-Agent", user_agent);

        let headers = stealth.generate_headers();

        for (name, value) in headers {
            request_builder = request_builder.header(name, value);
        }
    }

    // Add session cookies if available
    if let Some(sid) = session_id {
        if let Ok(parsed_url) = url::Url::parse(url) {
            if let Some(domain) = parsed_url.host_str() {
                if let Ok(cookies) = state
                    .session_manager
                    .get_cookies_for_domain(sid, domain)
                    .await
                {
                    if !cookies.is_empty() {
                        let cookie_header = cookies
                            .iter()
                            .map(|c| format!("{}={}", c.name, c.value))
                            .collect::<Vec<_>>()
                            .join("; ");
                        request_builder = request_builder.header("Cookie", cookie_header);

                        debug!(
                            session_id = %sid,
                            domain = %domain,
                            cookie_count = cookies.len(),
                            "Added session cookies to static request"
                        );
                    }
                }
            }
        }
    }

    let response = request_builder.send().await.map_err(|e| {
        ApiError::dependency("http_client", format!("Failed to fetch content: {}", e))
    })?;

    if !response.status().is_success() {
        return Err(ApiError::dependency(
            "http_client",
            format!("HTTP error: {}", response.status()),
        ));
    }

    let final_url = response.url().to_string();
    let html = response.text().await.map_err(|e| {
        ApiError::dependency("http_client", format!("Failed to read content: {}", e))
    })?;

    let render_result = DynamicRenderResult {
        success: true,
        html,
        artifacts: None,
        error: None,
        render_time_ms: 100,
        actions_executed: Vec::new(),
        wait_conditions_met: Vec::new(),
    };

    Ok((final_url, Some(render_result), None))
}

/// Process content adaptively
pub async fn process_adaptive(
    state: &AppState,
    url: &str,
    request: &RenderRequest,
    stealth_controller: Option<&mut StealthController>,
    session_id: Option<&str>,
) -> ApiResult<(
    String,
    Option<DynamicRenderResult>,
    Option<riptide_core::pdf::PdfProcessingResult>,
)> {
    debug!(url = %url, "Processing with adaptive rendering");

    // Check if it's a PDF based on URL extension
    if url.ends_with(".pdf") || url.contains(".pdf?") {
        return process_pdf(state, url, request.pdf_config.as_ref()).await;
    }

    // Perform content analysis to determine optimal rendering strategy
    let needs_dynamic = super::strategies::analyze_url_for_dynamic_content(url).await;

    if needs_dynamic || request.dynamic_config.is_some() {
        debug!(url = %url, "Content analysis suggests dynamic rendering");

        // Use provided config or create adaptive config
        let dynamic_config = request
            .dynamic_config
            .clone()
            .unwrap_or_else(|| super::strategies::create_adaptive_dynamic_config(url));

        process_dynamic(state, url, &dynamic_config, stealth_controller, session_id).await
    } else {
        debug!(url = %url, "Content analysis suggests static rendering is sufficient");
        process_static(state, url, stealth_controller, session_id).await
    }
}
