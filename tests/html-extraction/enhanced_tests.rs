//! Comprehensive tests for enhanced HTML extraction
//!
//! This test suite covers:
//! - Metadata extraction (Open Graph, meta tags)
//! - Main content detection with article heuristics
//! - Link extraction with URL resolution
//! - Image and media extraction with alt text
//! - Quality score calculation

use riptide_core::html_parser::{EnhancedHtmlExtractor, MediaType};

#[test]
fn test_extract_og_metadata() {
    let html = r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <title>Test Article Title</title>
            <meta name="description" content="This is a test article description">
            <meta name="author" content="John Doe">
            <meta name="keywords" content="rust, web scraping, html extraction">
            <meta property="og:title" content="Enhanced OG Title">
            <meta property="og:description" content="Enhanced OG Description">
            <meta property="og:image" content="https://example.com/image.jpg">
            <meta name="date" content="2025-10-11">
        </head>
        <body>
            <article>
                <h1>Main Heading</h1>
                <p>Article content goes here.</p>
            </article>
        </body>
        </html>
    "#;

    let extractor = EnhancedHtmlExtractor::new(Some("https://example.com")).unwrap();
    let result = extractor.extract(html, "https://example.com").unwrap();

    // Check metadata extraction
    assert_eq!(result.metadata.title, Some("Test Article Title".to_string()));
    assert_eq!(result.metadata.description, Some("This is a test article description".to_string()));
    assert_eq!(result.metadata.author, Some("John Doe".to_string()));
    assert_eq!(result.metadata.og_title, Some("Enhanced OG Title".to_string()));
    assert_eq!(result.metadata.og_description, Some("Enhanced OG Description".to_string()));
    assert_eq!(result.metadata.og_image, Some("https://example.com/image.jpg".to_string()));
    assert_eq!(result.metadata.published_date, Some("2025-10-11".to_string()));
    assert_eq!(result.metadata.lang, Some("en".to_string()));
    assert_eq!(result.metadata.keywords.len(), 3);
    assert!(result.metadata.keywords.contains(&"rust".to_string()));

    // Verify it's detected as an article
    assert!(result.is_article);

    // Quality score should be high for complete metadata
    assert!(result.quality_score > 0.7, "Quality score: {}", result.quality_score);
}

#[test]
fn test_find_main_article_content() {
    let html = r#"
        <html>
        <head><title>Article Page</title></head>
        <body>
            <header>
                <nav>Navigation items should be excluded</nav>
            </header>
            <main>
                <article>
                    <h1>The Real Article Title</h1>
                    <p>This is the first paragraph of the article. It contains important information.</p>
                    <p>This is the second paragraph with more details. Articles typically have multiple paragraphs.</p>
                    <p>Here's a third paragraph to make it more realistic.</p>
                </article>
            </main>
            <footer>Footer content should be excluded</footer>
        </body>
        </html>
    "#;

    let extractor = EnhancedHtmlExtractor::new(None).unwrap();
    let result = extractor.extract(html, "https://example.com").unwrap();

    // Main content should not contain navigation or footer
    assert!(!result.main_content.contains("Navigation items"));
    assert!(!result.main_content.contains("Footer content"));

    // Should contain the article content
    assert!(result.main_content.contains("The Real Article Title"));
    assert!(result.main_content.contains("first paragraph"));
    assert!(result.main_content.contains("second paragraph"));
    assert!(result.main_content.contains("third paragraph"));

    // Should be detected as an article
    assert!(result.is_article);

    // Content length should be substantial
    assert!(result.main_content.len() > 100);
}

#[test]
fn test_extract_all_links() {
    let html = r#"
        <html>
        <head><title>Links Test</title></head>
        <body>
            <article>
                <a href="https://example.com/page1" title="Page 1">Link to Page 1</a>
                <a href="/relative/path" title="Relative">Relative Link</a>
                <a href="https://external.com" rel="nofollow">External Link</a>
                <a href="#anchor">Anchor Link</a>
            </article>
        </body>
        </html>
    "#;

    let extractor = EnhancedHtmlExtractor::new(Some("https://example.com")).unwrap();
    let result = extractor.extract(html, "https://example.com").unwrap();

    assert_eq!(result.links.len(), 4);

    // Check absolute URL
    let link1 = result.links.iter().find(|l| l.text == "Link to Page 1").unwrap();
    assert_eq!(link1.url, "https://example.com/page1");
    assert_eq!(link1.title, Some("Page 1".to_string()));

    // Check relative URL resolution
    let link2 = result.links.iter().find(|l| l.text == "Relative Link").unwrap();
    assert!(link2.url.contains("relative/path"));

    // Check rel attribute
    let link3 = result.links.iter().find(|l| l.text == "External Link").unwrap();
    assert_eq!(link3.rel, Some("nofollow".to_string()));

    // Check anchor link
    let link4 = result.links.iter().find(|l| l.text == "Anchor Link").unwrap();
    assert!(link4.url.contains("#anchor"));
}

