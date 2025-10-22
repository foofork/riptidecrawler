/// End-to-end CLI workflow tests for RipTide
/// Tests complete user workflows and command pipelines
use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;
use std::time::Duration;
use wiremock::{
    matchers::{method, path, query_param},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn test_complete_crawl_workflow() {
    let mock_server = MockServer::start().await;

    // Mock crawl initiation
    Mock::given(method("POST"))
        .and(path("/api/v1/crawl"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "job_id": "crawl-12345",
            "status": "started",
            "url": "https://example.com",
            "max_depth": 2
        })))
        .mount(&mock_server)
        .await;

    // Mock crawl status
    Mock::given(method("GET"))
        .and(path("/api/v1/crawl/crawl-12345"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "job_id": "crawl-12345",
            "status": "completed",
            "pages_crawled": 45,
            "urls_discovered": 120,
            "duration_ms": 5400
        })))
        .mount(&mock_server)
        .await;

    // Start crawl
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("crawl")
        .arg("--url")
        .arg("https://example.com")
        .arg("--max-depth")
        .arg("2")
        .assert()
        .success()
        .stdout(predicate::str::contains("crawl-12345"));

    // Check crawl status
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("crawl")
        .arg("status")
        .arg("crawl-12345")
        .assert()
        .success()
        .stdout(predicate::str::contains("completed"))
        .stdout(predicate::str::contains("45"));
}

#[tokio::test]
async fn test_search_and_extract_pipeline() {
    let mock_server = MockServer::start().await;

    // Mock search endpoint
    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .and(query_param("q", "rust programming"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [
                {
                    "url": "https://rust-lang.org/learn",
                    "title": "Learn Rust",
                    "snippet": "Get started with Rust programming"
                },
                {
                    "url": "https://doc.rust-lang.org/book",
                    "title": "The Rust Programming Language",
                    "snippet": "The official Rust book"
                }
            ],
            "total": 2
        })))
        .mount(&mock_server)
        .await;

    // Mock extract for first result
    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "Get started with Rust programming language. Learn the basics and advanced concepts.",
            "method_used": "trek",
            "confidence": 0.93,
            "extraction_time_ms": 67
        })))
        .mount(&mock_server)
        .await;

    // Search
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("search")
        .arg("--query")
        .arg("rust programming")
        .assert()
        .success()
        .stdout(predicate::str::contains("Learn Rust"))
        .stdout(predicate::str::contains("rust-lang.org"));

    // Extract from search result
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://rust-lang.org/learn")
        .assert()
        .success()
        .stdout(predicate::str::contains("Get started with Rust"));
}

#[tokio::test]
async fn test_cache_utilization_workflow() {
    let mock_server = MockServer::start().await;

    // First extraction - cache miss
    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "Cached content example",
            "method_used": "trek",
            "confidence": 0.91,
            "cache_hit": false,
            "extraction_time_ms": 120
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Second extraction - cache hit
    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "Cached content example",
            "method_used": "trek",
            "confidence": 0.91,
            "cache_hit": true,
            "extraction_time_ms": 5
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Cache status
    Mock::given(method("GET"))
        .and(path("/admin/cache/stats"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "total_keys": 1,
            "memory_used": 2048,
            "hit_rate": 0.50,
            "hits": 1,
            "misses": 1
        })))
        .mount(&mock_server)
        .await;

    // First extraction (miss)
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com/page")
        .assert()
        .success();

    // Second extraction (hit)
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com/page")
        .assert()
        .success();

    // Check cache stats
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("cache")
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Total Keys: 1"))
        .stdout(predicate::str::contains("Hit Rate: 50"));
}

#[tokio::test]
async fn test_output_format_consistency() {
    let mock_server = MockServer::start().await;

    let extraction_data = json!({
        "content": "Test content for format validation",
        "method_used": "trek",
        "confidence": 0.88,
        "metadata": {
            "title": "Test Page",
            "word_count": 5
        },
        "extraction_time_ms": 75
    });

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(extraction_data.clone()))
        .expect(3)
        .mount(&mock_server)
        .await;

    // Test JSON output
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("--output")
        .arg("json")
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"content\""))
        .stdout(predicate::str::contains("\"confidence\""));

    // Test table output
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("--output")
        .arg("table")
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .assert()
        .success();

    // Test plain text output (default)
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .assert()
        .success()
        .stdout(predicate::str::contains("Test content for format validation"));
}

