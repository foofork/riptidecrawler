# Hexser Integration Testing & Validation Strategy

**Project**: RipTide EventMesh - Hexser Integration
**Version**: 1.0
**Date**: 2025-10-12
**Agent**: Tester (Hive Mind Swarm)
**Task ID**: task-1760283735470-abee91hwd

---

## Executive Summary

This document outlines the comprehensive testing and validation strategy for integrating the hexser HTML extraction library into the RipTide EventMesh project. The strategy covers all phases from compatibility validation through production deployment, with emphasis on risk mitigation, performance validation, and maintaining system reliability.

### Key Objectives

1. **Zero-Regression**: Ensure existing functionality remains intact
2. **Performance Validation**: Verify hexser meets or exceeds current extraction performance
3. **Compatibility Assurance**: Validate hexser works across all EventMesh use cases
4. **Risk Mitigation**: Identify and address integration risks early
5. **Migration Confidence**: Provide clear validation checkpoints for phased rollout

---

## 1. Testing Strategy Overview

### 1.1 Testing Pyramid

```
                    /\
                   /E2E\           <- 5% (Critical workflows)
                  /------\
                 /  API   \        <- 15% (Integration tests)
                /----------\
               / Component  \      <- 30% (Module integration)
              /--------------\
             /   Unit Tests   \    <- 50% (Isolated functionality)
            /------------------\
```

### 1.2 Test Categories

| Category | Coverage Target | Purpose |
|----------|----------------|---------|
| **Unit Tests** | 80%+ | Validate hexser functions in isolation |
| **Component Tests** | 70%+ | Test module interactions |
| **Integration Tests** | Key paths | End-to-end extraction workflows |
| **Performance Tests** | All scenarios | Benchmark against current performance |
| **Regression Tests** | Critical flows | Ensure no functionality breaks |
| **Compatibility Tests** | All edge cases | Validate across HTML variations |

---

## 2. Phase 1: Compatibility Testing

### 2.1 HTML Parsing Compatibility

**Objective**: Validate hexser correctly parses various HTML structures

#### Test Scenarios

```rust
// Test Suite: HTML Parsing Compatibility
#[cfg(test)]
mod hexser_html_compatibility {
    use super::*;

    #[tokio::test]
    async fn test_standard_html5() {
        // Valid HTML5 documents
        let html = r#"<!DOCTYPE html><html><head><title>Test</title></head><body>Content</body></html>"#;
        let result = hexser::extract(html, "https://example.com").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_malformed_html() {
        // Missing closing tags, nested incorrectly
        let html = r#"<html><body><div><p>Text</div></p></body>"#;
        let result = hexser::extract(html, "https://example.com").await;
        assert!(result.is_ok(), "Should handle malformed HTML gracefully");
    }

    #[tokio::test]
    async fn test_legacy_html4() {
        // HTML4 without DOCTYPE
        let html = r#"<html><head><meta http-equiv="Content-Type" content="text/html; charset=utf-8"></head><body>Content</body></html>"#;
        let result = hexser::extract(html, "https://example.com").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_xhtml() {
        // XHTML strict syntax
        let html = r#"<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd"><html xmlns="http://www.w3.org/1999/xhtml"><head><title>Test</title></head><body><p>Content</p></body></html>"#;
        let result = hexser::extract(html, "https://example.com").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unicode_content() {
        // Multi-language Unicode content
        let html = r#"<html><body><p>English</p><p>日本語</p><p>العربية</p><p>עברית</p></body></html>"#;
        let result = hexser::extract(html, "https://example.com").await.unwrap();
        assert!(result.text.contains("日本語"));
        assert!(result.text.contains("العربية"));
    }

    #[tokio::test]
    async fn test_special_characters() {
        // HTML entities and special characters
        let html = r#"<html><body><p>&lt;script&gt; &amp; &quot;quotes&quot;</p></body></html>"#;
        let result = hexser::extract(html, "https://example.com").await.unwrap();
        assert!(result.text.contains("<script>"));
        assert!(result.text.contains("&"));
    }
}
```

#### Validation Checkpoints

- [ ] HTML5 documents parse correctly
- [ ] Malformed HTML handled gracefully (no panics)
- [ ] Legacy HTML4/XHTML supported
- [ ] Unicode content preserved accurately
- [ ] HTML entities decoded correctly
- [ ] Nested structures handled properly
- [ ] Large documents (>10MB) process without errors

### 2.2 CSS Selector Compatibility

**Objective**: Ensure hexser's selector support matches current extraction needs

