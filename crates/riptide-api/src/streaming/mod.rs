//! Streaming module for real-time data delivery.
//!
//! This module provides a comprehensive streaming infrastructure supporting multiple
//! protocols (NDJSON, Server-Sent Events, WebSocket) with features including:
//!
//! - Dynamic buffer management with backpressure handling
//! - Configurable connection limits and timeouts
//! - Comprehensive error handling and recovery
//! - Performance monitoring and metrics
//! - Protocol-specific optimizations
//!
//! # Architecture
//!
//! The streaming module is organized into several key components:
//!
//! - **error**: Error types and recovery strategies
//! - **config**: Configuration management for all streaming protocols
//! - **buffer**: Dynamic buffer management with adaptive sizing
//! - **processor**: Common processing logic for URL handling
//! - **pipeline**: High-level orchestration and coordination
//! - **ndjson**: NDJSON streaming implementation
//! - **sse**: Server-Sent Events implementation
//! - **websocket**: WebSocket bidirectional communication
//!
//! # Usage Examples
//!
//! ## NDJSON Streaming
//! ```rust,no_run
//! use axum::{extract::State, Json};
//! use crate::streaming::ndjson::crawl_stream;
//! use crate::models::CrawlBody;
//! use crate::state::AppState;
//!
//! pub async fn handle_ndjson_stream(
//!     State(app): State<AppState>,
//!     Json(body): Json<CrawlBody>,
//! ) -> impl IntoResponse {
//!     crawl_stream(State(app), Json(body)).await
//! }
//! ```
//!
//! ## WebSocket Streaming
//! ```rust,no_run
//! use axum::extract::{WebSocketUpgrade, State};
//! use crate::streaming::websocket::crawl_websocket;
//! use crate::state::AppState;
//!
//! pub async fn handle_websocket(
//!     ws: WebSocketUpgrade,
//!     State(app): State<AppState>,
//! ) -> impl IntoResponse {
//!     crawl_websocket(ws, State(app)).await
//! }
//! ```
//!
//! ## Server-Sent Events
//! ```rust,no_run
//! use axum::{extract::State, Json};
//! use crate::streaming::sse::crawl_sse;
//! use crate::models::CrawlBody;
//! use crate::state::AppState;
//!
//! pub async fn handle_sse_stream(
//!     State(app): State<AppState>,
//!     Json(body): Json<CrawlBody>,
//! ) -> impl IntoResponse {
//!     crawl_sse(State(app), Json(body)).await
//! }
//! ```

// Re-export core modules
pub mod buffer;
pub mod config;
pub mod error;
pub mod lifecycle;
pub mod metrics;
pub mod ndjson;

// Re-export NDJSON handlers for convenience
pub mod pipeline;
pub mod processor;
pub mod response_helpers;
pub mod sse;
pub mod websocket;

// Re-export commonly used types for convenience
pub use buffer::BufferManager;
pub use config::StreamConfig;
pub use error::StreamingError;
pub use lifecycle::StreamLifecycleManager;
pub use pipeline::StreamingPipeline;

// Note: Public API functions are now exposed through handlers::streaming
// The ndjson module contains internal implementation details

/// Streaming protocol types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Public API - streaming protocol enum
pub enum StreamingProtocol {
    /// NDJSON (Newline Delimited JSON) streaming
    Ndjson,
    /// Server-Sent Events
    Sse,
    /// WebSocket bidirectional communication
    WebSocket,
}

impl StreamingProtocol {
    /// Get the content type for this protocol
    #[allow(dead_code)] // Public API - gets content type for protocol
    pub fn content_type(&self) -> &'static str {
        match self {
            StreamingProtocol::Ndjson => "application/x-ndjson",
            StreamingProtocol::Sse => "text/event-stream",
            StreamingProtocol::WebSocket => "application/json", // For WebSocket messages
        }
    }

    /// Check if protocol supports bidirectional communication
    #[allow(dead_code)] // Public API - checks if protocol is bidirectional
    pub fn is_bidirectional(&self) -> bool {
        matches!(self, StreamingProtocol::WebSocket)
    }

    /// Get default buffer size for this protocol
    #[allow(dead_code)] // Public API - gets default buffer size
    pub fn default_buffer_size(&self) -> usize {
        match self {
            StreamingProtocol::Ndjson => 256,
            StreamingProtocol::Sse => 128,
            StreamingProtocol::WebSocket => 64, // Smaller for real-time
        }
    }

    /// Get recommended keep-alive interval
    #[allow(dead_code)] // Public API - gets keep-alive interval
    pub fn keep_alive_interval(&self) -> std::time::Duration {
        match self {
            StreamingProtocol::Ndjson => std::time::Duration::from_secs(60), // Less frequent
            StreamingProtocol::Sse => std::time::Duration::from_secs(30),
            StreamingProtocol::WebSocket => std::time::Duration::from_secs(30),
        }
    }
}

