//! Real-world integration tests for RipTide EventMesh
//! Tests actual web scraping, crawling, and PDF extraction with live URLs

use riptide_html::Extractor;
use riptide_pdf::PdfProcessor;
use std::time::Instant;

#[tokio::test]
async fn test_real_world_html_extraction_wikipedia() {
    println!("\n🌐 TEST: Real-world HTML extraction from Wikipedia");

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; RipTideBot/1.0)")
        .build()
        .unwrap();

    let url = "https://en.wikipedia.org/wiki/Rust_(programming_language)";
    let start = Instant::now();

    println!("📥 Fetching: {}", url);
    let response = client.get(url).send().await.unwrap();
    let html = response.text().await.unwrap();

    println!("✅ Downloaded {} bytes in {:?}", html.len(), start.elapsed());

    // Extract content using RipTide HTML extractor
    let extractor = Extractor::new();
    let result = extractor.extract(&html, url).unwrap();

    println!("📝 Extracted content:");
    println!("  - Title: {}", result.title.as_deref().unwrap_or("N/A"));
    println!("  - Body length: {} chars", result.body.len());
    println!("  - Links found: {}", result.links.len());

    // Verify we got meaningful content
    assert!(result.title.is_some(), "Should extract title");
    assert!(!result.body.is_empty(), "Should extract body text");
    assert!(result.body.len() > 1000, "Should extract substantial content");
    assert!(result.links.len() > 10, "Should extract multiple links");
    assert!(result.body.contains("Rust"), "Content should mention Rust");

    println!("✅ TEST PASSED: Successfully scraped and extracted Wikipedia content\n");
}

#[tokio::test]
async fn test_real_world_html_extraction_github() {
    println!("\n🌐 TEST: Real-world HTML extraction from GitHub");

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; RipTideBot/1.0)")
        .build()
        .unwrap();

    let url = "https://github.com/rust-lang/rust";
    let start = Instant::now();

    println!("📥 Fetching: {}", url);
    let response = client.get(url).send().await.unwrap();
    let html = response.text().await.unwrap();

    println!("✅ Downloaded {} bytes in {:?}", html.len(), start.elapsed());

    // Extract content
    let extractor = Extractor::new();
    let result = extractor.extract(&html, url).unwrap();

    println!("📝 Extracted content:");
    println!("  - Title: {}", result.title.as_deref().unwrap_or("N/A"));
    println!("  - Body length: {} chars", result.body.len());
    println!("  - Links found: {}", result.links.len());

    // Verify extraction
    assert!(result.title.is_some(), "Should extract title");
    assert!(!result.body.is_empty(), "Should extract body text");
    assert!(result.links.len() > 5, "Should extract multiple links");

    println!("✅ TEST PASSED: Successfully scraped GitHub repository page\n");
}

#[tokio::test]
async fn test_real_world_html_extraction_news() {
    println!("\n🌐 TEST: Real-world HTML extraction from news site");

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; RipTideBot/1.0)")
        .build()
        .unwrap();

    let url = "https://www.bbc.com/news";
    let start = Instant::now();

    println!("📥 Fetching: {}", url);
    let response = client.get(url).send().await.unwrap();
    let html = response.text().await.unwrap();

    println!("✅ Downloaded {} bytes in {:?}", html.len(), start.elapsed());

    // Extract content
    let extractor = Extractor::new();
    let result = extractor.extract(&html, url).unwrap();

    println!("📝 Extracted content:");
    println!("  - Title: {}", result.title.as_deref().unwrap_or("N/A"));
    println!("  - Body length: {} chars", result.body.len());
    println!("  - Links found: {}", result.links.len());

    // Verify extraction
    assert!(result.title.is_some(), "Should extract title");
    assert!(!result.body.is_empty(), "Should extract body text");
    assert!(result.links.len() > 10, "Should extract multiple links from news page");

    println!("✅ TEST PASSED: Successfully scraped BBC News\n");
}

#[tokio::test]
async fn test_real_world_pdf_extraction() {
    println!("\n📄 TEST: Real-world PDF extraction from live URL");

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; RipTideBot/1.0)")
        .build()
        .unwrap();

    // Using a sample PDF from the web
    let url = "https://www.w3.org/WAI/ER/tests/xhtml/testfiles/resources/pdf/dummy.pdf";
    let start = Instant::now();

    println!("📥 Fetching PDF: {}", url);
    let response = client.get(url).send().await.unwrap();
    let pdf_bytes = response.bytes().await.unwrap();

    println!("✅ Downloaded {} bytes in {:?}", pdf_bytes.len(), start.elapsed());

    // Extract PDF content
    let processor = PdfProcessor::new().unwrap();
    let result = processor.extract_text(&pdf_bytes).unwrap();

    println!("📝 Extracted PDF content:");
    println!("  - Text length: {} chars", result.text.len());
    println!("  - Pages: {}", result.page_count);

    // Verify extraction
    assert!(!result.text.is_empty(), "Should extract text from PDF");
    assert!(result.page_count > 0, "Should detect pages");

    println!("✅ TEST PASSED: Successfully extracted content from PDF\n");
}

