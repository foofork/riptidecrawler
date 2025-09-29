use crate::circuit::CircuitBreaker;

/// Basic link extraction using regex (simplified for core)
fn extract_links_basic(content: &str, base_url: &Url) -> Result<Vec<Url>> {
    let link_regex = regex::Regex::new(r#"href\s*=\s*["']([^"']+)["']"#)?;
    let mut links = Vec::new();

    for cap in link_regex.captures_iter(content) {
        if let Some(link_str) = cap.get(1) {
            if let Ok(url) = base_url.join(link_str.as_str()) {
                links.push(url);
            }
        }
    }

    Ok(links)
}
use crate::fetch::FetchEngine;
use crate::memory_manager::MemoryManager;
use crate::robots::RobotsManager;
use crate::spider::{
    adaptive_stop::{AdaptiveStopEngine, StopDecision},
    budget::BudgetManager,
    config::SpiderConfig,
    frontier::FrontierManager,
    query_aware::{QueryAwareStats, QueryAwareScorer},
    session::SessionManager,
    sitemap::SitemapParser,
    strategy::StrategyEngine,
    types::{CrawlRequest, CrawlResult, Priority},
    url_utils::UrlUtils,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::sleep;
use tracing::{debug, error, info, instrument};
use url::Url;

/// Serde module for Duration serialization
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs_f64().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = f64::deserialize(deserializer)?;
        Ok(Duration::from_secs_f64(secs))
    }
}

/// Main Spider engine for deep crawling
pub struct Spider {
    config: SpiderConfig,
    
    // Core components
    frontier_manager: Arc<FrontierManager>,
    strategy_engine: Arc<RwLock<StrategyEngine>>,
    budget_manager: Arc<BudgetManager>,
    adaptive_stop_engine: Arc<AdaptiveStopEngine>,
    url_utils: Arc<RwLock<UrlUtils>>,
    
    // Session and authentication
    session_manager: Arc<SessionManager>,
    sitemap_parser: Arc<RwLock<SitemapParser>>,
    
    // Integration with existing systems
    robots_manager: Arc<RobotsManager>,
    circuit_breaker: Option<Arc<CircuitBreaker>>,
    memory_manager: Option<Arc<MemoryManager>>,
    fetch_engine: Option<Arc<FetchEngine>>,
    
    // Concurrency control
    global_semaphore: Arc<Semaphore>,
    host_semaphores: Arc<RwLock<HashMap<String, Arc<Semaphore>>>>,
    
    // State tracking
    crawl_state: Arc<RwLock<CrawlState>>,
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,

    // Query-aware functionality
    query_aware_scorer: Arc<RwLock<Option<QueryAwareScorer>>>,
}

/// Current crawl state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrawlState {
    /// Whether crawling is currently active
    pub active: bool,
    /// Crawl start time
    #[serde(skip)]
    pub start_time: Option<Instant>,
    /// Total pages crawled
    pub pages_crawled: u64,
    /// Total pages failed
    pub pages_failed: u64,
    /// Current frontier size
    pub frontier_size: usize,
    /// Last adaptive stop decision
    pub last_stop_decision: Option<StopDecision>,
    /// Domains being crawled
    pub active_domains: std::collections::HashSet<String>,
}

/// Performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Pages per second
    pub pages_per_second: f64,
    /// Average response time
    #[serde(with = "duration_serde")]
    pub avg_response_time: Duration,
    /// Memory usage
    pub memory_usage: usize,
    /// Error rate
    pub error_rate: f64,
    /// Last metrics update
    #[serde(skip)]
    pub last_update: Option<Instant>,
}

/// Spider crawl result
#[derive(Debug)]
pub struct SpiderResult {
    /// Total pages crawled
    pub pages_crawled: u64,
    /// Total pages failed
    pub pages_failed: u64,
    /// Crawl duration
    pub duration: Duration,
    /// Reason for stopping
    pub stop_reason: String,
    /// Final performance metrics
    pub performance: PerformanceMetrics,
    /// Domains crawled
    pub domains: Vec<String>,
}

