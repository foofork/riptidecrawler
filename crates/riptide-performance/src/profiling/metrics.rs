//! Performance metrics collection and analysis

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::{MemoryProfile, CpuProfile, BottleneckReport};
use super::monitor::RealTimeMetrics;

/// Performance snapshot containing all profiling data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub memory_profile: Option<MemoryProfile>,
    pub cpu_profile: Option<CpuProfile>,
    pub bottleneck_report: Option<BottleneckReport>,
    pub current_metrics: Option<RealTimeMetrics>,
}

impl PerformanceSnapshot {
    pub fn new() -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            memory_profile: None,
            cpu_profile: None,
            bottleneck_report: None,
            current_metrics: None,
        }
    }
}

/// Comprehensive metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub collection_period: Duration,
    pub total_samples: u64,
    pub memory_stats: MemoryStats,
    pub cpu_stats: CpuStats,
    pub performance_trends: PerformanceTrends,
    pub health_score: f64, // 0.0 to 100.0
}

/// Memory statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub average_usage_percent: f64,
    pub peak_usage_percent: f64,
    pub allocation_rate_avg: u64,
    pub allocation_rate_peak: u64,
    pub fragmentation_avg: f64,
    pub gc_frequency: f64, // collections per hour
}

/// CPU statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuStats {
    pub average_usage_percent: f64,
    pub peak_usage_percent: f64,
    pub average_load: (f64, f64, f64),
    pub context_switches_per_sec: f64,
    pub efficiency_score: f64, // 0.0 to 100.0
}

/// Performance trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrends {
    pub memory_trend: TrendDirection,
    pub cpu_trend: TrendDirection,
    pub latency_trend: TrendDirection,
    pub throughput_trend: TrendDirection,
    pub stability_score: f64, // 0.0 to 100.0
}

/// Trend direction indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
    Volatile,
}

/// Profiler metrics collector and analyzer
pub struct ProfilerMetrics {
    snapshots: Arc<RwLock<Vec<PerformanceSnapshot>>>,
    collection_start: Instant,
    max_snapshots: usize,
}

impl ProfilerMetrics {
    pub fn new() -> Self {
        Self {
            snapshots: Arc::new(RwLock::new(Vec::new())),
            collection_start: Instant::now(),
            max_snapshots: 1000, // Keep last 1000 snapshots
        }
    }

    pub async fn record_snapshot(&self, snapshot: PerformanceSnapshot) {
        let mut snapshots = self.snapshots.write().await;
        snapshots.push(snapshot);

        // Keep only the most recent snapshots
        if snapshots.len() > self.max_snapshots {
            snapshots.remove(0);
        }
    }

    pub async fn get_latest_snapshot(&self) -> Option<PerformanceSnapshot> {
        let snapshots = self.snapshots.read().await;
        snapshots.last().cloned()
    }

    pub async fn get_snapshots_since(&self, since: chrono::DateTime<chrono::Utc>) -> Vec<PerformanceSnapshot> {
        let snapshots = self.snapshots.read().await;
        snapshots.iter()
            .filter(|snapshot| snapshot.timestamp >= since)
            .cloned()
            .collect()
    }

    pub async fn get_summary(&self) -> crate::Result<MetricsSummary> {
        let snapshots = self.snapshots.read().await;

        if snapshots.is_empty() {
            return Ok(MetricsSummary {
                collection_period: self.collection_start.elapsed(),
                total_samples: 0,
                memory_stats: MemoryStats {
                    average_usage_percent: 0.0,
                    peak_usage_percent: 0.0,
                    allocation_rate_avg: 0,
                    allocation_rate_peak: 0,
                    fragmentation_avg: 0.0,
                    gc_frequency: 0.0,
                },
                cpu_stats: CpuStats {
                    average_usage_percent: 0.0,
                    peak_usage_percent: 0.0,
                    average_load: (0.0, 0.0, 0.0),
                    context_switches_per_sec: 0.0,
                    efficiency_score: 100.0,
                },
                performance_trends: PerformanceTrends {
                    memory_trend: TrendDirection::Stable,
                    cpu_trend: TrendDirection::Stable,
                    latency_trend: TrendDirection::Stable,
                    throughput_trend: TrendDirection::Stable,
                    stability_score: 100.0,
                },
                health_score: 100.0,
            });
        }

        let memory_stats = self.calculate_memory_stats(&snapshots).await;
        let cpu_stats = self.calculate_cpu_stats(&snapshots).await;
        let performance_trends = self.calculate_trends(&snapshots).await;
        let health_score = self.calculate_health_score(&memory_stats, &cpu_stats, &performance_trends).await;

        Ok(MetricsSummary {
            collection_period: self.collection_start.elapsed(),
            total_samples: snapshots.len() as u64,
            memory_stats,
            cpu_stats,
            performance_trends,
            health_score,
        })
    }

