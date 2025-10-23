//! Adaptive timeout manager implementation

use super::profile::{TimeoutProfile, TimeoutStats};
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use url::Url;

// Re-export constants
pub use super::profile::{
    BACKOFF_MULTIPLIER, DEFAULT_TIMEOUT_SECS, MAX_TIMEOUT_SECS, MIN_TIMEOUT_SECS, SUCCESS_REDUCTION,
};

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
///
/// Thread-safe manager for learning and applying adaptive timeouts across domains.
pub struct AdaptiveTimeoutManager {
    config: TimeoutConfig,
    timeout_profiles: Arc<RwLock<HashMap<String, TimeoutProfile>>>,
    storage_path: PathBuf,
}

impl AdaptiveTimeoutManager {
    /// Create a new adaptive timeout manager
    ///
    /// Loads existing timeout profiles from disk if available.
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
    ///
    /// Returns the learned timeout for the domain, or default timeout if unknown.
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
    ///
    /// Updates the domain's profile and potentially reduces timeout after
    /// consecutive successes.
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
    ///
    /// Updates the domain's profile and increases timeout with exponential backoff.
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

/// Global adaptive timeout manager instance
static GLOBAL_TIMEOUT_MANAGER: tokio::sync::OnceCell<Arc<AdaptiveTimeoutManager>> =
    tokio::sync::OnceCell::const_new();

/// Get or initialize global timeout manager
///
/// Returns a shared reference to the global timeout manager, initializing it
/// with default configuration if not already initialized.
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

    #[tokio::test]
    async fn test_persistence() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = TimeoutConfig {
            storage_path: temp_dir.path().join("timeouts.json"),
            default_timeout_secs: 30,
            auto_save: false,
        };

        // Create manager and record some data
        {
            let manager = AdaptiveTimeoutManager::new(config.clone()).await.unwrap();
            manager
                .record_success("https://example.com", Duration::from_secs(1))
                .await;
            manager.save_profiles().await.unwrap();
        }

        // Create new manager and verify data persisted
        {
            let manager = AdaptiveTimeoutManager::new(config).await.unwrap();
            let profile = manager.get_profile("example.com").await;
            assert!(profile.is_some());
            assert_eq!(profile.unwrap().successful_requests, 1);
        }
    }

    #[tokio::test]
    async fn test_extract_domain() {
        assert_eq!(
            AdaptiveTimeoutManager::extract_domain("https://example.com/path").unwrap(),
            "example.com"
        );
        assert_eq!(
            AdaptiveTimeoutManager::extract_domain("https://api.example.com:8080/path").unwrap(),
            "api.example.com"
        );
        assert!(AdaptiveTimeoutManager::extract_domain("not-a-url").is_err());
    }
}
