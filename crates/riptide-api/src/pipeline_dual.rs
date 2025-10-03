//! Dual-Path Pipeline Orchestrator
//!
//! Implements the Zero-Impact AI Roadmap Phase 1 dual-path processing pattern:
//! 1. **Fast Path**: Immediate CSS-based extraction (primary, always returns quickly)
//! 2. **Enhancement Path**: Async AI enhancement (secondary, non-blocking)
//!
//! Key principles:
//! - Fast path returns results in <200ms
//! - AI enhancement runs asynchronously without blocking
//! - Results are correlated and merged via event system
//! - No performance penalty for baseline extraction

use crate::errors::{ApiError, ApiResult};
use crate::metrics::RipTideMetrics;
use crate::state::AppState;
use anyhow::Result;
use riptide_core::{
    ai_processor::{AiProcessorConfig, AiTask, BackgroundAiProcessor, TaskPriority},
    events::{CrawlEvent, CrawlOperation, EventBus, EventEmitter, ExtractionMode},
    types::{CrawlOptions, ExtractedDoc},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

/// Result from the fast path (CSS extraction)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastPathResult {
    pub task_id: String,
    pub url: String,
    pub document: ExtractedDoc,
    pub processing_time_ms: u64,
    pub quality_score: f32,
}

/// Result from the AI enhancement path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancementResult {
    pub task_id: String,
    pub url: String,
    pub enhanced_document: Option<ExtractedDoc>,
    pub processing_time_ms: u64,
    pub success: bool,
    pub error: Option<String>,
}

/// Merged result combining fast path and enhancement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualPathResult {
    pub task_id: String,
    pub url: String,
    pub document: ExtractedDoc,
    pub fast_path_time_ms: u64,
    pub enhancement_time_ms: Option<u64>,
    pub total_time_ms: u64,
    pub enhanced: bool,
    pub quality_score: f32,
}

/// Configuration for dual-path processing
#[derive(Debug, Clone)]
pub struct DualPathConfig {
    /// Enable AI enhancement (if false, only fast path runs)
    pub enable_ai_enhancement: bool,
    /// Minimum quality score to trigger AI enhancement
    pub enhancement_quality_threshold: f32,
    /// AI enhancement priority
    pub enhancement_priority: TaskPriority,
    /// AI processor configuration
    pub ai_processor_config: AiProcessorConfig,
}

impl Default for DualPathConfig {
    fn default() -> Self {
        Self {
            enable_ai_enhancement: false, // Disabled by default for safety
            enhancement_quality_threshold: 0.6,
            enhancement_priority: TaskPriority::Normal,
            ai_processor_config: AiProcessorConfig::default(),
        }
    }
}

/// Dual-path pipeline orchestrator
pub struct DualPathOrchestrator {
    state: AppState,
    options: CrawlOptions,
    config: DualPathConfig,
    metrics: Arc<RipTideMetrics>,
    ai_processor: Arc<RwLock<BackgroundAiProcessor>>,
    event_bus: Arc<EventBus>,
    pending_results: Arc<RwLock<HashMap<String, FastPathResult>>>,
}