    async fn calculate_memory_stats(&self, snapshots: &[PerformanceSnapshot]) -> MemoryStats {
        let memory_profiles: Vec<_> = snapshots.iter()
            .filter_map(|s| s.memory_profile.as_ref())
            .collect();

        if memory_profiles.is_empty() {
            return MemoryStats {
                average_usage_percent: 0.0,
                peak_usage_percent: 0.0,
                allocation_rate_avg: 0,
                allocation_rate_peak: 0,
                fragmentation_avg: 0.0,
                gc_frequency: 0.0,
            };
        }

        let usage_percentages: Vec<f64> = memory_profiles.iter()
            .map(|p| p.heap_usage_percent)
            .collect();

        let allocation_rates: Vec<u64> = memory_profiles.iter()
            .map(|p| p.allocation_rate)
            .collect();

        let fragmentation_percentages: Vec<f64> = memory_profiles.iter()
            .map(|p| p.fragmentation_percent)
            .collect();

        let gc_collections: Vec<u64> = memory_profiles.iter()
            .map(|p| p.gc_collections)
            .collect();

        MemoryStats {
            average_usage_percent: usage_percentages.iter().sum::<f64>() / usage_percentages.len() as f64,
            peak_usage_percent: usage_percentages.iter().copied().fold(0.0f64, f64::max),
            allocation_rate_avg: allocation_rates.iter().sum::<u64>() / allocation_rates.len() as u64,
            allocation_rate_peak: *allocation_rates.iter().max().unwrap_or(&0),
            fragmentation_avg: fragmentation_percentages.iter().sum::<f64>() / fragmentation_percentages.len() as f64,
            gc_frequency: if !gc_collections.is_empty() {
                let total_collections = gc_collections.iter().sum::<u64>();
                let time_hours = self.collection_start.elapsed().as_secs_f64() / 3600.0;
                total_collections as f64 / time_hours.max(1.0)
            } else {
                0.0
            },
        }
    }

    async fn calculate_cpu_stats(&self, snapshots: &[PerformanceSnapshot]) -> CpuStats {
        let cpu_profiles: Vec<_> = snapshots.iter()
            .filter_map(|s| s.cpu_profile.as_ref())
            .collect();

        if cpu_profiles.is_empty() {
            return CpuStats {
                average_usage_percent: 0.0,
                peak_usage_percent: 0.0,
                average_load: (0.0, 0.0, 0.0),
                context_switches_per_sec: 0.0,
                efficiency_score: 100.0,
            };
        }

        let usage_percentages: Vec<f64> = cpu_profiles.iter()
            .map(|p| p.cpu_usage_percent)
            .collect();

        let load_averages: Vec<(f64, f64, f64)> = cpu_profiles.iter()
            .map(|p| p.load_average)
            .collect();

        let context_switches: Vec<u64> = cpu_profiles.iter()
            .map(|p| p.context_switches)
            .collect();

        let avg_usage = usage_percentages.iter().sum::<f64>() / usage_percentages.len() as f64;
        let peak_usage = usage_percentages.iter().copied().fold(0.0f64, f64::max);

        let avg_load = if !load_averages.is_empty() {
            let sum_1min = load_averages.iter().map(|l| l.0).sum::<f64>();
            let sum_5min = load_averages.iter().map(|l| l.1).sum::<f64>();
            let sum_15min = load_averages.iter().map(|l| l.2).sum::<f64>();
            let count = load_averages.len() as f64;
            (sum_1min / count, sum_5min / count, sum_15min / count)
        } else {
            (0.0, 0.0, 0.0)
        };

        let efficiency_score = if avg_usage > 0.0 {
            ((100.0 - (peak_usage - avg_usage).abs()) * (1.0 - avg_usage / 100.0)).max(0.0)
        } else {
            100.0
        };

        CpuStats {
            average_usage_percent: avg_usage,
            peak_usage_percent: peak_usage,
            average_load: avg_load,
            context_switches_per_sec: if !context_switches.is_empty() {
                let total_switches = context_switches.iter().sum::<u64>();
                let time_seconds = self.collection_start.elapsed().as_secs_f64();
                total_switches as f64 / time_seconds.max(1.0)
            } else {
                0.0
            },
            efficiency_score,
        }
    }

