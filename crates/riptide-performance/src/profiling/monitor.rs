//! Real-time performance monitoring

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, error, info, warn};

use super::AlertThresholds;

/// Real-time monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub sampling_interval: Duration,
    pub alert_thresholds: AlertThresholds,
    pub buffer_size: usize,
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub severity: AlertSeverity,
    pub category: AlertCategory,
    pub message: String,
    pub current_value: f64,
    pub threshold_value: f64,
    pub recommendations: Vec<String>,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Alert categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCategory {
    Memory,
    Cpu,
    Latency,
    ErrorRate,
    Throughput,
}

/// Real-time performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeMetrics {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub memory_usage_mb: f64,
    pub memory_usage_percent: f64,
    pub cpu_usage_percent: f64,
    pub latency_p99_ms: f64,
    pub error_rate_percent: f64,
    pub throughput_rps: f64,
    pub active_connections: u64,
}

/// Real-time performance monitor
pub struct RealTimeMonitor {
    config: MonitorConfig,
    started: Arc<RwLock<bool>>,
    current_metrics: Arc<RwLock<RealTimeMetrics>>,
    metrics_buffer: Arc<RwLock<VecDeque<RealTimeMetrics>>>,
    alert_sender: mpsc::UnboundedSender<PerformanceAlert>,
    alert_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<PerformanceAlert>>>>,
}

impl RealTimeMonitor {
    pub fn new(config: MonitorConfig) -> crate::Result<Self> {
        let current_metrics = Arc::new(RwLock::new(RealTimeMetrics {
            timestamp: chrono::Utc::now(),
            memory_usage_mb: 0.0,
            memory_usage_percent: 0.0,
            cpu_usage_percent: 0.0,
            latency_p99_ms: 0.0,
            error_rate_percent: 0.0,
            throughput_rps: 0.0,
            active_connections: 0,
        }));

        let (alert_sender, alert_receiver) = mpsc::unbounded_channel();

        Ok(Self {
            config,
            started: Arc::new(RwLock::new(false)),
            current_metrics,
            metrics_buffer: Arc::new(RwLock::new(VecDeque::new())),
            alert_sender,
            alert_receiver: Arc::new(RwLock::new(Some(alert_receiver))),
        })
    }

    pub async fn start(&self) -> crate::Result<()> {
        if *self.started.read().await {
            return Ok(());
        }

        info!("Starting real-time performance monitor");
        *self.started.write().await = true;

        let interval = self.config.sampling_interval;
        let current_metrics = Arc::clone(&self.current_metrics);
        let metrics_buffer = Arc::clone(&self.metrics_buffer);
        let alert_sender = self.alert_sender.clone();
        let thresholds = self.config.alert_thresholds.clone();
        let buffer_size = self.config.buffer_size;
        let started = Arc::clone(&self.started);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);

