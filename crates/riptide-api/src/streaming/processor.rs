//! Stream processing logic and utilities.
//!
//! This module contains common processing logic shared across different
#![allow(dead_code)]
//! streaming protocols (NDJSON, SSE, WebSocket) including result conversion,
//! progress tracking, and performance monitoring.

use super::error::StreamingResult;
use crate::models::*;
use crate::pipeline::{PipelineOrchestrator, PipelineResult};
use serde::Serialize;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Stream processor for handling URL processing tasks
pub struct StreamProcessor {
    /// Pipeline orchestrator for URL processing
    pub pipeline: PipelineOrchestrator,
    /// Start time for performance tracking
    pub start_time: Instant,
    /// Request identifier for logging
    pub request_id: String,
    /// Statistics tracking
    pub stats: ProcessingStats,
}

/// Processing statistics for monitoring
#[derive(Debug, Default, Clone)]
pub struct ProcessingStats {
    pub total_urls: usize,
    pub completed_count: usize,
    pub error_count: usize,
    pub cache_hits: usize,
    pub total_processing_time_ms: u64,
    pub fastest_processing_ms: u64,
    pub slowest_processing_ms: u64,
    pub average_processing_ms: f64,
}

impl ProcessingStats {
    /// Update statistics with a new result
    pub fn update(&mut self, processing_time_ms: u64, from_cache: bool, success: bool) {
        if success {
            self.completed_count += 1;
        } else {
            self.error_count += 1;
        }

        if from_cache {
            self.cache_hits += 1;
        }

        self.total_processing_time_ms += processing_time_ms;

        // Update min/max processing times (only for non-cached results)
        if !from_cache && processing_time_ms > 0 {
            if self.fastest_processing_ms == 0 || processing_time_ms < self.fastest_processing_ms {
                self.fastest_processing_ms = processing_time_ms;
            }
            if processing_time_ms > self.slowest_processing_ms {
                self.slowest_processing_ms = processing_time_ms;
            }
        }

        // Update running average
        let total_processed = self.completed_count + self.error_count;
        if total_processed > 0 {
            self.average_processing_ms =
                self.total_processing_time_ms as f64 / total_processed as f64;
        }
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.completed_count + self.error_count;
        if total > 0 {
            self.completed_count as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Calculate cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        if self.total_urls > 0 {
            self.cache_hits as f64 / self.total_urls as f64
        } else {
            0.0
        }
    }

    /// Calculate processing throughput (items per second)
    pub fn throughput(&self, elapsed_duration: Duration) -> f64 {
        let elapsed_seconds = elapsed_duration.as_secs_f64();
        if elapsed_seconds > 0.0 {
            (self.completed_count + self.error_count) as f64 / elapsed_seconds
        } else {
            0.0
        }
    }

    /// Get current progress percentage
    pub fn progress_percentage(&self) -> f64 {
        if self.total_urls > 0 {
            (self.completed_count + self.error_count) as f64 / self.total_urls as f64 * 100.0
        } else {
            0.0
        }
    }
}

impl StreamProcessor {
    /// Create a new stream processor
    pub fn new(pipeline: PipelineOrchestrator, request_id: String, total_urls: usize) -> Self {
        Self {
            pipeline,
            start_time: Instant::now(),
            request_id,
            stats: ProcessingStats {
                total_urls,
                ..Default::default()
            },
        }
    }

    /// Start the stream processor with initialization tasks
    pub async fn start(&mut self) -> StreamingResult<()> {
        info!(
            request_id = %self.request_id,
            total_urls = self.stats.total_urls,
            "Starting stream processor"
        );

        self.start_time = Instant::now();

        // Reset stats for a fresh start
        self.stats = ProcessingStats {
            total_urls: self.stats.total_urls,
            ..Default::default()
        };

        // Log initialization
        info!(
            request_id = %self.request_id,
            "Stream processor started successfully"
        );

        Ok(())
    }

    /// Flush any pending operations and ensure consistency
    pub async fn flush(&mut self) -> StreamingResult<()> {
        debug!(
            request_id = %self.request_id,
            "Flushing stream processor operations"
        );

        // Ensure all stats are up-to-date
        let processing_time = self.start_time.elapsed();

        // Update final stats
        let completed_total = self.stats.completed_count + self.stats.error_count;
        if completed_total > 0 && self.stats.total_processing_time_ms == 0 {
            self.stats.total_processing_time_ms = processing_time.as_millis() as u64;
        }

        info!(
            request_id = %self.request_id,
            completed = self.stats.completed_count,
            failed = self.stats.error_count,
            processing_time_ms = processing_time.as_millis(),
            "Stream processor flush completed"
        );

        Ok(())
    }

