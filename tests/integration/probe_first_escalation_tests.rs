//! Probe-First Escalation Integration Tests - Phase 10
//!
//! Integration tests for the probe-first escalation strategy where SPAs
//! are first attempted with WASM extraction, then escalated to headless
//! only if the quality score is insufficient.
//!
//! These tests validate the complete workflow including quality assessment.

#![cfg(test)]

use riptide_reliability::engine_selection::{
    decide_engine_with_flags, should_escalate_to_headless, Engine, EngineSelectionFlags,
};

// ============================================================================
// SPA Probe-First Strategy Tests
// ============================================================================

#[test]
fn test_spa_probe_first_react_app() {
    let mut flags = EngineSelectionFlags::default();
    flags.probe_first_spa = true;

    let react_html = r#"
    <html>
    <head>
        <script>window.__NEXT_DATA__ = {page: 'home'}</script>
    </head>
    <body>
        <div id="__next">
            <article>
                Server-side rendered content from Next.js application.
                This content is available in the initial HTML payload.
                Lorem ipsum dolor sit amet, consectetur adipiscing elit.
            </article>
        </div>
    </body>
    </html>"#;

    let engine = decide_engine_with_flags(react_html, "https://example.com", flags);
    assert_eq!(
        engine,
        Engine::Wasm,
        "Probe-first should attempt WASM for React SSR"
    );
}

#[test]
fn test_spa_probe_first_vue_ssr() {
    let mut flags = EngineSelectionFlags::default();
    flags.probe_first_spa = true;

    let vue_html = r#"
    <html>
    <head>
        <script>createApp({})</script>
    </head>
    <body>
        <div id="app" data-server-rendered="true">
            <main>
                <h1>Vue SSR Content</h1>
                <p>This Vue application uses server-side rendering.</p>
                <p>Initial content is available for WASM extraction.</p>
            </main>
        </div>
    </body>
    </html>"#;

    let engine = decide_engine_with_flags(vue_html, "https://example.com", flags);
    assert_eq!(engine, Engine::Wasm, "Probe-first should try WASM for Vue SSR");
}

#[test]
fn test_spa_probe_first_disabled_by_default() {
    let flags = EngineSelectionFlags::default();
    assert!(!flags.probe_first_spa, "Probe-first should be disabled by default");

    let spa_html = r#"<html><script>window.__NEXT_DATA__={}</script></html>"#;
    let engine = decide_engine_with_flags(spa_html, "https://example.com", flags);
    assert_eq!(
        engine,
        Engine::Headless,
        "Default behavior: SPAs go directly to headless"
    );
}

#[test]
fn test_probe_first_low_content_ratio() {
    let mut flags = EngineSelectionFlags::default();
    flags.probe_first_spa = true;

    // SPA with very low content ratio (mostly empty)
    let low_content_html = r#"
    <html>
    <head>
        <script src="bundle.js"></script>
        <link rel="stylesheet" href="styles.css">
    </head>
    <body>
        <div id="root"></div>
        <div id="app"></div>
    </body>
    </html>"#;

    let engine = decide_engine_with_flags(low_content_html, "https://example.com", flags);
    assert_eq!(
        engine,
        Engine::Wasm,
        "Even low-content pages get WASM probe attempt"
    );
}

// ============================================================================
// Escalation Decision Tests
// ============================================================================

#[test]
fn test_escalation_high_quality_no_escalation() {
    // Good extraction results - no need to escalate
    let html = "<html><article>Good content</article></html>";

    // High quality score, good word count
    assert!(
        !should_escalate_to_headless(80, 300, html),
        "High quality extraction should not escalate"
    );

    // Decent quality, moderate word count
    assert!(
        !should_escalate_to_headless(60, 150, html),
        "Decent extraction should not escalate"
    );

    // Borderline but acceptable
    assert!(
        !should_escalate_to_headless(50, 100, html),
        "Borderline acceptable should not escalate"
    );
}