impl Spider {
    /// Create a new Spider instance
    pub async fn new(config: SpiderConfig) -> Result<Self> {
        // Validate configuration
        config.validate().map_err(|e| anyhow::anyhow!("Invalid configuration: {}", e))?;
        
        info!("Initializing Spider with configuration: {:?}", config);
        
        // Initialize core components
        let frontier_manager = Arc::new(FrontierManager::new(config.frontier.clone())?);
        let strategy_engine = Arc::new(RwLock::new(StrategyEngine::new(config.strategy.to_crawling_strategy())));
        let budget_manager = Arc::new(BudgetManager::new(config.budget.clone()));
        let adaptive_stop_engine = Arc::new(AdaptiveStopEngine::new(config.adaptive_stop.clone()));
        let url_utils = Arc::new(RwLock::new(UrlUtils::new(config.url_processing.clone().into())));
        
        // Initialize session and sitemap components
        let session_manager = Arc::new(SessionManager::new(config.session.clone()));
        let sitemap_parser = Arc::new(RwLock::new(SitemapParser::new(config.sitemap.clone())));
        
        // Initialize robots manager
        let robots_manager = Arc::new(RobotsManager::new(config.robots.clone())?);        // Initialize concurrency control
        let global_semaphore = Arc::new(Semaphore::new(config.performance.max_concurrent_global));
        let host_semaphores = Arc::new(RwLock::new(HashMap::new()));
        
        // Initialize state tracking
        let crawl_state = Arc::new(RwLock::new(CrawlState::default()));
        let performance_metrics = Arc::new(RwLock::new(PerformanceMetrics::default()));

        // Initialize query-aware scorer if enabled
        let query_aware_scorer = if config.query_aware.query_foraging {
            Arc::new(RwLock::new(Some(QueryAwareScorer::new(config.query_aware.clone()))))
        } else {
            Arc::new(RwLock::new(None))
        };

        info!("Spider initialization completed");
        
        Ok(Self {
            config,
            frontier_manager,
            strategy_engine,
            budget_manager,
            adaptive_stop_engine,
            url_utils,
            session_manager,
            sitemap_parser,
            robots_manager,
            circuit_breaker: None,
            memory_manager: None,
            fetch_engine: None,
            global_semaphore,
            host_semaphores,
            crawl_state,
            performance_metrics,
            query_aware_scorer,
        })
    }
    
    /// Set integration components
    pub fn with_circuit_breaker(mut self, circuit_breaker: Arc<CircuitBreaker>) -> Self {
        self.circuit_breaker = Some(circuit_breaker);
        self
    }
    
    pub fn with_memory_manager(mut self, memory_manager: Arc<MemoryManager>) -> Self {
        self.memory_manager = Some(memory_manager);
        self
    }
    
    pub fn with_fetch_engine(mut self, fetch_engine: Arc<FetchEngine>) -> Self {
        self.fetch_engine = Some(fetch_engine);
        self
    }
    
    /// Start crawling from seed URLs
    #[instrument(skip(self), fields(seeds = seeds.len()))]
    pub async fn crawl(&self, seeds: Vec<Url>) -> Result<SpiderResult> {
        info!("Starting crawl with {} seed URLs", seeds.len());
        
        // Initialize crawl state
        {
            let mut state = self.crawl_state.write().await;
            state.active = true;
            state.start_time = Some(Instant::now());
            state.pages_crawled = 0;
            state.pages_failed = 0;
            state.active_domains = seeds.iter()
                .filter_map(|url| url.host_str().map(|h| h.to_string()))
                .collect();
        }
        
        // Discover and add sitemap URLs
        for seed in &seeds {
            if let Ok(sitemap_urls) = self.discover_sitemap_urls(seed).await {
                info!("Discovered {} URLs from sitemaps for {}", sitemap_urls.len(), seed.host_str().unwrap_or("unknown"));
            }
        }
        
        // Add seed URLs to frontier
        for seed in seeds {
            let request = CrawlRequest::new(seed).with_priority(Priority::High);
            self.frontier_manager.add_request(request).await?;
        }
        
        // Start main crawl loop
        let result = self.crawl_loop().await?;
        
        // Clean up
        {
            let mut state = self.crawl_state.write().await;
            state.active = false;
        }
        
        info!("Crawl completed: {} pages crawled, {} failed", result.pages_crawled, result.pages_failed);
        Ok(result)
    }
    
