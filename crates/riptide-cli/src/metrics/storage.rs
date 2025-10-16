//! Local metrics storage with automatic rotation
//!
//! This module handles persistent storage of metrics to disk with
//! automatic rotation and cleanup to prevent unbounded growth.

use super::types::{CliMetricsSummary, CommandMetrics, MetricsStorageConfig};
use anyhow::{Context, Result};
use chrono::{Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

/// Persistent metrics storage with rotation support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsStorage {
    /// Storage configuration
    config: MetricsStorageConfig,

    /// Command execution history
    command_history: Vec<CommandMetrics>,

    /// Overall metrics summary
    summary: CliMetricsSummary,

    /// Storage file path
    #[serde(skip)]
    storage_path: PathBuf,
}

impl MetricsStorage {
    /// Create new metrics storage with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(MetricsStorageConfig::default())
    }

    /// Create metrics storage with custom configuration
    pub fn with_config(config: MetricsStorageConfig) -> Result<Self> {
        let storage_path = PathBuf::from(&config.storage_path);

        // Ensure parent directory exists
        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent).context("Failed to create metrics storage directory")?;
        }

        let mut storage = Self {
            config,
            command_history: Vec::new(),
            summary: CliMetricsSummary::default(),
            storage_path,
        };

        // Try to load existing metrics
        if storage.storage_path.exists() {
            storage.load()?;
        }

        Ok(storage)
    }

    /// Record a command execution
    pub fn record_command(&mut self, metrics: CommandMetrics) -> Result<()> {
        // Add to history
        self.command_history.push(metrics.clone());

        // Update summary statistics
        self.update_summary(&metrics);

        // Check if rotation is needed
        if self.command_history.len() >= self.config.rotation_threshold {
            self.rotate()?;
        }

        // Auto-save after each command
        self.save()?;

        Ok(())
    }

    /// Update summary statistics with new command metrics
    fn update_summary(&mut self, metrics: &CommandMetrics) {
        self.summary.total_commands += 1;
        self.summary.last_command_time = Utc::now();

        // Update total execution time
        if let Some(duration_ms) = metrics.duration_ms {
            self.summary.total_execution_time_ms += duration_ms;
        }

        // Update totals
        self.summary.total_bytes_transferred += metrics.bytes_transferred;
        self.summary.total_api_calls += metrics.api_calls;

        // Calculate new average
        if self.summary.total_commands > 0 {
            self.summary.avg_command_duration_ms =
                self.summary.total_execution_time_ms as f64 / self.summary.total_commands as f64;
        }

        // Update per-command statistics
        let command_stats = self
            .summary
            .command_stats
            .entry(metrics.command_name.clone())
            .or_insert_with(|| super::types::CommandAggregates::new(metrics.command_name.clone()));

        command_stats.total_executions += 1;
        command_stats.last_executed = Utc::now();

        if metrics.success {
            command_stats.successful_executions += 1;
        } else {
            command_stats.failed_executions += 1;

            // Update error distribution
            if let Some(ref error) = metrics.error {
                let error_type = extract_error_category(error);
                *command_stats
                    .error_distribution
                    .entry(error_type)
                    .or_insert(0) += 1;
            }
        }

        // Update aggregates
        command_stats.total_items_processed += metrics.items_processed;
        command_stats.total_bytes_transferred += metrics.bytes_transferred;
        command_stats.total_api_calls += metrics.api_calls;

        // Calculate overall success rate
        let total_successful: u64 = self
            .summary
            .command_stats
            .values()
            .map(|stats| stats.successful_executions)
            .sum();
        self.summary.overall_success_rate = if self.summary.total_commands > 0 {
            (total_successful as f64 / self.summary.total_commands as f64) * 100.0
        } else {
            0.0
        };
    }

    /// Rotate metrics by removing old entries
    fn rotate(&mut self) -> Result<()> {
        if !self.config.auto_cleanup {
            return Ok(());
        }

        let now = Utc::now();
        let retention_cutoff = now - ChronoDuration::days(self.config.retention_days as i64);

        // Remove old entries
        self.command_history
            .retain(|m| m.started_at > retention_cutoff);

        // Keep only the most recent entries if still over threshold
        if self.command_history.len() > self.config.max_command_history {
            let excess = self.command_history.len() - self.config.max_command_history;
            self.command_history.drain(0..excess);
        }

        // Archive old data before rotation
        if self.command_history.len() > self.config.rotation_threshold / 2 {
            self.archive()?;
        }

        Ok(())
    }

    /// Archive metrics to a separate file
    fn archive(&self) -> Result<()> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let archive_path = self
            .storage_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(format!("metrics_archive_{}.json", timestamp));

        let file = File::create(&archive_path).context("Failed to create archive file")?;
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, &self.command_history)
            .context("Failed to write archive")?;

        Ok(())
    }

    /// Save metrics to disk
    pub fn save(&self) -> Result<()> {
        // Create temporary file
        let temp_path = self.storage_path.with_extension("tmp");

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)
            .context("Failed to create temporary metrics file")?;

        let writer = BufWriter::new(file);

        // Serialize to JSON
        serde_json::to_writer_pretty(writer, &self).context("Failed to serialize metrics")?;

        // Atomic rename
        fs::rename(&temp_path, &self.storage_path)
            .context("Failed to move metrics file into place")?;

        Ok(())
    }

    /// Load metrics from disk
    pub fn load(&mut self) -> Result<()> {
        let file = File::open(&self.storage_path).context("Failed to open metrics file")?;
        let reader = BufReader::new(file);

        let loaded: Self =
            serde_json::from_reader(reader).context("Failed to deserialize metrics")?;

        self.command_history = loaded.command_history;
        self.summary = loaded.summary;

        // Run cleanup in case retention policy changed
        if self.config.auto_cleanup {
            self.rotate()?;
        }

        Ok(())
    }

    /// Get command history
    pub fn get_command_history(&self) -> &[CommandMetrics] {
        &self.command_history
    }

    /// Get recent command history (last N commands)
    pub fn get_recent_commands(&self, limit: usize) -> Vec<CommandMetrics> {
        let len = self.command_history.len();
        let start = if len > limit { len - limit } else { 0 };
        self.command_history[start..].to_vec()
    }

    /// Get commands filtered by name
    pub fn get_commands_by_name(&self, command_name: &str) -> Vec<CommandMetrics> {
        self.command_history
            .iter()
            .filter(|m| m.command_name == command_name)
            .cloned()
            .collect()
    }

    /// Get metrics summary
    pub fn get_summary(&self) -> &CliMetricsSummary {
        &self.summary
    }

    /// Clear all metrics (for testing)
    pub fn clear(&mut self) -> Result<()> {
        self.command_history.clear();
        self.summary = CliMetricsSummary::default();
        self.save()
    }

    /// Export metrics in different formats
    pub fn export(&self, format: ExportFormat) -> Result<String> {
        match format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(&self).context("Failed to serialize to JSON")
            }
            ExportFormat::Csv => self.export_csv(),
            ExportFormat::Prometheus => self.export_prometheus(),
        }
    }

    /// Export metrics as CSV
    fn export_csv(&self) -> Result<String> {
        let mut csv = String::from("timestamp,command,duration_ms,success,items_processed,bytes_transferred,cache_hits,api_calls,peak_memory_bytes,error\n");

        for metrics in &self.command_history {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{}\n",
                metrics.started_at.to_rfc3339(),
                metrics.command_name,
                metrics.duration_ms.unwrap_or(0),
                metrics.success,
                metrics.items_processed,
                metrics.bytes_transferred,
                metrics.cache_hits,
                metrics.api_calls,
                metrics.peak_memory_bytes,
                metrics.error.as_deref().unwrap_or("")
            ));
        }

        Ok(csv)
    }

    /// Export metrics in Prometheus format
    fn export_prometheus(&self) -> Result<String> {
        let mut output = String::new();

        // Total commands
        output.push_str("# HELP riptide_cli_commands_total Total CLI commands executed\n");
        output.push_str("# TYPE riptide_cli_commands_total counter\n");
        output.push_str(&format!(
            "riptide_cli_commands_total {}\n",
            self.summary.total_commands
        ));

        // Success rate
        output.push_str("# HELP riptide_cli_success_rate_percent CLI command success rate\n");
        output.push_str("# TYPE riptide_cli_success_rate_percent gauge\n");
        output.push_str(&format!(
            "riptide_cli_success_rate_percent {:.2}\n",
            self.summary.overall_success_rate
        ));

        // Per-command metrics
        for (cmd, stats) in &self.summary.command_stats {
            let safe_cmd = cmd.replace('-', "_");

            output.push_str(&format!(
                "# HELP riptide_cli_command_{}_total Total executions of {} command\n",
                safe_cmd, cmd
            ));
            output.push_str(&format!(
                "# TYPE riptide_cli_command_{}_total counter\n",
                safe_cmd
            ));
            output.push_str(&format!(
                "riptide_cli_command_{}_total {}\n",
                safe_cmd, stats.total_executions
            ));

            output.push_str(&format!(
                "riptide_cli_command_{}_success_rate {:.2}\n",
                safe_cmd,
                stats.success_rate()
            ));
        }

        Ok(output)
    }
}

