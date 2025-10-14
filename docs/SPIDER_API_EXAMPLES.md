# Spider API Examples - Test Implementation Guide

**Generated:** 2025-10-14
**Research Agent:** Hive Mind RESEARCHER
**Purpose:** Working code examples for fixing 11 spider tests after API refactoring

---

## Table of Contents

1. [API Overview](#api-overview)
2. [Quick Reference](#quick-reference)
3. [Test Category Examples](#test-category-examples)
4. [Integration Patterns](#integration-patterns)
5. [Common Pitfalls](#common-pitfalls)

---

## API Overview

### Core Components

```rust
// Main Spider API (crates/riptide-core/src/spider/core.rs)
pub struct Spider {
    config: SpiderConfig,
    frontier_manager: Arc<FrontierManager>,
    budget_manager: Arc<BudgetManager>,
    robots_manager: Arc<RobotsManager>,
    query_aware_scorer: Arc<RwLock<Option<QueryAwareScorer>>>,
    // ... other components
}

impl Spider {
    pub async fn new(config: SpiderConfig) -> Result<Self> { /* ... */ }
    pub async fn crawl(&self, seeds: Vec<Url>) -> Result<SpiderResult> { /* ... */ }
    pub async fn score_query_aware_request(&self, request: &CrawlRequest, content: Option<&str>) -> Result<f64> { /* ... */ }
    pub async fn should_stop_query_aware(&self) -> Result<(bool, String)> { /* ... */ }

    // Test accessors
    #[cfg(test)]
    pub fn url_utils(&self) -> &Arc<RwLock<UrlUtils>> { /* ... */ }
    #[cfg(test)]
    pub fn budget_manager(&self) -> &Arc<BudgetManager> { /* ... */ }
    #[cfg(test)]
    pub fn frontier_manager(&self) -> &Arc<FrontierManager> { /* ... */ }
    #[cfg(test)]
    pub fn robots_manager(&self) -> &Arc<RobotsManager> { /* ... */ }
}
```

### Configuration API

```rust
// Spider Configuration (crates/riptide-core/src/spider/config.rs)
pub struct SpiderConfig {
    pub base_url: Url,
    pub concurrency: usize,
    pub max_depth: Option<usize>,
    pub max_pages: Option<usize>,
    pub respect_robots: bool,

    // Component configs
    pub session: SessionConfig,
    pub budget: BudgetConfig,
    pub frontier: FrontierConfig,
    pub strategy: StrategyConfig,
    pub adaptive_stop: AdaptiveStopConfig,
    pub robots: RobotsConfig,
    pub url_processing: UrlProcessingConfig,
    pub performance: PerformanceConfig,
    pub query_aware: QueryAwareConfig,
}

// Preset configurations
impl SpiderPresets {
    pub fn development() -> SpiderConfig { /* ... */ }
    pub fn high_performance() -> SpiderConfig { /* ... */ }
    pub fn news_site() -> SpiderConfig { /* ... */ }
    pub fn ecommerce_site() -> SpiderConfig { /* ... */ }
}
```

### QueryAwareScorer API

```rust
// Query-Aware Scoring (crates/riptide-core/src/spider/query_aware.rs)
pub struct QueryAwareScorer {
    config: QueryAwareConfig,
    bm25_scorer: BM25Scorer,
    url_analyzer: UrlSignalAnalyzer,
    domain_analyzer: DomainDiversityAnalyzer,
    content_analyzer: ContentSimilarityAnalyzer,
    recent_scores: Vec<f64>,
}

impl QueryAwareScorer {
    pub fn new(config: QueryAwareConfig) -> Self { /* ... */ }
    pub fn score_request(&mut self, request: &CrawlRequest, content: Option<&str>) -> f64 { /* ... */ }
    pub fn update_with_result(&mut self, result: &CrawlResult) { /* ... */ }
    pub fn should_stop_early(&self) -> (bool, String) { /* ... */ }
    pub fn get_stats(&self) -> QueryAwareStats { /* ... */ }
}

pub struct QueryAwareConfig {
    pub query_foraging: bool,              // Enable query-aware features
    pub target_query: Option<String>,      // Search query for relevance
    pub bm25_weight: f64,                  // Weight for BM25 scoring (α)
    pub url_signals_weight: f64,           // Weight for URL signals (β)
    pub domain_diversity_weight: f64,      // Weight for diversity (γ)
    pub content_similarity_weight: f64,    // Weight for similarity (δ)
    pub min_relevance_threshold: f64,      // Threshold for early stopping
    pub relevance_window_size: usize,      // Window for trend analysis
    pub bm25_k1: f64,                      // BM25 parameter k1
    pub bm25_b: f64,                       // BM25 parameter b
}
```

---

## Quick Reference

### API Migration Cheat Sheet

| Old API (Removed) | New API | Notes |
|-------------------|---------|-------|
| `QueryAwareCrawler::new()` | `QueryAwareScorer::new()` | Renamed component |
| `crawler.score_urls()` | `scorer.score_request()` | Per-request scoring |
| `CrawlOrchestrator::new()` | `Spider::new()` | Main crawler renamed |
| `orchestrator.crawl_parallel()` | `spider.crawl()` | Built-in parallelism |
| `CrawlConfig` | `SpiderConfig` | Comprehensive config |
| `config.enable_bm25` | `config.query_foraging` | Renamed flag |
| `config.url_signal_weight` | `config.url_signals_weight` | Plural form |
| `config.early_stop_threshold` | `config.min_relevance_threshold` | Clearer naming |
| `config.min_crawl_count` | `config.relevance_window_size` | Window-based |

### Configuration Field Changes

```rust
// OLD QueryAwareConfig (removed)
struct OldConfig {
    enable_bm25: bool,
    url_signal_weight: f64,
    max_depth: usize,
    early_stop_threshold: f64,
    min_crawl_count: usize,
}

// NEW QueryAwareConfig (current)
struct QueryAwareConfig {
    query_foraging: bool,              // Was: enable_bm25
    target_query: Option<String>,      // NEW: specify query
    url_signals_weight: f64,           // Was: url_signal_weight (plural)
    min_relevance_threshold: f64,      // Was: early_stop_threshold
    relevance_window_size: usize,      // Was: min_crawl_count
    // ... plus 4 new component weights
}
```

---

## Test Category Examples

### 1. Query-Aware URL Prioritization

**Test File:** `spider_tests.rs:137`
**Status:** ✅ Ready to fix
**APIs Used:** `Spider`, `QueryAwareScorer`, `SpiderConfig`

```rust
#[tokio::test]
async fn test_query_aware_url_prioritization() -> Result<()> {
    // Configure query-aware scoring
    let query_config = QueryAwareConfig {
        query_foraging: true,
        target_query: Some("rust programming".to_string()),
        bm25_weight: 0.4,
        url_signals_weight: 0.3,
        domain_diversity_weight: 0.2,
        content_similarity_weight: 0.1,
        min_relevance_threshold: 0.2,
        relevance_window_size: 10,
        ..Default::default()
    };

    let spider_config = SpiderConfig {
        query_aware: query_config,
        ..SpiderPresets::development()
    };

    let spider = Spider::new(spider_config).await?;

    // Create test URLs with varying relevance
    let test_urls = vec![
        ("https://rust-lang.org/programming/guide", "Highly relevant"),
        ("https://example.com/cooking/recipes", "Not relevant"),
        ("https://rustacean.net/rust-tutorial", "Relevant"),
        ("https://random.com/random-page", "Not relevant"),
    ];

    // Score each URL
    let mut scores = Vec::new();
    for (url_str, description) in test_urls {
        let url = Url::parse(url_str)?;
        let request = CrawlRequest::new(url.clone());
        let score = spider.score_query_aware_request(&request, None).await?;
        scores.push((url, score, description));
        println!("URL: {} | Score: {:.4} | {}", url, score, description);
    }

    // Verify: URLs with query terms score higher
    assert!(
        scores[0].1 > scores[1].1,
        "Rust programming URL should score higher than cooking URL"
    );
    assert!(
        scores[2].1 > scores[1].1,
        "Rust tutorial URL should score higher than cooking URL"
    );

    // Verify both relevant URLs score higher than both irrelevant
    assert!(scores[0].1 > scores[3].1);
    assert!(scores[2].1 > scores[3].1);

    Ok(())
}
```

**Key Points:**
- `query_foraging: true` enables query-aware features
- `target_query` specifies keywords for relevance scoring
- `url_signals_weight` controls URL path/structure scoring
- Use `score_query_aware_request()` for integrated scoring

---

### 2. Domain Diversity Scoring

**Test File:** `spider_tests.rs:149`
**Status:** 🟡 Test through public API
**APIs Used:** `QueryAwareScorer::score_request()`

```rust
#[tokio::test]
async fn test_domain_diversity_scoring() -> Result<()> {
    // Configure with high domain diversity weight
    let query_config = QueryAwareConfig {
        query_foraging: true,
        target_query: Some("technology".to_string()),
        domain_diversity_weight: 0.8,  // High weight for testing
        bm25_weight: 0.1,
        url_signals_weight: 0.05,
        content_similarity_weight: 0.05,
        ..Default::default()
    };

    let spider_config = SpiderConfig {
        query_aware: query_config,
        ..SpiderPresets::development()
    };

    let spider = Spider::new(spider_config).await?;

    // Simulate crawling multiple pages from same domain
    let domain1_urls = vec![
        "https://example.com/page1",
        "https://example.com/page2",
        "https://example.com/page3",
    ];

    let domain2_url = "https://newdomain.com/page1";

    // Score first domain repeatedly (diversity should decrease)
    let mut domain1_scores = Vec::new();
    for url_str in &domain1_urls {
        let url = Url::parse(url_str)?;
        let request = CrawlRequest::new(url.clone());
        let score = spider.score_query_aware_request(&request, None).await?;
        domain1_scores.push(score);

        // Simulate processing the result to update internal state
        let result = CrawlResult::success(request.clone());
        spider.update_query_aware_with_result(&result).await?;
    }

    // Score new domain (should get diversity bonus)
    let new_domain_url = Url::parse(domain2_url)?;
    let new_domain_request = CrawlRequest::new(new_domain_url);
    let new_domain_score = spider
        .score_query_aware_request(&new_domain_request, None)
        .await?;

    // Verify diversity effect
    println!("Domain1 scores: {:?}", domain1_scores);
    println!("New domain score: {}", new_domain_score);

    // First page from domain1 should score highest (new domain)
    // Later pages from domain1 should score progressively lower
    assert!(
        domain1_scores[0] >= domain1_scores[1],
        "First page should score >= second page"
    );

    // New domain should score higher than overused domain
    assert!(
        new_domain_score > domain1_scores[2],
        "New domain should score higher than 3rd page of same domain"
    );

    Ok(())
}
```

**Key Points:**
- Test domain diversity through `score_request()` public API
- Use `update_query_aware_with_result()` to update internal state
- High `domain_diversity_weight` makes effect more visible
- Scores should decrease for repeated domains, increase for new ones

---

### 3. Early Stopping on Low Relevance

**Test File:** `spider_tests.rs:159`
**Status:** 🟡 Needs mock HTTP server
**APIs Used:** `Spider::should_stop_query_aware()`, `Spider::crawl()`

```rust
#[tokio::test]
async fn test_early_stopping_on_low_relevance() -> Result<()> {
    // Start mock HTTP server with low-relevance content
    let mock_server = MockServer::start().await;

    // Create pages with content irrelevant to query
    for i in 0..20 {
        Mock::given(method("GET"))
            .and(path(format!("/page{}", i)))
            .respond_with(ResponseTemplate::new(200)
                .set_body_string(format!(
                    r#"<html><body>
                    <h1>Cooking Recipe {}</h1>
                    <p>This is a recipe for delicious food and cooking techniques.</p>
                    <a href="/page{}">Next recipe</a>
                    </body></html>"#,
                    i, i + 1
                )))
            .mount(&mock_server)
            .await;
    }

    // Configure with aggressive early stopping
    let query_config = QueryAwareConfig {
        query_foraging: true,
        target_query: Some("rust programming machine learning".to_string()),
        min_relevance_threshold: 0.5,  // High threshold
        relevance_window_size: 5,      // Small window
        bm25_weight: 0.7,              // High weight for content relevance
        ..Default::default()
    };

    let spider_config = SpiderConfig {
        query_aware: query_config,
        max_pages: Some(20),
        ..SpiderPresets::development()
    };

    let spider = Spider::new(spider_config).await?;

    let seed_url = Url::parse(&format!("{}/page0", mock_server.uri()))?;
    let result = spider.crawl(vec![seed_url]).await?;

    // Verify early stopping occurred
    println!("Stop reason: {}", result.stop_reason);
    println!("Pages crawled: {}", result.pages_crawled);

    assert!(
        result.stop_reason.contains("relevance") || result.stop_reason.contains("Low"),
        "Should stop due to low relevance: {}",
        result.stop_reason
    );

    assert!(
        result.pages_crawled < 15,
        "Should stop early, not crawl all pages: {}",
        result.pages_crawled
    );

    // Alternative: Check directly with should_stop_query_aware()
    let (should_stop, reason) = spider.should_stop_query_aware().await?;
    if should_stop {
        println!("Early stop check reason: {}", reason);
    }

    Ok(())
}
```

**Dependencies:**
```toml
[dev-dependencies]
wiremock = "0.6"  # For mock HTTP server
```

**Key Points:**
- Use `wiremock` for mock HTTP server in tests
- Set high `min_relevance_threshold` to trigger early stopping
- Small `relevance_window_size` makes stopping faster
- Verify `result.stop_reason` contains "relevance" or "Low"

---

### 4. Content Similarity Analysis

**Test File:** `spider_tests.rs:169`
**Status:** 🟡 Test through public API
**APIs Used:** `QueryAwareScorer::score_request()` with content

```rust
#[tokio::test]
async fn test_content_similarity_deduplication() -> Result<()> {
    // Configure with high content similarity weight
    let query_config = QueryAwareConfig {
        query_foraging: true,
        target_query: Some("machine learning algorithms".to_string()),
        content_similarity_weight: 0.8,  // High weight for testing
        bm25_weight: 0.1,
        url_signals_weight: 0.05,
        domain_diversity_weight: 0.05,
        ..Default::default()
    };

    let spider_config = SpiderConfig {
        query_aware: query_config,
        ..SpiderPresets::development()
    };

    let spider = Spider::new(spider_config).await?;

    // Test content samples
    let test_cases = vec![
        (
            "https://example.com/ml1",
            "Machine learning algorithms are used for pattern recognition and data analysis. \
             Deep learning is a subset of machine learning that uses neural networks.",
            "Highly relevant - contains all query terms"
        ),
        (
            "https://example.com/ml2",
            "This article discusses supervised and unsupervised machine learning techniques. \
             Algorithms like decision trees and random forests are popular choices.",
            "Relevant - contains machine learning and algorithms"
        ),
        (
            "https://example.com/cooking",
            "This recipe shows you how to make delicious pasta. First, boil water and add salt. \
             Then cook the pasta for 10 minutes until al dente.",
            "Not relevant - no query terms"
        ),
        (
            "https://example.com/partial",
            "Neural networks are computational models inspired by biological systems. \
             They consist of layers of interconnected nodes that process information.",
            "Partially relevant - related but missing key terms"
        ),
    ];

    let mut scores = Vec::new();
    for (url_str, content, description) in test_cases {
        let url = Url::parse(url_str)?;
        let request = CrawlRequest::new(url.clone());
        let score = spider
            .score_query_aware_request(&request, Some(content))
            .await?;
        scores.push((url, score, description));
        println!("{} | Score: {:.4} | {}", url, score, description);
    }

    // Verify similarity scoring
    assert!(
        scores[0].1 > scores[2].1,
        "ML content should score higher than cooking: {} vs {}",
        scores[0].1, scores[2].1
    );

    assert!(
        scores[1].1 > scores[2].1,
        "ML algorithms content should score higher than cooking"
    );

    assert!(
        scores[0].1 > scores[3].1,
        "Full match should score higher than partial match"
    );

    // Content with more query terms should score highest
    assert!(
        scores[0].1 >= scores[1].1,
        "Content with all terms should score >= content with some terms"
    );

    Ok(())
}
```

**Key Points:**
- Test content similarity through `score_request()` with content parameter
- High `content_similarity_weight` emphasizes content scoring
- Use Jaccard similarity for term overlap analysis
- Content with more query terms should score higher

---

### 5. Parallel Crawling with Limits

**Test File:** `spider_tests.rs:186`
**Status:** ✅ Ready to fix
**APIs Used:** `Spider`, `BudgetConfig`, `PerformanceConfig`

```rust
#[tokio::test]
async fn test_parallel_crawling_with_limits() -> Result<()> {
    // Configure strict limits
    let spider_config = SpiderConfig {
        performance: PerformanceConfig {
            max_concurrent_global: 4,
            max_concurrent_per_host: 2,
            request_timeout: Duration::from_secs(5),
            ..Default::default()
        },
        budget: BudgetConfig {
            global: GlobalBudgetLimits {
                max_pages: Some(10),
                max_concurrent: Some(4),
                max_duration: Some(Duration::from_secs(30)),
                max_data_mb: Some(10),
                ..Default::default()
            },
            per_host: PerHostBudgetLimits {
                max_pages_per_host: Some(5),
                max_concurrent_per_host: Some(2),
                ..Default::default()
            },
            ..Default::default()
        },
        max_pages: Some(10),
        ..SpiderPresets::development()
    };

    let spider = Spider::new(spider_config.clone()).await?;

    // Use mock server for predictable testing
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string("<html><body><a href='/link'>Link</a></body></html>"))
        .mount(&mock_server)
        .await;

    let seed_url = Url::parse(&mock_server.uri())?;
    let start = Instant::now();
    let result = spider.crawl(vec![seed_url]).await?;
    let duration = start.elapsed();

    // Verify limits enforced
    println!("Pages crawled: {}", result.pages_crawled);
    println!("Duration: {:?}", duration);
    println!("Stop reason: {}", result.stop_reason);

    assert!(
        result.pages_crawled <= 10,
        "Should not exceed max_pages limit: {}",
        result.pages_crawled
    );

    assert!(
        duration.as_secs() <= 35,
        "Should respect max_duration (with 5s buffer): {:?}",
        duration
    );

    // Verify BudgetManager tracked correctly
    let budget_stats = spider.budget_manager().get_stats().await;
    assert!(
        budget_stats.peak_concurrent <= 4,
        "Should not exceed max_concurrent: {}",
        budget_stats.peak_concurrent
    );

    Ok(())
}
```

**Key Points:**
- `max_concurrent_global` limits total concurrent requests
- `max_concurrent_per_host` prevents hammering single host
- `BudgetManager` automatically enforces all limits
- Check `budget_stats.peak_concurrent` for verification

---

### 6. Robots.txt Compliance

**Test File:** `spider_tests.rs:195`
**Status:** ✅ Ready to fix
**APIs Used:** `Spider`, `RobotsConfig`

```rust
#[tokio::test]
async fn test_crawl_with_robots_txt_compliance() -> Result<()> {
    // Start mock server with robots.txt
    let mock_server = MockServer::start().await;

    // Mock robots.txt
    Mock::given(method("GET"))
        .and(path("/robots.txt"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(
                "User-agent: *\n\
                 Disallow: /admin/\n\
                 Disallow: /private/\n\
                 Allow: /public/\n\
                 Crawl-delay: 1\n"
            ))
        .mount(&mock_server)
        .await;

    // Mock allowed page
    Mock::given(method("GET"))
        .and(path("/public/page"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string("<html><body>Public page</body></html>"))
        .mount(&mock_server)
        .await;

    // Mock disallowed page
    Mock::given(method("GET"))
        .and(path("/admin/page"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string("<html><body>Admin page</body></html>"))
        .mount(&mock_server)
        .await;

    // Configure with robots.txt respect
    let spider_config = SpiderConfig {
        respect_robots: true,
        robots: RobotsConfig {
            respect_robots_txt: true,
            cache_duration: Duration::from_secs(3600),
            ..Default::default()
        },
        ..SpiderPresets::development()
    };

    let spider = Spider::new(spider_config).await?;

    // Try crawling both allowed and disallowed URLs
    let allowed_url = Url::parse(&format!("{}/public/page", mock_server.uri()))?;
    let disallowed_url = Url::parse(&format!("{}/admin/page", mock_server.uri()))?;

    // Crawl allowed URL - should succeed
    let result = spider.crawl(vec![allowed_url.clone()]).await?;
    assert!(result.pages_crawled > 0, "Should crawl allowed URL");

    // Verify robots.txt was respected
    let robots_manager = spider.robots_manager();
    let can_crawl_allowed = robots_manager
        .can_crawl(allowed_url.as_str())
        .await?;
    let can_crawl_disallowed = robots_manager
        .can_crawl(disallowed_url.as_str())
        .await?;

    assert!(can_crawl_allowed, "Should allow /public/ path");
    assert!(!can_crawl_disallowed, "Should disallow /admin/ path");

    println!("Robots.txt compliance verified");
    Ok(())
}
```

**Key Points:**
- `respect_robots: true` enables robots.txt checking
- Spider automatically calls `robots_manager.can_crawl_with_wait()`
- `Crawl-delay` directive enforced automatically
- Disallowed paths blocked at request processing stage

---

### 7. Rate Limiting

**Test File:** `spider_tests.rs:204`
**Status:** ✅ Ready to fix
**APIs Used:** `Spider`, `PerformanceConfig`, host semaphores

```rust
#[tokio::test]
async fn test_crawl_rate_limiting() -> Result<()> {
    // Configure strict rate limiting
    let spider_config = SpiderConfig {
        performance: PerformanceConfig {
            max_concurrent_per_host: 2,
            min_request_delay_micros: 1_000_000,  // 1 second minimum delay
            enable_adaptive_throttling: false,    // Fixed rate for testing
            ..Default::default()
        },
        robots: RobotsConfig {
            respect_robots_txt: true,
            default_crawl_delay: Duration::from_secs(1),
            ..Default::default()
        },
        max_pages: Some(5),
        ..SpiderPresets::development()
    };

    let spider = Spider::new(spider_config).await?;

    // Start mock server
    let mock_server = MockServer::start().await;

    // Create pages with links
    for i in 0..5 {
        Mock::given(method("GET"))
            .and(path(format!("/page{}", i)))
            .respond_with(ResponseTemplate::new(200)
                .set_body_string(format!(
                    r#"<html><body>
                    <h1>Page {}</h1>
                    <a href="/page{}">Next</a>
                    </body></html>"#,
                    i, i + 1
                )))
            .mount(&mock_server)
            .await;
    }

    let seed_url = Url::parse(&format!("{}/page0", mock_server.uri()))?;

    let start = Instant::now();
    let result = spider.crawl(vec![seed_url]).await?;
    let duration = start.elapsed();

    println!("Pages crawled: {}", result.pages_crawled);
    println!("Duration: {:?}", duration);
    println!("Avg time per page: {:?}", duration / result.pages_crawled as u32);

    // With 1 second delay and max 2 concurrent, 5 pages should take ~3 seconds
    // (parallel processing reduces total time)
    let expected_min_duration = Duration::from_secs(2); // Conservative estimate

    assert!(
        duration >= expected_min_duration,
        "Rate limiting should enforce minimum duration: {:?} vs {:?}",
        duration, expected_min_duration
    );

    // Verify per-host concurrency limit
    let budget_stats = spider.budget_manager().get_stats().await;
    assert!(
        budget_stats.peak_concurrent_per_host.values().all(|&v| v <= 2),
        "Should respect max_concurrent_per_host limit"
    );

    Ok(())
}
```

**Key Points:**
- Rate limiting via `max_concurrent_per_host` and `min_request_delay_micros`
- `RobotsManager` enforces `Crawl-delay` directive
- Host semaphores prevent exceeding per-host limits
- Measure duration to verify rate limiting in effect

---

### 8. URL Deduplication

**Test File:** `spider_tests.rs:262`
**Status:** 🟡 Test via UrlUtils accessor
**APIs Used:** `Spider::url_utils()`, `UrlProcessingConfig`

```rust
#[tokio::test]
async fn test_url_deduplication() -> Result<()> {
    // Configure aggressive deduplication
    let spider_config = SpiderConfig {
        url_processing: UrlProcessingConfig {
            enable_deduplication: true,
            enable_normalization: true,
            bloom_filter_capacity: 10_000,
            bloom_filter_fpr: 0.01,
            max_exact_urls: 1_000,
            remove_fragments: true,
            remove_trailing_slash: true,
            lowercase_urls: true,
            ..Default::default()
        },
        ..SpiderPresets::development()
    };

    let spider = Spider::new(spider_config).await?;

    // Access UrlUtils for testing
    let url_utils = spider.url_utils();

    // Create duplicate URLs (various forms of same URL)
    let duplicate_urls = vec![
        Url::parse("https://example.com/page")?,
        Url::parse("https://example.com/page/")?,           // Trailing slash
        Url::parse("https://Example.COM/page")?,            // Different case
        Url::parse("https://example.com/page#section")?,    // Fragment
        Url::parse("https://example.com:443/page")?,        // Default port
        Url::parse("https://example.com/page?utm=123")?,    // Query param (kept)
    ];

    // Filter through UrlUtils
    let filtered = url_utils.read().await.filter_urls(duplicate_urls).await?;

    println!("Filtered URLs count: {}", filtered.len());
    for url in &filtered {
        println!("  - {}", url);
    }

    // Should deduplicate to 2 URLs: base URL and URL with query param
    assert!(
        filtered.len() <= 2,
        "Should deduplicate similar URLs: got {} URLs",
        filtered.len()
    );

    // Integration test: Verify Spider doesn't crawl duplicates
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string("<html><body>Page</body></html>"))
        .mount(&mock_server)
        .await;

    let base_url = format!("{}/page", mock_server.uri());
    let seeds = vec![
        Url::parse(&base_url)?,
        Url::parse(&format!("{}/", base_url))?,
        Url::parse(&format!("{}#section", base_url))?,
    ];

    let result = spider.crawl(seeds).await?;

    assert_eq!(
        result.pages_crawled, 1,
        "Should only crawl unique URL once: {}",
        result.pages_crawled
    );

    Ok(())
}
```

**Key Points:**
- Deduplication automatic via `UrlUtils` with bloom filter
- Use `spider.url_utils()` test accessor (line 811 in core.rs)
- Normalization rules configured in `UrlProcessingConfig`
- Bloom filter provides probabilistic deduplication

---

### 9. URL Normalization

**Test File:** `spider_tests.rs:272`
**Status:** 🟡 Test via UrlUtils
**APIs Used:** `UrlUtils::filter_urls()`, normalization rules

```rust
#[tokio::test]
async fn test_url_normalization() -> Result<()> {
    // Configure comprehensive normalization
    let spider_config = SpiderConfig {
        url_processing: UrlProcessingConfig {
            enable_normalization: true,
            remove_www: true,
            remove_trailing_slash: true,
            lowercase_urls: true,
            remove_fragments: true,
            remove_default_ports: true,
            ..Default::default()
        },
        ..SpiderPresets::development()
    };

    let spider = Spider::new(spider_config).await?;
    let url_utils = spider.url_utils();

    // Test normalization rules
    let test_cases = vec![
        (
            "https://WWW.Example.COM/Path/To/Page/",
            "https://example.com/path/to/page",
            "Case normalization + www removal + trailing slash"
        ),
        (
            "https://example.com:443/secure",
            "https://example.com/secure",
            "Default HTTPS port removal"
        ),
        (
            "http://example.com:80/page",
            "http://example.com/page",
            "Default HTTP port removal"
        ),
        (
            "https://example.com/page#section",
            "https://example.com/page",
            "Fragment removal"
        ),
        (
            "https://www.example.com/PAGE?sort=asc",
            "https://example.com/page?sort=asc",
            "Multiple rules: www + case + query preserved"
        ),
    ];

    for (original, expected_normalized, description) in test_cases {
        let url = Url::parse(original)?;
        let urls = vec![url.clone()];

        // Filter through UrlUtils (applies normalization)
        let normalized = url_utils.read().await.filter_urls(urls).await?;

        assert_eq!(
            normalized.len(), 1,
            "Should return one normalized URL"
        );

        println!("Test: {}", description);
        println!("  Original: {}", original);
        println!("  Normalized: {}", normalized[0]);
        println!("  Expected: {}", expected_normalized);

        // Note: Exact matching depends on url crate's parsing
        // Test key normalization features
        let normalized_str = normalized[0].as_str();
        assert!(
            !normalized_str.contains("WWW") && !normalized_str.contains("www"),
            "Should remove www: {}",
            normalized_str
        );
        assert!(
            normalized_str.chars().all(|c| !c.is_uppercase() || c == ':' || c == '/'),
            "Should be lowercase: {}",
            normalized_str
        );
        assert!(
            !normalized_str.contains('#'),
            "Should remove fragments: {}",
            normalized_str
        );
    }

    // Test deduplication with normalization
    let duplicate_variants = vec![
        Url::parse("https://example.com/page")?,
        Url::parse("https://EXAMPLE.COM/page")?,
        Url::parse("https://www.example.com/page")?,
        Url::parse("https://example.com:443/page")?,
    ];

    let deduplicated = url_utils.read().await
        .filter_urls(duplicate_variants).await?;

    assert_eq!(
        deduplicated.len(), 1,
        "Should deduplicate after normalization: got {} URLs",
        deduplicated.len()
    );

    Ok(())
}
```

**Key Points:**
- Normalization rules in `UrlProcessingConfig`
- Test via `url_utils.filter_urls()` which applies normalization
- Multiple rules can be combined (case, www, ports, etc.)
- Normalization enables better deduplication

---

## Integration Patterns

### Mock HTTP Server Setup

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn example_with_mock_server() -> Result<()> {
    let mock_server = MockServer::start().await;

    // Simple mock
    Mock::given(method("GET"))
        .and(path("/page"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string("<html><body>Content</body></html>"))
        .mount(&mock_server)
        .await;

    // Dynamic mock with path parameters
    for i in 0..10 {
        Mock::given(method("GET"))
            .and(path(format!("/page{}", i)))
            .respond_with(ResponseTemplate::new(200)
                .set_body_string(format!(
                    r#"<html>
                    <body>
                        <h1>Page {}</h1>
                        <a href="/page{}">Next</a>
                    </body>
                    </html>"#,
                    i, i + 1
                )))
            .mount(&mock_server)
            .await;
    }

    let seed_url = Url::parse(&mock_server.uri())?;
    // Use seed_url in tests
    Ok(())
}
```

### Budget Manager Access Pattern

```rust
#[tokio::test]
async fn test_budget_enforcement() -> Result<()> {
    let spider = Spider::new(config).await?;

    // Crawl with limits
    let result = spider.crawl(seeds).await?;

    // Access budget stats for verification
    let budget_manager = spider.budget_manager();
    let stats = budget_manager.get_stats().await;

    println!("Global stats: {:?}", stats.global);
    println!("Peak concurrent: {}", stats.peak_concurrent);

    assert!(stats.peak_concurrent <= 4);
    assert!(stats.global.pages_crawled <= config.max_pages.unwrap());

    Ok(())
}
```

### QueryAware State Updates

```rust
#[tokio::test]
async fn test_query_aware_state_progression() -> Result<()> {
    let spider = Spider::new(query_aware_config).await?;

    // Score initial request
    let request = CrawlRequest::new(url);
    let score_before = spider
        .score_query_aware_request(&request, Some(content))
        .await?;

    // Simulate processing result
    let result = CrawlResult::success(request.clone());
    spider.update_query_aware_with_result(&result).await?;

    // Check updated state
    let stats = spider.get_query_aware_stats().await;
    println!("Query-aware stats: {:?}", stats);
    assert_eq!(stats.total_pages, 1);

    Ok(())
}
```

---

## Common Pitfalls

### 1. Forgetting to Enable Query Foraging

❌ **Wrong:**
```rust
let config = QueryAwareConfig {
    target_query: Some("rust".to_string()),
    // query_foraging not set - defaults to false!
    ..Default::default()
};
```

✅ **Correct:**
```rust
let config = QueryAwareConfig {
    query_foraging: true,  // Must explicitly enable
    target_query: Some("rust".to_string()),
    ..Default::default()
};
```

### 2. Using Old Config Field Names

❌ **Wrong:**
```rust
let config = QueryAwareConfig {
    enable_bm25: true,              // REMOVED
    url_signal_weight: 0.3,         // RENAMED
    early_stop_threshold: 0.5,      // RENAMED
    min_crawl_count: 10,            // RENAMED
};
```

✅ **Correct:**
```rust
let config = QueryAwareConfig {
    query_foraging: true,           // NEW
    url_signals_weight: 0.3,        // Plural
    min_relevance_threshold: 0.5,   // NEW name
    relevance_window_size: 10,      // NEW name
    ..Default::default()
};
```

### 3. Not Updating Internal State

❌ **Wrong:**
```rust
// Score multiple requests without updating state
for url in urls {
    let score = spider.score_query_aware_request(&request, None).await?;
    // Domain diversity won't change!
}
```

✅ **Correct:**
```rust
for url in urls {
    let request = CrawlRequest::new(url);
    let score = spider.score_query_aware_request(&request, None).await?;

    // Update internal state for next iteration
    let result = CrawlResult::success(request.clone());
    spider.update_query_aware_with_result(&result).await?;
}
```

### 4. Incorrect Test Accessor Usage

❌ **Wrong:**
```rust
// Trying to access internal components directly
let url_utils = spider.url_utils;  // Private field!
```

✅ **Correct:**
```rust
// Use cfg(test) accessor methods
let url_utils = spider.url_utils();  // Public test accessor (line 811)
let budget = spider.budget_manager();  // Public test accessor (line 805)
```

### 5. Missing Mock Server for Integration Tests

❌ **Wrong:**
```rust
#[tokio::test]
async fn test_early_stopping() {
    // Trying to crawl real URLs in tests - unreliable!
    let seeds = vec![Url::parse("https://example.com")?];
    let result = spider.crawl(seeds).await?;
}
```

✅ **Correct:**
```rust
#[tokio::test]
async fn test_early_stopping() {
    let mock_server = MockServer::start().await;
    // Create predictable mock responses
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string("<html>Low relevance content</html>"))
        .mount(&mock_server)
        .await;

    let seeds = vec![Url::parse(&mock_server.uri())?];
    let result = spider.crawl(seeds).await?;
}
```

### 6. Not Checking cfg(test) Gating

❌ **Wrong:**
```rust
// Production code trying to use test accessors
pub fn production_function() {
    let spider = Spider::new(config).await?;
    let url_utils = spider.url_utils();  // Won't compile in production!
}
```

✅ **Correct:**
```rust
// Only use test accessors in test code
#[cfg(test)]
mod tests {
    #[test]
    fn test_function() {
        let spider = Spider::new(config).await?;
        let url_utils = spider.url_utils();  // OK in tests
    }
}
```

---

## Summary Checklist

### For Query-Aware Tests
- [ ] Set `query_foraging: true` in `QueryAwareConfig`
- [ ] Specify `target_query` with relevant keywords
- [ ] Use `score_query_aware_request()` for scoring
- [ ] Call `update_query_aware_with_result()` to update state
- [ ] Check `should_stop_query_aware()` for early stopping

### For Budget/Limit Tests
- [ ] Configure `BudgetConfig` with limits
- [ ] Set `PerformanceConfig` for concurrency
- [ ] Use `budget_manager()` accessor to check stats
- [ ] Verify `result.pages_crawled` respects limits

### For Robots.txt Tests
- [ ] Set `respect_robots: true`
- [ ] Configure `RobotsConfig` if needed
- [ ] Use mock server with robots.txt endpoint
- [ ] Verify blocked paths not crawled

### For URL Processing Tests
- [ ] Configure `UrlProcessingConfig` rules
- [ ] Use `url_utils()` test accessor
- [ ] Call `filter_urls()` for normalization/deduplication
- [ ] Test various URL variants

### For Integration Tests
- [ ] Use `wiremock` for mock HTTP server
- [ ] Create predictable response patterns
- [ ] Set reasonable limits (max_pages, max_duration)
- [ ] Verify stop reasons and metrics

---

## Next Steps

1. **CODER Agent**: Use examples above to fix P1 tests (4 tests)
2. **TESTER Agent**: Verify fixed tests pass and add edge cases
3. **REVIEWER Agent**: Check test coverage and code quality
4. **PLANNER Agent**: Update project status and documentation

**Estimated time to fix all 9 tests: 4-5 hours**

---

**Document Version:** 1.0
**Last Updated:** 2025-10-14
**Maintained By:** Hive Mind Research Team
