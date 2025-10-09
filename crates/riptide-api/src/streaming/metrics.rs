//! Shared streaming metrics for SSE and WebSocket connections
//!
//! This module provides unified metrics tracking for all streaming protocols,
//! eliminating duplication between SSE and WebSocket implementations.

use std::time::Duration;

/// Unified streaming connection metrics
///
/// Tracks common metrics across all streaming protocols (SSE, WebSocket, NDJSON)
#[derive(Debug, Default, Clone)]
pub struct StreamingMetrics {
    // Connection tracking (common)
    #[allow(dead_code)] // Public API - active connections count
    pub active_connections: usize,
    #[allow(dead_code)] // Public API - total connections count
    pub total_connections: usize,
    #[allow(dead_code)] // Public API - average connection duration
    pub average_connection_duration_ms: f64,

    // Message tracking (common - generic name works for both events and messages)
    #[allow(dead_code)] // Public API - total items sent
    pub total_items_sent: usize,
    #[allow(dead_code)] // Public API - total items received
    pub total_items_received: usize,
    #[allow(dead_code)] // Public API - items dropped count
    pub items_dropped: usize,

    // Error tracking (common)
    #[allow(dead_code)] // Public API - error count
    pub error_count: usize,
    #[allow(dead_code)] // Public API - reconnection count
    pub reconnection_count: usize,
}

impl StreamingMetrics {
    /// Record a new streaming connection
    #[allow(dead_code)] // Public API - records new connection
    pub fn record_connection(&mut self) {
        self.active_connections += 1;
        self.total_connections += 1;
    }

    /// Record connection closure with duration tracking
    #[allow(dead_code)] // Public API - records disconnection
    pub fn record_disconnection(&mut self, duration: Duration) {
        self.active_connections = self.active_connections.saturating_sub(1);

        // Update rolling average connection duration
        let total_duration =
            self.average_connection_duration_ms * (self.total_connections - 1) as f64;
        self.average_connection_duration_ms =
            (total_duration + duration.as_millis() as f64) / self.total_connections as f64;
    }

    /// Record an item (event/message) sent to client
    #[allow(dead_code)] // Public API - records item sent
    pub fn record_item_sent(&mut self) {
        self.total_items_sent += 1;
    }

    /// Record an item (message) received from client
    #[allow(dead_code)] // Public API - records item received
    pub fn record_item_received(&mut self) {
        self.total_items_received += 1;
    }

    /// Record an item (event/message) dropped due to backpressure
    #[allow(dead_code)] // Public API - records item dropped
    pub fn record_item_dropped(&mut self) {
        self.items_dropped += 1;
    }

    /// Record a client reconnection
    #[allow(dead_code)] // Public API - records reconnection
    pub fn record_reconnection(&mut self) {
        self.reconnection_count += 1;
    }

    /// Record an error condition
    #[allow(dead_code)] // Public API - records error
    pub fn record_error(&mut self) {
        self.error_count += 1;
    }

    /// Get delivery ratio (sent / total attempted)
    #[allow(dead_code)] // Public API - calculates delivery ratio
    pub fn delivery_ratio(&self) -> f64 {
        let total_items = self.total_items_sent + self.items_dropped;
        if total_items == 0 {
            1.0
        } else {
            self.total_items_sent as f64 / total_items as f64
        }
    }

    /// Get reconnection rate (reconnections / total connections)
    #[allow(dead_code)] // Public API - calculates reconnection rate
    pub fn reconnection_rate(&self) -> f64 {
        if self.total_connections == 0 {
            0.0
        } else {
            self.reconnection_count as f64 / self.total_connections as f64
        }
    }

    /// Get connection health ratio (1.0 - error_rate)
    #[allow(dead_code)] // Public API - calculates health ratio
    pub fn health_ratio(&self) -> f64 {
        if self.total_connections == 0 {
            1.0
        } else {
            1.0 - (self.error_count as f64 / self.total_connections as f64)
        }
    }

    /// Get average items per connection
    #[allow(dead_code)] // Public API - calculates average items per connection
    pub fn average_items_per_connection(&self) -> f64 {
        if self.total_connections == 0 {
            0.0
        } else {
            self.total_items_sent as f64 / self.total_connections as f64
        }
    }
}

// Convenience type aliases for clarity in specific protocols
#[allow(dead_code)] // Public API - SSE metrics type alias
pub type SseMetrics = StreamingMetrics;
#[allow(dead_code)] // Public API - WebSocket metrics type alias
pub type WebSocketMetrics = StreamingMetrics;
#[allow(dead_code)] // Type alias for streaming metrics compatibility
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