    /// Close the stream processor and perform cleanup
    pub async fn close(&mut self) -> StreamingResult<()> {
        info!(
            request_id = %self.request_id,
            "Closing stream processor"
        );

        // Flush any remaining operations
        self.flush().await?;

        // Log final statistics
        self.log_completion();

        info!(
            request_id = %self.request_id,
            total_duration_ms = self.start_time.elapsed().as_millis(),
            "Stream processor closed successfully"
        );

        Ok(())
    }

    /// Process URLs concurrently and return a receiver for results
    pub async fn process_urls_concurrent(
        &self,
        urls: Vec<String>,
    ) -> StreamingResult<mpsc::Receiver<ProcessedResult>> {
        let (result_tx, result_rx) = mpsc::channel(urls.len());

        debug!(
            request_id = %self.request_id,
            url_count = urls.len(),
            "Starting concurrent URL processing"
        );

        // Spawn individual URL processing tasks
        for (index, url) in urls.iter().enumerate() {
            let pipeline_clone = self.pipeline.clone();
            let url_clone = url.clone();
            let result_tx_clone = result_tx.clone();
            let request_id = self.request_id.clone();

            tokio::spawn(async move {
                let processing_start = Instant::now();
                let result = pipeline_clone
                    .execute_single(&url_clone)
                    .await
                    .map_err(|e| e.into());
                let processing_time = processing_start.elapsed();

                let processed_result = ProcessedResult {
                    index,
                    url: url_clone,
                    result,
                    processing_time_ms: processing_time.as_millis() as u64,
                };

                if let Err(e) = result_tx_clone.send(processed_result).await {
                    warn!(
                        request_id = %request_id,
                        error = %e,
                        "Failed to send processed result"
                    );
                }
            });
        }

        drop(result_tx); // Close the sender
        Ok(result_rx)
    }

    /// Convert pipeline result to crawl result
    pub fn convert_to_crawl_result(&mut self, processed_result: ProcessedResult) -> CrawlResult {
        match processed_result.result {
            Ok(pipeline_result) => {
                let from_cache = pipeline_result.from_cache;
                let processing_time = if from_cache {
                    0
                } else {
                    processed_result.processing_time_ms
                };

                self.stats.update(processing_time, from_cache, true);

                CrawlResult {
                    url: processed_result.url,
                    status: pipeline_result.http_status,
                    from_cache: pipeline_result.from_cache,
                    gate_decision: pipeline_result.gate_decision,
                    quality_score: pipeline_result.quality_score,
                    processing_time_ms: pipeline_result.processing_time_ms,
                    document: Some(pipeline_result.document),
                    error: None,
                    cache_key: pipeline_result.cache_key,
                }
            }
            Err(e) => {
                self.stats
                    .update(processed_result.processing_time_ms, false, false);

                warn!(
                    request_id = %self.request_id,
                    url = %processed_result.url,
                    error = %e,
                    "Failed to process URL"
                );

                CrawlResult {
                    url: processed_result.url,
                    status: 0,
                    from_cache: false,
                    gate_decision: "failed".to_string(),
                    quality_score: 0.0,
                    processing_time_ms: processed_result.processing_time_ms,
                    document: None,
                    error: Some(ErrorInfo {
                        error_type: "processing_error".to_string(),
                        message: format!("Failed to process URL: {}", e),
                        retryable: true,
                    }),
                    cache_key: "".to_string(),
                }
            }
        }
    }

    /// Create progress information
    pub fn create_progress(&self) -> StreamProgress {
        StreamProgress {
            completed: self.stats.completed_count + self.stats.error_count,
            total: self.stats.total_urls,
            success_rate: self.stats.success_rate(),
            cache_hit_rate: self.stats.cache_hit_rate(),
            processing_time_ms: self.start_time.elapsed().as_millis() as u64,
            throughput: self.stats.throughput(self.start_time.elapsed()),
            estimated_completion: self.estimate_completion(),
        }
    }

    /// Create operation progress for detailed tracking
    pub fn create_operation_progress(&self, current_item: Option<String>) -> OperationProgress {
        OperationProgress {
            operation_id: self.request_id.clone(),
            operation_type: "batch_crawl".to_string(),
            started_at: (chrono::Utc::now()
                - chrono::Duration::milliseconds(self.start_time.elapsed().as_millis() as i64))
            .to_rfc3339(),
            current_phase: "processing".to_string(),
            progress_percentage: self.stats.progress_percentage(),
            items_completed: self.stats.completed_count + self.stats.error_count,
            items_total: self.stats.total_urls,
            estimated_completion: self.estimate_completion(),
            current_item,
        }
    }

