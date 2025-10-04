// TODO: Enhanced pipeline orchestrator prepared for production use
#![allow(dead_code)]

use crate::errors::ApiResult;
use crate::metrics::{PhaseTimer, PhaseType, RipTideMetrics};
use crate::pipeline::{PipelineOrchestrator, PipelineResult, PipelineStats};
use crate::state::{AppState, EnhancedPipelineConfig};
use anyhow::Result;
use riptide_core::types::{CrawlOptions, ExtractedDoc};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, warn};

/// Enhanced pipeline orchestrator with comprehensive phase timing and metrics
///
/// This orchestrator wraps the standard PipelineOrchestrator and adds:
/// - Detailed phase timing for fetch, gate, wasm, and render
/// - Enhanced metrics collection
/// - Debug logging for each phase
/// - Phase visualization data
pub struct EnhancedPipelineOrchestrator {
    /// Application state
    state: AppState,

    /// Crawl options
    options: CrawlOptions,

    /// Metrics collector
    metrics: Arc<RipTideMetrics>,

    /// Enhanced pipeline configuration
    config: EnhancedPipelineConfig,

    /// Underlying pipeline orchestrator
    pipeline: PipelineOrchestrator,
}

impl EnhancedPipelineOrchestrator {
    /// Create new enhanced pipeline orchestrator
    pub fn new(state: AppState, options: CrawlOptions) -> Self {
        let metrics = state.metrics.clone();
        let config = state.config.enhanced_pipeline_config.clone();
        let pipeline = PipelineOrchestrator::new(state.clone(), options.clone());

        Self {
            state,
            options,
            metrics,
            config,
            pipeline,
        }
    }

    /// Execute single URL with enhanced metrics (delegates to standard pipeline)
    pub async fn execute_single_enhanced(&self, url: &str) -> ApiResult<EnhancedPipelineResult> {
        if !self.config.enable_enhanced_pipeline {
            // If enhanced pipeline is disabled, use standard pipeline and convert result
            let result = self.pipeline.execute_single(url).await?;
            return Ok(self.convert_to_enhanced_result(result));
        }

        self.execute_enhanced(url).await.map_err(|e| {
            crate::errors::ApiError::internal(format!("Enhanced pipeline failed: {}", e))
        })
    }

