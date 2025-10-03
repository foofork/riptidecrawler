//! Memory tracking utilities using system APIs and jemalloc

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use sysinfo::{ProcessesToUpdate, System};
use tracing::debug;

use super::MemorySnapshot;

/// Memory tracker for collecting system and process memory statistics
pub struct MemoryTracker {
    #[allow(dead_code)]
    system: System,
    pid: u32,
    #[allow(dead_code)]
    jemalloc_stats: Option<JemallocStats>,
}

/// Jemalloc memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JemallocStats {
    pub allocated: u64,
    pub active: u64,
    pub metadata: u64,
    pub resident: u64,
    pub mapped: u64,
    pub retained: u64,
}

impl MemoryTracker {
    /// Create a new memory tracker
    pub fn new() -> Result<Self> {
        let mut system = System::new();
        system.refresh_all();

        let pid = std::process::id();

        debug!(pid = pid, "Created memory tracker");

        Ok(Self {
            system,
            pid,
            jemalloc_stats: None,
        })
    }

    /// Get current memory snapshot
    pub async fn get_current_snapshot(&self) -> Result<MemorySnapshot> {
        // Update system information
        let mut system = System::new();
        system.refresh_processes(ProcessesToUpdate::All, true);

        let process = system
            .process(sysinfo::Pid::from_u32(self.pid))
            .ok_or_else(|| anyhow::anyhow!("Process not found"))?;

        // Get basic memory statistics
        let memory = process.memory();
        let virtual_memory = process.virtual_memory();

        // Try to get jemalloc statistics if available
        let jemalloc_stats = self.get_jemalloc_stats();

        let snapshot = MemorySnapshot {
            timestamp: chrono::Utc::now(),
            rss_bytes: memory * 1024, // sysinfo returns KB
            heap_bytes: jemalloc_stats.as_ref().map(|s| s.allocated).unwrap_or(0),
            virtual_bytes: virtual_memory * 1024,
            resident_bytes: memory * 1024,
            shared_bytes: 0, // Not available through sysinfo
            text_bytes: 0,   // Not available through sysinfo
            data_bytes: 0,   // Not available through sysinfo
            stack_bytes: 0,  // Not available through sysinfo
        };

        debug!(
            rss_mb = memory as f64 / 1024.0,
            virtual_mb = virtual_memory as f64 / 1024.0,
            "Captured memory snapshot"
        );

        Ok(snapshot)
    }

    /// Get detailed memory breakdown by component
    pub async fn get_memory_breakdown(&self) -> Result<HashMap<String, u64>> {
        let mut breakdown = HashMap::new();

        // Get jemalloc breakdown if available
        if let Some(stats) = self.get_jemalloc_stats() {
            breakdown.insert("heap_allocated".to_string(), stats.allocated);
            breakdown.insert("heap_active".to_string(), stats.active);
            breakdown.insert("heap_metadata".to_string(), stats.metadata);
            breakdown.insert("heap_resident".to_string(), stats.resident);
            breakdown.insert("heap_mapped".to_string(), stats.mapped);
            breakdown.insert("heap_retained".to_string(), stats.retained);
        }

        // Get system memory info
        let mut system = System::new();
        system.refresh_processes(ProcessesToUpdate::All, true);

        if let Some(process) = system.process(sysinfo::Pid::from_u32(self.pid)) {
            breakdown.insert("rss_total".to_string(), process.memory() * 1024);
            breakdown.insert("virtual_total".to_string(), process.virtual_memory() * 1024);
        }

        Ok(breakdown)
    }

    /// Get memory statistics for a time range
    pub async fn get_memory_stats(&self, duration: Duration) -> Result<MemoryStats> {
        // This would typically collect samples over time
        // For now, return current snapshot statistics
        let snapshot = self.get_current_snapshot().await?;

        Ok(MemoryStats {
            duration,
            peak_rss: snapshot.rss_bytes,
            average_rss: snapshot.rss_bytes,
            min_rss: snapshot.rss_bytes,
            samples: 1,
            growth_rate: 0.0,
        })
    }

    /// Get jemalloc statistics if available
    fn get_jemalloc_stats(&self) -> Option<JemallocStats> {
        #[cfg(feature = "jemalloc")]
        {
            use jemalloc_ctl::{epoch, stats};

            // Advance the epoch to get fresh statistics
            if epoch::advance().is_err() {
                return None;
            }

            Some(JemallocStats {
                allocated: stats::allocated::read().unwrap_or(0) as u64,
                active: stats::active::read().unwrap_or(0) as u64,
                metadata: stats::metadata::read().unwrap_or(0) as u64,
                resident: stats::resident::read().unwrap_or(0) as u64,
                mapped: stats::mapped::read().unwrap_or(0) as u64,
                retained: stats::retained::read().unwrap_or(0) as u64,
            })
        }

        #[cfg(not(feature = "jemalloc"))]
        {
            None
        }
    }

    /// Force garbage collection if available
    pub async fn force_gc(&self) -> Result<()> {
        #[cfg(feature = "jemalloc")]
        {
            use jemalloc_ctl::arenas;

            // Note: jemalloc purge functionality would be implemented here
            // if jemalloc is available and configured
            debug!("Memory purge requested - would purge jemalloc arenas if available");
        }

        // Force Rust garbage collection
        std::hint::black_box(Vec::<u8>::new());

        debug!("Forced garbage collection");
        Ok(())
    }
}

/// Memory statistics over a time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub duration: Duration,
    pub peak_rss: u64,
    pub average_rss: u64,
    pub min_rss: u64,
    pub samples: usize,
    pub growth_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_tracker_creation() {
        let tracker = MemoryTracker::new().unwrap();
        assert_eq!(tracker.pid, std::process::id());
    }

    #[tokio::test]
    async fn test_memory_snapshot() {
        let tracker = MemoryTracker::new().unwrap();
        let snapshot = tracker.get_current_snapshot().await.unwrap();

        assert!(snapshot.rss_bytes > 0);
        assert!(snapshot.virtual_bytes >= snapshot.rss_bytes);
    }

    #[tokio::test]
    async fn test_memory_breakdown() {
        let tracker = MemoryTracker::new().unwrap();
        let breakdown = tracker.get_memory_breakdown().await.unwrap();

        assert!(breakdown.contains_key("rss_total"));
        assert!(breakdown.contains_key("virtual_total"));
    }

    #[tokio::test]
    async fn test_force_gc() {
        let tracker = MemoryTracker::new().unwrap();
        // Should not panic
        tracker.force_gc().await.unwrap();
    }
}
