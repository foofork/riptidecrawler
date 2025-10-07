//! Performance alert types and memory profiling alert management

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::profiling::{LeakAnalysis, MemoryReport};

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Alert category for categorizing different types of alerts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertCategory {
    MemoryLeak,
    MemoryGrowth,
    MemoryEfficiency,
    MemoryThreshold,
    CpuUsage,
    DiskIo,
    NetworkIo,
    ApplicationLatency,
    General,
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub id: Uuid,
    pub severity: AlertSeverity,
    pub category: AlertCategory,
    pub metric: String,
    pub current_value: f64,
    pub threshold_value: f64,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub acknowledged: bool,
    pub component: Option<String>,
    pub recommendations: Vec<String>,
}

impl PerformanceAlert {
    /// Create a new performance alert
    pub fn new(
        severity: AlertSeverity,
        category: AlertCategory,
        metric: String,
        current_value: f64,
        threshold_value: f64,
        message: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            severity,
            category,
            metric,
            current_value,
            threshold_value,
            message,
            timestamp: chrono::Utc::now(),
            acknowledged: false,
            component: None,
            recommendations: Vec::new(),
        }
    }

    /// Create alert with component information
    pub fn with_component(mut self, component: String) -> Self {
        self.component = Some(component);
        self
    }

    /// Create alert with recommendations
    pub fn with_recommendations(mut self, recommendations: Vec<String>) -> Self {
        self.recommendations = recommendations;
        self
    }

    /// Acknowledge this alert
    pub fn acknowledge(&mut self) {
        self.acknowledged = true;
    }
}

/// Alert rule for defining when alerts should trigger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub name: String,
    pub category: AlertCategory,
    pub severity: AlertSeverity,
    pub threshold: f64,
    pub condition: AlertCondition,
    pub enabled: bool,
}

/// Alert condition types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertCondition {
    GreaterThan,
    LessThan,
    Equals,
    NotEquals,
}

impl AlertRule {
    /// Check if the rule should trigger for a given value
    pub fn should_trigger(&self, value: f64) -> bool {
        if !self.enabled {
            return false;
        }

        match self.condition {
            AlertCondition::GreaterThan => value > self.threshold,
            AlertCondition::LessThan => value < self.threshold,
            AlertCondition::Equals => (value - self.threshold).abs() < f64::EPSILON,
            AlertCondition::NotEquals => (value - self.threshold).abs() >= f64::EPSILON,
        }
    }
}

/// Trait for alert notification channels
#[async_trait]
pub trait AlertChannel: Send + Sync {
    async fn send_alert(&self, alert: &PerformanceAlert) -> Result<()>;
    fn name(&self) -> &str;
}

/// Console logger alert channel
pub struct ConsoleAlertChannel;

#[async_trait]
impl AlertChannel for ConsoleAlertChannel {
    async fn send_alert(&self, alert: &PerformanceAlert) -> Result<()> {
        match alert.severity {
            AlertSeverity::Critical => {
                warn!(
                    alert_id = %alert.id,
                    severity = "CRITICAL",
                    category = ?alert.category,
                    metric = %alert.metric,
                    current = alert.current_value,
                    threshold = alert.threshold_value,
                    message = %alert.message,
                    "Performance alert triggered"
                );
            }
            AlertSeverity::Warning => {
                warn!(
                    alert_id = %alert.id,
                    severity = "WARNING",
                    category = ?alert.category,
                    metric = %alert.metric,
                    current = alert.current_value,
                    threshold = alert.threshold_value,
                    message = %alert.message,
                    "Performance alert triggered"
                );
            }
            AlertSeverity::Info => {
                info!(
                    alert_id = %alert.id,
                    severity = "INFO",
                    category = ?alert.category,
                    metric = %alert.metric,
                    current = alert.current_value,
                    threshold = alert.threshold_value,
                    message = %alert.message,
                    "Performance alert triggered"
                );
            }
        }

        if let Some(component) = &alert.component {
            debug!(component = %component, "Alert component");
        }

        if !alert.recommendations.is_empty() {
            debug!(recommendations = ?alert.recommendations, "Alert recommendations");
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "console"
    }
}

/// Memory alert manager for managing memory-specific alerts
pub struct MemoryAlertManager {
    alert_rules: Vec<AlertRule>,
    alert_history: Arc<RwLock<Vec<PerformanceAlert>>>,
    notification_channels: Arc<RwLock<Vec<Box<dyn AlertChannel>>>>,
    max_history_size: usize,
}

impl MemoryAlertManager {
    /// Create a new memory alert manager with default rules
    pub fn new() -> Self {
        let mut manager = Self {
            alert_rules: Vec::new(),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            notification_channels: Arc::new(RwLock::new(Vec::new())),
            max_history_size: 1000,
        };

        // Initialize default alert rules
        manager.initialize_default_rules();

        // Add default console channel
        manager
            .notification_channels
            .blocking_write()
            .push(Box::new(ConsoleAlertChannel));

        manager
    }

