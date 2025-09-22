# Rust Coding Standards for RipTide Crawler

This document outlines the coding standards and best practices for RipTide Crawler development. Following these guidelines ensures consistency, maintainability, and performance across the codebase.

## General Principles

### 1. Clarity Over Cleverness
Write code that is easy to read and understand. Prefer explicit over implicit, even if it requires more lines.

```rust
// ✅ Good: Clear and explicit
pub fn calculate_score(features: &GateFeatures) -> f32 {
    let text_ratio = calculate_text_ratio(features);
    let script_density = calculate_script_density(features);

    let mut score = 0.0;
    score += apply_text_ratio_bonus(text_ratio);
    score += apply_content_structure_bonus(features);
    score -= apply_script_penalty(script_density);

    score.clamp(0.0, 1.0)
}

// ❌ Avoid: Clever but unclear
pub fn calculate_score(f: &GateFeatures) -> f32 {
    (f.visible_text_chars as f32 / f.html_bytes as f32 * 1.2
        + (f.p_count as f32 + 1.0).ln() * 0.06
        + if f.has_og { 0.08 } else { 0.0 }
        - f.script_bytes as f32 / f.html_bytes as f32 * 0.8)
        .clamp(0.0, 1.0)
}
```

### 2. Fail Fast and Explicitly
Use `Result` types for operations that can fail. Avoid `unwrap()` in library code.

```rust
// ✅ Good: Explicit error handling
pub fn parse_config(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {:?}", path))?;

    serde_yaml::from_str(&content)
        .with_context(|| "Failed to parse YAML configuration")
}

// ❌ Avoid: Silent failures or panics
pub fn parse_config(path: &Path) -> Config {
    let content = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&content).unwrap()
}
```

### 3. Optimize for Readability First
Write readable code first, then optimize for performance when needed and proven by benchmarks.

## Formatting and Style

### Automatic Formatting
Use `rustfmt` with the project's `.rustfmt.toml` configuration:

```bash
cargo fmt --all
```

### Naming Conventions

#### Types and Traits
```rust
// PascalCase for types
pub struct ExtractedDocument { /* ... */ }
pub enum CrawlStrategy { /* ... */ }
pub trait ContentExtractor { /* ... */ }

// Descriptive names for complex types
pub type ExtractionResult = Result<ExtractedDocument>;
pub type UrlBatch = Vec<String>;
```

#### Functions and Variables
```rust
// snake_case for functions and variables
pub fn extract_main_content(html: &str) -> Result<String> { /* ... */ }
let extraction_timeout = Duration::from_secs(30);

// Use descriptive names
let extracted_documents = process_urls(&urls).await?;

// Avoid abbreviations unless they're domain-standard
let url_count = urls.len();  // ✅ Good
let url_cnt = urls.len();    // ❌ Avoid
```

#### Constants
```rust
// SCREAMING_SNAKE_CASE for constants
const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
const MAX_REDIRECT_COUNT: usize = 5;
const USER_AGENT_HEADER: &str = "RipTide/1.0";
```

## Code Organization

### Module Structure
```rust
// Prefer flat module hierarchies
pub mod config;
pub mod extraction;
pub mod fetching;
pub mod storage;

// Group related functionality
pub mod extraction {
    pub mod wasm;
    pub mod text;
    pub mod metadata;
}
```

### File Organization
Keep files focused and reasonably sized (< 500 lines when possible):

```rust
// src/extraction/mod.rs
mod wasm;
mod text;
mod metadata;

pub use wasm::WasmExtractor;
pub use text::extract_text_content;
pub use metadata::extract_metadata;

// Re-export main types
pub use crate::types::{ExtractedDocument, ExtractionMode};
```

### Import Guidelines
```rust
// Standard library first
use std::collections::HashMap;
use std::time::Duration;

// External crates second
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::time::timeout;

// Internal modules last
use crate::config::Config;
use crate::types::ExtractedDocument;

// Group and sort imports
use reqwest::{Client, Response};
use tokio::{fs, io};
```

## Error Handling

### Use `anyhow` for Application Errors
```rust
use anyhow::{Context, Result};

pub async fn fetch_and_extract(url: &str) -> Result<ExtractedDocument> {
    let html = fetch_html(url)
        .await
        .with_context(|| format!("Failed to fetch URL: {}", url))?;

    extract_content(&html)
        .with_context(|| "Content extraction failed")
}
```

### Custom Error Types for Libraries
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExtractionError {
    #[error("WASM module failed to execute")]
    WasmExecutionFailed,

    #[error("Invalid HTML structure: {reason}")]
    InvalidHtml { reason: String },

    #[error("Timeout after {seconds}s")]
    Timeout { seconds: u64 },
}
```

### Error Context Guidelines
```rust
// ✅ Good: Specific, actionable context
.with_context(|| format!("Failed to parse config file at {}", path.display()))?

// ✅ Good: Include relevant data
.with_context(|| format!("WASM extraction failed for URL: {}", url))?

