// P3-T4.4: Import from crate::launcher which re-exports from riptide-engine
use crate::launcher::HeadlessLauncher;
use crate::models::*;
use axum::{extract::State, http::StatusCode, Json};
use chromiumoxide::Page;
use riptide_stealth::StealthPreset;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, warn};

/// Shared application state for the headless service
#[derive(Clone)]
pub struct AppState {
    /// Headless browser launcher with pooling and stealth
    pub launcher: Arc<HeadlessLauncher>,
}

/// Enhanced render function with browser pooling and timeout management
pub async fn render(
    State(state): State<AppState>,
    Json(req): Json<RenderReq>,
) -> Result<Json<RenderResp>, (StatusCode, Json<RenderErrorResp>)> {
    let start_time = Instant::now();
    let request_id = uuid::Uuid::new_v4().to_string();

    info!(
        request_id = %request_id,
        url = %req.url,
        wait_for = ?req.wait_for,
        scroll_steps = ?req.scroll_steps,
        "Starting headless render request"
    );

    // Hard timeout cap of 3 seconds as per requirements
    let render_timeout = Duration::from_secs(3);

    match timeout(
        render_timeout,
        render_internal(state, req.clone(), request_id.clone()),
    )
    .await
    {
        Ok(result) => {
            let duration = start_time.elapsed();
            match result {
                Ok(response) => {
                    info!(
                        request_id = %request_id,
                        duration_ms = duration.as_millis(),
                        "Headless render completed successfully"
                    );
                    Ok(Json(response))
                }
                Err(error_resp) => {
                    error!(
                        request_id = %request_id,
                        duration_ms = duration.as_millis(),
                        error = %error_resp.error,
                        "Headless render failed"
                    );
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_resp)))
                }
            }
        }
        Err(_) => {
            error!(
                request_id = %request_id,
                timeout_ms = render_timeout.as_millis(),
                "Headless render timed out"
            );
            Err((
                StatusCode::REQUEST_TIMEOUT,
                Json(RenderErrorResp {
                    error: "Render timeout exceeded".to_string(),
                    request_id: Some(request_id),
                    duration_ms: render_timeout.as_millis() as u64,
                }),
            ))
        }
    }
}

/// Execute page actions on a browser page
async fn exec_actions(page: &Page, actions: &[PageAction]) -> anyhow::Result<()> {
    for action in actions {
        match action {
            PageAction::WaitForCss { css, timeout_ms } => {
                // Note: timeout is not currently enforced by spider_chrome's find_element
                // TODO: Implement timeout mechanism similar to WaitForJs with deadline check
                let _ = page
                    .find_element(css)
                    .await
                    .map_err(|e| anyhow::anyhow!("CSS selector not found: {}", e))?;
                debug!(
                    "Waited for CSS selector: {} (timeout_ms: {:?})",
                    css, timeout_ms
                );
            }
            PageAction::WaitForJs { expr, timeout_ms } => {
                let deadline = Instant::now() + Duration::from_millis(timeout_ms.unwrap_or(5000));
                loop {
                    let result = page.evaluate(expr.as_str()).await?;
                    let ok: bool = result.into_value().unwrap_or_else(|e| {
                        debug!("JavaScript evaluation failed, treating as false: {}", e);
                        false
                    });
                    if ok {
                        debug!("JavaScript condition met: {}", expr);
                        break;
                    }
                    if Instant::now() >= deadline {
                        anyhow::bail!("wait_for_js timeout: {}", expr);
                    }
                    sleep(Duration::from_millis(100)).await;
                }
            }
            PageAction::Scroll {
                steps,
                step_px,
                delay_ms,
            } => {
                for i in 0..*steps {
                    let scroll_js = format!("window.scrollBy(0, {});", step_px);
                    page.evaluate(scroll_js.as_str()).await?;
                    debug!("Scrolled step {}/{}: {}px", i + 1, steps, step_px);
                    sleep(Duration::from_millis(*delay_ms)).await;
                }
            }
            PageAction::Js { code } => {
                page.evaluate(code.as_str()).await?;
                debug!("Executed JavaScript code");
            }
            PageAction::Click { css } => {
                page.find_element(css).await?.click().await?;
                debug!("Clicked element: {}", css);
            }
            PageAction::Type {
                css,
                text,
                delay_ms,
            } => {
                let element = page.find_element(css).await?;
                for ch in text.chars() {
                    element.type_str(&ch.to_string()).await?;
                    sleep(Duration::from_millis(delay_ms.unwrap_or(20))).await;
                }
                debug!("Typed text into element: {}", css);
            }
        }
    }
    Ok(())
}