    /// Initialize default memory alert rules
    fn initialize_default_rules(&mut self) {
        // Memory leak detection rules
        self.alert_rules.push(AlertRule {
            name: "memory_leak_critical".to_string(),
            category: AlertCategory::MemoryLeak,
            severity: AlertSeverity::Critical,
            threshold: 50.0, // 50 MB/hour
            condition: AlertCondition::GreaterThan,
            enabled: true,
        });

        self.alert_rules.push(AlertRule {
            name: "memory_leak_warning".to_string(),
            category: AlertCategory::MemoryLeak,
            severity: AlertSeverity::Warning,
            threshold: 0.0, // Any leak
            condition: AlertCondition::GreaterThan,
            enabled: true,
        });

        // Memory growth rate rules
        self.alert_rules.push(AlertRule {
            name: "memory_growth_critical".to_string(),
            category: AlertCategory::MemoryGrowth,
            severity: AlertSeverity::Critical,
            threshold: 5.0, // 5 MB/s
            condition: AlertCondition::GreaterThan,
            enabled: true,
        });

        self.alert_rules.push(AlertRule {
            name: "memory_growth_warning".to_string(),
            category: AlertCategory::MemoryGrowth,
            severity: AlertSeverity::Warning,
            threshold: 1.0, // 1 MB/s
            condition: AlertCondition::GreaterThan,
            enabled: true,
        });

        // Memory efficiency rules
        self.alert_rules.push(AlertRule {
            name: "memory_efficiency_warning".to_string(),
            category: AlertCategory::MemoryEfficiency,
            severity: AlertSeverity::Warning,
            threshold: 0.5, // 50% efficiency
            condition: AlertCondition::LessThan,
            enabled: true,
        });

        // Memory threshold rules
        self.alert_rules.push(AlertRule {
            name: "memory_threshold_critical".to_string(),
            category: AlertCategory::MemoryThreshold,
            severity: AlertSeverity::Critical,
            threshold: 700.0, // 700 MB
            condition: AlertCondition::GreaterThan,
            enabled: true,
        });

        self.alert_rules.push(AlertRule {
            name: "memory_threshold_warning".to_string(),
            category: AlertCategory::MemoryThreshold,
            severity: AlertSeverity::Warning,
            threshold: 650.0, // 650 MB
            condition: AlertCondition::GreaterThan,
            enabled: true,
        });
    }

    /// Check memory alerts based on a memory report
    pub async fn check_memory_alerts(
        &mut self,
        report: &MemoryReport,
    ) -> Result<Vec<PerformanceAlert>> {
        let mut alerts = Vec::new();

        // Check memory growth rate alerts
        let growth_rate_alerts = self.check_growth_rate_alerts(report).await?;
        alerts.extend(growth_rate_alerts);

        // Check efficiency alerts
        let efficiency_alerts = self.check_efficiency_alerts(report).await?;
        alerts.extend(efficiency_alerts);

        // Check memory threshold alerts
        let threshold_alerts = self.check_threshold_alerts(report).await?;
        alerts.extend(threshold_alerts);

        // Send alerts through notification channels
        self.send_alerts(&alerts).await?;

        // Add to history
        self.add_to_history(alerts.clone()).await?;

        Ok(alerts)
    }

