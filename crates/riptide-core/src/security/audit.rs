//! Audit Logging System for RipTide
//!
//! Provides comprehensive audit logging for security events, API requests,
//! and compliance tracking with structured logging and retention policies.

use crate::security::types::*;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Audit log entry containing all relevant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: SecurityEventType,
    pub severity: SecuritySeverity,
    pub tenant_id: Option<TenantId>,
    pub user_id: Option<String>,
    pub api_key_id: Option<String>,
    pub request_id: Option<String>,
    pub source_ip: Option<String>,
    pub user_agent: Option<String>,
    pub resource: Option<String>,
    pub action: String,
    pub outcome: AuditOutcome,
    pub details: AuditDetails,
    pub metadata: HashMap<String, Value>,
}

/// Outcome of an audited action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AuditOutcome {
    Success,
    Failure,
    Partial,
    Unknown,
}

impl std::fmt::Display for AuditOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditOutcome::Success => write!(f, "SUCCESS"),
            AuditOutcome::Failure => write!(f, "FAILURE"),
            AuditOutcome::Partial => write!(f, "PARTIAL"),
            AuditOutcome::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

/// Detailed information about the audited event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditDetails {
    pub description: String,
    pub error_message: Option<String>,
    pub request_payload: Option<Value>,
    pub response_payload: Option<Value>,
    pub duration_ms: Option<u64>,
    pub bytes_processed: Option<u64>,
    pub cost_usd: Option<f64>,
    pub tokens_used: Option<u64>,
    pub model_name: Option<String>,
    pub rate_limit_info: Option<RateLimitAuditInfo>,
    pub pii_redacted: bool,
    pub pii_detections: Option<u32>,
}

/// Rate limiting information for audit logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitAuditInfo {
    pub requests_remaining: u32,
    pub reset_time: DateTime<Utc>,
    pub burst_tokens_used: u32,
}

/// Audit log configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub log_directory: PathBuf,
    pub max_file_size_mb: u64,
    pub retention_days: u32,
    pub rotation_interval_hours: u32,
    pub enable_compression: bool,
    pub log_format: AuditLogFormat,
    pub log_levels: Vec<SecuritySeverity>,
    pub enable_real_time_alerts: bool,
    pub alert_thresholds: AlertThresholds,
    pub enable_structured_logging: bool,
    pub include_request_bodies: bool,
    pub include_response_bodies: bool,
    pub max_payload_size_kb: u32,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            log_directory: PathBuf::from("./logs/audit"),
            max_file_size_mb: 100,
            retention_days: 90,
            rotation_interval_hours: 24,
            enable_compression: true,
            log_format: AuditLogFormat::Json,
            log_levels: vec![
                SecuritySeverity::Low,
                SecuritySeverity::Medium,
                SecuritySeverity::High,
                SecuritySeverity::Critical,
            ],
            enable_real_time_alerts: true,
            alert_thresholds: AlertThresholds::default(),
            enable_structured_logging: true,
            include_request_bodies: true,
            include_response_bodies: false, // More sensitive
            max_payload_size_kb: 64,
        }
    }
}

/// Audit log output formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditLogFormat {
    Json,
    Csv,
    Syslog,
    Custom(String),
}

/// Alert thresholds for real-time monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub failed_auth_attempts_per_minute: u32,
    pub rate_limit_violations_per_hour: u32,
    pub budget_violations_per_hour: u32,
    pub critical_events_per_hour: u32,
    pub pii_detections_per_hour: u32,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            failed_auth_attempts_per_minute: 10,
            rate_limit_violations_per_hour: 50,
            budget_violations_per_hour: 5,
            critical_events_per_hour: 1,
            pii_detections_per_hour: 20,
        }
    }
}

/// Audit statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_events: u64,
    pub events_by_severity: HashMap<SecuritySeverity, u64>,
    pub events_by_type: HashMap<String, u64>,
    pub events_by_outcome: HashMap<AuditOutcome, u64>,
    pub events_by_tenant: HashMap<TenantId, u64>,
    pub current_file_size: u64,
    pub last_rotation: DateTime<Utc>,
    pub storage_usage_mb: f64,
}

