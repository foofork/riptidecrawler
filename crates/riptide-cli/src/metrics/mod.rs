//! CLI Metrics Collection Module
//!
//! This module provides lightweight metrics collection for CLI operations with:
//! - < 5ms overhead per command
//! - Thread-safe counters and timers
//! - Efficient local storage with automatic rotation
//! - Easy integration with existing command structure
//! - OpenTelemetry integration via riptide-core
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────┐
//! │   CLI Command   │
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │    Collector    │ ◄─── Thread-safe, low-latency
//! └────────┬────────┘
//!          │
//!          ├──────────────┬──────────────┐
//!          ▼              ▼              ▼
//!    ┌─────────┐    ┌──────────┐   ┌──────────┐
//!    │ Storage │    │Aggregator│   │Telemetry │
//!    └─────────┘    └──────────┘   └──────────┘
//!         │              │              │
//!         ▼              ▼              ▼
//!    [metrics.json] [Statistics]  [OpenTelemetry]
//! ```
//!
//! # Usage Example
//!
//! ```rust
//! use riptide_cli::metrics::MetricsManager;
//!
//! async fn execute_command() -> anyhow::Result<()> {
//!     let metrics = MetricsManager::global();
//!
//!     // Start tracking
//!     let tracking_id = metrics.start_command("extract").await?;
//!
//!     // Do work...
//!     metrics.record_progress(&tracking_id, 10, 1024, 5, 2).await?;
//!
//!     // Complete
//!     metrics.complete_command(&tracking_id).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod aggregator;
pub mod collector;
pub mod storage;
pub mod types;

use anyhow::Result;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::RwLock;

pub use aggregator::MetricsAggregator;
pub use collector::MetricsCollector;
pub use storage::{ExportFormat, MetricsStorage};
pub use types::{CliMetricsSummary, CommandAggregates, CommandMetrics, MetricsStorageConfig};

/// Global metrics manager instance
static GLOBAL_METRICS: Lazy<Arc<MetricsManager>> =
    Lazy::new(|| Arc::new(MetricsManager::new().expect("Failed to initialize metrics manager")));

/// Central metrics manager coordinating collection, storage, and aggregation
pub struct MetricsManager {
    /// Metrics collector
    collector: Arc<MetricsCollector>,

    /// Persistent storage
    storage: Arc<RwLock<MetricsStorage>>,

    /// Metrics aggregator
    aggregator: Arc<RwLock<MetricsAggregator>>,

    /// Configuration (stored for potential future use)
    #[allow(dead_code)]
    config: MetricsStorageConfig,
}

impl MetricsManager {
    /// Create new metrics manager with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(MetricsStorageConfig::default())
    }

    /// Create metrics manager with custom configuration
    pub fn with_config(config: MetricsStorageConfig) -> Result<Self> {
        let collector = Arc::new(MetricsCollector::new());
        let storage = Arc::new(RwLock::new(MetricsStorage::with_config(config.clone())?));
        let aggregator = Arc::new(RwLock::new(MetricsAggregator::new()));

        Ok(Self {
            collector,
            storage,
            aggregator,
            config,
        })
    }

    /// Get global metrics manager instance
    pub fn global() -> Arc<MetricsManager> {
        GLOBAL_METRICS.clone()
    }

    /// Start tracking a command execution
    pub async fn start_command(&self, command_name: impl Into<String>) -> Result<String> {
        self.collector.start_command(command_name)
    }

    /// Record progress during command execution
    pub async fn record_progress(
        &self,
        tracking_id: &str,
        items_processed: u64,
        bytes_transferred: u64,
        cache_hits: u64,
        api_calls: u64,
    ) -> Result<()> {
        self.collector.record_progress(
            tracking_id,
            items_processed,
            bytes_transferred,
            cache_hits,
            api_calls,
        )
    }

    /// Complete command successfully
    pub async fn complete_command(&self, tracking_id: &str) -> Result<()> {
        let metrics = self.collector.complete_command(tracking_id)?;

        // Store to disk
        let mut storage = self.storage.write().await;
        storage.record_command(metrics)?;

        Ok(())
    }

    /// Complete command with failure
    pub async fn fail_command(&self, tracking_id: &str, error: impl Into<String>) -> Result<()> {
        let metrics = self.collector.fail_command(tracking_id, error)?;

        // Store to disk
        let mut storage = self.storage.write().await;
        storage.record_command(metrics)?;

        Ok(())
    }

    /// Get metrics summary
    pub async fn get_summary(&self) -> Result<CliMetricsSummary> {
        let storage = self.storage.read().await;
        Ok(storage.get_summary().clone())
    }

    /// Get aggregated metrics by command
    pub async fn get_aggregates(
        &self,
    ) -> Result<std::collections::HashMap<String, CommandAggregates>> {
        let storage = self.storage.read().await;
        let mut aggregator = self.aggregator.write().await;

        let history = storage.get_command_history();
        Ok(aggregator.aggregate_by_command(history))
    }

    /// Get recent command history
    pub async fn get_recent_commands(&self, limit: usize) -> Result<Vec<CommandMetrics>> {
        let storage = self.storage.read().await;
        Ok(storage.get_recent_commands(limit))
    }

    /// Export metrics in specified format
    pub async fn export(&self, format: ExportFormat) -> Result<String> {
        let storage = self.storage.read().await;
        storage.export(format)
    }

    /// Get counter value
    pub fn get_counter(&self, name: &str) -> Result<u64> {
        self.collector.get_counter(name)
    }

    /// Increment counter
    pub fn increment_counter(&self, name: &str) -> Result<()> {
        self.collector.increment_counter(name)
    }

    /// Record metric data point
    pub fn record_metric(&self, metric_name: &str, value: f64) -> Result<()> {
        self.collector.record_metric(metric_name, value)
    }

    /// Get collector reference for advanced usage
    pub fn collector(&self) -> &Arc<MetricsCollector> {
        &self.collector
    }

    /// Get storage reference for advanced usage
    pub fn storage(&self) -> &Arc<RwLock<MetricsStorage>> {
        &self.storage
    }

    /// Get aggregator reference for advanced usage
    pub fn aggregator(&self) -> &Arc<RwLock<MetricsAggregator>> {
        &self.aggregator
    }
}

