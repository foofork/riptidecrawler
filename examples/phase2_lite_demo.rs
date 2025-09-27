//! RipTide Phase-2 Lite Demo
//!
//! This example demonstrates the cache optimization, input validation,
//! and security features implemented in RipTide Phase-2 Lite.

use anyhow::Result;
use std::collections::HashMap;
use reqwest::header::HeaderMap;
use chrono::Utc;

use riptide_core::{
    phase2::{Phase2Manager, Phase2Config, CacheCheckResult},
    conditional::ConditionalRequest,
    cache::CacheConfig,
    security::SecurityConfig,
    common::validation::ValidationConfig,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 RipTide Phase-2 Lite Demo");
    println!("=============================");

    // Demo 1: Basic cache operations with version-aware keys
    demo_version_aware_caching().await?;

    // Demo 2: Input validation and security
    demo_input_validation_security().await?;

    // Demo 3: HTTP conditional GET support
    demo_conditional_get().await?;

    // Demo 4: Comprehensive Phase-2 integration
    demo_phase2_integration().await?;

    println!("\n✅ All demos completed successfully!");
    Ok(())
}

/// Demo 1: Version-aware caching with TTL
async fn demo_version_aware_caching() -> Result<()> {
    println!("\n📦 Demo 1: Version-Aware Caching");
    println!("---------------------------------");

    // Create Phase-2 manager with custom configuration
    let config = Phase2Config {
        redis_url: "redis://localhost:6379/1".to_string(),
        cache: CacheConfig {
            default_ttl: 60, // 1 minute for demo
            max_content_size: 1024 * 1024, // 1MB for demo
            cache_version: "demo_v1".to_string(),
            enable_etag: true,
            enable_last_modified: true,
        },
        ..Default::default()
    };

    let mut phase2_manager = Phase2Manager::new_with_config(config).await?;

    // Test URL and extraction options
    let url = "https://example.com/article";
    let extractor_version = "v2.1.0";
    let mut options = HashMap::new();
    options.insert("mode".to_string(), "article".to_string());
    options.insert("language".to_string(), "en".to_string());

    // Check cache (should be miss first time)
    println!("🔍 Checking cache for URL: {}", url);
    match phase2_manager.validate_and_check_cache(url, extractor_version, &options, None).await? {
        CacheCheckResult::Hit(entry) => {
            println!("✅ Cache hit! Content size: {} bytes", entry.data.len());
        }
        CacheCheckResult::NotModified(entry) => {
            println!("✅ Not modified! Content size: {} bytes", entry.data.len());
        }
        CacheCheckResult::Miss { cache_key, validated_url } => {
            println!("❌ Cache miss for key: {}", cache_key);
            println!("📋 Validated URL: {}", validated_url);

            // Simulate processing and cache the content
            let mock_content = b"Mock article content for demo purposes";
            let mock_response = create_mock_response();

            let cached_content = phase2_manager.process_and_cache_response(
                &cache_key,
                url,
                mock_response,
                extractor_version,
                &options,
                mock_content,
            ).await?;

            println!("💾 Cached content with ETag: {}", cached_content.etag);
        }
    }

    // Check cache statistics
    let stats = phase2_manager.get_cache_stats().await?;
    println!("📊 Cache stats: {} keys, {} bytes memory",
             stats.cache_stats.total_keys,
             stats.cache_stats.memory_usage_bytes);

    Ok(())
}

