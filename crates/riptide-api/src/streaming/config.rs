// TODO(P2): Streaming infrastructure - will be activated when routes are added
// STATUS: Complete infrastructure prepared, waiting for route activation
// PLAN: Activate when streaming endpoints are implemented
// IMPLEMENTATION:
//   1. Create streaming route handlers in handlers/streaming/
//   2. Wire up NDJSON, SSE, and WebSocket endpoints
//   3. Connect to this config module for stream settings
//   4. Add streaming handlers to main router
//   5. Enable streaming feature flags in config
// DEPENDENCIES: None - infrastructure is ready
// EFFORT: Medium (6-8 hours for all streaming routes)
// PRIORITY: Future feature - not blocking production
// BLOCKER: None
#![allow(dead_code)]

//! Configuration for streaming operations.
//!
//! This module provides centralized configuration for all streaming
//! protocols including NDJSON, SSE, and WebSocket endpoints.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Global streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StreamConfig {
    /// Buffer configuration
    pub buffer: BufferConfig,
    /// WebSocket specific configuration
    pub websocket: WebSocketConfig,
    /// Server-Sent Events configuration
    pub sse: SseConfig,
    /// NDJSON streaming configuration
    pub ndjson: NdjsonConfig,
    /// General streaming settings
    pub general: GeneralConfig,
}

/// Buffer configuration for streaming operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferConfig {
    /// Default buffer size for new connections
    pub default_size: usize,
    /// Maximum buffer size before applying backpressure
    pub max_size: usize,
    /// Minimum buffer size for slow connections
    pub min_size: usize,
    /// Enable dynamic buffer resizing
    pub dynamic_sizing: bool,
    /// Growth factor for buffer expansion (1.0 = no growth)
    pub growth_factor: f64,
    /// Shrink factor for buffer reduction (1.0 = no shrinking)
    pub shrink_factor: f64,
    /// Memory limit per connection (bytes)
    pub memory_limit_per_connection: usize,
    /// Global memory limit for all streaming (bytes)
    pub global_memory_limit: usize,
}

/// WebSocket-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    /// Maximum message size (bytes)
    pub max_message_size: usize,
    /// Ping interval for keep-alive
    pub ping_interval: Duration,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Maximum concurrent connections per IP
    pub max_connections_per_ip: usize,
    /// Enable compression
    pub enable_compression: bool,
    /// Backpressure threshold (queue size)
    pub backpressure_threshold: usize,
    /// Maximum time to wait for client response
    pub client_response_timeout: Duration,
}

/// Server-Sent Events configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseConfig {
    /// Keep-alive interval
    pub keep_alive_interval: Duration,
    /// Maximum connection time
    pub max_connection_time: Duration,
    /// Retry interval for client reconnection
    pub retry_interval: Duration,
    /// Maximum number of events to buffer
    pub max_buffered_events: usize,
    /// Enable CORS for SSE endpoints
    pub enable_cors: bool,
    /// Custom headers to include
    pub custom_headers: Vec<(String, String)>,
}

/// NDJSON streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NdjsonConfig {
    /// Maximum line length (bytes)
    pub max_line_length: usize,
    /// Flush interval for buffered writes
    pub flush_interval: Duration,
    /// Enable streaming compression
    pub enable_compression: bool,
    /// Compression level (1-9, if compression enabled)
    pub compression_level: u32,
    /// Maximum response size before switching to chunked
    pub max_response_size: usize,
}

/// General streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Maximum concurrent streaming connections
    pub max_concurrent_streams: usize,
    /// Default timeout for streaming operations
    pub default_timeout: Duration,
    /// Enable detailed metrics collection
    pub enable_metrics: bool,
    /// Metrics collection interval
    pub metrics_interval: Duration,
    /// Enable request tracing
    pub enable_tracing: bool,
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
}

/// Rate limiting configuration for streaming endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Requests per second per IP
    pub requests_per_second: u32,
    /// Burst capacity
    pub burst_capacity: u32,
    /// Rate limit window duration
    pub window_duration: Duration,
    /// Action to take when rate limit exceeded
    pub action: RateLimitAction,
}

