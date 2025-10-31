//! Core data structures for instance pool management.

use std::time::Instant;

#[cfg(feature = "wasm-pool")]
use wasmtime::{component::*, Engine, Store};

/// Enhanced pooled instance with comprehensive tracking
#[cfg(feature = "wasm-pool")]
pub struct PooledInstance {
    pub id: String,
    pub engine: Arc<Engine>,
    pub component: Arc<Component>,
    pub linker: Arc<Linker<WasmResourceTracker>>,
    pub created_at: Instant,
    pub last_used: Instant,
    pub use_count: u64,
    pub failure_count: u64,
    pub memory_usage_bytes: u64,
    pub resource_tracker: WasmResourceTracker,
}

#[cfg(feature = "wasm-pool")]
impl PooledInstance {
    pub fn new(
        engine: Arc<Engine>,
        component: Arc<Component>,
        linker: Arc<Linker<WasmResourceTracker>>,
        _max_memory_pages: usize,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let now = Instant::now();

        Self {
            id,
            engine,
            component,
            linker,
            created_at: now,
            last_used: now,
            use_count: 0,
            failure_count: 0,
            memory_usage_bytes: 0,
            resource_tracker: WasmResourceTracker::default(),
        }
    }

    /// Check if instance is healthy and reusable
    pub fn is_healthy(&self, config: &ExtractorConfig) -> bool {
        self.use_count < 1000
            && self.failure_count < 5
            && self.memory_usage_bytes < config.memory_limit.unwrap_or(usize::MAX) as u64
            && self.resource_tracker.grow_failures() < 10
    }

    /// Update usage statistics
    pub fn record_usage(&mut self, success: bool) {
        self.last_used = Instant::now();
        self.use_count += 1;
        if !success {
            self.failure_count += 1;
        }
        self.memory_usage_bytes = (self.resource_tracker.current_memory_pages() * 64 * 1024) as u64;
    }

    /// Create fresh Store with resource limits
    pub fn create_fresh_store(&mut self) -> Store<WasmResourceTracker> {
        let mut store = Store::new(&self.engine, self.resource_tracker.clone());

        // Set resource limits
        store.limiter(|tracker| tracker);

        // Enable epoch interruption for timeouts
        store.epoch_deadline_trap();

        store
    }
}

#[cfg(feature = "wasm-pool")]
impl std::fmt::Debug for PooledInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PooledInstance")
            .field("id", &self.id)
            .field("created_at", &self.created_at)
            .field("last_used", &self.last_used)
            .field("use_count", &self.use_count)
            .field("failure_count", &self.failure_count)
            .field("memory_usage_bytes", &self.memory_usage_bytes)
            .field("resource_tracker", &self.resource_tracker)
            .finish()
    }
}

/// Circuit breaker states for WASM error handling
#[derive(Clone, Debug)]
pub enum CircuitBreakerState {
    Closed {
        failure_count: u64,
        success_count: u64,
        last_failure: Option<Instant>,
    },
    Open {
        opened_at: Instant,
        failure_count: u64,
    },
    HalfOpen {
        test_requests: u64,
        start_time: Instant,
    },
}