            while *started.read().await {
                interval.tick().await;

                if let Err(e) = Self::collect_metrics(
                    &current_metrics,
                    &metrics_buffer,
                    &alert_sender,
                    &thresholds,
                    buffer_size,
                ).await {
                    error!("Failed to collect real-time metrics: {}", e);
                }
            }
        });

        Ok(())
    }

    pub async fn stop(&self) -> crate::Result<()> {
        info!("Stopping real-time performance monitor");
        *self.started.write().await = false;
        Ok(())
    }

    pub async fn get_current_metrics(&self) -> crate::Result<RealTimeMetrics> {
        Ok(self.current_metrics.read().await.clone())
    }

    pub async fn get_metrics_history(&self) -> Vec<RealTimeMetrics> {
        self.metrics_buffer.read().await.iter().cloned().collect()
    }

    pub async fn get_alerts(&self) -> crate::Result<Vec<PerformanceAlert>> {
        let mut alerts = Vec::new();
        let mut receiver_guard = self.alert_receiver.write().await;

        if let Some(ref mut receiver) = receiver_guard.as_mut() {
            while let Ok(alert) = receiver.try_recv() {
                alerts.push(alert);
            }
        }

        Ok(alerts)
    }

    async fn collect_metrics(
        current_metrics: &Arc<RwLock<RealTimeMetrics>>,
        metrics_buffer: &Arc<RwLock<VecDeque<RealTimeMetrics>>>,
        alert_sender: &mpsc::UnboundedSender<PerformanceAlert>,
        thresholds: &AlertThresholds,
        buffer_size: usize,
    ) -> crate::Result<()> {
        let now = chrono::Utc::now();

        // Collect system metrics
        let (memory_mb, memory_percent, cpu_percent) = Self::collect_system_metrics().await;

        // Mock application metrics (in real implementation, these would come from actual monitoring)
        let latency_p99 = rand::random::<f64>() * 100.0; // 0-100ms
        let error_rate = rand::random::<f64>() * 2.0; // 0-2%
        let throughput = 100.0 + rand::random::<f64>() * 50.0; // 100-150 RPS
        let connections = (50 + rand::random::<u64>() % 100) as u64; // 50-150 connections

        let metrics = RealTimeMetrics {
            timestamp: now,
            memory_usage_mb: memory_mb,
            memory_usage_percent: memory_percent,
            cpu_usage_percent: cpu_percent,
            latency_p99_ms: latency_p99,
            error_rate_percent: error_rate,
            throughput_rps: throughput,
            active_connections: connections,
        };

        // Update current metrics
        *current_metrics.write().await = metrics.clone();

        // Add to buffer
        {
            let mut buffer = metrics_buffer.write().await;
            buffer.push_back(metrics.clone());
            if buffer.len() > buffer_size {
                buffer.pop_front();
            }
        }

        // Check for alerts
        Self::check_alerts(&metrics, thresholds, alert_sender).await;

        Ok(())
    }

    async fn collect_system_metrics() -> (f64, f64, f64) {
        #[cfg(feature = "system-monitoring")]
        {
            use sysinfo::{System, SystemExt, CpuExt};
            use memory_stats::memory_stats;

            let mut sys = System::new_all();
            sys.refresh_all();

            let cpu_usage = sys.global_cpu_info().cpu_usage() as f64;

            let (memory_mb, memory_percent) = if let Some(usage) = memory_stats() {
                let total_memory = sys.total_memory() as f64;
                let used_memory = usage.physical_mem as f64;
                let memory_mb = used_memory / (1024.0 * 1024.0);
                let memory_percent = (used_memory / total_memory) * 100.0;
                (memory_mb, memory_percent)
            } else {
                (0.0, 0.0)
            };

            (memory_mb, memory_percent, cpu_usage)
        }

        #[cfg(not(feature = "system-monitoring"))]
        {
            // Mock values for when system monitoring is disabled
            let memory_mb = 50.0 + rand::random::<f64>() * 50.0; // 50-100MB
            let memory_percent = 20.0 + rand::random::<f64>() * 40.0; // 20-60%
            let cpu_percent = 10.0 + rand::random::<f64>() * 30.0; // 10-40%
            (memory_mb, memory_percent, cpu_percent)
        }
    }

    async fn check_alerts(
        metrics: &RealTimeMetrics,
        thresholds: &AlertThresholds,
        alert_sender: &mpsc::UnboundedSender<PerformanceAlert>,
    ) {
        // Memory usage alert
        if metrics.memory_usage_percent > thresholds.memory_usage_percent {
            let alert = PerformanceAlert {
                timestamp: metrics.timestamp,
                severity: if metrics.memory_usage_percent > 90.0 {
                    AlertSeverity::Critical
                } else if metrics.memory_usage_percent > 85.0 {
                    AlertSeverity::High
                } else {
                    AlertSeverity::Medium
                },
                category: AlertCategory::Memory,
                message: format!("High memory usage detected: {:.1}%", metrics.memory_usage_percent),
                current_value: metrics.memory_usage_percent,
                threshold_value: thresholds.memory_usage_percent,
                recommendations: vec![
                    "Check for memory leaks".to_string(),
                    "Consider increasing memory limits".to_string(),
                    "Review memory allocation patterns".to_string(),
                ],
            };
            let _ = alert_sender.send(alert);
        }

        // CPU usage alert
        if metrics.cpu_usage_percent > thresholds.cpu_usage_percent {
            let alert = PerformanceAlert {
                timestamp: metrics.timestamp,
                severity: if metrics.cpu_usage_percent > 95.0 {
                    AlertSeverity::Critical
                } else if metrics.cpu_usage_percent > 90.0 {
                    AlertSeverity::High
                } else {
                    AlertSeverity::Medium
                },
                category: AlertCategory::Cpu,
                message: format!("High CPU usage detected: {:.1}%", metrics.cpu_usage_percent),
                current_value: metrics.cpu_usage_percent,
                threshold_value: thresholds.cpu_usage_percent,
                recommendations: vec![
                    "Profile CPU-intensive operations".to_string(),
                    "Consider horizontal scaling".to_string(),
                    "Optimize hot code paths".to_string(),
                ],
            };
            let _ = alert_sender.send(alert);
        }

        // Latency alert
        if metrics.latency_p99_ms > thresholds.latency_ms as f64 {
            let alert = PerformanceAlert {
                timestamp: metrics.timestamp,
                severity: if metrics.latency_p99_ms > (thresholds.latency_ms * 2) as f64 {
                    AlertSeverity::High
                } else {
                    AlertSeverity::Medium
                },
                category: AlertCategory::Latency,
                message: format!("High latency detected: {:.1}ms", metrics.latency_p99_ms),
                current_value: metrics.latency_p99_ms,
                threshold_value: thresholds.latency_ms as f64,
                recommendations: vec![
                    "Analyze slow queries and operations".to_string(),
                    "Check network connectivity".to_string(),
                    "Consider caching strategies".to_string(),
                ],
            };
            let _ = alert_sender.send(alert);
        }

        // Error rate alert
        if metrics.error_rate_percent > thresholds.error_rate_percent {
            let alert = PerformanceAlert {
                timestamp: metrics.timestamp,
                severity: if metrics.error_rate_percent > 10.0 {
                    AlertSeverity::Critical
                } else if metrics.error_rate_percent > 8.0 {
                    AlertSeverity::High
                } else {
                    AlertSeverity::Medium
                },
                category: AlertCategory::ErrorRate,
                message: format!("High error rate detected: {:.1}%", metrics.error_rate_percent),
                current_value: metrics.error_rate_percent,
                threshold_value: thresholds.error_rate_percent,
                recommendations: vec![
                    "Check application logs for errors".to_string(),
                    "Verify external service dependencies".to_string(),
                    "Review recent deployments".to_string(),
                ],
            };
            let _ = alert_sender.send(alert);
        }
    }
}