/// Audit Logger - main component for security audit logging
pub struct AuditLogger {
    config: AuditConfig,
    current_file: Arc<Mutex<Option<BufWriter<File>>>>,
    current_file_path: Arc<Mutex<Option<PathBuf>>>,
    stats: Arc<RwLock<AuditStats>>,
    alert_counters: Arc<RwLock<HashMap<String, u32>>>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(config: Option<AuditConfig>) -> Result<Self> {
        let config = config.unwrap_or_default();

        // Create log directory if it doesn't exist
        std::fs::create_dir_all(&config.log_directory)
            .map_err(|e| anyhow!("Failed to create audit log directory: {}", e))?;

        let stats = AuditStats {
            total_events: 0,
            events_by_severity: HashMap::new(),
            events_by_type: HashMap::new(),
            events_by_outcome: HashMap::new(),
            events_by_tenant: HashMap::new(),
            current_file_size: 0,
            last_rotation: Utc::now(),
            storage_usage_mb: 0.0,
        };

        let logger = Self {
            config,
            current_file: Arc::new(Mutex::new(None)),
            current_file_path: Arc::new(Mutex::new(None)),
            stats: Arc::new(RwLock::new(stats)),
            alert_counters: Arc::new(RwLock::new(HashMap::new())),
        };

        // Initialize the first log file
        logger.rotate_log_file()?;

        info!("Audit logger initialized successfully");
        Ok(logger)
    }

    /// Log a security event
    pub async fn log_event(&self, entry: AuditLogEntry) -> Result<()> {
        // Check if we should log this severity level
        if !self.config.log_levels.contains(&entry.severity) {
            return Ok(());
        }

        // Prepare the log entry for writing
        let formatted_entry = self.format_log_entry(&entry)?;

        // Write to file
        self.write_to_file(&formatted_entry).await?;

        // Update statistics
        self.update_stats(&entry).await;

        // Check for alerts
        if self.config.enable_real_time_alerts {
            self.check_alerts(&entry).await;
        }

        debug!(
            event_id = %entry.id,
            event_type = %entry.event_type,
            severity = %entry.severity,
            "Audit event logged"
        );

        Ok(())
    }

