use riptide_core::dynamic::{DynamicConfig, ScrollConfig, ScrollMode, ViewportConfig, WaitCondition};
use std::time::Duration;
use tracing::debug;

/// Analyze URL and content patterns to determine if dynamic rendering is needed
pub(super) async fn analyze_url_for_dynamic_content(url: &str) -> bool {
    // Check for common indicators that suggest dynamic content
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

    // Default to static for unknown patterns
    debug!(url = %url, "No dynamic content indicators found");
    false
}

/// Create adaptive dynamic configuration based on URL analysis
pub(super) fn create_adaptive_dynamic_config(url: &str) -> DynamicConfig {
    let url_lower = url.to_lowercase();

    // Determine wait strategy based on URL type
    let wait_for = if url_lower.contains("github.com") {
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
        // Generic wait for content
        Some(WaitCondition::Multiple(vec![
            WaitCondition::DomContentLoaded,
            WaitCondition::Timeout(Duration::from_millis(1000)),
        ]))
    };

    // Determine scroll strategy
    let scroll = if url_lower.contains("twitter.com")
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
                "window.scrollBy(0, 200); await new Promise(r => setTimeout(r, 300));".to_string(),
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
    };

    // Create viewport configuration
    let viewport = Some(ViewportConfig {
        width: 1920,
        height: 1080,
        device_scale_factor: 1.0,
        is_mobile: false,
        user_agent: None, // Let stealth controller handle this
    });

    DynamicConfig {
        wait_for,
        scroll,
        actions: Vec::new(),             // No custom actions for adaptive mode
        capture_artifacts: false,        // Controlled by request parameter
        timeout: Duration::from_secs(3), // Hard cap requirement
        viewport,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyze_url_for_dynamic_content() {
        // Test dynamic domains
        assert!(
            analyze_url_for_dynamic_content("https://twitter.com/user/status/123").await,
            "Twitter should require dynamic rendering"
        );
        assert!(
            analyze_url_for_dynamic_content("https://github.com/org/repo").await,
            "GitHub should require dynamic rendering"
        );
        assert!(
            analyze_url_for_dynamic_content("https://reddit.com/r/rust").await,
            "Reddit should require dynamic rendering"
        );

        // Test SPA indicators
        assert!(
            analyze_url_for_dynamic_content("https://example.com/#/dashboard").await,
            "SPA hash routing should require dynamic rendering"
        );
        assert!(
            analyze_url_for_dynamic_content("https://example.com/app/settings").await,
            "App path should require dynamic rendering"
        );

        // Test static content
        assert!(
            !analyze_url_for_dynamic_content("https://example.com/blog/article.html").await,
            "Static HTML should not require dynamic rendering"
        );
        assert!(
            !analyze_url_for_dynamic_content("https://docs.example.com/guide").await,
            "Documentation should not require dynamic rendering"
        );
    }

    #[test]
    fn test_create_adaptive_dynamic_config_github() {
        let config = create_adaptive_dynamic_config("https://github.com/rust-lang/rust");

        // Verify GitHub-specific configuration
        assert!(config.wait_for.is_some(), "Should have wait condition");
        assert!(config.scroll.is_some(), "Should have scroll configuration");
        assert_eq!(config.timeout, Duration::from_secs(3), "Should have 3s timeout");

        // Verify viewport
        if let Some(viewport) = config.viewport {
            assert_eq!(viewport.width, 1920);
            assert_eq!(viewport.height, 1080);
            assert!(!viewport.is_mobile);
        }
    }

    #[test]
    fn test_create_adaptive_dynamic_config_twitter() {
        let config = create_adaptive_dynamic_config("https://twitter.com/user/status/123");

        // Verify Twitter-specific configuration
        if let Some(scroll) = config.scroll {
            assert_eq!(scroll.steps, 5, "Social media should have more scroll steps");
            assert_eq!(scroll.step_px, Some(800));
            assert!(scroll.after_scroll_js.is_some(), "Should have custom scroll JS");
        }
    }

    #[test]
    fn test_create_adaptive_dynamic_config_medium() {
        let config = create_adaptive_dynamic_config("https://medium.com/@user/article");

        // Verify article site configuration
        if let Some(scroll) = config.scroll {
            assert_eq!(scroll.steps, 3, "Article sites should have moderate scrolling");
            assert_eq!(scroll.mode, ScrollMode::Smooth, "Should use smooth scrolling");
            assert!(scroll.after_scroll_js.is_none(), "Should not have custom scroll JS");
        }
    }

    #[test]
    fn test_create_adaptive_dynamic_config_generic() {
        let config = create_adaptive_dynamic_config("https://example.com/page");

        // Verify generic configuration
        assert!(config.wait_for.is_some(), "Should have wait condition");
        assert!(config.scroll.is_some(), "Should have default scroll");
        assert_eq!(config.timeout, Duration::from_secs(3), "Should have 3s timeout");
        assert!(config.actions.is_empty(), "Should have no custom actions");
    }
}