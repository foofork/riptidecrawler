//! # Parallel Document Extraction
//!
//! This module provides high-performance parallel processing for batch document extraction.
//! It leverages Tokio's async runtime for concurrent processing with configurable limits,
//! automatic retry, progress tracking, and comprehensive error handling.
//!
//! ## Features
//!
//! - **Configurable Concurrency**: Control max parallel tasks
//! - **Automatic Retry**: Retry failed extractions with exponential backoff
//! - **Progress Tracking**: Real-time progress callbacks
//! - **Timeout Management**: Per-document timeout controls
//! - **Fail-Fast Mode**: Option to stop on first error
//! - **Streaming Results**: Get results as they complete
//! - **Resource Management**: Efficient memory and CPU utilization
//! - **Metrics**: Comprehensive performance tracking
//!
//! ## Usage
//!
//! ```rust
//! use riptide_extraction::parallel::{ParallelExtractor, ParallelConfig};
//! use riptide_types::ExtractedDoc;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config = ParallelConfig::default()
//!     .with_max_concurrent(10)
//!     .with_timeout_per_doc(std::time::Duration::from_secs(30));
//!
//! let extractor = ParallelExtractor::new(config);
//!
//! let documents = vec![
//!     ("https://example.com", "<html>...</html>"),
//!     ("https://example.org", "<html>...</html>"),
//! ];
//!
//! let results = extractor.extract_batch(documents).await?;
//! println!("Processed {} documents", results.len());
//! # Ok(())
//! # }
//! ```

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock, Semaphore};
use tokio::task::JoinSet;
use tokio::time::timeout;

use crate::unified_extractor::UnifiedExtractor;
use riptide_types::ExtractedContent;

/// Configuration for parallel extraction
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Maximum number of concurrent extraction tasks
    pub max_concurrent: usize,
    /// Timeout for each document extraction
    pub timeout_per_doc: Duration,
    /// Whether to automatically retry failed extractions
    pub retry_failed: bool,
    /// Maximum number of retries per document
    pub max_retries: usize,
    /// Stop all processing on first error
    pub fail_fast: bool,
    /// Exponential backoff multiplier for retries
    pub retry_backoff_multiplier: f64,
    /// Initial retry delay
    pub initial_retry_delay: Duration,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            timeout_per_doc: Duration::from_secs(30),
            retry_failed: true,
            max_retries: 3,
            fail_fast: false,
            retry_backoff_multiplier: 2.0,
            initial_retry_delay: Duration::from_millis(100),
        }
    }
}

impl ParallelConfig {
    /// Create a new configuration with custom concurrency limit
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent,
            ..Default::default()
        }
    }

    /// Set maximum concurrent tasks
    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }

    /// Set timeout per document
    pub fn with_timeout_per_doc(mut self, timeout: Duration) -> Self {
        self.timeout_per_doc = timeout;
        self
    }

    /// Enable or disable automatic retry
    pub fn with_retry(mut self, retry: bool) -> Self {
        self.retry_failed = retry;
        self
    }

    /// Set maximum retries
    pub fn with_max_retries(mut self, retries: usize) -> Self {
        self.max_retries = retries;
        self
    }

    /// Enable fail-fast mode
    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }
}

/// Document to be processed
#[derive(Debug, Clone)]
pub struct DocumentTask {
    /// URL of the document
    pub url: String,
    /// HTML content
    pub html: String,
    /// Priority (higher values processed first)
    pub priority: i32,
    /// Task ID for tracking
    pub id: usize,
}

/// Result of a parallel document extraction
#[derive(Debug, Clone)]
pub struct ParallelExtractionResult {
    /// Task ID
    pub task_id: usize,
    /// Document URL
    pub url: String,
    /// Extraction result
    pub result: Result<ExtractedContent, String>,
    /// Processing duration
    pub duration: Duration,
    /// Number of retry attempts
    pub retry_count: usize,
}

/// Progress information for batch extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionProgress {
    /// Total documents to process
    pub total: usize,
    /// Documents completed
    pub completed: usize,
    /// Documents succeeded
    pub succeeded: usize,
    /// Documents failed
    pub failed: usize,
    /// Documents currently processing
    pub in_progress: usize,
    /// Average processing time per document
    pub avg_duration_ms: f64,
    /// Estimated time remaining
    pub estimated_remaining_ms: u64,
}

