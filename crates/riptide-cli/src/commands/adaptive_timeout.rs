// Infrastructure for planned features
#![allow(dead_code)]

/// Adaptive Timeout System
///
/// This module provides intelligent timeout management that learns optimal
/// timeouts per domain based on historical success/failure patterns.
///
/// Features:
/// - Track timeout success/failure per domain
/// - Learn optimal timeouts (5s-60s range)
/// - Exponential backoff on failures
/// - Persistent timeout profiles
/// - Automatic timeout adjustment
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use url::Url;

/// Timeout configuration limits
const MIN_TIMEOUT_SECS: u64 = 5;
const MAX_TIMEOUT_SECS: u64 = 60;
const DEFAULT_TIMEOUT_SECS: u64 = 30;
const BACKOFF_MULTIPLIER: f64 = 1.5;
const SUCCESS_REDUCTION: f64 = 0.9;

/// Timeout profile for a specific domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutProfile {
    /// Domain name
    pub domain: String,
    /// Current timeout in seconds
    pub timeout_secs: u64,
    /// Total requests made
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests (timeouts)
    pub failed_requests: u64,
    /// Consecutive successes
    pub consecutive_successes: u32,
    /// Consecutive failures
    pub consecutive_failures: u32,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Last update timestamp
    pub last_updated: u64,
}

impl TimeoutProfile {
    fn new(domain: String) -> Self {
        Self {
            domain,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            consecutive_successes: 0,
            consecutive_failures: 0,
            avg_response_time_ms: 0.0,
            last_updated: Self::current_timestamp(),
        }
    }

    /// Update profile after successful request
    fn record_success(&mut self, response_time: Duration) {
        self.total_requests += 1;
        self.successful_requests += 1;
        self.consecutive_successes += 1;
        self.consecutive_failures = 0;
        self.last_updated = Self::current_timestamp();

        // Update average response time
        let response_ms = response_time.as_millis() as f64;
        if self.avg_response_time_ms == 0.0 {
            self.avg_response_time_ms = response_ms;
        } else {
            // Exponential moving average
            self.avg_response_time_ms = 0.8 * self.avg_response_time_ms + 0.2 * response_ms;
        }

        // Reduce timeout after consecutive successes
        if self.consecutive_successes >= 3 {
            let new_timeout = (self.timeout_secs as f64 * SUCCESS_REDUCTION) as u64;
            self.timeout_secs = new_timeout.max(MIN_TIMEOUT_SECS);
            self.consecutive_successes = 0;

            debug!(
                domain = &self.domain,
                new_timeout = self.timeout_secs,
                "Reduced timeout after consecutive successes"
            );
        }
    }

    /// Update profile after timeout failure
    fn record_timeout(&mut self) {
        self.total_requests += 1;
        self.failed_requests += 1;
        self.consecutive_failures += 1;
        self.consecutive_successes = 0;
        self.last_updated = Self::current_timestamp();

        // Increase timeout with exponential backoff
        let new_timeout = (self.timeout_secs as f64 * BACKOFF_MULTIPLIER) as u64;
        self.timeout_secs = new_timeout.min(MAX_TIMEOUT_SECS);

        warn!(
            domain = &self.domain,
            new_timeout = self.timeout_secs,
            consecutive_failures = self.consecutive_failures,
            "Increased timeout after failure"
        );
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.successful_requests as f64 / self.total_requests as f64) * 100.0
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

/// Adaptive timeout configuration
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// Storage path for timeout profiles
    pub storage_path: PathBuf,
    /// Default timeout for unknown domains
    pub default_timeout_secs: u64,
    /// Enable auto-save after each update
    pub auto_save: bool,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        let storage_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".riptide")
            .join("timeout-profiles.json");

        Self {
            storage_path,
            default_timeout_secs: DEFAULT_TIMEOUT_SECS,
            auto_save: true,
        }
    }
}

/// Adaptive timeout manager
pub struct AdaptiveTimeoutManager {
    config: TimeoutConfig,
    timeout_profiles: Arc<RwLock<HashMap<String, TimeoutProfile>>>,
    storage_path: PathBuf,
}

impl AdaptiveTimeoutManager {
    /// Create a new adaptive timeout manager
    pub async fn new(config: TimeoutConfig) -> Result<Self> {
        let storage_path = config.storage_path.clone();

        // Ensure parent directory exists
        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        info!(storage_path = ?storage_path, "Initializing adaptive timeout manager");

        // Load existing profiles
        let timeout_profiles = Self::load_profiles(&storage_path).await?;

        Ok(Self {
            config,
            timeout_profiles: Arc::new(RwLock::new(timeout_profiles)),
            storage_path,
        })
    }

