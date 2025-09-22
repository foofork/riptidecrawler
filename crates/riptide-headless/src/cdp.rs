use crate::models::*;
use axum::{http::StatusCode, Json};
use chromiumoxide::{Browser, BrowserConfig, Page};
use futures::StreamExt;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

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

/// Internal render implementation with detailed error handling
async fn render_internal(
    req: RenderReq,
    request_id: String,
) -> Result<RenderResp, RenderErrorResp> {
    // Browser configuration with optimizations for speed
    let browser_config = BrowserConfig::builder()
        .with_head() // Keep headful for now, can be made headless for production
        .build()
        .map_err(|e| RenderErrorResp {
            error: format!("Failed to build browser config: {}", e),
            request_id: Some(request_id.clone()),
            duration_ms: 0,
        })?;

    // Launch browser with timeout
    let (browser, mut handler) = timeout(
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

    // Wait for DOMContentLoaded + 1s idle time as per requirements
    if let Err(e) = wait_for_content_and_idle(&page, &req, &request_id).await {
        warn!(
            request_id = %request_id,
            error = %e,
            "Content wait failed, proceeding with current state"
        );
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

    // Clean up
    handler_task.abort();
    if let Err(e) = browser.close().await {
        debug!(request_id = %request_id, "Browser cleanup warning: {}", e);
    }

    Ok(RenderResp {
        final_url,
        html,
        screenshot_b64: None,
    })
}

/// Create page and navigate with proper error handling
async fn create_and_navigate_page(browser: &Browser, url: &str) -> Result<Page, String> {
    let page = browser
        .new_page(url)
        .await
        .map_err(|e| format!("Failed to create page: {}", e))?;

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
    if let Err(e) = timeout(
        Duration::from_millis(500),
        page.wait_for_load_state(
            chromiumoxide::cdp::browser_protocol::page::LoadState::DomContentLoaded,
        ),
    )
    .await
    {
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
