//! Alert system for monitoring critical conditions

use crate::monitoring::metrics::PerformanceMetrics;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Alert rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub name: String,
    pub metric_name: String,
    pub threshold: f64,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub enabled: bool,
}

/// Alert condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    GreaterThan,
    LessThan,
    Equals,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Alert instance
#[derive(Debug, Clone, Serialize)]
pub struct Alert {
    pub rule_name: String,
    pub message: String,
    pub severity: AlertSeverity,
    #[serde(skip)]
    pub timestamp: Instant,
    pub timestamp_utc: DateTime<Utc>,
    pub current_value: f64,
    pub threshold: f64,
}

/// Alert manager for monitoring and notifications
pub struct AlertManager {
    rules: Vec<AlertRule>,
    active_alerts: HashMap<String, Instant>,
    cooldown_period: Duration,
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertManager {
    /// Create a new alert manager with default rules
    pub fn new() -> Self {
        Self {
            rules: Self::default_alert_rules(),
            active_alerts: HashMap::new(),
            cooldown_period: Duration::from_secs(5 * 60), // 5 minutes
        }
    }

    /// Create alert manager with custom configuration
    pub fn with_config(rules: Vec<AlertRule>, cooldown_secs: u64) -> Self {
        Self {
            rules,
            active_alerts: HashMap::new(),
            cooldown_period: Duration::from_secs(cooldown_secs),
        }
    }

    /// Add a new alert rule
    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.push(rule);
    }

    /// Remove an alert rule by name
    pub fn remove_rule(&mut self, name: &str) -> bool {
        if let Some(pos) = self.rules.iter().position(|r| r.name == name) {
            self.rules.remove(pos);
            true
        } else {
            false
        }
    }

