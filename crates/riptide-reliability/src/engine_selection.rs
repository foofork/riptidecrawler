//! # Engine Selection Module
//!
//! Consolidated engine selection logic for the Riptide extraction system.
//! This module provides intelligent decision-making for choosing the optimal
//! extraction engine (Raw, Wasm, or Headless) based on content analysis.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use riptide_reliability::engine_selection::{Engine, decide_engine};
//!
//! let html = "<html>...</html>";
//! let url = "https://example.com";
//! let engine = decide_engine(html, url);
//!
//! match engine {
//!     Engine::Raw => {
//!         // Use basic HTTP fetch
//!     }
//!     Engine::Wasm => {
//!         // Use WASM-based extraction
//!     }
//!     Engine::Headless => {
//!         // Use headless browser
//!     }
//!     Engine::Auto => {
//!         // This should not happen after decision
//!         unreachable!();
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};

/// Extraction engine types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Engine {
    /// Automatically select engine based on content
    Auto,
    /// Pure HTTP fetch without JavaScript execution
    Raw,
    /// WASM-based extraction (fast, local)
    Wasm,
    /// Headless browser extraction (for JavaScript-heavy sites)
    Headless,
}

impl std::str::FromStr for Engine {
    type Err = anyhow::Error;

    /// Parse engine from string
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(Engine::Auto),
            "raw" => Ok(Engine::Raw),
            "wasm" => Ok(Engine::Wasm),
            "headless" => Ok(Engine::Headless),
            _ => anyhow::bail!(
                "Invalid engine: {}. Must be one of: auto, raw, wasm, headless",
                s
            ),
        }
    }
}

impl std::fmt::Display for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Engine {
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Engine::Auto => "auto",
            Engine::Raw => "raw",
            Engine::Wasm => "wasm",
            Engine::Headless => "headless",
        }
    }
}

/// Content analysis results for engine selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnalysis {
    /// Detected React/Next.js framework
    pub has_react: bool,
    /// Detected Vue framework
    pub has_vue: bool,
    /// Detected Angular framework
    pub has_angular: bool,
    /// Single Page Application markers detected
    pub has_spa_markers: bool,
    /// Anti-scraping protection detected
    pub has_anti_scraping: bool,
    /// Content-to-markup ratio (0.0 to 1.0)
    pub content_ratio: f64,
    /// Has main content tags (article, main, etc.)
    pub has_main_content: bool,
    /// Recommended engine based on analysis
    pub recommended_engine: Engine,
}

/// Automatically decide engine based on content characteristics
///
/// This function analyzes HTML content to determine the optimal extraction engine.
/// It checks for:
/// - JavaScript frameworks (React, Vue, Angular)
/// - Single Page Application (SPA) markers
/// - Anti-scraping measures (Cloudflare, reCAPTCHA)
/// - Content-to-markup ratio
/// - Main content structure
///
/// # Arguments
///
/// * `html` - The HTML content to analyze
/// * `url` - The source URL (used for additional context)
///
/// # Returns
///
/// The recommended `Engine` for extracting content from this page.
///
/// # Examples
///
/// ```
/// use riptide_reliability::engine_selection::{Engine, decide_engine};
///
/// // React application
/// let react_html = r#"<html><script>window.__NEXT_DATA__={}</script></html>"#;
/// assert_eq!(decide_engine(react_html, "https://example.com"), Engine::Headless);
///
/// // Standard article
/// let article_html = r#"<html><body><article><p>Great content here!</p></article></body></html>"#;
/// assert_eq!(decide_engine(article_html, "https://example.com"), Engine::Wasm);
/// ```
pub fn decide_engine(html: &str, _url: &str) -> Engine {
    let html_lower = html.to_lowercase();

    // Check for heavy JavaScript frameworks (case-insensitive for better detection)
    let has_react = html_lower.contains("__next_data__")
        || html_lower.contains("_reactroot")
        || html_lower.contains("data-reactroot")
        || html_lower.contains("__webpack_require__");

    let has_vue = html_lower.contains("v-app")
        || html_lower.contains("createapp(")
        || html_lower.contains("data-vue-app");

    let has_angular = html_lower.contains("ng-app")
        || html_lower.contains("ng-version")
        || html_lower.contains("platformbrowserdynamic")
        || html_lower.contains("[ngclass]");

    // Check for SPA (Single Page Application) markers
    let has_spa_markers = html_lower.contains("<!-- rendered by")
        || html_lower.contains("__webpack")
        || html_lower.contains("window.__initial_state__")
        || html_lower.contains("data-react-helmet");

    // Check for anti-scraping measures (case-insensitive)
    let has_anti_scraping = html_lower.contains("cloudflare")
        || html_lower.contains("cf-browser-verification")
        || html_lower.contains("grecaptcha")
        || html_lower.contains("hcaptcha")
        || html_lower.contains("perimeterx");

    // Calculate content-to-markup ratio
    let content_ratio = calculate_content_ratio(html);

    // Analyze content structure (used for determination logic)
    let _has_main_content = html_lower.contains("<article")
        || html_lower.contains("class=\"content\"")
        || html_lower.contains("id=\"content\"")
        || html_lower.contains("<main");

    // Decision logic with clear priority ordering
    if has_anti_scraping {
        // Priority 1: Anti-scraping protection always requires headless
        Engine::Headless
    } else if has_react || has_vue || has_angular || has_spa_markers {
        // Priority 2: JavaScript frameworks typically require headless
        Engine::Headless
    } else if content_ratio < 0.1 {
        // Priority 3: Low content ratio suggests client-side rendering
        Engine::Headless
    } else {
        // Default: WASM for standard HTML extraction (including WASM content)
        // Note: Even with WASM markers, we prefer WASM engine by default
        Engine::Wasm
    }
}

