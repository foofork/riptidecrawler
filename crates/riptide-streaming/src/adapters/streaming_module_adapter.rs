//! Adapter implementing StreamingProvider port for StreamingCoordinator
//!
//! This adapter bridges the concrete StreamingCoordinator implementation
//! with the abstract StreamingProvider port trait, enabling dependency
//! inversion in the hexagonal architecture.

use async_trait::async_trait;
use riptide_types::error::Result as RiptideResult;
use riptide_types::ports::streaming::{StreamConfig, StreamMetrics};
use riptide_types::ports::streaming_provider::{StreamHandle, StreamingProvider};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::StreamingCoordinator;

/// Adapter that implements StreamingProvider using StreamingCoordinator
///
/// This adapter wraps the concrete StreamingCoordinator and implements
/// the abstract StreamingProvider trait, allowing the coordinator to be
/// injected as a dependency via the port interface.
#[derive(Debug, Clone)]
pub struct StreamingModuleAdapter {
    /// Inner coordinator wrapped in Arc for shared ownership
    coordinator: Arc<RwLock<StreamingCoordinator>>,

    /// Track active stream handles
    active_handles: Arc<RwLock<HashMap<StreamHandle, uuid::Uuid>>>,
}

impl StreamingModuleAdapter {
    /// Create a new adapter wrapping a StreamingCoordinator
    ///
    /// # Arguments
    ///
    /// * `coordinator` - The concrete StreamingCoordinator to wrap
    ///
    /// # Returns
    ///
    /// Arc-wrapped adapter ready for dependency injection
    pub fn new(coordinator: StreamingCoordinator) -> Arc<Self> {
        Arc::new(Self {
            coordinator: Arc::new(RwLock::new(coordinator)),
            active_handles: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get reference to inner coordinator (for testing/diagnostics)
    #[cfg(test)]
    pub fn inner(&self) -> Arc<RwLock<StreamingCoordinator>> {
        Arc::clone(&self.coordinator)
    }
}

#[async_trait]
impl StreamingProvider for StreamingModuleAdapter {
    async fn start_stream(&self, _config: StreamConfig) -> RiptideResult<StreamHandle> {
        // Generate extraction ID from config
        let extraction_id = format!("extraction-{}", uuid::Uuid::new_v4());

        // Start stream via coordinator
        let stream_id = {
            let mut coord = self.coordinator.write().await;
            coord.start_stream(extraction_id).await.map_err(|e| {
                riptide_types::error::RiptideError::custom(format!(
                    "Failed to start stream: {}",
                    e
                ))
            })?
        };

        // Create handle and track it
        let handle = StreamHandle::from_uuid(stream_id);
        {
            let mut handles = self.active_handles.write().await;
            handles.insert(handle, stream_id);
        }

        Ok(handle)
    }

    async fn metrics(&self) -> StreamMetrics {
        // Get metrics from coordinator streams
        let coordinator = self.coordinator.read().await;

        StreamMetrics {
            active_connections: coordinator.streams.len(),
            total_messages_sent: 0, // Coordinator doesn't track this yet
            total_messages_dropped: 0,
            average_latency_ms: 0.0,
            throughput_bytes_per_sec: 0.0,
            error_rate: 0.0,
        }
    }

    async fn stop_stream(&self, handle: StreamHandle) -> RiptideResult<()> {
        // Get the coordinator's stream ID
        let stream_id = {
            let mut handles = self.active_handles.write().await;
            handles.remove(&handle).ok_or_else(|| {
                riptide_types::error::RiptideError::custom(format!(
                    "Stream handle not found: {}",
                    handle
                ))
            })?
        };

        // Complete the stream
        {
            let mut coord = self.coordinator.write().await;
            coord.complete_stream(stream_id).await.map_err(|e| {
                riptide_types::error::RiptideError::custom(format!("Failed to stop stream: {}", e))
            })?;
        }

        Ok(())
    }

    async fn active_streams(&self) -> usize {
        let handles = self.active_handles.read().await;
        handles.len()
    }

    async fn is_stream_active(&self, handle: &StreamHandle) -> bool {
        let handles = self.active_handles.read().await;
        handles.contains_key(handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adapter_creation() {
        let coordinator = StreamingCoordinator::new();
        let adapter = StreamingModuleAdapter::new(coordinator);

        assert_eq!(adapter.active_streams(), 0);
    }

    #[tokio::test]
    async fn test_start_stream() {
        let coordinator = StreamingCoordinator::new();
        let adapter = StreamingModuleAdapter::new(coordinator);

        let config = StreamConfig::default();
        let handle = adapter.start_stream(config).await.unwrap();

        assert_eq!(adapter.active_streams(), 1);
        assert!(adapter.is_stream_active(&handle));
    }

    #[tokio::test]
    async fn test_stop_stream() {
        let coordinator = StreamingCoordinator::new();
        let adapter = StreamingModuleAdapter::new(coordinator);

        let config = StreamConfig::default();
        let handle = adapter.start_stream(config).await.unwrap();
        assert_eq!(adapter.active_streams(), 1);

        adapter.stop_stream(handle).await.unwrap();
        assert_eq!(adapter.active_streams(), 0);
        assert!(!adapter.is_stream_active(&handle));
    }

    #[tokio::test]
    async fn test_metrics() {
        let coordinator = StreamingCoordinator::new();
        let adapter = StreamingModuleAdapter::new(coordinator);

        let metrics = adapter.metrics().await;
        assert_eq!(metrics.active_connections, 0);
    }
}