#[test]
fn test_escalation_low_quality_score() {
    let html = "<html><div id='root'></div></html>";

    // Very low quality - definitely escalate
    assert!(
        should_escalate_to_headless(20, 100, html),
        "Very low quality should escalate"
    );

    // Below threshold even with decent word count
    assert!(
        should_escalate_to_headless(25, 200, html),
        "Low quality despite word count should escalate"
    );
}

#[test]
fn test_escalation_insufficient_content() {
    let html = "<html><p>Short</p></html>";

    // Good quality but too little content
    assert!(
        should_escalate_to_headless(70, 30, html),
        "Insufficient word count should escalate"
    );

    // Very little content extracted
    assert!(
        should_escalate_to_headless(60, 20, html),
        "Very low word count should escalate"
    );
}

#[test]
fn test_escalation_borderline_cases() {
    let html = "<html><article>Some content</article></html>";

    // Just below borderline threshold - escalate
    assert!(
        should_escalate_to_headless(45, 80, html),
        "Below borderline should escalate"
    );

    assert!(
        should_escalate_to_headless(48, 95, html),
        "Close to threshold should escalate"
    );

    // Just above borderline threshold - keep WASM results
    assert!(
        !should_escalate_to_headless(50, 100, html),
        "At threshold should not escalate"
    );

    assert!(
        !should_escalate_to_headless(51, 80, html),
        "Just above threshold should not escalate"
    );
}

// ============================================================================
// Anti-Scraping Override Tests
// ============================================================================

#[test]
fn test_anti_scraping_bypasses_probe_first() {
    let mut flags = EngineSelectionFlags::default();
    flags.probe_first_spa = true;

    let cloudflare_html = r#"
    <html>
    <head>
        <title>Cloudflare Protection</title>
    </head>
    <body>
        <div>Checking your browser before accessing...</div>
        <div>Cloudflare Ray ID: abc123</div>
    </body>
    </html>"#;

    let engine = decide_engine_with_flags(cloudflare_html, "https://example.com", flags);
    assert_eq!(
        engine,
        Engine::Headless,
        "Anti-scraping always requires headless"
    );
}

#[test]
fn test_recaptcha_bypasses_probe_first() {
    let mut flags = EngineSelectionFlags::default();
    flags.probe_first_spa = true;

    let recaptcha_html = r#"
    <html>
    <head>
        <script src="https://www.google.com/recaptcha/api.js"></script>
    </head>
    <body>
        <div class="g-recaptcha"></div>
    </body>
    </html>"#;

    let engine = decide_engine_with_flags(recaptcha_html, "https://example.com", flags);
    assert_eq!(
        engine,
        Engine::Headless,
        "reCAPTCHA requires headless browser"
    );
}

// ============================================================================
// Real-World Scenario Tests
// ============================================================================

#[test]
fn test_nextjs_blog_with_ssr() {
    let mut flags = EngineSelectionFlags::default();
    flags.probe_first_spa = true;

    // Realistic Next.js blog post with SSR
    let nextjs_blog = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width">
        <title>Blog Post Title - My Blog</title>
        <script id="__NEXT_DATA__" type="application/json">
        {"props":{"pageProps":{"post":{"title":"Blog Post"}}}}
        </script>
    </head>
    <body>
        <div id="__next">
            <article>
                <h1>Understanding Web Development</h1>
                <div class="meta">
                    <span class="author">by John Doe</span>
                    <span class="date">October 24, 2025</span>
                </div>
                <p>This is a comprehensive guide to modern web development practices.</p>
                <p>Next.js provides server-side rendering out of the box, which means
                   the initial HTML contains all the content needed for extraction.</p>
                <p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do
                   eiusmod tempor incididunt ut labore et dolore magna aliqua.</p>
                <p>More substantial content continues here with detailed explanations
                   and code examples that would be valuable to extract.</p>
            </article>
        </div>
    </body>
    </html>"#;

    let engine = decide_engine_with_flags(nextjs_blog, "https://blog.example.com/post", flags);
    assert_eq!(
        engine,
        Engine::Wasm,
        "Next.js blog with SSR should attempt WASM first"
    );

    // Simulate successful WASM extraction
    let quality_score = 75;
    let word_count = 250;
    assert!(
        !should_escalate_to_headless(quality_score, word_count, nextjs_blog),
        "Good SSR content should not need escalation"
    );
}

