//! Memory profiling and allocation tracking

use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// Memory profiling results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryProfile {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub heap_size: u64,
    pub heap_used: u64,
    pub heap_usage_percent: f64,
    pub allocation_rate: u64, // bytes per second
    pub deallocation_rate: u64,
    pub fragmentation_percent: f64,
    pub gc_collections: u64,
    pub gc_total_time: Duration,
}

/// Allocation tracking data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationTracker {
    pub total_allocations: u64,
    pub total_deallocations: u64,
    pub current_allocations: u64,
    pub peak_memory: u64,
    pub allocation_hotspots: Vec<AllocationHotspot>,
}

/// Memory allocation hotspot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationHotspot {
    pub function_name: String,
    pub file_location: String,
    pub allocation_count: u64,
    pub total_bytes: u64,
    pub average_size: f64,
}

/// Memory profiler implementation
pub struct MemoryProfiler {
    sampling_interval: Duration,
    retention_period: Duration,
    started: Arc<RwLock<bool>>,
    current_profile: Arc<RwLock<MemoryProfile>>,
    allocation_tracker: Arc<RwLock<AllocationTracker>>,
}

impl MemoryProfiler {
    pub fn new(sampling_interval: Duration, retention_period: Duration) -> crate::Result<Self> {
        let current_profile = Arc::new(RwLock::new(MemoryProfile {
            timestamp: chrono::Utc::now(),
            heap_size: 0,
            heap_used: 0,
            heap_usage_percent: 0.0,
            allocation_rate: 0,
            deallocation_rate: 0,
            fragmentation_percent: 0.0,
            gc_collections: 0,
            gc_total_time: Duration::from_secs(0),
        }));

        let allocation_tracker = Arc::new(RwLock::new(AllocationTracker {
            total_allocations: 0,
            total_deallocations: 0,
            current_allocations: 0,
            peak_memory: 0,
            allocation_hotspots: Vec::new(),
        }));

        Ok(Self {
            sampling_interval,
            retention_period,
            started: Arc::new(RwLock::new(false)),
            current_profile,
            allocation_tracker,
        })
    }

    pub async fn start(&self) -> crate::Result<()> {
        if *self.started.read().await {
            return Ok(());
        }

        info!("Starting memory profiler");
        *self.started.write().await = true;

        let sampling_interval = self.sampling_interval;
        let current_profile = Arc::clone(&self.current_profile);
        let started = Arc::clone(&self.started);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(sampling_interval);

            while *started.read().await {
                interval.tick().await;

                if let Err(e) = Self::collect_memory_stats(&current_profile).await {
                    error!("Failed to collect memory stats: {}", e);
                }
            }
        });

        Ok(())
    }

    pub async fn stop(&self) -> crate::Result<()> {
        info!("Stopping memory profiler");
        *self.started.write().await = false;
        Ok(())
    }

    pub async fn get_current_profile(&self) -> crate::Result<MemoryProfile> {
        Ok(self.current_profile.read().await.clone())
    }

    async fn collect_memory_stats(current_profile: &Arc<RwLock<MemoryProfile>>) -> crate::Result<()> {
        #[cfg(feature = "memory-profiling")]
        {
            use memory_stats::memory_stats;

            if let Some(usage) = memory_stats() {
                let mut profile = current_profile.write().await;
                profile.timestamp = chrono::Utc::now();
                profile.heap_used = usage.physical_mem as u64;
                profile.heap_size = usage.virtual_mem as u64;
                profile.heap_usage_percent = if profile.heap_size > 0 {
                    (profile.heap_used as f64 / profile.heap_size as f64) * 100.0
                } else {
                    0.0
                };
            }
        }

        #[cfg(not(feature = "memory-profiling"))]
        {
            // Mock implementation for when memory profiling is disabled
            let mut profile = current_profile.write().await;
            profile.timestamp = chrono::Utc::now();
            profile.heap_used = 50 * 1024 * 1024; // 50MB mock
            profile.heap_size = 100 * 1024 * 1024; // 100MB mock
            profile.heap_usage_percent = 50.0;
        }

        Ok(())
    }
}