    /// Check leak-specific alerts
    pub async fn check_leak_alerts(
        &mut self,
        analysis: &LeakAnalysis,
    ) -> Result<Vec<PerformanceAlert>> {
        let mut alerts = Vec::new();

        // Check if we have any potential leaks
        if !analysis.potential_leaks.is_empty() {
            // Determine severity based on growth rate
            let severity = if analysis.growth_rate_mb_per_hour > 50.0 {
                AlertSeverity::Critical
            } else {
                AlertSeverity::Warning
            };

            for leak in &analysis.potential_leaks {
                let mut alert = PerformanceAlert::new(
                    severity.clone(),
                    AlertCategory::MemoryLeak,
                    "memory_leak_detected".to_string(),
                    leak.total_size_bytes as f64 / 1024.0 / 1024.0, // Convert to MB
                    0.0,
                    format!(
                        "Memory leak detected in component '{}' - {} bytes allocated, growth rate: {:.2} MB/hour",
                        leak.component, leak.total_size_bytes, leak.growth_rate / 1024.0 / 1024.0 * 3600.0
                    ),
                )
                .with_component(leak.component.clone());

                // Add recommendations
                let recommendations = vec![
                    format!(
                        "Investigate component '{}' for potential memory leaks",
                        leak.component
                    ),
                    format!(
                        "Review allocation patterns - {} allocations with average size {} bytes",
                        leak.allocation_count, leak.average_size_bytes as u64
                    ),
                    "Enable detailed allocation tracking to identify leak source".to_string(),
                ];

                alert = alert.with_recommendations(recommendations);
                alerts.push(alert);
            }
        }

        // Check for suspicious patterns
        for pattern in &analysis.suspicious_patterns {
            let alert = PerformanceAlert::new(
                AlertSeverity::Warning,
                AlertCategory::MemoryLeak,
                "suspicious_pattern".to_string(),
                0.0,
                0.0,
                format!("Suspicious memory allocation pattern: {}", pattern),
            )
            .with_recommendations(vec![
                "Review the allocation pattern for potential inefficiencies".to_string(),
                "Consider implementing object pooling or caching".to_string(),
            ]);

            alerts.push(alert);
        }

        // Send and store alerts
        self.send_alerts(&alerts).await?;
        self.add_to_history(alerts.clone()).await?;

        Ok(alerts)
    }

    /// Check memory growth rate alerts
    async fn check_growth_rate_alerts(
        &self,
        report: &MemoryReport,
    ) -> Result<Vec<PerformanceAlert>> {
        let mut alerts = Vec::new();
        let growth_rate = report.memory_growth_rate_mb_s;

        for rule in &self.alert_rules {
            if rule.category != AlertCategory::MemoryGrowth {
                continue;
            }

            if rule.should_trigger(growth_rate) {
                let alert = PerformanceAlert::new(
                    rule.severity.clone(),
                    AlertCategory::MemoryGrowth,
                    "memory_growth_rate".to_string(),
                    growth_rate,
                    rule.threshold,
                    format!(
                        "High memory growth rate: {:.4} MB/s (threshold: {:.4} MB/s)",
                        growth_rate, rule.threshold
                    ),
                )
                .with_recommendations(vec![
                    "Check for memory leaks in recent code changes".to_string(),
                    "Review caching strategies and implement size limits".to_string(),
                    "Enable detailed memory profiling to identify growth sources".to_string(),
                ]);

                alerts.push(alert);
            }
        }

        Ok(alerts)
    }

    /// Check memory efficiency alerts
    async fn check_efficiency_alerts(
        &self,
        report: &MemoryReport,
    ) -> Result<Vec<PerformanceAlert>> {
        let mut alerts = Vec::new();
        let efficiency_score = report.memory_efficiency_score;

        for rule in &self.alert_rules {
            if rule.category != AlertCategory::MemoryEfficiency {
                continue;
            }

            if rule.should_trigger(efficiency_score) {
                let alert = PerformanceAlert::new(
                    rule.severity.clone(),
                    AlertCategory::MemoryEfficiency,
                    "memory_efficiency".to_string(),
                    efficiency_score,
                    rule.threshold,
                    format!(
                        "Low allocation efficiency: {:.2} (threshold: {:.2})",
                        efficiency_score, rule.threshold
                    ),
                )
                .with_recommendations(vec![
                    "Review memory allocation patterns for inefficiencies".to_string(),
                    "Consider using memory pools for frequently allocated objects".to_string(),
                    "Implement more aggressive garbage collection strategies".to_string(),
                ]);

                alerts.push(alert);
            }
        }

        Ok(alerts)
    }

    /// Check memory threshold alerts
    async fn check_threshold_alerts(&self, report: &MemoryReport) -> Result<Vec<PerformanceAlert>> {
        let mut alerts = Vec::new();
        let rss_mb = report.peak_memory_mb;

        for rule in &self.alert_rules {
            if rule.category != AlertCategory::MemoryThreshold {
                continue;
            }

            if rule.should_trigger(rss_mb) {
                let alert = PerformanceAlert::new(
                    rule.severity.clone(),
                    AlertCategory::MemoryThreshold,
                    "memory_usage".to_string(),
                    rss_mb,
                    rule.threshold,
                    format!(
                        "Memory usage {:.1} MB exceeds threshold {:.1} MB",
                        rss_mb, rule.threshold
                    ),
                )
                .with_recommendations(vec![
                    format!(
                        "Current memory usage ({:.1} MB) is approaching system limits",
                        rss_mb
                    ),
                    "Review memory-intensive operations and implement cleanup routines".to_string(),
                    "Consider implementing memory-mapped files for large datasets".to_string(),
                ]);

                alerts.push(alert);
            }
        }

        Ok(alerts)
    }

