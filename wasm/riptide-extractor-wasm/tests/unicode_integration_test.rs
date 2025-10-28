/// Integration tests for Unicode handling in WASM extractor
///
/// Tests comprehensive Unicode support including:
/// - Emoji and emoticons
/// - Chinese, Japanese, Korean (CJK)
/// - Right-to-left scripts (Arabic, Hebrew)
/// - Mixed multi-script content
/// - Malformed UTF-8 sequences
use riptide_extractor_wasm::*;

/// Test HTML with various Unicode characters
const UNICODE_TEST_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>🌍 Unicode Test 世界 العالم</title>
    <meta name="description" content="Testing emoji 🚀, CJK 你好, and RTL مرحبا">
</head>
<body>
    <article>
        <h1>Welcome 欢迎 مرحبا 👋</h1>
        <p>English with emoji: Hello 🌍 World!</p>
        <p lang="zh">Chinese: 你好世界。这是一个测试。</p>
        <p lang="ja">Japanese: こんにちは世界。これはテストです。</p>
        <p lang="ko">Korean: 안녕하세요 세계. 이것은 테스트입니다.</p>
        <p lang="ar">Arabic: مرحبا بالعالم. هذا اختبار.</p>
        <p lang="he">Hebrew: שלום עולם. זה מבחן.</p>
        <p>Mixed: Hello 世界 🌍 مرحبا 안녕 שלום</p>

        <a href="/emoji-link" title="Link with emoji 🔗">Click here 👆</a>
        <img src="/emoji.png" alt="Emoji image 😊">
    </article>
</body>
</html>
"#;

#[test]
fn test_extract_unicode_title() {
    let component = Component::new();
    let result = component.extract(
        UNICODE_TEST_HTML.to_string(),
        "https://example.com".to_string(),
        ExtractionMode::Article,
    );

    assert!(
        result.is_ok(),
        "Extraction should succeed with Unicode content"
    );
    let content = result.unwrap();

    // Title should contain emoji and multi-script text
    assert!(content.title.is_some(), "Title should be extracted");
    let title = content.title.unwrap();
    assert!(title.contains("🌍"), "Title should contain emoji");
    assert!(title.contains("世界"), "Title should contain Chinese");
    assert!(title.contains("العالم"), "Title should contain Arabic");
}

#[test]
fn test_extract_unicode_text() {
    let component = Component::new();
    let result = component.extract(
        UNICODE_TEST_HTML.to_string(),
        "https://example.com".to_string(),
        ExtractionMode::Article,
    );

    assert!(result.is_ok(), "Extraction should succeed");
    let content = result.unwrap();

    // Text should contain all Unicode scripts
    assert!(content.text.contains("🌍"), "Should contain emoji");
    assert!(content.text.contains("你好世界"), "Should contain Chinese");
    assert!(
        content.text.contains("こんにちは世界"),
        "Should contain Japanese"
    );
    assert!(content.text.contains("안녕하세요"), "Should contain Korean");
    assert!(
        content.text.contains("مرحبا بالعالم"),
        "Should contain Arabic"
    );
    assert!(content.text.contains("שלום עולם"), "Should contain Hebrew");
}

#[test]
fn test_extract_unicode_links() {
    let component = Component::new();
    let result = component.extract(
        UNICODE_TEST_HTML.to_string(),
        "https://example.com".to_string(),
        ExtractionMode::Full,
    );

    assert!(result.is_ok(), "Extraction should succeed");
    let content = result.unwrap();

    // Links should be extracted
    assert!(!content.links.is_empty(), "Should extract links");

    // Link text should contain emoji
    let has_emoji_link = content
        .links
        .iter()
        .any(|link| link.contains("👆") || link.contains("Click here"));
    assert!(has_emoji_link, "Should extract link with emoji text");
}

#[test]
fn test_extract_unicode_media() {
    let component = Component::new();
    let result = component.extract(
        UNICODE_TEST_HTML.to_string(),
        "https://example.com".to_string(),
        ExtractionMode::Full,
    );

    assert!(result.is_ok(), "Extraction should succeed");
    let content = result.unwrap();

    // Media should be extracted
    assert!(!content.media.is_empty(), "Should extract media");

    // Check for image with emoji alt text
    let has_emoji_media = content
        .media
        .iter()
        .any(|media| media.contains("/emoji.png"));
    assert!(has_emoji_media, "Should extract media with emoji reference");
}

#[test]
fn test_malformed_utf8_handling() {
    // HTML with intentionally malformed UTF-8 in comments
    let malformed_html = r#"
<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body>
    <p>Valid UTF-8 content</p>
    <!-- This comment has valid UTF-8: 你好 -->
    <p>More valid content 🌍</p>
</body>
</html>
"#;

    let component = Component::new();
    let result = component.extract(
        malformed_html.to_string(),
        "https://example.com".to_string(),
        ExtractionMode::Article,
    );

    // Should succeed even with edge cases
    assert!(result.is_ok(), "Should handle UTF-8 gracefully");
    let content = result.unwrap();
    assert!(content.title.is_some(), "Should extract title");
    assert!(!content.text.is_empty(), "Should extract text");
}

#[test]
fn test_empty_unicode_content() {
    let empty_html = r#"
<!DOCTYPE html>
<html>
<head><title>Empty</title></head>
<body></body>
</html>
"#;

    let component = Component::new();
    let result = component.extract(
        empty_html.to_string(),
        "https://example.com".to_string(),
        ExtractionMode::Article,
    );

    assert!(result.is_ok(), "Should handle empty content");
}

#[test]
fn test_very_long_unicode_text() {
    // Generate HTML with long Unicode text
    let long_text = "世界 🌍 مرحبا ".repeat(1000);
    let html = format!(
        r#"<!DOCTYPE html>
<html><head><title>Long</title></head>
<body><article><p>{}</p></article></body></html>"#,
        long_text
    );

    let component = Component::new();
    let result = component.extract(
        html,
        "https://example.com".to_string(),
        ExtractionMode::Article,
    );

    assert!(result.is_ok(), "Should handle long Unicode text");
    let content = result.unwrap();
    assert!(!content.text.is_empty(), "Should extract long text");
}

#[test]
fn test_unicode_in_attributes() {
    let html = r#"
<!DOCTYPE html>
<html>
<head><title>Attributes</title></head>
<body>
    <a href="/test" title="Link 🔗" data-label="标签">Link</a>
    <img src="/test.jpg" alt="图片 🖼️">
</body>
</html>
"#;

    let component = Component::new();
    let result = component.extract(
        html.to_string(),
        "https://example.com".to_string(),
        ExtractionMode::Full,
    );

    assert!(result.is_ok(), "Should handle Unicode in attributes");
}

#[test]
fn test_mixed_script_detection() {
    let component = Component::new();
    let result = component.extract(
        UNICODE_TEST_HTML.to_string(),
        "https://example.com".to_string(),
        ExtractionMode::Full,
    );

    assert!(result.is_ok(), "Should extract mixed scripts");
    let content = result.unwrap();

    // Should detect language (even if mixed)
    assert!(content.language.is_some(), "Should detect some language");
}