/// Internal render implementation using HeadlessLauncher with browser pooling
async fn render_internal(
    state: AppState,
    req: RenderReq,
    request_id: String,
) -> Result<RenderResp, RenderErrorResp> {
    // Determine stealth preset from request
    let stealth_preset = determine_stealth_preset(&req)?;

    debug!(
        request_id = %request_id,
        url = %req.url,
        stealth_preset = ?stealth_preset,
        "Launching browser page from pool"
    );

    // Launch page using the pooled launcher (this reuses browsers from the pool!)
    let session = timeout(
        Duration::from_millis(2000), // 2s for checkout + navigation
        state.launcher.launch_page(&req.url, Some(stealth_preset)),
    )
    .await
    .map_err(|_| RenderErrorResp {
        error: "Browser session launch timed out".to_string(),
        request_id: Some(request_id.clone()),
        duration_ms: 2000,
    })?
    .map_err(|e| RenderErrorResp {
        error: format!("Failed to launch browser session: {}", e),
        request_id: Some(request_id.clone()),
        duration_ms: 0,
    })?;

    // Get the page from the session
    let page = session.page();

    // Execute any custom actions if provided
    if let Some(actions) = &req.actions {
        if let Err(e) = exec_actions(page, actions).await {
            warn!(
                request_id = %request_id,
                error = %e,
                "Action execution failed, proceeding with current state"
            );
        }
    } else {
        // Legacy path: wait for DOMContentLoaded + 1s idle time
        if let Err(e) = wait_for_content_and_idle(page, &req, &request_id).await {
            warn!(
                request_id = %request_id,
                error = %e,
                "Content wait failed, proceeding with current state"
            );
        }
    }

    // Perform scrolling if requested
    if let Some(steps) = req.scroll_steps {
        if let Err(e) = perform_scrolling(page, steps, &request_id).await {
            warn!(
                request_id = %request_id,
                error = %e,
                "Scrolling failed, proceeding with current content"
            );
        }
    }

    // Extract content
    let (html, final_url) = extract_page_content(page, &req.url, &request_id).await?;

    // Capture artifacts if requested
    let mut artifacts_out = ArtifactsOut::default();
    if let Some(artifacts) = &req.artifacts {
        if artifacts.screenshot {
            match page
                .screenshot(chromiumoxide::page::ScreenshotParams::default())
                .await
            {
                Ok(screenshot_bytes) => {
                    use base64::Engine;
                    artifacts_out.screenshot_b64 =
                        Some(base64::engine::general_purpose::STANDARD.encode(&screenshot_bytes));
                    debug!(request_id = %request_id, "Screenshot captured");
                }
                Err(e) => {
                    warn!(request_id = %request_id, error = %e, "Screenshot capture failed");
                }
            }
        }
        if artifacts.mhtml {
            // MHTML capture would require CDP command not yet exposed in spider_chrome
            debug!(request_id = %request_id, "MHTML capture not yet implemented");
        }
    }

    // Session cleanup is automatic when session is dropped
    // Browser is returned to pool for reuse
    debug!(request_id = %request_id, "Browser session completed, returning to pool");

    Ok(RenderResp {
        final_url,
        html,
        screenshot_b64: artifacts_out.screenshot_b64.clone(), // For backward compatibility
        session_id: req.session_id.clone(),                   // Echo back session ID for now
        artifacts: artifacts_out,
    })
}

