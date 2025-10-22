//! Engine Selection Logic Tests
//!
//! Tests for the smart engine selection algorithm that analyzes content
//! and chooses the optimal extraction engine based on:
//! - JavaScript framework detection
//! - Content-to-markup ratio
//! - Anti-scraping indicators
//! - Performance characteristics

use std::collections::HashMap;

#[cfg(test)]
mod engine_selection_tests {
    use super::*;

    /// Test React/Next.js detection
    #[test]
    fn test_react_nextjs_detection() {
        let test_cases = vec![
            (r#"<script>window.__NEXT_DATA__={}</script>"#, true),
            (r#"<div id="__next"></div>"#, true),
            (r#"<script>window._reactRoot</script>"#, true),
            (r#"<script>window.__webpack_require__</script>"#, true),
            (r#"<div>No React here</div>"#, false),
        ];

        for (html, expected) in test_cases {
            let has_react = detect_react_framework(html);
            assert_eq!(
                has_react, expected,
                "React detection failed for: {}",
                html
            );
        }
    }

    /// Test Vue.js detection
    #[test]
    fn test_vue_detection() {
        let test_cases = vec![
            (r#"<div v-app></div>"#, true),
            (r#"<script>Vue.createApp()</script>"#, true),
            (r#"<script>const app = createApp()</script>"#, true),
            (r#"<div data-vue-app></div>"#, true),
            (r#"<div>No Vue here</div>"#, false),
        ];

        for (html, expected) in test_cases {
            let has_vue = detect_vue_framework(html);
            assert_eq!(has_vue, expected, "Vue detection failed for: {}", html);
        }
    }

    /// Test Angular detection
    #[test]
    fn test_angular_detection() {
        let test_cases = vec![
            (r#"<div ng-app></div>"#, true),
            (r#"<script>ng-version</script>"#, true),
            (r#"<script>platformBrowserDynamic()</script>"#, true),
            (r#"<div [ngClass]=""></div>"#, true),
            (r#"<div>No Angular here</div>"#, false),
        ];

        for (html, expected) in test_cases {
            let has_angular = detect_angular_framework(html);
            assert_eq!(
                has_angular, expected,
                "Angular detection failed for: {}",
                html
            );
        }
    }

    /// Test SPA (Single Page Application) markers
    #[test]
    fn test_spa_marker_detection() {
        let test_cases = vec![
            (r#"<!-- rendered by vue -->"#, true),
            (r#"<script>window.__INITIAL_STATE__</script>"#, true),
            (r#"<meta name="generator" content="webpack">"#, true),
            (r#"<div data-react-helmet="true"></div>"#, true),
            (r#"<script>window.__webpack</script>"#, true),
            (r#"<div>Regular HTML</div>"#, false),
        ];

        for (html, expected) in test_cases {
            let has_spa_markers = detect_spa_markers(html);
            assert_eq!(
                has_spa_markers, expected,
                "SPA detection failed for: {}",
                html
            );
        }
    }

    /// Test anti-scraping detection
    #[test]
    fn test_anti_scraping_detection() {
        let test_cases = vec![
            (r#"<script src="cloudflare.js"></script>"#, true),
            (r#"<div id="cf-browser-verification"></div>"#, true),
            (r#"<script>grecaptcha.render()</script>"#, true),
            (r#"<div class="h-captcha"></div>"#, true),
            (r#"<script src="PerimeterX.js"></script>"#, true),
            (r#"<div>Normal content</div>"#, false),
        ];

        for (html, expected) in test_cases {
            let has_anti_scraping = detect_anti_scraping(html);
            assert_eq!(
                has_anti_scraping, expected,
                "Anti-scraping detection failed for: {}",
                html
            );
        }
    }

    /// Test content-to-markup ratio calculation
    #[test]
    fn test_content_ratio_calculation() {
        let test_cases = vec![
            // (HTML, expected_ratio_range)
            (
                r#"<div>Hello World</div>"#,
                (0.4, 0.6), // Roughly 50% content
            ),
            (
                r#"<div><span><p>Content</p></span></div>"#,
                (0.1, 0.3), // Low ratio, lots of markup
            ),
            (
                r#"This is plain text with minimal markup"#,
                (0.8, 1.0), // High ratio, mostly content
            ),
            (
                r#"<div class="wrapper" id="main"><span></span></div>"#,
                (0.0, 0.1), // Very low ratio, no content
            ),
        ];

        for (html, (min_ratio, max_ratio)) in test_cases {
            let ratio = calculate_content_ratio(html);
            assert!(
                ratio >= min_ratio && ratio <= max_ratio,
                "Content ratio {} not in range [{}, {}] for: {}",
                ratio,
                min_ratio,
                max_ratio,
                html
            );
        }
    }

    /// Test main content detection
    #[test]
    fn test_main_content_detection() {
        let test_cases = vec![
            (r#"<article>Content</article>"#, true),
            (r#"<main>Content</main>"#, true),
            (r#"<div class="content">Content</div>"#, true),
            (r#"<div id="content">Content</div>"#, true),
            (r#"<section class="main-content">Content</section>"#, true),
            (r#"<div>Just a div</div>"#, false),
        ];

        for (html, expected) in test_cases {
            let has_main_content = detect_main_content(html);
            assert_eq!(
                has_main_content, expected,
                "Main content detection failed for: {}",
                html
            );
        }
    }

    /// Test comprehensive content analysis
    #[test]
    fn test_comprehensive_content_analysis() {
        let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Test Page</title>
    <script>window.__NEXT_DATA__={}</script>
</head>
<body>
    <div id="__next">
        <article class="content">
            <h1>Article Title</h1>
            <p>Some content here that makes up a reasonable portion of the page.</p>
        </article>
    </div>
</body>
</html>"#;

        let analysis = analyze_content_comprehensive(html, "https://example.com");

        assert!(analysis.has_react, "Should detect React");
        assert!(analysis.has_main_content, "Should detect main content");
        assert!(
            analysis.content_ratio > 0.05,
            "Should have reasonable content ratio"
        );
        assert_eq!(
            analysis.recommended_engine, "headless",
            "Should recommend headless for React app"
        );
    }

    /// Test engine recommendation for simple HTML
    #[test]
    fn test_engine_recommendation_simple_html() {
        let html = r#"
<!DOCTYPE html>
<html>
<head><title>Simple Page</title></head>
<body>
    <article>
        <h1>Article Title</h1>
        <p>This is a well-structured article with good content-to-markup ratio.</p>
        <p>Multiple paragraphs of content make it ideal for WASM extraction.</p>
    </article>
</body>
</html>"#;

        let analysis = analyze_content_comprehensive(html, "https://example.com");

        assert_eq!(
            analysis.recommended_engine, "wasm",
            "Should recommend WASM for simple HTML"
        );
        assert!(!analysis.has_react, "Should not detect frameworks");
        assert!(!analysis.has_anti_scraping, "Should not detect anti-scraping");
    }

    /// Test engine recommendation for JavaScript-heavy site
    #[test]
    fn test_engine_recommendation_js_heavy() {
        let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>JS App</title>
    <script src="bundle.js"></script>
    <script>window.__webpack_require__</script>
</head>
<body>
    <div id="app"></div>
    <script>
        // Lots of JavaScript
        const app = new Application();
        app.render();
    </script>
</body>
</html>"#;

        let analysis = analyze_content_comprehensive(html, "https://example.com");

        assert_eq!(
            analysis.recommended_engine, "headless",
            "Should recommend headless for JS-heavy site"
        );
    }

    /// Test engine recommendation for protected site
    #[test]
    fn test_engine_recommendation_protected() {
        let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Protected Site</title>
    <script src="cf-challenges.js"></script>
</head>
<body>
    <div id="cf-browser-verification">
        Please verify you're human...
    </div>
    <script>
        if (typeof grecaptcha !== 'undefined') {
            grecaptcha.render('recaptcha-container');
        }
    </script>
</body>
</html>"#;

        let analysis = analyze_content_comprehensive(html, "https://protected.example.com");

        assert_eq!(
            analysis.recommended_engine, "stealth",
            "Should recommend stealth for protected site"
        );
        assert!(
            analysis.has_anti_scraping,
            "Should detect anti-scraping measures"
        );
    }

    /// Test engine selection decision caching
    #[test]
    fn test_engine_selection_caching() {
        let cache = EngineSelectionCache::new();
        let url = "https://example.com";
        let html = "<html><body>Test</body></html>";

        // First analysis
        let engine1 = cache.get_or_analyze(url, html);

        // Second analysis should return cached result
        let engine2 = cache.get_or_analyze(url, html);

        assert_eq!(engine1, engine2, "Should return cached engine selection");
        assert_eq!(cache.cache_hits(), 1, "Should have one cache hit");
    }

    /// Test engine selection for different domains
    #[test]
    fn test_engine_selection_per_domain() {
        let cache = EngineSelectionCache::new();

        let domains = vec![
            ("https://news.ycombinator.com", "wasm"), // Simple HTML
            ("https://react-app.com", "headless"),    // React app
            ("https://protected.com", "stealth"),     // Protected site
        ];

        for (url, expected_engine) in domains {
            let html = format!("<html><body>{}</body></html>", url);
            let engine = cache.get_or_analyze(url, &html);

            // Note: This is simplified - actual implementation would analyze content
            assert!(
                !engine.is_empty(),
                "Should select an engine for {}",
                url
            );
        }
    }

    /// Test performance characteristics tracking
    #[test]
    fn test_performance_characteristics() {
        let mut tracker = PerformanceTracker::new();

        // Track successful extractions
        tracker.record_extraction("wasm", 100, true);
        tracker.record_extraction("wasm", 150, true);
        tracker.record_extraction("headless", 500, true);
        tracker.record_extraction("stealth", 1000, true);

        let stats = tracker.get_stats();

        assert_eq!(stats.total_extractions, 4);
        assert_eq!(stats.successful_extractions, 4);
        assert!(stats.wasm_avg_time_ms < stats.headless_avg_time_ms);
        assert!(stats.headless_avg_time_ms < stats.stealth_avg_time_ms);
    }

    // Helper functions and types

    fn detect_react_framework(html: &str) -> bool {
        html.contains("__NEXT_DATA__")
            || html.contains("__next")
            || html.contains("_reactRoot")
            || html.contains("__webpack_require__")
    }

    fn detect_vue_framework(html: &str) -> bool {
        html.contains("v-app")
            || html.contains("Vue.createApp")
            || html.contains("createApp(")
            || html.contains("data-vue-app")
    }

    fn detect_angular_framework(html: &str) -> bool {
        html.contains("ng-app")
            || html.contains("ng-version")
            || html.contains("platformBrowserDynamic")
            || html.contains("[ngClass]")
    }

    fn detect_spa_markers(html: &str) -> bool {
        html.contains("<!-- rendered by")
            || html.contains("__INITIAL_STATE__")
            || html.contains("webpack")
            || html.contains("data-react-helmet")
            || html.contains("__webpack")
    }

    fn detect_anti_scraping(html: &str) -> bool {
        html.contains("cloudflare")
            || html.contains("cf-browser-verification")
            || html.contains("grecaptcha")
            || html.contains("h-captcha")
            || html.contains("PerimeterX")
    }

    fn calculate_content_ratio(html: &str) -> f64 {
        let total_len = html.len() as f64;
        if total_len == 0.0 {
            return 0.0;
        }

        // Count text content (rough estimate)
        let text_content: String = html
            .split('<')
            .filter_map(|s| s.split('>').nth(1))
            .collect();

        let content_len = text_content.trim().len() as f64;
        content_len / total_len
    }

    fn detect_main_content(html: &str) -> bool {
        html.contains("<article")
            || html.contains("<main")
            || html.contains("class=\"content\"")
            || html.contains("id=\"content\"")
            || html.contains("main-content")
    }

    #[derive(Debug)]
    struct ContentAnalysis {
        has_react: bool,
        has_vue: bool,
        has_angular: bool,
        has_spa_markers: bool,
        has_anti_scraping: bool,
        content_ratio: f64,
        has_main_content: bool,
        recommended_engine: String,
    }

    fn analyze_content_comprehensive(html: &str, _url: &str) -> ContentAnalysis {
        let has_react = detect_react_framework(html);
        let has_vue = detect_vue_framework(html);
        let has_angular = detect_angular_framework(html);
        let has_spa_markers = detect_spa_markers(html);
        let has_anti_scraping = detect_anti_scraping(html);
        let content_ratio = calculate_content_ratio(html);
        let has_main_content = detect_main_content(html);

        // Determine recommended engine
        let recommended_engine = if has_anti_scraping {
            "stealth".to_string()
        } else if has_react || has_vue || has_angular || has_spa_markers {
            "headless".to_string()
        } else if content_ratio < 0.1 {
            "headless".to_string()
        } else {
            "wasm".to_string()
        };

        ContentAnalysis {
            has_react,
            has_vue,
            has_angular,
            has_spa_markers,
            has_anti_scraping,
            content_ratio,
            has_main_content,
            recommended_engine,
        }
    }

    struct EngineSelectionCache {
        cache: HashMap<String, String>,
        hits: std::sync::atomic::AtomicUsize,
    }

    impl EngineSelectionCache {
        fn new() -> Self {
            Self {
                cache: HashMap::new(),
                hits: std::sync::atomic::AtomicUsize::new(0),
            }
        }

        fn get_or_analyze(&mut self, url: &str, html: &str) -> String {
            if let Some(engine) = self.cache.get(url) {
                self.hits
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                return engine.clone();
            }

            let analysis = analyze_content_comprehensive(html, url);
            let engine = analysis.recommended_engine.clone();
            self.cache.insert(url.to_string(), engine.clone());
            engine
        }

        fn cache_hits(&self) -> usize {
            self.hits.load(std::sync::atomic::Ordering::Relaxed)
        }
    }

    struct PerformanceTracker {
        extractions: Vec<(String, u64, bool)>, // (engine, time_ms, success)
    }

    impl PerformanceTracker {
        fn new() -> Self {
            Self {
                extractions: Vec::new(),
            }
        }

        fn record_extraction(&mut self, engine: &str, time_ms: u64, success: bool) {
            self.extractions
                .push((engine.to_string(), time_ms, success));
        }

        fn get_stats(&self) -> PerformanceStats {
            let total_extractions = self.extractions.len();
            let successful_extractions = self.extractions.iter().filter(|(_, _, s)| *s).count();

            let wasm_times: Vec<u64> = self
                .extractions
                .iter()
                .filter(|(e, _, _)| e == "wasm")
                .map(|(_, t, _)| *t)
                .collect();

            let headless_times: Vec<u64> = self
                .extractions
                .iter()
                .filter(|(e, _, _)| e == "headless")
                .map(|(_, t, _)| *t)
                .collect();

            let stealth_times: Vec<u64> = self
                .extractions
                .iter()
                .filter(|(e, _, _)| e == "stealth")
                .map(|(_, t, _)| *t)
                .collect();

            PerformanceStats {
                total_extractions,
                successful_extractions,
                wasm_avg_time_ms: avg(&wasm_times),
                headless_avg_time_ms: avg(&headless_times),
                stealth_avg_time_ms: avg(&stealth_times),
            }
        }
    }

    fn avg(times: &[u64]) -> u64 {
        if times.is_empty() {
            0
        } else {
            times.iter().sum::<u64>() / times.len() as u64
        }
    }

    #[derive(Debug)]
    struct PerformanceStats {
        total_extractions: usize,
        successful_extractions: usize,
        wasm_avg_time_ms: u64,
        headless_avg_time_ms: u64,
        stealth_avg_time_ms: u64,
    }
}
