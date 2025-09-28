//! Real-time performance monitoring module
//!
//! This module provides continuous monitoring of system performance metrics,
//! including latency, throughput, memory usage, and resource utilization.

use crate::{PerformanceError, PerformanceMetrics, PerformanceTargets, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, watch};
use tracing::{info, warn, debug};
use uuid::Uuid;

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

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub id: Uuid,
    pub severity: AlertSeverity,
    pub metric: String,
    pub current_value: f64,
    pub threshold_value: f64,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub acknowledged: bool,
}

/// System resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cpu_usage_percent: f64,
    pub memory_rss_mb: f64,
    pub memory_heap_mb: f64,
    pub memory_virtual_mb: f64,
    pub disk_read_mbps: f64,
    pub disk_write_mbps: f64,
    pub network_in_mbps: f64,
    pub network_out_mbps: f64,
    pub open_file_descriptors: u32,
    pub thread_count: u32,
}

/// Application-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationMetrics {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub active_requests: u32,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub cache_hit_rate: f64,
    pub cache_size_mb: f64,
    pub ai_processing_queue_size: u32,
    pub ai_avg_processing_time_ms: f64,
}

/// Monitoring session report
#[derive(Debug, Serialize, Deserialize)]
pub struct MonitoringReport {
    pub session_id: Uuid,
    pub monitoring_duration: Duration,
    pub total_samples: usize,
    pub metrics: PerformanceMetrics,
    pub alerts: Vec<PerformanceAlert>,
    pub system_summary: SystemSummary,
    pub application_summary: ApplicationSummary,
    pub recommendations: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// System metrics summary
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemSummary {
    pub avg_cpu_usage: f64,
    pub peak_cpu_usage: f64,
    pub avg_memory_mb: f64,
    pub peak_memory_mb: f64,
    pub total_disk_io_mb: f64,
    pub total_network_io_mb: f64,
    pub uptime: Duration,
}

/// Application metrics summary
#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationSummary {
    pub total_requests_processed: u64,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub peak_concurrent_requests: u32,
    pub cache_efficiency: f64,
    pub ai_processing_efficiency: f64,
}

/// Real-time performance monitor
pub struct PerformanceMonitor {
    config: MonitoringConfig,
    targets: PerformanceTargets,
    session_id: Uuid,
    start_time: Option<Instant>,

    // Metric storage
    system_metrics: Arc<RwLock<VecDeque<SystemMetrics>>>,
    application_metrics: Arc<RwLock<VecDeque<ApplicationMetrics>>>,
    performance_metrics: Arc<RwLock<VecDeque<PerformanceMetrics>>>,

    // Alert management
    active_alerts: Arc<RwLock<HashMap<String, PerformanceAlert>>>,
    alert_sender: Option<watch::Sender<PerformanceAlert>>,

    // Monitoring state
    is_monitoring: Arc<RwLock<bool>>,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(targets: PerformanceTargets) -> Result<Self> {
        Self::with_config(targets, MonitoringConfig::default())
    }