    async fn calculate_trends(&self, snapshots: &[PerformanceSnapshot]) -> PerformanceTrends {
        if snapshots.len() < 10 {
            return PerformanceTrends {
                memory_trend: TrendDirection::Stable,
                cpu_trend: TrendDirection::Stable,
                latency_trend: TrendDirection::Stable,
                throughput_trend: TrendDirection::Stable,
                stability_score: 100.0,
            };
        }

        // Calculate trends using simple linear regression on recent data
        let recent_snapshots = &snapshots[snapshots.len().saturating_sub(20)..];

        let memory_trend = self.calculate_memory_trend(recent_snapshots).await;
        let cpu_trend = self.calculate_cpu_trend(recent_snapshots).await;
        let latency_trend = self.calculate_latency_trend(recent_snapshots).await;
        let throughput_trend = self.calculate_throughput_trend(recent_snapshots).await;

        let stability_score = self.calculate_stability_score(recent_snapshots).await;

        PerformanceTrends {
            memory_trend,
            cpu_trend,
            latency_trend,
            throughput_trend,
            stability_score,
        }
    }

    async fn calculate_memory_trend(&self, snapshots: &[PerformanceSnapshot]) -> TrendDirection {
        let memory_values: Vec<f64> = snapshots.iter()
            .filter_map(|s| s.memory_profile.as_ref())
            .map(|p| p.heap_usage_percent)
            .collect();

        self.analyze_trend(&memory_values).await
    }

    async fn calculate_cpu_trend(&self, snapshots: &[PerformanceSnapshot]) -> TrendDirection {
        let cpu_values: Vec<f64> = snapshots.iter()
            .filter_map(|s| s.cpu_profile.as_ref())
            .map(|p| p.cpu_usage_percent)
            .collect();

        self.analyze_trend(&cpu_values).await
    }

    async fn calculate_latency_trend(&self, snapshots: &[PerformanceSnapshot]) -> TrendDirection {
        let latency_values: Vec<f64> = snapshots.iter()
            .filter_map(|s| s.current_metrics.as_ref())
            .map(|m| m.latency_p99_ms)
            .collect();

        self.analyze_trend(&latency_values).await
    }

    async fn calculate_throughput_trend(&self, snapshots: &[PerformanceSnapshot]) -> TrendDirection {
        let throughput_values: Vec<f64> = snapshots.iter()
            .filter_map(|s| s.current_metrics.as_ref())
            .map(|m| m.throughput_rps)
            .collect();

        // For throughput, improving means increasing
        match self.analyze_trend(&throughput_values).await {
            TrendDirection::Improving => TrendDirection::Degrading,
            TrendDirection::Degrading => TrendDirection::Improving,
            other => other,
        }
    }