    /// Main crawl loop
    async fn crawl_loop(&self) -> Result<SpiderResult> {
        let start_time = Instant::now();
        let mut pages_crawled = 0u64;
        let mut pages_failed = 0u64;
        let mut last_metrics_update = Instant::now();
        
        loop {
            // Check if we should stop crawling
            if let Some(stop_reason) = self.should_stop_crawling().await? {
                return Ok(SpiderResult {
                    pages_crawled,
                    pages_failed,
                    duration: start_time.elapsed(),
                    stop_reason,
                    performance: self.performance_metrics.read().await.clone(),
                    domains: self.crawl_state.read().await.active_domains.iter().cloned().collect(),
                });
            }
            
            // Get next request from frontier
            let request = match self.frontier_manager.next_request().await? {
                Some(req) => req,
                None => {
                    // No more requests in frontier
                    if self.frontier_manager.size() == 0 {
                        return Ok(SpiderResult {
                            pages_crawled,
                            pages_failed,
                            duration: start_time.elapsed(),
                            stop_reason: "Frontier exhausted".to_string(),
                            performance: self.performance_metrics.read().await.clone(),
                            domains: self.crawl_state.read().await.active_domains.iter().cloned().collect(),
                        });
                    }
                    
                    // Wait a bit and try again
                    sleep(Duration::from_millis(100)).await;
                    continue;
                }
            };
            
            // Process the request
            match self.process_request(request).await {
                Ok(result) => {
                    if result.success {
                        pages_crawled += 1;

                        // Add extracted URLs to frontier - clone before moving to avoid partial move
                        let extracted_urls = result.extracted_urls.clone();
                        for extracted_url in extracted_urls {
                            let child_request = CrawlRequest::new(extracted_url)
                                .with_depth(result.request.depth + 1)
                                .with_parent(result.request.url.clone());
                            
                            // Check if URL should be crawled
                            if self.should_crawl_url_internal(&child_request).await? {
                                // Calculate priority based on strategy
                                let mut priority = self.strategy_engine.read().await.calculate_priority(&child_request).await;

                                // Apply query-aware scoring if enabled
                                if let Some(scorer) = self.query_aware_scorer.write().await.as_mut() {
                                    let relevance_score = scorer.score_request(&child_request, result.text_content.as_deref());
                                    // Blend strategy priority with relevance score using helper function
                                    priority = blend_priority_with_relevance(priority, relevance_score);
                                }

                                let final_request = child_request.with_priority(priority);
                                
                                self.frontier_manager.add_request(final_request).await?;
                            }
                        }
                        
                        // Update query-aware scorer if enabled
                        if let Some(scorer) = self.query_aware_scorer.write().await.as_mut() {
                            scorer.update_with_result(&result);
                        }

                        // Analyze result for adaptive stopping
                        let _metrics = self.adaptive_stop_engine.analyze_result(&result).await?;
                        
                        // Record strategy performance
                        self.strategy_engine.read().await.record_crawl_result(true).await;
                    } else {
                        pages_failed += 1;
                        self.strategy_engine.read().await.record_crawl_result(false).await;
                    }
                    
                    // Record result with frontier manager
                    self.frontier_manager.record_result(&result.request, result.success, result.error).await;
                }
                Err(e) => {
                    pages_failed += 1;
                    error!("Request processing failed: {}", e);
                }
            }
            
            // Update metrics periodically
            if last_metrics_update.elapsed() >= self.config.performance.metrics_interval {
                self.update_performance_metrics(pages_crawled, pages_failed, start_time.elapsed()).await;
                last_metrics_update = Instant::now();
            }
            
            // Update crawl state
            {
                let mut state = self.crawl_state.write().await;
                state.pages_crawled = pages_crawled;
                state.pages_failed = pages_failed;
                state.frontier_size = self.frontier_manager.size();
            }
        }
    }
    
