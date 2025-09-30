//! Monitoring configuration types

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Interval between metric collections
    pub collection_interval: Duration,
    /// Maximum number of metric samples to retain
    pub max_samples: usize,
    /// Enable real-time alerting
    pub enable_alerts: bool,
    /// Alert threshold multipliers
    pub alert_multipliers: AlertMultipliers,
    /// Enable detailed CPU profiling
    pub enable_cpu_profiling: bool,
    /// Enable network monitoring
    pub enable_network_monitoring: bool,
}

/// Alert threshold multipliers for different severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertMultipliers {
    pub warning: f64,   // e.g., 0.8 = alert at 80% of limit
    pub critical: f64,  // e.g., 0.95 = alert at 95% of limit
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            collection_interval: Duration::from_secs(10),
            max_samples: 360, // 1 hour of data at 10s intervals
            enable_alerts: true,
            alert_multipliers: AlertMultipliers {
                warning: 0.8,
                critical: 0.95,
            },
            enable_cpu_profiling: true,
            enable_network_monitoring: true,
        }
    }
}