impl std::fmt::Display for StreamingProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamingProtocol::Ndjson => write!(f, "ndjson"),
            StreamingProtocol::Sse => write!(f, "sse"),
            StreamingProtocol::WebSocket => write!(f, "websocket"),
        }
    }
}

impl std::str::FromStr for StreamingProtocol {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ndjson" | "nd-json" | "newline-delimited-json" => Ok(StreamingProtocol::Ndjson),
            "sse" | "server-sent-events" | "text/event-stream" => Ok(StreamingProtocol::Sse),
            "websocket" | "ws" | "web-socket" => Ok(StreamingProtocol::WebSocket),
            _ => Err(format!("Unknown streaming protocol: {}", s)),
        }
    }
}

/// Streaming health status
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum StreamingHealth {
    /// All systems operating normally
    #[default]
    Healthy,
    /// Some degradation but still operational
    Degraded,
    /// Critical issues affecting operation
    Critical,
    /// System is down or unavailable
    #[allow(dead_code)] // Public API - system down health status
    Down,
}

impl StreamingHealth {
    /// Check if the system is operational
    pub fn is_operational(&self) -> bool {
        matches!(self, StreamingHealth::Healthy | StreamingHealth::Degraded)
    }

    /// Get numeric score (0-100)
    #[allow(dead_code)] // Public API - gets health score
    pub fn score(&self) -> u8 {
        match self {
            StreamingHealth::Healthy => 100,
            StreamingHealth::Degraded => 60,
            StreamingHealth::Critical => 20,
            StreamingHealth::Down => 0,
        }
    }
}

/// Global streaming metrics aggregated across all protocols
#[derive(Debug, Default, Clone)]
pub struct GlobalStreamingMetrics {
    /// Total active connections across all protocols
    pub active_connections: usize,
    /// Total connections created since startup
    pub total_connections: usize,
    /// Total messages sent across all protocols
    pub total_messages_sent: usize,
    /// Total messages dropped due to backpressure
    pub total_messages_dropped: usize,
    /// Average connection duration in milliseconds
    #[allow(dead_code)] // Public API - average connection duration metric
    pub average_connection_duration_ms: f64,
    /// Current system health status
    pub health_status: StreamingHealth,
    /// Memory usage for streaming operations (bytes)
    pub memory_usage_bytes: usize,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
}

impl GlobalStreamingMetrics {
    /// Update health status based on current metrics
    pub fn update_health_status(&mut self) {
        let connection_ratio = if self.total_connections > 0 {
            self.active_connections as f64 / self.total_connections as f64
        } else {
            1.0
        };

        let message_drop_ratio = if self.total_messages_sent > 0 {
            self.total_messages_dropped as f64 / self.total_messages_sent as f64
        } else {
            0.0
        };

        self.health_status = if self.error_rate > 0.1 || message_drop_ratio > 0.2 {
            StreamingHealth::Critical
        } else if self.error_rate > 0.05 || message_drop_ratio > 0.1 || connection_ratio < 0.5 {
            StreamingHealth::Degraded
        } else {
            StreamingHealth::Healthy // No traffic is okay, or general healthy state
        };
    }

    /// Calculate overall system efficiency (0.0 to 1.0)
    #[allow(dead_code)] // Public API - calculates system efficiency
    pub fn efficiency(&self) -> f64 {
        if self.total_messages_sent == 0 {
            return 1.0; // No messages is perfectly efficient
        }

        let delivery_ratio = if self.total_messages_sent > 0 {
            (self.total_messages_sent - self.total_messages_dropped) as f64
                / self.total_messages_sent as f64
        } else {
            1.0
        };

        let error_factor = 1.0 - self.error_rate;

        (delivery_ratio + error_factor) / 2.0
    }

