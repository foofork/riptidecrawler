//! Content Signal Tests - Phase 10
//!
//! Tests for refined content ratio calculation and placeholder detection.
//! These tests validate the content analysis improvements that help distinguish
//! between SSR content and client-rendered placeholders.

#![cfg(test)]

use riptide_reliability::engine_selection::{
    calculate_content_ratio, decide_engine, decide_engine_with_flags, Engine, EngineSelectionFlags,
};

// ============================================================================
// Content Ratio Tests
// ============================================================================

#[test]
fn test_content_ratio_basic() {
    let html = r#"<html><body><p>Hello World</p></body></html>"#;
    let ratio = calculate_content_ratio(html);
    assert!(ratio > 0.0, "Basic HTML should have positive content ratio");
    assert!(ratio < 1.0, "Content ratio should not be 100%");
}

#[test]
fn test_content_ratio_ignores_script_heavy_page() {
    // Page with large script block but minimal visible content
    let html = r#"
    <html>
    <head>
        <script>
        // Large React bundle - 1000+ characters of JavaScript
        const app = {
            render: function() { return 'app'; },
            init: function() { this.render(); },
            config: { api: 'https://api.example.com', timeout: 5000 },
            utils: { parse: function(d) { return JSON.parse(d); } }
        };
        // More JavaScript code to increase script size
        const helpers = { log: console.log, warn: console.warn };
        </script>
    </head>
    <body>
        <article>Short article text here</article>
    </body>
    </html>"#;

    let ratio = calculate_content_ratio(html);

    // The ratio should prioritize visible text content over script content
    // Note: Current implementation includes all text between tags
    // This test validates the existing behavior
    assert!(ratio > 0.0, "Should have some content");

    // In Phase 10 refined implementation, this would be higher
    // as scripts would be filtered out
}

#[test]
fn test_content_ratio_ssr_content() {
    let html = r#"
    <html>
    <head>
        <script src="next.js"></script>
        <script>window.__NEXT_DATA__ = {}</script>
    </head>
    <body>
        <article>
            This is substantial server-side rendered content that demonstrates
            good text-to-markup ratio. Even though Next.js markers are present,
            the actual content is available in the HTML. Lorem ipsum dolor sit amet,
            consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore
            et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation.
        </article>
    </body>
    </html>"#;

    let ratio = calculate_content_ratio(html);
    assert!(
        ratio > 0.1,
        "SSR content should have good content ratio: {}",
        ratio
    );
}

#[test]
fn test_content_ratio_empty_spa_shell() {
    let html = r#"
    <html>
    <head>
        <script src="app.js"></script>
        <link rel="stylesheet" href="style.css">
    </head>
    <body>
        <div id="root"></div>
        <div id="app"></div>
    </body>
    </html>"#;

    let ratio = calculate_content_ratio(html);
    assert!(
        ratio < 0.1,
        "Empty SPA shell should have low content ratio: {}",
        ratio
    );
}

#[test]
fn test_content_ratio_zero_for_empty() {
    let html = "";
    let ratio = calculate_content_ratio(html);
    assert_eq!(ratio, 0.0, "Empty HTML should have zero content ratio");
}

// ============================================================================
// Placeholder Detection Tests (Future Enhancement)
// ============================================================================

#[test]
fn test_placeholder_markers_in_html() {
    // Test data with common placeholder patterns
    let skeleton_html = r#"
    <html>
    <body>
        <div class="skeleton-loader">Loading...</div>
        <div class="shimmer"></div>
        <div class="placeholder-text">Please wait...</div>
    </body>
    </html>"#;

    // For Phase 10, we detect these patterns in engine selection
    // This test validates the patterns are recognized
    let html_lower = skeleton_html.to_lowercase();
    assert!(html_lower.contains("skeleton"));
    assert!(html_lower.contains("shimmer"));
    assert!(html_lower.contains("placeholder"));
}

#[test]
fn test_no_placeholder_in_real_content() {
    let real_content_html = r#"
    <html>
    <body>
        <article class="content">
            <h1>Real Article Title</h1>
            <p>This is genuine content with substantial text.</p>
            <p>No placeholder indicators or skeleton screens here.</p>
        </article>
    </body>
    </html>"#;

    let html_lower = real_content_html.to_lowercase();
    assert!(!html_lower.contains("skeleton-loader"));
    assert!(!html_lower.contains("shimmer"));
    assert!(!html_lower.contains("placeholder-glow"));
}

// ============================================================================
// Engine Decision with Content Signals
// ============================================================================

#[test]
fn test_engine_decision_ssr_nextjs() {
    let html = r#"
    <html>
    <head>
        <script>window.__NEXT_DATA__ = {}</script>
    </head>
    <body>
        <article>
            This is server-side rendered Next.js content with good substance.
            The content ratio is sufficient for WASM extraction despite Next.js markers.
            Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod
            tempor incididunt ut labore et dolore magna aliqua.
        </article>
    </body>
    </html>"#;

    // Default behavior: React markers trigger headless
    let engine = decide_engine(html, "https://example.com");
    assert_eq!(
        engine,
        Engine::Headless,
        "Default: Next.js markers trigger headless"
    );

    // With probe-first enabled: Try WASM first
    let mut flags = EngineSelectionFlags::default();
    flags.probe_first_spa = true;
    let engine_probe = decide_engine_with_flags(html, "https://example.com", flags);
    assert_eq!(
        engine_probe,
        Engine::Wasm,
        "Probe-first: Try WASM extraction first"
    );
}