#[test]
fn test_empty_react_shell_requires_escalation() {
    let mut flags = EngineSelectionFlags::default();
    flags.probe_first_spa = true;

    // Client-side only React app (no SSR)
    let react_shell = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>React App</title>
        <script src="/static/js/main.chunk.js"></script>
    </head>
    <body>
        <noscript>You need to enable JavaScript to run this app.</noscript>
        <div id="root"></div>
    </body>
    </html>"#;

    let engine = decide_engine_with_flags(react_shell, "https://app.example.com", flags);
    assert_eq!(
        engine,
        Engine::Wasm,
        "Empty shell still gets probe attempt"
    );

    // Simulate poor WASM extraction
    let quality_score = 15;
    let word_count = 25;
    assert!(
        should_escalate_to_headless(quality_score, word_count, react_shell),
        "Empty shell should escalate after poor extraction"
    );
}

#[test]
fn test_vue_ssr_nuxt_application() {
    let mut flags = EngineSelectionFlags::default();
    flags.probe_first_spa = true;

    let nuxt_html = r#"
    <!DOCTYPE html>
    <html data-n-head-ssr>
    <head>
        <script>window.__NUXT__ = {data: {}, state: {}}</script>
    </head>
    <body>
        <div id="__nuxt">
            <div id="__layout">
                <main>
                    <article class="article">
                        <h1>Nuxt.js Article Title</h1>
                        <p>Nuxt.js provides excellent server-side rendering capabilities.</p>
                        <p>This content is pre-rendered and available in the initial HTML.</p>
                        <p>WASM extraction should work well with this structure.</p>
                    </article>
                </main>
            </div>
        </div>
    </body>
    </html>"#;

    let engine = decide_engine_with_flags(nuxt_html, "https://nuxt-site.example.com", flags);
    assert_eq!(engine, Engine::Wasm, "Nuxt SSR should try WASM first");
}

#[test]
fn test_angular_universal_ssr() {
    let mut flags = EngineSelectionFlags::default();
    flags.probe_first_spa = true;

    let angular_ssr = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <base href="/">
    </head>
    <body>
        <app-root ng-version="15.0.0">
            <div class="content">
                <h1>Angular Universal SSR</h1>
                <p>This content is server-side rendered by Angular Universal.</p>
                <p>The HTML is fully populated before client-side hydration.</p>
            </div>
        </app-root>
    </body>
    </html>"#;

    let engine = decide_engine_with_flags(angular_ssr, "https://angular-app.example.com", flags);
    assert_eq!(
        engine,
        Engine::Wasm,
        "Angular Universal SSR should try WASM"
    );
}

// ============================================================================
// Feature Flag Validation Tests
// ============================================================================

#[test]
fn test_feature_flags_default_values() {
    let flags = EngineSelectionFlags::default();
    assert!(!flags.use_visible_text_density, "Should default to false");
    assert!(!flags.detect_placeholders, "Should default to false");
    assert!(!flags.probe_first_spa, "Should default to false for safety");
}

#[test]
fn test_feature_flags_can_be_enabled() {
    let mut flags = EngineSelectionFlags::default();

    flags.use_visible_text_density = true;
    flags.detect_placeholders = true;
    flags.probe_first_spa = true;

    assert!(flags.use_visible_text_density);
    assert!(flags.detect_placeholders);
    assert!(flags.probe_first_spa);
}

#[test]
fn test_gradual_rollout_scenario() {
    // Simulate a gradual rollout where probe_first_spa is enabled
    // but other optimizations remain disabled
    let mut flags = EngineSelectionFlags::default();
    flags.probe_first_spa = true;
    // Keep other flags disabled for conservative rollout
    assert!(!flags.use_visible_text_density);
    assert!(!flags.detect_placeholders);

    let spa_html = r#"<html><script>window.__NEXT_DATA__={}</script></html>"#;
    let engine = decide_engine_with_flags(spa_html, "https://example.com", flags);
    assert_eq!(engine, Engine::Wasm, "Probe-first enabled independently");
}