// ❌ Avoid: Generic, unhelpful context
.with_context(|| "Something went wrong")?
```

## Async Programming

### Function Signatures
```rust
// ✅ Good: Clear async boundaries
pub async fn crawl_urls(urls: &[String]) -> Result<Vec<ExtractedDocument>> {
    // Implementation
}

// ✅ Good: Accept owned data for Send + 'static
pub fn spawn_crawler(urls: Vec<String>) -> tokio::task::JoinHandle<Result<()>> {
    tokio::spawn(async move {
        for url in urls {
            process_url(&url).await?;
        }
        Ok(())
    })
}
```

### Concurrency Patterns
```rust
use tokio::sync::Semaphore;

// Control concurrency with semaphores
pub async fn crawl_with_limit(
    urls: Vec<String>,
    max_concurrent: usize,
) -> Result<Vec<ExtractedDocument>> {
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let mut tasks = Vec::new();

    for url in urls {
        let semaphore = semaphore.clone();
        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire().await?;
            crawl_single_url(url).await
        });
        tasks.push(task);
    }

    // Collect results
    let mut results = Vec::new();
    for task in tasks {
        results.push(task.await??);
    }

    Ok(results)
}
```

### Timeout Handling
```rust
use tokio::time::{timeout, Duration};

pub async fn fetch_with_timeout(url: &str) -> Result<String> {
    let fetch_future = reqwest::get(url);

    timeout(Duration::from_secs(30), fetch_future)
        .await
        .context("Request timed out")?
        .context("HTTP request failed")?
        .text()
        .await
        .context("Failed to read response body")
}
```

## Memory Management

### Avoid Unnecessary Cloning
```rust
// ✅ Good: Use references when possible
pub fn analyze_content(content: &str) -> ContentAnalysis {
    ContentAnalysis {
        word_count: content.split_whitespace().count(),
        line_count: content.lines().count(),
    }
}

// ✅ Good: Take ownership when needed
pub fn store_content(content: String) -> StorageId {
    // Store takes ownership, avoiding clone
    storage.store(content)
}
```

### String Handling
```rust
// ✅ Good: Use &str for read-only operations
pub fn extract_title(html: &str) -> Option<&str> {
    // Return slice of input
}

// ✅ Good: Return String when creating new content
pub fn clean_text(input: &str) -> String {
    input.trim().replace("  ", " ")
}

// ✅ Good: Use Cow for conditional allocation
use std::borrow::Cow;

pub fn normalize_url(url: &str) -> Cow<str> {
    if url.ends_with('/') {
        Cow::Borrowed(url)
    } else {
        Cow::Owned(format!("{}/", url))
    }
}
```

## Documentation

### Module Documentation
```rust
//! Content extraction module.
//!
//! This module provides functionality for extracting structured content
//! from web pages using multiple strategies:
//!
//! - Fast path: Direct HTML parsing with lol-html
//! - WASM path: WebAssembly-based extraction using Trek
//! - Fallback path: Chrome DevTools Protocol rendering
//!
//! # Examples
//!
//! ```rust
//! use riptide_core::extraction::extract_content;
//!
//! let html = "<html><body><p>Content</p></body></html>";
//! let doc = extract_content(html.as_bytes(), "https://example.com").await?;
//! println!("Title: {:?}", doc.title);
//! ```
```

### Function Documentation
```rust
/// Extracts content from HTML using the configured strategy.
///
/// This function implements a three-tier extraction strategy:
/// 1. Fast parsing for simple, static content
/// 2. WASM-based extraction for complex content
/// 3. Headless browser fallback for dynamic content
///
/// # Arguments
///
/// * `html` - Raw HTML content as bytes
/// * `url` - Source URL for context and relative link resolution
/// * `config` - Extraction configuration options
///
/// # Returns
///
/// Returns `Ok(ExtractedDocument)` on success, containing:
/// - Extracted title, content, and metadata
/// - List of discovered links and media
/// - Markdown representation of the content
///
/// # Errors
///
/// This function returns an error if:
/// - The HTML is malformed beyond recovery
/// - All extraction strategies fail
/// - WASM module execution fails
/// - Network timeout occurs during headless fallback
///
/// # Examples
///
/// ```rust
/// use riptide_core::extraction::extract_content;
/// use riptide_core::config::ExtractionConfig;
///
/// # tokio_test::block_on(async {
/// let html = std::fs::read("example.html")?;
/// let config = ExtractionConfig::default();
///
/// let document = extract_content(&html, "https://example.com", &config).await?;
///
/// println!("Title: {}", document.title.unwrap_or_default());
/// println!("Content length: {}", document.text.len());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// # });
/// ```
pub async fn extract_content(
    html: &[u8],
    url: &str,
    config: &ExtractionConfig,
) -> Result<ExtractedDocument> {
    // Implementation
}
```

### Type Documentation
```rust
/// Configuration for content extraction behavior.
///
/// This struct controls how the extractor processes content and makes
/// decisions about which extraction strategy to use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionConfig {
    /// WASM module path for content extraction
    pub wasm_module_path: PathBuf,

    /// Timeout for individual extraction operations
    pub timeout: Duration,

    /// Whether to produce markdown output
    pub produce_markdown: bool,

    /// Maximum content length to process (bytes)
    pub max_content_length: usize,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            wasm_module_path: PathBuf::from("/opt/riptide/extractor.wasm"),
            timeout: Duration::from_secs(30),
            produce_markdown: true,
            max_content_length: 20 * 1024 * 1024, // 20MB
        }
    }
}
```

## Testing

### Unit Test Structure
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_extract_content_with_valid_html() {
        let html = include_bytes!("../test_data/article.html");
        let config = ExtractionConfig::default();

        let result = extract_content(html, "https://example.com", &config).await;

        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.title, Some("Test Article".to_string()));
        assert!(!doc.text.is_empty());
        assert!(!doc.links.is_empty());
    }

    #[tokio::test]
    async fn test_extract_content_with_malformed_html() {
        let html = b"<html><body><p>Unclosed paragraph";
        let config = ExtractionConfig::default();

        let result = extract_content(html, "https://example.com", &config).await;

        assert!(result.is_ok()); // Should handle gracefully
        let doc = result.unwrap();
        assert!(!doc.text.is_empty());
    }

    #[tokio::test]
    async fn test_extraction_timeout() {
        // Test with slow/hanging extraction
        let config = ExtractionConfig {
            timeout: Duration::from_millis(1),
            ..Default::default()
        };

        // Implementation would include timeout test
    }
}
```

