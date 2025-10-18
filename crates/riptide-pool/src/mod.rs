//! Instance pool management with WASM component lifecycle.
//!
//! This module provides a thread-safe pool of WASM component instances
//! with health monitoring, circuit breaker protection, and memory management.
//!
//! ## Organization
//!
//! - `pool`: Main pool implementation and extraction logic (moved from instance_pool.rs)
//! - `health`: Health monitoring and validation
//! - `memory`: Memory management and cleanup
//! - `models`: Core data structures for pooled instances and circuit breaker

pub mod health;
pub mod memory;
pub mod models;
pub mod pool;

// Re-export main public API
pub use models::{CircuitBreakerState, PooledInstance};
pub use pool::{create_event_aware_pool, get_instances_per_worker, AdvancedInstancePool};