#[test]
fn test_extract_images_with_alt_text() {
    let html = r#"
        <html>
        <head><title>Images Test</title></head>
        <body>
            <article>
                <img src="https://example.com/image1.jpg" alt="Beautiful landscape" width="800" height="600">
                <img src="/relative/image2.png" alt="Diagram" title="Technical Diagram">
                <img src="https://example.com/image3.gif" width="100" height="100">
            </article>
        </body>
        </html>
    "#;

    let extractor = EnhancedHtmlExtractor::new(Some("https://example.com")).unwrap();
    let result = extractor.extract(html, "https://example.com").unwrap();

    assert_eq!(result.media.len(), 3);

    // Check first image with dimensions
    let img1 = &result.media[0];
    assert_eq!(img1.media_type, MediaType::Image);
    assert_eq!(img1.url, "https://example.com/image1.jpg");
    assert_eq!(img1.alt_text, Some("Beautiful landscape".to_string()));
    assert_eq!(img1.width, Some(800));
    assert_eq!(img1.height, Some(600));

    // Check second image with title
    let img2 = &result.media[1];
    assert!(img2.url.contains("relative/image2.png"));
    assert_eq!(img2.alt_text, Some("Diagram".to_string()));
    assert_eq!(img2.title, Some("Technical Diagram".to_string()));

    // Check third image
    let img3 = &result.media[2];
    assert_eq!(img3.url, "https://example.com/image3.gif");
    assert_eq!(img3.width, Some(100));
    assert_eq!(img3.height, Some(100));
}

#[test]
fn test_quality_score_calculation() {
    // High quality HTML with complete metadata and content
    let high_quality_html = r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <title>High Quality Article</title>
            <meta name="description" content="A comprehensive article about web scraping">
            <meta name="author" content="Jane Smith">
            <meta name="keywords" content="web scraping, html, extraction">
            <meta property="og:title" content="High Quality Article">
            <meta property="og:description" content="OG Description">
            <meta property="og:image" content="https://example.com/og-image.jpg">
        </head>
        <body>
            <article>
                <h1>Main Article Title</h1>
                <p>This is a well-written article with substantial content. It has multiple sentences.
                The content is informative and well-structured. It provides value to readers.</p>
                <p>Another paragraph with additional information. Good articles have multiple paragraphs
                and cover topics in depth. This helps with quality scoring.</p>
                <p>A third paragraph to demonstrate length and depth. Quality content takes time to read
                and digest. This article exemplifies that principle.</p>
                <img src="https://example.com/img.jpg" alt="Relevant image">
                <a href="https://example.com/related">Related Article</a>
            </article>
        </body>
        </html>
    "#;

    let extractor = EnhancedHtmlExtractor::new(Some("https://example.com")).unwrap();
    let high_quality_result = extractor.extract(high_quality_html, "https://example.com").unwrap();

    // High quality content should score well
    assert!(high_quality_result.quality_score > 0.7,
            "High quality score: {}", high_quality_result.quality_score);
    assert!(high_quality_result.is_article);
    assert!(!high_quality_result.links.is_empty());
    assert!(!high_quality_result.media.is_empty());

    // Low quality HTML with minimal content
    let low_quality_html = r#"
        <html>
        <head></head>
        <body>
            <div>Short text</div>
        </body>
        </html>
    "#;

    let low_quality_result = extractor.extract(low_quality_html, "https://example.com").unwrap();

    // Low quality content should score lower
    assert!(low_quality_result.quality_score < 0.5,
            "Low quality score: {}", low_quality_result.quality_score);
    assert!(!low_quality_result.is_article);
    assert!(low_quality_result.links.is_empty());
    assert!(low_quality_result.media.is_empty());

    // Verify high quality scores better than low quality
    assert!(high_quality_result.quality_score > low_quality_result.quality_score);
}

