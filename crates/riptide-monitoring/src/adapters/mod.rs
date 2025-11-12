//! Adapters implementing port traits for monitoring infrastructure
//!
//! This module contains adapter implementations that bridge concrete
//! monitoring and telemetry systems with abstract port traits, enabling
//! dependency inversion in the hexagonal architecture.

pub mod monitoring_system_adapter;
pub mod telemetry_system_adapter;

pub use monitoring_system_adapter::MonitoringSystemAdapter;
pub use telemetry_system_adapter::TelemetrySystemAdapter;
