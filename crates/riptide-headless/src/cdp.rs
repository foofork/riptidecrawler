use crate::models::*;
use axum::{http::StatusCode, Json};
use chromiumoxide::{Browser, BrowserConfig, Page};
use futures::StreamExt;
use riptide_core::stealth::{StealthController, StealthPreset};
// Removed: std::collections::HashMap (only used by removed SessionStore)
// Removed: std::sync::Arc (only used by removed SessionStore)
use std::time::{Duration, Instant};
// Removed: tokio::sync::RwLock (only used by removed SessionStore)
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, warn};

// Removed: SessionStore type alias - never used

/// Enhanced render function with timeout management and resilience patterns
pub async fn render(
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
        render_internal(req.clone(), request_id.clone()),
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
let _ = Duration::from_millis(timeout_ms.unwrap_or(5000));
                let _ = page
                    .find_element(css)
                    .await
                    .map_err(|e| anyhow::anyhow!("CSS selector not found: {}", e))?;
                debug!("Waited for CSS selector: {}", css);
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

/// Internal render implementation with detailed error handling
async fn render_internal(
    req: RenderReq,
    request_id: String,
) -> Result<RenderResp, RenderErrorResp> {
    // Initialize stealth controller based on configuration or environment
    let stealth_preset = determine_stealth_preset(&req)?;
    let mut stealth_controller = StealthController::from_preset(stealth_preset.clone());

    // Browser configuration with Phase 3 stealth integration
    let mut builder = BrowserConfig::builder();

    // Apply stealth flags based on preset
    if stealth_preset != StealthPreset::None {
        let stealth_flags = stealth_controller.get_cdp_flags();
        for flag in stealth_flags {
            builder = builder.arg(&flag);
        }

        // Set user agent if stealth is enabled
        let user_agent = stealth_controller.next_user_agent();
        builder = builder.arg(format!("--user-agent={}", user_agent));

        debug!(
            request_id = %request_id,
            preset = ?stealth_preset,
            user_agent = %user_agent,
            "Applied stealth configuration"
        );
    } else {
        builder = builder.with_head(); // Keep headful for debugging when not in stealth
    }

    let browser_config = builder.build().map_err(|e| RenderErrorResp {
        error: format!("Failed to build browser config: {}", e),
        request_id: Some(request_id.clone()),
        duration_ms: 0,
    })?;

    // Launch browser with timeout
    let (mut browser, mut handler) = timeout(
        Duration::from_millis(1500), // Allow 1.5s for browser launch
        Browser::launch(browser_config),
    )
    .await
    .map_err(|_| RenderErrorResp {
        error: "Browser launch timed out".to_string(),
        request_id: Some(request_id.clone()),
        duration_ms: 1500,
    })?
    .map_err(|e| RenderErrorResp {
        error: format!("Failed to launch browser: {}", e),
        request_id: Some(request_id.clone()),
        duration_ms: 0,
    })?;

    // Spawn handler task
    let handler_task = tokio::spawn(async move {
        while let Some(event) = handler.next().await {
            if let Err(e) = event {
                debug!("Browser event error: {}", e);
            }
        }
    });

    // Create page with navigation timeout
    let page_result = timeout(
        Duration::from_millis(1000), // 1s for page creation and navigation
        create_and_navigate_page(&browser, &req.url),
    )
    .await
    .map_err(|_| RenderErrorResp {
        error: "Page navigation timed out".to_string(),
        request_id: Some(request_id.clone()),
        duration_ms: 1000,
    })?
    .map_err(|e| RenderErrorResp {
        error: e,
        request_id: Some(request_id.clone()),
        duration_ms: 0,
    })?;

    let page = page_result;

    // Execute any custom actions if provided
    if let Some(actions) = &req.actions {
        if let Err(e) = exec_actions(&page, actions).await {
            warn!(
                request_id = %request_id,
                error = %e,
                "Action execution failed, proceeding with current state"
            );
        }
    } else {
        // Legacy path: wait for DOMContentLoaded + 1s idle time
        if let Err(e) = wait_for_content_and_idle(&page, &req, &request_id).await {
            warn!(
                request_id = %request_id,
                error = %e,
                "Content wait failed, proceeding with current state"
            );
        }
    }

    // Perform scrolling if requested
    if let Some(steps) = req.scroll_steps {
        if let Err(e) = perform_scrolling(&page, steps, &request_id).await {
            warn!(
                request_id = %request_id,
                error = %e,
                "Scrolling failed, proceeding with current content"
            );
        }
    }

    // Extract content
    let (html, final_url) = extract_page_content(&page, &req.url, &request_id).await?;

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
            // MHTML capture would require CDP command not yet exposed in chromiumoxide
            debug!(request_id = %request_id, "MHTML capture not yet implemented");
        }
    }

    // Clean up
    handler_task.abort();
    if let Err(e) = browser.close().await {
        debug!(request_id = %request_id, "Browser cleanup warning: {}", e);
    }

    Ok(RenderResp {
        final_url,
        html,
        screenshot_b64: artifacts_out.screenshot_b64.clone(), // For backward compatibility
        session_id: req.session_id.clone(),                   // Echo back session ID for now
        artifacts: artifacts_out,
    })
}

/// Inject stealth JavaScript early in page lifecycle
async fn inject_stealth_js(page: &Page) -> Result<(), String> {
    // Read the stealth.js file and inject it
    let stealth_js = include_str!("stealth.js");

    // Use addScriptToEvaluateOnNewDocument to inject before any page scripts run
    page.evaluate_on_new_document(stealth_js)
        .await
        .map_err(|e| format!("Failed to inject stealth JS: {}", e))?;

    debug!("Stealth JavaScript injected successfully");
    Ok(())
}

/// Create page and navigate with proper error handling
async fn create_and_navigate_page(browser: &Browser, url: &str) -> Result<Page, String> {
    let page = browser
        .new_page(url)
        .await
        .map_err(|e| format!("Failed to create page: {}", e))?;

    // Inject stealth JavaScript early in page lifecycle
    if let Err(e) = inject_stealth_js(&page).await {
        debug!("Stealth JS injection failed (non-critical): {}", e);
    }

    // Wait for basic navigation to complete
    if let Err(e) = page.wait_for_navigation().await {
        debug!("Navigation wait failed (non-critical): {}", e);
    }

    Ok(page)
}

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
