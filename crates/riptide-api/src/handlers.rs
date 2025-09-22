use crate::models::*;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use tracing::info;

pub async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

pub async fn crawl(Json(body): Json<CrawlBody>) -> impl IntoResponse {
    info!("Received crawl request for {} URLs", body.urls.len());

    // For each URL: fetch -> gate -> fast extract or POST /render -> extract
    // Stream or collect results; here we return a simple JSON array.
    let results: Vec<CrawlResult> = vec![];

    Json(json!({
        "received": body.urls.len(),
        "results": results
    }))
}

pub async fn deepsearch(Json(body): Json<DeepSearchBody>) -> impl IntoResponse {
    info!("Deep search request: query={}", body.query);

    // Call Serper.dev -> take organic URLs -> reuse crawl flow
    // NOTE: you must set SERPER_API_KEY env var
    let limit = body.limit.unwrap_or(10);

    Json(json!({
        "query": body.query,
        "enqueued": limit,
        "status": "processing"
    }))
}

pub async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not Found")
}