```rust
#[cfg(test)]
mod hexser_css_compatibility {
    use super::*;

    #[tokio::test]
    async fn test_basic_selectors() {
        let html = r#"<html><body><div class="content"><p id="main">Text</p></div></body></html>"#;

        // Test class selector
        assert!(hexser::select(html, ".content").is_some());

        // Test ID selector
        assert!(hexser::select(html, "#main").is_some());

        // Test element selector
        assert!(hexser::select(html, "p").is_some());
    }

    #[tokio::test]
    async fn test_advanced_selectors() {
        let html = r#"
            <html><body>
                <article>
                    <header><h1>Title</h1></header>
                    <div class="content">
                        <p>First paragraph</p>
                        <p>Second paragraph</p>
                    </div>
                </article>
            </body></html>
        "#;

        // Descendant combinator
        assert!(hexser::select(html, "article div.content").is_some());

        // Child combinator
        assert!(hexser::select(html, "article > header").is_some());

        // Adjacent sibling
        assert!(hexser::select(html, "h1 + div").is_none()); // Not adjacent

        // Attribute selector
        assert!(hexser::select(html, "[class='content']").is_some());

        // Pseudo-class
        assert!(hexser::select(html, "p:first-child").is_some());
    }

    #[tokio::test]
    async fn test_complex_selectors() {
        let html = r#"
            <html><body>
                <nav><ul><li><a href="/">Home</a></li></ul></nav>
                <main><article class="post"><p>Content</p></article></main>
            </body></html>
        "#;

        // Complex multi-part selector
        let result = hexser::select(html, "main article.post p");
        assert!(result.is_some());

        // Multiple selectors (OR)
        let result = hexser::select_all(html, "nav a, main p");
        assert_eq!(result.len(), 2);
    }
}
```

#### Validation Checkpoints

- [ ] Basic selectors (class, id, element) work
- [ ] Combinators (descendant, child, sibling) supported
- [ ] Attribute selectors functional
- [ ] Pseudo-classes supported (:first-child, :nth-child, etc.)
- [ ] Complex multi-part selectors work
- [ ] Multiple selector queries (OR logic) work
- [ ] Performance acceptable for complex selectors

---

## 3. Phase 2: Migration Validation Tests

### 3.1 Extraction Parity Testing

**Objective**: Verify hexser produces equivalent or better extraction results

```rust
#[cfg(test)]
mod extraction_parity_tests {
    use super::*;

    struct ExtractionComparison {
        html: &'static str,
        url: &'static str,
        description: &'static str,
    }

    const TEST_CASES: &[ExtractionComparison] = &[
        ExtractionComparison {
            html: include_str!("../fixtures/news_article.html"),
            url: "https://example.com/article",
            description: "News article with standard structure",
        },
        ExtractionComparison {
            html: include_str!("../fixtures/blog_post.html"),
            url: "https://blog.example.com/post",
            description: "Blog post with sidebar and navigation",
        },
        ExtractionComparison {
            html: include_str!("../fixtures/ecommerce_product.html"),
            url: "https://shop.example.com/product/123",
            description: "E-commerce product page",
        },
        ExtractionComparison {
            html: include_str!("../fixtures/spa_content.html"),
            url: "https://app.example.com/view",
            description: "Single-page application content",
        },
    ];

    #[tokio::test]
    async fn test_extraction_quality_parity() {
        for test_case in TEST_CASES {
            // Extract with current system (baseline)
            let current_result = current_extractor::extract(test_case.html, test_case.url).await.unwrap();

            // Extract with hexser (new)
            let hexser_result = hexser::extract(test_case.html, test_case.url).await.unwrap();

            // Compare content quality
            let similarity = calculate_similarity(&current_result.text, &hexser_result.text);

            assert!(
                similarity >= 0.85,
                "Extraction similarity for '{}' is {:.2}%, expected >= 85%",
                test_case.description,
                similarity * 100.0
            );

            // Verify key metadata extracted
            assert!(hexser_result.title.is_some(), "Title should be extracted for '{}'", test_case.description);
            assert!(!hexser_result.text.is_empty(), "Content should not be empty for '{}'", test_case.description);
        }
    }

    #[tokio::test]
    async fn test_link_extraction_parity() {
        let html = include_str!("../fixtures/links_heavy.html");

        let current_links = current_extractor::extract_links(html).await.unwrap();
        let hexser_links = hexser::extract_links(html).await.unwrap();

        // Should extract same number or more links
        assert!(
            hexser_links.len() >= current_links.len(),
            "Hexser should extract at least as many links: {} vs {}",
            hexser_links.len(),
            current_links.len()
        );

        // All current links should be in hexser results
        let hexser_set: HashSet<_> = hexser_links.iter().collect();
        let missing_links: Vec<_> = current_links.iter()
            .filter(|link| !hexser_set.contains(link))
            .collect();

        assert!(
            missing_links.is_empty(),
            "Hexser missing {} links: {:?}",
            missing_links.len(),
            missing_links
        );
    }

    #[tokio::test]
    async fn test_metadata_extraction_parity() {
        let html = include_str!("../fixtures/rich_metadata.html");

        let current_meta = current_extractor::extract_metadata(html).await.unwrap();
        let hexser_meta = hexser::extract_metadata(html).await.unwrap();

        // Compare key metadata fields
        assert_eq!(hexser_meta.title, current_meta.title, "Titles should match");
        assert_eq!(hexser_meta.description, current_meta.description, "Descriptions should match");
        assert_eq!(hexser_meta.author, current_meta.author, "Authors should match");

        // Hexser should extract additional metadata
        assert!(hexser_meta.language.is_some(), "Language should be detected");
        assert!(hexser_meta.published_date.is_some(), "Published date should be extracted");
    }
}
```