    async fn analyze_trend(&self, values: &[f64]) -> TrendDirection {
        if values.len() < 5 {
            return TrendDirection::Stable;
        }

        // Calculate linear regression slope
        let n = values.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = values.iter().sum::<f64>() / n;

        let numerator: f64 = values.iter().enumerate()
            .map(|(i, &y)| (i as f64 - x_mean) * (y - y_mean))
            .sum();

        let denominator: f64 = (0..values.len())
            .map(|i| (i as f64 - x_mean).powi(2))
            .sum();

        if denominator.abs() < 1e-10 {
            return TrendDirection::Stable;
        }

        let slope = numerator / denominator;
        let relative_slope = slope / y_mean.abs().max(1.0);

        // Calculate variance to detect volatility
        let variance = values.iter()
            .map(|&x| (x - y_mean).powi(2))
            .sum::<f64>() / n;
        let std_dev = variance.sqrt();
        let coefficient_of_variation = std_dev / y_mean.abs().max(1.0);

        if coefficient_of_variation > 0.3 {
            TrendDirection::Volatile
        } else if relative_slope > 0.01 {
            TrendDirection::Degrading
        } else if relative_slope < -0.01 {
            TrendDirection::Improving
        } else {
            TrendDirection::Stable
        }
    }

    async fn calculate_stability_score(&self, snapshots: &[PerformanceSnapshot]) -> f64 {
        if snapshots.len() < 5 {
            return 100.0;
        }

        let mut total_volatility = 0.0;
        let mut metric_count = 0;

        // Memory stability
        let memory_values: Vec<f64> = snapshots.iter()
            .filter_map(|s| s.memory_profile.as_ref())
            .map(|p| p.heap_usage_percent)
            .collect();

        if !memory_values.is_empty() {
            total_volatility += self.calculate_coefficient_of_variation(&memory_values).await;
            metric_count += 1;
        }

        // CPU stability
        let cpu_values: Vec<f64> = snapshots.iter()
            .filter_map(|s| s.cpu_profile.as_ref())
            .map(|p| p.cpu_usage_percent)
            .collect();

        if !cpu_values.is_empty() {
            total_volatility += self.calculate_coefficient_of_variation(&cpu_values).await;
            metric_count += 1;
        }

        // Latency stability
        let latency_values: Vec<f64> = snapshots.iter()
            .filter_map(|s| s.current_metrics.as_ref())
            .map(|m| m.latency_p99_ms)
            .collect();

        if !latency_values.is_empty() {
            total_volatility += self.calculate_coefficient_of_variation(&latency_values).await;
            metric_count += 1;
        }

        if metric_count == 0 {
            return 100.0;
        }

        let average_volatility = total_volatility / metric_count as f64;
        // Convert coefficient of variation to stability score (lower volatility = higher stability)
        (100.0 * (1.0 - average_volatility.min(1.0))).max(0.0)
    }

    async fn calculate_coefficient_of_variation(&self, values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        if mean.abs() < 1e-10 {
            0.0
        } else {
            std_dev / mean.abs()
        }
    }

    async fn calculate_health_score(
        &self,
        memory_stats: &MemoryStats,
        cpu_stats: &CpuStats,
        trends: &PerformanceTrends,
    ) -> f64 {
        let mut score = 100.0;

        // Memory health (30% of total score)
        let memory_penalty = if memory_stats.average_usage_percent > 80.0 {
            (memory_stats.average_usage_percent - 80.0) * 0.5
        } else {
            0.0
        };
        score -= memory_penalty * 0.3;

        // CPU health (30% of total score)
        let cpu_penalty = if cpu_stats.average_usage_percent > 80.0 {
            (cpu_stats.average_usage_percent - 80.0) * 0.5
        } else {
            0.0
        };
        score -= cpu_penalty * 0.3;

        // Efficiency (20% of total score)
        let efficiency_bonus = (cpu_stats.efficiency_score - 50.0) * 0.004; // -0.2 to +0.2
        score += efficiency_bonus * 20.0;

        // Stability (20% of total score)
        let stability_contribution = trends.stability_score * 0.2;
        score = score * 0.8 + stability_contribution;

        score.max(0.0).min(100.0)
    }

    pub async fn clear_snapshots(&self) {
        let mut snapshots = self.snapshots.write().await;
        snapshots.clear();
    }

    pub async fn get_snapshot_count(&self) -> usize {
        self.snapshots.read().await.len()
    }
}