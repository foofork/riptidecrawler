use crate::spider::types::{CrawlRequest, HostState, FrontierMetrics, Priority};
use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, VecDeque};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};

/// Configuration for frontier management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontierConfig {
    /// Maximum number of requests to keep in memory
    pub memory_limit: usize,
    /// Memory limit in MB for the frontier
    pub memory_limit_mb: usize,
    /// Enable disk spillover for large frontiers
    pub enable_disk_spillover: bool,
    /// Path for disk spillover storage
    pub spillover_path: Option<String>,
    /// Maximum requests per host in frontier
    pub max_requests_per_host: usize,
    /// Enable host balancing to prevent monopolization
    pub enable_host_balancing: bool,
    /// Maximum host diversity score (higher = more diverse)
    pub max_host_diversity: f64,
    /// Cleanup interval for expired requests
    pub cleanup_interval: Duration,
    /// Maximum age for requests before cleanup
    pub max_request_age: Duration,
}

impl Default for FrontierConfig {
    fn default() -> Self {
        Self {
            memory_limit: 100_000,
            memory_limit_mb: 100,
            enable_disk_spillover: true,
            spillover_path: None,
            max_requests_per_host: 1000,
            enable_host_balancing: true,
            max_host_diversity: 0.3, // 30% from single host max
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            max_request_age: Duration::from_secs(3600), // 1 hour
        }
    }
}

/// Priority-ordered request for heap management
#[derive(Debug, Clone)]
struct PriorityRequest {
    request: CrawlRequest,
    score: f64,
    #[allow(dead_code)]
    insertion_time: Instant,
}

impl PartialEq for PriorityRequest {
    fn eq(&self, other: &Self) -> bool {
        self.score.partial_cmp(&other.score) == Some(std::cmp::Ordering::Equal)
    }
}

impl Eq for PriorityRequest {}

impl PartialOrd for PriorityRequest {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Reverse for max-heap behavior
        other.score.partial_cmp(&self.score)
    }
}

impl Ord for PriorityRequest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

/// Per-host queue management
#[derive(Debug)]
struct HostQueue {
    /// Host name
    #[allow(dead_code)]
    host: String,
    /// Requests for this host
    requests: VecDeque<CrawlRequest>,
    /// Host state and metrics
    state: HostState,
    /// Last access time for cleanup
    last_access: Instant,
}

impl HostQueue {
    fn new(host: String) -> Self {
        Self {
            state: HostState::new(host.clone()),
            host,
            requests: VecDeque::new(),
            last_access: Instant::now(),
        }
    }

    fn push_request(&mut self, request: CrawlRequest) {
        self.requests.push_back(request);
        self.last_access = Instant::now();
    }

    fn pop_request(&mut self) -> Option<CrawlRequest> {
        let request = self.requests.pop_front();
        if request.is_some() {
            self.last_access = Instant::now();
        }
        request
    }

    fn len(&self) -> usize {
        self.requests.len()
    }

    fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }
}

/// Disk-backed queue for spillover (simplified implementation)
#[derive(Debug)]
struct DiskBackedQueue {
    _path: String,
    // In a real implementation, this would use a persistent queue like SQLite or RocksDB
    // For now, we'll use memory as a placeholder
    _queue: Mutex<VecDeque<CrawlRequest>>,
}

impl DiskBackedQueue {
    fn new(path: String) -> Result<Self> {
        Ok(Self {
            _path: path,
            _queue: Mutex::new(VecDeque::new()),
        })
    }

    async fn _push(&self, _request: CrawlRequest) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    async fn _pop(&self) -> Result<Option<CrawlRequest>> {
        // Placeholder implementation
        Ok(None)
    }

    async fn _len(&self) -> Result<usize> {
        // Placeholder implementation
        Ok(0)
    }
}

/// Main frontier management system
pub struct FrontierManager {
    config: FrontierConfig,

    // Multi-priority queues
    high_priority: Arc<Mutex<VecDeque<CrawlRequest>>>,
    medium_priority: Arc<Mutex<VecDeque<CrawlRequest>>>,
    low_priority: Arc<Mutex<VecDeque<CrawlRequest>>>,

    // Best-first priority queue
    best_first_queue: Arc<Mutex<BinaryHeap<PriorityRequest>>>,

    // Per-host management
    host_queues: DashMap<String, Arc<Mutex<HostQueue>>>,

    // Disk spillover
    disk_queue: Option<Arc<DiskBackedQueue>>,