    /// Create an audit entry for API key usage
    pub fn create_api_key_usage_entry(
        &self,
        context: &SecurityContext,
        outcome: AuditOutcome,
        details: AuditDetails,
    ) -> AuditLogEntry {
        AuditLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: SecurityEventType::ApiKeyUsed,
            severity: if outcome == AuditOutcome::Success {
                SecuritySeverity::Low
            } else {
                SecuritySeverity::Medium
            },
            tenant_id: Some(context.tenant_id.clone()),
            user_id: context.user_id.clone(),
            api_key_id: Some(context.api_key_id.clone()),
            request_id: Some(context.request_id.clone()),
            source_ip: Some(context.ip_address.clone()),
            user_agent: context.user_agent.clone(),
            resource: None,
            action: "api_key_authentication".to_string(),
            outcome,
            details,
            metadata: context
                .metadata
                .iter()
                .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                .collect(),
        }
    }

    /// Create an audit entry for budget events
    pub fn create_budget_event_entry(
        &self,
        event_type: SecurityEventType,
        tenant_id: &TenantId,
        cost_info: &CostInfo,
        details: AuditDetails,
    ) -> AuditLogEntry {
        let severity = match event_type {
            SecurityEventType::BudgetExceeded => SecuritySeverity::Critical,
            SecurityEventType::BudgetWarning => SecuritySeverity::High,
            _ => SecuritySeverity::Medium,
        };

        AuditLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type,
            severity,
            tenant_id: Some(tenant_id.clone()),
            user_id: None,
            api_key_id: None,
            request_id: None,
            source_ip: None,
            user_agent: None,
            resource: Some(cost_info.model_name.clone()),
            action: cost_info.operation_type.clone(),
            outcome: AuditOutcome::Success,
            details,
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert(
                    "tokens_used".to_string(),
                    Value::Number(cost_info.tokens_used.into()),
                );
                metadata.insert(
                    "cost_usd".to_string(),
                    Value::Number(
                        serde_json::Number::from_f64(cost_info.estimated_cost_usd)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    ),
                );
                metadata.insert(
                    "model_name".to_string(),
                    Value::String(cost_info.model_name.clone()),
                );
                metadata
            },
        }
    }

    /// Create an audit entry for PII detection
    pub fn create_pii_detection_entry(
        &self,
        tenant_id: &TenantId,
        pii_count: u32,
        details: AuditDetails,
    ) -> AuditLogEntry {
        AuditLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: SecurityEventType::PiiDetected,
            severity: if pii_count > 5 {
                SecuritySeverity::High
            } else {
                SecuritySeverity::Medium
            },
            tenant_id: Some(tenant_id.clone()),
            user_id: None,
            api_key_id: None,
            request_id: None,
            source_ip: None,
            user_agent: None,
            resource: None,
            action: "pii_detection".to_string(),
            outcome: AuditOutcome::Success,
            details,
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert(
                    "pii_detections".to_string(),
                    Value::Number(pii_count.into()),
                );
                metadata
            },
        }
    }

    /// Create an audit entry for rate limiting
    pub fn create_rate_limit_entry(
        &self,
        context: &SecurityContext,
        rate_limit_info: RateLimitAuditInfo,
        details: AuditDetails,
    ) -> AuditLogEntry {
        AuditLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: SecurityEventType::RateLimitExceeded,
            severity: SecuritySeverity::Medium,
            tenant_id: Some(context.tenant_id.clone()),
            user_id: context.user_id.clone(),
            api_key_id: Some(context.api_key_id.clone()),
            request_id: Some(context.request_id.clone()),
            source_ip: Some(context.ip_address.clone()),
            user_agent: context.user_agent.clone(),
            resource: None,
            action: "rate_limit_violation".to_string(),
            outcome: AuditOutcome::Failure,
            details: AuditDetails {
                rate_limit_info: Some(rate_limit_info),
                ..details
            },
            metadata: HashMap::new(),
        }
    }

    /// Format log entry according to configuration
    fn format_log_entry(&self, entry: &AuditLogEntry) -> Result<String> {
        match self.config.log_format {
            AuditLogFormat::Json => serde_json::to_string(entry)
                .map_err(|e| anyhow!("Failed to serialize audit entry to JSON: {}", e)),
            AuditLogFormat::Csv => Ok(format!(
                "{},{},{},{},{},{},{},{},{},{}",
                entry.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                entry.event_type,
                entry.severity,
                entry
                    .tenant_id
                    .as_ref()
                    .map(|t| t.to_string())
                    .unwrap_or_default(),
                entry.action,
                entry.outcome,
                entry.source_ip.as_deref().unwrap_or(""),
                entry.request_id.as_deref().unwrap_or(""),
                entry.details.description,
                entry.details.error_message.as_deref().unwrap_or("")
            )),
            AuditLogFormat::Syslog => Ok(format!(
                "<{}>{} riptide[{}]: {} {} {} tenant={} action={} outcome={} {}",
                Self::severity_to_syslog_priority(&entry.severity),
                entry.timestamp.format("%b %d %H:%M:%S"),
                std::process::id(),
                entry.event_type,
                entry.id,
                entry.severity,
                entry
                    .tenant_id
                    .as_ref()
                    .map(|t| t.to_string())
                    .unwrap_or("unknown".to_string()),
                entry.action,
                entry.outcome,
                entry.details.description
            )),
            AuditLogFormat::Custom(_) => {
                // For custom format, default to JSON
                serde_json::to_string(entry)
                    .map_err(|e| anyhow!("Failed to serialize audit entry: {}", e))
            }
        }
    }

    /// Convert security severity to syslog priority
    fn severity_to_syslog_priority(severity: &SecuritySeverity) -> u8 {
        match severity {
            SecuritySeverity::Low => 22,      // Local use facility (16) + Info (6)
            SecuritySeverity::Medium => 20,   // Local use facility (16) + Warning (4)
            SecuritySeverity::High => 19,     // Local use facility (16) + Error (3)
            SecuritySeverity::Critical => 18, // Local use facility (16) + Critical (2)
        }
    }

    /// Write formatted entry to log file
    async fn write_to_file(&self, formatted_entry: &str) -> Result<()> {
        let entry_bytes = format!(
            "{}
",
            formatted_entry
        );
        let entry_size = entry_bytes.len() as u64;

        // Check if we need to rotate the log file
        if self.should_rotate_file(entry_size).await? {
            self.rotate_log_file()?;
        }

        // Write to current file
        {
            let mut file_lock = self
                .current_file
                .lock()
                .map_err(|e| anyhow!("Failed to acquire file lock: {}", e))?;

            if let Some(ref mut writer) = *file_lock {
                writer
                    .write_all(entry_bytes.as_bytes())
                    .map_err(|e| anyhow!("Failed to write to audit log: {}", e))?;
                writer
                    .flush()
                    .map_err(|e| anyhow!("Failed to flush audit log: {}", e))?;
            } else {
                return Err(anyhow!("No active audit log file"));
            }
        }

        // Update file size in stats
        {
            let mut stats = self.stats.write().await;
            stats.current_file_size += entry_size;
        }

        Ok(())
    }

    /// Check if log file should be rotated
    async fn should_rotate_file(&self, additional_bytes: u64) -> Result<bool> {
        let stats = self.stats.read().await;
        let current_size_mb = (stats.current_file_size + additional_bytes) / (1024 * 1024);

        Ok(current_size_mb >= self.config.max_file_size_mb)
    }

    /// Rotate log file to a new one
    fn rotate_log_file(&self) -> Result<()> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = match self.config.log_format {
            AuditLogFormat::Json => format!("audit_{}.jsonl", timestamp),
            AuditLogFormat::Csv => format!("audit_{}.csv", timestamp),
            AuditLogFormat::Syslog => format!("audit_{}.log", timestamp),
            AuditLogFormat::Custom(_) => format!("audit_{}.log", timestamp),
        };

        let file_path = self.config.log_directory.join(filename);

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .map_err(|e| anyhow!("Failed to create audit log file: {}", e))?;

        let writer = BufWriter::new(file);

        // Write CSV header if needed
        if matches!(self.config.log_format, AuditLogFormat::Csv) {
            // Note: In a real implementation, you'd write the CSV header here
        }

        // Update current file
        {
            let mut file_lock = self
                .current_file
                .lock()
                .map_err(|e| anyhow!("Failed to acquire file lock: {}", e))?;
            *file_lock = Some(writer);
        }

        {
            let mut path_lock = self
                .current_file_path
                .lock()
                .map_err(|e| anyhow!("Failed to acquire path lock: {}", e))?;
            *path_lock = Some(file_path.clone());
        }

        info!(file_path = %file_path.display(), "Audit log file rotated");
        Ok(())
    }

    /// Update audit statistics
    async fn update_stats(&self, entry: &AuditLogEntry) {
        let mut stats = self.stats.write().await;

        stats.total_events += 1;
        *stats
            .events_by_severity
            .entry(entry.severity.clone())
            .or_insert(0) += 1;
        *stats
            .events_by_type
            .entry(entry.event_type.to_string())
            .or_insert(0) += 1;
        *stats
            .events_by_outcome
            .entry(entry.outcome.clone())
            .or_insert(0) += 1;

        if let Some(ref tenant_id) = entry.tenant_id {
            *stats.events_by_tenant.entry(tenant_id.clone()).or_insert(0) += 1;
        }
    }

    /// Check for alert conditions
    async fn check_alerts(&self, entry: &AuditLogEntry) {
        let alert_key = format!("{}_{}", entry.event_type, entry.severity);

        let mut counters = self.alert_counters.write().await;
        let count = counters.entry(alert_key.clone()).or_insert(0);
        *count += 1;

        // Check thresholds (simplified implementation)
        let should_alert = match entry.event_type {
            SecurityEventType::UnauthorizedAccess => {
                *count >= self.config.alert_thresholds.failed_auth_attempts_per_minute
            }
            SecurityEventType::RateLimitExceeded => {
                *count >= self.config.alert_thresholds.rate_limit_violations_per_hour
            }
            SecurityEventType::BudgetExceeded => {
                *count >= self.config.alert_thresholds.budget_violations_per_hour
            }
            SecurityEventType::PiiDetected => {
                *count >= self.config.alert_thresholds.pii_detections_per_hour
            }
            _ => {
                entry.severity == SecuritySeverity::Critical
                    && *count >= self.config.alert_thresholds.critical_events_per_hour
            }
        };

        if should_alert {
            warn!(
                event_type = %entry.event_type,
                severity = %entry.severity,
                count = *count,
                alert_key = alert_key,
                "Alert threshold exceeded"
            );

            // In a production system, you would send actual alerts here
            // (email, Slack, PagerDuty, etc.)
        }
    }

    /// Get current audit statistics
    pub async fn get_stats(&self) -> AuditStats {
        self.stats.read().await.clone()
    }

    /// Clean up old log files based on retention policy
    pub async fn cleanup_old_logs(&self) -> Result<u32> {
        let cutoff_date = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);
        let mut removed_count = 0;

        let entries = std::fs::read_dir(&self.config.log_directory)
            .map_err(|e| anyhow!("Failed to read log directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| anyhow!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.is_file() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(created) = metadata.created() {
                        let created_datetime: DateTime<Utc> = created.into();
                        if created_datetime < cutoff_date {
                            if let Err(e) = std::fs::remove_file(&path) {
                                warn!(file = %path.display(), error = %e, "Failed to remove old audit log");
                            } else {
                                removed_count += 1;
                                debug!(file = %path.display(), "Removed old audit log file");
                            }
                        }
                    }
                }
            }
        }

        if removed_count > 0 {
            info!(removed_count, "Cleaned up old audit log files");
        }

        Ok(removed_count)
    }

    /// Search audit logs for specific criteria
    pub async fn search_logs(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        _event_types: Option<Vec<SecurityEventType>>,
        _tenant_id: Option<&TenantId>,
        _severity: Option<SecuritySeverity>,
    ) -> Result<Vec<AuditLogEntry>> {
        // This is a simplified implementation
        // In production, you might use a database or log indexing system
        let results = Vec::new();

        // For demonstration, return empty results
        // A real implementation would search through log files or a database

        debug!(
            start_time = %start_time,
            end_time = %end_time,
            "Audit log search completed"
        );

        Ok(results)
    }

    /// Export audit logs for compliance
    pub async fn export_logs(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        _format: AuditLogFormat,
        output_path: &Path,
    ) -> Result<u64> {
        // Simplified implementation
        // In production, this would collect logs from the specified period
        // and export them in the requested format

        info!(
            start_time = %start_time,
            end_time = %end_time,
            output_path = %output_path.display(),
            "Audit log export completed"
        );

        Ok(0) // Return number of exported entries
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(None).expect("Failed to create default audit logger")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio;

    fn create_test_config() -> AuditConfig {
        let temp_dir = TempDir::new().unwrap();
        AuditConfig {
            log_directory: temp_dir.path().to_path_buf(),
            max_file_size_mb: 1, // Small for testing
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_audit_logger_creation() {
        let config = create_test_config();
        let logger = AuditLogger::new(Some(config));
        assert!(logger.is_ok());
    }

    #[tokio::test]
    async fn test_audit_log_entry_creation() {
        let logger = AuditLogger::new(Some(create_test_config())).unwrap();
        let tenant_id = TenantId::from("test-tenant");

        let context =
            SecurityContext::new(tenant_id, "test-key".to_string(), "127.0.0.1".to_string());

        let details = AuditDetails {
            description: "Test API key usage".to_string(),
            error_message: None,
            request_payload: None,
            response_payload: None,
            duration_ms: Some(100),
            bytes_processed: Some(1024),
            cost_usd: Some(0.01),
            tokens_used: Some(500),
            model_name: Some("gpt-3.5-turbo".to_string()),
            rate_limit_info: None,
            pii_redacted: false,
            pii_detections: None,
        };

        let entry = logger.create_api_key_usage_entry(&context, AuditOutcome::Success, details);

        assert_eq!(entry.event_type, SecurityEventType::ApiKeyUsed);
        assert_eq!(entry.outcome, AuditOutcome::Success);
        assert!(entry.tenant_id.is_some());
    }

    #[tokio::test]
    async fn test_log_event() {
        let logger = AuditLogger::new(Some(create_test_config())).unwrap();

        let entry = AuditLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: SecurityEventType::ApiKeyUsed,
            severity: SecuritySeverity::Low,
            tenant_id: Some(TenantId::from("test")),
            user_id: None,
            api_key_id: Some("test-key".to_string()),
            request_id: Some("req-123".to_string()),
            source_ip: Some("127.0.0.1".to_string()),
            user_agent: None,
            resource: None,
            action: "authenticate".to_string(),
            outcome: AuditOutcome::Success,
            details: AuditDetails {
                description: "Test event".to_string(),
                error_message: None,
                request_payload: None,
                response_payload: None,
                duration_ms: None,
                bytes_processed: None,
                cost_usd: None,
                tokens_used: None,
                model_name: None,
                rate_limit_info: None,
                pii_redacted: false,
                pii_detections: None,
            },
            metadata: HashMap::new(),
        };

        let result = logger.log_event(entry).await;
        assert!(result.is_ok());

        let stats = logger.get_stats().await;
        assert_eq!(stats.total_events, 1);
    }

    #[test]
    fn test_log_formatting() {
        let logger = AuditLogger::new(Some(create_test_config())).unwrap();

        let entry = AuditLogEntry {
            id: "test-id".to_string(),
            timestamp: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            event_type: SecurityEventType::ApiKeyUsed,
            severity: SecuritySeverity::Low,
            tenant_id: Some(TenantId::from("test")),
            user_id: None,
            api_key_id: Some("key-123".to_string()),
            request_id: Some("req-123".to_string()),
            source_ip: Some("127.0.0.1".to_string()),
            user_agent: None,
            resource: None,
            action: "authenticate".to_string(),
            outcome: AuditOutcome::Success,
            details: AuditDetails {
                description: "Test event".to_string(),
                error_message: None,
                request_payload: None,
                response_payload: None,
                duration_ms: None,
                bytes_processed: None,
                cost_usd: None,
                tokens_used: None,
                model_name: None,
                rate_limit_info: None,
                pii_redacted: false,
                pii_detections: None,
            },
            metadata: HashMap::new(),
        };

        let json_result = logger.format_log_entry(&entry);
        assert!(json_result.is_ok());
        assert!(json_result.unwrap().contains("ApiKeyUsed"));
    }
}