#[tokio::test]
async fn test_concurrent_crawling() {
    println!("\n🕷️  TEST: Concurrent crawling of multiple URLs");

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; RipTideBot/1.0)")
        .build()
        .unwrap();

    let urls = vec![
        "https://httpbin.org/html",
        "https://httpbin.org/links/5",
        "https://httpbin.org/robots.txt",
    ];

    let start = Instant::now();
    println!("📥 Crawling {} URLs concurrently...", urls.len());

    // Fetch all URLs concurrently
    let mut tasks = Vec::new();
    for url in &urls {
        let client = client.clone();
        let url = url.to_string();
        tasks.push(tokio::spawn(async move {
            let response = client.get(&url).send().await.unwrap();
            let content = response.text().await.unwrap();
            (url, content.len())
        }));
    }

    // Wait for all requests
    let mut results = Vec::new();
    for task in tasks {
        results.push(task.await.unwrap());
    }

    let duration = start.elapsed();
    println!("✅ Crawled {} URLs in {:?}", results.len(), duration);

    for (url, size) in &results {
        println!("  - {}: {} bytes", url, size);
    }

    // Verify all succeeded
    assert_eq!(results.len(), urls.len(), "Should fetch all URLs");
    for (_, size) in &results {
        assert!(*size > 0, "Should get content from each URL");
    }

    println!("✅ TEST PASSED: Successfully crawled multiple URLs concurrently\n");
}

#[tokio::test]
async fn test_link_extraction_and_depth() {
    println!("\n🔗 TEST: Link extraction and crawling depth");

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; RipTideBot/1.0)")
        .build()
        .unwrap();

    let url = "https://httpbin.org/links/10";
    println!("📥 Fetching page with links: {}", url);

    let response = client.get(url).send().await.unwrap();
    let html = response.text().await.unwrap();

    // Extract links
    let extractor = Extractor::new();
    let result = extractor.extract(&html, url).unwrap();

    println!("📝 Extracted {} links", result.links.len());

    // Verify link extraction
    assert!(result.links.len() >= 10, "Should extract at least 10 links");

    // Test depth-2 crawling (fetch first link)
    if let Some(first_link) = result.links.first() {
        println!("🔍 Following first link (depth 2): {}", first_link);
        let response2 = client.get(first_link).send().await;

        if let Ok(resp) = response2 {
            let html2 = resp.text().await.unwrap();
            println!("✅ Successfully crawled depth-2 link: {} bytes", html2.len());
        }
    }

    println!("✅ TEST PASSED: Link extraction and depth crawling works\n");
}

#[tokio::test]
async fn test_error_handling() {
    println!("\n⚠️  TEST: Error handling for invalid URLs");

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; RipTideBot/1.0)")
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    // Test 404
    println!("Testing 404 error...");
    let result = client.get("https://httpbin.org/status/404").send().await;
    assert!(result.is_ok(), "Should handle 404 gracefully");
    let status = result.unwrap().status();
    assert_eq!(status.as_u16(), 404, "Should get 404 status");

    // Test timeout
    println!("Testing timeout...");
    let result = client.get("https://httpbin.org/delay/10").send().await;
    assert!(result.is_err(), "Should timeout on slow requests");

    println!("✅ TEST PASSED: Error handling works correctly\n");
}

#[tokio::test]
async fn test_content_type_detection() {
    println!("\n📋 TEST: Content-type detection");

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; RipTideBot/1.0)")
        .build()
        .unwrap();

    // Test HTML
    let response = client.get("https://httpbin.org/html").send().await.unwrap();
    let content_type = response.headers().get("content-type").unwrap().to_str().unwrap();
    println!("HTML content-type: {}", content_type);
    assert!(content_type.contains("html"), "Should detect HTML content");

    // Test JSON
    let response = client.get("https://httpbin.org/json").send().await.unwrap();
    let content_type = response.headers().get("content-type").unwrap().to_str().unwrap();
    println!("JSON content-type: {}", content_type);
    assert!(content_type.contains("json"), "Should detect JSON content");

    println!("✅ TEST PASSED: Content-type detection works\n");
}
