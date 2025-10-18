//! Global configuration for RipTide API with comprehensive resource controls.
//!
//! This module provides centralized configuration for all API operations including
//! resource limits, timeouts, rate limiting, and performance controls.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Global API configuration with resource management
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiConfig {
    /// Resource management configuration
    pub resources: ResourceConfig,
    /// Performance and timeout configuration
    pub performance: PerformanceConfig,
    /// Rate limiting configuration
    pub rate_limiting: RateLimitingConfig,
    /// Memory management configuration
    pub memory: MemoryConfig,
    /// Headless browser configuration
    pub headless: HeadlessConfig,
    /// PDF processing configuration
    pub pdf: PdfConfig,
    /// WASM runtime configuration
    pub wasm: WasmConfig,
    /// Search provider configuration
    pub search: SearchProviderConfig,
}

/// Resource management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Maximum concurrent render operations
    pub max_concurrent_renders: usize,
    /// Maximum concurrent PDF operations
    pub max_concurrent_pdf: usize,
    /// Maximum concurrent WASM instances
    pub max_concurrent_wasm: usize,
    /// Global request timeout in seconds
    pub global_timeout_secs: u64,
    /// Resource cleanup interval in seconds
    pub cleanup_interval_secs: u64,
    /// Enable resource monitoring
    pub enable_monitoring: bool,
    /// Resource pool health check interval
    pub health_check_interval_secs: u64,
}

/// Performance and timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Hard timeout for render operations (3s requirement)
    pub render_timeout_secs: u64,
    /// Timeout for PDF processing operations
    pub pdf_timeout_secs: u64,
    /// Timeout for WASM extraction operations
    pub wasm_timeout_secs: u64,
    /// Timeout for HTTP requests
    pub http_timeout_secs: u64,
    /// Memory cleanup threshold (MB)
    pub memory_cleanup_threshold_mb: usize,
    /// Enable automatic cleanup on timeouts
    pub auto_cleanup_on_timeout: bool,
    /// Performance degradation detection threshold
    pub degradation_threshold: f64,
}

/// Rate limiting configuration with per-host controls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Requests per second per host (1.5 RPS requirement)
    pub requests_per_second_per_host: f64,
    /// Jitter factor for rate limiting (0.0-1.0)
    pub jitter_factor: f64,
    /// Burst capacity per host
    pub burst_capacity_per_host: u32,
    /// Rate limit window duration in seconds
    pub window_duration_secs: u64,
    /// Cleanup interval for rate limit storage
    pub cleanup_interval_secs: u64,
    /// Maximum number of tracked hosts
    pub max_tracked_hosts: usize,
}

/// Memory management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Maximum memory per request (MB)
    pub max_memory_per_request_mb: usize,
    /// Global memory limit (MB)
    pub global_memory_limit_mb: usize,
    /// Memory soft limit - trigger warnings (MB) (QW-3)
    pub memory_soft_limit_mb: usize,
    /// Memory hard limit - reject requests (MB) (QW-3)
    pub memory_hard_limit_mb: usize,
    /// Memory pressure detection threshold (0.0-1.0)
    pub pressure_threshold: f64,
    /// Enable automatic garbage collection
    pub auto_gc: bool,
    /// GC trigger threshold (MB)
    pub gc_trigger_threshold_mb: usize,
    /// Memory monitoring interval in seconds
    pub monitoring_interval_secs: u64,
    /// Enable memory leak detection
    pub enable_leak_detection: bool,
    /// Enable proactive memory monitoring (QW-3)
    pub enable_proactive_monitoring: bool,
}

/// Headless browser configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadlessConfig {
    /// Maximum browser pool size (QW-1: increased to 20 for better scaling)
    pub max_pool_size: usize,
    /// Minimum browser pool size
    pub min_pool_size: usize,
    /// Browser idle timeout in seconds
    pub idle_timeout_secs: u64,
    /// Browser health check interval
    pub health_check_interval_secs: u64,
    /// Maximum pages per browser instance
    pub max_pages_per_browser: usize,
    /// Browser restart threshold (failed operations)
    pub restart_threshold: u32,
    /// Enable browser recycling
    pub enable_recycling: bool,
    /// Browser launch timeout
    pub launch_timeout_secs: u64,
    /// Maximum retries for browser operations
    pub max_retries: u32,
}

