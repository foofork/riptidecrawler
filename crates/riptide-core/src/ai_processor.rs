//! Background AI Processor for async content enhancement
//!
//! This module implements the Background AI Processor pattern from the Zero-Impact
//! AI Implementation Roadmap Phase 1 Week 2. It provides:
//!
//! - Priority-based task queuing for AI enhancement
//! - Work-stealing worker pool architecture
//! - Non-blocking async processing
//! - Result correlation and streaming
//! - Graceful error handling and fallback

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::events::{CrawlEvent, CrawlOperation, EventBus, EventEmitter, ExtractionMode};

/// Priority levels for AI enhancement tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// AI enhancement task
#[derive(Debug, Clone)]
pub struct AiTask {
    pub task_id: String,
    pub url: String,
    pub content: String,
    pub priority: TaskPriority,
    pub created_at: Instant,
    pub timeout: Duration,
    pub retry_count: u32,
    pub max_retries: u32,
}

impl AiTask {
    pub fn new(url: String, content: String) -> Self {
        Self {
            task_id: Uuid::new_v4().to_string(),
            url,
            content,
            priority: TaskPriority::Normal,
            created_at: Instant::now(),
            timeout: Duration::from_secs(30),
            retry_count: 0,
            max_retries: 3,
        }
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}

/// AI enhancement result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResult {
    pub task_id: String,
    pub url: String,
    pub enhanced_content: Option<String>,
    pub metadata: HashMap<String, String>,
    pub processing_time_ms: u64,
    pub success: bool,
    pub error: Option<String>,
}

/// Configuration for the Background AI Processor
#[derive(Debug, Clone)]
pub struct AiProcessorConfig {
    /// Number of worker threads
    pub num_workers: usize,
    /// Size of the task queue
    pub queue_size: usize,
    /// Maximum concurrent AI requests
    pub max_concurrent_requests: usize,
    /// Worker timeout
    pub worker_timeout: Duration,
    /// Enable result streaming
    pub stream_results: bool,
}

impl Default for AiProcessorConfig {
    fn default() -> Self {
        Self {
            num_workers: 4,
            queue_size: 1000,
            max_concurrent_requests: 10,
            worker_timeout: Duration::from_secs(60),
            stream_results: true,
        }
    }
}