impl Default for MetricsStorage {
    fn default() -> Self {
        Self::new().expect("Failed to create default metrics storage")
    }
}

/// Export format options
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    Csv,
    Prometheus,
}

/// Extract error category from error message
#[allow(dead_code)]
fn extract_error_category(error: &str) -> String {
    let error_lower = error.to_lowercase();

    if error_lower.contains("timeout") {
        "timeout".to_string()
    } else if error_lower.contains("network") || error_lower.contains("connection") {
        "network".to_string()
    } else if error_lower.contains("permission") || error_lower.contains("denied") {
        "permission".to_string()
    } else if error_lower.contains("not found") || error_lower.contains("404") {
        "not_found".to_string()
    } else if error_lower.contains("parse") || error_lower.contains("invalid") {
        "parse".to_string()
    } else {
        "unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::tempdir;

    #[test]
    fn test_storage_creation() {
        let dir = tempdir().unwrap();
        let config = MetricsStorageConfig {
            storage_path: dir
                .path()
                .join("metrics.json")
                .to_string_lossy()
                .to_string(),
            ..Default::default()
        };

        let storage = MetricsStorage::with_config(config).unwrap();
        assert_eq!(storage.command_history.len(), 0);
    }

    #[test]
    fn test_record_and_save() {
        let dir = tempdir().unwrap();
        let config = MetricsStorageConfig {
            storage_path: dir
                .path()
                .join("metrics.json")
                .to_string_lossy()
                .to_string(),
            ..Default::default()
        };

        let mut storage = MetricsStorage::with_config(config).unwrap();

        let mut metrics = CommandMetrics::new("extract");
        metrics.complete(Duration::from_millis(100));
        metrics.items_processed = 5;

        storage.record_command(metrics).unwrap();

        assert_eq!(storage.command_history.len(), 1);
        assert_eq!(storage.summary.total_commands, 1);
    }

    #[test]
    fn test_load_and_save() {
        let dir = tempdir().unwrap();
        let storage_path = dir.path().join("metrics.json");
        let config = MetricsStorageConfig {
            storage_path: storage_path.to_string_lossy().to_string(),
            ..Default::default()
        };

        // Create and save
        {
            let mut storage = MetricsStorage::with_config(config.clone()).unwrap();
            let mut metrics = CommandMetrics::new("crawl");
            metrics.complete(Duration::from_millis(200));
            storage.record_command(metrics).unwrap();
        }

        // Load
        {
            let storage = MetricsStorage::with_config(config).unwrap();
            assert_eq!(storage.command_history.len(), 1);
            assert_eq!(storage.command_history[0].command_name, "crawl");
        }
    }

    #[test]
    fn test_csv_export() {
        let dir = tempdir().unwrap();
        let config = MetricsStorageConfig {
            storage_path: dir
                .path()
                .join("metrics.json")
                .to_string_lossy()
                .to_string(),
            ..Default::default()
        };

        let mut storage = MetricsStorage::with_config(config).unwrap();

        let mut metrics = CommandMetrics::new("search");
        metrics.complete(Duration::from_millis(150));
        storage.record_command(metrics).unwrap();

        let csv = storage.export(ExportFormat::Csv).unwrap();
        assert!(csv.contains("timestamp,command,duration_ms"));
        assert!(csv.contains("search"));
    }

    #[test]
    fn test_prometheus_export() {
        let dir = tempdir().unwrap();
        let config = MetricsStorageConfig {
            storage_path: dir
                .path()
                .join("metrics.json")
                .to_string_lossy()
                .to_string(),
            ..Default::default()
        };

        let mut storage = MetricsStorage::with_config(config).unwrap();

        let mut metrics = CommandMetrics::new("extract");
        metrics.complete(Duration::from_millis(100));
        storage.record_command(metrics).unwrap();

        let prom = storage.export(ExportFormat::Prometheus).unwrap();
        assert!(prom.contains("riptide_cli_commands_total"));
        assert!(prom.contains("riptide_cli_success_rate_percent"));
    }
}