/// PDF processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfConfig {
    /// Maximum concurrent PDF operations (2 semaphore requirement)
    pub max_concurrent: usize,
    /// PDF processing timeout in seconds
    pub processing_timeout_secs: u64,
    /// Maximum PDF file size (MB)
    pub max_file_size_mb: usize,
    /// Enable PDF streaming processing
    pub enable_streaming: bool,
    /// PDF queue size
    pub queue_size: usize,
    /// Priority queue timeout
    pub queue_timeout_secs: u64,
}

/// WASM runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmConfig {
    /// Single instance per worker (requirement)
    pub instances_per_worker: usize,
    /// WASM module timeout in seconds
    pub module_timeout_secs: u64,
    /// Maximum WASM memory (MB)
    pub max_memory_mb: usize,
    /// Enable WASM instance recycling
    pub enable_recycling: bool,
    /// Instance health check interval
    pub health_check_interval_secs: u64,
    /// Maximum operations per instance
    pub max_operations_per_instance: u64,
    /// Instance restart threshold
    pub restart_threshold: u32,
}

/// Search provider configuration for deep search functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchProviderConfig {
    /// Search backend to use: "serper", "none", or "searxng"
    pub backend: String,
    /// Default timeout for search operations in seconds
    pub timeout_secs: u64,
    /// Enable URL parsing for None provider
    pub enable_url_parsing: bool,
    /// Circuit breaker failure threshold percentage (0-100)
    pub circuit_breaker_failure_threshold: u32,
    /// Circuit breaker minimum requests before opening
    pub circuit_breaker_min_requests: u32,
    /// Circuit breaker recovery timeout in seconds
    pub circuit_breaker_recovery_timeout_secs: u64,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_renders: 10,
            max_concurrent_pdf: 2, // Requirement: PDF semaphore = 2
            max_concurrent_wasm: 4,
            global_timeout_secs: 30,
            cleanup_interval_secs: 60,
            enable_monitoring: true,
            health_check_interval_secs: 30,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            render_timeout_secs: 3, // Requirement: 3s hard cap
            pdf_timeout_secs: 10,
            wasm_timeout_secs: 5,
            http_timeout_secs: 10,
            memory_cleanup_threshold_mb: 512,
            auto_cleanup_on_timeout: true,
            degradation_threshold: 0.8, // 80% degradation threshold
        }
    }
}

impl Default for RateLimitingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_second_per_host: 1.5, // Requirement: 1.5 RPS
            jitter_factor: 0.1,                // 10% jitter
            burst_capacity_per_host: 3,
            window_duration_secs: 60,
            cleanup_interval_secs: 300,
            max_tracked_hosts: 10000,
        }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_memory_per_request_mb: 256,
            global_memory_limit_mb: 2048,
            memory_soft_limit_mb: 400, // QW-3: Trigger warnings at 400MB
            memory_hard_limit_mb: 500, // QW-3: Reject requests at 500MB
            pressure_threshold: 0.85,  // 85% memory usage
            auto_gc: true,
            gc_trigger_threshold_mb: 1024,
            monitoring_interval_secs: 30,
            enable_leak_detection: true,
            enable_proactive_monitoring: true, // QW-3: Enable proactive monitoring
        }
    }
}

impl Default for HeadlessConfig {
    fn default() -> Self {
        Self {
            max_pool_size: 20, // QW-1: Increased from 3 to 20 for better scaling and performance
            min_pool_size: 1,
            idle_timeout_secs: 300, // 5 minutes
            health_check_interval_secs: 60,
            max_pages_per_browser: 10,
            restart_threshold: 5,
            enable_recycling: true,
            launch_timeout_secs: 30,
            max_retries: 3,
        }
    }
}

impl Default for PdfConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 2, // Requirement: semaphore = 2
            processing_timeout_secs: 30,
            max_file_size_mb: 100,
            enable_streaming: true,
            queue_size: 50,
            queue_timeout_secs: 60,
        }
    }
}

