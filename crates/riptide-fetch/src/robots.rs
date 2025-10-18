use anyhow::{Context, Result};
use dashmap::DashMap;
use rand::Rng;
use reqwest::Client;
use robotstxt::DefaultMatcher;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};
use url::Url;

/// Configuration for robots.txt compliance and rate limiting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotsConfig {
    /// Respect robots.txt files (can be disabled for development)
    pub respect_robots: bool,
    /// Default crawl delay in seconds (if not specified in robots.txt)
    pub default_crawl_delay: f64,
    /// Maximum crawl delay to respect (in seconds)
    pub max_crawl_delay: f64,
    /// Default requests per second when no crawl delay is specified
    pub default_rps: f64,
    /// Maximum requests per second (rate limiting cap)
    pub max_rps: f64,
    /// TTL for cached robots.txt files (in seconds)
    pub cache_ttl: u64,
    /// User agent string to use for robots.txt compliance checks
    pub user_agent: String,
    /// Jitter percentage (0.0 to 1.0) to add to request timing
    pub jitter_factor: f64,
    /// Development mode bypass (ignores robots.txt when true)
    pub development_mode: bool,
    /// Timeout for fetching robots.txt files
    pub fetch_timeout: Duration,
}

impl Default for RobotsConfig {
    fn default() -> Self {
        Self {
            respect_robots: true,
            default_crawl_delay: 1.0,
            max_crawl_delay: 10.0,
            default_rps: 2.0,
            max_rps: 10.0,
            cache_ttl: 3600, // 1 hour
            user_agent: "RipTide/1.0".to_string(),
            jitter_factor: 0.2, // ±20%
            development_mode: false,
            fetch_timeout: Duration::from_secs(10),
        }
    }
}

/// Cached robots.txt entry with TTL
#[derive(Debug, Clone)]
struct CachedRobots {
    robots_content: String,
    crawl_delay: Option<f64>,
    cached_at: Instant,
    ttl: Duration,
}

impl CachedRobots {
    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }
}

/// Token bucket for rate limiting
#[derive(Debug)]
struct TokenBucket {
    tokens: AtomicU64,
    capacity: u64,
    refill_rate: f64, // tokens per second
    last_refill: Mutex<Instant>,
}

impl TokenBucket {
    fn new(capacity: u64, refill_rate: f64) -> Self {
        Self {
            tokens: AtomicU64::new(capacity),
            capacity,
            refill_rate,
            last_refill: Mutex::new(Instant::now()),
        }
    }

    async fn try_consume(&self, tokens: u64) -> bool {
        self.refill().await;

        let current = self.tokens.load(Ordering::SeqCst);
        if current >= tokens {
            let new_value = current - tokens;
            self.tokens
                .compare_exchange(current, new_value, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
        } else {
            false
        }
    }

    async fn refill(&self) {
        let mut last_refill = self.last_refill.lock().await;
        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill).as_secs_f64();

        if elapsed > 0.0 {
            let tokens_to_add = (elapsed * self.refill_rate) as u64;
            if tokens_to_add > 0 {
                let current = self.tokens.load(Ordering::SeqCst);
                let new_tokens = (current + tokens_to_add).min(self.capacity);
                self.tokens.store(new_tokens, Ordering::SeqCst);
                *last_refill = now;
            }
        }
    }

    fn update_rate(&self, new_rate: f64) {
        // Note: This is a simplified update - in production you might want more sophisticated logic
        let new_capacity = (new_rate * 60.0) as u64; // 1 minute worth of tokens
        self.tokens
            .store(new_capacity.min(self.capacity), Ordering::SeqCst);
    }
}

/// Per-host rate limiting and robots.txt management
#[derive(Debug)]
pub struct RobotsManager {
    config: RobotsConfig,
    robots_cache: DashMap<String, CachedRobots>,
    rate_limiters: DashMap<String, Arc<TokenBucket>>,
    http_client: Client,
}

impl RobotsManager {
    pub fn new(config: RobotsConfig) -> Result<Self> {
        let http_client = Client::builder()
            .user_agent(&config.user_agent)
            .timeout(config.fetch_timeout)
            .gzip(true)
            .build()
            .context("Failed to create HTTP client for robots manager")?;

        Ok(Self {
            config,
            robots_cache: DashMap::new(),
            rate_limiters: DashMap::new(),
            http_client,
        })
    }

    /// Check if a URL is allowed to be crawled according to robots.txt
    pub async fn is_allowed(&self, url: &str) -> Result<bool> {
        // Development mode bypass
        if self.config.development_mode {
            debug!(url = %url, "Development mode: bypassing robots.txt check");
            return Ok(true);
        }

        // Skip robots.txt check if disabled
        if !self.config.respect_robots {
            debug!(url = %url, "Robots.txt respect disabled");
            return Ok(true);
        }

        let parsed_url = Url::parse(url).context("Failed to parse URL")?;
        let host = parsed_url
            .host_str()
            .context("URL has no host")?
            .to_string();

        // Get or fetch robots.txt for this host
        let robots = self.get_robots_for_host(&host).await?;

        // Check if the path is allowed using DefaultMatcher
        let path = parsed_url.path();
        let mut matcher = DefaultMatcher::default();
        let allowed = matcher.one_agent_allowed_by_robots(
            &robots.robots_content,
            &self.config.user_agent,
            url,
        );

        debug!(url = %url, path = %path, allowed = allowed, "Robots.txt check completed");
        Ok(allowed)
    }

