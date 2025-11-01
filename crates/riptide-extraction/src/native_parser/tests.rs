//! Tests for native HTML parser

#[cfg(test)]
mod native_parser_tests {
    use crate::native_parser::{NativeHtmlParser, ParserConfig};

    #[test]
    fn test_title_extraction() {
        let html = r#"
            <html>
            <head>
                <meta property="og:title" content="Test Article">
                <title>Fallback Title</title>
            </head>
            <body>
                <article>
                    <h1>Header</h1>
                    <p>This is a test article with enough content to pass quality checks. We need multiple paragraphs to ensure the quality score meets the minimum threshold.</p>
                    <p>Additional paragraph to increase word count and quality score.</p>
                </article>
            </body>
            </html>
        "#;

        let parser = NativeHtmlParser::new();
        let doc = parser
            .parse_headless_html(html, "https://example.com")
            .unwrap();

        assert_eq!(doc.title, Some("Test Article".to_string()));
    }

    #[test]
    fn test_content_extraction() {
        let html = r#"
            <html>
            <head><title>Test</title></head>
            <body>
                <article>
                    <h1>Article Title</h1>
                    <p>This is the first paragraph with meaningful content.</p>
                    <p>This is the second paragraph with more content.</p>
                </article>
            </body>
            </html>
        "#;

        let parser = NativeHtmlParser::new();
        let doc = parser
            .parse_headless_html(html, "https://example.com")
            .unwrap();

        assert!(!doc.text.is_empty());
        assert!(doc.text.contains("first paragraph"));
        assert!(doc.text.contains("second paragraph"));
    }

    #[test]
    fn test_metadata_extraction() {
        let html = r#"
            <html>
            <head>
                <title>Test Article</title>
                <meta name="author" content="John Doe">
                <meta property="article:published_time" content="2024-01-01T00:00:00Z">
                <meta name="description" content="Test description">
            </head>
            <body>
                <article>
                    <p>This is a comprehensive article with sufficient content to pass quality validation checks.</p>
                    <p>Multiple paragraphs ensure the extracted content meets minimum quality thresholds.</p>
                    <p>Additional content helps demonstrate proper metadata extraction functionality.</p>
                </article>
            </body>
            </html>
        "#;

        let parser = NativeHtmlParser::new();
        let doc = parser
            .parse_headless_html(html, "https://example.com")
            .unwrap();

        assert_eq!(doc.byline, Some("John Doe".to_string()));
        assert_eq!(doc.published_iso, Some("2024-01-01T00:00:00Z".to_string()));
        assert_eq!(doc.description, Some("Test description".to_string()));
    }

    #[test]
    fn test_quality_scoring() {
        let html = r#"
            <html>
            <head><title>High Quality Article</title></head>
            <body>
                <article>
                    <h1>Main Title</h1>
                    <p>This is a long paragraph with lots of meaningful content that should result in a high quality score.</p>
                    <p>Another paragraph with even more content to increase the word count and quality.</p>
                    <p>Yet another paragraph demonstrating rich content extraction capabilities.</p>
                </article>
            </body>
            </html>
        "#;

        let parser = NativeHtmlParser::new();
        let doc = parser
            .parse_headless_html(html, "https://example.com")
            .unwrap();

        assert!(doc.quality_score.is_some());
        assert!(doc.quality_score.unwrap() >= 30);
    }

    #[test]
    fn test_link_extraction() {
        let html = r##"
            <html>
            <body>
                <article>
                    <p>This article contains several links and sufficient content to meet quality requirements.</p>
                    <p>The content includes enough text to pass validation and demonstrate link extraction.</p>
                    <a href="https://example.com/page1">Link 1</a>
                    <a href="/relative">Relative Link</a>
                    <a href="#fragment">Fragment</a>
                    <p>Additional paragraphs ensure quality thresholds are met for successful extraction.</p>
                </article>
            </body>
            </html>
        "##;

        let parser = NativeHtmlParser::new();
        let doc = parser
            .parse_headless_html(html, "https://example.com")
            .unwrap();

        assert!(!doc.links.is_empty());
        assert!(doc.links.iter().any(|l| l.contains("example.com/page1")));
    }

    #[test]
    fn test_fallback_strategy() {
        let html = r#"
            <html>
            <body>
                <div>Some text without proper article structure</div>
            </body>
            </html>
        "#;

        let parser = NativeHtmlParser::new();
        let doc = parser
            .extract_with_fallbacks(html, "https://example.com")
            .unwrap();

        assert!(!doc.text.is_empty());
    }

    #[test]
    fn test_oversized_html_rejection() {
        let parser = NativeHtmlParser::with_config(ParserConfig {
            max_content_length: 100,
            ..Default::default()
        });

        let html = "a".repeat(200);
        let result = parser.parse_headless_html(&html, "https://example.com");

        assert!(result.is_err());
    }

    #[test]
    fn test_markdown_generation() {
        let html = r#"
            <html>
            <body>
                <article>
                    <h1>Main Title</h1>
                    <p>First paragraph with substantial content to ensure quality requirements are met.</p>
                    <h2>Subtitle</h2>
                    <p>Second paragraph with additional text to increase word count and quality score.</p>
                    <p>Third paragraph demonstrating comprehensive markdown generation capabilities.</p>
                </article>
            </body>
            </html>
        "#;

        let parser = NativeHtmlParser::new();
        let doc = parser
            .parse_headless_html(html, "https://example.com")
            .unwrap();

        assert!(doc.markdown.is_some());
        let md = doc.markdown.unwrap();
        assert!(md.contains("# Main Title") || md.contains("Main Title"));
    }
}