    /// Execute batch with enhanced metrics (delegates to standard pipeline)
    pub async fn execute_batch_enhanced(
        &self,
        urls: &[String],
    ) -> (Vec<Option<EnhancedPipelineResult>>, EnhancedBatchStats) {
        let overall_start = Instant::now();

        if !self.config.enable_enhanced_pipeline {
            // If enhanced pipeline is disabled, use standard pipeline
            let (results, stats) = self.pipeline.execute_batch(urls).await;
            let enhanced_results = results
                .into_iter()
                .map(|opt_result| opt_result.map(|result| self.convert_to_enhanced_result(result)))
                .collect();
            return (
                enhanced_results,
                self.convert_to_enhanced_stats(stats, overall_start),
            );
        }

        // Use enhanced pipeline for each URL
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.options.concurrency));
        let tasks: Vec<_> = urls
            .iter()
            .map(|url| {
                let semaphore = semaphore.clone();
                let orchestrator = self.clone();
                let url = url.clone();

                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.ok()?;
                    orchestrator.execute_enhanced(&url).await.ok()
                })
            })
            .collect();

        let results = futures::future::join_all(tasks).await;
        let enhanced_results: Vec<_> = results
            .into_iter()
            .map(|r| r.ok().and_then(|inner| inner))
            .collect();

        let stats = self.compute_enhanced_batch_stats(&enhanced_results, overall_start);
        (enhanced_results, stats)
    }

    /// Convert standard PipelineResult to EnhancedPipelineResult
    fn convert_to_enhanced_result(&self, result: PipelineResult) -> EnhancedPipelineResult {
        EnhancedPipelineResult {
            url: result.document.url.clone(),
            success: true,
            total_duration_ms: result.processing_time_ms,
            phase_timings: PhaseTiming::default(), // No detailed timings from standard pipeline
            document: Some(result.document),
            error: None,
            cache_hit: result.from_cache,
            gate_decision: result.gate_decision,
            quality_score: result.quality_score,
        }
    }

    /// Convert standard PipelineStats to EnhancedBatchStats
    fn convert_to_enhanced_stats(
        &self,
        stats: PipelineStats,
        start: Instant,
    ) -> EnhancedBatchStats {
        EnhancedBatchStats {
            total_urls: stats.total_processed,
            successful: stats.successful_extractions,
            failed: stats.failed_extractions,
            cache_hits: stats.cache_hits,
            total_duration_ms: start.elapsed().as_millis() as u64,
            avg_processing_time_ms: stats.avg_processing_time_ms,
            avg_phase_timings: PhaseTiming::default(),
            gate_decisions: stats.gate_decisions,
        }
    }

    /// Compute enhanced batch statistics
    fn compute_enhanced_batch_stats(
        &self,
        results: &[Option<EnhancedPipelineResult>],
        start: Instant,
    ) -> EnhancedBatchStats {
        let total_urls = results.len();
        let successful = results.iter().filter(|r| r.is_some()).count();
        let failed = total_urls - successful;

        let mut total_fetch = 0u64;
        let mut total_gate = 0u64;
        let mut total_wasm = 0u64;
        let mut total_render = 0u64;
        let mut render_count = 0usize;
        let mut cache_hits = 0;

        let mut gate_decisions = crate::pipeline::GateDecisionStats::default();

        for result in results.iter().flatten() {
            total_fetch += result.phase_timings.fetch_ms;
            total_gate += result.phase_timings.gate_ms;
            total_wasm += result.phase_timings.wasm_ms;
            if let Some(render_ms) = result.phase_timings.render_ms {
                total_render += render_ms;
                render_count += 1;
            }

            if result.cache_hit {
                cache_hits += 1;
            }

            match result.gate_decision.as_str() {
                "raw" => gate_decisions.raw += 1,
                "probes_first" => gate_decisions.probes_first += 1,
                "headless" => gate_decisions.headless += 1,
                _ => {}
            }
        }

        let avg_processing_time_ms = if successful > 0 {
            results
                .iter()
                .flatten()
                .map(|r| r.total_duration_ms)
                .sum::<u64>() as f64
                / successful as f64
        } else {
            0.0
        };

        let avg_phase_timings = PhaseTiming {
            fetch_ms: if successful > 0 {
                total_fetch / successful as u64
            } else {
                0
            },
            gate_ms: if successful > 0 {
                total_gate / successful as u64
            } else {
                0
            },
            wasm_ms: if successful > 0 {
                total_wasm / successful as u64
            } else {
                0
            },
            render_ms: if render_count > 0 {
                Some(total_render / render_count as u64)
            } else {
                None
            },
        };

        EnhancedBatchStats {
            total_urls,
            successful,
            failed,
            cache_hits,
            total_duration_ms: start.elapsed().as_millis() as u64,
            avg_processing_time_ms,
            avg_phase_timings,
            gate_decisions,
        }
    }

    /// Execute enhanced pipeline with detailed phase timing
    pub async fn execute_enhanced(&self, url: &str) -> Result<EnhancedPipelineResult> {
        let overall_start = Instant::now();

        info!(
            url = %url,
            cache_mode = %self.options.cache_mode,
            "Starting enhanced pipeline execution"
        );

        let mut result = EnhancedPipelineResult {
            url: url.to_string(),
            success: false,
            total_duration_ms: 0,
            phase_timings: PhaseTiming::default(),
            document: None,
            error: None,
            cache_hit: false,
            gate_decision: "unknown".to_string(),
            quality_score: 0.0,
        };

        // Phase 1: Fetch
        let fetch_result = self.execute_fetch_phase(url).await;
        result.phase_timings.fetch_ms = fetch_result.duration_ms;

        let (content, _http_status) = match fetch_result.result {
            Ok(data) => data,
            Err(e) => {
                result.error = Some(format!("Fetch phase failed: {}", e));
                self.metrics.record_error(crate::metrics::ErrorType::Http);
                return Ok(result);
            }
        };

        // Phase 2: Gate Analysis
        let gate_result = self.execute_gate_phase(url, &content).await;
        result.phase_timings.gate_ms = gate_result.duration_ms;
        result.gate_decision = gate_result.decision.clone();
        result.quality_score = gate_result.quality_score;

        // Record gate decision in metrics
        self.metrics.record_gate_decision(&gate_result.decision);

        // Phase 3: WASM Extraction
        let wasm_result = self
            .execute_wasm_phase(url, &content, &gate_result.decision)
            .await;
        result.phase_timings.wasm_ms = wasm_result.duration_ms;

        let document = match wasm_result.result {
            Ok(doc) => doc,
            Err(e) => {
                result.error = Some(format!("WASM phase failed: {}", e));
                self.metrics.record_error(crate::metrics::ErrorType::Wasm);
                return Ok(result);
            }
        };

        // Phase 4: Render (if headless required)
        if gate_result.decision == "headless" {
            let render_result = self.execute_render_phase(url).await;
            result.phase_timings.render_ms = Some(render_result.duration_ms);

            match render_result.result {
                Ok(enhanced_doc) => {
                    result.document = Some(enhanced_doc);
                }
                Err(e) => {
                    warn!("Render phase failed, using WASM result: {}", e);
                    result.document = Some(document);
                }
            }
        } else {
            result.document = Some(document);
        }

        result.success = true;
        result.total_duration_ms = overall_start.elapsed().as_millis() as u64;

        info!(
            url = %url,
            total_duration_ms = result.total_duration_ms,
            fetch_ms = result.phase_timings.fetch_ms,
            gate_ms = result.phase_timings.gate_ms,
            wasm_ms = result.phase_timings.wasm_ms,
            render_ms = ?result.phase_timings.render_ms,
            gate_decision = %result.gate_decision,
            quality_score = result.quality_score,
            "Enhanced pipeline execution completed"
        );

        Ok(result)
    }

    /// Execute fetch phase with timing
    async fn execute_fetch_phase(&self, url: &str) -> PhaseResult<(String, u16)> {
        let timer = PhaseTimer::start(PhaseType::Fetch, url.to_string());
        let start = Instant::now();

        let result = self.fetch_content(url).await;
        let duration_ms = start.elapsed().as_millis() as u64;

        // Record timing in metrics
        timer.end(&self.metrics, result.is_ok());

        PhaseResult {
            result,
            duration_ms,
        }
    }

    /// Execute gate analysis phase with timing
    async fn execute_gate_phase(&self, url: &str, content: &str) -> GateResult {
        let timer = PhaseTimer::start(PhaseType::Gate, url.to_string());
        let start = Instant::now();

        let (decision, quality_score) = self.analyze_content_gate(url, content).await;
        let duration_ms = start.elapsed().as_millis() as u64;

        // Record timing in metrics
        timer.end(&self.metrics, true); // Gate analysis always succeeds

        debug!(
            url = %url,
            gate_decision = %decision,
            quality_score = quality_score,
            duration_ms = duration_ms,
            "Gate analysis completed"
        );

        GateResult {
            decision,
            quality_score,
            duration_ms,
        }
    }

    /// Execute WASM extraction phase with timing
    async fn execute_wasm_phase(
        &self,
        url: &str,
        content: &str,
        gate_decision: &str,
    ) -> PhaseResult<ExtractedDoc> {
        let timer = PhaseTimer::start(PhaseType::Wasm, url.to_string());
        let start = Instant::now();

        let result = self.extract_with_wasm(url, content, gate_decision).await;
        let duration_ms = start.elapsed().as_millis() as u64;

        // Record timing in metrics
        timer.end(&self.metrics, result.is_ok());

        PhaseResult {
            result,
            duration_ms,
        }
    }

    /// Execute render phase with timing (for headless content)
    async fn execute_render_phase(&self, url: &str) -> PhaseResult<ExtractedDoc> {
        let timer = PhaseTimer::start(PhaseType::Render, url.to_string());
        let start = Instant::now();

        let result = self.render_with_headless(url).await;
        let duration_ms = start.elapsed().as_millis() as u64;

        // Record timing in metrics
        timer.end(&self.metrics, result.is_ok());

        PhaseResult {
            result,
            duration_ms,
        }
    }

    /// Fetch content from URL
    async fn fetch_content(&self, url: &str) -> Result<(String, u16)> {
        // Simplified implementation - integrate with actual fetch logic
        let response = self.state.http_client.get(url).send().await?;
        let status = response.status().as_u16();
        let content = response.text().await?;
        Ok((content, status))
    }

    /// Analyze content for gate decision
    async fn analyze_content_gate(&self, _url: &str, content: &str) -> (String, f32) {
        // Simplified gate analysis - integrate with actual gate logic
        let content_length = content.len();
        let quality_score = if content_length > 10000 {
            0.8
        } else if content_length > 1000 {
            0.5
        } else {
            0.2
        };

        let decision = if quality_score >= self.state.config.gate_hi_threshold {
            "raw"
        } else if quality_score >= self.state.config.gate_lo_threshold {
            "probes_first"
        } else {
            "headless"
        };

        (decision.to_string(), quality_score)
    }

    /// Extract content using WASM
    async fn extract_with_wasm(
        &self,
        url: &str,
        content: &str,
        _gate_decision: &str,
    ) -> Result<ExtractedDoc> {
        // Simplified implementation - integrate with actual WASM extraction
        let doc = ExtractedDoc {
            url: url.to_string(),
            title: Some("Extracted Title".to_string()),
            byline: None,
            published_iso: None,
            markdown: Some(content.chars().take(1000).collect::<String>()),
            text: content.chars().take(1000).collect::<String>(),
            links: vec![],
            media: vec![],
            language: Some("en".to_string()),
            reading_time: Some(5),
            quality_score: None,
            word_count: Some(content.split_whitespace().count() as u32),
            categories: vec![],
            site_name: None,
            description: None,
        };

        Ok(doc)
    }

    /// Render content using headless browser
    async fn render_with_headless(&self, url: &str) -> Result<ExtractedDoc> {
        // Simplified implementation - integrate with actual headless rendering
        if self.state.config.headless_url.is_none() {
            return Err(anyhow::anyhow!("Headless service not configured"));
        }

        // For now, return a placeholder
        let doc = ExtractedDoc {
            url: url.to_string(),
            title: Some("Headless Rendered Title".to_string()),
            byline: None,
            published_iso: None,
            markdown: Some("Enhanced content from headless rendering".to_string()),
            text: "Enhanced content from headless rendering".to_string(),
            links: vec![],
            media: vec![],
            language: Some("en".to_string()),
            reading_time: Some(2),
            quality_score: None,
            word_count: Some(100),
            categories: vec![],
            site_name: None,
            description: None,
        };

        Ok(doc)
    }
}

