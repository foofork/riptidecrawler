//! Progress tracking for streaming operations
//!
//! This module provides real-time progress tracking with rate limiting,
//! estimation, and performance metrics.

use crate::{StreamingError, StreamingResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

/// Progress tracking information for a stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub stream_id: Uuid,
    #[serde(skip, default = "Instant::now")]
    pub start_time: Instant,
    #[serde(skip, default = "Instant::now")]
    pub last_update: Instant,
    pub processed_items: usize,
    pub total_items: Option<usize>,
    pub current_rate: f64,
    pub average_rate: f64,
    #[serde(skip, default)]
    pub estimated_completion: Option<Duration>,
    pub stage: ProgressStage,
    pub bytes_processed: u64,
    pub errors_count: usize,
}

/// Current stage of the extraction process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgressStage {
    Initializing,
    Discovering,
    Extracting,
    Processing,
    Finalizing,
    Completed,
    Failed(String),
}

/// Progress update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub stream_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: ProgressEventType,
    pub data: serde_json::Value,
}

/// Types of progress events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgressEventType {
    Started,
    ItemProcessed,
    StageChanged,
    RateUpdated,
    ErrorOccurred,
    Completed,
    Failed,
}

/// Configuration for progress tracking
#[derive(Debug, Clone)]
pub struct ProgressConfig {
    pub update_interval: Duration,
    pub rate_calculation_window: Duration,
    pub enable_estimation: bool,
    pub max_event_history: usize,
}

impl Default for ProgressConfig {
    fn default() -> Self {
        Self {
            update_interval: Duration::from_secs(1),
            rate_calculation_window: Duration::from_secs(30),
            enable_estimation: true,
            max_event_history: 1000,
        }
    }
}

/// Progress tracker for managing multiple streams
#[derive(Debug, Clone)]
pub struct ProgressTracker {
    progress_info: Arc<RwLock<HashMap<Uuid, ProgressInfo>>>,
    event_senders: Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<ProgressEvent>>>>,
    config: ProgressConfig,
    #[allow(clippy::type_complexity)]
    rate_samples: Arc<RwLock<HashMap<Uuid, Vec<(Instant, usize)>>>>,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new() -> Self {
        Self::with_config(ProgressConfig::default())
    }

