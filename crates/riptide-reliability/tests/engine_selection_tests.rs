//! Engine Selection Tests
//!
//! Comprehensive tests for engine selection logic including framework detection,
//! content analysis, and decision-making algorithms.

use riptide_reliability::engine_selection::{
    analyze_content, calculate_content_ratio, decide_engine, Engine,
};

#[test]
fn test_react_framework_detection() {
    let html =
        r#"<html><head><script>window.__NEXT_DATA__={}</script></head><body>Content</body></html>"#;
    let engine = decide_engine(html, "https://example.com");
    assert_eq!(engine, Engine::Headless, "React apps require headless");

    let analysis = analyze_content(html, "https://example.com");
    assert!(analysis.has_react, "Should detect React framework");
    assert_eq!(analysis.recommended_engine, Engine::Headless);
}

#[test]
fn test_vue_framework_detection() {
    let html = r#"<html><body><div id="app" v-app>Vue App</div></body></html>"#;
    let engine = decide_engine(html, "https://example.com");
    assert_eq!(engine, Engine::Headless, "Vue apps require headless");

    let analysis = analyze_content(html, "https://example.com");
    assert!(analysis.has_vue, "Should detect Vue framework");
    assert_eq!(analysis.recommended_engine, Engine::Headless);
}

#[test]
fn test_angular_framework_detection() {
    let html = r#"<html><body ng-app="myApp"><div ng-version="15.0.0">Angular</div></body></html>"#;
    let engine = decide_engine(html, "https://example.com");
    assert_eq!(engine, Engine::Headless, "Angular apps require headless");

    let analysis = analyze_content(html, "https://example.com");
    assert!(analysis.has_angular, "Should detect Angular framework");
    assert_eq!(analysis.recommended_engine, Engine::Headless);
}

#[test]
fn test_anti_scraping_detection() {
    // Test Cloudflare protection
    let cloudflare_html = r#"<html><body>Cloudflare protection active</body></html>"#;
    assert_eq!(
        decide_engine(cloudflare_html, "https://protected.com"),
        Engine::Headless,
        "Cloudflare protection requires headless"
    );

    // Test reCAPTCHA
    let recaptcha_html = r#"<html><body><div class="g-recaptcha"></div></body></html>"#;
    assert_eq!(
        decide_engine(recaptcha_html, "https://protected.com"),
        Engine::Headless,
        "reCAPTCHA requires headless"
    );

    // Test hCaptcha
    let hcaptcha_html = r#"<html><body><div class="h-captcha"></div></body></html>"#;
    assert_eq!(
        decide_engine(hcaptcha_html, "https://protected.com"),
        Engine::Headless,
        "hCaptcha requires headless"
    );
}

#[test]
fn test_standard_html_static_content() {
    let html = r#"
        <html>
        <head><title>Article</title></head>
        <body>
            <article>
                <h1>Main Title</h1>
                <p>This is substantial content with enough text to pass the content ratio
                threshold for WASM engine selection. The content ratio calculation will show
                this has good text-to-markup ratio.</p>
                <p>Another paragraph with more content that helps establish this is a
                standard article page with good structure.</p>
            </article>
        </body>
        </html>
    "#;

    let engine = decide_engine(html, "https://example.com/article");
    assert_eq!(engine, Engine::Wasm, "Standard HTML should use WASM");

    let analysis = analyze_content(html, "https://example.com/article");
    assert!(analysis.has_main_content, "Should detect main content");
    assert!(
        analysis.content_ratio > 0.2,
        "Should have good content ratio"
    );
    assert_eq!(analysis.recommended_engine, Engine::Wasm);
}

#[test]
fn test_low_content_ratio_triggers_headless() {
    // Minimal content with lots of markup
    let html = r#"
        <html>
        <head>
            <script src="app.js"></script>
            <script src="vendor.js"></script>
            <link rel="stylesheet" href="style.css">
            <link rel="stylesheet" href="theme.css">
        </head>
        <body>
            <div id="root"></div>
            <div id="app"></div>
        </body>
        </html>
    "#;

    let engine = decide_engine(html, "https://spa.example.com");
    assert_eq!(
        engine,
        Engine::Headless,
        "Low content ratio should trigger headless"
    );

    let ratio = calculate_content_ratio(html);
    assert!(ratio < 0.1, "Content ratio should be less than 0.1");
}

#[test]
fn test_spa_markers_detection() {
    // Test webpack markers
    let webpack_html = r#"<html><head><script>window.__webpack_require__={}</script></head><body><div id="app"></div></body></html>"#;
    let analysis = analyze_content(webpack_html, "https://example.com");
    assert!(analysis.has_spa_markers, "Should detect webpack SPA marker");
    assert_eq!(analysis.recommended_engine, Engine::Headless);

    // Test React helmet marker
    let helmet_html = r#"<html data-react-helmet="true"><body><div id="root"></div></body></html>"#;
    let analysis = analyze_content(helmet_html, "https://example.com");
    assert!(
        analysis.has_spa_markers,
        "Should detect React Helmet marker"
    );
}

