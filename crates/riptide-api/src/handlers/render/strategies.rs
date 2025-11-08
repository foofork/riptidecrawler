use riptide_facade::facades::RenderStrategyFacade;
use riptide_headless::dynamic::DynamicConfig;

/// Global render strategy facade
static RENDER_FACADE: std::sync::OnceLock<RenderStrategyFacade> = std::sync::OnceLock::new();

fn get_render_facade() -> &'static RenderStrategyFacade {
    RENDER_FACADE.get_or_init(RenderStrategyFacade::new)
}

/// Analyze URL and content patterns to determine if dynamic rendering is needed
pub(super) async fn analyze_url_for_dynamic_content(url: &str) -> bool {
    let facade = get_render_facade();
    facade.requires_dynamic_rendering(url).await
}

/// Create adaptive dynamic configuration based on URL analysis
pub(super) fn create_adaptive_dynamic_config(url: &str) -> DynamicConfig {
    let facade = get_render_facade();
    facade.create_dynamic_config(url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use riptide_headless::dynamic::ScrollMode;
    use std::time::Duration;

    #[tokio::test]
    async fn test_analyze_url_for_dynamic_content() {
        assert!(analyze_url_for_dynamic_content("https://twitter.com/user/status/123").await);
        assert!(analyze_url_for_dynamic_content("https://github.com/org/repo").await);
        assert!(!analyze_url_for_dynamic_content("https://example.com/blog/article.html").await);
    }

    #[test]
    fn test_create_adaptive_dynamic_config() {
        let config = create_adaptive_dynamic_config("https://github.com/rust-lang/rust");
        assert!(config.wait_for.is_some());
        assert!(config.scroll.is_some());
        assert_eq!(config.timeout, Duration::from_secs(3));
    }
}