    /// Create a new progress tracker with custom configuration
    pub fn with_config(config: ProgressConfig) -> Self {
        Self {
            progress_info: Arc::new(RwLock::new(HashMap::new())),
            event_senders: Arc::new(RwLock::new(HashMap::new())),
            config,
            rate_samples: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start tracking progress for a stream
    pub async fn start_tracking(
        &self,
        stream_id: Uuid,
    ) -> StreamingResult<mpsc::UnboundedReceiver<ProgressEvent>> {
        let now = Instant::now();
        let info = ProgressInfo {
            stream_id,
            start_time: now,
            last_update: now,
            processed_items: 0,
            total_items: None,
            current_rate: 0.0,
            average_rate: 0.0,
            estimated_completion: None,
            stage: ProgressStage::Initializing,
            bytes_processed: 0,
            errors_count: 0,
        };

        let (tx, rx) = mpsc::unbounded_channel();

        {
            let mut progress_map = self.progress_info.write().await;
            progress_map.insert(stream_id, info);
        }

        {
            let mut senders_map = self.event_senders.write().await;
            senders_map.insert(stream_id, tx.clone());
        }

        {
            let mut samples_map = self.rate_samples.write().await;
            samples_map.insert(stream_id, Vec::new());
        }

        // Send started event
        let event = ProgressEvent {
            stream_id,
            timestamp: chrono::Utc::now(),
            event_type: ProgressEventType::Started,
            data: serde_json::json!({}),
        };

        let _ = tx.send(event);

        Ok(rx)
    }

    /// Update progress for a stream
    pub async fn update_progress(
        &self,
        stream_id: Uuid,
        processed: usize,
        total: Option<usize>,
    ) -> StreamingResult<()> {
        let now = Instant::now();
        let mut updated_info = None;

        {
            let mut progress_map = self.progress_info.write().await;
            if let Some(info) = progress_map.get_mut(&stream_id) {
                let old_processed = info.processed_items;
                info.processed_items = processed;
                info.total_items = total;
                info.last_update = now;

                // Calculate current rate
                let time_diff = now.duration_since(info.start_time).as_secs_f64();
                if time_diff > 0.0 {
                    info.current_rate = processed as f64 / time_diff;
                }

                // Update rate samples for average calculation
                {
                    let mut samples_map = self.rate_samples.write().await;
                    if let Some(samples) = samples_map.get_mut(&stream_id) {
                        samples.push((now, processed));

                        // Keep only samples within the window
                        let cutoff = now - self.config.rate_calculation_window;
                        samples.retain(|(time, _)| *time > cutoff);

                        // Calculate average rate
                        // Safety: We've already checked that samples.len() >= 2, so first() and last()
                        // are guaranteed to return Some. Using pattern matching for clarity.
                        if samples.len() >= 2 {
                            if let (Some(first), Some(last)) = (samples.first(), samples.last()) {
                                let time_span = last.0.duration_since(first.0).as_secs_f64();
                                if time_span > 0.0 {
                                    info.average_rate = (last.1 - first.1) as f64 / time_span;
                                }
                            }
                        }
                    }
                }

                // Estimate completion time
                if self.config.enable_estimation {
                    if let Some(total) = total {
                        if info.average_rate > 0.0 && processed < total {
                            let remaining = total - processed;
                            let remaining_seconds = remaining as f64 / info.average_rate;
                            info.estimated_completion =
                                Some(Duration::from_secs_f64(remaining_seconds));
                        }
                    }
                }

                updated_info = Some(info.clone());

                // Send progress event if significant change
                if processed > old_processed {
                    let event = ProgressEvent {
                        stream_id,
                        timestamp: chrono::Utc::now(),
                        event_type: ProgressEventType::ItemProcessed,
                        data: serde_json::json!({
                            "processed": processed,
                            "total": total,
                            "rate": info.current_rate
                        }),
                    };

                    if let Some(sender) = self.event_senders.read().await.get(&stream_id) {
                        let _ = sender.send(event);
                    }
                }
            }
        }

        if updated_info.is_some() {
            Ok(())
        } else {
            Err(StreamingError::StreamNotFound(stream_id))
        }
    }

    /// Set the stage for a stream
    pub async fn set_stage(&self, stream_id: Uuid, stage: ProgressStage) -> StreamingResult<()> {
        {
            let mut progress_map = self.progress_info.write().await;
            if let Some(info) = progress_map.get_mut(&stream_id) {
                info.stage = stage.clone();
            } else {
                return Err(StreamingError::StreamNotFound(stream_id));
            }
        }

        // Send stage change event
        let event = ProgressEvent {
            stream_id,
            timestamp: chrono::Utc::now(),
            event_type: ProgressEventType::StageChanged,
            data: serde_json::to_value(&stage).unwrap_or_default(),
        };

        if let Some(sender) = self.event_senders.read().await.get(&stream_id) {
            let _ = sender.send(event);
        }

        Ok(())
    }

    /// Add bytes processed
    pub async fn add_bytes_processed(&self, stream_id: Uuid, bytes: u64) -> StreamingResult<()> {
        let mut progress_map = self.progress_info.write().await;
        if let Some(info) = progress_map.get_mut(&stream_id) {
            info.bytes_processed += bytes;
            Ok(())
        } else {
            Err(StreamingError::StreamNotFound(stream_id))
        }
    }

    /// Increment error count
    pub async fn increment_errors(&self, stream_id: Uuid) -> StreamingResult<()> {
        {
            let mut progress_map = self.progress_info.write().await;
            if let Some(info) = progress_map.get_mut(&stream_id) {
                info.errors_count += 1;
            } else {
                return Err(StreamingError::StreamNotFound(stream_id));
            }
        }

        // Send error event
        let event = ProgressEvent {
            stream_id,
            timestamp: chrono::Utc::now(),
            event_type: ProgressEventType::ErrorOccurred,
            data: serde_json::json!({}),
        };

        if let Some(sender) = self.event_senders.read().await.get(&stream_id) {
            let _ = sender.send(event);
        }

        Ok(())
    }

    /// Get current progress information
    pub async fn get_progress(&self, stream_id: &Uuid) -> Option<ProgressInfo> {
        let progress_map = self.progress_info.read().await;
        progress_map.get(stream_id).cloned()
    }

    /// Get all active streams
    pub async fn get_all_progress(&self) -> HashMap<Uuid, ProgressInfo> {
        let progress_map = self.progress_info.read().await;
        progress_map.clone()
    }

    /// Complete tracking for a stream
    pub async fn complete_tracking(&self, stream_id: Uuid) -> StreamingResult<()> {
        {
            let mut progress_map = self.progress_info.write().await;
            if let Some(info) = progress_map.get_mut(&stream_id) {
                info.stage = ProgressStage::Completed;
            }
        }

        // Send completion event
        let event = ProgressEvent {
            stream_id,
            timestamp: chrono::Utc::now(),
            event_type: ProgressEventType::Completed,
            data: serde_json::json!({}),
        };

        if let Some(sender) = self.event_senders.read().await.get(&stream_id) {
            let _ = sender.send(event);
        }

        // Clean up
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(60)).await;
            // Cleanup would happen here in a real implementation
        });