/// Actions to take when rate limit is exceeded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitAction {
    /// Drop the request
    Drop,
    /// Delay the request
    Delay,
    /// Return rate limit error
    Error,
}

/// Health check configuration for streaming endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Enable health checks
    pub enabled: bool,
    /// Health check interval
    pub interval: Duration,
    /// Endpoint for health checks
    pub endpoint: String,
    /// Timeout for health check requests
    pub timeout: Duration,
    /// Unhealthy threshold (consecutive failures)
    pub unhealthy_threshold: u32,
    /// Healthy threshold (consecutive successes)
    pub healthy_threshold: u32,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            default_size: 256,
            max_size: 2048,
            min_size: 64,
            dynamic_sizing: true,
            growth_factor: 1.5,
            shrink_factor: 0.75,
            memory_limit_per_connection: 64 * 1024 * 1024, // 64MB
            global_memory_limit: 1024 * 1024 * 1024,       // 1GB
        }
    }
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            max_message_size: 16 * 1024 * 1024, // 16MB
            ping_interval: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(300), // 5 minutes
            max_connections_per_ip: 10,
            enable_compression: true,
            backpressure_threshold: 1000,
            client_response_timeout: Duration::from_secs(60),
        }
    }
}

impl Default for SseConfig {
    fn default() -> Self {
        Self {
            keep_alive_interval: Duration::from_secs(30),
            max_connection_time: Duration::from_secs(3600), // 1 hour
            retry_interval: Duration::from_secs(5),
            max_buffered_events: 100,
            enable_cors: true,
            custom_headers: vec![
                ("Cache-Control".to_string(), "no-cache".to_string()),
                ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
            ],
        }
    }
}

impl Default for NdjsonConfig {
    fn default() -> Self {
        Self {
            max_line_length: 1024 * 1024, // 1MB per line
            flush_interval: Duration::from_millis(100),
            enable_compression: false, // Disable by default for better streaming
            compression_level: 6,
            max_response_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            max_concurrent_streams: 1000,
            default_timeout: Duration::from_secs(300), // 5 minutes
            enable_metrics: true,
            metrics_interval: Duration::from_secs(60),
            enable_tracing: true,
            rate_limit: RateLimitConfig::default(),
            health_check: HealthCheckConfig::default(),
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_second: 10,
            burst_capacity: 20,
            window_duration: Duration::from_secs(60),
            action: RateLimitAction::Error,
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(30),
            endpoint: "/health/streaming".to_string(),
            timeout: Duration::from_secs(5),
            unhealthy_threshold: 3,
            healthy_threshold: 2,
        }
    }
}

