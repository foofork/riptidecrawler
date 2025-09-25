use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use url::Url;

/// Budget configuration for crawling limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Global budget limits
    pub global: GlobalBudgetLimits,
    /// Per-host budget limits
    pub per_host: PerHostBudgetLimits,
    /// Per-session budget limits
    pub per_session: Option<PerSessionBudgetLimits>,
    /// Enforcement strategy
    pub enforcement: EnforcementStrategy,
    /// Budget monitoring interval
    pub monitoring_interval: Duration,
    /// Enable budget warnings before exhaustion
    pub enable_warnings: bool,
    /// Warning threshold (percentage of budget)
    pub warning_threshold: f64,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            global: GlobalBudgetLimits::default(),
            per_host: PerHostBudgetLimits::default(),
            per_session: None,
            enforcement: EnforcementStrategy::Strict,
            monitoring_interval: Duration::from_secs(60),
            enable_warnings: true,
            warning_threshold: 0.8, // 80%
        }
    }
}

/// Global budget limits that apply to the entire crawl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalBudgetLimits {
    /// Maximum crawl depth from seed URLs
    pub max_depth: Option<u32>,
    /// Maximum number of pages to crawl
    pub max_pages: Option<u64>,
    /// Maximum crawling duration
    pub max_duration: Option<Duration>,
    /// Maximum bandwidth usage in bytes
    pub max_bandwidth: Option<u64>,
    /// Maximum memory usage for frontier
    pub max_memory: Option<usize>,
    /// Maximum concurrent requests
    pub max_concurrent: Option<usize>,
}

impl Default for GlobalBudgetLimits {
    fn default() -> Self {
        Self {
            max_depth: Some(10),
            max_pages: Some(10_000),
            max_duration: Some(Duration::from_secs(3600)), // 1 hour
            max_bandwidth: Some(1_000_000_000), // 1 GB
            max_memory: Some(100_000_000), // 100 MB
            max_concurrent: Some(10),
        }
    }
}

/// Per-host budget limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerHostBudgetLimits {
    /// Maximum pages per host
    pub max_pages_per_host: Option<u64>,
    /// Maximum depth per host
    pub max_depth_per_host: Option<u32>,
    /// Maximum duration per host
    pub max_duration_per_host: Option<Duration>,
    /// Maximum bandwidth per host
    pub max_bandwidth_per_host: Option<u64>,
    /// Maximum concurrent requests per host
    pub max_concurrent_per_host: Option<usize>,
}

impl Default for PerHostBudgetLimits {
    fn default() -> Self {
        Self {
            max_pages_per_host: Some(1000),
            max_depth_per_host: Some(8),
            max_duration_per_host: Some(Duration::from_secs(600)), // 10 minutes
            max_bandwidth_per_host: Some(100_000_000), // 100 MB
            max_concurrent_per_host: Some(2),
        }
    }
}

/// Per-session budget limits for authenticated crawling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerSessionBudgetLimits {
    /// Maximum pages per session
    pub max_pages_per_session: Option<u64>,
    /// Maximum session duration
    pub max_session_duration: Option<Duration>,
    /// Maximum bandwidth per session
    pub max_bandwidth_per_session: Option<u64>,
}

impl Default for PerSessionBudgetLimits {
    fn default() -> Self {
        Self {
            max_pages_per_session: Some(500),
            max_session_duration: Some(Duration::from_secs(1800)), // 30 minutes
            max_bandwidth_per_session: Some(50_000_000), // 50 MB
        }
    }
}

/// Budget enforcement strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnforcementStrategy {
    /// Strict enforcement - stop immediately when any limit is reached
    Strict,
    /// Soft enforcement - log warnings but continue crawling
    Soft,
    /// Adaptive enforcement - adjust crawling rate based on budget consumption
    Adaptive {
        /// Threshold for rate reduction (0.0 to 1.0)
        slowdown_threshold: f64,
        /// Rate reduction factor when threshold is reached
        rate_reduction_factor: f64,
    },
}

impl Default for EnforcementStrategy {
    fn default() -> Self {
        EnforcementStrategy::Adaptive {
            slowdown_threshold: 0.8,
            rate_reduction_factor: 0.5,
        }
    }
}

/// Current budget usage tracking
#[derive(Debug, Clone, Default)]
pub struct BudgetUsage {
    /// Current crawl depth
    pub current_depth: u32,
    /// Pages crawled so far
    pub pages_crawled: u64,
    /// Crawling start time
    pub start_time: Option<Instant>,
    /// Bandwidth used so far
    pub bandwidth_used: u64,
    /// Memory currently in use
    pub memory_used: usize,
    /// Current concurrent requests
    pub concurrent_requests: usize,
}