    /// Add a notification channel
    pub async fn add_channel(&self, channel: Box<dyn AlertChannel>) {
        let mut channels = self.notification_channels.write().await;
        channels.push(channel);
    }

    /// Remove a notification channel by name
    pub async fn remove_channel(&self, name: &str) -> Result<()> {
        let mut channels = self.notification_channels.write().await;
        channels.retain(|c| c.name() != name);
        Ok(())
    }

    /// Send alerts through all notification channels
    async fn send_alerts(&self, alerts: &[PerformanceAlert]) -> Result<()> {
        let channels = self.notification_channels.read().await;

        for alert in alerts {
            for channel in channels.iter() {
                if let Err(e) = channel.send_alert(alert).await {
                    warn!(
                        channel = channel.name(),
                        error = %e,
                        "Failed to send alert through channel"
                    );
                }
            }
        }

        Ok(())
    }

    /// Add alerts to history
    async fn add_to_history(&self, alerts: Vec<PerformanceAlert>) -> Result<()> {
        let mut history = self.alert_history.write().await;

        for alert in alerts {
            history.push(alert);
        }

        // Trim history if it exceeds max size
        if history.len() > self.max_history_size {
            let excess = history.len() - self.max_history_size;
            history.drain(0..excess);
        }

        Ok(())
    }

    /// Get recent alerts
    pub async fn get_recent_alerts(&self, count: usize) -> Vec<PerformanceAlert> {
        let history = self.alert_history.read().await;
        let start = history.len().saturating_sub(count);
        history[start..].to_vec()
    }

    /// Get alerts by category
    pub async fn get_alerts_by_category(&self, category: AlertCategory) -> Vec<PerformanceAlert> {
        let history = self.alert_history.read().await;
        history
            .iter()
            .filter(|a| a.category == category)
            .cloned()
            .collect()
    }

    /// Get unacknowledged alerts
    pub async fn get_unacknowledged_alerts(&self) -> Vec<PerformanceAlert> {
        let history = self.alert_history.read().await;
        history
            .iter()
            .filter(|a| !a.acknowledged)
            .cloned()
            .collect()
    }

    /// Acknowledge an alert by ID
    pub async fn acknowledge_alert(&self, alert_id: Uuid) -> Result<()> {
        let mut history = self.alert_history.write().await;

        for alert in history.iter_mut() {
            if alert.id == alert_id {
                alert.acknowledge();
                return Ok(());
            }
        }

        Err(anyhow::anyhow!("Alert not found: {}", alert_id))
    }

    /// Clear alert history
    pub async fn clear_history(&self) -> Result<()> {
        let mut history = self.alert_history.write().await;
        history.clear();
        Ok(())
    }

    /// Add a custom alert rule
    pub fn add_rule(&mut self, rule: AlertRule) {
        self.alert_rules.push(rule);
    }

    /// Remove an alert rule by name
    pub fn remove_rule(&mut self, name: &str) {
        self.alert_rules.retain(|r| r.name != name);
    }

    /// Enable/disable a rule by name
    pub fn set_rule_enabled(&mut self, name: &str, enabled: bool) {
        if let Some(rule) = self.alert_rules.iter_mut().find(|r| r.name == name) {
            rule.enabled = enabled;
        }
    }
}

impl Default for MemoryAlertManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiling::{AllocationInfo, LeakInfo};
    use std::time::Duration;

    #[tokio::test]
    async fn test_alert_creation() {
        let alert = PerformanceAlert::new(
            AlertSeverity::Warning,
            AlertCategory::MemoryLeak,
            "test_metric".to_string(),
            100.0,
            50.0,
            "Test alert".to_string(),
        );

        assert_eq!(alert.severity, AlertSeverity::Warning);
        assert_eq!(alert.category, AlertCategory::MemoryLeak);
        assert_eq!(alert.current_value, 100.0);
        assert_eq!(alert.threshold_value, 50.0);
        assert!(!alert.acknowledged);
    }

