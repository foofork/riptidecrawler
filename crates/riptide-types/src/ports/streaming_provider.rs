//! Streaming provider port for hexagonal architecture
//!
//! This module provides backend-agnostic streaming coordination interfaces.
//! Note: Low-level transport traits (StreamingTransport, StreamProcessor, StreamLifecycle)
//! are already defined in `streaming.rs`. This module provides higher-level coordination.
//!
//! # Design Goals
//!
//! - **Coordination Abstraction**: High-level streaming session management
//! - **Metrics Integration**: Stream performance monitoring
//! - **Testability**: Easy mocking without concrete streaming infrastructure
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────┐
//! │ Domain Layer (Ports)                     │
//! │  ├─ StreamingProvider (coordination)     │
//! │  └─ StreamingTransport (low-level)       │
//! └──────────────────────────────────────────┘
//!              ↑ implements          ↑ uses
//!              │                     │
//! ┌────────────┴──────────┐   ┌────┴──────────────┐
//! │ Infrastructure         │   │ Application       │
//! │ - StreamingModule      │   │ - Business logic  │
//! │ - Coordinators         │   │ - Handlers        │
//! └────────────────────────┘   └───────────────────┘
//! ```

use super::streaming::{StreamConfig, StreamMetrics};
use crate::error::Result as RiptideResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// High-level streaming coordination abstraction
///
/// This trait provides session-level streaming management,
/// complementing the low-level StreamingTransport trait.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` for use in async contexts.
#[async_trait]
pub trait StreamingProvider: Send + Sync {
    /// Start a new streaming session
    ///
    /// # Arguments
    ///
    /// * `config` - Stream configuration
    ///
    /// # Returns
    ///
    /// * `Ok(handle)` - Stream handle for managing the session
    /// * `Err(_)` - Configuration or initialization error
    async fn start_stream(&self, config: StreamConfig) -> RiptideResult<StreamHandle>;

    /// Get streaming metrics
    ///
    /// # Returns
    ///
    /// Current stream metrics across all active sessions
    async fn metrics(&self) -> StreamMetrics;

    /// Stop an active streaming session
    ///
    /// # Arguments
    ///
    /// * `handle` - Stream handle to stop
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Stream stopped successfully
    /// * `Err(_)` - Error during shutdown
    async fn stop_stream(&self, handle: StreamHandle) -> RiptideResult<()>;

    /// Get active stream count
    ///
    /// # Returns
    ///
    /// Number of currently active streams
    async fn active_streams(&self) -> usize;

    /// Check if a stream is active
    ///
    /// # Arguments
    ///
    /// * `handle` - Stream handle to check
    ///
    /// # Returns
    ///
    /// `true` if stream is active, `false` otherwise
    async fn is_stream_active(&self, handle: &StreamHandle) -> bool;
}

/// Handle for managing a streaming session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StreamHandle {
    /// Unique stream identifier
    pub id: Uuid,
}

impl StreamHandle {
    /// Create a new stream handle
    #[must_use]
    pub fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }

    /// Create a stream handle from a UUID
    #[must_use]
    pub fn from_uuid(id: Uuid) -> Self {
        Self { id }
    }

    /// Get the UUID
    #[must_use]
    pub fn uuid(&self) -> Uuid {
        self.id
    }
}

impl Default for StreamHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for StreamHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StreamHandle({})", self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_handle_creation() {
        let handle = StreamHandle::new();
        assert_ne!(handle.id, Uuid::nil());
    }

    #[test]
    fn test_stream_handle_from_uuid() {
        let uuid = Uuid::new_v4();
        let handle = StreamHandle::from_uuid(uuid);
        assert_eq!(handle.uuid(), uuid);
    }

    #[test]
    fn test_stream_handle_equality() {
        let uuid = Uuid::new_v4();
        let handle1 = StreamHandle::from_uuid(uuid);
        let handle2 = StreamHandle::from_uuid(uuid);
        assert_eq!(handle1, handle2);
    }

    #[test]
    fn test_stream_handle_display() {
        let handle = StreamHandle::new();
        let display = format!("{}", handle);
        assert!(display.starts_with("StreamHandle("));
    }
}