    /// Wait for rate limiting before making a request to a host
    pub async fn wait_for_rate_limit(&self, url: &str) -> Result<()> {
        let parsed_url = Url::parse(url).context("Failed to parse URL")?;
        let host = parsed_url
            .host_str()
            .context("URL has no host")?
            .to_string();

        // Get rate limiter for this host
        let rate_limiter = self.get_rate_limiter_for_host(&host).await?;

        // Try to consume a token
        while !rate_limiter.try_consume(1).await {
            // Calculate delay based on current rate
            let delay = self.calculate_delay(&host).await;
            debug!(host = %host, delay_ms = delay.as_millis(), "Rate limiting: waiting");
            tokio::time::sleep(delay).await;
        }

        Ok(())
    }

    /// Get robots.txt for a host (cached or fetch fresh)
    async fn get_robots_for_host(&self, host: &str) -> Result<CachedRobots> {
        // Check cache first
        if let Some(cached) = self.robots_cache.get(host) {
            if !cached.is_expired() {
                debug!(host = %host, "Using cached robots.txt");
                return Ok(cached.clone());
            }
            debug!(host = %host, "Cached robots.txt expired");
        }

        // Fetch fresh robots.txt
        let robots_url = format!("https://{}/robots.txt", host);
        debug!(host = %host, robots_url = %robots_url, "Fetching robots.txt");

        let robots_content = match self.fetch_robots_txt(&robots_url).await {
            Ok(content) => content,
            Err(e) => {
                warn!(host = %host, error = %e, "Failed to fetch robots.txt, allowing all");
                // If we can't fetch robots.txt, be permissive but conservative
                String::new()
            }
        };

        // Store robots.txt content for later parsing

        // Extract crawl delay for our user agent
        let crawl_delay = self.extract_crawl_delay(&robots_content);

        let cached_robots = CachedRobots {
            robots_content,
            crawl_delay,
            cached_at: Instant::now(),
            ttl: Duration::from_secs(self.config.cache_ttl),
        };

        // Update cache
        self.robots_cache
            .insert(host.to_string(), cached_robots.clone());

        // Update rate limiter if we have a new crawl delay
        if let Some(delay) = crawl_delay {
            self.update_rate_limiter_for_host(host, delay).await;
        }

        info!(host = %host, crawl_delay = ?crawl_delay, "Updated robots.txt cache");
        Ok(cached_robots)
    }

    /// Fetch robots.txt content from URL
    async fn fetch_robots_txt(&self, robots_url: &str) -> Result<String> {
        let response = self
            .http_client
            .get(robots_url)
            .send()
            .await
            .context("Failed to fetch robots.txt")?;

        if response.status().is_success() {
            let content = response
                .text()
                .await
                .context("Failed to read robots.txt content")?;
            Ok(content)
        } else {
            // If robots.txt doesn't exist or returns error, return empty (permissive)
            debug!(robots_url = %robots_url, status = %response.status(), "robots.txt not found or error");
            Ok(String::new())
        }
    }

    /// Extract crawl delay from robots.txt content
    fn extract_crawl_delay(&self, robots_content: &str) -> Option<f64> {
        // Simple parsing for crawl-delay directive
        for line in robots_content.lines() {
            let line = line.trim().to_lowercase();
            if line.starts_with("crawl-delay:") {
                if let Some(delay_str) = line.split(':').nth(1) {
                    if let Ok(delay) = delay_str.trim().parse::<f64>() {
                        // Clamp to reasonable bounds
                        let clamped_delay = delay.max(0.1_f64).min(self.config.max_crawl_delay);
                        debug!(
                            original_delay = delay,
                            clamped_delay = clamped_delay,
                            "Parsed crawl delay"
                        );
                        return Some(clamped_delay);
                    }
                }
            }
        }
        None
    }

    /// Get or create rate limiter for host
    async fn get_rate_limiter_for_host(&self, host: &str) -> Result<Arc<TokenBucket>> {
        if let Some(limiter) = self.rate_limiters.get(host) {
            return Ok(limiter.clone());
        }

        // Create new rate limiter
        let rps = self.config.default_rps.min(self.config.max_rps);
        let capacity = (rps * 60.0) as u64; // 1 minute worth of tokens
        let limiter = Arc::new(TokenBucket::new(capacity, rps));

        self.rate_limiters.insert(host.to_string(), limiter.clone());
        debug!(host = %host, rps = rps, capacity = capacity, "Created new rate limiter");

        Ok(limiter)
    }

