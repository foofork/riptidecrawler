//! Content analysis intelligence service
//!
//! Analyzes URLs and content to determine rendering strategies and dynamic content detection.

use riptide_headless::dynamic::{
    DynamicConfig, ScrollConfig, ScrollMode, ViewportConfig, WaitCondition,
};
use std::time::Duration;
use tracing::debug;

/// Analyzes content and URLs for rendering strategies
pub struct ContentAnalyzer;

impl ContentAnalyzer {
    /// Create a new content analyzer
    pub fn new() -> Self {
        Self
    }

    /// Analyze URL to determine if dynamic rendering is needed
    pub async fn analyze_url_for_dynamic_content(&self, url: &str) -> bool {
        let url_lower = url.to_lowercase();

        // Social media platforms and news sites with dynamic content
        let dynamic_domains = [
            "twitter.com",
            "x.com",
            "facebook.com",
            "instagram.com",
            "linkedin.com",
            "youtube.com",
            "tiktok.com",
            "reddit.com",
            "medium.com",
            "substack.com",
            "github.com",
            "stackoverflow.com",
            "discord.com",
            "slack.com",
            "notion.so",
            "airtable.com",
            "figma.com",
            "miro.com",
            "shopify.com",
            "woocommerce.com",
            "squarespace.com",
            "webflow.com",
        ];

        // Check if URL contains dynamic domain patterns
        for domain in &dynamic_domains {
            if url_lower.contains(domain) {
                debug!(url = %url, domain = %domain, "Found dynamic domain pattern");
                return true;
            }
        }

        // Check for SPA indicators in URL
        let spa_indicators = [
            "/#/",
            "#!/",
            "/app/",
            "/dashboard/",
            "/admin/",
            "?page=",
            "&view=",
            "#page",
            "#view",
            "#section",
        ];

        for indicator in &spa_indicators {
            if url_lower.contains(indicator) {
                debug!(url = %url, indicator = %indicator, "Found SPA URL pattern");
                return true;
            }
        }

        // Check for JavaScript framework patterns
        let js_frameworks = [
            "react",
            "angular",
            "vue",
            "svelte",
            "next",
            "nuxt",
            "gatsby",
            "webpack",
            "vite",
            "parcel",
            "app.js",
            "bundle.js",
            "main.js",
        ];

        for framework in &js_frameworks {
            if url_lower.contains(framework) {
                debug!(url = %url, framework = %framework, "Found JS framework pattern");
                return true;
            }
        }

        debug!(url = %url, "No dynamic content indicators found");
        false
    }

    /// Create adaptive dynamic configuration based on URL analysis
    pub fn create_adaptive_dynamic_config(&self, url: &str) -> DynamicConfig {
        let url_lower = url.to_lowercase();

        // Determine wait strategy based on URL type
        let wait_for = self.select_wait_strategy(&url_lower);

        // Determine scroll strategy
        let scroll = self.select_scroll_strategy(&url_lower);

        // Create viewport configuration
        let viewport = Some(ViewportConfig {
            width: 1920,
            height: 1080,
            device_scale_factor: 1.0,
            is_mobile: false,
            user_agent: None,
        });

        DynamicConfig {
            wait_for,
            scroll,
            actions: Vec::new(),
            capture_artifacts: false,
            timeout: Duration::from_secs(3),
            viewport,
        }
    }

    /// Select appropriate wait strategy for URL
    fn select_wait_strategy(&self, url_lower: &str) -> Option<WaitCondition> {
        if url_lower.contains("github.com") {
            Some(WaitCondition::Selector {
                selector: ".repository-content, .file-navigation, .js-repo-nav".to_string(),
                timeout: Duration::from_secs(2),
            })
        } else if url_lower.contains("reddit.com") {
            Some(WaitCondition::Selector {
                selector: "[data-testid='post'], .Post".to_string(),
                timeout: Duration::from_secs(2),
            })
        } else if url_lower.contains("medium.com") || url_lower.contains("substack.com") {
            Some(WaitCondition::Selector {
                selector: "article, .post-content, main".to_string(),
                timeout: Duration::from_secs(2),
            })
        } else if url_lower.contains("twitter.com") || url_lower.contains("x.com") {
            Some(WaitCondition::Multiple(vec![
                WaitCondition::Selector {
                    selector: "[data-testid='tweet'], article".to_string(),
                    timeout: Duration::from_millis(1500),
                },
                WaitCondition::NetworkIdle {
                    timeout: Duration::from_millis(1000),
                    idle_time: Duration::from_millis(500),
                },
            ]))
        } else {
            Some(WaitCondition::Multiple(vec![
                WaitCondition::DomContentLoaded,
                WaitCondition::Timeout(Duration::from_millis(1000)),
            ]))
        }
    }