    /// Get timeout for a specific URL
    pub async fn get_timeout(&self, url: &str) -> Duration {
        let domain = match Self::extract_domain(url) {
            Ok(d) => d,
            Err(_) => return Duration::from_secs(self.config.default_timeout_secs),
        };

        let profiles = self.timeout_profiles.read().await;

        if let Some(profile) = profiles.get(&domain) {
            debug!(
                domain = &domain,
                timeout_secs = profile.timeout_secs,
                success_rate = profile.success_rate(),
                "Using learned timeout"
            );
            Duration::from_secs(profile.timeout_secs)
        } else {
            debug!(
                domain = &domain,
                timeout_secs = self.config.default_timeout_secs,
                "Using default timeout for new domain"
            );
            Duration::from_secs(self.config.default_timeout_secs)
        }
    }

    /// Record successful request
    pub async fn record_success(&self, url: &str, duration: Duration) {
        let domain = match Self::extract_domain(url) {
            Ok(d) => d,
            Err(e) => {
                warn!(error = %e, "Failed to extract domain from URL");
                return;
            }
        };

        let mut profiles = self.timeout_profiles.write().await;

        let profile = profiles
            .entry(domain.clone())
            .or_insert_with(|| TimeoutProfile::new(domain.clone()));

        profile.record_success(duration);

        // Auto-save if enabled
        if self.config.auto_save {
            let storage_path = self.storage_path.clone();
            let profiles_clone = profiles.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::save_profiles_internal(&storage_path, &profiles_clone).await {
                    warn!(error = %e, "Failed to save timeout profiles");
                }
            });
        }
    }

    /// Record timeout failure
    pub async fn record_timeout(&self, url: &str) {
        let domain = match Self::extract_domain(url) {
            Ok(d) => d,
            Err(e) => {
                warn!(error = %e, "Failed to extract domain from URL");
                return;
            }
        };

        let mut profiles = self.timeout_profiles.write().await;

        let profile = profiles
            .entry(domain.clone())
            .or_insert_with(|| TimeoutProfile::new(domain.clone()));

        profile.record_timeout();

        // Auto-save if enabled
        if self.config.auto_save {
            let storage_path = self.storage_path.clone();
            let profiles_clone = profiles.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::save_profiles_internal(&storage_path, &profiles_clone).await {
                    warn!(error = %e, "Failed to save timeout profiles");
                }
            });
        }
    }

    /// Get profile for a specific domain
    pub async fn get_profile(&self, domain: &str) -> Option<TimeoutProfile> {
        let profiles = self.timeout_profiles.read().await;
        profiles.get(domain).cloned()
    }

    /// Get all profiles
    pub async fn get_all_profiles(&self) -> Vec<TimeoutProfile> {
        let profiles = self.timeout_profiles.read().await;
        profiles.values().cloned().collect()
    }

    /// Get statistics across all profiles
    pub async fn get_stats(&self) -> TimeoutStats {
        let profiles = self.timeout_profiles.read().await;

        let total_domains = profiles.len();
        let total_requests: u64 = profiles.values().map(|p| p.total_requests).sum();
        let total_successes: u64 = profiles.values().map(|p| p.successful_requests).sum();
        let total_failures: u64 = profiles.values().map(|p| p.failed_requests).sum();

        let avg_timeout = if !profiles.is_empty() {
            profiles.values().map(|p| p.timeout_secs).sum::<u64>() as f64 / profiles.len() as f64
        } else {
            self.config.default_timeout_secs as f64
        };

        let avg_success_rate = if total_requests > 0 {
            (total_successes as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        TimeoutStats {
            total_domains,
            total_requests,
            total_successes,
            total_failures,
            avg_timeout_secs: avg_timeout,
            avg_success_rate,
        }
    }

    /// Save profiles to disk
    pub async fn save_profiles(&self) -> Result<()> {
        let profiles = self.timeout_profiles.read().await;
        Self::save_profiles_internal(&self.storage_path, &profiles).await
    }

    /// Clear all profiles
    pub async fn clear_profiles(&self) -> Result<()> {
        let mut profiles = self.timeout_profiles.write().await;
        profiles.clear();

        Self::save_profiles_internal(&self.storage_path, &profiles).await?;

        info!("Cleared all timeout profiles");
        Ok(())
    }

    /// Extract domain from URL
    fn extract_domain(url: &str) -> Result<String> {
        let parsed = Url::parse(url)?;
        let domain = parsed
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("No host in URL"))?;
        Ok(domain.to_string())
    }

    /// Load profiles from disk
    async fn load_profiles(path: &Path) -> Result<HashMap<String, TimeoutProfile>> {
        if !path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(path).await?;
        let profiles: HashMap<String, TimeoutProfile> = serde_json::from_str(&content)?;

        info!(
            profile_count = profiles.len(),
            "Loaded timeout profiles from disk"
        );

        Ok(profiles)
    }

    /// Save profiles to disk (internal helper)
    async fn save_profiles_internal(
        path: &Path,
        profiles: &HashMap<String, TimeoutProfile>,
    ) -> Result<()> {
        let content = serde_json::to_string_pretty(profiles)?;

        // Atomic write using temporary file
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, content).await?;
        fs::rename(temp_path, path).await?;

        debug!(
            profile_count = profiles.len(),
            "Saved timeout profiles to disk"
        );

        Ok(())
    }
}

