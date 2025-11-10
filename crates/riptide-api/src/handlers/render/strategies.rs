use super::models::RenderRequest;
use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;
use riptide_facade::facades::RenderStrategyFacade;
use riptide_headless::dynamic::{DynamicConfig, DynamicRenderResult};
use riptide_stealth::StealthController;
use tokio::time::{timeout, Duration};
use tracing::warn;

/// Global render strategy facade
static RENDER_FACADE: std::sync::OnceLock<RenderStrategyFacade> = std::sync::OnceLock::new();

fn get_render_facade() -> &'static RenderStrategyFacade {
    RENDER_FACADE.get_or_init(RenderStrategyFacade::new)
}

/// Analyze URL and content patterns to determine if dynamic rendering is needed
pub(super) async fn analyze_url_for_dynamic_content(url: &str) -> bool {
    let facade = get_render_facade();
    facade.requires_dynamic_rendering(url).await
}

/// Create adaptive dynamic configuration based on URL analysis
pub(super) fn create_adaptive_dynamic_config(url: &str) -> DynamicConfig {
    let facade = get_render_facade();
    facade.create_dynamic_config(url)
}

/// Process by render mode - unified entry point
pub(super) async fn process_by_mode(
    state: &AppState,
    request: &RenderRequest,
    session_id: Option<&str>,
) -> ApiResult<(
    String,
    Option<DynamicRenderResult>,
    Option<riptide_pdf::PdfProcessingResult>,
)> {
    let mut stealth_controller = request
        .stealth_config
        .as_ref()
        .map(|c| StealthController::new(c.clone()));

    match &request.mode {
        Some(riptide_types::RenderMode::Pdf) | None if request.url.ends_with(".pdf") => {
            process_pdf(state, &request.url, request.pdf_config.as_ref()).await
        }
        Some(riptide_types::RenderMode::Dynamic) => {
            process_dynamic(
                state,
                &request.url,
                &request.dynamic_config.clone().unwrap_or_default(),
                stealth_controller.as_mut(),
                session_id,
            )
            .await
        }
        Some(riptide_types::RenderMode::Adaptive) => {
            process_adaptive(
                state,
                &request.url,
                request,
                stealth_controller.as_mut(),
                session_id,
            )
            .await
        }
        _ => process_static(state, &request.url, stealth_controller.as_mut(), session_id).await,
    }
}

/// Process PDF content
async fn process_pdf(
    state: &AppState,
    url: &str,
    pdf_config: Option<&riptide_pdf::PdfConfig>,
) -> ApiResult<(
    String,
    Option<DynamicRenderResult>,
    Option<riptide_pdf::PdfProcessingResult>,
)> {
    let data = state
        .scraper_facade
        .fetch_bytes(url)
        .await
        .map_err(|e| ApiError::dependency("scraper", e.to_string()))?;
    if !riptide_pdf::utils::is_pdf_content(None, &data) {
        return Err(ApiError::validation("Not a valid PDF"));
    }
    let pdf_result = riptide_pdf::create_pdf_processor()
        .process_pdf(&data, &pdf_config.cloned().unwrap_or_default())
        .await
        .map_err(|e| ApiError::dependency("pdf", e.to_string()))?;
    Ok((url.to_string(), None, Some(pdf_result)))
}

