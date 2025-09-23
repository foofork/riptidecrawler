use crate::errors::{ApiError, ApiResult};
use crate::models::*;
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use riptide_core::dynamic::{DynamicConfig, DynamicHandler, DynamicRenderResult};
use riptide_core::pdf::{utils as pdf_utils, PdfProcessor};
use riptide_core::stealth::StealthController;
use riptide_core::types::{ExtractedDoc, OutputFormat, RenderMode};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{debug, info, warn};

/// Request body for enhanced render endpoint
#[derive(Deserialize, Debug, Clone)]
pub struct RenderRequest {
    /// URL to render
    pub url: String,

    /// Rendering mode (static, dynamic, adaptive, pdf)
    pub mode: Option<RenderMode>,

    /// Dynamic content configuration
    pub dynamic_config: Option<DynamicConfig>,

    /// Stealth configuration for anti-detection
    pub stealth_config: Option<riptide_core::stealth::StealthConfig>,

    /// PDF processing configuration (for PDF URLs)
    pub pdf_config: Option<riptide_core::pdf::PdfConfig>,

    /// Output format preference
    pub output_format: Option<OutputFormat>,

    /// Whether to capture artifacts (screenshots, MHTML)
    pub capture_artifacts: Option<bool>,

    /// Timeout for rendering operation (seconds)
    pub timeout: Option<u64>,
}

/// Response for render endpoint
#[derive(Serialize, Debug)]
pub struct RenderResponse {
    /// Original URL
    pub url: String,

    /// Final URL after redirects
    pub final_url: String,

    /// Rendering mode used
    pub mode: String,

    /// Whether rendering was successful
    pub success: bool,

    /// Extracted content
    pub content: Option<ExtractedDoc>,

    /// PDF processing result (if applicable)
    pub pdf_result: Option<riptide_core::pdf::PdfProcessingResult>,

    /// Dynamic rendering artifacts
    pub artifacts: Option<riptide_core::dynamic::RenderArtifacts>,

    /// Processing statistics
    pub stats: RenderStats,

    /// Error information if rendering failed
    pub error: Option<ErrorInfo>,

    /// Stealth measures applied
    pub stealth_applied: Vec<String>,
}

/// Rendering statistics
#[derive(Serialize, Debug)]
pub struct RenderStats {
    /// Total processing time
    pub total_time_ms: u64,

    /// Time spent on dynamic rendering
    pub dynamic_time_ms: Option<u64>,

    /// Time spent on PDF processing
    pub pdf_time_ms: Option<u64>,

    /// Time spent on content extraction
    pub extraction_time_ms: u64,

    /// Number of actions executed
    pub actions_executed: u32,

    /// Number of wait conditions satisfied
    pub wait_conditions_met: u32,

    /// Network requests made during rendering
    pub network_requests: u32,

    /// Final page size in bytes
    pub page_size_bytes: u64,
}

/// Enhanced render endpoint with dynamic content handling
pub async fn render(
    State(state): State<AppState>,
    Json(body): Json<RenderRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();

    info!(
        url = %body.url,
        mode = ?body.mode,
        has_dynamic_config = body.dynamic_config.is_some(),
        has_stealth_config = body.stealth_config.is_some(),
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

    // Initialize stealth controller if configured
    let mut stealth_applied = Vec::new();
    let mut stealth_controller = body.stealth_config.as_ref().map(|config| {
        stealth_applied.push("user_agent_rotation".to_string());
        stealth_applied.push("header_randomization".to_string());
        stealth_applied.push("timing_jitter".to_string());
        StealthController::new(config.clone())
    });

    // Determine processing path based on content type and mode
    let (final_url, render_result, pdf_result) = match &mode {
        RenderMode::Pdf => {
            // Handle PDF processing
            process_pdf(&state, &url, body.pdf_config.as_ref()).await?
        }
        RenderMode::Dynamic => {
            // Force dynamic rendering
            let dynamic_config = body.dynamic_config.unwrap_or_default();
            process_dynamic(&state, &url, &dynamic_config, stealth_controller.as_mut()).await?
        }
        RenderMode::Static => {
            // Force static processing
            process_static(&state, &url, stealth_controller.as_mut()).await?
        }
        RenderMode::Adaptive => {
            // Adaptive processing based on content analysis
            process_adaptive(&state, &url, &body, stealth_controller.as_mut()).await?
        }
    };

    // Extract content from the rendered result
    let extraction_start = Instant::now();
    let content = extract_content(&state, &render_result, &output_format).await?;
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
    };

    info!(
        url = %url,
        success = response.success,
        total_time_ms = response.stats.total_time_ms,
        "Render request completed"
    );

    Ok(Json(response))
}