/// Background AI Processor with work-stealing queue system
pub struct BackgroundAiProcessor {
    config: AiProcessorConfig,
    task_queue: Arc<RwLock<Vec<AiTask>>>,
    result_sender: mpsc::UnboundedSender<AiResult>,
    result_receiver: Arc<RwLock<mpsc::UnboundedReceiver<AiResult>>>,
    event_bus: Option<Arc<EventBus>>,
    workers: Vec<JoinHandle<()>>,
    semaphore: Arc<Semaphore>,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl BackgroundAiProcessor {
    /// Create a new Background AI Processor
    pub fn new(config: AiProcessorConfig) -> Self {
        let (result_sender, result_receiver) = mpsc::unbounded_channel();

        Self {
            semaphore: Arc::new(Semaphore::new(config.max_concurrent_requests)),
            config,
            task_queue: Arc::new(RwLock::new(Vec::new())),
            result_sender,
            result_receiver: Arc::new(RwLock::new(result_receiver)),
            event_bus: None,
            workers: Vec::new(),
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Set the event bus for event emission
    pub fn with_event_bus(mut self, event_bus: Arc<EventBus>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    /// Start the processor and spawn workers
    pub async fn start(&mut self) -> Result<()> {
        if self.running.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(());
        }

        self.running
            .store(true, std::sync::atomic::Ordering::Relaxed);

        info!(
            "Starting Background AI Processor with {} workers",
            self.config.num_workers
        );

        // Spawn worker tasks
        for worker_id in 0..self.config.num_workers {
            let task_queue = self.task_queue.clone();
            let result_sender = self.result_sender.clone();
            let event_bus = self.event_bus.clone();
            let semaphore = self.semaphore.clone();
            let running = self.running.clone();
            let worker_timeout = self.config.worker_timeout;

            let worker = tokio::spawn(async move {
                debug!("Worker {} started", worker_id);

                while running.load(std::sync::atomic::Ordering::Relaxed) {
                    // Try to acquire a task (work-stealing pattern)
                    let task = {
                        let mut queue = task_queue.write().await;
                        queue.sort_by(|a, b| b.priority.cmp(&a.priority));
                        queue.pop()
                    };

                    if let Some(task) = task {
                        // Acquire semaphore permit for concurrency control
                        // RAII guard: must remain in scope to hold the permit during task processing
                        let _permit = semaphore.acquire().await.unwrap();
                        // Process the task
                        if let Err(e) = Self::process_task_worker(
                            worker_id,
                            task,
                            result_sender.clone(),
                            event_bus.clone(),
                            worker_timeout,
                        )
                        .await
                        {
                            error!("Worker {} task processing error: {}", worker_id, e);
                        }
                    } else {
                        // No tasks available, sleep briefly
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }

                debug!("Worker {} stopped", worker_id);
            });

            self.workers.push(worker);
        }

        Ok(())
    }

    /// Stop the processor and wait for workers to complete
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping Background AI Processor");
        self.running
            .store(false, std::sync::atomic::Ordering::Relaxed);

        // Wait for all workers to complete
        for worker in self.workers.drain(..) {
            let _ = worker.await;
        }

        Ok(())
    }

    /// Queue a task for AI enhancement
    pub async fn queue_task(&self, task: AiTask) -> Result<()> {
        let mut queue = self.task_queue.write().await;

        if queue.len() >= self.config.queue_size {
            warn!("Task queue is full, dropping task: {}", task.task_id);
            return Err(anyhow::anyhow!("Task queue is full"));
        }

        // Emit queued event
        if let Some(event_bus) = &self.event_bus {
            let event = CrawlEvent::new(
                CrawlOperation::AiEnhancementQueued,
                task.task_id.clone(),
                task.url.clone(),
                ExtractionMode::AiEnhancement,
                "ai_processor",
            );
            let _ = event_bus.emit_event(event).await;
        }

        queue.push(task);
        Ok(())
    }

    /// Get the next result (non-blocking)
    pub async fn try_recv_result(&self) -> Option<AiResult> {
        let mut receiver = self.result_receiver.write().await;
        receiver.try_recv().ok()
    }

    /// Get all pending results
    pub async fn recv_all_results(&self) -> Vec<AiResult> {
        let mut results = Vec::new();
        while let Some(result) = self.try_recv_result().await {
            results.push(result);
        }
        results
    }

    /// Worker task processing logic
    async fn process_task_worker(
        worker_id: usize,
        mut task: AiTask,
        result_sender: mpsc::UnboundedSender<AiResult>,
        event_bus: Option<Arc<EventBus>>,
        timeout: Duration,
    ) -> Result<()> {
        debug!(
            "Worker {} processing task {} for URL: {}",
            worker_id, task.task_id, task.url
        );

        let start = Instant::now();

        // Emit started event
        if let Some(event_bus) = &event_bus {
            let event = CrawlEvent::new(
                CrawlOperation::AiEnhancementStarted,
                task.task_id.clone(),
                task.url.clone(),
                ExtractionMode::AiEnhancement,
                "ai_processor",
            );
            let _ = event_bus.emit_event(event).await;
        }

        // Process with timeout
        let result = tokio::time::timeout(timeout, Self::enhance_content(&task)).await;

        let processing_time = start.elapsed();

        match result {
            Ok(Ok(enhanced_content)) => {
                let result = AiResult {
                    task_id: task.task_id.clone(),
                    url: task.url.clone(),
                    enhanced_content: Some(enhanced_content),
                    metadata: HashMap::new(),
                    processing_time_ms: processing_time.as_millis() as u64,
                    success: true,
                    error: None,
                };

                // Emit completed event
                if let Some(event_bus) = &event_bus {
                    let event = CrawlEvent::new(
                        CrawlOperation::AiEnhancementCompleted,
                        task.task_id.clone(),
                        task.url.clone(),
                        ExtractionMode::AiEnhancement,
                        "ai_processor",
                    )
                    .with_duration(processing_time);
                    let _ = event_bus.emit_event(event).await;
                }

                let _ = result_sender.send(result);
            }
            Ok(Err(e)) => {
                // Enhancement failed, retry if possible
                if task.can_retry() {
                    task.increment_retry();
                    warn!(
                        "Task {} failed (attempt {}), will retry: {}",
                        task.task_id, task.retry_count, e
                    );
                    // Re-queue for retry (could use exponential backoff)
                    // For now, we'll just send error result
                } else {
                    error!("Task {} failed after all retries: {}", task.task_id, e);
                }

                let result = AiResult {
                    task_id: task.task_id.clone(),
                    url: task.url.clone(),
                    enhanced_content: None,
                    metadata: HashMap::new(),
                    processing_time_ms: processing_time.as_millis() as u64,
                    success: false,
                    error: Some(e.to_string()),
                };

                // Emit failed event
                if let Some(event_bus) = &event_bus {
                    let mut event = CrawlEvent::new(
                        CrawlOperation::AiEnhancementFailed,
                        task.task_id.clone(),
                        task.url.clone(),
                        ExtractionMode::AiEnhancement,
                        "ai_processor",
                    )
                    .with_duration(processing_time);
                    event.add_metadata("error", &e.to_string());
                    let _ = event_bus.emit_event(event).await;
                }

                let _ = result_sender.send(result);
            }
            Err(_) => {
                // Timeout
                warn!("Task {} timed out after {:?}", task.task_id, timeout);

                let result = AiResult {
                    task_id: task.task_id.clone(),
                    url: task.url.clone(),
                    enhanced_content: None,
                    metadata: HashMap::new(),
                    processing_time_ms: timeout.as_millis() as u64,
                    success: false,
                    error: Some("Task timed out".to_string()),
                };

                // Emit timeout event
                if let Some(event_bus) = &event_bus {
                    let mut event = CrawlEvent::new(
                        CrawlOperation::Timeout,
                        task.task_id.clone(),
                        task.url.clone(),
                        ExtractionMode::AiEnhancement,
                        "ai_processor",
                    );
                    event.add_metadata("timeout_ms", &timeout.as_millis().to_string());
                    let _ = event_bus.emit_event(event).await;
                }

                let _ = result_sender.send(result);
            }
        }

        Ok(())
    }

    /// Placeholder for actual AI enhancement logic
    /// TODO: Integrate with LLM client pool
    async fn enhance_content(task: &AiTask) -> Result<String> {
        // Simulate AI processing
        tokio::time::sleep(Duration::from_millis(50)).await;

        // For now, return placeholder enhanced content
        Ok(format!(
            "AI Enhanced: {} (length: {})",
            task.url,
            task.content.len()
        ))
    }

    /// Get processor statistics
    pub async fn stats(&self) -> AiProcessorStats {
        let queue = self.task_queue.read().await;
        let available_permits = self.semaphore.available_permits();

        AiProcessorStats {
            queue_size: queue.len(),
            active_workers: self.config.max_concurrent_requests - available_permits,
            total_workers: self.config.num_workers,
            is_running: self.running.load(std::sync::atomic::Ordering::Relaxed),
        }
    }
}

/// Processor statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProcessorStats {
    pub queue_size: usize,
    pub active_workers: usize,
    pub total_workers: usize,
    pub is_running: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_priority_ordering() {
        let low = AiTask::new("url1".to_string(), "content1".to_string())
            .with_priority(TaskPriority::Low);
        let high = AiTask::new("url2".to_string(), "content2".to_string())
            .with_priority(TaskPriority::High);

        assert!(high.priority > low.priority);
    }

    #[tokio::test]
    async fn test_ai_processor_creation() {
        let config = AiProcessorConfig::default();
        let processor = BackgroundAiProcessor::new(config);

        let stats = processor.stats().await;
        assert_eq!(stats.queue_size, 0);
        assert!(!stats.is_running);
    }

    #[tokio::test]
    async fn test_task_queuing() {
        let config = AiProcessorConfig::default();
        let processor = BackgroundAiProcessor::new(config);

        let task = AiTask::new(
            "https://example.com".to_string(),
            "test content".to_string(),
        );
        let result = processor.queue_task(task).await;

        assert!(result.is_ok());

        let stats = processor.stats().await;
        assert_eq!(stats.queue_size, 1);
    }
}