/// Helper macro for tracking command execution with automatic cleanup
///
/// # Example
///
/// ```ignore
/// track_command!("extract", {
///     // Your command logic here
///     let result = extract_data().await?;
///     Ok(result)
/// })
/// ```
#[macro_export]
macro_rules! track_command {
    ($command_name:expr, $block:block) => {{
        let metrics = $crate::metrics::MetricsManager::global();
        let tracking_id = metrics.start_command($command_name).await?;

        let result: anyhow::Result<_> = async { $block }.await;

        match result {
            Ok(value) => {
                metrics.complete_command(&tracking_id).await?;
                Ok(value)
            }
            Err(e) => {
                metrics.fail_command(&tracking_id, e.to_string()).await?;
                Err(e)
            }
        }
    }};
}

/// Integration helper for OpenTelemetry
pub mod telemetry_integration {
    use super::*;

    /// Record command metrics to OpenTelemetry if available
    pub async fn record_to_telemetry(
        metrics: &CommandMetrics,
        _telemetry: Option<&()>, // TODO: Replace with riptide-monitoring::TelemetrySystem when needed
    ) -> Result<()> {
        // P2-F1 Day 4-5: Removed riptide-core dependency
        // Integration with riptide-monitoring telemetry system (to be implemented)

        if let Some(duration_ms) = metrics.duration_ms {
            tracing::info!(
                command = %metrics.command_name,
                duration_ms = %duration_ms,
                success = %metrics.success,
                items = %metrics.items_processed,
                "Command execution recorded"
            );
        }

        Ok(())
    }

    /// Export metrics to OpenTelemetry format
    pub fn to_otel_attributes(metrics: &CommandMetrics) -> Vec<opentelemetry::KeyValue> {
        let mut attrs = vec![
            opentelemetry::KeyValue::new("command.name", metrics.command_name.clone()),
            opentelemetry::KeyValue::new("command.success", metrics.success),
            opentelemetry::KeyValue::new("command.items_processed", metrics.items_processed as i64),
            opentelemetry::KeyValue::new(
                "command.bytes_transferred",
                metrics.bytes_transferred as i64,
            ),
            opentelemetry::KeyValue::new("command.cache_hits", metrics.cache_hits as i64),
            opentelemetry::KeyValue::new("command.api_calls", metrics.api_calls as i64),
        ];

        if let Some(duration_ms) = metrics.duration_ms {
            attrs.push(opentelemetry::KeyValue::new(
                "command.duration_ms",
                duration_ms as i64,
            ));
        }

        if let Some(ref error) = metrics.error {
            attrs.push(opentelemetry::KeyValue::new("command.error", error.clone()));
        }

        attrs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_manager() {
        let manager = MetricsManager::new().unwrap();

        let tracking_id = manager.start_command("test").await.unwrap();
        manager
            .record_progress(&tracking_id, 5, 512, 2, 1)
            .await
            .unwrap();
        manager.complete_command(&tracking_id).await.unwrap();

        let summary = manager.get_summary().await.unwrap();
        assert_eq!(summary.total_commands, 1);
    }

    #[tokio::test]
    async fn test_global_instance() {
        let metrics1 = MetricsManager::global();
        let metrics2 = MetricsManager::global();

        assert!(Arc::ptr_eq(&metrics1, &metrics2));
    }

    #[tokio::test]
    async fn test_export_formats() {
        let manager = MetricsManager::new().unwrap();

        let tracking_id = manager.start_command("export_test").await.unwrap();
        manager.complete_command(&tracking_id).await.unwrap();

        // Test JSON export
        let json = manager.export(ExportFormat::Json).await.unwrap();
        assert!(json.contains("export_test"));

        // Test CSV export
        let csv = manager.export(ExportFormat::Csv).await.unwrap();
        assert!(csv.contains("timestamp,command,duration_ms"));

        // Test Prometheus export
        let prom = manager.export(ExportFormat::Prometheus).await.unwrap();
        assert!(prom.contains("riptide_cli_commands_total"));
    }
}