#### Validation Checkpoints

- [ ] Content extraction quality >= 85% similarity
- [ ] Link extraction comprehensive (no missing links)
- [ ] Metadata extraction complete (title, description, author)
- [ ] Additional metadata captured (language, dates, categories)
- [ ] Edge cases handled (empty content, no metadata, etc.)
- [ ] Error handling graceful (malformed HTML, missing elements)

### 3.2 API Contract Validation

**Objective**: Ensure hexser integration maintains existing API contracts

```rust
#[cfg(test)]
mod api_contract_tests {
    use super::*;
    use riptide_html::{HtmlProcessor, ProcessingResult};

    #[tokio::test]
    async fn test_html_processor_api_unchanged() {
        let processor = HtmlProcessor::new();
        let html = "<html><body><p>Test</p></body></html>";

        // API should remain the same
        let result: Result<ProcessingResult, _> = processor.process(html, "https://example.com").await;

        assert!(result.is_ok());
        let result = result.unwrap();

        // Response structure unchanged
        assert!(result.title.is_some());
        assert!(!result.content.is_empty());
        assert!(result.url == "https://example.com");
    }

    #[tokio::test]
    async fn test_extraction_strategies_compatible() {
        use riptide_html::extraction_strategies::*;

        let html = "<html><body><article>Content</article></body></html>";

        // TrekExtractor should still work
        let trek_result = TrekExtractor::extract(html, "https://example.com").await;
        assert!(trek_result.is_ok());

        // CssExtractorStrategy should still work
        let css_result = CssExtractorStrategy::extract(html, "https://example.com", &default_selectors()).await;
        assert!(css_result.is_ok());

        // New HexserExtractor should be available
        let hexser_result = HexserExtractor::extract(html, "https://example.com").await;
        assert!(hexser_result.is_ok());
    }

    #[tokio::test]
    async fn test_wasm_integration_unchanged() {
        use riptide_html::wasm_extraction::*;

        let config = ExtractorConfig::default();
        let extractor = WasmExtractor::new(config).unwrap();

        let html = "<html><body><p>WASM test</p></body></html>";
        let result = extractor.extract(html, "https://example.com").await;

        assert!(result.is_ok());
        assert!(!result.unwrap().text.is_empty());
    }
}
```

#### Validation Checkpoints

- [ ] `HtmlProcessor` API unchanged
- [ ] `ExtractionStrategies` trait implementations work
- [ ] WASM integration unaffected
- [ ] Error types remain compatible
- [ ] Configuration options preserved
- [ ] Return types consistent

---

## 4. Phase 3: Performance Benchmarking

### 4.1 Performance Test Strategy

**Objective**: Validate hexser meets or exceeds current performance metrics

#### Benchmark Scenarios

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tokio::runtime::Runtime;

fn benchmark_extraction_speed(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let test_cases = vec![
        ("small", include_str!("../fixtures/small.html"), 1024),          // 1KB
        ("medium", include_str!("../fixtures/medium.html"), 50_000),      // 50KB
        ("large", include_str!("../fixtures/large.html"), 500_000),       // 500KB
        ("xlarge", include_str!("../fixtures/xlarge.html"), 5_000_000),   // 5MB
    ];

    let mut group = c.benchmark_group("extraction_performance");

    for (name, html, _size) in test_cases {
        // Benchmark current extractor
        group.bench_with_input(
            BenchmarkId::new("current", name),
            &html,
            |b, html| {
                b.to_async(&rt).iter(|| async {
                    black_box(current_extractor::extract(html, "https://example.com").await)
                });
            },
        );

        // Benchmark hexser
        group.bench_with_input(
            BenchmarkId::new("hexser", name),
            &html,
            |b, html| {
                b.to_async(&rt).iter(|| async {
                    black_box(hexser::extract(html, "https://example.com").await)
                });
            },
        );
    }

    group.finish();
}