/// Analyze content characteristics to determine optimal engine (with detailed results)
///
/// This function provides more detailed analysis results than `decide_engine()`,
/// including all the detected features and ratios.
///
/// # Arguments
///
/// * `html` - The HTML content to analyze
/// * `url` - The source URL (used for additional context)
///
/// # Returns
///
/// A `ContentAnalysis` struct containing detailed analysis results.
///
/// # Examples
///
/// ```
/// use riptide_reliability::engine_selection::{Engine, analyze_content};
///
/// let html = r#"<html><head><script>window.__NEXT_DATA__={}</script></head>
///                <body><article>Content</article></body></html>"#;
/// let analysis = analyze_content(html, "https://example.com");
///
/// assert!(analysis.has_react);
/// assert!(analysis.has_main_content);
/// assert_eq!(analysis.recommended_engine, Engine::Headless);
/// ```
pub fn analyze_content(html: &str, url: &str) -> ContentAnalysis {
    let html_lower = html.to_lowercase();

    // Check for heavy JavaScript frameworks (case-insensitive for better detection)
    let has_react = html_lower.contains("__next_data__")
        || html_lower.contains("_reactroot")
        || html_lower.contains("data-reactroot")
        || html_lower.contains("__webpack_require__");

    let has_vue = html_lower.contains("v-app")
        || html_lower.contains("createapp(")
        || html_lower.contains("data-vue-app");

    let has_angular = html_lower.contains("ng-app")
        || html_lower.contains("ng-version")
        || html_lower.contains("platformbrowserdynamic")
        || html_lower.contains("[ngclass]");

    // Check for SPA markers
    let has_spa_markers = html_lower.contains("<!-- rendered by")
        || html_lower.contains("__webpack")
        || html_lower.contains("window.__initial_state__")
        || html_lower.contains("data-react-helmet");

    // Check for anti-scraping measures (case-insensitive)
    let has_anti_scraping = html_lower.contains("cloudflare")
        || html_lower.contains("cf-browser-verification")
        || html_lower.contains("grecaptcha")
        || html_lower.contains("hcaptcha")
        || html_lower.contains("perimeterx");

    // Calculate content-to-markup ratio
    let content_ratio = calculate_content_ratio(html);

    // Analyze content structure
    let has_main_content = html_lower.contains("<article")
        || html_lower.contains("class=\"content\"")
        || html_lower.contains("id=\"content\"")
        || html_lower.contains("<main");

    // Determine recommended engine based on analysis with clear priority
    let recommended_engine = if has_anti_scraping {
        // Priority 1: Anti-scraping always needs headless
        Engine::Headless
    } else if has_react || has_vue || has_angular || has_spa_markers {
        // Priority 2: JavaScript frameworks need headless
        Engine::Headless
    } else if content_ratio < 0.1 {
        // Priority 3: Low content ratio suggests client-side rendering
        Engine::Headless
    } else if html.contains("wasm") || url.contains(".wasm") {
        // Priority 4: WASM content
        Engine::Wasm
    } else if has_main_content && content_ratio > 0.2 {
        // Priority 5: Good structured content
        Engine::Wasm
    } else {
        // Default: WASM for standard extraction
        Engine::Wasm
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

/// Calculate content-to-markup ratio (heuristic for client-side rendering)
///
/// This function estimates how much of the HTML is actual text content
/// versus markup tags. A low ratio suggests the page relies heavily on
/// client-side rendering.
///
/// # Arguments
///
/// * `html` - The HTML content to analyze
///
/// # Returns
///
/// A ratio between 0.0 and 1.0, where higher values indicate more text content.
pub fn calculate_content_ratio(html: &str) -> f64 {
    let total_len = html.len() as f64;
    if total_len == 0.0 {
        return 0.0;
    }

    // Count text content (rough estimate)
    // Extract text between tags
    let text_content: String = html
        .split('<')
        .filter_map(|s| s.split('>').nth(1))
        .collect();

    let content_len = text_content.trim().len() as f64;
    content_len / total_len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_from_str() {
        assert_eq!("auto".parse::<Engine>().unwrap(), Engine::Auto);
        assert_eq!("raw".parse::<Engine>().unwrap(), Engine::Raw);
        assert_eq!("wasm".parse::<Engine>().unwrap(), Engine::Wasm);
        assert_eq!("headless".parse::<Engine>().unwrap(), Engine::Headless);
        assert!("invalid".parse::<Engine>().is_err());
    }

    #[test]
    fn test_engine_name() {
        assert_eq!(Engine::Auto.name(), "auto");
        assert_eq!(Engine::Raw.name(), "raw");
        assert_eq!(Engine::Wasm.name(), "wasm");
        assert_eq!(Engine::Headless.name(), "headless");
    }

    #[test]
    fn test_engine_display() {
        assert_eq!(format!("{}", Engine::Auto), "auto");
        assert_eq!(format!("{}", Engine::Wasm), "wasm");
    }

    #[test]
    fn test_content_ratio_calculation() {
        let html = "<html><body>Hello World</body></html>";
        let ratio = calculate_content_ratio(html);
        assert!(ratio > 0.0 && ratio < 1.0);
    }

    #[test]
    fn test_empty_html_ratio() {
        let html = "";
        let ratio = calculate_content_ratio(html);
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn test_spa_detection() {
        let html = r#"<html><head><script>window.__INITIAL_STATE__={}</script></head></html>"#;
        let engine = decide_engine(html, "https://example.com");
        assert_eq!(engine, Engine::Headless);
    }

    #[test]
    fn test_react_detection() {
        let html = r#"<html><head><script>window.__NEXT_DATA__={}</script></head></html>"#;
        let analysis = analyze_content(html, "https://example.com");
        assert!(analysis.has_react);
        assert_eq!(analysis.recommended_engine, Engine::Headless);
    }

    #[test]
    fn test_vue_detection() {
        let html = r#"<html><body><div id="app" v-app>Test</div></body></html>"#;
        let analysis = analyze_content(html, "https://example.com");
        assert!(analysis.has_vue);
        assert_eq!(analysis.recommended_engine, Engine::Headless);
    }

    #[test]
    fn test_angular_detection() {
        let html = r#"<html><body ng-app="myApp">Test</body></html>"#;
        let analysis = analyze_content(html, "https://example.com");
        assert!(analysis.has_angular);
        assert_eq!(analysis.recommended_engine, Engine::Headless);
    }

    #[test]
    fn test_anti_scraping_detection() {
        let html = r#"<html><body>Please enable JavaScript. Cloudflare protection.</body></html>"#;
        let engine = decide_engine(html, "https://example.com");
        assert_eq!(engine, Engine::Headless);
    }

    #[test]
    fn test_standard_html_detection() {
        let html = r#"<html><body><article>Hello World with good content ratio for extraction. This is a longer piece of text that should be sufficient for proper content ratio calculation.</article></body></html>"#;
        let analysis = analyze_content(html, "https://example.com");
        assert!(analysis.has_main_content);
        assert_eq!(analysis.recommended_engine, Engine::Wasm);
    }

    #[test]
    fn test_low_content_ratio() {
        let html = r#"<html><head><script src="app.js"></script><link rel="stylesheet" href="style.css"></head><body><div id="root"></div></body></html>"#;
        let engine = decide_engine(html, "https://example.com");
        assert_eq!(engine, Engine::Headless);
    }

    #[test]
    fn test_wasm_content_detection() {
        // HTML with WASM reference but low content ratio triggers Headless
        // because content ratio is the determining factor
        let html = r#"<html><head><script src="app.wasm"></script></head><body>Test</body></html>"#;
        let engine = decide_engine(html, "https://example.com");
        // Low content ratio (<0.1) triggers Headless engine despite WASM presence
        assert_eq!(engine, Engine::Headless);

        // HTML with WASM and good content ratio prefers WASM
        let html_with_content = r#"<html><head><script src="app.wasm"></script></head>
            <body><article>This is substantial content with enough text to pass the content ratio
            threshold for WASM engine selection. The content ratio calculation will show this has
            good text-to-markup ratio.</article></body></html>"#;
        let engine_with_content = decide_engine(html_with_content, "https://example.com");
        assert_eq!(engine_with_content, Engine::Wasm);
    }

    #[test]
    fn test_detailed_analysis() {
        let html = r#"<html><head><script>window.__NEXT_DATA__={}</script></head><body><article>Content</article></body></html>"#;
        let analysis = analyze_content(html, "https://example.com");

        assert!(analysis.has_react);
        assert!(!analysis.has_vue);
        assert!(!analysis.has_angular);
        assert!(!analysis.has_anti_scraping);
        assert!(analysis.has_main_content);
        assert!(analysis.content_ratio > 0.0);
        assert_eq!(analysis.recommended_engine, Engine::Headless);
    }
}
