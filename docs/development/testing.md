# Testing Strategies for RipTide Crawler

This guide outlines the comprehensive testing approach for RipTide Crawler, covering unit tests, integration tests, performance tests, and end-to-end validation.

## Testing Philosophy

### Test Pyramid
RipTide follows the test pyramid approach:

1. **Unit Tests (70%)**: Fast, isolated tests for individual components
2. **Integration Tests (20%)**: Tests for component interactions
3. **End-to-End Tests (10%)**: Full system tests with real services

### Quality Gates
- **Coverage Target**: Minimum 80% code coverage
- **Performance Requirements**: P95 latency under 5 seconds
- **Reliability Target**: 99.9% success rate on stable URLs

## Test Categories

### 1. Unit Tests

#### Testing Strategy
- Test all public functions and methods
- Mock external dependencies
- Focus on edge cases and error conditions
- Use property-based testing for complex logic

#### Example: Gate Decision Logic
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_gate_decision_high_quality_content() {
        let features = GateFeatures {
            html_bytes: 5000,
            visible_text_chars: 3000,
            p_count: 15,
            article_count: 1,
            h1h2_count: 3,
            script_bytes: 500,
            has_og: true,
            has_jsonld_article: true,
            spa_markers: 0,
            domain_prior: 0.8,
        };

        let decision = decide(&features, 0.7, 0.3);
        assert_eq!(decision, Decision::Raw);

        let score = score(&features);
        assert!(score > 0.7, "Score should be high for quality content: {}", score);
    }

    #[test]
    fn test_gate_decision_spa_content() {
        let features = GateFeatures {
            html_bytes: 8000,
            visible_text_chars: 200,
            p_count: 2,
            article_count: 0,
            h1h2_count: 1,
            script_bytes: 6000,
            has_og: false,
            has_jsonld_article: false,
            spa_markers: 3, // High SPA markers
            domain_prior: 0.5,
        };

        let decision = decide(&features, 0.7, 0.3);
        assert_eq!(decision, Decision::Headless);

        let score = score(&features);
        assert!(score < 0.3, "Score should be low for SPA content: {}", score);
    }

    #[test]
    fn test_gate_decision_boundary_cases() {
        let features = GateFeatures {
            html_bytes: 0, // Edge case: empty content
            visible_text_chars: 0,
            p_count: 0,
            article_count: 0,
            h1h2_count: 0,
            script_bytes: 0,
            has_og: false,
            has_jsonld_article: false,
            spa_markers: 0,
            domain_prior: 0.5,
        };

        let decision = decide(&features, 0.7, 0.3);
        assert_eq!(decision, Decision::ProbesFirst);

        let score = score(&features);
        assert_eq!(score, 0.5); // Should be domain prior when no other signals
    }

    // Property-based testing
    proptest! {
        #[test]
        fn test_score_always_in_range(
            html_bytes in 0u32..100000,
            visible_text_chars in 0u32..50000,
            script_bytes in 0u32..20000,
            p_count in 0u32..100,
            domain_prior in 0.0f32..1.0,
        ) {
            let features = GateFeatures {
                html_bytes: html_bytes as usize,
                visible_text_chars: visible_text_chars as usize,
                script_bytes: script_bytes as usize,
                p_count,
                article_count: 0,
                h1h2_count: 0,
                has_og: false,
                has_jsonld_article: false,
                spa_markers: 0,
                domain_prior,
            };

            let score = score(&features);
            prop_assert!(score >= 0.0 && score <= 1.0);
        }
    }
}
```

#### Testing WASM Extraction
```rust
#[cfg(test)]
mod wasm_tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_wasm_extractor_with_simple_article() {
        let wasm_path = build_test_wasm_module().await;
        let extractor = WasmExtractor::new(&wasm_path).unwrap();

        let html = r#"
            <html>
                <head><title>Test Article</title></head>
                <body>
                    <article>
                        <h1>Main Title</h1>
                        <p>This is the main content of the article.</p>
                        <p>Another paragraph with <a href="/link">a link</a>.</p>
                    </article>
                </body>
            </html>
        "#;

        let result = extractor.extract(
            html.as_bytes(),
            "https://example.com/article",
            "article"
        ).unwrap();