fn benchmark_selector_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let html = include_str!("../fixtures/complex_dom.html");

    let selectors = vec![
        ("simple", "p"),
        ("class", ".article-content"),
        ("complex", "article.post > div.content p:not(.meta)"),
        ("multiple", "h1, h2, h3, .title, #main-title"),
    ];

    let mut group = c.benchmark_group("selector_performance");

    for (name, selector) in selectors {
        group.bench_with_input(
            BenchmarkId::new("current", name),
            &selector,
            |b, sel| {
                b.to_async(&rt).iter(|| async {
                    black_box(current_extractor::select(html, sel).await)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("hexser", name),
            &selector,
            |b, sel| {
                b.to_async(&rt).iter(|| async {
                    black_box(hexser::select(html, sel).await)
                });
            },
        );
    }

    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("memory_efficiency");
    group.sample_size(20); // Fewer samples for memory tests

    let large_html = include_str!("../fixtures/xlarge.html");

    group.bench_function("current_memory", |b| {
        b.to_async(&rt).iter(|| async {
            let initial = get_memory_usage();
            black_box(current_extractor::extract(large_html, "https://example.com").await);
            let peak = get_peak_memory();
            peak - initial
        });
    });

    group.bench_function("hexser_memory", |b| {
        b.to_async(&rt).iter(|| async {
            let initial = get_memory_usage();
            black_box(hexser::extract(large_html, "https://example.com").await);
            let peak = get_peak_memory();
            peak - initial
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_extraction_speed,
    benchmark_selector_performance,
    benchmark_memory_usage
);
criterion_main!(benches);
```

#### Performance Targets

| Metric | Current Baseline | Hexser Target | Acceptance |
|--------|-----------------|---------------|------------|
| Small HTML (<10KB) | ~2ms | ≤2ms | ✓ If ≤110% baseline |
| Medium HTML (50KB) | ~15ms | ≤15ms | ✓ If ≤110% baseline |
| Large HTML (500KB) | ~150ms | ≤150ms | ✓ If ≤110% baseline |
| XLarge HTML (5MB) | ~1.5s | ≤1.5s | ✓ If ≤110% baseline |
| Memory (5MB HTML) | ~50MB peak | ≤50MB | ✓ If ≤120% baseline |
| Simple selector | ~0.5ms | ≤0.5ms | ✓ If ≤110% baseline |
| Complex selector | ~5ms | ≤5ms | ✓ If ≤110% baseline |

#### Validation Checkpoints

- [ ] Extraction speed within 110% of baseline
- [ ] Memory usage within 120% of baseline
- [ ] No performance regression on small documents
- [ ] Acceptable performance on large documents
- [ ] Selector queries performant
- [ ] Concurrent extraction scales linearly
- [ ] No memory leaks in long-running tests

### 4.2 Stress Testing

```rust
#[cfg(test)]
mod stress_tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_extractions() {
        let html = include_str!("../fixtures/medium.html");
        let mut handles = vec![];

        // 100 concurrent extractions
        for i in 0..100 {
            let html = html.to_string();
            handles.push(tokio::spawn(async move {
                hexser::extract(&html, &format!("https://example.com/{}", i)).await
            }));
        }

        let results = futures::future::join_all(handles).await;

        // All should succeed
        assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 100);
    }

    #[tokio::test]
    async fn test_memory_stability_long_running() {
        let initial_memory = get_memory_usage();

        // Extract 1000 documents
        for _ in 0..1000 {
            let html = generate_test_html();
            let _ = hexser::extract(&html, "https://example.com").await;
        }

        // Force cleanup
        tokio::time::sleep(Duration::from_secs(1)).await;

        let final_memory = get_memory_usage();
        let increase = final_memory - initial_memory;

        // Memory growth should be minimal (<10MB)
        assert!(
            increase < 10 * 1024 * 1024,
            "Memory grew by {} bytes, expected <10MB",
            increase
        );
    }

    #[tokio::test]
    async fn test_malformed_html_resilience() {
        let malformed_cases = vec![
            "<html><body><div><p>Unclosed tags",
            "<<>><<invalid>>",
            "<html>" + &"a".repeat(10_000_000), // 10MB of 'a'
            "<html><script>while(true){}</script></body>",
        ];

        for (i, html) in malformed_cases.iter().enumerate() {
            let result = tokio::time::timeout(
                Duration::from_secs(5),
                hexser::extract(html, "https://example.com")
            ).await;

            assert!(
                result.is_ok(),
                "Case {} timed out or panicked",
                i
            );
        }
    }
}
```

---

## 5. Phase 4: Integration Testing

### 5.1 End-to-End Workflows

```rust
#[cfg(test)]
mod e2e_integration_tests {
    use super::*;
    use riptide_api::AppState;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_extract_endpoint_with_hexser() {
        let state = AppState::new_for_testing();
        let app = riptide_api::create_router(state);

        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/extract")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&json!({
                    "url": "https://example.com",
                    "strategy": "hexser"
                })).unwrap()
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let result: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(result["content"].is_string());
        assert!(result["title"].is_string());
        assert_eq!(result["strategy_used"], "hexser");
    }

    #[tokio::test]
    async fn test_streaming_extraction_with_hexser() {
        let state = AppState::new_for_testing();
        let app = riptide_api::create_router(state);

        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/stream/extract")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&json!({
                    "url": "https://example.com/large-article",
                    "strategy": "hexser",
                    "chunking": true
                })).unwrap()
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Verify streaming response
        let body = response.into_body();
        let chunks = collect_stream_chunks(body).await;

        assert!(chunks.len() > 0);
        assert!(chunks.iter().all(|chunk| !chunk.is_empty()));
    }

    #[tokio::test]
    async fn test_wasm_pipeline_with_hexser_fallback() {
        // Test that WASM extraction can fallback to hexser
        let state = AppState::new_for_testing();

        let html = "<html><body><p>Complex content that WASM can't handle</p></body></html>";

        let result = state
            .extract_with_fallback(html, "https://example.com")
            .await
            .unwrap();

        assert!(!result.content.is_empty());
        // Should use hexser fallback
        assert!(result.strategy_used == "hexser" || result.strategy_used == "wasm+hexser");
    }
}
```

### 5.2 Cross-Module Integration

```rust
#[cfg(test)]
mod cross_module_integration {
    use super::*;

