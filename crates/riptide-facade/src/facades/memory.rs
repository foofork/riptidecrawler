//! Memory facade for system memory monitoring.

use crate::error::RiptideResult;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Clone)]
pub struct MemoryFacade;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsageResponse {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percentage: f32,
    pub pressure_level: String,
    pub recommendations: Vec<String>,
}

impl MemoryFacade {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_memory_usage(&self) -> RiptideResult<MemoryUsageResponse> {
        info!("Getting memory usage");

        #[cfg(target_os = "linux")]
        let (total, available) = self.get_linux_memory()?;

        #[cfg(not(target_os = "linux"))]
        let (total, available) = (8_000_000_000u64, 4_000_000_000u64); // Mock values for non-Linux

        let used = total.saturating_sub(available);
        let usage_pct = (used as f64 / total as f64 * 100.0) as f32;

        let (pressure_level, recommendations) = self.analyze_memory_pressure(usage_pct);

        Ok(MemoryUsageResponse {
            total_bytes: total,
            used_bytes: used,
            available_bytes: available,
            usage_percentage: usage_pct,
            pressure_level,
            recommendations,
        })
    }

    #[cfg(target_os = "linux")]
    fn get_linux_memory(&self) -> RiptideResult<(u64, u64)> {
        use std::fs;

        let meminfo = fs::read_to_string("/proc/meminfo").map_err(|e| {
            crate::RiptideError::Other(anyhow::anyhow!("Failed to read meminfo: {}", e))
        })?;

        let mut total = 0u64;
        let mut available = 0u64;

        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    total = value.parse::<u64>().unwrap_or(0) * 1024;
                }
            } else if line.starts_with("MemAvailable:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    available = value.parse::<u64>().unwrap_or(0) * 1024;
                }
            }
        }

        Ok((total, available))
    }

    fn analyze_memory_pressure(&self, usage_pct: f32) -> (String, Vec<String>) {
        if usage_pct >= 90.0 {
            (
                "Critical".to_string(),
                vec![
                    "Immediate action required".to_string(),
                    "Stop non-essential services".to_string(),
                    "Clear caches".to_string(),
                ],
            )
        } else if usage_pct >= 80.0 {
            (
                "High".to_string(),
                vec![
                    "Consider reducing concurrent operations".to_string(),
                    "Review memory-intensive tasks".to_string(),
                ],
            )
        } else if usage_pct >= 60.0 {
            (
                "Moderate".to_string(),
                vec!["Monitor memory usage trends".to_string()],
            )
        } else {
            ("Normal".to_string(), vec![])
        }
    }
}

impl Default for MemoryFacade {
    fn default() -> Self {
        Self::new()
    }
}