    /// Process a single crawl request
    #[instrument(skip(self), fields(url = %request.url))]
    async fn process_request(&self, request: CrawlRequest) -> Result<CrawlResult> {
        let start_time = Instant::now();
        
        debug!("Processing request: {} (depth: {})", request.url, request.depth);
        
        // Check budget constraints
        if !self.budget_manager.can_make_request(&request.url, request.depth).await? {
            return Ok(CrawlResult::failure(request, "Budget constraints violated".to_string()));
        }
        
        // Acquire global semaphore
        let _global_permit = self.global_semaphore.acquire().await
            .context("Failed to acquire global semaphore")?;
        
        // Acquire host-specific semaphore
        let host_str = request.url.host_str().unwrap_or("unknown");
        let host = host_str.to_string();
        let host_semaphore = self.get_host_semaphore(&host).await;
        let _host_permit = host_semaphore.acquire().await
            .context("Failed to acquire host semaphore")?;
        
        // Start request tracking
        self.budget_manager.start_request(&request.url, request.depth).await?;
        
        // Check robots.txt compliance and rate limiting
        if !self.robots_manager.can_crawl_with_wait(request.url.as_str()).await? {
            self.budget_manager.complete_request(&request.url, 0, false).await?;
            return Ok(CrawlResult::failure(request, "Blocked by robots.txt".to_string()));
        }
        
        // Check circuit breaker if available
        if let Some(_circuit_breaker) = &self.circuit_breaker {
            // Use host as the key for circuit breaker
            // Note: This is simplified - you'd want proper integration
        }
        
        // Get session client if needed
        let client = if self.config.session.enable_session_persistence {
            self.session_manager.get_session_client(&host).await?
        } else {
            None
        };
        
        // Perform the actual fetch
        let fetch_result = if let Some(fetch_engine) = &self.fetch_engine {
            // Use integrated fetch engine
            self.fetch_with_engine(fetch_engine, &request).await
        } else {
            // Use basic fetch
            self.basic_fetch(&request, client).await
        };
        
        let (success, content_size, error) = match fetch_result {
            Ok((content, size)) => {
                // Extract URLs and analyze content
                let extracted_urls = self.extract_urls(&content, &request.url).await?;
                let text_content = self.extract_text_content(&content).await;

                let mut result = CrawlResult::success(request.clone());
                result.content_size = size;
                result.text_content = text_content;
                result.extracted_urls = extracted_urls;
                result.processing_time = start_time.elapsed();

                self.budget_manager.complete_request(&request.url, size, true).await?;
                return Ok(result);
            }
            Err(e) => {
                self.budget_manager.complete_request(&request.url, 0, false).await?;
                (false, 0, Some(e.to_string()))
            }
        };
        
        let mut result = if success {
            CrawlResult::success(request)
        } else {
            CrawlResult::failure(request, error.unwrap_or_else(|| "Unknown error".to_string()))
        };
        
        result.content_size = content_size;
        result.processing_time = start_time.elapsed();
        
        Ok(result)
    }
    
    /// Basic fetch implementation
    async fn basic_fetch(&self, request: &CrawlRequest, client: Option<reqwest::Client>) -> Result<(String, usize)> {
        let http_client = client.unwrap_or_else(|| {
            reqwest::Client::builder()
                .user_agent("RipTide Spider/1.0")
                .timeout(self.config.performance.request_timeout)
                .build()
                .unwrap()
        });
        
        let response = http_client
            .get(request.url.as_str())
            .send()
            .await
            .context("Failed to send HTTP request")?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
        }
        
        let content = response.text().await.context("Failed to read response body")?;
        let size = content.len();
        