    #[tokio::test]
    async fn test_hexser_with_persistence() {
        use riptide_persistence::CacheManager;

        let cache = CacheManager::new_for_testing();
        let html = "<html><body><p>Cached content</p></body></html>";

        // First extraction
        let result1 = hexser::extract(html, "https://example.com").await.unwrap();
        cache.store("test_key", &result1).await.unwrap();

        // Retrieve from cache
        let cached: ExtractedDoc = cache.get("test_key").await.unwrap().unwrap();

        assert_eq!(result1.text, cached.text);
        assert_eq!(result1.title, cached.title);
    }

    #[tokio::test]
    async fn test_hexser_with_streaming() {
        use riptide_streaming::StreamProcessor;

        let processor = StreamProcessor::new();
        let large_html = generate_large_html(1_000_000); // 1MB

        let stream = processor
            .extract_streamed(&large_html, "https://example.com", "hexser")
            .await
            .unwrap();

        let chunks: Vec<_> = stream.collect().await;

        assert!(chunks.len() > 0);
        assert!(chunks.iter().all(|c| c.is_ok()));
    }

    #[tokio::test]
    async fn test_hexser_with_workers() {
        use riptide_workers::WorkerPool;

        let pool = WorkerPool::new(4);

        let urls = vec![
            "https://example.com/1",
            "https://example.com/2",
            "https://example.com/3",
            "https://example.com/4",
        ];

        let results = pool
            .extract_batch_with_strategy(&urls, "hexser")
            .await
            .unwrap();

        assert_eq!(results.len(), 4);
        assert!(results.iter().all(|r| r.is_ok()));
    }
}
```

#### Validation Checkpoints

- [ ] REST API endpoints work with hexser
- [ ] Streaming extraction functional
- [ ] WASM fallback to hexser works
- [ ] Cache integration works
- [ ] Worker pool integration works
- [ ] PDF pipeline unaffected
- [ ] Stealth mode compatible

---

## 6. Phase 5: Regression Testing

### 6.1 Golden Test Suite

**Objective**: Ensure no regressions in critical functionality

```rust
#[cfg(test)]
mod golden_tests {
    use super::*;
    use std::fs;

    struct GoldenTest {
        name: &'static str,
        html_fixture: &'static str,
        golden_output: &'static str,
    }

    const GOLDEN_TESTS: &[GoldenTest] = &[
        GoldenTest {
            name: "news_article_extraction",
            html_fixture: "fixtures/golden/news_article.html",
            golden_output: "fixtures/golden/news_article.json",
        },
        GoldenTest {
            name: "blog_post_extraction",
            html_fixture: "fixtures/golden/blog_post.html",
            golden_output: "fixtures/golden/blog_post.json",
        },
        GoldenTest {
            name: "product_page_extraction",
            html_fixture: "fixtures/golden/product_page.html",
            golden_output: "fixtures/golden/product_page.json",
        },
        GoldenTest {
            name: "documentation_extraction",
            html_fixture: "fixtures/golden/documentation.html",
            golden_output: "fixtures/golden/documentation.json",
        },
    ];

