//! Integration tests for Spider Pages mode (result_mode=pages)
//!
//! Tests the population of CrawledPage objects in the spider handler response.
//! Note: These tests validate the current implementation which returns minimal
//! page metadata (URLs, status codes) without actual content, since the Spider
//! engine doesn't persist crawled page data.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use riptide_api::{
    dto::{CrawledPage, ResultMode, SpiderResultPages},
    handlers::spider::SpiderCrawlQuery,
    models::SpiderCrawlBody,
};
use serde_json::json;
use tower::ServiceExt;

mod common;
use common::helpers::{create_test_app, TestResponse};

#[tokio::test]
async fn test_spider_pages_mode_basic_response() {
    let app = create_test_app().await;

    let body = json!({
        "seed_urls": ["https://example.com"],
        "max_depth": 2,
        "max_pages": 5,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/spider/crawl?result_mode=pages")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let test_response: TestResponse<SpiderResultPages> =
        serde_json::from_slice(&axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap())
            .unwrap();

    // Validate response structure
    assert!(test_response.pages.is_some());
    let pages = test_response.pages.unwrap();

    // Should have pages array
    assert!(pages.len() > 0);

    // Validate page structure
    for page in &pages {
        // All pages should have URLs
        assert!(!page.url.is_empty());

        // All pages should have basic metadata
        assert_eq!(page.status_code, 200);
        assert!(page.final_url.is_some());
        assert!(page.robots_obeyed.is_some());

        // Content fields should be None (not persisted by Spider)
        assert!(page.content.is_none());
        assert!(page.markdown.is_none());
        assert!(page.title.is_none());
    }
}

#[tokio::test]
async fn test_spider_pages_mode_with_include_filter() {
    let app = create_test_app().await;

    let body = json!({
        "seed_urls": ["https://example.com"],
        "max_depth": 1,
        "max_pages": 3,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/spider/crawl?result_mode=pages&include=url,status_code,final_url")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let test_response: TestResponse<SpiderResultPages> =
        serde_json::from_slice(&axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap())
            .unwrap();

    let pages = test_response.pages.unwrap();

    // Validate filtering was applied
    for page in &pages {
        assert!(!page.url.is_empty());
        assert!(page.final_url.is_some());
        // Excluded fields should be None
        assert!(page.title.is_none());
        assert!(page.content.is_none());
    }
}

#[tokio::test]
async fn test_spider_pages_mode_content_not_available_warning() {
    let app = create_test_app().await;

    let body = json!({
        "seed_urls": ["https://example.com"],
        "max_depth": 1,
        "max_pages": 2,
    });

    // Request content field explicitly
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/spider/crawl?result_mode=pages&include=content,markdown")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let test_response: TestResponse<SpiderResultPages> =
        serde_json::from_slice(&axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap())
            .unwrap();

    let pages = test_response.pages.unwrap();

    // When content is requested but not available, should have fetch_error
    for page in &pages {
        assert!(page.fetch_error.is_some());
        let error = page.fetch_error.as_ref().unwrap();
        assert!(error.contains("Page content not available"));
        assert!(error.contains("Spider engine does not persist crawled data"));
    }
}

#[tokio::test]
async fn test_spider_pages_mode_exclude_filter() {
    let app = create_test_app().await;

    let body = json!({
        "seed_urls": ["https://example.com"],
        "max_depth": 1,
        "max_pages": 3,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/spider/crawl?result_mode=pages&exclude=links,mime")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let test_response: TestResponse<SpiderResultPages> =
        serde_json::from_slice(&axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap())
            .unwrap();

    let pages = test_response.pages.unwrap();

    // Validate exclusion was applied
    for page in &pages {
        // Excluded fields should be empty/None
        assert!(page.links.is_empty());
        assert!(page.mime.is_none());

        // Non-excluded fields should still be present
        assert!(!page.url.is_empty());
        assert!(page.final_url.is_some());
    }
}

#[tokio::test]
async fn test_spider_pages_mode_max_content_bytes() {
    let app = create_test_app().await;

    let body = json!({
        "seed_urls": ["https://example.com"],
        "max_depth": 1,
        "max_pages": 2,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/spider/crawl?result_mode=pages&max_content_bytes=500")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Should succeed even with content size limit (content is not available anyway)
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_spider_pages_mode_api_version() {
    let app = create_test_app().await;

    let body = json!({
        "seed_urls": ["https://example.com"],
        "max_depth": 1,
        "max_pages": 1,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/spider/crawl?result_mode=pages")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let test_response: TestResponse<SpiderResultPages> =
        serde_json::from_slice(&axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap())
            .unwrap();

    // Should include api_version field
    assert_eq!(test_response.api_version, Some("v1".to_string()));
}

#[tokio::test]
async fn test_spider_pages_mode_with_statistics() {
    let app = create_test_app().await;

    let body = json!({
        "seed_urls": ["https://example.com"],
        "max_depth": 2,
        "max_pages": 10,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/spider/crawl?result_mode=pages")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let test_response: TestResponse<SpiderResultPages> =
        serde_json::from_slice(&axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap())
            .unwrap();

    // Should include statistics fields
    assert!(test_response.pages_crawled.is_some());
    assert!(test_response.pages_failed.is_some());
    assert!(test_response.duration_seconds.is_some());
    assert!(test_response.stop_reason.is_some());
    assert!(test_response.domains.is_some());

    let pages = test_response.pages.unwrap();

    // Number of pages should match pages_crawled + discovered
    let pages_crawled = test_response.pages_crawled.unwrap();
    assert!(pages.len() as u64 <= pages_crawled + test_response.pages_failed.unwrap_or(0));
}

#[tokio::test]
async fn test_spider_pages_mode_empty_result() {
    let app = create_test_app().await;

    let body = json!({
        "seed_urls": ["https://invalid-nonexistent-domain-12345.com"],
        "max_depth": 1,
        "max_pages": 1,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/spider/crawl?result_mode=pages")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should handle empty/failed crawls gracefully
    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_spider_pages_mode_pagination_compatibility() {
    let app = create_test_app().await;

    let body = json!({
        "seed_urls": ["https://example.com"],
        "max_depth": 3,
        "max_pages": 50,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/spider/crawl?result_mode=pages")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let test_response: TestResponse<SpiderResultPages> =
        serde_json::from_slice(&axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap())
            .unwrap();

    // Response structure should support future pagination
    let pages = test_response.pages.unwrap();

    // Pages should be in discovery order
    // (This is a property of the current implementation)
    for (idx, page) in pages.iter().enumerate() {
        assert!(!page.url.is_empty());
        // URLs should be unique
        for other_page in pages.iter().skip(idx + 1) {
            assert_ne!(page.url, other_page.url, "Duplicate URLs in response");
        }
    }
}