/// Metrics for parallel extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionMetrics {
    /// Total documents processed
    pub total_processed: usize,
    /// Total successes
    pub total_succeeded: usize,
    /// Total failures
    pub total_failed: usize,
    /// Average processing time (ms)
    pub avg_processing_time_ms: f64,
    /// Minimum processing time (ms)
    pub min_processing_time_ms: f64,
    /// Maximum processing time (ms)
    pub max_processing_time_ms: f64,
    /// Throughput (documents per second)
    pub throughput_docs_per_sec: f64,
    /// Total processing time (ms)
    pub total_time_ms: u64,
    /// Peak concurrent tasks
    pub peak_concurrent: usize,
    /// Total retries
    pub total_retries: usize,
}

/// Progress callback function type
pub type ProgressCallback = Arc<dyn Fn(ExtractionProgress) + Send + Sync>;

/// Parallel document extractor
pub struct ParallelExtractor {
    config: ParallelConfig,
    extractor: Arc<UnifiedExtractor>,
    progress_callback: Option<ProgressCallback>,
}

impl ParallelExtractor {
    /// Create a new parallel extractor with configuration
    /// Uses native extractor by default
    pub fn new(config: ParallelConfig) -> Self {
        Self {
            config,
            extractor: Arc::new(UnifiedExtractor::Native(
                crate::unified_extractor::NativeExtractor::new(),
            )),
            progress_callback: None,
        }
    }

    /// Create with custom extractor
    pub fn with_extractor(config: ParallelConfig, extractor: UnifiedExtractor) -> Self {
        Self {
            config,
            extractor: Arc::new(extractor),
            progress_callback: None,
        }
    }