// Note: inject_stealth_js and create_and_navigate_page removed
// These are now handled by HeadlessLauncher internally

/// Wait for DOMContentLoaded + 1s idle time with fallback
async fn wait_for_content_and_idle(
    page: &Page,
    req: &RenderReq,
    request_id: &str,
) -> Result<(), String> {
    // First, wait for DOMContentLoaded event
    if let Err(e) = timeout(Duration::from_millis(500), page.wait_for_navigation()).await {
        debug!(
            request_id = %request_id,
            "DOMContentLoaded timeout (proceeding anyway): {:?}", e
        );
    }

    // Wait for specific element if requested
    if let Some(css_selector) = &req.wait_for {
        debug!(request_id = %request_id, selector = %css_selector, "Waiting for CSS selector");

        if let Err(e) = timeout(Duration::from_millis(800), page.find_element(css_selector)).await {
            debug!(
                request_id = %request_id,
                selector = %css_selector,
                "CSS selector wait failed: {:?}", e
            );
        }
    }

    // Additional 1s idle time as per requirements
    debug!(request_id = %request_id, "Waiting for 1s idle time");
    tokio::time::sleep(Duration::from_secs(1)).await;

    Ok(())
}

/// Perform scrolling with error resilience
async fn perform_scrolling(page: &Page, steps: u32, request_id: &str) -> Result<(), String> {
    debug!(request_id = %request_id, steps = steps, "Starting scrolling");

    for step in 0..steps {
        if let Err(e) = timeout(
            Duration::from_millis(200),
            page.evaluate("window.scrollBy(0, 2000);"),
        )
        .await
        {
            debug!(
                request_id = %request_id,
                step = step,
                "Scroll step failed: {:?}", e
            );
            // Continue with remaining steps
        }

        // Smaller delay between scroll steps to save time
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    debug!(request_id = %request_id, "Scrolling completed");
    Ok(())
}

/// Extract page content with fallback handling
async fn extract_page_content(
    page: &Page,
    original_url: &str,
    request_id: &str,
) -> Result<(String, String), RenderErrorResp> {
    // Extract HTML content
    let html = timeout(Duration::from_millis(500), page.content())
        .await
        .map_err(|_| RenderErrorResp {
            error: "Content extraction timed out".to_string(),
            request_id: Some(request_id.to_string()),
            duration_ms: 500,
        })?
        .unwrap_or_else(|e| {
            warn!(request_id = %request_id, "Content extraction failed: {}", e);
            "<html><body>Content extraction failed</body></html>".to_string()
        });

    // Get final URL
    let final_url = timeout(Duration::from_millis(100), page.url())
        .await
        .ok()
        .and_then(|result| result.ok())
        .flatten()
        .unwrap_or_else(|| {
            debug!(request_id = %request_id, "Using original URL as fallback");
            original_url.to_string()
        });

    debug!(
        request_id = %request_id,
        html_length = html.len(),
        final_url = %final_url,
        "Content extraction completed"
    );

    Ok((html, final_url))
}

/// Determine stealth preset based on request configuration and environment
fn determine_stealth_preset(req: &RenderReq) -> Result<StealthPreset, RenderErrorResp> {
    // Check if stealth is explicitly configured in request
    if let Some(stealth_config) = &req.stealth_config {
        return Ok(stealth_config.preset.clone());
    }

    // Check environment variables for stealth mode
    match std::env::var("STEALTH_MODE")
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "none" | "false" | "0" => Ok(StealthPreset::None),
        "low" => Ok(StealthPreset::Low),
        "medium" | "true" | "1" => Ok(StealthPreset::Medium),
        "high" => Ok(StealthPreset::High),
        _ => Ok(StealthPreset::Medium), // Default to medium stealth
    }
}