    // Metrics and monitoring
    metrics: Arc<RwLock<FrontierMetrics>>,
    total_size: AtomicUsize,
    requests_added: AtomicU64,
    requests_processed: AtomicU64,

    // Cleanup tracking
    last_cleanup: Arc<Mutex<Instant>>,
}

impl FrontierManager {
    pub fn new(config: FrontierConfig) -> Result<Self> {
        let disk_queue = if config.enable_disk_spillover {
            let path = config
                .spillover_path
                .clone()
                .unwrap_or_else(|| "/tmp/frontier_spillover".to_string());
            Some(Arc::new(DiskBackedQueue::new(path)?))
        } else {
            None
        };

        Ok(Self {
            config,
            high_priority: Arc::new(Mutex::new(VecDeque::new())),
            medium_priority: Arc::new(Mutex::new(VecDeque::new())),
            low_priority: Arc::new(Mutex::new(VecDeque::new())),
            best_first_queue: Arc::new(Mutex::new(BinaryHeap::new())),
            host_queues: DashMap::new(),
            disk_queue,
            metrics: Arc::new(RwLock::new(FrontierMetrics::default())),
            total_size: AtomicUsize::new(0),
            requests_added: AtomicU64::new(0),
            requests_processed: AtomicU64::new(0),
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        })
    }

    /// Add a request to the frontier
    pub async fn add_request(&self, request: CrawlRequest) -> Result<()> {
        // Clone request data for logging before moving
        let request_url = request.url.clone();
        let request_priority = request.priority;
        let request_depth = request.depth;
        let request_host = request.host().map(|h| h.to_string());

        // Check if we should perform cleanup
        self.maybe_cleanup().await?;

        // Check memory limits
        if self.total_size.load(Ordering::Relaxed) >= self.config.memory_limit {
            if let Some(disk_queue) = &self.disk_queue {
                disk_queue._push(request).await?;
                return Ok(());
            } else {
                warn!("Frontier at memory limit and no disk spillover configured");
                return Err(anyhow::anyhow!("Frontier memory limit exceeded"));
            }
        }

        // Check host limits if balancing is enabled
        if self.config.enable_host_balancing {
            if let Some(host) = request.host() {
                if let Some(host_queue) = self.host_queues.get(host) {
                    let queue = host_queue.lock().await;
                    if queue.len() >= self.config.max_requests_per_host {
                        debug!(
                            host = %host,
                            current_count = queue.len(),
                            max_allowed = self.config.max_requests_per_host,
                            "Host request limit reached, dropping request"
                        );
                        return Ok(());
                    }
                }
            }
        }

        // Add to appropriate queue based on priority and score
        if let Some(score) = request.score {
            let priority_request = PriorityRequest {
                request,
                score,
                insertion_time: Instant::now(),
            };
            self.best_first_queue.lock().await.push(priority_request);
        } else {
            match request.priority {
                Priority::Critical | Priority::High => {
                    self.high_priority.lock().await.push_back(request);
                }
                Priority::Medium => {
                    self.medium_priority.lock().await.push_back(request);
                }
                Priority::Low => {
                    self.low_priority.lock().await.push_back(request);
                }
            }
        }

        // Update host queue if host balancing is enabled
        if self.config.enable_host_balancing {
            if let Some(host) = &request_host {
                let _host_queue = self
                    .host_queues
                    .entry(host.to_string())
                    .or_insert_with(|| Arc::new(Mutex::new(HostQueue::new(host.to_string()))));

                // Note: We already added to main queue, so we don't add to host queue here
                // Host queue is used for balancing decisions
            }
        }

        // Update metrics
        self.total_size.fetch_add(1, Ordering::Relaxed);
        self.requests_added.fetch_add(1, Ordering::Relaxed);
        self.update_metrics().await;

        debug!(
            url = %request_url,
            priority = ?request_priority,
            depth = request_depth,
            "Added request to frontier"
        );

        Ok(())
    }