/// Demo 2: Input validation and security features
async fn demo_input_validation_security() -> Result<()> {
    println!("\n🔒 Demo 2: Input Validation & Security");
    println!("--------------------------------------");

    let phase2_manager = Phase2Manager::new("redis://localhost:6379/2").await?;

    // Test valid URLs
    let valid_urls = vec![
        "https://example.com",
        "http://test.org/path",
        "https://secure-site.net/article.html",
    ];

    for url in valid_urls {
        match phase2_manager.get_config().validation.clone() {
            config => {
                let validator = riptide_core::common::validation::CommonValidator::new(config);
                match validator.validate_url(url) {
                    Ok(_) => println!("✅ Valid URL: {}", url),
                    Err(e) => println!("❌ Invalid URL {}: {}", url, e),
                }
            }
        }
    }

    // Test invalid URLs
    let invalid_urls = vec![
        "ftp://example.com",           // Wrong scheme
        "https://localhost",           // Blocked private
        "https://192.168.1.1",         // Private IP
        "javascript:alert(1)",         // Dangerous scheme
    ];

    for url in invalid_urls {
        match phase2_manager.get_config().validation.clone() {
            config => {
                let validator = riptide_core::common::validation::CommonValidator::new(config);
                match validator.validate_url(url) {
                    Ok(_) => println!("⚠️  Unexpected valid URL: {}", url),
                    Err(e) => println!("✅ Correctly blocked URL {}: {}", url, e),
                }
            }
        }
    }

    // Test content type validation
    let content_types = vec![
        ("text/html", true),
        ("application/json", true),
        ("text/javascript", false),
        ("image/png", false),
    ];

    let validator = riptide_core::common::validation::CommonValidator::new(
        phase2_manager.get_config().validation.clone()
    );

    for (content_type, should_pass) in content_types {
        match validator.validate_content_type(content_type) {
            Ok(_) if should_pass => println!("✅ Allowed content type: {}", content_type),
            Ok(_) => println!("⚠️  Unexpected allowed type: {}", content_type),
            Err(_) if !should_pass => println!("✅ Correctly blocked type: {}", content_type),
            Err(e) => println!("❌ Unexpected blocked type {}: {}", content_type, e),
        }
    }

    // Demo security headers
    let mut headers = HeaderMap::new();
    let security_middleware = riptide_core::security::SecurityMiddleware::with_defaults()?;

    security_middleware.apply_security_headers(&mut headers)?;
    println!("🔐 Applied {} security headers", headers.len());

    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            println!("   {}: {}", name, value_str);
        }
    }

    Ok(())
}

/// Demo 3: HTTP conditional GET support
async fn demo_conditional_get() -> Result<()> {
    println!("\n🔄 Demo 3: HTTP Conditional GET");
    println!("-------------------------------");

    let mut phase2_manager = Phase2Manager::new("redis://localhost:6379/3").await?;

    let url = "https://example.com/dynamic-content";
    let extractor_version = "v2.1.0";
    let options = HashMap::new();

    // First request - cache miss
    println!("🔍 First request (cache miss):");
    match phase2_manager.validate_and_check_cache(url, extractor_version, &options, None).await? {
        CacheCheckResult::Miss { cache_key, .. } => {
            let mock_content = b"Dynamic content that changes";
            let mock_response = create_mock_response();

            let cached_content = phase2_manager.process_and_cache_response(
                &cache_key,
                url,
                mock_response,
                extractor_version,
                &options,
                mock_content,
            ).await?;

            println!("💾 Cached with ETag: {}", cached_content.etag);

            // Second request with conditional headers
            println!("\n🔍 Second request with If-None-Match:");
            let conditional = ConditionalRequest {
                if_none_match: Some(cached_content.etag.clone()),
                if_modified_since: None,
                if_match: None,
                if_unmodified_since: None,
            };

            match phase2_manager.validate_and_check_cache(
                url,
                extractor_version,
                &options,
                Some(conditional)
            ).await? {
                CacheCheckResult::NotModified(entry) => {
                    println!("✅ 304 Not Modified - content unchanged");
                    println!("📅 Last cached: {}", entry.created_at);
                }
                _ => println!("⚠️  Expected not modified response"),
            }
        }
        _ => println!("⚠️  Expected cache miss"),
    }

    Ok(())
}