    /// Create a new performance monitor with custom configuration
    pub fn with_config(targets: PerformanceTargets, config: MonitoringConfig) -> Result<Self> {
        let session_id = Uuid::new_v4();

        info!(
            session_id = %session_id,
            "Creating performance monitor with config: {:?}",
            config
        );

        let (alert_sender, _) = watch::channel(PerformanceAlert {
            id: Uuid::new_v4(),
            severity: AlertSeverity::Info,
            metric: "system".to_string(),
            current_value: 0.0,
            threshold_value: 0.0,
            message: "Monitor initialized".to_string(),
            timestamp: chrono::Utc::now(),
            acknowledged: true,
        });

        Ok(Self {
            config,
            targets,
            session_id,
            start_time: None,
            system_metrics: Arc::new(RwLock::new(VecDeque::new())),
            application_metrics: Arc::new(RwLock::new(VecDeque::new())),
            performance_metrics: Arc::new(RwLock::new(VecDeque::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_sender: Some(alert_sender),
            is_monitoring: Arc::new(RwLock::new(false)),
        })
    }

    /// Start monitoring
    pub async fn start(&mut self) -> Result<()> {
        let mut is_monitoring = self.is_monitoring.write().await;
        if *is_monitoring {
            warn!(session_id = %self.session_id, "Performance monitoring already started");
            return Ok(());
        }

        info!(session_id = %self.session_id, "Starting performance monitoring");

        self.start_time = Some(Instant::now());
        *is_monitoring = true;

        // Start monitoring tasks
        self.start_monitoring_tasks().await?;

        info!(session_id = %self.session_id, "Performance monitoring started successfully");
        Ok(())
    }

    /// Stop monitoring and generate report
    pub async fn stop(&mut self) -> Result<MonitoringReport> {
        let mut is_monitoring = self.is_monitoring.write().await;
        if !*is_monitoring {
            warn!(session_id = %self.session_id, "Performance monitoring not running");
            return Err(PerformanceError::MonitoringError("Monitoring not running".to_string()).into());
        }

        info!(session_id = %self.session_id, "Stopping performance monitoring");

        *is_monitoring = false;

        let monitoring_duration = self.start_time
            .map(|start| start.elapsed())
            .unwrap_or_default();

        // Generate comprehensive report
        let report = self.generate_monitoring_report(monitoring_duration).await?;

        info!(
            session_id = %self.session_id,
            duration_ms = monitoring_duration.as_millis(),
            "Performance monitoring stopped successfully"
        );

        Ok(report)
    }

    /// Get current performance metrics
    pub async fn get_current_metrics(&self) -> Result<PerformanceMetrics> {
        let system_metrics = self.collect_system_metrics().await?;
        let app_metrics = self.collect_application_metrics().await?;

        Ok(PerformanceMetrics {
            timestamp: chrono::Utc::now(),
            session_id: self.session_id,

            // Latency metrics (calculated from application metrics)
            latency_p50_ms: app_metrics.avg_response_time_ms * 0.8,
            latency_p95_ms: app_metrics.p95_response_time_ms,
            latency_p99_ms: app_metrics.p95_response_time_ms * 1.2,
            avg_latency_ms: app_metrics.avg_response_time_ms,

            // Memory metrics
            memory_rss_mb: system_metrics.memory_rss_mb,
            memory_heap_mb: system_metrics.memory_heap_mb,
            memory_virtual_mb: system_metrics.memory_virtual_mb,
            memory_growth_rate_mb_s: self.calculate_memory_growth_rate().await,

            // Throughput metrics
            throughput_pps: self.calculate_throughput().await,
            successful_requests: app_metrics.successful_requests,
            failed_requests: app_metrics.failed_requests,
            total_requests: app_metrics.total_requests,

            // AI processing metrics
            ai_processing_time_ms: app_metrics.ai_avg_processing_time_ms,
            ai_overhead_percent: self.calculate_ai_overhead().await,
            ai_cache_hit_rate: app_metrics.cache_hit_rate,

            // Resource utilization
            cpu_usage_percent: system_metrics.cpu_usage_percent,
            network_io_mbps: system_metrics.network_in_mbps + system_metrics.network_out_mbps,
            disk_io_mbps: system_metrics.disk_read_mbps + system_metrics.disk_write_mbps,

            // Cache metrics
            cache_hit_rate: app_metrics.cache_hit_rate,
            cache_size_mb: app_metrics.cache_size_mb,
            cache_evictions: 0, // Would be tracked separately in real implementation
        })
    }

    /// Get alert subscription receiver
    pub fn subscribe_to_alerts(&self) -> Option<watch::Receiver<PerformanceAlert>> {
        self.alert_sender.as_ref().map(|sender| sender.subscribe())
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(&self, alert_id: Uuid) -> Result<()> {
        let mut alerts = self.active_alerts.write().await;

        for alert in alerts.values_mut() {
            if alert.id == alert_id {
                alert.acknowledged = true;
                info!(
                    session_id = %self.session_id,
                    alert_id = %alert_id,
                    "Alert acknowledged"
                );
                return Ok(());
            }
        }

        Err(PerformanceError::MonitoringError(format!("Alert {} not found", alert_id)).into())
    }

    /// Start background monitoring tasks
    async fn start_monitoring_tasks(&self) -> Result<()> {
        let session_id = self.session_id;
        let collection_interval = self.config.collection_interval;
        let max_samples = self.config.max_samples;
        let targets = self.targets.clone();
        let config = self.config.clone();

        // Clone Arc references for background tasks
        let is_monitoring = Arc::clone(&self.is_monitoring);
        let system_metrics = Arc::clone(&self.system_metrics);
        let application_metrics = Arc::clone(&self.application_metrics);
        let _performance_metrics = Arc::clone(&self.performance_metrics);
        let active_alerts = Arc::clone(&self.active_alerts);
        let alert_sender = self.alert_sender.clone();

        // System metrics collection task
        let system_task_monitoring = Arc::clone(&is_monitoring);
        let system_task_metrics = Arc::clone(&system_metrics);
        tokio::spawn(async move {
            debug!(session_id = %session_id, "Starting system metrics collection task");

            while *system_task_monitoring.read().await {
                if let Ok(metrics) = Self::collect_system_metrics_impl().await {
                    let mut metrics_queue = system_task_metrics.write().await;
                    metrics_queue.push_back(metrics);

                    if metrics_queue.len() > max_samples {
                        metrics_queue.pop_front();
                    }
                }

                tokio::time::sleep(collection_interval).await;
            }

            debug!(session_id = %session_id, "System metrics collection task stopped");
        });

        // Application metrics collection task
        let app_task_monitoring = Arc::clone(&is_monitoring);
        let app_task_metrics = Arc::clone(&application_metrics);
        tokio::spawn(async move {
            debug!(session_id = %session_id, "Starting application metrics collection task");

            while *app_task_monitoring.read().await {
                if let Ok(metrics) = Self::collect_application_metrics_impl().await {
                    let mut metrics_queue = app_task_metrics.write().await;
                    metrics_queue.push_back(metrics);

                    if metrics_queue.len() > max_samples {
                        metrics_queue.pop_front();
                    }
                }

                tokio::time::sleep(collection_interval).await;
            }

            debug!(session_id = %session_id, "Application metrics collection task stopped");
        });

        // Alert monitoring task
        if config.enable_alerts {
            let alert_task_monitoring = Arc::clone(&is_monitoring);
            tokio::spawn(async move {
                debug!(session_id = %session_id, "Starting alert monitoring task");

                while *alert_task_monitoring.read().await {
                    if let Ok(system_metrics) = Self::collect_system_metrics_impl().await {
                        if let Ok(app_metrics) = Self::collect_application_metrics_impl().await {
                            let alerts = Self::check_thresholds(&targets, &config, &system_metrics, &app_metrics).await;

                            for alert in alerts {
                                let mut active_alerts_guard = active_alerts.write().await;
                                active_alerts_guard.insert(alert.metric.clone(), alert.clone());

                                if let Some(ref sender) = alert_sender {
                                    let _ = sender.send(alert);
                                }
                            }
                        }
                    }

                    tokio::time::sleep(collection_interval).await;
                }

                debug!(session_id = %session_id, "Alert monitoring task stopped");
            });
        }

        Ok(())
    }

    /// Collect current system metrics
    async fn collect_system_metrics(&self) -> Result<SystemMetrics> {
        Self::collect_system_metrics_impl().await
    }

    /// Collect current system metrics (static implementation)
    async fn collect_system_metrics_impl() -> Result<SystemMetrics> {
        // In a real implementation, this would use system APIs to collect actual metrics
        // For now, we'll simulate realistic values

        Ok(SystemMetrics {
            timestamp: chrono::Utc::now(),
            cpu_usage_percent: 45.0 + (rand::random::<f64>() * 20.0),
            memory_rss_mb: 250.0 + (rand::random::<f64>() * 100.0),
            memory_heap_mb: 180.0 + (rand::random::<f64>() * 80.0),
            memory_virtual_mb: 500.0 + (rand::random::<f64>() * 200.0),
            disk_read_mbps: rand::random::<f64>() * 50.0,
            disk_write_mbps: rand::random::<f64>() * 30.0,
            network_in_mbps: rand::random::<f64>() * 100.0,
            network_out_mbps: rand::random::<f64>() * 80.0,
            open_file_descriptors: 150 + (rand::random::<u32>() % 100),
            thread_count: 20 + (rand::random::<u32>() % 30),
        })
    }

    /// Collect current application metrics
    async fn collect_application_metrics(&self) -> Result<ApplicationMetrics> {
        Self::collect_application_metrics_impl().await
    }

    /// Collect current application metrics (static implementation)
    async fn collect_application_metrics_impl() -> Result<ApplicationMetrics> {
        // In a real implementation, this would collect metrics from the actual application
        // For now, we'll simulate realistic values

        let total_requests = 10000 + (rand::random::<u64>() % 5000);
        let failed_requests = total_requests / 50; // ~2% failure rate
        let successful_requests = total_requests - failed_requests;

        Ok(ApplicationMetrics {
            timestamp: chrono::Utc::now(),
            active_requests: 5 + (rand::random::<u32>() % 20),
            total_requests,
            successful_requests,
            failed_requests,
            avg_response_time_ms: 800.0 + (rand::random::<f64>() * 400.0),
            p95_response_time_ms: 1500.0 + (rand::random::<f64>() * 500.0),
            cache_hit_rate: 0.85 + (rand::random::<f64>() * 0.1),
            cache_size_mb: 50.0 + (rand::random::<f64>() * 30.0),
            ai_processing_queue_size: rand::random::<u32>() % 10,
            ai_avg_processing_time_ms: 200.0 + (rand::random::<f64>() * 100.0),
        })
    }

    /// Check performance thresholds and generate alerts
    async fn check_thresholds(
        targets: &PerformanceTargets,
        config: &MonitoringConfig,
        system_metrics: &SystemMetrics,
        app_metrics: &ApplicationMetrics,
    ) -> Vec<PerformanceAlert> {
        let mut alerts = Vec::new();

        // Memory threshold checks
        let memory_warning_threshold = targets.memory_alert_mb as f64 * config.alert_multipliers.warning;
        let memory_critical_threshold = targets.memory_alert_mb as f64 * config.alert_multipliers.critical;

        if system_metrics.memory_rss_mb > memory_critical_threshold {
            alerts.push(PerformanceAlert {
                id: Uuid::new_v4(),
                severity: AlertSeverity::Critical,
                metric: "memory_rss".to_string(),
                current_value: system_metrics.memory_rss_mb,
                threshold_value: memory_critical_threshold,
                message: format!(
                    "Critical memory usage: {:.1}MB exceeds {:.1}MB threshold",
                    system_metrics.memory_rss_mb, memory_critical_threshold
                ),
                timestamp: chrono::Utc::now(),
                acknowledged: false,
            });
        } else if system_metrics.memory_rss_mb > memory_warning_threshold {
            alerts.push(PerformanceAlert {
                id: Uuid::new_v4(),
                severity: AlertSeverity::Warning,
                metric: "memory_rss".to_string(),
                current_value: system_metrics.memory_rss_mb,
                threshold_value: memory_warning_threshold,
                message: format!(
                    "High memory usage: {:.1}MB exceeds {:.1}MB warning threshold",
                    system_metrics.memory_rss_mb, memory_warning_threshold
                ),
                timestamp: chrono::Utc::now(),
                acknowledged: false,
            });
        }

        // Latency threshold checks
        let latency_warning_threshold = targets.p95_latency_ms as f64 * config.alert_multipliers.warning;
        let latency_critical_threshold = targets.p95_latency_ms as f64 * config.alert_multipliers.critical;

        if app_metrics.p95_response_time_ms > latency_critical_threshold {
            alerts.push(PerformanceAlert {
                id: Uuid::new_v4(),
                severity: AlertSeverity::Critical,
                metric: "p95_latency".to_string(),
                current_value: app_metrics.p95_response_time_ms,
                threshold_value: latency_critical_threshold,
                message: format!(
                    "Critical latency: {:.1}ms exceeds {:.1}ms threshold",
                    app_metrics.p95_response_time_ms, latency_critical_threshold
                ),
                timestamp: chrono::Utc::now(),
                acknowledged: false,
            });
        } else if app_metrics.p95_response_time_ms > latency_warning_threshold {
            alerts.push(PerformanceAlert {
                id: Uuid::new_v4(),
                severity: AlertSeverity::Warning,
                metric: "p95_latency".to_string(),
                current_value: app_metrics.p95_response_time_ms,
                threshold_value: latency_warning_threshold,
                message: format!(
                    "High latency: {:.1}ms exceeds {:.1}ms warning threshold",
                    app_metrics.p95_response_time_ms, latency_warning_threshold
                ),
                timestamp: chrono::Utc::now(),
                acknowledged: false,
            });
        }

        // CPU threshold checks
        if system_metrics.cpu_usage_percent > 90.0 {
            alerts.push(PerformanceAlert {
                id: Uuid::new_v4(),
                severity: AlertSeverity::Critical,
                metric: "cpu_usage".to_string(),
                current_value: system_metrics.cpu_usage_percent,
                threshold_value: 90.0,
                message: format!(
                    "Critical CPU usage: {:.1}% exceeds 90% threshold",
                    system_metrics.cpu_usage_percent
                ),
                timestamp: chrono::Utc::now(),
                acknowledged: false,
            });
        } else if system_metrics.cpu_usage_percent > 75.0 {
            alerts.push(PerformanceAlert {
                id: Uuid::new_v4(),
                severity: AlertSeverity::Warning,
                metric: "cpu_usage".to_string(),
                current_value: system_metrics.cpu_usage_percent,
                threshold_value: 75.0,
                message: format!(
                    "High CPU usage: {:.1}% exceeds 75% warning threshold",
                    system_metrics.cpu_usage_percent
                ),
                timestamp: chrono::Utc::now(),
                acknowledged: false,
            });
        }

        alerts
    }

    /// Calculate memory growth rate
    async fn calculate_memory_growth_rate(&self) -> f64 {
        let metrics = self.system_metrics.read().await;
        if metrics.len() < 2 {
            return 0.0;
        }

        let recent = &metrics[metrics.len() - 1];
        let older = &metrics[metrics.len() - 2];

        let time_diff = (recent.timestamp - older.timestamp).num_seconds() as f64;
        if time_diff > 0.0 {
            (recent.memory_rss_mb - older.memory_rss_mb) / time_diff
        } else {
            0.0
        }
    }

    /// Calculate current throughput
    async fn calculate_throughput(&self) -> f64 {
        let metrics = self.application_metrics.read().await;
        if metrics.len() < 2 {
            return 0.0;
        }

        let recent = &metrics[metrics.len() - 1];
        let older = &metrics[metrics.len() - 2];

        let time_diff = (recent.timestamp - older.timestamp).num_seconds() as f64;
        if time_diff > 0.0 {
            (recent.total_requests - older.total_requests) as f64 / time_diff
        } else {
            0.0
        }
    }

    /// Calculate AI overhead percentage
    async fn calculate_ai_overhead(&self) -> f64 {
        let metrics = self.application_metrics.read().await;
        if let Some(latest) = metrics.back() {
            // Calculate AI overhead as percentage of total response time
            if latest.avg_response_time_ms > 0.0 {
                (latest.ai_avg_processing_time_ms / latest.avg_response_time_ms) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Generate comprehensive monitoring report
    async fn generate_monitoring_report(&self, monitoring_duration: Duration) -> Result<MonitoringReport> {
        let current_metrics = self.get_current_metrics().await?;
        let alerts: Vec<PerformanceAlert> = self.active_alerts.read().await.values().cloned().collect();

        // Generate system summary
        let system_summary = self.generate_system_summary().await?;
        let application_summary = self.generate_application_summary().await?;

        // Generate recommendations
        let recommendations = self.generate_monitoring_recommendations(&current_metrics, &alerts).await?;

        let system_metrics = self.system_metrics.read().await;
        let total_samples = system_metrics.len();

        Ok(MonitoringReport {
            session_id: self.session_id,
            monitoring_duration,
            total_samples,
            metrics: current_metrics,
            alerts,
            system_summary,
            application_summary,
            recommendations,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Generate system metrics summary
    async fn generate_system_summary(&self) -> Result<SystemSummary> {
        let metrics = self.system_metrics.read().await;

        if metrics.is_empty() {
            return Ok(SystemSummary {
                avg_cpu_usage: 0.0,
                peak_cpu_usage: 0.0,
                avg_memory_mb: 0.0,
                peak_memory_mb: 0.0,
                total_disk_io_mb: 0.0,
                total_network_io_mb: 0.0,
                uptime: Duration::default(),
            });
        }

        let cpu_values: Vec<f64> = metrics.iter().map(|m| m.cpu_usage_percent).collect();
        let memory_values: Vec<f64> = metrics.iter().map(|m| m.memory_rss_mb).collect();

        let avg_cpu_usage = cpu_values.iter().sum::<f64>() / cpu_values.len() as f64;
        let peak_cpu_usage = cpu_values.iter().fold(0.0_f64, |max, &val| max.max(val));
        let avg_memory_mb = memory_values.iter().sum::<f64>() / memory_values.len() as f64;
        let peak_memory_mb = memory_values.iter().fold(0.0_f64, |max, &val| max.max(val));

        let total_disk_io_mb = metrics.iter()
            .map(|m| m.disk_read_mbps + m.disk_write_mbps)
            .sum::<f64>();

        let total_network_io_mb = metrics.iter()
            .map(|m| m.network_in_mbps + m.network_out_mbps)
            .sum::<f64>();

        let uptime = self.start_time.map(|start| start.elapsed()).unwrap_or_default();

        Ok(SystemSummary {
            avg_cpu_usage,
            peak_cpu_usage,
            avg_memory_mb,
            peak_memory_mb,
            total_disk_io_mb,
            total_network_io_mb,
            uptime,
        })
    }

    /// Generate application metrics summary
    async fn generate_application_summary(&self) -> Result<ApplicationSummary> {
        let metrics = self.application_metrics.read().await;

        if metrics.is_empty() {
            return Ok(ApplicationSummary {
                total_requests_processed: 0,
                success_rate: 0.0,
                avg_latency_ms: 0.0,
                p95_latency_ms: 0.0,
                p99_latency_ms: 0.0,
                peak_concurrent_requests: 0,
                cache_efficiency: 0.0,
                ai_processing_efficiency: 0.0,
            });
        }

        let latest = &metrics[metrics.len() - 1];
        let total_requests_processed = latest.total_requests;
        let success_rate = if latest.total_requests > 0 {
            (latest.successful_requests as f64 / latest.total_requests as f64) * 100.0
        } else {
            0.0
        };

        let latency_values: Vec<f64> = metrics.iter().map(|m| m.avg_response_time_ms).collect();
        let p95_values: Vec<f64> = metrics.iter().map(|m| m.p95_response_time_ms).collect();

        let avg_latency_ms = latency_values.iter().sum::<f64>() / latency_values.len() as f64;
        let p95_latency_ms = p95_values.iter().sum::<f64>() / p95_values.len() as f64;
        let p99_latency_ms = p95_latency_ms * 1.2; // Estimate P99 as 1.2x P95

        let peak_concurrent_requests = metrics.iter()
            .map(|m| m.active_requests)
            .max()
            .unwrap_or(0);

        let cache_efficiency = metrics.iter()
            .map(|m| m.cache_hit_rate)
            .sum::<f64>() / metrics.len() as f64;

        let ai_processing_efficiency = 100.0 - (latest.ai_avg_processing_time_ms / latest.avg_response_time_ms * 100.0).min(100.0);

        Ok(ApplicationSummary {
            total_requests_processed,
            success_rate,
            avg_latency_ms,
            p95_latency_ms,
            p99_latency_ms,
            peak_concurrent_requests,
            cache_efficiency,
            ai_processing_efficiency,
        })
    }

    /// Generate monitoring recommendations
    async fn generate_monitoring_recommendations(
        &self,
        _metrics: &PerformanceMetrics,
        alerts: &[PerformanceAlert],
    ) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();

        // Alert-based recommendations
        let critical_alerts = alerts.iter().filter(|a| a.severity == AlertSeverity::Critical).count();
        let warning_alerts = alerts.iter().filter(|a| a.severity == AlertSeverity::Warning).count();

        if critical_alerts > 0 {
            recommendations.push(format!(
                "URGENT: {} critical alerts require immediate attention",
                critical_alerts
            ));
        }

        if warning_alerts > 3 {
            recommendations.push(format!(
                "Multiple warning alerts ({}) indicate potential performance issues",
                warning_alerts
            ));
        }

        // System-specific recommendations
        for alert in alerts {
            match alert.metric.as_str() {
                "memory_rss" => {
                    recommendations.push("Consider implementing memory pooling or increasing garbage collection frequency".to_string());
                }
                "p95_latency" => {
                    recommendations.push("Investigate slow queries or implement response caching".to_string());
                }
                "cpu_usage" => {
                    recommendations.push("Consider horizontal scaling or CPU optimization".to_string());
                }
                _ => {}
            }
        }

        if recommendations.is_empty() {
            recommendations.push("System performance is within acceptable ranges".to_string());
        }

        Ok(recommendations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_monitor_creation() {
        let targets = PerformanceTargets::default();
        let monitor = PerformanceMonitor::new(targets).unwrap();
        assert!(!monitor.session_id.is_nil());
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let targets = PerformanceTargets::default();
        let monitor = PerformanceMonitor::new(targets).unwrap();

        let system_metrics = monitor.collect_system_metrics().await.unwrap();
        assert!(system_metrics.cpu_usage_percent >= 0.0);
        assert!(system_metrics.memory_rss_mb > 0.0);

        let app_metrics = monitor.collect_application_metrics().await.unwrap();
        assert!(app_metrics.total_requests > 0);
    }

    #[tokio::test]
    async fn test_current_metrics() {
        let targets = PerformanceTargets::default();
        let monitor = PerformanceMonitor::new(targets).unwrap();

        let metrics = monitor.get_current_metrics().await.unwrap();
        assert!(metrics.memory_rss_mb > 0.0);
        assert!(metrics.cpu_usage_percent >= 0.0);
    }
}