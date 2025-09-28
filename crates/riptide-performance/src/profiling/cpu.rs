//! CPU profiling and performance analysis

use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// CPU profiling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingConfig {
    pub interval: Duration,
    pub duration: Duration,
    pub stack_trace_depth: usize,
}

/// CPU profiling results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuProfile {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cpu_usage_percent: f64,
    pub user_time_percent: f64,
    pub system_time_percent: f64,
    pub idle_time_percent: f64,
    pub context_switches: u64,
    pub interrupts: u64,
    pub load_average: (f64, f64, f64), // 1min, 5min, 15min
}

/// CPU profiler implementation
pub struct CpuProfiler {
    config: SamplingConfig,
    started: Arc<RwLock<bool>>,
    current_profile: Arc<RwLock<CpuProfile>>,
}

impl CpuProfiler {
    pub fn new(config: SamplingConfig) -> crate::Result<Self> {
        let current_profile = Arc::new(RwLock::new(CpuProfile {
            timestamp: chrono::Utc::now(),
            cpu_usage_percent: 0.0,
            user_time_percent: 0.0,
            system_time_percent: 0.0,
            idle_time_percent: 100.0,
            context_switches: 0,
            interrupts: 0,
            load_average: (0.0, 0.0, 0.0),
        }));

        Ok(Self {
            config,
            started: Arc::new(RwLock::new(false)),
            current_profile,
        })
    }

    pub async fn start(&self) -> crate::Result<()> {
        if *self.started.read().await {
            return Ok(());
        }

        info!("Starting CPU profiler");
        *self.started.write().await = true;

        let interval = self.config.interval;
        let current_profile = Arc::clone(&self.current_profile);
        let started = Arc::clone(&self.started);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);

            while *started.read().await {
                interval.tick().await;

                if let Err(e) = Self::collect_cpu_stats(&current_profile).await {
                    error!("Failed to collect CPU stats: {}", e);
                }
            }
        });

        Ok(())
    }

    pub async fn stop(&self) -> crate::Result<()> {
        info!("Stopping CPU profiler");
        *self.started.write().await = false;
        Ok(())
    }

    pub async fn get_current_profile(&self) -> crate::Result<CpuProfile> {
        Ok(self.current_profile.read().await.clone())
    }

    async fn collect_cpu_stats(current_profile: &Arc<RwLock<CpuProfile>>) -> crate::Result<()> {
        #[cfg(feature = "cpu-profiling")]
        {
            use sysinfo::{System, SystemExt, CpuExt};

            let mut sys = System::new_all();
            sys.refresh_cpu();

            tokio::time::sleep(Duration::from_millis(100)).await;
            sys.refresh_cpu();

            let cpus = sys.cpus();
            let cpu_usage = if !cpus.is_empty() {
                cpus.iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / cpus.len() as f32
            } else {
                0.0
            };

            let load_avg = sys.load_average();

            let mut profile = current_profile.write().await;
            profile.timestamp = chrono::Utc::now();
            profile.cpu_usage_percent = cpu_usage as f64;
            profile.load_average = (load_avg.one, load_avg.five, load_avg.fifteen);
            profile.idle_time_percent = 100.0 - profile.cpu_usage_percent;
        }

        #[cfg(not(feature = "cpu-profiling"))]
        {
            // Mock implementation for when CPU profiling is disabled
            let mut profile = current_profile.write().await;
            profile.timestamp = chrono::Utc::now();
            profile.cpu_usage_percent = 25.0; // Mock 25% usage
            profile.user_time_percent = 20.0;
            profile.system_time_percent = 5.0;
            profile.idle_time_percent = 75.0;
            profile.load_average = (0.5, 0.3, 0.2);
        }

        Ok(())
    }
}