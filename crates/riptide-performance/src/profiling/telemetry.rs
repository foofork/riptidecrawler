//! OpenTelemetry telemetry integration for memory profiling
//!
//! This module exports memory profiling data to OpenTelemetry Protocol (OTLP)
//! for integration with observability platforms like Grafana, Prometheus, and Jaeger.

use anyhow::Result;
use opentelemetry::{
    metrics::{Histogram, Meter, MeterProvider, ObservableGauge},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{metrics::SdkMeterProvider, runtime, Resource};
use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, SERVICE_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::{AllocationInfo, LeakAnalysis, LeakInfo, MemorySnapshot};

/// Configuration for telemetry export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// OTLP endpoint (e.g., "http://localhost:4317")
    pub endpoint: String,
    /// Service name for telemetry
    pub service_name: String,
    /// Service version
    pub service_version: String,
    /// Export interval in seconds
    pub export_interval_seconds: u64,
    /// Enable telemetry export
    pub enabled: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:4317".to_string(),
            service_name: "riptide-performance".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            export_interval_seconds: 10,
            enabled: false, // Disabled by default
        }
    }
}

/// Memory telemetry exporter for OpenTelemetry
#[allow(dead_code)]
pub struct MemoryTelemetryExporter {
    config: TelemetryConfig,
    meter_provider: Option<SdkMeterProvider>,
    meter: Option<Meter>,

    // Gauge metrics
    memory_rss_gauge: Option<ObservableGauge<u64>>,
    memory_heap_gauge: Option<ObservableGauge<u64>>,
    memory_virtual_gauge: Option<ObservableGauge<u64>>,

    // Histograms for size distributions
    allocation_size_histogram: Option<Histogram<u64>>,

    // Cached snapshot for observable gauges
    cached_snapshot: Arc<RwLock<Option<MemorySnapshot>>>,
    cached_leak_analysis: Arc<RwLock<Option<LeakAnalysis>>>,
}

impl MemoryTelemetryExporter {
    /// Create a new memory telemetry exporter
    pub fn new(config: TelemetryConfig) -> Result<Self> {
        if !config.enabled {
            info!("Telemetry export is disabled");
            return Ok(Self {
                config,
                meter_provider: None,
                meter: None,
                memory_rss_gauge: None,
                memory_heap_gauge: None,
                memory_virtual_gauge: None,
                allocation_size_histogram: None,
                cached_snapshot: Arc::new(RwLock::new(None)),
                cached_leak_analysis: Arc::new(RwLock::new(None)),
            });
        }

        info!(
            endpoint = %config.endpoint,
            service = %config.service_name,
            "Initializing OpenTelemetry metrics exporter"
        );

        // Create resource with service information
        let resource = Resource::new(vec![
            KeyValue::new(SERVICE_NAME, config.service_name.clone()),
            KeyValue::new(SERVICE_VERSION, config.service_version.clone()),
        ]);

        // Create OTLP metrics pipeline
        let meter_provider = opentelemetry_otlp::new_pipeline()
            .metrics(runtime::Tokio)
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(&config.endpoint),
            )
            .with_resource(resource)
            .with_period(std::time::Duration::from_secs(
                config.export_interval_seconds,
            ))
            .build()?;

        // Create meter
        let meter = meter_provider.meter("riptide-performance-profiling");

        let cached_snapshot = Arc::new(RwLock::new(None));
        let cached_leak_analysis = Arc::new(RwLock::new(None));

        // Create gauge metrics with callbacks
        let snapshot_clone: Arc<RwLock<Option<MemorySnapshot>>> = Arc::clone(&cached_snapshot);
        let memory_rss_gauge = meter
            .u64_observable_gauge("memory.rss_bytes")
            .with_description("Resident Set Size (RSS) in bytes")
            .with_callback(move |observer| {
                let snapshot = snapshot_clone.blocking_read();
                if let Some(ref snap) = *snapshot {
                    observer.observe(snap.rss_bytes, &[]);
                }
            })
            .init();

        let snapshot_clone: Arc<RwLock<Option<MemorySnapshot>>> = Arc::clone(&cached_snapshot);
        let memory_heap_gauge = meter
            .u64_observable_gauge("memory.heap_bytes")
            .with_description("Heap memory usage in bytes")
            .with_callback(move |observer| {
                let snapshot = snapshot_clone.blocking_read();
                if let Some(ref snap) = *snapshot {
                    observer.observe(snap.heap_bytes, &[]);
                }
            })
            .init();

