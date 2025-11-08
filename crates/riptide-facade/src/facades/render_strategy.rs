//! Render strategy facade for determining rendering approaches

use riptide_headless::dynamic::DynamicConfig;

#[cfg(feature = "llm")]
use riptide_intelligence::ContentAnalyzer;

/// Facade for rendering strategy selection
pub struct RenderStrategyFacade {
    #[cfg(feature = "llm")]
    analyzer: ContentAnalyzer,
}

impl RenderStrategyFacade {
    /// Create a new render strategy facade
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "llm")]
            analyzer: ContentAnalyzer::new(),
        }
    }

    /// Analyze URL to determine if dynamic rendering is needed
    pub async fn requires_dynamic_rendering(&self, _url: &str) -> bool {
        #[cfg(feature = "llm")]
        {
            self.analyzer.analyze_url_for_dynamic_content(_url).await
        }
        #[cfg(not(feature = "llm"))]
        {
            // Default fallback: assume dynamic rendering may be needed
            false
        }
    }

    /// Create adaptive dynamic configuration for a URL
    pub fn create_dynamic_config(&self, _url: &str) -> DynamicConfig {
        #[cfg(feature = "llm")]
        {
            self.analyzer.create_adaptive_dynamic_config(_url)
        }
        #[cfg(not(feature = "llm"))]
        {
            // Default fallback configuration
            DynamicConfig::default()
        }
    }
}

impl Default for RenderStrategyFacade {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(feature = "llm")]
    async fn test_requires_dynamic_rendering() {
        let facade = RenderStrategyFacade::new();

        // Dynamic sites
        assert!(
            facade
                .requires_dynamic_rendering("https://twitter.com/user")
                .await
        );
        assert!(
            facade
                .requires_dynamic_rendering("https://github.com/org/repo")
                .await
        );

        // Static sites
        assert!(
            !facade
                .requires_dynamic_rendering("https://example.com/blog")
                .await
        );
    }

    #[test]
    fn test_create_dynamic_config() {
        let facade = RenderStrategyFacade::new();
        let _config = facade.create_dynamic_config("https://github.com/rust-lang/rust");

        #[cfg(feature = "llm")]
        {
            assert!(_config.wait_for.is_some());
            assert!(_config.scroll.is_some());
            assert!(_config.viewport.is_some());
        }
    }
}