    /// Get the next request to process
    pub async fn next_request(&self) -> Result<Option<CrawlRequest>> {
        // Try each queue in priority order

        // 1. Best-first queue (highest priority)
        if let Some(priority_request) = self.best_first_queue.lock().await.pop() {
            self.total_size.fetch_sub(1, Ordering::Relaxed);
            self.requests_processed.fetch_add(1, Ordering::Relaxed);
            return Ok(Some(priority_request.request));
        }

        // 2. High priority queue
        if let Some(request) = self.high_priority.lock().await.pop_front() {
            self.total_size.fetch_sub(1, Ordering::Relaxed);
            self.requests_processed.fetch_add(1, Ordering::Relaxed);
            return Ok(Some(request));
        }

        // 3. Medium priority queue
        if let Some(request) = self.medium_priority.lock().await.pop_front() {
            self.total_size.fetch_sub(1, Ordering::Relaxed);
            self.requests_processed.fetch_add(1, Ordering::Relaxed);
            return Ok(Some(request));
        }

        // 4. Low priority queue
        if let Some(request) = self.low_priority.lock().await.pop_front() {
            self.total_size.fetch_sub(1, Ordering::Relaxed);
            self.requests_processed.fetch_add(1, Ordering::Relaxed);
            return Ok(Some(request));
        }

        // 5. Try disk spillover if available
        if let Some(disk_queue) = &self.disk_queue {
            if let Some(request) = disk_queue._pop().await? {
                self.requests_processed.fetch_add(1, Ordering::Relaxed);
                return Ok(Some(request));
            }
        }

        Ok(None)
    }

    /// Get the next request for a specific host
    pub async fn next_request_for_host(&self, host: &str) -> Result<Option<CrawlRequest>> {
        if let Some(host_queue_ref) = self.host_queues.get(host) {
            let mut host_queue = host_queue_ref.lock().await;
            if host_queue.state.can_accept_request() {
                if let Some(request) = host_queue.pop_request() {
                    host_queue.state.start_request()?;
                    self.total_size.fetch_sub(1, Ordering::Relaxed);
                    self.requests_processed.fetch_add(1, Ordering::Relaxed);
                    return Ok(Some(request));
                }
            }
        }
        Ok(None)
    }

    /// Record the result of a crawl operation
    pub async fn record_result(&self, request: &CrawlRequest, success: bool, error: Option<String>) {
        if let Some(host) = request.host() {
            if let Some(host_queue_ref) = self.host_queues.get(host) {
                let mut host_queue = host_queue_ref.lock().await;
                if success {
                    host_queue.state.record_success();
                } else {
                    host_queue.state.record_error(error.unwrap_or_else(|| "Unknown error".to_string()));
                }
            }
        }
    }

    /// Get current frontier size
    pub fn size(&self) -> usize {
        self.total_size.load(Ordering::Relaxed)
    }

    /// Check if frontier is empty
    pub async fn is_empty(&self) -> bool {
        if self.total_size.load(Ordering::Relaxed) > 0 {
            return false;
        }

        // Check disk spillover
        if let Some(disk_queue) = &self.disk_queue {
            if let Ok(size) = disk_queue._len().await {
                return size == 0;
            }
        }

        true
    }

    /// Get frontier metrics
    pub async fn get_metrics(&self) -> FrontierMetrics {
        self.metrics.read().await.clone()
    }

    /// Update metrics
    async fn update_metrics(&self) {
        let mut metrics = self.metrics.write().await;

        metrics.total_requests = self.total_size.load(Ordering::Relaxed);

        // Update request counts by priority
        metrics.requests_by_priority.clear();
        metrics.requests_by_priority.insert(
            Priority::High,
            self.high_priority.lock().await.len() + self.best_first_queue.lock().await.len(),
        );
        metrics.requests_by_priority.insert(
            Priority::Medium,
            self.medium_priority.lock().await.len(),
        );
        metrics.requests_by_priority.insert(
            Priority::Low,
            self.low_priority.lock().await.len(),
        );

        // Update host distribution
        metrics.requests_by_host.clear();
        for entry in self.host_queues.iter() {
            let host_queue = entry.value().lock().await;
            metrics.requests_by_host.insert(entry.key().clone(), host_queue.len());
        }

        // Calculate memory usage (rough estimate)
        metrics.memory_usage = metrics.total_requests * 1024; // Rough estimate

        // Update rates
        let added = self.requests_added.load(Ordering::Relaxed);
        let processed = self.requests_processed.load(Ordering::Relaxed);
        metrics.update_rates(added, processed, Duration::from_secs(60));
    }

    /// Perform cleanup of expired requests
    async fn maybe_cleanup(&self) -> Result<()> {
        let mut last_cleanup = self.last_cleanup.lock().await;
        if last_cleanup.elapsed() >= self.config.cleanup_interval {
            self.cleanup_expired_requests().await?;
            *last_cleanup = Instant::now();
        }
        Ok(())
    }

