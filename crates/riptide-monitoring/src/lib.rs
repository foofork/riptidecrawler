//! # Riptide Monitoring
//!
//! Comprehensive monitoring, telemetry, and metrics collection for the Riptide framework.
//!
//! ## Components
//!
//! - **Telemetry**: OpenTelemetry integration and distributed tracing
//! - **Metrics Collector**: System and application metrics collection
//! - **Alerts**: Alert management and notification system
//! - **Health Checks**: Service health monitoring
//! - **Reports**: Performance and usage reporting
//! - **Time Series**: Time-series data collection and analysis

pub mod telemetry;

#[cfg(feature = "collector")]
pub mod monitoring {
    //! Monitoring submodules
    pub mod alerts;
    pub mod collector;
    pub mod error;
    pub mod health;
    pub mod metrics;
    pub mod reports;
    pub mod time_series;
}

// Re-export commonly used types
pub use telemetry::*;

#[cfg(feature = "collector")]
pub use monitoring::{
    alerts::*, collector::*, error::*, health::*, metrics::*, reports::*, time_series::*,
};