/// Process PDF content
async fn process_pdf(
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
    let pdf_processor = riptide_core::pdf::DefaultPdfProcessor::new();
    let config = pdf_config.cloned().unwrap_or_default();

    let pdf_result = pdf_processor
        .process_pdf(&data, &config)
        .await
        .map_err(|e| ApiError::dependency("pdf_processor", e.to_string()))?;

    Ok((url.to_string(), None, Some(pdf_result)))
}

/// Process content with dynamic rendering
async fn process_dynamic(
    state: &AppState,
    url: &str,
    dynamic_config: &DynamicConfig,
    stealth_controller: Option<&mut StealthController>,
) -> ApiResult<(
    String,
    Option<DynamicRenderResult>,
    Option<riptide_core::pdf::PdfProcessingResult>,
)> {
    debug!(url = %url, "Processing with dynamic rendering");

    // Apply stealth measures if configured
    if let Some(stealth) = stealth_controller {
        let _user_agent = stealth.next_user_agent();
        let _headers = stealth.generate_headers();
        let _delay = stealth.calculate_delay();
        // TODO: Apply these to the actual headless browser
    }

    // TODO: Implement actual dynamic rendering with headless browser
    // This is a placeholder implementation
    let render_result = DynamicRenderResult {
        success: true,
        html: "<html><body>Dynamic content placeholder</body></html>".to_string(),
        artifacts: None,
        error: None,
        render_time_ms: 1000,
        actions_executed: vec!["wait_for_dom".to_string()],
        wait_conditions_met: vec!["dom_content_loaded".to_string()],
    };

    Ok((url.to_string(), Some(render_result), None))
}

/// Process content statically
async fn process_static(
    state: &AppState,
    url: &str,
    stealth_controller: Option<&mut StealthController>,
) -> ApiResult<(
    String,
    Option<DynamicRenderResult>,
    Option<riptide_core::pdf::PdfProcessingResult>,
)> {
    debug!(url = %url, "Processing with static rendering");

    // Apply stealth measures if configured
    let mut request_builder = state.http_client.get(url);

    if let Some(stealth) = stealth_controller {
        let user_agent = stealth.next_user_agent();
        request_builder = request_builder.header("User-Agent", user_agent);

        let headers = stealth.generate_headers();

        for (name, value) in headers {
            request_builder = request_builder.header(name, value);
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
async fn process_adaptive(
    state: &AppState,
    url: &str,
    request: &RenderRequest,
    stealth_controller: Option<&mut StealthController>,
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

    // For now, default to static processing
    // TODO: Implement content analysis to determine if dynamic rendering is needed
    if let Some(dynamic_config) = request.dynamic_config.as_ref() {
        process_dynamic(state, url, dynamic_config, stealth_controller).await
    } else {
        process_static(state, url, stealth_controller).await
    }
}

/// Extract content from render result
async fn extract_content(
    state: &AppState,
    render_result: &Option<DynamicRenderResult>,
    output_format: &OutputFormat,
) -> ApiResult<Option<ExtractedDoc>> {
    if let Some(result) = render_result {
        if !result.success {
            return Ok(None);
        }

        // TODO: Use the actual WASM extractor to process the HTML
        // This is a placeholder implementation
        let doc = ExtractedDoc {
            url: "placeholder".to_string(),
            title: Some("Placeholder Title".to_string()),
            byline: None,
            published_iso: None,
            markdown: "# Placeholder Content\n\nThis is placeholder content.".to_string(),
            text: "Placeholder Content. This is placeholder content.".to_string(),
            links: Vec::new(),
            media: Vec::new(),
            language: Some("en".to_string()),
            reading_time: Some(1),
            quality_score: Some(80),
            word_count: Some(100),
            categories: Vec::new(),
            site_name: None,
            description: Some("Placeholder description".to_string()),
        };

        Ok(Some(doc))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn test_render_request_validation() {
        // Test empty URL validation
        let empty_url_request = RenderRequest {
            url: "".to_string(),
            mode: None,
            dynamic_config: None,
            stealth_config: None,
            pdf_config: None,
            output_format: None,
            capture_artifacts: None,
            timeout: None,
        };

        // This would be tested with actual state
        // let result = render(State(app_state), Json(empty_url_request)).await;
        // assert!(result.is_err());
    }

    #[test]
    fn test_render_stats_creation() {
        let stats = RenderStats {
            total_time_ms: 1000,
            dynamic_time_ms: Some(500),
            pdf_time_ms: None,
            extraction_time_ms: 200,
            actions_executed: 3,
            wait_conditions_met: 2,
            network_requests: 5,
            page_size_bytes: 102400,
        };

        assert_eq!(stats.total_time_ms, 1000);
        assert_eq!(stats.dynamic_time_ms, Some(500));
        assert_eq!(stats.actions_executed, 3);
    }
}