    #[tokio::test]
    async fn test_golden_extraction_results() {
        for test in GOLDEN_TESTS {
            let html = fs::read_to_string(test.html_fixture).unwrap();
            let expected: ExtractedDoc = serde_json::from_str(
                &fs::read_to_string(test.golden_output).unwrap()
            ).unwrap();

            let result = hexser::extract(&html, "https://example.com").await.unwrap();

            // Compare key fields
            assert_eq!(
                result.title, expected.title,
                "Golden test '{}' failed: title mismatch",
                test.name
            );

            let similarity = calculate_similarity(&result.text, &expected.text);
            assert!(
                similarity >= 0.95,
                "Golden test '{}' failed: content similarity {:.2}% < 95%",
                test.name,
                similarity * 100.0
            );

            assert_eq!(
                result.links.len(), expected.links.len(),
                "Golden test '{}' failed: link count mismatch",
                test.name
            );
        }
    }

    #[tokio::test]
    async fn test_no_regression_in_edge_cases() {
        let edge_cases = vec![
            ("empty_html", "<html></html>"),
            ("only_text", "Plain text without HTML"),
            ("nested_tables", include_str!("../fixtures/nested_tables.html")),
            ("heavy_javascript", include_str!("../fixtures/spa_heavy_js.html")),
            ("iframes", include_str!("../fixtures/multiple_iframes.html")),
        ];

        for (name, html) in edge_cases {
            let result = hexser::extract(html, "https://example.com").await;

            assert!(
                result.is_ok(),
                "Edge case '{}' should not fail",
                name
            );

            // Verify expected behavior for each edge case
            match name {
                "empty_html" => assert!(result.unwrap().text.is_empty()),
                "only_text" => assert!(result.unwrap().text == "Plain text without HTML"),
                _ => assert!(result.is_ok()),
            }
        }
    }
}
```

### 6.2 Behavioral Regression Tests

```rust
#[cfg(test)]
mod behavioral_regression_tests {
    use super::*;

    #[tokio::test]
    async fn test_error_handling_unchanged() {
        // Invalid URL should return specific error
        let result = hexser::extract("<html></html>", "not-a-url");
        assert!(matches!(result, Err(HexserError::InvalidUrl(_))));

        // Timeout should be handled gracefully
        let result = tokio::time::timeout(
            Duration::from_millis(100),
            hexser::extract_with_timeout(&large_html(), "https://example.com", Duration::from_millis(50))
        ).await;
        assert!(matches!(result, Err(_))); // Should timeout
    }

    #[tokio::test]
    async fn test_configuration_behavior() {
        let config = HexserConfig {
            max_content_length: Some(1000),
            extract_links: false,
            extract_media: false,
        };

        let result = hexser::extract_with_config(
            &large_html(),
            "https://example.com",
            &config
        ).await.unwrap();

        // Should respect config
        assert!(result.text.len() <= 1000);
        assert!(result.links.is_empty());
        assert!(result.media.is_empty());
    }