        Ok((content, size))
    }
    
    /// Fetch using integrated fetch engine
    async fn fetch_with_engine(&self, _fetch_engine: &Arc<FetchEngine>, request: &CrawlRequest) -> Result<(String, usize)> {
        // Placeholder - integrate with actual fetch engine
        self.basic_fetch(request, None).await
    }
    
    /// Extract URLs from content using riptide-html DOM parser
    async fn extract_urls(&self, content: &str, base_url: &Url) -> Result<Vec<Url>> {
        // Use riptide-html for proper DOM-based link extraction
        // Basic link extraction using regex (simplified for core)
        // Full DOM-based extraction is available in riptide-html crate
        let links = extract_links_basic(content, base_url)?;

        // Filter URLs using URL utils
        let filtered_urls = self.url_utils.read().await.filter_urls(links).await?;
        Ok(filtered_urls)
    }
    
    /// Extract text content from HTML using riptide-html DOM parser
    async fn extract_text_content(&self, content: &str) -> Option<String> {
        // Use riptide-html for proper DOM-based text extraction
        // Simplified for core - full DOM extraction in riptide-html
        // Basic text extraction (simplified for core)
        self.simple_text_extraction(content)
    }

    /// Simple text extraction fallback method
    fn simple_text_extraction(&self, content: &str) -> Option<String> {
        let mut text = String::new();
        let mut in_tag = false;

        for char in content.chars() {
            match char {
                '<' => in_tag = true,
                '>' => in_tag = false,
                c if !in_tag && !c.is_control() => text.push(c),
                _ => {}
            }
        }

        if text.trim().is_empty() {
            None
        } else {
            Some(text)
        }
    }
    
    /// Check if a URL should be crawled
    #[cfg(test)]
    pub async fn should_crawl_url(&self, request: &CrawlRequest) -> Result<bool> {
        self.should_crawl_url_internal(request).await
    }

    /// Check if a URL should be crawled (internal)
    async fn should_crawl_url_internal(&self, request: &CrawlRequest) -> Result<bool> {
        // Check URL validity
        if !self.url_utils.read().await.is_valid_for_crawling(&request.url).await? {
            return Ok(false);
        }
        
        // Check budget constraints
        if !self.budget_manager.can_make_request(&request.url, request.depth).await? {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// Check if crawling should stop
    async fn should_stop_crawling(&self) -> Result<Option<String>> {
        // Check query-aware early stopping
        if let Some(scorer) = self.query_aware_scorer.read().await.as_ref() {
            let (should_stop, reason) = scorer.should_stop_early();
            if should_stop {
                return Ok(Some(reason));
            }
        }

        // Check adaptive stop conditions
        let stop_decision = self.adaptive_stop_engine.should_stop().await?;
        if stop_decision.should_stop {
            return Ok(Some(stop_decision.reason));
        }

        // Check memory pressure if memory manager is available
        if let Some(_memory_manager) = &self.memory_manager {
            // Placeholder for memory pressure check
        }

        Ok(None)
    }
    
    /// Discover sitemap URLs for a domain
    async fn discover_sitemap_urls(&self, seed: &Url) -> Result<Vec<CrawlRequest>> {
        let sitemap_urls = self.sitemap_parser.write().await
            .discover_and_parse(seed).await?;
        
        let requests = self.sitemap_parser.read().await
            .urls_to_crawl_requests(sitemap_urls);
        
        Ok(requests)
    }
    
    /// Get or create host-specific semaphore
    async fn get_host_semaphore(&self, host: &str) -> Arc<Semaphore> {
        let semaphores = self.host_semaphores.read().await;
        if let Some(semaphore) = semaphores.get(host) {
            return semaphore.clone();
        }
        
        drop(semaphores);
        
        let mut semaphores = self.host_semaphores.write().await;
        // Double-check after acquiring write lock
        if let Some(semaphore) = semaphores.get(host) {
            return semaphore.clone();
        }
        
        let semaphore = Arc::new(Semaphore::new(self.config.performance.max_concurrent_per_host));
        semaphores.insert(host.to_string(), semaphore.clone());
        semaphore
    }
    
    /// Update performance metrics
    async fn update_performance_metrics(&self, pages_crawled: u64, pages_failed: u64, duration: Duration) {
        let mut metrics = self.performance_metrics.write().await;
        
        if duration.as_secs_f64() > 0.0 {
            metrics.pages_per_second = pages_crawled as f64 / duration.as_secs_f64();
        }
        
        let total_requests = pages_crawled + pages_failed;
        if total_requests > 0 {
            metrics.error_rate = pages_failed as f64 / total_requests as f64;
        }
        
        // Get memory usage from config estimation
        metrics.memory_usage = self.config.estimate_memory_usage();
        metrics.last_update = Some(Instant::now());
        
        debug!("Performance metrics updated: {:.2} pages/sec, {:.2}% error rate", 
               metrics.pages_per_second, metrics.error_rate * 100.0);
    }
    
    /// Get current crawl state
    pub async fn get_crawl_state(&self) -> CrawlState {
        self.crawl_state.read().await.clone()
    }
    
    /// Get current performance metrics
    pub async fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics.read().await.clone()
    }
    
    /// Get frontier statistics
    pub async fn get_frontier_stats(&self) -> crate::spider::types::FrontierMetrics {
        self.frontier_manager.get_metrics().await
    }
    
    /// Get adaptive stop statistics
    pub async fn get_adaptive_stop_stats(&self) -> crate::spider::adaptive_stop::AdaptiveStopStats {
        self.adaptive_stop_engine.get_stats().await
    }

    /// Get reference to session manager for testing
    #[cfg(test)]
    pub fn session_manager(&self) -> &Arc<SessionManager> {
        &self.session_manager
    }

    /// Get reference to adaptive stop engine for testing
    #[cfg(test)]
    pub fn adaptive_stop_engine(&self) -> &Arc<AdaptiveStopEngine> {
        &self.adaptive_stop_engine
    }

    /// Get reference to frontier manager for testing
    #[cfg(test)]
    pub fn frontier_manager(&self) -> &Arc<FrontierManager> {
        &self.frontier_manager
    }

    /// Get reference to robots manager for testing
    #[cfg(test)]
    pub fn robots_manager(&self) -> &Arc<RobotsManager> {
        &self.robots_manager
    }

    /// Get reference to budget manager for testing
    #[cfg(test)]
    pub fn budget_manager(&self) -> &Arc<BudgetManager> {
        &self.budget_manager
    }

    /// Get reference to URL utils for testing
    #[cfg(test)]
    pub fn url_utils(&self) -> &Arc<RwLock<UrlUtils>> {
        &self.url_utils
    }

    /// Get query-aware statistics
    pub async fn get_query_aware_stats(&self) -> QueryAwareStats {
        match self.query_aware_scorer.read().await.as_ref() {
            Some(scorer) => scorer.get_stats(),
            None => QueryAwareStats {
                enabled: false,
                unique_domains: 0,
                total_pages: 0,
                avg_recent_relevance: 0.0,
                corpus_size: 0,
                recent_scores: Vec::new(),
            },
        }
    }

    /// Score a request using query-aware algorithm
    pub async fn score_query_aware_request(&self, request: &CrawlRequest, content: Option<&str>) -> Result<f64> {
        match self.query_aware_scorer.write().await.as_mut() {
            Some(scorer) => Ok(scorer.score_request(request, content)),
            None => Ok(1.0), // Neutral score when query-aware is disabled
        }
    }

    /// Update query-aware scorer with crawl result
    pub async fn update_query_aware_with_result(&self, result: &CrawlResult) -> Result<()> {
        if let Some(scorer) = self.query_aware_scorer.write().await.as_mut() {
            scorer.update_with_result(result);
        }
        Ok(())
    }

    /// Check if crawling should stop due to low query relevance
    pub async fn should_stop_query_aware(&self) -> Result<(bool, String)> {
        match self.query_aware_scorer.read().await.as_ref() {
            Some(scorer) => Ok(scorer.should_stop_early()),
            None => Ok((false, String::new())),
        }
    }

    
    /// Stop the current crawl
    pub async fn stop(&self) {
        let mut state = self.crawl_state.write().await;
        state.active = false;
        info!("Crawl stop requested");
    }
    
    /// Clear all state and reset spider
    pub async fn reset(&self) -> Result<()> {
        // Clear frontier
        self.frontier_manager.clear().await;
        
        // Reset adaptive stop engine
        self.adaptive_stop_engine.reset().await;
        
        // Clear URL utils
        self.url_utils.write().await.clear().await;
        
        // Clear sessions
        self.session_manager.clear_sessions().await;
        
        // Clear sitemap cache
        self.sitemap_parser.write().await.clear_cache();
        
        // Reset state
        {
            let mut state = self.crawl_state.write().await;
            *state = CrawlState::default();
        }
        
        {
            let mut metrics = self.performance_metrics.write().await;
            *metrics = PerformanceMetrics::default();
        }
        
        info!("Spider reset completed");
        Ok(())
    }
}

