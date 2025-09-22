use crate::models::*;
use axum::Json;
use chromiumoxide::{Browser, BrowserConfig};
use futures::StreamExt;

pub async fn render(Json(req): Json<RenderReq>) -> Json<RenderResp> {
    let browser_config = BrowserConfig::builder()
        .with_head()
        .build()
        .expect("Failed to build browser config");

    let (browser, mut handler) = Browser::launch(browser_config)
        .await
        .expect("Failed to launch browser");

    tokio::spawn(async move {
        while let Some(_e) = handler.next().await {}
    });

    let page = browser
        .new_page(&req.url)
        .await
        .expect("Failed to create new page");

    if let Some(css) = &req.wait_for {
        page.wait_for_element(css).await.ok();
    }

    if let Some(steps) = req.scroll_steps {
        for _ in 0..steps {
            page.evaluate("window.scrollBy(0, 2000);").await.ok();
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }
    }

    let html = page.content().await.unwrap_or_default();
    let final_url = page.url().await.unwrap_or_else(|_| req.url.clone());

    Json(RenderResp {
        final_url,
        html,
        screenshot_b64: None,
    })
}