    /// Update rate limiter for host based on crawl delay
    async fn update_rate_limiter_for_host(&self, host: &str, crawl_delay: f64) {
        let rps = (1.0 / crawl_delay).min(self.config.max_rps);

        if let Some(limiter) = self.rate_limiters.get(host) {
            limiter.update_rate(rps);
            debug!(host = %host, crawl_delay = crawl_delay, new_rps = rps, "Updated rate limiter");
        }
    }

    /// Calculate delay with jitter
    async fn calculate_delay(&self, host: &str) -> Duration {
        let base_delay = if let Some(cached) = self.robots_cache.get(host) {
            if let Some(crawl_delay) = cached.crawl_delay {
                Duration::from_secs_f64(crawl_delay)
            } else {
                Duration::from_secs_f64(1.0 / self.config.default_rps)
            }
        } else {
            Duration::from_secs_f64(1.0 / self.config.default_rps)
        };

        // Add jitter (±20% by default)
        let jitter_range = base_delay.as_secs_f64() * self.config.jitter_factor;
        let jitter = rand::thread_rng().gen_range(-jitter_range..=jitter_range);
        let final_delay = base_delay.as_secs_f64() + jitter;

        Duration::from_secs_f64(final_delay.max(0.1_f64)) // Minimum 100ms delay
    }

    /// Check if a URL can be crawled and wait for rate limit
    pub async fn can_crawl_with_wait(&self, url: &str) -> Result<bool> {
        // First check if allowed by robots.txt
        if !self.is_allowed(url).await? {
            debug!(url = %url, "URL blocked by robots.txt");
            return Ok(false);
        }

        // Wait for rate limiting
        self.wait_for_rate_limit(url).await?;

        Ok(true)
    }

    /// Get current configuration
    pub fn get_config(&self) -> &RobotsConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, new_config: RobotsConfig) {
        self.config = new_config;
    }

    /// Clear cache (useful for testing or manual refresh)
    pub fn clear_cache(&self) {
        self.robots_cache.clear();
        self.rate_limiters.clear();
        info!("Cleared robots.txt cache and rate limiters");
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        (self.robots_cache.len(), self.rate_limiters.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[test]
    fn test_robots_config_default() {
        let config = RobotsConfig::default();
        assert!(config.respect_robots);
        assert_eq!(config.default_rps, 2.0);
        assert_eq!(config.jitter_factor, 0.2);
        assert!(!config.development_mode);
    }

    #[test]
    fn test_cached_robots_expiry() {
        let cached = CachedRobots {
            robots_content: String::new(),
            crawl_delay: None,
            cached_at: Instant::now() - Duration::from_secs(3700), // Past TTL
            ttl: Duration::from_secs(3600),
        };
        assert!(cached.is_expired());
    }

    #[tokio::test]
    async fn test_token_bucket() {
        let bucket = TokenBucket::new(5, 1.0); // 5 tokens, refill 1 per second

        // Should be able to consume 5 tokens initially
        for _ in 0..5 {
            assert!(bucket.try_consume(1).await);
        }

        // Should fail on 6th
        assert!(!bucket.try_consume(1).await);

        // Wait and refill
        sleep(Duration::from_millis(1100)).await;
        assert!(bucket.try_consume(1).await);
    }

    #[tokio::test]
    async fn test_robots_manager_development_mode() {
        let config = RobotsConfig {
            development_mode: true,
            ..Default::default()
        };
        let manager = RobotsManager::new(config).expect("Failed to create manager for test");

        // Should allow everything in development mode
        let result = manager.is_allowed("https://example.com/blocked-path").await;
        assert!(result.is_ok());
        assert!(result.expect("Should succeed in development mode"));
    }

    #[tokio::test]
    async fn test_crawl_delay_parsing() {
        let manager =
            RobotsManager::new(RobotsConfig::default()).expect("Failed to create manager for test");

        let robots_content = r#"
User-agent: *
Crawl-delay: 2.5
Disallow: /admin
"#;

        let delay = manager.extract_crawl_delay(robots_content);
        assert_eq!(delay, Some(2.5));
    }

    #[tokio::test]
    async fn test_crawl_delay_clamping() {
        let config = RobotsConfig {
            max_crawl_delay: 5.0,
            ..Default::default()
        };
        let manager = RobotsManager::new(config).expect("Failed to create manager for test");

        let robots_content = "Crawl-delay: 100"; // Very high delay
        let delay = manager.extract_crawl_delay(robots_content);
        assert_eq!(delay, Some(5.0)); // Should be clamped to max
    }

    #[test]
    fn test_url_parsing() {
        let url = "https://example.com/path/to/resource";
        let parsed = Url::parse(url).expect("Valid URL for test");
        assert_eq!(parsed.host_str().expect("URL has host"), "example.com");
        assert_eq!(parsed.path(), "/path/to/resource");
    }
}
