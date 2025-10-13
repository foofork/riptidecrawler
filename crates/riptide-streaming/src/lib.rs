//! RipTide Streaming - Real-time extraction results streaming and report generation
//!
//! This crate provides:
//! - NDJSON streaming for real-time extraction results
//! - HTML report generation with dynamic templates
//! - Progress tracking and backpressure handling
//! - CLI tools for riptide

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Result from an extraction operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub id: String,
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub extraction_time_ms: u64,
    pub word_count: usize,
    pub links: Vec<String>,
    pub images: Vec<String>,
}

/// Error types for streaming operations
#[derive(Debug, thiserror::Error)]
pub enum StreamingError {
    #[error("Stream not found: {0}")]
    StreamNotFound(Uuid),

    #[error("Stream already completed: {0}")]
    StreamCompleted(Uuid),

    #[error("Backpressure limit exceeded")]
    BackpressureExceeded,

    #[error("Report generation failed: {0}")]
    ReportGenerationFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl From<tokio_util::codec::LinesCodecError> for StreamingError {
    fn from(err: tokio_util::codec::LinesCodecError) -> Self {
        match err {
            tokio_util::codec::LinesCodecError::MaxLineLengthExceeded => {
                StreamingError::ConfigError("Line length exceeded maximum".to_string())
            }
            tokio_util::codec::LinesCodecError::Io(io_err) => StreamingError::IoError(io_err),
        }
    }
}

pub type StreamingResult<T> = Result<T, StreamingError>;

pub mod api_handlers;
pub mod backpressure;
pub mod config;
pub mod ndjson;
pub mod openapi;
pub mod progress;
pub mod reports;
pub mod server;

pub use api_handlers::*; // ✅ ENABLED - API handlers for report endpoints
pub use ndjson::*;
pub use reports::{
    DomainStats, ReportConfig, ReportData, ReportFormat, ReportGenerator, ReportTheme,
    TimelineEntry, WordFrequency,
}; // ✅ ENABLED - ReportGenerator activated for Sprint 5A (selective export to avoid monitoring conflict)
   // pub use backpressure::*; // BackpressureController not needed - tests can use backpressure::BackpressureController directly
pub use config::*; // ✅ ENABLED - includes BackpressureConfig
pub use progress::*; // ✅ ENABLED // ✅ ENABLED - for streaming tests
pub use server::{create_server, ServerState}; // ✅ ENABLED - HTTP server for streaming endpoints
                                              // pub use openapi::*;       // Verify before enabling

// Note: ReportGenerator has been activated for Sprint 5A with HTML templates,
// chart generation, and visualization support. The module uses its own
// ExtractionResult type defined in this crate for streaming reports.

/// Main streaming coordinator for extraction results
#[derive(Debug, Clone)]
pub struct StreamingCoordinator {
    pub streams: HashMap<Uuid, StreamInfo>,
    pub progress_tracker: ProgressTracker, // ✅ ENABLED
                                           // pub reporter: ReportGenerator,        // Keep disabled
}

/// Information about an active stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamInfo {
    pub id: Uuid,
    pub extraction_id: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub total_items: Option<usize>,
    pub processed_items: usize,
    pub status: StreamStatus,
}

/// Status of a streaming operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamStatus {
    Active,
    Paused,
    Completed,
    Failed(String),
}

impl StreamingCoordinator {
    /// Create a new streaming coordinator
    pub fn new() -> Self {
        Self {
            streams: HashMap::new(),
            progress_tracker: ProgressTracker::new(), // ✅ ENABLED
                                                      // reporter: ReportGenerator::new(),        // Available but not used in coordinator
        }
    }

    /// Start a new streaming session for an extraction
    pub async fn start_stream(&mut self, extraction_id: String) -> Result<Uuid> {
        let stream_id = Uuid::new_v4();
        let stream_info = StreamInfo {
            id: stream_id,
            extraction_id,
            start_time: chrono::Utc::now(),
            total_items: None,
            processed_items: 0,
            status: StreamStatus::Active,
        };

        self.streams.insert(stream_id, stream_info);
        self.progress_tracker.start_tracking(stream_id).await?; // ✅ ENABLED

        Ok(stream_id)
    }

    /// Get stream information
    pub fn get_stream(&self, stream_id: &Uuid) -> Option<&StreamInfo> {
        self.streams.get(stream_id)
    }

    /// Update stream progress
    pub async fn update_progress(
        &mut self,
        stream_id: Uuid,
        processed: usize,
        total: Option<usize>,
    ) -> Result<()> {
        if let Some(stream) = self.streams.get_mut(&stream_id) {
            stream.processed_items = processed;
            if let Some(total) = total {
                stream.total_items = Some(total);
            }
            self.progress_tracker
                .update_progress(stream_id, processed, total)
                .await?; // ✅ ENABLED
        }
        Ok(())
    }

    /// Complete a stream
    pub async fn complete_stream(&mut self, stream_id: Uuid) -> Result<()> {
        if let Some(stream) = self.streams.get_mut(&stream_id) {
            stream.status = StreamStatus::Completed;
            self.progress_tracker.complete_tracking(stream_id).await?; // ✅ ENABLED
        }
        Ok(())
    }

    // Note: Report generation is available via ReportGenerator.
    // The coordinator can optionally integrate it in the future:
    /*
    /// Generate report for a completed extraction
    pub async fn generate_report(&self, extraction_id: &str, format: ReportFormat) -> Result<Vec<u8>> {
        let generator = ReportGenerator::new();
        generator.generate_report(extraction_id, format).await
    }
    */
}

impl Default for StreamingCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress update for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub stream_id: Uuid,
    pub extraction_id: String,
    pub processed: usize,
    pub total: Option<usize>,
    pub current_item: Option<ExtractionResult>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub rate_per_second: f64,
    pub estimated_completion: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_streaming_coordinator_creation() {
        let coordinator = StreamingCoordinator::new();
        assert!(coordinator.streams.is_empty());
    }

    #[tokio::test]
    async fn test_start_and_get_stream() {
        let mut coordinator = StreamingCoordinator::new();
        let extraction_id = "test-extraction".to_string();

        let stream_id = coordinator
            .start_stream(extraction_id.clone())
            .await
            .unwrap();
        let stream_info = coordinator.get_stream(&stream_id).unwrap();

        assert_eq!(stream_info.extraction_id, extraction_id);
        assert_eq!(stream_info.processed_items, 0);
        assert!(matches!(stream_info.status, StreamStatus::Active));
    }

    #[tokio::test]
    async fn test_update_progress() {
        let mut coordinator = StreamingCoordinator::new();
        let stream_id = coordinator.start_stream("test".to_string()).await.unwrap();

        coordinator
            .update_progress(stream_id, 50, Some(100))
            .await
            .unwrap();

        let stream_info = coordinator.get_stream(&stream_id).unwrap();
        assert_eq!(stream_info.processed_items, 50);
        assert_eq!(stream_info.total_items, Some(100));
    }

    #[tokio::test]
    async fn test_complete_stream() {
        let mut coordinator = StreamingCoordinator::new();
        let stream_id = coordinator.start_stream("test".to_string()).await.unwrap();

        coordinator.complete_stream(stream_id).await.unwrap();

        let stream_info = coordinator.get_stream(&stream_id).unwrap();
        assert!(matches!(stream_info.status, StreamStatus::Completed));
    }
}
