//! Data models for hybrid headless launcher

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for browser pool management
#[derive(Clone, Debug)]
pub struct PoolConfig {
    /// Initial pool size
    pub initial_size: usize,
    /// Minimum pool size
    pub min_size: usize,
    /// Maximum pool size
    pub max_size: usize,
    /// Browser idle timeout
    pub idle_timeout: Duration,
    /// Health check interval
    pub health_check_interval: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            initial_size: 2,
            min_size: 1,
            max_size: 10,
            idle_timeout: Duration::from_secs(300),
            health_check_interval: Duration::from_secs(60),
        }
    }
}

/// Session statistics
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SessionStats {
    /// Number of pages launched
    pub pages_launched: u64,
    /// Number of successful operations
    pub successful_ops: u64,
    /// Number of failed operations
    pub failed_ops: u64,
    /// Average page load time in milliseconds
    pub avg_page_load_ms: f64,
}

/// Browser capabilities
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BrowserCapabilities {
    /// Whether JavaScript is enabled
    pub javascript_enabled: bool,
    /// Whether images are enabled
    pub images_enabled: bool,
    /// Whether CSS is enabled
    pub css_enabled: bool,
    /// Viewport dimensions
    pub viewport: (u32, u32),
}

impl Default for BrowserCapabilities {
    fn default() -> Self {
        Self {
            javascript_enabled: true,
            images_enabled: true,
            css_enabled: true,
            viewport: (1920, 1080),
        }
    }
}
