//! Telemetry Adapter for hexagonal architecture
//!
//! Adapts the concrete TelemetrySystem to monitoring port traits.

use std::sync::Arc;

/// Telemetry backend trait for dependency inversion
pub trait TelemetryBackend: Send + Sync {
    /// Export telemetry data
    fn export(&self) -> impl std::future::Future<Output = ()> + Send;

    /// Check if telemetry is enabled
    fn is_enabled(&self) -> bool;

    /// Flush pending telemetry data
    fn flush(&self) -> impl std::future::Future<Output = ()> + Send;
}

/// Adapter that implements TelemetryBackend for TelemetrySystem
pub struct TelemetryAdapter {
    inner: Arc<riptide_monitoring::TelemetrySystem>,
}

impl TelemetryAdapter {
    /// Create a new TelemetryAdapter wrapping the concrete implementation
    pub fn new(system: Arc<riptide_monitoring::TelemetrySystem>) -> Self {
        Self { inner: system }
    }
}

impl TelemetryBackend for TelemetryAdapter {
    async fn export(&self) {
        // TelemetrySystem handles export internally
    }

    fn is_enabled(&self) -> bool {
        // TelemetrySystem is always enabled when constructed
        true
    }

    async fn flush(&self) {
        // TelemetrySystem handles flush internally
    }
}