    #[tokio::test]
    async fn test_alert_with_component() {
        let alert = PerformanceAlert::new(
            AlertSeverity::Critical,
            AlertCategory::MemoryLeak,
            "test".to_string(),
            100.0,
            50.0,
            "Test".to_string(),
        )
        .with_component("test_component".to_string());

        assert_eq!(alert.component, Some("test_component".to_string()));
    }

    #[tokio::test]
    async fn test_alert_rule_triggering() {
        let rule = AlertRule {
            name: "test_rule".to_string(),
            category: AlertCategory::MemoryGrowth,
            severity: AlertSeverity::Warning,
            threshold: 50.0,
            condition: AlertCondition::GreaterThan,
            enabled: true,
        };

        assert!(rule.should_trigger(51.0));
        assert!(!rule.should_trigger(49.0));
        assert!(!rule.should_trigger(50.0));
    }

    #[tokio::test]
    async fn test_alert_rule_disabled() {
        let mut rule = AlertRule {
            name: "test_rule".to_string(),
            category: AlertCategory::MemoryGrowth,
            severity: AlertSeverity::Warning,
            threshold: 50.0,
            condition: AlertCondition::GreaterThan,
            enabled: true,
        };

        assert!(rule.should_trigger(60.0));

        rule.enabled = false;
        assert!(!rule.should_trigger(60.0));
    }

    #[tokio::test]
    async fn test_memory_alert_manager_creation() {
        let manager = MemoryAlertManager::new();
        assert!(!manager.alert_rules.is_empty());
    }

    #[tokio::test]
    async fn test_leak_alerts() {
        let mut manager = MemoryAlertManager::new();

        let leak_analysis = LeakAnalysis {
            potential_leaks: vec![LeakInfo {
                component: "test_component".to_string(),
                allocation_count: 1000,
                total_size_bytes: 100 * 1024 * 1024, // 100 MB
                average_size_bytes: 102400.0,
                growth_rate: 10.0 * 1024.0 * 1024.0, // 10 MB/s
                first_seen: chrono::Utc::now(),
                last_seen: chrono::Utc::now(),
            }],
            growth_rate_mb_per_hour: 60.0,
            largest_allocations: vec![],
            suspicious_patterns: vec!["Pattern 1".to_string()],
        };

        let alerts = manager.check_leak_alerts(&leak_analysis).await.unwrap();

        assert!(!alerts.is_empty());
        assert!(alerts.iter().any(|a| a.severity == AlertSeverity::Critical));
    }

    #[tokio::test]
    async fn test_alert_history() {
        let manager = MemoryAlertManager::new();

        let alert = PerformanceAlert::new(
            AlertSeverity::Warning,
            AlertCategory::MemoryLeak,
            "test".to_string(),
            100.0,
            50.0,
            "Test".to_string(),
        );

        manager.add_to_history(vec![alert.clone()]).await.unwrap();

        let recent = manager.get_recent_alerts(10).await;
        assert_eq!(recent.len(), 1);
    }

    #[tokio::test]
    async fn test_alert_acknowledgment() {
        let manager = MemoryAlertManager::new();

        let alert = PerformanceAlert::new(
            AlertSeverity::Warning,
            AlertCategory::MemoryLeak,
            "test".to_string(),
            100.0,
            50.0,
            "Test".to_string(),
        );

        let alert_id = alert.id;
        manager.add_to_history(vec![alert]).await.unwrap();

        manager.acknowledge_alert(alert_id).await.unwrap();

        let unack = manager.get_unacknowledged_alerts().await;
        assert!(unack.is_empty());
    }

    #[tokio::test]
    async fn test_alerts_by_category() {
        let manager = MemoryAlertManager::new();

        let alert1 = PerformanceAlert::new(
            AlertSeverity::Warning,
            AlertCategory::MemoryLeak,
            "test1".to_string(),
            100.0,
            50.0,
            "Test 1".to_string(),
        );

        let alert2 = PerformanceAlert::new(
            AlertSeverity::Critical,
            AlertCategory::MemoryGrowth,
            "test2".to_string(),
            200.0,
            100.0,
            "Test 2".to_string(),
        );

        manager.add_to_history(vec![alert1, alert2]).await.unwrap();

        let leak_alerts = manager
            .get_alerts_by_category(AlertCategory::MemoryLeak)
            .await;
        assert_eq!(leak_alerts.len(), 1);

        let growth_alerts = manager
            .get_alerts_by_category(AlertCategory::MemoryGrowth)
            .await;
        assert_eq!(growth_alerts.len(), 1);
    }
}