/// Environment-based configuration loader
impl StreamConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Buffer configuration
        if let Ok(size) = std::env::var("STREAM_BUFFER_SIZE") {
            if let Ok(size) = size.parse() {
                config.buffer.default_size = size;
            }
        }

        if let Ok(max_size) = std::env::var("STREAM_BUFFER_MAX_SIZE") {
            if let Ok(max_size) = max_size.parse() {
                config.buffer.max_size = max_size;
            }
        }

        // WebSocket configuration
        if let Ok(max_msg) = std::env::var("WS_MAX_MESSAGE_SIZE") {
            if let Ok(max_msg) = max_msg.parse() {
                config.websocket.max_message_size = max_msg;
            }
        }

        if let Ok(ping_interval) = std::env::var("WS_PING_INTERVAL") {
            if let Ok(ping_interval) = ping_interval.parse::<u64>() {
                config.websocket.ping_interval = Duration::from_secs(ping_interval);
            }
        }

        // General configuration
        if let Ok(max_streams) = std::env::var("STREAM_MAX_CONCURRENT") {
            if let Ok(max_streams) = max_streams.parse() {
                config.general.max_concurrent_streams = max_streams;
            }
        }

        if let Ok(timeout) = std::env::var("STREAM_DEFAULT_TIMEOUT") {
            if let Ok(timeout) = timeout.parse::<u64>() {
                config.general.default_timeout = Duration::from_secs(timeout);
            }
        }

        // Rate limiting
        if let Ok(enabled) = std::env::var("STREAM_RATE_LIMIT_ENABLED") {
            config.general.rate_limit.enabled = enabled.to_lowercase() == "true";
        }

        if let Ok(rps) = std::env::var("STREAM_RATE_LIMIT_RPS") {
            if let Ok(rps) = rps.parse() {
                config.general.rate_limit.requests_per_second = rps;
            }
        }

        config
    }

    /// Validate configuration settings
    pub fn validate(&self) -> Result<(), String> {
        // Validate buffer configuration
        if self.buffer.min_size > self.buffer.default_size {
            return Err("Buffer min_size cannot be greater than default_size".to_string());
        }

        if self.buffer.default_size > self.buffer.max_size {
            return Err("Buffer default_size cannot be greater than max_size".to_string());
        }

        if self.buffer.growth_factor <= 1.0 {
            return Err("Buffer growth_factor must be greater than 1.0".to_string());
        }

        if self.buffer.shrink_factor <= 0.0 || self.buffer.shrink_factor >= 1.0 {
            return Err("Buffer shrink_factor must be between 0.0 and 1.0".to_string());
        }

        // Validate WebSocket configuration
        if self.websocket.max_message_size == 0 {
            return Err("WebSocket max_message_size must be greater than 0".to_string());
        }

        if self.websocket.ping_interval.as_secs() == 0 {
            return Err("WebSocket ping_interval must be greater than 0".to_string());
        }

        // Validate general configuration
        if self.general.max_concurrent_streams == 0 {
            return Err("max_concurrent_streams must be greater than 0".to_string());
        }

        // Validate rate limiting
        if self.general.rate_limit.enabled && self.general.rate_limit.requests_per_second == 0 {
            return Err(
                "rate_limit requests_per_second must be greater than 0 when enabled".to_string(),
            );
        }

        Ok(())
    }

    /// Get optimal buffer size based on connection characteristics
    pub fn optimal_buffer_size(&self, is_slow_connection: bool, message_rate: f64) -> usize {
        if is_slow_connection {
            self.buffer.min_size
        } else if message_rate > 100.0 {
            // High message rate, use larger buffer
            (self.buffer.default_size as f64 * 1.5) as usize
        } else {
            self.buffer.default_size
        }
    }

    /// Check if streaming is healthy based on current metrics
    pub fn is_streaming_healthy(&self, current_connections: usize, error_rate: f64) -> bool {
        let connection_ratio =
            current_connections as f64 / self.general.max_concurrent_streams as f64;
        connection_ratio < 0.9 && error_rate < 0.05 // Less than 90% capacity and 5% error rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = StreamConfig::default();
        assert_eq!(config.buffer.default_size, 256);
        assert_eq!(config.websocket.max_connections_per_ip, 10);
        assert!(config.general.enable_metrics);
    }

    #[test]
    fn test_config_validation() {
        let mut config = StreamConfig::default();
        assert!(config.validate().is_ok());

        // Test invalid buffer configuration
        config.buffer.min_size = 1000;
        config.buffer.default_size = 500;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_optimal_buffer_size() {
        let config = StreamConfig::default();

        // Slow connection should get minimum buffer
        assert_eq!(
            config.optimal_buffer_size(true, 10.0),
            config.buffer.min_size
        );

        // High message rate should get larger buffer
        assert!(config.optimal_buffer_size(false, 150.0) > config.buffer.default_size);

        // Normal rate should get default buffer
        assert_eq!(
            config.optimal_buffer_size(false, 50.0),
            config.buffer.default_size
        );
    }

    #[test]
    fn test_streaming_health() {
        let config = StreamConfig::default();

        // Healthy state
        assert!(config.is_streaming_healthy(100, 0.01));

        // Unhealthy due to high connection count
        assert!(!config.is_streaming_healthy(950, 0.01));

        // Unhealthy due to high error rate
        assert!(!config.is_streaming_healthy(100, 0.10));
    }

    #[test]
    fn test_rate_limit_action() {
        let action = RateLimitAction::Drop;
        assert!(matches!(action, RateLimitAction::Drop));
    }
}
