//! Factory functions for creating browser engines

use crate::{
    traits::{BrowserEngine, EngineType},
    AbstractionError, AbstractionResult,
};

/// Create a browser engine of the specified type
///
/// Note: This is a placeholder factory. In practice, you should create engines directly:
/// ```ignore
/// use riptide_browser_abstraction::ChromiumoxideEngine;
///
/// // Create browser instance first
/// let (browser, handler) = chromiumoxide::Browser::launch(config).await?;
///
/// // Wrap in abstraction
/// let engine = ChromiumoxideEngine::new(browser);
/// ```
pub async fn create_engine(engine_type: EngineType) -> AbstractionResult<Box<dyn BrowserEngine>> {
    match engine_type {
        EngineType::Chromiumoxide => {
            // This is a placeholder - actual implementation needs Browser instance
            Err(AbstractionError::Other(
                "Factory function requires Browser instance parameter. Use ChromiumoxideEngine::new() directly.".to_string()
            ))
        }
        EngineType::SpiderChrome => {
            // This is a placeholder - actual implementation needs Browser instance
            Err(AbstractionError::Other(
                "Factory function requires Browser instance parameter. Use SpiderChromeEngine::new() directly.".to_string()
            ))
        }
    }
}