impl BudgetUsage {
    /// Calculate current crawling duration
    pub fn duration(&self) -> Duration {
        if let Some(start) = self.start_time {
            start.elapsed()
        } else {
            Duration::from_secs(0)
        }
    }

    /// Calculate budget utilization percentage for a specific limit
    pub fn utilization_percentage(&self, limit_type: BudgetLimitType, limit_value: u64) -> f64 {
        let current_value = match limit_type {
            BudgetLimitType::Pages => self.pages_crawled,
            BudgetLimitType::Duration => self.duration().as_secs(),
            BudgetLimitType::Bandwidth => self.bandwidth_used,
            BudgetLimitType::Memory => self.memory_used as u64,
            BudgetLimitType::Concurrent => self.concurrent_requests as u64,
            BudgetLimitType::Depth => self.current_depth as u64,
        };

        if limit_value == 0 {
            0.0
        } else {
            (current_value as f64 / limit_value as f64).min(1.0_f64)
        }
    }
}

/// Types of budget limits
#[derive(Debug, Clone, Copy)]
pub enum BudgetLimitType {
    Pages,
    Duration,
    Bandwidth,
    Memory,
    Concurrent,
    Depth,
}

/// Per-host budget tracking
#[derive(Debug, Clone)]
pub struct HostBudget {
    pub host: String,
    pub usage: BudgetUsage,
    pub limits: PerHostBudgetLimits,
    pub last_activity: Instant,
    pub warnings_issued: Vec<BudgetWarning>,
}

impl HostBudget {
    pub fn new(host: String, limits: PerHostBudgetLimits) -> Self {
        Self {
            host,
            usage: BudgetUsage::default(),
            limits,
            last_activity: Instant::now(),
            warnings_issued: Vec::new(),
        }
    }

    /// Check if any host-specific limit is exceeded
    pub fn is_limit_exceeded(&self) -> Option<BudgetViolation> {
        if let Some(max_pages) = self.limits.max_pages_per_host {
            if self.usage.pages_crawled >= max_pages {
                return Some(BudgetViolation {
                    limit_type: BudgetLimitType::Pages,
                    current_value: self.usage.pages_crawled,
                    limit_value: max_pages,
                    host: Some(self.host.clone()),
                });
            }
        }

        if let Some(max_depth) = self.limits.max_depth_per_host {
            if self.usage.current_depth >= max_depth {
                return Some(BudgetViolation {
                    limit_type: BudgetLimitType::Depth,
                    current_value: self.usage.current_depth as u64,
                    limit_value: max_depth as u64,
                    host: Some(self.host.clone()),
                });
            }
        }

        if let Some(max_duration) = self.limits.max_duration_per_host {
            if self.usage.duration() >= max_duration {
                return Some(BudgetViolation {
                    limit_type: BudgetLimitType::Duration,
                    current_value: self.usage.duration().as_secs(),
                    limit_value: max_duration.as_secs(),
                    host: Some(self.host.clone()),
                });
            }
        }

        if let Some(max_bandwidth) = self.limits.max_bandwidth_per_host {
            if self.usage.bandwidth_used >= max_bandwidth {
                return Some(BudgetViolation {
                    limit_type: BudgetLimitType::Bandwidth,
                    current_value: self.usage.bandwidth_used,
                    limit_value: max_bandwidth,
                    host: Some(self.host.clone()),
                });
            }
        }

        None
    }
}

/// Budget violation information
#[derive(Debug, Clone)]
pub struct BudgetViolation {
    pub limit_type: BudgetLimitType,
    pub current_value: u64,
    pub limit_value: u64,
    pub host: Option<String>,
}

impl BudgetViolation {
    pub fn utilization_percentage(&self) -> f64 {
        if self.limit_value == 0 {
            100.0
        } else {
            (self.current_value as f64 / self.limit_value as f64) * 100.0
        }
    }
}

/// Budget warning information
#[derive(Debug, Clone)]
pub struct BudgetWarning {
    pub limit_type: BudgetLimitType,
    pub utilization_percentage: f64,
    pub issued_at: Instant,
    pub message: String,
}

/// Main budget management system
pub struct BudgetManager {
    config: BudgetConfig,

    // Global budget tracking
    global_usage: Arc<RwLock<BudgetUsage>>,

