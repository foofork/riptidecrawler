//! Streaming infrastructure module.
//!
//! This module provides core streaming infrastructure components:
//! - **buffer**: Dynamic buffer management with backpressure handling
//! - **config**: Configuration for streaming protocols
//! - **error**: Error types and recovery strategies
//!
//! # Architecture
//!
//! Business logic has been moved to:
//! - `crates/riptide-facade/src/facades/streaming.rs` (StreamingFacade)
//! - `crates/riptide-api/src/adapters/sse_transport.rs` (SSE transport)
//! - `crates/riptide-api/src/adapters/websocket_transport.rs` (WebSocket transport)
//!
//! This module now contains only core infrastructure that is protocol-agnostic.

// Core infrastructure modules
pub mod buffer;
pub mod config;
pub mod error;

// Re-export commonly used types
pub use buffer::BufferManager;
pub use config::StreamConfig;
pub use error::{StreamingError, StreamingResult};

/// Streaming protocol types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub fn content_type(&self) -> &'static str {
        match self {
            StreamingProtocol::Ndjson => "application/x-ndjson",
            StreamingProtocol::Sse => "text/event-stream",
            StreamingProtocol::WebSocket => "application/json",
        }
    }

    /// Check if protocol supports bidirectional communication
    pub fn is_bidirectional(&self) -> bool {
        matches!(self, StreamingProtocol::WebSocket)
    }

    /// Get default buffer size for this protocol
    pub fn default_buffer_size(&self) -> usize {
        match self {
            StreamingProtocol::Ndjson => 256,
            StreamingProtocol::Sse => 128,
            StreamingProtocol::WebSocket => 64,
        }
    }

    /// Get recommended keep-alive interval
    pub fn keep_alive_interval(&self) -> std::time::Duration {
        match self {
            StreamingProtocol::Ndjson => std::time::Duration::from_secs(60),
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
    Down,
}

impl StreamingHealth {
    /// Check if the system is operational
    pub fn is_operational(&self) -> bool {
        matches!(self, StreamingHealth::Healthy | StreamingHealth::Degraded)
    }

    /// Get numeric score (0-100)
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
            StreamingHealth::Healthy
        };
    }

    /// Calculate overall system efficiency (0.0 to 1.0)
    pub fn efficiency(&self) -> f64 {
        if self.total_messages_sent == 0 {
            return 1.0;
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
    pub fn memory_usage_mb(&self) -> f64 {
        self.memory_usage_bytes as f64 / (1024.0 * 1024.0)
    }
}

/// Streaming module initialization and configuration
pub struct StreamingModule {
    config: StreamConfig,
    buffer_manager: std::sync::Arc<BufferManager>,
    metrics: std::sync::Arc<tokio::sync::RwLock<GlobalStreamingMetrics>>,
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
        }
    }

    /// Initialize with lifecycle manager and metrics
    pub fn with_lifecycle_manager(
        _lifecycle: Option<std::sync::Arc<()>>, // Placeholder type
        _metrics: std::sync::Arc<crate::metrics_transport::TransportMetrics>,
    ) -> Self {
        // TODO: Integrate lifecycle manager in Phase 6
        Self::new(None)
    }

    /// Get the configuration
    pub fn config(&self) -> &StreamConfig {
        &self.config
    }

    /// Get the buffer manager
    pub fn buffer_manager(&self) -> &std::sync::Arc<BufferManager> {
        &self.buffer_manager
    }

    /// Get global metrics
    pub async fn metrics(&self) -> GlobalStreamingMetrics {
        self.metrics.read().await.clone()
    }

    /// Update global metrics
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
                    .map(|stats| stats.current_size * 1024)
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

/// Get protocol-specific optimal configuration
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
