//! Riptide CLI library exports for testing
//!
//! This module re-exports the CLI components for use in integration tests.
//!
//! Phase 1: Minimal stub - most modules removed until dependencies are resolved

pub mod api_client;
pub mod config;
pub mod output;

// Stubbed modules - to be re-implemented in Phase 2
// pub mod api_wrapper;     // Uses tracing
// pub mod cache;           // Uses futures
// pub mod client;          // Uses tracing
// pub mod commands;        // Uses many removed dependencies
// pub mod execution_mode;  // May be needed later
// pub mod job;             // Uses riptide-workers, rand
// pub mod metrics;         // Uses once_cell, opentelemetry, riptide-monitoring
// pub mod session;         // May be needed later
// pub mod validation_adapter;