    /// Get memory usage in MB
    #[allow(dead_code)] // Public API - gets memory usage in MB
    pub fn memory_usage_mb(&self) -> f64 {
        self.memory_usage_bytes as f64 / (1024.0 * 1024.0)
    }
}

/// Streaming module initialization and configuration
pub struct StreamingModule {
    config: StreamConfig,
    buffer_manager: std::sync::Arc<BufferManager>,
    metrics: std::sync::Arc<tokio::sync::RwLock<GlobalStreamingMetrics>>,
    #[allow(dead_code)] // Public API - lifecycle manager for stream lifecycle
    lifecycle_manager: Option<std::sync::Arc<StreamLifecycleManager>>,
}

impl StreamingModule {
    /// Initialize the streaming module with configuration
    pub fn new(config: Option<StreamConfig>) -> Self {
        let config = config.unwrap_or_else(StreamConfig::from_env);

        Self {
            config,
            buffer_manager: std::sync::Arc::new(BufferManager::new()),
            metrics: std::sync::Arc::new(tokio::sync::RwLock::new(
                GlobalStreamingMetrics::default(),
            )),
            lifecycle_manager: None, // Will be initialized with metrics when available
        }
    }

    /// Initialize with lifecycle manager
    pub fn with_lifecycle_manager(
        config: Option<StreamConfig>,
        metrics: std::sync::Arc<crate::metrics::RipTideMetrics>,
    ) -> Self {
        let config = config.unwrap_or_else(StreamConfig::from_env);
        let lifecycle_manager = std::sync::Arc::new(StreamLifecycleManager::new(metrics));

        Self {
            config,
            buffer_manager: std::sync::Arc::new(BufferManager::new()),
            metrics: std::sync::Arc::new(tokio::sync::RwLock::new(
                GlobalStreamingMetrics::default(),
            )),
            lifecycle_manager: Some(lifecycle_manager),
        }
    }

    /// Get the configuration
    #[allow(dead_code)] // Public API - gets stream configuration
    pub fn config(&self) -> &StreamConfig {
        &self.config
    }

    /// Get the buffer manager
    #[allow(dead_code)] // Public API - gets buffer manager
    pub fn buffer_manager(&self) -> &std::sync::Arc<BufferManager> {
        &self.buffer_manager
    }

    /// Get the lifecycle manager
    #[allow(dead_code)] // Public API - gets lifecycle manager
    pub fn lifecycle_manager(&self) -> Option<&std::sync::Arc<StreamLifecycleManager>> {
        self.lifecycle_manager.as_ref()
    }

    /// Get global metrics
    pub async fn metrics(&self) -> GlobalStreamingMetrics {
        self.metrics.read().await.clone()
    }

    /// Update global metrics
    #[allow(dead_code)] // Public API - updates global metrics
    pub async fn update_metrics<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut GlobalStreamingMetrics),
    {
        let mut metrics = self.metrics.write().await;
        update_fn(&mut metrics);
        metrics.update_health_status();
    }

    /// Check if streaming is healthy
    pub async fn is_healthy(&self) -> bool {
        let metrics = self.metrics.read().await;
        metrics.health_status.is_operational()
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        self.config.validate()
    }

    /// Start background maintenance tasks
    pub async fn start_maintenance_tasks(&self) -> Result<(), StreamingError> {
        let buffer_manager = self.buffer_manager.clone();
        let metrics = self.metrics.clone();

        // Start buffer cleanup task
        let cleanup_interval = self.config.general.metrics_interval;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            loop {
                interval.tick().await;

                // Clean up old buffers
                buffer_manager
                    .cleanup_old_buffers(std::time::Duration::from_secs(3600))
                    .await;

                // Update metrics
                let mut metrics_guard = metrics.write().await;
                let buffer_stats = buffer_manager.global_stats().await;

                metrics_guard.memory_usage_bytes = buffer_stats
                    .values()
                    .map(|stats| stats.current_size * 1024) // Estimate memory usage
                    .sum();

                metrics_guard.update_health_status();
            }
        });

        Ok(())
    }
}

impl Default for StreamingModule {
    fn default() -> Self {
        Self::new(None)
    }
}

/// Convenience function to create a streaming pipeline
#[allow(dead_code)] // Public API - creates streaming pipeline
pub fn create_pipeline(
    app: crate::state::AppState,
    request_id: Option<String>,
) -> StreamingPipeline {
    StreamingPipeline::new(app, request_id)
}

