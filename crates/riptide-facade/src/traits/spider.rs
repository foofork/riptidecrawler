//! Spider trait for URL discovery
//!
//! Provides async trait for crawling websites and discovering URLs.

use async_trait::async_trait;
use futures::stream::BoxStream;
use crate::error::RiptideResult;
use url::Url;

/// Options for spider crawling behavior
#[derive(Debug, Clone)]
pub struct SpiderOpts {
    /// Maximum depth to crawl (default: 3)
    pub max_depth: usize,
    /// Maximum number of pages to crawl (default: 100)
    pub max_pages: usize,
    /// Follow external links (default: false)
    pub follow_external: bool,
    /// Respect robots.txt (default: true)
    pub respect_robots_txt: bool,
    /// Concurrent requests (default: 10)
    pub concurrency: usize,
}

impl Default for SpiderOpts {
    fn default() -> Self {
        Self {
            max_depth: 3,
            max_pages: 100,
            follow_external: false,
            respect_robots_txt: true,
            concurrency: 10,
        }
    }
}

/// Spider trait for discovering URLs through crawling
///
/// This trait provides async URL discovery capabilities. It returns a stream
/// of URLs that can be composed with extractors for full pipeline processing.
///
/// # Examples
///
/// ```no_run
/// use riptide_facade::traits::{Spider, SpiderOpts};
/// use futures::StreamExt;
///
/// # async fn example(spider: impl Spider) -> Result<(), Box<dyn std::error::Error>> {
/// let urls = spider.crawl("https://example.com", SpiderOpts::default()).await?;
///
/// // Collect all discovered URLs
/// let urls: Vec<_> = urls.collect().await;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait Spider: Send + Sync {
    /// Crawl a URL and discover linked pages
    ///
    /// Returns a stream of discovered URLs. The stream continues until
    /// all pages within the configured limits have been discovered.
    ///
    /// # Arguments
    ///
    /// * `url` - Starting URL to begin crawling
    /// * `opts` - Configuration options for crawl behavior
    ///
    /// # Returns
    ///
    /// BoxStream of Result<Url> - discovered URLs or errors
    async fn crawl(
        &self,
        url: &str,
        opts: SpiderOpts,
    ) -> RiptideResult<BoxStream<'static, RiptideResult<Url>>>;
}