#[test]
fn test_script_and_style_removal() {
    let html = r#"
        <html>
        <head>
            <title>Content with Scripts</title>
            <style>
                body { margin: 0; }
                .container { width: 100%; }
            </style>
        </head>
        <body>
            <script>
                console.log('This script should be removed');
                var x = 10;
            </script>
            <article>
                <h1>Article Title</h1>
                <p>This is the real content that should be extracted.</p>
            </article>
            <script src="https://example.com/analytics.js"></script>
        </body>
        </html>
    "#;

    let extractor = EnhancedHtmlExtractor::new(None).unwrap();
    let result = extractor.extract(html, "https://example.com").unwrap();

    // Content should not contain script or style code
    assert!(!result.main_content.contains("console.log"));
    assert!(!result.main_content.contains("var x = 10"));
    assert!(!result.main_content.contains("margin: 0"));
    assert!(!result.main_content.contains(".container"));

    // Should contain the actual article content
    assert!(result.main_content.contains("Article Title"));
    assert!(result.main_content.contains("real content"));
}

#[test]
fn test_video_extraction() {
    let html = r#"
        <html>
        <head><title>Video Test</title></head>
        <body>
            <article>
                <video src="https://example.com/video.mp4" title="Tutorial Video">
                    <source src="https://example.com/video.webm" type="video/webm">
                </video>
            </article>
        </body>
        </html>
    "#;

    let extractor = EnhancedHtmlExtractor::new(Some("https://example.com")).unwrap();
    let result = extractor.extract(html, "https://example.com").unwrap();

    // Should extract video elements
    assert!(!result.media.is_empty());

    let videos: Vec<_> = result.media.iter()
        .filter(|m| m.media_type == MediaType::Video)
        .collect();

    assert!(!videos.is_empty(), "Should extract at least one video");
    assert!(videos[0].url.contains("video"));
}

#[test]
fn test_markdown_conversion() {
    let html = r#"
        <html>
        <head><title>Markdown Test</title></head>
        <body>
            <article>
                <h1>Main Heading</h1>
                <p>First paragraph with some text.</p>
                <p>Second paragraph with more text.</p>
            </article>
        </body>
        </html>
    "#;

    let extractor = EnhancedHtmlExtractor::new(None).unwrap();
    let result = extractor.extract(html, "https://example.com").unwrap();

    // Markdown content should be clean text
    assert!(result.markdown_content.contains("Main Heading"));
    assert!(result.markdown_content.contains("First paragraph"));
    assert!(result.markdown_content.contains("Second paragraph"));

    // Should not contain HTML tags
    assert!(!result.markdown_content.contains("<h1>"));
    assert!(!result.markdown_content.contains("<p>"));
    assert!(!result.markdown_content.contains("</article>"));

    // Should have normalized whitespace
    assert!(!result.markdown_content.contains("  ")); // No double spaces
}

#[test]
fn test_relative_url_resolution() {
    let html = r#"
        <html>
        <head><title>URL Resolution Test</title></head>
        <body>
            <article>
                <a href="/path/to/page">Relative Path</a>
                <a href="../parent/page">Parent Path</a>
                <a href="sibling/page">Sibling Path</a>
                <img src="/images/photo.jpg" alt="Photo">
                <img src="./local/image.png" alt="Local">
            </article>
        </body>
        </html>
    "#;

    let extractor = EnhancedHtmlExtractor::new(Some("https://example.com/articles/post")).unwrap();
    let result = extractor.extract(html, "https://example.com/articles/post").unwrap();

    // All URLs should be resolved to absolute URLs
    assert!(result.links.iter().all(|link|
        link.url.starts_with("https://") || link.url.starts_with("http://")
    ));

    assert!(result.media.iter().all(|media|
        media.url.starts_with("https://") || media.url.starts_with("http://")
    ));
}

#[test]
fn test_empty_html() {
    let html = "";

    let extractor = EnhancedHtmlExtractor::new(None).unwrap();
    let result = extractor.extract(html, "https://example.com").unwrap();

    assert!(result.main_content.is_empty());
    assert!(result.links.is_empty());
    assert!(result.media.is_empty());
    assert!(result.quality_score < 0.3);
    assert!(!result.is_article);
}

#[test]
fn test_malformed_html() {
    let html = r#"
        <html>
        <head><title>Malformed</title>
        <body>
            <article>
                <p>Unclosed paragraph
                <div>Content here
            </article>
    "#;

    let extractor = EnhancedHtmlExtractor::new(None).unwrap();
    // Should not panic on malformed HTML
    let result = extractor.extract(html, "https://example.com");
    assert!(result.is_ok());

    let extracted = result.unwrap();
    // Should still extract some content
    assert!(!extracted.main_content.is_empty());
}
