//! Browser facade for headless browser automation.

use crate::config::RiptideConfig;
use crate::error::Result;
use crate::runtime::RiptideRuntime;
use std::sync::Arc;

pub struct BrowserFacade {
    config: RiptideConfig,
    runtime: Arc<RiptideRuntime>,
}

impl BrowserFacade {
    pub(crate) fn new(config: RiptideConfig, runtime: Arc<RiptideRuntime>) -> Self {
        Self { config, runtime }
    }

    pub async fn screenshot(&self, url: &str, options: ScreenshotOptions) -> Result<Vec<u8>> {
        unimplemented!("Browser facade not yet implemented")
    }
}

#[derive(Debug, Clone, Default)]
pub struct ScreenshotOptions {
    pub full_page: bool,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone)]
pub enum BrowserAction {
    Click { selector: String },
    Type { selector: String, text: String },
    Wait { duration_ms: u64 },
}
