//! Workflow orchestration and resource management.
//!
//! This module provides:
//! - Transactional workflows with ACID guarantees
//! - Backpressure and concurrency control
//! - Cancellation token management
//! - Resource cleanup and RAII patterns

pub mod backpressure;
pub mod transactional;

pub use backpressure::{BackpressureGuard, BackpressureManager};
pub use transactional::TransactionalWorkflow;