        assert_eq!(result.title, Some("Test Article".to_string()));
        assert!(result.text.contains("main content"));
        assert!(result.links.contains(&"https://example.com/link".to_string()));
        assert!(!result.markdown.is_empty());
    }

    #[tokio::test]
    async fn test_wasm_extractor_error_handling() {
        let wasm_path = build_test_wasm_module().await;
        let extractor = WasmExtractor::new(&wasm_path).unwrap();

        // Test with malformed HTML
        let html = b"<html><body><p>Unclosed tags everywhere";

        let result = extractor.extract(html, "https://example.com", "article");

        // Should handle gracefully, not panic
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert!(!doc.text.is_empty()); // Should extract something
    }

    async fn build_test_wasm_module() -> String {
        // Build the WASM module for testing
        let output = std::process::Command::new("cargo")
            .args(&[
                "build",
                "--release",
                "--target", "wasm32-wasip2",
                "--manifest-path", "wasm/riptide-extractor-wasm/Cargo.toml"
            ])
            .output()
            .expect("Failed to build WASM module");

        assert!(output.status.success(), "WASM build failed");

        "target/wasm32-wasip2/release/riptide-extractor-wasm.wasm".to_string()
    }
}
```

### 2. Integration Tests

#### Testing API Endpoints
```rust
// tests/api_integration.rs
use riptide_api::create_app;
use axum_test::TestServer;
use serde_json::json;