/// Process with dynamic rendering
async fn process_dynamic(
    state: &AppState,
    url: &str,
    config: &DynamicConfig,
    stealth: Option<&mut StealthController>,
    session_id: Option<&str>,
) -> ApiResult<(
    String,
    Option<DynamicRenderResult>,
    Option<riptide_pdf::PdfProcessingResult>,
)> {
    let stealth_config = stealth.as_ref().map(|s| s.config().clone());
    let rpc_client = state
        .config
        .headless_url
        .as_ref()
        .map(|u| crate::rpc_client::RpcClient::with_url(u.clone()))
        .unwrap_or_default();

    if rpc_client.health_check().await.is_err() {
        warn!("Headless unavailable, falling back to static");
        return process_static(state, url, stealth, session_id).await;
    }

    let user_data_dir = if let Some(sid) = session_id {
        state
            .session_manager
            .get_user_data_dir(sid)
            .await
            .ok()
            .map(|p| p.to_string_lossy().to_string())
    } else {
        None
    };
    let render_result = timeout(
        Duration::from_secs(state.api_config.performance.render_timeout_secs),
        rpc_client.render_dynamic_with_session(
            url,
            config,
            stealth_config.as_ref(),
            session_id,
            user_data_dir.as_deref(),
        ),
    )
    .await;

    match render_result {
        Ok(Ok(mut result)) => {
            let final_url = result
                .artifacts
                .as_ref()
                .and_then(|a| {
                    if !a.metadata.final_url.is_empty() {
                        Some(a.metadata.final_url.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| url.to_string());
            if let Some(ref mut artifacts) = result.artifacts {
                artifacts.metadata.final_url = final_url.clone();
            }
            Ok((final_url, Some(result), None))
        }
        Ok(Err(_)) => process_static(state, url, stealth, session_id).await,
        Err(_) => Err(ApiError::timeout("render", "Operation exceeded timeout")),
    }
}

/// Process statically
async fn process_static(
    state: &AppState,
    url: &str,
    stealth: Option<&mut StealthController>,
    session_id: Option<&str>,
) -> ApiResult<(
    String,
    Option<DynamicRenderResult>,
    Option<riptide_pdf::PdfProcessingResult>,
)> {
    let (final_url, html) = if stealth.is_none() && session_id.is_none() {
        (
            url.to_string(),
            state
                .scraper_facade
                .fetch_html(url)
                .await
                .map_err(|e| ApiError::dependency("scraper", e.to_string()))?,
        )
    } else {
        let mut req = state.http_client.get(url);
        if let Some(s) = stealth {
            req = req.header("User-Agent", s.next_user_agent());
            for (k, v) in s.generate_headers() {
                req = req.header(k, v);
            }
        }
        if let Some(sid) = session_id {
            if let Ok(parsed) = url::Url::parse(url) {
                if let Some(domain) = parsed.host_str() {
                    if let Ok(cookies) = state
                        .session_manager
                        .get_cookies_for_domain(sid, domain)
                        .await
                    {
                        if !cookies.is_empty() {
                            req = req.header(
                                "Cookie",
                                cookies
                                    .iter()
                                    .map(|c| format!("{}={}", c.name, c.value))
                                    .collect::<Vec<_>>()
                                    .join("; "),
                            );
                        }
                    }
                }
            }
        }
        let resp = req
            .send()
            .await
            .map_err(|e| ApiError::dependency("http", e.to_string()))?;
        if !resp.status().is_success() {
            return Err(ApiError::dependency(
                "http",
                format!("HTTP {}", resp.status()),
            ));
        }
        (
            resp.url().to_string(),
            resp.text()
                .await
                .map_err(|e| ApiError::dependency("http", e.to_string()))?,
        )
    };
    Ok((
        final_url.clone(),
        Some(DynamicRenderResult {
            success: true,
            html,
            artifacts: None,
            error: None,
            render_time_ms: 100,
            actions_executed: vec![],
            wait_conditions_met: vec![],
        }),
        None,
    ))
}

/// Process adaptively
async fn process_adaptive(
    state: &AppState,
    url: &str,
    request: &RenderRequest,
    stealth: Option<&mut StealthController>,
    session_id: Option<&str>,
) -> ApiResult<(
    String,
    Option<DynamicRenderResult>,
    Option<riptide_pdf::PdfProcessingResult>,
)> {
    if url.ends_with(".pdf") || url.contains(".pdf?") {
        return process_pdf(state, url, request.pdf_config.as_ref()).await;
    }
    let needs_dynamic = analyze_url_for_dynamic_content(url).await;
    if needs_dynamic || request.dynamic_config.is_some() {
        let config = request
            .dynamic_config
            .clone()
            .unwrap_or_else(|| create_adaptive_dynamic_config(url));
        process_dynamic(state, url, &config, stealth, session_id).await
    } else {
        process_static(state, url, stealth, session_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riptide_headless::dynamic::ScrollMode;
    use std::time::Duration;

    #[tokio::test]
    async fn test_analyze_url_for_dynamic_content() {
        assert!(analyze_url_for_dynamic_content("https://twitter.com/user/status/123").await);
        assert!(analyze_url_for_dynamic_content("https://github.com/org/repo").await);
        assert!(!analyze_url_for_dynamic_content("https://example.com/blog/article.html").await);
    }

    #[test]
    fn test_create_adaptive_dynamic_config() {
        let config = create_adaptive_dynamic_config("https://github.com/rust-lang/rust");
        assert!(config.wait_for.is_some());
        assert!(config.scroll.is_some());
        assert_eq!(config.timeout, Duration::from_secs(3));
    }
}