#[tokio::test]
async fn test_multi_page_batch_extraction() {
    let mock_server = MockServer::start().await;

    // Mock batch extraction endpoint
    Mock::given(method("POST"))
        .and(path("/api/v1/batch/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [
                {
                    "url": "https://example.com/page1",
                    "content": "Content from page 1",
                    "confidence": 0.92,
                    "success": true
                },
                {
                    "url": "https://example.com/page2",
                    "content": "Content from page 2",
                    "confidence": 0.88,
                    "success": true
                },
                {
                    "url": "https://example.com/page3",
                    "content": "Content from page 3",
                    "confidence": 0.90,
                    "success": true
                }
            ],
            "total": 3,
            "successful": 3,
            "failed": 0
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("batch")
        .arg("extract")
        .arg("--urls")
        .arg("https://example.com/page1,https://example.com/page2,https://example.com/page3")
        .assert()
        .success()
        .stdout(predicate::str::contains("page 1"))
        .stdout(predicate::str::contains("page 2"))
        .stdout(predicate::str::contains("page 3"));
}

#[tokio::test]
async fn test_health_check_before_operations() {
    let mock_server = MockServer::start().await;

    // Health check
    Mock::given(method("GET"))
        .and(path("/healthz"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/health/detailed"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "healthy",
            "healthy": true,
            "redis": "connected",
            "extractor": "ready",
            "http_client": "ready",
            "worker_service": "ready"
        })))
        .mount(&mock_server)
        .await;

    // Extract operation
    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "Extraction after health check",
            "method_used": "trek",
            "confidence": 0.89,
            "extraction_time_ms": 82
        })))
        .mount(&mock_server)
        .await;

    // Check health first
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("health")
        .assert()
        .success();

    // Then perform extraction
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .assert()
        .success()
        .stdout(predicate::str::contains("Extraction after health check"));
}

#[tokio::test]
async fn test_error_recovery_workflow() {
    let mock_server = MockServer::start().await;

    // First attempt fails
    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Retry succeeds
    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "Successfully extracted after retry",
            "method_used": "trek",
            "confidence": 0.87,
            "extraction_time_ms": 95
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .arg("--retry")
        .arg("3")
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully extracted"));
}

#[tokio::test]
async fn test_validate_then_extract_workflow() {
    let mock_server = MockServer::start().await;

    // Validation mocks
    Mock::given(method("GET"))
        .and(path("/healthz"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/health/detailed"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "redis": "connected"
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/monitoring/wasm-instances"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/workers/stats/workers"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    // Extract mock
    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "Validated system, extraction successful",
            "method_used": "trek",
            "confidence": 0.94,
            "extraction_time_ms": 71
        })))
        .mount(&mock_server)
        .await;

    // Validate
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("validate")
        .assert()
        .success();

    // Extract
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .assert()
        .success()
        .stdout(predicate::str::contains("Validated system"));
}

#[tokio::test]
async fn test_concurrent_extractions() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "Concurrent extraction result",
            "method_used": "trek",
            "confidence": 0.86,
            "extraction_time_ms": 60
        })))
        .expect(5)
        .mount(&mock_server)
        .await;

    // Simulate concurrent extractions
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let server_uri = mock_server.uri();
            tokio::spawn(async move {
                let mut cmd = Command::cargo_bin("riptide").unwrap();
                cmd.arg("--api-url")
                    .arg(&server_uri)
                    .arg("extract")
                    .arg("--url")
                    .arg(format!("https://example.com/page{}", i))
                    .assert()
                    .success();
            })
        })
        .collect();

    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_progress_monitoring_workflow() {
    let mock_server = MockServer::start().await;

    // Start long-running job
    Mock::given(method("POST"))
        .and(path("/api/v1/crawl"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "job_id": "progress-test-123",
            "status": "running",
            "progress": 0
        })))
        .mount(&mock_server)
        .await;

    // Progress updates
    Mock::given(method("GET"))
        .and(path("/api/v1/crawl/progress-test-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "job_id": "progress-test-123",
            "status": "running",
            "progress": 45,
            "pages_crawled": 23,
            "urls_discovered": 67
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("crawl")
        .arg("status")
        .arg("progress-test-123")
        .assert()
        .success()
        .stdout(predicate::str::contains("running"))
        .stdout(predicate::str::contains("45"));
}

#[tokio::test]
async fn test_configuration_persistence() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "Extraction with persisted config",
            "method_used": "trek",
            "confidence": 0.91,
            "extraction_time_ms": 68
        })))
        .expect(2)
        .mount(&mock_server)
        .await;

    // First command with API URL
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com/first")
        .assert()
        .success();

    // Second command should use same config
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com/second")
        .assert()
        .success();
}

#[tokio::test]
async fn test_metadata_extraction_workflow() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "Article with rich metadata",
            "method_used": "trek",
            "confidence": 0.93,
            "metadata": {
                "title": "Complete Guide to Web Scraping",
                "author": "Tech Writer",
                "published_date": "2025-10-01",
                "tags": ["web-scraping", "automation", "rust"],
                "word_count": 1500,
                "reading_time_minutes": 7,
                "images": 5,
                "links": 23
            },
            "extraction_time_ms": 112
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com/guide")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("metadata"))
        .stdout(predicate::str::contains("Complete Guide"))
        .stdout(predicate::str::contains("Tech Writer"));
}

#[tokio::test]
async fn test_strategy_performance_comparison() {
    let mock_server = MockServer::start().await;

    // Trek strategy
    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "Trek extraction result",
            "method_used": "trek",
            "confidence": 0.94,
            "extraction_time_ms": 85
        })))
        .mount(&mock_server)
        .await;

    // CSS strategy
    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "CSS extraction result",
            "method_used": "css:article",
            "confidence": 0.96,
            "extraction_time_ms": 42
        })))
        .mount(&mock_server)
        .await;

    // Test Trek
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com/test")
        .arg("--strategy")
        .arg("trek")
        .assert()
        .success()
        .stdout(predicate::str::contains("Trek extraction"));

    // Test CSS
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com/test")
        .arg("--strategy")
        .arg("css:article")
        .assert()
        .success()
        .stdout(predicate::str::contains("CSS extraction"));
}