    // Per-host budget tracking
    host_budgets: Arc<RwLock<HashMap<String, HostBudget>>>,

    // Per-session budget tracking
    #[allow(dead_code)]
    session_budgets: Arc<RwLock<HashMap<String, BudgetUsage>>>,

    // Atomic counters for performance
    pages_crawled: AtomicU64,
    bandwidth_used: AtomicU64,
    concurrent_requests: AtomicUsize,

    // Monitoring
    last_monitoring: Arc<RwLock<Instant>>,
    warnings_issued: Arc<RwLock<Vec<BudgetWarning>>>,
}

impl BudgetManager {
    pub fn new(config: BudgetConfig) -> Self {
        let global_usage = BudgetUsage {
            start_time: Some(Instant::now()),
            ..Default::default()
        };

        Self {
            config,
            global_usage: Arc::new(RwLock::new(global_usage)),
            host_budgets: Arc::new(RwLock::new(HashMap::new())),
            session_budgets: Arc::new(RwLock::new(HashMap::new())),
            pages_crawled: AtomicU64::new(0),
            bandwidth_used: AtomicU64::new(0),
            concurrent_requests: AtomicUsize::new(0),
            last_monitoring: Arc::new(RwLock::new(Instant::now())),
            warnings_issued: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Check if a request can be made within budget constraints
    pub async fn can_make_request(&self, url: &Url, depth: u32) -> Result<bool> {
        // Check global limits first
        if let Some(violation) = self.check_global_limits(depth).await? {
            match self.config.enforcement {
                EnforcementStrategy::Strict => {
                    info!(
                        limit_type = ?violation.limit_type,
                        current = violation.current_value,
                        limit = violation.limit_value,
                        "Global budget limit exceeded"
                    );
                    return Ok(false);
                }
                EnforcementStrategy::Soft => {
                    warn!(
                        limit_type = ?violation.limit_type,
                        current = violation.current_value,
                        limit = violation.limit_value,
                        "Global budget limit exceeded (soft enforcement)"
                    );
                }
                EnforcementStrategy::Adaptive { .. } => {
                    // Continue but will be handled in rate limiting
                }
            }
        }

        // Check per-host limits
        if let Some(host) = url.host_str() {
            if let Some(violation) = self.check_host_limits(host, depth).await? {
                match self.config.enforcement {
                    EnforcementStrategy::Strict => {
                        info!(
                            host = %host,
                            limit_type = ?violation.limit_type,
                            current = violation.current_value,
                            limit = violation.limit_value,
                            "Host budget limit exceeded"
                        );
                        return Ok(false);
                    }
                    EnforcementStrategy::Soft => {
                        warn!(
                            host = %host,
                            limit_type = ?violation.limit_type,
                            current = violation.current_value,
                            limit = violation.limit_value,
                            "Host budget limit exceeded (soft enforcement)"
                        );
                    }
                    EnforcementStrategy::Adaptive { .. } => {
                        // Continue but will be handled in rate limiting
                    }
                }
            }
        }

        // Check concurrent request limit
        let current_concurrent = self.concurrent_requests.load(Ordering::Relaxed);
        if let Some(max_concurrent) = self.config.global.max_concurrent {
            if current_concurrent >= max_concurrent {
                debug!(
                    current = current_concurrent,
                    max = max_concurrent,
                    "Concurrent request limit reached"
                );
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Record the start of a request
    pub async fn start_request(&self, url: &Url, depth: u32) -> Result<()> {
        self.concurrent_requests.fetch_add(1, Ordering::Relaxed);

        // Update global usage
        {
            let mut global = self.global_usage.write().await;
            global.current_depth = global.current_depth.max(depth);
            global.concurrent_requests = self.concurrent_requests.load(Ordering::Relaxed);
        }

        // Update host usage
        if let Some(host) = url.host_str() {
            let mut host_budgets = self.host_budgets.write().await;
            let host_budget = host_budgets
                .entry(host.to_string())
                .or_insert_with(|| HostBudget::new(host.to_string(), self.config.per_host.clone()));

            host_budget.usage.current_depth = host_budget.usage.current_depth.max(depth);
            host_budget.usage.concurrent_requests += 1;
            host_budget.last_activity = Instant::now();
        }

        debug!(url = %url, depth = depth, "Request started");
        Ok(())
    }

    /// Record the completion of a request
    pub async fn complete_request(&self, url: &Url, content_size: usize, success: bool) -> Result<()> {
        self.concurrent_requests.fetch_sub(1, Ordering::Relaxed);

        if success {
            self.pages_crawled.fetch_add(1, Ordering::Relaxed);
            self.bandwidth_used.fetch_add(content_size as u64, Ordering::Relaxed);

            // Update global usage
            {
                let mut global = self.global_usage.write().await;
                global.pages_crawled = self.pages_crawled.load(Ordering::Relaxed);
                global.bandwidth_used = self.bandwidth_used.load(Ordering::Relaxed);
                global.concurrent_requests = self.concurrent_requests.load(Ordering::Relaxed);
            }

            // Update host usage
            if let Some(host) = url.host_str() {
                let mut host_budgets = self.host_budgets.write().await;
                if let Some(host_budget) = host_budgets.get_mut(host) {
                    host_budget.usage.pages_crawled += 1;
                    host_budget.usage.bandwidth_used += content_size as u64;
                    host_budget.usage.concurrent_requests = host_budget.usage.concurrent_requests.saturating_sub(1);
                    host_budget.last_activity = Instant::now();
                }
            }

            debug!(
                url = %url,
                content_size = content_size,
                "Request completed successfully"
            );
        } else {
            debug!(url = %url, "Request completed with error");
        }

        // Check if monitoring is needed
        self.maybe_monitor().await?;

        Ok(())
    }

    /// Check global budget limits
    async fn check_global_limits(&self, depth: u32) -> Result<Option<BudgetViolation>> {
        let global = self.global_usage.read().await;

        if let Some(max_depth) = self.config.global.max_depth {
            if depth >= max_depth {
                return Ok(Some(BudgetViolation {
                    limit_type: BudgetLimitType::Depth,
                    current_value: depth as u64,
                    limit_value: max_depth as u64,
                    host: None,
                }));
            }
        }

        if let Some(max_pages) = self.config.global.max_pages {
            if global.pages_crawled >= max_pages {
                return Ok(Some(BudgetViolation {
                    limit_type: BudgetLimitType::Pages,
                    current_value: global.pages_crawled,
                    limit_value: max_pages,
                    host: None,
                }));
            }
        }

        if let Some(max_duration) = self.config.global.max_duration {
            if global.duration() >= max_duration {
                return Ok(Some(BudgetViolation {
                    limit_type: BudgetLimitType::Duration,
                    current_value: global.duration().as_secs(),
                    limit_value: max_duration.as_secs(),
                    host: None,
                }));
            }
        }

        if let Some(max_bandwidth) = self.config.global.max_bandwidth {
            if global.bandwidth_used >= max_bandwidth {
                return Ok(Some(BudgetViolation {
                    limit_type: BudgetLimitType::Bandwidth,
                    current_value: global.bandwidth_used,
                    limit_value: max_bandwidth,
                    host: None,
                }));
            }
        }

        Ok(None)
    }

    /// Check host-specific budget limits
    async fn check_host_limits(&self, host: &str, _depth: u32) -> Result<Option<BudgetViolation>> {
        let host_budgets = self.host_budgets.read().await;
        if let Some(host_budget) = host_budgets.get(host) {
            return Ok(host_budget.is_limit_exceeded());
        }
        Ok(None)
    }

    /// Perform periodic monitoring and issue warnings
    async fn maybe_monitor(&self) -> Result<()> {
        let mut last_monitoring = self.last_monitoring.write().await;
        if last_monitoring.elapsed() >= self.config.monitoring_interval {
            self.monitor_budgets().await?;
            *last_monitoring = Instant::now();
        }
        Ok(())
    }

    /// Monitor budget usage and issue warnings
    async fn monitor_budgets(&self) -> Result<()> {
        if !self.config.enable_warnings {
            return Ok(());
        }

        let global = self.global_usage.read().await;
        let mut warnings = self.warnings_issued.write().await;

        // Check global budget warnings
        if let Some(max_pages) = self.config.global.max_pages {
            let utilization = global.utilization_percentage(BudgetLimitType::Pages, max_pages);
            if utilization >= self.config.warning_threshold {
                let warning = BudgetWarning {
                    limit_type: BudgetLimitType::Pages,
                    utilization_percentage: utilization * 100.0,
                    issued_at: Instant::now(),
                    message: format!(
                        "Global page budget at {:.1}% utilization ({}/{})",
                        utilization * 100.0,
                        global.pages_crawled,
                        max_pages
                    ),
                };
                warnings.push(warning.clone());
                warn!("{}", warning.message);
            }
        }

        if let Some(max_bandwidth) = self.config.global.max_bandwidth {
            let utilization = global.utilization_percentage(BudgetLimitType::Bandwidth, max_bandwidth);
            if utilization >= self.config.warning_threshold {
                let warning = BudgetWarning {
                    limit_type: BudgetLimitType::Bandwidth,
                    utilization_percentage: utilization * 100.0,
                    issued_at: Instant::now(),
                    message: format!(
                        "Global bandwidth budget at {:.1}% utilization ({} bytes / {} bytes)",
                        utilization * 100.0,
                        global.bandwidth_used,
                        max_bandwidth
                    ),
                };
                warnings.push(warning.clone());
                warn!("{}", warning.message);
            }
        }

        if let Some(max_duration) = self.config.global.max_duration {
            let utilization = global.utilization_percentage(BudgetLimitType::Duration, max_duration.as_secs());
            if utilization >= self.config.warning_threshold {
                let warning = BudgetWarning {
                    limit_type: BudgetLimitType::Duration,
                    utilization_percentage: utilization * 100.0,
                    issued_at: Instant::now(),
                    message: format!(
                        "Global duration budget at {:.1}% utilization ({:.0}s / {:.0}s)",
                        utilization * 100.0,
                        global.duration().as_secs(),
                        max_duration.as_secs()
                    ),
                };
                warnings.push(warning.clone());
                warn!("{}", warning.message);
            }
        }

        Ok(())
    }

    /// Get current global budget usage
    pub async fn get_global_usage(&self) -> BudgetUsage {
        let mut global = self.global_usage.read().await.clone();
        global.pages_crawled = self.pages_crawled.load(Ordering::Relaxed);
        global.bandwidth_used = self.bandwidth_used.load(Ordering::Relaxed);
        global.concurrent_requests = self.concurrent_requests.load(Ordering::Relaxed);
        global
    }

    /// Get usage for a specific host
    pub async fn get_host_usage(&self, host: &str) -> Option<BudgetUsage> {
        let host_budgets = self.host_budgets.read().await;
        host_budgets.get(host).map(|hb| hb.usage.clone())
    }

    /// Get all warnings issued
    pub async fn get_warnings(&self) -> Vec<BudgetWarning> {
        self.warnings_issued.read().await.clone()
    }

    /// Clear all warnings
    pub async fn clear_warnings(&self) {
        self.warnings_issued.write().await.clear();
    }

    /// Get current configuration
    pub fn get_config(&self) -> &BudgetConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: BudgetConfig) {
        self.config = config;
        info!("Budget configuration updated");
    }

    /// Calculate recommended delay based on adaptive enforcement
    pub async fn calculate_adaptive_delay(&self) -> Duration {
        if let EnforcementStrategy::Adaptive { slowdown_threshold, rate_reduction_factor } = &self.config.enforcement {
            let global = self.global_usage.read().await;

            // Check if any budget is approaching the slowdown threshold
            let mut max_utilization = 0.0_f64;

            if let Some(max_pages) = self.config.global.max_pages {
                let utilization = global.utilization_percentage(BudgetLimitType::Pages, max_pages);
                max_utilization = max_utilization.max(utilization);
            }

            if let Some(max_bandwidth) = self.config.global.max_bandwidth {
                let utilization = global.utilization_percentage(BudgetLimitType::Bandwidth, max_bandwidth);
                max_utilization = max_utilization.max(utilization);
            }

            if max_utilization >= *slowdown_threshold {
                let delay_factor = (max_utilization - slowdown_threshold) / (1.0 - slowdown_threshold);
                let max_delay = Duration::from_secs(5); // Maximum 5 second delay
                return Duration::from_secs_f64(max_delay.as_secs_f64() * delay_factor * rate_reduction_factor);
            }
        }

        Duration::from_millis(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_basic_budget_tracking() {
        let config = BudgetConfig::default();
        let budget_manager = BudgetManager::new(config);

        let url = Url::from_str("https://example.com/page").expect("Valid URL");

        // Should be able to make first request
        assert!(budget_manager.can_make_request(&url, 0).await.expect("Should work"));

        // Start and complete request
        budget_manager.start_request(&url, 0).await.expect("Should work");
        budget_manager.complete_request(&url, 1024, true).await.expect("Should work");

        let usage = budget_manager.get_global_usage().await;
        assert_eq!(usage.pages_crawled, 1);
        assert_eq!(usage.bandwidth_used, 1024);
    }

    #[tokio::test]
    async fn test_page_limit_enforcement() {
        let mut config = BudgetConfig::default();
        config.global.max_pages = Some(2);
        config.enforcement = EnforcementStrategy::Strict;

        let budget_manager = BudgetManager::new(config);
        let url = Url::from_str("https://example.com/page").expect("Valid URL");

        // First two requests should work
        for _i in 0..2 {
            assert!(budget_manager.can_make_request(&url, 0).await.expect("Should work"));
            budget_manager.start_request(&url, 0).await.expect("Should work");
            budget_manager.complete_request(&url, 1024, true).await.expect("Should work");
        }

        // Third request should be blocked
        assert!(!budget_manager.can_make_request(&url, 0).await.expect("Should work"));
    }

    #[tokio::test]
    async fn test_depth_limit_enforcement() {
        let mut config = BudgetConfig::default();
        config.global.max_depth = Some(3);
        config.enforcement = EnforcementStrategy::Strict;

        let budget_manager = BudgetManager::new(config);
        let url = Url::from_str("https://example.com/page").expect("Valid URL");

        // Depths 0, 1, 2 should work
        for depth in 0..3 {
            assert!(budget_manager.can_make_request(&url, depth).await.expect("Should work"));
        }

        // Depth 3 should be blocked
        assert!(!budget_manager.can_make_request(&url, 3).await.expect("Should work"));
    }

    #[tokio::test]
    async fn test_host_budget_limits() {
        let mut config = BudgetConfig::default();
        config.per_host.max_pages_per_host = Some(1);
        config.enforcement = EnforcementStrategy::Strict;

        let budget_manager = BudgetManager::new(config);
        let url1 = Url::from_str("https://example.com/page1").expect("Valid URL");
        let url2 = Url::from_str("https://example.com/page2").expect("Valid URL");
        let url3 = Url::from_str("https://other.com/page").expect("Valid URL");

        // First request to example.com should work
        assert!(budget_manager.can_make_request(&url1, 0).await.expect("Should work"));
        budget_manager.start_request(&url1, 0).await.expect("Should work");
        budget_manager.complete_request(&url1, 1024, true).await.expect("Should work");

        // Second request to example.com should be blocked
        assert!(!budget_manager.can_make_request(&url2, 0).await.expect("Should work"));

        // Request to other.com should still work
        assert!(budget_manager.can_make_request(&url3, 0).await.expect("Should work"));
    }

    #[tokio::test]
    async fn test_concurrent_request_limit() {
        let mut config = BudgetConfig::default();
        config.global.max_concurrent = Some(2);

        let budget_manager = BudgetManager::new(config);
        let url = Url::from_str("https://example.com/page").expect("Valid URL");

        // Start two concurrent requests
        assert!(budget_manager.can_make_request(&url, 0).await.expect("Should work"));
        budget_manager.start_request(&url, 0).await.expect("Should work");

        assert!(budget_manager.can_make_request(&url, 0).await.expect("Should work"));
        budget_manager.start_request(&url, 0).await.expect("Should work");

        // Third concurrent request should be blocked
        assert!(!budget_manager.can_make_request(&url, 0).await.expect("Should work"));

        // Complete one request, should free up capacity
        budget_manager.complete_request(&url, 1024, true).await.expect("Should work");
        assert!(budget_manager.can_make_request(&url, 0).await.expect("Should work"));
    }

    #[test]
    fn test_budget_usage_calculations() {
        let usage = BudgetUsage {
            start_time: Some(Instant::now() - Duration::from_secs(60)),
            pages_crawled: 50,
            ..Default::default()
        };

        assert!(usage.duration().as_secs() >= 60);
        assert_eq!(usage.utilization_percentage(BudgetLimitType::Pages, 100), 0.5);
        assert_eq!(usage.utilization_percentage(BudgetLimitType::Pages, 0), 0.0);
    }

    #[tokio::test]
    async fn test_adaptive_delay_calculation() {
        let mut config = BudgetConfig::default();
        config.global.max_pages = Some(100);
        config.enforcement = EnforcementStrategy::Adaptive {
            slowdown_threshold: 0.8,
            rate_reduction_factor: 0.5,
        };

        let budget_manager = BudgetManager::new(config);

        // Simulate 90% budget usage
        for _ in 0..90 {
            let url = Url::from_str("https://example.com/page").expect("Valid URL");
            budget_manager.start_request(&url, 0).await.expect("Should work");
            budget_manager.complete_request(&url, 1024, true).await.expect("Should work");
        }

        let delay = budget_manager.calculate_adaptive_delay().await;
        assert!(delay > Duration::from_millis(0));
    }
}