    /// Create final summary
    pub fn create_summary(&self) -> StreamSummary {
        StreamSummary {
            total_urls: self.stats.total_urls,
            successful: self.stats.completed_count,
            failed: self.stats.error_count,
            from_cache: self.stats.cache_hits,
            total_processing_time_ms: self.start_time.elapsed().as_millis() as u64,
            cache_hit_rate: self.stats.cache_hit_rate(),
            success_rate: self.stats.success_rate(),
            average_processing_time_ms: self.stats.average_processing_ms,
            throughput_per_second: self.stats.throughput(self.start_time.elapsed()),
            fastest_processing_ms: self.stats.fastest_processing_ms,
            slowest_processing_ms: self.stats.slowest_processing_ms,
        }
    }

    /// Estimate completion time
    pub fn estimate_completion(&self) -> Option<String> {
        let completed = self.stats.completed_count + self.stats.error_count;
        if completed == 0 || self.stats.total_urls == 0 {
            return None;
        }

        let elapsed = self.start_time.elapsed();
        let avg_time_per_item = elapsed.as_secs_f64() / completed as f64;
        let remaining_items = self.stats.total_urls.saturating_sub(completed);
        let estimated_remaining_secs = avg_time_per_item * remaining_items as f64;

        let completion_time =
            chrono::Utc::now() + chrono::Duration::seconds(estimated_remaining_secs as i64);
        Some(completion_time.to_rfc3339())
    }

    /// Check if processing should send progress update
    pub fn should_send_progress_update(&self, update_interval: usize) -> bool {
        let completed = self.stats.completed_count + self.stats.error_count;
        self.stats.total_urls > 10 && completed.is_multiple_of(update_interval)
    }

    /// Get current statistics
    pub fn stats(&self) -> &ProcessingStats {
        &self.stats
    }

    /// Log final processing summary
    pub fn log_completion(&self) {
        info!(
            request_id = %self.request_id,
            total_urls = self.stats.total_urls,
            successful = self.stats.completed_count,
            failed = self.stats.error_count,
            cache_hits = self.stats.cache_hits,
            total_time_ms = self.start_time.elapsed().as_millis(),
            success_rate = self.stats.success_rate(),
            cache_hit_rate = self.stats.cache_hit_rate(),
            average_processing_ms = self.stats.average_processing_ms,
            throughput = self.stats.throughput(self.start_time.elapsed()),
            "Stream processing completed"
        );
    }
}

/// Result of URL processing with metadata
#[derive(Debug)]
pub struct ProcessedResult {
    pub index: usize,
    pub url: String,
    pub result: Result<PipelineResult, anyhow::Error>,
    pub processing_time_ms: u64,
}

/// Enhanced progress information for streaming
#[derive(Debug, Clone, Serialize)]
pub struct StreamProgress {
    pub completed: usize,
    pub total: usize,
    pub success_rate: f64,
    pub cache_hit_rate: f64,
    pub processing_time_ms: u64,
    pub throughput: f64,
    pub estimated_completion: Option<String>,
}

/// Enhanced summary for streaming operations
#[derive(Debug, Clone, Serialize)]
pub struct StreamSummary {
    pub total_urls: usize,
    pub successful: usize,
    pub failed: usize,
    pub from_cache: usize,
    pub total_processing_time_ms: u64,
    pub cache_hit_rate: f64,
    pub success_rate: f64,
    pub average_processing_time_ms: f64,
    pub throughput_per_second: f64,
    pub fastest_processing_ms: u64,
    pub slowest_processing_ms: u64,
}

/// Progress tracking for long-running operations
#[derive(Debug, Clone, Serialize)]
pub struct OperationProgress {
    pub operation_id: String,
    pub operation_type: String,
    pub started_at: String,
    pub current_phase: String,
    pub progress_percentage: f64,
    pub items_completed: usize,
    pub items_total: usize,
    pub estimated_completion: Option<String>,
    pub current_item: Option<String>,
}

/// Performance monitoring utilities
pub struct PerformanceMonitor {
    checkpoints: Vec<PerformanceCheckpoint>,
    start_time: Instant,
}

#[derive(Debug, Clone)]
pub struct PerformanceCheckpoint {
    pub name: String,
    pub timestamp: Instant,
    pub items_processed: usize,
    pub memory_usage_bytes: Option<usize>,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            checkpoints: Vec::new(),
            start_time: Instant::now(),
        }
    }

    /// Add a performance checkpoint
    pub fn checkpoint(&mut self, name: impl Into<String>, items_processed: usize) {
        self.checkpoints.push(PerformanceCheckpoint {
            name: name.into(),
            timestamp: Instant::now(),
            items_processed,
            memory_usage_bytes: None, // Could be implemented with system calls
        });
    }

    /// Get performance analysis
    pub fn analyze(&self) -> PerformanceAnalysis {
        let total_duration = self.start_time.elapsed();
        let total_items = self
            .checkpoints
            .last()
            .map(|c| c.items_processed)
            .unwrap_or(0);

        let mut phase_durations = Vec::new();
        for i in 1..self.checkpoints.len() {
            let prev = &self.checkpoints[i - 1];
            let curr = &self.checkpoints[i];
            let duration = curr.timestamp.duration_since(prev.timestamp);
            let items_in_phase = curr.items_processed - prev.items_processed;

            phase_durations.push(PhaseDuration {
                phase_name: curr.name.clone(),
                duration_ms: duration.as_millis() as u64,
                items_processed: items_in_phase,
                throughput: if duration.as_secs_f64() > 0.0 {
                    items_in_phase as f64 / duration.as_secs_f64()
                } else {
                    0.0
                },
            });
        }

        PerformanceAnalysis {
            total_duration_ms: total_duration.as_millis() as u64,
            total_items,
            overall_throughput: if total_duration.as_secs_f64() > 0.0 {
                total_items as f64 / total_duration.as_secs_f64()
            } else {
                0.0
            },
            phase_durations,
            checkpoints: self.checkpoints.clone(),
        }
    }
}