    /// Enable/disable a rule
    pub fn set_rule_enabled(&mut self, name: &str, enabled: bool) -> bool {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.name == name) {
            rule.enabled = enabled;
            true
        } else {
            false
        }
    }

    /// Check for alerts based on current metrics
    pub async fn check_alerts(&mut self, metrics: &PerformanceMetrics) -> Vec<Alert> {
        let mut triggered_alerts = Vec::new();
        let now = Instant::now();

        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }

            let current_value = self.get_metric_value(metrics, &rule.metric_name);
            let should_alert =
                self.evaluate_condition(current_value, rule.threshold, &rule.condition);

            if should_alert {
                // Check cooldown period
                if let Some(last_alert) = self.active_alerts.get(&rule.name) {
                    if now.duration_since(*last_alert) < self.cooldown_period {
                        continue; // Skip this alert due to cooldown
                    }
                }

                let alert = Alert {
                    rule_name: rule.name.clone(),
                    message: self.format_alert_message(rule, current_value),
                    severity: rule.severity.clone(),
                    timestamp: now,
                    timestamp_utc: Utc::now(),
                    current_value,
                    threshold: rule.threshold,
                };

                triggered_alerts.push(alert);
                self.active_alerts.insert(rule.name.clone(), now);
            } else {
                // Clear active alert if condition is no longer met
                self.active_alerts.remove(&rule.name);
            }
        }

        triggered_alerts
    }

    /// Clear all active alerts
    pub fn clear_active_alerts(&mut self) {
        self.active_alerts.clear();
    }

    /// Get all configured rules
    pub fn get_rules(&self) -> &[AlertRule] {
        &self.rules
    }

    /// Get active alerts
    pub fn get_active_alerts(&self) -> Vec<String> {
        self.active_alerts.keys().cloned().collect()
    }

    /// Evaluate condition
    fn evaluate_condition(
        &self,
        current_value: f64,
        threshold: f64,
        condition: &AlertCondition,
    ) -> bool {
        match condition {
            AlertCondition::GreaterThan => current_value > threshold,
            AlertCondition::LessThan => current_value < threshold,
            AlertCondition::Equals => (current_value - threshold).abs() < 0.001,
        }
    }

    /// Format alert message
    fn format_alert_message(&self, rule: &AlertRule, current_value: f64) -> String {
        let condition_text = match rule.condition {
            AlertCondition::GreaterThan => "exceeds",
            AlertCondition::LessThan => "is below",
            AlertCondition::Equals => "equals",
        };

        format!(
            "[{}] Alert: {} {} threshold - Current: {:.2}, Threshold: {:.2}",
            match rule.severity {
                AlertSeverity::Info => "INFO",
                AlertSeverity::Warning => "WARNING",
                AlertSeverity::Error => "ERROR",
                AlertSeverity::Critical => "CRITICAL",
            },
            rule.metric_name.replace('_', " "),
            condition_text,
            current_value,
            rule.threshold
        )
    }

    /// Default alert rules
    fn default_alert_rules() -> Vec<AlertRule> {
        vec![
            AlertRule {
                name: "high_error_rate".to_string(),
                metric_name: "error_rate".to_string(),
                threshold: 10.0,
                condition: AlertCondition::GreaterThan,
                severity: AlertSeverity::Error,
                enabled: true,
            },
            AlertRule {
                name: "critical_error_rate".to_string(),
                metric_name: "error_rate".to_string(),
                threshold: 20.0,
                condition: AlertCondition::GreaterThan,
                severity: AlertSeverity::Critical,
                enabled: true,
            },
            AlertRule {
                name: "high_cpu_usage".to_string(),
                metric_name: "cpu_usage_percent".to_string(),
                threshold: 80.0,
                condition: AlertCondition::GreaterThan,
                severity: AlertSeverity::Warning,
                enabled: true,
            },
            AlertRule {
                name: "critical_cpu_usage".to_string(),
                metric_name: "cpu_usage_percent".to_string(),
                threshold: 90.0,
                condition: AlertCondition::GreaterThan,
                severity: AlertSeverity::Critical,
                enabled: true,
            },
            AlertRule {
                name: "low_health_score".to_string(),
                metric_name: "health_score".to_string(),
                threshold: 50.0,
                condition: AlertCondition::LessThan,
                severity: AlertSeverity::Critical,
                enabled: true,
            },
            AlertRule {
                name: "high_extraction_time".to_string(),
                metric_name: "p99_extraction_time_ms".to_string(),
                threshold: 10000.0,
                condition: AlertCondition::GreaterThan,
                severity: AlertSeverity::Warning,
                enabled: true,
            },
            AlertRule {
                name: "high_memory_usage".to_string(),
                metric_name: "memory_usage_bytes".to_string(),
                threshold: 4.0 * 1024.0 * 1024.0 * 1024.0, // 4GB
                condition: AlertCondition::GreaterThan,
                severity: AlertSeverity::Warning,
                enabled: true,
            },
            AlertRule {
                name: "circuit_breaker_tripped".to_string(),
                metric_name: "circuit_breaker_trips".to_string(),
                threshold: 0.0,
                condition: AlertCondition::GreaterThan,
                severity: AlertSeverity::Warning,
                enabled: true,
            },
        ]
    }

    /// Get metric value from performance metrics
    fn get_metric_value(&self, metrics: &PerformanceMetrics, metric_name: &str) -> f64 {
        match metric_name {
            "error_rate" => metrics.error_rate,
            "cpu_usage_percent" => metrics.cpu_usage_percent as f64,
            "health_score" => metrics.health_score as f64,
            "p95_extraction_time_ms" => metrics.p95_extraction_time_ms,
            "p99_extraction_time_ms" => metrics.p99_extraction_time_ms,
            "avg_extraction_time_ms" => metrics.avg_extraction_time_ms,
            "memory_usage_bytes" => metrics.memory_usage_bytes as f64,
            "requests_per_second" => metrics.requests_per_second,
            "circuit_breaker_trips" => metrics.circuit_breaker_trips as f64,
            "timeout_rate" => metrics.timeout_rate,
            "cache_hit_ratio" => metrics.cache_hit_ratio,
            "total_extractions" => metrics.total_extractions as f64,
            "successful_extractions" => metrics.successful_extractions as f64,
            "failed_extractions" => metrics.failed_extractions as f64,
            _ => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_alert_triggering() {
        let mut alert_manager = AlertManager::new();

        let high_error_metrics = PerformanceMetrics {
            error_rate: 15.0,
            cpu_usage_percent: 95.0,
            health_score: 40.0,
            ..Default::default()
        };

        let alerts = alert_manager.check_alerts(&high_error_metrics).await;

        // Should trigger multiple alerts
        assert!(alerts.len() >= 3);

        // Check specific alerts
        assert!(alerts.iter().any(|a| a.rule_name == "high_error_rate"));
        assert!(alerts.iter().any(|a| a.rule_name == "critical_cpu_usage"));
        assert!(alerts.iter().any(|a| a.rule_name == "low_health_score"));
    }

    #[tokio::test]
    async fn test_alert_cooldown() {
        let mut alert_manager = AlertManager::with_config(
            vec![AlertRule {
                name: "test_alert".to_string(),
                metric_name: "error_rate".to_string(),
                threshold: 5.0,
                condition: AlertCondition::GreaterThan,
                severity: AlertSeverity::Warning,
                enabled: true,
            }],
            0, // No cooldown for testing
        );

        let metrics = PerformanceMetrics {
            error_rate: 10.0,
            ..Default::default()
        };

        // First check should trigger alert
        let alerts1 = alert_manager.check_alerts(&metrics).await;
        assert_eq!(alerts1.len(), 1);

        // Immediate second check should also trigger (no cooldown)
        let alerts2 = alert_manager.check_alerts(&metrics).await;
        assert_eq!(alerts2.len(), 1);

        // With cooldown enabled
        alert_manager.cooldown_period = Duration::from_secs(60);
        let alerts3 = alert_manager.check_alerts(&metrics).await;
        assert_eq!(alerts3.len(), 0); // Should be blocked by cooldown
    }

    #[test]
    fn test_rule_management() {
        let mut alert_manager = AlertManager::new();

        // Add custom rule
        alert_manager.add_rule(AlertRule {
            name: "custom_rule".to_string(),
            metric_name: "cache_hit_ratio".to_string(),
            threshold: 0.3,
            condition: AlertCondition::LessThan,
            severity: AlertSeverity::Warning,
            enabled: true,
        });

        // Test rule exists
        assert!(alert_manager
            .get_rules()
            .iter()
            .any(|r| r.name == "custom_rule"));

        // Disable rule
        assert!(alert_manager.set_rule_enabled("custom_rule", false));
        let rule = alert_manager
            .get_rules()
            .iter()
            .find(|r| r.name == "custom_rule")
            .unwrap();
        assert!(!rule.enabled);

        // Remove rule
        assert!(alert_manager.remove_rule("custom_rule"));
        assert!(!alert_manager
            .get_rules()
            .iter()
            .any(|r| r.name == "custom_rule"));
    }
}