impl Default for WasmConfig {
    fn default() -> Self {
        Self {
            instances_per_worker: 1, // Requirement: single instance per worker
            module_timeout_secs: 10,
            max_memory_mb: 128,
            enable_recycling: false, // Single instance, no recycling needed
            health_check_interval_secs: 120,
            max_operations_per_instance: 10000,
            restart_threshold: 10,
        }
    }
}

impl Default for SearchProviderConfig {
    fn default() -> Self {
        Self {
            backend: "serper".to_string(), // Default to Serper for compatibility
            timeout_secs: 30,
            enable_url_parsing: true,
            circuit_breaker_failure_threshold: 50, // 50% failure rate
            circuit_breaker_min_requests: 5,
            circuit_breaker_recovery_timeout_secs: 60,
        }
    }
}

impl ApiConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Resource configuration
        if let Ok(val) = std::env::var("RIPTIDE_MAX_CONCURRENT_RENDERS") {
            if let Ok(val) = val.parse() {
                config.resources.max_concurrent_renders = val;
            }
        }

        if let Ok(val) = std::env::var("RIPTIDE_MAX_CONCURRENT_PDF") {
            if let Ok(val) = val.parse() {
                config.resources.max_concurrent_pdf = val;
            }
        }

        // Performance configuration
        if let Ok(val) = std::env::var("RIPTIDE_RENDER_TIMEOUT") {
            if let Ok(val) = val.parse() {
                config.performance.render_timeout_secs = val;
            }
        }

        // Rate limiting configuration
        if let Ok(val) = std::env::var("RIPTIDE_RATE_LIMIT_RPS") {
            if let Ok(val) = val.parse() {
                config.rate_limiting.requests_per_second_per_host = val;
            }
        }

        if let Ok(val) = std::env::var("RIPTIDE_RATE_LIMIT_JITTER") {
            if let Ok(val) = val.parse() {
                config.rate_limiting.jitter_factor = val;
            }
        }

        // Headless configuration
        if let Ok(val) = std::env::var("RIPTIDE_HEADLESS_POOL_SIZE") {
            if let Ok(val) = val.parse() {
                config.headless.max_pool_size = val;
            }
        }

        // Memory configuration
        if let Ok(val) = std::env::var("RIPTIDE_MEMORY_LIMIT_MB") {
            if let Ok(val) = val.parse() {
                config.memory.global_memory_limit_mb = val;
            }
        }

        // Search provider configuration
        if let Ok(val) = std::env::var("SEARCH_BACKEND") {
            config.search.backend = val;
        }

        if let Ok(val) = std::env::var("SEARCH_TIMEOUT") {
            if let Ok(val) = val.parse() {
                config.search.timeout_secs = val;
            }
        }

        if let Ok(val) = std::env::var("SEARCH_ENABLE_URL_PARSING") {
            config.search.enable_url_parsing = val.to_lowercase() == "true";
        }

        config
    }

    /// Validate configuration settings
    pub fn validate(&self) -> Result<(), String> {
        // Validate resource limits
        if self.resources.max_concurrent_renders == 0 {
            return Err("max_concurrent_renders must be greater than 0".to_string());
        }

        if self.resources.max_concurrent_pdf == 0 {
            return Err("max_concurrent_pdf must be greater than 0".to_string());
        }

        // Validate performance settings
        if self.performance.render_timeout_secs == 0 {
            return Err("render_timeout_secs must be greater than 0".to_string());
        }

        // Validate rate limiting
        if self.rate_limiting.enabled {
            if self.rate_limiting.requests_per_second_per_host <= 0.0 {
                return Err("requests_per_second_per_host must be greater than 0".to_string());
            }

            if self.rate_limiting.jitter_factor < 0.0 || self.rate_limiting.jitter_factor > 1.0 {
                return Err("jitter_factor must be between 0.0 and 1.0".to_string());
            }
        }

        // Validate memory settings
        if self.memory.pressure_threshold <= 0.0 || self.memory.pressure_threshold > 1.0 {
            return Err("memory pressure_threshold must be between 0.0 and 1.0".to_string());
        }

        // Validate headless settings
        if self.headless.max_pool_size == 0 {
            return Err("headless max_pool_size must be greater than 0".to_string());
        }

        if self.headless.min_pool_size > self.headless.max_pool_size {
            return Err("headless min_pool_size cannot be greater than max_pool_size".to_string());
        }

        // Validate WASM settings
        if self.wasm.instances_per_worker == 0 {
            return Err("wasm instances_per_worker must be greater than 0".to_string());
        }

        // Validate search provider settings
        if self.search.backend.is_empty() {
            return Err("search backend cannot be empty".to_string());
        }

        match self.search.backend.as_str() {
            "serper" | "none" | "searxng" => {} // Valid backends
            _ => {
                return Err(format!(
                    "Invalid search backend '{}'. Valid options: serper, none, searxng",
                    self.search.backend
                ));
            }
        }

        if self.search.timeout_secs == 0 {
            return Err("search timeout_secs must be greater than 0".to_string());
        }

        if self.search.circuit_breaker_failure_threshold > 100 {
            return Err("search circuit_breaker_failure_threshold must be <= 100".to_string());
        }

        Ok(())
    }

    /// Get optimal timeout based on operation type
    pub fn get_timeout(&self, operation: &str) -> Duration {
        match operation {
            "render" => Duration::from_secs(self.performance.render_timeout_secs),
            "pdf" => Duration::from_secs(self.performance.pdf_timeout_secs),
            "wasm" => Duration::from_secs(self.performance.wasm_timeout_secs),
            "http" => Duration::from_secs(self.performance.http_timeout_secs),
            "search" => Duration::from_secs(self.search.timeout_secs),
            _ => Duration::from_secs(self.resources.global_timeout_secs),
        }
    }

    /// Check if system is under memory pressure
    pub fn is_memory_pressure(&self, current_usage_mb: usize) -> bool {
        let usage_ratio = current_usage_mb as f64 / self.memory.global_memory_limit_mb as f64;
        usage_ratio >= self.memory.pressure_threshold
    }

    /// Calculate jittered delay for rate limiting
    pub fn calculate_jittered_delay(&self) -> Duration {
        let base_delay = 1.0 / self.rate_limiting.requests_per_second_per_host;
        let jitter = self.rate_limiting.jitter_factor * base_delay * (rand::random::<f64>() - 0.5);
        let final_delay = base_delay + jitter;
        Duration::from_secs_f64(final_delay.max(0.001)) // Minimum 1ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ApiConfig::default();

        // Test requirements
        assert_eq!(config.headless.max_pool_size, 3); // Pool cap = 3
        assert_eq!(config.performance.render_timeout_secs, 3); // 3s timeout
        assert_eq!(config.rate_limiting.requests_per_second_per_host, 1.5); // 1.5 RPS
        assert_eq!(config.pdf.max_concurrent, 2); // PDF semaphore = 2
        assert_eq!(config.wasm.instances_per_worker, 1); // Single instance per worker
    }

    #[test]
    fn test_config_validation() {
        let mut config = ApiConfig::default();
        assert!(config.validate().is_ok());

        // Test invalid resource settings
        config.resources.max_concurrent_renders = 0;
        assert!(config.validate().is_err());

        config = ApiConfig::default();
        config.rate_limiting.jitter_factor = 1.5; // > 1.0
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_timeout_selection() {
        let config = ApiConfig::default();

        assert_eq!(config.get_timeout("render"), Duration::from_secs(3));

        assert_eq!(config.get_timeout("pdf"), Duration::from_secs(10));
    }

    #[test]
    fn test_memory_pressure_detection() {
        let config = ApiConfig::default();

        // Below threshold
        assert!(!config.is_memory_pressure(1000)); // 1GB < 85% of 2GB

        // Above threshold
        assert!(config.is_memory_pressure(1800)); // 1.8GB > 85% of 2GB
    }

    #[test]
    fn test_jittered_delay() {
        let config = ApiConfig::default();

        let delay1 = config.calculate_jittered_delay();
        let delay2 = config.calculate_jittered_delay();

        // Delays should be different due to jitter
        assert_ne!(delay1, delay2);

        // Both should be positive and reasonable
        assert!(delay1.as_secs_f64() > 0.0);
        assert!(delay2.as_secs_f64() > 0.0);
        assert!(delay1.as_secs_f64() < 2.0); // Should be around 1/1.5 = 0.67s ± jitter
    }
}