#[derive(Debug)]
pub struct PerformanceAnalysis {
    pub total_duration_ms: u64,
    pub total_items: usize,
    pub overall_throughput: f64,
    pub phase_durations: Vec<PhaseDuration>,
    pub checkpoints: Vec<PerformanceCheckpoint>,
}

#[derive(Debug)]
pub struct PhaseDuration {
    pub phase_name: String,
    pub duration_ms: u64,
    pub items_processed: usize,
    pub throughput: f64,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_processing_stats_update() {
        let mut stats = ProcessingStats {
            total_urls: 10,
            ..Default::default()
        };

        // Test successful processing
        stats.update(100, false, true);
        assert_eq!(stats.completed_count, 1);
        assert_eq!(stats.success_rate(), 1.0);
        assert_eq!(stats.fastest_processing_ms, 100);

        // Test cached result
        stats.update(0, true, true);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_hit_rate(), 0.1);

        // Test failed processing
        stats.update(200, false, false);
        assert_eq!(stats.error_count, 1);
        assert!(stats.success_rate() < 1.0);
    }

    #[tokio::test]
    #[ignore = "Requires Redis connection"]
    async fn test_stream_processor_creation() {
        use crate::models::CrawlOptions;
        use crate::tests::test_helpers::AppStateBuilder;

        // Use test builder to construct AppState
        let app = AppStateBuilder::new()
            .build()
            .await
            .expect("Failed to create AppState");
        let pipeline = PipelineOrchestrator::new(app, CrawlOptions::default());
        let processor = StreamProcessor::new(pipeline, "test-123".to_string(), 5);

        assert_eq!(processor.request_id, "test-123");
        assert_eq!(processor.stats.total_urls, 5);
    }

    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();

        monitor.checkpoint("start", 0);
        std::thread::sleep(std::time::Duration::from_millis(10));
        monitor.checkpoint("middle", 50);
        std::thread::sleep(std::time::Duration::from_millis(10));
        monitor.checkpoint("end", 100);

        let analysis = monitor.analyze();
        assert_eq!(analysis.total_items, 100);
        assert!(analysis.total_duration_ms >= 20);
        assert_eq!(analysis.phase_durations.len(), 2);
    }

    #[test]
    fn test_progress_calculation() {
        let stats = ProcessingStats {
            total_urls: 100,
            completed_count: 80,
            error_count: 10,
            ..Default::default()
        };

        assert_eq!(stats.progress_percentage(), 90.0);
        assert_eq!(stats.success_rate(), 8.0 / 9.0);
    }

    #[test]
    fn test_throughput_calculation() {
        let stats = ProcessingStats {
            completed_count: 60,
            error_count: 10,
            ..Default::default()
        };

        let throughput = stats.throughput(Duration::from_secs(60));
        assert_eq!(throughput, 70.0 / 60.0);
    }
}