        let snapshot_clone: Arc<RwLock<Option<MemorySnapshot>>> = Arc::clone(&cached_snapshot);
        let memory_virtual_gauge = meter
            .u64_observable_gauge("memory.virtual_bytes")
            .with_description("Virtual memory usage in bytes")
            .with_callback(move |observer| {
                let snapshot = snapshot_clone.blocking_read();
                if let Some(ref snap) = *snapshot {
                    observer.observe(snap.virtual_bytes, &[]);
                }
            })
            .init();

        // Create histogram for allocation sizes
        let allocation_size_histogram = meter
            .u64_histogram("memory.allocation.size_bytes")
            .with_description("Distribution of allocation sizes in bytes")
            .with_unit("bytes")
            .init();

        info!("OpenTelemetry metrics exporter initialized successfully");

        Ok(Self {
            config,
            meter_provider: Some(meter_provider),
            meter: Some(meter),
            memory_rss_gauge: Some(memory_rss_gauge),
            memory_heap_gauge: Some(memory_heap_gauge),
            memory_virtual_gauge: Some(memory_virtual_gauge),
            allocation_size_histogram: Some(allocation_size_histogram),
            cached_snapshot,
            cached_leak_analysis,
        })
    }

    /// Export memory snapshot to OpenTelemetry
    pub async fn export_snapshot(
        &self,
        snapshot: &MemorySnapshot,
        component: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        debug!(
            rss_mb = snapshot.rss_bytes as f64 / 1024.0 / 1024.0,
            heap_mb = snapshot.heap_bytes as f64 / 1024.0 / 1024.0,
            "Exporting memory snapshot to telemetry"
        );

        // Update cached snapshot for observable gauges
        *self.cached_snapshot.write().await = Some(snapshot.clone());

        // Create labels
        let mut labels = Vec::new();
        if let Some(component) = component {
            labels.push(KeyValue::new("component", component.to_string()));
        }
        if let Some(session_id) = session_id {
            labels.push(KeyValue::new("session_id", session_id.to_string()));
        }

        // Note: Observable gauges are automatically exported via callbacks
        // No need to manually record gauge values here

        Ok(())
    }

    /// Export leak analysis to OpenTelemetry
    pub async fn export_leak_analysis(&self, analysis: &LeakAnalysis) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let Some(ref meter) = self.meter else {
            warn!("Meter not initialized, cannot export leak analysis");
            return Ok(());
        };

        info!(
            potential_leaks = analysis.potential_leaks.len(),
            growth_rate_mb_h = analysis.growth_rate_mb_per_hour,
            "Exporting leak analysis to telemetry"
        );

        // Update cached leak analysis
        *self.cached_leak_analysis.write().await = Some(analysis.clone());

        // Create counters for leak metrics
        let leak_count_counter = meter
            .u64_counter("memory.leak.count")
            .with_description("Number of potential memory leaks detected")
            .init();

        let leak_growth_gauge = meter
            .f64_gauge("memory.leak.growth_rate_mb_h")
            .with_description("Memory leak growth rate in MB/hour")
            .init();

        let leak_size_histogram = meter
            .u64_histogram("memory.leak.size_bytes")
            .with_description("Size distribution of potential memory leaks")
            .with_unit("bytes")
            .init();

        // Export leak count
        leak_count_counter.add(
            analysis.potential_leaks.len() as u64,
            &[KeyValue::new("status", "detected")],
        );

        // Export growth rate
        leak_growth_gauge.record(
            analysis.growth_rate_mb_per_hour,
            &[KeyValue::new("metric", "growth_rate")],
        );

        // Export individual leak information
        for leak in &analysis.potential_leaks {
            let labels = vec![
                KeyValue::new("component", leak.component.clone()),
                KeyValue::new("severity", self.calculate_leak_severity(leak)),
            ];

            leak_size_histogram.record(leak.total_size_bytes, &labels);

            // Record allocation count for this leak
            let allocation_counter = meter
                .u64_counter("memory.leak.allocation_count")
                .with_description("Number of allocations contributing to leak")
                .init();

            allocation_counter.add(leak.allocation_count, &labels);
        }

        debug!("Leak analysis exported successfully");
        Ok(())
    }

    /// Export allocation statistics to OpenTelemetry
    pub async fn export_allocations(
        &self,
        allocations: &[AllocationInfo],
        component: Option<&str>,
    ) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let Some(ref meter) = self.meter else {
            warn!("Meter not initialized, cannot export allocations");
            return Ok(());
        };

        let Some(ref allocation_histogram) = self.allocation_size_histogram else {
            warn!("Allocation histogram not initialized");
            return Ok(());
        };

        debug!(
            allocation_count = allocations.len(),
            "Exporting allocation statistics to telemetry"
        );

        // Create allocation metrics
        let allocation_count_counter = meter
            .u64_counter("memory.allocation.count")
            .with_description("Total number of memory allocations")
            .init();

        let allocation_total_bytes = meter
            .u64_counter("memory.allocation.total_bytes")
            .with_description("Total bytes allocated")
            .with_unit("bytes")
            .init();

        // Calculate statistics
        let total_count = allocations.len() as u64;
        let total_bytes: u64 = allocations.iter().map(|a| a.size as u64).sum();

        let mut labels = Vec::new();
        if let Some(component) = component {
            labels.push(KeyValue::new("component", component.to_string()));
        }

        // Export aggregate metrics
        allocation_count_counter.add(total_count, &labels);
        allocation_total_bytes.add(total_bytes, &labels);

        // Export size distribution to histogram
        for allocation in allocations {
            let mut alloc_labels = labels.clone();
            alloc_labels.push(KeyValue::new("operation", allocation.operation.clone()));

            allocation_histogram.record(allocation.size as u64, &alloc_labels);
        }

        // Calculate and export efficiency score
        let efficiency_score = self.calculate_allocation_efficiency(allocations);
        let efficiency_gauge = meter
            .f64_gauge("memory.allocation.efficiency_score")
            .with_description("Memory allocation efficiency score (0.0-1.0)")
            .init();

        efficiency_gauge.record(efficiency_score, &labels);

        debug!("Allocation statistics exported successfully");
        Ok(())
    }

    /// Export allocation statistics from analyzer data
    pub async fn export_allocation_stats(
        &self,
        top_allocators: &[(String, u64)],
        size_distribution: &HashMap<String, u64>,
    ) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let Some(ref meter) = self.meter else {
            warn!("Meter not initialized, cannot export allocation stats");
            return Ok(());
        };

        debug!("Exporting detailed allocation statistics");

        // Export top allocators
        let allocator_bytes_gauge = meter
            .u64_gauge("memory.allocator.total_bytes")
            .with_description("Total bytes allocated by component")
            .with_unit("bytes")
            .init();

        for (component, bytes) in top_allocators.iter().take(20) {
            allocator_bytes_gauge.record(*bytes, &[KeyValue::new("component", component.clone())]);
        }

        // Export size distribution
        let distribution_counter = meter
            .u64_counter("memory.allocation.distribution")
            .with_description("Distribution of allocations by size category")
            .init();

        for (category, count) in size_distribution {
            distribution_counter.add(*count, &[KeyValue::new("size_category", category.clone())]);
        }

        Ok(())
    }

    /// Flush all pending telemetry data
    pub async fn flush(&self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        if let Some(ref provider) = self.meter_provider {
            info!("Flushing telemetry data");
            provider.force_flush()?;
        }

        Ok(())
    }

    /// Shutdown the telemetry exporter
    pub async fn shutdown(mut self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        if let Some(provider) = self.meter_provider.take() {
            info!("Shutting down telemetry exporter");
            provider.shutdown()?;
        }

        Ok(())
    }

    /// Calculate leak severity level
    fn calculate_leak_severity(&self, leak: &LeakInfo) -> String {
        let size_mb = leak.total_size_bytes as f64 / 1024.0 / 1024.0;
        let growth_mb_h = leak.growth_rate / 1024.0 / 1024.0 * 3600.0;

        if size_mb > 100.0 || growth_mb_h > 50.0 {
            "critical"
        } else if size_mb > 50.0 || growth_mb_h > 20.0 {
            "high"
        } else if size_mb > 10.0 || growth_mb_h > 5.0 {
            "medium"
        } else {
            "low"
        }
        .to_string()
    }

    /// Calculate allocation efficiency score (0.0 = poor, 1.0 = excellent)
    fn calculate_allocation_efficiency(&self, allocations: &[AllocationInfo]) -> f64 {
        if allocations.is_empty() {
            return 1.0;
        }

        let total_size: usize = allocations.iter().map(|a| a.size).sum();
        let avg_size = total_size as f64 / allocations.len() as f64;

        // Efficiency based on:
        // 1. Prefer medium-sized allocations (4KB-64KB)
        // 2. Avoid very small (<1KB) or very large (>1MB) allocations
        let size_efficiency = if (4096.0..=65536.0).contains(&avg_size) {
            1.0
        } else if avg_size < 1024.0 {
            0.3 // Penalty for tiny allocations
        } else if avg_size > 1_048_576.0 {
            0.5 // Penalty for huge allocations
        } else {
            0.7
        };

        // Alignment efficiency
        let well_aligned = allocations
            .iter()
            .filter(|a| a.size % a.alignment == 0)
            .count();
        let alignment_efficiency = well_aligned as f64 / allocations.len() as f64;

        // Overall efficiency
        (size_efficiency + alignment_efficiency) / 2.0
    }
}