/// Blend strategy priority with query-aware relevance score
fn blend_priority_with_relevance(strategy_priority: Priority, relevance_score: f64) -> Priority {
    use Priority::*;

    // Convert priority to numeric score
    let priority_score = match strategy_priority {
        Critical => 4.0,
        High => 3.0,
        Medium => 2.0,
        Low => 1.0,
    };

    // Blend scores (70% strategy, 30% relevance)
    let blended_score = 0.7 * priority_score + 0.3 * relevance_score * 4.0;

    // Convert back to priority
    if blended_score >= 3.5 {
        Critical
    } else if blended_score >= 2.5 {
        High
    } else if blended_score >= 1.5 {
        Medium
    } else {
        Low
    }
}

// Convert UrlProcessingConfig to UrlUtilsConfig
impl From<crate::spider::config::UrlProcessingConfig> for crate::spider::url_utils::UrlUtilsConfig {
    fn from(config: crate::spider::config::UrlProcessingConfig) -> Self {
        Self {
            enable_bloom_filter: config.enable_deduplication,
            bloom_filter_capacity: config.bloom_filter_capacity,
            bloom_filter_fpr: config.bloom_filter_fpr,
            enable_exact_tracking: config.enable_deduplication,
            max_exact_urls: config.max_exact_urls,
            enable_normalization: config.enable_normalization,
            remove_fragments: true,
            sort_query_params: true,
            remove_default_ports: true,
            lowercase_hostname: true,
            remove_trailing_slash: true,
            remove_www_prefix: false,
            exclude_patterns: config.exclude_patterns,
            exclude_extensions: config.exclude_extensions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spider::config::SpiderPresets;
    use std::str::FromStr;
    
    #[tokio::test]
    async fn test_spider_creation() {
        let config = SpiderPresets::development();
        let spider = Spider::new(config).await;
        assert!(spider.is_ok());
    }
    
    #[tokio::test]
    async fn test_spider_state_management() {
        let config = SpiderPresets::development();
        let spider = Spider::new(config).await.expect("Spider should be created");
        
        let initial_state = spider.get_crawl_state().await;
        assert!(!initial_state.active);
        assert_eq!(initial_state.pages_crawled, 0);
        
        let metrics = spider.get_performance_metrics().await;
        assert_eq!(metrics.pages_per_second, 0.0);
    }
    
    #[tokio::test]
    async fn test_spider_reset() {
        let config = SpiderPresets::development();
        let spider = Spider::new(config).await.expect("Spider should be created");
        
        spider.reset().await.expect("Reset should work");
        
        let state = spider.get_crawl_state().await;
        assert!(!state.active);
        assert_eq!(state.pages_crawled, 0);
    }
    
    #[tokio::test]
    async fn test_should_crawl_url() {
        let config = SpiderPresets::development();
        let spider = Spider::new(config).await.expect("Spider should be created");
        
        let valid_url = Url::from_str("https://example.com/page.html").expect("Valid URL");
        let valid_request = CrawlRequest::new(valid_url);
        
        let _should_crawl = spider.should_crawl_url(&valid_request).await.expect("Check should work");
        // Result depends on URL utils configuration
    }
    
    #[tokio::test]
    async fn test_host_semaphore_creation() {
        let config = SpiderPresets::development();
        let spider = Spider::new(config).await.expect("Spider should be created");
        
        let sem1 = spider.get_host_semaphore("example.com").await;
        let sem2 = spider.get_host_semaphore("example.com").await;
        
        // Should return the same semaphore instance
        assert!(Arc::ptr_eq(&sem1, &sem2));
        
        let sem3 = spider.get_host_semaphore("other.com").await;
        assert!(!Arc::ptr_eq(&sem1, &sem3));
    }
}