/// Demo 4: Complete Phase-2 integration
async fn demo_phase2_integration() -> Result<()> {
    println!("\n🎯 Demo 4: Complete Phase-2 Integration");
    println!("---------------------------------------");

    // Create optimized Phase-2 manager
    let mut phase2_manager = riptide_core::phase2::create_optimized_phase2_manager(
        "redis://localhost:6379/4"
    ).await?;

    // Simulate a complete extraction workflow
    let url = "https://news.example.com/breaking-news";
    let extractor_version = "v2.1.0";
    let mut options = HashMap::new();
    options.insert("mode".to_string(), "article".to_string());
    options.insert("extract_images".to_string(), "true".to_string());
    options.insert("language".to_string(), "en".to_string());

    println!("🚀 Processing URL: {}", url);

    // Step 1: Validate and check cache
    match phase2_manager.validate_and_check_cache(url, extractor_version, &options, None).await? {
        CacheCheckResult::Hit(entry) => {
            println!("⚡ Cache hit! Serving from cache (size: {} bytes)", entry.data.len());
        }
        CacheCheckResult::Miss { cache_key, validated_url } => {
            println!("🔍 Cache miss - processing URL: {}", validated_url);

            // Step 2: Simulate extraction
            let mock_html = r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>Breaking News Article</title>
                    <meta name="description" content="Important news content">
                </head>
                <body>
                    <article>
                        <h1>Breaking News: Technology Advancement</h1>
                        <p>This is the content of the breaking news article...</p>
                        <img src="https://example.com/image.jpg" alt="News image">
                    </article>
                </body>
                </html>
            "#;

            // Step 3: Process and cache with security validation
            let mock_response = create_mock_response();
            let cached_content = phase2_manager.process_and_cache_response(
                &cache_key,
                url,
                mock_response,
                extractor_version,
                &options,
                mock_html.as_bytes(),
            ).await?;

            println!("💾 Content processed and cached:");
            println!("   Size: {} bytes", cached_content.content.len());
            println!("   ETag: {}", cached_content.etag);
            println!("   Cached at: {}", cached_content.cached_at);
        }
        CacheCheckResult::NotModified(entry) => {
            println!("✅ Content not modified (ETag match)");
        }
    }

    // Step 4: Get comprehensive statistics
    let stats = phase2_manager.get_cache_stats().await?;
    println!("\n📊 Final Statistics:");
    println!("   Cache keys: {}", stats.cache_stats.total_keys);
    println!("   Memory usage: {} bytes", stats.cache_stats.memory_usage_bytes);
    println!("   Cache version: {}", stats.cache_stats.cache_version);
    println!("   Max content size: {} bytes", stats.cache_stats.max_content_size);
    println!("   Default TTL: {} seconds", stats.cache_stats.default_ttl);
    println!("   Security features: {} enabled",
             if stats.security_config.enable_xss_protection { "XSS protection" } else { "Basic" });

    // Step 5: Cleanup demo cache
    let deleted = phase2_manager.clear_cache().await?;
    println!("🧹 Cleaned up {} cache entries", deleted);

    Ok(())
}

/// Create a mock HTTP response for testing
fn create_mock_response() -> reqwest::Response {
    // This would normally be a real HTTP response
    // For demo purposes, we'll create a mock that includes the necessary headers

    // Note: In a real implementation, you'd use the actual reqwest::Response
    // This is simplified for the demo
    use reqwest::Client;

    // Create a mock response - in practice this would come from actual HTTP fetch
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let client = Client::new();
            client.get("https://httpbin.org/html").send().await.unwrap()
        })
    })
}

/// Helper function to demonstrate usage patterns
async fn demonstrate_usage_patterns() -> Result<()> {
    println!("\n💡 Usage Patterns");
    println!("-----------------");

    // Pattern 1: Simple cache check
    let mut manager = Phase2Manager::new("redis://localhost:6379").await?;
    let options = HashMap::new();

    match manager.validate_and_check_cache(
        "https://example.com",
        "v1.0.0",
        &options,
        None
    ).await? {
        CacheCheckResult::Hit(entry) => {
            println!("Cache hit: {} bytes", entry.data.len());
        }
        CacheCheckResult::Miss { cache_key, .. } => {
            println!("Cache miss for key: {}", cache_key);
        }
        CacheCheckResult::NotModified(_) => {
            println!("Content not modified");
        }
    }

    // Pattern 2: With conditional request
    let conditional = ConditionalRequest {
        if_none_match: Some("\"abc123\"".to_string()),
        if_modified_since: Some(Utc::now()),
        if_match: None,
        if_unmodified_since: None,
    };

    match manager.validate_and_check_cache(
        "https://example.com",
        "v1.0.0",
        &options,
        Some(conditional)
    ).await? {
        CacheCheckResult::NotModified(_) => {
            println!("✅ 304 Not Modified response");
        }
        _ => {
            println!("Content was modified or cache miss");
        }
    }

    Ok(())
}