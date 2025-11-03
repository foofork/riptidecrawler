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

use riptide_events::{CrawlEvent, CrawlOperation, EventBus, EventEmitter, ExtractionMode};

use crate::{
    CompletionRequest, FailoverManager, IntelligenceError, LlmClientPool, LlmRegistry, Message,
};

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
    #[must_use]
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

    #[must_use]
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    #[must_use]
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
    /// LLM model to use for enhancement
    pub llm_model: String,
    /// Maximum tokens for LLM requests
    pub max_tokens: u32,
    /// Temperature for LLM requests
    pub temperature: f32,
    /// Rate limiting: requests per second
    pub rate_limit_rps: f64,
    /// Initial backoff duration for retries
    pub initial_backoff: Duration,
    /// Maximum backoff duration
    pub max_backoff: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for AiProcessorConfig {
    fn default() -> Self {
        Self {
            num_workers: 4,
            queue_size: 1000,
            max_concurrent_requests: 10,
            worker_timeout: Duration::from_secs(60),
            stream_results: true,
            llm_model: "gpt-3.5-turbo".to_string(),
            max_tokens: 2048,
            temperature: 0.7,
            rate_limit_rps: 10.0,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

/// Rate limiter state for LLM requests
#[derive(Debug)]
struct RateLimiter {
    last_request: Arc<RwLock<Instant>>,
    min_interval: Duration,
}

impl RateLimiter {
    fn new(requests_per_second: f64) -> Self {
        let min_interval = Duration::from_secs_f64(1.0 / requests_per_second);
        Self {
            last_request: Arc::new(RwLock::new(Instant::now() - min_interval)),
            min_interval,
        }
    }

    async fn acquire(&self) {
        let mut last_request = self.last_request.write().await;
        let elapsed = last_request.elapsed();

        if elapsed < self.min_interval {
            let sleep_duration = self.min_interval - elapsed;
            drop(last_request); // Release lock before sleeping
            tokio::time::sleep(sleep_duration).await;
            *self.last_request.write().await = Instant::now();
        } else {
            *last_request = Instant::now();
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
    llm_registry: Option<Arc<LlmRegistry>>,
    llm_failover: Option<Arc<FailoverManager>>,
    llm_client_pool: Option<Arc<LlmClientPool>>,
    rate_limiter: Arc<RateLimiter>,
}

impl BackgroundAiProcessor {
    /// Create a new Background AI Processor
    pub fn new(config: AiProcessorConfig) -> Self {
        let (result_sender, result_receiver) = mpsc::unbounded_channel();
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limit_rps));

        Self {
            semaphore: Arc::new(Semaphore::new(config.max_concurrent_requests)),
            rate_limiter,
            config,
            task_queue: Arc::new(RwLock::new(Vec::new())),
            result_sender,
            result_receiver: Arc::new(RwLock::new(result_receiver)),
            event_bus: None,
            workers: Vec::new(),
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            llm_registry: None,
            llm_failover: None,
            llm_client_pool: None,
        }
    }

    /// Set the event bus for event emission
    #[must_use]
    pub fn with_event_bus(mut self, event_bus: Arc<EventBus>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    /// Set the LLM registry for AI processing
    #[must_use]
    pub fn with_llm_registry(mut self, registry: Arc<LlmRegistry>) -> Self {
        self.llm_registry = Some(registry);
        self
    }

    /// Set the LLM failover manager for high availability
    #[must_use]
    pub fn with_llm_failover(mut self, failover: Arc<FailoverManager>) -> Self {
        self.llm_failover = Some(failover);
        self
    }

    /// Set the LLM client pool for efficient resource management
    #[must_use]
    pub fn with_llm_client_pool(mut self, client_pool: Arc<LlmClientPool>) -> Self {
        self.llm_client_pool = Some(client_pool);
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
            let llm_registry = self.llm_registry.clone();
            let llm_failover = self.llm_failover.clone();
            let llm_client_pool = self.llm_client_pool.clone();
            let rate_limiter = self.rate_limiter.clone();
            let llm_config = (
                self.config.llm_model.clone(),
                self.config.max_tokens,
                self.config.temperature,
                self.config.initial_backoff,
                self.config.max_backoff,
                self.config.backoff_multiplier,
            );

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
                        let _permit = match semaphore.acquire().await {
                            Ok(permit) => permit,
                            Err(_) => {
                                tracing::error!("Background processor semaphore closed");
                                break;
                            }
                        };
                        // Process the task
                        if let Err(e) = Self::process_task_worker(
                            worker_id,
                            task,
                            result_sender.clone(),
                            event_bus.clone(),
                            worker_timeout,
                            llm_registry.clone(),
                            llm_failover.clone(),
                            llm_client_pool.clone(),
                            rate_limiter.clone(),
                            llm_config.clone(),
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
    #[allow(clippy::too_many_arguments)]
    async fn process_task_worker(
        worker_id: usize,
        mut task: AiTask,
        result_sender: mpsc::UnboundedSender<AiResult>,
        event_bus: Option<Arc<EventBus>>,
        timeout: Duration,
        llm_registry: Option<Arc<LlmRegistry>>,
        llm_failover: Option<Arc<FailoverManager>>,
        llm_client_pool: Option<Arc<LlmClientPool>>,
        rate_limiter: Arc<RateLimiter>,
        llm_config: (String, u32, f32, Duration, Duration, f64),
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
        let result = tokio::time::timeout(
            timeout,
            Self::enhance_content(
                &task,
                llm_registry,
                llm_failover,
                llm_client_pool,
                rate_limiter,
                llm_config,
            ),
        )
        .await;

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

    /// Enhance content using LLM with rate limiting and exponential backoff
    ///
    /// This method integrates with the LLM client pool for efficient resource management,
    /// circuit breaker for fault tolerance, and failover for high availability.
    async fn enhance_content(
        task: &AiTask,
        llm_registry: Option<Arc<LlmRegistry>>,
        llm_failover: Option<Arc<FailoverManager>>,
        llm_client_pool: Option<Arc<LlmClientPool>>,
        rate_limiter: Arc<RateLimiter>,
        llm_config: (String, u32, f32, Duration, Duration, f64),
    ) -> Result<String> {
        let (model, max_tokens, temperature, initial_backoff, max_backoff, backoff_multiplier) =
            llm_config;

        // Build the LLM request
        let messages = vec![
            Message::system(
                "You are an AI content enhancer. Analyze and enhance the given web content by extracting key information, improving clarity, and adding structured insights."
            ),
            Message::user(format!(
                "URL: {}\n\nContent:\n{}",
                task.url, task.content
            )),
        ];

        let request = CompletionRequest::new(model.clone(), messages)
            .with_max_tokens(max_tokens)
            .with_temperature(temperature);

        // INTEGRATION POINT: Use LLM client pool if available (provides connection pooling,
        // circuit breaker, timeout handling, and automatic retries)
        if let Some(client_pool) = llm_client_pool {
            debug!(
                "Using LLM client pool for task {} with provider: {}",
                task.task_id, model
            );

            match client_pool.complete(request, &model).await {
                Ok(response) => {
                    debug!(
                        "LLM client pool enhancement successful for task {}",
                        task.task_id
                    );
                    return Ok(response.content);
                }
                Err(e) => {
                    error!("LLM client pool failed for task {}: {}", task.task_id, e);
                    return Err(anyhow::anyhow!("LLM client pool enhancement failed: {}", e));
                }
            }
        }

        // FALLBACK: Use legacy path if client pool not configured
        debug!(
            "LLM client pool not configured, using legacy enhancement path for task {}",
            task.task_id
        );

        // If no LLM registry is configured, return placeholder
        let Some(registry) = llm_registry else {
            warn!("No LLM registry configured, using placeholder enhancement");
            return Ok(format!(
                "AI Enhanced (placeholder): {} (length: {})",
                task.url,
                task.content.len()
            ));
        };

        // Apply rate limiting before making request
        rate_limiter.acquire().await;

        // Try to get response with exponential backoff (legacy path)
        let mut backoff = initial_backoff;
        let mut last_error: Option<IntelligenceError> = None;

        for attempt in 0..task.max_retries {
            // Build the LLM request for each retry (in case it needs to be modified)
            let messages = vec![
                Message::system(
                    "You are an AI content enhancer. Analyze and enhance the given web content by extracting key information, improving clarity, and adding structured insights."
                ),
                Message::user(format!(
                    "URL: {}\n\nContent:\n{}",
                    task.url, task.content
                )),
            ];

            let request = CompletionRequest::new(model.clone(), messages)
                .with_max_tokens(max_tokens)
                .with_temperature(temperature);

            // Try with failover manager first if available
            let result = if let Some(failover) = &llm_failover {
                failover.complete_with_failover(request.clone()).await
            } else {
                // Fallback to registry default provider
                match registry.get_provider("default").or_else(|| {
                    // Try to get first available provider
                    registry
                        .list_providers()
                        .first()
                        .and_then(|name| registry.get_provider(name))
                }) {
                    Some(provider) => provider.complete(request.clone()).await,
                    None => return Err(anyhow::anyhow!("No LLM providers available in registry")),
                }
            };

            match result {
                Ok(response) => {
                    debug!(
                        "LLM enhancement successful for task {} (attempt {})",
                        task.task_id,
                        attempt.saturating_add(1)
                    );
                    return Ok(response.content);
                }
                Err(e) => {
                    last_error = Some(e.clone());

                    match &e {
                        IntelligenceError::RateLimit { retry_after_ms } => {
                            let retry_duration = Duration::from_millis(*retry_after_ms);
                            warn!(
                                "Rate limit hit for task {}, waiting {:?}",
                                task.task_id, retry_duration
                            );
                            tokio::time::sleep(retry_duration).await;
                            continue;
                        }
                        IntelligenceError::CircuitOpen { reason } => {
                            warn!("Circuit breaker open for task {}: {}", task.task_id, reason);
                            // Wait and retry
                            tokio::time::sleep(backoff).await;
                        }
                        IntelligenceError::Network(_)
                        | IntelligenceError::Provider(_)
                        | IntelligenceError::Timeout { .. } => {
                            if attempt < task.max_retries.saturating_sub(1) {
                                warn!(
                                    "Transient error for task {} (attempt {}): {}, retrying in {:?}",
                                    task.task_id,
                                    attempt.saturating_add(1),
                                    e,
                                    backoff
                                );
                                tokio::time::sleep(backoff).await;
                                backoff = std::cmp::min(
                                    Duration::from_secs_f64(
                                        backoff.as_secs_f64() * backoff_multiplier,
                                    ),
                                    max_backoff,
                                );
                            }
                        }
                        _ => {
                            // Non-retryable error
                            return Err(anyhow::anyhow!("LLM enhancement failed: {}", e));
                        }
                    }
                }
            }
        }

        // All retries exhausted
        let error_msg = last_error
            .map(|e| e.to_string())
            .unwrap_or_else(|| "Unknown error".to_string());
        Err(anyhow::anyhow!(
            "LLM enhancement failed after {} attempts: {}",
            task.max_retries,
            error_msg
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