impl Drop for MemoryTelemetryExporter {
    fn drop(&mut self) {
        if self.config.enabled {
            if let Some(ref provider) = self.meter_provider {
                if let Err(e) = provider.force_flush() {
                    warn!("Failed to flush telemetry data on drop: {}", e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_telemetry_exporter_creation_disabled() {
        let config = TelemetryConfig {
            enabled: false,
            ..Default::default()
        };

        let exporter = MemoryTelemetryExporter::new(config).unwrap();
        assert!(exporter.meter.is_none());
    }

    #[tokio::test]
    async fn test_export_snapshot_disabled() {
        let config = TelemetryConfig {
            enabled: false,
            ..Default::default()
        };

        let exporter = MemoryTelemetryExporter::new(config).unwrap();

        let snapshot = MemorySnapshot {
            timestamp: chrono::Utc::now(),
            rss_bytes: 1024 * 1024,
            heap_bytes: 512 * 1024,
            virtual_bytes: 2048 * 1024,
            resident_bytes: 1024 * 1024,
            shared_bytes: 0,
            text_bytes: 0,
            data_bytes: 0,
            stack_bytes: 0,
        };

        // Should not error when disabled
        exporter
            .export_snapshot(&snapshot, Some("test"), Some("session-123"))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_export_allocations_disabled() {
        let config = TelemetryConfig {
            enabled: false,
            ..Default::default()
        };

        let exporter = MemoryTelemetryExporter::new(config).unwrap();

        let allocations = vec![AllocationInfo {
            timestamp: chrono::Utc::now(),
            size: 1024,
            alignment: 8,
            stack_trace: vec!["test_function".to_string()],
            component: "test_component".to_string(),
            operation: "test_operation".to_string(),
        }];

        // Should not error when disabled
        exporter
            .export_allocations(&allocations, Some("test"))
            .await
            .unwrap();
    }

    #[test]
    fn test_leak_severity_calculation() {
        let config = TelemetryConfig {
            enabled: false,
            ..Default::default()
        };
        let exporter = MemoryTelemetryExporter::new(config).unwrap();

        // Critical leak
        let critical_leak = LeakInfo {
            component: "test".to_string(),
            allocation_count: 1000,
            total_size_bytes: 150 * 1024 * 1024, // 150 MB
            average_size_bytes: 150.0,
            growth_rate: 0.0,
            first_seen: chrono::Utc::now(),
            last_seen: chrono::Utc::now(),
        };
        assert_eq!(exporter.calculate_leak_severity(&critical_leak), "critical");

        // Low severity leak
        let low_leak = LeakInfo {
            component: "test".to_string(),
            allocation_count: 10,
            total_size_bytes: 1024 * 1024, // 1 MB
            average_size_bytes: 100.0,
            growth_rate: 0.0,
            first_seen: chrono::Utc::now(),
            last_seen: chrono::Utc::now(),
        };
        assert_eq!(exporter.calculate_leak_severity(&low_leak), "low");
    }

    #[test]
    fn test_allocation_efficiency_calculation() {
        let config = TelemetryConfig {
            enabled: false,
            ..Default::default()
        };
        let exporter = MemoryTelemetryExporter::new(config).unwrap();

        // Efficient allocation (medium-sized, well-aligned)
        let efficient_allocations = vec![
            AllocationInfo {
                timestamp: chrono::Utc::now(),
                size: 8192,
                alignment: 8,
                stack_trace: vec!["test".to_string()],
                component: "test".to_string(),
                operation: "test".to_string(),
            },
            AllocationInfo {
                timestamp: chrono::Utc::now(),
                size: 16384,
                alignment: 8,
                stack_trace: vec!["test".to_string()],
                component: "test".to_string(),
                operation: "test".to_string(),
            },
        ];

        let efficiency = exporter.calculate_allocation_efficiency(&efficient_allocations);
        assert!(efficiency > 0.9);

        // Inefficient allocation (tiny sizes)
        let inefficient_allocations = vec![AllocationInfo {
            timestamp: chrono::Utc::now(),
            size: 100,
            alignment: 8,
            stack_trace: vec!["test".to_string()],
            component: "test".to_string(),
            operation: "test".to_string(),
        }];

        let efficiency = exporter.calculate_allocation_efficiency(&inefficient_allocations);
        assert!(efficiency < 0.7);
    }
}