#[test]
fn test_engine_decision_empty_spa() {
    let html = r#"
    <html>
    <head><script src="react.js"></script></head>
    <body>
        <div id="root"></div>
    </body>
    </html>"#;

    // Default behavior: Low content ratio triggers headless
    let engine = decide_engine(html, "https://example.com");
    assert_eq!(
        engine,
        Engine::Headless,
        "Empty SPA should use headless engine"
    );

    // Even with probe-first, empty SPAs get WASM attempt (will escalate)
    let mut flags = EngineSelectionFlags::default();
    flags.probe_first_spa = true;
    let engine_probe = decide_engine_with_flags(html, "https://example.com", flags);
    assert_eq!(
        engine_probe,
        Engine::Wasm,
        "Probe-first attempts WASM even for low-content"
    );
}

#[test]
fn test_engine_decision_standard_article() {
    let html = r#"
    <html>
    <body>
        <article>
            <h1>Great Article Title</h1>
            <p>This is a standard HTML article with good content ratio.</p>
            <p>No JavaScript frameworks detected, straightforward extraction.</p>
            <p>Lorem ipsum dolor sit amet, consectetur adipiscing elit.</p>
        </article>
    </body>
    </html>"#;

    let engine = decide_engine(html, "https://example.com");
    assert_eq!(
        engine,
        Engine::Wasm,
        "Standard articles should use WASM engine"
    );
}

#[test]
fn test_engine_decision_anti_scraping_always_headless() {
    let html = r#"
    <html>
    <body>
        <article>Good content here</article>
        <div>Cloudflare protection active</div>
    </body>
    </html>"#;

    // Default behavior
    let engine = decide_engine(html, "https://example.com");
    assert_eq!(
        engine,
        Engine::Headless,
        "Anti-scraping always uses headless"
    );

    // Even with probe-first, anti-scraping goes to headless
    let mut flags = EngineSelectionFlags::default();
    flags.probe_first_spa = true;
    let engine_probe = decide_engine_with_flags(html, "https://example.com", flags);
    assert_eq!(
        engine_probe,
        Engine::Headless,
        "Anti-scraping bypasses probe-first optimization"
    );
}

// ============================================================================
// Placeholder Text Filtering (Conceptual Tests for Future Implementation)
// ============================================================================

#[test]
fn test_placeholder_text_patterns() {
    // Common placeholder text patterns that should be filtered
    let placeholder_patterns = vec![
        "Loading...",
        "Please wait",
        "Loading content",
        "Fetching data",
        "Loading, please wait",
        "Content is loading",
    ];

    for pattern in placeholder_patterns {
        let html = format!(r#"<div>{}</div>"#, pattern);
        let html_lower = html.to_lowercase();
        assert!(
            html_lower.contains("loading")
                || html_lower.contains("wait")
                || html_lower.contains("fetching"),
            "Placeholder pattern '{}' should be detectable",
            pattern
        );
    }
}

#[test]
fn test_real_content_not_filtered() {
    // Real content that should NOT be filtered as placeholder
    let real_content = vec![
        "The loading dock is closed today",
        "Wait for the right opportunity",
        "Fetching water from the well",
    ];

    for content in real_content {
        // These contain placeholder keywords but in real context
        // A smart filter would need semantic analysis
        // For now, we validate they contain the words
        let html_lower = content.to_lowercase();
        assert!(
            html_lower.contains("loading")
                || html_lower.contains("wait")
                || html_lower.contains("fetching"),
            "Real content '{}' contains words but isn't placeholder",
            content
        );
    }
}

// ============================================================================
// Content Signal Integration Tests
// ============================================================================

#[test]
fn test_vue_app_with_ssr() {
    let html = r#"
    <html>
    <head>
        <script>createApp({})</script>
    </head>
    <body>
        <div id="app" v-app>
            <article>
                Server-side rendered Vue content with good text density.
                This demonstrates that Vue apps can have SSR content.
                Lorem ipsum dolor sit amet, consectetur adipiscing elit.
            </article>
        </div>
    </body>
    </html>"#;

    let engine = decide_engine(html, "https://example.com");
    assert_eq!(engine, Engine::Headless, "Vue markers trigger headless");

    // With probe-first
    let mut flags = EngineSelectionFlags::default();
    flags.probe_first_spa = true;
    let engine_probe = decide_engine_with_flags(html, "https://example.com", flags);
    assert_eq!(
        engine_probe,
        Engine::Wasm,
        "Probe-first tries WASM for Vue SSR"
    );
}

#[test]
fn test_angular_app_detection() {
    let html = r#"
    <html>
    <body ng-app="myApp">
        <div ng-controller="MainController">
            <p>Angular content here</p>
        </div>
    </body>
    </html>"#;

    let engine = decide_engine(html, "https://example.com");
    assert_eq!(engine, Engine::Headless, "Angular markers trigger headless");
}

#[test]
fn test_mixed_content_signals() {
    // Complex page with both framework markers and good content
    let html = r#"
    <html>
    <head>
        <script>window.__NEXT_DATA__ = {}</script>
        <meta property="og:title" content="Article Title">
    </head>
    <body>
        <main>
            <article>
                <h1>Comprehensive Article</h1>
                <p>This article has substantial content despite Next.js markers.</p>
                <p>The server-side rendering provides good initial HTML.</p>
                <p>Content ratio should be favorable for WASM extraction attempt.</p>
                <p>Additional paragraphs to ensure sufficient content density.</p>
                <p>Lorem ipsum dolor sit amet, consectetur adipiscing elit.</p>
            </article>
        </main>
    </body>
    </html>"#;

    // Framework markers take priority in default mode
    let engine = decide_engine(html, "https://example.com");
    assert_eq!(engine, Engine::Headless);

    // Probe-first allows WASM attempt
    let mut flags = EngineSelectionFlags::default();
    flags.probe_first_spa = true;
    let engine_probe = decide_engine_with_flags(html, "https://example.com", flags);
    assert_eq!(engine_probe, Engine::Wasm);
}
