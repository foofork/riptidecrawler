/// Real-world CLI integration tests for RipTide
/// Tests against mock servers simulating real websites
use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

/// Helper to create a mock HTML response
#[allow(dead_code)]
fn create_html_response(title: &str, content: &str, article_tag: bool) -> String {
    if article_tag {
        format!(
            r#"<!DOCTYPE html>
<html>
<head><title>{}</title></head>
<body>
    <article>
        <h1>{}</h1>
        <p>{}</p>
    </article>
</body>
</html>"#,
            title, title, content
        )
    } else {
        format!(
            r#"<!DOCTYPE html>
<html>
<head><title>{}</title></head>
<body>
    <div class="content">
        <h1>{}</h1>
        <p>{}</p>
    </div>
</body>
</html>"#,
            title, title, content
        )
    }
}

#[tokio::test]
async fn test_wikipedia_article_extraction() {
    let mock_server = MockServer::start().await;

    // Simulate Wikipedia article structure
    let _wiki_html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <div id="content" class="mw-body">
            <h1 id="firstHeading">Rust Programming Language</h1>
            <div id="bodyContent">
                <div id="mw-content-text" class="mw-body-content">
                    <p>Rust is a multi-paradigm programming language designed for performance and safety.</p>
                    <p>It provides memory safety without using garbage collection.</p>
                </div>
            </div>
        </div>
    </body>
    </html>"#;

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "Rust is a multi-paradigm programming language designed for performance and safety. It provides memory safety without using garbage collection.",
            "method_used": "css:.mw-body-content",
            "confidence": 0.92,
            "metadata": {
                "title": "Rust Programming Language",
                "word_count": 18
            },
            "extraction_time_ms": 45
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://en.wikipedia.org/wiki/Rust")
        .arg("--show-confidence")
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust is a multi-paradigm"))
        .stdout(predicate::str::contains("Confidence"));
}

#[tokio::test]
async fn test_github_readme_extraction() {
    let mock_server = MockServer::start().await;

    // Simulate GitHub README structure
    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "# RipTide\n\nHigh-performance web crawler and content extraction engine.\n\n## Features\n- Multi-strategy extraction\n- WASM acceleration\n- Confidence scoring",
            "method_used": "css:article.markdown-body",
            "confidence": 0.98,
            "metadata": {
                "title": "RipTide README",
                "type": "markdown",
                "word_count": 15
            },
            "extraction_time_ms": 32
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://github.com/user/riptide/blob/main/README.md")
        .arg("--strategy")
        .arg("css:article.markdown-body")
        .assert()
        .success()
        .stdout(predicate::str::contains("RipTide"))
        .stdout(predicate::str::contains("High-performance"));
}

#[tokio::test]
async fn test_news_article_extraction() {
    let mock_server = MockServer::start().await;

    // Simulate news article with high confidence
    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "Breaking news: Major breakthrough in quantum computing announced today. Researchers have achieved a significant milestone in quantum error correction.",
            "method_used": "trek",
            "confidence": 0.96,
            "metadata": {
                "title": "Quantum Computing Breakthrough",
                "author": "Tech Reporter",
                "published_date": "2025-10-11",
                "word_count": 20
            },
            "extraction_time_ms": 78
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://technews.example.com/quantum-breakthrough")
        .arg("--show-confidence")
        .assert()
        .success()
        .stdout(predicate::str::contains("quantum computing"))
        .stdout(predicate::str::contains("Confidence: 0.96"));
}

#[tokio::test]
async fn test_documentation_page_extraction() {
    let mock_server = MockServer::start().await;

    // Simulate technical documentation
    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "## Installation\n\nInstall RipTide using cargo:\n\n```bash\ncargo install riptide\n```\n\n## Usage\n\nExtract content from a URL:\n\n```bash\nriptide extract --url https://example.com\n```",
            "method_used": "css:.documentation-content",
            "confidence": 0.94,
            "metadata": {
                "title": "RipTide Documentation - Installation",
                "section": "getting-started",
                "code_blocks": 2
            },
            "extraction_time_ms": 55
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://docs.riptide.io/installation")
        .arg("--strategy")
        .arg("css:.documentation-content")
        .assert()
        .success()
        .stdout(predicate::str::contains("Installation"))
        .stdout(predicate::str::contains("cargo install"));
}

#[tokio::test]
async fn test_blog_post_extraction_with_metadata() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "Web scraping has evolved significantly in recent years. Modern tools like RipTide offer sophisticated extraction strategies that adapt to different content types and structures.",
            "method_used": "trek",
            "confidence": 0.89,
            "metadata": {
                "title": "The Evolution of Web Scraping",
                "author": "Jane Developer",
                "published_date": "2025-10-01",
                "tags": ["web-scraping", "automation", "rust"],
                "word_count": 25,
                "reading_time_minutes": 1
            },
            "extraction_time_ms": 120
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://devblog.example.com/web-scraping-evolution")
        .arg("--show-confidence")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("Web scraping"))
        .stdout(predicate::str::contains("metadata"));
}

#[tokio::test]
async fn test_strategy_fallback_cascade() {
    let mock_server = MockServer::start().await;

    // First strategy fails, falls back to second
    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "Content extracted successfully using fallback strategy.",
            "method_used": "fallback:css:article->trek",
            "confidence": 0.82,
            "metadata": {
                "strategies_tried": ["css:article", "trek"],
                "successful_strategy": "trek"
            },
            "extraction_time_ms": 145
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com/complex-page")
        .arg("--strategy")
        .arg("fallback:css:article,trek")
        .assert()
        .success()
        .stdout(predicate::str::contains("Content extracted successfully"));
}

