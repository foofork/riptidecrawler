//! Adapters module for metrics collection
//!
//! This module contains adapters that implement the MetricsCollector port trait.

pub mod composite_metrics;

pub use composite_metrics::{
    BusinessMetricsPort, CompositeMetricsAdapter, PdfMetricsPort, PerformanceMetricsPort,
    TransportMetricsPort,
};