/// Timeout statistics
#[derive(Debug, Clone)]
pub struct TimeoutStats {
    pub total_domains: usize,
    pub total_requests: u64,
    pub total_successes: u64,
    pub total_failures: u64,
    pub avg_timeout_secs: f64,
    pub avg_success_rate: f64,
}

/// Global adaptive timeout manager instance
static GLOBAL_TIMEOUT_MANAGER: tokio::sync::OnceCell<Arc<AdaptiveTimeoutManager>> =
    tokio::sync::OnceCell::const_new();

/// Get or initialize global timeout manager
pub async fn get_global_timeout_manager() -> Result<Arc<AdaptiveTimeoutManager>> {
    GLOBAL_TIMEOUT_MANAGER
        .get_or_try_init(|| async {
            let config = TimeoutConfig::default();
            let manager = AdaptiveTimeoutManager::new(config).await?;
            Ok(Arc::new(manager))
        })
        .await
        .map(Arc::clone)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_timeout_manager_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = TimeoutConfig {
            storage_path: temp_dir.path().join("timeouts.json"),
            default_timeout_secs: 30,
            auto_save: false,
        };

        let manager = AdaptiveTimeoutManager::new(config).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_default_timeout() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = TimeoutConfig {
            storage_path: temp_dir.path().join("timeouts.json"),
            default_timeout_secs: 25,
            auto_save: false,
        };

        let manager = AdaptiveTimeoutManager::new(config).await.unwrap();
        let timeout = manager.get_timeout("https://example.com").await;

        assert_eq!(timeout.as_secs(), 25);
    }

    #[tokio::test]
    async fn test_record_success() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = TimeoutConfig {
            storage_path: temp_dir.path().join("timeouts.json"),
            default_timeout_secs: 30,
            auto_save: false,
        };

        let manager = AdaptiveTimeoutManager::new(config).await.unwrap();
        let url = "https://example.com/page";

        manager.record_success(url, Duration::from_secs(2)).await;

        let profile = manager.get_profile("example.com").await;
        assert!(profile.is_some());

        let profile = profile.unwrap();
        assert_eq!(profile.successful_requests, 1);
        assert_eq!(profile.failed_requests, 0);
    }

    #[tokio::test]
    async fn test_record_timeout() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = TimeoutConfig {
            storage_path: temp_dir.path().join("timeouts.json"),
            default_timeout_secs: 30,
            auto_save: false,
        };

        let manager = AdaptiveTimeoutManager::new(config).await.unwrap();
        let url = "https://slow-site.com/page";

        manager.record_timeout(url).await;

        let profile = manager.get_profile("slow-site.com").await;
        assert!(profile.is_some());

        let profile = profile.unwrap();
        assert_eq!(profile.failed_requests, 1);
        assert!(profile.timeout_secs > 30);
    }

    #[tokio::test]
    async fn test_adaptive_timeout_reduction() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = TimeoutConfig {
            storage_path: temp_dir.path().join("timeouts.json"),
            default_timeout_secs: 30,
            auto_save: false,
        };

        let manager = AdaptiveTimeoutManager::new(config).await.unwrap();
        let url = "https://fast-site.com/page";

        // Record multiple successes
        for _ in 0..4 {
            manager
                .record_success(url, Duration::from_millis(500))
                .await;
        }

        let timeout = manager.get_timeout(url).await;
        assert!(timeout.as_secs() < 30);
    }

    #[tokio::test]
    async fn test_stats() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = TimeoutConfig {
            storage_path: temp_dir.path().join("timeouts.json"),
            default_timeout_secs: 30,
            auto_save: false,
        };

        let manager = AdaptiveTimeoutManager::new(config).await.unwrap();

        manager
            .record_success("https://example.com", Duration::from_secs(1))
            .await;
        manager
            .record_success("https://example.com", Duration::from_secs(2))
            .await;
        manager.record_timeout("https://example.com").await;

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.total_successes, 2);
        assert_eq!(stats.total_failures, 1);
    }
}