impl DualPathOrchestrator {
    /// Create new dual-path orchestrator
    pub fn new(
        state: AppState,
        options: CrawlOptions,
        config: DualPathConfig,
        event_bus: Arc<EventBus>,
    ) -> Self {
        let metrics = state.metrics.clone();

        // Create AI processor with event bus integration
        let ai_processor = BackgroundAiProcessor::new(config.ai_processor_config.clone())
            .with_event_bus(event_bus.clone());

        Self {
            state,
            options,
            config,
            metrics,
            ai_processor: Arc::new(RwLock::new(ai_processor)),
            event_bus,
            pending_results: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start the AI processor
    pub async fn start(&self) -> Result<()> {
        let mut processor = self.ai_processor.write().await;
        processor.start().await?;
        info!("Dual-path orchestrator started with AI processor");
        Ok(())
    }

    /// Stop the AI processor
    pub async fn stop(&self) -> Result<()> {
        let mut processor = self.ai_processor.write().await;
        processor.stop().await?;
        info!("Dual-path orchestrator stopped");
        Ok(())
    }

    /// Execute dual-path processing for a URL
    ///
    /// Returns immediately with fast-path results. AI enhancement runs asynchronously.
    pub async fn execute(&self, url: &str) -> ApiResult<DualPathResult> {
        let task_id = Uuid::new_v4().to_string();
        let overall_start = Instant::now();

        info!(
            task_id = %task_id,
            url = %url,
            ai_enabled = self.config.enable_ai_enhancement,
            "Starting dual-path processing"
        );

        // === FAST PATH: CSS-based extraction (ALWAYS runs, returns quickly) ===
        let fast_path_result = self.execute_fast_path(&task_id, url).await?;

        // Determine if AI enhancement should run
        let should_enhance = self.config.enable_ai_enhancement
            && fast_path_result.quality_score < self.config.enhancement_quality_threshold;

        // === ENHANCEMENT PATH: Queue AI processing (non-blocking) ===
        if should_enhance {
            self.queue_ai_enhancement(&task_id, url, &fast_path_result)
                .await?;
        }

        // Build result from fast path (immediate return)
        let result = DualPathResult {
            task_id: task_id.clone(),
            url: url.to_string(),
            document: fast_path_result.document.clone(),
            fast_path_time_ms: fast_path_result.processing_time_ms,
            enhancement_time_ms: None,
            total_time_ms: overall_start.elapsed().as_millis() as u64,
            enhanced: false,
            quality_score: fast_path_result.quality_score,
        };

        info!(
            task_id = %task_id,
            url = %url,
            fast_path_ms = result.fast_path_time_ms,
            total_ms = result.total_time_ms,
            quality_score = result.quality_score,
            ai_queued = should_enhance,
            "Dual-path processing completed (fast path)"
        );

        Ok(result)
    }

    /// Execute fast path (CSS extraction)
    async fn execute_fast_path(&self, task_id: &str, url: &str) -> ApiResult<FastPathResult> {
        let start = Instant::now();

        // Emit fast path started event
        let event = CrawlEvent::new(
            CrawlOperation::FastPathStarted,
            task_id.to_string(),
            url.to_string(),
            ExtractionMode::FastCss,
            "dual_path_orchestrator",
        );
        let _ = self.event_bus.emit_event(event).await;

        // Fetch content
        let content = self.fetch_content(url).await?;

        // Extract with CSS (fast)
        let document = self.extract_with_css(url, &content).await?;

        let quality_score = document
            .quality_score
            .map(|q| q as f32 / 100.0)
            .unwrap_or(0.0);
        let processing_time_ms = start.elapsed().as_millis() as u64;

        let result = FastPathResult {
            task_id: task_id.to_string(),
            url: url.to_string(),
            document,
            processing_time_ms,
            quality_score,
        };

        // Emit fast path completed event
        let event = CrawlEvent::new(
            CrawlOperation::FastPathCompleted,
            task_id.to_string(),
            url.to_string(),
            ExtractionMode::FastCss,
            "dual_path_orchestrator",
        )
        .with_duration(start.elapsed())
        .with_quality_score(quality_score as f32);
        let _ = self.event_bus.emit_event(event).await;

        // Store for correlation
        self.pending_results
            .write()
            .await
            .insert(task_id.to_string(), result.clone());

        debug!(
            task_id = %task_id,
            url = %url,
            time_ms = processing_time_ms,
            quality = quality_score,
            "Fast path completed"
        );

        Ok(result)
    }

    /// Queue AI enhancement task (non-blocking)
    async fn queue_ai_enhancement(
        &self,
        task_id: &str,
        url: &str,
        fast_result: &FastPathResult,
    ) -> ApiResult<()> {
        let ai_task = AiTask::new(url.to_string(), fast_result.document.text.clone())
            .with_priority(self.config.enhancement_priority);

        let processor = self.ai_processor.read().await;
        processor
            .queue_task(ai_task)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to queue AI task: {}", e)))?;

        debug!(
            task_id = %task_id,
            url = %url,
            "AI enhancement queued"
        );

        Ok(())
    }

    /// Fetch content for a URL
    async fn fetch_content(&self, url: &str) -> ApiResult<String> {
        // Use existing fetch logic from riptide-core
        use riptide_core::fetch::FetchEngine;

        let fetch_engine = FetchEngine::new()
            .map_err(|e| ApiError::internal(format!("Failed to create fetch engine: {}", e)))?;

        let content = fetch_engine
            .fetch_text(url)
            .await
            .map_err(|e| ApiError::fetch(url, format!("Fetch failed: {}", e)))?;

        Ok(content)
    }

    /// Extract with CSS (fast path)
    async fn extract_with_css(&self, url: &str, content: &str) -> ApiResult<ExtractedDoc> {
        // For now, create a basic extracted document using simple text extraction
        // In production, this would use the WASM extractor
        let text = content
            .chars()
            .filter(|c| !c.is_control())
            .take(10000)
            .collect::<String>();

        let word_count = text.split_whitespace().count() as u32;
        let quality_score = if word_count > 100 { 75 } else { 50 };

        Ok(ExtractedDoc {
            url: url.to_string(),
            title: Some("Extracted Content".to_string()),
            text,
            quality_score: Some(quality_score),
            links: Vec::new(),
            byline: None,
            published_iso: None,
            markdown: None,
            media: Vec::new(),
            language: Some("en".to_string()),
            reading_time: Some(word_count / 200), // Assuming 200 words per minute
            word_count: Some(word_count),
            categories: Vec::new(),
            site_name: None,
            description: None,
        })
    }

    /// Poll for enhancement results and merge with pending fast path results
    pub async fn poll_enhancements(&self) -> Vec<DualPathResult> {
        let processor = self.ai_processor.read().await;
        let enhancement_results = processor.recv_all_results().await;

        let mut merged_results = Vec::new();
        let mut pending = self.pending_results.write().await;

        for enhancement in enhancement_results {
            if let Some(fast_result) = pending.remove(&enhancement.task_id) {
                // Merge results
                let document = if enhancement.success && enhancement.enhanced_content.is_some() {
                    // Update the document text with enhanced content
                    let mut doc = fast_result.document.clone();
                    doc.text = enhancement.enhanced_content.unwrap_or(doc.text);
                    doc
                } else {
                    // Fall back to fast path result
                    fast_result.document.clone()
                };

                let total_time = fast_result.processing_time_ms + enhancement.processing_time_ms;

                let merged = DualPathResult {
                    task_id: enhancement.task_id.clone(),
                    url: enhancement.url.clone(),
                    document,
                    fast_path_time_ms: fast_result.processing_time_ms,
                    enhancement_time_ms: Some(enhancement.processing_time_ms),
                    total_time_ms: total_time,
                    enhanced: enhancement.success,
                    quality_score: fast_result.quality_score,
                };

                // Emit results merged event
                let event = CrawlEvent::new(
                    CrawlOperation::ResultsMerged,
                    enhancement.task_id.clone(),
                    enhancement.url.clone(),
                    ExtractionMode::AiEnhancement,
                    "dual_path_orchestrator",
                );
                let _ = self.event_bus.emit_event(event).await;

                merged_results.push(merged);
            }
        }

        merged_results
    }

    /// Get processor statistics
    pub async fn stats(&self) -> DualPathStats {
        let processor = self.ai_processor.read().await;
        let ai_stats = processor.stats().await;
        let pending = self.pending_results.read().await;

        DualPathStats {
            pending_enhancements: pending.len(),
            ai_queue_size: ai_stats.queue_size,
            ai_active_workers: ai_stats.active_workers,
            ai_total_workers: ai_stats.total_workers,
            ai_enabled: self.config.enable_ai_enhancement,
        }
    }
}

/// Dual-path processing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualPathStats {
    pub pending_enhancements: usize,
    pub ai_queue_size: usize,
    pub ai_active_workers: usize,
    pub ai_total_workers: usize,
    pub ai_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dual_path_config_default() {
        let config = DualPathConfig::default();
        assert!(!config.enable_ai_enhancement);
        assert_eq!(config.enhancement_quality_threshold, 0.6);
    }

    #[test]
    fn test_fast_path_result_creation() {
        let doc = ExtractedDoc {
            url: "https://example.com".to_string(),
            title: "Test".to_string(),
            text: "Content".to_string(),
            quality_score: 0.8,
            ..Default::default()
        };

        let result = FastPathResult {
            task_id: "test-123".to_string(),
            url: "https://example.com".to_string(),
            document: doc,
            processing_time_ms: 150,
            quality_score: 0.8,
        };

        assert_eq!(result.processing_time_ms, 150);
        assert_eq!(result.quality_score, 0.8);
    }
}