        Ok(())
    }

    /// Fail tracking for a stream
    pub async fn fail_tracking(&self, stream_id: Uuid, error: String) -> StreamingResult<()> {
        {
            let mut progress_map = self.progress_info.write().await;
            if let Some(info) = progress_map.get_mut(&stream_id) {
                info.stage = ProgressStage::Failed(error.clone());
            }
        }

        // Send failure event
        let event = ProgressEvent {
            stream_id,
            timestamp: chrono::Utc::now(),
            event_type: ProgressEventType::Failed,
            data: serde_json::json!({ "error": error }),
        };

        if let Some(sender) = self.event_senders.read().await.get(&stream_id) {
            let _ = sender.send(event);
        }

        Ok(())
    }

    /// Remove tracking for a stream
    pub async fn remove_tracking(&self, stream_id: Uuid) {
        {
            let mut progress_map = self.progress_info.write().await;
            progress_map.remove(&stream_id);
        }

        {
            let mut senders_map = self.event_senders.write().await;
            senders_map.remove(&stream_id);
        }

        {
            let mut samples_map = self.rate_samples.write().await;
            samples_map.remove(&stream_id);
        }
    }
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_progress_tracker_creation() {
        let tracker = ProgressTracker::new();
        let all_progress = tracker.get_all_progress().await;
        assert!(all_progress.is_empty());
    }

    #[tokio::test]
    async fn test_start_and_update_tracking() {
        let tracker = ProgressTracker::new();
        let stream_id = Uuid::new_v4();
        // Event receiver not monitored in this test - we only test progress state updates
        let _rx = tracker.start_tracking(stream_id).await.unwrap();
        tracker
            .update_progress(stream_id, 50, Some(100))
            .await
            .unwrap();

        let progress = tracker.get_progress(&stream_id).await.unwrap();
        assert_eq!(progress.processed_items, 50);
        assert_eq!(progress.total_items, Some(100));
    }

    #[tokio::test]
    async fn test_stage_changes() {
        let tracker = ProgressTracker::new();
        let stream_id = Uuid::new_v4();
        // Event receiver not monitored - test focuses on stage transitions
        let _rx = tracker.start_tracking(stream_id).await.unwrap();
        tracker
            .set_stage(stream_id, ProgressStage::Extracting)
            .await
            .unwrap();

        let progress = tracker.get_progress(&stream_id).await.unwrap();
        assert!(matches!(progress.stage, ProgressStage::Extracting));
    }

    #[tokio::test]
    async fn test_rate_calculation() {
        let config = ProgressConfig {
            update_interval: Duration::from_millis(100),
            rate_calculation_window: Duration::from_secs(1),
            enable_estimation: true,
            max_event_history: 100,
        };

        let tracker = ProgressTracker::with_config(config);
        let stream_id = Uuid::new_v4();
        // Event receiver not monitored - test validates rate calculation logic
        let _rx = tracker.start_tracking(stream_id).await.unwrap();
        // Simulate processing over time
        tracker
            .update_progress(stream_id, 10, Some(100))
            .await
            .unwrap();
        sleep(Duration::from_millis(100)).await;
        tracker
            .update_progress(stream_id, 20, Some(100))
            .await
            .unwrap();
        sleep(Duration::from_millis(100)).await;
        tracker
            .update_progress(stream_id, 30, Some(100))
            .await
            .unwrap();

        let progress = tracker.get_progress(&stream_id).await.unwrap();
        assert!(progress.current_rate > 0.0);
        assert!(progress.estimated_completion.is_some());
    }

    #[tokio::test]
    async fn test_complete_tracking() {
        let tracker = ProgressTracker::new();
        let stream_id = Uuid::new_v4();
        // Event receiver not monitored - test validates completion state
        let _rx = tracker.start_tracking(stream_id).await.unwrap();
        tracker.complete_tracking(stream_id).await.unwrap();

        let progress = tracker.get_progress(&stream_id).await.unwrap();
        assert!(matches!(progress.stage, ProgressStage::Completed));
    }
}
