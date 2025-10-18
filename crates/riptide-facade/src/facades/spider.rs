//! Spider facade for web crawling operations.

use crate::config::RiptideConfig;
use crate::error::Result;
use crate::runtime::RiptideRuntime;
use riptide_types::ExtractedDoc;
use std::sync::Arc;

pub struct SpiderFacade {
    config: RiptideConfig,
    runtime: Arc<RiptideRuntime>,
}

impl SpiderFacade {
    pub(crate) fn new(config: RiptideConfig, runtime: Arc<RiptideRuntime>) -> Self {
        Self { config, runtime }
    }

    pub async fn crawl(&self, start_url: &str) -> Result<CrawlResult> {
        unimplemented!("Spider facade not yet implemented")
    }
}

#[derive(Debug, Clone)]
pub struct CrawlResult {
    pub pages: Vec<ExtractedDoc>,
    pub total_pages: usize,
}

#[derive(Debug, Clone, Default)]
pub struct CrawlBudget {
    pub max_pages: Option<usize>,
    pub max_depth: Option<u32>,
    pub timeout_secs: Option<u64>,
}