#[tokio::test]
async fn test_crawl_endpoint_success() {
    let app = create_app().await;
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/crawl")
        .json(&json!({
            "urls": ["https://httpbin.org/html"],
            "options": {
                "concurrency": 1,
                "cache_mode": "bypass"
            }
        }))
        .await;

    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert!(body["results"].is_array());

    let results = body["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);

    let result = &results[0];
    assert_eq!(result["url"], "https://httpbin.org/html");
    assert_eq!(result["status"], 200);
    assert!(result["json_path"].is_string());
}

#[tokio::test]
async fn test_crawl_endpoint_with_invalid_urls() {
    let app = create_app().await;
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/crawl")
        .json(&json!({
            "urls": ["not-a-url", "ftp://invalid-scheme.com"]
        }))
        .await;

    response.assert_status(400);

    let body: serde_json::Value = response.json();
    assert!(body["error"].is_string());
    assert!(body["error"].as_str().unwrap().contains("Invalid URL"));
}

#[tokio::test]
async fn test_deepsearch_endpoint() {
    let app = create_app().await;
    let server = TestServer::new(app).unwrap();

    // Skip if no API key
    if std::env::var("SERPER_API_KEY").is_err() {
        return;
    }

    let response = server
        .post("/deepsearch")
        .json(&json!({
            "query": "rust programming language",
            "limit": 5,
            "country": "us",
            "locale": "en"
        }))
        .await;

    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert!(body["query"].is_string());
    assert!(body["results"].is_array());
}
```

#### Testing Headless Service Integration
```rust
// tests/headless_integration.rs
use riptide_headless::models::{RenderReq, RenderResp};
use reqwest::Client;

#[tokio::test]
async fn test_headless_render_static_page() {
    let client = Client::new();

    let request = RenderReq {
        url: "https://httpbin.org/html".to_string(),
        wait_for: None,
        scroll_steps: None,
    };

    let response = client
        .post("http://localhost:9123/render")
        .json(&request)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let render_resp: RenderResp = response.json().await.unwrap();
    assert!(!render_resp.html.is_empty());
    assert!(render_resp.html.contains("<html"));
    assert_eq!(render_resp.final_url, request.url);
}

#[tokio::test]
async fn test_headless_render_with_wait_for() {
    let client = Client::new();

    let request = RenderReq {
        url: "https://httpbin.org/delay/2".to_string(),
        wait_for: Some("body".to_string()),
        scroll_steps: None,
    };

    let start_time = std::time::Instant::now();
    let response = client
        .post("http://localhost:9123/render")
        .json(&request)
        .send()
        .await
        .unwrap();

    let duration = start_time.elapsed();
    assert!(duration.as_secs() >= 2); // Should wait for delay

    assert_eq!(response.status(), 200);
    let render_resp: RenderResp = response.json().await.unwrap();
    assert!(!render_resp.html.is_empty());
}
```

### 3. End-to-End Tests

#### Full Stack Testing
```rust
// tests/e2e_full_stack.rs
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_full_stack_crawl_workflow() {
    // Start services using docker-compose
    let _compose = DockerCompose::start().await;

    // Wait for services to be ready
    wait_for_service_health().await;

    // Test the full crawl workflow
    let client = reqwest::Client::new();

    // 1. Crawl a simple page
    let crawl_response = client
        .post("http://localhost:8080/crawl")
        .json(&json!({
            "urls": ["https://example.com"]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(crawl_response.status(), 200);

    let body: serde_json::Value = crawl_response.json().await.unwrap();
    let results = body["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);

    // 2. Verify extracted content was stored
    let result = &results[0];
    let json_path = result["json_path"].as_str().unwrap();

    // Check that the file exists and contains valid JSON
    let content = std::fs::read_to_string(json_path).unwrap();
    let extracted_doc: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert!(extracted_doc["title"].is_string());
    assert!(extracted_doc["text"].is_string());
    assert!(extracted_doc["url"].as_str().unwrap().contains("example.com"));

    // 3. Test caching behavior
    let cached_response = client
        .post("http://localhost:8080/crawl")
        .json(&json!({
            "urls": ["https://example.com"]
        }))
        .send()
        .await
        .unwrap();

    let cached_body: serde_json::Value = cached_response.json().await.unwrap();
    let cached_result = &cached_body["results"][0];
    assert_eq!(cached_result["from_cache"], true);
}

struct DockerCompose;

impl DockerCompose {
    async fn start() -> Self {
        let output = Command::new("docker-compose")
            .args(&["-f", "docker-compose.test.yml", "up", "-d"])
            .output()
            .expect("Failed to start docker-compose");

        assert!(output.status.success(), "Docker compose failed to start");
        Self
    }
}

impl Drop for DockerCompose {
    fn drop(&mut self) {
        let _ = Command::new("docker-compose")
            .args(&["-f", "docker-compose.test.yml", "down"])
            .output();
    }
}

async fn wait_for_service_health() {
    let client = reqwest::Client::new();
    let max_attempts = 30;

    for attempt in 1..=max_attempts {
        if let Ok(response) = client.get("http://localhost:8080/health").send().await {
            if response.status().is_success() {
                println!("Services ready after {} attempts", attempt);
                return;
            }
        }

        if attempt == max_attempts {
            panic!("Services failed to become healthy");
        }

        sleep(Duration::from_secs(2)).await;
    }
}
```

### 4. Golden Tests

#### Reference Data Testing
```rust
// tests/golden_tests.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct GoldenTestCase {
    url: String,
    expected_title: Option<String>,
    expected_text_contains: Vec<String>,
    expected_links_contain: Vec<String>,
    min_text_length: usize,
}

#[tokio::test]
async fn test_golden_extraction_cases() {
    let test_cases = load_golden_test_cases();

    for test_case in test_cases {
        println!("Testing golden case: {}", test_case.url);

        let extracted = extract_url(&test_case.url).await
            .expect(&format!("Failed to extract {}", test_case.url));

        // Verify title
        if let Some(expected_title) = &test_case.expected_title {
            assert_eq!(
                extracted.title.as_ref().unwrap(),
                expected_title,
                "Title mismatch for {}", test_case.url
            );
        }

        // Verify text content
        assert!(
            extracted.text.len() >= test_case.min_text_length,
            "Text too short for {}: {} < {}",
            test_case.url,
            extracted.text.len(),
            test_case.min_text_length
        );

        // Verify text contains expected strings
        for expected_text in &test_case.expected_text_contains {
            assert!(
                extracted.text.contains(expected_text),
                "Text should contain '{}' for {}",
                expected_text,
                test_case.url
            );
        }

        // Verify links
        for expected_link in &test_case.expected_links_contain {
            assert!(
                extracted.links.iter().any(|link| link.contains(expected_link)),
                "Links should contain '{}' for {}",
                expected_link,
                test_case.url
            );
        }
    }
}

fn load_golden_test_cases() -> Vec<GoldenTestCase> {
    let content = std::fs::read_to_string("tests/golden/test_cases.json")
        .expect("Failed to read golden test cases");

    serde_json::from_str(&content)
        .expect("Failed to parse golden test cases")
}

// Create golden test data
#[ignore] // Run manually to update golden data
#[tokio::test]
async fn create_golden_test_data() {
    let urls = vec![
        "https://en.wikipedia.org/wiki/Rust_(programming_language)",
        "https://blog.rust-lang.org/2021/05/06/Rust-1.52.0.html",
        "https://doc.rust-lang.org/book/ch01-01-installation.html",
    ];

    let mut test_cases = Vec::new();

    for url in urls {
        let extracted = extract_url(url).await.unwrap();

        let test_case = GoldenTestCase {
            url: url.to_string(),
            expected_title: extracted.title.clone(),
            expected_text_contains: vec![
                // Extract key phrases from the content
                extracted.text.split_whitespace()
                    .take(5)
                    .collect::<Vec<_>>()
                    .join(" ")
            ],
            expected_links_contain: extracted.links
                .iter()
                .filter(|link| link.starts_with("http"))
                .take(3)
                .map(|link| {
                    url::Url::parse(link)
                        .map(|u| u.domain().unwrap_or("").to_string())
                        .unwrap_or_default()
                })
                .collect(),
            min_text_length: extracted.text.len() / 2, // Allow 50% variation
        };

        test_cases.push(test_case);
    }

    let json = serde_json::to_string_pretty(&test_cases).unwrap();
    std::fs::write("tests/golden/test_cases.json", json).unwrap();
}
```

### 5. Performance Tests

#### Load Testing
```rust
// tests/performance_tests.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

fn bench_extraction_performance(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let test_cases = vec![
        ("small_article", include_str!("fixtures/small_article.html")),
        ("large_article", include_str!("fixtures/large_article.html")),
        ("spa_page", include_str!("fixtures/spa_page.html")),
    ];

    let mut group = c.benchmark_group("extraction");
    group.measurement_time(Duration::from_secs(10));

    for (name, html) in test_cases {
        group.bench_with_input(
            BenchmarkId::new("wasm_extraction", name),
            &html,
            |b, html| {
                b.to_async(&rt).iter(|| async {
                    extract_content_wasm(html.as_bytes(), "https://example.com").await
                });
            },
        );
    }

    group.finish();
}

fn bench_concurrent_crawling(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let urls: Vec<String> = (0..100)
        .map(|i| format!("https://httpbin.org/delay/1?id={}", i))
        .collect();

    let mut group = c.benchmark_group("concurrent_crawling");

    for concurrency in [1, 5, 10, 20] {
        group.bench_with_input(
            BenchmarkId::new("crawl_urls", concurrency),
            &concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    crawl_urls_with_concurrency(&urls, concurrency).await
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_extraction_performance, bench_concurrent_crawling);
criterion_main!(benches);
```

#### Stress Testing
```rust
// tests/stress_tests.rs
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::time::{Duration, Instant};

#[tokio::test]
async fn test_high_concurrency_crawling() {
    let success_count = Arc::new(AtomicUsize::new(0));
    let error_count = Arc::new(AtomicUsize::new(0));

    let start_time = Instant::now();
    let mut tasks = Vec::new();

    // Spawn 100 concurrent crawl tasks
    for i in 0..100 {
        let success_count = success_count.clone();
        let error_count = error_count.clone();

        let task = tokio::spawn(async move {
            let url = format!("https://httpbin.org/html?test={}", i);

            match crawl_single_url(&url).await {
                Ok(_) => {
                    success_count.fetch_add(1, Ordering::Relaxed);
                }
                Err(_) => {
                    error_count.fetch_add(1, Ordering::Relaxed);
                }
            }
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    for task in tasks {
        task.await.unwrap();
    }

    let duration = start_time.elapsed();
    let success = success_count.load(Ordering::Relaxed);
    let errors = error_count.load(Ordering::Relaxed);

    println!(
        "Completed {} requests in {:?} ({} success, {} errors)",
        success + errors, duration, success, errors
    );

    // Assertions
    assert!(duration < Duration::from_secs(30), "Test took too long");
    assert!(success >= 95, "Success rate too low: {}/100", success);
    assert!(errors <= 5, "Error rate too high: {}/100", errors);
}

#[tokio::test]
async fn test_memory_usage_stability() {
    let initial_memory = get_memory_usage();

    // Process many URLs to test for memory leaks
    for batch in 0..10 {
        let urls: Vec<String> = (0..50)
            .map(|i| format!("https://httpbin.org/html?batch={}&item={}", batch, i))
            .collect();

        let _results = crawl_urls(urls).await.unwrap();

        // Force garbage collection
        tokio::task::yield_now().await;

        let current_memory = get_memory_usage();
        let memory_growth = current_memory - initial_memory;

        println!("Batch {}: Memory usage: {} KB (+{} KB)",
                batch, current_memory / 1024, memory_growth / 1024);

        // Memory should not grow excessively
        assert!(
            memory_growth < 100 * 1024 * 1024, // 100MB max growth
            "Memory usage grew too much: {} bytes",
            memory_growth
        );
    }
}

fn get_memory_usage() -> usize {
    // Platform-specific memory usage detection
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status").unwrap();
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let kb: usize = line
                    .split_whitespace()
                    .nth(1)
                    .unwrap()
                    .parse()
                    .unwrap();
                return kb * 1024; // Convert to bytes
            }
        }
    }

    0 // Fallback for other platforms
}
```

## Testing Tools and Infrastructure

### Test Data Management
```bash
# tests/fixtures/setup.sh
#!/bin/bash

# Download test HTML files
curl -o tests/fixtures/wikipedia_rust.html \
  "https://en.wikipedia.org/wiki/Rust_(programming_language)"

curl -o tests/fixtures/github_readme.html \
  "https://raw.githubusercontent.com/rust-lang/rust/master/README.md"

# Create synthetic test cases
cat > tests/fixtures/spa_page.html << 'EOF'
<html>
<head><title>SPA Test Page</title></head>
<body>
  <div id="root">Loading...</div>
  <script>
    // Simulate SPA content loading
    document.getElementById('root').innerHTML =
      '<h1>Dynamic Content</h1><p>Loaded via JavaScript</p>';
  </script>
</body>
</html>
EOF
```

### Continuous Integration
```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Run unit tests
        run: cargo test --workspace --lib

  integration-tests:
    runs-on: ubuntu-latest
    services:
      redis:
        image: redis:7
        ports:
          - 6379:6379
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Start headless service
        run: |
          cargo build --bin riptide-headless
          ./target/debug/riptide-headless &
      - name: Run integration tests
        run: cargo test --test '*'

  golden-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Run golden tests
        run: cargo test golden_
        env:
          # Skip tests that require external services
          SKIP_EXTERNAL_TESTS: "true"

  performance-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Run performance benchmarks
        run: cargo bench --bench extraction_bench
```

### Local Testing Scripts
```bash
# scripts/test.sh
#!/bin/bash
set -e

echo "Running code formatting check..."
cargo fmt --all --check

echo "Running linter..."
cargo clippy --workspace --all-targets -- -D warnings

echo "Running unit tests..."
cargo test --workspace --lib

echo "Building WASM module..."
cd wasm/riptide-extractor-wasm
cargo build --release --target wasm32-wasip2
cd ../..

echo "Running integration tests..."
docker-compose -f docker-compose.test.yml up -d
sleep 10  # Wait for services to start

cargo test --test '*'

docker-compose -f docker-compose.test.yml down

echo "Running golden tests..."
cargo test golden_

echo "All tests passed!"
```

## Testing Best Practices

### 1. Test Organization
- Keep test files close to the code they test
- Use descriptive test names that explain the scenario
- Group related tests in modules
- Use setup and teardown functions for common code

### 2. Test Data
- Use realistic test data that represents production scenarios
- Keep test fixtures small and focused
- Version control test data with the code
- Use builders or factories for complex test objects

### 3. Mocking and Stubbing
- Mock external dependencies (HTTP services, databases)
- Use dependency injection to make testing easier
- Keep mocks simple and focused
- Test both success and failure scenarios

### 4. Performance Testing
- Establish baseline performance metrics
- Test with realistic data volumes
- Monitor memory usage and resource leaks
- Use profiling tools to identify bottlenecks

### 5. Continuous Testing
- Run tests automatically on every commit
- Use multiple test environments (different OS, Rust versions)
- Monitor test reliability and fix flaky tests
- Generate and publish test reports

This comprehensive testing strategy ensures that RipTide Crawler maintains high quality, reliability, and performance as it evolves.