/// Convenience function to validate streaming configuration
#[allow(dead_code)] // Public API - validates streaming configuration
pub fn validate_config(config: &StreamConfig) -> Result<(), String> {
    config.validate()
}

/// Get protocol-specific optimal configuration
#[allow(dead_code)] // Public API - gets protocol-specific configuration
pub fn get_protocol_config(
    protocol: StreamingProtocol,
    base_config: &StreamConfig,
) -> StreamConfig {
    let mut config = base_config.clone();

    // Adjust buffer configuration based on protocol
    config.buffer.default_size = protocol.default_buffer_size();

    // Adjust other settings
    match protocol {
        StreamingProtocol::Ndjson => {
            config.general.default_timeout = std::time::Duration::from_secs(600);
            // Longer for batch ops
        }
        StreamingProtocol::Sse => {
            config.sse.keep_alive_interval = protocol.keep_alive_interval();
        }
        StreamingProtocol::WebSocket => {
            config.websocket.ping_interval = protocol.keep_alive_interval();
        }
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_protocol_parsing() {
        assert_eq!(
            "ndjson".parse::<StreamingProtocol>().unwrap(),
            StreamingProtocol::Ndjson
        );
        assert_eq!(
            "sse".parse::<StreamingProtocol>().unwrap(),
            StreamingProtocol::Sse
        );
        assert_eq!(
            "websocket".parse::<StreamingProtocol>().unwrap(),
            StreamingProtocol::WebSocket
        );

        assert!("invalid".parse::<StreamingProtocol>().is_err());
    }

    #[test]
    fn test_streaming_protocol_properties() {
        assert_eq!(
            StreamingProtocol::Ndjson.content_type(),
            "application/x-ndjson"
        );
        assert!(!StreamingProtocol::Ndjson.is_bidirectional());
        assert!(StreamingProtocol::WebSocket.is_bidirectional());

        assert_eq!(StreamingProtocol::WebSocket.default_buffer_size(), 64);
        assert!(
            StreamingProtocol::Ndjson.default_buffer_size()
                > StreamingProtocol::WebSocket.default_buffer_size()
        );
    }

    #[test]
    fn test_streaming_health() {
        assert!(StreamingHealth::Healthy.is_operational());
        assert!(StreamingHealth::Degraded.is_operational());
        assert!(!StreamingHealth::Critical.is_operational());
        assert!(!StreamingHealth::Down.is_operational());

        assert_eq!(StreamingHealth::Healthy.score(), 100);
        assert_eq!(StreamingHealth::Down.score(), 0);
    }

    #[test]
    fn test_global_metrics() {
        let mut metrics = GlobalStreamingMetrics {
            total_messages_sent: 100,
            total_messages_dropped: 5,
            error_rate: 0.02,
            ..Default::default()
        };

        // Efficiency = (delivery_ratio + error_factor) / 2.0
        // delivery_ratio = (100 - 5) / 100 = 0.95
        // error_factor = 1.0 - 0.02 = 0.98
        // efficiency = (0.95 + 0.98) / 2.0 = 0.965
        assert!(
            (metrics.efficiency() - 0.965).abs() < 0.01,
            "Expected ~0.965, got {}",
            metrics.efficiency()
        );

        metrics.update_health_status();
        assert_eq!(metrics.health_status, StreamingHealth::Healthy);

        // Test degraded state
        metrics.error_rate = 0.08;
        metrics.update_health_status();
        assert_eq!(metrics.health_status, StreamingHealth::Degraded);
    }

    #[tokio::test]
    async fn test_streaming_module() {
        let module = StreamingModule::new(None);
        assert!(module.validate().is_ok());
        assert!(module.is_healthy().await);

        let metrics = module.metrics().await;
        assert_eq!(metrics.active_connections, 0);
        assert_eq!(metrics.health_status, StreamingHealth::Healthy);
    }

    #[test]
    fn test_protocol_config() {
        let base_config = StreamConfig::default();
        let ws_config = get_protocol_config(StreamingProtocol::WebSocket, &base_config);

        assert_eq!(
            ws_config.buffer.default_size,
            StreamingProtocol::WebSocket.default_buffer_size()
        );
        assert_eq!(
            ws_config.websocket.ping_interval,
            StreamingProtocol::WebSocket.keep_alive_interval()
        );
    }
}