### Integration Test Patterns
```rust
// tests/integration_crawl.rs
use riptide_api::test_utils::TestServer;
use serde_json::json;

#[tokio::test]
async fn test_crawl_endpoint_with_real_urls() {
    let server = TestServer::start().await;

    let response = server
        .post("/crawl")
        .json(&json!({
            "urls": ["https://httpbin.org/html"]
        }))
        .send()
        .await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await;
    assert!(body["results"].is_array());
    assert_eq!(body["results"].as_array().unwrap().len(), 1);
}
```

## Performance Guidelines

### Benchmarking
```rust
// benches/extraction_bench.rs
use criterion::{criterion_group, criterion_main, Criterion};
use riptide_core::extraction::extract_content;

fn bench_extraction(c: &mut Criterion) {
    let html = include_bytes!("../test_data/large_article.html");
    let config = ExtractionConfig::default();

    c.bench_function("extract_large_article", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| extract_content(html, "https://example.com", &config))
    });
}

criterion_group!(benches, bench_extraction);
criterion_main!(benches);
```

### Memory Optimization
```rust
// ✅ Good: Stream processing for large data
use tokio_stream::{Stream, StreamExt};

pub async fn process_large_sitemap(
    sitemap_stream: impl Stream<Item = String>,
) -> Result<Vec<ExtractedDocument>> {
    let mut results = Vec::new();

    tokio::pin!(sitemap_stream);
    while let Some(url) = sitemap_stream.next().await {
        // Process one URL at a time to control memory usage
        let doc = extract_url(&url).await?;
        results.push(doc);

        // Optional: yield control periodically
        if results.len() % 100 == 0 {
            tokio::task::yield_now().await;
        }
    }

    Ok(results)
}
```

## Security Guidelines

### Input Validation
```rust
use url::Url;

pub fn validate_crawl_url(url: &str) -> Result<Url> {
    let parsed = Url::parse(url)
        .context("Invalid URL format")?;

    // Only allow HTTP(S) schemes
    match parsed.scheme() {
        "http" | "https" => Ok(parsed),
        scheme => Err(anyhow!("Unsupported URL scheme: {}", scheme)),
    }
}
```

### Safe Defaults
```rust
impl Default for CrawlConfig {
    fn default() -> Self {
        Self {
            // Conservative defaults
            max_concurrent_requests: 5,
            request_timeout: Duration::from_secs(30),
            max_response_size: 10 * 1024 * 1024, // 10MB
            follow_redirects: true,
            max_redirects: 5,
            // Enable safety features by default
            respect_robots_txt: true,
            enable_rate_limiting: true,
        }
    }
}
```

## Configuration Management

### Environment Variables
```rust
use std::env;

pub struct AppConfig {
    pub redis_url: String,
    pub api_key: Option<String>,
    pub log_level: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            api_key: env::var("SERPER_API_KEY").ok(),
            log_level: env::var("RUST_LOG")
                .unwrap_or_else(|_| "info".to_string()),
        })
    }
}
```

## Clippy Configuration

Enable these clippy lints in your code:

```rust
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

// Allow specific lints when justified
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)] // When errors are obvious
```

These standards ensure that RipTide Crawler maintains high code quality, performance, and maintainability as it grows and evolves.