#[tokio::test]
async fn test_parallel_strategy_execution() {
    let mock_server = MockServer::start().await;

    // Multiple strategies run in parallel, best result selected
    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "Parallel extraction results in highest quality content.",
            "method_used": "parallel:trek|css|regex",
            "confidence": 0.97,
            "metadata": {
                "strategies_executed": ["trek", "css:article", "regex"],
                "results_count": 3,
                "best_confidence": 0.97,
                "selected_strategy": "trek"
            },
            "extraction_time_ms": 89
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com/article")
        .arg("--strategy")
        .arg("parallel:trek,css:article,regex")
        .arg("--show-confidence")
        .assert()
        .success()
        .stdout(predicate::str::contains("Parallel extraction"))
        .stdout(predicate::str::contains("0.97"));
}

#[tokio::test]
async fn test_confidence_threshold_filtering() {
    let mock_server = MockServer::start().await;

    // Low confidence extraction should warn user
    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "Extracted content with lower confidence.",
            "method_used": "css:.unknown-selector",
            "confidence": 0.45,
            "metadata": {
                "warning": "Low confidence extraction"
            },
            "extraction_time_ms": 67
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com/difficult-page")
        .arg("--show-confidence")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.45"));
}

#[tokio::test]
async fn test_structured_data_extraction() {
    let mock_server = MockServer::start().await;

    // Extract structured product data
    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "Product: Premium Headphones - $299.99",
            "method_used": "css:.product-details",
            "confidence": 0.93,
            "metadata": {
                "product_name": "Premium Headphones",
                "price": 299.99,
                "currency": "USD",
                "availability": "in_stock",
                "rating": 4.5
            },
            "extraction_time_ms": 42
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://shop.example.com/product/headphones")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("Premium Headphones"))
        .stdout(predicate::str::contains("299.99"));
}

#[tokio::test]
async fn test_error_404_not_found() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Page not found",
            "status": 404
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com/nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("404").or(predicate::str::contains("not found")));
}

#[tokio::test]
async fn test_error_timeout() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(408).set_body_json(json!({
            "error": "Request timeout",
            "status": 408
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://slowsite.example.com/page")
        .assert()
        .failure();
}

#[tokio::test]
async fn test_error_invalid_url() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "Invalid URL format",
            "status": 400
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("not-a-valid-url")
        .assert()
        .failure();
}

#[tokio::test]
async fn test_rate_limiting_handling() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(429).set_body_json(json!({
            "error": "Rate limit exceeded",
            "status": 429,
            "retry_after": 60
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com/page")
        .assert()
        .failure();
}

#[tokio::test]
async fn test_content_type_detection() {
    let mock_server = MockServer::start().await;

    // Test HTML content
    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "HTML article content",
            "method_used": "trek",
            "confidence": 0.91,
            "metadata": {
                "content_type": "text/html",
                "charset": "utf-8"
            },
            "extraction_time_ms": 56
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com/article.html")
        .assert()
        .success()
        .stdout(predicate::str::contains("HTML article content"));
}

#[tokio::test]
async fn test_large_content_extraction() {
    let mock_server = MockServer::start().await;

    // Simulate large document extraction
    let large_content = "Lorem ipsum ".repeat(500); // ~6KB of text

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": large_content,
            "method_used": "trek",
            "confidence": 0.88,
            "metadata": {
                "word_count": 1000,
                "size_bytes": 6000
            },
            "extraction_time_ms": 234
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com/long-article")
        .assert()
        .success()
        .stdout(predicate::str::contains("Lorem ipsum"));
}

#[tokio::test]
async fn test_multilingual_content_extraction() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "こんにちは、世界！Hello, World! Bonjour le monde!",
            "method_used": "trek",
            "confidence": 0.90,
            "metadata": {
                "languages_detected": ["ja", "en", "fr"],
                "primary_language": "ja"
            },
            "extraction_time_ms": 78
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.jp/multilingual")
        .assert()
        .success()
        .stdout(predicate::str::contains("こんにちは"));
}

#[tokio::test]
async fn test_extraction_with_custom_headers() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "Extracted with custom headers",
            "method_used": "trek",
            "confidence": 0.87,
            "extraction_time_ms": 91
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com/protected")
        .arg("--api-key")
        .arg("custom-api-key")
        .assert()
        .success()
        .stdout(predicate::str::contains("Extracted with custom headers"));
}

#[tokio::test]
async fn test_regex_strategy_extraction() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "Email: contact@example.com, Phone: +1-555-0123",
            "method_used": "regex",
            "confidence": 0.85,
            "metadata": {
                "pattern": r"[\w\.-]+@[\w\.-]+\.\w+",
                "matches": 1
            },
            "extraction_time_ms": 23
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com/contact")
        .arg("--strategy")
        .arg("regex")
        .assert()
        .success()
        .stdout(predicate::str::contains("contact@example.com"));
}

#[tokio::test]
async fn test_css_selector_strategy() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "Main article content extracted via CSS selector",
            "method_used": "css:main.content article",
            "confidence": 0.95,
            "metadata": {
                "selector": "main.content article",
                "elements_matched": 1
            },
            "extraction_time_ms": 38
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com/article")
        .arg("--strategy")
        .arg("css:main.content article")
        .assert()
        .success()
        .stdout(predicate::str::contains("Main article content"));
}
