//! Pool configuration and resource tracking types

use serde::{Deserialize, Serialize};
use wasmtime::ResourceLimiter;

/// Configuration for the pool and extractor component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractorConfig {
    pub max_instances: usize,
    pub enable_metrics: bool,
    pub timeout_ms: u64,
    pub memory_limit_pages: Option<u32>,
    pub extraction_timeout: Option<u64>,
    pub max_pool_size: usize,
    pub initial_pool_size: usize,
    pub epoch_timeout_ms: u64,
    pub health_check_interval: u64,
    pub memory_limit: Option<usize>,
    pub circuit_breaker_timeout: u64,
    pub circuit_breaker_failure_threshold: u32,
    /// Enable WIT (WebAssembly Interface Types) validation before component instantiation
    pub enable_wit_validation: bool,
}

impl Default for ExtractorConfig {
    fn default() -> Self {
        Self {
            max_instances: 4,
            enable_metrics: true,
            timeout_ms: 5000,
            memory_limit_pages: Some(256),
            extraction_timeout: Some(30000),
            max_pool_size: 8,
            initial_pool_size: 2,
            epoch_timeout_ms: 60000,
            health_check_interval: 30000,
            memory_limit: Some(512 * 1024 * 1024), // 512MB
            circuit_breaker_timeout: 5000,
            circuit_breaker_failure_threshold: 5,
            enable_wit_validation: true, // Enable WIT validation by default
        }
    }
}

/// Performance metrics for extraction and pool operations
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_latency_ms: f64,
    pub circuit_breaker_trips: u64,
    pub total_extractions: u64,
    pub successful_extractions: u64,
    pub semaphore_wait_time_ms: f64,
    pub avg_processing_time_ms: f64,
    pub pool_size: usize,
    pub fallback_extractions: u64,
    pub wasm_memory_pages: u64,
    pub wasm_peak_memory_pages: u64,
    pub wasm_grow_failed_total: u64,
    pub failed_extractions: u64,
    pub epoch_timeouts: u64,
    /// P2-2: WIT validation metrics
    pub wit_validations_total: u64,
    pub wit_validations_passed: u64,
    pub wit_validations_failed: u64,
    pub wit_validation_warnings: u64,
}

/// Resource tracking for WASM instances
#[derive(Debug, Clone)]
pub struct WasmResourceTracker {
    pub memory_usage: usize,
    pub cpu_usage: f32,
    pub instance_count: usize,
}

impl WasmResourceTracker {
    pub fn grow_failures(&self) -> u64 {
        // Placeholder for tracking growth failures
        0
    }

    pub fn current_memory_pages(&self) -> u32 {
        // Placeholder for memory pages tracking
        256
    }
}

impl Default for WasmResourceTracker {
    fn default() -> Self {
        Self {
            memory_usage: 0,
            cpu_usage: 0.0,
            instance_count: 0,
        }
    }
}

impl ResourceLimiter for WasmResourceTracker {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        // Simple memory limit check
        const MAX_MEMORY: usize = 512 * 1024 * 1024; // 512MB
        Ok(desired <= MAX_MEMORY)
    }

    fn table_growing(
        &mut self,
        _current: usize,
        _desired: usize,
        _maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        Ok(true) // Allow table growth
    }
}