    /// Clean up expired requests
    async fn cleanup_expired_requests(&self) -> Result<()> {
        let _now = Instant::now();
        let mut cleaned = 0;

        // Clean up host queues
        let hosts_to_remove: Vec<String> = self
            .host_queues
            .iter()
            .filter_map(|entry| {
                let host_queue = entry.value().try_lock();
                if let Ok(queue) = host_queue {
                    if queue.last_access.elapsed() > self.config.max_request_age && queue.is_empty() {
                        Some(entry.key().clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        for host in hosts_to_remove {
            self.host_queues.remove(&host);
            cleaned += 1;
        }

        if cleaned > 0 {
            info!(cleaned_hosts = cleaned, "Cleaned up expired host queues");
        }

        Ok(())
    }

    /// Clear all requests from frontier
    pub async fn clear(&self) {
        self.high_priority.lock().await.clear();
        self.medium_priority.lock().await.clear();
        self.low_priority.lock().await.clear();
        self.best_first_queue.lock().await.clear();
        self.host_queues.clear();
        self.total_size.store(0, Ordering::Relaxed);

        info!("Cleared frontier");
    }

    /// Get configuration
    pub fn get_config(&self) -> &FrontierConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: FrontierConfig) {
        self.config = config;
        info!("Updated frontier configuration");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_frontier_basic_operations() {
        let config = FrontierConfig::default();
        let frontier = FrontierManager::new(config).expect("Failed to create frontier");

        let url = Url::from_str("https://example.com/").expect("Valid URL");
        let request = CrawlRequest::new(url.clone());

        // Add request
        frontier.add_request(request).await.expect("Failed to add request");
        assert_eq!(frontier.size(), 1);

        // Get request
        let next = frontier.next_request().await.expect("Failed to get request");
        assert!(next.is_some());
        assert_eq!(next.unwrap().url, url);
        assert_eq!(frontier.size(), 0);
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let config = FrontierConfig::default();
        let frontier = FrontierManager::new(config).expect("Failed to create frontier");

        // Add requests with different priorities
        let high_url = Url::from_str("https://example.com/high").expect("Valid URL");
        let medium_url = Url::from_str("https://example.com/medium").expect("Valid URL");
        let low_url = Url::from_str("https://example.com/low").expect("Valid URL");

        frontier
            .add_request(CrawlRequest::new(low_url.clone()).with_priority(Priority::Low))
            .await
            .expect("Failed to add low priority");

        frontier
            .add_request(CrawlRequest::new(high_url.clone()).with_priority(Priority::High))
            .await
            .expect("Failed to add high priority");

        frontier
            .add_request(CrawlRequest::new(medium_url.clone()).with_priority(Priority::Medium))
            .await
            .expect("Failed to add medium priority");

        // Should get high priority first
        let first = frontier.next_request().await.expect("Failed to get first").expect("Should have request");
        assert_eq!(first.url, high_url);

        // Then medium
        let second = frontier.next_request().await.expect("Failed to get second").expect("Should have request");
        assert_eq!(second.url, medium_url);

        // Finally low
        let third = frontier.next_request().await.expect("Failed to get third").expect("Should have request");
        assert_eq!(third.url, low_url);
    }

    #[tokio::test]
    async fn test_best_first_scoring() {
        let config = FrontierConfig::default();
        let frontier = FrontierManager::new(config).expect("Failed to create frontier");

        let url1 = Url::from_str("https://example.com/1").expect("Valid URL");
        let url2 = Url::from_str("https://example.com/2").expect("Valid URL");

        // Add requests with scores (higher score should come first)
        frontier
            .add_request(CrawlRequest::new(url1.clone()).with_score(0.5))
            .await
            .expect("Failed to add first");

        frontier
            .add_request(CrawlRequest::new(url2.clone()).with_score(0.8))
            .await
            .expect("Failed to add second");

        // Should get higher score first
        let first = frontier.next_request().await.expect("Failed to get first").expect("Should have request");
        assert_eq!(first.url, url2);

        let second = frontier.next_request().await.expect("Failed to get second").expect("Should have request");
        assert_eq!(second.url, url1);
    }

    #[tokio::test]
    async fn test_host_state_management() {
        let mut host_state = HostState::new("example.com".to_string());

        assert!(host_state.can_accept_request());
        assert_eq!(host_state.success_rate(), 1.0);

        // Start a request
        host_state.start_request().expect("Should accept request");
        assert_eq!(host_state.in_flight, 1);

        // Record success
        host_state.record_success();
        assert_eq!(host_state.in_flight, 0);
        assert_eq!(host_state.pages_crawled, 1);
        assert_eq!(host_state.success_rate(), 1.0);

        // Record error
        host_state.record_error("Test error".to_string());
        assert_eq!(host_state.error_count, 1);
        assert_eq!(host_state.success_rate(), 0.5);
    }
}