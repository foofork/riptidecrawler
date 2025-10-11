//! Shared streaming metrics for SSE and WebSocket connections
//!
//! This module provides unified metrics tracking for all streaming protocols,
//! eliminating duplication between SSE and WebSocket implementations.
//!
//! ## Prometheus Integration
//!
//! All metrics in this module are integrated with Prometheus via the RipTideMetrics
//! struct in `crate::metrics`. The StreamLifecycleManager automatically records
//! these metrics during stream lifecycle events.
//!
//! ## Grafana Dashboard Queries
//!
//! ### Active Connections
//! ```promql
//! # Current active streaming connections
//! riptide_streaming_active_connections
//!
//! # Connection growth rate
//! rate(riptide_streaming_total_connections[5m])
//! ```
//!
//! ### Throughput & Performance
//! ```promql
//! # Messages sent per second
//! rate(riptide_streaming_messages_sent_total[5m])
//!
//! # Message drop rate
//! rate(riptide_streaming_messages_dropped_total[5m])
//!
//! # Delivery success ratio
//! (rate(riptide_streaming_messages_sent_total[5m]) /
//!  (rate(riptide_streaming_messages_sent_total[5m]) +
//!   rate(riptide_streaming_messages_dropped_total[5m]))) * 100
//! ```
//!
//! ### Connection Duration
//! ```promql
//! # P50 connection duration
//! histogram_quantile(0.50, rate(riptide_streaming_connection_duration_seconds_bucket[5m]))
//!
//! # P95 connection duration
//! histogram_quantile(0.95, rate(riptide_streaming_connection_duration_seconds_bucket[5m]))
//!
//! # P99 connection duration
//! histogram_quantile(0.99, rate(riptide_streaming_connection_duration_seconds_bucket[5m]))
//! ```
//!
//! ### Error Rates
//! ```promql
//! # Current error rate
//! riptide_streaming_error_rate
//!
//! # Alert: High error rate
//! riptide_streaming_error_rate > 0.05
//! ```
//!
//! ### Memory Usage
//! ```promql
//! # Memory usage in MB
//! riptide_streaming_memory_usage_bytes / (1024 * 1024)
//!
//! # Alert: High memory usage
//! riptide_streaming_memory_usage_bytes > 500000000  # 500MB
//! ```
//!
//! ## Alert Thresholds
//!
//! Recommended alert configurations:
//! - **Error Rate**: > 5% (0.05) - Warning, > 10% (0.10) - Critical
//! - **Message Drop Rate**: > 10% - Warning, > 20% - Critical
//! - **Memory Usage**: > 500MB - Warning, > 1GB - Critical
//! - **Connection Duration P99**: > 60s - Warning

use std::time::Duration;

/// Unified streaming connection metrics
///
/// Tracks common metrics across all streaming protocols (SSE, WebSocket, NDJSON)
#[derive(Debug, Default, Clone)]
pub struct StreamingMetrics {
    // Connection tracking (common)
    pub active_connections: usize,
    pub total_connections: usize,
    pub average_connection_duration_ms: f64,

    // Message tracking (common - generic name works for both events and messages)
    pub total_items_sent: usize,
    pub total_items_received: usize,
    pub items_dropped: usize,

    // Error tracking (common)
    pub error_count: usize,
    pub reconnection_count: usize,
}

impl StreamingMetrics {
    /// Record a new streaming connection
    ///
    /// Automatically synced to Prometheus via StreamLifecycleManager
    pub fn record_connection(&mut self) {
        self.active_connections += 1;
        self.total_connections += 1;
    }

    /// Record connection closure with duration tracking
    ///
    /// Automatically synced to Prometheus via StreamLifecycleManager
    pub fn record_disconnection(&mut self, duration: Duration) {
        self.active_connections = self.active_connections.saturating_sub(1);

        // Update rolling average connection duration
        let total_duration =
            self.average_connection_duration_ms * (self.total_connections - 1) as f64;
        self.average_connection_duration_ms =
            (total_duration + duration.as_millis() as f64) / self.total_connections as f64;
    }

    /// Record an item (event/message) sent to client
    ///
    /// Automatically synced to Prometheus via StreamLifecycleManager
    pub fn record_item_sent(&mut self) {
        self.total_items_sent += 1;
    }

    /// Record an item (message) received from client
    ///
    /// WebSocket-specific: tracks bidirectional communication
    pub fn record_item_received(&mut self) {
        self.total_items_received += 1;
    }

    /// Record an item (event/message) dropped due to backpressure
    ///
    /// Triggers Prometheus counter and affects delivery ratio
    pub fn record_item_dropped(&mut self) {
        self.items_dropped += 1;
    }

    /// Record a client reconnection
    ///
    /// Used for SSE and WebSocket reconnection tracking
    pub fn record_reconnection(&mut self) {
        self.reconnection_count += 1;
    }

    /// Record an error condition
    ///
    /// Affects error_rate metric in Prometheus
    pub fn record_error(&mut self) {
        self.error_count += 1;
    }

