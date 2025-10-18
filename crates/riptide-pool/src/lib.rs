//! # Riptide Pool
//!
//! WASM instance pool management with health monitoring, circuit breakers,
//! and memory management for the Riptide web scraping framework.
//!
//! ## Features
//!
//! - **Instance Pooling**: Thread-safe pool of WASM component instances
//! - **Health Monitoring**: Continuous health checks and validation
//! - **Circuit Breaker**: Fault tolerance and resilience patterns
//! - **Memory Management**: Advanced memory allocation and cleanup
//! - **Event Integration**: Pub/sub messaging for pool operations
//!
//! ## Usage
//!
//! ```no_run
//! use riptide_pool::{AdvancedInstancePool, create_event_aware_pool, ExtractorConfig};
//! use wasmtime::Engine;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let engine = Engine::default();
//! let config = ExtractorConfig::default();
//! let pool = AdvancedInstancePool::new(config, engine, "path/to/component.wasm").await?;
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod events_integration;
pub mod health;
pub mod health_monitor;
pub mod memory;
pub mod memory_manager;
pub mod models;
pub mod pool;

// Re-export main public API
pub use config::{ExtractorConfig, PerformanceMetrics, WasmResourceTracker};
pub use events_integration::{EventAwareInstancePool, PoolEventEmitter};
pub use health_monitor::{
    HealthLevel, HealthTrend, MemoryHealthStats, MemoryPressureLevel, PoolHealthMonitor,
    PoolHealthStatus,
};
pub use memory_manager::{
    MemoryEvent, MemoryManager, MemoryManagerConfig, MemoryStats, TrackedWasmInstance,
};
// Re-export PoolMetrics from riptide-events
pub use models::{CircuitBreakerState, PooledInstance};
pub use pool::{create_event_aware_pool, get_instances_per_worker, AdvancedInstancePool};
pub use riptide_events::types::PoolMetrics;