/// Result from enhanced pipeline execution
#[derive(Debug, Clone)]
pub struct EnhancedPipelineResult {
    pub url: String,
    pub success: bool,
    pub total_duration_ms: u64,
    pub phase_timings: PhaseTiming,
    pub document: Option<ExtractedDoc>,
    pub error: Option<String>,
    pub cache_hit: bool,
    pub gate_decision: String,
    pub quality_score: f32,
}

/// Detailed phase timing information
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PhaseTiming {
    pub fetch_ms: u64,
    pub gate_ms: u64,
    pub wasm_ms: u64,
    pub render_ms: Option<u64>,
}

/// Enhanced batch processing statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnhancedBatchStats {
    pub total_urls: usize,
    pub successful: usize,
    pub failed: usize,
    pub cache_hits: usize,
    pub total_duration_ms: u64,
    pub avg_processing_time_ms: f64,
    pub avg_phase_timings: PhaseTiming,
    pub gate_decisions: crate::pipeline::GateDecisionStats,
}

/// Generic phase result with timing
#[derive(Debug)]
struct PhaseResult<T> {
    result: Result<T>,
    duration_ms: u64,
}

/// Gate analysis result
#[derive(Debug)]
struct GateResult {
    decision: String,
    quality_score: f32,
    duration_ms: u64,
}

// Implement Clone for EnhancedPipelineOrchestrator
impl Clone for EnhancedPipelineOrchestrator {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            options: self.options.clone(),
            metrics: self.metrics.clone(),
            config: self.config.clone(),
            pipeline: self.pipeline.clone(),
        }
    }
}