    /// Get delivery ratio (sent / total attempted)
    ///
    /// Returns value between 0.0 and 1.0
    /// Used in Grafana dashboards for delivery success rate
    pub fn delivery_ratio(&self) -> f64 {
        let total_items = self.total_items_sent + self.items_dropped;
        if total_items == 0 {
            1.0
        } else {
            self.total_items_sent as f64 / total_items as f64
        }
    }

    /// Get reconnection rate (reconnections / total connections)
    ///
    /// Returns value between 0.0 and 1.0
    /// High values indicate connection instability
    pub fn reconnection_rate(&self) -> f64 {
        if self.total_connections == 0 {
            0.0
        } else {
            self.reconnection_count as f64 / self.total_connections as f64
        }
    }

    /// Get connection health ratio (1.0 - error_rate)
    ///
    /// Returns value between 0.0 and 1.0
    /// 1.0 = perfect health, 0.0 = all connections failed
    pub fn health_ratio(&self) -> f64 {
        if self.total_connections == 0 {
            1.0
        } else {
            1.0 - (self.error_count as f64 / self.total_connections as f64)
        }
    }

    /// Get average items per connection
    ///
    /// Useful for capacity planning and performance analysis
    pub fn average_items_per_connection(&self) -> f64 {
        if self.total_connections == 0 {
            0.0
        } else {
            self.total_items_sent as f64 / self.total_connections as f64
        }
    }

    /// Get error rate (errors / total connections)
    ///
    /// Exported to Prometheus as riptide_streaming_error_rate
    #[allow(dead_code)]
    pub fn error_rate(&self) -> f64 {
        if self.total_connections == 0 {
            0.0
        } else {
            self.error_count as f64 / self.total_connections as f64
        }
    }

    /// Export metrics to Prometheus format
    ///
    /// Used by RipTideMetrics to sync with Prometheus registry
    #[allow(dead_code)]
    pub fn to_prometheus(&self, metrics: &crate::metrics::RipTideMetrics) {
        metrics
            .streaming_active_connections
            .set(self.active_connections as f64);
        metrics
            .streaming_total_connections
            .set(self.total_connections as f64);
        metrics.streaming_error_rate.set(self.error_rate());
    }
}

// Convenience type aliases for clarity in specific protocols
/// SSE-specific metrics type alias
pub type SseMetrics = StreamingMetrics;

/// WebSocket-specific metrics type alias
pub type WebSocketMetrics = StreamingMetrics;

/// NDJSON-specific metrics type alias
pub type NdjsonMetrics = StreamingMetrics;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_tracking() {
        let mut metrics = StreamingMetrics::default();

        metrics.record_connection();
        assert_eq!(metrics.active_connections, 1);
        assert_eq!(metrics.total_connections, 1);

        metrics.record_connection();
        assert_eq!(metrics.active_connections, 2);
        assert_eq!(metrics.total_connections, 2);

        metrics.record_disconnection(Duration::from_secs(30));
        assert_eq!(metrics.active_connections, 1);
        // Average is calculated over total_connections (2), so 30000 / 2 = 15000
        assert_eq!(metrics.average_connection_duration_ms, 15000.0);
    }

    #[test]
    fn test_item_delivery_tracking() {
        let mut metrics = StreamingMetrics::default();

        metrics.record_item_sent();
        metrics.record_item_sent();
        metrics.record_item_dropped();

        assert_eq!(metrics.total_items_sent, 2);
        assert_eq!(metrics.items_dropped, 1);
        assert_eq!(metrics.delivery_ratio(), 2.0 / 3.0);
    }

    #[test]
    fn test_reconnection_tracking() {
        let mut metrics = StreamingMetrics::default();
        metrics.record_connection();
        metrics.record_reconnection();

        assert_eq!(metrics.reconnection_count, 1);
        assert_eq!(metrics.reconnection_rate(), 1.0);
    }

    #[test]
    fn test_health_ratio() {
        let mut metrics = StreamingMetrics::default();

        metrics.record_connection();
        metrics.record_connection();
        metrics.record_error();

        assert_eq!(metrics.health_ratio(), 0.5); // 1 error out of 2 connections
    }

    #[test]
    fn test_average_items_per_connection() {
        let mut metrics = StreamingMetrics::default();

        metrics.record_connection();
        metrics.record_item_sent();
        metrics.record_item_sent();

        metrics.record_connection();
        metrics.record_item_sent();

        assert_eq!(metrics.average_items_per_connection(), 1.5); // 3 items / 2 connections
    }

    #[test]
    fn test_type_aliases_work() {
        let sse_metrics: SseMetrics = StreamingMetrics::default();
        let ws_metrics: WebSocketMetrics = StreamingMetrics::default();
        let ndjson_metrics: NdjsonMetrics = StreamingMetrics::default();

        // All should have the same structure
        assert_eq!(sse_metrics.active_connections, 0);
        assert_eq!(ws_metrics.active_connections, 0);
        assert_eq!(ndjson_metrics.active_connections, 0);
    }
}
