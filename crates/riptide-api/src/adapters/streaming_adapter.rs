//! Streaming Module Adapter for hexagonal architecture
//!
//! Adapts the concrete StreamingModule implementation to streaming port traits.

use std::sync::Arc;

/// Adapter that provides streaming capabilities using the StreamingModule
pub struct StreamingProviderAdapter {
    inner: Arc<crate::streaming::StreamingModule>,
}

impl StreamingProviderAdapter {
    /// Create a new StreamingProviderAdapter wrapping the concrete implementation
    pub fn new(module: Arc<crate::streaming::StreamingModule>) -> Self {
        Self { inner: module }
    }

    /// Check if streaming is healthy
    pub async fn is_healthy(&self) -> bool {
        self.inner.is_healthy().await
    }

    /// Get streaming metrics
    pub async fn metrics(&self) -> crate::streaming::GlobalStreamingMetrics {
        self.inner.metrics().await
    }
}

// Note: StreamingModule doesn't directly implement StreamingTransport
// It provides the infrastructure for transport adapters (WebSocket, SSE, NDJSON)
// Those are implemented in the adapters module already