    #[tokio::test]
    async fn test_unicode_handling_unchanged() {
        let multilang_html = r#"
            <html><body>
                <p lang="en">English content</p>
                <p lang="ja">日本語コンテンツ</p>
                <p lang="ar">المحتوى العربي</p>
                <p lang="ru">Русский контент</p>
                <p lang="zh">中文内容</p>
            </body></html>
        "#;

        let result = hexser::extract(multilang_html, "https://example.com").await.unwrap();

        // All languages should be preserved
        assert!(result.text.contains("日本語コンテンツ"));
        assert!(result.text.contains("المحتوى العربي"));
        assert!(result.text.contains("Русский контент"));
        assert!(result.text.contains("中文内容"));
    }
}
```

---

## 7. Risk Mitigation & Validation Checkpoints

### 7.1 Integration Risk Matrix

| Risk | Probability | Impact | Mitigation | Checkpoint |
|------|------------|--------|------------|------------|
| **Extraction Quality Degradation** | Medium | High | Parity tests with 85%+ similarity | Phase 2.1 |
| **Performance Regression** | Medium | High | Benchmark tests within 110% baseline | Phase 3.1 |
| **API Breaking Changes** | Low | Critical | Contract tests for all public APIs | Phase 2.2 |
| **Memory Leaks** | Low | High | Long-running stress tests | Phase 3.2 |
| **Unicode Handling Issues** | Low | Medium | Comprehensive multilingual tests | Phase 1.1 |
| **Concurrency Problems** | Medium | High | Concurrent extraction tests | Phase 3.2 |
| **CSS Selector Incompatibility** | Medium | Medium | Selector compatibility suite | Phase 1.2 |
| **Integration Failures** | Low | High | Cross-module integration tests | Phase 4.2 |

### 7.2 Phase-Based Validation Gates

#### Phase 1: Compatibility (Gate 1)
- [ ] All HTML parsing tests pass (100%)
- [ ] CSS selector tests pass (95%+)
- [ ] Unicode handling verified
- [ ] No panics or crashes on malformed HTML

**Blocker**: Cannot proceed if extraction quality <85%

#### Phase 2: Migration Validation (Gate 2)
- [ ] Extraction parity tests pass (85%+ similarity)
- [ ] API contract tests pass (100%)
- [ ] Link extraction complete
- [ ] Metadata extraction verified

**Blocker**: Cannot proceed if API contracts broken

#### Phase 3: Performance (Gate 3)
- [ ] Benchmark tests within acceptable ranges
- [ ] Memory usage acceptable (<120% baseline)
- [ ] No memory leaks detected
- [ ] Concurrent extractions work

**Blocker**: Cannot proceed if performance >120% of baseline

#### Phase 4: Integration (Gate 4)
- [ ] End-to-end workflows pass
- [ ] Cross-module integration verified
- [ ] Streaming extraction works
- [ ] Worker pool integration functional

**Blocker**: Cannot proceed if critical workflows fail

#### Phase 5: Regression (Gate 5)
- [ ] All golden tests pass
- [ ] No behavioral regressions
- [ ] Edge cases handled
- [ ] Error handling unchanged

**Blocker**: Cannot proceed if golden tests fail

---

## 8. Testing Infrastructure

### 8.1 Test Organization

```
tests/
├── hexser_integration/
│   ├── compatibility/
│   │   ├── html_parsing_tests.rs
│   │   ├── css_selector_tests.rs
│   │   └── unicode_tests.rs
│   ├── migration/
│   │   ├── extraction_parity_tests.rs
│   │   ├── api_contract_tests.rs
│   │   └── comparison_utils.rs
│   ├── performance/
│   │   ├── benchmarks.rs
│   │   ├── stress_tests.rs
│   │   └── memory_tests.rs
│   ├── integration/
│   │   ├── e2e_workflows.rs
│   │   ├── cross_module_tests.rs
│   │   └── api_integration_tests.rs
│   ├── regression/
│   │   ├── golden_tests.rs
│   │   ├── behavioral_tests.rs
│   │   └── edge_case_tests.rs
│   └── fixtures/
│       ├── html/
│       ├── golden/
│       └── test_data.rs
├── benches/
│   └── hexser_benchmarks.rs
└── support/
    ├── test_helpers.rs
    ├── comparison_utils.rs
    └── memory_profiling.rs
```

### 8.2 Test Execution Commands

```bash
# Run all hexser integration tests
cargo test --package riptide-html --test hexser_integration

# Run specific test phase
cargo test --package riptide-html --test compatibility_tests
cargo test --package riptide-html --test migration_validation_tests
cargo test --package riptide-html --test performance_tests

# Run benchmarks
cargo bench --package riptide-html --bench hexser_benchmarks

# Run with coverage
cargo tarpaulin --package riptide-html --out Html --output-dir ./coverage

# Run stress tests (longer duration)
cargo test --package riptide-html stress_tests -- --test-threads=1 --nocapture

# Run golden tests
cargo test --package riptide-html golden_tests --features golden-tests
```

### 8.3 CI/CD Integration

```yaml
# .github/workflows/hexser-tests.yml
name: Hexser Integration Tests

on: [push, pull_request]

jobs:
  compatibility:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run Compatibility Tests
        run: cargo test --package riptide-html --test compatibility_tests

  migration:
    runs-on: ubuntu-latest
    needs: compatibility
    steps:
      - uses: actions/checkout@v3
      - name: Run Migration Validation
        run: cargo test --package riptide-html --test migration_validation_tests

  performance:
    runs-on: ubuntu-latest
    needs: migration
    steps:
      - uses: actions/checkout@v3
      - name: Run Performance Benchmarks
        run: cargo bench --package riptide-html --bench hexser_benchmarks -- --save-baseline main
      - name: Compare Performance
        run: cargo bench --package riptide-html --bench hexser_benchmarks -- --baseline main

  integration:
    runs-on: ubuntu-latest
    needs: [compatibility, migration]
    steps:
      - uses: actions/checkout@v3
      - name: Run Integration Tests
        run: cargo test --package riptide-html --test integration_tests

  regression:
    runs-on: ubuntu-latest
    needs: [migration, integration]
    steps:
      - uses: actions/checkout@v3
      - name: Run Regression Tests
        run: cargo test --package riptide-html --test regression_tests --features golden-tests