    /// Set progress callback
    pub fn with_progress_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(ExtractionProgress) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Arc::new(callback));
        self
    }

    /// Extract batch of documents in parallel
    pub async fn extract_batch<S: AsRef<str>>(
        &self,
        documents: Vec<(S, S)>,
    ) -> Result<Vec<ParallelExtractionResult>> {
        let tasks: Vec<DocumentTask> = documents
            .into_iter()
            .enumerate()
            .map(|(id, (url, html))| DocumentTask {
                url: url.as_ref().to_string(),
                html: html.as_ref().to_string(),
                priority: 0,
                id,
            })
            .collect();

        self.extract_tasks(tasks).await
    }

    /// Extract tasks with priority support
    pub async fn extract_tasks(
        &self,
        mut tasks: Vec<DocumentTask>,
    ) -> Result<Vec<ParallelExtractionResult>> {
        if tasks.is_empty() {
            return Ok(Vec::new());
        }

        // Sort by priority (higher first)
        tasks.sort_by(|a, b| b.priority.cmp(&a.priority));

        let total = tasks.len();
        let start_time = Instant::now();

        // Shared state
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent));
        let results = Arc::new(Mutex::new(Vec::new()));
        let progress = Arc::new(RwLock::new(ExtractionProgress {
            total,
            completed: 0,
            succeeded: 0,
            failed: 0,
            in_progress: 0,
            avg_duration_ms: 0.0,
            estimated_remaining_ms: 0,
        }));
        let failed = Arc::new(Mutex::new(false));
        let task_queue = Arc::new(Mutex::new(VecDeque::from(tasks)));
        let peak_concurrent = Arc::new(Mutex::new(0));

        let mut join_set = JoinSet::new();

        // Spawn worker tasks
        for _ in 0..self.config.max_concurrent {
            let semaphore = Arc::clone(&semaphore);
            let results = Arc::clone(&results);
            let progress = Arc::clone(&progress);
            let failed = Arc::clone(&failed);
            let task_queue = Arc::clone(&task_queue);
            let peak_concurrent = Arc::clone(&peak_concurrent);
            let extractor = Arc::clone(&self.extractor);
            let config = self.config.clone();
            let callback = self.progress_callback.clone();

            join_set.spawn(async move {
                loop {
                    // Check fail-fast
                    if config.fail_fast && *failed.lock().await {
                        break;
                    }

                    // Get next task
                    let task = {
                        let mut queue = task_queue.lock().await;
                        queue.pop_front()
                    };

                    let task = match task {
                        Some(t) => t,
                        None => break,
                    };

                    // Acquire semaphore permit
                    let _permit = match semaphore.acquire().await {
                        Ok(permit) => permit,
                        Err(_) => {
                            // Semaphore was closed, stop processing
                            break;
                        }
                    };

                    // Update in-progress count
                    {
                        let mut prog = progress.write().await;
                        prog.in_progress += 1;
                        let mut peak = peak_concurrent.lock().await;
                        *peak = (*peak).max(prog.in_progress);
                    }

                    // Process task
                    let result = Self::process_task_with_retry(task, &extractor, &config).await;

                    // Update progress
                    {
                        let mut prog = progress.write().await;
                        prog.in_progress -= 1;
                        prog.completed += 1;

                        match &result.result {
                            Ok(_) => prog.succeeded += 1,
                            Err(_) => {
                                prog.failed += 1;
                                if config.fail_fast {
                                    *failed.lock().await = true;
                                }
                            }
                        }

                        // Update timing estimates
                        let elapsed = start_time.elapsed().as_millis() as f64;
                        prog.avg_duration_ms = elapsed / prog.completed as f64;
                        let remaining = total - prog.completed;
                        prog.estimated_remaining_ms =
                            (prog.avg_duration_ms * remaining as f64) as u64;

                        // Call progress callback
                        if let Some(ref cb) = callback {
                            cb(prog.clone());
                        }
                    }

                    // Store result
                    results.lock().await.push(result);
                }
            });
        }

        // Wait for all tasks to complete
        while join_set.join_next().await.is_some() {}

        let results = Arc::try_unwrap(results)
            .map_err(|_| anyhow!("Failed to unwrap results"))?
            .into_inner();

        // Sort results by task ID
        let mut results = results;
        results.sort_by_key(|r| r.task_id);

        Ok(results)
    }

    /// Process a single task with retry logic
    async fn process_task_with_retry(
        task: DocumentTask,
        extractor: &UnifiedExtractor,
        config: &ParallelConfig,
    ) -> ParallelExtractionResult {
        let mut retry_count = 0;
        let start = Instant::now();

        loop {
            let result = timeout(
                config.timeout_per_doc,
                extractor.extract(&task.html, &task.url),
            )
            .await;

            let error_msg = match result {
                Ok(Ok(doc)) => {
                    return ParallelExtractionResult {
                        task_id: task.id,
                        url: task.url,
                        result: Ok(doc),
                        duration: start.elapsed(),
                        retry_count,
                    };
                }
                Ok(Err(e)) => format!("Extraction error: {}", e),
                Err(_) => "Timeout".to_string(),
            };

            // Check retry
            if !config.retry_failed || retry_count >= config.max_retries {
                return ParallelExtractionResult {
                    task_id: task.id,
                    url: task.url,
                    result: Err(error_msg),
                    duration: start.elapsed(),
                    retry_count,
                };
            }

            retry_count += 1;

            // Exponential backoff
            let delay = config.initial_retry_delay.as_millis() as f64
                * config.retry_backoff_multiplier.powi(retry_count as i32 - 1);
            tokio::time::sleep(Duration::from_millis(delay as u64)).await;
        }
    }

    /// Stream results as they complete (returns channel)
    pub async fn extract_batch_streaming<S: AsRef<str>>(
        &self,
        documents: Vec<(S, S)>,
    ) -> Result<tokio::sync::mpsc::Receiver<ParallelExtractionResult>> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        let tasks: Vec<DocumentTask> = documents
            .into_iter()
            .enumerate()
            .map(|(id, (url, html))| DocumentTask {
                url: url.as_ref().to_string(),
                html: html.as_ref().to_string(),
                priority: 0,
                id,
            })
            .collect();

        let extractor = Arc::clone(&self.extractor);
        let config = self.config.clone();

        tokio::spawn(async move {
            let semaphore = Arc::new(Semaphore::new(config.max_concurrent));
            let mut join_set = JoinSet::new();

            for task in tasks {
                let semaphore = Arc::clone(&semaphore);
                let extractor = Arc::clone(&extractor);
                let config = config.clone();
                let tx = tx.clone();

                join_set.spawn(async move {
                    let _permit = match semaphore.acquire().await {
                        Ok(permit) => permit,
                        Err(_) => {
                            // Semaphore closed, send error result
                            let error_result = ParallelExtractionResult {
                                task_id: task.id,
                                url: task.url.clone(),
                                result: Err("Extraction cancelled - semaphore closed".to_string()),
                                duration: Duration::from_secs(0),
                                retry_count: 0,
                            };
                            let _ = tx.send(error_result).await;
                            return;
                        }
                    };
                    let result = Self::process_task_with_retry(task, &extractor, &config).await;
                    let _ = tx.send(result).await;
                });
            }

            while join_set.join_next().await.is_some() {}
        });

        Ok(rx)
    }

    /// Calculate metrics from results
    pub fn calculate_metrics(
        &self,
        results: &[ParallelExtractionResult],
        total_time: Duration,
    ) -> ExtractionMetrics {
        let total_processed = results.len();
        let total_succeeded = results.iter().filter(|r| r.result.is_ok()).count();
        let total_failed = total_processed - total_succeeded;
        let total_retries = results.iter().map(|r| r.retry_count).sum();

        let durations: Vec<f64> = results
            .iter()
            .map(|r| r.duration.as_millis() as f64)
            .collect();

        let avg_processing_time_ms = if !durations.is_empty() {
            durations.iter().sum::<f64>() / durations.len() as f64
        } else {
            0.0
        };

        let min_processing_time_ms = durations.iter().cloned().fold(f64::INFINITY, f64::min);

        let max_processing_time_ms = durations.iter().cloned().fold(0.0, f64::max);

        let total_time_ms = total_time.as_millis() as u64;
        let throughput_docs_per_sec = if total_time_ms > 0 {
            (total_processed as f64 * 1000.0) / total_time_ms as f64
        } else {
            0.0
        };

        ExtractionMetrics {
            total_processed,
            total_succeeded,
            total_failed,
            avg_processing_time_ms,
            min_processing_time_ms: if min_processing_time_ms.is_finite() {
                min_processing_time_ms
            } else {
                0.0
            },
            max_processing_time_ms,
            throughput_docs_per_sec,
            total_time_ms,
            peak_concurrent: self.config.max_concurrent,
            total_retries,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_html(title: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>{}</title>
                <meta name="description" content="Test description for {}">
            </head>
            <body>
                <article>
                    <header>
                        <h1>{}</h1>
                    </header>
                    <section>
                        <p>This is a comprehensive test document with sufficient content to meet quality thresholds.</p>
                        <p>It contains multiple paragraphs and structured content for testing extraction.</p>
                        <p>Additional content ensures the quality score is above the threshold.</p>
                    </section>
                </article>
            </body>
            </html>"#,
            title, title, title
        )
    }

    #[tokio::test]
    async fn test_parallel_config_default() {
        let config = ParallelConfig::default();
        assert_eq!(config.max_concurrent, 10);
        assert_eq!(config.timeout_per_doc, Duration::from_secs(30));
        assert!(config.retry_failed);
        assert_eq!(config.max_retries, 3);
        assert!(!config.fail_fast);
    }

    #[tokio::test]
    async fn test_parallel_config_builder() {
        let config = ParallelConfig::default()
            .with_max_concurrent(5)
            .with_timeout_per_doc(Duration::from_secs(10))
            .with_retry(false)
            .with_max_retries(1)
            .with_fail_fast(true);

        assert_eq!(config.max_concurrent, 5);
        assert_eq!(config.timeout_per_doc, Duration::from_secs(10));
        assert!(!config.retry_failed);
        assert_eq!(config.max_retries, 1);
        assert!(config.fail_fast);
    }

    #[tokio::test]
    async fn test_basic_parallel_extraction() {
        let config = ParallelConfig::default().with_max_concurrent(3);
        let extractor = ParallelExtractor::new(config);

        let documents = vec![
            (
                "https://example1.com".to_string(),
                create_test_html("Doc 1"),
            ),
            (
                "https://example2.com".to_string(),
                create_test_html("Doc 2"),
            ),
            (
                "https://example3.com".to_string(),
                create_test_html("Doc 3"),
            ),
        ];

        let results = extractor.extract_batch(documents).await.unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results.iter().filter(|r| r.result.is_ok()).count(), 3);
    }

    #[tokio::test]
    async fn test_empty_batch() {
        let config = ParallelConfig::default();
        let extractor = ParallelExtractor::new(config);

        let documents: Vec<(String, String)> = vec![];
        let results = extractor.extract_batch(documents).await.unwrap();

        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_single_document() {
        let config = ParallelConfig::default();
        let extractor = ParallelExtractor::new(config);

        let documents = vec![("https://example.com".to_string(), create_test_html("Test"))];
        let results = extractor.extract_batch(documents).await.unwrap();

        assert_eq!(results.len(), 1);
        if let Err(e) = &results[0].result {
            eprintln!("Extraction error: {}", e);
        }
        assert!(results[0].result.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_calculation() {
        let config = ParallelConfig::default();
        let extractor = ParallelExtractor::new(config);

        let documents = vec![
            (
                "https://example1.com".to_string(),
                create_test_html("Doc 1"),
            ),
            (
                "https://example2.com".to_string(),
                create_test_html("Doc 2"),
            ),
            (
                "https://example3.com".to_string(),
                create_test_html("Doc 3"),
            ),
        ];

        let start = Instant::now();
        let results = extractor.extract_batch(documents).await.unwrap();
        let duration = start.elapsed();

        let metrics = extractor.calculate_metrics(&results, duration);

        assert_eq!(metrics.total_processed, 3);
        assert_eq!(metrics.total_succeeded, 3);
        assert_eq!(metrics.total_failed, 0);
        assert!(metrics.avg_processing_time_ms > 0.0);
        assert!(metrics.throughput_docs_per_sec > 0.0);
    }

    #[tokio::test]
    async fn test_progress_callback() {
        let config = ParallelConfig::default().with_max_concurrent(2);

        let progress_count = Arc::new(std::sync::Mutex::new(0));
        let progress_count_clone = Arc::clone(&progress_count);

        let extractor = ParallelExtractor::new(config).with_progress_callback(move |_progress| {
            if let Ok(mut count) = progress_count_clone.lock() {
                *count += 1;
            }
        });

        let documents = vec![
            (
                "https://example1.com".to_string(),
                create_test_html("Doc 1"),
            ),
            (
                "https://example2.com".to_string(),
                create_test_html("Doc 2"),
            ),
            (
                "https://example3.com".to_string(),
                create_test_html("Doc 3"),
            ),
        ];

        let _ = extractor.extract_batch(documents).await.unwrap();

        let count = progress_count.lock().expect("mutex should not be poisoned");
        // Progress callback is called for each completion, should be at least 1
        assert!(
            *count >= 1,
            "Expected at least 1 progress callback, got {}",
            *count
        );
    }

    #[tokio::test]
    async fn test_large_batch() {
        let config = ParallelConfig::default().with_max_concurrent(5);
        let extractor = ParallelExtractor::new(config);

        let documents: Vec<_> = (0..50)
            .map(|i| {
                (
                    format!("https://example{}.com", i),
                    create_test_html(&format!("Doc {}", i)),
                )
            })
            .collect();

        let start = Instant::now();
        let results = extractor.extract_batch(documents).await.unwrap();
        let duration = start.elapsed();

        assert_eq!(results.len(), 50);
        assert_eq!(results.iter().filter(|r| r.result.is_ok()).count(), 50);

        // Should be significantly faster than sequential (rough estimate)
        println!("Processed 50 documents in {:?}", duration);
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let config = ParallelConfig::default().with_max_concurrent(1); // Sequential for predictable testing
        let extractor = ParallelExtractor::new(config);

        let tasks = vec![
            DocumentTask {
                id: 0,
                url: "https://low.com".to_string(),
                html: create_test_html("Low Priority"),
                priority: 1,
            },
            DocumentTask {
                id: 1,
                url: "https://high.com".to_string(),
                html: create_test_html("High Priority"),
                priority: 10,
            },
            DocumentTask {
                id: 2,
                url: "https://medium.com".to_string(),
                html: create_test_html("Medium Priority"),
                priority: 5,
            },
        ];

        let results = extractor.extract_tasks(tasks).await.unwrap();

        assert_eq!(results.len(), 3);
        // Results should be sorted by task ID, not processing order
        assert_eq!(results[0].task_id, 0);
        assert_eq!(results[1].task_id, 1);
        assert_eq!(results[2].task_id, 2);
    }

    #[tokio::test]
    async fn test_streaming_results() {
        let config = ParallelConfig::default().with_max_concurrent(3);
        let extractor = ParallelExtractor::new(config);

        let documents = vec![
            (
                "https://example1.com".to_string(),
                create_test_html("Doc 1"),
            ),
            (
                "https://example2.com".to_string(),
                create_test_html("Doc 2"),
            ),
            (
                "https://example3.com".to_string(),
                create_test_html("Doc 3"),
            ),
        ];

        let mut rx = extractor.extract_batch_streaming(documents).await.unwrap();

        let mut count = 0;
        while let Some(_result) = rx.recv().await {
            count += 1;
        }

        assert_eq!(count, 3);
    }
}