    /// Select appropriate scroll strategy for URL
    fn select_scroll_strategy(&self, url_lower: &str) -> Option<ScrollConfig> {
        if url_lower.contains("twitter.com")
            || url_lower.contains("x.com")
            || url_lower.contains("instagram.com")
            || url_lower.contains("linkedin.com")
        {
            // Social media needs more scrolling for infinite feeds
            Some(ScrollConfig {
                steps: 5,
                step_px: Some(800),
                delay_ms: 800,
                mode: ScrollMode::Stepped,
                after_scroll_js: Some(
                    "window.scrollBy(0, 200); await new Promise(r => setTimeout(r, 300));"
                        .to_string(),
                ),
                stop_condition: None,
            })
        } else if url_lower.contains("medium.com") || url_lower.contains("substack.com") {
            // Article sites need gentle scrolling
            Some(ScrollConfig {
                steps: 3,
                step_px: Some(1000),
                delay_ms: 500,
                mode: ScrollMode::Smooth,
                after_scroll_js: None,
                stop_condition: None,
            })
        } else {
            // Default moderate scrolling
            Some(ScrollConfig {
                steps: 2,
                step_px: Some(800),
                delay_ms: 600,
                mode: ScrollMode::Stepped,
                after_scroll_js: None,
                stop_condition: None,
            })
        }
    }
}

impl Default for ContentAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyze_url_for_dynamic_content_twitter() {
        let analyzer = ContentAnalyzer::new();
        assert!(
            analyzer
                .analyze_url_for_dynamic_content("https://twitter.com/user/status/123")
                .await
        );
    }

    #[tokio::test]
    async fn test_analyze_url_for_dynamic_content_github() {
        let analyzer = ContentAnalyzer::new();
        assert!(
            analyzer
                .analyze_url_for_dynamic_content("https://github.com/org/repo")
                .await
        );
    }

    #[tokio::test]
    async fn test_analyze_url_for_dynamic_content_spa() {
        let analyzer = ContentAnalyzer::new();
        assert!(
            analyzer
                .analyze_url_for_dynamic_content("https://example.com/#/dashboard")
                .await
        );
    }

    #[tokio::test]
    async fn test_analyze_url_for_dynamic_content_static() {
        let analyzer = ContentAnalyzer::new();
        assert!(
            !analyzer
                .analyze_url_for_dynamic_content("https://example.com/blog/article.html")
                .await
        );
    }

    #[test]
    fn test_create_adaptive_dynamic_config_github() {
        let analyzer = ContentAnalyzer::new();
        let config = analyzer.create_adaptive_dynamic_config("https://github.com/rust-lang/rust");

        assert!(config.wait_for.is_some());
        assert!(config.scroll.is_some());
        assert_eq!(config.timeout, Duration::from_secs(3));

        if let Some(viewport) = config.viewport {
            assert_eq!(viewport.width, 1920);
            assert_eq!(viewport.height, 1080);
            assert!(!viewport.is_mobile);
        }
    }

    #[test]
    fn test_create_adaptive_dynamic_config_twitter() {
        let analyzer = ContentAnalyzer::new();
        let config = analyzer.create_adaptive_dynamic_config("https://twitter.com/user/status/123");

        if let Some(scroll) = config.scroll {
            assert_eq!(scroll.steps, 5);
            assert_eq!(scroll.step_px, Some(800));
            assert!(scroll.after_scroll_js.is_some());
        }
    }

    #[test]
    fn test_create_adaptive_dynamic_config_medium() {
        let analyzer = ContentAnalyzer::new();
        let config = analyzer.create_adaptive_dynamic_config("https://medium.com/@user/article");

        if let Some(scroll) = config.scroll {
            assert_eq!(scroll.steps, 3);
            assert!(matches!(scroll.mode, ScrollMode::Smooth));
            assert!(scroll.after_scroll_js.is_none());
        }
    }
}