#[test]
fn test_content_ratio_calculation() {
    // Empty HTML
    assert_eq!(calculate_content_ratio(""), 0.0, "Empty HTML has 0 ratio");

    // Pure text (no tags) - note: content ratio extracts text between tags,
    // so pure text without any tags will return 0.0
    let ratio = calculate_content_ratio("Hello World");
    assert_eq!(
        ratio, 0.0,
        "Pure text without tags has no extractable content"
    );

    // Text with tags
    let html_with_tags = "<p>Hello World</p>";
    let ratio = calculate_content_ratio(html_with_tags);
    assert!(ratio > 0.0, "Text with tags should have positive ratio");

    // Balanced content
    let html = "<html><body><p>Hello World</p></body></html>";
    let ratio = calculate_content_ratio(html);
    assert!(ratio > 0.0 && ratio < 1.0, "Should be between 0 and 1");

    // Heavy markup
    let heavy_markup = "<div><span><i><b><u></u></b></i></span></div>";
    let ratio = calculate_content_ratio(heavy_markup);
    assert!(ratio < 0.1, "Heavy markup should have low ratio");
}

#[test]
fn test_engine_string_parsing() {
    use std::str::FromStr;

    assert_eq!(Engine::from_str("auto").unwrap(), Engine::Auto);
    assert_eq!(Engine::from_str("raw").unwrap(), Engine::Raw);
    assert_eq!(Engine::from_str("wasm").unwrap(), Engine::Wasm);
    assert_eq!(Engine::from_str("headless").unwrap(), Engine::Headless);

    // Case insensitive
    assert_eq!(Engine::from_str("AUTO").unwrap(), Engine::Auto);
    assert_eq!(Engine::from_str("WASM").unwrap(), Engine::Wasm);

    // Invalid input
    assert!(Engine::from_str("invalid").is_err());
    assert!(Engine::from_str("").is_err());
}

#[test]
fn test_engine_display_and_name() {
    assert_eq!(Engine::Auto.name(), "auto");
    assert_eq!(Engine::Raw.name(), "raw");
    assert_eq!(Engine::Wasm.name(), "wasm");
    assert_eq!(Engine::Headless.name(), "headless");

    assert_eq!(format!("{}", Engine::Auto), "auto");
    assert_eq!(format!("{}", Engine::Wasm), "wasm");
}

#[test]
fn test_wasm_content_with_good_ratio() {
    let html = r#"
        <html>
        <head><script src="app.wasm"></script></head>
        <body>
            <article>
                This is substantial content with enough text to pass the content ratio
                threshold for WASM engine selection. The content ratio calculation will
                show this has good text-to-markup ratio. This ensures we properly handle
                WASM modules with good content structure.
            </article>
        </body>
        </html>
    "#;

    let engine = decide_engine(html, "https://example.com");
    assert_eq!(
        engine,
        Engine::Wasm,
        "WASM content with good ratio should use WASM"
    );
}

#[test]
fn test_wasm_content_with_low_ratio() {
    let html = r#"<html><head><script src="app.wasm"></script></head><body><div id="root"></div></body></html>"#;

    let engine = decide_engine(html, "https://example.com");
    assert_eq!(
        engine,
        Engine::Headless,
        "WASM with low content ratio should use headless"
    );
}

#[test]
fn test_detailed_content_analysis() {
    let html = r#"
        <html>
        <head>
            <script>window.__NEXT_DATA__={}</script>
            <meta property="og:title" content="Article">
        </head>
        <body>
            <article>
                <h1>Title</h1>
                <p>Content here</p>
            </article>
        </body>
        </html>
    "#;

    let analysis = analyze_content(html, "https://example.com");

    assert!(analysis.has_react, "Should detect React");
    assert!(!analysis.has_vue, "Should not detect Vue");
    assert!(!analysis.has_angular, "Should not detect Angular");
    assert!(
        !analysis.has_anti_scraping,
        "Should not detect anti-scraping"
    );
    assert!(analysis.has_main_content, "Should detect main content");
    assert!(analysis.content_ratio > 0.0, "Should have content");
    assert_eq!(analysis.recommended_engine, Engine::Headless);
}

#[test]
fn test_case_insensitive_detection() {
    // Mixed case React detection
    let html_upper = r#"<HTML><BODY><DIV DATA-REACTROOT>Test</DIV></BODY></HTML>"#;
    let analysis = analyze_content(html_upper, "https://example.com");
    assert!(analysis.has_react, "Should detect uppercase React markers");

    // Mixed case Cloudflare
    let html_cf = r#"<html><body>CLOUDFLARE Protection Active</body></html>"#;
    let engine = decide_engine(html_cf, "https://example.com");
    assert_eq!(
        engine,
        Engine::Headless,
        "Should detect uppercase Cloudflare"
    );
}

#[test]
fn test_engine_decision_priority() {
    // Priority 1: Anti-scraping always wins
    let anti_scraping_with_good_content = r#"
        <html><body>
            <article>Lots of content here</article>
            Cloudflare protection active
        </body></html>
    "#;
    assert_eq!(
        decide_engine(anti_scraping_with_good_content, "https://example.com"),
        Engine::Headless,
        "Anti-scraping has highest priority"
    );

    // Priority 2: Framework detection
    let react_with_content = r#"
        <html><head><script>window.__NEXT_DATA__={}</script></head>
        <body><article>Good content ratio</article></body></html>
    "#;
    assert_eq!(
        decide_engine(react_with_content, "https://example.com"),
        Engine::Headless,
        "Framework detection has second priority"
    );
}