```

---

## 9. Test Metrics & Reporting

### 9.1 Success Criteria

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Test Coverage** | ≥80% | `cargo tarpaulin` |
| **Pass Rate** | 100% | CI pipeline |
| **Performance** | ≤110% baseline | Criterion benchmarks |
| **Memory** | ≤120% baseline | Memory profiling |
| **Extraction Quality** | ≥85% similarity | Parity tests |
| **API Compatibility** | 100% | Contract tests |

### 9.2 Reporting Dashboard

Test results should be tracked in:
- GitHub Actions CI logs
- Coverage reports (HTML)
- Benchmark comparison charts
- Integration test summaries

---

## 10. Rollout Plan with Testing Gates

### Phase A: Development (Week 1-2)
- **Testing Focus**: Unit tests, compatibility tests
- **Gate**: All compatibility tests pass
- **Risk**: Low (isolated testing)

### Phase B: Integration (Week 3)
- **Testing Focus**: Migration validation, API contracts
- **Gate**: Extraction parity ≥85%, API tests pass
- **Risk**: Medium (cross-module impacts)

### Phase C: Performance Validation (Week 4)
- **Testing Focus**: Benchmarks, stress tests
- **Gate**: Performance within acceptable ranges
- **Risk**: Medium (performance issues)

### Phase D: Staging Deployment (Week 5)
- **Testing Focus**: E2E tests, integration tests
- **Gate**: All integration tests pass
- **Risk**: High (production-like environment)

### Phase E: Production Rollout (Week 6-8)
- **Testing Focus**: Regression tests, monitoring
- **Gate**: Golden tests pass, no critical issues
- **Risk**: High (production impact)
- **Strategy**: Gradual rollout with feature flags

---

## 11. Contingency Plans

### Rollback Triggers

1. **Extraction quality drops below 80%**: Immediate rollback
2. **Performance degrades >150% baseline**: Rollback within 24h
3. **Critical API breakage**: Immediate rollback
4. **Memory leaks detected**: Rollback within 24h
5. **Production errors >1%**: Rollback within 1h

### Feature Flag Strategy

```rust
// Feature flag for hexser integration
#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    pub use_hexser: bool,
    pub hexser_rollout_percentage: f32, // 0.0 to 1.0
    pub fallback_to_current: bool,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            use_hexser: false, // Start disabled
            hexser_rollout_percentage: 0.0,
            fallback_to_current: true, // Always have fallback
        }
    }
}
```

### Gradual Rollout Schedule

| Week | Rollout % | Monitoring Focus | Rollback Threshold |
|------|-----------|------------------|-------------------|
| 1 | 5% | Error rate, latency | >0.5% errors |
| 2 | 10% | Performance, quality | >0.3% errors |
| 3 | 25% | Memory usage, throughput | >0.2% errors |
| 4 | 50% | Stability, edge cases | >0.1% errors |
| 5 | 75% | Full integration | >0.1% errors |
| 6 | 100% | Production monitoring | >0.05% errors |

---

## 12. Summary & Next Steps

### Test Strategy Summary

This comprehensive testing strategy provides:

1. **Multi-layered validation**: From unit tests to E2E workflows
2. **Risk mitigation**: Clearly defined checkpoints and rollback triggers
3. **Performance assurance**: Benchmarking against current system
4. **Quality gates**: Phase-based validation before progression
5. **Gradual rollout**: Feature flags for controlled deployment

### Immediate Next Steps

1. **Set up test infrastructure** (Week 1)
   - Create test directories
   - Set up fixtures and golden test files
   - Configure CI/CD pipeline

2. **Implement Phase 1 tests** (Week 1-2)
   - HTML parsing compatibility
   - CSS selector compatibility
   - Unicode handling

3. **Implement Phase 2 tests** (Week 2-3)
   - Extraction parity tests
   - API contract validation
   - Create comparison utilities

4. **Implement Phase 3 tests** (Week 3-4)
   - Performance benchmarks
   - Stress tests
   - Memory profiling

5. **Execute full test suite** (Week 4-5)
   - Run all tests
   - Analyze results
   - Create test report

### Success Metrics

- ✅ 100% of tests pass before production rollout
- ✅ Performance within 110% of baseline
- ✅ Extraction quality ≥85% similarity
- ✅ Zero critical bugs in production
- ✅ Successful gradual rollout to 100%

---

**Document Status**: DRAFT v1.0
**Last Updated**: 2025-10-12
**Next Review**: After Phase 1 completion
